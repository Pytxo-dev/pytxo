use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::Mutex;

use pytxo_core::PytxoConfig;
use pytxo_orchestrate::{
    commit_workspace_for_agent, dispatch_run, dry_run_json, hitl_respond as orch_hitl_respond,
    list_catalog_domains as orch_list_catalog_domains, list_domains,
    list_hitl_pending as orch_list_hitl_pending, list_project_manifests as orch_list_projects,
    stop, CatalogEntry, DomainSummary, RunOptions,
};
use pytxo_store::{AgentRecord, EventRecord, RunRecord};
use serde::Serialize;
use tauri::State;

pub struct AppState {
    pub config_path: Mutex<Option<PathBuf>>,
    /// Per (domain_id, agent_id) cursor for incremental log polling.
    pub poll_cursors: Mutex<HashMap<(String, String), i64>>,
    pub selected_domain_id: Mutex<Option<String>>,
}

fn open_store_for_domain(
    cfg: &PytxoConfig,
    domain_id: &str,
) -> Result<pytxo_store::PytxoStore, String> {
    let repo = Path::new(domain_id);
    pytxo_store::PytxoStore::open(&cfg.db_path_at(repo)).map_err(|e| e.to_string())
}

#[derive(Serialize)]
pub struct DomainDto {
    pub domain_id: String,
    pub repo_root: String,
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
    pub root_id: Option<String>,
}

#[derive(Serialize)]
pub struct ProjectDto {
    pub id: String,
    pub manifest_path: String,
}

#[derive(Serialize)]
pub struct EventDto {
    pub id: i64,
    pub agent_id: String,
    pub kind: String,
    pub payload: String,
    pub ts: String,
}

fn load_cfg_for_domain(domain_id: &str, state: &AppState) -> Result<PytxoConfig, String> {
    let path = state.config_path.lock().map_err(|e| e.to_string())?.clone();
    let repo = PathBuf::from(domain_id);
    if let Some(p) = path {
        if repo.join("pytxo.toml").exists() {
            return PytxoConfig::load(&repo.join("pytxo.toml")).map_err(|e| e.to_string());
        }
        return PytxoConfig::load(&p).map_err(|e| e.to_string());
    }
    if repo.join("pytxo.toml").exists() {
        PytxoConfig::load(&repo.join("pytxo.toml")).map_err(|e| e.to_string())
    } else {
        Ok(PytxoConfig::default())
    }
}

fn resolve_domain(state: &AppState, domain_id: Option<String>) -> Result<String, String> {
    if let Some(id) = domain_id {
        return Ok(id);
    }
    if let Some(id) = state
        .selected_domain_id
        .lock()
        .map_err(|e| e.to_string())?
        .clone()
    {
        return Ok(id);
    }
    let cwd = std::env::current_dir().map_err(|e| e.to_string())?;
    Ok(cwd.to_string_lossy().into_owned())
}

#[tauri::command]
pub fn list_domains_cmd() -> Result<Vec<DomainDto>, String> {
    Ok(list_domains().into_iter().map(domain_to_dto).collect())
}

/// "All projects" home: every domain the hypervisor catalog knows about,
/// including ones not yet loaded into this session ([[execution-domains]] v2).
#[tauri::command]
pub fn list_all_domains() -> Result<Vec<CatalogEntry>, String> {
    orch_list_catalog_domains().map_err(|e| e.to_string())
}

/// Project manifests from `~/.pytxo/projects` for the Deck project picker.
#[tauri::command]
pub fn list_projects() -> Result<Vec<ProjectDto>, String> {
    Ok(orch_list_projects()
        .map_err(|e| e.to_string())?
        .into_iter()
        .map(|(id, path)| ProjectDto {
            id,
            manifest_path: path.to_string_lossy().into_owned(),
        })
        .collect())
}

#[tauri::command]
pub fn select_domain(state: State<'_, AppState>, domain_id: String) -> Result<(), String> {
    *state.selected_domain_id.lock().map_err(|e| e.to_string())? = Some(domain_id);
    Ok(())
}

#[tauri::command]
pub fn list_runs(
    state: State<'_, AppState>,
    limit: usize,
    domain_id: Option<String>,
) -> Result<Vec<RunDto>, String> {
    let domain = resolve_domain(&state, domain_id)?;
    let cfg = load_cfg_for_domain(&domain, &state)?;
    let store = open_store_for_domain(&cfg, &domain)?;
    Ok(store
        .list_runs(limit)
        .map_err(|e| e.to_string())?
        .into_iter()
        .map(run_to_dto)
        .collect())
}

#[tauri::command]
pub fn list_agents(
    state: State<'_, AppState>,
    run_id: String,
    domain_id: Option<String>,
) -> Result<Vec<AgentDto>, String> {
    let domain = resolve_domain(&state, domain_id)?;
    let cfg = load_cfg_for_domain(&domain, &state)?;
    let store = open_store_for_domain(&cfg, &domain)?;
    Ok(store
        .list_agents_for_run(&run_id)
        .map_err(|e| e.to_string())?
        .into_iter()
        .map(agent_to_dto)
        .collect())
}

#[derive(Serialize)]
pub struct AgentArbitrageDto {
    pub agent_id: String,
    pub saved_tokens: i64,
    pub edited_paths: i64,
}

/// Per-agent Signal Core arbitrage stats for the topology graph.
#[tauri::command]
pub fn agent_arbitrage(
    state: State<'_, AppState>,
    run_id: String,
    domain_id: Option<String>,
) -> Result<Vec<AgentArbitrageDto>, String> {
    let domain = resolve_domain(&state, domain_id)?;
    let cfg = load_cfg_for_domain(&domain, &state)?;
    let store = open_store_for_domain(&cfg, &domain)?;
    Ok(store
        .arbitrage_by_agent(&run_id)
        .map_err(|e| e.to_string())?
        .into_iter()
        .map(|(agent_id, saved_tokens, edited_paths)| AgentArbitrageDto {
            agent_id,
            saved_tokens,
            edited_paths,
        })
        .collect())
}

#[tauri::command]
pub fn tail_events(
    state: State<'_, AppState>,
    agent_id: String,
    tail: usize,
    domain_id: Option<String>,
) -> Result<Vec<EventDto>, String> {
    let domain = resolve_domain(&state, domain_id)?;
    let cfg = load_cfg_for_domain(&domain, &state)?;
    let store = open_store_for_domain(&cfg, &domain)?;
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
    domain_id: Option<String>,
) -> Result<Vec<EventDto>, String> {
    let domain = resolve_domain(&state, domain_id)?;
    let cfg = load_cfg_for_domain(&domain, &state)?;
    let store = open_store_for_domain(&cfg, &domain)?;
    let cursor_key = (domain.clone(), agent_id.clone());
    let mut cursors = state.poll_cursors.lock().map_err(|e| e.to_string())?;
    let after = *cursors.get(&cursor_key).unwrap_or(&0);
    let events = store
        .tail_events_after(&agent_id, after, limit)
        .map_err(|e| e.to_string())?;
    if let Some(last) = events.last() {
        cursors.insert(cursor_key, last.id);
    }
    Ok(events.into_iter().map(event_to_dto).collect())
}

#[tauri::command]
pub fn dry_run(
    state: State<'_, AppState>,
    agents: usize,
    domain_id: Option<String>,
) -> Result<String, String> {
    let domain = resolve_domain(&state, domain_id)?;
    let path = state.config_path.lock().map_err(|e| e.to_string())?.clone();
    dry_run_json(path, Some(PathBuf::from(domain)), agents).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn dispatch_run_cmd(
    state: State<'_, AppState>,
    cmd: String,
    agents: usize,
    repo_root: Option<String>,
) -> Result<String, String> {
    let path = state.config_path.lock().map_err(|e| e.to_string())?.clone();
    let repo = repo_root
        .map(PathBuf::from)
        .or_else(|| std::env::current_dir().ok());
    let (domain_id, run_id) = dispatch_run(RunOptions {
        agents,
        cmd,
        config: path,
        dry_run: false,
        keep_worktrees: false,
        repo,
        execution: None,
        project: None,
        tasks: None,
        task_cmd_template: None,
        task_prompts: None,
    })
    .map_err(|e| e.to_string())?;
    *state.selected_domain_id.lock().map_err(|e| e.to_string())? = Some(domain_id.clone());
    Ok(run_id)
}

#[tauri::command]
pub async fn stop_run(state: State<'_, AppState>, all: bool) -> Result<(), String> {
    let path = state.config_path.lock().map_err(|e| e.to_string())?.clone();
    let domain = resolve_domain(&state, None)?;
    stop(path, Some(PathBuf::from(domain)), all, false)
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub fn commit_workspace(
    state: State<'_, AppState>,
    run_id: String,
    agent_id: String,
    domain_id: Option<String>,
) -> Result<(), String> {
    let domain = resolve_domain(&state, domain_id)?;
    let path = state.config_path.lock().map_err(|e| e.to_string())?.clone();
    commit_workspace_for_agent(path, Some(PathBuf::from(domain)), &run_id, &agent_id)
        .map_err(|e| e.to_string())
}

#[derive(Serialize)]
pub struct HitlDto {
    pub id: String,
    pub agent_key: String,
    pub action: String,
    pub reason: String,
    pub created_at_ms: String,
}

#[tauri::command]
pub fn list_hitl(
    state: State<'_, AppState>,
    domain_id: Option<String>,
) -> Result<Vec<HitlDto>, String> {
    let domain = resolve_domain(&state, domain_id)?;
    let pending = orch_list_hitl_pending(Some(PathBuf::from(domain))).map_err(|e| e.to_string())?;
    Ok(pending
        .into_iter()
        .map(|r| HitlDto {
            id: r.id,
            agent_key: r.agent_key,
            action: r.action,
            reason: r.reason,
            created_at_ms: r.created_at_ms.to_string(),
        })
        .collect())
}

#[tauri::command]
pub fn hitl_respond(
    state: State<'_, AppState>,
    request_id: String,
    approve: bool,
    domain_id: Option<String>,
) -> Result<bool, String> {
    let domain = resolve_domain(&state, domain_id)?;
    orch_hitl_respond(Some(PathBuf::from(domain)), &request_id, approve).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn git_diff(
    state: State<'_, AppState>,
    agent_id: String,
    domain_id: Option<String>,
) -> Result<String, String> {
    let domain = resolve_domain(&state, domain_id)?;
    let cfg = load_cfg_for_domain(&domain, &state)?;
    let store = open_store_for_domain(&cfg, &domain)?;
    let agent = store
        .get_agent(&agent_id)
        .map_err(|e| e.to_string())?
        .ok_or_else(|| format!("agent not found: {agent_id}"))?;
    let worktree = agent
        .worktree_path
        .filter(|p| !p.is_empty())
        .ok_or_else(|| "no worktree path for agent".to_string())?;
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

fn domain_to_dto(d: DomainSummary) -> DomainDto {
    DomainDto {
        domain_id: d.domain_id,
        repo_root: d.repo_root,
    }
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
        root_id: a.root_id,
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
