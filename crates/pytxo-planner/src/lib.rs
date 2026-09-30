//! Optional mission planner for Hypervisor Shell (v0.2.x).
//!
//! Disabled by default. Enable with `PYTXO_PLANNER=1` or `[planner] enabled = true` in `pytxo.toml`.
//! Use `PYTXO_PLANNER=signal` or `[planner] mode = "signal"]` for Signal Core graph inference.

pub mod advisor;

use std::collections::{HashMap, HashSet};
use std::path::{Component, Path};

use anyhow::{bail, Context};
use pytxo_core::{
    find_custom_provider, get_provider, resolve_openai_base_url, CoordinatorTransport, ProviderId,
    PytxoConfig, Task, TaskId,
};
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
    if let Some(mode) = explicit_local_planner_mode() {
        return mode;
    }
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

/// Cloud planning requires explicit process opt-in. A provider credential is
/// transport configuration, never consent to disclose mission/repository context.
pub fn llm_planner_enabled(config: &PytxoConfig) -> bool {
    explicit_local_planner_mode().is_none()
        && llm_planner_flag()
        && coordinator_endpoint(config).is_ok()
}

fn explicit_local_planner_mode() -> Option<PlannerMode> {
    match std::env::var("PYTXO_PLANNER")
        .ok()?
        .to_ascii_lowercase()
        .as_str()
    {
        "heuristic" => Some(PlannerMode::Heuristic),
        // Flow always supports local planning; 0/false still disable legacy shell planning.
        "signal" | "0" | "false" => Some(PlannerMode::Signal),
        _ => None,
    }
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
        let n = parts.len();
        let mut tasks = Vec::with_capacity(n);
        let mut task_prompts = HashMap::new();
        for (i, part) in parts.into_iter().enumerate() {
            let id = TaskId(format!("mission-{i}"));
            let paths = infer_paths_from_chunk(part, ctx.repo);
            if paths.is_empty() {
                bail!(
                    "planner could not determine safe ownership paths for task {id}: {part}. Name a repo-relative file or directory"
                );
            }
            task_prompts.insert(id.0.clone(), part.to_string());
            tasks.push(Task {
                id,
                agent: format!("agent-{i}"),
                paths,
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

/// Advisory model coordinator used for mission decomposition. The returned
/// tasks still pass through Pytxo's deterministic plan validation.
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
    fn call_proxy(mission: &str, ctx: &PlannerContext<'_>) -> anyhow::Result<LlmPlanResponse> {
        if !llm_planner_enabled(ctx.config) {
            bail!("model coordinator is not enabled: explicitly set PYTXO_PLANNER_LLM=1 and configure its direct, local, or managed transport; local planner selections take precedence");
        }
        let endpoint = coordinator_endpoint(ctx.config)?;
        let system = "Decompose the mission into parallel-safe coding tasks. Return JSON: {\"tasks\":[{\"id\":\"task-a\",\"agent\":\"agent-0\",\"paths\":[\"src/foo.ts\"],\"depends_on\":[],\"prompt\":\"...\",\"verify\":[\"npm test\"]}]}. Use explicit repo-relative ownership paths from the supplied repository brief. Never use \".\", absolute paths, parent traversal, or glob patterns. If ownership is unclear, return no tasks. Dependencies must reference unique task ids. Suggest verify commands only when they are supported by the supplied manifests.";
        let repository = repository_brief(ctx.repo);
        let user = format!("Mission:\n{mission}\n\n{repository}");
        let body = serde_json::json!({
            "model": endpoint.model,
            "response_format": { "type": "json_object" },
            "messages": [
                { "role": "system", "content": system },
                { "role": "user", "content": user }
            ]
        });

        let mut request = ureq::post(&endpoint.url).set("Content-Type", "application/json");
        if let Some(token) = endpoint.bearer {
            request = request.set("Authorization", &format!("Bearer {token}"));
        }
        let resp = request
            .send_json(body)
            .map_err(|e| anyhow::anyhow!("coordinator request failed: {e}"))?;
        if !(200..300).contains(&resp.status()) {
            bail!("coordinator returned HTTP {}", resp.status());
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
        let parsed = Self::call_proxy(text, ctx)?;
        let mut tasks = Vec::new();
        let mut task_prompts = HashMap::new();
        for row in parsed.tasks {
            task_prompts.insert(row.id.clone(), row.prompt);
            tasks.push(Task {
                id: TaskId(row.id),
                agent: row.agent,
                paths: row.paths,
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
        // max_agents limits concurrent workers in the scheduler, not mission scope.
        let n = parts.len();
        let chunk_paths: Vec<Vec<String>> = parts
            .iter()
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

        for (i, part) in parts.into_iter().enumerate() {
            let paths = chunk_paths[i].clone();
            if paths.is_empty() {
                bail!(
                    "planner could not determine safe ownership paths for task {}: {part}. Name a repo-relative file or directory",
                    task_ids[i].0
                );
            }
            let mut depends_on = infer_depends_on(i, &paths, &chunk_paths, &graph.edges, &task_ids);
            // Documentation of the result consumes earlier implementation/test
            // outcomes even though Markdown has no structural import edges.
            // Only infer backward edges; explicit manifest plans are untouched.
            if describes_result(part, &paths) {
                depends_on.extend(
                    chunk_paths[..i]
                        .iter()
                        .enumerate()
                        .filter(|(_, paths)| !documentation_paths(paths))
                        .map(|(index, _)| task_ids[index].0.clone()),
                );
                depends_on.sort();
                depends_on.dedup();
            }
            // Unknown/broad ownership stays conservative. Explicit, unrelated
            // files remain parallel so Race Shield can build the widest safe
            // wave instead of serializing every natural-language chunk.
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
        let prompt = if mission_chunks.len() == task_ids.len() {
            mission_chunks[i].to_string()
        } else {
            format!(
                "Mission: {}\nYour task: {}. Work only within these ownership paths: {}.",
                mission.text.trim(),
                task.id.0,
                task.paths.join(", ")
            )
        };
        let prompt = if prompt.trim().is_empty() {
            format!("Execute task {}", task.id.0)
        } else {
            prompt
        };
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
        // A sentence-ending period is prose punctuation, not part of a path.
        // This matters on Windows, where `README.md.` resolves to `README.md`
        // and would otherwise survive the existence check with the wrong
        // reader-facing claim.
        let cleaned = cleaned.trim_end_matches('.');
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

fn documentation_paths(paths: &[String]) -> bool {
    !paths.is_empty()
        && paths.iter().all(|path| {
            Path::new(path)
                .extension()
                .and_then(|ext| ext.to_str())
                .is_some_and(|ext| {
                    matches!(ext.to_ascii_lowercase().as_str(), "md" | "mdx" | "rst")
                })
        })
}

fn describes_result(chunk: &str, paths: &[String]) -> bool {
    documentation_paths(paths)
        && chunk.split_whitespace().any(|word| {
            // Trim sentence punctuation without splitting paths such as docs/explain.md.
            let word = word.trim_matches(|c: char| !c.is_alphabetic());
            matches!(
                word.to_ascii_lowercase().as_str(),
                "document" | "describe" | "explain" | "summarize"
            )
        })
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

/// Planner for Flow / `pytxo mission` — always on (ADR-0031), local by default.
pub fn default_mission_planner(config: &PytxoConfig) -> Box<dyn MissionPlanner> {
    match mission_planner_mode(config) {
        PlannerMode::Llm => Box::new(LlmPlanner),
        PlannerMode::Heuristic => Box::new(HeuristicPlanner),
        PlannerMode::Signal => Box::new(SignalBackedPlanner),
    }
}

fn mission_planner_mode(config: &PytxoConfig) -> PlannerMode {
    explicit_local_planner_mode().unwrap_or_else(|| {
        if llm_planner_enabled(config) {
            PlannerMode::Llm
        } else {
            PlannerMode::Signal
        }
    })
}

fn llm_planner_flag() -> bool {
    std::env::var("PYTXO_PLANNER_LLM")
        .ok()
        .is_some_and(|v| v == "1" || v.eq_ignore_ascii_case("true"))
}

#[derive(Debug)]
struct CoordinatorEndpoint {
    url: String,
    bearer: Option<String>,
    model: String,
}

/// Resolve one explicit OpenAI-compatible coordinator profile. This deliberately
/// does not scan ambient provider keys and choose a provider by accident.
fn coordinator_endpoint(config: &PytxoConfig) -> anyhow::Result<CoordinatorEndpoint> {
    let profile = config.coordinator.profile().map_err(anyhow::Error::msg)?;
    let model = std::env::var("PYTXO_PLANNER_MODEL")
        .ok()
        .filter(|value| !value.trim().is_empty())
        .unwrap_or_else(|| profile.model.as_str().to_string());

    let provider_spec = get_provider(profile.provider);
    let custom_spec = find_custom_provider(&profile.provider_label);
    let openai_compatible = custom_spec
        .as_ref()
        .map(|spec| spec.openai_compatible)
        .or_else(|| provider_spec.map(|spec| spec.openai_compatible))
        .unwrap_or(false);
    if !openai_compatible {
        bail!(
            "coordinator provider {} has no OpenAI-compatible adapter",
            profile.provider_label
        );
    }

    match profile.transport {
        CoordinatorTransport::Direct => {
            let base =
                resolve_openai_base_url(profile.provider, Some(profile.provider_label.as_str()))
                    .with_context(|| {
                        format!(
                            "coordinator provider {} has no configured base URL",
                            profile.provider_label
                        )
                    })?;
            let key_env = custom_spec
                .as_ref()
                .map(|spec| spec.api_key_env.as_str())
                .or_else(|| provider_spec.map(|spec| spec.api_key_env))
                .unwrap_or("");
            let bearer = if key_env.is_empty() {
                None
            } else {
                Some(
                    std::env::var(key_env)
                        .ok()
                        .filter(|value| !value.trim().is_empty())
                        .with_context(|| {
                            format!(
                                "coordinator provider {} requires {key_env}",
                                profile.provider_label
                            )
                        })?,
                )
            };
            Ok(CoordinatorEndpoint {
                url: format!("{}/chat/completions", base.trim_end_matches('/')),
                bearer,
                model,
            })
        }
        CoordinatorTransport::Managed => {
            if !ultra_billing_active(config) {
                bail!("managed coordinator transport requires an active managed entitlement");
            }
            if !matches!(
                profile.provider,
                ProviderId::Openai | ProviderId::Deepseek | ProviderId::Openrouter
            ) {
                bail!(
                    "managed coordinator transport does not expose provider {}",
                    profile.provider_label
                );
            }
            let base = config.billing.inference_proxy_base_url();
            if base.is_empty() {
                bail!("managed coordinator transport requires inference_proxy_url");
            }
            let bearer = std::env::var("PYTXO_ULTRA_SESSION")
                .ok()
                .filter(|value| !value.trim().is_empty());
            Ok(CoordinatorEndpoint {
                url: format!(
                    "{}/{}/v1/chat/completions",
                    base.trim_end_matches('/'),
                    profile.provider.as_str()
                ),
                bearer,
                model,
            })
        }
    }
}

fn suggest_verify_commands(repo: &Path) -> Vec<String> {
    let mut cmds = Vec::new();
    if repo.join("Cargo.toml").is_file() {
        cmds.push("cargo test".into());
    }
    if repo.join("package.json").is_file() {
        // Prefer npm test when present; do not invent scripts.
        if let Ok(raw) = std::fs::read_to_string(repo.join("package.json")) {
            if serde_json::from_str::<serde_json::Value>(&raw)
                .ok()
                .and_then(|value| {
                    value
                        .get("scripts")?
                        .get("test")?
                        .as_str()
                        .map(str::to_owned)
                })
                .is_some_and(|script| !script.trim().is_empty())
            {
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
    if repo.join("build.gradle").is_file() || repo.join("build.gradle.kts").is_file() {
        #[cfg(windows)]
        if repo.join("gradlew.bat").is_file() {
            cmds.push(".\\gradlew.bat test build".into());
        }
        #[cfg(not(windows))]
        if repo.join("gradlew").is_file() {
            cmds.push("./gradlew test build".into());
        }
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

fn validate_mission_plan(mut plan: MissionPlan, repo: &Path) -> anyhow::Result<MissionPlan> {
    if plan.tasks.is_empty() {
        bail!("planner returned no reviewable tasks");
    }
    let mut ids = HashSet::new();
    for task in &mut plan.tasks {
        let id = task.id.0.trim();
        if id.is_empty()
            || !id.chars().all(|character| {
                character.is_ascii_alphanumeric() || matches!(character, '-' | '_')
            })
        {
            bail!("planner returned unsafe task id: {}", task.id.0);
        }
        if !ids.insert(task.id.0.clone()) {
            bail!("planner returned duplicate task id: {}", task.id.0);
        }
        if task.agent.trim().is_empty() {
            bail!("planner task {} has no agent", task.id.0);
        }
        let prompt = plan
            .task_prompts
            .get(&task.id.0)
            .map(|prompt| prompt.trim())
            .unwrap_or_default();
        if prompt.is_empty() {
            bail!("planner task {} has no execution prompt", task.id.0);
        }
        if task.paths.is_empty() {
            bail!("planner task {} has no safe ownership paths", task.id.0);
        }
        let mut unique_paths = HashSet::new();
        for path in &mut task.paths {
            let normalized = validate_planner_path(repo, path)?;
            if !unique_paths.insert(normalized.clone()) {
                bail!(
                    "planner task {} repeats ownership path {normalized}",
                    task.id.0
                );
            }
            *path = normalized;
        }
    }

    for task in &plan.tasks {
        let mut dependencies = HashSet::new();
        for dependency in &task.depends_on {
            if dependency == &task.id.0 {
                bail!("planner task {} depends on itself", task.id.0);
            }
            if !ids.contains(dependency) {
                bail!(
                    "planner task {} depends on unknown task {dependency}",
                    task.id.0
                );
            }
            if !dependencies.insert(dependency) {
                bail!("planner task {} repeats dependency {dependency}", task.id.0);
            }
        }
    }
    validate_acyclic(&plan.tasks)?;
    Ok(plan)
}

fn validate_planner_path(repo: &Path, path: &str) -> anyhow::Result<String> {
    let normalized = path
        .replace('\\', "/")
        .trim()
        .trim_end_matches('/')
        .to_string();
    let parsed = Path::new(&normalized);
    let windows_absolute = normalized.as_bytes().get(1) == Some(&b':');
    if normalized.is_empty()
        || normalized == "."
        || normalized.starts_with('/')
        || windows_absolute
        || parsed.is_absolute()
        || parsed
            .components()
            .any(|component| !matches!(component, Component::Normal(_)))
        || normalized
            .chars()
            .any(|character| matches!(character, '*' | '?' | '[' | ']' | '{' | '}'))
    {
        bail!("planner returned unsafe ownership path: {path}");
    }
    let candidate = repo.join(&normalized);
    if !candidate.exists() && !candidate.parent().is_some_and(|parent| parent.is_dir()) {
        bail!("planner returned path outside the known repository structure: {normalized}");
    }
    Ok(normalized)
}

fn validate_acyclic(tasks: &[Task]) -> anyhow::Result<()> {
    let mut remaining: HashMap<&str, usize> = tasks
        .iter()
        .map(|task| (task.id.0.as_str(), task.depends_on.len()))
        .collect();
    let mut ready: Vec<&str> = remaining
        .iter()
        .filter_map(|(id, count)| (*count == 0).then_some(*id))
        .collect();
    let mut processed = 0usize;
    while let Some(completed) = ready.pop() {
        processed += 1;
        for task in tasks {
            if task
                .depends_on
                .iter()
                .any(|dependency| dependency == completed)
            {
                let count = remaining
                    .get_mut(task.id.0.as_str())
                    .expect("validated task id");
                *count -= 1;
                if *count == 0 {
                    ready.push(task.id.0.as_str());
                }
            }
        }
    }
    if processed != tasks.len() {
        bail!("planner returned a dependency cycle");
    }
    Ok(())
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
    let plan = default_mission_planner(config)
        .decompose(&spec, &ctx)
        .with_context(|| format!("planner failed for mission: {}", spec.text))?;
    validate_mission_plan(plan, repo)
        .with_context(|| format!("planner produced an unsafe plan for mission: {}", spec.text))
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
    let plan = default_planner(config)
        .decompose(&spec, &ctx)
        .with_context(|| format!("planner failed for mission: {}", spec.text))?;
    validate_mission_plan(plan, repo)
        .with_context(|| format!("planner produced an unsafe plan for mission: {}", spec.text))
}

const BRIEF_MAX_FILES: usize = 160;
const BRIEF_MAX_CHARS: usize = 16_000;
const MANIFEST_MAX_CHARS: usize = 3_000;

/// Build the bounded repository context used by cloud planners.
///
/// This is an Orbit/Galaxy planning boundary scoped to one execution domain:
/// every repository-derived byte is sanitized before either BYOK or managed
/// transport can observe it. Local planners continue to read the repository
/// directly and do not consume this egress representation.
fn repository_brief(repo: &Path) -> String {
    let mut files = Vec::new();
    collect_inventory(repo, repo, 0, &mut files);
    files.sort();
    files.truncate(BRIEF_MAX_FILES);

    let mut brief = String::from("Repository inventory (repo-relative, bounded):\n");
    for path in &files {
        brief.push_str("- ");
        brief.push_str(path);
        brief.push('\n');
    }

    for manifest in [
        "Cargo.toml",
        "package.json",
        "pyproject.toml",
        "go.mod",
        "pytxo.toml",
    ] {
        let path = repo.join(manifest);
        let Ok(raw) = std::fs::read_to_string(path) else {
            continue;
        };
        brief.push_str("\nManifest excerpt: ");
        brief.push_str(manifest);
        brief.push('\n');
        brief.extend(raw.chars().take(MANIFEST_MAX_CHARS));
        brief.push('\n');
    }

    pytxo_sanitize::sanitize_line(&brief)
        .chars()
        .take(BRIEF_MAX_CHARS)
        .collect()
}

fn collect_inventory(repo: &Path, directory: &Path, depth: usize, files: &mut Vec<String>) {
    if depth > 4 || files.len() >= BRIEF_MAX_FILES {
        return;
    }
    let Ok(entries) = std::fs::read_dir(directory) else {
        return;
    };
    let mut entries: Vec<_> = entries.flatten().collect();
    entries.sort_by_key(|entry| entry.file_name());
    for entry in entries {
        if files.len() >= BRIEF_MAX_FILES {
            break;
        }
        let path = entry.path();
        let name = entry.file_name();
        let name = name.to_string_lossy();
        if path.is_dir() {
            if matches!(
                name.as_ref(),
                ".git"
                    | ".pytxo"
                    | ".next"
                    | "node_modules"
                    | "target"
                    | "dist"
                    | "build"
                    | "coverage"
            ) {
                continue;
            }
            collect_inventory(repo, &path, depth + 1, files);
        } else if path.is_file() {
            if let Ok(relative) = path.strip_prefix(repo) {
                files.push(normalize_rel(&relative.to_string_lossy()));
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use std::sync::Mutex;

    use super::*;

    static PLANNER_ENV_LOCK: Mutex<()> = Mutex::new(());

    fn planner_env_guard() -> std::sync::MutexGuard<'static, ()> {
        PLANNER_ENV_LOCK
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
    }

    #[test]
    fn ambient_provider_key_does_not_authorize_cloud_planning() {
        let _guard = planner_env_guard();
        let _env = PlannerTestEnv::new();
        std::env::set_var("OPENAI_API_KEY", "test-only-not-a-real-key");
        let config = PytxoConfig::default();
        assert!(
            !llm_planner_enabled(&config),
            "an inherited key is not planner consent"
        );
        assert_ne!(planner_mode(&config), PlannerMode::Llm);
        assert_eq!(mission_planner_mode(&config), PlannerMode::Signal);
        let dir = tempfile::tempdir().unwrap();
        std::fs::write(dir.path().join("lib.rs"), "pub fn example() {}\n").unwrap();
        let context = PlannerContext {
            repo: dir.path(),
            config: &config,
        };
        let mission = MissionSpec {
            text: "Improve lib.rs".into(),
        };
        let plan = default_mission_planner(&config)
            .decompose(&mission, &context)
            .unwrap();
        assert_eq!(plan.tasks.len(), 1);
        assert!(LlmPlanner
            .decompose(&mission, &context)
            .unwrap_err()
            .to_string()
            .contains("model coordinator is not enabled"));
    }

    #[test]
    fn explicit_local_planner_wins_over_cloud_flag_and_key() {
        let _guard = planner_env_guard();
        let _env = PlannerTestEnv::new();
        std::env::set_var("OPENAI_API_KEY", "test-only-not-a-real-key");
        std::env::set_var("PYTXO_PLANNER_LLM", "1");
        for mode in ["heuristic", "signal", "0", "false"] {
            std::env::set_var("PYTXO_PLANNER", mode);
            assert!(
                !llm_planner_enabled(&PytxoConfig::default()),
                "explicit {mode} must remain local"
            );
            let expected = if mode == "heuristic" {
                PlannerMode::Heuristic
            } else {
                PlannerMode::Signal
            };
            assert_eq!(mission_planner_mode(&PytxoConfig::default()), expected);
            assert_eq!(planner_mode(&PytxoConfig::default()), expected);
        }
    }

    #[test]
    fn cloud_planner_requires_explicit_flag_and_available_transport() {
        let _guard = planner_env_guard();
        let _env = PlannerTestEnv::new();
        std::env::set_var("PYTXO_PLANNER_LLM", "1");
        assert!(!llm_planner_enabled(&PytxoConfig::default()));
        std::env::set_var("OPENAI_API_KEY", "test-only-not-a-real-key");
        assert!(
            !llm_planner_enabled(&PytxoConfig::default()),
            "an unrelated provider key must not change the configured coordinator"
        );
        std::env::set_var("DEEPSEEK_API_KEY", "test-only-not-a-real-key");
        assert!(llm_planner_enabled(&PytxoConfig::default()));
        assert_eq!(
            mission_planner_mode(&PytxoConfig::default()),
            PlannerMode::Llm
        );
        std::env::remove_var("DEEPSEEK_API_KEY");
        let mut managed = PytxoConfig::default();
        managed.coordinator.transport = CoordinatorTransport::Managed;
        std::env::set_var("PYTXO_LINK_TIER", "ultra");
        assert!(llm_planner_enabled(&managed));
        assert_eq!(mission_planner_mode(&managed), PlannerMode::Llm);
        std::env::remove_var("PYTXO_PLANNER_LLM");
        assert!(!llm_planner_enabled(&PytxoConfig::default()));
    }

    #[test]
    fn local_openai_compatible_coordinator_needs_no_cloud_or_key() {
        let _guard = planner_env_guard();
        let _env = PlannerTestEnv::new();
        std::env::set_var("PYTXO_PLANNER_LLM", "1");
        let mut config = PytxoConfig::default();
        config.coordinator.provider = "ollama".into();
        config.coordinator.model = "qwen3:8b".into();

        assert!(llm_planner_enabled(&config));
        let endpoint = coordinator_endpoint(&config).unwrap();
        assert_eq!(endpoint.url, "http://127.0.0.1:11434/v1/chat/completions");
        assert_eq!(endpoint.model, "qwen3:8b");
        assert!(endpoint.bearer.is_none());
    }

    // Restore even after an assertion fails; never send a request with these keys.
    struct PlannerTestEnv(Vec<(&'static str, Option<std::ffi::OsString>)>);

    impl PlannerTestEnv {
        fn new() -> Self {
            Self(
                [
                    "PYTXO_PLANNER",
                    "PYTXO_PLANNER_LLM",
                    "PYTXO_MISSION_PLAN",
                    "PYTXO_LINK_TIER",
                    "PYTXO_PLANNER_MODEL",
                    "PYTXO_ULTRA_SESSION",
                    "DEEPSEEK_API_KEY",
                    "OPENAI_API_KEY",
                    "OPENROUTER_API_KEY",
                    "MISTRAL_API_KEY",
                ]
                .into_iter()
                .map(|name| {
                    let previous = std::env::var_os(name);
                    std::env::remove_var(name);
                    (name, previous)
                })
                .collect(),
            )
        }
    }

    impl Drop for PlannerTestEnv {
        fn drop(&mut self) {
            for (name, previous) in &self.0 {
                match previous {
                    Some(value) => std::env::set_var(name, value),
                    None => std::env::remove_var(name),
                }
            }
        }
    }

    #[test]
    fn one_worker_preserves_every_requested_task() {
        let dir = tempfile::tempdir().unwrap();
        for name in ["one.rs", "two.rs", "three.rs"] {
            std::fs::write(dir.path().join(name), "pub fn example() {}\n").unwrap();
        }
        let config = PytxoConfig {
            max_agents: 1,
            ..PytxoConfig::default()
        };
        let context = PlannerContext {
            repo: dir.path(),
            config: &config,
        };
        let mission = MissionSpec {
            text: "fix one.rs; fix two.rs; fix three.rs".into(),
        };
        for planner in [
            &SignalBackedPlanner as &dyn MissionPlanner,
            &HeuristicPlanner,
        ] {
            let plan = planner.decompose(&mission, &context).unwrap();
            assert_eq!(
                plan.tasks.len(),
                3,
                "concurrency must not truncate mission scope"
            );
            assert_eq!(plan.task_prompts["mission-2"], "fix three.rs");
        }
    }

    #[test]
    fn npm_verification_requires_an_actual_nonempty_script() {
        let dir = tempfile::tempdir().unwrap();
        for manifest in [
            r#"{"name":"test"}"#,
            r#"{"scripts":{"test":""}}"#,
            r#"{"scripts":{"test":true}}"#,
        ] {
            std::fs::write(dir.path().join("package.json"), manifest).unwrap();
            assert!(suggest_verify_commands(dir.path()).is_empty());
        }
        std::fs::write(
            dir.path().join("package.json"),
            r#"{"scripts":{"test":"node --test"}}"#,
        )
        .unwrap();
        assert_eq!(suggest_verify_commands(dir.path()), vec!["npm test"]);
    }

    #[test]
    fn gradle_verification_uses_the_repository_wrapper() {
        let dir = tempfile::tempdir().unwrap();
        std::fs::write(dir.path().join("build.gradle.kts"), "plugins {}\n").unwrap();
        assert!(suggest_verify_commands(dir.path()).is_empty());
        #[cfg(windows)]
        let (wrapper, command) = ("gradlew.bat", ".\\gradlew.bat test build");
        #[cfg(not(windows))]
        let (wrapper, command) = ("gradlew", "./gradlew test build");
        std::fs::write(dir.path().join(wrapper), "").unwrap();
        assert_eq!(suggest_verify_commands(dir.path()), vec![command]);
    }

    #[test]
    fn stub_errors_when_disabled() {
        let _guard = planner_env_guard();
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
        let _guard = planner_env_guard();
        let prev = std::env::var("PYTXO_PLANNER").ok();
        std::env::set_var("PYTXO_PLANNER", "0");
        let dir = tempfile::tempdir().unwrap();
        std::fs::create_dir_all(dir.path().join("src")).unwrap();
        std::fs::create_dir_all(dir.path().join("docs")).unwrap();
        std::fs::write(dir.path().join("src/auth.rs"), "pub fn auth() {}\n").unwrap();
        std::fs::write(dir.path().join("docs/README.md"), "# Docs\n").unwrap();
        let mut cfg = PytxoConfig::default();
        cfg.planner.enabled = false;
        cfg.max_agents = 2;
        let plan =
            plan_mission("fix src/auth.rs; update docs/README.md", dir.path(), &cfg).unwrap();
        assert!(!plan.tasks.is_empty());
        match prev {
            Some(v) => std::env::set_var("PYTXO_PLANNER", v),
            None => std::env::remove_var("PYTXO_PLANNER"),
        }
    }

    #[test]
    fn heuristic_splits_sentences_when_enabled() {
        let _guard = planner_env_guard();
        let prev = std::env::var("PYTXO_PLANNER").ok();
        std::env::set_var("PYTXO_PLANNER", "heuristic");
        let dir = tempfile::tempdir().unwrap();
        std::fs::create_dir_all(dir.path().join("tests")).unwrap();
        std::fs::write(dir.path().join("tests/auth.rs"), "#[test] fn auth() {}\n").unwrap();
        std::fs::write(
            dir.path().join("package.json"),
            "{\"scripts\":{\"test\":\"x\"}}\n",
        )
        .unwrap();
        let mut cfg = PytxoConfig::default();
        cfg.planner.enabled = true;
        cfg.max_agents = 3;
        let plan =
            plan_mission("fix tests/auth.rs; update package.json", dir.path(), &cfg).unwrap();
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
        let _guard = planner_env_guard();
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
        let _guard = planner_env_guard();
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

    #[test]
    fn signal_planner_keeps_unrelated_explicit_paths_parallel() {
        let _guard = planner_env_guard();
        let prev = std::env::var("PYTXO_PLANNER").ok();
        std::env::set_var("PYTXO_PLANNER", "signal");
        let dir = tempfile::tempdir().unwrap();
        let repo = dir.path();
        std::fs::create_dir_all(repo.join("src")).unwrap();
        std::fs::write(repo.join("src/a.ts"), "export const a = 1;\n").unwrap();
        std::fs::write(repo.join("README.md"), "# Demo\n").unwrap();

        let mut cfg = PytxoConfig::default();
        cfg.planner.enabled = true;
        cfg.planner.mode = "signal".into();
        cfg.max_agents = 3;

        let plan = plan_mission("update src/a.ts; fix a typo in README.md.", repo, &cfg).unwrap();
        assert_eq!(plan.tasks[1].paths, vec!["README.md"]);
        assert!(
            plan.tasks.iter().all(|task| task.depends_on.is_empty()),
            "unrelated explicit paths should share a safe parallel wave"
        );

        match prev {
            Some(v) => std::env::set_var("PYTXO_PLANNER", v),
            None => std::env::remove_var("PYTXO_PLANNER"),
        }
    }

    #[test]
    fn documentation_of_combined_results_waits_for_earlier_code_and_tests() {
        let dir = tempfile::tempdir().unwrap();
        for file in ["src/a.mjs", "test/a.test.mjs", "README.md"] {
            let path = dir.path().join(file);
            std::fs::create_dir_all(path.parent().unwrap()).unwrap();
            std::fs::write(path, "// fixture\n").unwrap();
        }
        let cfg = PytxoConfig::default();
        let plan = SignalBackedPlanner.decompose(&MissionSpec { text:
            "update src/a.mjs; add regressions in test/a.test.mjs; in README.md, document the final combined behavior".into()
        }, &PlannerContext { repo: dir.path(), config: &cfg }).unwrap();
        assert_eq!(plan.tasks[2].depends_on, vec!["mission-0", "mission-1"]);
        assert!(plan.tasks[0].depends_on.is_empty());
        assert!(plan.tasks[1].depends_on.is_empty());
        let doc = dir.path().join("docs/explain.md");
        std::fs::create_dir_all(doc.parent().unwrap()).unwrap();
        std::fs::write(doc, "typo\n").unwrap();
        let independent = SignalBackedPlanner
            .decompose(
                &MissionSpec {
                    text: "update src/a.mjs; fix a typo in docs/explain.md".into(),
                },
                &PlannerContext {
                    repo: dir.path(),
                    config: &cfg,
                },
            )
            .unwrap();
        assert!(independent
            .tasks
            .iter()
            .all(|task| task.depends_on.is_empty()));
    }

    #[test]
    fn repository_brief_is_bounded_and_excludes_build_output() {
        let dir = tempfile::tempdir().unwrap();
        let repo = dir.path();
        std::fs::create_dir_all(repo.join("src")).unwrap();
        std::fs::create_dir_all(repo.join("target/debug")).unwrap();
        std::fs::write(
            repo.join("Cargo.toml"),
            "[package]\nname = \"brief-demo\"\nversion = \"0.1.0\"\n",
        )
        .unwrap();
        std::fs::write(repo.join("src/lib.rs"), "pub fn demo() {}\n").unwrap();
        std::fs::write(
            repo.join("target/debug/generated.rs"),
            "secret build output",
        )
        .unwrap();

        let brief = repository_brief(repo);

        assert!(brief.contains("Cargo.toml"));
        assert!(brief.contains("src/lib.rs"));
        assert!(brief.contains("brief-demo"));
        assert!(!brief.contains("target/debug"));
        assert!(brief.len() <= 16_000);
    }

    #[test]
    fn repository_brief_redacts_manifest_secrets_before_cloud_egress() {
        let dir = tempfile::tempdir().unwrap();
        let repo = dir.path();
        std::fs::write(
            repo.join("pytxo.toml"),
            "provider_api_key = \"sk-abcdefghijklmnopqrstuvwxyz1234567890\"\n\
             session_token = \"super-secret-session-token\"\n\
             [planner]\nmode = \"signal\"\n",
        )
        .unwrap();

        let brief = repository_brief(repo);

        assert!(
            brief.contains("[planner]"),
            "useful manifest structure is retained"
        );
        assert!(brief.contains("mode = \"signal\""));
        assert!(!brief.contains("sk-abcdefghijklmnopqrstuvwxyz1234567890"));
        assert!(!brief.contains("super-secret-session-token"));
        assert!(brief.contains("[REDACTED"));
    }

    #[test]
    fn signal_planner_rejects_an_unscoped_mission_instead_of_claiming_repo_root() {
        let _guard = planner_env_guard();
        let previous = std::env::var("PYTXO_PLANNER").ok();
        std::env::set_var("PYTXO_PLANNER", "signal");
        let dir = tempfile::tempdir().unwrap();
        std::fs::create_dir_all(dir.path().join("src")).unwrap();
        std::fs::write(dir.path().join("src/lib.rs"), "pub fn demo() {}\n").unwrap();
        let mut cfg = PytxoConfig::default();
        cfg.planner.enabled = true;
        cfg.planner.mode = "signal".into();

        let error = plan_mission("make it better", dir.path(), &cfg)
            .expect_err("unknown ownership must require review");

        assert!(
            format!("{error:#}").contains("safe ownership paths"),
            "unexpected error: {error:#}"
        );
        match previous {
            Some(value) => std::env::set_var("PYTXO_PLANNER", value),
            None => std::env::remove_var("PYTXO_PLANNER"),
        }
    }
}
