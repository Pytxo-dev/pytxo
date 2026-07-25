use keyring::Entry;
use serde::Serialize;
use tauri::{AppHandle, Emitter};
use tauri_plugin_shell::ShellExt;

use crate::ipc_error::{map_io_err, PytxoIpcError, IpcResult};

const SERVICE: &str = "com.pytxo.reality-deck";
const ACCOUNT: &str = "clerk-session";
pub const AUTH_CHANGED_EVENT: &str = "deck-auth-changed";
pub const AUTH_ERROR_EVENT: &str = "deck-auth-error";

#[derive(Serialize)]
pub struct AuthStatusDto {
    pub signed_in: bool,
    pub session_present: bool,
}

#[derive(Clone, Serialize)]
struct AuthErrorPayload {
    message: String,
}

#[tauri::command]
pub fn auth_status() -> IpcResult<AuthStatusDto> {
    let session_present = read_session().is_ok();
    Ok(AuthStatusDto {
        signed_in: session_present,
        session_present,
    })
}

fn store_session_token(token: &str) -> IpcResult<()> {
    validate_session_jwt(token)?;
    let entry = Entry::new(SERVICE, ACCOUNT).map_err(map_io_err)?;
    entry.set_password(token).map_err(map_io_err)?;
    // Keep the bearer out of process env so agent PTY/subprocess children cannot read it.
    pytxo_orchestrate::set_runtime_session_token(Some(token.to_string()));
    std::env::remove_var("PYTXO_ULTRA_SESSION");
    Ok(())
}

#[tauri::command]
pub fn auth_clear_session(app: AppHandle) -> IpcResult<()> {
    if let Ok(entry) = Entry::new(SERVICE, ACCOUNT) {
        let _ = entry.delete_credential();
    }
    pytxo_orchestrate::set_runtime_session_token(None);
    std::env::remove_var("PYTXO_ULTRA_SESSION");
    pytxo_orchestrate::invalidate_entitlements_cache();
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
            pytxo_orchestrate::set_runtime_session_token(Some(token));
            std::env::remove_var("PYTXO_ULTRA_SESSION");
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
                match store_session_token(&decoded) {
                    Ok(()) => {
                        pytxo_orchestrate::invalidate_entitlements_cache();
                        let _ = app.emit(AUTH_CHANGED_EVENT, ());
                        focus_main_window(app);
                        return Ok(());
                    }
                    Err(e) => {
                        let message = format!("Could not store desktop session: {}", e.message);
                        let _ = app.emit(
                            AUTH_ERROR_EVENT,
                            AuthErrorPayload {
                                message: message.clone(),
                            },
                        );
                        return Err(e);
                    }
                }
            }
        }
    }
    if url.starts_with("pytxo-deck://auth") {
        let _ = app.emit(
            AUTH_ERROR_EVENT,
            AuthErrorPayload {
                message: "Desktop auth link was missing a session token.".into(),
            },
        );
        focus_main_window(app);
        return Ok(());
    }
    Ok(())
}

pub fn focus_main_window(app: &AppHandle) {
    crate::tray::show_main_window(app);
}

fn read_session() -> Result<String, ()> {
    let entry = Entry::new(SERVICE, ACCOUNT).map_err(|_| ())?;
    entry.get_password().map_err(|_| ())
}

fn percent_decode(input: &str) -> String {
    let bytes = input.as_bytes();
    let mut out = Vec::with_capacity(bytes.len());
    let mut i = 0;
    while i < bytes.len() {
        if bytes[i] == b'%' && i + 2 < bytes.len() {
            if let (Some(h), Some(l)) = (from_hex(bytes[i + 1]), from_hex(bytes[i + 2])) {
                out.push((h << 4) | l);
                i += 3;
                continue;
            }
        }
        if bytes[i] == b'+' {
            out.push(b' ');
        } else {
            out.push(bytes[i]);
        }
        i += 1;
    }
    String::from_utf8_lossy(&out).into_owned()
}

fn from_hex(b: u8) -> Option<u8> {
    match b {
        b'0'..=b'9' => Some(b - b'0'),
        b'a'..=b'f' => Some(b - b'a' + 10),
        b'A'..=b'F' => Some(b - b'A' + 10),
        _ => None,
    }
}

/// Accept Clerk-shaped JWTs only: three segments + JSON payload with unexpired `exp`.
fn validate_session_jwt(token: &str) -> IpcResult<()> {
    let parts: Vec<&str> = token.split('.').collect();
    if parts.len() != 3 || parts.iter().any(|p| p.is_empty()) {
        return Err(PytxoIpcError::new(
            "auth",
            "Session token must be a JWT (three segments).",
        ));
    }
    let payload = decode_jwt_segment(parts[1]).map_err(|e| PytxoIpcError::new("auth", e))?;
    let value: serde_json::Value = serde_json::from_slice(&payload)
        .map_err(|_| PytxoIpcError::new("auth", "Session token payload is not valid JSON."))?;
    let exp = value
        .get("exp")
        .and_then(|v| v.as_u64())
        .ok_or_else(|| PytxoIpcError::new("auth", "Session token is missing exp."))?;
    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0);
    if exp < now {
        return Err(PytxoIpcError::new("auth", "Session token has expired."));
    }
    Ok(())
}

fn decode_jwt_segment(segment: &str) -> Result<Vec<u8>, String> {
    let mut s = segment.replace('-', "+").replace('_', "/");
    while s.len() % 4 != 0 {
        s.push('=');
    }
    base64_decode(&s).map_err(|_| "Session token segment is not valid base64.".into())
}

fn base64_decode(input: &str) -> Result<Vec<u8>, ()> {
    const TABLE: &[u8] =
        b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";
    let mut out = Vec::new();
    let mut buf = 0u32;
    let mut bits = 0u32;
    for &b in input.as_bytes() {
        if b == b'=' {
            break;
        }
        let val = TABLE.iter().position(|&c| c == b).ok_or(())? as u32;
        buf = (buf << 6) | val;
        bits += 6;
        if bits >= 8 {
            bits -= 8;
            out.push((buf >> bits) as u8);
            buf &= (1 << bits) - 1;
        }
    }
    Ok(out)
}
