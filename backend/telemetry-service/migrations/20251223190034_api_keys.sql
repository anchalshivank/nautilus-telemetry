-- API Keys Table
CREATE TABLE IF NOT EXISTS api_keys (
                                        id SERIAL PRIMARY KEY,
                                        vessel_id VARCHAR(50) NOT NULL,
                                        api_key VARCHAR(64) UNIQUE NOT NULL,
                                        is_active BOOLEAN NOT NULL DEFAULT TRUE,
                                        created_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
                                        expires_at TIMESTAMPTZ,
                                        last_used_at TIMESTAMPTZ,
                                        FOREIGN KEY (vessel_id) REFERENCES vessel_register_table(vessel_id) ON DELETE CASCADE
);

CREATE INDEX idx_api_key ON api_keys(api_key) WHERE is_active = TRUE;
CREATE INDEX idx_api_key_vessel ON api_keys(vessel_id);