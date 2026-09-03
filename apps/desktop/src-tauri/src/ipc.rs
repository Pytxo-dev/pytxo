use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::Mutex;

use pytxo_core::PytxoConfig;
use pytxo_orchestrate::{
    apply_run_changes as orch_apply_run_changes, default_hypervisor,
    discard_run_review as orch_discard_run_review, dispatch_run, dry_run_json, fleet_run_status,
    fleet_status, forget_catalog_domain as orch_forget_domain, hitl_respond as orch_hitl_respond,
    is_repo_trusted, list_catalog_domains as orch_list_catalog_domains,
    list_catalog_domains_enriched as orch_list_domains_status, list_domains,
    list_hitl_pending as orch_list_hitl_pending,
    list_hitl_pending_all as orch_list_hitl_pending_all,
    list_project_manifests as orch_list_projects, project_add_root as orch_project_add_root,
    project_remove_root as orch_project_remove_root, project_roots as orch_project_roots,
    reconcile_run_recovery as orch_reconcile_run_recovery,
    refresh_run_review as orch_refresh_run_review, stop, stop_exact,
    structural_graph as orch_structural_graph, trust_repo, trusted_permission_for,
    workspace_structural_graph as orch_workspace_structural_graph, CatalogEntry,
    CatalogEntryStatus, DomainSummary, RunOptions,
};
use pytxo_runner::{isolation_backend_label, RecoveryOutcome};
use pytxo_store::{AgentRecord, EventRecord, RunContractRecord, RunRecord};
use serde::{Deserialize, Serialize};
use tauri::{Emitter, State};

use crate::ipc_error::{
    map_config_err, map_io_err, map_lock_err, map_orch_err, map_store_err, IpcResult, PytxoIpcError,
};

pub struct AppState {
    pub config_path: Mutex<Option<PathBuf>>,
    /// Per (domain_id, agent_id) cursor for incremental log polling.
    pub poll_cursors: Mutex<HashMap<(String, String), i64>>,
    pub selected_domain_id: Mutex<Option<String>>,
    pub voice_sessions: std::sync::Arc<Mutex<HashMap<uuid::Uuid, pytxo_voice::VoiceSession>>>,
    pub voice_captures: std::sync::Arc<Mutex<HashMap<uuid::Uuid, pytxo_voice::CpalCapture>>>,
    pub voice_cancellations:
        Mutex<HashMap<uuid::Uuid, std::sync::Arc<std::sync::atomic::AtomicBool>>>,
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
    pub domain_id: String,
    pub status: String,
    pub repo_root: String,
    pub started_at: String,
    pub estimated_cost_usd: Option<f64>,
    pub permission_profile: Option<String>,
    pub isolation_mode: String,
    pub isolation_backend: String,
    pub apply_status: Option<String>,
    pub applied_at: Option<String>,
    pub prepared_digest: Option<String>,
    pub prepared_at: Option<String>,
    pub last_apply_error: Option<pytxo_core::RunApplyError>,
    pub recovery_state: Option<String>,
}

#[derive(Serialize)]
pub struct RunReviewDto {
    pub run_id: String,
    pub base_revision: Option<String>,
    pub apply_status: String,
    pub applied_at: Option<String>,
    pub plan: serde_json::Value,
    pub enforcement: serde_json::Value,
    pub apply_manifest: Option<serde_json::Value>,
    pub prepared_manifest: Option<pytxo_core::PreparedRunManifest>,
    pub prepared_digest: Option<String>,
    pub prepared_at: Option<String>,
    pub last_apply_error: Option<pytxo_core::RunApplyError>,
    pub recovery_state: Option<String>,
    pub apply_attempts: Vec<RunApplyAttemptDto>,
}

#[derive(Clone, Debug, Serialize)]
pub struct PreparedContentChunkDto {
    pub run_id: String,
    pub package_digest: String,
    pub path: String,
    pub side: String,
    pub digest: String,
    pub byte_count: u64,
    pub binary: bool,
    pub offset: u64,
    pub length: u64,
    pub next_offset: u64,
    pub complete: bool,
    pub data_base64: String,
}

fn load_review_content_chunk(
    repo: &Path,
    cfg: &PytxoConfig,
    run_id: &str,
    path: &str,
    side: &str,
    offset: u64,
    limit: usize,
) -> IpcResult<PreparedContentChunkDto> {
    use base64::Engine;

    const MAX_CHUNK_BYTES: usize = 64 * 1024;
    if limit == 0 || limit > MAX_CHUNK_BYTES {
        return Err(PytxoIpcError::new(
            "run_review_content",
            format!("content chunk limit must be 1..={MAX_CHUNK_BYTES}"),
        ));
    }
    let data_dir = repo.join(&cfg.data_dir);
    let store = open_store_for_domain(cfg, &repo.to_string_lossy())?;
    let contract = store
        .get_run_contract(run_id)
        .map_err(map_store_err)?
        .ok_or_else(|| {
            PytxoIpcError::new("run_review_content", "run has no persisted review contract")
        })?;
    let persisted_manifest = contract.prepared_manifest.as_ref().ok_or_else(|| {
        PytxoIpcError::new(
            "run_review_content",
            "persisted review contract has no prepared manifest",
        )
    })?;
    let persisted_digest = contract.prepared_digest.as_deref().ok_or_else(|| {
        PytxoIpcError::new(
            "run_review_content",
            "persisted review contract has no prepared digest",
        )
    })?;
    let manifest = pytxo_runner::load_review_manifest(&data_dir, run_id)
        .map_err(|error| PytxoIpcError::new("run_review_content", error.to_string()))?;
    if persisted_manifest != &manifest
        || persisted_manifest.run_id != run_id
        || persisted_manifest.package_digest != persisted_digest
        || manifest.package_digest != persisted_digest
    {
        return Err(PytxoIpcError::new(
            "run_review_content",
            "prepared package does not match the persisted review contract",
        ));
    }
    let file = manifest
        .files
        .iter()
        .find(|candidate| candidate.path == path)
        .ok_or_else(|| {
            PytxoIpcError::new("run_review_content", "path is not in the prepared package")
        })?;
    let (content_side, digest) = match side {
        "before" => (
            pytxo_runner::ReviewContentSide::Before,
            file.before_sha256.as_ref(),
        ),
        "after" => (
            pytxo_runner::ReviewContentSide::After,
            file.after_sha256.as_ref(),
        ),
        _ => {
            return Err(PytxoIpcError::new(
                "run_review_content",
                "content side must be before or after",
            ))
        }
    };
    let digest = digest.ok_or_else(|| {
        PytxoIpcError::new(
            "run_review_content",
            format!("{side} content does not exist for {path}"),
        )
    })?;
    let chunk = pytxo_runner::read_review_content_chunk(
        &data_dir,
        &manifest,
        path,
        content_side,
        offset,
        limit,
    )
    .map_err(|error| PytxoIpcError::new("run_review_content", error.to_string()))?;
    if chunk.digest != *digest {
        return Err(PytxoIpcError::new(
            "run_review_content",
            "validated content digest does not match the selected manifest side",
        ));
    }
    Ok(PreparedContentChunkDto {
        run_id: run_id.to_owned(),
        package_digest: manifest.package_digest,
        path: path.to_owned(),
        side: side.to_owned(),
        digest: chunk.digest,
        byte_count: chunk.byte_count,
        binary: chunk.binary,
        offset: chunk.offset,
        length: chunk.length,
        next_offset: chunk.next_offset,
        complete: chunk.complete,
        data_base64: base64::engine::general_purpose::STANDARD.encode(chunk.bytes),
    })
}

#[derive(Clone, Serialize)]
pub struct RunApplyAttemptDto {
    pub attempt_id: String,
    pub created_at: Option<String>,
    pub phase: String,
    pub outcome: String,
    pub error_code: Option<String>,
    pub error_message: Option<String>,
    pub rollback_confirmed: bool,
}

#[derive(Deserialize)]
struct ApplyJournalEvidence {
    attempt_id: String,
    created_at: String,
    phase: String,
}

fn load_run_apply_attempts(
    repo: &Path,
    cfg: &PytxoConfig,
    run_id: &str,
    contract: &RunContractRecord,
) -> IpcResult<Vec<RunApplyAttemptDto>> {
    let root = repo.join(&cfg.data_dir).join("apply").join(run_id);
    let mut attempts = Vec::new();
    if root.exists() {
        let entries = std::fs::read_dir(&root).map_err(map_io_err)?;
        for entry in entries {
            let entry = entry.map_err(map_io_err)?;
            if !entry.path().is_dir() {
                continue;
            }
            let fallback_id = entry.file_name().to_string_lossy().into_owned();
            let journal_path = entry.path().join("journal.json");
            let journal = std::fs::read(&journal_path)
                .map_err(map_io_err)
                .and_then(|bytes| {
                    serde_json::from_slice::<ApplyJournalEvidence>(&bytes).map_err(|error| {
                        PytxoIpcError::new(
                            "run_review",
                            format!("invalid Apply journal for {fallback_id}: {error}"),
                        )
                    })
                })?;
            let matching_error = contract
                .last_apply_error
                .as_ref()
                .filter(|error| error.attempt_id.as_deref() == Some(&journal.attempt_id));
            let outcome = match journal.phase.as_str() {
                "committed" => "committed",
                "rolled_back" => "rolled_back",
                _ if contract.recovery_state.as_deref() == Some("unprovable") => {
                    "recovery_required"
                }
                _ => "interrupted",
            };
            let derived_rollback = journal.phase == "rolled_back";
            attempts.push(RunApplyAttemptDto {
                attempt_id: journal.attempt_id,
                created_at: Some(journal.created_at),
                phase: journal.phase,
                outcome: outcome.into(),
                error_code: matching_error
                    .map(|error| error.code.clone())
                    .or_else(|| derived_rollback.then(|| "apply_failed".into())),
                error_message: matching_error
                    .map(|error| error.message.clone())
                    .or_else(|| {
                        derived_rollback.then(|| {
                            "Apply attempt rolled back; the exact earlier error is no longer in the current contract.".into()
                        })
                    }),
                rollback_confirmed: matching_error
                    .map(|error| error.rollback_confirmed)
                    .unwrap_or(derived_rollback),
            });
        }
    }

    if let Some(error) = contract.last_apply_error.as_ref() {
        if let Some(attempt_id) = error.attempt_id.as_ref() {
            if !attempts
                .iter()
                .any(|attempt| &attempt.attempt_id == attempt_id)
            {
                attempts.push(RunApplyAttemptDto {
                    attempt_id: attempt_id.clone(),
                    created_at: Some(error.at.clone()),
                    phase: if error.rollback_confirmed {
                        "rolled_back".into()
                    } else {
                        "unknown".into()
                    },
                    outcome: if error.rollback_confirmed {
                        "rolled_back".into()
                    } else {
                        "recovery_required".into()
                    },
                    error_code: Some(error.code.clone()),
                    error_message: Some(error.message.clone()),
                    rollback_confirmed: error.rollback_confirmed,
                });
            }
        }
    }

    if let Some(manifest_json) = contract.apply_manifest_json.as_deref() {
        let manifest: pytxo_runner::RunApplyManifest = serde_json::from_str(manifest_json)
            .map_err(|error| {
                PytxoIpcError::new("run_review", format!("invalid apply manifest: {error}"))
            })?;
        if !attempts
            .iter()
            .any(|attempt| attempt.attempt_id == manifest.transaction_id)
        {
            attempts.push(RunApplyAttemptDto {
                attempt_id: manifest.transaction_id,
                created_at: contract.applied_at.clone(),
                phase: "committed".into(),
                outcome: "committed".into(),
                error_code: None,
                error_message: None,
                rollback_confirmed: false,
            });
        }
    }
    attempts.sort_by(|left, right| {
        right
            .created_at
            .cmp(&left.created_at)
            .then_with(|| right.attempt_id.cmp(&left.attempt_id))
    });
    Ok(attempts)
}

#[derive(Serialize)]
pub struct DomainChangeDto {
    pub sequence: i64,
    pub entity_kind: String,
    pub entity_id: String,
    pub changed_at: String,
}

#[derive(Serialize)]
pub struct DomainChangesPageDto {
    pub changes: Vec<DomainChangeDto>,
    pub next_cursor: i64,
    pub has_more: bool,
    pub cursor_gap: bool,
}

#[derive(Clone, Serialize)]
pub struct DesktopChangedEvent {
    pub domain_id: String,
    pub entity_kind: String,
    pub entity_id: String,
}

pub(crate) fn emit_domain_changed(
    app: &tauri::AppHandle,
    domain_id: &str,
    entity_kind: &str,
    entity_id: &str,
) {
    let _ = app.emit(
        "pytxo://domain-changed",
        DesktopChangedEvent {
            domain_id: domain_id.to_owned(),
            entity_kind: entity_kind.to_owned(),
            entity_id: entity_id.to_owned(),
        },
    );
}

fn notify_domain_mutation_result<T>(result: IpcResult<T>, notify: impl FnOnce()) -> IpcResult<T> {
    notify();
    result
}

#[derive(Serialize)]
pub struct AgentDto {
    pub id: String,
    pub domain_id: String,
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

/// Remove one stale/temporary domain reference from the global catalog only.
/// Never deletes the repository on disk; refuses domains with active runs or
/// pending approvals (see `pytxo_orchestrate::forget_catalog_domain`).
#[tauri::command]
pub fn forget_domain(domain_id: String) -> IpcResult<()> {
    orch_forget_domain(&domain_id).map_err(map_orch_err)
}

/// Project manifests from `~/.pytxo/projects` for the Desktop Workspace picker.
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

#[derive(Serialize)]
pub struct TrustedDomainDto {
    pub domain_id: String,
    pub permission_profile: String,
    pub trusted_at: String,
    pub label: Option<String>,
}

#[tauri::command]
pub fn list_trusted_domains() -> IpcResult<Vec<TrustedDomainDto>> {
    let store = pytxo_core::TrustedDomainStore::open_default()
        .map_err(|e| PytxoIpcError::from_err("trust", e))?;
    Ok(store
        .list()
        .into_iter()
        .map(|(domain_id, entry)| TrustedDomainDto {
            domain_id,
            permission_profile: entry.permission_profile.as_str().to_string(),
            trusted_at: entry.trusted_at.to_rfc3339(),
            label: entry.label,
        })
        .collect())
}

#[tauri::command]
pub fn get_domain_permission(repo_root: String) -> IpcResult<Option<String>> {
    let profile = trusted_permission_for(Path::new(&repo_root)).map_err(map_orch_err)?;
    Ok(profile.map(|p| p.as_str().to_string()))
}

#[tauri::command]
pub fn set_domain_permission(repo_root: String, profile: String) -> IpcResult<String> {
    let parsed = pytxo_core::PermissionProfile::parse(&profile).ok_or_else(|| {
        PytxoIpcError::new("trust", format!("unknown permission profile: {profile}"))
    })?;
    trust_repo(Path::new(&repo_root), parsed).map_err(map_orch_err)?;
    Ok(parsed.as_str().to_string())
}

#[tauri::command]
pub fn domain_is_trusted(repo_root: String) -> IpcResult<bool> {
    is_repo_trusted(Path::new(&repo_root)).map_err(map_orch_err)
}

/// Canonicalize repo path and register hypervisor domain before topology/dispatch.
#[tauri::command]
pub fn ensure_workspace(state: State<'_, AppState>, domain_id: String) -> IpcResult<String> {
    let repo = PathBuf::from(&domain_id);
    let canonical = repo
        .canonicalize()
        .map_err(map_io_err)?
        .to_string_lossy()
        .into_owned();
    let cfg = load_cfg_for_domain(&canonical, &state)?;
    let domain = default_hypervisor()
        .ensure_domain(Path::new(&canonical), &cfg)
        .map_err(map_orch_err)?;
    let id = domain.repo_root.to_string_lossy().into_owned();
    *state.selected_domain_id.lock().map_err(map_lock_err)? = Some(id.clone());
    Ok(id)
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
    let runs = store.list_runs(limit).map_err(map_store_err)?;
    Ok(runs
        .into_iter()
        .map(|run| {
            let contract = store.get_run_contract(&run.id).ok().flatten();
            run_to_dto(run, &domain, &cfg, contract.as_ref())
        })
        .collect())
}

#[tauri::command]
pub fn run_review(
    state: State<'_, AppState>,
    run_id: String,
    domain_id: Option<String>,
) -> IpcResult<RunReviewDto> {
    let domain = resolve_domain(&state, domain_id)?;
    let cfg = load_cfg_for_domain(&domain, &state)?;
    let store = open_store_for_domain(&cfg, &domain)?;
    let contract = store
        .get_run_contract(&run_id)
        .map_err(map_store_err)?
        .ok_or_else(|| PytxoIpcError::new("run_review", "run has no review/apply contract"))?;
    let parse = |field: &str, value: Option<&str>| -> IpcResult<serde_json::Value> {
        serde_json::from_str(value.unwrap_or("null"))
            .map_err(|error| PytxoIpcError::new("run_review", format!("invalid {field}: {error}")))
    };
    let apply_attempts = load_run_apply_attempts(Path::new(&domain), &cfg, &run_id, &contract)?;
    Ok(RunReviewDto {
        run_id: run_id.clone(),
        base_revision: contract.base_revision,
        apply_status: contract.apply_status,
        applied_at: contract.applied_at,
        plan: parse("execution plan", contract.plan_json.as_deref())?,
        enforcement: parse("enforcement receipt", contract.enforcement_json.as_deref())?,
        apply_manifest: contract
            .apply_manifest_json
            .as_deref()
            .map(|json| parse("apply manifest", Some(json)))
            .transpose()?,
        prepared_manifest: contract.prepared_manifest,
        prepared_digest: contract.prepared_digest,
        prepared_at: contract.prepared_at,
        last_apply_error: contract.last_apply_error,
        recovery_state: contract.recovery_state,
        apply_attempts,
    })
}

#[tauri::command]
pub fn run_review_content(
    state: State<'_, AppState>,
    run_id: String,
    path: String,
    side: String,
    offset: u64,
    limit: usize,
    domain_id: Option<String>,
) -> IpcResult<PreparedContentChunkDto> {
    let domain = resolve_domain(&state, domain_id)?;
    let cfg = load_cfg_for_domain(&domain, &state)?;
    load_review_content_chunk(
        Path::new(&domain),
        &cfg,
        &run_id,
        &path,
        &side,
        offset,
        limit,
    )
}

#[tauri::command]
pub fn domain_changes(
    state: State<'_, AppState>,
    domain_id: String,
    cursor: i64,
    limit: Option<usize>,
) -> IpcResult<DomainChangesPageDto> {
    let cfg = load_cfg_for_domain(&domain_id, &state)?;
    let store = open_store_for_domain(&cfg, &domain_id)?;
    let page = store
        .changes_since(cursor, limit.unwrap_or(200))
        .map_err(map_store_err)?;
    Ok(DomainChangesPageDto {
        changes: page
            .changes
            .into_iter()
            .map(|change| DomainChangeDto {
                sequence: change.sequence,
                entity_kind: change.entity_kind,
                entity_id: change.entity_id,
                changed_at: change.changed_at,
            })
            .collect(),
        next_cursor: page.next_cursor,
        has_more: page.has_more,
        cursor_gap: page.cursor_gap,
    })
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
        .map(|agent| agent_to_dto(agent, &domain))
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
        .map(
            |(agent_id, saved_tokens, edited_paths, fallback_paths)| AgentArbitrageDto {
                agent_id,
                saved_tokens,
                edited_paths,
                fallback_paths,
            },
        )
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
pub async fn dispatch_run_cmd(
    app: tauri::AppHandle,
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
    emit_domain_changed(&app, &domain_id, "run", &run_id);
    Ok(run_id)
}

#[tauri::command]
pub async fn stop_run(
    app: tauri::AppHandle,
    state: State<'_, AppState>,
    all: bool,
    domain_id: Option<String>,
    run_id: Option<String>,
) -> IpcResult<()> {
    let path = state.config_path.lock().map_err(map_lock_err)?.clone();
    let domain = resolve_domain(&state, domain_id)?;
    if let Some(expected_run_id) = run_id {
        if all {
            return Err(PytxoIpcError::new(
                "stop",
                "an exact run stop cannot also target all runs",
            ));
        }
        stop_exact(path, Some(PathBuf::from(&domain)), &expected_run_id, false)
            .await
            .map_err(map_orch_err)?;
        emit_domain_changed(&app, &domain, "run", &expected_run_id);
        return Ok(());
    }
    stop(path, Some(PathBuf::from(&domain)), all, false)
        .await
        .map_err(map_orch_err)?;
    emit_domain_changed(&app, &domain, "run", "*");
    Ok(())
}

#[tauri::command]
pub fn apply_run_changes(
    app: tauri::AppHandle,
    state: State<'_, AppState>,
    run_id: String,
    domain_id: Option<String>,
) -> IpcResult<pytxo_runner::RunApplyManifest> {
    let domain = resolve_domain(&state, domain_id)?;
    let path = state.config_path.lock().map_err(map_lock_err)?.clone();
    let result =
        orch_apply_run_changes(path, Some(PathBuf::from(&domain)), &run_id).map_err(map_orch_err);
    notify_domain_mutation_result(result, || {
        emit_domain_changed(&app, &domain, "contract", &run_id);
    })
}

#[tauri::command]
pub fn refresh_run_review(
    app: tauri::AppHandle,
    state: State<'_, AppState>,
    run_id: String,
    domain_id: Option<String>,
) -> IpcResult<pytxo_core::PreparedRunManifest> {
    let domain = resolve_domain(&state, domain_id)?;
    let path = state.config_path.lock().map_err(map_lock_err)?.clone();
    let manifest = orch_refresh_run_review(path, Some(PathBuf::from(&domain)), &run_id)
        .map_err(map_orch_err)?;
    emit_domain_changed(&app, &domain, "contract", &run_id);
    Ok(manifest)
}

#[tauri::command]
pub fn discard_run_review(
    app: tauri::AppHandle,
    state: State<'_, AppState>,
    run_id: String,
    domain_id: Option<String>,
) -> IpcResult<()> {
    let domain = resolve_domain(&state, domain_id)?;
    let path = state.config_path.lock().map_err(map_lock_err)?.clone();
    orch_discard_run_review(path, Some(PathBuf::from(&domain)), &run_id).map_err(map_orch_err)?;
    emit_domain_changed(&app, &domain, "contract", &run_id);
    Ok(())
}

#[derive(Serialize)]
pub struct RecoveryOutcomeDto {
    pub outcome: String,
    pub attempt_id: Option<String>,
}

fn reconcile_run_recovery_for_domain(
    config: Option<PathBuf>,
    repo: &Path,
    run_id: &str,
) -> IpcResult<RecoveryOutcomeDto> {
    let outcome = orch_reconcile_run_recovery(config, Some(repo.to_path_buf()), run_id)
        .map_err(map_orch_err)?;
    Ok(match outcome {
        RecoveryOutcome::NothingToDo => RecoveryOutcomeDto {
            outcome: "nothing_to_do".into(),
            attempt_id: None,
        },
        RecoveryOutcome::RolledBack { attempt_id } => RecoveryOutcomeDto {
            outcome: "rolled_back".into(),
            attempt_id: Some(attempt_id),
        },
        RecoveryOutcome::Committed(manifest) => RecoveryOutcomeDto {
            outcome: "committed".into(),
            attempt_id: Some(manifest.transaction_id),
        },
        RecoveryOutcome::RecoveryRequired { attempt_id } => RecoveryOutcomeDto {
            outcome: "recovery_required".into(),
            attempt_id,
        },
    })
}

#[tauri::command]
pub fn reconcile_run_recovery(
    app: tauri::AppHandle,
    state: State<'_, AppState>,
    run_id: String,
    domain_id: Option<String>,
) -> IpcResult<RecoveryOutcomeDto> {
    let domain = resolve_domain(&state, domain_id)?;
    let config = state.config_path.lock().map_err(map_lock_err)?.clone();
    let dto = reconcile_run_recovery_for_domain(config, Path::new(&domain), &run_id)?;
    emit_domain_changed(&app, &domain, "contract", &run_id);
    Ok(dto)
}

#[derive(Serialize)]
pub struct HitlDto {
    pub id: String,
    pub agent_key: String,
    pub action: String,
    pub reason: String,
    pub created_at_ms: String,
    pub domain_id: String,
    pub run_id: Option<String>,
    pub agent_id: Option<String>,
}

fn hitl_to_dto(request: pytxo_runner::HitlRequest, domain_id: String) -> HitlDto {
    let (run_id, agent_id) = request
        .agent_key
        .split_once(':')
        .map(|(run, agent)| (Some(run.to_string()), Some(agent.to_string())))
        .unwrap_or((None, None));
    HitlDto {
        id: request.id,
        agent_key: request.agent_key,
        action: request.action,
        reason: request.reason,
        created_at_ms: request.created_at_ms.to_string(),
        domain_id,
        run_id,
        agent_id,
    }
}

#[tauri::command]
pub fn list_hitl(state: State<'_, AppState>, domain_id: Option<String>) -> IpcResult<Vec<HitlDto>> {
    let domain = resolve_domain(&state, domain_id)?;
    let domain_id = domain.clone();
    let pending = orch_list_hitl_pending(Some(PathBuf::from(domain))).map_err(map_orch_err)?;
    Ok(pending
        .into_iter()
        .map(|request| hitl_to_dto(request, domain_id.clone()))
        .collect())
}

#[tauri::command]
pub fn list_hitl_all() -> IpcResult<Vec<HitlDto>> {
    Ok(orch_list_hitl_pending_all()
        .map_err(map_orch_err)?
        .into_iter()
        .map(|row| hitl_to_dto(row.request, row.domain_id))
        .collect())
}

#[tauri::command]
pub fn hitl_respond(
    app: tauri::AppHandle,
    state: State<'_, AppState>,
    request_id: String,
    approve: bool,
    domain_id: Option<String>,
) -> IpcResult<bool> {
    let domain = resolve_domain(&state, domain_id)?;
    let result = orch_hitl_respond(Some(PathBuf::from(&domain)), &request_id, approve)
        .map_err(map_orch_err)?;
    emit_domain_changed(&app, &domain, "approval", &request_id);
    Ok(result)
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

#[derive(Serialize)]
pub struct DesktopSnapshotDto {
    pub domains: Vec<CatalogEntryStatus>,
    pub runs: Vec<RunDto>,
    pub agents: Vec<AgentDto>,
    pub approvals: Vec<HitlDto>,
    pub fleets: Vec<FleetRunDto>,
    pub diagnostics: Vec<DesktopSnapshotDiagnosticDto>,
}

#[derive(Clone, Debug, Serialize)]
pub struct DesktopSnapshotDiagnosticDto {
    pub domain_id: String,
    pub stage: String,
    pub run_id: Option<String>,
    pub message: String,
}

#[derive(Default)]
struct DomainDesktopSnapshot {
    runs: Vec<RunDto>,
    agents: Vec<AgentDto>,
    diagnostics: Vec<DesktopSnapshotDiagnosticDto>,
}

fn snapshot_diagnostic(
    domain_id: &str,
    stage: &str,
    run_id: Option<&str>,
    error: &PytxoIpcError,
) -> DesktopSnapshotDiagnosticDto {
    DesktopSnapshotDiagnosticDto {
        domain_id: domain_id.to_string(),
        stage: stage.to_string(),
        run_id: run_id.map(str::to_string),
        message: error.message.clone(),
    }
}

fn load_domain_desktop_snapshot(
    state: &AppState,
    domain_id: &str,
    run_limit: usize,
    include_agents: bool,
) -> DomainDesktopSnapshot {
    let mut loaded = DomainDesktopSnapshot::default();
    let cfg = match load_cfg_for_domain(domain_id, state) {
        Ok(cfg) => cfg,
        Err(error) => {
            loaded
                .diagnostics
                .push(snapshot_diagnostic(domain_id, "config", None, &error));
            return loaded;
        }
    };
    let store = match open_store_for_domain(&cfg, domain_id) {
        Ok(store) => store,
        Err(error) => {
            loaded
                .diagnostics
                .push(snapshot_diagnostic(domain_id, "store", None, &error));
            return loaded;
        }
    };
    let domain_runs = match store.list_runs(run_limit).map_err(map_store_err) {
        Ok(rows) => rows,
        Err(error) => {
            loaded
                .diagnostics
                .push(snapshot_diagnostic(domain_id, "runs", None, &error));
            return loaded;
        }
    };
    for run in domain_runs {
        let run_id = run.id.clone();
        let contract = match store.get_run_contract(&run_id).map_err(map_store_err) {
            Ok(contract) => contract,
            Err(error) => {
                loaded.diagnostics.push(snapshot_diagnostic(
                    domain_id,
                    "run_contract",
                    Some(&run_id),
                    &error,
                ));
                None
            }
        };
        loaded
            .runs
            .push(run_to_dto(run, domain_id, &cfg, contract.as_ref()));
        if include_agents {
            match store.list_agents_for_run(&run_id).map_err(map_store_err) {
                Ok(rows) => loaded
                    .agents
                    .extend(rows.into_iter().map(|agent| agent_to_dto(agent, domain_id))),
                Err(error) => loaded.diagnostics.push(snapshot_diagnostic(
                    domain_id,
                    "agents",
                    Some(&run_id),
                    &error,
                )),
            }
        }
    }
    loaded
}

/// Single-round-trip snapshot for Desktop 2 polling (avoids N+1 list_runs/list_agents IPC).
#[tauri::command]
pub fn load_desktop_snapshot(
    state: State<'_, AppState>,
    run_limit: Option<usize>,
    fleet_limit: Option<usize>,
    include_agents: Option<bool>,
) -> IpcResult<DesktopSnapshotDto> {
    let run_limit = run_limit.unwrap_or(30);
    let fleet_limit = fleet_limit.unwrap_or(20);
    let include_agents = include_agents.unwrap_or(true);
    let domains = orch_list_domains_status().map_err(map_orch_err)?;
    let mut runs = Vec::new();
    let mut agents = Vec::new();
    let mut diagnostics = Vec::new();
    for domain in &domains {
        let domain_id = domain.domain_id.clone();
        let loaded = load_domain_desktop_snapshot(&state, &domain_id, run_limit, include_agents);
        runs.extend(loaded.runs);
        agents.extend(loaded.agents);
        diagnostics.extend(loaded.diagnostics);
    }
    let approvals = orch_list_hitl_pending_all()
        .map_err(map_orch_err)?
        .into_iter()
        .map(|row| hitl_to_dto(row.request, row.domain_id))
        .collect();
    let fleets = fleet_status(None, fleet_limit)
        .map_err(map_orch_err)?
        .into_iter()
        .map(|r| FleetRunDto {
            id: r.id,
            fleet_id: r.fleet_id,
            started_at: r.started_at,
            finished_at: r.finished_at,
            status: r.status,
        })
        .collect();
    Ok(DesktopSnapshotDto {
        domains,
        runs,
        agents,
        approvals,
        fleets,
        diagnostics,
    })
}

#[tauri::command]
pub fn project_add_root_cmd(
    project_id: String,
    path: String,
    read_only: bool,
) -> IpcResult<Vec<ProjectRootDto>> {
    orch_project_add_root(
        None,
        Some(project_id.clone()),
        PathBuf::from(path),
        read_only,
    )
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
pub fn project_remove_root_cmd(
    project_id: String,
    label: String,
) -> IpcResult<Vec<ProjectRootDto>> {
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

/// IPC schema version for Pytxo Desktop 3D topology consumer.
pub const STRUCTURAL_GRAPH_VERSION: u32 = 3;

#[derive(Serialize)]
pub struct StructuralGraphDto {
    pub version: u32,
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
pub fn workspace_structural_graph(
    state: State<'_, AppState>,
    domain_id: Option<String>,
) -> IpcResult<StructuralGraphDto> {
    let domain = resolve_domain(&state, domain_id)?;
    let graph =
        orch_workspace_structural_graph(Some(PathBuf::from(domain))).map_err(map_orch_err)?;
    Ok(graph_to_dto(graph))
}

fn graph_to_dto(graph: pytxo_signal::StructuralGraph) -> StructuralGraphDto {
    StructuralGraphDto {
        version: STRUCTURAL_GRAPH_VERSION,
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
    }
}

#[tauri::command]
pub fn structural_graph(
    state: State<'_, AppState>,
    run_id: String,
    domain_id: Option<String>,
) -> IpcResult<StructuralGraphDto> {
    let domain = resolve_domain(&state, domain_id)?;
    let graph =
        orch_structural_graph(Some(PathBuf::from(domain)), &run_id).map_err(map_orch_err)?;
    Ok(graph_to_dto(graph))
}

#[tauri::command]
pub fn list_providers() -> IpcResult<Vec<ProviderStatusDto>> {
    Ok(pytxo_core::list_provider_status()
        .into_iter()
        .map(|p| ProviderStatusDto {
            id: p.id,
            name: p.name,
            api_key_env: p.api_key_env,
            key_configured: p.key_configured,
            openai_compatible: p.openai_compatible,
            builtin: p.builtin,
        })
        .collect())
}

#[derive(Serialize)]
pub struct ProviderStatusDto {
    pub id: String,
    pub name: String,
    pub api_key_env: String,
    pub key_configured: bool,
    pub openai_compatible: bool,
    pub builtin: bool,
}

fn domain_to_dto(d: DomainSummary) -> DomainDto {
    DomainDto {
        domain_id: d.domain_id,
        repo_root: d.repo_root,
    }
}

fn run_to_dto(
    r: RunRecord,
    domain_id: &str,
    cfg: &PytxoConfig,
    contract: Option<&RunContractRecord>,
) -> RunDto {
    let (isolation_mode, isolation_backend) = run_isolation_status(
        cfg,
        contract.and_then(|value| value.enforcement_json.as_deref()),
    );
    RunDto {
        id: r.id,
        domain_id: domain_id.to_string(),
        status: r.status,
        repo_root: r.repo_root,
        started_at: r.started_at.to_rfc3339(),
        estimated_cost_usd: r.estimated_cost_usd,
        permission_profile: r.permission_profile,
        isolation_mode,
        isolation_backend,
        apply_status: contract.map(|contract| contract.apply_status.clone()),
        applied_at: contract.and_then(|contract| contract.applied_at.clone()),
        prepared_digest: contract.and_then(|contract| contract.prepared_digest.clone()),
        prepared_at: contract.and_then(|contract| contract.prepared_at.clone()),
        last_apply_error: contract.and_then(|contract| contract.last_apply_error.clone()),
        recovery_state: contract.and_then(|contract| contract.recovery_state.clone()),
    }
}

fn run_isolation_status(cfg: &PytxoConfig, enforcement_json: Option<&str>) -> (String, String) {
    let effective_isolation = pytxo_runner::effective_isolation_mode(cfg);
    let mut isolation_mode = effective_isolation.as_str().to_string();
    let mut isolation_backend =
        isolation_backend_label(effective_isolation, &cfg.blast.sparse_exclude);
    if let Some(mechanism) = enforcement_json
        .and_then(|json| serde_json::from_str::<serde_json::Value>(json).ok())
        .and_then(|value| {
            value
                .pointer("/run/workspace_isolation/mechanism")
                .and_then(serde_json::Value::as_str)
                .map(str::to_string)
        })
        .filter(|mechanism| !mechanism.trim().is_empty())
    {
        isolation_mode = if mechanism.contains("worktree") {
            "worktree".into()
        } else if mechanism.contains("overlay") || mechanism.starts_with("projfs") {
            "overlay".into()
        } else {
            isolation_mode
        };
        isolation_backend = mechanism;
    }
    (isolation_mode, isolation_backend)
}

fn agent_to_dto(a: AgentRecord, domain_id: &str) -> AgentDto {
    AgentDto {
        id: a.id,
        domain_id: domain_id.to_string(),
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

#[cfg(test)]
mod mission_control_contract_tests {
    use super::*;
    use pytxo_core::{
        PreparedRunFile, PreparedRunFileKind, PreparedRunManifest, PreparedRunSummary,
        RunApplyError,
    };
    use pytxo_runner::{prepare_review_package, AgentWorkspaceInput, ApplyFaultPoint};
    use pytxo_store::PytxoStore;

    fn test_app_state() -> AppState {
        AppState {
            config_path: Mutex::new(None),
            poll_cursors: Mutex::new(HashMap::new()),
            selected_domain_id: Mutex::new(None),
            voice_sessions: std::sync::Arc::new(Mutex::new(HashMap::new())),
            voice_captures: std::sync::Arc::new(Mutex::new(HashMap::new())),
            voice_cancellations: Mutex::new(HashMap::new()),
        }
    }

    fn persist_review_contract(
        repo: &Path,
        cfg: &PytxoConfig,
        run_id: &str,
        manifest: &PreparedRunManifest,
    ) {
        let store = PytxoStore::open(&cfg.db_path_at(repo)).unwrap();
        store
            .insert_run_with_profile(run_id, &repo.to_string_lossy(), Some("orbit"))
            .unwrap();
        store.save_run_contract(run_id, "base", "{}", "{}").unwrap();
        assert!(store.begin_run_preparation(run_id).unwrap());
        store.finish_run_preparation(run_id, manifest).unwrap();
    }

    #[test]
    fn run_review_dto_serializes_immutable_manifest_and_recovery_fields() {
        let manifest = PreparedRunManifest {
            version: 2,
            run_id: "run-1".into(),
            base_revision: "base-1".into(),
            prepared_at: "2026-08-01T00:00:00Z".into(),
            package_digest: "digest-1".into(),
            summary: PreparedRunSummary {
                added: 1,
                modified: 0,
                deleted: 0,
                bytes: 3,
            },
            files: vec![PreparedRunFile {
                path: "src/new.rs".into(),
                kind: PreparedRunFileKind::Add,
                before_sha256: None,
                after_sha256: Some("after".into()),
                byte_count: 3,
                task_id: "task".into(),
                agent_id: "agent".into(),
                blob_digest: Some("blob".into()),
                before_mode: None,
                after_mode: None,
                before_byte_count: 0,
                after_byte_count: 3,
                before_is_binary: None,
                after_is_binary: Some(false),
                before_chunks: vec![],
                after_chunks: vec![],
            }],
        };
        let dto = RunReviewDto {
            run_id: "run-1".into(),
            base_revision: Some("base-1".into()),
            apply_status: "ready".into(),
            applied_at: None,
            plan: serde_json::json!({"waves": []}),
            enforcement: serde_json::json!({}),
            apply_manifest: None,
            prepared_manifest: Some(manifest),
            prepared_digest: Some("digest-1".into()),
            prepared_at: Some("2026-08-01T00:00:00Z".into()),
            last_apply_error: None,
            recovery_state: None,
            apply_attempts: vec![],
        };

        let value = serde_json::to_value(dto).unwrap();
        assert_eq!(value["prepared_manifest"]["files"][0]["kind"], "add");
        assert_eq!(value["prepared_digest"], "digest-1");
        assert!(value.get("last_apply_error").is_some());
        assert!(value.get("recovery_state").is_some());
    }

    #[test]
    fn prepared_content_chunks_read_exact_before_and_after_bytes_from_the_package() {
        let temp = tempfile::tempdir().unwrap();
        let repo = temp.path().join("repo");
        std::fs::create_dir_all(&repo).unwrap();
        std::fs::write(repo.join("owned.txt"), b"before\n").unwrap();
        let workspace = temp.path().join("workspace");
        std::fs::create_dir_all(&workspace).unwrap();
        std::fs::write(workspace.join("owned.txt"), b"after\n").unwrap();
        let cfg = PytxoConfig::default();
        let manifest = prepare_review_package(
            &repo,
            &repo.join(&cfg.data_dir),
            "desktop-exact-content",
            "base",
            &[AgentWorkspaceInput {
                agent_id: "agent".into(),
                task_id: "task".into(),
                workspace_path: workspace,
                claims: vec!["owned.txt".into()],
                depends_on: vec![],
            }],
            &[],
        )
        .unwrap();
        persist_review_contract(&repo, &cfg, "desktop-exact-content", &manifest);

        let before = load_review_content_chunk(
            &repo,
            &cfg,
            "desktop-exact-content",
            "owned.txt",
            "before",
            0,
            64,
        )
        .unwrap();
        let after = load_review_content_chunk(
            &repo,
            &cfg,
            "desktop-exact-content",
            "owned.txt",
            "after",
            0,
            64,
        )
        .unwrap();

        assert_eq!(before.data_base64, "YmVmb3JlCg==");
        assert_eq!(after.data_base64, "YWZ0ZXIK");
        assert_eq!(before.run_id, "desktop-exact-content");
        assert_eq!(before.package_digest, manifest.package_digest);
        assert_eq!(before.length, 7);
        assert_eq!(before.next_offset, before.offset + before.length);
        assert!(before.complete && after.complete);
        assert!(!before.binary && !after.binary);
    }

    #[test]
    fn prepared_content_refuses_a_replacement_package_before_contract_settlement() {
        let temp = tempfile::tempdir().unwrap();
        let repo = temp.path().join("repo");
        let workspace = temp.path().join("workspace");
        std::fs::create_dir_all(&repo).unwrap();
        std::fs::create_dir_all(&workspace).unwrap();
        std::fs::write(workspace.join("owned.txt"), b"first package\n").unwrap();
        let cfg = PytxoConfig::default();
        let first = prepare_review_package(
            &repo,
            &repo.join(&cfg.data_dir),
            "desktop-unsettled-replacement",
            "base",
            &[AgentWorkspaceInput {
                agent_id: "agent".into(),
                task_id: "task".into(),
                workspace_path: workspace.clone(),
                claims: vec!["owned.txt".into()],
                depends_on: vec![],
            }],
            &[],
        )
        .unwrap();
        persist_review_contract(&repo, &cfg, "desktop-unsettled-replacement", &first);
        std::fs::write(
            workspace.join("owned.txt"),
            b"replacement before settlement\n",
        )
        .unwrap();
        let replacement = prepare_review_package(
            &repo,
            &repo.join(&cfg.data_dir),
            "desktop-unsettled-replacement",
            "base",
            &[AgentWorkspaceInput {
                agent_id: "agent".into(),
                task_id: "task".into(),
                workspace_path: workspace,
                claims: vec!["owned.txt".into()],
                depends_on: vec![],
            }],
            &[],
        )
        .unwrap();
        assert_ne!(replacement.package_digest, first.package_digest);

        let error = load_review_content_chunk(
            &repo,
            &cfg,
            "desktop-unsettled-replacement",
            "owned.txt",
            "after",
            0,
            64,
        )
        .expect_err("an unsettled package must never be served as the persisted review");
        assert!(error.message.contains("persisted review contract"));
    }

    #[test]
    fn domain_changes_page_preserves_cursor_reset_contract() {
        let dto = DomainChangesPageDto {
            changes: vec![],
            next_cursor: 2,
            has_more: false,
            cursor_gap: true,
        };
        let value = serde_json::to_value(dto).unwrap();
        assert_eq!(value["next_cursor"], 2);
        assert_eq!(value["cursor_gap"], true);
    }

    #[test]
    fn run_isolation_status_prefers_the_persisted_receipt() {
        let cfg = PytxoConfig::default();
        let receipt = serde_json::json!({
            "run": {
                "workspace_isolation": {
                    "mechanism": "projfs-sparse-copy-v2"
                }
            }
        });
        let encoded = serde_json::to_string(&receipt).unwrap();

        let (mode, backend) = run_isolation_status(&cfg, Some(&encoded));

        assert_eq!(mode, "overlay");
        assert_eq!(backend, "projfs-sparse-copy-v2");
    }

    #[test]
    fn two_domain_snapshot_retains_completed_run_agent_ownership() {
        let temp = tempfile::tempdir().unwrap();
        let state = test_app_state();
        let cfg = PytxoConfig::default();
        let mut loaded = Vec::new();

        for (folder, run_id) in [("alpha", "run-alpha"), ("beta", "run-beta")] {
            let repo = temp.path().join(folder);
            std::fs::create_dir_all(&repo).unwrap();
            let store = PytxoStore::open(&cfg.db_path_at(&repo)).unwrap();
            store
                .insert_run_with_profile(run_id, &repo.to_string_lossy(), Some("orbit"))
                .unwrap();
            store
                .insert_agent(
                    &format!("{run_id}:agent"),
                    run_id,
                    "task",
                    0,
                    None,
                    "internal",
                )
                .unwrap();
            store.finish_run(run_id, "completed").unwrap();
            drop(store);

            let domain_id = repo.to_string_lossy().into_owned();
            let snapshot = load_domain_desktop_snapshot(&state, &domain_id, 10, true);
            assert!(snapshot.diagnostics.is_empty());
            loaded.push((domain_id, snapshot));
        }

        for (domain_id, snapshot) in loaded {
            assert_eq!(snapshot.runs.len(), 1);
            assert_eq!(snapshot.agents.len(), 1);
            assert_eq!(snapshot.runs[0].domain_id, domain_id);
            assert_eq!(snapshot.agents[0].domain_id, domain_id);
        }
    }

    #[test]
    fn domain_snapshot_reports_config_and_store_failures_as_partial_diagnostics() {
        let temp = tempfile::tempdir().unwrap();
        let state = test_app_state();

        let invalid_config = temp.path().join("invalid-config");
        std::fs::create_dir_all(&invalid_config).unwrap();
        std::fs::write(invalid_config.join("pytxo.toml"), "not = [valid").unwrap();
        let invalid_id = invalid_config.to_string_lossy().into_owned();
        let config_snapshot = load_domain_desktop_snapshot(&state, &invalid_id, 10, true);
        assert!(config_snapshot.runs.is_empty());
        assert_eq!(config_snapshot.diagnostics.len(), 1);
        assert_eq!(config_snapshot.diagnostics[0].domain_id, invalid_id);
        assert_eq!(config_snapshot.diagnostics[0].stage, "config");

        let invalid_store = temp.path().join("invalid-store");
        let db_path = PytxoConfig::default().db_path_at(&invalid_store);
        std::fs::create_dir_all(&db_path).unwrap();
        let store_id = invalid_store.to_string_lossy().into_owned();
        let store_snapshot = load_domain_desktop_snapshot(&state, &store_id, 10, true);
        assert!(store_snapshot.runs.is_empty());
        assert_eq!(store_snapshot.diagnostics.len(), 1);
        assert_eq!(store_snapshot.diagnostics[0].domain_id, store_id);
        assert_eq!(store_snapshot.diagnostics[0].stage, "store");
    }

    #[test]
    fn failed_mutation_still_notifies_desktop_before_returning_original_error() {
        let notified = std::cell::Cell::new(false);
        let original: IpcResult<()> = Err(PytxoIpcError::new("apply", "stale"));

        let returned = notify_domain_mutation_result(original, || notified.set(true));

        assert!(notified.get());
        let error = returned.unwrap_err();
        assert_eq!(error.code, "apply");
        assert_eq!(error.message, "stale");
    }

    #[test]
    fn native_recovery_adapter_persists_rolled_back_contract() {
        let temp = tempfile::tempdir().unwrap();
        let repo = temp.path().join("repo");
        std::fs::create_dir_all(&repo).unwrap();
        std::fs::write(repo.join("owned.txt"), "before\n").unwrap();
        let workspace = temp.path().join("workspace");
        std::fs::create_dir_all(&workspace).unwrap();
        std::fs::write(workspace.join("owned.txt"), "after\n").unwrap();
        let run_id = "desktop-recovery";
        let data_dir = repo.join(".pytxo/data");
        let manifest = prepare_review_package(
            &repo,
            &data_dir,
            run_id,
            "base",
            &[AgentWorkspaceInput {
                agent_id: "agent".into(),
                task_id: "task".into(),
                workspace_path: workspace,
                claims: vec!["owned.txt".into()],
                depends_on: vec![],
            }],
            &[],
        )
        .unwrap();
        let store = PytxoStore::open(&data_dir.join("pytxo.db")).unwrap();
        store
            .insert_run_with_profile(run_id, &repo.to_string_lossy(), Some("orbit"))
            .unwrap();
        store.save_run_contract(run_id, "base", "{}", "{}").unwrap();
        assert!(store.begin_run_preparation(run_id).unwrap());
        store.finish_run_preparation(run_id, &manifest).unwrap();
        store.finish_run(run_id, "completed").unwrap();
        assert!(store.claim_run_apply(run_id).unwrap());
        pytxo_runner::apply_prepared_review_with_fault(
            &repo,
            &data_dir,
            &manifest,
            Some(ApplyFaultPoint::InterruptAfterRename(1)),
        )
        .expect_err("interrupt");
        store
            .finish_run_apply_error(
                run_id,
                "recovery_required",
                &RunApplyError {
                    at: "2026-08-01T00:00:00Z".into(),
                    code: "apply_recovery_required".into(),
                    message: "reconciliation required".into(),
                    attempt_id: None,
                    rollback_confirmed: false,
                },
                Some("unprovable"),
            )
            .unwrap();
        drop(store);

        let dto = reconcile_run_recovery_for_domain(None, &repo, run_id).unwrap();

        assert_eq!(dto.outcome, "rolled_back");
        let contract = PytxoStore::open(&data_dir.join("pytxo.db"))
            .unwrap()
            .get_run_contract(run_id)
            .unwrap()
            .unwrap();
        assert_eq!(contract.apply_status, "ready");
        assert_eq!(contract.recovery_state.as_deref(), Some("rolled_back"));

        let store = PytxoStore::open(&data_dir.join("pytxo.db")).unwrap();
        assert!(store.claim_run_apply(run_id).unwrap());
        pytxo_runner::apply_prepared_review_with_fault(
            &repo,
            &data_dir,
            &manifest,
            Some(ApplyFaultPoint::InterruptAfterJournalPrepared),
        )
        .expect_err("second interrupt");
        store
            .finish_run_apply_error(
                run_id,
                "recovery_required",
                &RunApplyError {
                    at: "2026-08-01T00:05:00Z".into(),
                    code: "apply_recovery_required".into(),
                    message: "second reconciliation required".into(),
                    attempt_id: None,
                    rollback_confirmed: false,
                },
                Some("unprovable"),
            )
            .unwrap();
        drop(store);
        let second = reconcile_run_recovery_for_domain(None, &repo, run_id).unwrap();
        assert_eq!(second.outcome, "rolled_back");
        let store = PytxoStore::open(&data_dir.join("pytxo.db")).unwrap();
        let contract = store.get_run_contract(run_id).unwrap().unwrap();
        let attempts =
            load_run_apply_attempts(&repo, &PytxoConfig::default(), run_id, &contract).unwrap();
        assert_eq!(attempts.len(), 2);
        assert!(attempts
            .iter()
            .all(|attempt| attempt.outcome == "rolled_back" && attempt.rollback_confirmed));
    }

    #[test]
    fn native_recovery_adapter_persists_committed_audit() {
        let temp = tempfile::tempdir().unwrap();
        let repo = temp.path().join("repo");
        std::fs::create_dir_all(&repo).unwrap();
        std::fs::write(repo.join("owned.txt"), "before\n").unwrap();
        let workspace = temp.path().join("workspace");
        std::fs::create_dir_all(&workspace).unwrap();
        std::fs::write(workspace.join("owned.txt"), "after\n").unwrap();
        let run_id = "desktop-committed-recovery";
        let data_dir = repo.join(".pytxo/data");
        let manifest = prepare_review_package(
            &repo,
            &data_dir,
            run_id,
            "base",
            &[AgentWorkspaceInput {
                agent_id: "agent".into(),
                task_id: "task".into(),
                workspace_path: workspace,
                claims: vec!["owned.txt".into()],
                depends_on: vec![],
            }],
            &[],
        )
        .unwrap();
        let store = PytxoStore::open(&data_dir.join("pytxo.db")).unwrap();
        store
            .insert_run_with_profile(run_id, &repo.to_string_lossy(), Some("orbit"))
            .unwrap();
        store.save_run_contract(run_id, "base", "{}", "{}").unwrap();
        assert!(store.begin_run_preparation(run_id).unwrap());
        store.finish_run_preparation(run_id, &manifest).unwrap();
        store.finish_run(run_id, "completed").unwrap();
        assert!(store.claim_run_apply(run_id).unwrap());
        let applied = pytxo_runner::apply_prepared_review(&repo, &data_dir, &manifest).unwrap();
        store
            .finish_run_apply_error(
                run_id,
                "recovery_required",
                &RunApplyError {
                    at: "2026-08-01T00:00:00Z".into(),
                    code: "apply_recovery_required".into(),
                    message: "commit outcome was not persisted".into(),
                    attempt_id: Some(applied.transaction_id.clone()),
                    rollback_confirmed: false,
                },
                Some("unprovable"),
            )
            .unwrap();
        drop(store);

        let dto = reconcile_run_recovery_for_domain(None, &repo, run_id).unwrap();

        assert_eq!(dto.outcome, "committed");
        assert_eq!(
            dto.attempt_id.as_deref(),
            Some(applied.transaction_id.as_str())
        );
        let store = PytxoStore::open(&data_dir.join("pytxo.db")).unwrap();
        let contract = store.get_run_contract(run_id).unwrap().unwrap();
        assert_eq!(contract.apply_status, "applied");
        assert!(contract.applied_at.is_some());
        assert!(contract.last_apply_error.is_none());
        assert!(contract.recovery_state.is_none());
        let attempts =
            load_run_apply_attempts(&repo, &PytxoConfig::default(), run_id, &contract).unwrap();
        assert_eq!(attempts.len(), 1);
        assert_eq!(attempts[0].outcome, "committed");
    }
}
