# pw-core Refactoring Plan

Three files in `pw-core` have grown past 1100 lines. Each conflates multiple responsibilities that would be clearer as separate modules. This document proposes splitting them without changing public APIs.

## The Problem

`page.rs` at 1212 lines contains the Page struct, navigation logic, screenshot capture, JavaScript evaluation, keyboard/mouse input delegation, and three distinct event systems (routing, downloads, dialogs). Finding the route handler registration means scrolling past 400 lines of input device methods.

`connection.rs` at 1188 lines mixes message type definitions, connection state, dispatch logic, object registry operations, and 265 lines of tests. The `Connection` struct itself is obscured by the protocol message boilerplate above it.

`frame.rs` at 1147 lines has 20+ `locator_*` methods that exist solely to delegate from `Locator` to the Frame's channel. These occupy 600 lines of nearly identical boilerplate, burying the actual frame logic.


## page.rs

The Page struct coordinates too many concerns. Navigation and screenshot methods share nothing except the underlying channel. Event handlers (route, download, dialog) each maintain their own Arc-wrapped handler vectors with identical registration patterns. The keyboard and mouse accessors return thin wrappers, but their 15 internal `keyboard_*` and `mouse_*` methods live inline.

Proposed split:

```
protocol/page/
├── mod.rs           # Page struct, ChannelOwner impl, core lifecycle
├── navigation.rs    # goto(), reload(), GotoOptions, WaitUntil, Response
├── content.rs       # query_selector variants, locator(), title()
├── screenshot.rs    # screenshot(), screenshot_to_file()
├── evaluate.rs      # evaluate(), evaluate_value()
├── input.rs         # keyboard()/mouse() accessors plus internal methods
├── events.rs        # route(), on_download(), on_dialog(), dispatch logic
└── types.rs         # RouteHandlerEntry, handler type aliases
```

The `Response` struct currently lives in `page.rs` but represents an HTTP response from navigation. It belongs with `GotoOptions` and `WaitUntil` in `navigation.rs`. The type aliases for handler futures (`RouteHandlerFuture`, `DownloadHandlerFuture`) move to `types.rs` where they can be imported by both `mod.rs` and `events.rs`.

`mod.rs` re-exports `GotoOptions`, `Response`, and `WaitUntil` to preserve the existing `use pw::protocol::page::Response` paths. The Page struct stays in `mod.rs` with its fields, constructor, and `ChannelOwner` implementation. Each method delegates to the appropriate submodule via `impl Page` blocks that import the submodule functions.


## connection.rs

The file opens with 180 lines of message type definitions (`Request`, `Response`, `Event`, `Message`, `Metadata`, error wrappers) before reaching the actual `Connection` struct. These types are pure data definitions with serde derives; they have no behavioral dependency on Connection.

The dispatch logic starting at line 652 (`dispatch_internal`, `handle_create`, `handle_dispose`, `handle_adopt`) forms a coherent unit responsible for routing incoming messages to objects. It reads from the object registry but doesn't touch the callback map or transport.

Proposed split:

```
server/connection/
├── mod.rs       # Connection struct, ConnectionLike trait, run loop, send_message
├── messages.rs  # Request, Response, Event, Message, Metadata, serde helpers
├── dispatch.rs  # dispatch_internal, handle_create/dispose/adopt, parse_protocol_error
└── tests.rs     # 265 lines of unit tests
```

The `serialize_arc_str` and `deserialize_arc_str` helpers are used by other modules (Frame's goto deserializes `ResponseReference` with `deserialize_arc_str`). These stay in `messages.rs` and get re-exported from `mod.rs`.

Moving tests to `tests.rs` with `#[cfg(test)] mod tests;` keeps them discoverable while removing 265 lines from the implementation file. The test helper `create_test_connection()` can become `pub(super)` for use by dispatch tests if needed.


## frame.rs

Frame's 20+ `locator_*` methods follow an identical pattern: build a JSON params object, call `self.channel().send()` or `send_no_result()`, deserialize the response. The methods differ only in the RPC method name, parameter construction, and return type. They exist because `Locator` holds a `Frame` reference and delegates all operations to it.

```rust
// This pattern repeats ~20 times with minor variations
pub(crate) async fn locator_is_visible(&self, selector: &str) -> Result<bool> {
    #[derive(Deserialize)]
    struct IsVisibleResponse { value: bool }
    let response: IsVisibleResponse = self.channel()
        .send("isVisible", serde_json::json!({
            "selector": selector,
            "strict": true,
            "timeout": crate::DEFAULT_TIMEOUT_MS
        }))
        .await?;
    Ok(response.value)
}
```

Grouping by concern:

```
protocol/frame/
├── mod.rs              # Frame struct, ChannelOwner impl
├── navigation.rs       # goto() with its Response extraction logic
├── queries.rs          # query_selector(), query_selector_all(), title()
├── evaluate.rs         # frame_evaluate_expression variants
├── locator_state.rs    # locator_count, text_content, inner_text/html, get_attribute, is_* methods
├── locator_actions.rs  # click, dblclick, fill, clear, press, check, uncheck, hover
└── locator_input.rs    # input_value, select_option variants, set_input_files variants
```

The `goto()` method at 100 lines deserves isolation. It constructs `GotoOptions`, sends the RPC, polls the connection for the Response object (with a retry loop that should become proper GUID replacement in a future refactor), and extracts response data from the initializer. This complexity shouldn't share a file with boilerplate state queries.

The `set_input_files` methods (4 variants for path/payload × single/multiple) read files, base64-encode them, and construct payloads. They share encoding logic that could be extracted to a helper, but at minimum they should be grouped together in `locator_input.rs` where their commonality is visible.


## Migration Order

Start with `connection.rs`. Its tests are self-contained and the message types have no dependencies on Connection internals. Extract `messages.rs` first, verify the build, then extract `dispatch.rs`, then move tests. Each step is independently verifiable.

`frame.rs` comes second. The locator methods are tedious but mechanical to move. Group them by the submodule they'll inhabit, move each group, update visibility. The Frame struct itself barely changes.

`page.rs` is most complex because of the event handler closures and their type aliases. The `on_event` match arms dispatch to methods that live in `events.rs`, requiring careful import management. Save this for last when the pattern is established.


## Visibility

Types currently `pub` stay `pub`. Methods marked `pub(crate)` (the locator delegates, keyboard/mouse internals) stay `pub(crate)`. Submodule functions become `pub(super)` when only called by the parent `mod.rs`, or `pub(crate)` when called from outside the module.

The `ChannelOwner` trait implementation stays in `mod.rs` for each refactored type. Its `on_event` method dispatches to submodule handlers, so it needs visibility into `events.rs` internals. Using `pub(super)` for the handler functions keeps them hidden from the rest of the crate.


## Verification

After each file split:

1. `cargo build --package pw-core` must succeed
2. `cargo test --package pw-core --lib` must pass (69 tests currently)
3. `cargo test --package pw-cli` must pass (40 unit + 29 e2e tests)
4. `cargo doc --package pw-core` must generate without warnings

The public API paths (`pw::protocol::Page`, `pw::protocol::page::Response`, `pw::server::connection::Message`) must continue resolving. Add `pub use` re-exports in the module root files to preserve them.


## Target State

No implementation file exceeds 400 lines excluding tests. Each file has a single clear responsibility. Related code lives together: all navigation logic in one place, all locator state queries in another. Finding where route handlers are registered means opening `page/events.rs`, not scrolling through a 1200-line file.

The refactor changes zero public API signatures. Downstream code using `pw::protocol::Page` continues working. The improvement is entirely internal: maintainability, navigability, and the reduced cognitive load of smaller, focused files.
