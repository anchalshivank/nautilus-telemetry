use crate::error::AppError;
use crate::state::AppState;
use axum::{
    extract::{Request, State},
    middleware::Next,
    response::Response,
};

pub async fn auth_middleware(
    State(state): State<AppState>,
    mut req: Request,
    next: Next,
) -> Result<Response, AppError> {
    let auth_header = req
        .headers()
        .get("x-api-key")
        .and_then(|h| h.to_str().ok())
        .ok_or_else(|| AppError::Unauthorized("Missing API key".to_string()))?;

    let vessel_id = state
        .services()
        .auth_service()
        .validate_api_key(auth_header)
        .await?;

    // Store vessel_id in request extensions for later use
    req.extensions_mut().insert(vessel_id);

    Ok(next.run(req).await)
}
