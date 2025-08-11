use axum::{
    extract::{Request, State},
    http::StatusCode,
    middleware::Next,
    response::Response,
};
use sea_orm::DatabaseConnection;
use tracing::{info, warn};

use crate::{
    bridge::types::{admin::AdminUser, auth::AuthUser},
    control::services::{
        session_service::SessionService, token_service::TokenService, user_service::UserService,
    },
    infrastructure::{app_error::AppError, logging::LoggingManager},
};

/// Admin middleware that handles JWT extraction and validation; no permission checking here, all done at the handler
pub async fn admin_middleware(
    State(db): State<DatabaseConnection>,
    mut request: Request,
    next: Next,
) -> Result<Response, AppError> {
    let request_id = LoggingManager::generate_request_id();

    // Extract token from Authorization header
    let token = TokenService::extract_token_from_header(&request)?;

    // Extract and validate token with session validation
    let (user_id, session_id) =
        TokenService::extract_and_validate_token_with_session(&db, &token).await?;

    // Update session activity (fire and forget)
    let db_clone = db.clone();
    tokio::spawn(async move {
        let _ = SessionService::update_session_activity(&db_clone, session_id).await;
    });

    let user = UserService::find_user_by_id(&db, user_id)
        .await?
        .ok_or(AppError {
            message: "User not found".to_string(),
            status_code: StatusCode::NOT_FOUND,
        })?;

    info!(
        request_id = %request_id,
        user_id = %user_id,
        email = %user.email,
        "Admin access granted"
    );

    // Add both AuthUser and AdminUser to request extensions for downstream handlers
    request.extensions_mut().insert(AuthUser { user_id });

    request.extensions_mut().insert(AdminUser {
        user_id,
        email: user.email.clone(),
    });

    Ok(next.run(request).await)
}

/// Log admin access attempts (for security monitoring)
pub fn _log_admin_access_attempt(
    user_id: Option<uuid::Uuid>,
    success: bool,
    path: &str,
    ip_address: Option<&str>,
) {
    if success {
        info!(
            user_id = ?user_id,
            path = %path,
            ip_address = ?ip_address,
            "Admin access granted"
        );
    } else {
        warn!(
            user_id = ?user_id,
            path = %path,
            ip_address = ?ip_address,
            "Admin access denied"
        );
    }
}
