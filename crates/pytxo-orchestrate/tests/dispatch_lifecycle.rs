use std::fs;
use std::process::Command;
use std::sync::OnceLock;
use std::time::Duration;

use pytxo_core::{PermissionProfile, PytxoConfig};
use pytxo_orchestrate::{dispatch, trust_repo, HypervisorRegistry, RunOptions};
use pytxo_store::PytxoStore;
use tempfile::{tempdir, TempDir};

fn isolate_process_state() {
    static STATE: OnceLock<()> = OnceLock::new();
    STATE.get_or_init(|| {
        let home = TempDir::new().expect("pytxo home tempdir");
        let trust = TempDir::new().expect("trust tempdir");
        let home_path = home.path().to_path_buf();
        let trust_path = trust.path().join("trusted-domains.json");
        std::mem::forget(home);
        std::mem::forget(trust);
        // SAFETY: this integration-test process owns these variables.
        unsafe {
            std::env::set_var("PYTXO_HOME", home_path);
            std::env::set_var("PYTXO_TRUST_STORE", trust_path);
        }
    });
}

fn init_git_repo(path: &std::path::Path) {
    fs::write(path.join("README.md"), "test\n").unwrap();
    for args in [
        vec!["init"],
        vec!["config", "user.email", "pytxo@test.local"],
        vec!["config", "user.name", "Pytxo Test"],
        vec!["add", "."],
        vec!["commit", "-m", "init"],
    ] {
        assert!(Command::new("git")
            .args(args)
            .current_dir(path)
            .status()
            .unwrap()
            .success());
    }
}

fn run_opts(repo: &std::path::Path) -> RunOptions {
    RunOptions {
        agents: 1,
        cmd: "echo pytxo".into(),
        config: None,
        dry_run: false,
        keep_worktrees: false,
        repo: Some(repo.to_path_buf()),
        execution: None,
        project: None,
        tasks: None,
        task_cmd_template: None,
        task_prompts: None,
    }
}

#[tokio::test(flavor = "current_thread")]
async fn dispatch_persists_starting_run_before_returning_id() {
    isolate_process_state();
    let repo = tempdir().unwrap();
    init_git_repo(repo.path());
    trust_repo(repo.path(), PermissionProfile::Orbit).unwrap();

    let (_, run_id) = dispatch(run_opts(repo.path())).expect("dispatch run");
    // The detached supervisor must retain admission after dispatch returns,
    // including before this current-thread runtime first polls the worker.
    assert!(pytxo_core::UpgradeGuard::upgrade().is_err());

    let store = PytxoStore::open(&PytxoConfig::default().db_path_at(repo.path())).unwrap();
    let row = store
        .get_run(&run_id.0)
        .unwrap()
        .expect("dispatch must durably insert its run before returning");
    assert_eq!(row.status, "starting");
    assert!(repo.path().join(".pytxo/data/active_run.json").exists());
}

#[tokio::test(flavor = "current_thread")]
async fn detached_startup_failure_settles_the_visible_row() {
    isolate_process_state();
    let repo = tempdir().unwrap();
    trust_repo(repo.path(), PermissionProfile::Orbit).unwrap();

    let (_, run_id) = dispatch(run_opts(repo.path())).expect("dispatch accepted startup");
    let store_path = PytxoConfig::default().db_path_at(repo.path());
    let deadline = tokio::time::Instant::now() + Duration::from_secs(5);
    loop {
        let status = PytxoStore::open(&store_path)
            .unwrap()
            .get_run_status(&run_id.0)
            .unwrap()
            .map(|(status, _)| status);
        if status.as_deref() == Some("failed_startup") {
            break;
        }
        assert!(
            tokio::time::Instant::now() < deadline,
            "startup failure did not settle; last status={status:?}"
        );
        tokio::time::sleep(Duration::from_millis(10)).await;
    }

    assert!(!repo.path().join(".pytxo/data/active_run.json").exists());
}

#[tokio::test(flavor = "current_thread")]
async fn concurrent_dispatch_cannot_overwrite_active_run_ownership() {
    isolate_process_state();
    let repo = tempdir().unwrap();
    init_git_repo(repo.path());
    trust_repo(repo.path(), PermissionProfile::Orbit).unwrap();
    let hypervisor = HypervisorRegistry::new();

    let (_, first_run) = hypervisor.dispatch(run_opts(repo.path())).unwrap();
    let error = hypervisor
        .dispatch(run_opts(repo.path()))
        .expect_err("one execution domain must have one active owner");
    assert!(error.to_string().contains("already owns active run"));

    let marker: serde_json::Value = serde_json::from_str(
        &fs::read_to_string(repo.path().join(".pytxo/data/active_run.json")).unwrap(),
    )
    .unwrap();
    assert_eq!(marker["run_id"], first_run.0);

    let store = PytxoStore::open(&PytxoConfig::default().db_path_at(repo.path())).unwrap();
    let runs = store.list_runs(10).unwrap();
    assert_eq!(
        runs.iter().filter(|run| run.status == "starting").count(),
        1
    );
    assert_eq!(
        runs.iter()
            .filter(|run| run.status == "failed_startup")
            .count(),
        1
    );
}

#[tokio::test(flavor = "current_thread")]
async fn dispatch_reconciles_a_crashed_supervisor_marker() {
    isolate_process_state();
    let repo = tempdir().unwrap();
    init_git_repo(repo.path());
    trust_repo(repo.path(), PermissionProfile::Orbit).unwrap();
    let cfg = PytxoConfig::default();
    let store = PytxoStore::open(&cfg.db_path_at(repo.path())).unwrap();
    store
        .insert_run("crashed-run", &repo.path().to_string_lossy())
        .unwrap();
    fs::write(
        repo.path().join(".pytxo/data/active_run.json"),
        serde_json::json!({
            "run_id": "crashed-run",
            "repo_root": repo.path().to_string_lossy(),
            "supervisor_pid": u32::MAX,
            "supervisor_start_identity": "not-a-live-process"
        })
        .to_string(),
    )
    .unwrap();

    let (_, replacement) = HypervisorRegistry::new()
        .dispatch(run_opts(repo.path()))
        .expect("stale crash ownership should be reconciled");

    let (old_status, old_finished_at) = store.get_run_status("crashed-run").unwrap().unwrap();
    assert_eq!(old_status, "failed");
    assert!(old_finished_at.is_some());
    let marker: serde_json::Value = serde_json::from_str(
        &fs::read_to_string(repo.path().join(".pytxo/data/active_run.json")).unwrap(),
    )
    .unwrap();
    assert_eq!(marker["run_id"], replacement.0);
}
