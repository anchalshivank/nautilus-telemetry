use crate::error::AppError;
use crate::models::metrics::{MetricData, MetricsSummary};
use crate::repositories::metrics::MetricsRepository;
use std::sync::Arc;
use std::time::Instant;

pub struct MetricsService {
    metrics_repo: Arc<MetricsRepository>,
    start_time: Instant,
}

impl MetricsService {
    pub fn new(metrics_repo: Arc<MetricsRepository>) -> Self {
        Self {
            metrics_repo,
            start_time: Instant::now(),
        }
    }

    pub async fn get_uptime_seconds(&self) -> u64 {
        self.start_time.elapsed().as_secs()
    }

    pub async fn get_request_count_last_minute(&self) -> Result<i64, AppError> {
        self.metrics_repo
            .get_request_count_last_n_minutes(1.0)
            .await
            .map_err(|e| AppError::Database(e))
    }

    pub async fn get_metrics(
        &self,
        vessel_id: Option<String>,
        hours: f64,
    ) -> Result<Vec<MetricData>, AppError> {
        let metrics = self.metrics_repo.get_metrics(vessel_id, hours).await?;

        Ok(metrics
            .into_iter()
            .map(|(metric_type, metric_value, timestamp)| MetricData {
                metric_type,
                metric_value: metric_value.to_string().parse().unwrap_or(0.0),
                timestamp,
            })
            .collect())
    }

    pub async fn get_metrics_summary(
        &self,
        vessel_id: Option<String>,
        hours: f64,
    ) -> Result<MetricsSummary, AppError> {
        let summary = self
            .metrics_repo
            .get_metrics_summary(vessel_id.clone(), hours)
            .await?;

        Ok(MetricsSummary {
            vessel_id,
            time_range: format!("Last {} hours", hours),
            request_volume: summary.0,
            avg_validation_latency_ms: summary.1.to_string().parse().unwrap_or(0.0),
            avg_ingestion_latency_ms: summary.2.to_string().parse().unwrap_or(0.0),
            avg_total_latency_ms: summary.3.to_string().parse().unwrap_or(0.0),
            p95_total_latency_ms: summary.4.map(|v| v.to_string().parse().unwrap_or(0.0)),
            p99_total_latency_ms: summary.5.map(|v| v.to_string().parse().unwrap_or(0.0)),
        })
    }

    pub async fn get_all_vessels_summary(
        &self,
        hours: f64,
    ) -> Result<Vec<MetricsSummary>, AppError> {
        let summaries = self.metrics_repo.get_all_vessels_summary(hours).await?;

        Ok(summaries
            .into_iter()
            .map(
                |(vessel_id, req_vol, avg_val, avg_ing, avg_tot, p95, p99)| MetricsSummary {
                    vessel_id: Some(vessel_id),
                    time_range: format!("Last {} hours", hours),
                    request_volume: req_vol,
                    avg_validation_latency_ms: avg_val.to_string().parse().unwrap_or(0.0),
                    avg_ingestion_latency_ms: avg_ing.to_string().parse().unwrap_or(0.0),
                    avg_total_latency_ms: avg_tot.to_string().parse().unwrap_or(0.0),
                    p95_total_latency_ms: p95.map(|v| v.to_string().parse().unwrap_or(0.0)),
                    p99_total_latency_ms: p99.map(|v| v.to_string().parse().unwrap_or(0.0)),
                },
            )
            .collect())
    }
}
