use crate::repositories::{
    auth::AuthRepository, // Add this
    metrics::MetricsRepository,
    signal::SignalRepository,
    telemetry::TelemetryRepository,
    vessel::VesselRepository,
};
use crate::services::Services;
use sqlx::PgPool;
use std::sync::Arc;

#[derive(Clone)]
pub struct AppState {
    services: Services,
}

impl AppState {
    pub fn builder() -> AppStateBuilder {
        AppStateBuilder::default()
    }

    pub fn services(&self) -> &Services {
        &self.services
    }
}

#[derive(Default)]
pub struct AppStateBuilder {
    db: Option<PgPool>,
}

impl AppStateBuilder {
    pub fn db(mut self, db: PgPool) -> Self {
        self.db = Some(db);
        self
    }

    pub fn build(self) -> AppState {
        let db = self.db.expect("Database pool is required");

        let vessel_repo = Arc::new(VesselRepository::new(db.clone()));
        let signal_repo = Arc::new(SignalRepository::new(db.clone()));
        let telemetry_repo = Arc::new(TelemetryRepository::new(db.clone()));
        let metrics_repo = Arc::new(MetricsRepository::new(db.clone()));
        let auth_repo = Arc::new(AuthRepository::new(db.clone())); // Add this

        let services = Services::new(
            vessel_repo,
            signal_repo,
            telemetry_repo,
            metrics_repo,
            auth_repo, // Add this
        );

        AppState { services }
    }
}
