//! Dedicated Flow webview window (`label: "flow"`).

use tauri::{AppHandle, Emitter, Manager, WebviewUrl, WebviewWindowBuilder};

pub const FLOW_WINDOW_LABEL: &str = "flow";

/// Focus an existing Flow window or create one loading `#/flow-standalone`.
pub fn focus_or_open_flow(app: &AppHandle) {
    if let Some(win) = app.get_webview_window(FLOW_WINDOW_LABEL) {
        let _ = win.show();
        let _ = win.unminimize();
        let _ = win.set_focus();
        return;
    }

    let url = WebviewUrl::App("#/flow-standalone".into());
    match WebviewWindowBuilder::new(app, FLOW_WINDOW_LABEL, url)
        .title("Pytxo Flow")
        .inner_size(1100.0, 760.0)
        .min_inner_size(720.0, 480.0)
        .decorations(false)
        .shadow(true)
        .build()
    {
        Ok(win) => {
            let _ = win.set_focus();
        }
        Err(e) => {
            eprintln!("flow window: failed to create: {e:?}");
            crate::tray::show_main_window(app);
            let _ = app.emit("pytxo-deep-link", "pytxo://flow");
        }
    }
}

#[tauri::command]
pub fn open_flow_window(app: AppHandle) -> Result<(), String> {
    focus_or_open_flow(&app);
    Ok(())
}
