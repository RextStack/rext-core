use serde::{Deserialize, Serialize};

// JWT Claims
#[derive(Debug, Serialize, Deserialize)]
pub struct Claims {
    pub sub: String,        // subject (user id)
    pub exp: usize,         // expiration time
    pub session_id: String, // session UUID for tracking
}
