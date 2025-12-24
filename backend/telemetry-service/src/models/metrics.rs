use chrono::{DateTime, Utc};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use serde_json::Value as JsonValue;
use uuid::Uuid;

#[derive(Debug)]
pub struct ServerMetric {
    pub id: i64,
    pub vessel_id: Option<String>,
    pub metric_type: String,
    pub metric_value: Decimal,
    pub timestamp: DateTime<Utc>,
    pub additional_metadata: Option<JsonValue>,
    pub correlation_id: Uuid,
    pub trace_id: Option<String>,
}

#[derive(Debug)]
pub struct MetricType {
    pub request_volume: String,
    pub latency_validation: String,
    pub latency_ingestion: String,
    pub latency_total: String,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct MetricsResponse {
    pub vessel_id: Option<String>,
    pub metrics: Vec<MetricData>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct MetricData {
    pub metric_type: String,
    pub metric_value: f64,
    pub timestamp: DateTime<Utc>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct MetricsSummary {
    pub vessel_id: Option<String>,
    pub time_range: String,
    pub request_volume: i64,
    pub avg_validation_latency_ms: f64,
    pub avg_ingestion_latency_ms: f64,
    pub avg_total_latency_ms: f64,
    pub p95_total_latency_ms: Option<f64>,
    pub p99_total_latency_ms: Option<f64>,
}

#[derive(Debug, Deserialize)]
pub struct MetricsQuery {
    pub vessel_id: Option<String>,
    pub hours: Option<f64>,
}
