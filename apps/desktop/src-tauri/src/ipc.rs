use std::path::PathBuf;
use std::sync::Mutex;

use pytxo_core::PytxoConfig;
use pytxo_orchestrate::{dry_run_json, run, stop, RunOptions};
use pytxo_store::{AgentRecord, EventRecord, RunRecord};
use serde::Serialize;
use tauri::State;

pub struct AppState {
    pub config_path: Mutex<Option<PathBuf>>,
    pub last_event_id: Mutex<i64>,
}

#[derive(Serialize)]
pub struct RunDto {
    pub id: String,
    pub status: String,
    pub repo_root: String,
    pub started_at: String,
    pub estimated_cost_usd: Option<f64>,
}

#[derive(Serialize)]
pub struct AgentDto {
    pub id: String,
    pub run_id: String,
    pub task_id: String,
    pub wave: i32,
    pub status: String,
    pub exit_code: Option<i32>,
}

#[derive(Serialize)]
pub struct EventDto {
    pub id: i64,
    pub agent_id: String,
    pub kind: String,
    pub payload: String,
    pub ts: String,
}

fn load_cfg(state: &AppState) -> Result<PytxoConfig, String> {
    let repo = std::env::current_dir().map_err(|e| e.to_string())?;
    let path = state.config_path.lock().map_err(|e| e.to_string())?.clone();
    if let Some(p) = path {
        PytxoConfig::load(&p).map_err(|e| e.to_string())
    } else if repo.join("pytxo.toml").exists() {
        PytxoConfig::load(&repo.join("pytxo.toml")).map_err(|e| e.to_string())
    } else {
        Ok(PytxoConfig::default())
    }
}

#[tauri::command]
pub fn list_runs(state: State<'_, AppState>, limit: usize) -> Result<Vec<RunDto>, String> {
    let cfg = load_cfg(&state)?;
    let store = pytxo_store::PytxoStore::open(&cfg.db_path()).map_err(|e| e.to_string())?;
    Ok(store
        .list_runs(limit)
        .map_err(|e| e.to_string())?
        .into_iter()
        .map(run_to_dto)
        .collect())
}

#[tauri::command]
pub fn list_agents(state: State<'_, AppState>, run_id: String) -> Result<Vec<AgentDto>, String> {
    let cfg = load_cfg(&state)?;
    let store = pytxo_store::PytxoStore::open(&cfg.db_path()).map_err(|e| e.to_string())?;
    Ok(store
        .list_agents_for_run(&run_id)
        .map_err(|e| e.to_string())?
        .into_iter()
        .map(agent_to_dto)
        .collect())
}

#[tauri::command]
pub fn tail_events(
    state: State<'_, AppState>,
    agent_id: String,
    tail: usize,
) -> Result<Vec<EventDto>, String> {
    let cfg = load_cfg(&state)?;
    let store = pytxo_store::PytxoStore::open(&cfg.db_path()).map_err(|e| e.to_string())?;
    Ok(store
        .list_events(&agent_id, tail)
        .map_err(|e| e.to_string())?
        .into_iter()
        .map(event_to_dto)
        .collect())
}

#[tauri::command]
pub fn poll_log_lines(
    state: State<'_, AppState>,
    agent_id: String,
    limit: usize,
) -> Result<Vec<EventDto>, String> {
    let cfg = load_cfg(&state)?;
    let store = pytxo_store::PytxoStore::open(&cfg.db_path()).map_err(|e| e.to_string())?;
    let after = *state.last_event_id.lock().map_err(|e| e.to_string())?;
    let events = store
        .tail_events_after(&agent_id, after, limit)
        .map_err(|e| e.to_string())?;
    if let Some(last) = events.last() {
        *state.last_event_id.lock().map_err(|e| e.to_string())? = last.id;
    }
    Ok(events.into_iter().map(event_to_dto).collect())
}

#[tauri::command]
pub fn dry_run(state: State<'_, AppState>, agents: usize) -> Result<String, String> {
    let path = state.config_path.lock().map_err(|e| e.to_string())?.clone();
    dry_run_json(path, None, agents).map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn start_run(
    state: State<'_, AppState>,
    cmd: String,
    agents: usize,
) -> Result<String, String> {
    let path = state.config_path.lock().map_err(|e| e.to_string())?.clone();
    let run_id = run(RunOptions {
        agents,
        cmd,
        config: path,
        dry_run: false,
        keep_worktrees: false,
        repo: None,
    })
    .await
    .map_err(|e| e.to_string())?;
    Ok(run_id.0)
}

#[tauri::command]
pub async fn stop_run(state: State<'_, AppState>, all: bool) -> Result<(), String> {
    let path = state.config_path.lock().map_err(|e| e.to_string())?.clone();
    stop(path, None, all, false)
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub fn git_diff(state: State<'_, AppState>, agent_id: String) -> Result<String, String> {
    let cfg = load_cfg(&state)?;
    let store = pytxo_store::PytxoStore::open(&cfg.db_path()).map_err(|e| e.to_string())?;
    let agent = store
        .get_agent(&agent_id)
        .map_err(|e| e.to_string())?
        .ok_or_else(|| format!("agent not found: {agent_id}"))?;
    let worktree = agent
        .worktree_path
        .filter(|p| !p.is_empty())
        .ok_or_else(|| "no worktree path for agent (run may have removed worktrees)".to_string())?;
    let output = std::process::Command::new("git")
        .args(["-C", &worktree, "diff", "--no-color", "HEAD"])
        .output()
        .map_err(|e| e.to_string())?;
    if !output.status.success() {
        return Err(format!(
            "git diff failed: {}",
            String::from_utf8_lossy(&output.stderr)
        ));
    }
    Ok(String::from_utf8_lossy(&output.stdout).to_string())
}

fn run_to_dto(r: RunRecord) -> RunDto {
    RunDto {
        id: r.id,
        status: r.status,
        repo_root: r.repo_root,
        started_at: r.started_at.to_rfc3339(),
        estimated_cost_usd: r.estimated_cost_usd,
    }
}

fn agent_to_dto(a: AgentRecord) -> AgentDto {
    AgentDto {
        id: a.id,
        run_id: a.run_id,
        task_id: a.task_id,
        wave: a.wave,
        status: a.status,
        exit_code: a.exit_code,
    }
}

fn event_to_dto(e: EventRecord) -> EventDto {
    EventDto {
        id: e.id,
        agent_id: e.agent_id,
        kind: e.kind,
        payload: e.payload,
        ts: e.ts.to_rfc3339(),
    }
}
