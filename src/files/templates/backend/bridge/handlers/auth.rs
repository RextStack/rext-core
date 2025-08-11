use axum::{
    Extension, Json,
    extract::{Request, State},
    http::StatusCode,
    response::IntoResponse,
};
use sea_orm::DatabaseConnection;

use crate::bridge::types::{
    auth::{
        AUTH_TAG, AuthUser, LoginRequest, LoginResponse, ProfileResponse, RegisterRequest,
        RegisterResponse, VerifyEmailRequest, VerifyEmailResponse,
    },
    logging::LoggingInfo,
};
use crate::control::services::{
    auth_service::AuthService, session_service::SessionService, token_service::TokenService,
    user_service::UserService,
};
use crate::domain::user::*;
use crate::infrastructure::app_error::{AppError, ErrorResponse, MessageResponse};

/// Registers a new user
#[utoipa::path(
    post,
    path = "/register",
    request_body = RegisterRequest,
    responses(
        (status = 201, description = "User created successfully", body = RegisterResponse),
        (status = 400, description = "Bad request - validation errors", body = ErrorResponse, examples(
            ("empty_fields" = (value = json!({"message": "Email and password are required"}))),
        )),
        (status = 409, description = "Conflict - user already exists", body = ErrorResponse, examples(
            ("user_exists" = (value = json!({"message": "User already exists"})))
        )),
        (status = 500, description = "Internal server error", body = ErrorResponse, examples(
            ("hash_error" = (value = json!({"message": "Failed to hash password"}))),
            ("database_error" = (value = json!({"message": "Failed to create user"})))
        ))
    ),
    summary = "Register a new user",
    description = "Creates a new user account with email and password. Password is securely hashed using Argon2.",
    tag = AUTH_TAG
)]
pub async fn register_handler(
    State(db): State<DatabaseConnection>,
    Json(payload): Json<RegisterRequest>,
) -> Result<impl IntoResponse, AppError> {
    // Convert request to user domain model
    let registration = UserRegistration::new(payload.email, payload.password);

    // Delegate to user service, errors bubble up correctl
    let user = UserService::create_user(&db, registration).await?;

    Ok((
        StatusCode::CREATED,
        Json(RegisterResponse {
            message: "User created successfully".to_string(),
            user_id: user.id.to_string(),
            email: user.email,
            created_at: user.created_at.map(|dt| dt.to_rfc3339()),
        }),
    ))
}

/// Logs in an existing user
#[utoipa::path(
    post,
    path = "/login",
    request_body = LoginRequest,
    responses(
        (status = 200, description = "Login successful", body = LoginResponse),
        (status = 400, description = "Bad request - validation errors", body = ErrorResponse, examples(
            ("empty_fields" = (value = json!({"message": "Email and password are required"}))),
        )),
        (status = 401, description = "Unauthorized - invalid credentials", body = ErrorResponse, examples(
            ("invalid_credentials" = (value = json!({"message": "Invalid credentials"})))
        )),
        (status = 500, description = "Internal server error", body = ErrorResponse, examples(
            ("hash_error" = (value = json!({"message": "Invalid password hash"}))),
            ("token_error" = (value = json!({"message": "Failed to generate token"})))
        ))
    ),
    summary = "Login user",
    description = "Authenticates a user with email and password, returns a JWT token on success.",
    tag = AUTH_TAG
)]
pub async fn login_handler(
    State(db): State<DatabaseConnection>,
    Extension(logging_info): Extension<LoggingInfo>,
    Json(payload): Json<LoginRequest>,
) -> Result<impl IntoResponse, AppError> {
    // Convert request to user domain model
    let login = UserLogin::new(payload.email, payload.password);

    // Delegate to user service with session tracking
    let auth_token = AuthService::authenticate_user(
        &db,
        login,
        logging_info.user_agent,
        logging_info.ip_address,
    )
    .await?;

    Ok(Json(LoginResponse {
        token: auth_token.token,
    }))
}

/// Logs out the current user
#[utoipa::path(
    post,
    path = "/logout",
    responses(
        (status = 200, description = "Logout successful", body = MessageResponse, examples(
            ("success" = (value = json!({"message": "Logged out successfully"})))
        )),
        (status = 401, description = "Unauthorized - authentication required", body = ErrorResponse),
        (status = 500, description = "Internal server error", body = ErrorResponse)
    ),
    summary = "Logout user",
    description = "Logs out the current user and invalidates their session in the database.",
    tag = AUTH_TAG,
    security(
        ("jwt_token" = [])
    )
)]
pub async fn logout_handler(
    State(db): State<DatabaseConnection>,
    request: Request,
) -> Result<impl IntoResponse, AppError> {
    // Extract token from Authorization header
    let token = TokenService::extract_token_from_header(&request)?;

    // Validate token and extract claims to get session_id
    let claims = TokenService::validate_token_claims(&token)?;

    // Parse session ID
    let session_id = uuid::Uuid::parse_str(&claims.session_id).map_err(|_| AppError {
        message: "Invalid session ID in token".to_string(),
        status_code: StatusCode::UNAUTHORIZED,
    })?;

    // Invalidate the session
    SessionService::invalidate_session(&db, session_id).await?;

    Ok(Json(MessageResponse {
        message: "Logged out successfully".to_string(),
    }))
}

/// Gets the current user's profile information
#[utoipa::path(
    get,
    path = "/profile",
    responses(
        (status = 200, description = "Profile retrieved successfully", body = ProfileResponse),
        (status = 401, description = "Unauthorized - authentication required", body = ErrorResponse, examples(
            ("not_authenticated" = (value = json!({"message": "User not authenticated"})))
        )),
        (status = 404, description = "Not found - user not found", body = ErrorResponse, examples(
            ("user_not_found" = (value = json!({"message": "User not found"})))
        )),
        (status = 500, description = "Internal server error", body = ErrorResponse)
    ),
    summary = "Get user profile",
    description = "Retrieves the authenticated user's profile information including ID, email, and creation timestamp.",
    tag = AUTH_TAG,
    security(
        ("jwt_token" = [])
    )
)]
pub async fn profile_handler(
    State(db): State<DatabaseConnection>,
    request: Request,
) -> Result<impl IntoResponse, AppError> {
    // Extract user from request extensions (set by middleware)
    let auth_user = request.extensions().get::<AuthUser>().ok_or(AppError {
        message: "User not authenticated".to_string(),
        status_code: StatusCode::UNAUTHORIZED,
    })?;

    // Delegate to user service, errors bubble up correctly
    let user = UserService::find_user_by_id(&db, auth_user.user_id)
        .await?
        .ok_or(AppError {
            message: "User not found".to_string(),
            status_code: StatusCode::NOT_FOUND,
        })?;

    Ok(Json(ProfileResponse {
        id: user.id.to_string(),
        email: user.email,
        created_at: user.created_at,
    }))
}

/// Gets the current user's profile information
#[utoipa::path(
    post,
    path = "/verify-email",
    request_body = VerifyEmailRequest,
    responses(
        (status = 200, description = "Email verified successfully", body = VerifyEmailResponse),
        (status = 500, description = "Internal server error", body = ErrorResponse)
    ),
    summary = "Verify email",
    description = "Verifies a user's email address.",
    tag = AUTH_TAG,
    security(
        ("jwt_token" = [])
    )
)]
pub async fn verify_email_handler(
    State(db): State<DatabaseConnection>,
    Json(payload): Json<VerifyEmailRequest>,
) -> Result<impl IntoResponse, AppError> {
    let user_id = uuid::Uuid::parse_str(&payload.user_id).map_err(|_| AppError {
        message: "Invalid user ID".to_string(),
        status_code: StatusCode::BAD_REQUEST,
    })?;
    UserService::verify_email(&db, user_id).await?;

    Ok(Json(VerifyEmailResponse {
        message: "Email verified successfully".to_string(),
        success: true,
    }))
}
