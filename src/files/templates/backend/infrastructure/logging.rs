use std::env;
use tracing_subscriber::{
    EnvFilter,
    fmt::{format::FmtSpan, time::UtcTime},
};

use crate::infrastructure::websocket::broadcast_system_log;

/// Logging configuration manager
pub struct LoggingManager;

impl LoggingManager {
    /// Initialize logging with environment-based configuration
    pub fn initialize() {
        let environment = env::var("ENVIRONMENT").unwrap_or_else(|_| "development".to_string());
        let log_level = env::var("RUST_LOG").unwrap_or_else(|_| {
            if environment == "development" {
                "debug".to_string()
            } else {
                "info".to_string()
            }
        });

        // Create environment filter with specific crate filters to reduce noise
        let env_filter = EnvFilter::try_from_default_env().unwrap_or_else(|_| {
            EnvFilter::new(log_level)
                .add_directive("sqlx=warn".parse().unwrap())
                .add_directive("apalis=warn".parse().unwrap())
                .add_directive("tower_http=warn".parse().unwrap())
        });

        // Configure tracing subscriber with custom layer for WebSocket broadcasting
        let subscriber = tracing_subscriber::fmt()
            .with_env_filter(env_filter)
            .with_timer(UtcTime::rfc_3339())
            .with_span_events(FmtSpan::CLOSE)
            .with_target(false)
            .with_thread_ids(true)
            .with_thread_names(true)
            .with_ansi(environment != "production");

        // Use JSON format in production, pretty format in development
        if environment == "production" {
            subscriber.json().init();
        } else {
            subscriber.pretty().init();
        }

        // Set up custom event subscriber for WebSocket broadcasting
        tracing::info!("Logging initialized for environment: {}", environment);

        // Broadcast the initialization message
        let environment_clone = environment.clone();
        tokio::spawn(async move {
            broadcast_system_log(
                "info".to_string(),
                format!(
                    "Logging system initialized for environment: {}",
                    environment_clone
                ),
                "logging".to_string(),
            )
            .await;
        });
    }

    /// Create a request ID for tracking requests across the system
    pub fn generate_request_id() -> String {
        uuid::Uuid::new_v4().to_string()
    }

    /// Broadcast a log message to WebSocket clients
    pub async fn broadcast_log(level: &str, message: &str, target: &str) {
        broadcast_system_log(level.to_string(), message.to_string(), target.to_string()).await;
    }
}
