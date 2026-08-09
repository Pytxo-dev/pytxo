use pytxo_core::{DomainId, IsolationMode, PermissionProfile, PytxoError, Result};

#[derive(Clone, Debug, serde::Deserialize, serde::Serialize)]
pub struct EnforcementSurfaceReceipt {
    pub status: String,
    pub mechanism: String,
    pub detail: String,
}

#[derive(Clone, Debug, serde::Deserialize, serde::Serialize)]
pub struct PermissionEnforcementReceipt {
    pub requested_profile: String,
    pub effective_profile: String,
    pub execution_domain: String,
    pub workspace_isolation: EnforcementSurfaceReceipt,
    pub host_filesystem_boundary: EnforcementSurfaceReceipt,
    pub network: EnforcementSurfaceReceipt,
    pub apply_boundary: EnforcementSurfaceReceipt,
}

pub fn permission_enforcement_receipt(
    requested_profile: PermissionProfile,
    effective_profile: PermissionProfile,
    domain_id: &DomainId,
    isolation_mode: IsolationMode,
    sparse_exclude: &[String],
) -> Result<PermissionEnforcementReceipt> {
    let mechanism = crate::network_isolation::isolation_mechanism();
    let receipt = permission_enforcement_receipt_for_mechanism(
        requested_profile,
        effective_profile,
        domain_id,
        isolation_mode,
        sparse_exclude,
        mechanism,
    )?;
    if effective_profile == PermissionProfile::DeepSpace {
        let (blocked, detail) = crate::network_isolation::doctor_deepspace_socket_probe();
        if !blocked {
            return Err(PytxoError::Runner(format!(
                "DeepSpace refused to start because socket isolation was not enforced: {detail}"
            )));
        }
    }
    Ok(receipt)
}

pub fn permission_enforcement_receipt_for_mechanism(
    requested_profile: PermissionProfile,
    effective_profile: PermissionProfile,
    domain_id: &DomainId,
    isolation_mode: IsolationMode,
    sparse_exclude: &[String],
    network_mechanism: &str,
) -> Result<PermissionEnforcementReceipt> {
    let workspace_isolation = if effective_profile == PermissionProfile::Supernova {
        surface(
            "bypassed",
            "host-direct",
            "Supernova runs in the primary repository by explicit policy",
        )
    } else {
        surface(
            "enforced",
            &crate::blast::isolation_backend_label(isolation_mode, sparse_exclude),
            "agent mutations remain outside the primary repository until run-level Apply",
        )
    };
    let host_filesystem_boundary = if effective_profile == PermissionProfile::Supernova {
        surface(
            "bypassed",
            "host-user",
            "the child intentionally retains host-user filesystem access",
        )
    } else {
        surface(
            "advisory",
            "child-cwd-and-policy-gates",
            "workspace cwd and Pytxo-mediated path gates are active; syscall sandboxing is not",
        )
    };
    let network = match effective_profile {
        PermissionProfile::DeepSpace => {
            if !matches!(
                network_mechanism,
                "linux-netns-unshare"
                    | "macos-sandbox-exec"
                    | "windows-wfp-rule-present"
            ) {
                return Err(PytxoError::Runner(format!(
                    "DeepSpace refused to start because network isolation is unavailable: {network_mechanism}"
                )));
            }
            surface(
                "enforced",
                network_mechanism,
                "socket isolation is installed and must pass an egress probe before dispatch",
            )
        }
        PermissionProfile::Orbit => surface(
            "advisory",
            "spawn-command-policy",
            "known network commands are denied, but arbitrary child socket syscalls are not intercepted",
        ),
        PermissionProfile::Galaxy => surface(
            "advisory",
            "hitl-command-and-mcp-gates",
            "recognized public egress requires approval; arbitrary child socket syscalls are not intercepted",
        ),
        PermissionProfile::Supernova => surface(
            "bypassed",
            "host-network",
            "full host-user network access is explicitly enabled",
        ),
    };
    let apply_boundary = match effective_profile {
        PermissionProfile::DeepSpace => surface(
            "enforced",
            "non-flushable",
            "DeepSpace results cannot be applied to the primary repository",
        ),
        PermissionProfile::Orbit | PermissionProfile::Galaxy => surface(
            "enforced",
            "reviewed-run-atomic-apply",
            "all claimed run changes are validated and in-process write failures roll back as one guarded operation",
        ),
        PermissionProfile::Supernova => {
            surface("bypassed", "host-direct", "there is no deferred Apply step")
        }
    };

    Ok(PermissionEnforcementReceipt {
        requested_profile: requested_profile.as_str().into(),
        effective_profile: effective_profile.as_str().into(),
        execution_domain: domain_id.0.clone(),
        workspace_isolation,
        host_filesystem_boundary,
        network,
        apply_boundary,
    })
}

fn surface(status: &str, mechanism: &str, detail: &str) -> EnforcementSurfaceReceipt {
    EnforcementSurfaceReceipt {
        status: status.into(),
        mechanism: mechanism.into(),
        detail: detail.into(),
    }
}
