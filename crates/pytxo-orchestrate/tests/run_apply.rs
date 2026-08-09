use std::path::Path;
use std::process::Command;

use pytxo_core::{
    DomainId, ExecutionPlan, IsolationMode, PermissionProfile, PreparedRunManifest, ScheduledTask,
    TaskId,
};
use pytxo_orchestrate::{apply_run_changes, discard_run_review, refresh_run_review};
use pytxo_runner::{
    apply_prepared_review, permission_enforcement_receipt, prepare_review_package,
    AgentWorkspaceInput, RunApplyManifest,
};
use pytxo_store::PytxoStore;

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
        "agents": { "codex": receipt }
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
    assert_eq!(refreshed.version, 2);
    apply_run_changes(None, Some(fixture.repo.clone()), &fixture.run_id)
        .expect("refreshed v2 package remains applicable");
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
            verify: vec![],
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
    store.finish_run_preparation(run_id, &prepared).unwrap();
    store.finish_run(run_id, "completed").expect("finish run");
    drop(store);

    write(
        &repo.join("unrelated-local-note.txt"),
        "must not block apply\n",
    );

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
            verify: vec![],
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
        "agents": { "codex": receipt }
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
        "agents": { "codex": receipt }
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
