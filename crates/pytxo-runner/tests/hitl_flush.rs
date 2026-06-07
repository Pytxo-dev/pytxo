//! Galaxy HITL enforcement on Blast flush ([[permission-profile-engine]]).
//! A flush under a profile requiring approval blocks on the HITL queue until a
//! human resolves it; a denial aborts the flush before any repo mutation.

use std::collections::HashMap;
use std::process::Command;
use std::time::Duration;

use pytxo_core::{
    BillingMode, FidelityTier, IsolationMode, PermissionProfile, RunId, WorkspaceHandle,
};
use pytxo_runner::{commit_workspace, HitlQueue, RunContext};
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

fn ctx_with_hitl(repo: &std::path::Path, hitl: HitlQueue) -> RunContext {
    let (domain_id, model_router, managed_transport, token_estimator) =
        RunContext::default_metering(repo);
    RunContext {
        run_id: RunId::new(),
        repo_root: repo.to_path_buf(),
        worktree_base: repo.join(".pytxo/worktrees"),
        data_dir: repo.join(".pytxo/data"),
        cmd: String::new(),
        task_cmd_template: None,
        task_prompts: HashMap::new(),
        keep_worktrees: true,
        on_event: None,
        signal_core: false,
        signal_fidelity: FidelityTier::Low,
        isolation_mode: IsolationMode::Worktree,
        permission_profile: PermissionProfile::Galaxy,
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
        hitl: Some(hitl),
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
    }
}

#[test]
fn galaxy_flush_blocks_until_denied() {
    let tmp = TempDir::new().unwrap();
    let repo = tmp.path();
    init_git_repo(repo);

    let hitl = HitlQueue::new();
    let ctx = ctx_with_hitl(repo, hitl.clone());
    let handle = WorkspaceHandle {
        cwd: repo.to_path_buf(),
        branch: "pytxo/run-x/agent-0".into(),
        backend: IsolationMode::Worktree,
    };

    // Reviewer denies once a request appears.
    let reviewer = std::thread::spawn(move || loop {
        if let Some(req) = hitl.pending().into_iter().next() {
            assert!(hitl.resolve(&req.id, false));
            break;
        }
        std::thread::sleep(Duration::from_millis(10));
    });

    let err = commit_workspace(&ctx, &handle, PermissionProfile::Galaxy)
        .expect_err("denied flush must error");
    reviewer.join().unwrap();
    assert!(
        err.to_string().contains("denied by human reviewer"),
        "unexpected error: {err}"
    );
}
