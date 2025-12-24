use crate::models::signal::Signal;
use sqlx::PgPool;
use std::collections::HashMap;

pub struct SignalRepository {
    pool: PgPool,
}

impl SignalRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    pub async fn find_all(&self) -> Result<HashMap<String, Signal>, sqlx::Error> {
        let signals = sqlx::query_as!(
            Signal,
            r#"
            SELECT signal_id, signal_name, signal_type, min_value, max_value, description, created_at, updated_at, correlation_id, trace_id
            FROM signal_register_table
            "#
        )
            .fetch_all(&self.pool)
            .await?;

        let map = signals
            .into_iter()
            .map(|s| (s.signal_name.clone(), s))
            .collect();

        Ok(map)
    }
}
