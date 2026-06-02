use std::io::{BufRead, BufReader};
use std::path::{Path, PathBuf};
use std::process::Stdio;
use std::sync::Arc;
use std::thread;

use pytxo_core::{AgentId, ExecutionPlan, PytxoError, Result, RunId, ScheduledTask};

use crate::git::{branch_name, create_worktree, remove_worktree, worktree_path};
use crate::process::{ChildRecord, ProcessRegistry};
use crate::process_registry_file::{ProcessEntry, ProcessRegistryFile, registry_path};

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
}

pub async fn execute_plan(
    ctx: &RunContext,
    plan: &ExecutionPlan,
    registry: &ProcessRegistry,
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
            set.spawn(async move {
                let result = run_one_agent(&ctx, &task, &agent_id, &registry).await?;
                Ok::<_, PytxoError>(AgentRunResult {
                    agent_id,
                    task_id: task.task_id.0.clone(),
                    wave: task.wave,
                    worktree_path: result.worktree_path,
                    exit_code: result.exit_code,
                    stdout: result.stdout,
                    stderr: result.stderr,
                })
            });
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
    _task: &ScheduledTask,
    agent_id: &AgentId,
    registry: &ProcessRegistry,
) -> Result<SingleResult> {
    let branch = branch_name(&ctx.run_id.0, &agent_id.0);
    let wt_path = worktree_path(&ctx.worktree_base, &ctx.run_id.0, &agent_id.0);
    let agent_key = format!("{}:{}", ctx.run_id, agent_id);

    create_worktree(&ctx.repo_root, &wt_path, &branch)?;

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
        move || {
            run_command_streaming(
                &wt_path,
                &cmd,
                on_event.as_ref(),
                &agent_key,
                Some(ProcessPersist {
                    run_id,
                    repo_root,
                    data_dir,
                    branch,
                }),
            )
        }
    })
    .await
    .map_err(|e| PytxoError::Runner(format!("join: {e}")))??;

    if !ctx.keep_worktrees {
        let _ = remove_worktree(&ctx.repo_root, &wt_path, &branch, true);
    }

    Ok(result)
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
    persist: Option<ProcessPersist>,
) -> Result<SingleResult> {
    let shell = shell_command();
    let mut child = std::process::Command::new(&shell.0)
        .args(&shell.1)
        .arg(cmd)
        .current_dir(worktree)
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
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
    let pids: Vec<u32> = file
        .for_run(run_id)
        .iter()
        .map(|e| e.pid)
        .collect();
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
            let _ = remove_worktree(
                &ctx.repo_root,
                &record.worktree_path,
                &record.branch,
                true,
            );
        }
    }
    Ok(())
}
