use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt, EnvFilter};

/// Initialize the tracing subscriber with environment-based filtering
///
/// # Examples
///
/// ```
/// // Set RUST_LOG=lighter_rust=debug for debug logging
/// // Set RUST_LOG=lighter_rust=trace for trace logging
/// // Set RUST_LOG=warn for warnings only
///
/// lighter_rust::init_logging();
/// ```
pub fn init_logging() {
    let filter =
        EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("lighter_rust=info"));

    tracing_subscriber::registry()
        .with(filter)
        .with(tracing_subscriber::fmt::layer())
        .init();
}

/// Initialize logging with a custom filter
pub fn init_logging_with_filter(filter: &str) {
    let env_filter = EnvFilter::new(filter);

    tracing_subscriber::registry()
        .with(env_filter)
        .with(tracing_subscriber::fmt::layer())
        .init();
}
