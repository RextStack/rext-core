mod bridge;
mod control;
mod domain;
mod entity;
mod infrastructure;

use control::services::startup::StartupService;
use infrastructure::{logging::LoggingManager};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    
    // Create and start the logging manager
    LoggingManager::initialize();
    tracing::info!("Starting the Rext Server 🦖");

    // Initialize the database
    let db = StartupService::initialize().await?;

    // Start the server
    let _ = tokio::join!(
        StartupService::run_server(db)
    );

    Ok(())
}
