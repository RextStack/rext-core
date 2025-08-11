/// Logging information for audit logs and for downstream handlers
#[derive(Clone)]
#[allow(dead_code)]
pub struct LoggingInfo {
    pub method: String,
    pub path: String,
    pub user_id: String,
    pub ip_address: Option<String>,
    pub user_agent: Option<String>,
}
