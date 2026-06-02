use pytxo_core::{ExecutionPlan, FidelityTier, IsolationMode, RunId, ScheduledTask, TaskId};
use pytxo_runner::{execute_plan, ProcessRegistry, RunContext, SwarmRegistry};
use std::process::Command;
use tempfile::TempDir;

fn init_git_repo(path: &std::path::Path) {
    assert!(Command::new("git")
        .args(["init"])
        .current_dir(path)
        .status()
        .unwrap()
        .success());
    assert!(Command::new("git")
        .args(["config", "user.email", "pytxo@test.local"])
        .current_dir(path)
        .status()
        .unwrap()
        .success());
    assert!(Command::new("git")
        .args(["config", "user.name", "Pytxo Test"])
        .current_dir(path)
        .status()
        .unwrap()
        .success());
    std::fs::write(path.join("README.md"), "test\n").unwrap();
    Command::new("git")
        .args(["add", "."])
        .current_dir(path)
        .status()
        .unwrap();
    Command::new("git")
        .args(["commit", "-m", "init"])
        .current_dir(path)
        .status()
        .unwrap();
}

#[tokio::test]
async fn single_agent_echo_in_worktree() {
    let tmp = TempDir::new().unwrap();
    let repo = tmp.path();
    init_git_repo(repo);

    let run_id = RunId::new();
    let worktree_base = repo.join(".pytxo/worktrees");
    std::fs::create_dir_all(&worktree_base).unwrap();

    let plan = ExecutionPlan {
        waves: vec![vec![ScheduledTask {
            task_id: TaskId("t0".into()),
            agent: "default".into(),
            paths: vec!["README.md".into()],
            wave: 0,
        }]],
        conflicts: vec![],
        max_agents: 3,
    };

    let data_dir = repo.join(".pytxo/data");
    std::fs::create_dir_all(&data_dir).unwrap();
    let ctx = RunContext {
        run_id,
        repo_root: repo.to_path_buf(),
        worktree_base,
        data_dir,
        cmd: "echo pytxo".into(),
        keep_worktrees: false,
        on_event: None,
        signal_core: false,
        signal_fidelity: FidelityTier::Low,
        isolation_mode: IsolationMode::Worktree,
    };

    let registry = ProcessRegistry::default();
    let swarm = SwarmRegistry::new();
    let results = execute_plan(&ctx, &plan, &registry, &swarm).await.unwrap();
    assert_eq!(results.len(), 1);
    assert_eq!(results[0].exit_code, Some(0));
    assert!(results[0].stdout.contains("pytxo"));
}
