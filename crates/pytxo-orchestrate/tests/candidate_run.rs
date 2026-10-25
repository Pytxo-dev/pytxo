use pytxo_core::{ExecutionBackend, PermissionProfile, Task, TaskId};
use pytxo_orchestrate::{
    apply_run_changes as apply_reviewed_run_changes, dispatch_desktop_beta_flow, preview_flow,
    refresh_run_review, run, save_reviewed_flow_plan, trust_repo, FlowDraftInput, FlowSource,
    RunOptions,
};
use pytxo_runner::load_review_package;
use pytxo_store::{Catalog, PytxoStore};
use std::{fs, path::Path, process::Command, time::Duration};

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
    assert!(
        error.to_string().contains("review preparation failed"),
        "{error:#}"
    );
    let store = PytxoStore::open(&bad_repo.join(".pytxo/data/pytxo.db")).unwrap();
    let bad_run = store.list_runs(1).unwrap().remove(0);
    assert!(store
        .list_agents_for_run(&bad_run.id)
        .unwrap()
        .iter()
        .all(|agent| agent.status == "completed"));
    let contract = store.get_run_contract(&bad_run.id).unwrap().unwrap();
    assert_eq!(contract.apply_status, "review_failed");
    // A check that ran and failed is recorded as its own outcome.
    let failure = contract.last_apply_error.unwrap();
    assert_eq!(failure.code, "candidate_check_failed");
    assert_eq!(
        failure.message,
        format!("a: {verify} failed on the combined change")
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
    // Refreshing while the combination still fails records the same outcome.
    assert!(refresh_run_review(None, Some(bad_repo.clone()), &bad_run.id).is_err());
    assert_eq!(
        store
            .get_run_contract(&bad_run.id)
            .unwrap()
            .unwrap()
            .last_apply_error
            .unwrap()
            .code,
        "candidate_check_failed"
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
    // The worker settles when it exits, not when the whole plan returns.
    assert!(live_events
        .iter()
        .any(|event| event.kind == "agent-exit" && event.payload == "completed 0"));
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
    // A committed Apply leaves exactly one Apply receipt naming this run.
    let receipts = pytxo_core::list_receipts(&pytxo_orchestrate::repo_receipts_dir(&repo).unwrap());
    let target = format!("run:{}", id.0);
    let apply: Vec<_> = receipts
        .iter()
        .filter(|(_, r)| r.predicate.target.as_deref() == Some(target.as_str()))
        .collect();
    assert_eq!(apply.len(), 1, "{receipts:?}");
    let receipt = &apply[0].1;
    assert_eq!(receipt.predicate.kind, "apply");
    assert_eq!(receipt.predicate.decision, "applied");
    assert_eq!(
        receipt.predicate.result.as_deref(),
        Some(refreshed.package_digest.as_str())
    );
    assert!(receipt.predicate.files_changed >= 1);
    assert!(receipt.digest_matches());
    // Its check is the one candidate verification observed, not placeholders.
    let [check] = receipt.predicate.checks.as_slice() else {
        panic!("one combined check: {:?}", receipt.predicate.checks);
    };
    let recorded = &refreshed.candidate_verification.as_ref().unwrap().checks[0];
    assert_eq!(check.scope, "combined");
    assert_eq!(check.command, verify);
    assert!(check.passed);
    assert_eq!(check.exit_code, Some(0));
    assert!(check.duration_ms > 0);
    assert_eq!(Some(check.duration_ms), recorded.duration_ms);
    assert!(receipt.predicate.duration_ms > 0, "Apply wall time");

    // Desktop's Flow dispatch records the same failed combination the same way.
    let flow_repo = temp.path().join("flow-combined-fails");
    let flow_verify = fixture(&flow_repo);
    fs::write(
        flow_repo.join("pytxo.toml"),
        "max_agents = 2\npermission_profile = \"orbit\"\nexecution_backend = \"pty\"\n",
    )
    .unwrap();
    for args in [
        vec!["add", "pytxo.toml"],
        vec![
            "-c",
            "user.name=Test",
            "-c",
            "user.email=test@pytxo.local",
            "-c",
            "commit.gpgsign=false",
            "commit",
            "-qm",
            "config",
        ],
    ] {
        assert!(Command::new("git")
            .args(args)
            .current_dir(&flow_repo)
            .status()
            .unwrap()
            .success());
    }
    // A stand-in Codex that writes the one file its task names.
    let bin = temp.path().join("bin");
    fs::create_dir_all(&bin).unwrap();
    #[cfg(windows)]
    fs::write(
        bin.join("codex.cmd"),
        "@findstr /c:\"a.txt\" >nul && (>a.txt echo 1) || (>b.txt echo 1)\r\n@exit /b 0\r\n",
    )
    .unwrap();
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        fs::write(
            bin.join("codex"),
            "#!/bin/sh\ncase \"$PYTXO_TASK_PROMPT\" in *a.txt*) echo 1 > a.txt;; *) echo 1 > b.txt;; esac\n",
        )
        .unwrap();
        fs::set_permissions(bin.join("codex"), fs::Permissions::from_mode(0o755)).unwrap();
    }
    let mut paths = vec![bin];
    paths.extend(std::env::split_paths(
        &std::env::var_os("PATH").unwrap_or_default(),
    ));
    std::env::set_var("PATH", std::env::join_paths(paths).unwrap());
    let catalog = Catalog::open(&temp.path().join("catalog.db")).unwrap();
    let plan = preview_flow(
        &catalog,
        FlowDraftInput {
            id: "combined".into(),
            title: "Both files".into(),
            mission_text: "update a.txt; update b.txt".into(),
            source: FlowSource::Text,
            domain_id: Some(flow_repo.to_string_lossy().into_owned()),
            project_id: None,
            ade_id: Some("codex".into()),
            ade_ids: Vec::new(),
            task_ades: Default::default(),
            ade_accounts: Default::default(),
            max_workers: None,
            verification_commands: vec![flow_verify.clone()],
        },
    )
    .unwrap();
    save_reviewed_flow_plan(&catalog, plan).unwrap();
    let flow_run = dispatch_desktop_beta_flow(&catalog, "combined").unwrap();
    let flow_store = PytxoStore::open(&flow_repo.join(".pytxo/data/pytxo.db")).unwrap();
    let deadline = tokio::time::Instant::now() + Duration::from_secs(60);
    while matches!(
        flow_store
            .get_run(&flow_run)
            .unwrap()
            .unwrap()
            .status
            .as_str(),
        "starting" | "running"
    ) {
        assert!(
            tokio::time::Instant::now() < deadline,
            "Flow run did not settle"
        );
        tokio::time::sleep(Duration::from_millis(50)).await;
    }
    assert!(flow_store
        .list_agents_for_run(&flow_run)
        .unwrap()
        .iter()
        .all(|agent| agent.status == "completed" && agent.exit_code == Some(0)));
    let contract = flow_store.get_run_contract(&flow_run).unwrap().unwrap();
    assert_eq!(contract.apply_status, "review_failed");
    let failure = contract.last_apply_error.unwrap();
    assert_eq!(failure.code, "candidate_check_failed");
    assert_eq!(
        failure.message,
        format!("mission-0: {flow_verify} failed on the combined change")
    );
}
