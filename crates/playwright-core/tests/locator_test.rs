// Integration tests for Locator functionality
//
// Following TDD: Write tests first (Red), then implement (Green)
//
// Tests cover:
// - Locator creation (page.locator)
// - Locator chaining (first, last, nth, locator)
// - Query methods (count, text_content, inner_text, inner_html, get_attribute)
// - State queries (is_visible, is_enabled, is_checked, is_editable)

mod test_server;

use playwright_core::protocol::Playwright;
use test_server::TestServer;

#[tokio::test]
async fn test_locator_creation() {
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

    page.goto(&format!("{}/locator.html", server.url()), None)
        .await
        .expect("Failed to navigate");

    // Test: Create a locator
    let heading = page.locator("h1").await;

    // Locator should be created (doesn't execute until action)
    assert_eq!(heading.selector(), "h1");

    browser.close().await.expect("Failed to close browser");
    server.shutdown();
}

#[tokio::test]
async fn test_locator_count() {
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

    page.goto(&format!("{}/locator.html", server.url()), None)
        .await
        .expect("Failed to navigate");

    // Test: Count elements
    let paragraphs = page.locator("p").await;
    let count = paragraphs.count().await.expect("Failed to get count");

    // locator.html has exactly 3 paragraphs
    assert_eq!(count, 3);

    browser.close().await.expect("Failed to close browser");
    server.shutdown();
}

#[tokio::test]
async fn test_locator_text_content() {
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

    page.goto(&format!("{}/locator.html", server.url()), None)
        .await
        .expect("Failed to navigate");

    // Test: Get text content
    let heading = page.locator("h1").await;
    let text = heading
        .text_content()
        .await
        .expect("Failed to get text content");

    assert_eq!(text, Some("Test Page".to_string()));

    browser.close().await.expect("Failed to close browser");
    server.shutdown();
}

#[tokio::test]
async fn test_locator_chaining_first() {
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

    page.goto(&format!("{}/locator.html", server.url()), None)
        .await
        .expect("Failed to navigate");

    // Test: Get first paragraph
    let paragraphs = page.locator("p").await;
    let first = paragraphs.first();

    assert_eq!(first.selector(), "p >> nth=0");

    let text = first
        .text_content()
        .await
        .expect("Failed to get text content");
    assert_eq!(text, Some("First paragraph".to_string()));

    browser.close().await.expect("Failed to close browser");
    server.shutdown();
}

#[tokio::test]
async fn test_locator_chaining_last() {
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

    page.goto(&format!("{}/locator.html", server.url()), None)
        .await
        .expect("Failed to navigate");

    // Test: Get last paragraph
    let paragraphs = page.locator("p").await;
    let last = paragraphs.last();

    assert_eq!(last.selector(), "p >> nth=-1");

    let text = last
        .text_content()
        .await
        .expect("Failed to get text content");
    assert_eq!(text, Some("Third paragraph".to_string()));

    browser.close().await.expect("Failed to close browser");
    server.shutdown();
}

#[tokio::test]
async fn test_locator_chaining_nth() {
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

    page.goto(&format!("{}/locator.html", server.url()), None)
        .await
        .expect("Failed to navigate");

    // Test: Get nth element
    let paragraphs = page.locator("p").await;
    let second = paragraphs.nth(1);

    assert_eq!(second.selector(), "p >> nth=1");

    let text = second
        .text_content()
        .await
        .expect("Failed to get text content");
    assert_eq!(text, Some("Second paragraph".to_string()));

    browser.close().await.expect("Failed to close browser");
    server.shutdown();
}

#[tokio::test]
async fn test_locator_nested() {
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

    page.goto(&format!("{}/locator.html", server.url()), None)
        .await
        .expect("Failed to navigate");

    // Test: Nested locators
    let container = page.locator(".container").await;
    let nested = container.locator("#nested");

    assert_eq!(nested.selector(), ".container >> #nested");

    let text = nested
        .text_content()
        .await
        .expect("Failed to get text content");
    assert_eq!(text, Some("Nested element".to_string()));

    browser.close().await.expect("Failed to close browser");
    server.shutdown();
}

#[tokio::test]
async fn test_locator_inner_text() {
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

    page.goto(&format!("{}/locator.html", server.url()), None)
        .await
        .expect("Failed to navigate");

    // Test: Get visible text
    let heading = page.locator("h1").await;
    let text = heading
        .inner_text()
        .await
        .expect("Failed to get inner text");

    assert_eq!(text, "Test Page");

    browser.close().await.expect("Failed to close browser");
    server.shutdown();
}

#[tokio::test]
async fn test_locator_is_visible() {
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

    page.goto(&format!("{}/locator.html", server.url()), None)
        .await
        .expect("Failed to navigate");

    // Test: Check visibility
    let heading = page.locator("h1").await;
    let visible = heading
        .is_visible()
        .await
        .expect("Failed to check visibility");

    assert!(visible);

    // Test: Hidden element should not be visible
    let hidden = page.locator("#hidden").await;
    let hidden_visible = hidden
        .is_visible()
        .await
        .expect("Failed to check visibility");

    assert!(!hidden_visible);

    browser.close().await.expect("Failed to close browser");
    server.shutdown();
}

// Cross-browser tests

#[tokio::test]
async fn test_locator_firefox() {
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

    page.goto(&format!("{}/locator.html", server.url()), None)
        .await
        .expect("Failed to navigate");

    // Test locator creation and text content
    let heading = page.locator("h1").await;
    let text = heading
        .text_content()
        .await
        .expect("Failed to get text content");

    assert_eq!(text, Some("Test Page".to_string()));

    browser.close().await.expect("Failed to close browser");
    server.shutdown();
}

#[tokio::test]
async fn test_locator_webkit() {
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

    page.goto(&format!("{}/locator.html", server.url()), None)
        .await
        .expect("Failed to navigate");

    // Test locator creation and visibility
    let heading = page.locator("h1").await;
    let visible = heading
        .is_visible()
        .await
        .expect("Failed to check visibility");

    assert!(visible);

    browser.close().await.expect("Failed to close browser");
    server.shutdown();
}
