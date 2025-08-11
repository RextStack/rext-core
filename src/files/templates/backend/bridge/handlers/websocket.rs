use axum::{
    extract::{
        State,
        ws::{Message, WebSocket, WebSocketUpgrade},
    },
    response::IntoResponse,
};
use futures_util::{SinkExt, StreamExt};
use sea_orm::DatabaseConnection;
use serde_json;
use tokio::sync::mpsc;
use uuid::Uuid;

use crate::infrastructure::websocket::{WEBSOCKET_MANAGER, WebSocketMessage};

/// WebSocket handler for real-time monitoring
pub async fn websocket_handler(
    ws: WebSocketUpgrade,
    State(_db): State<DatabaseConnection>,
) -> impl IntoResponse {
    ws.on_upgrade(|socket| handle_socket(socket))
}

/// Handle individual WebSocket connection
async fn handle_socket(socket: WebSocket) {
    let connection_id = Uuid::new_v4().to_string();

    // Broadcast connection event
    crate::infrastructure::websocket::broadcast_system_log(
        "info".to_string(),
        format!("WebSocket connection established: {}", connection_id),
        "websocket".to_string(),
    )
    .await;

    // Send connection status
    let status_message = WebSocketMessage::ConnectionStatus {
        status: "connected".to_string(),
        message: "WebSocket connection established".to_string(),
        timestamp: chrono::Utc::now().to_rfc3339(),
    };

    // Split the socket into sender and receiver
    let (mut sender, mut receiver) = socket.split();

    // Subscribe to broadcast channel
    let mut broadcast_rx = WEBSOCKET_MANAGER.subscribe();

    // Send initial connection status
    if let Ok(message_json) = serde_json::to_string(&status_message) {
        let _ = sender.send(Message::Text(message_json.into())).await;
    }

    // Create a channel for sending messages from ping/pong task to sender task
    let (tx, mut rx) = mpsc::channel::<String>(100);

    // Clone values for the broadcast task
    let connection_id_broadcast = connection_id.clone();

    // Spawn task to forward broadcast messages to this client
    let tx_broadcast = tx.clone();
    let broadcast_task = tokio::spawn(async move {
        while let Ok(message) = broadcast_rx.recv().await {
            if let Ok(message_json) = serde_json::to_string(&message) {
                if let Err(e) = tx_broadcast.send(message_json).await {
                    tracing::warn!(
                        "Failed to send message to client {}: {}",
                        connection_id_broadcast,
                        e
                    );
                    break;
                }
            }
        }
    });

    // Clone values for the ping/pong task
    let connection_id_ping = connection_id.clone();

    // Handle incoming messages from client
    let ping_pong_task = tokio::spawn(async move {
        while let Some(Ok(message)) = receiver.next().await {
            match message {
                Message::Text(text) => {
                    // Handle text messages (could be commands or ping)
                    if text == "ping" {
                        let pong = WebSocketMessage::Pong;
                        if let Ok(pong_json) = serde_json::to_string(&pong) {
                            let _ = tx.send(pong_json).await;
                        }
                    }
                }
                Message::Close(_) => {
                    tracing::info!(
                        "WebSocket connection {} closed by client",
                        connection_id_ping
                    );
                    break;
                }
                _ => {
                    // Ignore other message types
                }
            }
        }
    });

    // Main sender task that handles both broadcast and ping/pong messages
    let sender_task = tokio::spawn(async move {
        while let Some(message_json) = rx.recv().await {
            if let Err(e) = sender.send(Message::Text(message_json.into())).await {
                tracing::warn!("Failed to send message to WebSocket: {}", e);
                break;
            }
        }
    });

    // Wait for any task to complete
    tokio::select! {
        _ = broadcast_task => {
            tracing::info!("Broadcast task ended for connection {}", connection_id);
        }
        _ = ping_pong_task => {
            tracing::info!("Ping/pong task ended for connection {}", connection_id);
        }
        _ = sender_task => {
            tracing::info!("Sender task ended for connection {}", connection_id);
        }
    }

    // Clean up connection
    WEBSOCKET_MANAGER.remove_connection(&connection_id).await;

    // Broadcast disconnection event
    crate::infrastructure::websocket::broadcast_system_log(
        "info".to_string(),
        format!("WebSocket connection closed: {}", connection_id),
        "websocket".to_string(),
    )
    .await;

    tracing::info!("WebSocket connection {} cleaned up", connection_id);
}
