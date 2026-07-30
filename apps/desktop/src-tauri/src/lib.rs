mod flow_window;
mod ipc;
mod ipc_auth;
mod ipc_error;
mod ipc_flow;
mod ipc_install;
mod ipc_meta;
mod ipc_voice;
mod tray;

use ipc::{
    agent_arbitrage, commit_workspace, dispatch_run_cmd, domain_is_trusted, dry_run,
    ensure_workspace, fleet_run_status_cmd, forget_domain, get_domain_permission, git_diff,
    hitl_respond, list_agents, list_all_domains, list_domains_cmd, list_domains_status,
    list_fleet_runs, list_hitl, list_hitl_all, list_projects, list_providers, list_runs,
    list_trusted_domains, load_desktop_snapshot, poll_log_lines, project_add_root_cmd,
    project_remove_root_cmd, project_roots_cmd, select_domain, set_domain_permission, stop_run,
    structural_graph, tail_events, workspace_structural_graph, AppState,
};
use ipc_auth::{auth_clear_session, auth_open_sign_in, auth_status};
use ipc_flow::{
    flow_delete, flow_dispatch, flow_history, flow_preview, flow_save_draft,
    flow_save_reviewed_plan,
};
use ipc_install::{
    create_example_workspace, install_pytxo_cli, install_pytxo_cli_status, pick_workspace_folder,
};
use ipc_meta::{check_pytxo_cli, entitlement_status, ipc_version, list_ade_clis, start_ade_login};
use ipc_voice::{
    voice_cancel_session, voice_default_model, voice_finish_session, voice_get_session,
    voice_install_default_model, voice_list_devices, voice_local_available, voice_model_status,
    voice_pause_session, voice_resume_session, voice_start_session,
};
use std::sync::Mutex;
use tauri::Manager;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    // single-instance MUST be first so Windows protocol argv reaches this process.
    #[allow(unused_mut)]
    let mut builder = {
        #[cfg(desktop)]
        {
            tauri::Builder::default().plugin(tauri_plugin_single_instance::init(
                |app, argv, _cwd| {
                    let mut handled_auth = false;
                    for arg in &argv {
                        if ipc_auth::looks_like_deep_link(arg) {
                            eprintln!("single-instance: forwarding deep link {arg}");
                            if let Err(e) = ipc_auth::handle_deck_deep_link(app, arg) {
                                eprintln!("single-instance deep link: {e:?}");
                            }
                            if arg.to_ascii_lowercase().contains("://auth")
                                || arg.to_ascii_lowercase().contains("auth?")
                            {
                                handled_auth = true;
                            }
                        }
                    }
                    if !handled_auth {
                        // Still focus main when a second instance launches without a URL.
                        ipc_auth::focus_main_window(app);
                    }
                },
            ))
        }
        #[cfg(not(desktop))]
        {
            tauri::Builder::default()
        }
    };

    builder = builder
        .plugin(tauri_plugin_shell::init())
        .plugin(tauri_plugin_process::init())
        .plugin(tauri_plugin_notification::init())
        .plugin(tauri_plugin_updater::Builder::new().build());

    // Verification-only bridge for the Tauri MCP driver; requires the opt-in
    // `mcp-bridge` feature and never runs in a release build.
    #[cfg(all(feature = "mcp-bridge", debug_assertions))]
    {
        builder = builder.plugin(tauri_plugin_mcp_bridge::init());
    }

    builder
        .plugin(tauri_plugin_deep_link::init())
        .manage(tray::TrayPrefs::load())
        .manage(AppState {
            config_path: Mutex::new(None),
            poll_cursors: Mutex::new(std::collections::HashMap::new()),
            selected_domain_id: Mutex::new(None),
            voice_sessions: std::sync::Arc::new(Mutex::new(std::collections::HashMap::new())),
            voice_captures: std::sync::Arc::new(Mutex::new(std::collections::HashMap::new())),
            voice_cancellations: Mutex::new(std::collections::HashMap::new()),
        })
        .on_window_event(|window, event| {
            if let tauri::WindowEvent::CloseRequested { api, .. } = event {
                // Flow window closes normally; only main uses close-to-tray.
                if window.label() == "main" && tray::close_to_tray_enabled(window.app_handle()) {
                    api.prevent_close();
                    tray::hide_main_window_to_tray(window.app_handle());
                }
            }
        })
        .invoke_handler(tauri::generate_handler![
            list_domains_cmd,
            list_all_domains,
            list_domains_status,
            forget_domain,
            ensure_workspace,
            select_domain,
            list_trusted_domains,
            get_domain_permission,
            set_domain_permission,
            domain_is_trusted,
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
            load_desktop_snapshot,
            fleet_run_status_cmd,
            structural_graph,
            workspace_structural_graph,
            agent_arbitrage,
            ipc_version,
            list_providers,
            check_pytxo_cli,
            install_pytxo_cli,
            install_pytxo_cli_status,
            pick_workspace_folder,
            create_example_workspace,
            entitlement_status,
            list_ade_clis,
            start_ade_login,
            auth_status,
            auth_clear_session,
            auth_open_sign_in,
            flow_save_draft,
            flow_preview,
            flow_save_reviewed_plan,
            flow_dispatch,
            flow_history,
            flow_delete,
            flow_window::open_flow_window,
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
            tray::get_close_to_tray,
            tray::set_close_to_tray,
            tray::set_tray_needs_you,
        ])
        .setup(|app| {
            #[cfg(desktop)]
            {
                if let Err(e) = tray::install_tray(app.handle()) {
                    eprintln!("tray: failed to install system tray: {e:?}");
                }
            }
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
                    // Register custom URL schemes for portable/dev installs.
                    // Packaged MSI/NSIS also registers schemes from tauri.conf.
                    let _ = app.deep_link().register_all();
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
