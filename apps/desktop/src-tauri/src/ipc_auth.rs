use keyring::Entry;
use serde::Serialize;
use tauri::{AppHandle, Emitter, Manager};
use tauri_plugin_shell::ShellExt;

use crate::ipc_error::{map_io_err, IpcResult};

const SERVICE: &str = "com.pytxo.reality-deck";
const ACCOUNT: &str = "clerk-session";
pub const AUTH_CHANGED_EVENT: &str = "deck-auth-changed";

#[derive(Serialize)]
pub struct AuthStatusDto {
    pub signed_in: bool,
    pub session_present: bool,
}

#[tauri::command]
pub fn auth_status() -> IpcResult<AuthStatusDto> {
    let session_present = read_session().is_ok();
    Ok(AuthStatusDto {
        signed_in: session_present,
        session_present,
    })
}

#[tauri::command]
pub fn auth_store_session(token: String) -> IpcResult<()> {
    store_session_token(&token)
}

fn store_session_token(token: &str) -> IpcResult<()> {
    let entry = Entry::new(SERVICE, ACCOUNT).map_err(map_io_err)?;
    entry.set_password(token).map_err(map_io_err)?;
    std::env::set_var("PYTXO_ULTRA_SESSION", token);
    Ok(())
}

#[tauri::command]
pub fn auth_clear_session(app: AppHandle) -> IpcResult<()> {
    if let Ok(entry) = Entry::new(SERVICE, ACCOUNT) {
        let _ = entry.delete_credential();
    }
    std::env::remove_var("PYTXO_ULTRA_SESSION");
    let _ = app.emit(AUTH_CHANGED_EVENT, ());
    Ok(())
}

#[tauri::command]
#[allow(deprecated)] // Compatibility path until the existing shell plugin is replaced by opener.
pub async fn auth_open_sign_in(app: AppHandle) -> IpcResult<()> {
    let url = "https://pytxo.com/account?deck_callback=pytxo-deck";
    app.shell().open(url, None).map_err(map_io_err)?;
    Ok(())
}

pub fn hydrate_session_env() {
    if let Ok(token) = read_session() {
        if !token.is_empty() {
            std::env::set_var("PYTXO_ULTRA_SESSION", token);
        }
    }
}

/// Handle `pytxo-deck://auth?token=...` deep-link callbacks from the account page.
pub fn handle_deck_deep_link(app: &AppHandle, url: &str) -> IpcResult<()> {
    if url.starts_with("pytxo:") && !url.starts_with("pytxo-deck:") {
        app.emit("pytxo-deep-link", url).map_err(map_io_err)?;
        focus_main_window(app);
        return Ok(());
    }
    if !url.starts_with("pytxo-deck:") {
        return Ok(());
    }
    let query = url.split('?').nth(1).unwrap_or("");
    for pair in query.split('&') {
        if let Some(token) = pair.strip_prefix("token=") {
            if !token.is_empty() {
                let decoded = percent_decode(token);
                store_session_token(&decoded)?;
                let _ = app.emit(AUTH_CHANGED_EVENT, ());
                focus_main_window(app);
                return Ok(());
            }
        }
    }
    if !url.starts_with("pytxo-deck://auth") {
        app.emit("pytxo-deep-link", url).map_err(map_io_err)?;
        focus_main_window(app);
    }
    Ok(())
}

fn percent_decode(input: &str) -> String {
    let bytes = input.as_bytes();
    let mut out = Vec::with_capacity(bytes.len());
    let mut i = 0;
    while i < bytes.len() {
        if bytes[i] == b'%' && i + 2 < bytes.len() {
            if let Ok(v) =
                u8::from_str_radix(std::str::from_utf8(&bytes[i + 1..i + 3]).unwrap_or(""), 16)
            {
                out.push(v);
                i += 3;
                continue;
            }
        }
        out.push(bytes[i]);
        i += 1;
    }
    String::from_utf8_lossy(&out).into_owned()
}

pub fn focus_main_window(app: &AppHandle) {
    if let Some(win) = app.get_webview_window("main") {
        let _ = win.unminimize();
        let _ = win.show();
        let _ = win.set_focus();
    }
}

fn read_session() -> Result<String, String> {
    let entry = Entry::new(SERVICE, ACCOUNT).map_err(|e| e.to_string())?;
    entry.get_password().map_err(|e| e.to_string())
}
