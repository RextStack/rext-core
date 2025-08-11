use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::{RwLock, broadcast};

/// WebSocket message types for real-time monitoring
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum WebSocketMessage {
    /// New audit log entry
    #[serde(rename = "AuditLog")]
    AuditLog {
        id: String,
        timestamp: String,
        method: String,
        path: String,
        status_code: Option<i32>,
        response_time_ms: Option<i32>,
        user_id: Option<String>,
        ip_address: Option<String>,
        user_agent: Option<String>,
        error_message: Option<String>,
    },
    /// System log message
    #[serde(rename = "SystemLog")]
    SystemLog {
        level: String,
        message: String,
        timestamp: String,
        target: String,
    },
    /// Performance metrics update
    #[serde(rename = "PerformanceMetrics")]
    PerformanceMetrics {
        total_requests: u64,
        success_rate: f64,
        avg_response_time: f64,
        error_rate: f64,
        active_connections: u32,
    },
    /// Connection status
    #[serde(rename = "ConnectionStatus")]
    ConnectionStatus {
        status: String,
        message: String,
        timestamp: String,
    },
    /// Ping/Pong for connection health
    #[serde(rename = "Ping")]
    Ping,
    #[serde(rename = "Pong")]
    Pong,
}

/// WebSocket connection manager
pub struct WebSocketManager {
    /// Broadcast channel for sending messages to all connected clients
    tx: broadcast::Sender<WebSocketMessage>,
    /// Active connections with their IDs
    connections: Arc<RwLock<HashMap<String, broadcast::Sender<WebSocketMessage>>>>,
}

impl WebSocketManager {
    /// Create a new WebSocket manager
    pub fn new() -> Self {
        let (tx, _) = broadcast::channel(1000); // Buffer size of 1000 messages
        Self {
            tx,
            connections: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Subscribe to the broadcast channel
    pub fn subscribe(&self) -> broadcast::Receiver<WebSocketMessage> {
        self.tx.subscribe()
    }

    /// Broadcast a message to all connected clients
    pub async fn broadcast(&self, message: WebSocketMessage) {
        if let Err(e) = self.tx.send(message) {
            // Don't particularly care if the channel is closed, this is normal if no one is connected
            // log all other errors
            if !e.to_string().contains("channel closed") {
                tracing::warn!("Failed to broadcast message: {}", e);
            }
        }
    }

    /// Add a new connection
    #[allow(dead_code)]
    pub async fn add_connection(
        &self,
        connection_id: String,
    ) -> broadcast::Receiver<WebSocketMessage> {
        let (tx, _rx) = broadcast::channel(100);
        self.connections.write().await.insert(connection_id, tx);
        self.subscribe()
    }

    /// Remove a connection
    pub async fn remove_connection(&self, connection_id: &str) {
        self.connections.write().await.remove(connection_id);
    }

    /// Get the number of active connections
    pub async fn connection_count(&self) -> usize {
        self.connections.read().await.len()
    }

    /// Send a message to a specific connection
    #[allow(dead_code)]
    pub async fn send_to_connection(&self, connection_id: &str, message: WebSocketMessage) -> bool {
        if let Some(tx) = self.connections.read().await.get(connection_id) {
            tx.send(message).is_ok()
        } else {
            false
        }
    }
}

/// Global WebSocket manager instance
pub static WEBSOCKET_MANAGER: once_cell::sync::Lazy<WebSocketManager> =
    once_cell::sync::Lazy::new(WebSocketManager::new);

/// Helper function to broadcast audit log entries
pub async fn broadcast_audit_log(
    id: String,
    timestamp: String,
    method: String,
    path: String,
    status_code: Option<i32>,
    response_time_ms: Option<i32>,
    user_id: Option<String>,
    ip_address: Option<String>,
    user_agent: Option<String>,
    error_message: Option<String>,
) {
    let message = WebSocketMessage::AuditLog {
        id,
        timestamp,
        method,
        path,
        status_code,
        response_time_ms,
        user_id,
        ip_address,
        user_agent,
        error_message,
    };
    WEBSOCKET_MANAGER.broadcast(message).await;
}

/// Helper function to broadcast system logs
pub async fn broadcast_system_log(level: String, message: String, target: String) {
    let message = WebSocketMessage::SystemLog {
        level,
        message,
        timestamp: chrono::Utc::now().to_rfc3339(),
        target,
    };
    WEBSOCKET_MANAGER.broadcast(message).await;
}

/// Helper function to broadcast performance metrics
#[allow(dead_code)]
pub async fn broadcast_performance_metrics(
    total_requests: u64,
    success_rate: f64,
    avg_response_time: f64,
    error_rate: f64,
) {
    let active_connections = WEBSOCKET_MANAGER.connection_count().await as u32;
    let message = WebSocketMessage::PerformanceMetrics {
        total_requests,
        success_rate,
        avg_response_time,
        error_rate,
        active_connections,
    };
    WEBSOCKET_MANAGER.broadcast(message).await;
}

/// Start a background task that periodically broadcasts performance metrics
pub async fn start_metrics_broadcaster() {
    let manager = &WEBSOCKET_MANAGER;

    tokio::spawn(async move {
        let mut interval = tokio::time::interval(tokio::time::Duration::from_secs(30)); // Every 30 seconds

        loop {
            interval.tick().await;

            // Get connection count
            let active_connections = manager.connection_count().await as u32;

            // For now, we'll send basic metrics
            // In a real implementation, you'd calculate these from audit logs
            let message = WebSocketMessage::PerformanceMetrics {
                total_requests: 0, // This will be calculated from audit logs
                success_rate: 0.0,
                avg_response_time: 0.0,
                error_rate: 0.0,
                active_connections,
            };

            manager.broadcast(message).await;

            // Broadcast a heartbeat log
            broadcast_system_log(
                "debug".to_string(),
                format!(
                    "Metrics broadcast - Active connections: {}",
                    active_connections
                ),
                "metrics_broadcaster".to_string(),
            )
            .await;
        }
    });
}
