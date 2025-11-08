# Phase 5: Advanced Testing Features

**Status:** Planning

**Goal:** Implement advanced testing features including assertions with auto-retry, network interception, and other testing capabilities.

**Feature:** Assertions, network interception, route mocking, downloads, dialogs, and deferred Phase 4 enhancements

**User Story:** As a Rust developer, I want powerful testing features like auto-retry assertions and network mocking so that I can write robust, maintainable test suites.

**Related ADRs:**
- [ADR-0001: Protocol Architecture](../adr/0001-protocol-architecture.md)

---

## Prerequisites from Phase 4

Phase 5 builds on Phase 4's advanced features:
- ✅ ElementHandle protocol objects
- ✅ Screenshot options (type, quality, full_page, clip)
- ✅ Action options (Click, Fill, Press, Check, Hover, Select)
- ✅ SelectOption variants (value, label, index)
- ✅ Keyboard/Mouse options
- ✅ Navigation error handling

---

## Deferred from Phase 4

Low-priority items deferred from Phase 4 that can be implemented in Phase 5:

1. **set_checked() Convenience Method**
   - `locator.set_checked(checked: bool)`
   - Calls check() or uncheck() based on boolean

2. **FilePayload Struct**
   - In-memory file creation without PathBuf
   - `FilePayload { name: String, mime_type: String, buffer: Vec<u8> }`

3. **Modifier Key Parsing**
   - Keyboard.press with compound keys (e.g., "Control+A")

4. **Screenshot Mask Options**
   - `mask`: Hide sensitive elements
   - `mask_color`: Color for masked elements

---

## Proposed Scope for Phase 5

### High Priority

1. **Assertions with Auto-Retry** (Highest Priority)
   - `expect(locator).to_be_visible()` API
   - Auto-retry logic (poll until condition met or timeout)
   - Common assertions: to_be_visible, to_be_hidden, to_have_text, to_have_value
   - Negation: to_not_be_visible, etc.
   - Custom timeout configuration

2. **Network Interception Basics** (High Priority)
   - `page.route()` for request interception
   - Route matching by URL patterns
   - Request continuation, fulfillment, abort
   - Access to request/response data

### Medium Priority

3. **Downloads Handling**
   - Download event handling
   - Save downloaded files
   - Download metadata access

4. **Dialogs Handling**
   - Alert, confirm, prompt handling
   - Accept/dismiss dialogs
   - Access dialog messages

5. **Deferred Phase 4 Items** (As time permits)
   - set_checked() convenience method
   - FilePayload struct
   - Modifier key parsing
   - Screenshot mask options

### Future Phases (Not Phase 5)

Defer to Phase 6 or later:
- **Mobile Emulation** - Device descriptors, viewport emulation
- **Videos and Tracing** - Recording and trace generation
- **Advanced Network** - HAR export, service workers
- **Context Options** - Geolocation, permissions, user agent

---

## Success Criteria

Phase 5 will be considered complete when:

- [ ] Assertions API implemented with auto-retry
- [ ] Common assertions work (visible, hidden, text, value, etc.)
- [ ] Network route() API implemented
- [ ] Request interception works (continue, fulfill, abort)
- [ ] Downloads can be captured and saved
- [ ] Dialogs can be handled (accept, dismiss)
- [ ] All tests passing cross-browser
- [ ] Documentation complete

---

## Implementation Plan

**Status:** Planning - Ready to start Slice 1

Phase 5 follows the same TDD and vertical slicing approach as previous phases.

### Slice 1: Assertions Foundation - expect() API and to_be_visible()

**Status:** ✅ COMPLETE

**Goal:** Implement the `expect()` API foundation with auto-retry logic and the first assertion (to_be_visible).

**Why First:** Assertions are the highest-priority testing feature and the foundation for the rest of the assertions API.

**Research Completed:**
- ✅ Playwright's expect API uses standalone function (matches Python/JS)
- ✅ Auto-retry: poll with configurable interval (default 100ms) until timeout (default 5s)
- ✅ Negation via .not() method
- ✅ Error messages include selector, condition, and timeout

**Tasks:**
- [x] Research Playwright's expect API and auto-retry logic
- [x] Design Rust API (chose standalone `expect(locator)` for cross-language consistency)
- [x] Create Expectation struct with timeout configuration
- [x] Implement auto-retry polling mechanism
- [x] Implement to_be_visible() assertion
- [x] Implement to_be_hidden() assertion (reuses to_be_visible with negation)
- [x] Implement Page.evaluate() for dynamic element testing
- [x] Cross-browser testing (Chromium, Firefox, WebKit all passing)
- [x] Documentation with examples

**Implementation Details:**

**Files Created:**
- `crates/playwright-core/src/assertions.rs` - expect() API and Expectation struct
- `crates/playwright-core/tests/assertions_test.rs` - Integration tests

**Files Modified:**
- `crates/playwright-core/src/error.rs` - Added AssertionTimeout error variant
- `crates/playwright-core/src/lib.rs` - Exported expect() function
- `crates/playwright-core/src/protocol/page.rs` - Added evaluate() method
- `crates/playwright-core/src/protocol/frame.rs` - Added frame_evaluate_expression() method

**Test Results:**
  - `test_to_be_visible_element_already_visible` - Basic visibility check
  - `test_to_be_hidden_element_not_exists` - Hidden check for nonexistent element
  - `test_not_to_be_visible` - Negation support
  - `test_to_be_visible_timeout` - Timeout behavior
  - `test_to_be_visible_with_auto_retry` - Auto-retry with delayed element (500ms)
  - `test_to_be_hidden_with_auto_retry` - Auto-retry with element hiding
  - `test_custom_timeout` - Custom timeout configuration (2s delay)
  - `test_to_be_visible_firefox` - Firefox compatibility
  - `test_to_be_hidden_webkit` - WebKit compatibility
  - `test_auto_retry_webkit` - WebKit auto-retry (300ms delay)

**Key Implementation Details:**
- Auto-retry polling: 100ms interval, 5s default timeout
- Protocol integration: Implemented Page.evaluate() via Frame.evaluateExpression
- Visibility detection: Elements need non-zero dimensions (textContent required for empty elements)
- Cross-browser: All tests pass on Chromium, Firefox, and WebKit

**API Design Considerations:**

Option 1: Standalone function (matches Playwright Python/JS)
```rust
use playwright_core::expect;

expect(page.locator("button")).to_be_visible().await?;
expect(page.locator("input")).to_have_value("hello").await?;
```

Option 2: Trait-based (more Rust-idiomatic)
```rust
page.locator("button").expect().to_be_visible().await?;
page.locator("input").expect().to_have_value("hello").await?;
```

**Recommendation:** Option 1 (standalone) for consistency with other Playwright bindings.

---

### Slice 2: Text and Value Assertions

**Goal:** Implement text-based assertions (to_have_text, to_contain_text, to_have_value).

**Tasks:**
- [ ] Implement to_have_text() - exact match
- [ ] Implement to_contain_text() - substring match
- [ ] Implement to_have_value() - for input elements
- [ ] Support for regex patterns
- [ ] Tests for all text assertions
- [ ] Cross-browser testing

---

### Slice 3: State Assertions

**Goal:** Implement state-based assertions (enabled, disabled, checked, editable).

**Tasks:**
- [ ] Implement to_be_enabled() / to_be_disabled()
- [ ] Implement to_be_checked() / to_be_unchecked()
- [ ] Implement to_be_editable()
- [ ] Implement to_be_focused()
- [ ] Tests for all state assertions
- [ ] Cross-browser testing

---

### Slice 4: Network Route API Foundation

**Goal:** Implement page.route() for basic request interception.

**Tasks:**
- [ ] Research Playwright route API
- [ ] Implement route matching (URL patterns, regex)
- [ ] Implement route handlers (closure-based)
- [ ] Implement route.continue()
- [ ] Implement route.abort()
- [ ] Basic route tests
- [ ] Cross-browser testing

---

### Slice 5: Network Response Fulfillment

**Goal:** Implement route.fulfill() for mocking responses.

**Tasks:**
- [ ] Implement route.fulfill() with custom response
- [ ] Support for status, headers, body
- [ ] JSON response helpers
- [ ] Tests for response mocking
- [ ] Cross-browser testing

---

### Slice 6: Downloads and Dialogs

**Goal:** Implement download and dialog event handling.

**Tasks:**
- [ ] Implement download event handling
- [ ] Download save functionality
- [ ] Dialog event handling (alert, confirm, prompt)
- [ ] Accept/dismiss dialogs
- [ ] Tests for downloads
- [ ] Tests for dialogs
- [ ] Cross-browser testing

---

### Slice 7: Phase 4 Deferrals and Polish

**Goal:** Implement remaining low-priority items and complete documentation.

**Tasks:**
- [ ] Implement set_checked() convenience method
- [ ] Implement FilePayload struct (if time permits)
- [ ] Implement modifier key parsing (if time permits)
- [ ] Complete all rustdoc
- [ ] Update README with Phase 5 examples
- [ ] Update roadmap.md
- [ ] Mark Phase 5 complete

---

This order prioritizes:
- Highest-value testing features first (assertions)
- Network mocking before advanced features
- Progressive complexity (simple assertions → complex network handling)
- Deferred items last (lowest priority)

---

**Created:** 2025-11-08
**Last Updated:** 2025-11-08

---
