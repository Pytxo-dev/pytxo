use std::sync::OnceLock;
use std::time::Instant;

use serde::Serialize;

static START: OnceLock<Instant> = OnceLock::new();

fn service_start() -> &'static Instant {
    START.get_or_init(Instant::now)
}

pub fn uptime_secs() -> u64 {
    service_start().elapsed().as_secs()
}

#[derive(Serialize)]
pub struct HealthBody {
    pub status: &'static str,
    pub uptime_secs: u64,
    pub service: &'static str,
    pub providers_configured: Vec<&'static str>,
}

/// Provider ids with non-empty API key env on this host (never exposes secrets).
pub fn configured_providers() -> Vec<&'static str> {
    let mut out = Vec::new();
    let checks: &[(&str, &str)] = &[
        ("deepseek", "DEEPSEEK_API_KEY"),
        ("anthropic", "ANTHROPIC_API_KEY"),
        ("openai", "OPENAI_API_KEY"),
        ("google", "GOOGLE_API_KEY"),
        ("openrouter", "OPENROUTER_API_KEY"),
    ];
    for (id, env) in checks {
        if std::env::var(env)
            .ok()
            .is_some_and(|v| !v.trim().is_empty())
        {
            out.push(*id);
        }
    }
    out
}

pub fn health_body() -> HealthBody {
    HealthBody {
        status: "ok",
        uptime_secs: uptime_secs(),
        service: "pytxo-proxy",
        providers_configured: configured_providers(),
    }
}

pub fn init() {
    let _ = service_start();
    let json_logs = std::env::var("RUST_LOG")
        .map(|v| v.eq_ignore_ascii_case("json"))
        .unwrap_or(false);

    let filter = if json_logs {
        let level = std::env::var("PYTXO_LOG_LEVEL").unwrap_or_else(|_| "info".into());
        tracing_subscriber::EnvFilter::new(level)
    } else {
        tracing_subscriber::EnvFilter::from_default_env()
    };

    if json_logs {
        tracing_subscriber::fmt()
            .json()
            .with_env_filter(filter)
            .init();
    } else {
        tracing_subscriber::fmt().with_env_filter(filter).init();
    }

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
