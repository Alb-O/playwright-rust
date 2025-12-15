# Reconnectable Local Sessions for `pw-cli`

## Goal
Enable `pw-cli` to launch browsers locally and reconnect across commands via a stable WebSocket (CDP) endpoint, so the SessionBroker can reuse a live browser/page without requiring remote relays.

## Why this matters (UX wins)
- Zero-boilerplate consecutive commands: no re-launch cost; page state persists.
- Consistent behavior between remote (relay) and local runs: both expose a CDP endpoint and reuse flows are identical from the broker’s perspective.
- Clear controls: users can `pw session start` locally, run many commands fast, then `pw session stop`.

## Requirements
- Produce a reconnectable endpoint for locally launched browsers (WS/CDP) and keep the Playwright driver alive between commands.
- Add a WebSocket transport to `pw-core` connection layer alongside the existing pipe transport.
- Expose `launch_server` (Playwright driver) and return `wsEndpoint` to clients; provide `connect_over_cdp` via WS transport too.
- Keep API parity: existing pipe launch stays the default; WS launch is opt-in from pw-cli SessionBroker.
- Safe teardown: allow `stop` to kill the launched server and clean descriptors.
- Descriptor fidelity: store endpoint, pid, browser kind, headless flag, and driver version hash for invalidation.

## Architecture Overview
- **pw-core**
  - Add WebSocket transport (tokio-tungstenite) implementing `Transport` trait and message pump like PipeTransport.
  - Extend `Connection` to be transport-agnostic (trait object for Transport + message receiver) and add a constructor for WS.
  - Add `Playwright::connect_ws(ws_url)` to initialize protocol over WS (used for `connect_over_cdp` reconnects).
  - Add `BrowserType::launch_server_with_options(options)` returning `{ ws_endpoint, browser }`, mirroring Playwright’s `launchServer`.
  - Surface `ws_endpoint` on a `LaunchedServer` handle that can also `close()` the server.

- **pw-cli**
  - `BrowserSession` gains a launch-server path: start driver via `launch_server_with_options`, capture `ws_endpoint`, and connect via WS transport.
  - SessionBroker writes descriptors only when an endpoint exists; for local launches, it now will because launch-server provides `ws_endpoint`.
  - Add broker mode selection: pipe launch (default, non-reusable) vs server launch (reusable) driven by a flag/context setting (`--reuse-local` or context key `launch_server=true`).
  - `session start/stop`: start uses launch-server and persists descriptor; stop reads descriptor and closes server via WS.

## Flow (happy path)
1) `pw session start --headful`:
   - Broker chooses launch-server mode, calls BrowserSession launch-server, gets `ws_endpoint`, saves descriptor.
   - Leaves server running; no page navigation yet.
2) `pw screenshot --context foo`:
   - Broker sees descriptor, reuses via `connect_over_cdp(ws_endpoint)`, creates context/page, runs command, persists last_url/output.
3) `pw session stop`:
   - Broker loads descriptor, connects over WS, closes browser/server, removes descriptor.

## Invalidation Rules
- Invalidate descriptor when browser kind differs, headless flips, driver version hash changes, or `--refresh-context` is set.
- If WS connect fails or health check fails, delete descriptor and relaunch via launch-server.

## Surfacing Controls
- Global flag: `--launch-server` (or context key `launch_server=true`) to opt into reusable local sessions; default remains pipe for minimal change risk initially.
- `pw session status` shows whether descriptor is reusable, endpoint, and alive check.
- `pw session start/stop` manage lifecycle explicitly; regular commands may auto-start if no descriptor and `--launch-server` is set.

## Plan of Work
1) **Transport + Connection (pw-core)**
   - Add WebSocket transport (tokio-tungstenite), with send/recv pump matching PipeTransport framing (raw JSON, no length prefix).
   - Generalize `Connection` to accept any transport implementing a small trait; add constructor for WS URLs.
2) **Launch Server API (pw-core)**
   - Implement `BrowserType::launch_server_with_options` RPC wrapper; model response `{ wsEndpoint, browser }`.
   - Add `LaunchedServer` handle with `ws_endpoint()` and `close()`.
3) **BrowserSession updates (pw-cli)**
   - Add launch-server path: call `launch_server_with_options`, store endpoint, connect via WS transport, and avoid closing server in `close()` when launched-server mode (provide `shutdown_server()` method).
   - Return endpoint to SessionBroker for descriptor persistence.
4) **SessionBroker changes (pw-cli)**
   - Add flag/context setting to prefer launch-server.
   - On launch, store descriptor with endpoint; on reuse, connect via WS and rebuild context/page.
   - `session stop` should connect and close server, then delete descriptor.
5) **CLI UX and Docs**
   - Document `--launch-server`, session start/stop behavior, and reuse rules in README/docs.
6) **Tests**
   - pw-core: unit test WS transport framing; integration test launch-server + connect to `wsEndpoint` and run a simple page op.
   - pw-cli: integration test that `session start` → `screenshot` reuses via descriptor; `session stop` tears down.

## UX Satisfaction Checks
- Defaults unchanged: users not opting in keep today’s behavior.
- Opt-in reuse: `pw --launch-server screenshot https://example.com` creates a reusable session; next command reuses without relaunch.
- Clear feedback: status shows endpoint/pid/alive; errors log why reuse fell back to relaunch.
- Safety: `--refresh-context` wipes descriptors; `session stop` cleans up.
