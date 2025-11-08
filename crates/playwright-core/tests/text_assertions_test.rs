// Integration tests for Text Assertions (Phase 5, Slice 2)
//
// Following TDD: Write tests first (Red), then implement (Green)
//
// Tests cover:
// - expect().to_have_text() - exact text match
// - expect().to_contain_text() - substring match
// - expect().to_have_value() - input value match
// - Regex pattern support for all
// - Auto-retry behavior
// - Cross-browser compatibility

mod test_server;

use playwright_core::{expect, protocol::Playwright};
use test_server::TestServer;

#[tokio::test]
async fn test_to_have_text_exact_match() {
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

    page.goto(&format!("{}/text.html", server.url()), None)
        .await
        .expect("Failed to navigate");

    // Test: Element with exact text should pass
    let heading = page.locator("h1").await;
    expect(heading)
        .to_have_text("Welcome to Playwright")
        .await
        .expect("Heading should have exact text");

    browser.close().await.expect("Failed to close browser");
    server.shutdown();
}

#[tokio::test]
async fn test_to_have_text_with_trimming() {
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

    page.goto(&format!("{}/text.html", server.url()), None)
        .await
        .expect("Failed to navigate");

    // Test: Text with whitespace should be trimmed
    let paragraph = page.locator("#whitespace").await;
    expect(paragraph)
        .to_have_text("Text with whitespace")
        .await
        .expect("Should match trimmed text");

    browser.close().await.expect("Failed to close browser");
    server.shutdown();
}

#[tokio::test]
async fn test_to_have_text_failure() {
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

    page.goto(&format!("{}/text.html", server.url()), None)
        .await
        .expect("Failed to navigate");

    // Test: Wrong text should timeout
    let heading = page.locator("h1").await;
    let result = expect(heading)
        .with_timeout(std::time::Duration::from_millis(500))
        .to_have_text("Wrong Text")
        .await;

    assert!(result.is_err(), "Should fail for wrong text");

    browser.close().await.expect("Failed to close browser");
    server.shutdown();
}

#[tokio::test]
async fn test_to_contain_text_substring() {
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

    page.goto(&format!("{}/text.html", server.url()), None)
        .await
        .expect("Failed to navigate");

    // Test: Substring should match
    let paragraph = page.locator("#long-text").await;
    expect(paragraph)
        .to_contain_text("middle of the text")
        .await
        .expect("Should contain substring");

    browser.close().await.expect("Failed to close browser");
    server.shutdown();
}

#[tokio::test]
async fn test_to_contain_text_not_present() {
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

    page.goto(&format!("{}/text.html", server.url()), None)
        .await
        .expect("Failed to navigate");

    // Test: Non-existent substring should fail
    let paragraph = page.locator("#long-text").await;
    let result = expect(paragraph)
        .with_timeout(std::time::Duration::from_millis(500))
        .to_contain_text("nonexistent text")
        .await;

    assert!(result.is_err(), "Should fail for missing substring");

    browser.close().await.expect("Failed to close browser");
    server.shutdown();
}

#[tokio::test]
async fn test_to_have_value_input() {
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

    page.goto(&format!("{}/text.html", server.url()), None)
        .await
        .expect("Failed to navigate");

    // Test: Input with value should match
    let input = page.locator("#name-input").await;
    expect(input)
        .to_have_value("John Doe")
        .await
        .expect("Input should have value");

    browser.close().await.expect("Failed to close browser");
    server.shutdown();
}

#[tokio::test]
async fn test_to_have_value_empty() {
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

    page.goto(&format!("{}/text.html", server.url()), None)
        .await
        .expect("Failed to navigate");

    // Test: Empty input should have empty value
    let input = page.locator("#empty-input").await;
    expect(input)
        .to_have_value("")
        .await
        .expect("Empty input should have empty value");

    browser.close().await.expect("Failed to close browser");
    server.shutdown();
}

#[tokio::test]
async fn test_to_have_text_with_regex() {
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

    page.goto(&format!("{}/text.html", server.url()), None)
        .await
        .expect("Failed to navigate");

    // Test: Regex pattern should match
    let heading = page.locator("h1").await;
    expect(heading)
        .to_have_text_regex(r"Welcome to .*")
        .await
        .expect("Should match regex pattern");

    browser.close().await.expect("Failed to close browser");
    server.shutdown();
}

#[tokio::test]
async fn test_to_contain_text_with_regex() {
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

    page.goto(&format!("{}/text.html", server.url()), None)
        .await
        .expect("Failed to navigate");

    // Test: Regex pattern for substring
    let paragraph = page.locator("#long-text").await;
    expect(paragraph)
        .to_contain_text_regex(r"middle of .* text")
        .await
        .expect("Should contain regex pattern");

    browser.close().await.expect("Failed to close browser");
    server.shutdown();
}

#[tokio::test]
async fn test_to_have_value_with_regex() {
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

    page.goto(&format!("{}/text.html", server.url()), None)
        .await
        .expect("Failed to navigate");

    // Test: Regex pattern for input value
    let input = page.locator("#name-input").await;
    expect(input)
        .to_have_value_regex(r"John .*")
        .await
        .expect("Should match value regex pattern");

    browser.close().await.expect("Failed to close browser");
    server.shutdown();
}

#[tokio::test]
async fn test_to_have_text_with_auto_retry() {
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

    // Inject element with text that changes after delay
    page.evaluate(
        r#"
        const div = document.createElement('div');
        div.id = 'changing-text';
        div.textContent = 'Initial text';
        document.body.appendChild(div);

        setTimeout(() => {
            div.textContent = 'Changed text';
        }, 100);
        "#,
    )
    .await
    .expect("Failed to inject script");

    // Test: Should wait for text to change
    let div = page.locator("#changing-text").await;
    let start = std::time::Instant::now();

    expect(div)
        .to_have_text("Changed text")
        .await
        .expect("Should eventually have changed text");

    let elapsed = start.elapsed();
    assert!(
        elapsed.as_millis() >= 100,
        "Should have waited at least 100ms"
    );

    browser.close().await.expect("Failed to close browser");
    server.shutdown();
}

#[tokio::test]
async fn test_to_have_value_with_auto_retry() {
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

    // Inject input with value that changes after delay
    page.evaluate(
        r#"
        const input = document.createElement('input');
        input.id = 'changing-input';
        input.value = 'initial';
        document.body.appendChild(input);

        setTimeout(() => {
            input.value = 'updated';
        }, 100);
        "#,
    )
    .await
    .expect("Failed to inject script");

    // Test: Should wait for value to change
    let input = page.locator("#changing-input").await;
    expect(input)
        .to_have_value("updated")
        .await
        .expect("Should eventually have updated value");

    browser.close().await.expect("Failed to close browser");
    server.shutdown();
}

// Cross-browser tests

#[tokio::test]
async fn test_to_have_text_firefox() {
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

    page.goto(&format!("{}/text.html", server.url()), None)
        .await
        .expect("Failed to navigate");

    let heading = page.locator("h1").await;
    expect(heading)
        .to_have_text("Welcome to Playwright")
        .await
        .expect("Should work in Firefox");

    browser.close().await.expect("Failed to close browser");
    server.shutdown();
}

#[tokio::test]
async fn test_to_contain_text_webkit() {
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

    page.goto(&format!("{}/text.html", server.url()), None)
        .await
        .expect("Failed to navigate");

    let paragraph = page.locator("#long-text").await;
    expect(paragraph)
        .to_contain_text("middle of the text")
        .await
        .expect("Should work in WebKit");

    browser.close().await.expect("Failed to close browser");
    server.shutdown();
}

#[tokio::test]
async fn test_to_have_value_webkit() {
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

    page.goto(&format!("{}/text.html", server.url()), None)
        .await
        .expect("Failed to navigate");

    let input = page.locator("#name-input").await;
    expect(input)
        .to_have_value("John Doe")
        .await
        .expect("Should work in WebKit");

    browser.close().await.expect("Failed to close browser");
    server.shutdown();
}
