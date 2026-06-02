use std::io::{BufRead, BufReader};
use std::path::{Path, PathBuf};
use std::process::Stdio;
use std::sync::Arc;
use std::thread;

use pytxo_core::{
    AgentId, ExecutionPlan, FidelityTier, IsolationCtx, IsolationMode, PytxoError, RaceShield,
    Result, RunId, ScheduledTask,
};

use crate::context::prepare_agent_context;
use crate::git::remove_worktree;
use crate::process::{ChildRecord, ProcessRegistry};
use crate::process_registry_file::{registry_path, ProcessEntry, ProcessRegistryFile};
use crate::race::SwarmRegistry;

pub type EventCallback = Arc<dyn Fn(&str, &str, &str) + Send + Sync>;

#[derive(Clone)]
pub struct RunContext {
    pub run_id: RunId,
    pub repo_root: PathBuf,
    pub worktree_base: PathBuf,
    pub data_dir: PathBuf,
    pub cmd: String,
    pub keep_worktrees: bool,
    pub on_event: Option<EventCallback>,
    pub signal_core: bool,
    pub signal_fidelity: FidelityTier,
    pub isolation_mode: IsolationMode,
}

#[derive(Clone, Debug)]
pub struct AgentRunResult {
    pub agent_id: AgentId,
    pub task_id: String,
    pub wave: u32,
    pub worktree_path: PathBuf,
    pub exit_code: Option<i32>,
    pub stdout: String,
    pub stderr: String,
}

struct SingleResult {
    worktree_path: PathBuf,
    exit_code: Option<i32>,
    stdout: String,
    stderr: String,
    pid: Option<u32>,
}

pub async fn execute_plan(
    ctx: &RunContext,
    plan: &ExecutionPlan,
    registry: &ProcessRegistry,
    swarm: &SwarmRegistry,
) -> Result<Vec<AgentRunResult>> {
    let mut all_results = Vec::new();
    let mut agent_index = 0usize;

    for wave in &plan.waves {
        let mut set = tokio::task::JoinSet::new();
        for task in wave {
            let agent_id = AgentId::new(agent_index);
            agent_index += 1;
            let ctx = ctx.clone();
            let task = task.clone();
            let registry = registry.clone();
            let swarm = swarm.clone();
            set.spawn(
                async move { run_one_agent(&ctx, &task, &agent_id, &registry, &swarm).await },
            );
        }

        while let Some(joined) = set.join_next().await {
            let result = joined.map_err(|e| PytxoError::Runner(format!("join: {e}")))??;
            all_results.push(result);
        }
    }

    let mut proc_file = ProcessRegistryFile::load(&registry_path(&ctx.data_dir))?;
    proc_file.remove_run(&ctx.run_id.0);
    proc_file.save(&registry_path(&ctx.data_dir))?;

    Ok(all_results)
}

async fn run_one_agent(
    ctx: &RunContext,
    task: &ScheduledTask,
    agent_id: &AgentId,
    registry: &ProcessRegistry,
    swarm: &SwarmRegistry,
) -> Result<AgentRunResult> {
    let agent_key = format!("{}:{}", ctx.run_id, agent_id);
    swarm.try_claim_paths(&agent_key, &task.paths)?;

    let isolation = crate::blast::isolation_for_mode(ctx.isolation_mode);
    let iso_ctx = IsolationCtx {
        run_id: ctx.run_id.clone(),
        agent_id: agent_id.clone(),
        repo_root: ctx.repo_root.clone(),
        worktree_base: ctx.worktree_base.clone(),
    };

    let workspace = isolation.prepare(&iso_ctx)?;
    let wt_path = workspace.cwd.clone();
    let branch = workspace.branch.clone();

    let context_dir = prepare_agent_context(
        &ctx.repo_root,
        &ctx.data_dir,
        &ctx.run_id,
        &agent_id.0,
        &task.paths,
        ctx.signal_core,
        ctx.signal_fidelity,
    )?;

    registry.register(ChildRecord {
        run_id: ctx.run_id.clone(),
        agent_id: agent_id.clone(),
        worktree_path: wt_path.clone(),
        branch: branch.clone(),
        pid: None,
    });

    let result = tokio::task::spawn_blocking({
        let wt_path = wt_path.clone();
        let cmd = ctx.cmd.clone();
        let on_event = ctx.on_event.clone();
        let agent_key = agent_key.clone();
        let run_id = ctx.run_id.0.clone();
        let repo_root = ctx.repo_root.to_string_lossy().to_string();
        let data_dir = ctx.data_dir.clone();
        let branch = branch.clone();
        let context_dir = context_dir.clone();
        let swarm = swarm.clone();
        move || {
            let out = run_command_streaming(
                &wt_path,
                &cmd,
                on_event.as_ref(),
                &agent_key,
                context_dir.as_deref(),
                Some(ProcessPersist {
                    run_id,
                    repo_root,
                    data_dir,
                    branch,
                }),
            );
            if let Ok(ref res) = out {
                if let Some(pid) = res.pid {
                    swarm.register_pid(&agent_key, pid);
                }
            }
            out
        }
    })
    .await
    .map_err(|e| PytxoError::Runner(format!("join: {e}")))??;

    swarm.release(&agent_key);

    if !ctx.keep_worktrees {
        let _ = isolation.rollback(&iso_ctx, &workspace);
    }

    Ok(AgentRunResult {
        agent_id: agent_id.clone(),
        task_id: task.task_id.0.clone(),
        wave: task.wave,
        worktree_path: result.worktree_path,
        exit_code: result.exit_code,
        stdout: result.stdout,
        stderr: result.stderr,
    })
}

struct ProcessPersist {
    run_id: String,
    repo_root: String,
    data_dir: PathBuf,
    branch: String,
}

fn run_command_streaming(
    worktree: &Path,
    cmd: &str,
    on_event: Option<&EventCallback>,
    agent_key: &str,
    context_dir: Option<&Path>,
    persist: Option<ProcessPersist>,
) -> Result<SingleResult> {
    let shell = shell_command();
    let mut command = std::process::Command::new(&shell.0);
    command
        .args(&shell.1)
        .arg(cmd)
        .current_dir(worktree)
        .stdout(Stdio::piped())
        .stderr(Stdio::piped());
    if let Some(dir) = context_dir {
        command.env("PYTXO_CONTEXT_DIR", dir);
        command.env("PYTXO_SIGNAL_CORE", "1");
    }
    let mut child = command
        .spawn()
        .map_err(|e| PytxoError::Runner(format!("spawn command: {e}")))?;

    let pid = child.id();
    if let Some(p) = persist {
        let mut proc_file = ProcessRegistryFile::load(&registry_path(&p.data_dir))?;
        proc_file.push(ProcessEntry {
            run_id: p.run_id,
            repo_root: p.repo_root,
            agent_key: agent_key.to_string(),
            pid,
            worktree_path: worktree.to_string_lossy().to_string(),
            branch: p.branch,
        });
        proc_file.save(&registry_path(&p.data_dir))?;
    }
    let mut stdout_acc = String::new();
    let mut stderr_acc = String::new();

    let stdout = child.stdout.take();
    let stderr = child.stderr.take();
    let on_event_out = on_event.cloned();
    let on_event_err = on_event.cloned();
    let agent_key_out = agent_key.to_string();
    let agent_key_err = agent_key.to_string();

    let out_handle = stdout.map(|out| {
        thread::spawn(move || {
            let reader = BufReader::new(out);
            let mut acc = String::new();
            for line in reader.lines().map_while(|line| line.ok()) {
                if let Some(cb) = &on_event_out {
                    cb(&agent_key_out, "stdout", &line);
                }
                acc.push_str(&line);
                acc.push('\n');
            }
            acc
        })
    });

    let err_handle = stderr.map(|err| {
        thread::spawn(move || {
            let reader = BufReader::new(err);
            let mut acc = String::new();
            for line in reader.lines().map_while(|line| line.ok()) {
                if let Some(cb) = &on_event_err {
                    cb(&agent_key_err, "stderr", &line);
                }
                acc.push_str(&line);
                acc.push('\n');
            }
            acc
        })
    });

    let status = child
        .wait()
        .map_err(|e| PytxoError::Runner(format!("wait: {e}")))?;

    if let Some(h) = out_handle {
        if let Ok(acc) = h.join() {
            stdout_acc = acc;
        }
    }
    if let Some(h) = err_handle {
        if let Ok(acc) = h.join() {
            stderr_acc = acc;
        }
    }

    Ok(SingleResult {
        worktree_path: worktree.to_path_buf(),
        exit_code: status.code(),
        stdout: stdout_acc,
        stderr: stderr_acc,
        pid: Some(pid),
    })
}

fn shell_command() -> (String, Vec<String>) {
    if cfg!(windows) {
        ("cmd".into(), vec!["/C".into()])
    } else {
        ("sh".into(), vec!["-c".into()])
    }
}

pub fn stop_run(data_dir: &Path, run_id: &str, kill: bool) -> Result<Vec<u32>> {
    use crate::kill::kill_pids;

    let path = registry_path(data_dir);
    let mut file = ProcessRegistryFile::load(&path)?;
    let pids: Vec<u32> = file.for_run(run_id).iter().map(|e| e.pid).collect();
    if kill {
        kill_pids(&pids)?;
    }
    file.remove_run(run_id);
    file.save(&path)?;
    Ok(pids)
}

pub fn stop_all(data_dir: &Path, kill: bool) -> Result<()> {
    use crate::kill::kill_pids;

    let path = registry_path(data_dir);
    let mut file = ProcessRegistryFile::load(&path)?;
    let pids = file.all_pids();
    if kill {
        kill_pids(&pids)?;
    }
    file.clear();
    file.save(&path)?;
    Ok(())
}

pub fn cleanup_worktrees(ctx: &RunContext, registry: &ProcessRegistry) -> Result<()> {
    for record in registry.list() {
        if record.run_id == ctx.run_id {
            let _ = remove_worktree(&ctx.repo_root, &record.worktree_path, &record.branch, true);
        }
    }
    Ok(())
}
