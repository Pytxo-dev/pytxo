use pytxo_orchestrate::{
    apply_run_changes, commit_workspace_for_agent, discard_run_review, reconcile_run_recovery,
    refresh_run_review, RunOptions,
};

#[test]
fn upgrade_admission_child() {
    if std::env::var_os("PYTXO_TEST_UPGRADE_ADMISSION").is_none() {
        return;
    }
    let repo = std::path::PathBuf::from(std::env::var_os("PYTXO_HOME").unwrap());
    let guard = pytxo_core::UpgradeGuard::upgrade().unwrap();
    let catalog = pytxo_store::Catalog::open_default().unwrap();
    let refused = |result: anyhow::Result<()>| {
        let message = result.unwrap_err().to_string();
        assert!(message.contains("An update is in progress"), "{message}");
    };
    refused(pytxo_orchestrate::dispatch_flow(&catalog, "draft").map(|_| ()));
    refused(apply_run_changes(None, Some(repo.clone()), "run", "digest").map(|_| ()));
    refused(commit_workspace_for_agent(
        None,
        Some(repo.clone()),
        "run",
        "agent",
    ));
    refused(refresh_run_review(None, Some(repo.clone()), "run").map(|_| ()));
    refused(discard_run_review(None, Some(repo.clone()), "run"));
    refused(reconcile_run_recovery(None, Some(repo.clone()), "run").map(|_| ()));
    let opts = || RunOptions {
        agents: 1,
        cmd: "echo guarded".into(),
        config: None,
        dry_run: false,
        keep_worktrees: false,
        repo: Some(repo.clone()),
        execution: None,
        project: None,
        tasks: None,
        task_cmd_template: None,
        task_prompts: None,
    };
    let runtime = tokio::runtime::Runtime::new().unwrap();
    refused(runtime.block_on(pytxo_orchestrate::run(opts())).map(|_| ()));
    let _entered = runtime.enter();
    refused(pytxo_orchestrate::dispatch(opts()).map(|_| ()));
    assert!(!repo.join(".pytxo/data/pytxo.db").exists());
    drop(guard);
    assert!(pytxo_core::UpgradeGuard::work().is_ok());
}

#[test]
fn upgrade_admission_refuses_run_and_repository_mutation_before_side_effects() {
    let dir = tempfile::tempdir().unwrap();
    let status = pytxo_core::background_command(std::env::current_exe().unwrap())
        .args(["--exact", "upgrade_admission_child", "--nocapture"])
        .env("PYTXO_HOME", dir.path())
        .env("PYTXO_TEST_UPGRADE_ADMISSION", "1")
        .status()
        .unwrap();
    assert!(status.success());
}
