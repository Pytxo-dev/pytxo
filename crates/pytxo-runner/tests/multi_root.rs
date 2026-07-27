use pytxo_core::{
    BillingMode, ExecutionPlan, FidelityTier, IsolationMode, PermissionProfile, RunId,
    ScheduledTask, TaskId,
};
use pytxo_runner::{execute_plan, ProcessRegistry, RootExec, RunContext, SwarmRegistry};
use std::collections::HashMap;
use std::process::Command;
use tempfile::TempDir;

fn init_git_repo(path: &std::path::Path, marker: &str) {
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
    std::fs::write(path.join("README.md"), format!("{marker}\n")).unwrap();
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
async fn secondary_root_worktree_under_web_repo() {
    let primary = TempDir::new().unwrap();
    let web = TempDir::new().unwrap();
    init_git_repo(primary.path(), "api");
    init_git_repo(web.path(), "web");

    let run_id = RunId::new();
    let primary_wt = primary.path().join(".pytxo/worktrees");
    let web_wt = web.path().join(".pytxo/worktrees");
    std::fs::create_dir_all(&primary_wt).unwrap();
    std::fs::create_dir_all(&web_wt).unwrap();

    let mut roots = HashMap::new();
    roots.insert(
        "web".into(),
        RootExec {
            repo_root: web.path().to_path_buf(),
            worktree_base: web_wt.clone(),
            read_only: false,
            permission_profile: PermissionProfile::Orbit,
        },
    );

    let plan = ExecutionPlan {
        waves: vec![vec![ScheduledTask {
            task_id: TaskId("t-web".into()),
            agent: "default".into(),
            paths: vec!["README.md".into()],
            wave: 0,
            root: Some("web".into()),
            signal_fidelity: None,
        verify: vec![],        }]],
        conflicts: vec![],
        max_agents: 3,
        warnings: vec![],
    };

    let data_dir = primary.path().join(".pytxo/data");
    std::fs::create_dir_all(&data_dir).unwrap();
    let (domain_id, model_router, managed_transport, token_estimator) =
        RunContext::default_metering(primary.path());
    let ctx = RunContext {
        run_id,
        repo_root: primary.path().to_path_buf(),
        worktree_base: primary_wt,
        data_dir,
        cmd: "echo pytxo".into(),
        task_cmd_template: None,
        task_prompts: HashMap::new(),
        keep_worktrees: true,
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
        agent_paths: HashMap::new(),
        agent_fidelity: HashMap::new(),
        roots,
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
    assert_eq!(results[0].root_id.as_deref(), Some("web"));

    let wt_canon = std::fs::canonicalize(&results[0].worktree_path).unwrap();
    let web_canon = std::fs::canonicalize(web.path()).unwrap();
    assert!(
        wt_canon.starts_with(&web_canon),
        "worktree should live under web repo, got {}",
        wt_canon.display()
    );
}
