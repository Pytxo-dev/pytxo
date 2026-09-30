use pytxo_core::{ExecutionBackend, PermissionProfile, Task, TaskId};
use pytxo_orchestrate::{
    apply_run_changes as apply_reviewed_run_changes, refresh_run_review, run, trust_repo,
    RunOptions,
};
use pytxo_runner::load_review_package;
use pytxo_store::PytxoStore;
use std::{fs, path::Path, process::Command};

fn apply_run_changes(
    config: Option<std::path::PathBuf>,
    repo: Option<std::path::PathBuf>,
    run_id: &str,
) -> anyhow::Result<pytxo_runner::RunApplyManifest> {
    let store = PytxoStore::open(&repo.as_ref().unwrap().join(".pytxo/data/pytxo.db"))?;
    let digest = store
        .get_run_contract(run_id)?
        .and_then(|contract| contract.prepared_digest)
        .unwrap_or_default();
    apply_reviewed_run_changes(config, repo, run_id, &digest)
}

fn fixture(repo: &Path) -> String {
    fs::create_dir_all(repo).unwrap();
    fs::write(repo.join("a.txt"), "0\n").unwrap();
    fs::write(repo.join("b.txt"), "0\n").unwrap();
    fs::write(repo.join(".gitignore"), ".pytxo/\n").unwrap();
    let verify = if cfg!(windows) {
        fs::write(repo.join("verify.ps1"), "if ((Get-Content -Raw a.txt).Trim() -eq '1' -and (Get-Content -Raw b.txt).Trim() -eq '1') { exit 1 }\n").unwrap();
        "powershell -NoProfile -NonInteractive -File verify.ps1"
    } else {
        fs::write(repo.join("verify.sh"), "! { [ \"$(cat a.txt | tr -d ' \\r\\n')\" = 1 ] && [ \"$(cat b.txt | tr -d ' \\r\\n')\" = 1 ]; }\n").unwrap();
        "sh verify.sh"
    };
    for args in [
        vec!["init", "-q"],
        vec!["add", "."],
        vec![
            "-c",
            "user.name=Test",
            "-c",
            "user.email=test@pytxo.local",
            "-c",
            "commit.gpgsign=false",
            "commit",
            "-qm",
            "baseline",
        ],
    ] {
        assert!(Command::new("git")
            .args(args)
            .current_dir(repo)
            .status()
            .unwrap()
            .success());
    }
    trust_repo(repo, PermissionProfile::Orbit).unwrap();
    verify.into()
}

fn options(repo: &Path, names: &[&str], verify: &str) -> RunOptions {
    RunOptions {
        agents: names.len(),
        cmd: "echo fixture".into(),
        config: None,
        dry_run: false,
        keep_worktrees: true,
        repo: Some(repo.into()),
        execution: Some(ExecutionBackend::Subprocess),
        project: None,
        tasks: Some(
            names
                .iter()
                .map(|name| Task {
                    id: TaskId((*name).into()),
                    agent: (*name).into(),
                    paths: vec![format!("{name}.txt")],
                    depends_on: vec![],
                    root: None,
                    signal_fidelity: None,
                    verify: vec![verify.into()],
                })
                .collect(),
        ),
        task_cmd_template: Some("echo 1 > {task_id}.txt".into()),
        task_prompts: None,
    }
}

#[tokio::test]
async fn real_candidate_checks_gate_apply_and_refresh_frozen_bytes() {
    let temp = tempfile::tempdir().unwrap();
    std::env::set_var("PYTXO_HOME", temp.path().join("home"));
    std::env::set_var("PYTXO_TRUST_STORE", temp.path().join("trust.json"));
    let event_repo = temp.path().join("event-write-fails");
    let event_verify = fixture(&event_repo);
    let event_db = event_repo.join(".pytxo/data/pytxo.db");
    drop(PytxoStore::open(&event_db).unwrap());
    let connection = rusqlite::Connection::open(&event_db).unwrap();
    connection
        .execute_batch(
            "CREATE TRIGGER reject_boundary_event BEFORE INSERT ON events
         WHEN NEW.kind = 'verify-boundary'
         BEGIN SELECT RAISE(ABORT, 'injected event write failure'); END;",
        )
        .unwrap();
    let error = run(options(&event_repo, &["a"], &event_verify))
        .await
        .unwrap_err();
    assert!(error.to_string().contains("review preparation failed"));
    let event_store = PytxoStore::open(&event_db).unwrap();
    let event_run = event_store.list_runs(1).unwrap().remove(0);
    assert_eq!(event_run.status, "failed");
    let contract = event_store
        .get_run_contract(&event_run.id)
        .unwrap()
        .unwrap();
    assert_eq!(contract.apply_status, "review_failed");
    assert_eq!(
        contract.last_apply_error.unwrap().code,
        "event_persistence_failed"
    );
    let actor = event_store
        .list_agents_for_run(&event_run.id)
        .unwrap()
        .remove(0);
    assert!(event_store
        .list_events(&actor.id, 100)
        .unwrap()
        .iter()
        .any(|event| event.kind == "evidence-gap"));
    assert!(apply_run_changes(None, Some(event_repo.clone()), &event_run.id).is_err());
    assert!(refresh_run_review(None, Some(event_repo.clone()), &event_run.id).is_err());
    assert_eq!(
        event_store
            .get_run_contract(&event_run.id)
            .unwrap()
            .unwrap()
            .last_apply_error
            .unwrap()
            .code,
        "event_persistence_failed"
    );
    assert_eq!(
        fs::read_to_string(event_repo.join("a.txt")).unwrap().trim(),
        "0"
    );

    let bad_repo = temp.path().join("combined-fails");
    let verify = fixture(&bad_repo);
    let error = run(options(&bad_repo, &["a", "b"], &verify))
        .await
        .unwrap_err();
    assert!(error.to_string().contains("review preparation failed"));
    let store = PytxoStore::open(&bad_repo.join(".pytxo/data/pytxo.db")).unwrap();
    let bad_run = store.list_runs(1).unwrap().remove(0);
    assert!(store
        .list_agents_for_run(&bad_run.id)
        .unwrap()
        .iter()
        .all(|agent| agent.status == "completed"));
    assert_eq!(
        store
            .get_run_contract(&bad_run.id)
            .unwrap()
            .unwrap()
            .apply_status,
        "review_failed"
    );
    assert!(apply_run_changes(None, Some(bad_repo.clone()), &bad_run.id).is_err());
    assert_eq!(
        fs::read_to_string(bad_repo.join("a.txt")).unwrap().trim(),
        "0"
    );
    assert_eq!(
        fs::read_to_string(bad_repo.join("b.txt")).unwrap().trim(),
        "0"
    );

    // All workers succeeded; only the independent candidate check failed.
    // Explicit refresh with repaired primary verification inputs must recover
    // the review, while retaining the exact frozen worker targets.
    let verify_file = if cfg!(windows) {
        "verify.ps1"
    } else {
        "verify.sh"
    };
    fs::write(bad_repo.join(verify_file), "exit 0\n").unwrap();
    let recovered = refresh_run_review(None, Some(bad_repo.clone()), &bad_run.id).unwrap();
    assert_eq!(recovered.version, 3);
    assert_eq!(store.list_runs(1).unwrap()[0].status, "completed");
    apply_run_changes(None, Some(bad_repo.clone()), &bad_run.id).unwrap();
    assert_eq!(
        fs::read_to_string(bad_repo.join("a.txt")).unwrap().trim(),
        "1"
    );
    assert_eq!(
        fs::read_to_string(bad_repo.join("b.txt")).unwrap().trim(),
        "1"
    );

    let repo = temp.path().join("single-passes");
    let verify = fixture(&repo);
    let id = run(options(&repo, &["a"], &verify)).await.unwrap();
    let data = repo.join(".pytxo/data");
    let live_store = PytxoStore::open(&data.join("pytxo.db")).unwrap();
    let live_events = live_store
        .list_events(&format!("{}:agent-0", id.0), 100)
        .unwrap();
    assert!(live_events
        .iter()
        .any(|event| event.kind == "verify-boundary"));
    assert!(!live_events.iter().any(|event| event.kind == "evidence-gap"));
    let prepared = load_review_package(&data, &id.0).unwrap();
    assert_eq!(prepared.version, 3);
    assert_eq!(
        prepared
            .candidate_verification
            .as_ref()
            .unwrap()
            .checks
            .len(),
        1
    );
    fs::write(repo.join("operator-note.txt"), "preserve me\n").unwrap();
    assert!(apply_run_changes(None, Some(repo.clone()), &id.0)
        .unwrap_err()
        .to_string()
        .contains("inventory drifted"));
    let refreshed = refresh_run_review(None, Some(repo.clone()), &id.0).unwrap();
    assert_ne!(prepared.package_digest, refreshed.package_digest);
    apply_run_changes(None, Some(repo.clone()), &id.0).unwrap();
    assert_eq!(fs::read_to_string(repo.join("a.txt")).unwrap().trim(), "1");
    assert_eq!(
        fs::read_to_string(repo.join("operator-note.txt")).unwrap(),
        "preserve me\n"
    );
}
