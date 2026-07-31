use std::collections::HashMap;
use std::ffi::OsString;
use std::path::Path;
use std::process::Command;

use crate::billing::{ManagedTransport, ModelRoute};
use crate::moat::{PermissionEngine, PermissionProfile};

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
        if engine.profile() != PermissionProfile::DeepSpace {
            transport.inject_into(&mut self, route);
        }
        engine.sanitize_env_map(&mut self.vars);
        self
    }

    pub fn apply_command(&self, command: &mut Command) {
        command.env_clear();
        for (key, value) in Self::inherited_baseline() {
            command.env(key, value);
        }
        for (k, v) in &self.vars {
            if k.eq_ignore_ascii_case("PYTXO_ULTRA_SESSION") {
                continue;
            }
            command.env(k, v);
        }
    }

    /// Minimal host environment required to locate shells and vendor CLIs.
    ///
    /// Agent processes must not inherit Desktop wholesale: a GUI session often
    /// contains unrelated cloud credentials, CI tokens, and editor secrets.
    /// Vendor-owned sessions remain reachable through HOME/USERPROFILE and the
    /// OS credential manager; a selected provider key is injected separately.
    pub fn inherited_baseline() -> Vec<(OsString, OsString)> {
        std::env::vars_os()
            .filter(|(key, _)| {
                key.to_str()
                    .map(|key| is_baseline_key(&key.to_ascii_uppercase()))
                    .unwrap_or(false)
            })
            .collect()
    }

    pub fn for_each(&self, mut f: impl FnMut(&str, &str)) {
        for (k, v) in &self.vars {
            f(k, v);
        }
    }
}

fn is_baseline_key(key: &str) -> bool {
    matches!(
        key,
        "PATH"
            | "PATHEXT"
            | "SYSTEMROOT"
            | "SYSTEMDRIVE"
            | "WINDIR"
            | "COMSPEC"
            | "PSMODULEPATH"
            | "TEMP"
            | "TMP"
            | "TMPDIR"
            | "HOME"
            | "USERPROFILE"
            | "HOMEDRIVE"
            | "HOMEPATH"
            | "LOCALAPPDATA"
            | "APPDATA"
            | "PROGRAMDATA"
            | "PROGRAMFILES"
            | "PROGRAMFILES(X86)"
            | "PROGRAMW6432"
            | "USER"
            | "USERNAME"
            | "LOGNAME"
            | "SHELL"
            | "LANG"
            | "LANGUAGE"
            | "LC_ALL"
            | "TERM"
            | "COLORTERM"
            | "XDG_CONFIG_HOME"
            | "XDG_CACHE_HOME"
            | "XDG_DATA_HOME"
            | "XDG_STATE_HOME"
            | "SSL_CERT_FILE"
            | "SSL_CERT_DIR"
            | "NODE_EXTRA_CA_CERTS"
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn baseline_excludes_unrelated_credentials() {
        assert!(!is_baseline_key("PYTXO_SENTINEL_SECRET"));
        assert!(!is_baseline_key("OPENAI_API_KEY"));
        assert!(!is_baseline_key("GITHUB_TOKEN"));
        assert!(is_baseline_key("PATH"));
        assert!(is_baseline_key("USERPROFILE"));
    }

    #[test]
    fn subprocess_does_not_inherit_parent_sentinel() {
        const SENTINEL: &str = "PYTXO_SENTINEL_SUBPROCESS_SECRET";
        std::env::set_var(SENTINEL, "must-not-leak");

        let mut command = if cfg!(windows) {
            let mut command = Command::new("cmd");
            command.args([
                "/D",
                "/C",
                "if defined PYTXO_SENTINEL_SUBPROCESS_SECRET (exit /b 9) else (exit /b 0)",
            ]);
            command
        } else {
            let mut command = Command::new("sh");
            command.args(["-c", "test -z \"$PYTXO_SENTINEL_SUBPROCESS_SECRET\""]);
            command
        };

        ChildLaunchEnv::new().apply_command(&mut command);
        let status = command.status().expect("spawn sentinel probe");
        std::env::remove_var(SENTINEL);
        assert!(status.success(), "unrelated parent secret reached child");
    }

    #[test]
    fn explicitly_selected_values_survive_clear() {
        let mut launch = ChildLaunchEnv::new();
        launch.set("PYTXO_EXPLICIT_TEST", "selected");
        let mut command = Command::new(if cfg!(windows) { "cmd" } else { "sh" });
        launch.apply_command(&mut command);
        let explicit = command
            .get_envs()
            .find(|(key, _)| *key == std::ffi::OsStr::new("PYTXO_EXPLICIT_TEST"))
            .and_then(|(_, value)| value);
        assert_eq!(explicit, Some(std::ffi::OsStr::new("selected")));
    }
}
