use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};

use crate::billing::{BillingConfig, BillingMode};
use crate::cloud::{CloudConfig, McpHubConfig};
use crate::coordinator::CoordinatorConfig;
use crate::execution::ExecutionBackend;
use crate::moat::{FidelityTier, IsolationMode, PermissionProfile};

/// Blast Shield overlay tuning ([[blast-shield]], Phase 33).
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct BlastConfig {
    /// Top-level directory names skipped in overlay copy / sparse lowerdir (Phase 33).
    #[serde(default = "default_sparse_exclude")]
    pub sparse_exclude: Vec<String>,
    /// When true and doctor overlay probe passes, Orbit may use kernel overlay instead of worktree (Phase 62).
    #[serde(default = "default_prefer_kernel_overlay")]
    pub prefer_kernel_overlay: bool,
}

fn default_prefer_kernel_overlay() -> bool {
    true
}

fn default_sparse_exclude() -> Vec<String> {
    vec![
        "node_modules".into(),
        ".git".into(),
        "target".into(),
        "dist".into(),
        "build".into(),
    ]
}

impl Default for BlastConfig {
    fn default() -> Self {
        Self {
            sparse_exclude: default_sparse_exclude(),
            prefer_kernel_overlay: default_prefer_kernel_overlay(),
        }
    }
}
use crate::{AgentSpec, PytxoError, Result, Task, TaskId};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct PytxoConfig {
    #[serde(default = "default_max_agents")]
    pub max_agents: usize,
    #[serde(default = "default_worktree_dir")]
    pub worktree_dir: PathBuf,
    #[serde(default = "default_data_dir")]
    pub data_dir: PathBuf,
    #[serde(default = "default_true")]
    pub fail_fast: bool,
    #[serde(default)]
    pub agent: Vec<AgentSpec>,
    #[serde(default)]
    pub task: Vec<TaskConfig>,
    #[serde(default = "default_true")]
    pub sanitize: bool,
    #[serde(default)]
    pub dag_explicit_deps: bool,
    #[serde(default = "default_true")]
    pub signal_core: bool,
    #[serde(default = "default_fidelity")]
    pub signal_fidelity: FidelityTier,
    #[serde(default = "default_isolation")]
    pub isolation: IsolationMode,
    #[serde(default = "default_permission_profile")]
    pub permission_profile: PermissionProfile,
    #[serde(default)]
    pub billing: BillingConfig,
    #[serde(default)]
    pub cloud: CloudConfig,
    #[serde(default)]
    pub mcp_hub: McpHubConfig,
    #[serde(default = "default_execution_backend")]
    pub execution_backend: ExecutionBackend,
    #[serde(default = "default_pty_rows")]
    pub pty_rows: u16,
    #[serde(default = "default_pty_cols")]
    pub pty_cols: u16,
    /// Pump Race Shield stdin queue into subprocess children ([[race-shield]] Phase 22b).
    #[serde(default)]
    pub subprocess_stdin: bool,
    /// OSS Core tier agent cap ([[tiers-hobbyist-pro-max]]).
    #[serde(default = "default_tier_max_agents")]
    pub tier_max_agents: usize,
    /// Optional NL mission planner ([[ADR-0012-hypervisor-shell-default-ux]]).
    #[serde(default)]
    pub planner: PlannerConfig,
    /// Advisory model used for planning, route proposals, and diagnosis.
    #[serde(default)]
    pub coordinator: CoordinatorConfig,
    #[serde(default)]
    pub blast: BlastConfig,
    /// Out-of-band folder trust ceiling. This is runtime policy metadata and is
    /// never read from or written to repository-controlled TOML.
    #[serde(skip)]
    pub permission_ceiling: Option<PermissionProfile>,
    /// Repository-requested run profile retained for honest enforcement receipts.
    #[serde(skip)]
    pub requested_permission_profile: Option<PermissionProfile>,
}

#[derive(Clone, Debug, Serialize, Deserialize, Default)]
pub struct PlannerConfig {
    #[serde(default)]
    pub enabled: bool,
    /// `heuristic` (default) or `signal` for Signal Core graph-backed depends_on.
    #[serde(default = "default_planner_mode")]
    pub mode: String,
}

fn default_planner_mode() -> String {
    "heuristic".to_string()
}

fn default_true() -> bool {
    true
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct TaskConfig {
    pub id: String,
    pub agent: String,
    pub paths: Vec<String>,
    #[serde(default)]
    pub depends_on: Vec<String>,
    /// Modular project root label ([[ADR-0011-modular-project-manifest]]); paths
    /// resolve under that labeled root. `None` uses the run's primary root.
    #[serde(default)]
    pub root: Option<String>,
    /// Per-task Signal Core fidelity override ([[closed-loop-fidelity]]).
    #[serde(default)]
    pub signal_fidelity: Option<FidelityTier>,
    /// Post-task verify shell commands (mission loop).
    #[serde(default)]
    pub verify: Vec<String>,
}

fn default_max_agents() -> usize {
    3
}

fn default_worktree_dir() -> PathBuf {
    PathBuf::from(".pytxo/worktrees")
}

fn default_data_dir() -> PathBuf {
    PathBuf::from(".pytxo/data")
}

fn default_fidelity() -> FidelityTier {
    FidelityTier::Low
}

fn default_isolation() -> IsolationMode {
    IsolationMode::Worktree
}

fn default_permission_profile() -> PermissionProfile {
    PermissionProfile::Orbit
}

fn default_execution_backend() -> ExecutionBackend {
    ExecutionBackend::Pty
}

fn default_pty_rows() -> u16 {
    24
}

fn default_pty_cols() -> u16 {
    80
}

/// Local Core runs up to eight concurrent workers (one mixed-CLI fleet).
fn default_tier_max_agents() -> usize {
    8
}

impl Default for PytxoConfig {
    fn default() -> Self {
        Self {
            max_agents: default_max_agents(),
            worktree_dir: default_worktree_dir(),
            data_dir: default_data_dir(),
            fail_fast: true,
            agent: Vec::new(),
            task: Vec::new(),
            sanitize: true,
            dag_explicit_deps: false,
            signal_core: true,
            signal_fidelity: FidelityTier::Low,
            isolation: IsolationMode::Worktree,
            permission_profile: PermissionProfile::Orbit,
            billing: BillingConfig::default(),
            cloud: CloudConfig::default(),
            mcp_hub: McpHubConfig::default(),
            execution_backend: ExecutionBackend::Pty,
            pty_rows: default_pty_rows(),
            pty_cols: default_pty_cols(),
            subprocess_stdin: false,
            tier_max_agents: default_tier_max_agents(),
            planner: PlannerConfig::default(),
            coordinator: CoordinatorConfig::default(),
            blast: BlastConfig::default(),
            permission_ceiling: None,
            requested_permission_profile: None,
        }
    }
}

impl PytxoConfig {
    pub fn planner_enabled(&self) -> bool {
        self.planner.enabled
    }

    pub fn billing_mode(&self) -> BillingMode {
        self.billing.mode
    }

    pub fn load(path: &Path) -> Result<Self> {
        let raw = std::fs::read_to_string(path)
            .map_err(|e| PytxoError::Config(format!("read {}: {e}", path.display())))?;
        toml::from_str(&raw).map_err(|e| PytxoError::Config(format!("parse toml: {e}")))
    }

    pub fn tasks(&self) -> Vec<Task> {
        self.task
            .iter()
            .map(|t| Task {
                id: TaskId(t.id.clone()),
                agent: t.agent.clone(),
                paths: t.paths.clone(),
                depends_on: t.depends_on.clone(),
                root: t.root.clone(),
                signal_fidelity: t.signal_fidelity,
                verify: t.verify.clone(),
            })
            .collect()
    }

    /// SQLite path relative to the process CWD (legacy). Prefer [`Self::db_path_at`].
    pub fn db_path(&self) -> PathBuf {
        self.data_dir.join("pytxo.db")
    }

    /// SQLite path anchored at a repository root (matches run/hypervisor persistence).
    pub fn db_path_at(&self, repo_root: &Path) -> PathBuf {
        repo_root.join(&self.data_dir).join("pytxo.db")
    }

    /// Active-run marker relative to CWD (legacy). Prefer [`Self::state_path_at`].
    pub fn state_path(&self) -> PathBuf {
        self.data_dir.join("active_run.json")
    }

    pub fn state_path_at(&self, repo_root: &Path) -> PathBuf {
        repo_root.join(&self.data_dir).join("active_run.json")
    }

    pub fn resolve_profile_for_agent(&self, agent_name: &str) -> PermissionProfile {
        let requested = self
            .agent
            .iter()
            .find(|a| a.name == agent_name)
            .and_then(|a| a.permission_profile)
            .unwrap_or(self.permission_profile);
        self.permission_ceiling
            .map(|ceiling| requested.capped_at(ceiling))
            .unwrap_or(requested)
    }

    pub fn agent_profile_map(&self) -> std::collections::HashMap<String, PermissionProfile> {
        self.agent
            .iter()
            .map(|a| (a.name.clone(), self.resolve_profile_for_agent(&a.name)))
            .collect()
    }

    /// Profiles requested by repository configuration before folder/org ceilings.
    pub fn requested_agent_profile_map(
        &self,
    ) -> std::collections::HashMap<String, PermissionProfile> {
        let requested_default = self
            .requested_permission_profile
            .unwrap_or(self.permission_profile);
        self.agent
            .iter()
            .map(|agent| {
                (
                    agent.name.clone(),
                    agent.permission_profile.unwrap_or(requested_default),
                )
            })
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_max_agents_is_three() {
        let cfg = PytxoConfig::default();
        assert_eq!(cfg.max_agents, 3);
    }

    #[test]
    fn omitted_fail_fast_defaults_true_in_minimal_config() {
        let cfg: PytxoConfig = toml::from_str("").unwrap();
        assert!(cfg.fail_fast);
    }

    #[test]
    fn omitted_fail_fast_defaults_true_in_legacy_agent_config() {
        let cfg: PytxoConfig = toml::from_str(
            r#"
max_agents = 2

[[agent]]
name = "builder"
"#,
        )
        .unwrap();
        assert!(cfg.fail_fast);
    }

    #[test]
    fn explicit_false_fail_fast_remains_supported() {
        let cfg: PytxoConfig = toml::from_str("fail_fast = false").unwrap();
        assert!(!cfg.fail_fast);
    }

    #[test]
    fn coordinator_profile_parses_independently_from_worker_agents() {
        let cfg: PytxoConfig = toml::from_str(
            r#"
[coordinator]
provider = "ollama"
model = "qwen3:8b"
transport = "direct"

[[agent]]
name = "builder"
provider = "deepseek"
model = "deepseek-chat"
"#,
        )
        .unwrap();

        assert_eq!(cfg.coordinator.provider, "ollama");
        assert_eq!(cfg.coordinator.model, "qwen3:8b");
        assert_eq!(
            cfg.coordinator.transport,
            crate::CoordinatorTransport::Direct
        );
        assert_eq!(cfg.agent[0].provider.as_deref(), Some("deepseek"));
        assert_eq!(cfg.agent[0].model.as_deref(), Some("deepseek-chat"));
    }

    #[test]
    fn db_path_at_anchors_to_repo_root() {
        let cfg = PytxoConfig::default();
        let repo = PathBuf::from("/tmp/my-repo");
        assert_eq!(
            cfg.db_path_at(&repo),
            PathBuf::from("/tmp/my-repo/.pytxo/data/pytxo.db")
        );
        assert_eq!(
            cfg.state_path_at(&repo),
            PathBuf::from("/tmp/my-repo/.pytxo/data/active_run.json")
        );
    }

    #[test]
    fn blast_sparse_exclude_defaults() {
        let cfg = PytxoConfig::default();
        assert!(cfg
            .blast
            .sparse_exclude
            .contains(&"node_modules".to_string()));
        assert!(cfg.blast.sparse_exclude.contains(&".git".to_string()));
    }

    #[test]
    fn permission_profile_toml_round_trip() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("pytxo.toml");
        std::fs::write(
            &path,
            r#"
permission_profile = "galaxy"
max_agents = 2

[[agent]]
name = "builder"
permission_profile = "deep_space"
"#,
        )
        .unwrap();
        let cfg = PytxoConfig::load(&path).unwrap();
        assert_eq!(cfg.permission_profile, PermissionProfile::Galaxy);
        assert_eq!(
            cfg.resolve_profile_for_agent("builder"),
            PermissionProfile::DeepSpace
        );
        assert_eq!(
            cfg.resolve_profile_for_agent("unknown"),
            PermissionProfile::Galaxy
        );
    }
}
