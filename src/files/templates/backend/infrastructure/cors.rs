use axum::http::{HeaderName, HeaderValue, Method};
use std::env;
use tower_http::cors::CorsLayer;

/// CORS configuration manager
pub struct CorsManager;

impl CorsManager {
    /// Creates CORS layer based on environment
    pub fn create_cors_layer() -> CorsLayer {
        let environment = env::var("ENVIRONMENT").unwrap_or_else(|_| "development".to_string());

        if environment == "development" {
            Self::create_development_cors()
        } else {
            Self::create_production_cors()
        }
    }

    /// Creates CORS configuration for development
    fn create_development_cors() -> CorsLayer {
        CorsLayer::new()
            .allow_origin("http://localhost:5173".parse::<HeaderValue>().unwrap())
            .allow_methods([Method::GET, Method::POST, Method::PUT, Method::DELETE])
            .allow_headers([
                "authorization".parse::<HeaderName>().unwrap(),
                "content-type".parse::<HeaderName>().unwrap(),
                "accept".parse::<HeaderName>().unwrap(),
                "origin".parse::<HeaderName>().unwrap(),
                "x-requested-with".parse::<HeaderName>().unwrap(),
            ])
            .allow_credentials(true)
    }

    /// Creates CORS configuration for production
    fn create_production_cors() -> CorsLayer {
        let allowed_origin =
            env::var("ALLOWED_ORIGIN").unwrap_or_else(|_| "https://yourdomain.com".to_string());

        CorsLayer::new()
            .allow_origin(
                allowed_origin
                    .parse::<HeaderValue>()
                    .unwrap_or_else(|_| "https://yourdomain.com".parse::<HeaderValue>().unwrap()),
            )
            .allow_methods([Method::GET, Method::POST, Method::PUT, Method::DELETE])
            .allow_headers([
                "authorization".parse::<HeaderName>().unwrap(),
                "content-type".parse::<HeaderName>().unwrap(),
                "accept".parse::<HeaderName>().unwrap(),
                "origin".parse::<HeaderName>().unwrap(),
                "x-requested-with".parse::<HeaderName>().unwrap(),
            ])
            .allow_credentials(true)
            .max_age(std::time::Duration::from_secs(3600)) // Cache preflight for 1 hour
    }
}
