use axum::{
    Json,
    extract::{Extension, Path, Query, State},
    http::StatusCode,
    response::IntoResponse,
};
use sea_orm::DatabaseConnection;
use uuid::Uuid;

use crate::{
    bridge::types::{
        admin::{AdminUser, *},
        auth::AuthUser,
        logging::LoggingInfo,
    },
    check_single_permission,
    control::services::admin_service::AdminService,
    domain::permissions::Permission::{AdminDelete, AdminRead, AdminWrite},
    infrastructure::app_error::{AppError, ErrorResponse, MessageResponse},
};

/// Admin login endpoint
#[utoipa::path(
    post,
    path = "/login",
    request_body = AdminLoginRequest,
    responses(
        (status = 200, description = "Admin login successful", body = AdminLoginResponse),
        (status = 400, description = "Bad request - validation errors", body = ErrorResponse),
        (status = 401, description = "Unauthorized - invalid credentials", body = ErrorResponse),
        (status = 403, description = "Forbidden - admin privileges required", body = ErrorResponse),
        (status = 500, description = "Internal server error", body = ErrorResponse)
    ),
    summary = "Admin login",
    description = "Authenticates an admin user and returns a JWT token. Requires admin:read permission.",
    tag = ADMIN_TAG
)]
pub async fn admin_login_handler(
    State(db): State<DatabaseConnection>,
    Extension(logging_info): Extension<LoggingInfo>,
    Json(payload): Json<AdminLoginRequest>,
) -> Result<impl IntoResponse, AppError> {
    check_single_permission!(&payload.email, &AdminRead, &db);
    let response = AdminService::authenticate_admin(
        &db,
        payload,
        logging_info.user_agent,
        logging_info.ip_address,
    )
    .await?;
    Ok((StatusCode::OK, Json(response)))
}

/// Admin logout endpoint
#[utoipa::path(
    post,
    path = "/logout",
    responses(
        (status = 200, description = "Admin logout successful", body = MessageResponse),
        (status = 401, description = "Unauthorized - authentication required", body = ErrorResponse),
        (status = 500, description = "Internal server error", body = ErrorResponse)
    ),
    summary = "Admin logout",
    description = "Logs out the current admin user and invalidates their session",
    tag = ADMIN_TAG,
    security(
        ("jwt_token" = [])
    )
)]
pub async fn admin_logout_handler(
    State(db): State<DatabaseConnection>,
    request: axum::extract::Request,
) -> Result<impl IntoResponse, AppError> {
    // Extract token from Authorization header
    let token =
        crate::control::services::token_service::TokenService::extract_token_from_header(&request)?;

    // Validate token and extract claims to get session_id
    let claims =
        crate::control::services::token_service::TokenService::validate_token_claims(&token)?;

    // Parse session ID
    let session_id = uuid::Uuid::parse_str(&claims.session_id).map_err(|_| AppError {
        message: "Invalid session ID in token".to_string(),
        status_code: StatusCode::UNAUTHORIZED,
    })?;

    // Invalidate the session
    crate::control::services::session_service::SessionService::invalidate_session(&db, session_id)
        .await?;

    Ok((
        StatusCode::OK,
        Json(MessageResponse {
            message: "Admin logged out successfully".to_string(),
        }),
    ))
}

/// Get audit logs endpoint
#[utoipa::path(
    get,
    path = "/logs",
    params(LogsQueryParams),
    responses(
        (status = 200, description = "Audit logs retrieved successfully", body = PaginatedResponse<AuditLogResponse>),
        (status = 401, description = "Unauthorized - authentication required", body = ErrorResponse),
        (status = 403, description = "Forbidden - admin privileges required", body = ErrorResponse),
        (status = 500, description = "Internal server error", body = ErrorResponse)
    ),
    summary = "Get audit logs",
    description = "Retrieves paginated audit logs with optional filtering. Requires admin:read permission.",
    tag = ADMIN_TAG,
    security(
        ("jwt_token" = [])
    )
)]
pub async fn get_audit_logs_handler(
    State(db): State<DatabaseConnection>,
    Extension(admin_user): Extension<AdminUser>,
    Query(params): Query<LogsQueryParams>,
) -> Result<impl IntoResponse, AppError> {
    check_single_permission!(&admin_user.email, &AdminRead, &db);
    let response = AdminService::get_audit_logs(&db, params).await?;
    Ok((StatusCode::OK, Json(response)))
}

/// Get users endpoint
#[utoipa::path(
    get,
    path = "/users",
    params(UsersQueryParams),
    responses(
        (status = 200, description = "Users retrieved successfully", body = PaginatedResponse<UserResponse>),
        (status = 401, description = "Unauthorized - authentication required", body = ErrorResponse),
        (status = 403, description = "Forbidden - admin privileges required", body = ErrorResponse),
        (status = 500, description = "Internal server error", body = ErrorResponse)
    ),
    summary = "Get users",
    description = "Retrieves paginated users with optional filtering",
    tag = ADMIN_TAG,
    security(
        ("jwt_token" = [])
    )
)]
pub async fn get_users_handler(
    State(db): State<DatabaseConnection>,
    Extension(admin_user): Extension<AdminUser>,
    Query(params): Query<UsersQueryParams>,
) -> Result<impl IntoResponse, AppError> {
    check_single_permission!(&admin_user.email, &AdminRead, &db);
    let response = AdminService::get_users(&db, params).await?;
    Ok((StatusCode::OK, Json(response)))
}

/// Get specific user endpoint
#[utoipa::path(
    get,
    path = "/users/{id}",
    params(
        ("id" = String, Path, description = "User ID")
    ),
    responses(
        (status = 200, description = "User retrieved successfully", body = UserResponse),
        (status = 401, description = "Unauthorized - authentication required", body = ErrorResponse),
        (status = 403, description = "Forbidden - admin privileges required", body = ErrorResponse),
        (status = 404, description = "User not found", body = ErrorResponse),
        (status = 500, description = "Internal server error", body = ErrorResponse)
    ),
    summary = "Get user",
    description = "Retrieves a specific user by ID",
    tag = ADMIN_TAG,
    security(
        ("jwt_token" = [])
    )
)]
pub async fn get_user_handler(
    State(db): State<DatabaseConnection>,
    Extension(admin_user): Extension<AdminUser>,
    Path(user_id): Path<String>,
) -> Result<impl IntoResponse, AppError> {
    check_single_permission!(&admin_user.email, &AdminRead, &db);
    let user_id = Uuid::parse_str(&user_id).map_err(|_| AppError {
        message: "Invalid user ID format".to_string(),
        status_code: StatusCode::BAD_REQUEST,
    })?;

    let response = AdminService::get_user(&db, user_id).await?;
    Ok((StatusCode::OK, Json(response)))
}

/// Create user endpoint
#[utoipa::path(
    post,
    path = "/users",
    request_body = CreateUserRequest,
    responses(
        (status = 201, description = "User created successfully", body = UserResponse),
        (status = 400, description = "Bad request - validation errors", body = ErrorResponse),
        (status = 401, description = "Unauthorized - authentication required", body = ErrorResponse),
        (status = 403, description = "Forbidden - admin privileges required", body = ErrorResponse),
        (status = 409, description = "Conflict - user already exists", body = ErrorResponse),
        (status = 500, description = "Internal server error", body = ErrorResponse)
    ),
    summary = "Create user",
    description = "Creates a new user account",
    tag = ADMIN_TAG,
    security(
        ("jwt_token" = [])
    )
)]
pub async fn create_user_handler(
    State(db): State<DatabaseConnection>,
    Extension(admin_user): Extension<AdminUser>,
    Json(payload): Json<CreateUserRequest>,
) -> Result<impl IntoResponse, AppError> {
    check_single_permission!(&admin_user.email, &AdminWrite, &db);
    let response = AdminService::create_user(&db, payload).await?;
    Ok((StatusCode::CREATED, Json(response)))
}

/// Update user endpoint
#[utoipa::path(
    put,
    path = "/users/{id}",
    params(
        ("id" = String, Path, description = "User ID")
    ),
    request_body = UpdateUserRequest,
    responses(
        (status = 200, description = "User updated successfully", body = UserResponse),
        (status = 400, description = "Bad request - validation errors", body = ErrorResponse),
        (status = 401, description = "Unauthorized - authentication required", body = ErrorResponse),
        (status = 403, description = "Forbidden - admin privileges required", body = ErrorResponse),
        (status = 404, description = "User not found", body = ErrorResponse),
        (status = 409, description = "Conflict - email already taken", body = ErrorResponse),
        (status = 500, description = "Internal server error", body = ErrorResponse)
    ),
    summary = "Update user",
    description = "Updates an existing user account",
    tag = ADMIN_TAG,
    security(
        ("jwt_token" = [])
    )
)]
pub async fn update_user_handler(
    State(db): State<DatabaseConnection>,
    Extension(admin_user): Extension<AdminUser>,
    Path(user_id): Path<String>,
    Json(payload): Json<UpdateUserRequest>,
) -> Result<impl IntoResponse, AppError> {
    check_single_permission!(&admin_user.email, &AdminWrite, &db);
    let user_id = Uuid::parse_str(&user_id).map_err(|_| AppError {
        message: "Invalid user ID format".to_string(),
        status_code: StatusCode::BAD_REQUEST,
    })?;

    let response = AdminService::update_user(&db, user_id, payload).await?;
    Ok((StatusCode::OK, Json(response)))
}

/// Delete user endpoint
#[utoipa::path(
    delete,
    path = "/users/{id}",
    params(
        ("id" = String, Path, description = "User ID")
    ),
    responses(
        (status = 200, description = "User deleted successfully", body = MessageResponse),
        (status = 400, description = "Bad request - cannot delete own account", body = ErrorResponse),
        (status = 401, description = "Unauthorized - authentication required", body = ErrorResponse),
        (status = 403, description = "Forbidden - admin privileges required", body = ErrorResponse),
        (status = 404, description = "User not found", body = ErrorResponse),
        (status = 500, description = "Internal server error", body = ErrorResponse)
    ),
    summary = "Delete user",
    description = "Deletes a user account",
    tag = ADMIN_TAG,
    security(
        ("jwt_token" = [])
    )
)]
pub async fn delete_user_handler(
    State(db): State<DatabaseConnection>,
    Extension(admin_user): Extension<AdminUser>,
    Path(user_id): Path<String>,
    request: axum::extract::Request,
) -> Result<impl IntoResponse, AppError> {
    check_single_permission!(&admin_user.email, &AdminDelete, &db);
    let user_id = Uuid::parse_str(&user_id).map_err(|_| AppError {
        message: "Invalid user ID format".to_string(),
        status_code: StatusCode::BAD_REQUEST,
    })?;

    // Get current admin ID from request extensions
    let auth_user = request.extensions().get::<AuthUser>().ok_or(AppError {
        message: "User not authenticated".to_string(),
        status_code: StatusCode::UNAUTHORIZED,
    })?;

    AdminService::delete_user(&db, user_id, auth_user.user_id).await?;
    Ok((
        StatusCode::OK,
        Json(MessageResponse {
            message: "User deleted successfully".to_string(),
        }),
    ))
}

/// Get database tables endpoint
#[utoipa::path(
    get,
    path = "/database/tables",
    responses(
        (status = 200, description = "Database tables retrieved successfully", body = Vec<DatabaseTableResponse>),
        (status = 401, description = "Unauthorized - authentication required", body = ErrorResponse),
        (status = 403, description = "Forbidden - admin privileges required", body = ErrorResponse),
        (status = 500, description = "Internal server error", body = ErrorResponse)
    ),
    summary = "Get database tables",
    description = "Retrieves a list of all database tables with record counts",
    tag = ADMIN_TAG,
    security(
        ("jwt_token" = [])
    )
)]
pub async fn get_database_tables_handler(
    State(db): State<DatabaseConnection>,
    Extension(admin_user): Extension<AdminUser>,
) -> Result<impl IntoResponse, AppError> {
    check_single_permission!(&admin_user.email, &AdminRead, &db);
    let response = AdminService::get_database_tables(&db).await?;
    Ok((StatusCode::OK, Json(response)))
}

/// Get table records endpoint
#[utoipa::path(
    get,
    path = "/database/tables/{table_name}",
    params(
        ("table_name" = String, Path, description = "Table name"),
        TableRecordsQueryParams
    ),
    responses(
        (status = 200, description = "Table records retrieved successfully", body = TableRecordResponse),
        (status = 401, description = "Unauthorized - authentication required", body = ErrorResponse),
        (status = 403, description = "Forbidden - admin privileges required", body = ErrorResponse),
        (status = 500, description = "Internal server error", body = ErrorResponse)
    ),
    summary = "Get table records",
    description = "Retrieves paginated records from a specific database table",
    tag = ADMIN_TAG,
    security(
        ("jwt_token" = [])
    )
)]
pub async fn get_table_records_handler(
    State(db): State<DatabaseConnection>,
    Extension(admin_user): Extension<AdminUser>,
    Path(table_name): Path<String>,
    Query(params): Query<TableRecordsQueryParams>,
) -> Result<impl IntoResponse, AppError> {
    check_single_permission!(&admin_user.email, &AdminRead, &db);
    let response = AdminService::get_table_records(&db, table_name, params).await?;
    Ok((StatusCode::OK, Json(response)))
}

/// System health endpoint
#[utoipa::path(
    get,
    path = "/health",
    responses(
        (status = 200, description = "System health check successful", body = HealthResponse),
        (status = 401, description = "Unauthorized - authentication required", body = ErrorResponse),
        (status = 403, description = "Forbidden - admin privileges required", body = ErrorResponse)
    ),
    summary = "System health check",
    description = "Returns system health status",
    tag = ADMIN_TAG,
    security(
        ("jwt_token" = [])
    )
)]
pub async fn health_handler(
    State(db): State<DatabaseConnection>,
    Extension(admin_user): Extension<AdminUser>,
) -> Result<impl IntoResponse, AppError> {
    check_single_permission!(&admin_user.email, &AdminRead, &db);
    let response = AdminService::get_health_status(&db).await;
    Ok((StatusCode::OK, Json(response)))
}

/// Get user sessions endpoint
#[utoipa::path(
    get,
    path = "/users/{user_id}/sessions",
    params(
        ("user_id" = String, Path, description = "User ID")
    ),
    responses(
        (status = 200, description = "User sessions retrieved successfully", body = Vec<SessionResponse>),
        (status = 401, description = "Unauthorized", body = ErrorResponse),
        (status = 403, description = "Forbidden - admin privileges required", body = ErrorResponse),
        (status = 404, description = "User not found", body = ErrorResponse),
        (status = 500, description = "Internal server error", body = ErrorResponse)
    ),
    summary = "Get user sessions",
    description = "Retrieves all active sessions for a specific user",
    tag = ADMIN_TAG,
    security(
        ("jwt_token" = [])
    )
)]
pub async fn get_user_sessions_handler(
    State(db): State<DatabaseConnection>,
    Path(user_id): Path<String>,
    Extension(admin_user): Extension<AdminUser>,
) -> Result<impl IntoResponse, AppError> {
    check_single_permission!(&admin_user.email, &AdminRead, &db);

    let user_uuid = Uuid::parse_str(&user_id).map_err(|_| AppError {
        message: "Invalid user ID format".to_string(),
        status_code: StatusCode::BAD_REQUEST,
    })?;

    let sessions = AdminService::get_user_sessions(&db, user_uuid).await?;
    Ok((StatusCode::OK, Json(sessions)))
}

/// Invalidate specific session endpoint
#[utoipa::path(
    delete,
    path = "/sessions/{session_id}",
    params(
        ("session_id" = String, Path, description = "Session ID")
    ),
    responses(
        (status = 200, description = "Session invalidated successfully", body = SessionInvalidationResponse),
        (status = 401, description = "Unauthorized", body = ErrorResponse),
        (status = 403, description = "Forbidden - admin privileges required", body = ErrorResponse),
        (status = 404, description = "Session not found", body = ErrorResponse),
        (status = 500, description = "Internal server error", body = ErrorResponse)
    ),
    summary = "Invalidate session",
    description = "Invalidates a specific user session (remote logout)",
    tag = ADMIN_TAG,
    security(
        ("jwt_token" = [])
    )
)]
pub async fn invalidate_session_handler(
    State(db): State<DatabaseConnection>,
    Path(session_id): Path<String>,
    Extension(admin_user): Extension<AdminUser>,
) -> Result<impl IntoResponse, AppError> {
    check_single_permission!(&admin_user.email, &AdminDelete, &db);

    let session_uuid = Uuid::parse_str(&session_id).map_err(|_| AppError {
        message: "Invalid session ID format".to_string(),
        status_code: StatusCode::BAD_REQUEST,
    })?;

    AdminService::invalidate_user_session(&db, session_uuid).await?;

    let response = SessionInvalidationResponse {
        message: "Session invalidated successfully".to_string(),
        invalidated_count: Some(1),
    };

    Ok((StatusCode::OK, Json(response)))
}

/// Invalidate all user sessions endpoint
#[utoipa::path(
    delete,
    path = "/users/{user_id}/sessions",
    params(
        ("user_id" = String, Path, description = "User ID")
    ),
    responses(
        (status = 200, description = "All user sessions invalidated successfully", body = SessionInvalidationResponse),
        (status = 401, description = "Unauthorized", body = ErrorResponse),
        (status = 403, description = "Forbidden - admin privileges required", body = ErrorResponse),
        (status = 404, description = "User not found", body = ErrorResponse),
        (status = 500, description = "Internal server error", body = ErrorResponse)
    ),
    summary = "Invalidate all user sessions",
    description = "Invalidates all sessions for a specific user (force logout from all devices)",
    tag = ADMIN_TAG,
    security(
        ("jwt_token" = [])
    )
)]
pub async fn invalidate_all_user_sessions_handler(
    State(db): State<DatabaseConnection>,
    Path(user_id): Path<String>,
    Extension(admin_user): Extension<AdminUser>,
) -> Result<impl IntoResponse, AppError> {
    check_single_permission!(&admin_user.email, &AdminDelete, &db);

    let user_uuid = Uuid::parse_str(&user_id).map_err(|_| AppError {
        message: "Invalid user ID format".to_string(),
        status_code: StatusCode::BAD_REQUEST,
    })?;

    let count = AdminService::invalidate_all_user_sessions(&db, user_uuid).await?;

    let response = SessionInvalidationResponse {
        message: format!("All user sessions invalidated successfully"),
        invalidated_count: Some(count),
    };

    Ok((StatusCode::OK, Json(response)))
}
