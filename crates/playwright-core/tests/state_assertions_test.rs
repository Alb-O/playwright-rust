// Integration tests for State Assertions (Phase 5, Slice 3)
//
// Following TDD: Write tests first (Red), then implement (Green)
//
// Tests cover:
// - expect().to_be_enabled() / to_be_disabled()
// - expect().to_be_checked() / to_be_unchecked()
// - expect().to_be_editable()
// - expect().to_be_focused()
// - Auto-retry behavior
// - Cross-browser compatibility

mod test_server;

use playwright_core::{expect, protocol::Playwright};
use test_server::TestServer;

// ============================================================================
// to_be_enabled() / to_be_disabled() tests
// ============================================================================

#[tokio::test]
async fn test_to_be_enabled() {
    let server = TestServer::start().await;
    let playwright = Playwright::launch()
        .await
        .expect("Failed to launch Playwright");
    let browser = playwright
        .chromium()
        .launch()
        .await
        .expect("Failed to launch browser");
    let page = browser.new_page().await.expect("Failed to create page");

    page.goto(&format!("{}/button.html", server.url()), None)
        .await
        .expect("Failed to navigate");

    // Test: Enabled button should pass
    let button = page.locator("#btn").await;
    expect(button)
        .to_be_enabled()
        .await
        .expect("Button should be enabled");

    browser.close().await.expect("Failed to close browser");
    server.shutdown();
}

#[tokio::test]
async fn test_to_be_disabled() {
    let server = TestServer::start().await;
    let playwright = Playwright::launch()
        .await
        .expect("Failed to launch Playwright");
    let browser = playwright
        .chromium()
        .launch()
        .await
        .expect("Failed to launch browser");
    let page = browser.new_page().await.expect("Failed to create page");

    page.goto(&format!("{}/", server.url()), None)
        .await
        .expect("Failed to navigate");

    // Create a disabled button
    page.evaluate(
        r#"
        const btn = document.createElement('button');
        btn.id = 'disabled-btn';
        btn.textContent = 'Disabled';
        btn.disabled = true;
        document.body.appendChild(btn);
        "#,
    )
    .await
    .expect("Failed to inject script");

    // Test: Disabled button should pass
    let button = page.locator("#disabled-btn").await;
    expect(button)
        .to_be_disabled()
        .await
        .expect("Button should be disabled");

    browser.close().await.expect("Failed to close browser");
    server.shutdown();
}

#[tokio::test]
async fn test_to_be_enabled_with_auto_retry() {
    let server = TestServer::start().await;
    let playwright = Playwright::launch()
        .await
        .expect("Failed to launch Playwright");
    let browser = playwright
        .chromium()
        .launch()
        .await
        .expect("Failed to launch browser");
    let page = browser.new_page().await.expect("Failed to create page");

    page.goto(&format!("{}/", server.url()), None)
        .await
        .expect("Failed to navigate");

    // Create a button that becomes enabled after delay
    page.evaluate(
        r#"
        const btn = document.createElement('button');
        btn.id = 'delayed-btn';
        btn.textContent = 'Will be enabled';
        btn.disabled = true;
        document.body.appendChild(btn);

        setTimeout(() => {
            btn.disabled = false;
        }, 100);
        "#,
    )
    .await
    .expect("Failed to inject script");

    // Test: Should wait for button to become enabled
    let button = page.locator("#delayed-btn").await;
    expect(button)
        .to_be_enabled()
        .await
        .expect("Button should eventually be enabled");

    browser.close().await.expect("Failed to close browser");
    server.shutdown();
}

#[tokio::test]
async fn test_not_to_be_enabled() {
    let server = TestServer::start().await;
    let playwright = Playwright::launch()
        .await
        .expect("Failed to launch Playwright");
    let browser = playwright
        .chromium()
        .launch()
        .await
        .expect("Failed to launch browser");
    let page = browser.new_page().await.expect("Failed to create page");

    page.goto(&format!("{}/", server.url()), None)
        .await
        .expect("Failed to navigate");

    // Create a disabled button
    page.evaluate(
        r#"
        const btn = document.createElement('button');
        btn.id = 'disabled-btn';
        btn.disabled = true;
        document.body.appendChild(btn);
        "#,
    )
    .await
    .expect("Failed to inject script");

    // Test: Negation - disabled button should NOT be enabled
    let button = page.locator("#disabled-btn").await;
    expect(button)
        .not()
        .to_be_enabled()
        .await
        .expect("Disabled button should NOT be enabled");

    browser.close().await.expect("Failed to close browser");
    server.shutdown();
}

// ============================================================================
// to_be_checked() / to_be_unchecked() tests
// ============================================================================

#[tokio::test]
async fn test_to_be_checked() {
    let server = TestServer::start().await;
    let playwright = Playwright::launch()
        .await
        .expect("Failed to launch Playwright");
    let browser = playwright
        .chromium()
        .launch()
        .await
        .expect("Failed to launch browser");
    let page = browser.new_page().await.expect("Failed to create page");

    page.goto(&format!("{}/", server.url()), None)
        .await
        .expect("Failed to navigate");

    // Create a checked checkbox
    page.evaluate(
        r#"
        const checkbox = document.createElement('input');
        checkbox.type = 'checkbox';
        checkbox.id = 'checked-box';
        checkbox.checked = true;
        document.body.appendChild(checkbox);
        "#,
    )
    .await
    .expect("Failed to inject script");

    // Test: Checked checkbox should pass
    let checkbox = page.locator("#checked-box").await;
    expect(checkbox)
        .to_be_checked()
        .await
        .expect("Checkbox should be checked");

    browser.close().await.expect("Failed to close browser");
    server.shutdown();
}

#[tokio::test]
async fn test_to_be_unchecked() {
    let server = TestServer::start().await;
    let playwright = Playwright::launch()
        .await
        .expect("Failed to launch Playwright");
    let browser = playwright
        .chromium()
        .launch()
        .await
        .expect("Failed to launch browser");
    let page = browser.new_page().await.expect("Failed to create page");

    page.goto(&format!("{}/", server.url()), None)
        .await
        .expect("Failed to navigate");

    // Create an unchecked checkbox
    page.evaluate(
        r#"
        const checkbox = document.createElement('input');
        checkbox.type = 'checkbox';
        checkbox.id = 'unchecked-box';
        checkbox.checked = false;
        document.body.appendChild(checkbox);
        "#,
    )
    .await
    .expect("Failed to inject script");

    // Test: Unchecked checkbox should pass
    let checkbox = page.locator("#unchecked-box").await;
    expect(checkbox)
        .to_be_unchecked()
        .await
        .expect("Checkbox should be unchecked");

    browser.close().await.expect("Failed to close browser");
    server.shutdown();
}

#[tokio::test]
async fn test_to_be_checked_with_auto_retry() {
    let server = TestServer::start().await;
    let playwright = Playwright::launch()
        .await
        .expect("Failed to launch Playwright");
    let browser = playwright
        .chromium()
        .launch()
        .await
        .expect("Failed to launch browser");
    let page = browser.new_page().await.expect("Failed to create page");

    page.goto(&format!("{}/", server.url()), None)
        .await
        .expect("Failed to navigate");

    // Create checkbox that becomes checked after delay
    page.evaluate(
        r#"
        const checkbox = document.createElement('input');
        checkbox.type = 'checkbox';
        checkbox.id = 'delayed-checkbox';
        checkbox.checked = false;
        document.body.appendChild(checkbox);

        setTimeout(() => {
            checkbox.checked = true;
        }, 100);
        "#,
    )
    .await
    .expect("Failed to inject script");

    // Test: Should wait for checkbox to become checked
    let checkbox = page.locator("#delayed-checkbox").await;
    expect(checkbox)
        .to_be_checked()
        .await
        .expect("Checkbox should eventually be checked");

    browser.close().await.expect("Failed to close browser");
    server.shutdown();
}

// ============================================================================
// to_be_editable() tests
// ============================================================================

#[tokio::test]
async fn test_to_be_editable() {
    let server = TestServer::start().await;
    let playwright = Playwright::launch()
        .await
        .expect("Failed to launch Playwright");
    let browser = playwright
        .chromium()
        .launch()
        .await
        .expect("Failed to launch browser");
    let page = browser.new_page().await.expect("Failed to create page");

    page.goto(&format!("{}/", server.url()), None)
        .await
        .expect("Failed to navigate");

    // Create an editable input
    page.evaluate(
        r#"
        const input = document.createElement('input');
        input.type = 'text';
        input.id = 'editable-input';
        document.body.appendChild(input);
        "#,
    )
    .await
    .expect("Failed to inject script");

    // Test: Editable input should pass
    let input = page.locator("#editable-input").await;
    expect(input)
        .to_be_editable()
        .await
        .expect("Input should be editable");

    browser.close().await.expect("Failed to close browser");
    server.shutdown();
}

#[tokio::test]
async fn test_not_to_be_editable() {
    let server = TestServer::start().await;
    let playwright = Playwright::launch()
        .await
        .expect("Failed to launch Playwright");
    let browser = playwright
        .chromium()
        .launch()
        .await
        .expect("Failed to launch browser");
    let page = browser.new_page().await.expect("Failed to create page");

    page.goto(&format!("{}/", server.url()), None)
        .await
        .expect("Failed to navigate");

    // Create a readonly input
    page.evaluate(
        r#"
        const input = document.createElement('input');
        input.type = 'text';
        input.id = 'readonly-input';
        input.readOnly = true;
        document.body.appendChild(input);
        "#,
    )
    .await
    .expect("Failed to inject script");

    // Test: Readonly input should NOT be editable
    let input = page.locator("#readonly-input").await;
    expect(input)
        .not()
        .to_be_editable()
        .await
        .expect("Readonly input should NOT be editable");

    browser.close().await.expect("Failed to close browser");
    server.shutdown();
}

// NOTE: to_be_focused() tests are not included - the assertion is deferred to future implementation
// because Playwright doesn't expose isFocused() at the protocol level. Requires implementing the
// 'expect' protocol command or properly handling evalOnSelector return values.

//  ============================================================================
// Cross-browser tests
// ============================================================================

// Removed test_to_be_focused() and test_not_to_be_focused() - deferred

/* DEFERRED
#[tokio::test]
async fn test_to_be_focused() {
    let server = TestServer::start().await;
    let playwright = Playwright::launch()
        .await
        .expect("Failed to launch Playwright");
    let browser = playwright
        .chromium()
        .launch()
        .await
        .expect("Failed to launch browser");
    let page = browser.new_page().await.expect("Failed to create page");

    page.goto(&format!("{}/", server.url()), None)
        .await
        .expect("Failed to navigate");

    // Create and focus an input
    page.evaluate(
        r#"
        const input = document.createElement('input');
        input.type = 'text';
        input.id = 'focused-input';
        document.body.appendChild(input);
        input.focus();
        "#,
    )
    .await
    .expect("Failed to inject script");

    // Test: Focused input should pass
    let input = page.locator("#focused-input").await;
    expect(input)
        .to_be_focused()
        .await
        .expect("Input should be focused");

    browser.close().await.expect("Failed to close browser");
    server.shutdown();
}

#[tokio::test]
async fn test_not_to_be_focused() {
    let server = TestServer::start().await;
    let playwright = Playwright::launch()
        .await
        .expect("Failed to launch Playwright");
    let browser = playwright
        .chromium()
        .launch()
        .await
        .expect("Failed to launch browser");
    let page = browser.new_page().await.expect("Failed to create page");

    page.goto(&format!("{}/", server.url()), None)
        .await
        .expect("Failed to navigate");

    // Create an unfocused input
    page.evaluate(
        r#"
        const input = document.createElement('input');
        input.type = 'text';
        input.id = 'unfocused-input';
        document.body.appendChild(input);
        "#,
    )
    .await
    .expect("Failed to inject script");

    // Test: Unfocused input should NOT be focused
    let input = page.locator("#unfocused-input").await;
    expect(input)
        .not()
        .to_be_focused()
        .await
        .expect("Input should NOT be focused");

    browser.close().await.expect("Failed to close browser");
    server.shutdown();
}
*/

#[tokio::test]
async fn test_to_be_enabled_firefox() {
    let server = TestServer::start().await;
    let playwright = Playwright::launch()
        .await
        .expect("Failed to launch Playwright");
    let browser = playwright
        .firefox()
        .launch()
        .await
        .expect("Failed to launch Firefox");
    let page = browser.new_page().await.expect("Failed to create page");

    page.goto(&format!("{}/button.html", server.url()), None)
        .await
        .expect("Failed to navigate");

    let button = page.locator("#btn").await;
    expect(button)
        .to_be_enabled()
        .await
        .expect("Should work in Firefox");

    browser.close().await.expect("Failed to close browser");
    server.shutdown();
}

#[tokio::test]
async fn test_to_be_checked_webkit() {
    let server = TestServer::start().await;
    let playwright = Playwright::launch()
        .await
        .expect("Failed to launch Playwright");
    let browser = playwright
        .webkit()
        .launch()
        .await
        .expect("Failed to launch WebKit");
    let page = browser.new_page().await.expect("Failed to create page");

    page.goto(&format!("{}/", server.url()), None)
        .await
        .expect("Failed to navigate");

    // Create a checked checkbox
    page.evaluate(
        r#"
        const checkbox = document.createElement('input');
        checkbox.type = 'checkbox';
        checkbox.id = 'webkit-checkbox';
        checkbox.checked = true;
        document.body.appendChild(checkbox);
        "#,
    )
    .await
    .expect("Failed to inject script");

    let checkbox = page.locator("#webkit-checkbox").await;
    expect(checkbox)
        .to_be_checked()
        .await
        .expect("Should work in WebKit");

    browser.close().await.expect("Failed to close browser");
    server.shutdown();
}

#[tokio::test]
async fn test_to_be_editable_webkit() {
    let server = TestServer::start().await;
    let playwright = Playwright::launch()
        .await
        .expect("Failed to launch Playwright");
    let browser = playwright
        .webkit()
        .launch()
        .await
        .expect("Failed to launch WebKit");
    let page = browser.new_page().await.expect("Failed to create page");

    page.goto(&format!("{}/", server.url()), None)
        .await
        .expect("Failed to navigate");

    // Create an editable input
    page.evaluate(
        r#"
        const input = document.createElement('input');
        input.type = 'text';
        input.id = 'webkit-input';
        document.body.appendChild(input);
        "#,
    )
    .await
    .expect("Failed to inject script");

    let input = page.locator("#webkit-input").await;
    expect(input)
        .to_be_editable()
        .await
        .expect("Should work in WebKit");

    browser.close().await.expect("Failed to close browser");
    server.shutdown();
}
