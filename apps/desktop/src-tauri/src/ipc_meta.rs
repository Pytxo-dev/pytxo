use serde::Serialize;
use std::process::Command;

use pytxo_core::PytxoConfig;
use pytxo_orchestrate::effective_entitlements;

use crate::ipc_error::{map_config_err, IpcResult};

#[derive(Serialize)]
pub struct EntitlementStatusDto {
    pub tier: String,
    pub max_agents: usize,
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
pub fn entitlement_status() -> IpcResult<EntitlementStatusDto> {
    let cfg = PytxoConfig::default();
    let ent = effective_entitlements(&cfg).map_err(map_config_err)?;
    Ok(EntitlementStatusDto {
        tier: ent.tier,
        max_agents: ent.max_agents,
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
