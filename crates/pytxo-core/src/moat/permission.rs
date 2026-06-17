use std::path::Path;
use std::process::Command;

use serde::{Deserialize, Serialize};

use crate::moat::FidelityTier;
use crate::{canonical_repo_root, PytxoError, Result};

/// Local capability ladder ([[permission-profile-engine]], ADR-0008).
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PermissionProfile {
    /// Tier 1 — air-gapped process directory
    DeepSpace,
    /// Tier 2 — default engineering; CoW bubble, approve-to-flush
    #[default]
    Orbit,
    /// Tier 3 — host tools + HITL for high-risk actions
    Galaxy,
    /// Tier 4 — full host user privileges
    Supernova,
}

impl PermissionProfile {
    pub fn parse(s: &str) -> Option<Self> {
        match s.to_ascii_lowercase().as_str() {
            "deep_space" => Some(Self::DeepSpace),
            "orbit" => Some(Self::Orbit),
            "galaxy" => Some(Self::Galaxy),
            "supernova" => Some(Self::Supernova),
            _ => None,
        }
    }

    pub fn as_str(self) -> &'static str {
        match self {
            Self::DeepSpace => "deep_space",
            Self::Orbit => "orbit",
            Self::Galaxy => "galaxy",
            Self::Supernova => "supernova",
        }
    }
}

/// Stable execution-domain id from canonical repo root ([[execution-domains]]).
#[derive(Clone, Debug, Eq, Hash, PartialEq, Serialize, Deserialize)]
pub struct DomainId(pub String);

impl DomainId {
    pub fn from_repo_root(repo: &Path) -> Result<Self> {
        let canon = canonical_repo_root(repo).map_err(PytxoError::Io)?;
        Ok(Self(canon.to_string_lossy().into_owned()))
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

/// Orchestration-side policy facade for a single effective profile.
#[derive(Clone, Copy, Debug)]
pub struct PermissionEngine {
    profile: PermissionProfile,
}

impl PermissionEngine {
    pub fn new(profile: PermissionProfile) -> Self {
        Self { profile }
    }

    pub fn profile(&self) -> PermissionProfile {
        self.profile
    }

    pub fn max_fidelity(&self, configured: FidelityTier) -> FidelityTier {
        match self.profile {
            PermissionProfile::DeepSpace => FidelityTier::Low,
            _ => configured,
        }
    }

    pub fn flush_requires_approval(&self) -> bool {
        matches!(
            self.profile,
            PermissionProfile::DeepSpace | PermissionProfile::Orbit | PermissionProfile::Galaxy
        )
    }

    pub fn may_flush(&self) -> bool {
        !matches!(self.profile, PermissionProfile::DeepSpace)
    }

    pub fn use_worktree_isolation(&self) -> bool {
        !matches!(self.profile, PermissionProfile::Supernova)
    }

    pub fn sanitize_child_env(&self, command: &mut Command) {
        if !matches!(
            self.profile,
            PermissionProfile::DeepSpace | PermissionProfile::Orbit
        ) {
            return;
        }
        let keys: Vec<String> = command
            .get_envs()
            .filter_map(|(k, _)| k.to_str().map(str::to_string))
            .collect();
        for key in &keys {
            let upper = key.to_ascii_uppercase();
            if Self::should_strip_env_key(&upper) {
                command.env_remove(key);
            }
        }
    }

    pub fn sanitize_env_map(&self, vars: &mut std::collections::HashMap<String, String>) {
        if !matches!(
            self.profile,
            PermissionProfile::DeepSpace | PermissionProfile::Orbit
        ) {
            return;
        }
        vars.retain(|key, _| !Self::should_strip_env_key(&key.to_ascii_uppercase()));
    }

    fn should_strip_env_key(upper: &str) -> bool {
        upper.starts_with("SSH_") || upper == "GIT_SSH_COMMAND" || upper == "SSH_AUTH_SOCK"
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_permission_profiles() {
        assert_eq!(
            PermissionProfile::parse("orbit"),
            Some(PermissionProfile::Orbit)
        );
        assert_eq!(
            PermissionProfile::parse("deep_space"),
            Some(PermissionProfile::DeepSpace)
        );
        assert!(PermissionProfile::parse("invalid").is_none());
    }

    #[test]
    fn engine_orbit_flush_rules() {
        let e = PermissionEngine::new(PermissionProfile::Orbit);
        assert!(e.flush_requires_approval());
        assert!(e.may_flush());
        assert!(e.use_worktree_isolation());
    }

    #[test]
    fn engine_deep_space_caps_fidelity() {
        let e = PermissionEngine::new(PermissionProfile::DeepSpace);
        assert_eq!(e.max_fidelity(FidelityTier::High), FidelityTier::Low);
        assert!(!e.may_flush());
    }

    #[test]
    fn engine_supernova_skips_worktree() {
        let e = PermissionEngine::new(PermissionProfile::Supernova);
        assert!(!e.use_worktree_isolation());
        assert!(!e.flush_requires_approval());
        assert!(e.may_flush());
    }
}
