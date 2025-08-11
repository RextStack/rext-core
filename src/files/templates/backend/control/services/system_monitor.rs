use crate::control::services::{
    database_service::{DatabaseMonitorService, DatabasePerformanceMetrics},
    server_config::ServerConfigService,
};
use chrono::{Duration, Utc};
use sea_orm::DatabaseConnection;
use std::fs;
use sysinfo::{Components, Disks, Networks, System};

/// System monitoring service for collecting system metrics
pub struct SystemMonitorService;

/// System metrics data structure
#[derive(Debug, Clone)]
pub struct SystemMetrics {
    pub cpu_usage: f32,
    pub memory_total: u64,
    pub memory_used: u64,
    pub memory_available: u64,
    pub disk_total: u64,
    pub disk_used: u64,
    pub disk_available: u64,
    pub network_bytes_sent: u64,
    pub network_bytes_received: u64,
    pub uptime: u64,
    pub process_count: usize,
    pub database_connections: Option<u32>,
    #[allow(dead_code)]
    pub database_performance: Option<DatabasePerformanceMetrics>,
    // System information
    pub system_name: Option<String>,
    pub kernel_version: Option<String>,
    pub os_version: Option<String>,
    pub host_name: Option<String>,
    pub cpu_count: usize,
    pub temperature: Option<f32>,
}

/// User analytics data structure
#[derive(Debug, Clone)]
pub struct UserAnalytics {
    pub total_users: u64,
    pub active_users_7_days: u64,
    pub new_users_24_hours: u64,
    pub new_users_7_days: u64,
    pub new_users_30_days: u64,
}

impl SystemMonitorService {
    /// Get current system metrics
    pub async fn get_system_metrics(db: &DatabaseConnection) -> SystemMetrics {
        let mut sys = System::new_all();
        sys.refresh_all();

        // Get CPU usage (average across all cores)
        let cpu_usage = sys.global_cpu_usage();

        // Get memory information
        let memory_total = sys.total_memory();
        let memory_used = sys.used_memory();
        let memory_available = sys.free_memory();

        // Get disk information
        let disks = Disks::new_with_refreshed_list();
        let mut disk_total = 0u64;
        let mut disk_used = 0u64;
        let mut disk_available = 0u64;

        for disk in &disks {
            disk_total += disk.total_space();
            disk_used += disk.total_space() - disk.available_space();
            disk_available += disk.available_space();
        }

        // Get network information
        let networks = Networks::new_with_refreshed_list();
        let mut network_bytes_sent = 0u64;
        let mut network_bytes_received = 0u64;

        for (_, data) in &networks {
            network_bytes_sent += data.total_transmitted();
            network_bytes_received += data.total_received();
        }

        // Get system uptime
        let uptime = System::uptime();

        // Get process count
        let process_count = sys.processes().len();

        // Get system information
        let system_name = System::name();
        let kernel_version = System::kernel_version();
        let os_version = System::os_version();
        let host_name = System::host_name();
        let cpu_count = sys.cpus().len();

        // Get temperature information if available
        let components = Components::new_with_refreshed_list();
        let temperature = components
            .iter()
            .find(|component| component.label().to_lowercase().contains("cpu"))
            .and_then(|component| component.temperature());

        println!("temperature: {:?}", temperature);
        // Get database connection count (if available)
        let database_connections = Self::get_database_connections(db).await;

        // Get database performance metrics
        let database_performance = DatabaseMonitorService::get_performance_metrics(db)
            .await
            .ok();

        SystemMetrics {
            cpu_usage,
            memory_total,
            memory_used,
            memory_available,
            disk_total,
            disk_used,
            disk_available,
            network_bytes_sent,
            network_bytes_received,
            uptime,
            process_count,
            database_connections,
            database_performance,
            system_name,
            kernel_version,
            os_version,
            host_name,
            cpu_count,
            temperature,
        }
    }

    /// Get user analytics
    pub async fn get_user_analytics(
        db: &DatabaseConnection,
    ) -> Result<UserAnalytics, sea_orm::DbErr> {
        use crate::entity::models::{prelude::*, *};
        use sea_orm::*;

        let now = Utc::now();
        let seven_days_ago = now - Duration::days(7);
        let one_day_ago = now - Duration::days(1);
        let thirty_days_ago = now - Duration::days(30);

        // Get total users
        let total_users = Users::find().count(db).await?;

        // Get active users (logged in within last 7 days)
        let active_users_7_days = Users::find()
            .filter(users::Column::LastLogin.gte(seven_days_ago))
            .count(db)
            .await?;

        // Get new users in last 24 hours
        let new_users_24_hours = Users::find()
            .filter(users::Column::CreatedAt.gte(one_day_ago))
            .count(db)
            .await?;

        // Get new users in last 7 days
        let new_users_7_days = Users::find()
            .filter(users::Column::CreatedAt.gte(seven_days_ago))
            .count(db)
            .await?;

        // Get new users in last 30 days
        let new_users_30_days = Users::find()
            .filter(users::Column::CreatedAt.gte(thirty_days_ago))
            .count(db)
            .await?;

        Ok(UserAnalytics {
            total_users,
            active_users_7_days,
            new_users_24_hours,
            new_users_7_days,
            new_users_30_days,
        })
    }

    /// Get database connection count from SeaORM connection
    async fn get_database_connections(db: &DatabaseConnection) -> Option<u32> {
        // Try to get the underlying sqlx pool from SeaORM
        // This is a bit of a hack since SeaORM doesn't expose the pool directly
        // We'll try to execute a simple query to check if the connection is alive
        // and return None if we can't get the connection count
        match db.ping().await {
            Ok(_) => {
                // For now, we'll return None since we can't easily get the connection count
                // from SeaORM's DatabaseConnection. We could implement a more sophisticated
                // approach if needed.
                None
            }
            Err(_) => None,
        }
    }

    /// Get memory usage percentage
    pub fn get_memory_usage_percentage(metrics: &SystemMetrics) -> f32 {
        if metrics.memory_total > 0 {
            (metrics.memory_used as f32 / metrics.memory_total as f32) * 100.0
        } else {
            0.0
        }
    }

    /// Get disk usage percentage
    pub fn get_disk_usage_percentage(metrics: &SystemMetrics) -> f32 {
        if metrics.disk_total > 0 {
            (metrics.disk_used as f32 / metrics.disk_total as f32) * 100.0
        } else {
            0.0
        }
    }

    /// Format bytes to human readable format
    pub fn format_bytes(bytes: u64) -> String {
        const UNITS: [&str; 4] = ["B", "KB", "MB", "GB"];
        let mut size = bytes as f64;
        let mut unit_index = 0;

        while size >= 1024.0 && unit_index < UNITS.len() - 1 {
            size /= 1024.0;
            unit_index += 1;
        }

        format!("{:.1} {}", size, UNITS[unit_index])
    }

    /// Format uptime to human readable format
    pub fn format_uptime(seconds: u64) -> String {
        let days = seconds / 86400;
        let hours = (seconds % 86400) / 3600;
        let minutes = (seconds % 3600) / 60;

        if days > 0 {
            format!("{}d {}h {}m", days, hours, minutes)
        } else if hours > 0 {
            format!("{}h {}m", hours, minutes)
        } else {
            format!("{}m", minutes)
        }
    }

    /// Get system health status based on metrics
    pub fn get_health_status(metrics: &SystemMetrics) -> String {
        let cpu_usage = metrics.cpu_usage;
        let memory_usage = Self::get_memory_usage_percentage(metrics);
        let disk_usage = Self::get_disk_usage_percentage(metrics);

        // Define thresholds
        if cpu_usage > 90.0 || memory_usage > 90.0 || disk_usage > 90.0 {
            "Critical".to_string()
        } else if cpu_usage > 80.0 || memory_usage > 80.0 || disk_usage > 80.0 {
            "Warning".to_string()
        } else if cpu_usage > 70.0 || memory_usage > 70.0 || disk_usage > 70.0 {
            "Degraded".to_string()
        } else {
            "Healthy".to_string()
        }
    }

    /// Get project information from Cargo.toml
    pub fn get_project_info() -> (String, String) {
        // Try to read Cargo.toml from the project root
        let cargo_toml_path = "Cargo.toml";

        match fs::read_to_string(cargo_toml_path) {
            Ok(content) => {
                match toml::from_str::<toml::Value>(&content) {
                    Ok(toml_value) => {
                        if let Some(package) = toml_value.get("package") {
                            let name = package
                                .get("name")
                                .and_then(|v| v.as_str())
                                .unwrap_or("unknown")
                                .to_string();
                            let version = package
                                .get("version")
                                .and_then(|v| v.as_str())
                                .unwrap_or("unknown")
                                .to_string();
                            (name, version)
                        } else {
                            ("unknown".to_string(), "unknown".to_string())
                        }
                    }
                    Err(_) => {
                        // Fallback to hardcoded values if parsing fails
                        ("unknown".to_string(), "unknown".to_string())
                    }
                }
            }
            Err(_) => {
                // Fallback to hardcoded values if file can't be read
                ("unknown".to_string(), "unknown".to_string())
            }
        }
    }

    /// Get server information
    pub fn get_server_info() -> (String, u16, String, String) {
        let host = ServerConfigService::get_host();
        let port = ServerConfigService::get_port();
        let protocol = ServerConfigService::get_protocol();
        let environment = ServerConfigService::get_environment();

        (host, port, protocol, environment)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_format_bytes() {
        assert_eq!(SystemMonitorService::format_bytes(1024), "1.0 KB");
        assert_eq!(SystemMonitorService::format_bytes(1048576), "1.0 MB");
        assert_eq!(SystemMonitorService::format_bytes(1073741824), "1.0 GB");
        assert_eq!(SystemMonitorService::format_bytes(512), "512.0 B");
    }

    #[test]
    fn test_format_uptime() {
        assert_eq!(SystemMonitorService::format_uptime(3661), "1h 1m");
        assert_eq!(SystemMonitorService::format_uptime(86400), "1d 0h 0m");
        assert_eq!(SystemMonitorService::format_uptime(3600), "1h 0m");
        assert_eq!(SystemMonitorService::format_uptime(120), "2m");
    }

    #[test]
    fn test_memory_usage_percentage() {
        let metrics = SystemMetrics {
            cpu_usage: 0.0,
            memory_total: 1000,
            memory_used: 500,
            memory_available: 500,
            disk_total: 0,
            disk_used: 0,
            disk_available: 0,
            network_bytes_sent: 0,
            network_bytes_received: 0,
            uptime: 0,
            process_count: 0,
            database_connections: None,
            database_performance: None,
            system_name: None,
            kernel_version: None,
            os_version: None,
            host_name: None,
            cpu_count: 0,
            temperature: None,
        };

        assert_eq!(
            SystemMonitorService::get_memory_usage_percentage(&metrics),
            50.0
        );
    }

    #[test]
    fn test_health_status() {
        let healthy_metrics = SystemMetrics {
            cpu_usage: 30.0,
            memory_total: 1000,
            memory_used: 300,
            memory_available: 700,
            disk_total: 0,
            disk_used: 0,
            disk_available: 0,
            network_bytes_sent: 0,
            network_bytes_received: 0,
            uptime: 0,
            process_count: 0,
            database_connections: None,
            database_performance: None,
            system_name: None,
            kernel_version: None,
            os_version: None,
            host_name: None,
            cpu_count: 0,
            temperature: None,
        };

        assert_eq!(
            SystemMonitorService::get_health_status(&healthy_metrics),
            "Healthy"
        );
    }
}
