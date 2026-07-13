mod ipc;
mod ipc_auth;
mod ipc_error;
mod ipc_flow;
mod ipc_install;
mod ipc_meta;
mod ipc_voice;

use ipc::{
    agent_arbitrage, commit_workspace, dispatch_run_cmd, dry_run, ensure_workspace,
    fleet_run_status_cmd, git_diff, hitl_respond, list_agents, list_all_domains, list_domains_cmd,
    list_domains_status, list_fleet_runs, list_hitl, list_hitl_all, list_projects, list_runs,
    poll_log_lines, project_add_root_cmd, project_remove_root_cmd, project_roots_cmd,
    select_domain, stop_run, structural_graph, tail_events, workspace_structural_graph, AppState,
};
use ipc_auth::{auth_clear_session, auth_open_sign_in, auth_status, auth_store_session};
use ipc_flow::{
    flow_delete, flow_dispatch, flow_history, flow_preview, flow_save_draft,
    flow_save_reviewed_plan,
};
use ipc_install::{install_pytxo_cli, install_pytxo_cli_status, pick_workspace_folder};
use ipc_meta::{check_pytxo_cli, entitlement_status, ipc_version};
use ipc_voice::{
    voice_cancel_session, voice_default_model, voice_finish_session, voice_get_session,
    voice_install_default_model, voice_list_devices, voice_local_available, voice_model_status,
    voice_pause_session, voice_resume_session, voice_start_session,
};
use std::sync::Mutex;
use tauri::Manager;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let mut builder = tauri::Builder::default()
        .plugin(tauri_plugin_shell::init())
        .plugin(tauri_plugin_process::init())
        .plugin(tauri_plugin_notification::init())
        .plugin(tauri_plugin_updater::Builder::new().build());

    #[cfg(desktop)]
    {
        builder = builder.plugin(tauri_plugin_single_instance::init(|app, _argv, _cwd| {
            ipc_auth::focus_main_window(app);
        }));
    }

    builder
        .plugin(tauri_plugin_deep_link::init())
        .manage(AppState {
            config_path: Mutex::new(None),
            poll_cursors: Mutex::new(std::collections::HashMap::new()),
            selected_domain_id: Mutex::new(None),
            voice_sessions: std::sync::Arc::new(Mutex::new(std::collections::HashMap::new())),
            voice_captures: std::sync::Arc::new(Mutex::new(std::collections::HashMap::new())),
            voice_cancellations: Mutex::new(std::collections::HashMap::new()),
        })
        .invoke_handler(tauri::generate_handler![
            list_domains_cmd,
            list_all_domains,
            list_domains_status,
            ensure_workspace,
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
            list_hitl_all,
            hitl_respond,
            list_projects,
            project_roots_cmd,
            project_add_root_cmd,
            project_remove_root_cmd,
            list_fleet_runs,
            fleet_run_status_cmd,
            structural_graph,
            workspace_structural_graph,
            agent_arbitrage,
            ipc_version,
            check_pytxo_cli,
            install_pytxo_cli,
            install_pytxo_cli_status,
            pick_workspace_folder,
            entitlement_status,
            auth_status,
            auth_store_session,
            auth_clear_session,
            auth_open_sign_in,
            flow_save_draft,
            flow_preview,
            flow_save_reviewed_plan,
            flow_dispatch,
            flow_history,
            flow_delete,
            voice_list_devices,
            voice_default_model,
            voice_local_available,
            voice_model_status,
            voice_install_default_model,
            voice_start_session,
            voice_pause_session,
            voice_resume_session,
            voice_finish_session,
            voice_cancel_session,
            voice_get_session,
        ])
        .setup(|app| {
            #[cfg(any(target_os = "macos", target_os = "windows", target_os = "linux"))]
            {
                use tauri_plugin_deep_link::DeepLinkExt;
                let handle = app.handle().clone();
                app.deep_link().on_open_url(move |event| {
                    for url in event.urls() {
                        if let Err(e) = ipc_auth::handle_deck_deep_link(&handle, url.as_ref()) {
                            eprintln!("deck deep link: {e:?}");
                        }
                    }
                });
                if let Ok(Some(urls)) = app.deep_link().get_current() {
                    for url in urls {
                        let _ = ipc_auth::handle_deck_deep_link(app.handle(), url.as_ref());
                    }
                }
                #[cfg(any(target_os = "windows", target_os = "linux"))]
                {
                    #[cfg(debug_assertions)]
                    {
                        let _ = app.deep_link().register_all();
                    }
                }
            }
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
