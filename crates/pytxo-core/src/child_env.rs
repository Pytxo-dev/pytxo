use std::collections::HashMap;
use std::path::Path;
use std::process::Command;

use crate::billing::{ManagedTransport, ModelRoute};
use crate::moat::PermissionEngine;

/// Environment variables applied to every agent child (PTY or subprocess).
#[derive(Clone, Debug, Default)]
pub struct ChildLaunchEnv {
    vars: HashMap<String, String>,
}

impl ChildLaunchEnv {
    pub fn new() -> Self {
        Self {
            vars: HashMap::new(),
        }
    }

    pub fn set(&mut self, key: impl Into<String>, value: impl Into<String>) {
        self.vars.insert(key.into(), value.into());
    }

    pub fn remove(&mut self, key: &str) {
        self.vars.remove(key);
    }

    pub fn vars(&self) -> &HashMap<String, String> {
        &self.vars
    }

    pub fn with_context_dir(mut self, context_dir: &Path) -> Self {
        self.set("PYTXO_CONTEXT_DIR", context_dir.to_string_lossy());
        self.set("PYTXO_SIGNAL_CORE", "1");
        self
    }

    pub fn with_managed_and_profile(
        mut self,
        transport: &ManagedTransport,
        route: &ModelRoute,
        engine: &PermissionEngine,
    ) -> Self {
        transport.inject_into(&mut self, route);
        engine.sanitize_env_map(&mut self.vars);
        self
    }

    pub fn apply_command(&self, command: &mut Command) {
        // Never inherit a desktop/CLI session bearer into agent children.
        command.env_remove("PYTXO_ULTRA_SESSION");
        for (k, v) in &self.vars {
            if k.eq_ignore_ascii_case("PYTXO_ULTRA_SESSION") {
                continue;
            }
            command.env(k, v);
        }
    }

    pub fn for_each(&self, mut f: impl FnMut(&str, &str)) {
        for (k, v) in &self.vars {
            f(k, v);
        }
    }
}
