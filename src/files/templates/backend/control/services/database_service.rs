use crate::entity::models::{prelude::*, *};
use crate::infrastructure::query_performance::record_database_query;
use chrono::{Duration, Utc};
use sea_orm::*;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::time::Instant;
use uuid::Uuid;

/// Database performance monitoring service
pub struct DatabaseMonitorService;

/// Database performance metrics following industry standards
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DatabasePerformanceMetrics {
    pub total_queries: u64,
    pub avg_execution_time_ms: f64,
    pub p50_execution_time_ms: f64,
    pub p95_execution_time_ms: f64,
    pub p99_execution_time_ms: f64,
    pub max_execution_time_ms: f64,
    pub error_rate: f64,
    pub queries_per_second: f64,
    pub slow_query_count: u64,     // queries > 500ms
    pub critical_query_count: u64, // queries > 1000ms
}

/// Query type breakdown metrics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QueryTypeMetrics {
    pub select_count: u64,
    pub insert_count: u64,
    pub update_count: u64,
    pub delete_count: u64,
    pub other_count: u64,
}

/// Table-specific performance metrics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TablePerformanceMetrics {
    pub table_name: String,
    pub query_count: u64,
    pub avg_execution_time_ms: f64,
    pub total_rows_affected: u64,
}

impl DatabaseMonitorService {
    /// Record a database query metric
    pub async fn record_query_metric(
        db: &DatabaseConnection,
        query_hash: String,
        query_type: String,
        table_name: Option<String>,
        execution_time_ms: i64,
        rows_affected: Option<i64>,
        error_message: Option<String>,
    ) -> Result<(), DbErr> {
        let metric = database_metrics::ActiveModel {
            id: Set(Uuid::new_v4()),
            query_hash: Set(query_hash),
            query_type: Set(query_type),
            table_name: Set(table_name),
            execution_time_ms: Set(execution_time_ms),
            rows_affected: Set(rows_affected),
            error_message: Set(error_message),
            timestamp: Set(Utc::now().into()),
            created_at: Set(Utc::now().into()),
        };

        DatabaseMetrics::insert(metric).exec(db).await?;
        Ok(())
    }

    /// Get database performance metrics for the last hour (industry standard)
    pub async fn get_performance_metrics(
        db: &DatabaseConnection,
    ) -> Result<DatabasePerformanceMetrics, DbErr> {
        let one_hour_ago = Utc::now() - Duration::hours(1);

        // Get all metrics from the last hour
        let metrics = DatabaseMetrics::find()
            .filter(database_metrics::Column::Timestamp.gte(one_hour_ago))
            .all(db)
            .await?;

        if metrics.is_empty() {
            return Ok(DatabasePerformanceMetrics {
                total_queries: 0,
                avg_execution_time_ms: 0.0,
                p50_execution_time_ms: 0.0,
                p95_execution_time_ms: 0.0,
                p99_execution_time_ms: 0.0,
                max_execution_time_ms: 0.0,
                error_rate: 0.0,
                queries_per_second: 0.0,
                slow_query_count: 0,
                critical_query_count: 0,
            });
        }

        // Extract execution times for percentile calculation
        let mut execution_times: Vec<i64> = metrics.iter().map(|m| m.execution_time_ms).collect();
        execution_times.sort_unstable();

        let total_queries = metrics.len() as u64;
        let total_time: i64 = execution_times.iter().sum();
        let avg_execution_time_ms = total_time as f64 / total_queries as f64;

        // Calculate percentiles
        let p50_execution_time_ms = Self::calculate_percentile(&execution_times, 50.0);
        let p95_execution_time_ms = Self::calculate_percentile(&execution_times, 95.0);
        let p99_execution_time_ms = Self::calculate_percentile(&execution_times, 99.0);
        let max_execution_time_ms = *execution_times.last().unwrap_or(&0) as f64;

        // Calculate error rate
        let error_count = metrics.iter().filter(|m| m.error_message.is_some()).count() as u64;
        let error_rate = if total_queries > 0 {
            (error_count as f64 / total_queries as f64) * 100.0
        } else {
            0.0
        };

        // Calculate queries per second (over the last hour)
        let queries_per_second = total_queries as f64 / 3600.0; // 3600 seconds in an hour

        // Count slow and critical queries
        let slow_query_count = metrics.iter().filter(|m| m.execution_time_ms > 500).count() as u64;
        let critical_query_count = metrics
            .iter()
            .filter(|m| m.execution_time_ms > 1000)
            .count() as u64;

        Ok(DatabasePerformanceMetrics {
            total_queries,
            avg_execution_time_ms,
            p50_execution_time_ms,
            p95_execution_time_ms,
            p99_execution_time_ms,
            max_execution_time_ms,
            error_rate,
            queries_per_second,
            slow_query_count,
            critical_query_count,
        })
    }

    /// Get query type breakdown metrics
    #[allow(dead_code)]
    pub async fn get_query_type_metrics(
        db: &DatabaseConnection,
    ) -> Result<QueryTypeMetrics, DbErr> {
        let one_hour_ago = Utc::now() - Duration::hours(1);

        let metrics = DatabaseMetrics::find()
            .filter(database_metrics::Column::Timestamp.gte(one_hour_ago))
            .all(db)
            .await?;

        let mut type_counts = HashMap::new();
        for metric in &metrics {
            let count = type_counts.entry(&metric.query_type).or_insert(0);
            *count += 1;
        }

        Ok(QueryTypeMetrics {
            select_count: *type_counts.get(&String::from("SELECT")).unwrap_or(&0),
            insert_count: *type_counts.get(&String::from("INSERT")).unwrap_or(&0),
            update_count: *type_counts.get(&String::from("UPDATE")).unwrap_or(&0),
            delete_count: *type_counts.get(&String::from("DELETE")).unwrap_or(&0),
            other_count: type_counts.values().sum::<u64>()
                - type_counts.get(&String::from("SELECT")).unwrap_or(&0)
                - type_counts.get(&String::from("INSERT")).unwrap_or(&0)
                - type_counts.get(&String::from("UPDATE")).unwrap_or(&0)
                - type_counts.get(&String::from("DELETE")).unwrap_or(&0),
        })
    }

    /// Get table-specific performance metrics
    #[allow(dead_code)]
    pub async fn get_table_performance_metrics(
        db: &DatabaseConnection,
    ) -> Result<Vec<TablePerformanceMetrics>, DbErr> {
        let one_hour_ago = Utc::now() - Duration::hours(1);

        let metrics = DatabaseMetrics::find()
            .filter(database_metrics::Column::Timestamp.gte(one_hour_ago))
            .filter(database_metrics::Column::TableName.is_not_null())
            .all(db)
            .await?;

        let mut table_metrics: HashMap<String, (u64, i64, u64)> = HashMap::new();

        for metric in &metrics {
            if let Some(table_name) = &metric.table_name {
                let entry = table_metrics.entry(table_name.clone()).or_insert((0, 0, 0));
                entry.0 += 1; // query count
                entry.1 += metric.execution_time_ms; // total execution time
                entry.2 += metric.rows_affected.unwrap_or(0) as u64; // total rows affected
            }
        }

        let mut result = Vec::new();
        for (table_name, (query_count, total_time, total_rows)) in table_metrics {
            let avg_execution_time_ms = if query_count > 0 {
                total_time as f64 / query_count as f64
            } else {
                0.0
            };

            result.push(TablePerformanceMetrics {
                table_name,
                query_count,
                avg_execution_time_ms,
                total_rows_affected: total_rows,
            });
        }

        // Sort by query count descending
        result.sort_by(|a, b| b.query_count.cmp(&a.query_count));
        Ok(result)
    }

    /// Clean up old metrics (keep last 7 days)
    #[allow(dead_code)]
    pub async fn cleanup_old_metrics(db: &DatabaseConnection) -> Result<u64, DbErr> {
        let seven_days_ago = Utc::now() - Duration::days(7);

        let result = DatabaseMetrics::delete_many()
            .filter(database_metrics::Column::Timestamp.lt(seven_days_ago))
            .exec(db)
            .await?;

        Ok(result.rows_affected)
    }

    /// Calculate percentile from sorted array
    fn calculate_percentile(sorted_values: &[i64], percentile: f64) -> f64 {
        if sorted_values.is_empty() {
            return 0.0;
        }

        let index = (percentile / 100.0 * (sorted_values.len() - 1) as f64).round() as usize;
        *sorted_values.get(index).unwrap_or(&0) as f64
    }

    /// Get database health status based on performance metrics
    pub async fn get_database_health_status(db: &DatabaseConnection) -> String {
        match Self::get_performance_metrics(db).await {
            Ok(metrics) => {
                // Industry standard thresholds
                if metrics.p95_execution_time_ms > 1000.0 || metrics.error_rate > 5.0 {
                    "Critical".to_string()
                } else if metrics.p95_execution_time_ms > 500.0 || metrics.error_rate > 1.0 {
                    "Warning".to_string()
                } else {
                    "Healthy".to_string()
                }
            }
            Err(_) => "Unknown".to_string(),
        }
    }
}

/// Database service wrapper that automatically tracks performance metrics
pub struct DatabaseService;

impl DatabaseService {
    /// Execute a query with automatic performance tracking
    #[allow(dead_code)]
    pub async fn execute_with_tracking<F, T, E>(
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

    /// Find all records with tracking
    #[allow(dead_code)]
    pub async fn find_all_with_tracking<T>(
        db: &DatabaseConnection,
        table_name: &str,
        query: Select<T>,
    ) -> Result<Vec<T::Model>, DbErr>
    where
        T: EntityTrait,
    {
        let start = Instant::now();
        let result = query.all(db).await;
        let execution_time = start.elapsed();

        // Record the operation
        let error_message = result.as_ref().err().map(|e| e.to_string());
        let _ = record_database_query(
            db,
            &format!("SELECT * FROM {}", table_name),
            "SELECT",
            Some(table_name),
            execution_time.as_millis() as i64,
            result.as_ref().map(|r| r.len() as i64).ok(),
            error_message.as_deref(),
        )
        .await;

        result
    }

    /// Find one record with tracking
    #[allow(dead_code)]
    pub async fn find_one_with_tracking<T>(
        db: &DatabaseConnection,
        table_name: &str,
        query: Select<T>,
    ) -> Result<Option<T::Model>, DbErr>
    where
        T: EntityTrait,
    {
        let start = Instant::now();
        let result = query.one(db).await;
        let execution_time = start.elapsed();

        // Record the operation
        let error_message = result.as_ref().err().map(|e| e.to_string());
        let _ = record_database_query(
            db,
            &format!("SELECT * FROM {} LIMIT 1", table_name),
            "SELECT",
            Some(table_name),
            execution_time.as_millis() as i64,
            result
                .as_ref()
                .map(|r| if r.is_some() { Some(1) } else { None })
                .unwrap_or(None),
            error_message.as_deref(),
        )
        .await;

        result
    }
}
