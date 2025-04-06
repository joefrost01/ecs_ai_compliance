use tracing::Level;
use tracing_subscriber::EnvFilter;

/// Initializes tracing/logging based on environment variables.
pub fn init_tracing() {
    let enable_json =
        std::env::var("LOG_JSON").unwrap_or_else(|_| "false".to_string()) == "true";

    let env_filter = std::env::var("RUST_LOG")
        .ok()
        .map(EnvFilter::new)
        .unwrap_or_else(|| EnvFilter::new("info"));

    let subscriber = tracing_subscriber::fmt()
        .with_max_level(Level::INFO)
        .with_target(true)
        .with_thread_ids(false)
        .with_env_filter(env_filter);

    if enable_json {
        subscriber.json().init();
    } else {
        subscriber.init();
    }
}
