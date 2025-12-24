use crate::error::AppError;
use crate::models::metrics::{MetricsQuery, MetricsResponse, MetricsSummary};
use crate::state::AppState;
use axum::Json;
use axum::extract::State;
use chrono::Utc;
use tracing::info;

// Get raw metrics
pub async fn get_metrics(
    State(state): State<AppState>,
    axum::extract::Query(query): axum::extract::Query<MetricsQuery>,
) -> Result<Json<MetricsResponse>, AppError> {
    info!("Fetching metrics for vessel: {:?}", query.vessel_id);

    let hours = query.hours.unwrap_or(24.0);
    let metrics = state
        .services()
        .metrics_service()
        .get_metrics(query.vessel_id.clone(), hours)
        .await?;

    Ok(Json(MetricsResponse {
        vessel_id: query.vessel_id,
        metrics,
    }))
}

// Get metrics summary with aggregations
pub async fn get_metrics_summary(
    State(state): State<AppState>,
    axum::extract::Query(query): axum::extract::Query<MetricsQuery>,
) -> Result<Json<MetricsSummary>, AppError> {
    info!("Fetching metrics summary for vessel: {:?}", query.vessel_id);

    let hours = query.hours.unwrap_or(24.0);
    let summary = state
        .services()
        .metrics_service()
        .get_metrics_summary(query.vessel_id.clone(), hours)
        .await?;

    Ok(Json(summary))
}

// Get metrics for all vessels
pub async fn get_all_vessels_metrics(
    State(state): State<AppState>,
    axum::extract::Query(query): axum::extract::Query<MetricsQuery>,
) -> Result<Json<Vec<MetricsSummary>>, AppError> {
    info!("Fetching metrics for all vessels");

    let hours = query.hours.unwrap_or(24.0);
    let summaries = state
        .services()
        .metrics_service()
        .get_all_vessels_summary(hours)
        .await?;

    Ok(Json(summaries))
}

// Health check that includes basic metrics
pub async fn health_with_metrics(
    State(state): State<AppState>,
) -> Result<Json<serde_json::Value>, AppError> {
    info!("Health check with metrics");

    let uptime = state
        .services()
        .metrics_service()
        .get_uptime_seconds()
        .await;

    let recent_requests = state
        .services()
        .metrics_service()
        .get_request_count_last_minute()
        .await?;

    Ok(Json(serde_json::json!({
        "status": "healthy",
        "uptime_seconds": uptime,
        "requests_last_minute": recent_requests,
        "timestamp": Utc::now()
    })))
}
