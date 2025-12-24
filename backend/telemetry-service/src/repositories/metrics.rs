use chrono::{DateTime, Utc};
use rust_decimal::Decimal;
use sqlx::PgPool;
use uuid::Uuid;

pub struct MetricsRepository {
    pool: PgPool,
}

impl MetricsRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    pub async fn insert_metric(
        &self,
        vessel_id: Option<String>,
        metric_type: String,
        metric_value: Decimal,
        correlation_id: Uuid,
        trace_id: String,
    ) -> Result<(), sqlx::Error> {
        sqlx::query!(
            r#"
            INSERT INTO server_metrics (vessel_id, metric_type, metric_value, correlation_id, trace_id)
            VALUES ($1, $2, $3, $4, $5)
            "#,
            vessel_id,
            metric_type,
            metric_value,
            correlation_id,
            trace_id
        )
            .execute(&self.pool)
            .await?;

        Ok(())
    }

    pub async fn get_request_count_last_n_minutes(&self, minutes: f64) -> Result<i64, sqlx::Error> {
        let result = sqlx::query!(
            r#"
            SELECT COUNT(*) as count
            FROM server_metrics
            WHERE metric_type = 'request_volume'
              AND timestamp > NOW() - INTERVAL '1 minute' * $1
            "#,
            minutes
        )
        .fetch_one(&self.pool)
        .await?;

        Ok(result.count.unwrap_or(0))
    }

    pub async fn get_metrics(
        &self,
        vessel_id: Option<String>,
        hours: f64,
    ) -> Result<Vec<(String, Decimal, DateTime<Utc>)>, sqlx::Error> {
        let metrics = match vessel_id {
            Some(vid) => sqlx::query!(
                r#"
                    SELECT metric_type, metric_value, timestamp
                    FROM server_metrics
                    WHERE vessel_id = $1
                      AND timestamp > NOW() - INTERVAL '1 hour' * $2
                    ORDER BY timestamp DESC
                    "#,
                vid,
                hours
            )
            .fetch_all(&self.pool)
            .await?
            .into_iter()
            .map(|r| (r.metric_type, r.metric_value, r.timestamp))
            .collect(),
            None => sqlx::query!(
                r#"
                    SELECT metric_type, metric_value, timestamp
                    FROM server_metrics
                    WHERE timestamp > NOW() - INTERVAL '1 hour' * $1
                    ORDER BY timestamp DESC
                    "#,
                hours
            )
            .fetch_all(&self.pool)
            .await?
            .into_iter()
            .map(|r| (r.metric_type, r.metric_value, r.timestamp))
            .collect(),
        };

        Ok(metrics)
    }

    pub async fn get_metrics_summary(
        &self,
        vessel_id: Option<String>,
        hours: f64,
    ) -> Result<(i64, Decimal, Decimal, Decimal, Option<f64>, Option<f64>), sqlx::Error> {
        let summary = sqlx::query!(
        r#"
        SELECT
            COUNT(CASE WHEN metric_type = 'request_volume' THEN 1 END) as request_volume,
            AVG(CASE WHEN metric_type = 'latency_validation' THEN metric_value END) as avg_validation,
            AVG(CASE WHEN metric_type = 'latency_ingestion' THEN metric_value END) as avg_ingestion,
            AVG(CASE WHEN metric_type = 'latency_total' THEN metric_value END) as avg_total,
            PERCENTILE_CONT(0.95) WITHIN GROUP (ORDER BY metric_value)
                FILTER (WHERE metric_type = 'latency_total') as p95_total,
            PERCENTILE_CONT(0.99) WITHIN GROUP (ORDER BY metric_value)
                FILTER (WHERE metric_type = 'latency_total') as p99_total
        FROM server_metrics
        WHERE ($1::text IS NULL OR vessel_id = $1)
          AND timestamp > NOW() - INTERVAL '1 hour' * $2
        "#,
        vessel_id,
        hours
    )
            .fetch_one(&self.pool)
            .await?;

        Ok((
            summary.request_volume.unwrap_or(0),
            summary.avg_validation.unwrap_or(Decimal::ZERO),
            summary.avg_ingestion.unwrap_or(Decimal::ZERO),
            summary.avg_total.unwrap_or(Decimal::ZERO),
            summary.p95_total,
            summary.p99_total,
        ))
    }

    pub async fn get_all_vessels_summary(
        &self,
        hours: f64,
    ) -> Result<
        Vec<(
            String,
            i64,
            Decimal,
            Decimal,
            Decimal,
            Option<f64>,
            Option<f64>,
        )>,
        sqlx::Error,
    > {
        let summaries = sqlx::query!(
            r#"
            SELECT
                vessel_id,
                COUNT(CASE WHEN metric_type = 'request_volume' THEN 1 END) as request_volume,
                AVG(CASE WHEN metric_type = 'latency_validation' THEN metric_value END) as avg_validation,
                AVG(CASE WHEN metric_type = 'latency_ingestion' THEN metric_value END) as avg_ingestion,
                AVG(CASE WHEN metric_type = 'latency_total' THEN metric_value END) as avg_total,
                PERCENTILE_CONT(0.95) WITHIN GROUP (ORDER BY metric_value)
                    FILTER (WHERE metric_type = 'latency_total') as p95_total,
                PERCENTILE_CONT(0.99) WITHIN GROUP (ORDER BY metric_value)
                    FILTER (WHERE metric_type = 'latency_total') as p99_total
            FROM server_metrics
            WHERE timestamp > NOW() - INTERVAL '1 hour' * $1
              AND vessel_id IS NOT NULL
            GROUP BY vessel_id
            ORDER BY request_volume DESC
            "#,
            hours
        )
            .fetch_all(&self.pool)
            .await?;

        Ok(summaries
            .into_iter()
            .map(|r| {
                (
                    r.vessel_id.unwrap_or_default(),
                    r.request_volume.unwrap_or(0),
                    r.avg_validation.unwrap_or(Decimal::ZERO),
                    r.avg_ingestion.unwrap_or(Decimal::ZERO),
                    r.avg_total.unwrap_or(Decimal::ZERO),
                    r.p95_total,
                    r.p99_total,
                )
            })
            .collect())
    }
}
