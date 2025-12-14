# Continuous Session/Context Mode for `pw-cli`

## Objectives

- Remove repetitive boilerplate between consecutive CLI invocations (URLs, selectors, auth paths, browser choice).
- Keep a living Playwright context/page alive across commands for speed and continuity.
- Provide explicit, scriptable controls to set/inspect/reset context, not implicit magic only.
- Support both project-scoped and global state; respect existing project detection.
- Fail gracefully: auto-fallback to fresh sessions when cached state is invalid or unsafe.

## Current State (gaps)

- Each command re-parses flags, rebuilds `CommandContext`, and spins a fresh `BrowserSession` (new Playwright process + new page).
- URLs are mandatory positional args for most commands; selectors/outputs must be retyped.
- No persisted state besides auth files; no notion of an "active" page/context.

## Target UX and CLI Surface

- **Contexts as first-class objects**: `pw ctx new foo`, `pw ctx use foo`, `pw ctx show [foo]`, `pw ctx list`, `pw ctx rm foo`, `pw ctx set key=value`, `pw ctx unset key`.
- **Inline overrides**: `pw --context foo screenshot --selector .hero` uses stored URL/base, while `pw --no-context ...` bypasses cache entirely.
- **Automatic last-used context**: commands without `--context` use the project-default active context (per project), falling back to a global "default".
- **URL/selector defaults**:
  - Commands accept optional `url` when context provides `last_url` or `base_url + last_path`.
  - Selectors become optional when `last_selector` is present; commands still allow explicit selector to override.
- **Session lifecycle commands**: `pw session start [--context foo] [--headful]`, `pw session stop [--context foo]`, `pw session status`, `pw session restart`.
- **Safety switches**: `--no-save-context` (do not persist run results), `--refresh-context` (discard cached session and start clean), `--context-only` (use context defaults but do not launch/reuse a persistent browser).

## Data Model and Storage

- **Schema** (versioned):
  ```json
  {
    "schema": 1,
    "active": {"global": "default", "projects": {"/abs/project/path": "proj-default"}},
    "contexts": {
      "default": {
        "scope": "global|project",
        "project_root": "/abs/path" | null,
        "base_url": "https://app.example.com" | null,
        "last_url": "https://app.example.com/dashboard" | null,
        "last_path": "/dashboard" | null,
        "last_selector": "#login" | null,
        "last_output": "playwright/screenshots/latest.png" | null,
        "browser": "chromium|firefox|webkit",
        "headless": true,
        "wait_until": "load|domcontentloaded|networkidle",
        "viewport": {"width": 1280, "height": 720} | null,
        "auth_file": "/abs/auth.json" | null,
        "cdp_endpoint": "ws://..." | null,
        "storage_state_fingerprint": "sha256-of-file",
        "session": {
          "mode": "attached|launched",
          "cdp_endpoint": "ws://...",
          "context_id": "...",          // if exposed by relay or sessiond
          "page_id": "...",
          "pid": 12345,
          "started_at": "2025-01-01T12:00:00Z"
        },
        "last_used_at": "2025-01-01T12:34:56Z"
      }
    }
  }
  ```
- **Locations**:
  - Global store: `${XDG_CONFIG_HOME:-~/.config}/pw/cli/contexts.json`.
  - Project store: `<project-root>/playwright/.pw-cli/contexts.json` (gitignored alongside other generated outputs).
- **Scoping rules**: project-scoped contexts may shadow global ones; names are reused per scope but stored separately.

## Resolution Order (per invocation)

1. CLI flags/env vars (highest precedence).
1. Explicit `--context <name>` selection (project first, else global).
1. Active project context (from `active.projects[project_root]`).
1. Active global context (from `active.global`).
1. Built-in defaults (Chromium, headless true, required positional args if still missing).

## Command Argument Inference

- **URL**: if `url` arg omitted, use `context.last_url`; if only a path is provided (e.g., `/invoices`), join with `context.base_url`. Store `last_url` after successful navigation-based commands.
- **Selector**: if omitted, use `context.last_selector` when available. Update it after commands that target selectors (click/text/html/coords/wait).
- **Outputs**: default `output` resolves to `context.last_output` or project screenshot dir; update `last_output` after screenshot/auth login output, etc.
- **Auth/browser/cdp**: resolved from context unless overridden by flags; mismatches can trigger session restart (see invalidation).

## Session/Caching Architecture

- **SessionBroker** (new component in `pw-cli`):
  - Loads resolved context; decides whether to attach to an existing session or start a new `BrowserSession`.
  - Uses a lightweight control file (per context) containing PID, CDP endpoint, and Playwright version hash.
  - Performs health checks (PID alive + websocket ping) before reuse; otherwise purges and relaunches.
- **Daemonized continuous mode**:
  - `pw session start` launches a background process (tokio runtime) that holds Playwright browser + context + page open, exposes a local websocket/pipe for subsequent commands.
  - Commands connect via stored CDP endpoint; they reuse the same page/context unless flags request isolation (`--new-page` or different browser/auth).
  - Idle timeout configurable per context (e.g., default 15 minutes); auto-shutdown when idle or on `pw session stop`.
- **Non-daemon fast path**:
  - Even without an explicit daemon, when commands are run back-to-back in the same shell, cache resolved context to disk to auto-fill args and to decide whether to reuse a short-lived Playwright process started by a previous command (if still alive).

## Invalidation and Safety

- Restart session when any of: browser kind changes, headless/headful flips, auth file mtime/hash changes, base URL changes, Playwright driver version changes, CDP endpoint differs, or when last command failed with navigation/session errors.
- If attach fails, log reason at debug level, start fresh session, and update context session info.
- Allow `pw ctx clear-cache [name]` to delete session descriptors while preserving logical defaults.
- Never store secrets; only file paths and hashes. Respect restrictive file permissions on context store (0600).

## Flow per Command

1. Parse CLI + load stores + resolve context (precedence above) into a `ResolvedContext` struct.
1. SessionBroker decides reuse vs launch; obtains a `BrowserSessionHandle` exposing page/context.
1. Command executes using inferred args (URL/selector/output) and resolved auth/browser/CDP.
1. On success: persist `last_url`, `last_selector`, `last_output`, `last_used_at`; optionally persist session descriptor. On failure: persist `last_used_at` and error code, but do not overwrite successful last\_\* fields unless requested.
1. Honor `--no-save-context` to skip step 4; honor `--refresh-context` to drop cached session before step 2.

## CLI Additions (draft)

- Global flags: `--context <name>`, `--no-context`, `--no-save-context`, `--refresh-context`, `--new-page`, `--base-url <url>`.
- New subcommand group `ctx` for lifecycle and inspection (new file `commands/context.rs`).
- New subcommand group `session` for daemon control.
- Positional `url` for commands becomes optional when context can satisfy it; clap validation must be relaxed accordingly with clear errors when absent.

## Telemetry and Logging

- Add debug logging for resolution decisions (which context, why session reused/restarted, inferred url/selector).
- Emit user-facing hints when inference filled a missing arg ("using last_url=https://... from context 'proj-default'").

## Edge Cases

- Running outside a Playwright project: only global contexts apply; still allow contexts to store absolute paths.
- Multiple projects in same repo: contexts keyed by project root; active context tracked per root.
- Concurrent shells: use atomic writes to context store (write temp + rename) to avoid corruption.
- Relay/extension users: allow contexts to carry `cdp_endpoint` that points at relay; do not auto-start Playwright in that case.

## Implementation Steps (phased)

1. **Context store + resolution**: add schema/types, load/save helpers, precedence logic, clap changes to make URL optional when context exists, and context subcommands.
1. **SessionBroker (non-daemon reuse)**: track last launched session per context with PID + endpoint; attempt reuse within grace period.
1. **Daemonized mode**: `pw session start/stop/status`, background process management, heartbeat checks, and idle timeout handling.
1. **Polish**: better hints, error messages, `ctx export/import`, and docs/examples.
