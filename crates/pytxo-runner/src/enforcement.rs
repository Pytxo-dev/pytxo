use std::time::Duration;

use pytxo_core::{
    DomainId, ExecutionBackend, IsolationMode, PermissionProfile, PytxoError, Result,
};

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

/// Evidence for the post-agent verification actor.
///
/// Verification deliberately has its own receipt because it is a second code-execution
/// boundary. Reusing the run receipt would incorrectly imply that the agent's launch
/// environment, timeout, and output controls also covered verifier processes.
#[derive(Clone, Debug, serde::Deserialize, serde::Serialize)]
pub struct VerificationEnforcementReceipt {
    pub actor: String,
    pub effective_profile: String,
    pub execution_domain: String,
    pub execution_backend: String,
    pub workspace_isolation: EnforcementSurfaceReceipt,
    pub host_filesystem_boundary: EnforcementSurfaceReceipt,
    pub environment: EnforcementSurfaceReceipt,
    pub network: EnforcementSurfaceReceipt,
    pub timeout: EnforcementSurfaceReceipt,
    pub output_capture: EnforcementSurfaceReceipt,
}

pub fn verification_enforcement_receipt(
    effective_profile: PermissionProfile,
    domain_id: &DomainId,
    execution_backend: ExecutionBackend,
    workspace_isolated: bool,
    timeout: Duration,
    output_limit_bytes: usize,
) -> Result<VerificationEnforcementReceipt> {
    let workspace_isolation = if workspace_isolated {
        surface(
            "enforced",
            "agent-workspace",
            "verification cwd is the same isolated execution-domain workspace as the agent",
        )
    } else {
        surface(
            "bypassed",
            "host-direct",
            "Supernova verification runs in the primary repository by explicit policy",
        )
    };
    let host_filesystem_boundary = if effective_profile == PermissionProfile::Supernova {
        surface(
            "bypassed",
            "host-user",
            "the verifier intentionally retains host-user filesystem access",
        )
    } else {
        surface(
            "advisory",
            "child-cwd-and-policy-gates",
            "workspace cwd is enforced; syscall-level host filesystem isolation is unavailable",
        )
    };
    let network = match effective_profile {
        PermissionProfile::DeepSpace => {
            let mechanism = crate::network_isolation::isolation_mechanism();
            if !matches!(
                mechanism,
                "linux-netns-unshare" | "macos-sandbox-exec" | "windows-wfp-rule-present"
            ) {
                return Err(PytxoError::Runner(format!(
                    "DeepSpace verification refused because network isolation is unavailable: {mechanism}"
                )));
            }
            surface(
                "enforced",
                mechanism,
                "the verifier process uses the same socket-isolation mechanism as the agent",
            )
        }
        PermissionProfile::Orbit => surface(
            "advisory",
            "spawn-command-policy",
            "known network commands are denied; arbitrary child socket syscalls are not intercepted",
        ),
        PermissionProfile::Galaxy => surface(
            "advisory",
            "hitl-command-gate",
            "recognized public egress requires approval; arbitrary child socket syscalls are not intercepted",
        ),
        PermissionProfile::Supernova => surface(
            "bypassed",
            "host-network",
            "full host-user network access is explicitly enabled",
        ),
    };
    let execution_backend = match execution_backend {
        ExecutionBackend::Pty => "local-bounded-subprocess",
        ExecutionBackend::Subprocess => "local-bounded-subprocess",
        // Remote exec currently has no cancellable timeout contract. The runner records
        // this boundary and then fails closed instead of silently verifying on the host.
        ExecutionBackend::Cloud => "remote-unsupported-fail-closed",
    };

    Ok(VerificationEnforcementReceipt {
        actor: "verification".into(),
        effective_profile: effective_profile.as_str().into(),
        execution_domain: domain_id.0.clone(),
        execution_backend: execution_backend.into(),
        workspace_isolation,
        host_filesystem_boundary,
        environment: surface(
            "enforced",
            "clear-and-minimal-baseline",
            "the verifier does not inherit unrelated parent credentials or agent provider secrets",
        ),
        network,
        timeout: surface(
            "enforced",
            "process-tree-deadline",
            &format!("deadline_ms={}", timeout.as_millis()),
        ),
        output_capture: surface(
            "enforced",
            "bounded-pipe-drain",
            &format!("max_bytes_per_stream={output_limit_bytes}"),
        ),
    })
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
