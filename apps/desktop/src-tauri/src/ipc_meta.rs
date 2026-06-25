use serde::Serialize;
use std::path::Path;
use std::process::Command;

use pytxo_core::DomainId;
use pytxo_orchestrate::{effective_entitlements, fetch_link_wallet_balance};
use tauri::State;

use crate::ipc::{load_cfg_for_domain, open_store_for_domain, resolve_domain, AppState};
use crate::ipc_error::{map_config_err, IpcResult};

#[derive(Serialize)]
pub struct EntitlementStatusDto {
    pub tier: String,
    pub max_agents: usize,
    pub cloud_enabled: bool,
    pub wallet_balance_microcredits: Option<i64>,
    pub permission_ceiling: Option<String>,
    pub subscription_portal_url: Option<String>,
}

#[tauri::command]
pub fn ipc_version() -> String {
    env!("CARGO_PKG_VERSION").to_string()
}

#[tauri::command]
pub fn check_pytxo_cli() -> bool {
    if let Ok(sidecar) = std::env::var("PYTXO_SIDECAR") {
        if !sidecar.is_empty() && std::path::Path::new(&sidecar).exists() {
            return true;
        }
    }
    Command::new(if cfg!(windows) { "where" } else { "which" })
        .arg("pytxo")
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false)
}

#[tauri::command]
pub fn entitlement_status(
    state: State<'_, AppState>,
    domain_id: Option<String>,
) -> IpcResult<EntitlementStatusDto> {
    let domain = resolve_domain(&state, domain_id)?;
    let cfg = load_cfg_for_domain(&domain, &state)?;
    let ent = effective_entitlements(&cfg).map_err(map_config_err)?;
    let mut wallet_balance = open_store_for_domain(&cfg, &domain)
        .ok()
        .and_then(|store| {
            DomainId::from_repo_root(Path::new(&domain))
                .ok()
                .and_then(|d| store.wallet_balance_microcredits(&d).ok())
        });
    if ent.tier == "ultra" && cfg.billing.link_reconcile_enabled() {
        if let Ok(remote) = fetch_link_wallet_balance(cfg.billing.link_base_url()) {
            wallet_balance = Some(remote);
        }
    }
    let portal = if ent.tier != "core" {
        Some("https://pytxo.com/account#subscription".into())
    } else {
        Some("https://pytxo.com/plans".into())
    };
    Ok(EntitlementStatusDto {
        tier: ent.tier,
        max_agents: ent.max_agents,
        cloud_enabled: ent.cloud_enabled,
        wallet_balance_microcredits: wallet_balance,
        permission_ceiling: ent.permission_ceiling.map(|p| p.as_str().to_string()),
        subscription_portal_url: portal,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ipc_version_matches_crate() {
        assert_eq!(ipc_version(), env!("CARGO_PKG_VERSION"));
    }
}
