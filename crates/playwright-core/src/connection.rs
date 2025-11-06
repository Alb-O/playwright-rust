//! JSON-RPC Connection layer for Playwright protocol
//!
//! This module implements the request/response correlation layer on top of the transport.
//! It handles:
//! - Generating unique request IDs
//! - Correlating responses with pending requests
//! - Distinguishing events from responses
//! - Dispatching events to protocol objects
//!
//! # Message Flow
//!
//! 1. Client calls `send_message()` with GUID, method, and params
//! 2. Connection generates unique ID and creates oneshot channel
//! 3. Request is serialized and sent via transport
//! 4. Client awaits on the oneshot receiver
//! 5. Message loop receives response from transport
//! 6. Response is correlated by ID and sent via oneshot channel
//! 7. Client receives result
//!
//! # Example
//!
//! ```no_run
//! # use playwright_core::connection::Connection;
//! # use playwright_core::transport::PipeTransport;
//! # use serde_json::json;
//! # #[tokio::main]
//! # async fn main() -> Result<(), Box<dyn std::error::Error>> {
//! // Create transport (after launching server)
//! // let (transport, message_rx) = PipeTransport::new(stdin, stdout);
//!
//! // Create connection
//! // let connection = Connection::new(transport, message_rx);
//!
//! // Spawn message loop in background
//! // let conn = connection.clone();
//! // tokio::spawn(async move {
//! //     conn.run().await;
//! // });
//!
//! // Send request and await response
//! // let result = connection.send_message(
//! //     "page@abc123",
//! //     "goto",
//! //     json!({"url": "https://example.com"})
//! // ).await?;
//! # Ok(())
//! # }
//! ```
//!
//! # References
//!
//! Based on research of official Playwright bindings:
//! - Python: `playwright/_impl/_connection.py`
//! - Java: `com/microsoft/playwright/impl/Connection.java`
//! - .NET: `Microsoft.Playwright/Core/Connection.cs`

use crate::error::{Error, Result};
use crate::transport::{PipeTransport, Transport};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashMap;
use std::sync::atomic::{AtomicU32, Ordering};
use std::sync::Arc;
use tokio::sync::{mpsc, oneshot, Mutex};

/// Protocol request message sent to Playwright server
///
/// Format matches Playwright's JSON-RPC protocol:
/// ```json
/// {
///   "id": 42,
///   "guid": "page@3ee5e10621a15eaf80cb985dbccb9a28",
///   "method": "goto",
///   "params": {
///     "url": "https://example.com"
///   }
/// }
/// ```
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Request {
    /// Unique request ID for correlating responses
    pub id: u32,
    /// GUID of the target object (format: "type@hash")
    pub guid: String,
    /// Method name to invoke
    pub method: String,
    /// Method parameters as JSON object
    pub params: Value,
}

/// Protocol response message from Playwright server
///
/// Format matches Playwright's JSON-RPC protocol:
/// ```json
/// {
///   "id": 42,
///   "result": { "response": { "guid": "response@..." } }
/// }
/// ```
///
/// Or with error:
/// ```json
/// {
///   "id": 42,
///   "error": {
///     "error": {
///       "message": "Navigation timeout",
///       "name": "TimeoutError",
///       "stack": "..."
///     }
///   }
/// }
/// ```
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Response {
    /// Request ID this response correlates to
    pub id: u32,
    /// Success result (mutually exclusive with error)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub result: Option<Value>,
    /// Error result (mutually exclusive with result)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<ErrorWrapper>,
}

/// Wrapper for protocol error payload
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ErrorWrapper {
    pub error: ErrorPayload,
}

/// Protocol error details
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ErrorPayload {
    /// Error message
    pub message: String,
    /// Error type name (e.g., "TimeoutError", "TargetClosedError")
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    /// Stack trace
    #[serde(skip_serializing_if = "Option::is_none")]
    pub stack: Option<String>,
}

/// Protocol event message from Playwright server
///
/// Events are distinguished from responses by the absence of an `id` field:
/// ```json
/// {
///   "guid": "page@3ee5e10621a15eaf80cb985dbccb9a28",
///   "method": "console",
///   "params": {
///     "message": { "type": "log", "text": "Hello world" }
///   }
/// }
/// ```
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Event {
    /// GUID of the object that emitted the event
    pub guid: String,
    /// Event method name
    pub method: String,
    /// Event parameters as JSON object
    pub params: Value,
}

/// Discriminated union of protocol messages
///
/// Uses serde's `untagged` to distinguish based on presence of `id` field:
/// - Messages with `id` are responses
/// - Messages without `id` are events
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum Message {
    /// Response message (has `id` field)
    Response(Response),
    /// Event message (no `id` field)
    Event(Event),
}

/// JSON-RPC connection to Playwright server
///
/// Manages request/response correlation and event dispatch.
/// Uses sequential request IDs and oneshot channels for correlation.
///
/// # Thread Safety
///
/// Connection is thread-safe and can be shared across async tasks using `Arc`.
/// Multiple concurrent requests are supported.
///
/// # Architecture
///
/// This follows the pattern from official Playwright bindings:
/// - Python: Direct callback on message receive
/// - Java: Callback map with synchronized access
/// - .NET: ConcurrentDictionary with TaskCompletionSource
///
/// Rust implementation uses:
/// - `AtomicU32` for thread-safe ID generation
/// - `Arc<Mutex<HashMap>>` for callback storage
/// - `tokio::sync::oneshot` for request/response correlation
pub struct Connection<W, R>
where
    W: tokio::io::AsyncWrite + Unpin + Send + Sync + 'static,
    R: tokio::io::AsyncRead + Unpin + Send + Sync + 'static,
{
    /// Sequential request ID counter (atomic for thread safety)
    last_id: AtomicU32,
    /// Pending request callbacks keyed by request ID
    callbacks: Arc<Mutex<HashMap<u32, oneshot::Sender<Result<Value>>>>>,
    /// Transport layer for sending/receiving messages
    transport: Arc<Mutex<PipeTransport<W, R>>>,
    /// Receiver for incoming messages from transport
    message_rx: Arc<Mutex<Option<mpsc::UnboundedReceiver<Value>>>>,
}

impl<W, R> Connection<W, R>
where
    W: tokio::io::AsyncWrite + Unpin + Send + Sync + 'static,
    R: tokio::io::AsyncRead + Unpin + Send + Sync + 'static,
{
    /// Create a new Connection with the given transport
    ///
    /// # Arguments
    ///
    /// * `transport` - Transport connected to Playwright server
    /// * `message_rx` - Receiver for incoming messages from transport
    ///
    /// # Example
    ///
    /// ```no_run
    /// # use playwright_core::connection::Connection;
    /// # use playwright_core::transport::PipeTransport;
    /// # use tokio::io::duplex;
    /// # #[tokio::main]
    /// # async fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// let (stdin_read, stdin_write) = duplex(1024);
    /// let (stdout_read, stdout_write) = duplex(1024);
    ///
    /// let (transport, message_rx) = PipeTransport::new(stdin_write, stdout_read);
    /// let connection = Connection::new(transport, message_rx);
    /// # Ok(())
    /// # }
    /// ```
    pub fn new(transport: PipeTransport<W, R>, message_rx: mpsc::UnboundedReceiver<Value>) -> Self {
        Self {
            last_id: AtomicU32::new(0),
            callbacks: Arc::new(Mutex::new(HashMap::new())),
            transport: Arc::new(Mutex::new(transport)),
            message_rx: Arc::new(Mutex::new(Some(message_rx))),
        }
    }

    /// Send a message to the Playwright server and await response
    ///
    /// This method:
    /// 1. Generates a unique request ID
    /// 2. Creates a oneshot channel for the response
    /// 3. Stores the channel sender in the callbacks map
    /// 4. Serializes and sends the request via transport
    /// 5. Awaits the response on the receiver
    ///
    /// # Arguments
    ///
    /// * `guid` - GUID of the target object (e.g., "page@abc123")
    /// * `method` - Method name to invoke (e.g., "goto")
    /// * `params` - Method parameters as JSON value
    ///
    /// # Returns
    ///
    /// The result value from the server, or an error if:
    /// - Transport send fails
    /// - Server returns an error
    /// - Connection is closed before response arrives
    ///
    /// # Example
    ///
    /// ```no_run
    /// # use playwright_core::connection::Connection;
    /// # use playwright_core::transport::PipeTransport;
    /// # use serde_json::json;
    /// # use tokio::io::duplex;
    /// # #[tokio::main]
    /// # async fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// # let (stdin_read, stdin_write) = duplex(1024);
    /// # let (stdout_read, stdout_write) = duplex(1024);
    /// # let (transport, message_rx) = PipeTransport::new(stdin_write, stdout_read);
    /// # let connection = Connection::new(transport, message_rx);
    /// let result = connection.send_message(
    ///     "page@abc123",
    ///     "goto",
    ///     json!({"url": "https://example.com"})
    /// ).await?;
    /// # Ok(())
    /// # }
    /// ```
    pub async fn send_message(&self, guid: &str, method: &str, params: Value) -> Result<Value> {
        // Generate unique ID (atomic increment for thread safety)
        let id = self.last_id.fetch_add(1, Ordering::SeqCst);

        // Create oneshot channel for response
        let (tx, rx) = oneshot::channel();

        // Store callback
        self.callbacks.lock().await.insert(id, tx);

        // Build request
        let request = Request {
            id,
            guid: guid.to_string(),
            method: method.to_string(),
            params,
        };

        // Send via transport
        let request_value = serde_json::to_value(&request)?;
        self.transport.lock().await.send(request_value).await?;

        // Await response
        rx.await
            .map_err(|_| Error::ChannelClosed)
            .and_then(|result| result)
    }

    /// Run the message dispatch loop
    ///
    /// This method continuously reads messages from the transport and dispatches them:
    /// - Responses (with `id`) are correlated with pending requests
    /// - Events (without `id`) are dispatched to protocol objects (TODO: Slice 4)
    ///
    /// The loop runs until the transport channel is closed.
    ///
    /// # Usage
    ///
    /// This method should be spawned in a background task:
    ///
    /// ```no_run
    /// # use playwright_core::connection::Connection;
    /// # use playwright_core::transport::PipeTransport;
    /// # use std::sync::Arc;
    /// # use tokio::io::duplex;
    /// # #[tokio::main]
    /// # async fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// # let (stdin_read, stdin_write) = duplex(1024);
    /// # let (stdout_read, stdout_write) = duplex(1024);
    /// # let (transport, message_rx) = PipeTransport::new(stdin_write, stdout_read);
    /// # let connection = Arc::new(Connection::new(transport, message_rx));
    /// let conn = Arc::clone(&connection);
    /// tokio::spawn(async move {
    ///     conn.run().await;
    /// });
    /// # Ok(())
    /// # }
    /// ```
    pub async fn run(&self) {
        // Spawn transport read loop
        let transport = Arc::clone(&self.transport);
        let transport_handle = tokio::spawn(async move {
            let mut transport = transport.lock().await;
            if let Err(e) = transport.run().await {
                tracing::error!("Transport error: {}", e);
            }
        });

        // Take the receiver out of the Option (can only be called once)
        let mut message_rx = self
            .message_rx
            .lock()
            .await
            .take()
            .expect("run() can only be called once");

        while let Some(message_value) = message_rx.recv().await {
            // Parse message as Response or Event
            match serde_json::from_value::<Message>(message_value.clone()) {
                Ok(message) => {
                    if let Err(e) = self.dispatch(message).await {
                        tracing::error!("Error dispatching message: {}", e);
                    }
                }
                Err(e) => {
                    tracing::error!(
                        "Failed to parse message: {} - message: {}",
                        e,
                        message_value
                    );
                }
            }
        }

        tracing::debug!("Message loop ended (transport closed)");

        // Wait for transport task to finish
        let _ = transport_handle.await;
    }

    /// Dispatch an incoming message from the transport
    ///
    /// This method:
    /// - Parses the message as Response or Event
    /// - For responses: correlates by ID and completes the oneshot channel
    /// - For events: dispatches to the appropriate object (TODO: Slice 4)
    ///
    /// # Arguments
    ///
    /// * `message` - Parsed protocol message
    ///
    /// # Errors
    ///
    /// Returns error if:
    /// - Response ID doesn't match any pending request
    /// - Event GUID doesn't match any registered object
    async fn dispatch(&self, message: Message) -> Result<()> {
        match message {
            Message::Response(response) => {
                // Correlate response with pending request
                let callback = self
                    .callbacks
                    .lock()
                    .await
                    .remove(&response.id)
                    .ok_or_else(|| {
                        Error::ProtocolError(format!(
                            "Cannot find request to respond: id={}",
                            response.id
                        ))
                    })?;

                // Convert protocol error to Rust error
                let result = if let Some(error_wrapper) = response.error {
                    Err(parse_protocol_error(error_wrapper.error))
                } else {
                    Ok(response.result.unwrap_or(Value::Null))
                };

                // Complete the oneshot channel (ignore if receiver was dropped)
                let _ = callback.send(result);
                Ok(())
            }
            Message::Event(event) => {
                // TODO: Implement event dispatch in Slice 4 (Object Factory)
                // For now, just log events
                tracing::debug!(
                    "Received event: guid={}, method={}, params={}",
                    event.guid,
                    event.method,
                    event.params
                );
                Ok(())
            }
        }
    }
}

/// Parse protocol error into Rust error type
fn parse_protocol_error(error: ErrorPayload) -> Error {
    match error.name.as_deref() {
        Some("TimeoutError") => Error::Timeout(error.message),
        Some("TargetClosedError") => Error::TargetClosed(error.message),
        _ => Error::ProtocolError(error.message),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tokio::io::duplex;

    // Helper to create test connection with mock transport
    fn create_test_connection() -> (
        Connection<tokio::io::DuplexStream, tokio::io::DuplexStream>,
        tokio::io::DuplexStream,
        tokio::io::DuplexStream,
    ) {
        let (stdin_read, stdin_write) = duplex(1024);
        let (stdout_read, stdout_write) = duplex(1024);

        let (transport, message_rx) = PipeTransport::new(stdin_write, stdout_read);
        let connection = Connection::new(transport, message_rx);

        (connection, stdin_read, stdout_write)
    }

    #[test]
    fn test_request_id_increments() {
        let (connection, _, _) = create_test_connection();

        // Generate IDs by incrementing the counter directly
        let id1 = connection.last_id.fetch_add(1, Ordering::SeqCst);
        let id2 = connection.last_id.fetch_add(1, Ordering::SeqCst);
        let id3 = connection.last_id.fetch_add(1, Ordering::SeqCst);

        assert_eq!(id1, 0);
        assert_eq!(id2, 1);
        assert_eq!(id3, 2);
    }

    #[test]
    fn test_request_format() {
        let request = Request {
            id: 0,
            guid: "page@abc123".to_string(),
            method: "goto".to_string(),
            params: serde_json::json!({"url": "https://example.com"}),
        };

        assert_eq!(request.id, 0);
        assert_eq!(request.guid, "page@abc123");
        assert_eq!(request.method, "goto");
        assert_eq!(request.params["url"], "https://example.com");
    }

    #[tokio::test]
    async fn test_dispatch_response_success() {
        let (connection, _, _) = create_test_connection();

        // Generate ID
        let id = connection.last_id.fetch_add(1, Ordering::SeqCst);

        // Create oneshot channel and store callback
        let (tx, rx) = oneshot::channel();
        connection.callbacks.lock().await.insert(id, tx);

        // Simulate response from server
        let response = Message::Response(Response {
            id,
            result: Some(serde_json::json!({"status": "ok"})),
            error: None,
        });

        // Dispatch response
        connection.dispatch(response).await.unwrap();

        // Verify result
        let result = rx.await.unwrap().unwrap();
        assert_eq!(result["status"], "ok");
    }

    #[tokio::test]
    async fn test_dispatch_response_error() {
        let (connection, _, _) = create_test_connection();

        // Generate ID
        let id = connection.last_id.fetch_add(1, Ordering::SeqCst);

        // Create oneshot channel and store callback
        let (tx, rx) = oneshot::channel();
        connection.callbacks.lock().await.insert(id, tx);

        // Simulate error response from server
        let response = Message::Response(Response {
            id,
            result: None,
            error: Some(ErrorWrapper {
                error: ErrorPayload {
                    message: "Navigation timeout".to_string(),
                    name: Some("TimeoutError".to_string()),
                    stack: None,
                },
            }),
        });

        // Dispatch response
        connection.dispatch(response).await.unwrap();

        // Verify error
        let result = rx.await.unwrap();
        assert!(result.is_err());
        match result.unwrap_err() {
            Error::Timeout(msg) => assert_eq!(msg, "Navigation timeout"),
            _ => panic!("Expected Timeout error"),
        }
    }

    #[tokio::test]
    async fn test_dispatch_invalid_id() {
        let (connection, _, _) = create_test_connection();

        // Create response with ID that doesn't match any request
        let response = Message::Response(Response {
            id: 999,
            result: Some(Value::Null),
            error: None,
        });

        // Dispatch should return error
        let result = connection.dispatch(response).await;
        assert!(result.is_err());
        match result.unwrap_err() {
            Error::ProtocolError(msg) => assert!(msg.contains("Cannot find request")),
            _ => panic!("Expected ProtocolError"),
        }
    }

    #[tokio::test]
    async fn test_concurrent_requests() {
        let (connection, _, _) = create_test_connection();
        let connection = Arc::new(connection);

        // Create callbacks for multiple requests
        let id1 = connection.last_id.fetch_add(1, Ordering::SeqCst);
        let id2 = connection.last_id.fetch_add(1, Ordering::SeqCst);
        let id3 = connection.last_id.fetch_add(1, Ordering::SeqCst);

        let (tx1, rx1) = oneshot::channel();
        let (tx2, rx2) = oneshot::channel();
        let (tx3, rx3) = oneshot::channel();

        connection.callbacks.lock().await.insert(id1, tx1);
        connection.callbacks.lock().await.insert(id2, tx2);
        connection.callbacks.lock().await.insert(id3, tx3);

        // Verify IDs are unique
        assert_eq!(id1, 0);
        assert_eq!(id2, 1);
        assert_eq!(id3, 2);

        // Simulate responses arriving in different order
        let conn1 = Arc::clone(&connection);
        let conn2 = Arc::clone(&connection);
        let conn3 = Arc::clone(&connection);

        let handle1 = tokio::spawn(async move {
            conn1
                .dispatch(Message::Response(Response {
                    id: 1,
                    result: Some(serde_json::json!({"page": "2"})),
                    error: None,
                }))
                .await
                .unwrap();
        });

        let handle2 = tokio::spawn(async move {
            conn2
                .dispatch(Message::Response(Response {
                    id: 0,
                    result: Some(serde_json::json!({"page": "1"})),
                    error: None,
                }))
                .await
                .unwrap();
        });

        let handle3 = tokio::spawn(async move {
            conn3
                .dispatch(Message::Response(Response {
                    id: 2,
                    result: Some(serde_json::json!({"page": "3"})),
                    error: None,
                }))
                .await
                .unwrap();
        });

        // Wait for all dispatches to complete
        handle1.await.unwrap();
        handle2.await.unwrap();
        handle3.await.unwrap();

        // Verify each receiver gets the correct response
        let result1 = rx1.await.unwrap().unwrap();
        let result2 = rx2.await.unwrap().unwrap();
        let result3 = rx3.await.unwrap().unwrap();

        assert_eq!(result1["page"], "1");
        assert_eq!(result2["page"], "2");
        assert_eq!(result3["page"], "3");
    }

    #[test]
    fn test_message_deserialization_response() {
        let json = r#"{"id": 42, "result": {"status": "ok"}}"#;
        let message: Message = serde_json::from_str(json).unwrap();

        match message {
            Message::Response(response) => {
                assert_eq!(response.id, 42);
                assert!(response.result.is_some());
                assert!(response.error.is_none());
            }
            _ => panic!("Expected Response"),
        }
    }

    #[test]
    fn test_message_deserialization_event() {
        let json = r#"{"guid": "page@abc", "method": "console", "params": {"text": "hello"}}"#;
        let message: Message = serde_json::from_str(json).unwrap();

        match message {
            Message::Event(event) => {
                assert_eq!(event.guid, "page@abc");
                assert_eq!(event.method, "console");
                assert_eq!(event.params["text"], "hello");
            }
            _ => panic!("Expected Event"),
        }
    }

    #[test]
    fn test_error_type_parsing() {
        // TimeoutError
        let error = parse_protocol_error(ErrorPayload {
            message: "timeout".to_string(),
            name: Some("TimeoutError".to_string()),
            stack: None,
        });
        assert!(matches!(error, Error::Timeout(_)));

        // TargetClosedError
        let error = parse_protocol_error(ErrorPayload {
            message: "closed".to_string(),
            name: Some("TargetClosedError".to_string()),
            stack: None,
        });
        assert!(matches!(error, Error::TargetClosed(_)));

        // Generic error
        let error = parse_protocol_error(ErrorPayload {
            message: "generic".to_string(),
            name: None,
            stack: None,
        });
        assert!(matches!(error, Error::ProtocolError(_)));
    }
}
