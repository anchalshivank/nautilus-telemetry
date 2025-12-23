use crate::controller::api_key::{create_api_key, list_api_keys, revoke_api_key};
use crate::controller::metrics::{
    get_all_vessels_metrics, get_metrics, get_metrics_summary, health_with_metrics,
};
use crate::controller::telemetry::ingest_telemetry;
use crate::controller::vessel::{create_vessel, deactivate_vessel, get_vessel, list_vessels};
use crate::middleware::admin_middleware;
use crate::middleware::auth::auth_middleware;
use crate::state::AppState;
use axum::routing::{delete, get, post};
use axum::{Json, Router, middleware};
use serde_json::{Value, json};
use tracing::{info, instrument};

pub fn api_routes(state: AppState) -> Router<AppState> {
    // Public routes (no auth)
    let public_routes = Router::new().route("/health", get(health_with_metrics));

    // Telemetry ingestion (requires API key)
    let telemetry_routes = Router::new()
        .route("/telemetry", post(ingest_telemetry))
        .layer(middleware::from_fn_with_state(
            state.clone(),
            auth_middleware,
        ));

    // Admin routes (requires admin key)
    let admin_routes = Router::new()
        // Vessel management
        .route("/vessels", post(create_vessel))
        .route("/vessels", get(list_vessels))
        .route("/vessels/{vessel_id}", get(get_vessel))
        .route("/vessels/{vessel_id}", delete(deactivate_vessel))
        // API key management
        .route("/api-keys", post(create_api_key))
        .route("/api-keys/vessel/{vessel_id}", get(list_api_keys))
        .route("/api-keys/revoke/{api_key}", delete(revoke_api_key))
        // Metrics APIs
        .route("/metrics", get(get_metrics))
        .route("/metrics/summary", get(get_metrics_summary))
        .route("/metrics/vessels", get(get_all_vessels_metrics))
        .layer(middleware::from_fn(admin_middleware));

    Router::new()
        .merge(public_routes)
        .merge(telemetry_routes)
        .merge(admin_routes)
}

#[instrument]
pub async fn root() -> Json<Value> {
    info!("Root endpoint called");
    Json(json!({
        "service": "telemetry-service",
        "status": "running",
        "version": "0.1.0",
        "endpoints": {
            "health": "/api/v1/health",
            "telemetry": "/api/v1/telemetry (requires x-api-key)",
            "admin": {
                "vessels": "/api/v1/vessels (requires x-admin-key)",
                "api_keys": "/api/v1/api-keys (requires x-admin-key)",
                "metrics": "/api/v1/metrics (requires x-admin-key)"
            }
        }
    }))
}
