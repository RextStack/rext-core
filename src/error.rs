use std::net::SocketAddr;

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
