use std::fs;
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};

use pytxo_core::{canonical_repo_root, PytxoConfig, PytxoError, RunId, Task, TaskId};
use pytxo_runner::{execute_plan, stop_all, stop_run, ProcessRegistry, RunContext};
use pytxo_scheduler::build_plan;
use pytxo_store::PytxoStore;
use serde::{Deserialize, Serialize};

mod cost;
pub use cost::{parse_cost_from_lines, CostEstimate};

#[cfg(feature = "sanitize")]
use pytxo_sanitize::sanitize_line;

pub struct RunOptions {
    pub agents: usize,
    pub cmd: String,
    pub config: Option<PathBuf>,
    pub dry_run: bool,
    pub keep_worktrees: bool,
    pub repo: Option<PathBuf>,
}

#[derive(Serialize, Deserialize)]
struct ActiveRunState {
    run_id: String,
    repo_root: String,
}

#[derive(Clone, Debug, Serialize)]
pub struct StatusJson {
    pub runs: Vec<RunStatusJson>,
}

#[derive(Clone, Debug, Serialize)]
pub struct RunStatusJson {
    pub id: String,
    pub status: String,
    pub repo_root: String,
    pub started_at: String,
    pub estimated_tokens_in: Option<i64>,
    pub estimated_tokens_out: Option<i64>,
    pub estimated_cost_usd: Option<f64>,
    pub agents: Vec<AgentStatusJson>,
}

#[derive(Clone, Debug, Serialize)]
pub struct AgentStatusJson {
    pub id: String,
    pub task_id: String,
    pub wave: i32,
    pub status: String,
    pub exit_code: Option<i32>,
}

pub fn init(repo: Option<PathBuf>) -> anyhow::Result<()> {
    let repo = repo.unwrap_or_else(|| std::env::current_dir().expect("cwd"));
    let cfg = PytxoConfig::default();
    fs::create_dir_all(repo.join(&cfg.worktree_dir))?;
    fs::create_dir_all(repo.join(&cfg.data_dir))?;
    ensure_gitignore(&repo)?;
    Ok(())
}

pub async fn run(opts: RunOptions) -> anyhow::Result<RunId> {
    let repo_root = opts
        .repo
        .unwrap_or_else(|| std::env::current_dir().expect("cwd"));
    let repo_root = canonical_repo_root(&repo_root)?;

    let mut cfg = load_config(opts.config.as_deref(), &repo_root)?;
    if opts.agents > 0 {
        cfg.max_agents = opts.agents;
    }

    let tasks = if cfg.task.is_empty() {
        synthetic_tasks(cfg.max_agents)
    } else {
        cfg.tasks()
    };

    let plan = build_plan(&tasks, cfg.max_agents, cfg.dag_explicit_deps)
        .map_err(|e| anyhow::anyhow!(e))?;

    if opts.dry_run {
        println!("{}", serde_json::to_string_pretty(&plan)?);
        return Ok(RunId::new());
    }

    fs::create_dir_all(repo_root.join(&cfg.worktree_dir))?;
    fs::create_dir_all(repo_root.join(&cfg.data_dir))?;

    let run_id = RunId::new();
    let store = PytxoStore::open(&cfg.db_path())?;
    store.insert_run(&run_id.0, &repo_root.to_string_lossy())?;

    let sanitize = cfg.sanitize;
    let store_cb = Arc::new(Mutex::new(store));
    let store_for_events = Arc::clone(&store_cb);
    let on_event = Arc::new(move |agent_key: &str, kind: &str, line: &str| {
        let payload = if sanitize {
            #[cfg(feature = "sanitize")]
            {
                sanitize_line(line)
            }
            #[cfg(not(feature = "sanitize"))]
            {
                line.to_string()
            }
        } else {
            line.to_string()
        };
        if let Ok(guard) = store_for_events.lock() {
            let _ = guard.append_event(agent_key, kind, &payload);
        }
    });

    let ctx = RunContext {
        run_id: run_id.clone(),
        repo_root: repo_root.clone(),
        worktree_base: repo_root.join(&cfg.worktree_dir),
        data_dir: repo_root.join(&cfg.data_dir),
        cmd: opts.cmd.clone(),
        keep_worktrees: opts.keep_worktrees,
        on_event: Some(on_event),
    };

    let registry = ProcessRegistry::default();
    save_active_run(&cfg, &run_id, &repo_root).map_err(|e| anyhow::anyhow!(e))?;

    let results = execute_plan(&ctx, &plan, &registry).await?;

    let mut failed = false;
    let mut all_lines: Vec<String> = Vec::new();
    let store = store_cb.lock().expect("store lock");
    for result in &results {
        let agent_key = format!("{}:{}", run_id, result.agent_id);
        store.insert_agent(
            &agent_key,
            &run_id.0,
            &result.task_id,
            result.wave,
            Some(&result.worktree_path.to_string_lossy()),
            &opts.cmd,
        )?;
        if !result.stdout.is_empty() {
            let payload = maybe_sanitize(&result.stdout, sanitize);
            store.append_event(&agent_key, "stdout", &payload)?;
            all_lines.extend(result.stdout.lines().map(String::from));
        }
        if !result.stderr.is_empty() {
            let payload = maybe_sanitize(&result.stderr, sanitize);
            store.append_event(&agent_key, "stderr", &payload)?;
            all_lines.extend(result.stderr.lines().map(String::from));
        }
        let exit = result.exit_code.unwrap_or(-1);
        let status = if exit == 0 { "completed" } else { "failed" };
        if exit != 0 {
            failed = true;
        }
        store.finish_agent(&agent_key, result.exit_code, status)?;
    }

    let line_refs: Vec<&str> = all_lines.iter().map(String::as_str).collect();
    let cost = parse_cost_from_lines(&line_refs);
    if cost.tokens_in > 0 || cost.tokens_out > 0 || cost.cost_usd > 0.0 {
        store.update_run_cost(
            &run_id.0,
            cost.tokens_in,
            cost.tokens_out,
            cost.cost_usd,
        )?;
    }

    let run_status = if failed { "failed" } else { "completed" };
    store.finish_run(&run_id.0, run_status)?;
    let _ = fs::remove_file(cfg.state_path());

    if failed && cfg.fail_fast {
        anyhow::bail!("one or more agents failed (fail_fast=true)");
    }

    Ok(run_id)
}

pub fn dry_run_json(
    config: Option<PathBuf>,
    repo: Option<PathBuf>,
    agents: usize,
) -> anyhow::Result<String> {
    let repo_root = repo.unwrap_or_else(|| std::env::current_dir().expect("cwd"));
    let repo_root = canonical_repo_root(&repo_root)?;
    let mut cfg = load_config(config.as_deref(), &repo_root)?;
    if agents > 0 {
        cfg.max_agents = agents;
    }
    let tasks = if cfg.task.is_empty() {
        synthetic_tasks(cfg.max_agents)
    } else {
        cfg.tasks()
    };
    let plan = build_plan(&tasks, cfg.max_agents, cfg.dag_explicit_deps)
        .map_err(|e| anyhow::anyhow!(e))?;
    Ok(serde_json::to_string_pretty(&plan)?)
}

pub fn status(config: Option<PathBuf>, limit: usize, json: bool) -> anyhow::Result<()> {
    let repo = std::env::current_dir()?;
    let cfg = load_config(config.as_deref(), &repo)?;
    let store = PytxoStore::open(&cfg.db_path())?;
    let runs = store.list_runs(limit)?;
    if json {
        let mut out = Vec::new();
        for run in &runs {
            let agents: Vec<AgentStatusJson> = store
                .list_agents_for_run(&run.id)?
                .into_iter()
                .map(|a| AgentStatusJson {
                    id: a.id,
                    task_id: a.task_id,
                    wave: a.wave,
                    status: a.status,
                    exit_code: a.exit_code,
                })
                .collect();
            out.push(RunStatusJson {
                id: run.id.clone(),
                status: run.status.clone(),
                repo_root: run.repo_root.clone(),
                started_at: run.started_at.to_rfc3339(),
                estimated_tokens_in: run.estimated_tokens_in,
                estimated_tokens_out: run.estimated_tokens_out,
                estimated_cost_usd: run.estimated_cost_usd,
                agents,
            });
        }
        println!("{}", serde_json::to_string_pretty(&StatusJson { runs: out })?);
        return Ok(());
    }
    if runs.is_empty() {
        println!("No runs recorded.");
        return Ok(());
    }
    for run in runs {
        println!(
            "run {}  status={}  repo={}  started={}  cost_usd={:?}",
            run.id,
            run.status,
            run.repo_root,
            run.started_at,
            run.estimated_cost_usd
        );
        for agent in store.list_agents_for_run(&run.id)? {
            println!(
                "  agent {}  task={}  wave={}  status={}  exit={:?}",
                agent.id, agent.task_id, agent.wave, agent.status, agent.exit_code
            );
        }
    }
    Ok(())
}

pub fn logs(config: Option<PathBuf>, agent: &str, tail: usize) -> anyhow::Result<Vec<String>> {
    let repo = std::env::current_dir()?;
    let cfg = load_config(config.as_deref(), &repo)?;
    let store = PytxoStore::open(&cfg.db_path())?;
    let events = store.list_events(agent, tail)?;
    Ok(events
        .into_iter()
        .map(|ev| format!("[{}] {}: {}", ev.ts, ev.kind, ev.payload.trim_end()))
        .collect())
}

pub async fn stop(
    config: Option<PathBuf>,
    all: bool,
    cleanup_worktrees: bool,
) -> anyhow::Result<()> {
    let repo = std::env::current_dir()?;
    let cfg = load_config(config.as_deref(), &repo)?;
    let data_dir = repo.join(&cfg.data_dir);

    if all {
        stop_all(&data_dir, true).map_err(|e| anyhow::anyhow!(e))?;
        if cfg.state_path().exists() {
            fs::remove_file(cfg.state_path())?;
        }
        println!("Stopped all tracked processes.");
        return Ok(());
    }

    let state_path = cfg.state_path();
    if !state_path.exists() {
        println!("No active run.");
        return Ok(());
    }
    let raw = fs::read_to_string(&state_path)?;
    let state: ActiveRunState = serde_json::from_str(&raw)?;
    let pids = stop_run(&data_dir, &state.run_id, true).map_err(|e| anyhow::anyhow!(e))?;
    if cleanup_worktrees {
        let ctx = RunContext {
            run_id: RunId(state.run_id.clone()),
            repo_root: PathBuf::from(&state.repo_root),
            worktree_base: PathBuf::from(&state.repo_root).join(&cfg.worktree_dir),
            data_dir: data_dir.clone(),
            cmd: String::new(),
            keep_worktrees: false,
            on_event: None,
        };
        let registry = ProcessRegistry::default();
        pytxo_runner::cleanup_worktrees(&ctx, &registry).map_err(|e| anyhow::anyhow!(e))?;
    }
    fs::remove_file(&state_path)?;
    println!("Stopped run {} (killed {} process(es))", state.run_id, pids.len());
    Ok(())
}

pub fn open_store(config: Option<PathBuf>) -> anyhow::Result<(PytxoConfig, PytxoStore)> {
    let repo = std::env::current_dir()?;
    let cfg = load_config(config.as_deref(), &repo)?;
    let store = PytxoStore::open(&cfg.db_path())?;
    Ok((cfg, store))
}

fn maybe_sanitize(s: &str, enabled: bool) -> String {
    if !enabled {
        return s.to_string();
    }
    #[cfg(feature = "sanitize")]
    {
        sanitize_line(s)
    }
    #[cfg(not(feature = "sanitize"))]
    {
        s.to_string()
    }
}

fn load_config(path: Option<&Path>, repo: &Path) -> anyhow::Result<PytxoConfig> {
    let path = path
        .map(|p| p.to_path_buf())
        .or_else(|| {
            let candidate = repo.join("pytxo.toml");
            if candidate.exists() {
                Some(candidate)
            } else {
                None
            }
        });
    if let Some(path) = path {
        PytxoConfig::load(&path).map_err(|e| anyhow::anyhow!(e))
    } else {
        Ok(PytxoConfig::default())
    }
}

fn synthetic_tasks(count: usize) -> Vec<Task> {
    (0..count)
        .map(|i| Task {
            id: TaskId(format!("synthetic-{i}")),
            agent: "default".into(),
            paths: vec![format!("src/agent-{i}.ts")],
            depends_on: Vec::new(),
        })
        .collect()
}

fn save_active_run(cfg: &PytxoConfig, run_id: &RunId, repo: &Path) -> Result<(), PytxoError> {
    let state = ActiveRunState {
        run_id: run_id.0.clone(),
        repo_root: repo.to_string_lossy().to_string(),
    };
    let json = serde_json::to_string_pretty(&state).map_err(|e| PytxoError::Other(e.to_string()))?;
    fs::write(cfg.state_path(), json).map_err(PytxoError::Io)?;
    Ok(())
}

fn ensure_gitignore(repo: &Path) -> anyhow::Result<()> {
    let gi = repo.join(".gitignore");
    let marker = ".pytxo/";
    if gi.exists() {
        let content = fs::read_to_string(&gi)?;
        if content.contains(marker) {
            return Ok(());
        }
        fs::write(gi, format!("{content}\n{marker}\n"))?;
    } else {
        fs::write(gi, format!("{marker}\n"))?;
    }
    Ok(())
}
