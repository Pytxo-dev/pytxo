//! Reference Pytxo Cloud sandbox service for Max tier execution.

use std::collections::HashMap;
use std::process::Command;
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

use axum::extract::{Path, Query, State};
use axum::http::{HeaderMap, StatusCode};
use axum::routing::{delete, get, post};
use axum::{Json, Router};
use serde::{Deserialize, Serialize};
use tracing::{info, warn};
use uuid::Uuid;

#[derive(Clone)]
struct AppState {
    api_key: Option<String>,
    require_auth: bool,
    link_base: Option<String>,
    max_workers: usize,
    sandbox_ttl: Duration,
    sandboxes: Arc<Mutex<HashMap<String, SandboxRecord>>>,
    cache: Arc<Mutex<HashMap<String, CachedScaffold>>>,
    workers: Arc<Mutex<WorkerPool>>,
    http: reqwest::Client,
}

#[derive(Clone, Debug)]
struct WorkerPool {
    available: Vec<WorkerSlot>,
    leased: HashMap<String, usize>,
}

#[derive(Clone, Debug)]
struct WorkerSlot {
    id: usize,
    label: String,
}

impl WorkerPool {
    fn new(max_workers: usize) -> Self {
        let available = (0..max_workers)
            .map(|id| WorkerSlot {
                id,
                label: format!("worker-{id}"),
            })
            .collect();
        Self {
            available,
            leased: HashMap::new(),
        }
    }

    fn lease(&mut self, sandbox_id: &str) -> Option<String> {
        if let Some(&worker_id) = self.leased.get(sandbox_id) {
            return Some(format!("worker-{worker_id}"));
        }
        let slot = self.available.pop()?;
        self.leased.insert(sandbox_id.to_string(), slot.id);
        Some(slot.label)
    }

    fn release(&mut self, sandbox_id: &str) {
        if let Some(worker_id) = self.leased.remove(sandbox_id) {
            self.available.push(WorkerSlot {
                id: worker_id,
                label: format!("worker-{worker_id}"),
            });
        }
    }
}

#[derive(Clone, Debug)]
struct SandboxRecord {
    domain_id: String,
    run_id: String,
    agent_id: String,
    created_at: Instant,
    worker: Option<String>,
    container_id: Option<String>,
    container_name: Option<String>,
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
struct SyncRequest {
    files: Vec<SyncFileEntry>,
}

#[derive(Debug, Deserialize, Serialize)]
struct SyncFileEntry {
    path: String,
    content: String,
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

#[derive(Debug, Deserialize)]
struct LinkEntitlements {
    cloud_enabled: bool,
}

fn cache_key(q: &CacheGetQuery) -> String {
    format!(
        "{}:{}:{}:{}",
        q.domain_id, q.path, q.fidelity, q.content_hash
    )
}

fn constant_time_eq(a: &[u8], b: &[u8]) -> bool {
    if a.len() != b.len() {
        return false;
    }
    let mut diff = 0u8;
    for (x, y) in a.iter().zip(b.iter()) {
        diff |= x ^ y;
    }
    diff == 0
}

fn docker_available() -> bool {
    Command::new("docker")
        .arg("version")
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false)
}

fn docker_start_container(sandbox_id: &str) -> Option<(String, String)> {
    if !docker_available() {
        return None;
    }
    let name = format!("pytxo-{sandbox_id}");
    let output = Command::new("docker")
        .args([
            "run",
            "-d",
            "--name",
            &name,
            "alpine:3.20",
            "sh",
            "-c",
            "mkdir -p /workspace && sleep infinity",
        ])
        .output()
        .ok()?;
    if !output.status.success() {
        warn!(
            stderr = %String::from_utf8_lossy(&output.stderr),
            "docker run failed"
        );
        return None;
    }
    let container_id = String::from_utf8_lossy(&output.stdout).trim().to_string();
    Some((container_id, name))
}

fn docker_rm_force(container_name: &str) {
    let _ = Command::new("docker")
        .args(["rm", "-f", container_name])
        .output();
}

fn docker_sync_files(container_name: &str, files: &[SyncFileEntry]) -> Result<(), String> {
    if files.is_empty() {
        return Ok(());
    }
    let tar_path = std::env::temp_dir().join(format!("pytxo-sync-{}.tar", Uuid::new_v4()));
    {
        let file = std::fs::File::create(&tar_path).map_err(|e| e.to_string())?;
        let mut builder = tar::Builder::new(file);
        for entry in files {
            let mut header = tar::Header::new_gnu();
            let data = entry.content.as_bytes();
            header.set_size(data.len() as u64);
            header.set_mode(0o644);
            header.set_cksum();
            builder
                .append_data(&mut header, &entry.path, data)
                .map_err(|e| e.to_string())?;
        }
        builder.finish().map_err(|e| e.to_string())?;
    }

    let cp = Command::new("docker")
        .args([
            "cp",
            tar_path.to_str().unwrap_or_default(),
            &format!("{container_name}:/tmp/pytxo-sync.tar"),
        ])
        .output()
        .map_err(|e| e.to_string())?;
    std::fs::remove_file(&tar_path).ok();
    if !cp.status.success() {
        return Err(format!(
            "docker cp failed: {}",
            String::from_utf8_lossy(&cp.stderr)
        ));
    }

    let extract = Command::new("docker")
        .args([
            "exec",
            container_name,
            "sh",
            "-c",
            "cd /workspace && tar xf /tmp/pytxo-sync.tar && rm -f /tmp/pytxo-sync.tar",
        ])
        .output()
        .map_err(|e| e.to_string())?;
    if !extract.status.success() {
        return Err(format!(
            "tar extract failed: {}",
            String::from_utf8_lossy(&extract.stderr)
        ));
    }
    Ok(())
}

fn docker_exec(container_id: &str, cmd: &str, cwd: Option<&str>) -> ExecResponse {
    let workdir = cwd.unwrap_or("/workspace");
    let args = vec!["exec", "-i", "-w", workdir, container_id, "sh", "-c", cmd];
    match Command::new("docker").args(&args).output() {
        Ok(output) => ExecResponse {
            exit_code: output.status.code().unwrap_or(1),
            stdout: String::from_utf8_lossy(&output.stdout).into_owned(),
            stderr: String::from_utf8_lossy(&output.stderr).into_owned(),
        },
        Err(e) => ExecResponse {
            exit_code: 1,
            stdout: String::new(),
            stderr: format!("docker exec failed: {e}"),
        },
    }
}

fn authorized(headers: &HeaderMap, state: &AppState) -> bool {
    if !state.require_auth {
        return true;
    }
    let Some(expected) = state.api_key.as_ref() else {
        return false;
    };
    let Some(header) = headers
        .get("authorization")
        .and_then(|v| v.to_str().ok())
    else {
        return false;
    };
    let prefix = "Bearer ";
    if !header.starts_with(prefix) {
        return false;
    }
    constant_time_eq(header[prefix.len()..].as_bytes(), expected.as_bytes())
}

async fn cloud_entitled(state: &AppState, headers: &HeaderMap) -> bool {
    let Some(base) = state.link_base.as_ref() else {
        return true;
    };
    let user_id = headers
        .get("x-pytxo-user-id")
        .and_then(|v| v.to_str().ok())
        .unwrap_or("");
    if user_id.is_empty() {
        return false;
    }
    let url = format!("{}/v1/entitlements/status", base.trim_end_matches('/'));
    let mut req = state.http.get(&url);
    if let Some(auth) = headers.get("authorization").and_then(|v| v.to_str().ok()) {
        req = req.header("Authorization", auth);
    }
    req = req.header("x-pytxo-user-id", user_id);
    match req.send().await {
        Ok(resp) if resp.status().is_success() => resp
            .json::<LinkEntitlements>()
            .await
            .map(|b| b.cloud_enabled)
            .unwrap_or(false),
        Ok(resp) => {
            warn!(status = %resp.status(), "entitlements fetch failed");
            false
        }
        Err(e) => {
            warn!(error = %e, "entitlements fetch error");
            false
        }
    }
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
    if !cloud_entitled(&state, &headers).await {
        return Err(StatusCode::FORBIDDEN);
    }
    let id = format!("sbx-{}", Uuid::new_v4());
    let worker = {
        let mut pool = state.workers.lock().unwrap();
        pool.lease(&id)
    };
    if worker.is_none() {
        return Err(StatusCode::SERVICE_UNAVAILABLE);
    }
    info!(sandbox_id = %id, agent = %body.agent_id, worker = ?worker, "start sandbox");
    let (container_id, container_name) = docker_start_container(&id)
        .ok_or(StatusCode::SERVICE_UNAVAILABLE)?;
    state.sandboxes.lock().unwrap().insert(
        id.clone(),
        SandboxRecord {
            domain_id: body.domain_id,
            run_id: body.run_id,
            agent_id: body.agent_id,
            created_at: Instant::now(),
            worker,
            container_id: Some(container_id),
            container_name: Some(container_name),
        },
    );
    Ok(Json(StartResponse {
        sandbox_id: id,
        ws_url: None,
    }))
}

async fn sync_sandbox(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(sandbox_id): Path<String>,
    Json(body): Json<SyncRequest>,
) -> Result<StatusCode, StatusCode> {
    if !authorized(&headers, &state) {
        return Err(StatusCode::UNAUTHORIZED);
    }
    let record = state
        .sandboxes
        .lock()
        .unwrap()
        .get(&sandbox_id)
        .cloned();
    let Some(record) = record else {
        return Err(StatusCode::NOT_FOUND);
    };
    let Some(name) = record.container_name.as_deref() else {
        return Err(StatusCode::SERVICE_UNAVAILABLE);
    };
    info!(
        sandbox_id = %sandbox_id,
        files = body.files.len(),
        "delta sync"
    );
    docker_sync_files(name, &body.files).map_err(|e| {
        warn!(error = %e, "sync failed");
        StatusCode::INTERNAL_SERVER_ERROR
    })?;
    Ok(StatusCode::OK)
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
    if !cloud_entitled(&state, &headers).await {
        return Err(StatusCode::FORBIDDEN);
    }
    let record = state
        .sandboxes
        .lock()
        .unwrap()
        .get(&sandbox_id)
        .cloned();
    let Some(record) = record else {
        return Err(StatusCode::NOT_FOUND);
    };
    info!(sandbox_id = %sandbox_id, cmd = %body.cmd, worker = ?record.worker, "exec");
    let Some(cid) = record.container_id.as_deref() else {
        return Err(StatusCode::SERVICE_UNAVAILABLE);
    };
    let response = docker_exec(cid, &body.cmd, body.cwd.as_deref());
    Ok(Json(response))
}

async fn teardown(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(sandbox_id): Path<String>,
) -> StatusCode {
    if !authorized(&headers, &state) {
        return StatusCode::UNAUTHORIZED;
    }
    let record = state.sandboxes.lock().unwrap().remove(&sandbox_id);
    state.workers.lock().unwrap().release(&sandbox_id);
    if let Some(rec) = record {
        if let Some(name) = rec.container_name.as_deref() {
            docker_rm_force(name);
        }
    }
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

fn sweep_expired(state: &AppState) {
    let ttl = state.sandbox_ttl;
    let expired: Vec<(String, Option<String>)> = state
        .sandboxes
        .lock()
        .unwrap()
        .iter()
        .filter(|(_, rec)| rec.created_at.elapsed() > ttl)
        .map(|(id, rec)| (id.clone(), rec.container_name.clone()))
        .collect();
    for (id, name) in expired {
        state.sandboxes.lock().unwrap().remove(&id);
        state.workers.lock().unwrap().release(&id);
        if let Some(n) = name.as_deref() {
            docker_rm_force(n);
            info!(sandbox_id = %id, "ttl sweep removed sandbox");
        }
    }
}

async fn ttl_sweeper(state: AppState) {
    let interval = Duration::from_secs(60);
    loop {
        tokio::time::sleep(interval).await;
        sweep_expired(&state);
    }
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

fn max_workers() -> usize {
    std::env::var("CLOUD_MAX_WORKERS")
        .ok()
        .and_then(|s| s.parse().ok())
        .unwrap_or(4)
}

fn sandbox_ttl() -> Duration {
    std::env::var("CLOUD_SANDBOX_TTL_SECS")
        .ok()
        .and_then(|s| s.parse().ok())
        .map(Duration::from_secs)
        .unwrap_or_else(|| Duration::from_secs(3600))
}

#[tokio::main]
async fn main() {
    crate::telemetry::init();

    let api_key = std::env::var("CLOUD_API_KEY").ok().filter(|s| !s.is_empty());
    let require_auth = std::env::var("CLOUD_REQUIRE_AUTH")
        .map(|v| v == "1" || v.eq_ignore_ascii_case("true"))
        .unwrap_or_else(|_| api_key.is_some());
    let workers_n = max_workers();
    let link_base = std::env::var("LINK_BASE_URL")
        .ok()
        .filter(|s| !s.is_empty());

    let state = AppState {
        api_key: api_key.clone(),
        require_auth,
        link_base,
        max_workers: workers_n,
        sandbox_ttl: sandbox_ttl(),
        sandboxes: Arc::new(Mutex::new(HashMap::new())),
        cache: Arc::new(Mutex::new(HashMap::new())),
        workers: Arc::new(Mutex::new(WorkerPool::new(workers_n))),
        http: reqwest::Client::new(),
    };

    if require_auth && api_key.is_none() {
        tracing::warn!("CLOUD_REQUIRE_AUTH set but CLOUD_API_KEY missing — all requests will be rejected");
    }

    let sweeper_state = state.clone();
    tokio::spawn(async move {
        ttl_sweeper(sweeper_state).await;
    });

    let app = Router::new()
        .route("/health", get(health))
        .route("/v1/sandboxes/start", post(start_sandbox))
        .route("/v1/sandboxes/{id}/sync", post(sync_sandbox))
        .route("/v1/sandboxes/{id}/exec", post(exec_sandbox))
        .route("/v1/sandboxes/{id}", delete(teardown))
        .route("/v1/cache/scaffold", get(cache_get).put(cache_put))
        .with_state(state);

    let addr = listen_addr();
    let listener = tokio::net::TcpListener::bind(&addr)
        .await
        .expect("bind cloud sandbox");
    info!(
        workers = workers_n,
        require_auth = require_auth,
        %addr,
        "pytxo-cloud-sandbox listening"
    );
    axum::serve(listener, app)
        .await
        .expect("serve cloud sandbox");
}

mod telemetry;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn worker_pool_leases_and_releases() {
        let mut pool = WorkerPool::new(2);
        let w1 = pool.lease("sbx-1").unwrap();
        assert_eq!(w1, "worker-1");
        let w2 = pool.lease("sbx-2").unwrap();
        assert!(w2.starts_with("worker-"));
        assert!(pool.lease("sbx-3").is_none());
        pool.release("sbx-1");
        assert!(pool.lease("sbx-3").is_some());
    }

    #[test]
    fn constant_time_compare() {
        assert!(constant_time_eq(b"secret", b"secret"));
        assert!(!constant_time_eq(b"secret", b"secrex"));
        assert!(!constant_time_eq(b"a", b"ab"));
    }
}
