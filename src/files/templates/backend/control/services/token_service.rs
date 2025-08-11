//! Token service for extracting, decoding, and validating JWT tokens
use axum::http::{StatusCode, header};
use jsonwebtoken::{DecodingKey, Validation, decode};
use sea_orm::DatabaseConnection;
use std::env;
use std::time::{SystemTime, UNIX_EPOCH};
use uuid::Uuid;

use crate::{
    control::services::session_service::SessionService,
    infrastructure::{app_error::AppError, jwt_claims::Claims},
};

/// Service for JWT token operations
pub struct TokenService;

impl TokenService {
    /// Extracts and validates a JWT token from the Authorization header
    /// Returns the user ID if the token is valid (JWT validation only)
    #[allow(dead_code)]
    pub fn extract_and_validate_token(
        request: &axum::http::Request<axum::body::Body>,
    ) -> Result<Uuid, AppError> {
        // Extract token from Authorization header
        let token = Self::extract_token_from_header(request)?;

        // Validate the token
        let user_id = Self::validate_token(&token)?;

        Ok(user_id)
    }

    /// Extracts and validates a JWT token with session validation
    /// Returns the user ID and session ID if both token and session are valid
    pub async fn extract_and_validate_token_with_session(
        db: &DatabaseConnection,
        token: &str,
    ) -> Result<(Uuid, Uuid), AppError> {
        // Validate JWT token and extract claims
        let claims = Self::validate_token_claims(&token)?;

        // Parse user ID
        let user_id = Uuid::parse_str(&claims.sub).map_err(|_| AppError {
            message: "Invalid user ID in token".to_string(),
            status_code: StatusCode::UNAUTHORIZED,
        })?;

        // Parse session ID
        let session_id = Uuid::parse_str(&claims.session_id).map_err(|_| AppError {
            message: "Invalid session ID in token".to_string(),
            status_code: StatusCode::UNAUTHORIZED,
        })?;

        // Validate session exists and is active
        SessionService::validate_session(db, &claims.session_id).await?;

        Ok((user_id, session_id))
    }

    /// Extracts JWT token from Authorization header
    /// Returns the token (no validation is performed)
    pub fn extract_token_from_header(
        request: &axum::http::Request<axum::body::Body>,
    ) -> Result<String, AppError> {
        let auth_header = request
            .headers()
            .get(header::AUTHORIZATION)
            .and_then(|header| header.to_str().ok())
            .ok_or(AppError {
                message: "Missing Authorization header".to_string(),
                status_code: StatusCode::UNAUTHORIZED,
            })?;

        let token = auth_header.strip_prefix("Bearer ").ok_or(AppError {
            message: "Invalid Authorization header format".to_string(),
            status_code: StatusCode::UNAUTHORIZED,
        })?;

        Ok(token.to_string())
    }

    /// Validates a JWT token and returns the user ID
    /// Returns the user ID if the token is valid
    pub fn validate_token(token: &str) -> Result<Uuid, AppError> {
        let jwt_secret = env::var("JWT_SECRET").unwrap_or_else(|_| "default-secret".to_string());
        let decoding_key = DecodingKey::from_secret(jwt_secret.as_ref());

        // Decode and validate the token
        let token_data =
            decode::<Claims>(token, &decoding_key, &Validation::default()).map_err(|_| {
                AppError {
                    message: "Invalid token".to_string(),
                    status_code: StatusCode::UNAUTHORIZED,
                }
            })?;

        // Check if token is expired
        let current_time = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs() as usize;

        if token_data.claims.exp < current_time {
            return Err(AppError {
                message: "Token expired".to_string(),
                status_code: StatusCode::UNAUTHORIZED,
            });
        }

        // Parse user ID from token
        let user_id = Uuid::parse_str(&token_data.claims.sub).map_err(|_| AppError {
            message: "Invalid user ID in token".to_string(),
            status_code: StatusCode::UNAUTHORIZED,
        })?;

        Ok(user_id)
    }

    /// Validates a JWT token and returns the Claims struct
    #[allow(dead_code)]
    pub fn validate_token_claims(token: &str) -> Result<Claims, AppError> {
        let jwt_secret = env::var("JWT_SECRET").unwrap_or_else(|_| "default-secret".to_string());
        let decoding_key = DecodingKey::from_secret(jwt_secret.as_ref());

        // Decode and validate the token
        let token_data =
            decode::<Claims>(token, &decoding_key, &Validation::default()).map_err(|_| {
                AppError {
                    message: "Invalid token".to_string(),
                    status_code: StatusCode::UNAUTHORIZED,
                }
            })?;

        // Check if token is expired
        let current_time = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs() as usize;

        if token_data.claims.exp < current_time {
            return Err(AppError {
                message: "Token expired".to_string(),
                status_code: StatusCode::UNAUTHORIZED,
            });
        }

        Ok(token_data.claims)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::http::{HeaderMap, HeaderValue};
    use jsonwebtoken::{EncodingKey, Header, encode};
    use std::time::{SystemTime, UNIX_EPOCH};

    fn create_test_token(user_id: &str, expires_in: i64) -> String {
        let jwt_secret = "test-secret";
        let encoding_key = EncodingKey::from_secret(jwt_secret.as_ref());

        let expiration = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs() as i64
            + expires_in;

        let claims = Claims {
            sub: user_id.to_string(),
            exp: expiration as usize,
            session_id: "".to_string(),
        };

        encode(&Header::default(), &claims, &encoding_key).unwrap()
    }

    fn create_test_request_with_token(token: &str) -> axum::http::Request<axum::body::Body> {
        let mut headers = HeaderMap::new();
        headers.insert(
            header::AUTHORIZATION,
            HeaderValue::from_str(&format!("Bearer {}", token)).unwrap(),
        );

        let mut request = axum::http::Request::new(axum::body::Body::empty());
        *request.headers_mut() = headers;
        request
    }

    #[test]
    fn test_extract_token_from_header_valid() {
        let token = "test-token";
        let request = create_test_request_with_token(token);

        let result = TokenService::extract_token_from_header(&request);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), token);
    }

    #[test]
    fn test_extract_token_from_header_missing() {
        let request = axum::http::Request::new(axum::body::Body::empty());

        let result = TokenService::extract_token_from_header(&request);
        assert!(result.is_err());
        assert_eq!(result.unwrap_err().status_code, StatusCode::UNAUTHORIZED);
    }

    #[test]
    fn test_extract_token_from_header_invalid_format() {
        let mut headers = HeaderMap::new();
        headers.insert(
            header::AUTHORIZATION,
            HeaderValue::from_str("InvalidFormat test-token").unwrap(),
        );

        let mut request = axum::http::Request::new(axum::body::Body::empty());
        *request.headers_mut() = headers;

        let result = TokenService::extract_token_from_header(&request);
        assert!(result.is_err());
        assert_eq!(result.unwrap_err().status_code, StatusCode::UNAUTHORIZED);
    }

    #[test]
    fn test_validate_token_valid() {
        // Set the JWT_SECRET environment variable for testing
        unsafe {
            std::env::set_var("JWT_SECRET", "test-secret");
        }

        let user_id = "123e4567-e89b-12d3-a456-426614174000";
        let token = create_test_token(user_id, 3600); // 1 hour from now

        let result = TokenService::validate_token(&token);
        assert!(result.is_ok());
        assert_eq!(result.unwrap().to_string(), user_id);
    }

    #[test]
    fn test_validate_token_expired() {
        // Set the JWT_SECRET environment variable for testing
        unsafe {
            std::env::set_var("JWT_SECRET", "test-secret");
        }

        let user_id = "123e4567-e89b-12d3-a456-426614174000";
        let token = create_test_token(user_id, -3600); // Expired 1 hour ago

        let result = TokenService::validate_token(&token);
        assert!(result.is_err());
        assert_eq!(result.unwrap_err().status_code, StatusCode::UNAUTHORIZED);
    }

    #[test]
    fn test_validate_token_invalid() {
        let result = TokenService::validate_token("invalid-token");
        assert!(result.is_err());
        assert_eq!(result.unwrap_err().status_code, StatusCode::UNAUTHORIZED);
    }

    #[test]
    fn test_extract_and_validate_token_valid() {
        // Set the JWT_SECRET environment variable for testing
        unsafe {
            std::env::set_var("JWT_SECRET", "test-secret");
        }

        let user_id = "123e4567-e89b-12d3-a456-426614174000";
        let token = create_test_token(user_id, 3600); // 1 hour from now
        let request = create_test_request_with_token(&token);

        let result = TokenService::extract_and_validate_token(&request);
        assert!(result.is_ok());
        assert_eq!(result.unwrap().to_string(), user_id);
    }

    #[test]
    fn test_validate_token_claims_valid() {
        // Set the JWT_SECRET environment variable for testing
        unsafe {
            std::env::set_var("JWT_SECRET", "test-secret");
        }

        let user_id = "123e4567-e89b-12d3-a456-426614174000";
        let token = create_test_token(user_id, 3600); // 1 hour from now

        let result = TokenService::validate_token_claims(&token);
        assert!(result.is_ok());
        let claims = result.unwrap();
        assert_eq!(claims.sub, user_id);
    }
}
