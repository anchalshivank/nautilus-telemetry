use tracing_subscriber::{EnvFilter, fmt, layer::SubscriberExt, util::SubscriberInitExt};

pub fn init_logging() -> Result<(), Box<dyn std::error::Error>> {
    // JSON formatted logging to stdout
    let fmt_layer = fmt::layer()
        .json()
        .with_target(true)
        .with_thread_ids(true)
        .with_thread_names(true)
        .with_current_span(true)
        .with_span_list(true);

    // Filter based on RUST_LOG environment variable
    let filter_layer =
        EnvFilter::try_from_default_env().unwrap_or_else(|_| "info,telemetry_service=debug".into());

    tracing_subscriber::registry()
        .with(filter_layer)
        .with(fmt_layer)
        .init();

    tracing::info!("Structured JSON logging initialized");

    Ok(())
}
