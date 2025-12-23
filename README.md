# Telemetry Service

A high-performance Rust-based telemetry ingestion service for maritime vessels. Handles real-time signal data validation, storage, and performance monitoring.

## Architecture Overview

### System Components

```
┌─────────────────┐
│   Client/Vessel │
└────────┬────────┘
         │ HTTP + x-api-key
         ↓
┌─────────────────────────────────┐
│   Telemetry Service (Rust)      │
│   - Authentication Middleware   │
│   - Validation Layer            │
│   - Ingestion Layer             │
│   - Metrics Recording           │
└────────┬────────────────────────┘
         │
         ↓
┌─────────────────────────────────┐
│   PostgreSQL Database           │
│   - Vessel Registry             │
│   - Signal Registry             │
│   - Telemetry Data              │
│   - Performance Metrics         │
└─────────────────────────────────┘
```

### Technology Stack

- **Language**: Rust (Edition 2024)
- **Web Framework**: Axum 0.8
- **Database**: PostgreSQL 16 with SQLx
- **Logging**: Structured JSON logs via tracing
- **Containerization**: Docker & Docker Compose

## Data Flow

### 1. Authentication
```
Request → Middleware validates x-api-key → Extract vessel_id → Continue
```

### 2. Validation Layer
```
Check vessel exists → Load registered signals → Validate signal values
├─ Digital signals: Must be 0 or 1
└─ Analog signals: Must be within min/max range
```

### 3. Ingestion Layer
```
Valid signals → telemetry_raw table
Invalid signals → telemetry_filtered table (with reason)
```

### 4. Metrics Recording
```
Record: request_volume, latency_validation, latency_ingestion, latency_total
```

## Database Schema

### Core Tables

**vessel_register_table**
- Stores registered vessels
- Primary Key: `vessel_id`

**signal_register_table**
- Defines valid signals (Signal_1 to Signal_200)
- Types: digital (0/1) or analog (1.0-65535.0)

**api_keys**
- Authentication tokens per vessel
- Links API key to vessel_id

**telemetry_raw**
- Valid telemetry data
- Indexed by vessel_id and timestamp

**telemetry_filtered**
- Invalid telemetry with rejection reasons
- Used for data quality monitoring

**server_metrics**
- Performance metrics (request counts, latencies)
- Queryable via REST APIs

## How to Run Locally

### Prerequisites
- Docker & Docker Compose
- Git

### Setup Steps

1. **Clone the repository**
```bash
git clone https://github.com/anchalshivank/nautilus-telemetry.git
cd nautilus-telemetry
```

2. **Start services**
```bash
docker-compose up --build -d
```

3. **Verify services are running**
```bash
docker-compose ps
curl http://localhost:3000/health
```

4. **Setup test data**
```bash
# Create a vessel
curl -X POST http://localhost:3000/api/v1/vessels \
  -H "Content-Type: application/json" \
  -H "x-admin-key: admin_secret_key_change_me" \
  -d '{"vesselId": "VESSEL_001", "vesselName": "Test Ship"}'

# Generate API key
curl -X POST http://localhost:3000/api/v1/api-keys \
  -H "Content-Type: application/json" \
  -H "x-admin-key: admin_secret_key_change_me" \
  -d '{"vesselId": "VESSEL_001"}'

# Save the returned API key for testing
```

5. **Insert signal definitions**
```bash
docker exec -it telemetry-postgres psql -U telemetry -d telemetry_db

-- Insert digital signals (Signal_1 to Signal_50)
INSERT INTO signal_register_table (signal_name, signal_type, min_value, max_value) VALUES
  ('Signal_1', 'digital', 0, 1),
  ('Signal_2', 'digital', 0, 1);

-- Insert analog signals (Signal_51 to Signal_200)  
INSERT INTO signal_register_table (signal_name, signal_type, min_value, max_value) VALUES
  ('Signal_51', 'analog', 1.0, 65535.0),
  ('Signal_52', 'analog', 1.0, 65535.0);

\q
```

6. **Send test telemetry**
```bash
curl -X POST http://localhost:3000/api/v1/telemetry \
  -H "Content-Type: application/json" \
  -H "x-api-key: <your-api-key>" \
  -d '{
    "vesselId": "VESSEL_001",
    "timestampUTC": "2025-12-24T10:30:00Z",
    "epochUTC": "1735039800",
    "signals": {
      "Signal_1": 1,
      "Signal_2": 0,
      "Signal_51": 1450.25,
      "Signal_52": 32000.12
    }
  }'
```

### Available Endpoints

**Public:**
- `GET /` - Service info
- `GET /api/v1/health` - Health check

**Telemetry (requires x-api-key):**
- `POST /api/v1/telemetry` - Ingest telemetry data

**Admin (requires x-admin-key):**
- Vessel management: `/api/v1/vessels`
- API keys: `/api/v1/api-keys`
- Metrics: `/api/v1/metrics`, `/api/v1/metrics/summary`

## Scaling Considerations

### Current Bottlenecks
1. **Database connections**: Limited by connection pool size (default: 5)
2. **Single instance**: No horizontal scaling yet
3. **Synchronous metrics writes**: Could slow down request processing

### Scaling Strategies

**Horizontal Scaling:**
- Deploy multiple service instances behind a load balancer
- Shared PostgreSQL with connection pooling
- Stateless design allows easy replication

**Database Optimization:**
- Table partitioning by vessel_id or timestamp
- Read replicas for metrics queries
- TimescaleDB for time-series optimization

**Async Improvements:**
- Move metrics writes to background tasks
- Batch insert telemetry data
- Use message queue for high-volume ingestion

**Caching:**
- Cache vessel and signal registrations
- Reduce database lookups on validation layer

## Known Limitations

1. **Metrics in Main Database**: Performance metrics share the same database as telemetry data, which could impact write performance at scale

2. **No Rate Limiting**: Currently no protection against request flooding from a single vessel

3. **Signal Registry Caching**: Loads all signals from database on every request instead of caching

4. **Synchronous Validation**: Validation happens inline with request processing

5. **Limited Error Recovery**: No retry mechanism for failed database writes

6. **Admin Key Security**: Admin key stored in environment variable, not ideal for production

## Production Improvements

### Security
- Replace simple admin key with proper JWT-based authentication
- Add API key rotation mechanism
- Implement rate limiting per vessel
- Add TLS/HTTPS support
- Audit logging for all admin operations

### Performance
- Separate metrics database or use time-series database (InfluxDB/TimescaleDB)
- Cache signal definitions in memory with periodic refresh
- Async metrics recording (fire-and-forget)
- Connection pool tuning based on load
- Add request queuing for burst traffic

### Observability
- Add Prometheus exporter for metrics
- Implement distributed tracing (OpenTelemetry)
- Real-time dashboards (Grafana)
- Alerting for high latency or error rates

### Reliability
- Database connection retry logic
- Circuit breakers for external dependencies
- Health checks with readiness/liveness probes
- Graceful degradation when metrics DB is unavailable

### Data Quality
- Data validation rules engine
- Anomaly detection for signal values
- Automated data quality reports
- Signal value trend analysis

### Operations
- Kubernetes deployment manifests
- CI/CD pipeline
- Automated database backups
- Database migration management
- Configuration management (Consul/etcd)

## Development

### Build locally
```bash
cd backend/telemetry-service
cargo build --release
```

## Project Structure

```
nautilus-telemetry/
├── backend/
│   └── telemetry-service/
│       ├── src/
│       │   ├── controller/     # HTTP handlers
│       │   ├── middleware/     # Auth & validation
│       │   ├── models/         # Data structures
│       │   ├── repositories/   # Database access
│       │   ├── services/       # Business logic
│       │   └── main.rs
│       ├── migrations/         # Database migrations
│       └── Dockerfile
├── docker-compose.yml
└── README.md
```

## License

MIT