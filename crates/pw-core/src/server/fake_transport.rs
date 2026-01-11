//! Fake transport for unit testing JSON-RPC correlation and event dispatch.
//!
//! Provides an in-memory transport for testing the protocol layer without browsers.
//!
//! # Example
//!
//! ```ignore
//! let (parts, controller) = FakeTransportBuilder::new().build();
//! let connection = Arc::new(Connection::new(parts));
//!
//! tokio::spawn({
//!     let conn = Arc::clone(&connection);
//!     async move { conn.run().await }
//! });
//!
//! let fut = connection.send_message("page@1", "click", json!({}));
//! controller.inject_response(0, json!({"ok": true}));
//! let result = fut.await?;
//! ```

use crate::Result;
use serde_json::Value as JsonValue;
use std::future::Future;
use std::pin::Pin;
use std::sync::Arc;
use tokio::sync::{Mutex, mpsc};

use super::transport::{Transport, TransportParts, TransportReceiver};

/// Builder for creating fake transport instances.
pub struct FakeTransportBuilder {
    // Nothing needed for now, but allows future extensibility
}

impl FakeTransportBuilder {
    /// Create a new fake transport builder.
    pub fn new() -> Self {
        Self {}
    }

    /// Build the fake transport and return both parts and a controller.
    ///
    /// Returns [`TransportParts`] for creating a [`Connection`] and a
    /// [`FakeTransportController`] for injecting responses and inspecting sent messages.
    ///
    /// [`Connection`]: crate::server::connection::Connection
    pub fn build(self) -> (TransportParts, FakeTransportController) {
        let (inbound_tx, inbound_rx) = mpsc::unbounded_channel();
        let (message_tx, message_rx) = mpsc::unbounded_channel();
        let sent_messages = Arc::new(Mutex::new(Vec::new()));

        let sender = FakeTransportSender {
            sent: Arc::clone(&sent_messages),
        };

        let receiver = FakeTransportReceiver {
            inbound_rx,
            message_tx,
        };

        let controller = FakeTransportController {
            inbound_tx,
            sent: sent_messages,
        };

        let parts = TransportParts {
            sender: Box::new(sender),
            receiver: Box::new(receiver),
            message_rx,
        };

        (parts, controller)
    }
}

impl Default for FakeTransportBuilder {
    fn default() -> Self {
        Self::new()
    }
}

/// Controller for injecting responses/events and inspecting sent messages.
pub struct FakeTransportController {
    inbound_tx: mpsc::UnboundedSender<JsonValue>,
    sent: Arc<Mutex<Vec<JsonValue>>>,
}

impl FakeTransportController {
    /// Inject a raw JSON message into the connection.
    ///
    /// Use this to simulate receiving a message from the server.
    pub fn inject(&self, message: JsonValue) {
        let _ = self.inbound_tx.send(message);
    }

    /// Inject a response message with the given ID and result.
    pub fn inject_response(&self, id: u32, result: JsonValue) {
        self.inject(serde_json::json!({
            "id": id,
            "result": result
        }));
    }

    /// Inject an error response message.
    pub fn inject_error(&self, id: u32, name: &str, message: &str) {
        self.inject(serde_json::json!({
            "id": id,
            "error": {
                "error": {
                    "message": message,
                    "name": name
                }
            }
        }));
    }

    /// Inject an event message.
    pub fn inject_event(&self, guid: &str, method: &str, params: JsonValue) {
        self.inject(serde_json::json!({
            "guid": guid,
            "method": method,
            "params": params
        }));
    }

    /// Take all sent messages, clearing the buffer.
    pub async fn take_sent(&self) -> Vec<JsonValue> {
        std::mem::take(&mut *self.sent.lock().await)
    }
}

struct FakeTransportSender {
    sent: Arc<Mutex<Vec<JsonValue>>>,
}

impl Transport for FakeTransportSender {
    fn send(
        &mut self,
        message: JsonValue,
    ) -> Pin<Box<dyn Future<Output = Result<()>> + Send + '_>> {
        let sent = Arc::clone(&self.sent);
        Box::pin(async move {
            sent.lock().await.push(message);
            Ok(())
        })
    }
}

struct FakeTransportReceiver {
    inbound_rx: mpsc::UnboundedReceiver<JsonValue>,
    message_tx: mpsc::UnboundedSender<JsonValue>,
}

impl TransportReceiver for FakeTransportReceiver {
    fn run(mut self: Box<Self>) -> Pin<Box<dyn Future<Output = Result<()>> + Send>> {
        Box::pin(async move {
            while let Some(message) = self.inbound_rx.recv().await {
                if self.message_tx.send(message).is_err() {
                    break;
                }
            }
            Ok(())
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::server::connection::Connection;
    use std::sync::Arc;

    #[tokio::test]
    async fn test_fake_transport_send_capture() {
        let (parts, controller) = FakeTransportBuilder::new().build();
        let connection = Arc::new(Connection::new(parts));

        // Spawn connection loop
        let conn_clone = Arc::clone(&connection);
        tokio::spawn(async move {
            conn_clone.run().await;
        });

        // Give the connection time to start
        tokio::time::sleep(std::time::Duration::from_millis(10)).await;

        // Send a message (will be captured)
        let send_fut =
            connection.send_message("test@1", "testMethod", serde_json::json!({"key": "value"}));

        // Inject a response for it
        controller.inject_response(0, serde_json::json!({"status": "ok"}));

        // Wait for the response
        let result = send_fut.await.unwrap();
        assert_eq!(result["status"], "ok");

        // Check what was sent
        let sent = controller.take_sent().await;
        assert_eq!(sent.len(), 1);
        assert_eq!(sent[0]["guid"], "test@1");
        assert_eq!(sent[0]["method"], "testMethod");
        assert_eq!(sent[0]["params"]["key"], "value");
        assert_eq!(sent[0]["id"], 0);
    }

    #[tokio::test]
    async fn test_fake_transport_multiple_requests_correlation() {
        let (parts, controller) = FakeTransportBuilder::new().build();
        let connection = Arc::new(Connection::new(parts));

        let conn_clone = Arc::clone(&connection);
        tokio::spawn(async move {
            conn_clone.run().await;
        });

        tokio::time::sleep(std::time::Duration::from_millis(10)).await;

        // Send two requests concurrently
        let conn1 = Arc::clone(&connection);
        let conn2 = Arc::clone(&connection);

        let fut1 = tokio::spawn(async move {
            conn1
                .send_message(
                    "page@1",
                    "goto",
                    serde_json::json!({"url": "https://a.com"}),
                )
                .await
        });

        let fut2 = tokio::spawn(async move {
            conn2
                .send_message(
                    "page@2",
                    "goto",
                    serde_json::json!({"url": "https://b.com"}),
                )
                .await
        });

        // Give time for both requests to be sent
        tokio::time::sleep(std::time::Duration::from_millis(10)).await;

        // Respond in reverse order to test correlation
        controller.inject_response(1, serde_json::json!({"url": "https://b.com"}));
        controller.inject_response(0, serde_json::json!({"url": "https://a.com"}));

        let result1 = fut1.await.unwrap().unwrap();
        let result2 = fut2.await.unwrap().unwrap();

        // Despite responses arriving in reverse order, each future should get the correct response
        assert_eq!(result1["url"], "https://a.com");
        assert_eq!(result2["url"], "https://b.com");
    }

    #[tokio::test]
    async fn test_fake_transport_error_response() {
        let (parts, controller) = FakeTransportBuilder::new().build();
        let connection = Arc::new(Connection::new(parts));

        let conn_clone = Arc::clone(&connection);
        tokio::spawn(async move {
            conn_clone.run().await;
        });

        tokio::time::sleep(std::time::Duration::from_millis(10)).await;

        let send_fut = connection.send_message(
            "page@1",
            "click",
            serde_json::json!({"selector": ".missing"}),
        );

        // Inject an error response
        controller.inject_error(0, "TimeoutError", "Element not found: .missing");

        let result = send_fut.await;
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(err.is_timeout());
    }

    #[tokio::test]
    async fn test_fake_transport_event_injection() {
        use crate::server::channel_owner::{ChannelOwner, ChannelOwnerImpl, ParentOrConnection};
        use crate::server::connection::ConnectionLike;
        use parking_lot::Mutex as PLMutex;
        use serde_json::Value;
        use std::sync::Arc as StdArc;

        // Create a mock channel owner to receive events
        struct MockOwner {
            base: ChannelOwnerImpl,
            events: StdArc<PLMutex<Vec<(String, Value)>>>,
        }

        impl crate::server::channel_owner::private::Sealed for MockOwner {}

        impl ChannelOwner for MockOwner {
            fn guid(&self) -> &str {
                self.base.guid()
            }
            fn type_name(&self) -> &str {
                "MockOwner"
            }
            fn parent(&self) -> Option<Arc<dyn ChannelOwner>> {
                self.base.parent()
            }
            fn connection(&self) -> Arc<dyn ConnectionLike> {
                self.base.connection()
            }
            fn initializer(&self) -> &Value {
                self.base.initializer()
            }
            fn channel(&self) -> &crate::server::channel::Channel {
                self.base.channel()
            }
            fn dispose(&self, reason: crate::server::channel_owner::DisposeReason) {
                self.base.dispose(reason)
            }
            fn adopt(&self, child: Arc<dyn ChannelOwner>) {
                self.base.adopt(child)
            }
            fn add_child(&self, guid: Arc<str>, child: Arc<dyn ChannelOwner>) {
                self.base.add_child(guid, child)
            }
            fn remove_child(&self, guid: &str) {
                self.base.remove_child(guid)
            }
            fn on_event(&self, method: &str, params: Value) {
                self.events.lock().push((method.to_string(), params));
            }
            fn was_collected(&self) -> bool {
                self.base.was_collected()
            }
        }

        let (parts, controller) = FakeTransportBuilder::new().build();
        let connection = Arc::new(Connection::new(parts));

        // Create and register a mock owner
        let events = StdArc::new(PLMutex::new(Vec::new()));
        let owner = Arc::new(MockOwner {
            base: ChannelOwnerImpl::new(
                ParentOrConnection::Connection(Arc::clone(&connection) as Arc<dyn ConnectionLike>),
                "MockOwner".to_string(),
                Arc::from("mock@1"),
                serde_json::json!({}),
            ),
            events: StdArc::clone(&events),
        });

        connection
            .register_object(Arc::from("mock@1"), owner as Arc<dyn ChannelOwner>)
            .await;

        let conn_clone = Arc::clone(&connection);
        tokio::spawn(async move {
            conn_clone.run().await;
        });

        tokio::time::sleep(std::time::Duration::from_millis(10)).await;

        // Inject an event
        controller.inject_event("mock@1", "testEvent", serde_json::json!({"data": 42}));

        // Give time for event to be processed
        tokio::time::sleep(std::time::Duration::from_millis(10)).await;

        // Verify event was received
        let received = events.lock();
        assert_eq!(received.len(), 1);
        assert_eq!(received[0].0, "testEvent");
        assert_eq!(received[0].1["data"], 42);
    }
}
