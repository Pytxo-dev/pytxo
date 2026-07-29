use std::fs;
use std::process::Command;
use std::time::Duration;

use pytxo_core::{ExecutionBackend, PermissionProfile, PytxoConfig};
use pytxo_orchestrate::{dispatch, stop_exact, trust_repo, RunOptions};
use pytxo_runner::{registry_path, ProcessRegistryFile};
use pytxo_store::PytxoStore;
use tempfile::tempdir;

const ISOLATED_SCENARIO_SENTINEL: &str = "PYTXO_STOP_EXACT_ISOLATED_SCENARIO";

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
            .expect("run git")
            .success());
    }
    fs::write(path.join("README.md"), "test\n").expect("write repo file");
    assert!(Command::new("git")
        .args(["add", "."])
        .current_dir(path)
        .status()
        .expect("stage initial repo")
        .success());
    assert!(Command::new("git")
        .args(["commit", "-m", "init"])
        .current_dir(path)
        .status()
        .expect("commit initial repo")
        .success());
}

fn blocking_command() -> &'static str {
    if cfg!(windows) {
        "powershell -NoProfile -Command \"Start-Sleep -Seconds 2\""
    } else {
        "sleep 2"
    }
}

async fn wait_until(mut condition: impl FnMut() -> bool) -> bool {
    let deadline = tokio::time::Instant::now() + Duration::from_secs(30);
    while !condition() {
        if tokio::time::Instant::now() >= deadline {
            return false;
        }
        tokio::time::sleep(Duration::from_millis(25)).await;
    }
    true
}

fn write_active_run(repo: &std::path::Path, run_id: &str) -> std::path::PathBuf {
    let data_dir = repo.join(PytxoConfig::default().data_dir);
    fs::create_dir_all(&data_dir).expect("create data directory");
    let state_path = data_dir.join("active_run.json");
    fs::write(
        &state_path,
        serde_json::json!({
            "run_id": run_id,
            "repo_root": repo.to_string_lossy(),
        })
        .to_string(),
    )
    .expect("write active run");
    state_path
}

#[tokio::test]
async fn refuses_to_stop_when_the_active_run_changed() {
    let temp = tempdir().expect("tempdir");
    let repo = temp.path();
    let state_path = write_active_run(repo, "run-new");

    let error = stop_exact(None, Some(repo.to_path_buf()), "run-stale", false)
        .await
        .expect_err("stale run must be refused");

    let message = error.to_string();
    assert!(message.contains("refusing to stop run run-stale"));
    assert!(message.contains("active run"));
    assert!(message.contains("run-new"));
    assert!(
        state_path.exists(),
        "refusal must preserve active-run state"
    );
}

#[tokio::test]
async fn stops_only_the_expected_active_run() {
    let temp = tempdir().expect("tempdir");
    let repo = temp.path();
    let config = PytxoConfig::default();
    let store = PytxoStore::open(&config.db_path_at(repo)).expect("open store");
    store
        .insert_run("run-current", &repo.to_string_lossy())
        .expect("insert run");
    let state_path = write_active_run(repo, "run-current");

    stop_exact(None, Some(repo.to_path_buf()), "run-current", false)
        .await
        .expect("matching active run should stop");

    assert!(
        !state_path.exists(),
        "successful stop must clear active-run state"
    );
    let (status, finished_at) = store
        .get_run_status("run-current")
        .expect("read run status")
        .expect("run should exist");
    assert_eq!(status, "cancelled");
    assert!(finished_at.is_some());
}

#[test]
fn dispatched_blocking_run_stays_cancelled_after_exact_stop() {
    if std::env::var_os(ISOLATED_SCENARIO_SENTINEL).is_some() {
        tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .expect("build current-thread runtime")
            .block_on(run_dispatched_blocking_run_scenario());
        return;
    }

    let home = tempdir().expect("isolated Pytxo home tempdir");
    let status = Command::new(std::env::current_exe().expect("current test executable"))
        .arg("dispatched_blocking_run_stays_cancelled_after_exact_stop")
        .arg("--exact")
        .arg("--nocapture")
        .env("PYTXO_HOME", home.path())
        .env(
            "PYTXO_TRUST_STORE",
            home.path().join("trusted-domains.json"),
        )
        .env(ISOLATED_SCENARIO_SENTINEL, "1")
        .status()
        .expect("run isolated dispatched scenario");
    assert!(status.success(), "isolated dispatched scenario failed");
}

async fn run_dispatched_blocking_run_scenario() {
    let temp = tempdir().expect("tempdir");
    let repo = temp.path();
    init_git_repo(repo);
    trust_repo(repo, PermissionProfile::Orbit).expect("trust Orbit test domain");

    let (_, run_id) = dispatch(RunOptions {
        agents: 1,
        cmd: blocking_command().into(),
        config: None,
        dry_run: false,
        keep_worktrees: false,
        repo: Some(repo.to_path_buf()),
        execution: Some(ExecutionBackend::Subprocess),
        project: None,
        tasks: None,
        task_cmd_template: None,
        task_prompts: None,
    })
    .expect("dispatch blocking run");

    let config = PytxoConfig::default();
    let db_path = config.db_path_at(repo);
    let data_dir = repo.join(&config.data_dir);
    let result: Result<(), String> = async {
        let started = wait_until(|| {
            let status_is_running = PytxoStore::open(&db_path)
                .ok()
                .and_then(|store| store.get_run_status(&run_id.0).ok().flatten())
                .is_some_and(|(status, _)| status == "running");
            let child_is_persisted = ProcessRegistryFile::load(&registry_path(&data_dir))
                .ok()
                .is_some_and(|registry| !registry.for_run(&run_id.0).is_empty());
            status_is_running && child_is_persisted
        })
        .await;
        if !started {
            return Err("timed out waiting for the dispatched blocking child".into());
        }

        stop_exact(None, Some(repo.to_path_buf()), &run_id.0, false)
            .await
            .map_err(|error| format!("stop the exact active run: {error}"))?;

        let worker_unwound = wait_until(|| {
            PytxoStore::open(&db_path)
                .ok()
                .and_then(|store| store.list_agents_for_run(&run_id.0).ok())
                .is_some_and(|agents| !agents.is_empty())
        })
        .await;
        if !worker_unwound {
            return Err("timed out waiting for the killed worker to unwind".into());
        }

        let (status, finished_at) = PytxoStore::open(&db_path)
            .map_err(|error| format!("open store after worker unwind: {error}"))?
            .get_run_status(&run_id.0)
            .map_err(|error| format!("read final run: {error}"))?
            .ok_or_else(|| "run disappeared after worker unwind".to_string())?;
        if status != "cancelled" {
            return Err(format!(
                "expected cancelled after worker unwind, got {status}"
            ));
        }
        if finished_at.is_none() {
            return Err("cancelled run is missing finished_at".into());
        }
        Ok(())
    }
    .await;

    let _ = stop_exact(None, Some(repo.to_path_buf()), &run_id.0, false).await;
    result.expect("dispatched blocking run must remain cancelled after exact stop");
}
