use axum::{Router, middleware, routing::get};
use sea_orm::DatabaseConnection;
use std::{
    env,
    io::Error,
    net::{Ipv4Addr, SocketAddr},
};
use tokio::net::TcpListener;
use tower_http::services::{ServeDir, ServeFile};
use utoipa::OpenApi;
use utoipa_axum::router::OpenApiRouter;
use utoipa_rapidoc::RapiDoc;
use utoipa_redoc::{Redoc, Servable};
use utoipa_scalar::{Scalar, Servable as ScalarServable};
use utoipa_swagger_ui::SwaggerUi;

use crate::bridge::middleware::logging::request_logging_middleware;
use crate::bridge::routes::admin::admin_router;
use crate::bridge::routes::auth::auth_router;
use crate::infrastructure::cors::CorsManager;
use crate::infrastructure::openapi::ApiDoc;

/// Server manager
pub struct ServerManager;

impl ServerManager {
    /// Creates the main router with all endpoints
    pub fn create_router(db: DatabaseConnection) -> Router {
        let environment = env::var("ENVIRONMENT").unwrap_or_else(|_| "development".to_string());

        // Create the OpenAPI Router and nested routes
        let (router, api) = OpenApiRouter::with_openapi(ApiDoc::openapi())
            .nest("/api/v1/auth", auth_router(db.clone()))
            .nest("/api/v1/admin", admin_router(db.clone()))
            .split_for_parts();

        // Create WebSocket router with database state
        let websocket_router = Router::new()
            .route(
                "/api/v1/admin/ws",
                get(crate::bridge::handlers::websocket::websocket_handler),
            )
            .with_state(db.clone());

        // Merge routes with OpenAPI documentation and websocket and middleware
        let mut router = router
            .merge(SwaggerUi::new("/swagger-ui").url("/api-docs/openapi.json", api.clone()))
            .merge(Redoc::with_url("/redoc", api.clone()))
            .merge(RapiDoc::new("/api-docs/openapi.json").path("/rapidoc"))
            .merge(Scalar::with_url("/scalar", api))
            .merge(websocket_router)
            .route_layer(middleware::from_fn_with_state(
                db.clone(),
                request_logging_middleware,
            ));

        // Add CORS layer for development
        if environment == "development" {
            router = router.layer(CorsManager::create_cors_layer());
        }

        // Check if we're in production mode and serve static files
        if environment == "production" {
            println!("Production mode detected - serving static files from /dist directory");
            router = router.fallback_service(
                ServeDir::new("dist").fallback(ServeFile::new("dist/index.html")),
            );
        } else {
            println!("Development mode - static files not served by backend");
            println!("Frontend running on http://localhost:5173");
        }

        router
    }

    /// Starts the server
    pub async fn start_server(router: Router) -> Result<(), Error> {
        let address = SocketAddr::from((Ipv4Addr::UNSPECIFIED, 3000));
        let listener = TcpListener::bind(&address).await?;

        println!("Server running on http://localhost:{}", address.port());
        println!("View API docs at:");
        println!(
            "  http://localhost:{}/swagger-ui 📱 Swagger UI",
            address.port()
        );
        println!("  http://localhost:{}/redoc 📖 Redoc", address.port());
        println!(
            "  http://localhost:{}/api-docs/openapi.json ✏️ The OpenAPI JSON file",
            address.port()
        );
        println!(
            "  http://localhost:{}/scalar ⭐ Recommended for API testing",
            address.port()
        );

        axum::serve(listener, router.into_make_service())
            .await
            .map_err(|e| Error::new(std::io::ErrorKind::Interrupted, e))
    }
}
