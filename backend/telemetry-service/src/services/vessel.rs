use crate::error::AppError;
use crate::models::vessel::{CreateVesselRequest, VesselResponse};
use crate::repositories::vessel::VesselRepository;
use std::sync::Arc;
use tracing::info;
use uuid::Uuid;

pub struct VesselService {
    vessel_repo: Arc<VesselRepository>,
}

impl VesselService {
    pub fn new(vessel_repo: Arc<VesselRepository>) -> Self {
        Self { vessel_repo }
    }

    pub async fn create_vessel(
        &self,
        request: CreateVesselRequest,
    ) -> Result<VesselResponse, AppError> {
        // Check if vessel already exists
        if let Some(_) = self.vessel_repo.find_by_id(&request.vessel_id).await? {
            return Err(AppError::Conflict(format!(
                "Vessel {} already exists",
                request.vessel_id
            )));
        }

        let correlation_id = Uuid::new_v4();
        let trace_id = Some(Uuid::new_v4().to_string());

        let vessel = self
            .vessel_repo
            .create(
                &request.vessel_id,
                &request.vessel_name,
                correlation_id,
                trace_id,
            )
            .await?;

        info!("Vessel created: {}", vessel.vessel_id);

        Ok(VesselResponse {
            vessel_id: vessel.vessel_id,
            vessel_name: vessel.vessel_name,
            is_active: vessel.is_active,
            created_at: vessel.created_at,
            updated_at: vessel.updated_at,
        })
    }

    pub async fn get_vessel(&self, vessel_id: &str) -> Result<VesselResponse, AppError> {
        let vessel = self
            .vessel_repo
            .find_by_id(vessel_id)
            .await?
            .ok_or_else(|| AppError::NotFound(format!("Vessel {} not found", vessel_id)))?;

        Ok(VesselResponse {
            vessel_id: vessel.vessel_id,
            vessel_name: vessel.vessel_name,
            is_active: vessel.is_active,
            created_at: vessel.created_at,
            updated_at: vessel.updated_at,
        })
    }

    pub async fn list_vessels(&self) -> Result<Vec<VesselResponse>, AppError> {
        let vessels = self.vessel_repo.find_all().await?;

        Ok(vessels
            .into_iter()
            .map(|v| VesselResponse {
                vessel_id: v.vessel_id,
                vessel_name: v.vessel_name,
                is_active: v.is_active,
                created_at: v.created_at,
                updated_at: v.updated_at,
            })
            .collect())
    }

    pub async fn deactivate_vessel(&self, vessel_id: &str) -> Result<(), AppError> {
        // Check if vessel exists
        self.vessel_repo
            .find_by_id(vessel_id)
            .await?
            .ok_or_else(|| AppError::NotFound(format!("Vessel {} not found", vessel_id)))?;

        self.vessel_repo.deactivate(vessel_id).await?;

        info!("Vessel deactivated: {}", vessel_id);

        Ok(())
    }
}
