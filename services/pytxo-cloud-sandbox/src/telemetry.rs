pub fn init() {
    tracing_subscriber::fmt()
        .with_env_filter(tracing_subscriber::EnvFilter::from_default_env())
        .init();

    if let Ok(endpoint) = std::env::var("OTEL_EXPORTER_OTLP_ENDPOINT") {
        if !endpoint.is_empty() {
            tracing::info!(endpoint = %endpoint, "OTLP endpoint configured (export via collector sidecar)");
        }
    }

    if let Ok(dsn) = std::env::var("SENTRY_DSN") {
        if !dsn.is_empty() {
            tracing::info!("Sentry DSN configured");
        }
    }
}
