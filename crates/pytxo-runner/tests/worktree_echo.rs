use pytxo_core::{
    BillingMode, ExecutionPlan, FidelityTier, IsolationMode, PermissionProfile, RunId,
    ScheduledTask, TaskId,
};
use pytxo_runner::{
    execute_plan, prepare_run_change_set, AgentWorkspaceInput, ProcessRegistry, RunContext,
    SwarmRegistry,
};
use std::collections::HashMap;
use std::process::Command;
use std::sync::{Arc, Mutex};
use tempfile::TempDir;

fn git(path: &std::path::Path, args: &[&str]) {
    assert!(Command::new("git")
        .args(args)
        .current_dir(path)
        .status()
        .unwrap()
        .success());
}

fn init_git_repo(path: &std::path::Path) {
    git(path, &["init"]);
    git(path, &["config", "user.email", "pytxo@test.local"]);
    git(path, &["config", "user.name", "Pytxo Test"]);
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

fn plan() -> ExecutionPlan {
    ExecutionPlan {
        waves: vec![vec![ScheduledTask {
            task_id: TaskId("t0".into()),
            agent: "default".into(),
            paths: vec!["README.md".into()],
            depends_on: vec![],
            wave: 0,
            root: None,
            signal_fidelity: None,
            verify: vec![],
        }]],
        conflicts: vec![],
        max_agents: 3,
        warnings: vec![],
    }
}

fn context(repo: &std::path::Path) -> RunContext {
    let worktree_base = repo.join(".pytxo/worktrees");
    std::fs::create_dir_all(&worktree_base).unwrap();
    let data_dir = repo.join(".pytxo/data");
    std::fs::create_dir_all(&data_dir).unwrap();
    let (domain_id, model_router, managed_transport, token_estimator) =
        RunContext::default_metering(repo);
    RunContext {
        run_id: RunId::new(),
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
    }
}

#[tokio::test]
async fn single_agent_echo_in_worktree() {
    let tmp = TempDir::new().unwrap();
    let repo = tmp.path();
    init_git_repo(repo);
    let ctx = context(repo);

    let registry = ProcessRegistry::default();
    let swarm = SwarmRegistry::new();
    let results = execute_plan(&ctx, &plan(), &registry, &swarm)
        .await
        .unwrap();
    assert_eq!(results.len(), 1);
    assert_eq!(results[0].exit_code, Some(0));
    assert!(results[0].stdout.contains("pytxo"));
}

/// When no copy-layer can be built, the git worktree fallback must still hold the
/// primary checkout's bytes, which review compares byte for byte: here LF blobs
/// that core.autocrlf would check out as CRLF, an uncommitted edit, an
/// uncommitted delete and an untracked file.
#[tokio::test]
async fn overlay_copy_failure_falls_back_to_a_reviewable_worktree() {
    let tmp = TempDir::new().unwrap();
    let repo = tmp.path();
    init_git_repo(repo);
    std::fs::write(repo.join("kept.txt"), "kept\n").unwrap();
    std::fs::write(repo.join("gone.txt"), "gone\n").unwrap();
    git(repo, &["add", "."]);
    git(repo, &["commit", "-qm", "more"]);
    git(repo, &["config", "core.autocrlf", "true"]);
    std::fs::write(repo.join("kept.txt"), "edited\n").unwrap();
    std::fs::remove_file(repo.join("gone.txt")).unwrap();
    std::fs::write(repo.join("notes.txt"), "untracked\n").unwrap();

    let mut ctx = context(repo);
    ctx.isolation_mode = IsolationMode::Overlay;
    ctx.keep_worktrees = true;
    let events = Arc::new(Mutex::new(Vec::new()));
    let sink = events.clone();
    ctx.on_event = Some(Arc::new(move |_, kind, text| {
        sink.lock()
            .unwrap()
            .push((kind.to_string(), text.to_string()));
    }));
    // A file where each copy-layer upper must go makes every copy-layer fail.
    for layer in ["overlay", "projfs"] {
        let blocker = format!("{layer}-{}-agent-0", ctx.run_id.0);
        std::fs::write(ctx.worktree_base.join(blocker), "").unwrap();
    }

    let registry = ProcessRegistry::default();
    let swarm = SwarmRegistry::new();
    let results = execute_plan(&ctx, &plan(), &registry, &swarm)
        .await
        .unwrap();
    assert_eq!(results[0].exit_code, Some(0));
    let workspace = results[0].worktree_path.clone().unwrap();
    assert!(workspace.join(".git").is_file(), "expected a git worktree");

    let events = events.lock().unwrap();
    let reason = events
        .iter()
        .find(|(kind, _)| kind == "isolation-fallback")
        .map(|(_, text)| text.as_str());
    assert!(
        reason.is_some_and(|text| text.contains("copy-layer failed") && text.contains("mkdir")),
        "{events:?}"
    );

    let change_set = prepare_run_change_set(
        repo,
        &[AgentWorkspaceInput {
            agent_id: "agent-0".into(),
            task_id: "t0".into(),
            workspace_path: workspace,
            claims: vec![],
            depends_on: vec![],
        }],
        &[],
    )
    .unwrap();
    assert!(change_set.changes.is_empty(), "{:?}", change_set.changes);
}
