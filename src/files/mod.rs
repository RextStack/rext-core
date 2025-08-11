use crate::error::RextCoreError;
use std::path::{Path, PathBuf};

/// Represents all the files that can be created for a Rext application
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum RextFileType {
    /// Root files
    /// Configuration file for the Rext application
    RextConfig,

    /// example.env file
    ExampleEnv,

    /// Docker files
    DockerComposeYml,
    DockerIgnore,
    Dockerfile,

    /// Git files
    GitIgnore,

    /// README files
    ReadmeMd,

    /// Custom build file
    BuildRs,

    /// Cargo.toml for the Rust project
    CargoToml,

    /// Backend Files
    /// Main Rust source file
    MainRs,
    /// bridge layer source file
    BridgeModRs,
    /// bridge/handlers source file
    HandlersModRs,
    /// Handlers
    HanlderWebsocketRs,
    HandlersAdminRs,
    HandlersRolesRs,
    HandlersAuthRs,

    /// bridge/middleware source file
    MiddlewareModRs,
    /// Middleware
    MiddlewareAuthRs,
    MiddlewareAdminRs,
    MiddlewareLoggingRs,

    /// bridge/routes source file
    RoutesModRs,
    /// Routes
    RoutesAuthRs,
    RoutesAdminRs,

    /// bridge/types source file
    BridgeTypesModRs,
    /// bridge/types/admin
    BridgeTypesAdminRs,
    /// bridge/types/auth
    BridgeTypesAuthRs,
    /// bridge/types/logging
    BridgeTypesLoggingRs,

    /// control layer source file
    ControlModRs,

    /// control/services source file
    ServicesModRs,
    /// Services
    ServerConfigRs,
    StartupRs,
    UserServiceRs,
    DatabaseServiceRs,
    AdminServiceRs,
    TokenServiceRs,
    SessionServiceRs,
    AuthServiceRs,
    PermissionServiceRs,
    SystemMonitorServiceRs,

    /// domain layer source file
    DomainModRs,
    DomainPermissionsRs,
    DomainUserRs,
    DomainValidationRs,
    DomainAuthRs,

    /// entity layer source file
    EntityModRs,

    /// infrastructure layer source file
    InfrastructureModRs,
    /// infrastructure modules
    InfrastructureJobQueueRs,
    InfrastructureLoggingRs,
    InfrastructureSchedulerRs,
    InfrastructureWebsocketRs,
    InfrastructureAppErrorRs,
    InfrastructureDatabaseRs,
    InfrastructureEmailRs,
    InfrastructureQueryPerformanceRs,
    InfrastructureServerRs,
    InfrastructureCorsRs,
    InfrastructureOpenApiRs,
    InfrastructureJwtClaimsRs,

    /// infrastructure/macros source file
    MacrosModRs,
    /// Macros
    PermissionMacroRs,

    /// Frontend Files
    /// front end dependencies file
    PackageJson,
    /// Custom vite config
    ViteConfigTs,
    /// Custom unified config
    UnifiedConfigTs,
    /// Custom OpenAPI Config
    OpenApiConfigTs,
    /// Custom Typescript Config
    TsConfigTs,

    /// Migration Files
    MigrationLibRs,
    MigrationMainRs,
    InitialMigrationRs,
    MigrationCargoToml,
}

/// Represents the Rext module that a file belongs to
#[derive(Debug, Clone, PartialEq)]
pub enum RextModule {
    /// Core Rext functionality
    RextCore,
    /// Admin Panel Module
    RextAdmin,
    /// Vue Module
    RextVue,
    /// Task Scheduler/Job Queue Module
    RextQueue,
    /// Email Service Module
    RextEmail,
}

/// Represents a file to be created in a Rext application
#[derive(Debug, Clone)]
pub struct RextFile {
    /// The name of the file (including extension)
    pub name: String,
    /// The content of the file
    pub content: String,
    /// The relative path from the project root where the file should be created
    pub path: PathBuf,
    /// The Rext module this file belongs to
    pub module: RextModule,
    /// Whether this file needs directory creation
    pub needs_directory: bool,
}

impl RextFile {
    /// Create a new RextFile
    pub fn new(
        name: String,
        content: String,
        path: PathBuf,
        module: RextModule,
        needs_directory: bool,
    ) -> Self {
        Self {
            name,
            content,
            path,
            module,
            needs_directory,
        }
    }

    /// Get the full path where this file should be created
    pub fn full_path(&self, base_dir: &Path) -> PathBuf {
        base_dir.join(&self.path).join(&self.name)
    }

    /// Get the directory path where this file should be created
    pub fn directory_path(&self, base_dir: &Path) -> PathBuf {
        base_dir.join(&self.path)
    }
}

/// Configuration for file creation
pub struct FileCreationConfig {
    /// Application name to substitute in templates
    pub app_name: String,
    /// Modules to include (only files from these modules will be created)
    pub modules: Vec<RextModule>,
}

impl Default for FileCreationConfig {
    fn default() -> Self {
        Self {
            app_name: "my-rext-app".to_string(),
            modules: vec![RextModule::RextCore],
        }
    }
}

/// Load template content from the embedded templates
fn load_template_content(file_type: &RextFileType) -> String {
    match file_type {
        // Root Files
        RextFileType::RextConfig => include_str!("templates/rext.toml").to_string(),
        RextFileType::ExampleEnv => include_str!("templates/example.env").to_string(),
        RextFileType::DockerComposeYml => include_str!("templates/docker-compose.yml").to_string(),
        RextFileType::DockerIgnore => include_str!("templates/dockerignore").to_string(),
        RextFileType::Dockerfile => include_str!("templates/Dockerfile").to_string(),
        RextFileType::GitIgnore => include_str!("templates/gitignore").to_string(),
        RextFileType::ReadmeMd => include_str!("templates/README.md").to_string(),
        RextFileType::BuildRs => include_str!("templates/build.rs").to_string(),
        RextFileType::CargoToml => include_str!("templates/Cargo.toml").to_string(),
        // Backend Files
        RextFileType::MainRs => include_str!("templates/backend/main.rs").to_string(),
        RextFileType::BridgeModRs => include_str!("templates/backend/bridge/mod.rs").to_string(),

        // Handlers
        RextFileType::HandlersModRs => {
            include_str!("templates/backend/bridge/handlers/mod.rs").to_string()
        }
        RextFileType::HanlderWebsocketRs => {
            include_str!("templates/backend/bridge/handlers/websocket.rs").to_string()
        }
        RextFileType::HandlersAdminRs => {
            include_str!("templates/backend/bridge/handlers/admin.rs").to_string()
        }
        RextFileType::HandlersRolesRs => {
            include_str!("templates/backend/bridge/handlers/roles.rs").to_string()
        }
        RextFileType::HandlersAuthRs => {
            include_str!("templates/backend/bridge/handlers/auth.rs").to_string()
        }

        // Middleware
        RextFileType::MiddlewareModRs => {
            include_str!("templates/backend/bridge/middleware/mod.rs").to_string()
        }
        RextFileType::MiddlewareAuthRs => {
            include_str!("templates/backend/bridge/middleware/auth.rs").to_string()
        }
        RextFileType::MiddlewareAdminRs => {
            include_str!("templates/backend/bridge/middleware/admin.rs").to_string()
        }
        RextFileType::MiddlewareLoggingRs => {
            include_str!("templates/backend/bridge/middleware/logging.rs").to_string()
        }

        // Routes
        RextFileType::RoutesModRs => {
            include_str!("templates/backend/bridge/routes/mod.rs").to_string()
        }
        RextFileType::RoutesAuthRs => {
            include_str!("templates/backend/bridge/routes/auth.rs").to_string()
        }
        RextFileType::RoutesAdminRs => {
            include_str!("templates/backend/bridge/routes/admin.rs").to_string()
        }

        // Types
        RextFileType::BridgeTypesModRs => {
            include_str!("templates/backend/bridge/types/mod.rs").to_string()
        }
        RextFileType::BridgeTypesAdminRs => {
            include_str!("templates/backend/bridge/types/admin.rs").to_string()
        }
        RextFileType::BridgeTypesAuthRs => {
            include_str!("templates/backend/bridge/types/auth.rs").to_string()
        }
        RextFileType::BridgeTypesLoggingRs => {
            include_str!("templates/backend/bridge/types/logging.rs").to_string()
        }

        RextFileType::ControlModRs => include_str!("templates/backend/control/mod.rs").to_string(),

        // Services
        RextFileType::ServicesModRs => {
            include_str!("templates/backend/control/services/mod.rs").to_string()
        }
        RextFileType::ServerConfigRs => {
            include_str!("templates/backend/control/services/server_config.rs").to_string()
        }
        RextFileType::StartupRs => {
            include_str!("templates/backend/control/services/startup.rs").to_string()
        }
        RextFileType::UserServiceRs => {
            include_str!("templates/backend/control/services/user_service.rs").to_string()
        }
        RextFileType::DatabaseServiceRs => {
            include_str!("templates/backend/control/services/database_service.rs").to_string()
        }
        RextFileType::AdminServiceRs => {
            include_str!("templates/backend/control/services/admin_service.rs").to_string()
        }
        RextFileType::TokenServiceRs => {
            include_str!("templates/backend/control/services/token_service.rs").to_string()
        }
        RextFileType::AuthServiceRs => {
            include_str!("templates/backend/control/services/auth_service.rs").to_string()
        }
        RextFileType::SessionServiceRs => {
            include_str!("templates/backend/control/services/session_service.rs").to_string()
        }
        RextFileType::PermissionServiceRs => {
            include_str!("templates/backend/control/services/permission_service.rs").to_string()
        }
        RextFileType::SystemMonitorServiceRs => {
            include_str!("templates/backend/control/services/system_monitor.rs").to_string()
        }
        // Domain
        RextFileType::DomainModRs => include_str!("templates/backend/domain/mod.rs").to_string(),
        RextFileType::DomainPermissionsRs => {
            include_str!("templates/backend/domain/permissions.rs").to_string()
        }
        RextFileType::DomainUserRs => include_str!("templates/backend/domain/user.rs").to_string(),
        RextFileType::DomainValidationRs => {
            include_str!("templates/backend/domain/validation.rs").to_string()
        }
        RextFileType::DomainAuthRs => include_str!("templates/backend/domain/auth.rs").to_string(),

        RextFileType::EntityModRs => include_str!("templates/backend/entity/mod.rs").to_string(),

        // Infrastructure
        RextFileType::InfrastructureModRs => {
            include_str!("templates/backend/infrastructure/mod.rs").to_string()
        }
        RextFileType::InfrastructureJobQueueRs => {
            include_str!("templates/backend/infrastructure/job_queue.rs").to_string()
        }
        RextFileType::InfrastructureLoggingRs => {
            include_str!("templates/backend/infrastructure/logging.rs").to_string()
        }
        RextFileType::InfrastructureSchedulerRs => {
            include_str!("templates/backend/infrastructure/scheduler.rs").to_string()
        }
        RextFileType::InfrastructureWebsocketRs => {
            include_str!("templates/backend/infrastructure/websocket.rs").to_string()
        }
        RextFileType::InfrastructureAppErrorRs => {
            include_str!("templates/backend/infrastructure/app_error.rs").to_string()
        }
        RextFileType::InfrastructureDatabaseRs => {
            include_str!("templates/backend/infrastructure/database.rs").to_string()
        }
        RextFileType::InfrastructureEmailRs => {
            include_str!("templates/backend/infrastructure/email.rs").to_string()
        }
        RextFileType::InfrastructureQueryPerformanceRs => {
            include_str!("templates/backend/infrastructure/query_performance.rs").to_string()
        }
        RextFileType::InfrastructureServerRs => {
            include_str!("templates/backend/infrastructure/server.rs").to_string()
        }
        RextFileType::InfrastructureCorsRs => {
            include_str!("templates/backend/infrastructure/cors.rs").to_string()
        }
        RextFileType::InfrastructureOpenApiRs => {
            include_str!("templates/backend/infrastructure/openapi.rs").to_string()
        }
        RextFileType::InfrastructureJwtClaimsRs => {
            include_str!("templates/backend/infrastructure/jwt_claims.rs").to_string()
        }
        // Macors
        RextFileType::MacrosModRs => {
            include_str!("templates/backend/infrastructure/macros/mod.rs").to_string()
        }
        RextFileType::PermissionMacroRs => {
            include_str!("templates/backend/infrastructure/macros/permission_macro.rs").to_string()
        }
        // Frontend Files
        RextFileType::PackageJson => include_str!("templates/frontend/package.json").to_string(),
        RextFileType::ViteConfigTs => include_str!("templates/frontend/vite.config.ts").to_string(),
        RextFileType::UnifiedConfigTs => {
            include_str!("templates/frontend/config/unified.config.ts").to_string()
        }
        RextFileType::OpenApiConfigTs => {
            include_str!("templates/frontend/openapi-ts.config.ts").to_string()
        }
        RextFileType::TsConfigTs => include_str!("templates/frontend/tsconfig.json").to_string(),
        // Migration Files
        RextFileType::MigrationLibRs => include_str!("templates/migration/src/lib.rs").to_string(),
        RextFileType::MigrationMainRs => {
            include_str!("templates/migration/src/main.rs").to_string()
        }
        RextFileType::InitialMigrationRs => {
            include_str!("templates/migration/src/initial_migration.rs").to_string()
        }
        RextFileType::MigrationCargoToml => {
            include_str!("templates/migration/Cargo.toml").to_string()
        }
    }
}

/// Process template content by replacing placeholders
fn process_template(content: &str, config: &FileCreationConfig) -> String {
    content.replace("{app_name}", &config.app_name)
}

/// Get all files that should be created for the given configuration
pub fn get_rext_files(config: &FileCreationConfig) -> Vec<RextFile> {
    let mut files = Vec::new();

    // Define all files with their metadata
    let file_definitions = [
        // Root Files
        (
            RextFileType::RextConfig,
            "rext.toml",
            PathBuf::from("."),
            RextModule::RextCore,
            false,
        ),
        (
            RextFileType::ExampleEnv,
            "example.env",
            PathBuf::from("."),
            RextModule::RextCore,
            false,
        ),
        (
            RextFileType::DockerComposeYml,
            "docker-compose.yml",
            PathBuf::from("."),
            RextModule::RextCore,
            false,
        ),
        (
            RextFileType::DockerIgnore,
            "dockerignore",
            PathBuf::from("."),
            RextModule::RextCore,
            false,
        ),
        (
            RextFileType::Dockerfile,
            "Dockerfile",
            PathBuf::from("."),
            RextModule::RextCore,
            false,
        ),
        (
            RextFileType::GitIgnore,
            ".gitignore",
            PathBuf::from("."),
            RextModule::RextCore,
            false,
        ),
        (
            RextFileType::ReadmeMd,
            "README.md",
            PathBuf::from("."),
            RextModule::RextCore,
            false,
        ),
        (
            RextFileType::BuildRs,
            "build.rs",
            PathBuf::from("."),
            RextModule::RextCore,
            false,
        ),
        (
            RextFileType::CargoToml,
            "Cargo.toml",
            PathBuf::from("."),
            RextModule::RextCore,
            false,
        ),
        // Backend Files
        (
            RextFileType::MainRs,
            "main.rs",
            PathBuf::from("backend"),
            RextModule::RextCore,
            true,
        ),
        (
            RextFileType::BridgeModRs,
            "mod.rs",
            PathBuf::from("backend/bridge"),
            RextModule::RextCore,
            true,
        ),
        // Handlers
        (
            RextFileType::HandlersModRs,
            "mod.rs",
            PathBuf::from("backend/bridge/handlers"),
            RextModule::RextCore,
            true,
        ),
        (
            RextFileType::HanlderWebsocketRs,
            "websocket.rs",
            PathBuf::from("backend/bridge/handlers"),
            RextModule::RextCore,
            true,
        ),
        (
            RextFileType::HandlersAdminRs,
            "admin.rs",
            PathBuf::from("backend/bridge/handlers"),
            RextModule::RextCore,
            true,
        ),
        (
            RextFileType::HandlersRolesRs,
            "roles.rs",
            PathBuf::from("backend/bridge/handlers"),
            RextModule::RextCore,
            true,
        ),
        (
            RextFileType::HandlersAuthRs,
            "auth.rs",
            PathBuf::from("backend/bridge/handlers"),
            RextModule::RextCore,
            true,
        ),
        // Middleware
        (
            RextFileType::MiddlewareModRs,
            "mod.rs",
            PathBuf::from("backend/bridge/middleware"),
            RextModule::RextCore,
            true,
        ),
        (
            RextFileType::MiddlewareAuthRs,
            "auth.rs",
            PathBuf::from("backend/bridge/middleware"),
            RextModule::RextCore,
            true,
        ),
        (
            RextFileType::MiddlewareAdminRs,
            "admin.rs",
            PathBuf::from("backend/bridge/middleware"),
            RextModule::RextCore,
            true,
        ),
        (
            RextFileType::MiddlewareLoggingRs,
            "logging.rs",
            PathBuf::from("backend/bridge/middleware"),
            RextModule::RextCore,
            true,
        ),
        // Routes
        (
            RextFileType::RoutesModRs,
            "mod.rs",
            PathBuf::from("backend/bridge/routes"),
            RextModule::RextCore,
            true,
        ),
        (
            RextFileType::RoutesAdminRs,
            "admin.rs",
            PathBuf::from("backend/bridge/routes"),
            RextModule::RextCore,
            true,
        ),
        (
            RextFileType::RoutesAuthRs,
            "auth.rs",
            PathBuf::from("backend/bridge/routes"),
            RextModule::RextCore,
            true,
        ),
        (
            RextFileType::BridgeTypesModRs,
            "mod.rs",
            PathBuf::from("backend/bridge/types"),
            RextModule::RextCore,
            true,
        ),
        (
            RextFileType::BridgeTypesAdminRs,
            "admin.rs",
            PathBuf::from("backend/bridge/types"),
            RextModule::RextCore,
            true,
        ),
        (
            RextFileType::BridgeTypesAuthRs,
            "auth.rs",
            PathBuf::from("backend/bridge/types"),
            RextModule::RextCore,
            true,
        ),
        (
            RextFileType::BridgeTypesLoggingRs,
            "logging.rs",
            PathBuf::from("backend/bridge/types"),
            RextModule::RextCore,
            true,
        ),
        (
            RextFileType::ControlModRs,
            "mod.rs",
            PathBuf::from("backend/control"),
            RextModule::RextCore,
            true,
        ),
        // Services
        (
            RextFileType::ServicesModRs,
            "mod.rs",
            PathBuf::from("backend/control/services"),
            RextModule::RextCore,
            true,
        ),
        (
            RextFileType::ServerConfigRs,
            "server_config.rs",
            PathBuf::from("backend/control/services"),
            RextModule::RextCore,
            true,
        ),
        (
            RextFileType::StartupRs,
            "startup.rs",
            PathBuf::from("backend/control/services"),
            RextModule::RextCore,
            true,
        ),
        (
            RextFileType::UserServiceRs,
            "user_service.rs",
            PathBuf::from("backend/control/services"),
            RextModule::RextCore,
            true,
        ),
        (
            RextFileType::DatabaseServiceRs,
            "database_service.rs",
            PathBuf::from("backend/control/services"),
            RextModule::RextCore,
            true,
        ),
        (
            RextFileType::AdminServiceRs,
            "admin_service.rs",
            PathBuf::from("backend/control/services"),
            RextModule::RextCore,
            true,
        ),
        (
            RextFileType::TokenServiceRs,
            "token_service.rs",
            PathBuf::from("backend/control/services"),
            RextModule::RextCore,
            true,
        ),
        (
            RextFileType::SessionServiceRs,
            "session_service.rs",
            PathBuf::from("backend/control/services"),
            RextModule::RextCore,
            true,
        ),
        (
            RextFileType::AuthServiceRs,
            "auth_service.rs",
            PathBuf::from("backend/control/services"),
            RextModule::RextCore,
            true,
        ),
        (
            RextFileType::PermissionServiceRs,
            "permission_service.rs",
            PathBuf::from("backend/control/services"),
            RextModule::RextCore,
            true,
        ),
        (
            RextFileType::SystemMonitorServiceRs,
            "system_monitor.rs",
            PathBuf::from("backend/control/services"),
            RextModule::RextCore,
            true,
        ),
        (
            RextFileType::DomainModRs,
            "mod.rs",
            PathBuf::from("backend/domain"),
            RextModule::RextCore,
            true,
        ),
        (
            RextFileType::DomainPermissionsRs,
            "permissions.rs",
            PathBuf::from("backend/domain"),
            RextModule::RextCore,
            true,
        ),
        (
            RextFileType::DomainUserRs,
            "user.rs",
            PathBuf::from("backend/domain"),
            RextModule::RextCore,
            true,
        ),
        (
            RextFileType::DomainValidationRs,
            "validation.rs",
            PathBuf::from("backend/domain"),
            RextModule::RextCore,
            true,
        ),
        (
            RextFileType::DomainAuthRs,
            "auth.rs",
            PathBuf::from("backend/domain"),
            RextModule::RextCore,
            true,
        ),
        (
            RextFileType::EntityModRs,
            "mod.rs",
            PathBuf::from("backend/entity"),
            RextModule::RextCore,
            true,
        ),
        (
            RextFileType::InfrastructureModRs,
            "mod.rs",
            PathBuf::from("backend/infrastructure"),
            RextModule::RextCore,
            true,
        ),
        (
            RextFileType::InfrastructureJobQueueRs,
            "job_queue.rs",
            PathBuf::from("backend/infrastructure"),
            RextModule::RextCore,
            true,
        ),
        (
            RextFileType::InfrastructureLoggingRs,
            "logging.rs",
            PathBuf::from("backend/infrastructure"),
            RextModule::RextCore,
            true,
        ),
        (
            RextFileType::InfrastructureSchedulerRs,
            "scheduler.rs",
            PathBuf::from("backend/infrastructure"),
            RextModule::RextCore,
            true,
        ),
        (
            RextFileType::InfrastructureWebsocketRs,
            "websocket.rs",
            PathBuf::from("backend/infrastructure"),
            RextModule::RextCore,
            true,
        ),
        (
            RextFileType::InfrastructureAppErrorRs,
            "app_error.rs",
            PathBuf::from("backend/infrastructure"),
            RextModule::RextCore,
            true,
        ),
        (
            RextFileType::InfrastructureEmailRs,
            "email.rs",
            PathBuf::from("backend/infrastructure"),
            RextModule::RextCore,
            true,
        ),
        (
            RextFileType::InfrastructureDatabaseRs,
            "database.rs",
            PathBuf::from("backend/infrastructure"),
            RextModule::RextCore,
            true,
        ),
        (
            RextFileType::InfrastructureQueryPerformanceRs,
            "query_performance.rs",
            PathBuf::from("backend/infrastructure"),
            RextModule::RextCore,
            true,
        ),
        (
            RextFileType::InfrastructureServerRs,
            "server.rs",
            PathBuf::from("backend/infrastructure"),
            RextModule::RextCore,
            true,
        ),
        (
            RextFileType::InfrastructureCorsRs,
            "cors.rs",
            PathBuf::from("backend/infrastructure"),
            RextModule::RextCore,
            true,
        ),
        (
            RextFileType::InfrastructureOpenApiRs,
            "openapi.rs",
            PathBuf::from("backend/infrastructure"),
            RextModule::RextCore,
            true,
        ),
        (
            RextFileType::InfrastructureJwtClaimsRs,
            "jwt_claims.rs",
            PathBuf::from("backend/infrastructure"),
            RextModule::RextCore,
            true,
        ),
        (
            RextFileType::MacrosModRs,
            "mod.rs",
            PathBuf::from("backend/infrastructure/macros"),
            RextModule::RextCore,
            true,
        ),
        (
            RextFileType::PermissionMacroRs,
            "permission_macro.rs",
            PathBuf::from("backend/infrastructure/macros"),
            RextModule::RextCore,
            true,
        ),
        // Frontend Files
        (
            RextFileType::PackageJson,
            "package.json",
            PathBuf::from("frontend"),
            RextModule::RextCore,
            true,
        ),
        (
            RextFileType::ViteConfigTs,
            "vite.config.ts",
            PathBuf::from("frontend"),
            RextModule::RextCore,
            true,
        ),
        (
            RextFileType::UnifiedConfigTs,
            "unified.config.ts",
            PathBuf::from("frontend/config"),
            RextModule::RextCore,
            true,
        ),
        (
            RextFileType::OpenApiConfigTs,
            "openapi-ts.config.ts",
            PathBuf::from("frontend"),
            RextModule::RextCore,
            true,
        ),
        (
            RextFileType::TsConfigTs,
            "tsconfig.json",
            PathBuf::from("frontend"),
            RextModule::RextCore,
            true,
        ),
        // Migration Files
        (
            RextFileType::MigrationLibRs,
            "lib.rs",
            PathBuf::from("migration/src"),
            RextModule::RextCore,
            true,
        ),
        (
            RextFileType::MigrationMainRs,
            "main.rs",
            PathBuf::from("migration/src"),
            RextModule::RextCore,
            true,
        ),
        (
            RextFileType::InitialMigrationRs,
            "initial_migration.rs",
            PathBuf::from("migration/src"),
            RextModule::RextCore,
            true,
        ),
        (
            RextFileType::MigrationCargoToml,
            "Cargo.toml",
            PathBuf::from("migration"),
            RextModule::RextCore,
            true,
        ),
    ];

    // Create files for enabled modules
    for (file_type, name, path, module, needs_directory) in file_definitions {
        if config.modules.contains(&module) {
            let template_content = load_template_content(&file_type);
            let processed_content = process_template(&template_content, config);

            files.push(RextFile::new(
                name.to_string(),
                processed_content,
                path,
                module,
                needs_directory,
            ));
        }
    }

    files
}

/// Create all necessary directories for the files
pub fn create_directories(files: &[RextFile], base_dir: &Path) -> Result<(), RextCoreError> {
    let mut directories_to_create = std::collections::HashSet::new();

    // Collect all directories that need to be created
    for file in files {
        if file.needs_directory {
            directories_to_create.insert(file.directory_path(base_dir));
        }
    }

    // Create directories
    for dir in directories_to_create {
        std::fs::create_dir_all(&dir).map_err(RextCoreError::DirectoryCreation)?;
    }

    Ok(())
}

/// Create all files in the target directory
pub fn create_files(files: &[RextFile], base_dir: &Path) -> Result<(), RextCoreError> {
    // First, create all necessary directories
    create_directories(files, base_dir)?;

    // Then create all files
    for file in files {
        let full_path = file.full_path(base_dir);
        std::fs::write(&full_path, &file.content)
            .map_err(|e| RextCoreError::FileWrite(format!("{}: {}", full_path.display(), e)))?;
    }

    Ok(())
}

/// Create a new Rext application with the specified configuration
pub fn create_rext_app(base_dir: &Path, config: FileCreationConfig) -> Result<(), RextCoreError> {
    // Check if rext.toml already exists
    if base_dir.join("rext.toml").exists() {
        return Err(RextCoreError::AppAlreadyExists);
    }

    // Check if Cargo.toml already exists
    if base_dir.join("Cargo.toml").exists() {
        return Err(RextCoreError::AppAlreadyExists);
    }

    // Get all files to create
    let files = get_rext_files(&config);

    // Create the files
    create_files(&files, base_dir)?;

    Ok(())
}
