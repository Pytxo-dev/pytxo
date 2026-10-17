//! Optional Pytxo account in Desktop: browser sign-in with PKCE, sharing an
//! Apply receipt. Local work never depends on it. The credential lives in the
//! OS keyring, never in process env, so agent children cannot read it.

use std::sync::{Mutex, OnceLock};
use std::time::{Duration, Instant};

use base64::engine::general_purpose::URL_SAFE_NO_PAD;
use base64::Engine;
use keyring::Entry;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use tauri::{AppHandle, Emitter, State};
use tauri_plugin_shell::ShellExt;

use crate::ipc::{resolve_domain, AppState};
use crate::ipc_error::{map_io_err, IpcResult, PytxoIpcError};

const SERVICE: &str = "com.pytxo.desktop";
const ACCOUNT: &str = "pytxo-account";
pub const ACCOUNT_CHANGED_EVENT: &str = "pytxo-account-changed";
pub const ACCOUNT_ERROR_EVENT: &str = "pytxo-account-error";
const PENDING_TTL: Duration = Duration::from_secs(600);

struct Pending {
    state: String,
    verifier: String,
    started: Instant,
}

fn pending() -> &'static Mutex<Option<Pending>> {
    static PENDING: OnceLock<Mutex<Option<Pending>>> = OnceLock::new();
    PENDING.get_or_init(|| Mutex::new(None))
}

#[derive(Clone, Deserialize, Serialize)]
struct StoredAccount {
    account_id: String,
    token: String,
    expires_at: String,
}

#[derive(Serialize)]
pub struct AccountStatusDto {
    pub signed_in: bool,
    pub account_id: Option<String>,
    pub expires_at: Option<String>,
    pub sign_in_pending: bool,
}

#[derive(Serialize)]
pub struct SharedReceiptDto {
    pub url: String,
    pub receipt_id: String,
}

fn link_url() -> String {
    std::env::var("PYTXO_LINK_URL").unwrap_or_else(|_| "https://link.pytxo.com".into())
}

fn web_url() -> String {
    std::env::var("PYTXO_WEB_URL").unwrap_or_else(|_| "https://pytxo.com".into())
}

fn err(code: &str, message: impl Into<String>) -> PytxoIpcError {
    PytxoIpcError::new(code, message)
}

fn load() -> Option<StoredAccount> {
    let raw = Entry::new(SERVICE, ACCOUNT).ok()?.get_password().ok()?;
    serde_json::from_str(&raw).ok()
}

fn save(account: &StoredAccount) -> IpcResult<()> {
    let json = serde_json::to_string(account).map_err(map_io_err)?;
    Entry::new(SERVICE, ACCOUNT)
        .and_then(|e| e.set_password(&json))
        .map_err(map_io_err)
}

fn forget() {
    if let Ok(entry) = Entry::new(SERVICE, ACCOUNT) {
        let _ = entry.delete_credential();
    }
}

fn random_b64() -> IpcResult<String> {
    let mut bytes = [0u8; 32];
    getrandom::getrandom(&mut bytes).map_err(|e| err("account", e.to_string()))?;
    Ok(URL_SAFE_NO_PAD.encode(bytes))
}

fn http() -> IpcResult<reqwest::Client> {
    reqwest::Client::builder()
        .timeout(Duration::from_secs(20))
        .user_agent(concat!("pytxo-desktop/", env!("CARGO_PKG_VERSION")))
        .build()
        .map_err(|e| err("account", e.to_string()))
}

#[tauri::command(async)]
pub fn account_status() -> IpcResult<AccountStatusDto> {
    let account = load();
    let sign_in_pending = pending()
        .lock()
        .map(|p| {
            p.as_ref()
                .is_some_and(|p| p.started.elapsed() < PENDING_TTL)
        })
        .unwrap_or(false);
    Ok(AccountStatusDto {
        signed_in: account.is_some(),
        account_id: account.as_ref().map(|a| a.account_id.clone()),
        expires_at: account.map(|a| a.expires_at),
        sign_in_pending,
    })
}

/// Opens pytxo.com/account/connect; the browser returns `pytxo-deck://account?code&state`.
#[tauri::command]
#[allow(deprecated)] // Same shell open path as the existing sign-in until the opener plugin lands.
pub async fn account_sign_in(app: AppHandle) -> IpcResult<()> {
    let verifier = random_b64()?;
    let state = random_b64()?;
    let challenge = URL_SAFE_NO_PAD.encode(Sha256::digest(verifier.as_bytes()));
    *pending()
        .lock()
        .map_err(|e| err("account", e.to_string()))? = Some(Pending {
        state: state.clone(),
        verifier,
        started: Instant::now(),
    });
    let url = format!(
        "{}/account/connect?client=desktop&state={state}&code_challenge={challenge}",
        web_url()
    );
    app.shell().open(url, None).map_err(map_io_err)?;
    Ok(())
}

pub fn is_account_callback(url: &str) -> bool {
    tauri::Url::parse(url)
        .is_ok_and(|u| u.scheme() == "pytxo-deck" && u.host_str() == Some("account"))
}

/// Deep-link half of sign-in. Only a callback matching the pending state is exchanged.
pub fn handle_callback(app: &AppHandle, url: &str) -> IpcResult<()> {
    let parsed = tauri::Url::parse(url).map_err(|e| err("account", e.to_string()))?;
    let get = |key: &str| {
        parsed
            .query_pairs()
            .find(|(k, _)| k == key)
            .map(|(_, v)| v.into_owned())
    };
    let (Some(code), Some(state)) = (get("code"), get("state")) else {
        return Err(err(
            "account",
            "The sign-in link was incomplete. Start sign-in again from Settings.",
        ));
    };
    let verifier = {
        let mut guard = pending()
            .lock()
            .map_err(|e| err("account", e.to_string()))?;
        match guard.take() {
            Some(p) if p.state == state && p.started.elapsed() < PENDING_TTL => p.verifier,
            other => {
                *guard = other;
                return Err(err("account", "This sign-in link does not match the request from this Desktop. Start sign-in again."));
            }
        }
    };
    let app = app.clone();
    tauri::async_runtime::spawn(async move {
        let result = exchange(code, state, verifier).await;
        match result {
            Ok(()) => {
                let _ = app.emit(ACCOUNT_CHANGED_EVENT, ());
            }
            Err(e) => {
                let _ = app.emit(ACCOUNT_ERROR_EVENT, e.message);
            }
        }
    });
    Ok(())
}

async fn exchange(code: String, state: String, verifier: String) -> IpcResult<()> {
    let host = std::env::var("COMPUTERNAME").unwrap_or_else(|_| "this computer".into());
    let res = http()?
        .post(format!("{}/v1/account/exchange", link_url()))
        .json(&serde_json::json!({
            "code": code,
            "state": state,
            "code_verifier": verifier,
            "label": format!("desktop on {host}"),
        }))
        .send()
        .await
        .map_err(|e| err("account", format!("Could not reach Pytxo: {e}")))?;
    if !res.status().is_success() {
        return Err(err(
            "account",
            "Sign-in was not accepted. Start sign-in again from Settings.",
        ));
    }
    let body: serde_json::Value = res
        .json()
        .await
        .map_err(|e| err("account", e.to_string()))?;
    let account = StoredAccount {
        account_id: body["account_id"].as_str().unwrap_or_default().to_string(),
        token: body["token"].as_str().unwrap_or_default().to_string(),
        expires_at: body["expires_at"].as_str().unwrap_or_default().to_string(),
    };
    if account.account_id.is_empty() || !account.token.starts_with("pas1_") {
        return Err(err(
            "account",
            "Pytxo returned an unexpected sign-in response.",
        ));
    }
    save(&account)
}

#[tauri::command]
pub async fn account_sign_out(app: AppHandle) -> IpcResult<()> {
    if let Some(account) = load() {
        if let Ok(client) = http() {
            let _ = client
                .delete(format!("{}/v1/account/session", link_url()))
                .bearer_auth(&account.token)
                .send()
                .await;
        }
    }
    forget();
    let _ = app.emit(ACCOUNT_CHANGED_EVENT, ());
    Ok(())
}

/// Shares the newest Apply receipt for `run_id`: counts and outcomes only.
#[tauri::command]
pub async fn account_share_run_receipt(
    state: State<'_, AppState>,
    run_id: String,
    domain_id: Option<String>,
) -> IpcResult<SharedReceiptDto> {
    let account = load().ok_or_else(|| {
        err(
            "account",
            "Sign in from Settings → Account to share receipts.",
        )
    })?;
    let domain = resolve_domain(&state, domain_id)?;
    let dir = pytxo_orchestrate::repo_receipts_dir(std::path::Path::new(&domain))
        .map_err(|e| err("account", e.to_string()))?;
    let target = format!("run:{run_id}");
    let receipt = pytxo_core::list_receipts(&dir)
        .into_iter()
        .map(|(_, r)| r)
        .find(|r| {
            r.predicate.kind == "apply" && r.predicate.target.as_deref() == Some(target.as_str())
        })
        .ok_or_else(|| {
            err(
                "account",
                "This run has no Apply receipt yet. Receipts are written when Apply commits.",
            )
        })?;
    if !receipt.digest_matches() {
        return Err(err(
            "account",
            "The receipt file was edited after it was written; it will not be shared.",
        ));
    }
    let res = http()?
        .post(format!("{}/v1/receipts", link_url()))
        .bearer_auth(&account.token)
        .json(&receipt.share_payload(false))
        .send()
        .await
        .map_err(|e| err("account", format!("Could not reach Pytxo: {e}")))?;
    if res.status() == reqwest::StatusCode::UNAUTHORIZED {
        forget();
        return Err(err(
            "account",
            "Your sign-in expired. Sign in again from Settings → Account.",
        ));
    }
    if res.status() == reqwest::StatusCode::PAYMENT_REQUIRED {
        return Err(err(
            "account",
            "Free accounts keep 25 shared receipts. Delete one at pytxo.com/account, or upgrade to Pytxo Pro for unlimited receipts.",
        ));
    }
    if !res.status().is_success() {
        return Err(err("account", "Sharing failed. Try again later."));
    }
    let body: serde_json::Value = res
        .json()
        .await
        .map_err(|e| err("account", e.to_string()))?;
    Ok(SharedReceiptDto {
        url: body["url"].as_str().unwrap_or_default().to_string(),
        receipt_id: receipt.predicate.id,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    /// keyring 3 silently falls back to a per-entry mock when no platform
    /// store feature is enabled; then nothing survives a new `Entry`.
    #[cfg(windows)]
    #[test]
    fn credentials_persist_across_entries_in_the_os_store() {
        let service = format!("com.pytxo.desktop.test-{}", uuid::Uuid::new_v4().simple());
        Entry::new(&service, ACCOUNT)
            .unwrap()
            .set_password("probe")
            .unwrap();
        let read = Entry::new(&service, ACCOUNT).unwrap().get_password();
        let _ = Entry::new(&service, ACCOUNT).unwrap().delete_credential();
        assert_eq!(read.unwrap(), "probe");
    }

    #[test]
    fn only_account_host_callbacks_match() {
        assert!(is_account_callback(
            "pytxo-deck://account?code=pac1_x&state=y"
        ));
        assert!(!is_account_callback("pytxo-deck://auth?code=x&state=y"));
        assert!(!is_account_callback("https://account.example?code=x"));
    }
}
