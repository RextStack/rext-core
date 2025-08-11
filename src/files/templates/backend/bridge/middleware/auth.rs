use axum::{extract::Request, extract::State, middleware::Next, response::Response};
use sea_orm::DatabaseConnection;

use crate::{
    bridge::types::auth::AuthUser,
    control::services::{session_service::SessionService, token_service::TokenService},
    infrastructure::app_error::AppError,
};

// JWT middleware with session validation
pub async fn auth_middleware(
    State(db): State<DatabaseConnection>,
    mut request: Request,
    next: Next,
) -> Result<Response, AppError> {
    let token = TokenService::extract_token_from_header(&request)?;

    // Extract and validate token with session validation
    let (user_id, session_id) =
        TokenService::extract_and_validate_token_with_session(&db, &token).await?;
    // let user_id = TokenService::extract_and_validate_token(&request)?;

    // Update session activity (fire and forget)
    let db_clone = db.clone();
    tokio::spawn(async move {
        let _ = SessionService::update_session_activity(&db_clone, session_id).await;
    });

    // Add user to request extensions
    request.extensions_mut().insert(AuthUser { user_id });

    Ok(next.run(request).await)
}
