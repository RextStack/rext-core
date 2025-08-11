use crate::infrastructure::app_error::AppError;
use axum::http::StatusCode;

/// Validates email format
pub fn validate_email(email: &str) -> Result<(), AppError> {
    if email.is_empty() {
        return Err(AppError {
            message: "Email is required".to_string(),
            status_code: StatusCode::BAD_REQUEST,
        });
    }

    // Basic email validation - in production, consider using a proper email validation crate
    if !email.contains('@') || !email.contains('.') {
        return Err(AppError {
            message: "Invalid email format".to_string(),
            status_code: StatusCode::BAD_REQUEST,
        });
    }

    Ok(())
}

/// Validates password strength
pub fn validate_password(password: &str) -> Result<(), AppError> {
    if password.is_empty() {
        return Err(AppError {
            message: "Password is required".to_string(),
            status_code: StatusCode::BAD_REQUEST,
        });
    }

    if password.len() < 6 {
        return Err(AppError {
            message: "Password must be at least 6 characters".to_string(),
            status_code: StatusCode::BAD_REQUEST,
        });
    }

    Ok(())
}

/// Validates registration input
pub fn validate_registration_input(email: &str, password: &str) -> Result<(), AppError> {
    validate_email(email)?;
    validate_password(password)?;
    Ok(())
}

/// Validates login input
pub fn validate_login_input(email: &str, password: &str) -> Result<(), AppError> {
    validate_email(email)?;
    validate_password(password)?;
    Ok(())
}
