use chrono::{DateTime, Utc};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use uuid::Uuid;

#[derive(Debug, Serialize, Deserialize)]
pub struct TelemetryRequest {
    #[serde(rename = "vesselId")]
    pub vessel_id: String,
    #[serde(rename = "timestampUTC")]
    pub timestamp_utc: DateTime<Utc>,
    #[serde(rename = "epochUTC")]
    pub epoch_utc: String,
    pub signals: HashMap<String, serde_json::Value>,
}

#[derive(Debug, Serialize)]
pub struct TelemetryResponse {
    pub message: String,
    pub correlation_id: Uuid,
    pub valid_signals: usize,
    pub invalid_signals: usize,
}

#[derive(Debug)]
pub struct TelemetryRaw {
    pub id: i64,
    pub vessel_id: String,
    pub timestamp_utc: DateTime<Utc>,
    pub epoch_utc: i64,
    pub signal_name: String,
    pub signal_value: Decimal,
    pub ingested_at: DateTime<Utc>,
    pub correlation_id: Uuid,
    pub trace_id: Option<String>,
}

#[derive(Debug)]
pub struct TelemetryFiltered {
    pub id: i64,
    pub vessel_id: String,
    pub timestamp_utc: DateTime<Utc>,
    pub epoch_utc: i64,
    pub signal_name: String,
    pub signal_value: Decimal,
    pub reason: String,
    pub ingested_at: DateTime<Utc>,
    pub correlation_id: Uuid,
    pub trace_id: Option<String>,
}
