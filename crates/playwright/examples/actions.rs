// Actions example - Interacting with elements
//
// This example demonstrates:
// - Clicking elements
// - Double-clicking elements
// - Filling form inputs
// - Pressing keys
// - Basic element interactions
//
// Note: This is a smoke test showing the API.
// Full interaction testing requires custom test pages.

use playwright::Playwright;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🎭 Playwright Actions Example\n");

    // Launch Playwright
    let playwright = Playwright::launch().await?;
    let browser = playwright.chromium().launch().await?;
    let page = browser.new_page().await?;

    // Navigate to a page
    println!("🔗 Navigating to example.com...");
    page.goto("https://example.com", None).await?;
    println!("✅ Page loaded\\n");

    // Click action
    println!("🖱️  Testing click action:");
    let heading = page.locator("h1").await;
    heading.click(None).await?;
    println!("   • Click succeeded on heading");

    // Double-click action
    println!("\\n🖱️🖱️  Testing double-click action:");
    heading.dblclick(None).await?;
    println!("   • Double-click succeeded on heading");

    // Note: fill(), clear(), and press() require input elements
    // Full testing with forms will be added in future examples

    // Cleanup
    println!("\\n🧹 Cleaning up...");
    page.close().await?;
    browser.close().await?;

    println!("\\n🎉 Example complete!");

    Ok(())
}
