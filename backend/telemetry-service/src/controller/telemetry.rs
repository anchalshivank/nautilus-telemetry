use crate::error::AppError;
use crate::models::telemetry::{TelemetryRequest, TelemetryResponse};
use crate::state::AppState;
use axum::{Extension, Json, extract::State};
use tracing::info;

pub async fn ingest_telemetry(
    State(state): State<AppState>,
    Extension(authenticated_vessel_id): Extension<String>,
    Json(payload): Json<TelemetryRequest>,
) -> Result<Json<TelemetryResponse>, AppError> {
    info!("Received telemetry for vessel: {}", payload.vessel_id);
    if payload.vessel_id != authenticated_vessel_id {
        return Err(AppError::Forbidden(format!(
            "Vessel ID mismatch: authenticated as '{}' but payload contains '{}'",
            authenticated_vessel_id, payload.vessel_id
        )));
    }
    let response = state
        .services()
        .telemetry_service()
        .ingest_telemetry(payload)
        .await?;

    Ok(Json(response))
}
