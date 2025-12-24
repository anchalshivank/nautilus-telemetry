-- Vessel Register Table
CREATE TABLE IF NOT EXISTS vessel_register_table (
                                                     vessel_id VARCHAR(50) PRIMARY KEY,
                                                     vessel_name VARCHAR(255) NOT NULL,
                                                     is_active BOOLEAN NOT NULL DEFAULT TRUE,
                                                     created_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
                                                     updated_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
                                                     correlation_id UUID,
                                                     trace_id VARCHAR(100)
);

CREATE INDEX idx_vessel_active ON vessel_register_table(is_active);
CREATE INDEX idx_vessel_correlation ON vessel_register_table(correlation_id);

-- Signal Register Table
CREATE TABLE IF NOT EXISTS signal_register_table (
                                                     signal_id SERIAL PRIMARY KEY,
                                                     signal_name VARCHAR(100) UNIQUE NOT NULL,
                                                     signal_type VARCHAR(20) NOT NULL CHECK (signal_type IN ('digital', 'analog')),
                                                     min_value DECIMAL(10, 2),
                                                     max_value DECIMAL(10, 2),
                                                     description TEXT,
                                                     created_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
                                                     updated_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
                                                     correlation_id UUID,
                                                     trace_id VARCHAR(100)
);

CREATE INDEX idx_signal_name ON signal_register_table(signal_name);
CREATE INDEX idx_signal_type ON signal_register_table(signal_type);
CREATE INDEX idx_signal_correlation ON signal_register_table(correlation_id);

-- Telemetry Raw Table
CREATE TABLE IF NOT EXISTS telemetry_raw (
                                             id BIGSERIAL PRIMARY KEY,
                                             vessel_id VARCHAR(50) NOT NULL,
                                             timestamp_utc TIMESTAMPTZ NOT NULL,
                                             epoch_utc BIGINT NOT NULL,
                                             signal_name VARCHAR(100) NOT NULL,
                                             signal_value DECIMAL(10, 2) NOT NULL,
                                             ingested_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
                                             correlation_id UUID NOT NULL,
                                             trace_id VARCHAR(100),
                                             FOREIGN KEY (vessel_id) REFERENCES vessel_register_table(vessel_id) ON DELETE CASCADE
);

CREATE INDEX idx_telemetry_vessel_time ON telemetry_raw(vessel_id, timestamp_utc DESC);
CREATE INDEX idx_telemetry_signal ON telemetry_raw(signal_name);
CREATE INDEX idx_telemetry_epoch ON telemetry_raw(epoch_utc DESC);
CREATE INDEX idx_telemetry_correlation ON telemetry_raw(correlation_id);

-- Telemetry Filtered Table
CREATE TABLE IF NOT EXISTS telemetry_filtered (
                                                  id BIGSERIAL PRIMARY KEY,
                                                  vessel_id VARCHAR(50) NOT NULL,
                                                  timestamp_utc TIMESTAMPTZ NOT NULL,
                                                  epoch_utc BIGINT NOT NULL,
                                                  signal_name VARCHAR(100) NOT NULL,
                                                  signal_value DECIMAL(10, 2) NOT NULL,
                                                  reason VARCHAR(255) NOT NULL DEFAULT 'unregistered_signal',
                                                  ingested_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
                                                  correlation_id UUID NOT NULL,
                                                  trace_id VARCHAR(100)
);

CREATE INDEX idx_filtered_vessel_time ON telemetry_filtered(vessel_id, timestamp_utc DESC);
CREATE INDEX idx_filtered_signal ON telemetry_filtered(signal_name);
CREATE INDEX idx_filtered_correlation ON telemetry_filtered(correlation_id);

-- Server Metrics Table
CREATE TABLE IF NOT EXISTS server_metrics (
                                              id BIGSERIAL PRIMARY KEY,
                                              vessel_id VARCHAR(50),
                                              metric_type VARCHAR(50) NOT NULL,
                                              metric_value DECIMAL(10, 3) NOT NULL,
                                              timestamp TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
                                              additional_metadata JSONB,
                                              correlation_id UUID NOT NULL,
                                              trace_id VARCHAR(100)
);

CREATE INDEX idx_metrics_type_time ON server_metrics(metric_type, timestamp DESC);
CREATE INDEX idx_metrics_vessel ON server_metrics(vessel_id);
CREATE INDEX idx_metrics_correlation ON server_metrics(correlation_id);