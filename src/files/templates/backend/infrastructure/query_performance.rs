use crate::control::services::database_service::DatabaseMonitorService;
use sea_orm::DatabaseConnection;
use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};
use std::time::Instant;

/// Helper function to record a database query metric
/// This should be called from within request handlers when database queries are executed
pub async fn record_database_query(
    db: &DatabaseConnection,
    query_sql: &str,
    query_type: &str,
    table_name: Option<&str>,
    execution_time_ms: i64,
    rows_affected: Option<i64>,
    error_message: Option<&str>,
) {
    // Create a simple hash of the query for grouping similar queries
    let mut hasher = DefaultHasher::new();
    query_sql.hash(&mut hasher);
    let query_hash = hasher.finish().to_string();

    let _ = DatabaseMonitorService::record_query_metric(
        db,
        query_hash,
        query_type.to_string(),
        table_name.map(|s| s.to_string()),
        execution_time_ms,
        rows_affected,
        error_message.map(|s| s.to_string()),
    )
    .await;
}

/// Wrapper for database operations that automatically tracks performance
#[allow(dead_code)]
pub async fn track_database_operation<F, T, E>(
    db: &DatabaseConnection,
    operation_name: &str,
    table_name: Option<&str>,
    operation: F,
) -> Result<T, E>
where
    F: FnOnce() -> Result<T, E>,
    E: std::fmt::Display,
{
    let start = Instant::now();
    let result = operation();
    let execution_time = start.elapsed();

    // Record the operation (ignore errors in recording to avoid masking the actual error)
    let error_message = result.as_ref().err().map(|e| e.to_string());
    let _ = record_database_query(
        db,
        operation_name,
        "CUSTOM_OPERATION",
        table_name,
        execution_time.as_millis() as i64,
        None,
        error_message.as_deref(),
    )
    .await;

    result
}
