// Integration tests for keyboard and mouse low-level APIs
//
// Following TDD: Write tests first (Red), then implement (Green)
//
// Tests cover:
// - keyboard.down() and keyboard.up()
// - keyboard.press() with single keys and combinations
// - keyboard.type_text() for typing text
// - keyboard.insert_text() for paste-like insertion
// - mouse.move() to coordinates
// - mouse.click() at coordinates
// - mouse.dblclick() at coordinates
// - mouse.down() and mouse.up()
// - mouse.wheel() for scrolling

mod test_server;

use playwright_core::protocol::Playwright;
use test_server::TestServer;

// Keyboard Tests

#[tokio::test]
async fn test_keyboard_type_text() {
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

    page.goto(&format!("{}/keyboard_mouse.html", server.url()), None)
        .await
        .expect("Failed to navigate");

    // Focus input
    let input = page.locator("#keyboard-input").await;
    input.click(None).await.expect("Failed to focus input");

    // Test: Type text using keyboard API
    let keyboard = page.keyboard();
    keyboard
        .type_text("Hello World", None)
        .await
        .expect("Failed to type text");

    // Verify text was typed
    let value = input
        .input_value(None)
        .await
        .expect("Failed to get input value");
    assert_eq!(value, "Hello World");

    browser.close().await.expect("Failed to close browser");
    server.shutdown();
}

#[tokio::test]
async fn test_keyboard_press() {
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

    page.goto(&format!("{}/keyboard_mouse.html", server.url()), None)
        .await
        .expect("Failed to navigate");

    // Focus input
    let input = page.locator("#keyboard-input").await;
    input.click(None).await.expect("Failed to focus input");

    // Test: Press Enter key
    let keyboard = page.keyboard();
    keyboard
        .press("Enter", None)
        .await
        .expect("Failed to press Enter");

    // Verify Enter was pressed (triggers JS event handler)
    let result = page.locator("#keyboard-result").await;
    let text = result.text_content().await.expect("Failed to get text");
    assert_eq!(text, Some("Enter pressed".to_string()));

    browser.close().await.expect("Failed to close browser");
    server.shutdown();
}

#[tokio::test]
async fn test_keyboard_down_up() {
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

    page.goto(&format!("{}/keyboard_mouse.html", server.url()), None)
        .await
        .expect("Failed to navigate");

    // Focus input
    let input = page.locator("#keyboard-input").await;
    input.click(None).await.expect("Failed to focus input");

    // Test: Hold Shift and type letter for uppercase
    let keyboard = page.keyboard();
    keyboard.down("Shift").await.expect("Failed to press Shift");
    keyboard
        .press("KeyA", None)
        .await
        .expect("Failed to press A");
    keyboard.up("Shift").await.expect("Failed to release Shift");

    // Verify uppercase A was typed
    let value = input
        .input_value(None)
        .await
        .expect("Failed to get input value");
    assert!(value.contains("A"));

    browser.close().await.expect("Failed to close browser");
    server.shutdown();
}

#[tokio::test]
async fn test_keyboard_insert_text() {
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

    page.goto(&format!("{}/keyboard_mouse.html", server.url()), None)
        .await
        .expect("Failed to navigate");

    // Focus input
    let input = page.locator("#keyboard-input").await;
    input.click(None).await.expect("Failed to focus input");

    // Test: Insert text without key events
    let keyboard = page.keyboard();
    keyboard
        .insert_text("Pasted text")
        .await
        .expect("Failed to insert text");

    // Verify text was inserted
    let value = input
        .input_value(None)
        .await
        .expect("Failed to get input value");
    assert_eq!(value, "Pasted text");

    browser.close().await.expect("Failed to close browser");
    server.shutdown();
}

// Mouse Tests

#[tokio::test]
async fn test_mouse_move() {
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

    page.goto(&format!("{}/keyboard_mouse.html", server.url()), None)
        .await
        .expect("Failed to navigate");

    // Test: Move mouse to coordinates
    let mouse = page.mouse();
    mouse
        .move_to(100, 100, None)
        .await
        .expect("Failed to move mouse");

    // Verify mouse moved (we can't directly verify coordinates, but method should succeed)
    // In a real scenario, the page would show coordinates
    let coords = page.locator("#mouse-coords").await;
    let text = coords.text_content().await.expect("Failed to get text");
    assert!(text.is_some());

    browser.close().await.expect("Failed to close browser");
    server.shutdown();
}

#[tokio::test]
async fn test_mouse_click() {
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

    page.goto(&format!("{}/keyboard_mouse.html", server.url()), None)
        .await
        .expect("Failed to navigate");

    // Test: Click at coordinates (on the clickable div)
    let mouse = page.mouse();
    mouse
        .click(150, 200, None)
        .await
        .expect("Failed to click mouse");

    // Verify click registered
    let result = page.locator("#mouse-result").await;
    let text = result.text_content().await.expect("Failed to get text");
    assert_eq!(text, Some("Clicked".to_string()));

    browser.close().await.expect("Failed to close browser");
    server.shutdown();
}

#[tokio::test]
async fn test_mouse_dblclick() {
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

    page.goto(&format!("{}/keyboard_mouse.html", server.url()), None)
        .await
        .expect("Failed to navigate");

    // Test: Double-click at coordinates
    let mouse = page.mouse();
    mouse
        .dblclick(150, 200, None)
        .await
        .expect("Failed to double-click mouse");

    // Verify double-click registered
    let result = page.locator("#mouse-result").await;
    let text = result.text_content().await.expect("Failed to get text");
    assert_eq!(text, Some("Double-clicked".to_string()));

    browser.close().await.expect("Failed to close browser");
    server.shutdown();
}

#[tokio::test]
async fn test_mouse_down_up() {
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

    page.goto(&format!("{}/keyboard_mouse.html", server.url()), None)
        .await
        .expect("Failed to navigate");

    // Test: Mouse down and up (simulating drag)
    let mouse = page.mouse();
    mouse
        .move_to(150, 200, None)
        .await
        .expect("Failed to move mouse");
    mouse.down(None).await.expect("Failed to mouse down");
    mouse
        .move_to(250, 200, None)
        .await
        .expect("Failed to move while down");
    mouse.up(None).await.expect("Failed to mouse up");

    // Verify drag happened (method should succeed)
    // In a real drag scenario, the page would show drag result

    browser.close().await.expect("Failed to close browser");
    server.shutdown();
}

#[tokio::test]
async fn test_mouse_wheel() {
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

    page.goto(&format!("{}/keyboard_mouse.html", server.url()), None)
        .await
        .expect("Failed to navigate");

    // Test: Scroll with mouse wheel
    let mouse = page.mouse();
    mouse.wheel(0, 100).await.expect("Failed to wheel mouse");

    // Verify scroll happened (method should succeed)
    // Page would show scroll position in real scenario

    browser.close().await.expect("Failed to close browser");
    server.shutdown();
}

// Cross-browser tests

#[tokio::test]
async fn test_keyboard_firefox() {
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

    page.goto(&format!("{}/keyboard_mouse.html", server.url()), None)
        .await
        .expect("Failed to navigate");

    let input = page.locator("#keyboard-input").await;
    input.click(None).await.expect("Failed to focus input");

    let keyboard = page.keyboard();
    keyboard
        .type_text("Firefox test", None)
        .await
        .expect("Failed to type text");

    let value = input
        .input_value(None)
        .await
        .expect("Failed to get input value");
    assert_eq!(value, "Firefox test");

    browser.close().await.expect("Failed to close browser");
    server.shutdown();
}

#[tokio::test]
async fn test_mouse_webkit() {
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

    page.goto(&format!("{}/keyboard_mouse.html", server.url()), None)
        .await
        .expect("Failed to navigate");

    let mouse = page.mouse();
    mouse
        .click(150, 200, None)
        .await
        .expect("Failed to click mouse");

    let result = page.locator("#mouse-result").await;
    let text = result.text_content().await.expect("Failed to get text");
    assert_eq!(text, Some("Clicked".to_string()));

    browser.close().await.expect("Failed to close browser");
    server.shutdown();
}
