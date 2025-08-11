#[macro_export]
macro_rules! check_single_permission {
    ( $a:expr, $b:expr, $c:expr ) => {{
        // Find user by email using UserService
        let user = $crate::control::services::user_service::UserService::find_user_by_email($c, $a)
            .await?
            .ok_or($crate::infrastructure::app_error::AppError {
                message: "Invalid credentials".to_string(),
                status_code: axum::http::StatusCode::UNAUTHORIZED,
            })?;
        // check if a user has a specific permission
        let has_permission =
            $crate::control::services::permission_service::PermissionService::has_permission(
                $c, user.id, $b,
            )
            .await?;
        if !has_permission {
            return Err($crate::infrastructure::app_error::AppError {
                message: "Invalid Permissions".to_string(),
                status_code: axum::http::StatusCode::FORBIDDEN,
            });
        }
    }};
}
