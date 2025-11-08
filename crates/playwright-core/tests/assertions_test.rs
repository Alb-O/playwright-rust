// Integration tests for Assertions (Phase 5, Slice 1)
//
// Following TDD: Write tests first (Red), then implement (Green)
//
// Tests cover:
// - expect().to_be_visible() - auto-retry until visible
// - expect().to_be_hidden() - auto-retry until hidden
// - expect().not().to_be_visible() - negation support
// - Timeout behavior
// - Cross-browser compatibility

mod test_server;

use playwright_core::{expect, protocol::Playwright};
use test_server::TestServer;

#[tokio::test]
async fn test_to_be_visible_element_already_visible() {
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

    // Test: Element that is already visible should pass immediately
    let button = page.locator("#btn").await;
    expect(button)
        .to_be_visible()
        .await
        .expect("Button should be visible");

    browser.close().await.expect("Failed to close browser");
    server.shutdown();
}

#[tokio::test]
async fn test_to_be_hidden_element_not_exists() {
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

    // Test: Element that doesn't exist should be considered hidden
    let nonexistent = page.locator("#does-not-exist").await;
    expect(nonexistent)
        .to_be_hidden()
        .await
        .expect("Nonexistent element should be hidden");

    browser.close().await.expect("Failed to close browser");
    server.shutdown();
}

#[tokio::test]
async fn test_not_to_be_visible() {
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

    // Test: Negation - element should NOT be visible
    let nonexistent = page.locator("#does-not-exist").await;
    expect(nonexistent)
        .not()
        .to_be_visible()
        .await
        .expect("Nonexistent element should NOT be visible");

    browser.close().await.expect("Failed to close browser");
    server.shutdown();
}

#[tokio::test]
async fn test_to_be_visible_with_auto_retry() {
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

    // Create HTML with element that appears after delay
    page.goto(&format!("{}/", server.url()), None)
        .await
        .expect("Failed to navigate");

    // Inject JavaScript to show element after 500ms
    page.evaluate(
        r#"
        const div = document.createElement('div');
        div.id = 'delayed-element';
        div.textContent = 'I will appear!';
        div.style.display = 'none';
        document.body.appendChild(div);

        setTimeout(() => {
            div.style.display = 'block';
        }, 500);
        "#,
    )
    .await
    .expect("Failed to inject script");

    // Test: Assertion should wait and retry until element becomes visible
    let delayed = page.locator("#delayed-element").await;
    let start = std::time::Instant::now();

    expect(delayed)
        .to_be_visible()
        .await
        .expect("Delayed element should eventually be visible");

    let elapsed = start.elapsed();
    assert!(
        elapsed.as_millis() >= 500,
        "Should have waited at least 500ms, but was {:?}",
        elapsed
    );

    browser.close().await.expect("Failed to close browser");
    server.shutdown();
}

#[tokio::test]
async fn test_to_be_visible_timeout() {
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

    // Test: Should timeout if element never appears
    let nonexistent = page.locator("#does-not-exist").await;
    let result = expect(nonexistent)
        .with_timeout(std::time::Duration::from_millis(500))
        .to_be_visible()
        .await;

    assert!(result.is_err(), "Should timeout for nonexistent element");

    let error_message = result.unwrap_err().to_string();
    assert!(
        error_message.contains("timeout") || error_message.contains("Assertion"),
        "Error message should mention timeout: {}",
        error_message
    );

    browser.close().await.expect("Failed to close browser");
    server.shutdown();
}

#[tokio::test]
async fn test_to_be_hidden_with_auto_retry() {
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

    // Inject JavaScript to hide element after delay
    page.evaluate(
        r#"
        const btn = document.getElementById('btn');
        setTimeout(() => {
            btn.style.display = 'none';
        }, 500);
        "#,
    )
    .await
    .expect("Failed to inject script");

    // Test: Assertion should wait until element becomes hidden
    let button = page.locator("#btn").await;
    let start = std::time::Instant::now();

    expect(button)
        .to_be_hidden()
        .await
        .expect("Button should eventually be hidden");

    let elapsed = start.elapsed();
    assert!(
        elapsed.as_millis() >= 500,
        "Should have waited at least 500ms, but was {:?}",
        elapsed
    );

    browser.close().await.expect("Failed to close browser");
    server.shutdown();
}

#[tokio::test]
async fn test_custom_timeout() {
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

    // Inject element that appears after 2 seconds
    page.evaluate(
        r#"
        const div = document.createElement('div');
        div.id = 'slow-element';
        div.textContent = 'Slow element';
        div.style.display = 'none';
        document.body.appendChild(div);

        setTimeout(() => {
            div.style.display = 'block';
        }, 2000);
        "#,
    )
    .await
    .expect("Failed to inject script");

    // Test: With default timeout (5s), should succeed
    let slow = page.locator("#slow-element").await;
    expect(slow)
        .to_be_visible()
        .await
        .expect("Should wait up to 5s by default");

    browser.close().await.expect("Failed to close browser");
    server.shutdown();
}

// Cross-browser tests

#[tokio::test]
async fn test_to_be_visible_firefox() {
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
        .to_be_visible()
        .await
        .expect("Button should be visible in Firefox");

    browser.close().await.expect("Failed to close browser");
    server.shutdown();
}

#[tokio::test]
async fn test_to_be_hidden_webkit() {
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

    page.goto(&format!("{}/button.html", server.url()), None)
        .await
        .expect("Failed to navigate");

    let nonexistent = page.locator("#does-not-exist").await;
    expect(nonexistent)
        .to_be_hidden()
        .await
        .expect("Nonexistent element should be hidden in WebKit");

    browser.close().await.expect("Failed to close browser");
    server.shutdown();
}

#[tokio::test]
async fn test_auto_retry_webkit() {
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

    // Test auto-retry in WebKit
    page.evaluate(
        r#"
        const div = document.createElement('div');
        div.id = 'delayed-webkit';
        div.textContent = 'WebKit element';
        div.style.display = 'none';
        document.body.appendChild(div);

        setTimeout(() => {
            div.style.display = 'block';
        }, 300);
        "#,
    )
    .await
    .expect("Failed to inject script");

    let delayed = page.locator("#delayed-webkit").await;
    expect(delayed)
        .to_be_visible()
        .await
        .expect("Auto-retry should work in WebKit");

    browser.close().await.expect("Failed to close browser");
    server.shutdown();
}
