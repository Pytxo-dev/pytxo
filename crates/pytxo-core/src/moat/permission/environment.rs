//! Child environment policy ([[permission-profile-engine]], Phase 30).

use std::collections::HashMap;
use std::process::Command;

use super::PermissionProfile;

pub trait EnvironmentPolicy {
    fn profile(&self) -> PermissionProfile;

    fn sanitize_child_env(&self, command: &mut Command);

    fn sanitize_env_map(&self, vars: &mut HashMap<String, String>);
}

pub struct EnvironmentPolicyEngine {
    profile: PermissionProfile,
}

impl EnvironmentPolicyEngine {
    pub fn new(profile: PermissionProfile) -> Self {
        Self { profile }
    }

    pub fn should_strip_env_key(upper: &str) -> bool {
        upper.starts_with("SSH_") || upper == "GIT_SSH_COMMAND" || upper == "SSH_AUTH_SOCK"
    }
}

impl EnvironmentPolicy for EnvironmentPolicyEngine {
    fn profile(&self) -> PermissionProfile {
        self.profile
    }

    fn sanitize_child_env(&self, command: &mut Command) {
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

    fn sanitize_env_map(&self, vars: &mut HashMap<String, String>) {
        if !matches!(
            self.profile,
            PermissionProfile::DeepSpace | PermissionProfile::Orbit
        ) {
            return;
        }
        vars.retain(|key, _| !Self::should_strip_env_key(&key.to_ascii_uppercase()));
    }
}
