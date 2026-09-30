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

#[tokio::test]
async fn stop_during_execution_is_cancelled_for_both_local_backends() {
    use pytxo_runner::{registry_path, stop_run, ProcessRegistryFile};
    use std::sync::{
        atomic::{AtomicBool, Ordering},
        Arc,
    };
    use std::time::Duration;

    for backend in [ExecutionBackend::Subprocess, ExecutionBackend::Pty] {
        let repo = TempDir::new().unwrap();
        init_git_repo(repo.path());
        let command = if cfg!(windows) {
            "echo execution-ready & ping -n 31 127.0.0.1 >NUL"
        } else {
            "echo execution-ready; sleep 30"
        };
        let mut ctx = context(repo.path(), command.into());
        ctx.execution_backend = backend;
        ctx.signal_core = true;
        ctx.agent_paths
            .insert("default".into(), vec!["README.md".into()]);
        let ready = Arc::new(AtomicBool::new(false));
        let observed = ready.clone();
        let continued = Arc::new(AtomicBool::new(false));
        let unexpected = continued.clone();
        ctx.on_event = Some(Arc::new(move |_, kind, text| {
            if kind == "verify" || kind == "signal-retry" {
                unexpected.store(true, Ordering::SeqCst);
            }
            if kind == "stdout" && text.contains("execution-ready") {
                observed.store(true, Ordering::SeqCst);
            }
        }));
        let data_dir = ctx.data_dir.clone();
        let run_id = ctx.run_id.0.clone();
        let plan = ExecutionPlan {
            waves: vec![vec![task(
                "worker",
                0,
                &[],
                vec!["echo must-not-verify".into()],
            )]],
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
        while !ready.load(Ordering::SeqCst)
            || ProcessRegistryFile::load(&registry_path(&data_dir))
                .unwrap()
                .entries
                .is_empty()
        {
            assert!(
                tokio::time::Instant::now() < deadline,
                "worker did not become ready: {backend:?}"
            );
            tokio::time::sleep(Duration::from_millis(10)).await;
        }
        stop_run(&data_dir, &run_id, true).unwrap();
        let results = tokio::time::timeout(Duration::from_secs(10), worker)
            .await
            .unwrap()
            .unwrap()
            .unwrap();
        assert_eq!(
            results[0].outcome,
            AgentRunOutcome::Cancelled,
            "{backend:?}"
        );
        assert!(ProcessRegistryFile::load(&registry_path(&data_dir))
            .unwrap()
            .entries
            .is_empty());
        assert!(
            !continued.load(Ordering::SeqCst),
            "stopped worker entered retry or verification"
        );
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
    let mut ctx = context(temp.path(), "echo retained-worker-output".into());
    ctx.keep_worktrees = false;
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
    assert_eq!(results[0].outcome, AgentRunOutcome::Cancelled);
    assert_eq!(results[0].outcome.ledger_status(), "cancelled");
    assert!(!results[0].outcome.is_success());
    assert_eq!(
        results[0].exit_code,
        Some(0),
        "Preserve the completed worker exit; Stop is not a fabricated check failure"
    );
    assert!(results[0].stderr.contains("cancelled by Stop"));
    assert!(results[0].stdout.contains("retained-worker-output"));
    assert!(results[0].worktree_path.as_ref().unwrap().is_dir());
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
async fn durable_stop_at_agent_start_is_cancelled_not_process_failed() {
    use pytxo_runner::stop_run;
    use std::sync::{Arc, Mutex};

    let repo = TempDir::new().unwrap();
    init_git_repo(repo.path());
    let mut ctx = context(repo.path(), successful_write_command());
    let data_dir = ctx.data_dir.clone();
    let run_id = ctx.run_id.0.clone();
    let events = Arc::new(Mutex::new(Vec::new()));
    let recorded = events.clone();
    ctx.on_event = Some(Arc::new(move |_, kind, _| {
        recorded.lock().unwrap().push(kind.to_string());
        if kind == "agent-start" {
            stop_run(&data_dir, &run_id, true).unwrap();
        }
    }));
    let plan = ExecutionPlan {
        waves: vec![vec![task("stopped", 0, &[], vec![])]],
        conflicts: vec![],
        max_agents: 1,
        warnings: vec![],
    };
    let results = execute_plan(
        &ctx,
        &plan,
        &ProcessRegistry::default(),
        &SwarmRegistry::new(),
    )
    .await
    .unwrap();
    assert_eq!(results[0].outcome, AgentRunOutcome::Cancelled);
    assert_eq!(results[0].outcome.ledger_status(), "cancelled");
    let events = events.lock().unwrap();
    assert!(events.iter().any(|kind| kind == "agent-cancelled"));
    assert!(!events.iter().any(|kind| kind == "agent-lifecycle-failed"));
    assert_eq!(
        std::fs::read_to_string(repo.path().join("README.md")).unwrap(),
        "dependency outcome test\n"
    );
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

#[tokio::test]
async fn context_snapshot_matches_successful_dependency_output() {
    for absolute_extra_path in [false, true] {
        let repo = TempDir::new().unwrap();
        init_git_repo(repo.path());
        let command = if cfg!(windows) {
            "if {task_id}==upstream (echo updated>README.md) else (echo dependent>dependent.txt)"
        } else {
            "if [ {task_id} = upstream ]; then printf 'updated\n' > README.md; else printf dependent > dependent.txt; fi"
        };
        let mut ctx = context(repo.path(), command.into());
        ctx.signal_core = true;
        ctx.signal_fidelity = FidelityTier::High;
        ctx.agent_paths.insert(
            "default".into(),
            vec![if absolute_extra_path {
                repo.path().join("README.md").to_string_lossy().into_owned()
            } else {
                "README.md".into()
            }],
        );
        let mut upstream = task("upstream", 0, &[], vec![]);
        upstream.paths = vec!["README.md".into()];
        let plan = ExecutionPlan {
            waves: vec![
                vec![upstream],
                vec![task("dependent", 1, &["upstream"], vec![])],
            ],
            conflicts: vec![],
            max_agents: 1,
            warnings: vec![],
        };
        let results = execute_plan(
            &ctx,
            &plan,
            &ProcessRegistry::default(),
            &SwarmRegistry::new(),
        )
        .await
        .unwrap();
        let dependent = results.iter().find(|r| r.task_id == "dependent").unwrap();
        assert_eq!(
            dependent.outcome,
            AgentRunOutcome::Succeeded,
            "dependency fixture failed (absolute={absolute_extra_path}): {} {}",
            dependent.stdout,
            dependent.stderr
        );
        let scaffold = ctx
            .data_dir
            .join("context")
            .join(&ctx.run_id.0)
            .join(&dependent.agent_id.0)
            .join("README.md");
        assert_eq!(
            std::fs::read_to_string(scaffold).unwrap(),
            std::fs::read_to_string(dependent.worktree_path.as_ref().unwrap().join("README.md"))
                .unwrap(),
            "context differed from dependency-composed workspace (absolute={absolute_extra_path})"
        );
        assert_eq!(
            std::fs::read_to_string(repo.path().join("README.md")).unwrap(),
            "dependency outcome test\n"
        );
    }
}

#[tokio::test]
async fn context_snapshot_retry_uses_the_first_attempts_edits() {
    let repo = TempDir::new().unwrap();
    init_git_repo(repo.path());
    let command = if cfg!(windows) {
        "if exist attempt.txt (echo %PYTXO_CONTEXT_DIR% & exit /b 0) else (echo updated>README.md & echo attempted>attempt.txt & echo error at README.md:1:1 & exit /b 7)"
    } else {
        "if [ -f attempt.txt ]; then printf '%s\n' \"$PYTXO_CONTEXT_DIR\"; exit 0; else printf 'updated\n' > README.md; touch attempt.txt; echo 'error at README.md:1:1'; exit 7; fi"
    };
    let mut ctx = context(repo.path(), command.into());
    ctx.signal_core = true;
    let mut worker = task("retry", 0, &[], vec![]);
    worker.paths = vec!["README.md".into()];
    let plan = ExecutionPlan {
        waves: vec![vec![worker]],
        conflicts: vec![],
        max_agents: 1,
        warnings: vec![],
    };
    let results = execute_plan(
        &ctx,
        &plan,
        &ProcessRegistry::default(),
        &SwarmRegistry::new(),
    )
    .await
    .unwrap();
    assert_eq!(
        results[0].outcome,
        AgentRunOutcome::Succeeded,
        "retry context differed from the first attempt's edits: {} {}",
        results[0].stdout,
        results[0].stderr
    );
    let scaffold = std::path::Path::new(results[0].stdout.trim()).join("README.md");
    assert_eq!(
        std::fs::read_to_string(scaffold).unwrap(),
        std::fs::read_to_string(results[0].worktree_path.as_ref().unwrap().join("README.md"))
            .unwrap(),
        "retry context differed from the first attempt's edits"
    );
    assert_eq!(
        std::fs::read_to_string(repo.path().join("README.md")).unwrap(),
        "dependency outcome test\n"
    );
}

#[tokio::test]
async fn context_snapshot_still_denies_absolute_paths_outside_the_repository() {
    let tmp = TempDir::new().unwrap();
    let repo = tmp.path().join("repo");
    std::fs::create_dir_all(&repo).unwrap();
    init_git_repo(&repo);
    let outside = tmp.path().join("outside.md");
    std::fs::write(&outside, "outside source\n").unwrap();
    let mut ctx = context(&repo, successful_write_command());
    ctx.signal_core = true;
    ctx.agent_paths.insert(
        "default".into(),
        vec![outside.to_string_lossy().into_owned()],
    );
    let plan = ExecutionPlan {
        waves: vec![vec![task("denied", 0, &[], vec![])]],
        conflicts: vec![],
        max_agents: 1,
        warnings: vec![],
    };
    let results = execute_plan(
        &ctx,
        &plan,
        &ProcessRegistry::default(),
        &SwarmRegistry::new(),
    )
    .await
    .unwrap();
    assert_eq!(results[0].outcome, AgentRunOutcome::ProcessFailed);
    assert!(results[0].stderr.contains("read denied"));
    assert!(!results[0]
        .worktree_path
        .as_ref()
        .is_some_and(|workspace| workspace.join("denied.txt").exists()));
    assert_eq!(
        std::fs::read_to_string(outside).unwrap(),
        "outside source\n"
    );
}

#[tokio::test]
async fn context_snapshot_retry_does_not_expose_deleted_source() {
    let repo = TempDir::new().unwrap();
    init_git_repo(repo.path());
    let command = if cfg!(windows) {
        "if exist attempt.txt (echo %PYTXO_CONTEXT_DIR% & exit /b 0) else (del README.md & echo attempted>attempt.txt & echo error at README.md:1:1 & exit /b 7)"
    } else {
        "if [ -f attempt.txt ]; then printf '%s\n' \"$PYTXO_CONTEXT_DIR\"; exit 0; else rm README.md; touch attempt.txt; echo 'error at README.md:1:1'; exit 7; fi"
    };
    let mut ctx = context(repo.path(), command.into());
    ctx.signal_core = true;
    let mut worker = task("retry", 0, &[], vec![]);
    worker.paths = vec!["README.md".into()];
    let plan = ExecutionPlan {
        waves: vec![vec![worker]],
        conflicts: vec![],
        max_agents: 1,
        warnings: vec![],
    };
    let results = execute_plan(
        &ctx,
        &plan,
        &ProcessRegistry::default(),
        &SwarmRegistry::new(),
    )
    .await
    .unwrap();
    assert_eq!(results[0].outcome, AgentRunOutcome::Succeeded);
    let retry_context = std::path::Path::new(results[0].stdout.trim());
    assert!(retry_context.join("manifest.json").is_file());
    assert!(
        !retry_context.join("README.md").exists(),
        "retry exposed context for source deleted by its first attempt"
    );
    assert_eq!(
        std::fs::read_to_string(repo.path().join("README.md")).unwrap(),
        "dependency outcome test\n"
    );
}
