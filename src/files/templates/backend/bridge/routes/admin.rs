use axum::middleware;
use sea_orm::DatabaseConnection;
use utoipa_axum::{router::OpenApiRouter, routes};

use crate::bridge::middleware::admin::admin_middleware;

pub fn admin_router(db: DatabaseConnection) -> OpenApiRouter {
    // Admin authentication routes (no middleware needed)
    let auth_routes = OpenApiRouter::new()
        .routes(routes!(crate::bridge::handlers::admin::admin_login_handler))
        .routes(routes!(
            crate::bridge::handlers::admin::admin_logout_handler
        ));

    // Protected admin routes (require admin middleware)
    let protected_routes = OpenApiRouter::new()
        // Audit logs
        .routes(routes!(
            crate::bridge::handlers::admin::get_audit_logs_handler
        ))
        // User management
        .routes(routes!(crate::bridge::handlers::admin::get_users_handler))
        .routes(routes!(crate::bridge::handlers::admin::create_user_handler))
        .routes(routes!(crate::bridge::handlers::admin::get_user_handler))
        .routes(routes!(crate::bridge::handlers::admin::update_user_handler))
        .routes(routes!(crate::bridge::handlers::admin::delete_user_handler))
        // Session management
        .routes(routes!(
            crate::bridge::handlers::admin::get_user_sessions_handler
        ))
        .routes(routes!(
            crate::bridge::handlers::admin::invalidate_session_handler
        ))
        .routes(routes!(
            crate::bridge::handlers::admin::invalidate_all_user_sessions_handler
        ))
        // Role management
        .routes(routes!(crate::bridge::handlers::roles::get_roles_handler))
        .routes(routes!(crate::bridge::handlers::roles::create_role_handler))
        .routes(routes!(crate::bridge::handlers::roles::get_role_handler))
        .routes(routes!(crate::bridge::handlers::roles::update_role_handler))
        .routes(routes!(crate::bridge::handlers::roles::delete_role_handler))
        .routes(routes!(
            crate::bridge::handlers::roles::check_permission_handler
        ))
        // Database inspection
        .routes(routes!(
            crate::bridge::handlers::admin::get_database_tables_handler
        ))
        .routes(routes!(
            crate::bridge::handlers::admin::get_table_records_handler
        ))
        // System health
        .routes(routes!(crate::bridge::handlers::admin::health_handler))
        // Combined auth and admin middleware
        .route_layer(middleware::from_fn_with_state(db.clone(), admin_middleware));

    // Combine auth and protected routes
    auth_routes.merge(protected_routes).with_state(db)
}
