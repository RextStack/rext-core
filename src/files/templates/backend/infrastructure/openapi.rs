use utoipa::OpenApi;

use crate::bridge::types::admin::{
    ADMIN_TAG, AdminLoginRequest, AdminLoginResponse, AuditLogResponse, CreateUserRequest,
    DatabaseTableResponse, HealthResponse, LogsQueryParams, PaginatedResponse, PaginationMeta,
    TableRecordResponse, TableRecordsQueryParams, UpdateUserRequest, UserResponse,
    UsersQueryParams,
};
use crate::bridge::types::auth::{
    AUTH_TAG, AuthUser, LoginRequest, LoginResponse, ProfileResponse, RegisterRequest,
    RegisterResponse,
};
use crate::infrastructure::app_error::{ErrorResponse, MessageResponse};

/// OpenAPI documentation structure
#[derive(OpenApi)]
#[openapi(
    info(
        title = "Rext Example API",
        version = "1.0.0",
        description = "A full-stack web application API built with Rust and Axum",
        contact(
            name = "Rext Team",
            email = "team@rext.dev"
        )
    ),
    servers(
        (url = "http://localhost:3000", description = "Development server"),
    ),
    tags(
        (name = AUTH_TAG, description = "Authentication endpoints"),
        (name = ADMIN_TAG, description = "Admin panel endpoints")
    ),
    components(
        schemas(
            RegisterRequest, RegisterResponse, LoginRequest, LoginResponse, ProfileResponse, AuthUser,
            MessageResponse, ErrorResponse,
            AdminLoginRequest, AdminLoginResponse, AuditLogResponse,
            LogsQueryParams, UsersQueryParams, CreateUserRequest, UpdateUserRequest, UserResponse,
            DatabaseTableResponse, TableRecordsQueryParams, TableRecordResponse, HealthResponse,
            PaginatedResponse<AuditLogResponse>, PaginationMeta
        )
    ),
    security(
        ("jwt_token" = [])
    )
)]
pub struct ApiDoc;
