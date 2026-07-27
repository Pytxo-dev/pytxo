//! Optional mission planner for Hypervisor Shell (v0.2.x).
//!
//! Disabled by default. Enable with `PYTXO_PLANNER=1` or `[planner] enabled = true` in `pytxo.toml`.
//! Use `PYTXO_PLANNER=signal` or `[planner] mode = "signal"]` for Signal Core graph inference.

use std::collections::{HashMap, HashSet};
use std::path::Path;

use anyhow::{bail, Context};
use pytxo_core::{PytxoConfig, Task, TaskId};
use pytxo_signal::build_structural_graph;
use serde::Deserialize;

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

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum PlannerMode {
    Heuristic,
    Signal,
    Llm,
}

/// Whether planner features are active for this process.
pub fn planner_enabled(config: &PytxoConfig) -> bool {
    if let Ok(v) = std::env::var("PYTXO_PLANNER") {
        if v == "1" || v.eq_ignore_ascii_case("true") || v.eq_ignore_ascii_case("signal") {
            return true;
        }
        if v == "0" || v.eq_ignore_ascii_case("false") {
            return false;
        }
    }
    config.planner_enabled()
}

fn planner_mode(config: &PytxoConfig) -> PlannerMode {
    if llm_planner_enabled(config) {
        return PlannerMode::Llm;
    }
    if let Ok(v) = std::env::var("PYTXO_PLANNER") {
        if v.eq_ignore_ascii_case("signal") {
            return PlannerMode::Signal;
        }
    }
    if config.planner.mode.eq_ignore_ascii_case("signal") {
        PlannerMode::Signal
    } else {
        PlannerMode::Heuristic
    }
}

/// LLM planner: Ultra managed proxy **or** BYOK OpenAI-compatible providers.
pub fn llm_planner_enabled(config: &PytxoConfig) -> bool {
    if !planner_enabled(config) && !mission_planner_unlocked() {
        // Mission path unlocks BYOK even when [planner] is off.
        if byok_scout_endpoint().is_some() {
            return true;
        }
        return false;
    }
    if byok_scout_endpoint().is_some() {
        return true;
    }
    if !ultra_billing_active(config) {
        return false;
    }
    std::env::var("PYTXO_PLANNER_LLM")
        .ok()
        .is_some_and(|v| v == "1" || v.eq_ignore_ascii_case("true"))
}

/// Mission CLI/Flow always plan (ADR-0031); shell slash-run still respects planner flag.
fn mission_planner_unlocked() -> bool {
    std::env::var("PYTXO_MISSION_PLAN")
        .ok()
        .is_some_and(|v| v == "1" || v.eq_ignore_ascii_case("true"))
}

fn ultra_billing_active(config: &PytxoConfig) -> bool {
    if config.billing_mode().is_ultra() {
        return true;
    }
    std::env::var("PYTXO_LINK_TIER")
        .ok()
        .is_some_and(|t| t.eq_ignore_ascii_case("ultra"))
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

fn split_mission_chunks(text: &str) -> Vec<&str> {
    let chunks: Vec<&str> = text
        .split([';', '\n'])
        .map(str::trim)
        .filter(|s| !s.is_empty())
        .collect();
    if chunks.is_empty() {
        vec![text]
    } else {
        chunks
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
        let parts = split_mission_chunks(text);
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
                verify: vec![],
            });
        }
        Ok(attach_verify_suggestions(
            MissionPlan {
                tasks,
                task_prompts,
            },
            ctx.repo,
        ))
    }
}

/// Ultra LLM planner: decompose mission via managed inference proxy DeepSeek route.
pub struct LlmPlanner;

#[derive(Debug, Deserialize)]
struct LlmTaskRow {
    id: String,
    agent: String,
    paths: Vec<String>,
    #[serde(default)]
    depends_on: Vec<String>,
    prompt: String,
    #[serde(default)]
    verify: Vec<String>,
}

#[derive(Debug, Deserialize)]
struct LlmPlanResponse {
    tasks: Vec<LlmTaskRow>,
}

impl LlmPlanner {
    fn proxy_base(config: &PytxoConfig) -> String {
        config
            .billing
            .inference_proxy_url
            .trim_end_matches('/')
            .to_string()
    }

    fn planner_model() -> String {
        std::env::var("PYTXO_PLANNER_MODEL").unwrap_or_else(|_| "deepseek-chat".into())
    }

    fn call_proxy(mission: &str, config: &PytxoConfig) -> anyhow::Result<LlmPlanResponse> {
        let system = "Decompose the mission into parallel-safe coding tasks. Return JSON: {\"tasks\":[{\"id\":\"task-a\",\"agent\":\"agent-0\",\"paths\":[\"src/foo.ts\"],\"depends_on\":[],\"prompt\":\"...\",\"verify\":[\"npm test\"]}]}. Max tasks from user config. Use repo-relative paths. Suggest verify commands only when obvious (cargo test, npm test, pytest).";
        let body_for = |model: &str| {
            serde_json::json!({
                "model": model,
                "response_format": { "type": "json_object" },
                "messages": [
                    { "role": "system", "content": system },
                    { "role": "user", "content": mission }
                ]
            })
        };

        // Prefer BYOK OpenAI-compatible scout (ADR-0031).
        if let Some((base, key, model)) = byok_scout_endpoint() {
            let url = format!("{}/chat/completions", base.trim_end_matches('/'));
            let resp = ureq::post(&url)
                .set("Content-Type", "application/json")
                .set("Authorization", &format!("Bearer {key}"))
                .send_json(body_for(&model))
                .map_err(|e| anyhow::anyhow!("byok scout request failed: {e}"))?;
            if !(200..300).contains(&resp.status()) {
                bail!("byok scout returned HTTP {}", resp.status());
            }
            return Self::parse_chat_response(resp);
        }

        // Ultra managed proxy fallback.
        let url = format!("{}/deepseek/v1/chat/completions", Self::proxy_base(config));
        let model = Self::planner_model();
        let mut req = ureq::post(&url).set("Content-Type", "application/json");
        if let Ok(token) = std::env::var("PYTXO_ULTRA_SESSION") {
            if !token.trim().is_empty() {
                req = req.set("Authorization", &format!("Bearer {token}"));
            }
        }
        let resp = req
            .send_json(body_for(&model))
            .map_err(|e| anyhow::anyhow!("llm planner proxy request failed: {e}"))?;
        if !(200..300).contains(&resp.status()) {
            bail!("llm planner proxy returned HTTP {}", resp.status());
        }
        Self::parse_chat_response(resp)
    }

    fn parse_chat_response(resp: ureq::Response) -> anyhow::Result<LlmPlanResponse> {
        let envelope: serde_json::Value = resp
            .into_json()
            .map_err(|e| anyhow::anyhow!("llm planner invalid JSON: {e}"))?;
        let content = envelope["choices"][0]["message"]["content"]
            .as_str()
            .ok_or_else(|| anyhow::anyhow!("llm planner missing message content"))?;
        serde_json::from_str(content).context("parse llm planner task JSON")
    }
}

impl MissionPlanner for LlmPlanner {
    fn decompose(
        &self,
        mission: &MissionSpec,
        ctx: &PlannerContext<'_>,
    ) -> anyhow::Result<MissionPlan> {
        let text = mission.text.trim();
        if text.is_empty() {
            bail!("mission text is empty");
        }
        let parsed = Self::call_proxy(text, ctx.config)?;
        let n = ctx.config.max_agents.max(1);
        let mut tasks = Vec::new();
        let mut task_prompts = HashMap::new();
        for row in parsed.tasks.into_iter().take(n) {
            task_prompts.insert(row.id.clone(), row.prompt);
            tasks.push(Task {
                id: TaskId(row.id),
                agent: row.agent,
                paths: if row.paths.is_empty() {
                    vec![".".into()]
                } else {
                    row.paths
                },
                depends_on: row.depends_on,
                root: None,
                signal_fidelity: None,
                verify: row.verify,
            });
        }
        if tasks.is_empty() {
            bail!("llm planner returned no tasks");
        }
        Ok(attach_verify_suggestions(
            MissionPlan {
                tasks,
                task_prompts,
            },
            ctx.repo,
        ))
    }
}

/// Signal-backed v2: infer `depends_on` from structural import graph between chunk paths.
pub struct SignalBackedPlanner;

impl MissionPlanner for SignalBackedPlanner {
    fn decompose(
        &self,
        mission: &MissionSpec,
        ctx: &PlannerContext<'_>,
    ) -> anyhow::Result<MissionPlan> {
        let config_tasks = ctx.config.tasks();
        if !config_tasks.is_empty() {
            return enrich_config_tasks(config_tasks, mission, ctx);
        }

        let text = mission.text.trim();
        if text.is_empty() {
            bail!("mission text is empty and pytxo.toml has no [[task]] entries");
        }
        let parts = split_mission_chunks(text);
        let n = parts.len().min(ctx.config.max_agents.max(1));
        let chunk_paths: Vec<Vec<String>> = parts
            .iter()
            .take(n)
            .map(|part| infer_paths_from_chunk(part, ctx.repo))
            .collect();

        let edited: Vec<(String, String, Option<String>)> = chunk_paths
            .iter()
            .enumerate()
            .flat_map(|(i, paths)| {
                paths
                    .iter()
                    .map(move |p| (p.clone(), format!("mission-{i}"), None::<String>))
            })
            .collect();

        let graph = build_structural_graph(ctx.repo, &edited);
        let mut tasks = Vec::with_capacity(n);
        let mut task_prompts = HashMap::new();
        let task_ids: Vec<TaskId> = (0..n).map(|i| TaskId(format!("mission-{i}"))).collect();

        for (i, part) in parts.into_iter().take(n).enumerate() {
            let paths = chunk_paths[i].clone();
            let paths = if paths.is_empty() {
                vec![".".into()]
            } else {
                paths
            };
            let mut depends_on = infer_depends_on(i, &paths, &chunk_paths, &graph.edges, &task_ids);
            if depends_on.is_empty() && i > 0 {
                depends_on.push(task_ids[i - 1].0.clone());
            }
            task_prompts.insert(task_ids[i].0.clone(), part.to_string());
            tasks.push(Task {
                id: task_ids[i].clone(),
                agent: format!("agent-{i}"),
                paths,
                depends_on,
                root: None,
                signal_fidelity: None,
                verify: vec![],
            });
        }

        Ok(attach_verify_suggestions(
            MissionPlan {
                tasks,
                task_prompts,
            },
            ctx.repo,
        ))
    }
}

/// When `pytxo.toml` defines `[[task]]` rows, seed the plan from manifest tasks and
/// enrich `depends_on` via Signal Core import edges (Phase 48).
fn enrich_config_tasks(
    config_tasks: Vec<Task>,
    mission: &MissionSpec,
    ctx: &PlannerContext<'_>,
) -> anyhow::Result<MissionPlan> {
    let n = config_tasks.len().min(ctx.config.max_agents.max(1));
    let config_tasks: Vec<Task> = config_tasks.into_iter().take(n).collect();

    let edited: Vec<(String, String, Option<String>)> = config_tasks
        .iter()
        .flat_map(|t| {
            t.paths
                .iter()
                .map(move |p| (p.clone(), t.id.0.clone(), t.root.clone()))
        })
        .collect();

    let graph = build_structural_graph(ctx.repo, &edited);
    let mission_chunks = split_mission_chunks(mission.text.trim());
    let mut task_prompts = HashMap::new();
    let chunk_paths: Vec<Vec<String>> = config_tasks.iter().map(|t| t.paths.clone()).collect();
    let task_ids: Vec<TaskId> = config_tasks.iter().map(|t| t.id.clone()).collect();
    let mut tasks = Vec::with_capacity(config_tasks.len());

    for (i, mut task) in config_tasks.into_iter().enumerate() {
        if task.depends_on.is_empty() {
            let inferred = infer_depends_on(i, &task.paths, &chunk_paths, &graph.edges, &task_ids);
            if !inferred.is_empty() {
                task.depends_on = inferred;
            }
        }
        let prompt = mission_chunks
            .get(i)
            .map(|s| s.to_string())
            .unwrap_or_else(|| format!("Execute task {}", task.id.0));
        task_prompts.insert(task.id.0.clone(), prompt);
        tasks.push(task);
    }

    Ok(attach_verify_suggestions(
        MissionPlan {
            tasks,
            task_prompts,
        },
        ctx.repo,
    ))
}

fn infer_paths_from_chunk(chunk: &str, repo: &Path) -> Vec<String> {
    let mut out = Vec::new();
    let mut seen = HashSet::new();
    for token in chunk.split_whitespace() {
        let cleaned = token.trim_matches(|c: char| {
            !c.is_alphanumeric() && c != '.' && c != '/' && c != '-' && c != '_'
        });
        if cleaned.len() < 3 {
            continue;
        }
        if let Some(rel) = resolve_path_hint(repo, cleaned) {
            if seen.insert(rel.clone()) {
                out.push(rel);
            }
        }
    }
    out
}

fn resolve_path_hint(repo: &Path, hint: &str) -> Option<String> {
    let hint = hint.replace('\\', "/");
    let direct = repo.join(&hint);
    if direct.is_file() {
        return Some(normalize_rel(&hint));
    }
    if hint.contains('/') {
        return None;
    }
    if hint.contains('.') {
        let mut matches = Vec::new();
        walk_repo_for_filename(repo, repo, &hint, &mut matches);
        return matches.into_iter().next();
    }
    None
}

fn walk_repo_for_filename(repo: &Path, dir: &Path, name: &str, out: &mut Vec<String>) {
    if out.len() >= 4 {
        return;
    }
    let Ok(entries) = std::fs::read_dir(dir) else {
        return;
    };
    for entry in entries.flatten() {
        let path = entry.path();
        if path.is_dir() {
            let leaf = path.file_name().and_then(|s| s.to_str()).unwrap_or("");
            if leaf == ".git" || leaf == "node_modules" || leaf == "target" {
                continue;
            }
            walk_repo_for_filename(repo, &path, name, out);
        } else if path.file_name().and_then(|s| s.to_str()) == Some(name) {
            if let Ok(rel) = path.strip_prefix(repo) {
                out.push(normalize_rel(&rel.to_string_lossy()));
            }
        }
    }
}

fn normalize_rel(path: &str) -> String {
    path.replace('\\', "/").trim_start_matches("./").to_string()
}

fn infer_depends_on(
    task_idx: usize,
    paths: &[String],
    all_paths: &[Vec<String>],
    edges: &[pytxo_signal::GraphEdge],
    task_ids: &[TaskId],
) -> Vec<String> {
    let my_paths: HashSet<String> = paths.iter().map(|p| normalize_rel(p)).collect();
    let mut deps = HashSet::new();
    for edge in edges {
        if !my_paths.contains(&normalize_rel(&edge.from)) {
            continue;
        }
        for (j, other_paths) in all_paths.iter().enumerate() {
            if j >= task_idx {
                continue;
            }
            let other: HashSet<String> = other_paths.iter().map(|p| normalize_rel(p)).collect();
            if other.contains(&normalize_rel(&edge.to)) {
                deps.insert(task_ids[j].0.clone());
            }
        }
    }
    let mut ordered: Vec<String> = deps.into_iter().collect();
    ordered.sort();
    ordered
}

pub fn default_planner(config: &PytxoConfig) -> Box<dyn MissionPlanner> {
    if !planner_enabled(config) {
        return Box::new(StubPlanner);
    }
    match planner_mode(config) {
        PlannerMode::Llm => Box::new(LlmPlanner),
        PlannerMode::Signal => Box::new(SignalBackedPlanner),
        PlannerMode::Heuristic => Box::new(HeuristicPlanner),
    }
}

/// Planner for Flow / `pytxo mission` — always on (ADR-0031). Prefer BYOK scout LLM,
/// then Signal-backed, then heuristic when explicitly requested.
pub fn default_mission_planner(config: &PytxoConfig) -> Box<dyn MissionPlanner> {
    if byok_scout_endpoint().is_some() || (ultra_billing_active(config) && llm_planner_flag()) {
        return Box::new(LlmPlanner);
    }
    if std::env::var("PYTXO_PLANNER")
        .ok()
        .is_some_and(|v| v.eq_ignore_ascii_case("heuristic"))
    {
        return Box::new(HeuristicPlanner);
    }
    Box::new(SignalBackedPlanner)
}

fn llm_planner_flag() -> bool {
    std::env::var("PYTXO_PLANNER_LLM")
        .ok()
        .is_some_and(|v| v == "1" || v.eq_ignore_ascii_case("true"))
}

/// OpenAI-compatible BYOK scout: (base_url, api_key, model).
fn byok_scout_endpoint() -> Option<(String, String, String)> {
    let candidates = [
        ("DEEPSEEK_API_KEY", "https://api.deepseek.com/v1", "deepseek-chat"),
        ("OPENAI_API_KEY", "https://api.openai.com/v1", "gpt-4.1-mini"),
        ("OPENROUTER_API_KEY", "https://openrouter.ai/api/v1", "openai/gpt-4.1-mini"),
        ("MISTRAL_API_KEY", "https://api.mistral.ai/v1", "mistral-small-latest"),
    ];
    for (env, base, model) in candidates {
        if let Ok(key) = std::env::var(env) {
            if !key.trim().is_empty() {
                let model = std::env::var("PYTXO_PLANNER_MODEL").unwrap_or_else(|_| model.into());
                return Some((base.into(), key, model));
            }
        }
    }
    None
}

fn suggest_verify_commands(repo: &Path) -> Vec<String> {
    let mut cmds = Vec::new();
    if repo.join("Cargo.toml").is_file() {
        cmds.push("cargo test".into());
    }
    if repo.join("package.json").is_file() {
        // Prefer npm test when present; do not invent scripts.
        if let Ok(raw) = std::fs::read_to_string(repo.join("package.json")) {
            if raw.contains("\"test\"") {
                cmds.push("npm test".into());
            }
        }
    }
    if repo.join("pyproject.toml").is_file() || repo.join("pytest.ini").is_file() {
        cmds.push("pytest".into());
    }
    if repo.join("go.mod").is_file() {
        cmds.push("go test ./...".into());
    }
    cmds
}

fn attach_verify_suggestions(mut plan: MissionPlan, repo: &Path) -> MissionPlan {
    let suggested = suggest_verify_commands(repo);
    if suggested.is_empty() {
        return plan;
    }
    for task in &mut plan.tasks {
        if task.verify.is_empty() {
            task.verify = suggested.clone();
        }
    }
    plan
}

/// Decompose a mission for Flow / CLI mission (always uses mission planner).
pub fn plan_mission(
    mission: &str,
    repo: &Path,
    config: &PytxoConfig,
) -> anyhow::Result<MissionPlan> {
    let spec = MissionSpec {
        text: mission.to_string(),
    };
    let ctx = PlannerContext { repo, config };
    default_mission_planner(config)
        .decompose(&spec, &ctx)
        .with_context(|| format!("planner failed for mission: {}", spec.text))
}

/// Legacy shell entry: respects PYTXO_PLANNER / [planner] enabled gate.
pub fn plan_mission_gated(
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
        let err = plan_mission_gated("fix tests", Path::new("."), &cfg).unwrap_err();
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
    fn mission_planner_works_without_flag() {
        let _guard = PLANNER_ENV_LOCK.lock().unwrap();
        let prev = std::env::var("PYTXO_PLANNER").ok();
        std::env::set_var("PYTXO_PLANNER", "0");
        let mut cfg = PytxoConfig::default();
        cfg.planner.enabled = false;
        cfg.max_agents = 2;
        let plan = plan_mission("fix auth; update docs", Path::new("."), &cfg).unwrap();
        assert!(!plan.tasks.is_empty());
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

    #[test]
    fn signal_planner_uses_pytxo_toml_tasks() {
        let _guard = PLANNER_ENV_LOCK.lock().unwrap();
        let prev = std::env::var("PYTXO_PLANNER").ok();
        std::env::set_var("PYTXO_PLANNER", "signal");
        let dir = tempfile::tempdir().unwrap();
        let repo = dir.path();
        std::fs::write(
            repo.join("pytxo.toml"),
            r#"
max_agents = 3
[planner]
enabled = true
mode = "signal"

[[task]]
id = "task-a"
agent = "agent-0"
paths = ["src/utils.ts"]

[[task]]
id = "task-b"
agent = "agent-1"
paths = ["src/app.ts"]
"#,
        )
        .unwrap();
        std::fs::create_dir_all(repo.join("src")).unwrap();
        std::fs::write(repo.join("src/utils.ts"), "export const x = 1;\n").unwrap();
        std::fs::write(
            repo.join("src/app.ts"),
            "import './utils';\nexport const run = () => x;\n",
        )
        .unwrap();

        let cfg = PytxoConfig::load(&repo.join("pytxo.toml")).unwrap();
        let plan = plan_mission("", repo, &cfg).unwrap();
        assert_eq!(plan.tasks.len(), 2);
        assert_eq!(plan.tasks[0].id.0, "task-a");
        assert_eq!(plan.tasks[1].id.0, "task-b");

        match prev {
            Some(v) => std::env::set_var("PYTXO_PLANNER", v),
            None => std::env::remove_var("PYTXO_PLANNER"),
        }
    }

    #[test]
    fn signal_planner_chains_depends_on() {
        let _guard = PLANNER_ENV_LOCK.lock().unwrap();
        let prev = std::env::var("PYTXO_PLANNER").ok();
        std::env::set_var("PYTXO_PLANNER", "signal");
        let dir = tempfile::tempdir().unwrap();
        let repo = dir.path();
        std::fs::create_dir_all(repo.join("src")).unwrap();
        std::fs::write(repo.join("src/utils.ts"), "export const x = 1;\n").unwrap();
        std::fs::write(
            repo.join("src/app.ts"),
            "import './utils';\nexport const run = () => x;\n",
        )
        .unwrap();

        let mut cfg = PytxoConfig::default();
        cfg.planner.enabled = true;
        cfg.planner.mode = "signal".into();
        cfg.max_agents = 3;

        let plan = plan_mission("update src/utils.ts; wire src/app.ts", repo, &cfg).unwrap();
        assert_eq!(plan.tasks.len(), 2);
        assert!(
            plan.tasks[1].depends_on.contains(&"mission-0".to_string()),
            "expected mission-1 to depend on mission-0 via import graph, got {:?}",
            plan.tasks[1].depends_on
        );

        match prev {
            Some(v) => std::env::set_var("PYTXO_PLANNER", v),
            None => std::env::remove_var("PYTXO_PLANNER"),
        }
    }
}
