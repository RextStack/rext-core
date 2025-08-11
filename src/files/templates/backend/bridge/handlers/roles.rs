use axum::{
    Json,
    extract::{Path, Query, State},
    http::StatusCode,
    response::IntoResponse,
};
use sea_orm::DatabaseConnection;

use crate::{
    bridge::types::admin::*,
    control::services::admin_service::AdminService,
    infrastructure::app_error::{AppError, ErrorResponse, MessageResponse},
};

/// Get roles endpoint
#[utoipa::path(
    get,
    path = "/roles",
    params(RolesQueryParams),
    responses(
        (status = 200, description = "Roles retrieved successfully", body = PaginatedResponse<RoleResponse>),
        (status = 401, description = "Unauthorized - authentication required", body = ErrorResponse),
        (status = 403, description = "Forbidden - admin privileges required", body = ErrorResponse),
        (status = 500, description = "Internal server error", body = ErrorResponse)
    ),
    summary = "Get roles",
    description = "Retrieves paginated roles with optional filtering",
    tag = ADMIN_TAG,
    security(
        ("jwt_token" = [])
    )
)]
pub async fn get_roles_handler(
    State(db): State<DatabaseConnection>,
    Query(params): Query<RolesQueryParams>,
) -> Result<impl IntoResponse, AppError> {
    let response = AdminService::get_roles(&db, params).await?;
    Ok((StatusCode::OK, Json(response)))
}

/// Get role by ID endpoint
#[utoipa::path(
    get,
    path = "/roles/{id}",
    params(
        ("id" = i32, Path, description = "Role ID")
    ),
    responses(
        (status = 200, description = "Role retrieved successfully", body = RoleResponse),
        (status = 401, description = "Unauthorized - authentication required", body = ErrorResponse),
        (status = 403, description = "Forbidden - admin privileges required", body = ErrorResponse),
        (status = 404, description = "Role not found", body = ErrorResponse),
        (status = 500, description = "Internal server error", body = ErrorResponse)
    ),
    summary = "Get role by ID",
    description = "Retrieves a specific role by its ID",
    tag = ADMIN_TAG,
    security(
        ("jwt_token" = [])
    )
)]
pub async fn get_role_handler(
    State(db): State<DatabaseConnection>,
    Path(role_id): Path<i32>,
) -> Result<impl IntoResponse, AppError> {
    let response = AdminService::get_role(&db, role_id).await?;
    Ok((StatusCode::OK, Json(response)))
}

/// Create role endpoint
#[utoipa::path(
    post,
    path = "/roles",
    request_body = CreateRoleRequest,
    responses(
        (status = 201, description = "Role created successfully", body = RoleResponse),
        (status = 400, description = "Bad request - validation errors", body = ErrorResponse),
        (status = 401, description = "Unauthorized - authentication required", body = ErrorResponse),
        (status = 403, description = "Forbidden - admin privileges required", body = ErrorResponse),
        (status = 409, description = "Conflict - role name already exists", body = ErrorResponse),
        (status = 500, description = "Internal server error", body = ErrorResponse)
    ),
    summary = "Create role",
    description = "Creates a new role with specified permissions",
    tag = ADMIN_TAG,
    security(
        ("jwt_token" = [])
    )
)]
pub async fn create_role_handler(
    State(db): State<DatabaseConnection>,
    Json(payload): Json<CreateRoleRequest>,
) -> Result<impl IntoResponse, AppError> {
    let response = AdminService::create_role(&db, payload).await?;
    Ok((StatusCode::CREATED, Json(response)))
}

/// Update role endpoint
#[utoipa::path(
    put,
    path = "/roles/{id}",
    params(
        ("id" = i32, Path, description = "Role ID")
    ),
    request_body = UpdateRoleRequest,
    responses(
        (status = 200, description = "Role updated successfully", body = RoleResponse),
        (status = 400, description = "Bad request - validation errors", body = ErrorResponse),
        (status = 401, description = "Unauthorized - authentication required", body = ErrorResponse),
        (status = 403, description = "Forbidden - admin privileges required", body = ErrorResponse),
        (status = 404, description = "Role not found", body = ErrorResponse),
        (status = 409, description = "Conflict - role name already exists", body = ErrorResponse),
        (status = 500, description = "Internal server error", body = ErrorResponse)
    ),
    summary = "Update role",
    description = "Updates an existing role with new permissions",
    tag = ADMIN_TAG,
    security(
        ("jwt_token" = [])
    )
)]
pub async fn update_role_handler(
    State(db): State<DatabaseConnection>,
    Path(role_id): Path<i32>,
    Json(payload): Json<UpdateRoleRequest>,
) -> Result<impl IntoResponse, AppError> {
    let response = AdminService::update_role(&db, role_id, payload).await?;
    Ok((StatusCode::OK, Json(response)))
}

/// Delete role endpoint
#[utoipa::path(
    delete,
    path = "/roles/{id}",
    params(
        ("id" = i32, Path, description = "Role ID")
    ),
    responses(
        (status = 200, description = "Role deleted successfully", body = MessageResponse),
        (status = 401, description = "Unauthorized - authentication required", body = ErrorResponse),
        (status = 403, description = "Forbidden - admin privileges required", body = ErrorResponse),
        (status = 404, description = "Role not found", body = ErrorResponse),
        (status = 409, description = "Conflict - role is in use by users", body = ErrorResponse),
        (status = 500, description = "Internal server error", body = ErrorResponse)
    ),
    summary = "Delete role",
    description = "Deletes a role if it's not assigned to any users",
    tag = ADMIN_TAG,
    security(
        ("jwt_token" = [])
    )
)]
pub async fn delete_role_handler(
    State(db): State<DatabaseConnection>,
    Path(role_id): Path<i32>,
) -> Result<impl IntoResponse, AppError> {
    AdminService::delete_role(&db, role_id).await?;
    Ok((
        StatusCode::OK,
        Json(MessageResponse {
            message: "Role deleted successfully".to_string(),
        }),
    ))
}

/// Check permission endpoint
#[utoipa::path(
    post,
    path = "/permissions/check",
    request_body = PermissionCheckRequest,
    responses(
        (status = 200, description = "Permission check completed", body = PermissionCheckResponse),
        (status = 400, description = "Bad request - validation errors", body = ErrorResponse),
        (status = 401, description = "Unauthorized - authentication required", body = ErrorResponse),
        (status = 403, description = "Forbidden - admin privileges required", body = ErrorResponse),
        (status = 500, description = "Internal server error", body = ErrorResponse)
    ),
    summary = "Check permission",
    description = "Checks if a user has a specific permission based on their role",
    tag = ADMIN_TAG,
    security(
        ("jwt_token" = [])
    )
)]
pub async fn check_permission_handler(
    State(db): State<DatabaseConnection>,
    Json(payload): Json<PermissionCheckRequest>,
) -> Result<impl IntoResponse, AppError> {
    let response = AdminService::check_permission(&db, payload).await?;
    Ok((StatusCode::OK, Json(response)))
}
