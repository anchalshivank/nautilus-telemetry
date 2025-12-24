use crate::error::AppError;
use crate::models::vessel::{CreateVesselRequest, VesselResponse};
use crate::state::AppState;
use axum::{Json, extract::State};
use tracing::info;

pub async fn create_vessel(
    State(state): State<AppState>,
    Json(payload): Json<CreateVesselRequest>,
) -> Result<Json<VesselResponse>, AppError> {
    info!("Creating vessel: {}", payload.vessel_id);

    let vessel = state
        .services()
        .vessel_service()
        .create_vessel(payload)
        .await?;

    Ok(Json(vessel))
}

pub async fn get_vessel(
    State(state): State<AppState>,
    axum::extract::Path(vessel_id): axum::extract::Path<String>,
) -> Result<Json<VesselResponse>, AppError> {
    info!("Getting vessel: {}", vessel_id);

    let vessel = state
        .services()
        .vessel_service()
        .get_vessel(&vessel_id)
        .await?;

    Ok(Json(vessel))
}

pub async fn list_vessels(
    State(state): State<AppState>,
) -> Result<Json<Vec<VesselResponse>>, AppError> {
    info!("Listing all vessels");

    let vessels = state.services().vessel_service().list_vessels().await?;

    Ok(Json(vessels))
}

pub async fn deactivate_vessel(
    State(state): State<AppState>,
    axum::extract::Path(vessel_id): axum::extract::Path<String>,
) -> Result<Json<serde_json::Value>, AppError> {
    info!("Deactivating vessel: {}", vessel_id);

    state
        .services()
        .vessel_service()
        .deactivate_vessel(&vessel_id)
        .await?;

    Ok(Json(serde_json::json!({
        "message": "Vessel deactivated successfully"
    })))
}
