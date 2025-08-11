use chrono::{Duration, Utc};
use sea_orm::prelude::Expr;
use sea_orm::*;
use uuid::Uuid;

use crate::control::services::database_service::DatabaseService;
use crate::entity::models::{prelude::*, user_sessions};
use crate::infrastructure::app_error::AppError;
use axum::http::StatusCode;

/// Service for session-related business operations
pub struct SessionService;

impl SessionService {
    /// Creates a new session on login
    pub async fn create_session(
        db: &DatabaseConnection,
        user_id: Uuid,
        user_agent: Option<String>,
        ip_address: Option<String>,
        session_token: &str,
    ) -> Result<user_sessions::Model, AppError> {
        // Use the session token directly (UUID from JWT claims)
        let session_token_str = session_token.to_string();

        // Calculate expiration time (24 hours from now)
        let expires_at = Utc::now() + Duration::hours(24);

        // Create session ID
        let session_id = Uuid::new_v4();

        // Create session active model
        let session_active_model = user_sessions::ActiveModel {
            id: Set(session_id),
            user_id: Set(user_id),
            session_token: Set(session_token_str),
            user_agent: Set(user_agent),
            ip_address: Set(ip_address),
            created_at: Set(Some(Utc::now().fixed_offset())),
            last_activity: Set(Some(Utc::now().fixed_offset())),
            expires_at: Set(expires_at.fixed_offset()),
            is_active: Set(true),
        };

        // Insert into database
        let session = UserSessions::insert(session_active_model)
            .exec_with_returning(db)
            .await
            .map_err(|e| AppError {
                message: format!("Failed to create session: {}", e),
                status_code: StatusCode::INTERNAL_SERVER_ERROR,
            })?;

        Ok(session)
    }

    /// Validates that a session exists and is active
    pub async fn validate_session(
        db: &DatabaseConnection,
        session_token: &str,
    ) -> Result<user_sessions::Model, AppError> {
        // Find session by session token
        let session = DatabaseService::find_one_with_tracking(
            db,
            "user_sessions",
            UserSessions::find().filter(user_sessions::Column::SessionToken.eq(session_token)),
        )
        .await
        .map_err(|e| AppError {
            message: format!("Database error: {}", e),
            status_code: StatusCode::INTERNAL_SERVER_ERROR,
        })?
        .ok_or(AppError {
            message: "Session not found".to_string(),
            status_code: StatusCode::UNAUTHORIZED,
        })?;

        // Check if session is active
        if !session.is_active {
            return Err(AppError {
                message: "Session has been invalidated".to_string(),
                status_code: StatusCode::UNAUTHORIZED,
            });
        }

        // Check if session is expired
        let now = Utc::now();
        if session.expires_at.to_utc() < now {
            return Err(AppError {
                message: "Session expired".to_string(),
                status_code: StatusCode::UNAUTHORIZED,
            });
        }

        Ok(session)
    }

    /// Updates session activity timestamp
    pub async fn update_session_activity(
        db: &DatabaseConnection,
        session_id: Uuid,
    ) -> Result<(), AppError> {
        // Find the session by session_token (not by id) since the session_id from JWT
        // is stored in the session_token field, while the id field is a different UUID
        let session = DatabaseService::find_one_with_tracking(
            db,
            "user_sessions",
            UserSessions::find()
                .filter(user_sessions::Column::SessionToken.eq(session_id.to_string())),
        )
        .await
        .map_err(|e| AppError {
            message: format!("Database error: {}", e),
            status_code: StatusCode::INTERNAL_SERVER_ERROR,
        })?
        .ok_or(AppError {
            message: "Session not found".to_string(),
            status_code: StatusCode::UNAUTHORIZED,
        })?;

        // Update the found session's last activity
        let session_active_model = user_sessions::ActiveModel {
            id: Set(session.id),
            last_activity: Set(Some(Utc::now().fixed_offset())),
            ..Default::default()
        };

        session_active_model
            .update(db)
            .await
            .map_err(|e| AppError {
                message: format!("Failed to update session activity: {}", e),
                status_code: StatusCode::INTERNAL_SERVER_ERROR,
            })?;

        Ok(())
    }

    /// Gets active sessions for a user
    pub async fn get_user_sessions(
        db: &DatabaseConnection,
        user_id: Uuid,
    ) -> Result<Vec<user_sessions::Model>, AppError> {
        let sessions = DatabaseService::find_all_with_tracking(
            db,
            "user_sessions",
            UserSessions::find()
                .filter(user_sessions::Column::UserId.eq(user_id))
                .filter(user_sessions::Column::IsActive.eq(true))
                .order_by_desc(user_sessions::Column::LastActivity),
        )
        .await
        .map_err(|e| AppError {
            message: format!("Database error: {}", e),
            status_code: StatusCode::INTERNAL_SERVER_ERROR,
        })?;

        Ok(sessions)
    }

    /// Invalidates a specific session (for admin remote logout)
    pub async fn invalidate_session(
        db: &DatabaseConnection,
        session_id: Uuid,
    ) -> Result<(), AppError> {
        // Find the session by session_token (not by id) since the session_id from JWT
        // is stored in the session_token field, while the id field is a different UUID
        let session = DatabaseService::find_one_with_tracking(
            db,
            "user_sessions",
            UserSessions::find()
                .filter(user_sessions::Column::SessionToken.eq(session_id.to_string())),
        )
        .await
        .map_err(|e| AppError {
            message: format!("Database error: {}", e),
            status_code: StatusCode::INTERNAL_SERVER_ERROR,
        })?
        .ok_or(AppError {
            message: "Session not found".to_string(),
            status_code: StatusCode::UNAUTHORIZED,
        })?;

        // Update the found session to set is_active = false
        let session_active_model = user_sessions::ActiveModel {
            id: Set(session.id),
            is_active: Set(false),
            ..Default::default()
        };

        session_active_model
            .update(db)
            .await
            .map_err(|e| AppError {
                message: format!("Failed to invalidate session: {}", e),
                status_code: StatusCode::INTERNAL_SERVER_ERROR,
            })?;

        Ok(())
    }

    /// Invalidates all sessions for a user
    pub async fn invalidate_all_user_sessions(
        db: &DatabaseConnection,
        user_id: Uuid,
    ) -> Result<u64, AppError> {
        let result = UserSessions::update_many()
            .col_expr(user_sessions::Column::IsActive, Expr::value(false))
            .filter(user_sessions::Column::UserId.eq(user_id))
            .filter(user_sessions::Column::IsActive.eq(true))
            .exec(db)
            .await
            .map_err(|e| AppError {
                message: format!("Failed to invalidate user sessions: {}", e),
                status_code: StatusCode::INTERNAL_SERVER_ERROR,
            })?;

        Ok(result.rows_affected)
    }

    /// Cleanup expired sessions (background task)
    #[allow(dead_code)]
    pub async fn cleanup_expired_sessions(db: &DatabaseConnection) -> Result<u64, AppError> {
        let now = Utc::now();

        let result = UserSessions::delete_many()
            .filter(user_sessions::Column::ExpiresAt.lt(now.fixed_offset()))
            .exec(db)
            .await
            .map_err(|e| AppError {
                message: format!("Failed to cleanup expired sessions: {}", e),
                status_code: StatusCode::INTERNAL_SERVER_ERROR,
            })?;

        Ok(result.rows_affected)
    }

    /// Gets active session count for a user
    #[allow(dead_code)]
    pub async fn get_user_active_session_count(
        db: &DatabaseConnection,
        user_id: Uuid,
    ) -> Result<u64, AppError> {
        let count = UserSessions::find()
            .filter(user_sessions::Column::UserId.eq(user_id))
            .filter(user_sessions::Column::IsActive.eq(true))
            .filter(user_sessions::Column::ExpiresAt.gt(Utc::now().fixed_offset()))
            .count(db)
            .await
            .map_err(|e| AppError {
                message: format!("Database error: {}", e),
                status_code: StatusCode::INTERNAL_SERVER_ERROR,
            })?;

        Ok(count)
    }
}
