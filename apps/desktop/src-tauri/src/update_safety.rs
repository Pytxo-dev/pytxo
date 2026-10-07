//! Read-only upgrade preflight across every registered execution domain and
//! every permission profile. This does not grant or change runtime authority.
//! Handoff holds the cooperative user-catalog upgrade lock across installation.
use crate::{
    ipc::AppState,
    ipc_error::{IpcResult, PytxoIpcError},
    workspace_terminal::WorkspaceTerminals,
};
use std::{path::Path, sync::Mutex};
use tauri::{State, Window};

#[derive(Default)]
pub struct UpdateHandoff(Mutex<Option<(String, pytxo_core::UpgradeGuard)>>);

fn refuse(message: impl Into<String>) -> PytxoIpcError {
    PytxoIpcError::new("update_busy", message)
}

fn inspect_domain(path: &Path) -> IpcResult<()> {
    let count = pytxo_store::PytxoStore::open_existing_read_only(path)
        .and_then(|store| store.update_blocking_work_count())
        .map_err(|error| refuse(format!("Cannot verify workspace activity at {}: {error}. Resolve the workspace before updating.", path.display())))?;
    if count > 0 {
        return Err(refuse(format!(
            "{count} run(s) or repository operation(s) need to finish at {} before updating.",
            path.display()
        )));
    }
    Ok(())
}

#[tauri::command]
pub fn update_preflight(
    state: State<'_, AppState>,
    terminals: State<'_, WorkspaceTerminals>,
) -> IpcResult<()> {
    check_idle(&state, &terminals)
}

fn check_idle(state: &AppState, terminals: &WorkspaceTerminals) -> IpcResult<()> {
    if terminals.live_count() > 0 {
        return Err(refuse("End open workspace terminals before updating. Their sessions cannot resume after app exit."));
    }
    let sessions = state
        .voice_sessions
        .lock()
        .map_err(|_| refuse("Cannot verify voice activity. Restart after saving your work."))?;
    if sessions.values().any(|session| {
        matches!(
            session.state(),
            pytxo_voice::VoiceState::Recording
                | pytxo_voice::VoiceState::Paused
                | pytxo_voice::VoiceState::Transcribing
        )
    }) {
        return Err(refuse(
            "Finish or cancel voice capture/transcription before updating.",
        ));
    }
    drop(sessions);
    let catalog = pytxo_store::Catalog::open_default()
        .map_err(|error| refuse(format!("Cannot verify workspace catalog: {error}")))?;
    let fleets = catalog
        .active_fleet_count()
        .map_err(|error| refuse(format!("Cannot verify fleet activity: {error}")))?;
    if fleets > 0 {
        return Err(refuse(
            "Finish active multi-workspace runs before updating.",
        ));
    }
    let domains = catalog
        .list_domains()
        .map_err(|error| refuse(format!("Cannot verify workspace catalog: {error}")))?;
    for domain in domains {
        inspect_domain(Path::new(&domain.db_path))?;
    }
    Ok(())
}

#[tauri::command]
pub fn begin_update_handoff(
    window: Window,
    state: State<'_, AppState>,
    terminals: State<'_, WorkspaceTerminals>,
    handoff: State<'_, UpdateHandoff>,
) -> IpcResult<String> {
    if window.label() != "main" {
        return Err(refuse("Updates belong to the main window."));
    }
    let mut held = handoff
        .0
        .lock()
        .map_err(|_| refuse("Cannot inspect update ownership."))?;
    if held.is_some() {
        return Err(refuse("Another update handoff is already in progress."));
    }
    let lease = pytxo_core::UpgradeGuard::upgrade().map_err(|error| refuse(error.to_string()))?;
    // Admission is already closed while we check older persisted state and
    // in-process sessions. A failed preflight drops the exclusive lease.
    check_idle(&state, &terminals)?;
    let token = uuid::Uuid::new_v4().to_string();
    *held = Some((token.clone(), lease));
    Ok(token)
}

#[tauri::command]
pub fn finish_update_handoff(
    window: Window,
    handoff: State<'_, UpdateHandoff>,
    token: String,
) -> IpcResult<()> {
    if window.label() != "main" {
        return Err(refuse("Updates belong to the main window."));
    }
    let mut held = handoff
        .0
        .lock()
        .map_err(|_| refuse("Cannot release update ownership."))?;
    if held.as_ref().map(|(owner, _)| owner.as_str()) != Some(token.as_str()) {
        return Err(refuse(
            "Update handoff identity changed; restart Desktop after saving work.",
        ));
    }
    held.take();
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn missing_or_corrupt_store_never_becomes_idle() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("missing.db");
        assert!(inspect_domain(&path).is_err());
        assert!(!path.exists());
        std::fs::write(&path, b"not a database").unwrap();
        assert!(inspect_domain(&path).is_err());
    }

    #[test]
    fn older_active_run_is_not_hidden_by_recent_history() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("state.db");
        let store = pytxo_store::PytxoStore::open(&path).unwrap();
        store.insert_run("active", "/repo").unwrap();
        for index in 0..40 {
            let id = format!("completed-{index}");
            store.insert_run(&id, "/repo").unwrap();
            store.finish_run(&id, "completed").unwrap();
        }
        assert!(inspect_domain(&path).is_err());
        store.finish_run("active", "completed").unwrap();
        assert!(inspect_domain(&path).is_ok());
        store
            .save_run_contract("active", "base", "{}", "{}")
            .unwrap();
        assert!(store.begin_run_preparation("active").unwrap());
        assert!(inspect_domain(&path).is_err());
    }
}
