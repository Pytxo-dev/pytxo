//! Reference Pytxo Cloud sandbox service for Max tier execution.



use std::collections::HashMap;

use std::sync::{Arc, Mutex};



use axum::extract::{Path, Query, State};

use axum::http::{HeaderMap, StatusCode};

use axum::routing::{delete, get, post, put};

use axum::{Json, Router};

use serde::{Deserialize, Serialize};

use tracing::info;

use uuid::Uuid;



#[derive(Clone, Default)]

struct AppState {

    api_key: Option<String>,

    sandboxes: Arc<Mutex<HashMap<String, SandboxRecord>>>,

    cache: Arc<Mutex<HashMap<String, CachedScaffold>>>,

}



#[derive(Clone, Debug, Serialize)]

struct SandboxRecord {

    domain_id: String,

    run_id: String,

    agent_id: String,

}



#[derive(Debug, Deserialize)]

struct StartRequest {

    domain_id: String,

    run_id: String,

    agent_id: String,

    repo_fingerprint: String,

}



#[derive(Serialize)]

struct StartResponse {

    sandbox_id: String,

    ws_url: Option<String>,

}



#[derive(Debug, Deserialize)]

struct ExecRequest {

    cmd: String,

    cwd: Option<String>,

}



#[derive(Serialize)]

struct ExecResponse {

    exit_code: i32,

    stdout: String,

    stderr: String,

}



#[derive(Clone, Debug, Serialize, Deserialize)]

struct CachedScaffold {

    content: String,

    scaffolded_bytes: usize,

    language: Option<String>,

}



#[derive(Debug, Deserialize)]

struct CachePut {

    domain_id: String,

    path: String,

    fidelity: String,

    content_hash: String,

    content: String,

    scaffolded_bytes: usize,

    language: Option<String>,

}



#[derive(Debug, Deserialize)]

struct CacheGetQuery {

    domain_id: String,

    path: String,

    fidelity: String,

    content_hash: String,

}



fn cache_key(q: &CacheGetQuery) -> String {

    format!(

        "{}:{}:{}:{}",

        q.domain_id, q.path, q.fidelity, q.content_hash

    )

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



async fn start_sandbox(

    State(state): State<AppState>,

    headers: HeaderMap,

    Json(body): Json<StartRequest>,

) -> Result<Json<StartResponse>, StatusCode> {

    if !authorized(&headers, &state) {

        return Err(StatusCode::UNAUTHORIZED);

    }

    let id = format!("sbx-{}", Uuid::new_v4());

    info!(sandbox_id = %id, agent = %body.agent_id, "start sandbox");

    state.sandboxes.lock().unwrap().insert(

        id.clone(),

        SandboxRecord {

            domain_id: body.domain_id,

            run_id: body.run_id,

            agent_id: body.agent_id,

        },

    );

    Ok(Json(StartResponse {

        sandbox_id: id,

        ws_url: None,

    }))

}



async fn exec_sandbox(

    State(state): State<AppState>,

    headers: HeaderMap,

    Path(sandbox_id): Path<String>,

    Json(body): Json<ExecRequest>,

) -> Result<Json<ExecResponse>, StatusCode> {

    if !authorized(&headers, &state) {

        return Err(StatusCode::UNAUTHORIZED);

    }

    if !state.sandboxes.lock().unwrap().contains_key(&sandbox_id) {

        return Err(StatusCode::NOT_FOUND);

    }

    info!(sandbox_id = %sandbox_id, cmd = %body.cmd, "exec");

    Ok(Json(ExecResponse {

        exit_code: 0,

        stdout: format!("cloud-stub: {}", body.cmd),

        stderr: String::new(),

    }))

}



async fn teardown(

    State(state): State<AppState>,

    headers: HeaderMap,

    Path(sandbox_id): Path<String>,

) -> StatusCode {

    if !authorized(&headers, &state) {

        return StatusCode::UNAUTHORIZED;

    }

    state.sandboxes.lock().unwrap().remove(&sandbox_id);

    StatusCode::OK

}



async fn cache_get(

    State(state): State<AppState>,

    headers: HeaderMap,

    Query(q): Query<CacheGetQuery>,

) -> Result<Json<CachedScaffold>, StatusCode> {

    if !authorized(&headers, &state) {

        return Err(StatusCode::UNAUTHORIZED);

    }

    let key = cache_key(&q);

    state

        .cache

        .lock()

        .unwrap()

        .get(&key)

        .cloned()

        .map(Json)

        .ok_or(StatusCode::NOT_FOUND)

}



async fn cache_put(

    State(state): State<AppState>,

    headers: HeaderMap,

    Json(body): Json<CachePut>,

) -> StatusCode {

    if !authorized(&headers, &state) {

        return StatusCode::UNAUTHORIZED;

    }

    let key = format!(

        "{}:{}:{}:{}",

        body.domain_id, body.path, body.fidelity, body.content_hash

    );

    state.cache.lock().unwrap().insert(

        key,

        CachedScaffold {

            content: body.content,

            scaffolded_bytes: body.scaffolded_bytes,

            language: body.language,

        },

    );

    StatusCode::OK

}



/// Railway injects `PORT`; local dev uses `CLOUD_BIND` or 127.0.0.1:8788.
fn listen_addr() -> String {
    if let Ok(port) = std::env::var("PORT") {
        if !port.is_empty() {
            return format!("0.0.0.0:{port}");
        }
    }
    std::env::var("CLOUD_BIND").unwrap_or_else(|_| "127.0.0.1:8788".into())
}

#[tokio::main]
async fn main() {
    crate::telemetry::init();



    let state = AppState {

        api_key: std::env::var("CLOUD_API_KEY").ok().filter(|s| !s.is_empty()),

        sandboxes: Arc::new(Mutex::new(HashMap::new())),

        cache: Arc::new(Mutex::new(HashMap::new())),

    };



    let app = Router::new()

        .route("/health", get(health))

        .route("/v1/sandboxes", post(start_sandbox))

        .route("/v1/sandboxes/{id}/exec", post(exec_sandbox))
        .route("/v1/sandboxes/{id}", delete(teardown))

        .route("/v1/cache/scaffold", get(cache_get).put(cache_put))

        .with_state(state);



    let addr = listen_addr();

    let listener = tokio::net::TcpListener::bind(&addr)

        .await

        .expect("bind cloud sandbox");

    info!("pytxo-cloud-sandbox listening on http://{addr}");

    axum::serve(listener, app)

        .await

        .expect("serve cloud sandbox");

}



mod telemetry;


