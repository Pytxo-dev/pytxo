use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::Mutex;

use pytxo_core::PytxoConfig;
use pytxo_runner::isolation_backend_label;
use pytxo_orchestrate::{
    commit_workspace_for_agent, dispatch_run, dry_run_json, fleet_run_status, fleet_status,
    hitl_respond as orch_hitl_respond,
    list_catalog_domains as orch_list_catalog_domains,
    list_catalog_domains_enriched as orch_list_domains_status, list_domains,
    list_hitl_pending as orch_list_hitl_pending, list_hitl_pending_all as orch_list_hitl_pending_all,
    list_project_manifests as orch_list_projects, project_add_root as orch_project_add_root,
    project_remove_root as orch_project_remove_root, project_roots as orch_project_roots,
    structural_graph as orch_structural_graph, stop, CatalogEntry, CatalogEntryStatus, DomainSummary,
    RunOptions,
};
use pytxo_store::{AgentRecord, EventRecord, RunRecord};
use serde::Serialize;
use tauri::State;

use crate::ipc_error::{
    map_config_err, map_io_err, map_lock_err, map_orch_err, map_store_err, PytxoIpcError,
    IpcResult,
};

pub struct AppState {
    pub config_path: Mutex<Option<PathBuf>>,
    /// Per (domain_id, agent_id) cursor for incremental log polling.
    pub poll_cursors: Mutex<HashMap<(String, String), i64>>,
    pub selected_domain_id: Mutex<Option<String>>,
}

pub(crate) fn open_store_for_domain(
    cfg: &PytxoConfig,
    domain_id: &str,
) -> IpcResult<pytxo_store::PytxoStore> {
    let repo = Path::new(domain_id);
    pytxo_store::PytxoStore::open(&cfg.db_path_at(repo)).map_err(map_store_err)
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
    pub permission_profile: Option<String>,
    pub isolation_mode: String,
    pub isolation_backend: String,
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

pub(crate) fn load_cfg_for_domain(domain_id: &str, state: &AppState) -> IpcResult<PytxoConfig> {
    let path = state.config_path.lock().map_err(map_lock_err)?.clone();
    let repo = PathBuf::from(domain_id);
    if let Some(p) = path {
        if repo.join("pytxo.toml").exists() {
            return PytxoConfig::load(&repo.join("pytxo.toml")).map_err(map_config_err);
        }
        return PytxoConfig::load(&p).map_err(map_config_err);
    }
    if repo.join("pytxo.toml").exists() {
        PytxoConfig::load(&repo.join("pytxo.toml")).map_err(map_config_err)
    } else {
        Ok(PytxoConfig::default())
    }
}

pub(crate) fn resolve_domain(state: &AppState, domain_id: Option<String>) -> IpcResult<String> {
    if let Some(id) = domain_id {
        return Ok(id);
    }
    if let Some(id) = state
        .selected_domain_id
        .lock()
        .map_err(map_lock_err)?
        .clone()
    {
        return Ok(id);
    }
    let cwd = std::env::current_dir().map_err(map_io_err)?;
    Ok(cwd.to_string_lossy().into_owned())
}

#[tauri::command]
pub fn list_domains_cmd() -> IpcResult<Vec<DomainDto>> {
    Ok(list_domains().into_iter().map(domain_to_dto).collect())
}

/// "All projects" home: every domain the hypervisor catalog knows about,
/// including ones not yet loaded into this session ([[execution-domains]] v2).
#[tauri::command]
pub fn list_all_domains() -> IpcResult<Vec<CatalogEntry>> {
    orch_list_catalog_domains().map_err(map_orch_err)
}

/// Enriched catalog with per-domain run health ([[execution-domains]] Phase 3).
#[tauri::command]
pub fn list_domains_status() -> IpcResult<Vec<CatalogEntryStatus>> {
    orch_list_domains_status().map_err(map_orch_err)
}

/// Project manifests from `~/.pytxo/projects` for the Deck project picker.
#[tauri::command]
pub fn list_projects() -> IpcResult<Vec<ProjectDto>> {
    Ok(orch_list_projects()
        .map_err(map_orch_err)?
        .into_iter()
        .map(|(id, path)| ProjectDto {
            id,
            manifest_path: path.to_string_lossy().into_owned(),
        })
        .collect())
}

#[tauri::command]
pub fn select_domain(state: State<'_, AppState>, domain_id: String) -> IpcResult<()> {
    *state.selected_domain_id.lock().map_err(map_lock_err)? = Some(domain_id);
    Ok(())
}

#[tauri::command]
pub fn list_runs(
    state: State<'_, AppState>,
    limit: usize,
    domain_id: Option<String>,
) -> IpcResult<Vec<RunDto>> {
    let domain = resolve_domain(&state, domain_id)?;
    let cfg = load_cfg_for_domain(&domain, &state)?;
    let store = open_store_for_domain(&cfg, &domain)?;
    Ok(store
        .list_runs(limit)
        .map_err(map_store_err)?
        .into_iter()
        .map(|r| run_to_dto(r, &cfg))
        .collect())
}

#[tauri::command]
pub fn list_agents(
    state: State<'_, AppState>,
    run_id: String,
    domain_id: Option<String>,
) -> IpcResult<Vec<AgentDto>> {
    let domain = resolve_domain(&state, domain_id)?;
    let cfg = load_cfg_for_domain(&domain, &state)?;
    let store = open_store_for_domain(&cfg, &domain)?;
    Ok(store
        .list_agents_for_run(&run_id)
        .map_err(map_store_err)?
        .into_iter()
        .map(agent_to_dto)
        .collect())
}

#[derive(Serialize)]
pub struct AgentArbitrageDto {
    pub agent_id: String,
    pub saved_tokens: i64,
    pub edited_paths: i64,
    pub fallback_paths: i64,
}

/// Per-agent Signal Core arbitrage stats for the topology graph.
#[tauri::command]
pub fn agent_arbitrage(
    state: State<'_, AppState>,
    run_id: String,
    domain_id: Option<String>,
) -> IpcResult<Vec<AgentArbitrageDto>> {
    let domain = resolve_domain(&state, domain_id)?;
    let cfg = load_cfg_for_domain(&domain, &state)?;
    let store = open_store_for_domain(&cfg, &domain)?;
    Ok(store
        .arbitrage_by_agent(&run_id)
        .map_err(map_store_err)?
        .into_iter()
        .map(|(agent_id, saved_tokens, edited_paths, fallback_paths)| AgentArbitrageDto {
            agent_id,
            saved_tokens,
            edited_paths,
            fallback_paths,
        })
        .collect())
}

#[tauri::command]
pub fn tail_events(
    state: State<'_, AppState>,
    agent_id: String,
    tail: usize,
    domain_id: Option<String>,
) -> IpcResult<Vec<EventDto>> {
    let domain = resolve_domain(&state, domain_id)?;
    let cfg = load_cfg_for_domain(&domain, &state)?;
    let store = open_store_for_domain(&cfg, &domain)?;
    Ok(store
        .list_events(&agent_id, tail)
        .map_err(map_store_err)?
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
) -> IpcResult<Vec<EventDto>> {
    let domain = resolve_domain(&state, domain_id)?;
    let cfg = load_cfg_for_domain(&domain, &state)?;
    let store = open_store_for_domain(&cfg, &domain)?;
    let cursor_key = (domain.clone(), agent_id.clone());
    let mut cursors = state.poll_cursors.lock().map_err(map_lock_err)?;
    let after = *cursors.get(&cursor_key).unwrap_or(&0);
    let events = store
        .tail_events_after(&agent_id, after, limit)
        .map_err(map_store_err)?;
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
) -> IpcResult<String> {
    let domain = resolve_domain(&state, domain_id)?;
    let path = state.config_path.lock().map_err(map_lock_err)?.clone();
    dry_run_json(path, Some(PathBuf::from(domain)), agents).map_err(map_orch_err)
}

#[tauri::command]
pub fn dispatch_run_cmd(
    state: State<'_, AppState>,
    cmd: String,
    agents: usize,
    repo_root: Option<String>,
) -> IpcResult<String> {
    let path = state.config_path.lock().map_err(map_lock_err)?.clone();
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
    .map_err(map_orch_err)?;
    *state.selected_domain_id.lock().map_err(map_lock_err)? = Some(domain_id.clone());
    Ok(run_id)
}

#[tauri::command]
pub async fn stop_run(state: State<'_, AppState>, all: bool) -> IpcResult<()> {
    let path = state.config_path.lock().map_err(map_lock_err)?.clone();
    let domain = resolve_domain(&state, None)?;
    stop(path, Some(PathBuf::from(domain)), all, false)
        .await
        .map_err(map_orch_err)
}

#[tauri::command]
pub fn commit_workspace(
    state: State<'_, AppState>,
    run_id: String,
    agent_id: String,
    domain_id: Option<String>,
) -> IpcResult<()> {
    let domain = resolve_domain(&state, domain_id)?;
    let path = state.config_path.lock().map_err(map_lock_err)?.clone();
    commit_workspace_for_agent(path, Some(PathBuf::from(domain)), &run_id, &agent_id)
        .map_err(map_orch_err)
}

#[derive(Serialize)]
pub struct HitlDto {
    pub id: String,
    pub agent_key: String,
    pub action: String,
    pub reason: String,
    pub created_at_ms: String,
    pub domain_id: String,
}

#[tauri::command]
pub fn list_hitl(
    state: State<'_, AppState>,
    domain_id: Option<String>,
) -> IpcResult<Vec<HitlDto>> {
    let domain = resolve_domain(&state, domain_id)?;
    let domain_id = domain.clone();
    let pending = orch_list_hitl_pending(Some(PathBuf::from(domain))).map_err(map_orch_err)?;
    Ok(pending
        .into_iter()
        .map(|r| HitlDto {
            id: r.id,
            agent_key: r.agent_key,
            action: r.action,
            reason: r.reason,
            created_at_ms: r.created_at_ms.to_string(),
            domain_id: domain_id.clone(),
        })
        .collect())
}

#[tauri::command]
pub fn list_hitl_all() -> IpcResult<Vec<HitlDto>> {
    Ok(orch_list_hitl_pending_all()
        .map_err(map_orch_err)?
        .into_iter()
        .map(|row| HitlDto {
            id: row.request.id,
            agent_key: row.request.agent_key,
            action: row.request.action,
            reason: row.request.reason,
            created_at_ms: row.request.created_at_ms.to_string(),
            domain_id: row.domain_id,
        })
        .collect())
}

#[tauri::command]
pub fn hitl_respond(
    state: State<'_, AppState>,
    request_id: String,
    approve: bool,
    domain_id: Option<String>,
) -> IpcResult<bool> {
    let domain = resolve_domain(&state, domain_id)?;
    orch_hitl_respond(Some(PathBuf::from(domain)), &request_id, approve).map_err(map_orch_err)
}

#[tauri::command]
pub fn git_diff(
    state: State<'_, AppState>,
    agent_id: String,
    domain_id: Option<String>,
) -> IpcResult<String> {
    let domain = resolve_domain(&state, domain_id)?;
    let cfg = load_cfg_for_domain(&domain, &state)?;
    let store = open_store_for_domain(&cfg, &domain)?;
    let agent = store
        .get_agent(&agent_id)
        .map_err(map_store_err)?
        .ok_or_else(|| PytxoIpcError::new("agent", format!("agent not found: {agent_id}")))?;
    let worktree = agent
        .worktree_path
        .filter(|p| !p.is_empty())
        .ok_or_else(|| PytxoIpcError::new("git", "no worktree path for agent"))?;
    let output = std::process::Command::new("git")
        .args(["-C", &worktree, "diff", "--no-color", "HEAD"])
        .output()
        .map_err(map_io_err)?;
    if !output.status.success() {
        return Err(PytxoIpcError::new(
            "git",
            format!(
                "git diff failed: {}",
                String::from_utf8_lossy(&output.stderr)
            ),
        ));
    }
    Ok(String::from_utf8_lossy(&output.stdout).to_string())
}

#[derive(Serialize)]
pub struct ProjectRootDto {
    pub label: String,
    pub path: String,
    pub read_only: bool,
    pub primary: bool,
    pub permission_profile: Option<String>,
}

#[tauri::command]
pub fn project_roots_cmd(project_id: String) -> IpcResult<Vec<ProjectRootDto>> {
    Ok(orch_project_roots(None, Some(project_id))
        .map_err(map_orch_err)?
        .into_iter()
        .map(
            |(label, path, read_only, primary, permission_profile)| ProjectRootDto {
                label,
                path,
                read_only,
                primary,
                permission_profile,
            },
        )
        .collect())
}

#[derive(Serialize)]
pub struct FleetRunDto {
    pub id: String,
    pub fleet_id: String,
    pub started_at: String,
    pub finished_at: Option<String>,
    pub status: String,
}

#[tauri::command]
pub fn list_fleet_runs(limit: usize) -> IpcResult<Vec<FleetRunDto>> {
    Ok(fleet_status(None, limit)
        .map_err(map_orch_err)?
        .into_iter()
        .map(|r| FleetRunDto {
            id: r.id,
            fleet_id: r.fleet_id,
            started_at: r.started_at,
            finished_at: r.finished_at,
            status: r.status,
        })
        .collect())
}

#[tauri::command]
pub fn project_add_root_cmd(
    project_id: String,
    path: String,
    read_only: bool,
) -> IpcResult<Vec<ProjectRootDto>> {
    orch_project_add_root(None, Some(project_id.clone()), PathBuf::from(path), read_only)
        .map_err(map_orch_err)?;
    Ok(orch_project_roots(None, Some(project_id))
        .map_err(map_orch_err)?
        .into_iter()
        .map(
            |(label, path, read_only, primary, permission_profile)| ProjectRootDto {
                label,
                path,
                read_only,
                primary,
                permission_profile,
            },
        )
        .collect())
}

#[tauri::command]
pub fn project_remove_root_cmd(project_id: String, label: String) -> IpcResult<Vec<ProjectRootDto>> {
    orch_project_remove_root(None, Some(project_id.clone()), &label).map_err(map_orch_err)?;
    Ok(orch_project_roots(None, Some(project_id))
        .map_err(map_orch_err)?
        .into_iter()
        .map(
            |(label, path, read_only, primary, permission_profile)| ProjectRootDto {
                label,
                path,
                read_only,
                primary,
                permission_profile,
            },
        )
        .collect())
}

#[derive(Serialize)]
pub struct FleetNodeDto {
    pub node_id: String,
    pub domain_id: String,
    pub domain_run_id: Option<String>,
    pub wave: i32,
    pub status: String,
}

#[derive(Serialize)]
pub struct FleetRunStatusDto {
    pub id: String,
    pub fleet_id: String,
    pub started_at: String,
    pub finished_at: Option<String>,
    pub status: String,
    pub nodes: Vec<FleetNodeDto>,
}

#[tauri::command]
pub fn fleet_run_status_cmd(fleet_run_id: String) -> IpcResult<FleetRunStatusDto> {
    let status = fleet_run_status(&fleet_run_id).map_err(map_orch_err)?;
    Ok(FleetRunStatusDto {
        id: status.run.id,
        fleet_id: status.run.fleet_id,
        started_at: status.run.started_at,
        finished_at: status.run.finished_at,
        status: status.run.status,
        nodes: status
            .nodes
            .into_iter()
            .map(|n| FleetNodeDto {
                node_id: n.node_id,
                domain_id: n.domain_id,
                domain_run_id: n.domain_run_id,
                wave: n.wave,
                status: n.status,
            })
            .collect(),
    })
}

#[derive(Serialize)]
pub struct StructuralGraphDto {
    pub nodes: Vec<StructuralNodeDto>,
    pub edges: Vec<StructuralEdgeDto>,
}

#[derive(Serialize)]
pub struct StructuralNodeDto {
    pub id: String,
    pub label: String,
    pub edited: bool,
    pub root_id: Option<String>,
}

#[derive(Serialize)]
pub struct StructuralEdgeDto {
    pub from: String,
    pub to: String,
}

#[tauri::command]
pub fn structural_graph(
    state: State<'_, AppState>,
    run_id: String,
    domain_id: Option<String>,
) -> IpcResult<StructuralGraphDto> {
    let domain = resolve_domain(&state, domain_id)?;
    let graph = orch_structural_graph(Some(PathBuf::from(domain)), &run_id).map_err(map_orch_err)?;
    Ok(StructuralGraphDto {
        nodes: graph
            .nodes
            .into_iter()
            .map(|n| StructuralNodeDto {
                id: n.id,
                label: n.label,
                edited: n.edited,
                root_id: n.root_id,
            })
            .collect(),
        edges: graph
            .edges
            .into_iter()
            .map(|e| StructuralEdgeDto {
                from: e.from,
                to: e.to,
            })
            .collect(),
    })
}

fn domain_to_dto(d: DomainSummary) -> DomainDto {
    DomainDto {
        domain_id: d.domain_id,
        repo_root: d.repo_root,
    }
}

fn run_to_dto(r: RunRecord, cfg: &PytxoConfig) -> RunDto {
    RunDto {
        id: r.id,
        status: r.status,
        repo_root: r.repo_root,
        started_at: r.started_at.to_rfc3339(),
        estimated_cost_usd: r.estimated_cost_usd,
        permission_profile: r.permission_profile,
        isolation_mode: cfg.isolation.as_str().to_string(),
        isolation_backend: isolation_backend_label(
            cfg.isolation,
            &cfg.blast.sparse_exclude,
        ),
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
