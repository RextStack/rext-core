use std::env;
use std::sync::OnceLock;

/// Server configuration service
pub struct ServerConfigService;

static SERVER_CONFIG: OnceLock<ServerConfig> = OnceLock::new();

#[derive(Debug, Clone)]
pub struct ServerConfig {
    pub port: u16,
    pub host: String,
    pub protocol: String,
    pub environment: String,
}

impl ServerConfigService {
    /// Initialize server configuration
    pub fn initialize() {
        let port = env::var("SERVER_PORT")
            .unwrap_or_else(|_| "3000".to_string())
            .parse::<u16>()
            .unwrap_or(3000);

        let host = env::var("SERVER_HOST").unwrap_or_else(|_| "localhost".to_string());

        let environment = env::var("ENVIRONMENT").unwrap_or_else(|_| "development".to_string());

        // Determine protocol based on environment and TLS configuration
        let protocol = if environment == "production" {
            // In production, assume HTTPS unless explicitly disabled
            "HTTPS".to_string()
        } else {
            // In development, use HTTP
            "HTTP".to_string()
        };

        let config = ServerConfig {
            port,
            host,
            protocol,
            environment,
        };

        SERVER_CONFIG
            .set(config)
            .expect("Failed to set server config");
    }

    /// Get server configuration
    pub fn get_config() -> Option<&'static ServerConfig> {
        SERVER_CONFIG.get()
    }

    /// Get server port
    pub fn get_port() -> u16 {
        Self::get_config().map(|config| config.port).unwrap_or(3000)
    }

    /// Get server host
    pub fn get_host() -> String {
        Self::get_config()
            .map(|config| config.host.clone())
            .unwrap_or_else(|| "localhost".to_string())
    }

    /// Get server protocol
    pub fn get_protocol() -> String {
        Self::get_config()
            .map(|config| config.protocol.clone())
            .unwrap_or_else(|| "HTTP".to_string())
    }

    /// Get environment
    pub fn get_environment() -> String {
        Self::get_config()
            .map(|config| config.environment.clone())
            .unwrap_or_else(|| "development".to_string())
    }
}
