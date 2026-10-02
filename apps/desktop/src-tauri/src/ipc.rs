use std::collections::{HashMap, HashSet};
use std::fs::Metadata;
use std::path::{Path, PathBuf};
use std::sync::Mutex;
use std::time::SystemTime;

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
use tauri::{Emitter, Manager, State};

use crate::ipc_error::{
    map_config_err, map_io_err, map_lock_err, map_orch_err, map_store_err, IpcResult, PytxoIpcError,
};

pub struct AppState {
    pub config_path: Mutex<Option<PathBuf>>,
    /// Per (domain_id, agent_id) cursor for incremental log polling.
    pub poll_cursors: Mutex<HashMap<(String, String), i64>>,
    /// Last fully consumed change-log boundary for each observed store.
    ///
    /// This cache holds file identity only, never SQLite handles. That keeps
    /// Windows recovery free to replace a store while avoiding a read-only
    /// open for every unchanged domain on each Desktop poll.
    pub(crate) domain_change_observations: Mutex<HashMap<PathBuf, DomainChangeObservation>>,
    /// Resolved store paths from the latest successful snapshot.
    ///
    /// Domain polling uses these paths directly instead of re-reading up to one
    /// repository config per domain on every idle tick. The snapshot remains
    /// the authority and refreshes this cache when configuration changes.
    pub(crate) domain_store_paths: Mutex<HashMap<String, PathBuf>>,
    pub selected_domain_id: Mutex<Option<String>>,
    pub voice_sessions: std::sync::Arc<Mutex<HashMap<uuid::Uuid, pytxo_voice::VoiceSession>>>,
    pub voice_captures: std::sync::Arc<Mutex<HashMap<uuid::Uuid, pytxo_voice::CpalCapture>>>,
    pub voice_cancellations:
        Mutex<HashMap<uuid::Uuid, std::sync::Arc<std::sync::atomic::AtomicBool>>>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct DomainChangeObservation {
    fingerprint: StoreFingerprint,
    settled_cursor: i64,
}

#[derive(Clone, Debug, PartialEq, Eq)]
struct StoreFingerprint {
    database: Option<FileFingerprint>,
    wal: Option<FileFingerprint>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
struct FileFingerprint {
    len: u64,
    modified: Option<SystemTime>,
    created: Option<SystemTime>,
    #[cfg(windows)]
    windows_creation_time: u64,
    #[cfg(windows)]
    windows_last_write_time: u64,
    #[cfg(unix)]
    device: u64,
    #[cfg(unix)]
    inode: u64,
}

impl FileFingerprint {
    fn from_metadata(metadata: &Metadata) -> Self {
        #[cfg(windows)]
        {
            use std::os::windows::fs::MetadataExt;
            Self {
                len: metadata.len(),
                modified: metadata.modified().ok(),
                created: metadata.created().ok(),
                windows_creation_time: metadata.creation_time(),
                windows_last_write_time: metadata.last_write_time(),
            }
        }
        #[cfg(unix)]
        {
            use std::os::unix::fs::MetadataExt;
            Self {
                len: metadata.len(),
                modified: metadata.modified().ok(),
                created: metadata.created().ok(),
                device: metadata.dev(),
                inode: metadata.ino(),
            }
        }
        #[cfg(not(any(windows, unix)))]
        {
            Self {
                len: metadata.len(),
                modified: metadata.modified().ok(),
                created: metadata.created().ok(),
            }
        }
    }
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
    /// String preserves the SQLite u64 revision across JavaScript's number limit.
    pub routing_revision: Option<String>,
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
    /// Registry identity from the exact saved launch command, not a live session probe.
    pub launcher: Option<AgentLauncherDto>,
    pub workspace_path: Option<String>,
}

#[derive(Serialize)]
pub struct AgentLauncherDto {
    pub id: &'static str,
    pub display_name: &'static str,
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

fn remember_domain_store_path(state: &AppState, domain_id: &str, path: PathBuf) -> IpcResult<()> {
    let previous = state
        .domain_store_paths
        .lock()
        .map_err(map_lock_err)?
        .insert(domain_id.to_string(), path.clone());
    if let Some(previous) = previous.filter(|previous| previous != &path) {
        state
            .domain_change_observations
            .lock()
            .map_err(map_lock_err)?
            .remove(&previous);
    }
    Ok(())
}

fn forget_domain_store_path(state: &AppState, domain_id: &str) -> IpcResult<()> {
    let previous = state
        .domain_store_paths
        .lock()
        .map_err(map_lock_err)?
        .remove(domain_id);
    if let Some(path) = previous {
        state
            .domain_change_observations
            .lock()
            .map_err(map_lock_err)?
            .remove(&path);
    }
    Ok(())
}

fn resolved_domain_store_path(state: &AppState, domain_id: &str) -> IpcResult<PathBuf> {
    if let Some(path) = state
        .domain_store_paths
        .lock()
        .map_err(map_lock_err)?
        .get(domain_id)
        .cloned()
    {
        return Ok(path);
    }
    let cfg = load_cfg_for_domain(domain_id, state)?;
    let path = cfg.db_path_at(Path::new(domain_id));
    remember_domain_store_path(state, domain_id, path.clone())?;
    Ok(path)
}

/// Lightweight catalog identity used by Desktop to discover externally added
/// or removed workspaces without reopening every domain store on a timer.
#[tauri::command]
pub async fn catalog_fingerprint() -> IpcResult<String> {
    let path = pytxo_store::default_catalog_path()
        .ok_or_else(|| PytxoIpcError::new("catalog", "Pytxo home directory is unavailable"))?;
    tauri::async_runtime::spawn_blocking(move || {
        store_fingerprint(&path).map(|fingerprint| format!("{fingerprint:?}"))
    })
    .await
    .map_err(map_orch_err)?
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
    let canonical = pytxo_core::canonical_repo_root(&repo)
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
    runs.into_iter()
        .map(|run| {
            let contract = store.get_run_contract(&run.id).ok().flatten();
            let revision = routing_revision_for_run(&store, &domain, &run.id)?;
            Ok(run_to_dto(run, &domain, &cfg, contract.as_ref(), revision))
        })
        .collect::<IpcResult<Vec<_>>>()
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
pub async fn domain_changes(
    state: State<'_, AppState>,
    domain_id: String,
    cursor: i64,
    limit: Option<usize>,
) -> IpcResult<DomainChangesPageDto> {
    // Observation must neither initialize/migrate a store on every poll nor
    // block the native event loop on filesystem/SQLite work. A missing or
    // incompatible store remains an error; only explicit write paths create it.
    let path = resolved_domain_store_path(&state, &domain_id)?;
    let page = tauri::async_runtime::spawn_blocking(move || {
        let store = pytxo_store::PytxoStore::open_existing_read_only(&path)?;
        store.changes_since(cursor, limit.unwrap_or(200))
    })
    .await
    .map_err(map_orch_err)?
    .map_err(map_store_err)?;
    Ok(domain_changes_dto(page))
}

fn domain_changes_dto(page: pytxo_store::DomainChangesPage) -> DomainChangesPageDto {
    DomainChangesPageDto {
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
    }
}

#[derive(Deserialize)]
pub struct DomainChangesRequest {
    pub domain_id: String,
    pub cursor: i64,
}

#[tauri::command]
pub async fn routing_run_summary(
    state: State<'_, AppState>,
    run_id: String,
    domain_id: String,
) -> IpcResult<Option<pytxo_store::routing::RoutingDisplaySummary>> {
    let path = resolved_domain_store_path(&state, &domain_id)?;
    tauri::async_runtime::spawn_blocking(move || {
        let store =
            pytxo_store::PytxoStore::open_existing_read_only(&path).map_err(map_store_err)?;
        if store.get_run(&run_id).map_err(map_store_err)?.is_none() {
            return Err(PytxoIpcError::new(
                "routing_run_summary",
                "run unavailable in execution domain",
            ));
        }
        let scope = pytxo_store::routing::RoutingScope {
            domain_id: pytxo_core::DomainId(domain_id),
            run_id: pytxo_core::RunId(run_id),
        };
        store.routing_display_summary(&scope).map_err(map_store_err)
    })
    .await
    .map_err(map_orch_err)?
}

fn sqlite_sidecar_path(path: &Path, suffix: &str) -> PathBuf {
    let mut value = path.as_os_str().to_os_string();
    value.push(suffix);
    PathBuf::from(value)
}

fn optional_file_fingerprint(path: &Path) -> IpcResult<Option<FileFingerprint>> {
    match std::fs::metadata(path) {
        Ok(metadata) => Ok(Some(FileFingerprint::from_metadata(&metadata))),
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(None),
        Err(error) => Err(map_io_err(error)),
    }
}

fn store_fingerprint(path: &Path) -> IpcResult<StoreFingerprint> {
    Ok(StoreFingerprint {
        database: optional_file_fingerprint(path)?,
        wal: optional_file_fingerprint(&sqlite_sidecar_path(path, "-wal"))?,
    })
}

fn unchanged_domain_changes(cursor: i64) -> DomainChangesPageDto {
    DomainChangesPageDto {
        changes: Vec::new(),
        next_cursor: cursor,
        has_more: false,
        cursor_gap: false,
    }
}

fn observed_domain_changes(
    state: &AppState,
    path: &Path,
    cursor: i64,
    limit: usize,
) -> IpcResult<DomainChangesPageDto> {
    // Capture the identity before reading. If a writer commits during the
    // query, the next poll sees a different fingerprint and catches up.
    let fingerprint = store_fingerprint(path)?;
    let unchanged = state
        .domain_change_observations
        .lock()
        .map_err(map_lock_err)?
        .get(path)
        .is_some_and(|observation| {
            observation.fingerprint == fingerprint && observation.settled_cursor == cursor
        });
    if unchanged {
        return Ok(unchanged_domain_changes(cursor));
    }

    let page = {
        // Keep the handle inside this scope. Windows recovery must be able to
        // rename or replace the database immediately after observation.
        let store =
            pytxo_store::PytxoStore::open_existing_read_only(path).map_err(map_store_err)?;
        store.changes_since(cursor, limit).map_err(map_store_err)?
    };
    let dto = domain_changes_dto(page);
    let mut observations = state
        .domain_change_observations
        .lock()
        .map_err(map_lock_err)?;
    if dto.has_more {
        observations.remove(path);
    } else {
        observations.insert(
            path.to_path_buf(),
            DomainChangeObservation {
                fingerprint,
                settled_cursor: dto.next_cursor,
            },
        );
    }
    Ok(dto)
}

fn domain_changes_batch_blocking(
    state: &AppState,
    requests: Vec<DomainChangesRequest>,
    limit: usize,
) -> IpcResult<Vec<DomainChangesPageDto>> {
    if requests.len() > 128 {
        return Err(PytxoIpcError::new(
            "input",
            "At most 128 domains per change batch",
        ));
    }
    requests
        .into_iter()
        .map(|request| {
            let path = resolved_domain_store_path(state, &request.domain_id)?;
            let result = observed_domain_changes(state, &path, request.cursor, limit);
            if result.is_err() {
                // Errors are never remembered as settled observations. A repaired,
                // restored, or newly created store must be retried on the next poll.
                state
                    .domain_change_observations
                    .lock()
                    .map_err(map_lock_err)?
                    .remove(&path);
            }
            result
        })
        .collect()
}

#[tauri::command]
pub async fn domain_changes_batch(
    app: tauri::AppHandle,
    requests: Vec<DomainChangesRequest>,
    limit: Option<usize>,
) -> IpcResult<Vec<DomainChangesPageDto>> {
    tauri::async_runtime::spawn_blocking(move || {
        domain_changes_batch_blocking(&app.state::<AppState>(), requests, limit.unwrap_or(200))
    })
    .await
    .map_err(map_orch_err)?
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

/// Read-only observer API. The caller owns its cursor; another view cannot
/// consume it. Require exact run/agent membership inside the requested domain.
#[tauri::command]
pub fn read_agent_events(
    state: State<'_, AppState>,
    run_id: String,
    agent_id: String,
    domain_id: String,
    after: i64,
    limit: usize,
) -> IpcResult<Vec<EventDto>> {
    let domain = resolve_domain(&state, Some(domain_id))?;
    let cfg = load_cfg_for_domain(&domain, &state)?;
    let store = open_store_for_domain(&cfg, &domain)?;
    read_scoped_agent_events(&store, &run_id, &agent_id, after, limit)
}

fn read_scoped_agent_events(
    store: &pytxo_store::PytxoStore,
    run_id: &str,
    agent_id: &str,
    after: i64,
    limit: usize,
) -> IpcResult<Vec<EventDto>> {
    let agent = store
        .get_agent(agent_id)
        .map_err(map_store_err)?
        .ok_or_else(|| PytxoIpcError::new("missing_agent", "No recorded agent in this domain"))?;
    if agent.run_id != run_id {
        return Err(PytxoIpcError::new(
            "scope_mismatch",
            "Agent does not belong to the requested run",
        ));
    }
    store
        .tail_events_after(agent_id, after.max(0), limit.clamp(1, 200))
        .map_err(map_store_err)
        .map(|events| events.into_iter().map(event_to_dto).collect())
}

/// The last error a worker printed, reduced to one plain sentence so a failed
/// run can say why without opening raw output. Advisory only: the recorded exit
/// status stays authoritative, and payloads were already sanitized at capture.
#[tauri::command]
pub fn agent_failure_hint(
    state: State<'_, AppState>,
    run_id: String,
    agent_id: String,
    domain_id: String,
) -> IpcResult<Option<String>> {
    let domain = resolve_domain(&state, Some(domain_id))?;
    let cfg = load_cfg_for_domain(&domain, &state)?;
    let store = open_store_for_domain(&cfg, &domain)?;
    let agent = store
        .get_agent(&agent_id)
        .map_err(map_store_err)?
        .ok_or_else(|| PytxoIpcError::new("missing_agent", "No recorded agent in this domain"))?;
    if agent.run_id != run_id {
        return Err(PytxoIpcError::new(
            "scope_mismatch",
            "Agent does not belong to the requested run",
        ));
    }
    let events = store.list_events(&agent_id, 200).map_err(map_store_err)?;
    Ok(failure_hint(
        events
            .iter()
            .filter(|event| event.kind == "stdout" || event.kind == "stderr")
            .map(|event| event.payload.as_str()),
    ))
}

const FAILURE_HINT_MAX_CHARS: usize = 240;

fn failure_hint<'a>(payloads: impl Iterator<Item = &'a str>) -> Option<String> {
    // Events are lines; ConPTY wraps continue across them (see join_output_lines).
    let text =
        pytxo_orchestrate::strip_terminal_text(&pytxo_orchestrate::join_output_lines(payloads));
    // Prefer a structured provider message such as {"error":{"message":"…"}}.
    let structured = text.rfind("\"message\":\"").and_then(|at| {
        let rest = &text[at + "\"message\":\"".len()..];
        let mut escaped = false;
        rest.char_indices()
            .find(|&(_, c)| {
                let end = c == '"' && !escaped;
                escaped = c == '\\' && !escaped;
                end
            })
            .map(|(end, _)| rest[..end].replace("\\\"", "\""))
    });
    let hint = structured.or_else(|| {
        let at = text.to_ascii_lowercase().rfind("error")?;
        // Read past the limit so the truncation below can mark the cut.
        Some(
            text[at..]
                .chars()
                .take(FAILURE_HINT_MAX_CHARS * 2)
                .collect(),
        )
    })?;
    let mut collapsed = hint.split_whitespace().collect::<Vec<_>>().join(" ");
    if collapsed.chars().count() > FAILURE_HINT_MAX_CHARS {
        collapsed = collapsed
            .chars()
            .take(FAILURE_HINT_MAX_CHARS - 1)
            .collect::<String>()
            + "…";
    }
    (!collapsed.is_empty()).then_some(collapsed)
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
    // The old Deck is a development-only rollback surface. Its arbitrary-command
    // entry point must not bypass reviewed Flow admission in packaged Desktop.
    if !cfg!(debug_assertions) {
        return Err(PytxoIpcError::new(
            "unsupported",
            "Direct command runs are unavailable in this Desktop build. Use Work → New work to build and review a beta plan.",
        ));
    }
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
    expected_package_digest: String,
) -> IpcResult<pytxo_runner::RunApplyManifest> {
    let domain = resolve_domain(&state, domain_id)?;
    let path = state.config_path.lock().map_err(map_lock_err)?.clone();
    let result = orch_apply_run_changes(
        path,
        Some(PathBuf::from(&domain)),
        &run_id,
        &expected_package_digest,
    )
    .map_err(|error| {
        map_review_apply_error(&error, error.downcast_ref::<pytxo_core::PytxoError>())
    });
    notify_domain_mutation_result(result, || {
        emit_domain_changed(&app, &domain, "contract", &run_id);
    })
}

fn map_review_apply_error(
    error: impl ToString,
    cause: Option<&pytxo_core::PytxoError>,
) -> PytxoIpcError {
    if matches!(cause, Some(pytxo_core::PytxoError::StaleReview)) {
        PytxoIpcError::new("stale_review", error.to_string())
    } else {
        map_orch_err(error)
    }
}

#[tauri::command]
pub async fn refresh_run_review(
    app: tauri::AppHandle,
    state: State<'_, AppState>,
    run_id: String,
    domain_id: Option<String>,
) -> IpcResult<pytxo_core::PreparedRunManifest> {
    let domain = resolve_domain(&state, domain_id)?;
    let path = state.config_path.lock().map_err(map_lock_err)?.clone();
    let refresh_domain = PathBuf::from(&domain);
    let refresh_run_id = run_id.clone();
    let manifest = tauri::async_runtime::spawn_blocking(move || {
        orch_refresh_run_review(path, Some(refresh_domain), &refresh_run_id)
    })
    .await
    .map_err(map_orch_err)?
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
    let output = pytxo_core::background_command("git")
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
            let _ = forget_domain_store_path(state, domain_id);
            loaded
                .diagnostics
                .push(snapshot_diagnostic(domain_id, "config", None, &error));
            return loaded;
        }
    };
    let store_path = cfg.db_path_at(Path::new(domain_id));
    if let Err(error) = remember_domain_store_path(state, domain_id, store_path.clone()) {
        loaded
            .diagnostics
            .push(snapshot_diagnostic(domain_id, "store_path", None, &error));
        return loaded;
    }
    let store = match pytxo_store::PytxoStore::open(&store_path).map_err(map_store_err) {
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
        let routing_revision = match routing_revision_for_run(&store, domain_id, &run_id) {
            Ok(revision) => revision,
            Err(error) => {
                loaded.diagnostics.push(snapshot_diagnostic(
                    domain_id,
                    "routing_revision",
                    Some(&run_id),
                    &error,
                ));
                None
            }
        };
        loaded.runs.push(run_to_dto(
            run,
            domain_id,
            &cfg,
            contract.as_ref(),
            routing_revision,
        ));
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
pub async fn load_desktop_snapshot(
    app: tauri::AppHandle,
    run_limit: Option<usize>,
    fleet_limit: Option<usize>,
    include_agents: Option<bool>,
) -> IpcResult<DesktopSnapshotDto> {
    tauri::async_runtime::spawn_blocking(move || {
        load_desktop_snapshot_blocking(
            &app.state::<AppState>(),
            run_limit,
            fleet_limit,
            include_agents,
        )
    })
    .await
    .map_err(map_orch_err)?
}

fn load_desktop_snapshot_blocking(
    state: &AppState,
    run_limit: Option<usize>,
    fleet_limit: Option<usize>,
    include_agents: Option<bool>,
) -> IpcResult<DesktopSnapshotDto> {
    let run_limit = run_limit.unwrap_or(30);
    let fleet_limit = fleet_limit.unwrap_or(20);
    let include_agents = include_agents.unwrap_or(true);
    let domains = orch_list_domains_status().map_err(map_orch_err)?;
    let domain_ids = domains
        .iter()
        .map(|domain| domain.domain_id.as_str())
        .collect::<HashSet<_>>();
    state
        .domain_store_paths
        .lock()
        .map_err(map_lock_err)?
        .retain(|domain_id, _| domain_ids.contains(domain_id.as_str()));
    let mut runs = Vec::new();
    let mut agents = Vec::new();
    let mut diagnostics = Vec::new();
    for domain in &domains {
        let domain_id = domain.domain_id.clone();
        let loaded = load_domain_desktop_snapshot(state, &domain_id, run_limit, include_agents);
        runs.extend(loaded.runs);
        agents.extend(loaded.agents);
        diagnostics.extend(loaded.diagnostics);
    }
    let active_store_paths = state
        .domain_store_paths
        .lock()
        .map_err(map_lock_err)?
        .values()
        .cloned()
        .collect::<HashSet<_>>();
    state
        .domain_change_observations
        .lock()
        .map_err(map_lock_err)?
        .retain(|path, _| active_store_paths.contains(path));
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
pub fn project_create_cmd(
    state: State<'_, AppState>,
    domain_id: String,
    path: String,
) -> IpcResult<ProjectDto> {
    // Metadata grouping only: preserve the primary execution domain and all trust profiles.
    let catalog = pytxo_store::Catalog::open_default().map_err(map_store_err)?;
    if orch_list_catalog_domains()
        .map_err(map_orch_err)?
        .iter()
        .any(|entry| entry.domain_id == domain_id && entry.project_id.is_some())
    {
        return Err(PytxoIpcError::new("project_exists", "This workspace already has a project. Reopen workspace settings to refresh its folders."));
    }
    let manifest = crate::workspace_project::new_manifest(
        format!("workspace-{}", uuid::Uuid::new_v4()),
        Path::new(&domain_id),
        Path::new(&path),
    )?;
    let manifest_path = pytxo_core::ProjectManifest::user_manifest_path(&manifest.project.id)
        .ok_or_else(|| {
            PytxoIpcError::new("project_home", "Cannot resolve the project storage folder.")
        })?;
    let cfg = load_cfg_for_domain(&domain_id, &state)?;
    let primary = &manifest.roots[0].path;
    let domain = default_hypervisor()
        .ensure_domain(primary, &cfg)
        .map_err(map_orch_err)?;
    crate::workspace_project::write_new(&manifest_path, &manifest)?;
    catalog
        .upsert_domain(
            domain.id.as_str(),
            &primary.to_string_lossy(),
            &cfg.db_path_at(primary).to_string_lossy(),
            Some(&manifest.project.id),
        )
        .map_err(map_store_err)?;
    Ok(ProjectDto {
        id: manifest.project.id,
        manifest_path: manifest_path.to_string_lossy().into_owned(),
    })
}

#[tauri::command]
pub fn project_add_root_cmd(
    project_id: String,
    path: String,
    read_only: bool,
) -> IpcResult<Vec<ProjectRootDto>> {
    let existing = orch_project_roots(None, Some(project_id.clone())).map_err(map_orch_err)?;
    let paths: Vec<PathBuf> = existing.iter().map(|root| PathBuf::from(&root.1)).collect();
    let path = crate::workspace_project::checked_folder(Path::new(&path), &paths)?;
    crate::workspace_project::check_label(
        &path,
        &existing
            .iter()
            .map(|root| root.0.clone())
            .collect::<Vec<_>>(),
    )?;
    orch_project_add_root(None, Some(project_id.clone()), path, read_only).map_err(map_orch_err)?;
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
    routing_revision: Option<String>,
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
        routing_revision,
    }
}

fn routing_revision_for_run(
    store: &pytxo_store::PytxoStore,
    domain_id: &str,
    run_id: &str,
) -> IpcResult<Option<String>> {
    let scope = pytxo_store::routing::RoutingScope {
        domain_id: pytxo_core::DomainId(domain_id.to_string()),
        run_id: pytxo_core::RunId(run_id.to_string()),
    };
    store
        .routing_revision(&scope)
        .map(|revision| revision.map(|value| value.to_string()))
        .map_err(map_store_err)
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
    // Do not serialize argv: custom commands may contain prompts or credentials.
    // Aliases, wrappers and modified commands remain unknown rather than guessed.
    let launcher = pytxo_core::all_ade_clis()
        .iter()
        .find(|spec| spec.default_cmd == a.cmd.trim())
        .map(|spec| AgentLauncherDto {
            id: spec.id,
            display_name: spec.display_name,
        });
    AgentDto {
        id: a.id,
        domain_id: domain_id.to_string(),
        run_id: a.run_id,
        task_id: a.task_id,
        wave: a.wave,
        status: a.status,
        exit_code: a.exit_code,
        root_id: a.root_id,
        launcher,
        workspace_path: a.worktree_path,
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
    #[test]
    fn stale_review_is_a_structured_ipc_refusal() {
        let cause = pytxo_core::PytxoError::StaleReview;
        let error = super::map_review_apply_error(&cause, Some(&cause));
        let json = serde_json::to_value(error).unwrap();
        assert_eq!(json["code"], "stale_review");
        assert!(json["message"]
            .as_str()
            .unwrap()
            .contains("Reload and review"));
        assert_eq!(
            super::map_review_apply_error("disk unavailable", None).code,
            "orchestrate"
        );
    }
    use super::*;
    #[test]
    fn agent_identity_uses_saved_launcher_without_exposing_arguments() {
        let record = AgentRecord {
            id: "run:worker".into(),
            run_id: "run".into(),
            task_id: "task".into(),
            wave: 0,
            worktree_path: Some("C:/isolated/worker".into()),
            cmd: "codex exec --sandbox workspace-write".into(),
            exit_code: None,
            status: "running".into(),
            root_id: Some("api".into()),
        };
        let dto = agent_to_dto(record.clone(), "domain-a");
        assert_eq!(dto.launcher.unwrap().id, "codex");
        assert_eq!(dto.workspace_path.as_deref(), Some("C:/isolated/worker"));
        assert_eq!(dto.domain_id, "domain-a");
        for command in [
            "",
            "echo codex",
            "codex exec --token PRIVATE_SENTINEL",
            "wrapper codex exec --sandbox workspace-write",
        ] {
            let mut custom = record.clone();
            custom.cmd = command.into();
            let dto = agent_to_dto(custom, "domain-b");
            assert!(dto.launcher.is_none());
            let serialized = serde_json::to_string(&dto).unwrap();
            assert!(!serialized.contains("PRIVATE_SENTINEL"));
            assert!(!serialized.contains("\"cmd\""));
        }
    }
    #[test]
    fn dock_observers_have_independent_cursors_and_enforce_run_membership() {
        let dir = tempfile::tempdir().unwrap();
        let store = pytxo_store::PytxoStore::open(&dir.path().join("dock.db")).unwrap();
        store.insert_run("run-a", "/repo").unwrap();
        store.insert_run("run-b", "/repo").unwrap();
        store
            .insert_agent("agent-a", "run-a", "task", 0, None, "echo hi")
            .unwrap();
        store.append_event("agent-a", "stdout", "first").unwrap();
        store.append_event("agent-a", "stdout", "second").unwrap();
        let first = read_scoped_agent_events(&store, "run-a", "agent-a", 0, 1).unwrap();
        let other = read_scoped_agent_events(&store, "run-a", "agent-a", 0, 200).unwrap();
        assert_eq!(first.len(), 1);
        assert_eq!(other.len(), 2);
        assert_eq!(first[0].id, other[0].id);
        let next = read_scoped_agent_events(&store, "run-a", "agent-a", first[0].id, 200).unwrap();
        assert_eq!(next.len(), 1);
        assert_eq!(next[0].payload, "second");
        assert!(read_scoped_agent_events(&store, "run-b", "agent-a", 0, 200).is_err());
        assert!(read_scoped_agent_events(&store, "run-a", "missing", 0, 200).is_err());
    }
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
            domain_change_observations: Mutex::new(HashMap::new()),
            domain_store_paths: Mutex::new(HashMap::new()),
            selected_domain_id: Mutex::new(None),
            voice_sessions: std::sync::Arc::new(Mutex::new(HashMap::new())),
            voice_captures: std::sync::Arc::new(Mutex::new(HashMap::new())),
            voice_cancellations: Mutex::new(HashMap::new()),
        }
    }

    #[test]
    fn change_batches_preserve_errors_and_release_database_handles() {
        let dir = tempfile::tempdir().unwrap();
        let repo = dir.path();
        let db_path = PytxoConfig::default().db_path_at(repo);
        let writer = PytxoStore::open(&db_path).unwrap();
        writer.insert_run("first", "repo").unwrap();
        drop(writer);
        let state = test_app_state();
        let request = |cursor| DomainChangesRequest {
            domain_id: repo.to_string_lossy().into_owned(),
            cursor,
        };
        let first = domain_changes_batch_blocking(&state, vec![request(0)], 200).unwrap();
        assert!(!first[0].changes.is_empty());
        let cursor = first[0].next_cursor;
        let settled = domain_changes_batch_blocking(&state, vec![request(cursor)], 200).unwrap();
        assert!(settled[0].changes.is_empty());
        assert_eq!(settled[0].next_cursor, cursor);
        assert_eq!(
            state
                .domain_change_observations
                .lock()
                .unwrap()
                .get(&db_path)
                .unwrap()
                .settled_cursor,
            cursor,
        );

        let writer = PytxoStore::open(&db_path).unwrap();
        writer.insert_run("second", "repo").unwrap();
        let changed = domain_changes_batch_blocking(&state, vec![request(cursor)], 200).unwrap();
        assert!(changed[0]
            .changes
            .iter()
            .any(|change| change.entity_id == "second"));
        let cursor = changed[0].next_cursor;
        drop(writer);

        // This rename fails on Windows if observation retains an SQLite handle.
        std::fs::rename(&db_path, db_path.with_extension("previous")).unwrap();
        assert!(domain_changes_batch_blocking(&state, vec![request(cursor)], 200).is_err());
        assert!(!db_path.exists());
        let replacement = PytxoStore::open(&db_path).unwrap();
        drop(replacement);
        let reset = domain_changes_batch_blocking(&state, vec![request(cursor)], 200).unwrap();
        assert!(reset[0].cursor_gap);
        assert_eq!(reset[0].next_cursor, 0);
        std::fs::rename(&db_path, db_path.with_extension("reset")).unwrap();
        std::fs::write(&db_path, []).unwrap();
        assert!(domain_changes_batch_blocking(&state, vec![request(0)], 200).is_err());
        assert!(
            domain_changes_batch_blocking(&state, (0..129).map(|_| request(0)).collect(), 200)
                .is_err()
        );
    }

    #[test]
    fn snapshot_refreshes_the_cached_store_path_used_by_change_batches() {
        let dir = tempfile::tempdir().unwrap();
        let repo = dir.path();
        let state = test_app_state();
        let domain_id = repo.to_string_lossy().into_owned();
        let request = || DomainChangesRequest {
            domain_id: domain_id.clone(),
            cursor: 0,
        };

        let default_path = PytxoConfig::default().db_path_at(repo);
        let default_store = PytxoStore::open(&default_path).unwrap();
        default_store.insert_run("default-run", &domain_id).unwrap();
        drop(default_store);
        let first = domain_changes_batch_blocking(&state, vec![request()], 200).unwrap();
        assert!(first[0]
            .changes
            .iter()
            .any(|change| change.entity_id == "default-run"));
        assert_eq!(
            state.domain_store_paths.lock().unwrap().get(&domain_id),
            Some(&default_path),
        );

        std::fs::write(repo.join("pytxo.toml"), "data_dir = '.pytxo/alternate'\n").unwrap();
        let alternate_config = PytxoConfig {
            data_dir: PathBuf::from(".pytxo/alternate"),
            ..PytxoConfig::default()
        };
        let alternate_path = alternate_config.db_path_at(repo);
        let alternate_store = PytxoStore::open(&alternate_path).unwrap();
        alternate_store
            .insert_run("alternate-run", &domain_id)
            .unwrap();
        drop(alternate_store);

        let snapshot = load_domain_desktop_snapshot(&state, &domain_id, 10, true);
        assert!(snapshot.diagnostics.is_empty());
        let refreshed = domain_changes_batch_blocking(&state, vec![request()], 200).unwrap();
        assert!(refreshed[0]
            .changes
            .iter()
            .any(|change| change.entity_id == "alternate-run"));
        assert!(!refreshed[0]
            .changes
            .iter()
            .any(|change| change.entity_id == "default-run"));
        assert_eq!(
            state.domain_store_paths.lock().unwrap().get(&domain_id),
            Some(&alternate_path),
        );
        assert!(!state
            .domain_change_observations
            .lock()
            .unwrap()
            .contains_key(&default_path));
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
            candidate_verification: None,
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

#[cfg(test)]
mod failure_hint_tests {
    use super::failure_hint;

    #[test]
    fn wrapped_pty_provider_error_becomes_one_sentence() {
        // Captured shape of a real Codex PTY failure: the JSON error is split
        // across events by cursor moves, framed by OSC titles and colours.
        let events = [
            "\u{1b}]0;npm\u{7}\u{1b}[33m\u{1b}[1mwarning:\u{1b}[m Exceeded skills context budget",
            "\u{1b}[31m\u{1b}[1mERROR:\u{1b}[m {\"type\":\"error\",\"status\":400,\"error\":{\"type\":\"invalid_request_error\",\"mes",
            "\u{1b}[23;80Hssage\":\"The 'gpt-6.1-sol' model is not supported when using Codex with a ChatGPT ",
            "\u{1b}[23;80H account.\"}}",
            "\u{1b}[?9001l\u{1b}[?1004l",
        ];
        assert_eq!(
            failure_hint(events.into_iter()).as_deref(),
            Some(
                "The 'gpt-6.1-sol' model is not supported when using Codex with a ChatGPT account."
            )
        );
    }

    #[test]
    fn plain_error_text_is_bounded_and_silence_has_no_hint() {
        let hint = failure_hint(
            [
                "Error: Cannot find module './missing.mjs'\r\n",
                "x".repeat(400).as_str(),
            ]
            .into_iter(),
        )
        .unwrap();
        assert!(hint.starts_with("Error: Cannot find module './missing.mjs'"));
        assert_eq!(hint.chars().count(), 240);
        assert!(hint.ends_with('…'));
        assert_eq!(failure_hint(["\u{1b}[2J", "all good"].into_iter()), None);
    }
}
