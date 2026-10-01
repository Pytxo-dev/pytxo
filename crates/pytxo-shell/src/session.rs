use std::collections::HashMap;
use std::path::PathBuf;

use pytxo_catalog::ModelCatalog;
use pytxo_core::{ExecutionPlan, PytxoConfig, RunId, Task};
use pytxo_core::{PermissionProfile, ProviderId};
use pytxo_orchestrate::{
    dispatch, dry_run_with_tasks, is_repo_trusted, load_config_for_repo, logs, plan_tasks,
    resolve_repo_root, run_doctor, status_json, trust_repo, RunOptions,
};
use pytxo_planner::plan_mission;

use crate::command::{help_text, ModelsSub, ShellInput, SlashCommand};

/// Events streamed back to TUI / CLI surfaces.
#[derive(Clone, Debug)]
pub enum ShellEvent {
    Output(String),
    PlanPreview(String),
    RunStarted(RunId),
    RunFinished(RunId),
    Error(String),
}

/// Hypervisor Shell session state (control plane, no UI).
pub struct ShellSession {
    pub config_path: Option<PathBuf>,
    pub repo: PathBuf,
    pub config: PytxoConfig,
    pub last_plan: Option<ExecutionPlan>,
    pub last_tasks: Vec<Task>,
    pub last_task_prompts: HashMap<String, String>,
    pub active_run: Option<RunId>,
    pub default_cmd: String,
}

impl ShellSession {
    pub fn new(config_path: Option<PathBuf>, repo: Option<PathBuf>) -> anyhow::Result<Self> {
        let repo = resolve_repo_root(repo.as_deref())?;
        let config = load_config_for_repo(config_path.as_deref(), &repo)?;
        Ok(Self {
            config_path,
            repo,
            config,
            last_plan: None,
            last_tasks: Vec::new(),
            last_task_prompts: HashMap::new(),
            active_run: None,
            default_cmd: "echo pytxo".into(),
        })
    }

    pub fn is_trusted(&self) -> bool {
        is_repo_trusted(&self.repo).unwrap_or(false)
    }

    pub async fn handle(&mut self, input: ShellInput) -> Vec<ShellEvent> {
        match input {
            ShellInput::Empty => vec![],
            ShellInput::ReplExit => vec![ShellEvent::Output("bye".into())],
            ShellInput::Slash(cmd) => self.handle_slash(cmd).await,
            ShellInput::Mission(text) => self.handle_mission(&text).await,
        }
    }

    async fn handle_mission(&mut self, text: &str) -> Vec<ShellEvent> {
        match plan_mission(text, &self.repo, &self.config) {
            Ok(plan) => {
                self.last_tasks = plan.tasks.clone();
                self.last_task_prompts = plan.task_prompts;
                match plan_tasks(&plan.tasks, &self.config) {
                    Ok(exec) => {
                        self.last_plan = Some(exec.clone());
                        let preview = match dry_run_with_tasks(
                            self.config_path.clone(),
                            Some(self.repo.clone()),
                            self.config.max_agents,
                            Some(self.last_tasks.clone()),
                        ) {
                            Ok(json) => json,
                            Err(e) => return vec![ShellEvent::Error(e.to_string())],
                        };
                        vec![
                            ShellEvent::Output(
                                "Mission decomposed — review plan below, then /run to dispatch."
                                    .into(),
                            ),
                            ShellEvent::PlanPreview(preview),
                        ]
                    }
                    Err(e) => vec![ShellEvent::Error(e.to_string())],
                }
            }
            Err(e) => vec![ShellEvent::Error(e.to_string())],
        }
    }

    async fn handle_slash(&mut self, cmd: SlashCommand) -> Vec<ShellEvent> {
        match cmd {
            SlashCommand::Help => vec![ShellEvent::Output(help_text())],
            SlashCommand::Doctor => match run_doctor(Some(&self.repo)) {
                Ok(report) => {
                    let mut lines = Vec::new();
                    for c in &report.checks {
                        let mark = if c.ok { "ok" } else { "FAIL" };
                        lines.push(format!("[{mark}] {} — {}", c.name, c.detail));
                    }
                    if report.all_ok() {
                        lines.push("All checks passed.".into());
                    } else {
                        lines.push("Some checks failed.".into());
                    }
                    vec![ShellEvent::Output(lines.join("\n"))]
                }
                Err(e) => vec![ShellEvent::Error(e.to_string())],
            },
            SlashCommand::DryRun { agents } => {
                let runtime = if self.last_tasks.is_empty() {
                    None
                } else {
                    Some(self.last_tasks.clone())
                };
                match dry_run_with_tasks(
                    self.config_path.clone(),
                    Some(self.repo.clone()),
                    agents,
                    runtime,
                ) {
                    Ok(json) => {
                        if let Ok(plan) = serde_json::from_str::<ExecutionPlan>(&json) {
                            self.last_plan = Some(plan);
                        }
                        vec![ShellEvent::PlanPreview(json)]
                    }
                    Err(e) => vec![ShellEvent::Error(e.to_string())],
                }
            }
            SlashCommand::Agents => {
                vec![ShellEvent::Output(pytxo_core::format_agents_list())]
            }
            SlashCommand::Use { ade } => self.handle_use(&ade),
            SlashCommand::Run {
                agents,
                cmd,
                keep_worktrees,
                ade,
            } => {
                if let Some(id) = ade {
                    if let Some(spec) = pytxo_core::resolve_ade(&id) {
                        if !pytxo_core::ade_can_dispatch(spec) {
                            return vec![ShellEvent::Error(format!(
                                "ADE {id} is detection-only until its permission model is mapped"
                            ))];
                        }
                    }
                }
                self.dispatch_run(agents, cmd, keep_worktrees).await
            }
            SlashCommand::Logs { agent, tail } => {
                match logs(
                    self.config_path.clone(),
                    Some(self.repo.clone()),
                    &agent,
                    tail,
                ) {
                    Ok(lines) => {
                        if lines.is_empty() {
                            vec![ShellEvent::Output(format!("No log events for {agent}"))]
                        } else {
                            vec![ShellEvent::Output(lines.join("\n"))]
                        }
                    }
                    Err(e) => vec![ShellEvent::Error(e.to_string())],
                }
            }
            SlashCommand::Stop { all } => {
                match pytxo_orchestrate::stop(
                    self.config_path.clone(),
                    Some(self.repo.clone()),
                    all,
                    false,
                )
                .await
                {
                    Ok(()) => {
                        self.active_run = None;
                        vec![ShellEvent::Output("Stop requested.".into())]
                    }
                    Err(e) => vec![ShellEvent::Error(e.to_string())],
                }
            }
            SlashCommand::Status { limit } => {
                match status_json(self.config_path.clone(), Some(self.repo.clone()), limit) {
                    Ok(status) => match serde_json::to_string_pretty(&status) {
                        Ok(json) => vec![ShellEvent::Output(json)],
                        Err(e) => vec![ShellEvent::Error(e.to_string())],
                    },
                    Err(e) => vec![ShellEvent::Error(e.to_string())],
                }
            }
            SlashCommand::Trust { tier } => self.handle_trust(tier).await,
            SlashCommand::Models { sub } => self.handle_models(sub).await,
        }
    }

    fn handle_use(&mut self, ade: &str) -> Vec<ShellEvent> {
        match pytxo_core::resolve_ade(ade) {
            Some(spec) if !pytxo_core::ade_can_dispatch(spec) => vec![ShellEvent::Error(format!(
                "ADE {ade} is detection-only until its permission model is mapped"
            ))],
            Some(spec) => {
                self.default_cmd = spec.default_cmd.to_string();
                vec![ShellEvent::Output(format!(
                    "Default /run command set to: {} ({})",
                    spec.default_cmd, spec.id
                ))]
            }
            None => vec![ShellEvent::Error(format!(
                "unknown ADE {ade} — try /agents"
            ))],
        }
    }

    async fn handle_trust(&mut self, tier: Option<String>) -> Vec<ShellEvent> {
        let profile = match tier.as_deref() {
            Some("deep_space") | Some("deep-space") => PermissionProfile::DeepSpace,
            Some("galaxy") => PermissionProfile::Galaxy,
            Some("supernova") => PermissionProfile::Supernova,
            Some("orbit") | None => PermissionProfile::Orbit,
            Some(other) => {
                return vec![ShellEvent::Error(format!(
                    "unknown tier {other} — use orbit, galaxy, deep_space, or supernova"
                ))];
            }
        };
        match trust_repo(&self.repo, profile) {
            Ok(()) => {
                self.config = load_config_for_repo(self.config_path.as_deref(), &self.repo)
                    .unwrap_or_else(|_| self.config.clone());
                let tier = match profile {
                    PermissionProfile::DeepSpace => "deep_space",
                    PermissionProfile::Orbit => "orbit",
                    PermissionProfile::Galaxy => "galaxy",
                    PermissionProfile::Supernova => "supernova",
                };
                vec![ShellEvent::Output(format!("Trusted folder ({tier})"))]
            }
            Err(e) => vec![ShellEvent::Error(e.to_string())],
        }
    }

    async fn handle_models(&self, sub: ModelsSub) -> Vec<ShellEvent> {
        let catalog = match ModelCatalog::open_default() {
            Ok(c) => c,
            Err(e) => return vec![ShellEvent::Error(e.to_string())],
        };
        match sub {
            ModelsSub::List { provider, refresh } => {
                let pid = provider
                    .as_deref()
                    .and_then(ProviderId::parse)
                    .unwrap_or(ProviderId::Openrouter);
                match catalog.list(pid, refresh) {
                    Ok(models) => {
                        let lines: Vec<String> = models
                            .iter()
                            .take(40)
                            .map(|m| format!("{}  {}", m.provider, m.id))
                            .collect();
                        let mut out = lines.join("\n");
                        if models.len() > 40 {
                            out.push_str(&format!("\n… and {} more", models.len() - 40));
                        }
                        vec![ShellEvent::Output(out)]
                    }
                    Err(e) => vec![ShellEvent::Error(e.to_string())],
                }
            }
            ModelsSub::Search { query, provider } => {
                let pid = provider.as_deref().and_then(ProviderId::parse);
                match catalog.search(&query, pid, false) {
                    Ok(models) => {
                        if models.is_empty() {
                            return vec![ShellEvent::Output("No models matched.".into())];
                        }
                        let lines: Vec<String> = models
                            .iter()
                            .take(30)
                            .map(|m| format!("{}  {}  {}", m.provider, m.id, m.name))
                            .collect();
                        vec![ShellEvent::Output(lines.join("\n"))]
                    }
                    Err(e) => vec![ShellEvent::Error(e.to_string())],
                }
            }
            ModelsSub::Refresh { provider } => {
                if let Some(p) = provider.as_deref().and_then(ProviderId::parse) {
                    match catalog.list(p, true) {
                        Ok(n) => vec![ShellEvent::Output(format!("Refreshed {} models", n.len()))],
                        Err(e) => vec![ShellEvent::Error(e.to_string())],
                    }
                } else {
                    vec![ShellEvent::Error("--provider required for refresh".into())]
                }
            }
        }
    }

    async fn dispatch_run(
        &mut self,
        agents: usize,
        cmd: String,
        keep_worktrees: bool,
    ) -> Vec<ShellEvent> {
        let cmd = if cmd.is_empty() || cmd == "echo pytxo" {
            self.default_cmd.clone()
        } else {
            cmd
        };
        let tasks = if self.last_tasks.is_empty() {
            None
        } else {
            Some(self.last_tasks.clone())
        };
        let prompts = if self.last_task_prompts.is_empty() {
            None
        } else {
            Some(self.last_task_prompts.clone())
        };
        let opts = RunOptions {
            agents,
            cmd,
            config: self.config_path.clone(),
            dry_run: false,
            keep_worktrees,
            repo: Some(self.repo.clone()),
            execution: None,
            project: None,
            tasks,
            task_cmd_template: None,
            task_prompts: prompts,
        };
        if !self.is_trusted() {
            return vec![ShellEvent::Error(
                "folder not trusted — use /trust [orbit|galaxy|…] first".into(),
            )];
        }
        match dispatch(opts) {
            Ok((_domain, run_id)) => {
                self.active_run = Some(run_id.clone());
                vec![
                    ShellEvent::RunStarted(run_id.clone()),
                    ShellEvent::Output(format!("Dispatched run {}", run_id.0)),
                ]
            }
            Err(e) => vec![ShellEvent::Error(e.to_string())],
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::command::parse_line;

    /// Trust and catalog writes go to one throwaway home per test process,
    /// never the developer's real `~/.pytxo`.
    fn isolate_home() {
        static HOME: std::sync::OnceLock<std::path::PathBuf> = std::sync::OnceLock::new();
        HOME.get_or_init(|| {
            let home = std::env::temp_dir().join(format!("pytxo-test-home-{}", std::process::id()));
            std::fs::create_dir_all(&home).expect("isolated PYTXO_HOME");
            std::env::set_var("PYTXO_HOME", &home);
            home
        });
    }

    fn trust_cwd(session: &mut ShellSession) {
        isolate_home();
        let _ = pytxo_orchestrate::trust_repo(&session.repo, pytxo_core::PermissionProfile::Orbit);
    }

    #[tokio::test]
    async fn help_returns_text() {
        let mut session = ShellSession::new(None, Some(std::env::current_dir().unwrap())).unwrap();
        trust_cwd(&mut session);
        let events = session.handle(parse_line("/help")).await;
        assert!(events
            .iter()
            .any(|e| matches!(e, ShellEvent::Output(s) if s.contains("/dry-run"))));
    }

    #[tokio::test]
    async fn dry_run_produces_plan_json() {
        let mut session = ShellSession::new(None, Some(std::env::current_dir().unwrap())).unwrap();
        trust_cwd(&mut session);
        let events = session.handle(parse_line("/dry-run --agents 2")).await;
        assert!(events
            .iter()
            .any(|e| matches!(e, ShellEvent::PlanPreview(s) if s.contains("waves"))));
    }

    #[tokio::test]
    async fn agents_and_use_cursor() {
        let mut session = ShellSession::new(None, Some(std::env::current_dir().unwrap())).unwrap();
        trust_cwd(&mut session);
        let agents = session.handle(parse_line("/agents")).await;
        assert!(agents.iter().any(|e| {
            matches!(e, ShellEvent::Output(s) if s.contains("cursor") && s.contains("codex"))
        }));
        let use_cursor = session.handle(parse_line("/use cursor")).await;
        assert!(use_cursor
            .iter()
            .any(|e| { matches!(e, ShellEvent::Output(s) if s.contains("cursor-agent")) }));
        assert_eq!(
            session.default_cmd,
            "cursor-agent -p --trust --output-format text"
        );
        let dry = session.handle(parse_line("/dry-run")).await;
        assert!(dry
            .iter()
            .any(|e| matches!(e, ShellEvent::PlanPreview(s) if s.contains("waves"))));
    }

    #[test]
    fn detection_only_harness_cannot_become_the_default() {
        let mut session = ShellSession::new(None, Some(std::env::current_dir().unwrap())).unwrap();
        let previous = session.default_cmd.clone();
        let events = session.handle_use("qwen");
        assert!(events.iter().any(
            |event| matches!(event, ShellEvent::Error(message) if message.contains("detection-only"))
        ));
        assert_eq!(session.default_cmd, previous);
    }
}
