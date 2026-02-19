# `pw` Engine Architecture

## Scope

This document explains how `pw` executes commands and where control moves from Rust to Playwright JavaScript and then to browser-page JavaScript.

It focuses on the protocol-first path used by both:

* `pw-cli` commands (`crates/cli`)
* `pw-rs` library API (`crates/core`)

## High-Level Stack

```text
CLI / Rust API (crates/cli, crates/core)
        |
        | typed Rust calls (Page::goto, BrowserType::launch, etc.)
        v
Runtime + Protocol client (crates/runtime)
        |
        | JSON-RPC over stdio (length-prefixed frames)
        v
Playwright Node driver (playwright package JS)
        |
        | browser automation protocol + dispatchers
        v
Browser engine process (Chromium/Firefox/WebKit)
        |
        | optional evaluate/script injection
        v
Page JavaScript context (DOM / JS runtime in tab)
```

## The Rust/JavaScript Boundary

There are two concrete boundaries:

* Process boundary:
  * Rust spawns Node with Playwright CLI `run-driver`
  * `crates/runtime/src/playwright_server.rs:40`
  * `crates/runtime/src/playwright_server.rs:42`
  * `crates/runtime/src/playwright_server.rs:60`
* Transport boundary:
  * Rust serializes protocol requests and writes framed JSON to child stdin
  * `crates/runtime/src/transport/mod.rs:360`
  * `crates/runtime/src/transport/mod.rs:365`
  * `crates/runtime/src/transport/mod.rs:371`

From that point onward, method execution logic is in Playwright JS dispatchers until a result/event is sent back.

## Command Flow (CLI Example: `pw exec page.eval`)

### 1. CLI request dispatch

* `main` parses CLI and dispatches command:
  * `crates/cli/src/main.rs:8`
  * `crates/cli/src/main.rs:10`
* `exec`/`batch` request handling:
  * `crates/cli/src/commands/engine.rs:17`
  * `crates/cli/src/commands/engine.rs:165`

### 2. Session orchestration

* Command resolves runtime/profile and acquires session via `SessionManager`:
  * `crates/cli/src/commands/engine.rs:202`
  * `crates/cli/src/session/manager.rs:111`
* Page-oriented commands use shared flow:
  * `crates/cli/src/commands/flow/page.rs:27`
  * `crates/cli/src/session_helpers.rs:16`

### 3. Browser startup (Rust side)

* Session builder launches Playwright:
  * `crates/cli/src/browser/session/builder.rs:36`
* Core launch path:
  * `crates/core/src/playwright.rs:94`
  * `crates/core/src/playwright.rs:100`
  * `crates/core/src/playwright.rs:117`
  * `crates/core/src/playwright.rs:119`

### 4. Protocol bootstrap

* Rust creates a temporary `Root` channel owner and sends `initialize`:
  * `crates/core/src/init.rs:38`
  * `crates/core/src/root.rs:107`
* Connection loop handles `__create__`, `__dispose__`, `__adopt__`:
  * `crates/runtime/src/connection/mod.rs:436`
  * `crates/runtime/src/connection/mod.rs:461`
  * `crates/runtime/src/connection/mod.rs:508`
  * `crates/runtime/src/connection/mod.rs:525`

### 5. API call becomes RPC

For `page.eval`:

* CLI command calls `evaluate_value`:
  * `crates/cli/src/commands/page/eval/mod.rs:98`
* Page delegates to frame eval:
  * `crates/core/src/page/eval.rs:18`
  * `crates/core/src/frame.rs:920`
* Frame sends protocol method `evaluateExpression`:
  * `crates/core/src/frame.rs:934`
* Channel and connection package request:
  * `crates/runtime/src/channel.rs:32`
  * `crates/runtime/src/connection/mod.rs:318`
  * `crates/runtime/src/connection/mod.rs:328`
  * `crates/runtime/src/connection/mod.rs:336`

### 6. Node driver receives and dispatches

* Playwright CLI binds `run-driver`:
  * `playwright/drivers/playwright-1.57.0-linux/package/lib/cli/program.js:260`
* Driver sets up dispatcher connection and pipe transport:
  * `playwright/drivers/playwright-1.57.0-linux/package/lib/cli/driver.js:46`
  * `playwright/drivers/playwright-1.57.0-linux/package/lib/cli/driver.js:52`
  * `playwright/drivers/playwright-1.57.0-linux/package/lib/cli/driver.js:53`
* Dispatcher routes message by `guid` + `method`:
  * `playwright/drivers/playwright-1.57.0-linux/package/lib/server/dispatchers/dispatcher.js:256`

### 7. Playwright JS executes method

For frame eval:

* Frame dispatcher method:
  * `playwright/drivers/playwright-1.57.0-linux/package/lib/server/dispatchers/frameDispatcher.js:75`
* Frame implementation:
  * `playwright/drivers/playwright-1.57.0-linux/package/lib/server/frames.js:596`
* JS evaluation core:
  * `playwright/drivers/playwright-1.57.0-linux/package/lib/server/javascript.js:186`

### 8. Result serialization and return to Rust

* Playwright serializes values into protocol shape (`s`, `n`, `b`, `v`, etc.):
  * `playwright/drivers/playwright-1.57.0-linux/package/lib/protocol/serializers.js:90`
  * `playwright/drivers/playwright-1.57.0-linux/package/lib/protocol/serializers.js:113`
  * `playwright/drivers/playwright-1.57.0-linux/package/lib/protocol/serializers.js:117`
* Rust receives response/event and resolves pending callback by request id:
  * `crates/runtime/src/connection/mod.rs:420`
  * `crates/runtime/src/connection/mod.rs:422`
  * `crates/runtime/src/connection/mod.rs:433`

## Where Rust Logic Ends vs JS Logic Begins

In practical terms:

* Rust owns:
  * CLI protocol envelopes and command graph (`crates/cli`)
  * session policy, descriptor reuse, daemon/attach strategy (`crates/cli/src/session`)
  * typed protocol client objects (`crates/core`)
  * request/response correlation + transport framing (`crates/runtime`)
* Playwright Node JS owns:
  * protocol dispatch (`dispatcher.js`)
  * browser launch/connect internals (`browserType.js`, dispatcher classes)
  * selectors, waits, actionability, retries, execution contexts
* Browser-page JS owns:
  * code passed via `evaluateExpression`
  * scripts injected by caller or by Playwright internals

So the per-call seam is exactly when Rust `Channel::send*` writes JSON-RPC to the transport.

## Transport Details

`pw-runtime` mirrors Playwright bindings transport behavior:

* stdio mode uses 4-byte little-endian length prefix framing:
  * `crates/runtime/src/transport/mod.rs:19`
  * `crates/runtime/src/transport/mod.rs:33`
  * `crates/runtime/src/transport/mod.rs:155`
* matching JS transport implementation:
  * `playwright/drivers/playwright-1.57.0-linux/package/lib/server/utils/pipeTransport.js:43`
  * `playwright/drivers/playwright-1.57.0-linux/package/lib/server/utils/pipeTransport.js:49`
  * `playwright/drivers/playwright-1.57.0-linux/package/lib/server/utils/pipeTransport.js:65`

WebSocket mode is also supported for daemon/remote paths:

* Rust websocket transport:
  * `crates/runtime/src/transport/mod.rs:416`
  * `crates/runtime/src/transport/mod.rs:447`

## Object Lifecycle and Registry

Rust tracks remote objects by GUID and builds typed wrappers:

* object store:
  * `crates/runtime/src/connection/object_store.rs:17`
* create/dispose/adopt handling:
  * `crates/runtime/src/connection/mod.rs:437`
  * `crates/runtime/src/connection/mod.rs:462`
* Rust-side type factory mapping protocol type names:
  * `crates/core/src/object_factory.rs:61`
  * `crates/core/src/object_factory.rs:63`

This means Rust does not recreate Playwright behavior; it reflects remote object state and forwards methods.

## Special Case: `pw connect` and CDP

`pw connect --launch/--discover` has an extra control path:

* CLI first discovers or launches Chrome via CDP endpoint helpers:
  * `crates/cli/src/commands/connect/mod.rs:105`
  * `crates/cli/src/session/connect/mod.rs:185`
  * `crates/cli/src/session/connect/cdp_probe.rs:20`
* Then `pw-rs` still connects through Playwright driver (`connectOverCDP`), not direct raw CDP for normal command execution:
  * `crates/core/src/browser_type.rs:235`
  * `crates/core/src/browser_type.rs:259`

So even in connect mode, Rust command execution usually still traverses the same Rust -> Playwright JS boundary.

## What Runs in Browser JavaScript Context

Two categories run in page context:

* explicit user/command evaluation:
  * `crates/core/src/frame.rs:934`
  * `crates/cli/src/browser/js.rs:5`
* internal Playwright scripts used for selectors, waits, DOM/action helpers:
  * `playwright/drivers/playwright-1.57.0-linux/package/lib/server/frames.js:629`
  * `playwright/drivers/playwright-1.57.0-linux/package/lib/server/frames.js:673`

This is separate from Node driver JS runtime.

## Build-Time Driver Materialization

The repo vendors/downloads a specific Playwright version at build time:

* driver version constant:
  * `crates/runtime/build.rs:11`
* runtime locates node + cli.js from env/bundled/npm:
  * `crates/runtime/src/driver.rs:31`
  * `crates/runtime/src/driver.rs:46`
  * `crates/runtime/src/driver.rs:53`

This explains why Rust runtime can stay stable while Playwright JS internals are versioned and patched.

## Mental Model for Contributors

Use this rule when debugging:

* If issue appears before `Channel::send`/transport write, it is Rust orchestration/type logic.
* If request is sent but method behavior is unexpected, inspect Playwright JS dispatcher/server internals.
* If `evaluate`/selector DOM behavior is unexpected, inspect browser-page JS execution assumptions.

That rule usually identifies the right layer in one pass.
