use pytxo_core::{
    AgentId, BillingMode, ExecutionBackend, ExecutionPlan, FidelityTier, IsolationMode,
    PermissionProfile, RunId, ScheduledTask, TaskId,
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

#[cfg(not(windows))]
fn stdin_echo_cmd() -> &'static str {
    r#"IFS= read -r line && printf 'got:%s' "$line""#
}

#[tokio::test]
async fn subprocess_stdin_spawn_time_drain() {
    let tmp = TempDir::new().unwrap();
    let repo = tmp.path();
    init_git_repo(repo);

    let run_id = RunId::new();
    let agent_id = AgentId::new(0);
    let agent_key = format!("{run_id}:{agent_id}");
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
            verify: vec![],
        }]],
        conflicts: vec![],
        max_agents: 1,
        warnings: vec![],
    };

    let data_dir = repo.join(".pytxo/data");
    std::fs::create_dir_all(&data_dir).unwrap();
    let (domain_id, model_router, managed_transport, token_estimator) =
        RunContext::default_metering(repo);

    #[cfg(windows)]
    let cmd = "echo pytxo-subprocess-stdin-smoke".to_string();
    #[cfg(not(windows))]
    let cmd = stdin_echo_cmd().to_string();

    let ctx = RunContext {
        run_id: run_id.clone(),
        repo_root: repo.to_path_buf(),
        worktree_base,
        data_dir,
        cmd,
        task_cmd_template: None,
        task_prompts: HashMap::new(),
        keep_worktrees: true,
        on_event: None,
        signal_core: false,
        signal_fidelity: FidelityTier::Low,
        isolation_mode: IsolationMode::Worktree,
        permission_profile: PermissionProfile::Supernova,
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
        subprocess_stdin: true,
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
    swarm
        .enqueue_stdin(&agent_key, b"hello\n")
        .expect("enqueue before spawn");

    let results = execute_plan(&ctx, &plan, &registry, &swarm).await.unwrap();
    assert_eq!(results.len(), 1);
    assert_eq!(results[0].exit_code, Some(0));

    #[cfg(windows)]
    assert!(
        results[0].stdout.contains("pytxo-subprocess-stdin-smoke"),
        "stdout was {:?}",
        results[0].stdout
    );

    #[cfg(not(windows))]
    assert!(
        results[0].stdout.contains("got:hello"),
        "stdout was {:?}",
        results[0].stdout
    );

    let marker = repo.join("PROMPT_INJECTION_MARKER");
    let prompt = format!(
        "reviewed prompt && echo injected > {} ; $(echo unsafe)",
        marker.display()
    );
    let mut secure_ctx = ctx.clone();
    secure_ctx.run_id = RunId::new();
    secure_ctx.subprocess_stdin = false;
    secure_ctx.task_prompts.insert("t0".into(), prompt.clone());
    #[cfg(windows)]
    {
        secure_ctx.task_cmd_template = Some(
            r#"powershell -NoProfile -NonInteractive -Command "& { [Console]::Write($env:PYTXO_TASK_PROMPT) }""#
                .into(),
        );
    }
    #[cfg(not(windows))]
    {
        secure_ctx.task_cmd_template = Some(r#"printf '%s' "$PYTXO_TASK_PROMPT""#.into());
    }
    let secure_results = execute_plan(
        &secure_ctx,
        &plan,
        &ProcessRegistry::default(),
        &SwarmRegistry::new(),
    )
    .await
    .unwrap();
    assert!(secure_results[0].stdout.contains("reviewed prompt"));
    assert!(
        !marker.exists(),
        "prompt text escaped into a second command"
    );
}
