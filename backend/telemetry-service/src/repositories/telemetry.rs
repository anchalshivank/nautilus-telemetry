use crate::models::telemetry::{TelemetryFiltered, TelemetryRaw};
use chrono::{DateTime, Utc};
use rust_decimal::Decimal;
use sqlx::PgPool;
use uuid::Uuid;

pub struct TelemetryRepository {
    pool: PgPool,
}

impl TelemetryRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    pub async fn insert_raw_batch(
        &self,
        records: &Vec<(String, DateTime<Utc>, i64, String, Decimal, Uuid, String)>,
    ) -> Result<(), sqlx::Error> {
        let mut tx = self.pool.begin().await?;

        for (
            vessel_id,
            timestamp_utc,
            epoch_utc,
            signal_name,
            signal_value,
            correlation_id,
            trace_id,
        ) in records
        {
            sqlx::query!(
                r#"
                INSERT INTO telemetry_raw (vessel_id, timestamp_utc, epoch_utc, signal_name, signal_value, correlation_id, trace_id)
                VALUES ($1, $2, $3, $4, $5, $6, $7)
                "#,
                vessel_id,
                timestamp_utc,
                epoch_utc,
                signal_name,
                signal_value,
                correlation_id,
                trace_id
            )
                .execute(&mut *tx)
                .await?;
        }

        tx.commit().await?;
        Ok(())
    }

    pub async fn insert_filtered_batch(
        &self,
        records: &Vec<(
            String,
            DateTime<Utc>,
            i64,
            String,
            Decimal,
            String,
            Uuid,
            String,
        )>,
    ) -> Result<(), sqlx::Error> {
        let mut tx = self.pool.begin().await?;

        for (
            vessel_id,
            timestamp_utc,
            epoch_utc,
            signal_name,
            signal_value,
            reason,
            correlation_id,
            trace_id,
        ) in records
        {
            sqlx::query!(
                r#"
                INSERT INTO telemetry_filtered (vessel_id, timestamp_utc, epoch_utc, signal_name, signal_value, reason, correlation_id, trace_id)
                VALUES ($1, $2, $3, $4, $5, $6, $7, $8)
                "#,
                vessel_id,
                timestamp_utc,
                epoch_utc,
                signal_name,
                signal_value,
                reason,
                correlation_id,
                trace_id
            )
                .execute(&mut *tx)
                .await?;
        }

        tx.commit().await?;
        Ok(())
    }
}
