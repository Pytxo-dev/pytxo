//! Minimal Pytxo Link reference service for local Ultra dev.

use std::collections::HashMap;
use std::sync::{Arc, Mutex};

use axum::extract::State;
use axum::http::{HeaderMap, StatusCode};
use axum::routing::{get, post};
use axum::{Json, Router};
use serde::{Deserialize, Serialize};
use tracing::info;

#[derive(Clone, Default)]
struct AppState {
    api_key: Option<String>,
    runs: Arc<Mutex<HashMap<String, RunRecord>>>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
struct RunRecord {
    domain_id: String,
    started: bool,
    ended: bool,
    usage: Option<UsagePayload>,
}

#[derive(Clone, Debug, Deserialize)]
struct RunStartBody {
    domain_id: String,
    run_id: String,
}

#[derive(Clone, Debug, Deserialize)]
struct RunEndBody {
    domain_id: String,
    run_id: String,
    usage: UsagePayload,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
struct UsagePayload {
    tokens_in_billed: u64,
    tokens_in_sent: u64,
    tokens_out: u64,
    saved_tokens: u64,
    cost_micro_usd: i64,
}

fn authorized(headers: &HeaderMap, state: &AppState) -> bool {
    let Some(expected) = state.api_key.as_ref() else {
        return true;
    };
    headers
        .get("authorization")
        .and_then(|v| v.to_str().ok())
        .map(|v| v == format!("Bearer {expected}"))
        .unwrap_or(false)
}

async fn health() -> &'static str {
    "ok"
}

async fn runs_start(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(body): Json<RunStartBody>,
) -> StatusCode {
    if !authorized(&headers, &state) {
        return StatusCode::UNAUTHORIZED;
    }
    info!(run_id = %body.run_id, domain = %body.domain_id, "runs/start");
    state.runs.lock().unwrap().insert(
        body.run_id.clone(),
        RunRecord {
            domain_id: body.domain_id,
            started: true,
            ended: false,
            usage: None,
        },
    );
    StatusCode::OK
}

async fn runs_end(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(body): Json<RunEndBody>,
) -> StatusCode {
    if !authorized(&headers, &state) {
        return StatusCode::UNAUTHORIZED;
    }
    info!(run_id = %body.run_id, saved = body.usage.saved_tokens, "runs/end");
    let mut runs = state.runs.lock().unwrap();
    let entry = runs.entry(body.run_id).or_insert(RunRecord {
        domain_id: body.domain_id.clone(),
        started: false,
        ended: false,
        usage: None,
    });
    entry.ended = true;
    entry.usage = Some(body.usage);
    StatusCode::OK
}

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt()
        .with_env_filter(tracing_subscriber::EnvFilter::from_default_env())
        .init();

    let api_key = std::env::var("LINK_API_KEY").ok().filter(|s| !s.is_empty());
    let state = AppState {
        api_key,
        runs: Arc::new(Mutex::new(HashMap::new())),
    };

    let app = Router::new()
        .route("/health", get(health))
        .route("/v1/runs/start", post(runs_start))
        .route("/v1/runs/end", post(runs_end))
        .with_state(state);

    let addr = std::env::var("LINK_BIND").unwrap_or_else(|_| "127.0.0.1:8787".into());
    let listener = tokio::net::TcpListener::bind(&addr).await.unwrap();
    info!("pytxo-link listening on http://{addr}");
    axum::serve(listener, app).await.unwrap();
}
