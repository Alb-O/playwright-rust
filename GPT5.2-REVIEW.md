# GPT-5.2 Architecture Review of pw-rs

Generated from conversation with GPT-5.2 Thinking model after reviewing the full codemap (8500+ lines).

---

## Executive Summary

The codebase mirrors the official Playwright bindings mental model well, with a connection-based object registry and channel owner pattern. However, there are several architectural issues that could cause reliability problems at scale, particularly around async/sync boundaries and GUID resolution.

**Priority fixes:**
1. GUID reference replacement (remove retry/sleep loops)
2. Send pipeline refactor (no mutex across .await)
3. Error type restructuring
4. Evaluate API to return typed values

---

## 1. Architecture Review

### Strengths

- Internal "server layer" mirrors the official Playwright bindings' mental model: a connection that owns an object registry keyed by GUID, plus "channel owner" objects that form a parent/child tree and communicate via RPC "channels"
- Forward compatibility is handled reasonably: `Message` supports `Unknown(Value)` so new upstream message shapes won't instantly break parsing
- The connection loop is structured correctly: a transport read task plus a message dispatch loop that correlates responses by id (oneshot) and routes events by guid

### Weaknesses

#### A. Sync API surface + async cleanup via tokio::spawn

`ChannelOwnerImpl::dispose()` is synchronous, but it performs async cleanup by spawning a task to unregister the object from the connection registry.

**Failure modes:**
- If `dispose()` runs outside a Tokio runtime, `tokio::spawn` will panic
- Cleanup ordering becomes nondeterministic (registry removal races with other operations)
- Errors from `unregister_object` are silently dropped

#### B. Object lookup errors depend on GUID prefix guessing

`get_object` maps "missing guid" into `TargetClosed` by guessing type from guid prefixes like `page@`, `frame@`, etc., otherwise falling back to `ProtocolError("Object not found")`.

**Problems:**
- Protocol could change prefixes
- Conflates "closed/collected" with "never existed / race / bug"

#### C. Downcasting is pervasive

Even internal invariants are guarded via downcast + `ProtocolError` text. This makes it hard for users to programmatically handle failure causes.

#### D. GUID replacement race workaround

The codemap contains TODOs and retry loops around resolving response GUIDs into channel objects. This "sleep/retry 20 times" approach will be flaky under load and adds nondeterministic latency.

---

## 2. API Ergonomics

### Issues

#### A. `evaluate` returns String

`Page::evaluate_value` returns `Result<String>` rather than a typed value or `serde_json::Value`, and pw-cli trims quotes when using it.

**Better options:**
```rust
async fn evaluate_json(&self, expr: &str) -> Result<serde_json::Value>
async fn evaluate<T: DeserializeOwned>(&self, expr: &str) -> Result<T>
```

#### B. Event/listener APIs should return cancellable handles

Routing uses a shared vector of handlers under a mutex, and errors are printed via `eprintln!` inside the library.

**Better pattern:**
- Return a `Subscription`/`ListenerHandle` that unregisters on Drop
- Use `tracing` for logging (not `eprintln!`)
- Expose events as `Stream`/`broadcast::Receiver` when feasible

#### C. Avoid std::sync::Mutex on hot async paths

The `Page` struct holds handler lists and page state behind `std::sync::Mutex`/`RwLock`. If events are dispatched on Tokio worker threads, a contended `std::sync::Mutex` can block the executor thread.

**Use:** `parking_lot::Mutex` for short synchronous critical sections, or `tokio::sync::Mutex`/`RwLock` if lock acquisition might be contended by async work.

#### D. Seal internal traits

`ChannelOwner`/`ConnectionLike` are core invariants. Consider a sealed-trait pattern to prevent downstream users from implementing them.

---

## 3. Error Handling

### Current State

Protocol errors are parsed into:
- `Timeout`
- `TargetClosed`
- Otherwise `ProtocolError(message)` (dropping name + stack)

Some non-protocol failures are flattened into `ProtocolError(String)` (e.g., screenshot base64 decode errors).

### Recommended Structure

```rust
pub enum Error {
    /// Remote Playwright error with full context
    Remote {
        name: String,
        message: String,
        stack: Option<String>,
        method: String,
        guid: String,
    },
    
    /// Operation timed out
    Timeout {
        message: String,
        method: String,
        guid: String,
    },
    
    /// Target was closed/navigated away
    TargetClosed {
        target_type: &'static str,
        context: String,
    },
    
    /// Object not found in registry
    ObjectNotFound {
        guid: String,
        expected: Option<&'static str>,
    },
    
    /// Transport I/O error
    Transport(#[source] std::io::Error),
    
    /// JSON serialization error
    Serde(#[source] serde_json::Error),
    
    /// Base64 decode error
    Decode(#[source] base64::DecodeError),
}
```

**Key improvement:** Don't throw away remote stack and name.

---

## 4. Concurrency Model

### Issues

#### A. Outbound send lock held across .await

`send_message` takes a `transport_sender` mutex and awaits the send while holding it. This serializes all writers and can become a throughput bottleneck or deadlock risk.

**Better pattern:**
- Have an `mpsc::Sender<Request>` cloneable without locks
- One dedicated task owns the transport writer and processes the queue
- `send_message` enqueues + awaits response

#### B. Cancellation leakage

If a caller drops the future returned by `send_message` after inserting the callback, there's no obvious "remove callback on drop" guard. The response will eventually remove it if it arrives, but if the connection dies, you can leak entries.

**Solution:** RAII guard captured by the response future that removes its callback entry on Drop.

#### C. Library-level tokio::spawn in object constructors/lifecycle

`BrowserContext` enables dialog subscription by spawning an untracked task at construction time. This makes initialization nondeterministic and can fail silently.

**Recommendation:** Prefer explicit async init steps, or at least record the `JoinHandle` / propagate error.

#### D. Event dispatch backpressure

The connection loop calls `dispatch_internal` per message. If `on_event` handlers do meaningful work, they can block the loop.

**Better pattern:**
- Minimal parsing + enqueue onto per-object channels
- Handlers run in independent tasks with bounded queues (backpressure)

---

## 5. CLI Improvements for AI Agent Workflows

### Current Strengths

- `ContextState` persists cross-invocation "working memory"
- CDP "current page" sentinel (`__CURRENT_PAGE__`) is useful for attaching to existing browsers
- Daemon mode provides reuse key for persistent browsers

### Recommended Improvements

#### A. Multi-tenant daemon

Current daemon uses fixed `/tmp/pw-daemon.sock`. For agents:
- Per-user socket location (XDG runtime dir, permissioned)
- Optional auth token / nonce handshake
- Commands to list/inspect/evict sessions (by reuse_key)
- TTL/LRU eviction

#### B. Batch protocol

Agent loops pay too much overhead by invoking CLI repeatedly. Add:
```bash
pw run --format ndjson  # reads commands from stdin, streams responses
```
Extend into stable machine protocol (request id, status, artifact refs).

#### C. Structured state outputs

Agents need:
- "Active page" identity (url, title, opener, context id)
- Tab list and selection semantics
- Deterministic selectors (Playwright selector engine output) for follow-up actions

#### D. First-class artifact + trace pipeline

- `trace start/stop/export`
- Automatic trace/screenshot on failure (with stable paths)
- Machine-readable "artifact manifest" in every response

#### E. Tool contracts

Export JSON Schemas for command outputs. With current Toon/NDJSON formats, you're close; formal schemas let agents validate and recover.

---

## Action Items / TODOs

### Priority 1: Critical Architecture Fixes

- [ ] **GUID Resolution**: Implement proper guid ref resolution pass at connection boundary
  - Walk JSON response structure
  - Replace `{"guid": "..."}` references with typed objects
  - Block response future until referenced GUIDs are registered
  - Files: `crates/pw-core/src/server/connection.rs`

- [ ] **Send Pipeline**: Refactor to avoid mutex across .await
  - Add `mpsc::Sender<Request>` for cloneable sends
  - Dedicated writer task owns transport
  - `send_message` enqueues + awaits response via oneshot
  - Files: `crates/pw-core/src/server/connection.rs`, `transport.rs`

- [ ] **Sync Disposal**: Remove tokio::spawn from dispose()
  - Make `ConnectionLike::unregister_object` synchronous
  - Keep server RPC effects explicit and awaited in async fns
  - Files: `crates/pw-core/src/server/channel_owner.rs`

### Priority 2: Error Handling

- [ ] **Restructure Error enum**: Add structured variants with full context
  - `Error::Remote { name, message, stack, method, guid }`
  - `Error::ObjectNotFound { guid, expected }`
  - Preserve `#[source]` for wrapped errors
  - Files: `crates/pw-core/src/error.rs`

- [ ] **Object Registry**: Store type info, add distinct `ObjectNotFound` error
  - Don't guess type from GUID prefix
  - Files: `crates/pw-core/src/server/connection.rs`

### Priority 3: API Ergonomics

- [ ] **Typed evaluate**: Add `evaluate_json()` and generic `evaluate<T>()`
  - Keep `evaluate_value_string()` as convenience
  - Files: `crates/pw-core/src/protocol/page.rs`, `frame.rs`

- [ ] **Cancellable event handles**: Return `Subscription` that unregisters on Drop
  - Replace `Vec<Box<dyn Fn>>` with proper subscription model
  - Files: `crates/pw-core/src/protocol/page.rs`, `browser_context.rs`

- [ ] **Replace eprintln! with tracing**: Library should not print to stderr
  - Files: grep for `eprintln!` in `crates/pw-core/`

- [ ] **Seal internal traits**: Prevent downstream ChannelOwner implementations
  - Add private marker trait
  - Files: `crates/pw-core/src/server/channel_owner.rs`

### Priority 4: Concurrency

- [ ] **Cancellation guard**: RAII guard to remove callback on future drop
  - Files: `crates/pw-core/src/server/connection.rs`

- [ ] **Event backpressure**: Enqueue events to per-object channels
  - Handlers run in independent tasks
  - Files: `crates/pw-core/src/server/connection.rs`

- [ ] **Use parking_lot consistently**: Replace std::sync::Mutex on async paths
  - Or use tokio::sync for contended locks
  - Files: `crates/pw-core/src/protocol/page.rs`

### Priority 5: CLI Agent Features

- [ ] **Daemon security**: Per-user socket, auth tokens, session TTL
  - Files: `crates/pw-cli/src/daemon/`

- [ ] **Batch protocol**: `pw run --format ndjson` with stdin commands
  - Request ID, status, artifact refs
  - Files: `crates/pw-cli/src/commands/mod.rs`

- [ ] **JSON Schema exports**: Formal schemas for command outputs
  - Files: `crates/pw-cli/src/output.rs`

- [ ] **Artifact manifest**: Machine-readable artifact list in responses
  - Files: `crates/pw-cli/src/artifact_collector.rs`

---

## Summary of Highest-Impact Changes

1. **Proper GUID reference replacement** instead of retry/sleep loops
2. **Remove tokio::spawn from sync disposal/constructors**; make registry mutation synchronous
3. **Fix evaluate_value** to return typed/JSON values
4. **Rework outbound sending** to avoid holding mutex across .await
5. **Daemon**: make secure + add session management, then add batch stdin/stdout protocol

---

*Review generated by GPT-5.2 Thinking model, January 2026*
