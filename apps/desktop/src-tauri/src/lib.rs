mod ipc;

use ipc::{
    dry_run, git_diff, list_agents, list_runs, poll_log_lines, start_run, stop_run, tail_events,
    AppState,
};
use std::sync::Mutex;
use tauri::Manager;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_shell::init())
        .manage(AppState {
            config_path: Mutex::new(None),
            last_event_id: Mutex::new(0),
        })
        .invoke_handler(tauri::generate_handler![
            list_runs,
            list_agents,
            tail_events,
            poll_log_lines,
            dry_run,
            start_run,
            stop_run,
            git_diff,
        ])
        .setup(|app| {
            if let Ok(dir) = std::env::current_dir() {
                let cfg = dir.join("pytxo.toml");
                if cfg.exists() {
                    if let Some(state) = app.try_state::<AppState>() {
                        if let Ok(mut guard) = state.config_path.lock() {
                            *guard = Some(cfg);
                        }
                    }
                }
            }
            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
