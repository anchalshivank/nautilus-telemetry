# ============================================
# Build Stage
# ============================================
FROM rust:1.91-slim as builder

WORKDIR /app

# Install build dependencies
RUN apt-get update && \
    apt-get install -y pkg-config libssl-dev curl && \
    rm -rf /var/lib/apt/lists/*

# Copy manifests
COPY Cargo.toml ./

# Create dummy main to cache dependencies
RUN mkdir src && \
    echo "fn main() {println!(\"dummy\");}" > src/main.rs && \
    cargo build --release && \
    rm -rf src target/release/transaction-service* target/release/deps/transaction*

# Copy actual source code
COPY src ./src
COPY migrations ./migrations
COPY .sqlx ./.sqlx
# Build for release - force rebuild (with offline mode for sqlx)
ENV SQLX_OFFLINE=true
RUN cargo build --release

# ============================================
# Runtime Stage
# ============================================
FROM rust:1.91-slim

WORKDIR /app

# Install runtime dependencies (minimal since rust:slim already has most)
RUN apt-get update && \
    apt-get install -y ca-certificates && \
    rm -rf /var/lib/apt/lists/*

# Copy binary from builder
COPY --from=builder /app/target/release/telemetry-service /app/telemetry-service

# Copy migrations and keys
COPY --from=builder /app/migrations /app/migrations

# Expose port
EXPOSE 3000

# Run the binary
CMD ["/app/telemetry-service"]