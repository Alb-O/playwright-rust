// Integration tests for BrowserType::launch()
//
// These tests verify that we can launch real browsers using the Playwright server.

use playwright_core::api::LaunchOptions;
use playwright_core::protocol::Playwright;

#[tokio::test]
async fn test_launch_chromium() {
    // Launch Playwright
    let playwright = Playwright::launch()
        .await
        .expect("Failed to launch Playwright");

    // Get chromium browser type
    let chromium = playwright.chromium();

    // Launch browser with default options
    let browser = chromium.launch().await.expect("Failed to launch Chromium");

    // Verify browser was created
    assert_eq!(browser.name(), "chromium");
    assert!(!browser.version().is_empty());

    println!("Launched Chromium version: {}", browser.version());
}

#[tokio::test]
async fn test_launch_with_headless_option() {
    let playwright = Playwright::launch()
        .await
        .expect("Failed to launch Playwright");

    let chromium = playwright.chromium();

    // Launch with explicit headless option
    let options = LaunchOptions::default().headless(true);

    let browser = chromium
        .launch_with_options(options)
        .await
        .expect("Failed to launch Chromium with options");

    assert_eq!(browser.name(), "chromium");
    assert!(!browser.version().is_empty());
}

#[tokio::test]
async fn test_launch_all_three_browsers() {
    let playwright = Playwright::launch()
        .await
        .expect("Failed to launch Playwright");

    // Test Chromium
    let chromium = playwright.chromium();
    let chromium_browser = chromium.launch().await.expect("Failed to launch Chromium");
    assert_eq!(chromium_browser.name(), "chromium");
    println!("✓ Chromium: {}", chromium_browser.version());

    // Test Firefox
    let firefox = playwright.firefox();
    let firefox_browser = firefox.launch().await.expect("Failed to launch Firefox");
    assert_eq!(firefox_browser.name(), "firefox");
    println!("✓ Firefox: {}", firefox_browser.version());

    // Test WebKit
    let webkit = playwright.webkit();
    let webkit_browser = webkit.launch().await.expect("Failed to launch WebKit");
    assert_eq!(webkit_browser.name(), "webkit");
    println!("✓ WebKit: {}", webkit_browser.version());
}
