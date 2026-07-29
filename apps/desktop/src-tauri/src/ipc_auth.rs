use keyring::Entry;
use serde::Serialize;
use tauri::{AppHandle, Emitter};
use tauri_plugin_shell::ShellExt;

use crate::ipc_error::{map_io_err, IpcResult, PytxoIpcError};

/// Keyring service id. Kept distinct from bundle `identifier` (`com.pytxo.desktop`)
/// so existing signed-in sessions remain readable after the Reality Deck rename.
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

/// True when argv/URL looks like a Pytxo deep link we should handle.
pub fn looks_like_deep_link(arg: &str) -> bool {
    let lower = arg.to_ascii_lowercase();
    lower.starts_with("pytxo-deck:") || lower.starts_with("pytxo:") || lower.contains("://auth")
}

/// Handle `pytxo-deck://auth?token=...` (and `pytxo://auth?token=...`) from the account page,
/// plus `pytxo://…` navigation deep links.
pub fn handle_deck_deep_link(app: &AppHandle, url: &str) -> IpcResult<()> {
    eprintln!("deck deep link: received {url}");
    let trimmed = url.trim();
    if trimmed.is_empty() {
        return Ok(());
    }

    if let Some(token) = extract_auth_token(trimmed) {
        eprintln!("deck deep link: auth token present (len={})", token.len());
        match store_session_token(&token) {
            Ok(()) => {
                eprintln!("deck deep link: session stored");
                pytxo_orchestrate::invalidate_entitlements_cache();
                let _ = app.emit(AUTH_CHANGED_EVENT, ());
                focus_main_window(app);
                return Ok(());
            }
            Err(e) => {
                eprintln!("deck deep link: store failed: {}", e.message);
                let message = format!("Could not store desktop session: {}", e.message);
                let _ = app.emit(
                    AUTH_ERROR_EVENT,
                    AuthErrorPayload {
                        message: message.clone(),
                    },
                );
                focus_main_window(app);
                return Err(e);
            }
        }
    }

    if is_auth_callback_without_token(trimmed) {
        eprintln!("deck deep link: auth URL missing token");
        let _ = app.emit(
            AUTH_ERROR_EVENT,
            AuthErrorPayload {
                message: "Desktop auth link was missing a session token.".into(),
            },
        );
        focus_main_window(app);
        return Ok(());
    }

    // Navigation deep links (pytxo://flow, pytxo-deck://settings, …).
    if trimmed.to_ascii_lowercase().starts_with("pytxo") {
        app.emit("pytxo-deep-link", trimmed).map_err(map_io_err)?;
        // Flow deep links may focus the Flow window when present.
        if trimmed.to_ascii_lowercase().contains("flow") {
            crate::flow_window::focus_or_open_flow(app);
        } else {
            focus_main_window(app);
        }
    }
    Ok(())
}

pub fn focus_main_window(app: &AppHandle) {
    crate::tray::show_main_window(app);
}

fn is_auth_callback_without_token(url: &str) -> bool {
    let lower = url.to_ascii_lowercase();
    (lower.starts_with("pytxo-deck://auth") || lower.starts_with("pytxo://auth"))
        && extract_auth_token(url).is_none()
}

/// Pull `token=` from a deep-link URL (query string or after `auth?`).
pub fn extract_auth_token(url: &str) -> Option<String> {
    let lower = url.to_ascii_lowercase();
    let is_auth = lower.contains("://auth")
        || lower.starts_with("pytxo-deck://auth")
        || lower.starts_with("pytxo://auth");
    if !is_auth {
        // Still allow token= anywhere for malformed but usable callbacks.
        if !lower.contains("token=") {
            return None;
        }
    }

    let query = url.split_once('?').map(|(_, q)| q).unwrap_or(url);
    for pair in query.split('&') {
        let pair = pair.split('#').next().unwrap_or(pair);
        if let Some(raw) = pair.strip_prefix("token=") {
            if raw.is_empty() {
                continue;
            }
            let decoded = percent_decode(raw);
            if !decoded.is_empty() {
                return Some(decoded);
            }
        }
    }
    None
}

fn read_session() -> Result<String, ()> {
    let entry = Entry::new(SERVICE, ACCOUNT).map_err(|_| ())?;
    entry.get_password().map_err(|_| ())
}

pub fn percent_decode(input: &str) -> String {
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
pub fn validate_session_jwt(token: &str) -> IpcResult<()> {
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
    const TABLE: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";
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

#[cfg(test)]
mod tests {
    use super::*;

    fn make_jwt(exp: u64) -> String {
        let header = base64url(br#"{"alg":"none"}"#);
        let payload = base64url(format!(r#"{{"exp":{exp}}}"#).as_bytes());
        format!("{header}.{payload}.sig")
    }

    fn base64url(bytes: &[u8]) -> String {
        const TABLE: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";
        let mut out = String::new();
        let mut i = 0;
        while i < bytes.len() {
            let b0 = bytes[i] as u32;
            let b1 = if i + 1 < bytes.len() {
                bytes[i + 1] as u32
            } else {
                0
            };
            let b2 = if i + 2 < bytes.len() {
                bytes[i + 2] as u32
            } else {
                0
            };
            let triple = (b0 << 16) | (b1 << 8) | b2;
            out.push(TABLE[((triple >> 18) & 63) as usize] as char);
            out.push(TABLE[((triple >> 12) & 63) as usize] as char);
            if i + 1 < bytes.len() {
                out.push(TABLE[((triple >> 6) & 63) as usize] as char);
            }
            if i + 2 < bytes.len() {
                out.push(TABLE[(triple & 63) as usize] as char);
            }
            i += 3;
        }
        out.replace('+', "-").replace('/', "_")
    }

    #[test]
    fn extract_token_from_pytxo_deck() {
        let t = extract_auth_token("pytxo-deck://auth?token=abc.def.ghi").unwrap();
        assert_eq!(t, "abc.def.ghi");
    }

    #[test]
    fn extract_token_from_pytxo_scheme() {
        let t = extract_auth_token("pytxo://auth?token=abc.def.ghi").unwrap();
        assert_eq!(t, "abc.def.ghi");
    }

    #[test]
    fn extract_token_percent_decoded() {
        let t = extract_auth_token("pytxo-deck://auth?token=a%2Eb.c").unwrap();
        assert_eq!(t, "a.b.c");
    }

    #[test]
    fn validate_jwt_accepts_future_exp() {
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs();
        assert!(validate_session_jwt(&make_jwt(now + 3600)).is_ok());
    }

    #[test]
    fn validate_jwt_rejects_expired() {
        assert!(validate_session_jwt(&make_jwt(1)).is_err());
    }

    #[test]
    fn looks_like_deep_link_detects_schemes() {
        assert!(looks_like_deep_link("pytxo-deck://auth?token=x"));
        assert!(looks_like_deep_link("pytxo://flow"));
        assert!(!looks_like_deep_link("--flag"));
    }
}
