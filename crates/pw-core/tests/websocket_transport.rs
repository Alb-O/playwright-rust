//! Tests for WebSocketTransport integration

use futures_util::{SinkExt, StreamExt};
use pw::server::transport::WebSocketTransport;
use serde_json::json;
use tokio::net::TcpListener;
use tokio_tungstenite::tungstenite::Message;

#[tokio::test]
async fn websocket_transport_echo_round_trip() {
    let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = listener.local_addr().unwrap();

    let server = tokio::spawn(async move {
        let (stream, _) = listener.accept().await.unwrap();
        let ws = tokio_tungstenite::accept_async(stream).await.unwrap();
        let (mut ws_tx, mut ws_rx) = ws.split();

        let incoming = ws_rx.next().await.unwrap().unwrap();
        assert_eq!(incoming, Message::Text("{\"ping\":true}".into()));

        ws_tx
            .send(Message::Text("{\"pong\":true}".into()))
            .await
            .unwrap();
    });

    let url = format!("ws://{}", addr);
    let (transport, message_rx) = WebSocketTransport::connect(&url).await.unwrap();
    let parts = transport.into_transport_parts(message_rx);

    let mut sender = parts.sender;
    let receiver = parts.receiver;
    let mut rx = parts.message_rx;

    let recv_task = tokio::spawn(async move { receiver.run().await });

    sender.send(json!({ "ping": true })).await.unwrap();

    let reply = rx.recv().await.expect("should receive reply");
    assert_eq!(reply["pong"], true);

    // Receiver may exit with ConnectionClosed after server finishes; that's OK.
    recv_task.abort();
    let _ = recv_task.await;
    server.await.unwrap();
}
