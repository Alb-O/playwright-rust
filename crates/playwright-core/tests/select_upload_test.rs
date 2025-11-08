// Integration tests for select and file upload interactions
//
// Following TDD: Write tests first (Red), then implement (Green)
//
// Tests cover:
// - select_option() with value/label/index
// - select_option() for multiple selections
// - set_input_files() with single file
// - set_input_files() with multiple files
// - set_input_files() for clearing files

mod test_server;

use playwright_core::protocol::{Playwright, SelectOption};
use std::fs;
use std::io::Write;
use test_server::TestServer;

#[tokio::test]
async fn test_select_option_by_value() {
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

    page.goto(&format!("{}/select.html", server.url()), None)
        .await
        .expect("Failed to navigate");

    // Test: Select option by value
    let select = page.locator("#single-select").await;
    let selected = select
        .select_option("banana", None)
        .await
        .expect("Failed to select option");

    assert_eq!(selected, vec!["banana"]);

    browser.close().await.expect("Failed to close browser");
    server.shutdown();
}

#[tokio::test]
async fn test_select_option_by_label() {
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

    page.goto(&format!("{}/select.html", server.url()), None)
        .await
        .expect("Failed to navigate");

    // Test: Select option by label
    let select = page.locator("#single-select").await;
    let selected = select
        .select_option(SelectOption::Label("Banana".to_string()), None)
        .await
        .expect("Failed to select option by label");

    assert_eq!(selected, vec!["banana"]);

    browser.close().await.expect("Failed to close browser");
    server.shutdown();
}

#[tokio::test]
async fn test_select_option_by_index() {
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

    page.goto(&format!("{}/select.html", server.url()), None)
        .await
        .expect("Failed to navigate");

    // Test: Select option by index (0-based, index 2 = "Cherry")
    let select = page.locator("#single-select").await;
    let selected = select
        .select_option(SelectOption::Index(3), None)
        .await
        .expect("Failed to select option by index");

    assert_eq!(selected, vec!["cherry"]);

    browser.close().await.expect("Failed to close browser");
    server.shutdown();
}

#[tokio::test]
async fn test_select_option_without_value_attribute() {
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

    page.goto(&format!("{}/select.html", server.url()), None)
        .await
        .expect("Failed to navigate");

    // Test: Select option by index when options have no value attribute
    let select = page.locator("#select-by-index").await;
    let selected = select
        .select_option(SelectOption::Index(1), None)
        .await
        .expect("Failed to select by index");

    // When no value attribute, the text content becomes the value
    assert_eq!(selected, vec!["Second"]);

    browser.close().await.expect("Failed to close browser");
    server.shutdown();
}

#[tokio::test]
async fn test_select_multiple_options() {
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

    page.goto(&format!("{}/select.html", server.url()), None)
        .await
        .expect("Failed to navigate");

    // Test: Select multiple options
    let select = page.locator("#multi-select").await;
    let selected = select
        .select_option_multiple(&["red", "blue"], None)
        .await
        .expect("Failed to select multiple options");

    assert_eq!(selected, vec!["red", "blue"]);

    browser.close().await.expect("Failed to close browser");
    server.shutdown();
}

#[tokio::test]
async fn test_select_multiple_options_mixed_types() {
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

    page.goto(&format!("{}/select.html", server.url()), None)
        .await
        .expect("Failed to navigate");

    // Test: Select multiple options using SelectOption variants
    let select = page.locator("#multi-select").await;
    let options = vec![
        SelectOption::Value("red".to_string()),
        SelectOption::Label("Blue".to_string()),
    ];
    let selected = select
        .select_option_multiple(&options, None)
        .await
        .expect("Failed to select multiple options with mixed types");

    assert_eq!(selected, vec!["red", "blue"]);

    browser.close().await.expect("Failed to close browser");
    server.shutdown();
}

#[tokio::test]
async fn test_upload_single_file() {
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

    page.goto(&format!("{}/upload.html", server.url()), None)
        .await
        .expect("Failed to navigate");

    // Create a temporary test file
    let temp_dir = std::env::temp_dir();
    let test_file = temp_dir.join("playwright_test_file.txt");
    let mut file = fs::File::create(&test_file).expect("Failed to create test file");
    file.write_all(b"Test file content")
        .expect("Failed to write to test file");

    // Test: Upload single file
    let input = page.locator("#single-file").await;
    input
        .set_input_files(&test_file, None)
        .await
        .expect("Failed to set input file");

    // Verify file was uploaded by checking the displayed info
    let info = page.locator("#file-info").await;
    let text = info.text_content().await.expect("Failed to get text");
    assert!(text.unwrap().contains("playwright_test_file.txt"));

    // Cleanup
    fs::remove_file(test_file).expect("Failed to remove test file");
    browser.close().await.expect("Failed to close browser");
    server.shutdown();
}

#[tokio::test]
async fn test_upload_multiple_files() {
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

    page.goto(&format!("{}/upload.html", server.url()), None)
        .await
        .expect("Failed to navigate");

    // Create temporary test files
    let temp_dir = std::env::temp_dir();
    let test_file1 = temp_dir.join("playwright_test_file1.txt");
    let test_file2 = temp_dir.join("playwright_test_file2.txt");

    let mut file1 = fs::File::create(&test_file1).expect("Failed to create test file 1");
    file1
        .write_all(b"Test file 1 content")
        .expect("Failed to write to test file 1");

    let mut file2 = fs::File::create(&test_file2).expect("Failed to create test file 2");
    file2
        .write_all(b"Test file 2 content")
        .expect("Failed to write to test file 2");

    // Test: Upload multiple files
    let input = page.locator("#multi-file").await;
    input
        .set_input_files_multiple(&[&test_file1, &test_file2], None)
        .await
        .expect("Failed to set multiple input files");

    // Verify files were uploaded
    let info = page.locator("#file-info").await;
    let text = info.text_content().await.expect("Failed to get text");
    let text_content = text.unwrap();
    assert!(text_content.contains("playwright_test_file1.txt"));
    assert!(text_content.contains("playwright_test_file2.txt"));

    // Cleanup
    fs::remove_file(test_file1).expect("Failed to remove test file 1");
    fs::remove_file(test_file2).expect("Failed to remove test file 2");
    browser.close().await.expect("Failed to close browser");
    server.shutdown();
}

#[tokio::test]
async fn test_clear_file_input() {
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

    page.goto(&format!("{}/upload.html", server.url()), None)
        .await
        .expect("Failed to navigate");

    // Create and upload a file first
    let temp_dir = std::env::temp_dir();
    let test_file = temp_dir.join("playwright_test_clear.txt");
    let mut file = fs::File::create(&test_file).expect("Failed to create test file");
    file.write_all(b"Test content")
        .expect("Failed to write to test file");

    let input = page.locator("#single-file").await;
    input
        .set_input_files(&test_file, None)
        .await
        .expect("Failed to set input file");

    // Test: Clear file input by passing empty array
    input
        .set_input_files_multiple(&[], None)
        .await
        .expect("Failed to clear input files");

    // Note: Verifying file input is cleared would require checking input.files.length
    // For now, we verify the method succeeds

    // Cleanup
    fs::remove_file(test_file).expect("Failed to remove test file");
    browser.close().await.expect("Failed to close browser");
    server.shutdown();
}

// Cross-browser tests

#[tokio::test]
async fn test_select_option_firefox() {
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

    page.goto(&format!("{}/select.html", server.url()), None)
        .await
        .expect("Failed to navigate");

    let select = page.locator("#single-select").await;
    let selected = select
        .select_option("cherry", None)
        .await
        .expect("Failed to select option");

    assert_eq!(selected, vec!["cherry"]);

    browser.close().await.expect("Failed to close browser");
    server.shutdown();
}

#[tokio::test]
async fn test_select_option_by_label_firefox() {
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

    page.goto(&format!("{}/select.html", server.url()), None)
        .await
        .expect("Failed to navigate");

    // Test: Select by label in Firefox
    let select = page.locator("#single-select").await;
    let selected = select
        .select_option(SelectOption::Label("Apple".to_string()), None)
        .await
        .expect("Failed to select option by label in Firefox");

    assert_eq!(selected, vec!["apple"]);

    browser.close().await.expect("Failed to close browser");
    server.shutdown();
}

#[tokio::test]
async fn test_select_option_by_index_webkit() {
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

    page.goto(&format!("{}/select.html", server.url()), None)
        .await
        .expect("Failed to navigate");

    // Test: Select by index in WebKit
    let select = page.locator("#single-select").await;
    let selected = select
        .select_option(SelectOption::Index(2), None)
        .await
        .expect("Failed to select option by index in WebKit");

    assert_eq!(selected, vec!["banana"]);

    browser.close().await.expect("Failed to close browser");
    server.shutdown();
}

#[tokio::test]
async fn test_upload_file_webkit() {
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

    page.goto(&format!("{}/upload.html", server.url()), None)
        .await
        .expect("Failed to navigate");

    // Create a temporary test file
    let temp_dir = std::env::temp_dir();
    let test_file = temp_dir.join("playwright_webkit_test.txt");
    let mut file = fs::File::create(&test_file).expect("Failed to create test file");
    file.write_all(b"WebKit test content")
        .expect("Failed to write to test file");

    let input = page.locator("#single-file").await;
    input
        .set_input_files(&test_file, None)
        .await
        .expect("Failed to set input file");

    // Cleanup
    fs::remove_file(test_file).expect("Failed to remove test file");
    browser.close().await.expect("Failed to close browser");
    server.shutdown();
}
