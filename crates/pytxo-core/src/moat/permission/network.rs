//! Network egress policy trait ([[permission-profile-engine]], Phase 30).

use super::PermissionProfile;

/// Gates outbound network use at spawn and (future) runtime hooks.
pub trait NetworkPolicy {
    fn profile(&self) -> PermissionProfile;

    /// Whether egress to `host:port` is allowed without HITL.
    fn egress_allowed(&self, host: &str, port: u16) -> bool;

    /// Whether a spawn command should be treated as network egress (Phase 32 expands).
    fn spawn_egress_allowed(&self, cmd: &str) -> bool;
}

pub struct NetworkPolicyEngine {
    profile: PermissionProfile,
}

impl NetworkPolicyEngine {
    pub fn new(profile: PermissionProfile) -> Self {
        Self { profile }
    }
}

impl NetworkPolicy for NetworkPolicyEngine {
    fn profile(&self) -> PermissionProfile {
        self.profile
    }

    fn egress_allowed(&self, host: &str, port: u16) -> bool {
        let _ = (host, port);
        match self.profile {
            PermissionProfile::DeepSpace | PermissionProfile::Orbit => false,
            PermissionProfile::Galaxy => port == 5432 || port == 6379 || port == 8080,
            PermissionProfile::Supernova => true,
        }
    }

    fn spawn_egress_allowed(&self, cmd: &str) -> bool {
        let lower = cmd.to_ascii_lowercase();
        let networkish = lower.contains("curl ")
            || lower.contains("wget ")
            || lower.contains("nc ")
            || lower.contains("ncat ")
            || lower.starts_with("ssh ");
        if !networkish {
            return true;
        }
        matches!(
            self.profile,
            PermissionProfile::Supernova | PermissionProfile::Galaxy
        )
    }
}
