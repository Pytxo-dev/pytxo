//! Optional mission planner for Hypervisor Shell (v0.2.x).
//!
//! Disabled by default. Enable with `PYTXO_PLANNER=1` or `[planner] enabled = true` in `pytxo.toml`.

use std::collections::HashMap;
use std::path::Path;

use anyhow::{bail, Context};
use pytxo_core::{PytxoConfig, Task, TaskId};

/// Natural-language mission from the operator prompt.
#[derive(Clone, Debug)]
pub struct MissionSpec {
    pub text: String,
}

/// Repo context for planners (scaffold for future LLM / index integrations).
#[derive(Clone, Debug)]
pub struct PlannerContext<'a> {
    pub repo: &'a Path,
    pub config: &'a PytxoConfig,
}

#[derive(Clone, Debug, Default)]
pub struct MissionPlan {
    pub tasks: Vec<Task>,
    pub task_prompts: HashMap<String, String>,
}

pub trait MissionPlanner: Send + Sync {
    fn decompose(
        &self,
        mission: &MissionSpec,
        ctx: &PlannerContext<'_>,
    ) -> anyhow::Result<MissionPlan>;
}

/// Whether planner features are active for this process.
pub fn planner_enabled(config: &PytxoConfig) -> bool {
    if let Ok(v) = std::env::var("PYTXO_PLANNER") {
        if v == "1" || v.eq_ignore_ascii_case("true") {
            return true;
        }
        if v == "0" || v.eq_ignore_ascii_case("false") {
            return false;
        }
    }
    config.planner_enabled()
}

/// Default stub: instructs the operator to use `/run` or enable the planner.
pub struct StubPlanner;

impl MissionPlanner for StubPlanner {
    fn decompose(
        &self,
        _mission: &MissionSpec,
        _ctx: &PlannerContext<'_>,
    ) -> anyhow::Result<MissionPlan> {
        bail!("NL planner is disabled. Use slash commands (/run, /dry-run) or set PYTXO_PLANNER=1")
    }
}

/// Heuristic v1: one synthetic task per sentence chunk (testing / demos only).
pub struct HeuristicPlanner;

impl MissionPlanner for HeuristicPlanner {
    fn decompose(
        &self,
        mission: &MissionSpec,
        ctx: &PlannerContext<'_>,
    ) -> anyhow::Result<MissionPlan> {
        let text = mission.text.trim();
        if text.is_empty() {
            bail!("mission text is empty");
        }
        let chunks: Vec<&str> = text
            .split([';', '\n'])
            .map(str::trim)
            .filter(|s| !s.is_empty())
            .collect();
        let parts = if chunks.is_empty() {
            vec![text]
        } else {
            chunks
        };
        let n = parts.len().min(ctx.config.max_agents.max(1));
        let mut tasks = Vec::with_capacity(n);
        let mut task_prompts = HashMap::new();
        for (i, part) in parts.into_iter().take(n).enumerate() {
            let id = TaskId(format!("mission-{i}"));
            task_prompts.insert(id.0.clone(), part.to_string());
            tasks.push(Task {
                id,
                agent: format!("agent-{i}"),
                paths: vec![".".into()],
                depends_on: vec![],
                root: None,
                signal_fidelity: None,
            });
        }
        Ok(MissionPlan {
            tasks,
            task_prompts,
        })
    }
}

pub fn default_planner(config: &PytxoConfig) -> Box<dyn MissionPlanner> {
    if planner_enabled(config) {
        Box::new(HeuristicPlanner)
    } else {
        Box::new(StubPlanner)
    }
}

/// Decompose a mission line when planner is enabled; otherwise returns a helpful error.
pub fn plan_mission(
    mission: &str,
    repo: &Path,
    config: &PytxoConfig,
) -> anyhow::Result<MissionPlan> {
    let spec = MissionSpec {
        text: mission.to_string(),
    };
    let ctx = PlannerContext { repo, config };
    default_planner(config)
        .decompose(&spec, &ctx)
        .with_context(|| format!("planner failed for mission: {}", spec.text))
}

#[cfg(test)]
mod tests {
    use std::sync::Mutex;

    use super::*;

    static PLANNER_ENV_LOCK: Mutex<()> = Mutex::new(());

    #[test]
    fn stub_errors_when_disabled() {
        let _guard = PLANNER_ENV_LOCK.lock().unwrap();
        let prev = std::env::var("PYTXO_PLANNER").ok();
        std::env::set_var("PYTXO_PLANNER", "0");
        let cfg = PytxoConfig::default();
        assert!(!planner_enabled(&cfg));
        let err = plan_mission("fix tests", Path::new("."), &cfg).unwrap_err();
        let msg = format!("{err:#}");
        assert!(
            msg.contains("disabled") || msg.contains("NL planner"),
            "unexpected error: {msg}"
        );
        match prev {
            Some(v) => std::env::set_var("PYTXO_PLANNER", v),
            None => std::env::remove_var("PYTXO_PLANNER"),
        }
    }

    #[test]
    fn heuristic_splits_sentences_when_enabled() {
        let _guard = PLANNER_ENV_LOCK.lock().unwrap();
        let prev = std::env::var("PYTXO_PLANNER").ok();
        std::env::set_var("PYTXO_PLANNER", "1");
        let mut cfg = PytxoConfig::default();
        cfg.planner.enabled = true;
        cfg.max_agents = 3;
        let plan =
            plan_mission("fix auth tests; update package.json", Path::new("."), &cfg).unwrap();
        assert_eq!(plan.tasks.len(), 2);
        assert_eq!(plan.tasks[0].id.0, "mission-0");
        assert!(plan.task_prompts.contains_key("mission-0"));
        match prev {
            Some(v) => std::env::set_var("PYTXO_PLANNER", v),
            None => std::env::remove_var("PYTXO_PLANNER"),
        }
    }
}
