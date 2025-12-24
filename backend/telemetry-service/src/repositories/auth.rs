use chrono::{DateTime, Utc};
use sqlx::PgPool;

pub struct AuthRepository {
    pool: PgPool,
}

impl AuthRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    pub async fn validate_api_key(&self, api_key: &str) -> Result<Option<String>, sqlx::Error> {
        let result = sqlx::query!(
            r#"
            SELECT vessel_id
            FROM api_keys
            WHERE api_key = $1
              AND is_active = TRUE
              AND (expires_at IS NULL OR expires_at > NOW())
            "#,
            api_key
        )
        .fetch_optional(&self.pool)
        .await?;

        Ok(result.map(|r| r.vessel_id))
    }

    pub async fn update_last_used(&self, api_key: &str) -> Result<(), sqlx::Error> {
        sqlx::query!(
            r#"
            UPDATE api_keys
            SET last_used_at = $1
            WHERE api_key = $2
            "#,
            Utc::now(),
            api_key
        )
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    pub async fn create_api_key(
        &self,
        vessel_id: &str,
        api_key: &str,
        expires_at: Option<DateTime<Utc>>,
    ) -> Result<i32, sqlx::Error> {
        let result = sqlx::query!(
            r#"
            INSERT INTO api_keys (vessel_id, api_key, expires_at)
            VALUES ($1, $2, $3)
            RETURNING id
            "#,
            vessel_id,
            api_key,
            expires_at
        )
        .fetch_one(&self.pool)
        .await?;

        Ok(result.id)
    }

    pub async fn list_api_keys(
        &self,
        vessel_id: &str,
    ) -> Result<
        Vec<(
            i32,
            String,
            String,
            bool,
            DateTime<Utc>,
            Option<DateTime<Utc>>,
            Option<DateTime<Utc>>,
        )>,
        sqlx::Error,
    > {
        let keys = sqlx::query!(
            r#"
            SELECT id, vessel_id, api_key, is_active, created_at, expires_at, last_used_at
            FROM api_keys
            WHERE vessel_id = $1
            ORDER BY created_at DESC
            "#,
            vessel_id
        )
        .fetch_all(&self.pool)
        .await?;

        Ok(keys
            .into_iter()
            .map(|k| {
                (
                    k.id,
                    k.vessel_id,
                    k.api_key,
                    k.is_active,
                    k.created_at,
                    k.expires_at,
                    k.last_used_at,
                )
            })
            .collect())
    }

    pub async fn revoke_api_key(&self, api_key: &str) -> Result<(), sqlx::Error> {
        sqlx::query!(
            r#"
            UPDATE api_keys
            SET is_active = FALSE
            WHERE api_key = $1
            "#,
            api_key
        )
        .execute(&self.pool)
        .await?;

        Ok(())
    }
}
