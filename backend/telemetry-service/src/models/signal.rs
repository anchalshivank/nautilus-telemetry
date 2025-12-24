use chrono::{DateTime, Utc};
use rust_decimal::Decimal;
use uuid::Uuid;

#[derive(Debug, Clone)]
pub struct Signal {
    pub signal_id: i32,
    pub signal_name: String,
    pub signal_type: String,
    pub min_value: Option<Decimal>,
    pub max_value: Option<Decimal>,
    pub description: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub correlation_id: Option<Uuid>,
    pub trace_id: Option<String>,
}
