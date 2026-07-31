//! Closed-loop v2 ([[closed-loop-fidelity]]): a failing agent whose output
//! implicates one file re-scaffolds only that file at High fidelity, not the
//! whole task surface.

use std::collections::HashMap;
use std::process::Command;

use pytxo_core::{
    BillingMode, ExecutionBackend, ExecutionPlan, FidelityTier, IsolationMode, PermissionProfile,
    RunId, ScheduledTask, TaskId,
};
use pytxo_runner::{execute_plan, ProcessRegistry, RunContext, SwarmRegistry};
use tempfile::TempDir;

fn init_git_repo(path: &std::path::Path) {
    for args in [
        vec!["init"],
        vec!["config", "user.email", "pytxo@test.local"],
        vec!["config", "user.name", "Pytxo Test"],
    ] {
        assert!(Command::new("git")
            .args(&args)
            .current_dir(path)
            .status()
            .unwrap()
            .success());
    }
    std::fs::create_dir_all(path.join("src")).unwrap();
    std::fs::write(path.join("src/bad.rs"), "pub fn bad() -> i32 { 1 }\n").unwrap();
    std::fs::write(path.join("src/good.rs"), "pub fn good() -> i32 { 2 }\n").unwrap();
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
async fn failing_run_retries_only_implicated_file() {
    let tmp = TempDir::new().unwrap();
    let repo = tmp.path();
    init_git_repo(repo);

    let worktree_base = repo.join(".pytxo/worktrees");
    std::fs::create_dir_all(&worktree_base).unwrap();
    let data_dir = repo.join(".pytxo/data");
    std::fs::create_dir_all(&data_dir).unwrap();

    // Emit a diagnostic mentioning src/bad.rs, then fail. Avoid `>` so Windows
    // cmd does not treat the text as a redirection.
    let fail_cmd = if cfg!(windows) {
        "echo error at src/bad.rs:1:1 & exit 1"
    } else {
        "echo 'error at src/bad.rs:1:1'; exit 1"
    };

    let plan = ExecutionPlan {
        waves: vec![vec![ScheduledTask {
            task_id: TaskId("t0".into()),
            agent: "default".into(),
            paths: vec!["src/bad.rs".into(), "src/good.rs".into()],
            depends_on: vec![],
            wave: 0,
            root: None,
            signal_fidelity: None,
            verify: vec![],
        }]],
        conflicts: vec![],
        max_agents: 1,
        warnings: vec![],
    };

    let (domain_id, model_router, managed_transport, token_estimator) =
        RunContext::default_metering(repo);
    let run_id = RunId::new();
    let ctx = RunContext {
        run_id: run_id.clone(),
        repo_root: repo.to_path_buf(),
        worktree_base,
        data_dir: data_dir.clone(),
        cmd: fail_cmd.into(),
        task_cmd_template: None,
        task_prompts: HashMap::new(),
        keep_worktrees: false,
        on_event: None,
        signal_core: true,
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
        execution_backend: ExecutionBackend::Subprocess,
        pty_rows: 24,
        pty_cols: 80,
        hitl: None,
        hitl_manual_flush: false,
        agent_paths: HashMap::new(),
        agent_fidelity: HashMap::new(),
        roots: HashMap::new(),
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
    assert_eq!(results[0].exit_code, Some(1));

    // The retry overwrote the manifest with only the implicated file at High.
    let manifest = data_dir
        .join("context")
        .join(&run_id.0)
        .join("agent-0")
        .join("manifest.json");
    let body = std::fs::read_to_string(&manifest).unwrap();
    assert!(
        body.contains("src/bad.rs"),
        "retry manifest missing bad.rs: {body}"
    );
    assert!(
        !body.contains("src/good.rs"),
        "retry should not re-scaffold good.rs: {body}"
    );
    assert!(
        body.contains("\"high\""),
        "retry manifest not at high fidelity: {body}"
    );
}
