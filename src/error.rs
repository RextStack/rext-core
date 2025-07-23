/// Custom error codes for RextCore
#[derive(thiserror::Error, Debug)]
pub enum RextCoreError {
    #[error("Server failed to start: {0}")]
    ServerStart(#[from] std::io::Error),

    #[error("Failed to create directory: {0}")]
    DirectoryCreation(std::io::Error),

    #[error("Failed to write file: {0}")]
    FileWrite(String),

    #[error("Rext app already exists")]
    AppAlreadyExists,

    #[error("Failed to get current directory, either does not exist or permission denied: {0}")]
    CurrentDir(std::io::Error),

    #[error("Failed to read directory: {0}")]
    DirectoryRead(std::io::Error),

    #[error("Failed to remove file: {0}")]
    FileRemoval(String),

    #[error("Failed to remove directory: {0}")]
    DirectoryRemoval(String),

    #[error("Safety check failed: {0}")]
    SafetyCheck(String),

    #[error("Failed to execute sea-orm-cli generate entities command: {0}")]
    SeaOrmCliGenerateEntities(std::io::Error),
}
