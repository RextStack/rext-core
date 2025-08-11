use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

pub const AUTH_TAG: &str = "Authentication";

// Request/Response types
#[derive(Deserialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct RegisterRequest {
    /// User's email address
    #[schema(example = "user@example.com")]
    pub email: String,

    /// User's password
    #[schema(example = "securepassword123")]
    pub password: String,
}

#[derive(Deserialize, ToSchema)]
pub struct LoginRequest {
    pub email: String,
    pub password: String,
}

#[derive(Serialize, ToSchema)]
pub struct LoginResponse {
    pub token: String,
}

#[derive(Serialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct RegisterResponse {
    /// Success message
    #[schema(example = "User created successfully")]
    pub message: String,

    /// The newly created user's ID
    #[schema(example = "550e8400-e29b-41d4-a716-446655440000")]
    pub user_id: String,

    /// User's email address
    #[schema(example = "user@example.com")]
    pub email: String,

    /// Timestamp when the user was created (ISO 8601 format)
    #[schema(example = "2024-01-20T15:30:00Z", nullable = true)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub created_at: Option<String>,
}

#[derive(Serialize, ToSchema)]
pub struct ProfileResponse {
    pub id: String,
    pub email: String,
    pub created_at: Option<chrono::DateTime<chrono::Utc>>,
}

// JWT token extractor
#[derive(Clone, ToSchema)]
pub struct AuthUser {
    #[schema(value_type = String)]
    pub user_id: uuid::Uuid,
}

#[derive(Deserialize, ToSchema)]
pub struct VerifyEmailRequest {
    pub user_id: String,
}

#[derive(Serialize, ToSchema)]
pub struct VerifyEmailResponse {
    pub message: String,
    pub success: bool,
}
