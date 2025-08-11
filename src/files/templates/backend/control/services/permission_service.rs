//! Permission service
//!
//! Business logic to manage permissions.
//! Doesn't go in infrastructure, since it's used in the bridge layer with a handler as well. I guess.
//! TODO implement these services at handler level for granular control.

use sea_orm::*;
use serde_json;
use uuid::Uuid;

use crate::{
    control::services::database_service::DatabaseService,
    domain::permissions::{Permission, PermissionSet},
    entity::models::{roles, users},
    infrastructure::app_error::AppError,
};
use axum::http::StatusCode;

/// Service for permission-related business operations
#[allow(dead_code)]
pub struct PermissionService;

impl PermissionService {
    /// Check if a user has a specific permission
    #[allow(dead_code)]
    pub async fn has_permission(
        db: &DatabaseConnection,
        user_id: Uuid,
        permission: &Permission,
    ) -> Result<bool, AppError> {
        let user = DatabaseService::find_one_with_tracking(
            db,
            "users",
            users::Entity::find_by_id(user_id),
        )
        .await
        .map_err(|e| AppError {
            message: format!("Database error: {}", e),
            status_code: StatusCode::INTERNAL_SERVER_ERROR,
        })?
        .ok_or(AppError {
            message: "User not found".to_string(),
            status_code: StatusCode::NOT_FOUND,
        })?;

        // Check if user has a role
        if let Some(role_id) = user.role_id {
            let role = roles::Entity::find_by_id(role_id)
                .one(db)
                .await
                .map_err(|e| AppError {
                    message: format!("Database error: {}", e),
                    status_code: StatusCode::INTERNAL_SERVER_ERROR,
                })?;

            if let Some(role_model) = role {
                let permissions: Vec<String> =
                    serde_json::from_str(&role_model.permissions).unwrap_or_else(|_| vec![]);

                let permission_set = PermissionSet::from_strings(permissions);
                Ok(permission_set.contains(permission))
            } else {
                Ok(false)
            }
        } else {
            Ok(false)
        }
    }

    /// Check if a user has any of the given permissions
    #[allow(dead_code)]
    pub async fn has_any_permission(
        db: &DatabaseConnection,
        user_id: Uuid,
        _permissions: &[Permission],
    ) -> Result<bool, AppError> {
        let user = DatabaseService::find_one_with_tracking(
            db,
            "users",
            users::Entity::find_by_id(user_id),
        )
        .await
        .map_err(|e| AppError {
            message: format!("Database error: {}", e),
            status_code: StatusCode::INTERNAL_SERVER_ERROR,
        })?
        .ok_or(AppError {
            message: "User not found".to_string(),
            status_code: StatusCode::NOT_FOUND,
        })?;

        if let Some(role_id) = user.role_id {
            let role = roles::Entity::find_by_id(role_id)
                .one(db)
                .await
                .map_err(|e| AppError {
                    message: format!("Database error: {}", e),
                    status_code: StatusCode::INTERNAL_SERVER_ERROR,
                })?;

            if let Some(role_model) = role {
                let permissions: Vec<String> =
                    serde_json::from_str(&role_model.permissions).unwrap_or_else(|_| vec![]);

                let permission_set = PermissionSet::from_strings(permissions.clone());
                let permission_vec: Vec<Permission> = permissions
                    .iter()
                    .map(|p| Permission::from_string(p))
                    .collect();
                Ok(permission_set.contains_any(&permission_vec))
            } else {
                Ok(false)
            }
        } else {
            Ok(false)
        }
    }

    /// Check if a user has all of the given permissions
    #[allow(dead_code)]
    pub async fn has_all_permissions(
        db: &DatabaseConnection,
        user_id: Uuid,
        _permissions: &[Permission],
    ) -> Result<bool, AppError> {
        let user = DatabaseService::find_one_with_tracking(
            db,
            "users",
            users::Entity::find_by_id(user_id),
        )
        .await
        .map_err(|e| AppError {
            message: format!("Database error: {}", e),
            status_code: StatusCode::INTERNAL_SERVER_ERROR,
        })?
        .ok_or(AppError {
            message: "User not found".to_string(),
            status_code: StatusCode::NOT_FOUND,
        })?;

        if let Some(role_id) = user.role_id {
            let role = roles::Entity::find_by_id(role_id)
                .one(db)
                .await
                .map_err(|e| AppError {
                    message: format!("Database error: {}", e),
                    status_code: StatusCode::INTERNAL_SERVER_ERROR,
                })?;

            if let Some(role_model) = role {
                let permissions: Vec<String> =
                    serde_json::from_str(&role_model.permissions).unwrap_or_else(|_| vec![]);

                let permission_set = PermissionSet::from_strings(permissions.clone());
                let permission_vec: Vec<Permission> = permissions
                    .iter()
                    .map(|p| Permission::from_string(p))
                    .collect();
                Ok(permission_set.contains_all(&permission_vec))
            } else {
                Ok(false)
            }
        } else {
            Ok(false)
        }
    }

    /// Get all permissions for a user
    #[allow(dead_code)]
    pub async fn get_user_permissions(
        db: &DatabaseConnection,
        user_id: Uuid,
    ) -> Result<PermissionSet, AppError> {
        let user = DatabaseService::find_one_with_tracking(
            db,
            "users",
            users::Entity::find_by_id(user_id),
        )
        .await
        .map_err(|e| AppError {
            message: format!("Database error: {}", e),
            status_code: StatusCode::INTERNAL_SERVER_ERROR,
        })?
        .ok_or(AppError {
            message: "User not found".to_string(),
            status_code: StatusCode::NOT_FOUND,
        })?;

        if let Some(role_id) = user.role_id {
            let role = roles::Entity::find_by_id(role_id)
                .one(db)
                .await
                .map_err(|e| AppError {
                    message: format!("Database error: {}", e),
                    status_code: StatusCode::INTERNAL_SERVER_ERROR,
                })?;

            if let Some(role_model) = role {
                let permissions: Vec<String> =
                    serde_json::from_str(&role_model.permissions).unwrap_or_else(|_| vec![]);

                Ok(PermissionSet::from_strings(permissions))
            } else {
                Ok(PermissionSet::new())
            }
        } else {
            Ok(PermissionSet::new())
        }
    }

    /// Get all available permissions in the system
    #[allow(dead_code)]
    pub fn get_all_permissions() -> Vec<Permission> {
        vec![
            Permission::All,
            Permission::AdminRead,
            Permission::AdminWrite,
            Permission::AdminDelete,
            Permission::AdminUsers,
            Permission::AdminRoles,
            Permission::AdminLogs,
            Permission::AdminDatabase,
            Permission::AdminHealth,
            Permission::AdminMetrics,
            Permission::UserRead,
            Permission::UserWrite,
            Permission::UserDelete,
            Permission::UserProfile,
            Permission::UserCreate,
            Permission::SystemHealth,
            Permission::SystemMetrics,
            Permission::SystemLogs,
            Permission::SystemDatabase,
        ]
    }

    /// Get permissions by category
    #[allow(dead_code)]
    pub fn get_permissions_by_category() -> std::collections::HashMap<String, Vec<Permission>> {
        let mut categories = std::collections::HashMap::new();

        for permission in Self::get_all_permissions() {
            let category = permission.category().to_string();
            categories
                .entry(category)
                .or_insert_with(Vec::new)
                .push(permission);
        }

        categories
    }

    /// Validate permission strings
    #[allow(dead_code)]
    pub fn validate_permission_strings(permissions: &[String]) -> Result<Vec<String>, AppError> {
        let mut valid_permissions = Vec::new();

        for permission_str in permissions {
            // Convert to Permission enum to validate
            let permission = Permission::from_string(permission_str);
            valid_permissions.push(permission.to_string());
        }

        Ok(valid_permissions)
    }

    /// Check if a permission string is valid
    #[allow(dead_code)]
    pub fn is_valid_permission(permission_str: &str) -> bool {
        match permission_str {
            "*" | "admin:read" | "admin:write" | "admin:delete" | "admin:users" | "admin:roles"
            | "admin:logs" | "admin:database" | "admin:health" | "admin:metrics" | "user:read"
            | "user:write" | "user:delete" | "user:profile" | "user:create" | "system:health"
            | "system:metrics" | "system:logs" | "system:database" => true,
            _ => permission_str.contains(':'), // Custom permissions must contain ':'
        }
    }
}
