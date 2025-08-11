use sea_orm_migration::{prelude::*, schema::*};

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        // Create the users table
        manager
            .create_table(
                Table::create()
                    .table(Users::Table)
                    .col(ColumnDef::new(Users::Id).uuid().not_null().primary_key())
                    .col(
                        ColumnDef::new(Users::Email)
                            .string()
                            .not_null()
                            .unique_key(),
                    )
                    .col(ColumnDef::new(Users::PasswordHash).string().not_null())
                    .col(
                        ColumnDef::new(Users::CreatedAt)
                            .timestamp_with_time_zone()
                            .default(Expr::current_timestamp()),
                    )
                    .col(
                        ColumnDef::new(Users::LastLogin)
                            .timestamp_with_time_zone()
                            .null(),
                    )
                    .col(ColumnDef::new(Users::RoleId).integer().null())
                    .col(
                        ColumnDef::new(Users::EmailVerified)
                            .boolean()
                            .not_null()
                            .default(false),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk_users_role_id")
                            .from(Users::Table, Users::RoleId)
                            .to(Roles::Table, Roles::Id)
                            .on_delete(ForeignKeyAction::SetNull)
                            .on_update(ForeignKeyAction::Cascade),
                    )
                    .to_owned(),
            )
            .await?;

        // Create the audit logs table
        manager
            .create_table(
                Table::create()
                    .table(AuditLogs::Table)
                    .col(
                        ColumnDef::new(AuditLogs::Id)
                            .uuid()
                            .not_null()
                            .primary_key(),
                    )
                    .col(
                        ColumnDef::new(AuditLogs::Timestamp)
                            .timestamp_with_time_zone()
                            .default(Expr::current_timestamp()),
                    )
                    .col(ColumnDef::new(AuditLogs::Method).string_len(10).not_null())
                    .col(ColumnDef::new(AuditLogs::Path).text().not_null())
                    .col(ColumnDef::new(AuditLogs::StatusCode).integer())
                    .col(ColumnDef::new(AuditLogs::ResponseTimeMs).integer())
                    .col(ColumnDef::new(AuditLogs::UserId).uuid())
                    .col(ColumnDef::new(AuditLogs::IpAddress).string_len(45))
                    .col(ColumnDef::new(AuditLogs::UserAgent).text())
                    .col(ColumnDef::new(AuditLogs::RequestBody).text())
                    .col(ColumnDef::new(AuditLogs::ResponseBody).text())
                    .col(ColumnDef::new(AuditLogs::ErrorMessage).text())
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk_audit_logs_user_id")
                            .from(AuditLogs::Table, AuditLogs::UserId)
                            .to(Users::Table, Users::Id)
                            .on_delete(ForeignKeyAction::SetNull),
                    )
                    .to_owned(),
            )
            .await?;

        // Create the roles table
        manager
            .create_table(
                Table::create()
                    .table(Roles::Table)
                    .if_not_exists()
                    .col(pk_auto(Roles::Id))
                    .col(string(Roles::Name).not_null().unique_key())
                    .col(string(Roles::Description).null())
                    .col(string(Roles::Permissions).not_null()) // JSON string of permissions
                    .col(
                        timestamp_with_time_zone(Roles::CreatedAt)
                            .default(Expr::current_timestamp()),
                    )
                    .col(
                        timestamp_with_time_zone(Roles::UpdatedAt)
                            .default(Expr::current_timestamp()),
                    )
                    .to_owned(),
            )
            .await?;

        // Create the user sessions table
        manager
            .create_table(
                Table::create()
                    .table(UserSessions::Table)
                    .col(
                        ColumnDef::new(UserSessions::Id)
                            .uuid()
                            .not_null()
                            .primary_key(),
                    )
                    .col(ColumnDef::new(UserSessions::UserId).uuid().not_null())
                    .col(
                        ColumnDef::new(UserSessions::SessionToken)
                            .string()
                            .not_null()
                            .unique_key(),
                    )
                    .col(ColumnDef::new(UserSessions::UserAgent).text().null())
                    .col(ColumnDef::new(UserSessions::IpAddress).string().null())
                    .col(
                        ColumnDef::new(UserSessions::CreatedAt)
                            .timestamp_with_time_zone()
                            .default(Expr::current_timestamp()),
                    )
                    .col(
                        ColumnDef::new(UserSessions::LastActivity)
                            .timestamp_with_time_zone()
                            .default(Expr::current_timestamp()),
                    )
                    .col(
                        ColumnDef::new(UserSessions::ExpiresAt)
                            .timestamp_with_time_zone()
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(UserSessions::IsActive)
                            .boolean()
                            .default(true),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk_user_sessions_user_id")
                            .from(UserSessions::Table, UserSessions::UserId)
                            .to(Users::Table, Users::Id)
                            .on_delete(ForeignKeyAction::Cascade)
                            .on_update(ForeignKeyAction::Cascade),
                    )
                    .to_owned(),
            )
            .await?;

        // Create table database metrics
        manager
            .create_table(
                Table::create()
                    .table(DatabaseMetrics::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(DatabaseMetrics::Id)
                            .uuid()
                            .not_null()
                            .primary_key(),
                    )
                    .col(
                        ColumnDef::new(DatabaseMetrics::QueryHash)
                            .string()
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(DatabaseMetrics::QueryType)
                            .string()
                            .not_null(),
                    )
                    .col(ColumnDef::new(DatabaseMetrics::TableName).string().null())
                    .col(
                        ColumnDef::new(DatabaseMetrics::ExecutionTimeMs)
                            .big_integer()
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(DatabaseMetrics::RowsAffected)
                            .big_integer()
                            .null(),
                    )
                    .col(ColumnDef::new(DatabaseMetrics::ErrorMessage).text().null())
                    .col(
                        ColumnDef::new(DatabaseMetrics::Timestamp)
                            .timestamp_with_time_zone()
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(DatabaseMetrics::CreatedAt)
                            .timestamp_with_time_zone()
                            .not_null(),
                    )
                    .to_owned(),
            )
            .await?;

        // create indexes
        manager
            .create_index(
                Index::create()
                    .name("idx_user_sessions_user_id")
                    .table(UserSessions::Table)
                    .col(UserSessions::UserId)
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .name("idx_user_sessions_token")
                    .table(UserSessions::Table)
                    .col(UserSessions::SessionToken)
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .name("idx_user_sessions_active")
                    .table(UserSessions::Table)
                    .col(UserSessions::IsActive)
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .name("idx_user_sessions_expires")
                    .table(UserSessions::Table)
                    .col(UserSessions::ExpiresAt)
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .name("idx_user_sessions_user_active")
                    .table(UserSessions::Table)
                    .col(UserSessions::UserId)
                    .col(UserSessions::IsActive)
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .name("idx_database_metrics_timestamp")
                    .table(DatabaseMetrics::Table)
                    .col(DatabaseMetrics::Timestamp)
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .name("idx_database_metrics_query_type")
                    .table(DatabaseMetrics::Table)
                    .col(DatabaseMetrics::QueryType)
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .name("idx_database_metrics_table_name")
                    .table(DatabaseMetrics::Table)
                    .col(DatabaseMetrics::TableName)
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(Users::Table).to_owned())
            .await?;
        manager
            .drop_table(Table::drop().table(AuditLogs::Table).to_owned())
            .await?;
        manager
            .drop_table(Table::drop().table(Roles::Table).to_owned())
            .await?;
        manager
            .drop_table(Table::drop().table(UserSessions::Table).to_owned())
            .await?;
        manager
            .drop_table(Table::drop().table(DatabaseMetrics::Table).to_owned())
            .await
    }
}

#[derive(DeriveIden)]
enum Users {
    Table,
    Id,
    Email,
    PasswordHash,
    CreatedAt,
    LastLogin,
    RoleId,
    EmailVerified,
}

#[derive(DeriveIden)]
enum AuditLogs {
    Table,
    Id,
    Timestamp,
    Method,
    Path,
    StatusCode,
    ResponseTimeMs,
    UserId,
    IpAddress,
    UserAgent,
    RequestBody,
    ResponseBody,
    ErrorMessage,
}

#[derive(DeriveIden)]
enum Roles {
    Table,
    Id,
    Name,
    Description,
    Permissions,
    CreatedAt,
    UpdatedAt,
}

#[derive(DeriveIden)]
enum UserSessions {
    Table,
    Id,
    UserId,
    SessionToken,
    UserAgent,
    IpAddress,
    CreatedAt,
    LastActivity,
    ExpiresAt,
    IsActive,
}

#[derive(DeriveIden)]
enum DatabaseMetrics {
    Table,
    Id,
    QueryHash,
    QueryType,
    TableName,
    ExecutionTimeMs,
    RowsAffected,
    ErrorMessage,
    Timestamp,
    CreatedAt,
}
