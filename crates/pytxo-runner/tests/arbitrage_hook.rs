use std::collections::HashMap;
use std::sync::Arc;

use pytxo_core::{
    BillingMode, ExecutionPlan, FidelityTier, IsolationMode, PermissionProfile, RunId,
    ScheduledTask, TaskId,
};
use pytxo_runner::{execute_plan, ProcessRegistry, RunContext, SwarmRegistry};
use pytxo_store::SharedStore;
use std::process::Command;
use tempfile::TempDir;

fn init_git_repo(path: &std::path::Path) {
    Command::new("git")
        .args(["init"])
        .current_dir(path)
        .status()
        .unwrap();
    Command::new("git")
        .args(["config", "user.email", "pytxo@test.local"])
        .current_dir(path)
        .status()
        .unwrap();
    Command::new("git")
        .args(["config", "user.name", "Pytxo Test"])
        .current_dir(path)
        .status()
        .unwrap();
    std::fs::write(
        path.join("sample.rs"),
        "pub fn hello() { let x = 1; println!(\"{x}\"); }\n",
    )
    .unwrap();
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
async fn arbitrage_rows_written_when_signal_core_enabled() {
    let tmp = TempDir::new().unwrap();
    let repo = tmp.path();
    init_git_repo(repo);

    let run_id = RunId::new();
    let data_dir = repo.join(".pytxo/data");
    std::fs::create_dir_all(&data_dir).unwrap();
    let db = data_dir.join("pytxo.db");
    let meter = Arc::new(SharedStore::open(&db).unwrap());

    let (domain_id, model_router, managed_transport, token_estimator) =
        RunContext::default_metering(repo);

    let ctx = RunContext {
        run_id: run_id.clone(),
        repo_root: repo.to_path_buf(),
        worktree_base: repo.join(".pytxo/worktrees"),
        data_dir: data_dir.clone(),
        cmd: "echo ok".into(),
        task_cmd_template: None,
        task_prompts: HashMap::new(),
        keep_worktrees: true,
        on_event: None,
        signal_core: true,
        signal_fidelity: FidelityTier::Low,
        isolation_mode: IsolationMode::Worktree,
        permission_profile: PermissionProfile::Orbit,
        agent_profiles: HashMap::new(),
        route_agents: Vec::new(),
        billing_mode: BillingMode::Byok,
        domain_id: domain_id.clone(),
        model_router,
        managed_transport,
        usage_meter: Some(meter.clone()),
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

    let plan = ExecutionPlan {
        waves: vec![vec![ScheduledTask {
            task_id: TaskId("t0".into()),
            agent: "default".into(),
            paths: vec!["sample.rs".into()],
            wave: 0,
            root: None,
            signal_fidelity: None,
        verify: vec![],        }]],
        conflicts: vec![],
        max_agents: 1,
        warnings: vec![],
    };

    let _ = execute_plan(
        &ctx,
        &plan,
        &ProcessRegistry::default(),
        &SwarmRegistry::new(),
    )
    .await
    .unwrap();

    let saved = meter
        .lock()
        .unwrap()
        .arbitrage_saved_tokens_for_run(&run_id.0)
        .unwrap();
    assert!(saved > 0);
}
