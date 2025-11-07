# Phase 3: Page Interactions

**Status:** Not Started

**Goal:** Implement core page interactions (navigation, locators, actions) matching playwright-python API.

**Feature:** Navigate to URLs, find elements, and perform basic interactions

**User Story:** As a Rust developer, I want to navigate to web pages and interact with elements so that I can automate browser testing workflows.

**Related ADRs:**
- [ADR-0001: Protocol Architecture](../adr/0001-protocol-architecture.md)

---

## Prerequisites from Phase 2

Phase 3 builds on Phase 2's browser lifecycle management:
- ✅ Browser launching (all three browsers)
- ✅ Context and page creation
- ✅ Page objects at about:blank
- ✅ Lifecycle cleanup (close methods)

---

## Deferred from Phase 2

### Technical Improvements

1. **Windows Testing**
   - Current: Verified on macOS and Linux. Windows CI runs unit tests only (integration tests hang).
   - Issue: Integration tests hang on Windows after 60+ seconds when launching browsers
   - Root cause: Stdio pipe cleanup issue - Playwright server process doesn't terminate cleanly on Windows
   - Progress: ✅ Browser::close() implemented, but still hangs on Windows
   - Goal: Fix stdio pipe handling and implement proper cleanup
   - **When to re-enable full Windows CI**: After implementing explicit Drop for Playwright/Connection that:
     - Sends close/disconnect protocol messages to server
     - Waits for graceful server shutdown
     - Properly closes stdio pipes on Windows (different from Unix)
     - Kills child process if graceful shutdown times out
   - **Possible solutions**:
     1. Implement Drop for Playwright that calls a blocking cleanup method
     2. Add explicit `Playwright::disconnect()` method (like playwright-python)
     3. Better stdio pipe handling on Windows (tokio::process differences)
   - Priority: High (blocking full Windows support)
   - **Workaround**: CI runs `cargo test --lib` on Windows (unit tests only)

2. **Disposal Cleanup Refactor**
   - Current: Uses `tokio::spawn` for async unregister in `ChannelOwner::dispose()`
   - Goal: Refactor to fully synchronous disposal with background cleanup task
   - Rationale: All official bindings use synchronous disposal
   - Priority: Low (current approach works correctly)

3. **Error Message Improvements**
   - Current: Functional but terse error messages
   - Goal: Add context and suggestions to error messages
   - Priority: Low

### Testing Improvements

1. **IPC Performance Benchmarking**
   - Deferred from ADR-0001 validation checklist
   - Goal: Measure latency overhead (<5ms per operation expected)
   - Priority: Low (browser operations are 100+ms, IPC overhead negligible)

2. **Transport Reconnection**
   - Test reconnection scenarios after server crash/restart
   - Verify graceful degradation and recovery
   - Deferred from Phase 1 transport testing
   - Priority: Medium

### API Improvements

1. **Context Options API**
   - Phase 2 implemented minimal options (empty JSON)
   - Goal: Add full ContextOptions support:
     - Viewport configuration
     - User agent
     - Geolocation
     - Permissions
     - Locale/timezone
   - Priority: Medium (needed for mobile emulation in Phase 4)

2. **URL Tracking**
   - Phase 2: `page.url()` always returns "about:blank"
   - Goal: Track URL changes via page navigation events
   - Priority: High (required for Phase 3 navigation)

---

## Proposed Scope

### Core Features

1. **Navigation API**
   - `page.goto(url)` - Navigate to URL
   - `page.go_back()` - Navigate back
   - `page.go_forward()` - Navigate forward
   - `page.reload()` - Reload page
   - Navigation options: timeout, wait_until (load/domcontentloaded/networkidle)
   - Response handling

2. **Locators API**
   - `page.locator(selector)` - Create locator with auto-waiting
   - Selector strategies: CSS, text, XPath
   - Locator chaining
   - Auto-waiting and auto-retry

3. **Actions API**
   - `locator.click()` - Click element
   - `locator.fill(text)` - Fill input
   - `locator.type(text)` - Type with delays
   - `locator.press(key)` - Press keyboard key
   - `locator.select_option(value)` - Select dropdown option
   - `locator.check()` / `uncheck()` - Checkboxes
   - Action options: timeout, force, position

4. **Query API**
   - `locator.text_content()` - Get text
   - `locator.inner_text()` - Get visible text
   - `locator.inner_html()` - Get HTML
   - `locator.get_attribute(name)` - Get attribute
   - `locator.count()` - Count matching elements

5. **Waiting API**
   - `page.wait_for_selector(selector)` - Wait for element
   - `page.wait_for_url(pattern)` - Wait for URL match
   - `page.wait_for_load_state(state)` - Wait for page load state
   - `locator.wait_for()` - Wait for locator conditions

6. **Frame Support**
   - Basic frame handling
   - `page.frame_locator(selector)` - Locate iframe
   - Frame navigation

7. **Screenshots**
   - `page.screenshot()` - Capture screenshot
   - Options: path, full_page, clip, type (png/jpeg)

### Documentation

- Rustdoc for all public APIs
- Examples for navigation and interaction patterns
- Comparison with playwright-python API

### Testing

- Integration tests for navigation
- Tests for all action types
- Cross-browser tests
- Error handling tests (timeouts, element not found)

---

## Out of Scope (Future Phases)

- **Phase 4:** Assertions with auto-retry, network interception, route mocking, mobile emulation, videos, tracing, downloads, dialogs
- **Phase 5:** Production hardening, performance optimization, comprehensive documentation

---

## Success Criteria

- [ ] Can navigate to URLs
- [ ] Can find elements with locators
- [ ] Can perform basic actions (click, fill, type)
- [ ] Can query element content
- [ ] Can take screenshots
- [ ] Auto-waiting works correctly
- [ ] All tests passing with real browsers (macOS, Linux)
- [ ] Windows CI support (requires cleanup fixes from deferred items)
- [ ] Documentation complete
- [ ] Example code works

---

## Implementation Plan

**Note:** This implementation plan will be filled in just-in-time when Phase 3 begins, following the same vertical slicing approach as Phase 2.

Tentative slice breakdown:
1. Navigation API (`page.goto()`)
2. Locators foundation
3. Basic actions (click, fill)
4. Query API
5. Waiting API
6. Screenshots
7. Cleanup and documentation

---

**Created:** 2025-11-07
**Last Updated:** 2025-11-07

---
