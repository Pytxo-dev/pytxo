mod ipc;
mod ipc_auth;
mod ipc_error;
mod ipc_meta;

use ipc::{
    agent_arbitrage, commit_workspace, dispatch_run_cmd, dry_run, git_diff, hitl_respond,
    list_agents, list_all_domains, list_domains_cmd, list_hitl, list_projects, list_runs,
    poll_log_lines, select_domain, stop_run, tail_events, AppState,
};
use ipc_auth::{auth_clear_session, auth_open_sign_in, auth_status, auth_store_session};
use ipc_meta::{check_pytxo_cli, entitlement_status, ipc_version};
use std::sync::Mutex;
use tauri::Manager;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_shell::init())
        .plugin(tauri_plugin_process::init())
        .plugin(tauri_plugin_notification::init())
        .plugin(tauri_plugin_updater::Builder::new().build())
        .manage(AppState {
            config_path: Mutex::new(None),
            poll_cursors: Mutex::new(std::collections::HashMap::new()),
            selected_domain_id: Mutex::new(None),
        })
        .invoke_handler(tauri::generate_handler![
            list_domains_cmd,
            select_domain,
            list_runs,
            list_agents,
            tail_events,
            poll_log_lines,
            dry_run,
            dispatch_run_cmd,
            stop_run,
            git_diff,
            commit_workspace,
            list_hitl,
            hitl_respond,
            list_all_domains,
            list_projects,
            agent_arbitrage,
            ipc_version,
            check_pytxo_cli,
            entitlement_status,
            auth_status,
            auth_store_session,
            auth_clear_session,
            auth_open_sign_in,
        ])
        .setup(|app| {
            ipc_auth::hydrate_session_env();
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
