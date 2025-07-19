//! # rext_core
//!
//! The `rext_core` crate is the library that powers Rext, the fullstack, batteries included Rust framework for developing web applications.
//!
//! It handles the absolute most basic requirements nearly all web apps will share, such as routing, API documentation, and the front-end.
//!
//! Status: 0%
//!
//! [Visit Rext](https://rextstack.org)
//!

use axum::{Router, routing::get};
use std::net::SocketAddr;
use tokio::net::TcpListener;

mod error;

use crate::error::RextCoreError;

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
    let bound_addr = listener.local_addr().map_err(RextCoreError::ServerStart)?;

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
