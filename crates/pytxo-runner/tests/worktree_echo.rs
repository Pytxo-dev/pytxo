use pytxo_core::{
    BillingMode, ExecutionPlan, FidelityTier, IsolationMode, PermissionProfile, RunId,
    ScheduledTask, TaskId,
};
use pytxo_runner::{execute_plan, ProcessRegistry, RunContext, SwarmRegistry};
use std::collections::HashMap;
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
            root: None,
            signal_fidelity: None,
        verify: vec![],        }]],
        conflicts: vec![],
        max_agents: 3,
        warnings: vec![],
    };

    let data_dir = repo.join(".pytxo/data");
    std::fs::create_dir_all(&data_dir).unwrap();
    let (domain_id, model_router, managed_transport, token_estimator) =
        RunContext::default_metering(repo);
    let ctx = RunContext {
        run_id,
        repo_root: repo.to_path_buf(),
        worktree_base,
        data_dir,
        cmd: "echo pytxo".into(),
        task_cmd_template: None,
        task_prompts: HashMap::new(),
        keep_worktrees: false,
        on_event: None,
        signal_core: false,
        signal_fidelity: FidelityTier::Low,
        isolation_mode: IsolationMode::Worktree,
        permission_profile: PermissionProfile::Orbit,
        agent_profiles: HashMap::new(),
        route_agents: Vec::new(),
        billing_mode: BillingMode::Byok,
        domain_id,
        model_router,
        managed_transport,
        usage_meter: None,
        token_estimator,
        execution_backend: pytxo_core::ExecutionBackend::Subprocess,
        pty_rows: 24,
        pty_cols: 80,
        hitl: None,
        hitl_manual_flush: false,
        agent_paths: std::collections::HashMap::new(),
        agent_fidelity: std::collections::HashMap::new(),
        roots: std::collections::HashMap::new(),
        readonly_context_roots: Vec::new(),
        subprocess_stdin: false,
        cloud_dispatcher: None,
        context_cache: None,
        cloud_cache_enabled: false,
        cloud_fallback_local: true,
        mcp_hub: None,
        mcp_hub_enabled: false,
        mcp_allowlist: Vec::new(),
        sparse_exclude: Vec::new(),
    };

    let registry = ProcessRegistry::default();
    let swarm = SwarmRegistry::new();
    let results = execute_plan(&ctx, &plan, &registry, &swarm).await.unwrap();
    assert_eq!(results.len(), 1);
    assert_eq!(results[0].exit_code, Some(0));
    assert!(results[0].stdout.contains("pytxo"));
}
