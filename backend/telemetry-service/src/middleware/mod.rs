pub mod auth;

use crate::error::AppError;
use axum::{extract::Request, middleware::Next, response::Response};

pub async fn admin_middleware(req: Request, next: Next) -> Result<Response, AppError> {
    let admin_key =
        std::env::var("ADMIN_API_KEY").unwrap_or_else(|_| "admin_secret_key_change_me".to_string());

    let auth_header = req
        .headers()
        .get("x-admin-key")
        .and_then(|h| h.to_str().ok())
        .ok_or_else(|| AppError::Unauthorized("Missing admin key".to_string()))?;

    if auth_header != admin_key {
        return Err(AppError::Unauthorized("Invalid admin key".to_string()));
    }

    Ok(next.run(req).await)
}
