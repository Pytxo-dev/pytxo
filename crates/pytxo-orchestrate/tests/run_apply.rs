use std::path::Path;
use std::process::Command;

use pytxo_core::{
    DomainId, ExecutionPlan, IsolationMode, PermissionProfile, PreparedRunManifest, ScheduledTask,
    TaskId,
};
use pytxo_orchestrate::{
    apply_run_changes as apply_reviewed_run_changes, discard_run_review, refresh_run_review,
};
use pytxo_runner::{
    apply_prepared_review, permission_enforcement_receipt, prepare_review_package,
    run_candidate_check, AgentWorkspaceInput, CandidateCheckContext, CandidateVerification,
    RunApplyManifest, SwarmRegistry,
};
use pytxo_store::PytxoStore;

/// Apply registers its execution domain in the default hypervisor catalog.
/// Point that catalog at one temp home for this test process so test
/// repositories never appear in the developer's real `~/.pytxo`.
fn isolate_home() {
    static HOME: std::sync::OnceLock<tempfile::TempDir> = std::sync::OnceLock::new();
    HOME.get_or_init(|| {
        let home = tempfile::tempdir().expect("isolated PYTXO_HOME");
        std::env::set_var("PYTXO_HOME", home.path());
        home
    });
}

// Existing integrity/recovery tests explicitly review the current persisted
// snapshot. Stale-client tests below retain their original digest instead.
fn apply_run_changes(
    config: Option<std::path::PathBuf>,
    repo: Option<std::path::PathBuf>,
    run_id: &str,
) -> anyhow::Result<RunApplyManifest> {
    let store = PytxoStore::open(&repo.as_ref().unwrap().join(".pytxo/data/pytxo.db"))?;
    let digest = store
        .get_run_contract(run_id)?
        .and_then(|contract| contract.prepared_digest)
        .unwrap_or_default();
    apply_reviewed_run_changes(config, repo, run_id, &digest)
}

fn git(repo: &Path, args: &[&str]) -> String {
    let output = Command::new("git")
        .args(args)
        .current_dir(repo)
        .output()
        .expect("run git");
    assert!(
        output.status.success(),
        "git {:?}: {}",
        args,
        String::from_utf8_lossy(&output.stderr)
    );
    String::from_utf8_lossy(&output.stdout).trim().to_string()
}

fn write(path: &Path, contents: &str) {
    std::fs::create_dir_all(path.parent().expect("parent")).expect("create parent");
    std::fs::write(path, contents).expect("write fixture");
}

fn check_contents(path: &str, expected: &str) -> String {
    if cfg!(windows) {
        format!("powershell -NoProfile -NonInteractive -Command \"if ((Get-Content -Raw '{path}').Trim() -ne '{expected}') {{ exit 1 }}\"")
    } else {
        format!("grep -qx '{expected}' '{path}'")
    }
}

fn attest(
    repo: &Path,
    data: &Path,
    manifest: PreparedRunManifest,
    command: &str,
) -> PreparedRunManifest {
    let candidate = CandidateVerification::prepare(repo, data, &manifest, &[]).unwrap();
    let receipt = run_candidate_check(
        &CandidateCheckContext {
            cwd: candidate.workspace_root().into(),
            run_id: manifest.run_id.clone(),
            agent_key: format!("{}:candidate", manifest.run_id),
            repo_root: repo.into(),
            data_dir: data.into(),
            profile: PermissionProfile::Orbit,
            domain_id: DomainId::from_repo_root(repo).unwrap(),
            execution_backend: pytxo_core::ExecutionBackend::Subprocess,
            workspace_isolated: true,
            hitl: None,
            swarm: SwarmRegistry::new(),
            on_event: None,
        },
        command,
    )
    .unwrap();
    candidate.check_unchanged().unwrap();
    candidate
        .finish(vec![pytxo_core::CandidateCheckEvidence {
            task_id: "implement".into(),
            command: command.into(),
            effective_profile: "orbit".into(),
            passed: true,
            enforcement: serde_json::to_value(receipt).unwrap(),
        }])
        .unwrap()
}

struct PreparedFixture {
    _temp: tempfile::TempDir,
    repo: std::path::PathBuf,
    data_dir: std::path::PathBuf,
    db_path: std::path::PathBuf,
    workspace: std::path::PathBuf,
    run_id: String,
    prepared: PreparedRunManifest,
}

fn prepared_fixture(run_id: &str) -> PreparedFixture {
    isolate_home();
    let temp = tempfile::tempdir().expect("tempdir");
    let repo = temp.path().join("repo");
    std::fs::create_dir_all(&repo).unwrap();
    git(&repo, &["init"]);
    git(&repo, &["config", "user.email", "pytxo@example.invalid"]);
    git(&repo, &["config", "user.name", "Pytxo Test"]);
    write(&repo.join("value.txt"), "before\n");
    git(&repo, &["add", "."]);
    git(&repo, &["commit", "-m", "base"]);
    let base_revision = git(&repo, &["rev-parse", "HEAD"]);
    let workspace = temp.path().join("workspace");
    write(&workspace.join("value.txt"), "after\n");
    let plan = ExecutionPlan {
        waves: vec![vec![ScheduledTask {
            task_id: TaskId("implement".into()),
            agent: "codex".into(),
            paths: vec!["value.txt".into()],
            depends_on: vec![],
            wave: 0,
            root: None,
            signal_fidelity: None,
            verify: vec![check_contents("value.txt", "after")],
        }]],
        conflicts: vec![],
        max_agents: 1,
        warnings: vec![],
    };
    let receipt = permission_enforcement_receipt(
        PermissionProfile::Orbit,
        PermissionProfile::Orbit,
        &DomainId::from_repo_root(&repo).unwrap(),
        IsolationMode::Worktree,
        &[],
    )
    .unwrap();
    let enforcement = serde_json::json!({
        "run": receipt,
        "agents": { "codex": receipt, "agent-0": receipt }
    });
    let data_dir = repo.join(".pytxo/data");
    let db_path = data_dir.join("pytxo.db");
    let store = PytxoStore::open(&db_path).unwrap();
    store
        .insert_run_with_profile(run_id, &repo.to_string_lossy(), Some("orbit"))
        .unwrap();
    store
        .save_run_contract(
            run_id,
            &base_revision,
            &serde_json::to_string(&plan).unwrap(),
            &serde_json::to_string(&enforcement).unwrap(),
        )
        .unwrap();
    store
        .insert_agent(
            &format!("{run_id}:agent-0"),
            run_id,
            "implement",
            0,
            Some(&workspace.to_string_lossy()),
            "codex",
        )
        .unwrap();
    store
        .finish_agent(&format!("{run_id}:agent-0"), Some(0), "completed")
        .unwrap();
    assert!(store.begin_run_preparation(run_id).unwrap());
    let prepared = prepare_review_package(
        &repo,
        &data_dir,
        run_id,
        &base_revision,
        &[AgentWorkspaceInput {
            agent_id: "agent-0".into(),
            task_id: "implement".into(),
            workspace_path: workspace.clone(),
            claims: vec!["value.txt".into()],
            depends_on: vec![],
        }],
        &[],
    )
    .unwrap();
    let prepared = attest(&repo, &data_dir, prepared, &plan.waves[0][0].verify[0]);
    store.finish_run_preparation(run_id, &prepared).unwrap();
    store.finish_run(run_id, "completed").unwrap();
    drop(store);
    PreparedFixture {
        _temp: temp,
        repo,
        data_dir,
        db_path,
        workspace,
        run_id: run_id.into(),
        prepared,
    }
}

#[test]
fn stale_client_cannot_authorize_a_refreshed_candidate_for_the_same_run() {
    assert_stale_client_rejected(false);
}

#[test]
fn fresh_check_evidence_alone_invalidates_the_previous_authorization() {
    assert_stale_client_rejected(true);
}

fn assert_stale_client_rejected(evidence_only: bool) {
    let fixture = prepared_fixture("two-client-review");
    let reviewed_a = fixture.prepared.package_digest.clone();
    // Client one keeps A open. An operator edits the primary; client two's
    // Apply detects drift and explicitly refreshes the same run against it.
    write(&fixture.repo.join("value.txt"), "operator-edit\n");
    assert!(apply_run_changes(None, Some(fixture.repo.clone()), &fixture.run_id).is_err());
    if evidence_only {
        write(&fixture.repo.join("value.txt"), "before\n");
    }
    let reviewed_b = refresh_run_review(None, Some(fixture.repo.clone()), &fixture.run_id)
        .expect("legitimate refresh after drift");
    assert_ne!(reviewed_a, reviewed_b.package_digest);
    assert_eq!(
        fixture.prepared.files[0].after_sha256,
        reviewed_b.files[0].after_sha256
    );
    if evidence_only {
        assert_eq!(fixture.prepared.files, reviewed_b.files);
    }

    let stale = apply_reviewed_run_changes(
        None,
        Some(fixture.repo.clone()),
        &fixture.run_id,
        &reviewed_a,
    );
    assert!(
        stale.is_err(),
        "client one's review of A must not authorize B"
    );
    assert!(matches!(
        stale.unwrap_err().downcast_ref::<pytxo_core::PytxoError>(),
        Some(pytxo_core::PytxoError::StaleReview)
    ));
    assert_eq!(
        std::fs::read_to_string(fixture.repo.join("value.txt")).unwrap(),
        if evidence_only {
            "before\n"
        } else {
            "operator-edit\n"
        }
    );
    let store = PytxoStore::open(&fixture.db_path).unwrap();
    assert_eq!(
        store
            .get_run_contract(&fixture.run_id)
            .unwrap()
            .unwrap()
            .apply_status,
        "ready"
    );
    assert!(store
        .list_events(&format!("{}:agent-0", fixture.run_id), 100)
        .unwrap()
        .iter()
        .any(|event| event.kind == "review-authorization-refused"));
    drop(store);
    apply_reviewed_run_changes(
        None,
        Some(fixture.repo.clone()),
        &fixture.run_id,
        &reviewed_b.package_digest,
    )
    .expect("freshly reviewed B applies");
    assert_eq!(
        std::fs::read_to_string(fixture.repo.join("value.txt")).unwrap(),
        "after\n"
    );
}

#[test]
fn missing_or_wrong_review_identity_refuses_without_claiming_apply() {
    let fixture = prepared_fixture("missing-review-identity");
    for digest in ["", "another-runs-digest"] {
        let error =
            apply_reviewed_run_changes(None, Some(fixture.repo.clone()), &fixture.run_id, digest)
                .unwrap_err();
        assert!(matches!(
            error.downcast_ref::<pytxo_core::PytxoError>(),
            Some(pytxo_core::PytxoError::StaleReview)
        ));
    }
    assert!(
        pytxo_runner::apply_attempt_ids(&fixture.data_dir, &fixture.run_id)
            .unwrap()
            .is_empty()
    );
    assert_eq!(
        std::fs::read_to_string(fixture.repo.join("value.txt")).unwrap(),
        "before\n"
    );
    assert_eq!(
        PytxoStore::open(&fixture.db_path)
            .unwrap()
            .get_run_contract(&fixture.run_id)
            .unwrap()
            .unwrap()
            .apply_status,
        "ready"
    );
}

#[test]
fn concurrent_clients_cannot_apply_the_review_twice() {
    let fixture = prepared_fixture("concurrent-reviewed-apply");
    let barrier = std::sync::Arc::new(std::sync::Barrier::new(2));
    let clients: Vec<_> = (0..2)
        .map(|_| {
            let repo = fixture.repo.clone();
            let run_id = fixture.run_id.clone();
            let digest = fixture.prepared.package_digest.clone();
            let barrier = barrier.clone();
            std::thread::spawn(move || {
                barrier.wait();
                apply_reviewed_run_changes(None, Some(repo), &run_id, &digest)
            })
        })
        .collect();
    let successes = clients
        .into_iter()
        .filter_map(|client| client.join().unwrap().ok())
        .count();
    assert_eq!(successes, 1);
    assert_eq!(
        pytxo_runner::apply_attempt_ids(&fixture.data_dir, &fixture.run_id)
            .unwrap()
            .len(),
        1
    );
}

fn assert_apply_runtime_receipt_rejected(missing: bool) {
    let fixture = prepared_fixture(if missing {
        "missing-runtime-receipt"
    } else {
        "wrong-runtime-domain"
    });
    let store = PytxoStore::open(&fixture.db_path).unwrap();
    let contract = store.get_run_contract(&fixture.run_id).unwrap().unwrap();
    let mut enforcement: serde_json::Value =
        serde_json::from_str(contract.enforcement_json.as_deref().unwrap()).unwrap();
    if missing {
        enforcement["agents"]
            .as_object_mut()
            .unwrap()
            .remove("agent-0");
    } else {
        let mut receipt = enforcement["agents"]["codex"].clone();
        receipt["execution_domain"] = "another-domain".into();
        enforcement["agents"]["agent-0"] = receipt;
    }
    store
        .save_run_contract(
            &fixture.run_id,
            contract.base_revision.as_deref().unwrap(),
            contract.plan_json.as_deref().unwrap(),
            &enforcement.to_string(),
        )
        .unwrap();
    assert!(store.begin_run_preparation(&fixture.run_id).unwrap());
    store
        .finish_run_preparation(&fixture.run_id, &fixture.prepared)
        .unwrap();
    let error = apply_run_changes(None, Some(fixture.repo.clone()), &fixture.run_id)
        .expect_err("Apply accepted an invalid runtime receipt");
    assert!(
        error.to_string().contains(if missing {
            "no runtime enforcement receipt"
        } else {
            "another execution domain"
        }),
        "unexpected refusal: {error}"
    );
    assert_eq!(
        std::fs::read_to_string(fixture.repo.join("value.txt")).unwrap(),
        "before\n"
    );
}

#[test]
fn apply_runtime_receipt_missing_actor_fails_closed() {
    assert_apply_runtime_receipt_rejected(true);
}

#[test]
fn apply_runtime_receipt_wrong_domain_fails_closed() {
    assert_apply_runtime_receipt_rejected(false);
}

#[test]
fn missing_review_package_is_review_failed_and_discardable() {
    let fixture = prepared_fixture("run-missing-package");
    std::fs::remove_dir_all(fixture.data_dir.join("reviews").join(&fixture.run_id)).unwrap();

    apply_run_changes(None, Some(fixture.repo.clone()), &fixture.run_id)
        .expect_err("missing immutable package must fail safely");

    let store = PytxoStore::open(&fixture.db_path).unwrap();
    let contract = store.get_run_contract(&fixture.run_id).unwrap().unwrap();
    assert_eq!(contract.apply_status, "review_failed");
    assert_eq!(
        contract
            .last_apply_error
            .as_ref()
            .map(|error| error.code.as_str()),
        Some("review_package_invalid")
    );
    assert!(contract.recovery_state.is_none());
    drop(store);
    discard_run_review(None, Some(fixture.repo.clone()), &fixture.run_id)
        .expect("safe package failure remains discardable");
    assert_eq!(
        PytxoStore::open(&fixture.db_path)
            .unwrap()
            .get_run_contract(&fixture.run_id)
            .unwrap()
            .unwrap()
            .apply_status,
        "discarded"
    );
}

#[test]
fn applying_contract_without_a_journal_fails_closed_as_recovery_required() {
    let fixture = prepared_fixture("run-claim-crash");
    let store = PytxoStore::open(&fixture.db_path).unwrap();
    assert!(store.claim_run_apply(&fixture.run_id).unwrap());
    drop(store);

    let error = apply_run_changes(None, Some(fixture.repo.clone()), &fixture.run_id)
        .expect_err("a crash after claim but before journal must block Apply");
    assert!(error.to_string().contains("recovery_required"));

    let contract = PytxoStore::open(&fixture.db_path)
        .unwrap()
        .get_run_contract(&fixture.run_id)
        .unwrap()
        .unwrap();
    assert_eq!(contract.apply_status, "recovery_required");
    assert_eq!(
        contract
            .last_apply_error
            .as_ref()
            .map(|error| error.code.as_str()),
        Some("recovery_journal_missing")
    );
}

#[test]
fn corrupt_review_package_is_review_failed_and_refreshable() {
    let fixture = prepared_fixture("run-corrupt-package");
    std::fs::write(
        fixture
            .data_dir
            .join("reviews")
            .join(&fixture.run_id)
            .join("manifest.json"),
        b"{not-json",
    )
    .unwrap();

    apply_run_changes(None, Some(fixture.repo.clone()), &fixture.run_id)
        .expect_err("corrupt immutable package must fail safely");
    assert_eq!(
        PytxoStore::open(&fixture.db_path)
            .unwrap()
            .get_run_contract(&fixture.run_id)
            .unwrap()
            .unwrap()
            .apply_status,
        "review_failed"
    );
    refresh_run_review(None, Some(fixture.repo.clone()), &fixture.run_id)
        .expect("safe package failure remains refreshable");
    apply_run_changes(None, Some(fixture.repo.clone()), &fixture.run_id)
        .expect("refreshed package remains retryable");
    assert_eq!(
        std::fs::read_to_string(fixture.repo.join("value.txt")).unwrap(),
        "after\n"
    );
}

#[test]
fn version_one_review_package_becomes_visible_refreshable_upgrade_state() {
    let fixture = prepared_fixture("run-v1-package-upgrade");
    let manifest_path = fixture
        .data_dir
        .join("reviews")
        .join(&fixture.run_id)
        .join("manifest.json");
    let mut legacy: serde_json::Value =
        serde_json::from_slice(&std::fs::read(&manifest_path).unwrap()).unwrap();
    legacy["version"] = serde_json::json!(1);
    std::fs::write(&manifest_path, serde_json::to_vec_pretty(&legacy).unwrap()).unwrap();

    let error = apply_run_changes(None, Some(fixture.repo.clone()), &fixture.run_id)
        .expect_err("v1 package must require an explicit safe refresh");
    assert!(error.to_string().contains("version 1"));
    let contract = PytxoStore::open(&fixture.db_path)
        .unwrap()
        .get_run_contract(&fixture.run_id)
        .unwrap()
        .unwrap();
    assert_eq!(contract.apply_status, "review_failed");
    assert_eq!(
        contract
            .last_apply_error
            .as_ref()
            .map(|failure| failure.code.as_str()),
        Some("review_package_upgrade_required")
    );
    assert!(contract
        .last_apply_error
        .as_ref()
        .unwrap()
        .message
        .contains("explicit refresh"));

    let refreshed = refresh_run_review(None, Some(fixture.repo.clone()), &fixture.run_id)
        .expect("review_failed v1 package exposes a working refresh path");
    assert_eq!(refreshed.version, 3);
    apply_run_changes(None, Some(fixture.repo.clone()), &fixture.run_id)
        .expect("refreshed v3 package has passing candidate evidence");
}

#[test]
fn contract_package_digest_mismatch_is_review_failed_without_recovery_state() {
    let fixture = prepared_fixture("run-digest-mismatch");
    write(&fixture.workspace.join("value.txt"), "different package\n");
    let replacement = prepare_review_package(
        &fixture.repo,
        &fixture.data_dir,
        &fixture.run_id,
        &fixture.prepared.base_revision,
        &[AgentWorkspaceInput {
            agent_id: "agent-0".into(),
            task_id: "implement".into(),
            workspace_path: fixture.workspace.clone(),
            claims: vec!["value.txt".into()],
            depends_on: vec![],
        }],
        &[],
    )
    .unwrap();
    assert_ne!(replacement.package_digest, fixture.prepared.package_digest);

    apply_run_changes(None, Some(fixture.repo.clone()), &fixture.run_id)
        .expect_err("contract/package mismatch must fail safely");
    let contract = PytxoStore::open(&fixture.db_path)
        .unwrap()
        .get_run_contract(&fixture.run_id)
        .unwrap()
        .unwrap();
    assert_eq!(contract.apply_status, "review_failed");
    assert_eq!(
        contract
            .last_apply_error
            .as_ref()
            .map(|error| error.code.as_str()),
        Some("review_package_mismatch")
    );
    assert!(contract.recovery_state.is_none());
}

#[test]
fn review_manifest_persistence_failure_settles_contract_and_keeps_original_error() {
    let fixture = prepared_fixture("run-review-persistence");
    std::fs::remove_dir_all(fixture.data_dir.join("reviews").join(&fixture.run_id)).unwrap();
    apply_run_changes(None, Some(fixture.repo.clone()), &fixture.run_id)
        .expect_err("missing package enters refreshable review_failed");
    let connection = rusqlite::Connection::open(&fixture.db_path).unwrap();
    connection
        .execute_batch(
            "CREATE TRIGGER inject_review_persistence_failure
             BEFORE UPDATE OF apply_status ON run_contracts
             WHEN NEW.apply_status = 'ready'
             BEGIN
                 SELECT RAISE(FAIL, 'injected review persistence failure');
             END;",
        )
        .unwrap();
    drop(connection);

    let error = refresh_run_review(None, Some(fixture.repo.clone()), &fixture.run_id)
        .expect_err("manifest persistence must report the original failure");
    assert!(
        error
            .to_string()
            .contains("injected review persistence failure"),
        "unexpected error: {error}"
    );
    let contract = PytxoStore::open(&fixture.db_path)
        .unwrap()
        .get_run_contract(&fixture.run_id)
        .unwrap()
        .unwrap();
    assert_eq!(contract.apply_status, "review_failed");
    assert_eq!(
        contract
            .last_apply_error
            .as_ref()
            .map(|error| error.code.as_str()),
        Some("review_persistence_failed")
    );
    let connection = rusqlite::Connection::open(&fixture.db_path).unwrap();
    connection
        .execute_batch("DROP TRIGGER inject_review_persistence_failure;")
        .unwrap();
    drop(connection);
    refresh_run_review(None, Some(fixture.repo.clone()), &fixture.run_id)
        .expect("settled persistence failure remains refreshable");
    discard_run_review(None, Some(fixture.repo.clone()), &fixture.run_id)
        .expect("settled persistence failure remains discardable");
}

#[test]
fn applies_a_completed_single_domain_run_once() {
    isolate_home();
    let temp = tempfile::tempdir().expect("tempdir");
    let repo = temp.path().join("repo");
    std::fs::create_dir_all(&repo).expect("create repo");
    git(&repo, &["init"]);
    git(&repo, &["config", "user.email", "pytxo@example.invalid"]);
    git(&repo, &["config", "user.name", "Pytxo Test"]);
    write(&repo.join("src/value.txt"), "before\n");
    git(&repo, &["add", "."]);
    git(&repo, &["commit", "-m", "base"]);
    let base_revision = git(&repo, &["rev-parse", "HEAD"]);

    let workspace = temp.path().join("agent-workspace");
    write(&workspace.join("src/value.txt"), "after\n");

    let run_id = "run-apply";
    let plan = ExecutionPlan {
        waves: vec![vec![ScheduledTask {
            task_id: TaskId("implement".into()),
            agent: "codex".into(),
            paths: vec!["src/value.txt".into()],
            depends_on: vec![],
            wave: 0,
            root: None,
            signal_fidelity: None,
            verify: vec![check_contents("src/value.txt", "after")],
        }]],
        conflicts: vec![],
        max_agents: 1,
        warnings: vec![],
    };
    let db_path = repo.join(".pytxo/data/pytxo.db");
    let store = PytxoStore::open(&db_path).expect("open store");
    let receipt = permission_enforcement_receipt(
        PermissionProfile::Orbit,
        PermissionProfile::Orbit,
        &DomainId::from_repo_root(&repo).expect("domain id"),
        IsolationMode::Worktree,
        &[],
    )
    .expect("enforcement receipt");
    let enforcement = serde_json::json!({
        "run": receipt,
        "agents": {
            "codex": receipt,
            "agent-0": receipt,
        }
    });
    store
        .insert_run_with_profile(run_id, &repo.to_string_lossy(), Some("orbit"))
        .expect("insert run");
    store
        .save_run_contract(
            run_id,
            &base_revision,
            &serde_json::to_string(&plan).expect("serialize plan"),
            &serde_json::to_string(&enforcement).expect("serialize enforcement"),
        )
        .expect("save contract");
    store
        .insert_agent(
            &format!("{run_id}:agent-0"),
            run_id,
            "implement",
            0,
            Some(&workspace.to_string_lossy()),
            "codex",
        )
        .expect("insert agent");
    store
        .finish_agent(&format!("{run_id}:agent-0"), Some(0), "completed")
        .expect("finish agent");
    assert!(store.begin_run_preparation(run_id).unwrap());
    let prepared = prepare_review_package(
        &repo,
        &repo.join(".pytxo/data"),
        run_id,
        &base_revision,
        &[AgentWorkspaceInput {
            agent_id: "agent-0".into(),
            task_id: "implement".into(),
            workspace_path: workspace,
            claims: vec!["src/value.txt".into()],
            depends_on: vec![],
        }],
        &[],
    )
    .unwrap();
    let prepared = attest(
        &repo,
        &repo.join(".pytxo/data"),
        prepared,
        &plan.waves[0][0].verify[0],
    );
    store.finish_run_preparation(run_id, &prepared).unwrap();
    store.finish_run(run_id, "completed").expect("finish run");
    drop(store);

    write(
        &repo.join("unrelated-local-note.txt"),
        "must not block apply\n",
    );

    apply_run_changes(None, Some(repo.clone()), run_id)
        .expect_err("new source must invalidate candidate evidence");
    refresh_run_review(None, Some(repo.clone()), run_id)
        .expect("recheck includes and preserves the operator note");

    let manifest =
        apply_run_changes(None, Some(repo.clone()), run_id).expect("apply completed run");
    assert_eq!(manifest.changes.len(), 1);
    assert_eq!(
        std::fs::read_to_string(repo.join("src/value.txt")).expect("read applied file"),
        "after\n"
    );
    assert_eq!(
        std::fs::read_to_string(repo.join("unrelated-local-note.txt")).unwrap(),
        "must not block apply\n"
    );

    let store = PytxoStore::open(&db_path).expect("reopen store");
    assert_eq!(
        store
            .get_run_contract(run_id)
            .expect("load contract")
            .expect("contract")
            .apply_status,
        "applied"
    );
    assert!(apply_run_changes(None, Some(repo), run_id)
        .expect_err("run-level apply must be one-shot")
        .to_string()
        .contains("not ready"));
}

#[test]
fn rejects_affected_primary_edits_made_after_review() {
    isolate_home();
    let temp = tempfile::tempdir().expect("tempdir");
    let repo = temp.path().join("repo");
    std::fs::create_dir_all(&repo).expect("create repo");
    git(&repo, &["init"]);
    git(&repo, &["config", "user.email", "pytxo@example.invalid"]);
    git(&repo, &["config", "user.name", "Pytxo Test"]);
    write(&repo.join("src/value.txt"), "before\n");
    git(&repo, &["add", "."]);
    git(&repo, &["commit", "-m", "base"]);
    let base_revision = git(&repo, &["rev-parse", "HEAD"]);

    let workspace = temp.path().join("agent-workspace");
    write(&workspace.join("src/value.txt"), "agent-result\n");
    let run_id = "run-dirty";
    let plan = ExecutionPlan {
        waves: vec![vec![ScheduledTask {
            task_id: TaskId("implement".into()),
            agent: "codex".into(),
            paths: vec!["src/value.txt".into()],
            depends_on: vec![],
            wave: 0,
            root: None,
            signal_fidelity: None,
            verify: vec![check_contents("src/value.txt", "agent-result")],
        }]],
        conflicts: vec![],
        max_agents: 1,
        warnings: vec![],
    };
    let receipt = permission_enforcement_receipt(
        PermissionProfile::Orbit,
        PermissionProfile::Orbit,
        &DomainId::from_repo_root(&repo).expect("domain id"),
        IsolationMode::Worktree,
        &[],
    )
    .expect("enforcement receipt");
    let enforcement = serde_json::json!({
        "run": receipt,
        "agents": { "codex": receipt, "agent-0": receipt }
    });
    let db_path = repo.join(".pytxo/data/pytxo.db");
    let store = PytxoStore::open(&db_path).expect("open store");
    store
        .insert_run_with_profile(run_id, &repo.to_string_lossy(), Some("orbit"))
        .expect("insert run");
    store
        .save_run_contract(
            run_id,
            &base_revision,
            &serde_json::to_string(&plan).expect("serialize plan"),
            &serde_json::to_string(&enforcement).expect("serialize enforcement"),
        )
        .expect("save contract");
    store
        .insert_agent(
            &format!("{run_id}:agent-0"),
            run_id,
            "implement",
            0,
            Some(&workspace.to_string_lossy()),
            "codex",
        )
        .expect("insert agent");
    store
        .finish_agent(&format!("{run_id}:agent-0"), Some(0), "completed")
        .expect("finish agent");
    assert!(store.begin_run_preparation(run_id).unwrap());
    let prepared = prepare_review_package(
        &repo,
        &repo.join(".pytxo/data"),
        run_id,
        &base_revision,
        &[AgentWorkspaceInput {
            agent_id: "agent-0".into(),
            task_id: "implement".into(),
            workspace_path: workspace,
            claims: vec!["src/value.txt".into()],
            depends_on: vec![],
        }],
        &[],
    )
    .unwrap();
    let prepared = attest(
        &repo,
        &repo.join(".pytxo/data"),
        prepared,
        &plan.waves[0][0].verify[0],
    );
    store.finish_run_preparation(run_id, &prepared).unwrap();
    store.finish_run(run_id, "completed").expect("finish run");
    drop(store);

    write(&repo.join("src/value.txt"), "newer-human-edit\n");
    let error = apply_run_changes(None, Some(repo.clone()), run_id)
        .expect_err("dirty primary checkout must invalidate Apply");

    assert!(
        error.to_string().contains("changed since review")
            && error.to_string().contains("src/value.txt"),
        "unexpected error: {error}"
    );
    assert_eq!(
        std::fs::read_to_string(repo.join("src/value.txt")).expect("read primary repo"),
        "newer-human-edit\n"
    );

    refresh_run_review(None, Some(repo.clone()), run_id).expect("refresh stale review");
    apply_run_changes(None, Some(repo.clone()), run_id).expect("apply refreshed review");
    assert_eq!(
        std::fs::read_to_string(repo.join("src/value.txt")).unwrap(),
        "agent-result\n"
    );
}

#[test]
fn recovered_commit_persists_the_normal_apply_audit_schema_and_attempt_id() {
    isolate_home();
    let temp = tempfile::tempdir().expect("tempdir");
    let repo = temp.path().join("repo");
    std::fs::create_dir_all(&repo).unwrap();
    git(&repo, &["init"]);
    git(&repo, &["config", "user.email", "pytxo@example.invalid"]);
    git(&repo, &["config", "user.name", "Pytxo Test"]);
    write(&repo.join("value.txt"), "before\n");
    git(&repo, &["add", "."]);
    git(&repo, &["commit", "-m", "base"]);
    let base_revision = git(&repo, &["rev-parse", "HEAD"]);
    let workspace = temp.path().join("workspace");
    write(&workspace.join("value.txt"), "after\n");
    let run_id = "run-recovered-commit";
    let plan = ExecutionPlan {
        waves: vec![vec![ScheduledTask {
            task_id: TaskId("implement".into()),
            agent: "codex".into(),
            paths: vec!["value.txt".into()],
            depends_on: vec![],
            wave: 0,
            root: None,
            signal_fidelity: None,
            verify: vec![],
        }]],
        conflicts: vec![],
        max_agents: 1,
        warnings: vec![],
    };
    let receipt = permission_enforcement_receipt(
        PermissionProfile::Orbit,
        PermissionProfile::Orbit,
        &DomainId::from_repo_root(&repo).unwrap(),
        IsolationMode::Worktree,
        &[],
    )
    .unwrap();
    let enforcement = serde_json::json!({
        "run": receipt,
        "agents": { "codex": receipt, "agent-0": receipt }
    });
    let data_dir = repo.join(".pytxo/data");
    let db_path = data_dir.join("pytxo.db");
    let store = PytxoStore::open(&db_path).unwrap();
    store
        .insert_run_with_profile(run_id, &repo.to_string_lossy(), Some("orbit"))
        .unwrap();
    store
        .save_run_contract(
            run_id,
            &base_revision,
            &serde_json::to_string(&plan).unwrap(),
            &serde_json::to_string(&enforcement).unwrap(),
        )
        .unwrap();
    let prepared = prepare_review_package(
        &repo,
        &data_dir,
        run_id,
        &base_revision,
        &[AgentWorkspaceInput {
            agent_id: "agent-0".into(),
            task_id: "implement".into(),
            workspace_path: workspace,
            claims: vec!["value.txt".into()],
            depends_on: vec![],
        }],
        &[],
    )
    .unwrap();
    assert!(store.begin_run_preparation(run_id).unwrap());
    store.finish_run_preparation(run_id, &prepared).unwrap();
    store.finish_run(run_id, "completed").unwrap();
    assert!(store.claim_run_apply(run_id).unwrap());
    let committed = apply_prepared_review(&repo, &data_dir, &prepared).unwrap();
    drop(store);

    let recovered = apply_run_changes(None, Some(repo.clone()), run_id).unwrap();
    assert_eq!(recovered, committed);
    let store = PytxoStore::open(&db_path).unwrap();
    let contract = store.get_run_contract(run_id).unwrap().unwrap();
    let audit: RunApplyManifest =
        serde_json::from_str(contract.apply_manifest_json.as_deref().unwrap()).unwrap();
    assert_eq!(audit, committed);
    assert!(!audit.transaction_id.is_empty());
}
