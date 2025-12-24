use crate::models::vessel::Vessel;
use sqlx::PgPool;
use uuid::Uuid;

pub struct VesselRepository {
    pool: PgPool,
}

impl VesselRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    pub async fn find_by_id(&self, vessel_id: &str) -> Result<Option<Vessel>, sqlx::Error> {
        sqlx::query_as!(
            Vessel,
            r#"
            SELECT vessel_id, vessel_name, is_active as "is_active!", created_at as "created_at!", updated_at as "updated_at!", correlation_id, trace_id
            FROM vessel_register_table
            WHERE vessel_id = $1 AND is_active = TRUE
            "#,
            vessel_id
        )
            .fetch_optional(&self.pool)
            .await
    }

    pub async fn create(
        &self,
        vessel_id: &str,
        vessel_name: &str,
        correlation_id: Uuid,
        trace_id: Option<String>,
    ) -> Result<Vessel, sqlx::Error> {
        sqlx::query_as!(
            Vessel,
            r#"
            INSERT INTO vessel_register_table (vessel_id, vessel_name, correlation_id, trace_id)
            VALUES ($1, $2, $3, $4)
            RETURNING vessel_id, vessel_name, is_active as "is_active!", created_at as "created_at!", updated_at as "updated_at!", correlation_id, trace_id
            "#,
            vessel_id,
            vessel_name,
            correlation_id,
            trace_id
        )
            .fetch_one(&self.pool)
            .await
    }

    pub async fn find_all(&self) -> Result<Vec<Vessel>, sqlx::Error> {
        sqlx::query_as!(
            Vessel,
            r#"
            SELECT vessel_id, vessel_name, is_active as "is_active!", created_at as "created_at!", updated_at as "updated_at!", correlation_id, trace_id
            FROM vessel_register_table
            ORDER BY created_at DESC
            "#
        )
            .fetch_all(&self.pool)
            .await
    }

    pub async fn deactivate(&self, vessel_id: &str) -> Result<(), sqlx::Error> {
        sqlx::query!(
            r#"
            UPDATE vessel_register_table
            SET is_active = FALSE, updated_at = NOW()
            WHERE vessel_id = $1
            "#,
            vessel_id
        )
        .execute(&self.pool)
        .await?;

        Ok(())
    }
}
