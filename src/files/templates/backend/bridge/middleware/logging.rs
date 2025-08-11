use axum::{
    body::Body,
    extract::{Request, State},
    http::StatusCode,
    middleware::Next,
    response::Response,
};
use sea_orm::{ActiveModelTrait, DatabaseConnection, Set};
use serde_json::Value;
use std::time::Instant;
use tracing::{error, info};

use crate::{
    bridge::types::{auth::AuthUser, logging::LoggingInfo},
    entity::models::audit_logs,
    infrastructure::{logging::LoggingManager, websocket::broadcast_audit_log},
};

const MAX_BODY_LOG_BYTES: usize = 4096; // 4KB

/// Sensitive fields that should be redacted from logs
const SENSITIVE_FIELDS: &[&str] = &[
    "password",
    "passwd",
    "pwd",
    "secret",
    "token",
    "key",
    "auth",
    "authorization",
    "jwt",
    "api_key",
    "api_secret",
    "private_key",
    "private_secret",
];

/// Sanitize JSON content by redacting sensitive fields
fn sanitize_json_content(content: &str) -> String {
    if let Ok(mut json) = serde_json::from_str::<Value>(content) {
        if let Some(obj) = json.as_object_mut() {
            for field in SENSITIVE_FIELDS {
                if obj.contains_key(*field) {
                    obj.insert(field.to_string(), Value::String("[REDACTED]".to_string()));
                }
            }
        }
        json.to_string()
    } else {
        // If not valid JSON, check for common patterns and redact
        let mut sanitized = content.to_string();
        for field in SENSITIVE_FIELDS {
            // Simple pattern matching for common formats
            let patterns = [
                format!("\"{}\":", field),
                format!("{}:", field),
                format!("{} =", field),
            ];
            for pattern in patterns {
                if sanitized.contains(&pattern) {
                    // This is a simplified redaction - in production you might want more sophisticated parsing
                    sanitized = sanitized
                        .replace(&format!("{}", pattern), &format!("{}[REDACTED]", pattern));
                }
            }
        }
        sanitized
    }
}

/// Extracts, copies, and sanitizes the request and response bodies so we can log them
/// without interfering with the original request and response.
///
/// Returns a tuple of the response, the sanitized request body, and the sanitized response body.
///
/// The request and response bodies are not consumed, so we can reconstruct the original request and response.
///
/// The request and response bodies are sanitized by redacting sensitive fields.
///
/// The request and response bodies are truncated to MAX_BODY_LOG_BYTES.
pub async fn extract_request_response(
    req: Request<Body>,
    next: Next,
) -> Result<(Response, Option<String>, Option<String>), (StatusCode, String)> {
    // extract parts of the request so we can reconstruct it later
    let (req_parts, req_body) = req.into_parts();

    // read the entire request body
    let req_bytes = match axum::body::to_bytes(req_body, usize::MAX).await {
        Ok(bytes) => bytes,
        Err(err) => {
            return Err((
                StatusCode::BAD_REQUEST,
                format!("failed to read request body: {}", err),
            ));
        }
    };

    // clonse the request body and truncate the clone to MAX_BODY_LOG_BYTES
    let copy_req_bytes = req_bytes.clone();
    let copy_req_bytes = if copy_req_bytes.len() > MAX_BODY_LOG_BYTES {
        &copy_req_bytes[..MAX_BODY_LOG_BYTES]
    } else {
        &copy_req_bytes
    };

    // stringify the cloned request body and sanitize it
    let copy_req_string = String::from_utf8_lossy(copy_req_bytes).to_string();
    let copy_req_sanitized = Some(sanitize_json_content(&copy_req_string));

    // reconstruct the request with the original parts and original body
    let req = Request::from_parts(req_parts, Body::from(req_bytes));

    // send the request to the next middleware
    let response = next.run(req).await;

    // do the same thing for the response
    let (res_parts, res_body) = response.into_parts();
    let res_bytes = match axum::body::to_bytes(res_body, usize::MAX).await {
        Ok(bytes) => bytes,
        Err(err) => {
            return Err((
                StatusCode::BAD_REQUEST,
                format!("failed to read response body: {}", err),
            ));
        }
    };

    // clone the response body and truncate the clone to MAX_BODY_LOG_BYTES
    let copy_res_bytes = res_bytes.clone();
    let copy_res_bytes = if copy_res_bytes.len() > MAX_BODY_LOG_BYTES {
        &copy_res_bytes[..MAX_BODY_LOG_BYTES]
    } else {
        &copy_res_bytes
    };

    // stringify the cloned response body and sanitize it
    let copy_res_string = String::from_utf8_lossy(copy_res_bytes).to_string();
    let copy_res_sanitized = Some(sanitize_json_content(&copy_res_string));

    // reconstruct the response with the original parts and original body
    let res = Response::from_parts(res_parts, Body::from(res_bytes));

    Ok((res, copy_req_sanitized, copy_res_sanitized))
}

/// Request logging middleware for auditing all API requests
pub async fn request_logging_middleware(
    State(db): State<DatabaseConnection>,
    mut request: Request,
    next: Next,
) -> Result<Response, StatusCode> {
    let start = Instant::now();
    let request_id = LoggingManager::generate_request_id();

    // Extract request info
    let method = request.method().to_string();
    let path = request.uri().path().to_string();

    // if path is /api-docs/openapi.json, don't log
    if path == "/api-docs/openapi.json" {
        return Ok(next.run(request).await);
    }

    // Don't log the logs endpoint to prevent recursive logging
    if path == "/api/v1/admin/logs" {
        return Ok(next.run(request).await);
    }

    // Don't log database inspection endpoints as they can return large amounts of data
    if path.starts_with("/api/v1/admin/database") {
        return Ok(next.run(request).await);
    }

    // Don't log users endpoint as it can return large amounts of user data
    if path.starts_with("/api/v1/admin/users") {
        return Ok(next.run(request).await);
    }

    // Don't log WebSocket endpoint to prevent recursive logging
    if path == "/api/v1/admin/ws" {
        return Ok(next.run(request).await);
    }

    let ip_address = request
        .headers()
        .get("x-forwarded-for")
        .and_then(|v| v.to_str().ok())
        .map(|s| s.to_string())
        .or_else(|| {
            request
                .extensions()
                .get::<std::net::SocketAddr>()
                .map(|addr| addr.ip().to_string())
        });
    let user_agent = request
        .headers()
        .get("user-agent")
        .and_then(|v| v.to_str().ok())
        .map(|s| s.to_string());

    // Try to get user_id from extensions (set by auth middleware)
    let user_id = request.extensions().get::<AuthUser>().map(|u| u.user_id);

    // Clone values for logging info
    let method_for_logging_info = method.clone();
    let path_for_logging_info = path.clone();
    let ip_address_for_logging_info = ip_address.clone();
    let user_agent_for_logging_info = user_agent.clone();

    // Insert into logging info for downstream handlers
    let logging_info = LoggingInfo {
        method: method_for_logging_info,
        path: path_for_logging_info,
        user_id: user_id.unwrap_or_default().to_string(),
        ip_address: ip_address_for_logging_info,
        user_agent: user_agent_for_logging_info,
    };

    // Insert into request extensions for downstream handlers, must be done
    // before we extract the request and response bodies, otherwise the
    // request will have finished already.
    request.extensions_mut().insert(logging_info);

    // Capture request and response bodies (runs the next handler so we get the response)
    let (response, request_body, response_body) = extract_request_response(request, next).await.map_err(|(status, message)| {
        error!(request_id = %request_id, error = %message, "Failed to extract request and response bodies");
        status
    })?;

    let duration = start.elapsed();
    let response_time_ms = duration.as_millis() as i32;
    let status_code = response.status().as_u16() as i32;

    // Error message if status is error
    let error_message = if status_code >= 400 {
        Some(format!("Error status: {}", status_code))
    } else {
        None
    };

    // Clone values needed after move
    let method_clone = method.clone();
    let path_clone = path.clone();
    let ip_address_clone = ip_address.clone();
    let user_agent_clone = user_agent.clone();
    let request_id_clone = request_id.clone();
    let error_message_clone = error_message.clone();
    let user_id_clone = user_id.clone();

    // Create audit log entry
    let audit_log_id = uuid::Uuid::new_v4().to_string();
    let timestamp = chrono::Utc::now().to_rfc3339();

    // Clone values for WebSocket broadcast
    let method_for_ws = method_clone.clone();
    let path_for_ws = path_clone.clone();
    let ip_address_for_ws = ip_address_clone.clone();
    let user_agent_for_ws = user_agent_clone.clone();
    let error_message_for_ws = error_message_clone.clone();
    let user_id_for_ws = user_id_clone.clone();

    // Clone values for system log broadcasting
    let method_for_logs = method_clone.clone();
    let path_for_logs = path_clone.clone();

    // Insert audit log asynchronously (don't block response)
    let audit_log = audit_logs::ActiveModel {
        id: Set(uuid::Uuid::new_v4()),
        timestamp: Set(Some(chrono::Utc::now().into())),
        method: Set(method),
        path: Set(path),
        status_code: Set(Some(status_code)),
        response_time_ms: Set(Some(response_time_ms)),
        user_id: Set(user_id),
        ip_address: Set(ip_address),
        user_agent: Set(user_agent),
        request_body: Set(request_body),
        response_body: Set(response_body),
        error_message: Set(error_message_clone.clone()),
    };
    let db_clone = db.clone();
    tokio::spawn(async move {
        if let Err(e) = audit_log.insert(&db_clone).await {
            error!(request_id = %request_id_clone, error = ?e, "Failed to insert audit log");

            // Broadcast error log
            crate::infrastructure::websocket::broadcast_system_log(
                "error".to_string(),
                format!("Failed to insert audit log: {}", e),
                "audit_logging".to_string(),
            )
            .await;
        } else {
            info!(request_id = %request_id_clone, "Audit log inserted");

            // Broadcast the audit log to WebSocket clients
            broadcast_audit_log(
                audit_log_id,
                timestamp,
                method_for_ws,
                path_for_ws,
                Some(status_code),
                Some(response_time_ms),
                user_id_for_ws.map(|id| id.to_string()),
                ip_address_for_ws,
                user_agent_for_ws,
                error_message_for_ws,
            )
            .await;

            // Broadcast info log for successful requests (but not too frequently)
            if status_code >= 200 && status_code < 300 {
                crate::infrastructure::websocket::broadcast_system_log(
                    "info".to_string(),
                    format!(
                        "Request completed: {} {} ({}ms)",
                        method_for_logs, path_for_logs, response_time_ms
                    ),
                    "request_logging".to_string(),
                )
                .await;
            } else if status_code >= 400 {
                // Broadcast warning for client errors
                crate::infrastructure::websocket::broadcast_system_log(
                    "warn".to_string(),
                    format!(
                        "Client error: {} {} - {}",
                        method_for_logs, path_for_logs, status_code
                    ),
                    "request_logging".to_string(),
                )
                .await;
            } else if status_code >= 500 {
                // Broadcast error for server errors
                crate::infrastructure::websocket::broadcast_system_log(
                    "error".to_string(),
                    format!(
                        "Server error: {} {} - {}",
                        method_for_logs, path_for_logs, status_code
                    ),
                    "request_logging".to_string(),
                )
                .await;
            }
        }
    });

    // log to tracing with admin label if the path starts with /api/v1/admin
    let is_admin_request = path_clone.starts_with("/api/v1/admin");
    if let Some(ref err) = error_message_clone {
        error!(
            request_id = %request_id,
            status_code,
            user_id = ?user_id_clone,
            path = %path_clone,
            method = %method_clone,
            ip_address = ?ip_address_clone,
            user_agent = ?user_agent_clone,
            response_time_ms,
            error = %err,
            admin_request = %is_admin_request,
            "Request error"
        );
    } else {
        info!(
            request_id = %request_id,
            status_code,
            user_id = ?user_id_clone,
            path = %path_clone,
            method = %method_clone,
            ip_address = ?ip_address_clone,
            user_agent = ?user_agent_clone,
            response_time_ms,
            admin_request = %is_admin_request,
            "Request completed"
        );
    }

    Ok(response)
}
