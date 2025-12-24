use crate::error::AppError;
use crate::models::api_key::ApiKeyResponse;
use crate::repositories::auth::AuthRepository;
use chrono::{DateTime, Utc};
use std::sync::Arc;
use tracing::warn;
use uuid::Uuid;

pub struct AuthService {
    auth_repo: Arc<AuthRepository>,
}

impl AuthService {
    pub fn new(auth_repo: Arc<AuthRepository>) -> Self {
        Self { auth_repo }
    }

    pub async fn validate_api_key(&self, api_key: &str) -> Result<String, AppError> {
        let vessel_id = self.auth_repo.validate_api_key(api_key).await?;

        if let Some(vid) = vessel_id {
            // Update last_used_at asynchronously (fire and forget)
            let repo = self.auth_repo.clone();
            let key = api_key.to_string();
            tokio::spawn(async move {
                let _ = repo.update_last_used(&key).await;
            });

            Ok(vid)
        } else {
            warn!("Invalid API key attempt");
            Err(AppError::Unauthorized("Invalid API key".to_string()))
        }
    }

    pub async fn create_api_key(
        &self,
        vessel_id: &str,
        expires_at: Option<DateTime<Utc>>,
    ) -> Result<ApiKeyResponse, AppError> {
        // Generate secure random API key
        let api_key = format!("sk_{}", Uuid::new_v4().simple());

        let id = self
            .auth_repo
            .create_api_key(vessel_id, &api_key, expires_at)
            .await?;

        Ok(ApiKeyResponse {
            id,
            vessel_id: vessel_id.to_string(),
            api_key,
            is_active: true,
            created_at: Utc::now(),
            expires_at,
            last_used_at: None,
        })
    }

    pub async fn list_api_keys(&self, vessel_id: &str) -> Result<Vec<ApiKeyResponse>, AppError> {
        let keys = self.auth_repo.list_api_keys(vessel_id).await?;

        Ok(keys
            .into_iter()
            .map(
                |(id, vessel_id, api_key, is_active, created_at, expires_at, last_used_at)| {
                    ApiKeyResponse {
                        id,
                        vessel_id,
                        api_key,
                        is_active,
                        created_at,
                        expires_at,
                        last_used_at,
                    }
                },
            )
            .collect())
    }

    pub async fn revoke_api_key(&self, api_key: &str) -> Result<(), AppError> {
        self.auth_repo.revoke_api_key(api_key).await?;
        Ok(())
    }
}
