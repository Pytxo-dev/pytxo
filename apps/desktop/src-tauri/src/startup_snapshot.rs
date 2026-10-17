//! First Desktop snapshot, read while the main webview is still being created.
//!
//! Creating the WebView2 window blocks the main thread for most of a second
//! before any page script can ask for data. This reads the first consistent
//! snapshot in that window, in the order Desktop's first load uses: catalog
//! fingerprint, then every domain's change boundary, then the snapshot. The
//! first page load takes it once. Its cursors start at those boundaries, so a
//! change written after them is caught by the next change poll like any delta.
//! Read path only: no permission profile or execution domain is affected.

use std::path::PathBuf;
use std::sync::Mutex;
use std::thread::JoinHandle;
use std::time::{SystemTime, UNIX_EPOCH};

use serde::Serialize;
use tauri::Manager;

use crate::ipc::{
    domain_changes_batch_blocking, load_desktop_snapshot_blocking, store_fingerprint, AppState,
    DesktopSnapshotDto, DomainChangesRequest,
};
use crate::ipc_error::{map_lock_err, map_orch_err, IpcResult};

#[derive(Serialize)]
pub struct StartupSnapshotDto {
    pub catalog_fingerprint: String,
    pub boundaries: Vec<DomainBoundaryDto>,
    pub snapshot: DesktopSnapshotDto,
    /// When the snapshot read began, so the page never presents it as newer.
    pub read_at_ms: u64,
    /// The read's limits; it always includes agents. The page uses it only
    /// where its own first read would use the same limits.
    pub run_limit: usize,
    pub fleet_limit: usize,
}

#[derive(Serialize)]
pub struct DomainBoundaryDto {
    pub domain_id: String,
    pub cursor: i64,
}

struct StartupRead {
    dto: StartupSnapshotDto,
    /// Store paths and change observations the read resolved, for the app state.
    state: AppState,
}

const RUN_LIMIT: usize = 30;
const FLEET_LIMIT: usize = 20;

static STARTUP_READ: Mutex<Option<JoinHandle<Option<StartupRead>>>> = Mutex::new(None);

pub fn start(config_path: Option<PathBuf>) {
    let handle = std::thread::Builder::new()
        .name("pytxo-startup-snapshot".into())
        .spawn(move || read(config_path))
        .ok();
    if let Ok(mut slot) = STARTUP_READ.lock() {
        *slot = handle;
    }
}

/// Any failure leaves the page to its normal first load.
fn read(config_path: Option<PathBuf>) -> Option<StartupRead> {
    let state = AppState::new(config_path);
    let catalog = pytxo_store::default_catalog_path()?;
    let catalog_fingerprint = format!("{:?}", store_fingerprint(&catalog).ok()?);
    let domain_ids = pytxo_orchestrate::list_catalog_domains()
        .ok()?
        .into_iter()
        .map(|entry| entry.domain_id)
        .collect::<Vec<_>>();
    // A cursor past the end returns each store's boundary in one page.
    let requests = domain_ids
        .iter()
        .map(|domain_id| DomainChangesRequest {
            domain_id: domain_id.clone(),
            cursor: i64::MAX,
        })
        .collect();
    let pages = domain_changes_batch_blocking(&state, requests, 1).ok()?;
    let boundaries = domain_ids
        .into_iter()
        .zip(pages)
        .map(|(domain_id, page)| DomainBoundaryDto {
            domain_id,
            cursor: page.next_cursor,
        })
        .collect();
    let read_at_ms = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .ok()?
        .as_millis() as u64;
    let snapshot =
        load_desktop_snapshot_blocking(&state, Some(RUN_LIMIT), Some(FLEET_LIMIT), Some(true))
            .ok()?;
    Some(StartupRead {
        dto: StartupSnapshotDto {
            catalog_fingerprint,
            boundaries,
            snapshot,
            read_at_ms,
            run_limit: RUN_LIMIT,
            fleet_limit: FLEET_LIMIT,
        },
        state,
    })
}

/// Returns the startup read once; every later call (and a webview reload) gets none.
#[tauri::command]
pub async fn take_startup_snapshot(app: tauri::AppHandle) -> IpcResult<Option<StartupSnapshotDto>> {
    let Some(handle) = STARTUP_READ.lock().map_err(map_lock_err)?.take() else {
        return Ok(None);
    };
    let Some(read) = tauri::async_runtime::spawn_blocking(move || handle.join().ok().flatten())
        .await
        .map_err(map_orch_err)?
    else {
        return Ok(None);
    };
    app.state::<AppState>().adopt(read.state)?;
    Ok(Some(read.dto))
}
