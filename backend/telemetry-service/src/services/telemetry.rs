use crate::error::AppError;
use crate::models::signal::Signal;
use crate::models::telemetry::{TelemetryRequest, TelemetryResponse};
use crate::repositories::{
    metrics::MetricsRepository, signal::SignalRepository, telemetry::TelemetryRepository,
    vessel::VesselRepository,
};
use chrono::{DateTime, Utc};
use rust_decimal::Decimal;
use serde::Deserialize;
use std::collections::HashMap;
use std::sync::Arc;
use std::time::Instant;
use tracing::{info, warn};
use uuid::Uuid;

pub struct TelemetryService {
    vessel_repo: Arc<VesselRepository>,
    signal_repo: Arc<SignalRepository>,
    telemetry_repo: Arc<TelemetryRepository>,
    metrics_repo: Arc<MetricsRepository>,
}

// Struct to hold validated signal data
#[derive(Clone)]
struct ValidatedSignals {
    valid: Vec<(String, DateTime<Utc>, i64, String, Decimal, Uuid, String)>,
    invalid: Vec<(
        String,
        DateTime<Utc>,
        i64,
        String,
        Decimal,
        String,
        Uuid,
        String,
    )>,
}

impl TelemetryService {
    pub fn new(
        vessel_repo: Arc<VesselRepository>,
        signal_repo: Arc<SignalRepository>,
        telemetry_repo: Arc<TelemetryRepository>,
        metrics_repo: Arc<MetricsRepository>,
    ) -> Self {
        Self {
            vessel_repo,
            signal_repo,
            telemetry_repo,
            metrics_repo,
        }
    }

    pub async fn ingest_telemetry(
        &self,
        request: TelemetryRequest,
    ) -> Result<TelemetryResponse, AppError> {
        let correlation_id = Uuid::new_v4();
        let trace_id = Uuid::new_v4().to_string();
        let total_start = Instant::now();

        info!(
            correlation_id = %correlation_id,
            vessel_id = %request.vessel_id,
            "Starting telemetry ingestion"
        );

        // STEP 1: Record Request Volume
        self.record_request_volume(&request.vessel_id, correlation_id, trace_id.clone())
            .await?;

        // STEP 2: VALIDATION LAYER
        let validation_start = Instant::now();
        let registered_signals = self
            .validate_vessel_and_load_signals(&request.vessel_id)
            .await?;
        let validation_duration = validation_start.elapsed().as_millis();

        info!(
            validation_duration_ms = validation_duration,
            "Validation layer completed"
        );

        // STEP 3: SIGNAL VALIDATION (Part of Validation Layer)
        let validated = self.validate_all_signals(
            &request,
            &registered_signals,
            correlation_id,
            trace_id.clone(),
        );

        info!(
            valid_count = validated.valid.len(),
            invalid_count = validated.invalid.len(),
            "Signal validation completed"
        );
        self.record_validation_latency(
            &request.vessel_id,
            validation_duration,
            correlation_id,
            trace_id.clone(),
        )
        .await?;

        // STEP 4: INGESTION LAYER
        let ingestion_start = Instant::now();
        self.ingest_to_database(&validated).await?;
        let ingestion_duration = ingestion_start.elapsed().as_millis();

        info!(
            ingestion_duration_ms = ingestion_duration,
            "Ingestion layer completed"
        );

        self.record_ingestion_latency(
            &request.vessel_id,
            ingestion_duration,
            correlation_id,
            trace_id.clone(),
        )
        .await?;

        // STEP 5: Record Total Duration
        let total_duration = total_start.elapsed().as_millis();
        self.record_total_latency(
            &request.vessel_id,
            total_duration,
            correlation_id,
            trace_id.clone(),
        )
        .await?;

        info!(
            total_duration_ms = total_duration,
            "Telemetry ingestion completed successfully"
        );

        Ok(TelemetryResponse {
            message: "Telemetry ingested successfully".to_string(),
            correlation_id,
            valid_signals: validated.valid.len(),
            invalid_signals: validated.invalid.len(),
        })
    }

    async fn record_request_volume(
        &self,
        vessel_id: &str,
        correlation_id: Uuid,
        trace_id: String,
    ) -> Result<(), AppError> {
        info!("Recording request volume metric");

        self.metrics_repo
            .insert_metric(
                Some(vessel_id.to_string()),
                "request_volume".to_string(),
                Decimal::from(1),
                correlation_id,
                trace_id,
            )
            .await?;

        Ok(())
    }

    /// Validates vessel exists and loads all registered signals
    async fn validate_vessel_and_load_signals(
        &self,
        vessel_id: &str,
    ) -> Result<HashMap<String, Signal>, AppError> {
        info!(vessel_id = %vessel_id, "Validating vessel existence");

        // Check if vessel exists in vessel_register_table
        let vessel = self.vessel_repo.find_by_id(vessel_id).await?;

        if vessel.is_none() {
            warn!(vessel_id = %vessel_id, "Vessel not registered");
            return Err(AppError::Forbidden(format!(
                "Vessel {} is not registered in vessel_register_table",
                vessel_id
            )));
        }

        info!(vessel_id = %vessel_id, "Vessel validated successfully");

        // Load all registered signals from signal_register_table
        info!("Loading registered signals from signal_register_table");
        let registered_signals = self.signal_repo.find_all().await?;

        info!(
            signal_count = registered_signals.len(),
            "Registered signals loaded"
        );

        Ok(registered_signals)
    }

    /// Validates all signals in the request against registered signals
    fn validate_all_signals(
        &self,
        request: &TelemetryRequest,
        registered_signals: &HashMap<String, Signal>,
        correlation_id: Uuid,
        trace_id: String,
    ) -> ValidatedSignals {
        let mut valid_records = Vec::new();
        let mut invalid_records = Vec::new();

        info!(
            signal_count = request.signals.len(),
            "Starting signal validation"
        );

        for (signal_name, signal_value) in request.signals.iter() {
            // Convert JSON value to Decimal
            let value_decimal = match self.parse_signal_value(signal_value) {
                Ok(val) => val,
                Err(reason) => {
                    warn!(signal = %signal_name, reason = %reason, "Invalid value type");
                    invalid_records.push((
                        request.vessel_id.clone(),
                        request.timestamp_utc,
                        request.epoch_utc.parse::<i64>().unwrap_or(0),
                        signal_name.clone(),
                        Decimal::ZERO,
                        reason,
                        correlation_id,
                        trace_id.clone(),
                    ));
                    continue;
                }
            };

            // Check if signal exists in signal_register_table
            match registered_signals.get(signal_name) {
                Some(signal) => {
                    // Signal is registered, now validate its value
                    match self.validate_signal_value(signal, value_decimal) {
                        Ok(_) => {
                            // Valid signal and valid value
                            info!(
                                signal = %signal_name,
                                value = %value_decimal,
                                "Signal validated successfully"
                            );
                            valid_records.push((
                                request.vessel_id.clone(),
                                request.timestamp_utc,
                                request.epoch_utc.parse::<i64>().unwrap_or(0),
                                signal_name.clone(),
                                value_decimal,
                                correlation_id,
                                trace_id.clone(),
                            ));
                        }
                        Err(reason) => {
                            // Signal is registered but value is invalid
                            warn!(
                                signal = %signal_name,
                                value = %value_decimal,
                                reason = %reason,
                                "Signal value validation failed"
                            );
                            invalid_records.push((
                                request.vessel_id.clone(),
                                request.timestamp_utc,
                                request.epoch_utc.parse::<i64>().unwrap_or(0),
                                signal_name.clone(),
                                value_decimal,
                                reason,
                                correlation_id,
                                trace_id.clone(),
                            ));
                        }
                    }
                }
                None => {
                    // Signal not found in signal_register_table
                    warn!(
                        signal = %signal_name,
                        "Signal not found in signal_register_table"
                    );
                    invalid_records.push((
                        request.vessel_id.clone(),
                        request.timestamp_utc,
                        request.epoch_utc.parse::<i64>().unwrap_or(0),
                        signal_name.clone(),
                        value_decimal,
                        "unregistered_signal".to_string(),
                        correlation_id,
                        trace_id.clone(),
                    ));
                }
            }
        }

        ValidatedSignals {
            valid: valid_records,
            invalid: invalid_records,
        }
    }

    /// Parses JSON value into Decimal
    fn parse_signal_value(&self, value: &serde_json::Value) -> Result<Decimal, String> {
        match value {
            serde_json::Value::Number(n) => {
                if let Some(f) = n.as_f64() {
                    Decimal::try_from(f)
                        .map_err(|_| "Failed to convert float to decimal".to_string())
                } else if let Some(i) = n.as_i64() {
                    Ok(Decimal::from(i))
                } else {
                    Err("Number format not supported".to_string())
                }
            }
            _ => Err("invalid_value_type".to_string()),
        }
    }

    /// Validates signal value based on signal type (digital/analog)
    fn validate_signal_value(&self, signal: &Signal, value: Decimal) -> Result<(), String> {
        match signal.signal_type.as_str() {
            "digital" => {
                // Digital signals must be exactly 0 or 1
                if value != Decimal::ZERO && value != Decimal::ONE {
                    return Err(format!(
                        "Digital signal '{}' must be 0 or 1, got: {}",
                        signal.signal_name, value
                    ));
                }
                Ok(())
            }
            "analog" => {
                // Analog signals must be within min/max range
                if let Some(min) = signal.min_value {
                    if value < min {
                        return Err(format!(
                            "Analog signal '{}' value {} is below minimum {}",
                            signal.signal_name, value, min
                        ));
                    }
                }
                if let Some(max) = signal.max_value {
                    if value > max {
                        return Err(format!(
                            "Analog signal '{}' value {} is above maximum {}",
                            signal.signal_name, value, max
                        ));
                    }
                }
                Ok(())
            }
            _ => Err(format!(
                "Unknown signal type '{}' for signal '{}'",
                signal.signal_type, signal.signal_name
            )),
        }
    }

    async fn record_validation_latency(
        &self,
        vessel_id: &str,
        duration_ms: u128,
        correlation_id: Uuid,
        trace_id: String,
    ) -> Result<(), AppError> {
        self.metrics_repo
            .insert_metric(
                Some(vessel_id.to_string()),
                "latency_validation".to_string(),
                Decimal::from(duration_ms),
                correlation_id,
                trace_id,
            )
            .await?;

        Ok(())
    }

    /// Writes validated signals to database
    async fn ingest_to_database(&self, validated: &ValidatedSignals) -> Result<(), AppError> {
        // Write valid signals to telemetry_raw table
        if !validated.valid.is_empty() {
            info!(
                count = validated.valid.len(),
                "Writing valid signals to telemetry_raw"
            );
            self.telemetry_repo
                .insert_raw_batch(&validated.valid)
                .await?;
            info!("Valid signals written successfully");
        }

        // Write invalid signals to telemetry_filtered table
        if !validated.invalid.is_empty() {
            info!(
                count = validated.invalid.len(),
                "Writing invalid signals to telemetry_filtered"
            );
            self.telemetry_repo
                .insert_filtered_batch(&validated.invalid)
                .await?;
            info!("Invalid signals written successfully");
        }

        Ok(())
    }

    async fn record_ingestion_latency(
        &self,
        vessel_id: &str,
        duration_ms: u128,
        correlation_id: Uuid,
        trace_id: String,
    ) -> Result<(), AppError> {
        self.metrics_repo
            .insert_metric(
                Some(vessel_id.to_string()),
                "latency_ingestion".to_string(),
                Decimal::from(duration_ms),
                correlation_id,
                trace_id,
            )
            .await?;

        Ok(())
    }

    async fn record_total_latency(
        &self,
        vessel_id: &str,
        duration_ms: u128,
        correlation_id: Uuid,
        trace_id: String,
    ) -> Result<(), AppError> {
        self.metrics_repo
            .insert_metric(
                Some(vessel_id.to_string()),
                "latency_total".to_string(),
                Decimal::from(duration_ms),
                correlation_id,
                trace_id,
            )
            .await?;

        Ok(())
    }
}
