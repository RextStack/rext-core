use axum::middleware;
use sea_orm::DatabaseConnection;
use utoipa_axum::{router::OpenApiRouter, routes};

use crate::bridge::middleware::auth::auth_middleware;

pub fn auth_router(db: DatabaseConnection) -> OpenApiRouter {
    // Routes that don't need authentication
    let public_routes = OpenApiRouter::new()
        .routes(routes!(crate::bridge::handlers::auth::register_handler))
        .routes(routes!(crate::bridge::handlers::auth::login_handler))
        .routes(routes!(crate::bridge::handlers::auth::logout_handler))
        .routes(routes!(crate::bridge::handlers::auth::verify_email_handler));

    // Routes that need authentication
    let protected_routes = OpenApiRouter::new()
        .routes(routes!(crate::bridge::handlers::auth::profile_handler))
        .route_layer(middleware::from_fn_with_state(db.clone(), auth_middleware));

    // Combine both route groups - retains the middleware layers
    public_routes.merge(protected_routes).with_state(db)
}
