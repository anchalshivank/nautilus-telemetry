pub mod auth;
pub mod metrics;
pub mod telemetry;
pub mod vessel;
// Add this

use crate::repositories::{
    auth::AuthRepository, metrics::MetricsRepository, signal::SignalRepository,
    telemetry::TelemetryRepository, vessel::VesselRepository,
};
use crate::services::auth::AuthService;
use crate::services::metrics::MetricsService;
use crate::services::telemetry::TelemetryService;
use crate::services::vessel::VesselService; // Add this
use std::sync::Arc;

#[derive(Clone)]
pub struct Services {
    telemetry_service: Arc<TelemetryService>,
    auth_service: Arc<AuthService>,
    vessel_service: Arc<VesselService>, // Add this
    metrics_service: Arc<MetricsService>,
}

impl Services {
    pub fn new(
        vessel_repo: Arc<VesselRepository>,
        signal_repo: Arc<SignalRepository>,
        telemetry_repo: Arc<TelemetryRepository>,
        metrics_repo: Arc<MetricsRepository>,
        auth_repo: Arc<AuthRepository>,
    ) -> Self {
        let vessel_service = Arc::new(VesselService::new(vessel_repo.clone())); // Add this

        let telemetry_service = Arc::new(TelemetryService::new(
            vessel_repo,
            signal_repo,
            telemetry_repo,
            metrics_repo.clone(),
        ));

        let auth_service = Arc::new(AuthService::new(auth_repo));

        let metrics_service = Arc::new(MetricsService::new(metrics_repo));

        Self {
            telemetry_service,
            auth_service,
            vessel_service, // Add this
            metrics_service,
        }
    }

    pub fn telemetry_service(&self) -> Arc<TelemetryService> {
        self.telemetry_service.clone()
    }

    pub fn auth_service(&self) -> Arc<AuthService> {
        self.auth_service.clone()
    }

    pub fn vessel_service(&self) -> Arc<VesselService> {
        // Add this
        self.vessel_service.clone()
    }

    pub fn metrics_service(&self) -> Arc<MetricsService> {
        self.metrics_service.clone()
    }
}
