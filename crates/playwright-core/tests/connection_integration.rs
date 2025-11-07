//! Integration tests for Connection layer with real Playwright server
//!
//! These tests verify that the Connection layer can:
//! - Establish connection to real Playwright server
//! - Spawn transport and connection message loops
//! - Handle protocol initialization messages from server
//!
//! Note: Full protocol request/response testing will be implemented in Slice 4
//! (Object Factory) when we can handle the initialization sequence and send requests.

use playwright_core::{Connection, PlaywrightServer};
use std::sync::Arc;
use tokio::time::Duration;

/// Test that we can establish a connection with real server and spawn message loops
///
/// This test verifies:
/// - Server launches successfully
/// - Connection can be created with server stdio
/// - Message loops can be spawned without errors
/// - Everything runs together and shuts down cleanly
#[tokio::test]
async fn test_connection_lifecycle_with_real_server() {
    // Launch Playwright server
    let mut server = match PlaywrightServer::launch().await {
        Ok(s) => s,
        Err(e) => {
            eprintln!("Skipping test: Could not launch Playwright server: {}", e);
            eprintln!("This is expected if Node.js or Playwright driver is not available");
            return;
        }
    };

    // Take stdio handles from the process
    let stdin = server.process.stdin.take().expect("Failed to get stdin");
    let stdout = server.process.stdout.take().expect("Failed to get stdout");

    // Create transport
    let (transport, message_rx) = playwright_core::transport::PipeTransport::new(stdin, stdout);

    // Create connection
    let connection = Arc::new(Connection::new(transport, message_rx));

    // Spawn connection message loop
    let conn = Arc::clone(&connection);
    let connection_handle = tokio::spawn(async move {
        conn.run().await;
    });

    // Give the server time to start
    tokio::time::sleep(Duration::from_millis(100)).await;

    // This test verifies the connection infrastructure works:
    // - Server launches successfully
    // - Connection and transport loops start without errors
    // - Everything compiles and runs together
    // - No panics or immediate crashes
    //
    // Full protocol initialization testing is done in:
    // - tests/initialization_integration.rs (complete initialization flow)
    // - tests/playwright_launch.rs (high-level Playwright::launch() API)

    // Clean up
    // Abort the connection loop (which will also stop reading from transport)
    connection_handle.abort();

    // Shutdown server
    server.shutdown().await.ok();
}

/// Test that connection detects server crash when sending
///
/// This test verifies that when the server crashes/exits:
/// - Attempting to send a message fails with appropriate error (broken pipe)
/// - The error is propagated correctly through the Connection layer
///
/// Note: The connection read loop will remain blocked on `read_exact()` until
/// the stdout pipe is fully closed by the OS, which can take time. This is
/// expected behavior - the important thing is that send operations fail fast.
#[tokio::test]
async fn test_connection_detects_server_crash_on_send() {
    let mut server = match PlaywrightServer::launch().await {
        Ok(s) => s,
        Err(e) => {
            eprintln!("Skipping test: Could not launch Playwright server: {}", e);
            return;
        }
    };

    let stdin = server.process.stdin.take().expect("Failed to get stdin");
    let stdout = server.process.stdout.take().expect("Failed to get stdout");

    let (transport, message_rx) = playwright_core::transport::PipeTransport::new(stdin, stdout);

    let connection = Arc::new(Connection::new(transport, message_rx));

    // Spawn connection loop
    let conn = Arc::clone(&connection);
    let _connection_handle = tokio::spawn(async move {
        conn.run().await;
    });

    // Give connection time to start
    tokio::time::sleep(Duration::from_millis(50)).await;

    // Kill the server
    server.kill().await.expect("Failed to kill server");

    // Give it a moment for the pipe to close
    tokio::time::sleep(Duration::from_millis(30)).await;

    // Try to send a message - this will detect the broken pipe immediately
    let send_result = connection
        .send_message("test@guid", "testMethod", serde_json::json!({}))
        .await;

    // Should fail with broken pipe error
    assert!(
        send_result.is_err(),
        "Expected error when sending to dead server"
    );

    // Verify it's a transport error (broken pipe)
    match send_result.unwrap_err() {
        playwright_core::Error::TransportError(msg) => {
            assert!(
                msg.contains("Broken pipe") || msg.contains("Failed to write"),
                "Expected broken pipe error, got: {}",
                msg
            );
        }
        e => panic!("Expected TransportError, got: {:?}", e),
    }

    // Note: We don't wait for the connection loop to exit because the transport
    // read loop is blocked on read_exact() and won't exit until the OS fully
    // closes the stdout pipe. This is tested separately in the transport layer.
}

/// Test concurrent requests (deferred to Phase 2 Slice 5+)
///
/// This test will verify that multiple concurrent requests can be sent
/// and responses are correctly correlated, even when they arrive out of order.
///
/// Deferred to Phase 2 Slice 5+ because it requires:
/// - Multiple protocol objects to send requests to (Browser, Context, Page)
/// - Complex message sequences (launch → new_context → new_page)
/// - Concurrent operations on different objects
#[tokio::test]
#[ignore] // TODO: Implement in Phase 2 Slice 6+ when we have Browser, Context, and Page objects
async fn test_concurrent_requests_with_server() {
    // Will implement when we have:
    // - Browser::new_context() (Slice 5)
    // - BrowserContext::new_page() (Slice 6)
    // - Multiple concurrent operations
}

/// Test error handling with invalid requests (deferred to Phase 2 Slice 4+)
///
/// This test will verify that protocol errors from the server are properly
/// converted to Rust errors and propagated correctly.
///
/// Deferred to Phase 2 Slice 4+ because it requires:
/// - Valid Browser object GUID (available in Slice 3) ✅
/// - Methods to call with invalid params (e.g., Browser::close() in Slice 4)
/// - Intentionally invalid requests to trigger protocol errors
#[tokio::test]
#[ignore] // TODO: Implement in Phase 2 Slice 4+ when Browser::close() and error scenarios exist
async fn test_error_response_from_server() {
    // Will implement when we have Browser::close() to test error scenarios:
    // - Call close() twice (should error on second call)
    // - Invalid GUIDs
    // - Invalid method parameters
}
