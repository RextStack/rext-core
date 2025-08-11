use base64::Engine;
use sea_orm::*;
use uuid::Uuid;

use crate::{
    bridge::types::admin::*,
    control::services::{
        database_service::DatabaseMonitorService, session_service::SessionService,
        system_monitor::SystemMonitorService, user_service::UserService,
    },
    domain::validation::*,
    entity::models::{audit_logs, roles, users},
    infrastructure::{app_error::AppError, jwt_claims::Claims},
};
use axum::http::StatusCode;
use jsonwebtoken::{EncodingKey, Header, encode};
use std::env;

/// Service for admin-related business operations
pub struct AdminService;

impl AdminService {
    /// Authenticates an admin user and returns a JWT token
    /// Specifically for "super admin" privileges, defined with the "*" permission
    /// Other "admin" permissions like "admin:read" are handled by the user_can_perform_action function
    ///
    /// Example:
    /// ```rust,no_run
    /// // Super admin auth
    /// let response = AdminService::authenticate_admin(db, login).await;
    /// // other admin permission auth
    /// let user_can_perform_action = AdminService::user_can_perform_action(db, user_id, "admin:read").await.unwrap();
    /// ```
    pub async fn authenticate_admin(
        db: &DatabaseConnection,
        login: AdminLoginRequest,
        user_agent: Option<String>,
        ip_address: Option<String>,
    ) -> Result<AdminLoginResponse, AppError> {
        // Validate input
        validate_login_input(&login.email, &login.password)?;

        // Find user by email
        let user = UserService::find_user_by_email(db, &login.email)
            .await?
            .ok_or(AppError {
                message: "Invalid credentials".to_string(),
                status_code: StatusCode::UNAUTHORIZED,
            })?;

        // Verify password
        let is_valid = UserService::verify_password(&user, &login.password)?;
        if !is_valid {
            return Err(AppError {
                message: "Invalid credentials".to_string(),
                status_code: StatusCode::UNAUTHORIZED,
            });
        }

        // Generate session ID and JWT token
        let session_id = Uuid::new_v4();
        let jwt_secret = env::var("JWT_SECRET").unwrap_or_else(|_| "default-secret".to_string());
        let encoding_key = EncodingKey::from_secret(jwt_secret.as_ref());

        let claims = Claims {
            sub: user.id.to_string(),
            exp: (chrono::Utc::now() + chrono::Duration::hours(24)).timestamp() as usize,
            session_id: session_id.to_string(),
        };

        let token = encode(&Header::default(), &claims, &encoding_key).map_err(|_| AppError {
            message: "Failed to generate token".to_string(),
            status_code: StatusCode::INTERNAL_SERVER_ERROR,
        })?;

        // Create session record
        SessionService::create_session(
            db,
            user.id,
            user_agent,
            ip_address,
            &session_id.to_string(),
        )
        .await?;

        Ok(AdminLoginResponse {
            token,
            admin_id: user.id.to_string(),
            email: user.email,
        })
    }

    /// Get paginated audit logs with filtering
    pub async fn get_audit_logs(
        db: &DatabaseConnection,
        params: LogsQueryParams,
    ) -> Result<PaginatedResponse<AuditLogResponse>, AppError> {
        let offset = (params.page - 1) * params.limit;

        // Build query with filters
        let mut query = audit_logs::Entity::find();

        if let Some(method) = params.method {
            query = query.filter(audit_logs::Column::Method.eq(method));
        }

        if let Some(status_code) = params.status_code {
            query = query.filter(audit_logs::Column::StatusCode.eq(status_code));
        }

        if let Some(user_id) = params.user_id {
            if let Ok(uuid) = Uuid::parse_str(&user_id) {
                query = query.filter(audit_logs::Column::UserId.eq(uuid));
            }
        }

        if let Some(start_date) = params.start_date {
            if let Ok(datetime) = chrono::DateTime::parse_from_rfc3339(&start_date) {
                query = query.filter(audit_logs::Column::Timestamp.gte(datetime));
            }
        }

        if let Some(end_date) = params.end_date {
            if let Ok(datetime) = chrono::DateTime::parse_from_rfc3339(&end_date) {
                query = query.filter(audit_logs::Column::Timestamp.lte(datetime));
            }
        }

        // Get total count
        let total = query.clone().count(db).await.map_err(|e| AppError {
            message: format!("Database error: {}", e),
            status_code: StatusCode::INTERNAL_SERVER_ERROR,
        })?;

        // Get paginated results
        let logs = query
            .order_by_desc(audit_logs::Column::Timestamp)
            .offset(offset as u64)
            .limit(params.limit as u64)
            .all(db)
            .await
            .map_err(|e| AppError {
                message: format!("Database error: {}", e),
                status_code: StatusCode::INTERNAL_SERVER_ERROR,
            })?;

        let data = logs
            .into_iter()
            .map(|log| AuditLogResponse {
                id: log.id.to_string(),
                timestamp: log.timestamp.map(|t| t.to_rfc3339()),
                method: log.method,
                path: log.path,
                status_code: log.status_code,
                response_time_ms: log.response_time_ms,
                user_id: log.user_id.map(|id| id.to_string()),
                ip_address: log.ip_address,
                user_agent: log.user_agent,
                request_body: log.request_body,
                response_body: log.response_body,
                error_message: log.error_message,
            })
            .collect();

        let total_pages = (total + params.limit - 1) / params.limit;

        Ok(PaginatedResponse {
            data,
            pagination: PaginationMeta {
                page: params.page,
                limit: params.limit,
                total,
                total_pages,
            },
        })
    }

    /// Get paginated users with filtering
    pub async fn get_users(
        db: &DatabaseConnection,
        params: UsersQueryParams,
    ) -> Result<PaginatedResponse<UserResponse>, AppError> {
        let offset = (params.page - 1) * params.limit;

        // Build query with filters
        let mut query = users::Entity::find();

        if let Some(search) = params.search {
            query = query.filter(users::Column::Email.contains(&search));
        }

        // Get total count
        let total = query.clone().count(db).await.map_err(|e| AppError {
            message: format!("Database error: {}", e),
            status_code: StatusCode::INTERNAL_SERVER_ERROR,
        })?;

        // Get paginated results
        let users = query
            .order_by_desc(users::Column::CreatedAt)
            .offset(offset as u64)
            .limit(params.limit as u64)
            .all(db)
            .await
            .map_err(|e| AppError {
                message: format!("Database error: {}", e),
                status_code: StatusCode::INTERNAL_SERVER_ERROR,
            })?;

        let roles = roles::Entity::find().all(db).await.map_err(|e| AppError {
            message: format!("Database error: {}", e),
            status_code: StatusCode::INTERNAL_SERVER_ERROR,
        })?;

        let data = users
            .into_iter()
            .map(|user| UserResponse {
                id: user.id.to_string(),
                email: user.email,
                created_at: user.created_at.map(|t| t.to_rfc3339()),
                role_id: user.role_id,
                role_name: roles
                    .iter()
                    .find(|role| role.id == user.role_id.unwrap_or_default())
                    .map(|role| role.name.clone()),
            })
            .collect();

        let total_pages = (total + params.limit - 1) / params.limit;

        Ok(PaginatedResponse {
            data,
            pagination: PaginationMeta {
                page: params.page,
                limit: params.limit,
                total,
                total_pages,
            },
        })
    }

    /// Get specific user by ID using UserService
    pub async fn get_user(
        db: &DatabaseConnection,
        user_id: Uuid,
    ) -> Result<UserResponse, AppError> {
        let user = UserService::find_user_by_id(db, user_id)
            .await?
            .ok_or(AppError {
                message: "User not found".to_string(),
                status_code: StatusCode::NOT_FOUND,
            })?;

        Ok(UserResponse {
            id: user.id.to_string(),
            email: user.email,
            created_at: user.created_at.map(|t| t.to_rfc3339()),
            role_id: user.role_id,
            role_name: None, // Will be populated in a separate query if needed
        })
    }

    /// Create a new user using UserService
    pub async fn create_user(
        db: &DatabaseConnection,
        request: CreateUserRequest,
    ) -> Result<UserResponse, AppError> {
        let user = UserService::create_user_with_role(
            db,
            request.email,
            request.password,
            request.role_id,
        )
        .await?;

        Ok(UserResponse {
            id: user.id.to_string(),
            email: user.email,
            created_at: user.created_at.map(|t| t.to_rfc3339()),
            role_id: user.role_id,
            role_name: None, // Will be populated in a separate query if needed
        })
    }

    /// Update a user using UserService
    pub async fn update_user(
        db: &DatabaseConnection,
        user_id: Uuid,
        request: UpdateUserRequest,
    ) -> Result<UserResponse, AppError> {
        let user = UserService::update_user(
            db,
            user_id,
            request.email,
            request.password,
            request.role_id,
        )
        .await?;

        Ok(UserResponse {
            id: user.id.to_string(),
            email: user.email,
            created_at: user.created_at.map(|t| t.to_rfc3339()),
            role_id: user.role_id,
            role_name: None, // Will be populated in a separate query if needed
        })
    }

    /// Delete a user using UserService
    pub async fn delete_user(
        db: &DatabaseConnection,
        user_id: Uuid,
        current_admin_id: Uuid,
    ) -> Result<(), AppError> {
        // Prevent admin from deleting themselves
        if user_id == current_admin_id {
            return Err(AppError {
                message: "Cannot delete your own account".to_string(),
                status_code: StatusCode::BAD_REQUEST,
            });
        }

        UserService::delete_user(db, user_id).await
    }

    /// Get list of database tables
    pub async fn get_database_tables(
        db: &DatabaseConnection,
    ) -> Result<Vec<DatabaseTableResponse>, AppError> {
        // For SQLite, we can query the sqlite_master table
        let tables = db
            .query_all(Statement::from_sql_and_values(
                db.get_database_backend(),
                r#"SELECT name FROM sqlite_master WHERE type='table' ORDER BY name"#,
                vec![],
            ))
            .await
            .map_err(|e| AppError {
                message: format!("Database error: {}", e),
                status_code: StatusCode::INTERNAL_SERVER_ERROR,
            })?;

        let mut result = Vec::new();
        for row in tables {
            let table_name: String = row.try_get("", "name").map_err(|_| AppError {
                message: "Failed to parse table name".to_string(),
                status_code: StatusCode::INTERNAL_SERVER_ERROR,
            })?;

            // Skip system tables
            if table_name.starts_with("sqlite_")
                || table_name.starts_with("_sqlx_")
                || table_name.starts_with("seaql_")
            {
                continue;
            }

            // Get record count for each table
            let count_result = db
                .query_one(Statement::from_sql_and_values(
                    db.get_database_backend(),
                    format!("SELECT COUNT(*) as count FROM \"{}\"", table_name),
                    vec![],
                ))
                .await
                .map_err(|e| AppError {
                    message: format!("Database error: {}", e),
                    status_code: StatusCode::INTERNAL_SERVER_ERROR,
                })?;

            let record_count: u64 = count_result
                .and_then(|row| {
                    // Try different ways to access the count column
                    let result = row
                        .try_get::<i64>("", "count")
                        .map(|v| v as u64)
                        .or_else(|e| {
                            println!("Failed to get as i64: {:?}", e);
                            row.try_get::<u64>("", "count")
                        })
                        .or_else(|e| {
                            println!("Failed to get as u64: {:?}", e);
                            row.try_get::<i32>("", "count").map(|v| v as u64)
                        })
                        .or_else(|e| {
                            println!("Failed to get as i32: {:?}", e);
                            row.try_get::<u32>("", "count").map(|v| v as u64)
                        });
                    result.ok()
                })
                .unwrap_or(0);

            result.push(DatabaseTableResponse {
                name: table_name,
                record_count,
            });
        }

        Ok(result)
    }

    /// Get table records
    pub async fn get_table_records(
        db: &DatabaseConnection,
        table_name: String,
        params: TableRecordsQueryParams,
    ) -> Result<TableRecordResponse, AppError> {
        let offset = (params.page - 1) * params.limit;

        // Get column names
        let columns_result = db
            .query_all(Statement::from_sql_and_values(
                db.get_database_backend(),
                format!("PRAGMA table_info(\"{}\")", table_name),
                vec![],
            ))
            .await
            .map_err(|e| AppError {
                message: format!("Database error: {}", e),
                status_code: StatusCode::INTERNAL_SERVER_ERROR,
            })?;

        let mut columns = Vec::new();
        for row in columns_result {
            let column_name: String = row.try_get("", "name").map_err(|_| AppError {
                message: "Failed to parse column name".to_string(),
                status_code: StatusCode::INTERNAL_SERVER_ERROR,
            })?;
            columns.push(column_name);
        }

        // Get records
        let records_result = db
            .query_all(Statement::from_sql_and_values(
                db.get_database_backend(),
                format!(
                    "SELECT * FROM \"{}\" LIMIT {} OFFSET {}",
                    table_name, params.limit, offset
                ),
                vec![],
            ))
            .await
            .map_err(|e| AppError {
                message: format!("Database error: {}", e),
                status_code: StatusCode::INTERNAL_SERVER_ERROR,
            })?;

        let mut records = Vec::new();
        for row in records_result {
            let mut record = Vec::new();
            for column in &columns {
                // Try to get the value as different types and convert to JSON
                let value = if let Ok(v) = row.try_get::<String>("", column) {
                    serde_json::Value::String(v)
                } else if let Ok(v) = row.try_get::<i64>("", column) {
                    serde_json::Value::Number(serde_json::Number::from(v))
                } else if let Ok(v) = row.try_get::<f64>("", column) {
                    if let Some(n) = serde_json::Number::from_f64(v) {
                        serde_json::Value::Number(n)
                    } else {
                        serde_json::Value::Null
                    }
                } else if let Ok(v) = row.try_get::<bool>("", column) {
                    serde_json::Value::Bool(v)
                } else if let Ok(v) = row.try_get::<Vec<u8>>("", column) {
                    // Convert blob to base64 string
                    serde_json::Value::String(base64::engine::general_purpose::STANDARD.encode(v))
                } else {
                    serde_json::Value::Null
                };
                record.push(value);
            }
            records.push(record);
        }

        Ok(TableRecordResponse { columns, records })
    }

    /// Get system health status
    pub async fn get_health_status(db: &DatabaseConnection) -> HealthResponse {
        let system_metrics = SystemMonitorService::get_system_metrics(db).await;

        // Get user analytics
        let user_analytics = SystemMonitorService::get_user_analytics(db)
            .await
            .unwrap_or_else(
                |_| crate::control::services::system_monitor::UserAnalytics {
                    total_users: 0,
                    active_users_7_days: 0,
                    new_users_24_hours: 0,
                    new_users_7_days: 0,
                    new_users_30_days: 0,
                },
            );

        // Get database performance metrics
        let database_performance = DatabaseMonitorService::get_performance_metrics(db)
            .await
            .ok()
            .map(|metrics| DatabasePerformanceResponse {
                total_queries: metrics.total_queries,
                avg_execution_time_ms: metrics.avg_execution_time_ms,
                p50_execution_time_ms: metrics.p50_execution_time_ms,
                p95_execution_time_ms: metrics.p95_execution_time_ms,
                p99_execution_time_ms: metrics.p99_execution_time_ms,
                max_execution_time_ms: metrics.max_execution_time_ms,
                error_rate: metrics.error_rate,
                queries_per_second: metrics.queries_per_second,
                slow_query_count: metrics.slow_query_count,
                critical_query_count: metrics.critical_query_count,
            });

        // Get database health status
        let database_status = DatabaseMonitorService::get_database_health_status(db).await;

        // Calculate health status based on metrics
        let status = SystemMonitorService::get_health_status(&system_metrics);

        // Format memory and disk values
        let memory_usage = SystemMonitorService::get_memory_usage_percentage(&system_metrics);
        let disk_usage = SystemMonitorService::get_disk_usage_percentage(&system_metrics);

        // Get project information
        let (project_name, project_version) = SystemMonitorService::get_project_info();

        // Get server information
        let (server_host, server_port, server_protocol, environment) =
            SystemMonitorService::get_server_info();

        HealthResponse {
            status,
            timestamp: chrono::Utc::now().to_rfc3339(),
            uptime: SystemMonitorService::format_uptime(system_metrics.uptime),
            cpu_usage: system_metrics.cpu_usage,
            memory_usage,
            memory_total: SystemMonitorService::format_bytes(system_metrics.memory_total),
            memory_used: SystemMonitorService::format_bytes(system_metrics.memory_used),
            memory_available: SystemMonitorService::format_bytes(system_metrics.memory_available),
            disk_usage,
            disk_total: SystemMonitorService::format_bytes(system_metrics.disk_total),
            disk_used: SystemMonitorService::format_bytes(system_metrics.disk_used),
            disk_available: SystemMonitorService::format_bytes(system_metrics.disk_available),
            network_bytes_sent: SystemMonitorService::format_bytes(
                system_metrics.network_bytes_sent,
            ),
            network_bytes_received: SystemMonitorService::format_bytes(
                system_metrics.network_bytes_received,
            ),
            process_count: system_metrics.process_count,
            database_connections: system_metrics.database_connections,
            database_status,
            database_performance,
            // User Analytics
            total_users: user_analytics.total_users,
            active_users_7_days: user_analytics.active_users_7_days,
            new_users_24_hours: user_analytics.new_users_24_hours,
            new_users_7_days: user_analytics.new_users_7_days,
            new_users_30_days: user_analytics.new_users_30_days,
            // System Information
            system_name: system_metrics.system_name,
            kernel_version: system_metrics.kernel_version,
            os_version: system_metrics.os_version,
            host_name: system_metrics.host_name,
            cpu_count: system_metrics.cpu_count,
            temperature: system_metrics.temperature,
            project_name,
            project_version,
            // Server Information
            server_host,
            server_port,
            server_protocol,
            environment,
        }
    }

    /// Get paginated roles with filtering
    pub async fn get_roles(
        db: &DatabaseConnection,
        params: RolesQueryParams,
    ) -> Result<PaginatedResponse<RoleResponse>, AppError> {
        let offset = (params.page - 1) * params.limit;

        // Build query with filters
        let mut query = roles::Entity::find();

        if let Some(search) = params.search {
            if !search.is_empty() {
                query = query.filter(
                    roles::Column::Name
                        .contains(&search)
                        .or(roles::Column::Description.contains(&search)),
                );
            }
        }

        // Get total count
        let total = query.clone().count(db).await.map_err(|e| AppError {
            message: format!("Database error: {}", e),
            status_code: StatusCode::INTERNAL_SERVER_ERROR,
        })?;

        // Get paginated results
        let roles = query
            .offset(offset as u64)
            .limit(params.limit as u64)
            .all(db)
            .await
            .map_err(|e| AppError {
                message: format!("Database error: {}", e),
                status_code: StatusCode::INTERNAL_SERVER_ERROR,
            })?;

        // Convert to response format
        let role_responses: Vec<RoleResponse> = roles
            .into_iter()
            .map(|role| {
                let permissions: Vec<String> =
                    serde_json::from_str(&role.permissions).unwrap_or_else(|_| vec![]);

                RoleResponse {
                    id: role.id,
                    name: role.name,
                    description: role.description,
                    permissions,
                    created_at: role.created_at.map(|dt| dt.to_rfc3339()),
                    updated_at: role.updated_at.map(|dt| dt.to_rfc3339()),
                }
            })
            .collect();

        let total_pages = (total as f64 / params.limit as f64).ceil() as u32;

        Ok(PaginatedResponse {
            data: role_responses,
            pagination: PaginationMeta {
                page: params.page as u64,
                limit: params.limit as u64,
                total: total as u64,
                total_pages: total_pages as u64,
            },
        })
    }

    /// Get role by ID
    pub async fn get_role(db: &DatabaseConnection, role_id: i32) -> Result<RoleResponse, AppError> {
        let role = roles::Entity::find_by_id(role_id)
            .one(db)
            .await
            .map_err(|e| AppError {
                message: format!("Database error: {}", e),
                status_code: StatusCode::INTERNAL_SERVER_ERROR,
            })?
            .ok_or(AppError {
                message: "Role not found".to_string(),
                status_code: StatusCode::NOT_FOUND,
            })?;

        let permissions: Vec<String> =
            serde_json::from_str(&role.permissions).unwrap_or_else(|_| vec![]);

        Ok(RoleResponse {
            id: role.id,
            name: role.name,
            description: role.description,
            permissions,
            created_at: role.created_at.map(|dt| dt.to_rfc3339()),
            updated_at: role.updated_at.map(|dt| dt.to_rfc3339()),
        })
    }

    /// Create a new role
    pub async fn create_role(
        db: &DatabaseConnection,
        request: CreateRoleRequest,
    ) -> Result<RoleResponse, AppError> {
        // Check if role name already exists
        let existing_role = roles::Entity::find()
            .filter(roles::Column::Name.eq(&request.name))
            .one(db)
            .await
            .map_err(|e| AppError {
                message: format!("Database error: {}", e),
                status_code: StatusCode::INTERNAL_SERVER_ERROR,
            })?;

        if existing_role.is_some() {
            return Err(AppError {
                message: "Role name already exists".to_string(),
                status_code: StatusCode::CONFLICT,
            });
        }

        // Convert permissions to JSON string
        let permissions_json =
            serde_json::to_string(&request.permissions).map_err(|_| AppError {
                message: "Invalid permissions format".to_string(),
                status_code: StatusCode::BAD_REQUEST,
            })?;

        // Create new role
        let role_model = roles::ActiveModel {
            name: Set(request.name),
            description: Set(request.description),
            permissions: Set(permissions_json),
            ..Default::default()
        };

        let role = role_model.insert(db).await.map_err(|e| AppError {
            message: format!("Database error: {}", e),
            status_code: StatusCode::INTERNAL_SERVER_ERROR,
        })?;

        Ok(RoleResponse {
            id: role.id,
            name: role.name,
            description: role.description,
            permissions: request.permissions,
            created_at: role.created_at.map(|dt| dt.to_rfc3339()),
            updated_at: role.updated_at.map(|dt| dt.to_rfc3339()),
        })
    }

    /// Update an existing role
    pub async fn update_role(
        db: &DatabaseConnection,
        role_id: i32,
        request: UpdateRoleRequest,
    ) -> Result<RoleResponse, AppError> {
        // Get existing role
        let role = roles::Entity::find_by_id(role_id)
            .one(db)
            .await
            .map_err(|e| AppError {
                message: format!("Database error: {}", e),
                status_code: StatusCode::INTERNAL_SERVER_ERROR,
            })?
            .ok_or(AppError {
                message: "Role not found".to_string(),
                status_code: StatusCode::NOT_FOUND,
            })?;

        // Check if new name conflicts with existing role
        if let Some(new_name) = &request.name {
            let existing_role = roles::Entity::find()
                .filter(roles::Column::Name.eq(new_name))
                .filter(roles::Column::Id.ne(role_id))
                .one(db)
                .await
                .map_err(|e| AppError {
                    message: format!("Database error: {}", e),
                    status_code: StatusCode::INTERNAL_SERVER_ERROR,
                })?;

            if existing_role.is_some() {
                return Err(AppError {
                    message: "Role name already exists".to_string(),
                    status_code: StatusCode::CONFLICT,
                });
            }
        }

        // Prepare update model
        let mut role_model: roles::ActiveModel = role.into();

        if let Some(name) = request.name {
            role_model.name = Set(name);
        }

        if let Some(description) = request.description {
            role_model.description = Set(Some(description));
        }

        if let Some(permissions) = request.permissions {
            let permissions_json = serde_json::to_string(&permissions).map_err(|_| AppError {
                message: "Invalid permissions format".to_string(),
                status_code: StatusCode::BAD_REQUEST,
            })?;
            role_model.permissions = Set(permissions_json);
        }

        // Update timestamp
        role_model.updated_at = Set(Some(chrono::Utc::now().fixed_offset()));

        // Save updated role
        let updated_role = role_model.update(db).await.map_err(|e| AppError {
            message: format!("Database error: {}", e),
            status_code: StatusCode::INTERNAL_SERVER_ERROR,
        })?;

        // Get permissions for response
        let permissions: Vec<String> =
            serde_json::from_str(&updated_role.permissions).unwrap_or_else(|_| vec![]);

        Ok(RoleResponse {
            id: updated_role.id,
            name: updated_role.name,
            description: updated_role.description,
            permissions,
            created_at: updated_role.created_at.map(|dt| dt.to_rfc3339()),
            updated_at: updated_role.updated_at.map(|dt| dt.to_rfc3339()),
        })
    }

    /// Delete a role
    pub async fn delete_role(db: &DatabaseConnection, role_id: i32) -> Result<(), AppError> {
        // Check if role exists
        let _role = roles::Entity::find_by_id(role_id)
            .one(db)
            .await
            .map_err(|e| AppError {
                message: format!("Database error: {}", e),
                status_code: StatusCode::INTERNAL_SERVER_ERROR,
            })?
            .ok_or(AppError {
                message: "Role not found".to_string(),
                status_code: StatusCode::NOT_FOUND,
            })?;

        // Check if role is assigned to any users
        let users_with_role = users::Entity::find()
            .filter(users::Column::RoleId.eq(role_id))
            .count(db)
            .await
            .map_err(|e| AppError {
                message: format!("Database error: {}", e),
                status_code: StatusCode::INTERNAL_SERVER_ERROR,
            })?;

        if users_with_role > 0 {
            return Err(AppError {
                message: "Cannot delete role: it is assigned to users".to_string(),
                status_code: StatusCode::CONFLICT,
            });
        }

        // Delete the role
        roles::Entity::delete_by_id(role_id)
            .exec(db)
            .await
            .map_err(|e| AppError {
                message: format!("Database error: {}", e),
                status_code: StatusCode::INTERNAL_SERVER_ERROR,
            })?;

        Ok(())
    }

    /// Check if a user has a specific permission
    pub async fn check_permission(
        db: &DatabaseConnection,
        request: PermissionCheckRequest,
    ) -> Result<PermissionCheckResponse, AppError> {
        let user_id = Uuid::parse_str(&request.user_id).map_err(|_| AppError {
            message: "Invalid user ID".to_string(),
            status_code: StatusCode::BAD_REQUEST,
        })?;

        // Get user with role information
        let user = users::Entity::find_by_id(user_id)
            .find_also_related(roles::Entity)
            .one(db)
            .await
            .map_err(|e| AppError {
                message: format!("Database error: {}", e),
                status_code: StatusCode::INTERNAL_SERVER_ERROR,
            })?
            .ok_or(AppError {
                message: "User not found".to_string(),
                status_code: StatusCode::NOT_FOUND,
            })?;

        let (_, role_model) = user;

        // Check role-based permissions
        if let Some(role) = role_model {
            let permissions: Vec<String> =
                serde_json::from_str(&role.permissions).unwrap_or_else(|_| vec![]);

            let has_permission =
                permissions.contains(&"*".to_string()) || permissions.contains(&request.permission);

            return Ok(PermissionCheckResponse {
                has_permission,
                user_role: Some(role.name),
                required_permission: request.permission,
            });
        }

        // No role assigned
        Ok(PermissionCheckResponse {
            has_permission: false,
            user_role: None,
            required_permission: request.permission,
        })
    }

    /// Helper function to check if a user can perform an action
    #[allow(dead_code)]
    pub async fn user_can_perform_action(
        db: &DatabaseConnection,
        user_id: Uuid,
        permission: &str,
    ) -> Result<bool, AppError> {
        let response = Self::check_permission(
            db,
            PermissionCheckRequest {
                user_id: user_id.to_string(),
                permission: permission.to_string(),
            },
        )
        .await?;

        Ok(response.has_permission)
    }

    /// Get sessions for a specific user
    pub async fn get_user_sessions(
        db: &DatabaseConnection,
        user_id: Uuid,
    ) -> Result<Vec<SessionResponse>, AppError> {
        // Get sessions from SessionService
        let sessions = SessionService::get_user_sessions(db, user_id).await?;

        // Convert to response format
        let session_responses: Vec<SessionResponse> = sessions
            .into_iter()
            .map(|session| SessionResponse {
                id: session.id.to_string(),
                user_id: session.user_id.to_string(),
                device_info: session
                    .user_agent
                    .unwrap_or_else(|| "Unknown Device".to_string()),
                ip_address: session.ip_address,
                created_at: session
                    .created_at
                    .map(|dt| dt.to_rfc3339())
                    .unwrap_or_default(),
                last_activity: session
                    .last_activity
                    .map(|dt| dt.to_rfc3339())
                    .unwrap_or_default(),
                expires_at: session.expires_at.to_rfc3339(),
                is_current: false, // Will be determined on frontend based on current session
            })
            .collect();

        Ok(session_responses)
    }

    /// Invalidate a specific session
    pub async fn invalidate_user_session(
        db: &DatabaseConnection,
        session_id: Uuid,
    ) -> Result<(), AppError> {
        SessionService::invalidate_session(db, session_id).await
    }

    /// Invalidate all sessions for a user
    pub async fn invalidate_all_user_sessions(
        db: &DatabaseConnection,
        user_id: Uuid,
    ) -> Result<u64, AppError> {
        SessionService::invalidate_all_user_sessions(db, user_id).await
    }
}
