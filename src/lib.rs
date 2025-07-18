//! # rext_core
//!
//! The `rext_core` crate is the library that powers Rext, the fullstack, batteries included Rust framework for developing web applications.
//!
//! It handles the absolute most basic requirements nearly all web apps will share, such as routing, API documentation, and the front-end.
//!
//! Status: 0%
//!
//! [Visit Rext](https://rextstack.org)

use axum::{Router, routing::get};
use std::net::SocketAddr;
use tokio::net::TcpListener;

/// Custom error codes for RextCore
#[derive(thiserror::Error, Debug)]
pub enum RextCoreError {
    #[error("Failed to bind to address {address}: {source}")]
    BindError {
        address: SocketAddr,
        source: std::io::Error,
    },
    #[error("Server failed to start: {0}")]
    ServerStart(#[from] std::io::Error),
}

/// Configuration for the server
pub struct ServerConfig {
    pub host: [u8; 4],
    pub port: u16,
}

impl Default for ServerConfig {
    fn default() -> Self {
        Self {
            host: [0, 0, 0, 0],
            port: 3000,
        }
    }
}

/// Create and configure the router with all routes
pub fn create_router() -> Router {
    Router::new().route("/", get(root))
}

/// Start the server with the given configuration
/// Traditional blocking behavior
pub async fn server_blocking(config: ServerConfig) -> Result<(), RextCoreError> {
    let app = create_router();
    let address = SocketAddr::from((config.host, config.port));

    let listener = TcpListener::bind(address)
        .await
        .map_err(|e| RextCoreError::BindError { address, source: e })?;

    axum::serve(listener, app).await?;
    Ok(())
}

/// Start the server and return the bound address and a handle
/// This is non-blocking
pub async fn server_non_blocking(
    config: ServerConfig,
) -> Result<
    (
        SocketAddr,
        tokio::task::JoinHandle<Result<(), RextCoreError>>,
    ),
    RextCoreError,
> {
    let app = create_router();
    let address = SocketAddr::from((config.host, config.port));

    let listener = TcpListener::bind(address)
        .await
        .map_err(|e| RextCoreError::BindError { address, source: e })?;

    // Get the actual bound address (useful when port is 0 for dynamic allocation)
    let bound_addr = listener
        .local_addr()
        .map_err(|e| RextCoreError::ServerStart(e))?;

    // Spawn the server in a separate task
    let server_handle = tokio::spawn(async move {
        axum::serve(listener, app)
            .await
            .map_err(RextCoreError::ServerStart)
    });

    Ok((bound_addr, server_handle))
}

async fn root() -> &'static str {
    "Hello, World!"
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::Duration;

    #[tokio::test]
    async fn test_server_starts_and_responds() {
        // Use port 0 to get a random available port
        let config = ServerConfig {
            host: [127, 0, 0, 1], // localhost for testing
            port: 0,              // Let the OS choose an available port
        };

        // Start server with shutdown capability
        let (bound_addr, server_handle) = server_non_blocking(config)
            .await
            .expect("Failed to start server");

        // Give the server a moment to fully start
        tokio::time::sleep(Duration::from_millis(100)).await;

        // Create HTTP client and test the endpoint
        let client = reqwest::Client::new();
        let url = format!("http://{}", bound_addr);

        // Test the root endpoint
        let response = client
            .get(&url)
            .send()
            .await
            .expect("Failed to send request");

        // Verify the response
        assert_eq!(response.status(), 200);

        let body = response.text().await.expect("Failed to read response body");

        assert_eq!(body, "Hello, World!");

        // Test that we can make multiple requests
        let response2 = client
            .get(&url)
            .send()
            .await
            .expect("Failed to send second request");

        assert_eq!(response2.status(), 200);

        let body2 = response2
            .text()
            .await
            .expect("Failed to read response2 body");

        assert_eq!(body2, "Hello, World!");

        // Shutdown the server
        server_handle.abort();

        // Verify the server handle completes (either successfully or due to abort)
        let _ = server_handle.await;
    }

    #[tokio::test]
    async fn test_router_creation() {
        // ensure router creation works and compiles
        let _router = create_router();
    }
}
