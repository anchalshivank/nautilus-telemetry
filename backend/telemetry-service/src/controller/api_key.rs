use crate::error::AppError;
use crate::models::api_key::{ApiKeyResponse, CreateApiKeyRequest};
use crate::state::AppState;
use axum::{Json, extract::State};
use tracing::info;

pub async fn create_api_key(
    State(state): State<AppState>,
    Json(payload): Json<CreateApiKeyRequest>,
) -> Result<Json<ApiKeyResponse>, AppError> {
    info!("Creating API key for vessel: {}", payload.vessel_id);

    let api_key = state
        .services()
        .auth_service()
        .create_api_key(&payload.vessel_id, payload.expires_at)
        .await?;

    Ok(Json(api_key))
}

pub async fn list_api_keys(
    State(state): State<AppState>,
    axum::extract::Path(vessel_id): axum::extract::Path<String>,
) -> Result<Json<Vec<ApiKeyResponse>>, AppError> {
    info!("Listing API keys for vessel: {}", vessel_id);

    let keys = state
        .services()
        .auth_service()
        .list_api_keys(&vessel_id)
        .await?;

    Ok(Json(keys))
}

pub async fn revoke_api_key(
    State(state): State<AppState>,
    axum::extract::Path(api_key): axum::extract::Path<String>,
) -> Result<Json<serde_json::Value>, AppError> {
    info!("Revoking API key: {}", api_key);

    state
        .services()
        .auth_service()
        .revoke_api_key(&api_key)
        .await?;

    Ok(Json(serde_json::json!({
        "message": "API key revoked successfully"
    })))
}
