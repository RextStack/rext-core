use axum::http::StatusCode;
use sea_orm::ActiveValue::Set;
use sea_orm::{
    ActiveModelTrait, ColumnTrait, Database, DatabaseConnection, EntityTrait, QueryFilter,
};
use sea_orm_migration::prelude::*;
use std::env;

use crate::control::services::{server_config::ServerConfigService, user_service::UserService};
use crate::domain::permissions::DefaultPermissions;
use crate::entity::models::roles;
use crate::infrastructure::app_error::AppError;
use crate::infrastructure::{
    database::DatabaseManager, job_queue::JobQueueManager, scheduler::SchedulerManager,
    server::ServerManager,
};
use migration;

/// Application startup orchestrator
pub struct StartupService;

impl StartupService {
    /// Initializes the application and returns the database connection
    pub async fn initialize() -> Result<DatabaseConnection, Box<dyn std::error::Error>> {
        // Load environment variables from .env file
        dotenvy::dotenv().ok();

        // Initialize server configuration
        ServerConfigService::initialize();

        // Get environment configuration
        let environment = env::var("ENVIRONMENT").unwrap_or_else(|_| "development".to_string());
        println!("Environment: {}", environment);

        // Create database connection
        let db = DatabaseManager::create_connection().await?;

        // Run migrations
        println!("Running database migrations...");
        Self::run_migrations().await?;
        println!("Migrations completed successfully");

        // Create pool for job queue
        let pool = DatabaseManager::create_pool().await?;

        // Setup job queue storage
        DatabaseManager::setup_job_queue_storage(&pool).await?;

        // Create job storage
        let job_storage = JobQueueManager::create_storage(pool);

        // Queue test job
        println!("Queuing test job!");
        JobQueueManager::produce_messages(&job_storage).await?;

        // Seed default roles if enabled
        Self::seed_default_roles(&db).await?;

        // Seed admin user if enabled
        Self::seed_admin_user(&db).await?;

        Ok(db)
    }

    /// Runs database migrations using SeaORM Migration API
    async fn run_migrations() -> Result<(), Box<dyn std::error::Error>> {
        let database_url = env::var("DATABASE_URL")
            .map_err(|_| "DATABASE_URL environment variable is required")?;

        println!("Executing migrations with SeaORM Migration API...");

        // Create database connection for migrations
        let db = Database::connect(&database_url)
            .await
            .map_err(|e| format!("Failed to connect to database: {}", e))?;

        // Create schema manager to investigate the schema
        let schema_manager = SchemaManager::new(&db);

        // Run migrations using the Migrator
        migration::Migrator::up(&db, None)
            .await
            .map_err(|e| format!("Migration failed: {}", e))?;

        // Verify that migrations were applied successfully
        assert!(
            schema_manager
                .has_table("users")
                .await
                .map_err(|e| format!("Failed to verify users table: {}", e))?
        );
        assert!(
            schema_manager
                .has_table("roles")
                .await
                .map_err(|e| format!("Failed to verify roles table: {}", e))?
        );
        assert!(
            schema_manager
                .has_table("audit_logs")
                .await
                .map_err(|e| format!("Failed to verify audit_logs table: {}", e))?
        );
        assert!(
            schema_manager
                .has_table("database_metrics")
                .await
                .map_err(|e| format!("Failed to verify database_metrics table: {}", e))?
        );
        assert!(
            schema_manager
                .has_table("user_sessions")
                .await
                .map_err(|e| format!("Failed to verify user_sessions table: {}", e))?
        );

        println!("✅ Database migrations completed successfully");
        Ok(())
    }

    /// Seeds the admin user if it doesn't exist
    async fn seed_admin_user(db: &DatabaseConnection) -> Result<(), Box<dyn std::error::Error>> {
        // Check if admin user creation is enabled
        let create_admin = env::var("CREATE_ADMIN_USER")
            .unwrap_or_else(|_| "true".to_string())
            .parse::<bool>()
            .unwrap_or(true);

        if !create_admin {
            println!("Admin user creation is disabled");
            return Ok(());
        }

        // Get admin credentials from environment variables
        let admin_email = env::var("ADMIN_EMAIL").unwrap_or_else(|_| "admin@localhost".to_string());
        let admin_password = env::var("ADMIN_PASSWORD").unwrap_or_else(|_| "admin".to_string());

        // Check if admin user already exists
        match UserService::find_user_by_email(db, &admin_email).await {
            Ok(Some(_)) => {
                println!("Admin user already exists: {}", admin_email);
                Ok(())
            }
            Ok(None) => {
                // Find the admin role
                let admin_role = roles::Entity::find()
                    .filter(roles::Column::Name.eq("admin"))
                    .one(db)
                    .await
                    .map_err(|e| AppError {
                        message: format!("Database error: {}", e),
                        status_code: StatusCode::INTERNAL_SERVER_ERROR,
                    })?;

                let admin_role_id = admin_role.map(|role| role.id);

                // Create admin user with admin role
                match UserService::create_user_with_role(
                    db,
                    admin_email.clone(),
                    admin_password,
                    admin_role_id,
                )
                .await
                {
                    Ok(user) => {
                        println!(
                            "✅ Admin user created successfully: {} (ID: {})",
                            admin_email, user.id
                        );
                        println!("⚠️  IMPORTANT: Change the default admin password immediately!");
                        Ok(())
                    }
                    Err(e) => {
                        eprintln!("❌ Failed to create admin user: {}", e.message);
                        Err(Box::new(e))
                    }
                }
            }
            Err(e) => {
                eprintln!("❌ Error checking for existing admin user: {}", e.message);
                Err(Box::new(e))
            }
        }
    }

    async fn seed_default_roles(db: &DatabaseConnection) -> Result<(), Box<dyn std::error::Error>> {
        // Check if default roles creation is enabled
        let create_default_roles = env::var("CREATE_DEFAULT_ROLES")
            .unwrap_or_else(|_| "true".to_string())
            .parse::<bool>()
            .unwrap_or(true);

        if !create_default_roles {
            println!("Default roles creation is disabled");
            return Ok(());
        }

        // Get default roles from environment variables
        let default_roles = env::var("DEFAULT_ROLES").unwrap_or_else(|_| "admin,user".to_string());
        let default_roles = default_roles
            .split(',')
            .map(|r| r.trim().to_string())
            .collect::<Vec<String>>();

        // Define default role configurations using the new permission system
        let role_configs = vec![
            ("admin", "Full system access", DefaultPermissions::admin()),
            ("user", "Basic user access", DefaultPermissions::user()),
        ];

        // Create default roles (admin, user, only if found in .env)
        for role_name in default_roles {
            if let Some((_, description, permission_set)) =
                role_configs.iter().find(|(name, _, _)| name == &role_name)
            {
                // Check if role already exists
                let existing_role = roles::Entity::find()
                    .filter(roles::Column::Name.eq(&role_name))
                    .one(db)
                    .await
                    .map_err(|e| AppError {
                        message: format!("Database error: {}", e),
                        status_code: StatusCode::INTERNAL_SERVER_ERROR,
                    })?;

                if existing_role.is_some() {
                    println!("Role already exists: {}", role_name);
                    continue;
                }

                // Convert permission set to JSON string
                let permissions_json = serde_json::to_string(&permission_set.to_strings())
                    .map_err(|e| AppError {
                        message: format!("Failed to serialize permissions: {}", e),
                        status_code: StatusCode::INTERNAL_SERVER_ERROR,
                    })?;

                let role_model = roles::ActiveModel {
                    name: Set(role_name.clone()),
                    description: Set(Some(ToString::to_string(description))),
                    permissions: Set(permissions_json),
                    ..Default::default()
                };

                let role = role_model.insert(db).await.map_err(|e| AppError {
                    message: format!("Database error: {}", e),
                    status_code: StatusCode::INTERNAL_SERVER_ERROR,
                })?;

                println!("✅ Role created successfully: {}", role.name);
            }
        }

        Ok(())
    }

    /// Runs the server task
    pub async fn run_server(db: DatabaseConnection) -> Result<(), Box<dyn std::error::Error>> {
        let router = ServerManager::create_router(db);
        ServerManager::start_server(router).await?;
        Ok(())
    }

    /// Runs the job queue monitor task
    pub async fn run_job_queue_monitor() -> Result<(), Box<dyn std::error::Error>> {
        let pool = DatabaseManager::create_pool().await?;
        let job_storage = JobQueueManager::create_storage(pool);

        JobQueueManager::run_job_queue_monitor(job_storage).await?;
        Ok(())
    }

    /// Runs the task scheduler
    pub async fn run_scheduler() -> Result<(), Box<dyn std::error::Error>> {
        let database_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set in .env file");
        SchedulerManager::run_scheduler(&database_url).await?;
        Ok(())
    }
}
