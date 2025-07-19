use rext_core::*;
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
