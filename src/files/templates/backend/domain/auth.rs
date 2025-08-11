use chrono::{DateTime, Utc};
use uuid::Uuid;

/// Domain model for authentication token
#[derive(Debug)]
#[allow(dead_code)]
pub struct AuthToken {
    pub token: String,
    pub user_id: Uuid,
    pub expires_at: DateTime<Utc>,
}

impl AuthToken {
    pub fn new(token: String, user_id: Uuid, expires_at: DateTime<Utc>) -> Self {
        Self {
            token,
            user_id,
            expires_at,
        }
    }
}
