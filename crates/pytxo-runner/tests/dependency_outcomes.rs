use std::collections::HashMap;
use std::process::Command;

use pytxo_core::{
    BillingMode, ExecutionBackend, ExecutionPlan, FidelityTier, IsolationMode, PermissionProfile,
    RunId, ScheduledTask, TaskId,
};
use pytxo_runner::{execute_plan, AgentRunOutcome, ProcessRegistry, RunContext, SwarmRegistry};
use tempfile::TempDir;

fn init_git_repo(path: &std::path::Path) {
    for args in [
        vec!["init"],
        vec!["config", "user.email", "pytxo@test.local"],
        vec!["config", "user.name", "Pytxo Test"],
    ] {
        assert!(Command::new("git")
            .args(args)
            .current_dir(path)
            .status()
            .unwrap()
            .success());
    }
    std::fs::write(path.join("README.md"), "dependency outcome test\n").unwrap();
    assert!(Command::new("git")
        .args(["add", "."])
        .current_dir(path)
        .status()
        .unwrap()
        .success());
    assert!(Command::new("git")
        .args(["commit", "-m", "init"])
        .current_dir(path)
        .status()
        .unwrap()
        .success());
}

fn task(id: &str, wave: u32, depends_on: &[&str], verify: Vec<String>) -> ScheduledTask {
    ScheduledTask {
        task_id: TaskId(id.into()),
        agent: "default".into(),
        paths: vec![format!("{id}.txt")],
        depends_on: depends_on.iter().map(|id| (*id).to_string()).collect(),
        wave,
        root: None,
        signal_fidelity: None,
        verify,
    }
}

fn plan(upstream_verify: Vec<String>) -> ExecutionPlan {
    ExecutionPlan {
        waves: vec![
            vec![task("upstream", 0, &[], upstream_verify)],
            vec![
                task("dependent", 1, &["upstream"], Vec::new()),
                task("independent", 1, &[], Vec::new()),
            ],
        ],
        conflicts: Vec::new(),
        max_agents: 2,
        warnings: Vec::new(),
    }
}

fn context(repo: &std::path::Path, command_template: String) -> RunContext {
    let worktree_base = repo.join(".pytxo/worktrees");
    let data_dir = repo.join(".pytxo/data");
    std::fs::create_dir_all(&worktree_base).unwrap();
    std::fs::create_dir_all(&data_dir).unwrap();
    let (domain_id, model_router, managed_transport, token_estimator) =
        RunContext::default_metering(repo);
    RunContext {
        run_id: RunId::new(),
        repo_root: repo.to_path_buf(),
        worktree_base,
        data_dir,
        cmd: String::new(),
        task_cmd_template: Some(command_template),
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
    }
}

fn successful_write_command() -> String {
    if cfg!(windows) {
        "echo {task_id}>{task_id}.txt".into()
    } else {
        "printf '%s' '{task_id}' > {task_id}.txt".into()
    }
}

fn partial_then_fail_command() -> String {
    if cfg!(windows) {
        "if \"{task_id}\"==\"upstream\" (echo partial>upstream.txt & exit /b 7) else (echo {task_id}>{task_id}.txt)".into()
    } else {
        "if [ \"{task_id}\" = \"upstream\" ]; then printf partial > upstream.txt; exit 7; else printf '%s' '{task_id}' > {task_id}.txt; fi".into()
    }
}

fn failing_verify_command() -> String {
    if cfg!(windows) {
        "exit /b 9".into()
    } else {
        "exit 9".into()
    }
}

#[tokio::test(flavor = "current_thread")]
async fn verification_remains_stoppable_on_a_single_thread_runtime() {
    use pytxo_runner::{registry_path, stop_run, ProcessRegistryFile};
    use std::sync::atomic::{AtomicBool, Ordering};
    use std::sync::Arc;
    use std::time::Duration;

    let temp = TempDir::new().unwrap();
    init_git_repo(temp.path());
    let mut ctx = context(temp.path(), successful_write_command());
    let verifying = Arc::new(AtomicBool::new(false));
    let signal = Arc::clone(&verifying);
    ctx.on_event = Some(Arc::new(move |_, kind, _| {
        if kind == "verify" {
            signal.store(true, Ordering::SeqCst);
        }
    }));
    let data_dir = ctx.data_dir.clone();
    let run_id = ctx.run_id.0.clone();
    let verify = if cfg!(windows) {
        "ping -n 31 127.0.0.1 >NUL"
    } else {
        "sleep 30"
    };
    let plan = ExecutionPlan {
        waves: vec![vec![task("upstream", 0, &[], vec![verify.into()])]],
        conflicts: vec![],
        max_agents: 1,
        warnings: vec![],
    };
    let worker = tokio::spawn(async move {
        execute_plan(
            &ctx,
            &plan,
            &ProcessRegistry::default(),
            &SwarmRegistry::new(),
        )
        .await
    });
    let deadline = tokio::time::Instant::now() + Duration::from_secs(10);
    loop {
        if verifying.load(Ordering::SeqCst)
            && !ProcessRegistryFile::load(&registry_path(&data_dir))
                .unwrap()
                .entries
                .is_empty()
        {
            break;
        }
        assert!(
            tokio::time::Instant::now() < deadline,
            "verifier blocked the async runtime or never registered"
        );
        tokio::time::sleep(Duration::from_millis(10)).await;
    }
    stop_run(&data_dir, &run_id, true).expect("stop tracked verifier");
    let results = tokio::time::timeout(Duration::from_secs(5), worker)
        .await
        .expect("cancelled verification settled promptly")
        .unwrap()
        .unwrap();
    assert_eq!(results[0].outcome, AgentRunOutcome::VerificationFailed);
    assert!(results[0].stderr.contains("cancelled by Stop"));
}

fn assert_dependency_is_blocked_and_independent_completes(
    results: &[pytxo_runner::AgentRunResult],
) {
    let upstream = results
        .iter()
        .find(|result| result.task_id == "upstream")
        .unwrap();
    let dependent = results
        .iter()
        .find(|result| result.task_id == "dependent")
        .unwrap();
    let independent = results
        .iter()
        .find(|result| result.task_id == "independent")
        .unwrap();

    assert!(upstream
        .worktree_path
        .as_ref()
        .unwrap()
        .join("upstream.txt")
        .exists());
    assert_eq!(
        dependent.outcome,
        AgentRunOutcome::BlockedByDependency {
            task_ids: vec!["upstream".into()]
        }
    );
    assert!(
        dependent.worktree_path.is_none(),
        "dependent must never start"
    );
    assert_eq!(independent.outcome, AgentRunOutcome::Succeeded);
    assert!(independent
        .worktree_path
        .as_ref()
        .unwrap()
        .join("independent.txt")
        .exists());
}

#[tokio::test]
async fn nonzero_upstream_blocks_dependent_but_not_later_independent_task() {
    let repo = TempDir::new().unwrap();
    init_git_repo(repo.path());
    let ctx = context(repo.path(), partial_then_fail_command());

    let results = execute_plan(
        &ctx,
        &plan(Vec::new()),
        &ProcessRegistry::default(),
        &SwarmRegistry::new(),
    )
    .await
    .unwrap();

    assert_eq!(
        results
            .iter()
            .find(|result| result.task_id == "upstream")
            .unwrap()
            .outcome,
        AgentRunOutcome::ProcessFailed
    );
    assert_dependency_is_blocked_and_independent_completes(&results);
}

#[tokio::test]
async fn verification_failure_blocks_dependent_output_composition() {
    let repo = TempDir::new().unwrap();
    init_git_repo(repo.path());
    let ctx = context(repo.path(), successful_write_command());

    let results = execute_plan(
        &ctx,
        &plan(vec![failing_verify_command()]),
        &ProcessRegistry::default(),
        &SwarmRegistry::new(),
    )
    .await
    .unwrap();

    assert_eq!(
        results
            .iter()
            .find(|result| result.task_id == "upstream")
            .unwrap()
            .outcome,
        AgentRunOutcome::VerificationFailed
    );
    assert_dependency_is_blocked_and_independent_completes(&results);
}

#[tokio::test]
async fn one_agent_startup_error_is_recorded_without_aborting_its_wave_sibling() {
    let repo = TempDir::new().unwrap();
    init_git_repo(repo.path());
    let ctx = context(repo.path(), successful_write_command());
    let mut invalid = task("invalid-root", 0, &[], Vec::new());
    invalid.root = Some("missing".into());
    let valid = task("independent", 0, &[], Vec::new());
    let plan = ExecutionPlan {
        waves: vec![vec![invalid, valid]],
        conflicts: Vec::new(),
        max_agents: 2,
        warnings: Vec::new(),
    };

    let results = execute_plan(
        &ctx,
        &plan,
        &ProcessRegistry::default(),
        &SwarmRegistry::new(),
    )
    .await
    .unwrap();

    let failed = results
        .iter()
        .find(|result| result.task_id == "invalid-root")
        .unwrap();
    assert_eq!(failed.outcome, AgentRunOutcome::ProcessFailed);
    assert!(failed.stderr.contains("unknown root label"));
    assert!(failed.worktree_path.is_none());

    let independent = results
        .iter()
        .find(|result| result.task_id == "independent")
        .unwrap();
    assert_eq!(independent.outcome, AgentRunOutcome::Succeeded);
    assert!(independent
        .worktree_path
        .as_ref()
        .unwrap()
        .join("independent.txt")
        .exists());
}
