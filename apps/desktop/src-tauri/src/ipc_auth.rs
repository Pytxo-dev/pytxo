use keyring::Entry;
use serde::Serialize;
use tauri::AppHandle;
use tauri_plugin_shell::ShellExt;

use crate::ipc_error::{map_io_err, IpcResult};

const SERVICE: &str = "com.pytxo.reality-deck";
const ACCOUNT: &str = "clerk-session";

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
    let entry = Entry::new(SERVICE, ACCOUNT).map_err(map_io_err)?;
    entry.set_password(&token).map_err(map_io_err)?;
    std::env::set_var("PYTXO_ULTRA_SESSION", &token);
    Ok(())
}

#[tauri::command]
pub fn auth_clear_session() -> IpcResult<()> {
    if let Ok(entry) = Entry::new(SERVICE, ACCOUNT) {
        let _ = entry.delete_credential();
    }
    std::env::remove_var("PYTXO_ULTRA_SESSION");
    Ok(())
}

#[tauri::command]
pub async fn auth_open_sign_in(app: AppHandle) -> IpcResult<()> {
    let url = "https://pytxo.com/account?deck_callback=pytxo-deck";
    app.shell()
        .open(url, None)
        .map_err(map_io_err)?;
    Ok(())
}

pub fn hydrate_session_env() {
    if let Ok(token) = read_session() {
        if !token.is_empty() {
            std::env::set_var("PYTXO_ULTRA_SESSION", token);
        }
    }
}

fn read_session() -> Result<String, String> {
    let entry = Entry::new(SERVICE, ACCOUNT).map_err(|e| e.to_string())?;
    entry.get_password().map_err(|e| e.to_string())
}
