use std::collections::BTreeSet;
use std::fs;
use std::process::Command;
use std::sync::OnceLock;

use pytxo_core::routing::*;
use pytxo_core::{DomainId, ExecutionBackend, PermissionProfile, PytxoConfig, RunId, TaskId};
#[cfg(all(windows, feature = "routed-test-faults"))]
use pytxo_orchestrate::apply_run_changes;
use pytxo_orchestrate::flow::observe_experimental_routed_git_base;
use pytxo_orchestrate::{
    dispatch_experimental_routed_flow, dispatch_flow, enable_experimental_routed_advisor_consent,
    list_flow_drafts_with_routed_recovery, preview_experimental_routed_advisor_packet,
    preview_experimental_routed_flow, preview_proposed_hosted_advisor_packet,
    read_experimental_routed_advisor_consent, reconcile_routed_flow_startup,
    request_stop_experimental_routed_flow, revoke_experimental_routed_advisor_consent,
    save_reviewed_flow_plan, trust_repo, FlowDraftInput, FlowPlan, FlowSource, FlowStatus,
};
#[cfg(all(windows, feature = "routed-test-faults"))]
use pytxo_orchestrate::{
    preview_experimental_claude_proposal_flow_with_facts, ReviewedDemandFacts,
};
#[cfg(all(windows, feature = "routed-test-faults"))]
use pytxo_store::capacity::CapacityPoolConfig;
use pytxo_store::{routing::*, Catalog, PytxoStore, RoutedFlowDispatchOwner};

fn isolated_home() {
    static HOME: OnceLock<tempfile::TempDir> = OnceLock::new();
    HOME.get_or_init(|| {
        let home = tempfile::tempdir().unwrap();
        // This integration-test process exclusively owns these environment variables.
        unsafe {
            std::env::set_var("PYTXO_HOME", home.path().join("home"));
            std::env::set_var("PYTXO_TRUST_STORE", home.path().join("trust.json"));
            std::env::set_var("PYTXO_PLANNER", "signal");
        }
        home
    });
}

fn repo() -> tempfile::TempDir {
    isolated_home();
    let temp = tempfile::tempdir().unwrap();
    fs::create_dir_all(temp.path().join("src")).unwrap();
    fs::write(temp.path().join("src/lib.rs"), "pub fn value() {}\n").unwrap();
    for args in [
        vec!["init", "-q"],
        vec!["add", "."],
        vec![
            "-c",
            "user.name=Test",
            "-c",
            "user.email=test@pytxo.local",
            "commit",
            "-qm",
            "baseline",
        ],
    ] {
        let output = Command::new("git")
            .args(args)
            .current_dir(temp.path())
            .output()
            .unwrap();
        assert!(
            output.status.success(),
            "{}",
            String::from_utf8_lossy(&output.stderr)
        );
    }
    trust_repo(temp.path(), PermissionProfile::Orbit).unwrap();
    temp
}

#[test]
fn routed_git_base_rejects_clean_transformed_checkout_before_staging() {
    let source = repo();
    let normalized = tempfile::tempdir().unwrap();
    let repo = normalized.path().join("checkout");
    let output = Command::new("git")
        .args([
            "-c",
            "core.autocrlf=true",
            "clone",
            "--quiet",
            source.path().to_str().unwrap(),
            repo.to_str().unwrap(),
        ])
        .current_dir(source.path())
        .output()
        .unwrap();
    assert!(output.status.success());
    assert_eq!(
        fs::read(repo.join("src/lib.rs")).unwrap(),
        b"pub fn value() {}\r\n"
    );
    let output = Command::new("git")
        .args(["status", "--porcelain"])
        .current_dir(&repo)
        .output()
        .unwrap();
    assert!(output.status.success());
    assert!(output.stdout.is_empty());

    let error = observe_experimental_routed_git_base(&repo).unwrap_err();
    assert!(
        error
            .to_string()
            .contains("checkout bytes differ from Git blobs"),
        "{error}"
    );
}

#[test]
fn routed_flow_claim_owner_probe() {
    if std::env::var_os("PYTXO_TEST_CLAIM_PROBE").is_some() {
        std::thread::sleep(std::time::Duration::from_secs(30));
    }
}

fn claimed_flow_with_child() -> (
    tempfile::TempDir,
    Catalog,
    FlowPlan,
    String,
    std::path::PathBuf,
    std::process::Child,
    RoutedFlowDispatchOwner,
) {
    let repo = repo();
    let catalog = Catalog::open(&repo.path().join(".git/catalog.db")).unwrap();
    let plan = preview_experimental_routed_flow(&catalog, input(repo.path()), mission).unwrap();
    let reviewed = save_reviewed_flow_plan(&catalog, plan).unwrap();
    let run_id = staged_ref(&reviewed).run_id.0;
    let expected = catalog
        .get_flow_draft(&reviewed.draft_id)
        .unwrap()
        .unwrap()
        .plan_json
        .unwrap();
    let mut child = Command::new(std::env::current_exe().unwrap())
        .args(["--exact", "routed_flow_claim_owner_probe"])
        .env("PYTXO_TEST_CLAIM_PROBE", "1")
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::null())
        .spawn()
        .unwrap();
    let pid = child.id();
    let identity = pytxo_runner::process_start_identity(pid)
        .unwrap()
        .expect("live child has OS start identity");
    assert!(child.try_wait().unwrap().is_none());
    let store_path = fs::canonicalize(PytxoConfig::default().db_path_at(repo.path())).unwrap();
    let owner = RoutedFlowDispatchOwner {
        controller_pid: pid,
        controller_start_identity: identity,
        store_db_path: store_path.to_str().unwrap().into(),
        store_db_file_identity: Some(pytxo_runner::file_identity(&store_path).unwrap()),
    };
    assert!(catalog
        .claim_routed_flow_dispatch(&reviewed.draft_id, &expected, &run_id, &owner)
        .unwrap());
    (repo, catalog, reviewed, run_id, store_path, child, owner)
}

#[test]
fn reviewed_stop_before_claim_consumes_the_draft_without_creating_a_run() {
    let repo = repo();
    let catalog = Catalog::open(&repo.path().join(".git/catalog.db")).unwrap();
    let plan = preview_experimental_routed_flow(&catalog, input(repo.path()), mission).unwrap();
    let reviewed = save_reviewed_flow_plan(&catalog, plan).unwrap();
    let run_id = staged_ref(&reviewed).run_id.0;
    assert!(
        request_stop_experimental_routed_flow(&catalog, &reviewed.draft_id, &run_id)
            .unwrap()
            .is_none()
    );
    assert_eq!(
        catalog
            .get_flow_draft(&reviewed.draft_id)
            .unwrap()
            .unwrap()
            .status,
        "cancelled"
    );
    assert!(dispatch_experimental_routed_flow(&catalog, &reviewed.draft_id).is_err());
    let store = PytxoStore::open(&PytxoConfig::default().db_path_at(repo.path())).unwrap();
    assert!(store.get_run(&run_id).unwrap().is_none());
}

#[test]
fn dead_owner_before_startup_is_settled_without_inventing_a_run() {
    let (_repo, catalog, reviewed, run_id, store_path, mut child, _owner) =
        claimed_flow_with_child();
    assert!(list_flow_drafts_with_routed_recovery(&catalog)
        .unwrap()
        .iter()
        .any(|draft| draft.id == reviewed.draft_id && draft.status == "dispatching"));
    child.kill().unwrap();
    child.wait().unwrap();
    assert!(list_flow_drafts_with_routed_recovery(&catalog)
        .unwrap()
        .iter()
        .any(|draft| draft.id == reviewed.draft_id && draft.status == "failed"));
    assert!(!reconcile_routed_flow_startup(&catalog, &reviewed.draft_id).unwrap());
    let store = PytxoStore::open(&store_path).unwrap();
    assert!(store.get_run(&run_id).unwrap().is_none());
    assert!(catalog.delete_flow_draft(&reviewed.draft_id).is_err());
}

#[test]
fn dead_owner_with_stopped_prestart_claim_is_cancelled() {
    let (_repo, catalog, reviewed, run_id, store_path, mut child, _owner) =
        claimed_flow_with_child();
    assert!(
        request_stop_experimental_routed_flow(&catalog, &reviewed.draft_id, &run_id)
            .unwrap()
            .is_none()
    );
    child.kill().unwrap();
    child.wait().unwrap();
    assert!(list_flow_drafts_with_routed_recovery(&catalog)
        .unwrap()
        .iter()
        .any(|draft| draft.id == reviewed.draft_id && draft.status == "cancelled"));
    assert!(PytxoStore::open(&store_path)
        .unwrap()
        .get_run(&run_id)
        .unwrap()
        .is_none());
}

#[test]
#[cfg(windows)]
fn routed_stop_ack_waits_for_original_native_launch_gate() {
    let (repo, catalog, reviewed, run_id, _store_path, mut child, _owner) =
        claimed_flow_with_child();
    let lock_path = repo.path().join(".pytxo/data/active_run.json.lock");
    let lock = fs::OpenOptions::new()
        .create(true)
        .truncate(false)
        .read(true)
        .write(true)
        .open(lock_path)
        .unwrap();
    fs2::FileExt::lock_exclusive(&lock).unwrap();
    let (started_tx, started_rx) = std::sync::mpsc::channel();
    let (done_tx, done_rx) = std::sync::mpsc::channel();
    let draft_id = reviewed.draft_id.clone();
    let thread_run_id = run_id.clone();
    let catalog_path = repo.path().join(".git/catalog.db");
    let stop = std::thread::spawn(move || {
        let catalog = Catalog::open(&catalog_path).unwrap();
        started_tx.send(()).unwrap();
        done_tx
            .send(request_stop_experimental_routed_flow(
                &catalog,
                &draft_id,
                &thread_run_id,
            ))
            .unwrap();
    });
    started_rx
        .recv_timeout(std::time::Duration::from_secs(5))
        .unwrap();
    assert!(matches!(
        done_rx.recv_timeout(std::time::Duration::from_millis(200)),
        Err(std::sync::mpsc::RecvTimeoutError::Timeout)
    ));
    assert_eq!(
        catalog
            .routed_flow_stop_requested(&reviewed.draft_id, &run_id)
            .unwrap(),
        Some(false)
    );
    fs2::FileExt::unlock(&lock).unwrap();
    assert!(done_rx
        .recv_timeout(std::time::Duration::from_secs(5))
        .unwrap()
        .unwrap()
        .is_none());
    stop.join().unwrap();
    assert_eq!(
        catalog
            .routed_flow_stop_requested(&reviewed.draft_id, &run_id)
            .unwrap(),
        Some(true)
    );
    child.kill().unwrap();
    child.wait().unwrap();
}

#[test]
#[cfg(all(windows, feature = "routed-test-faults"))]
fn routed_stop_rechecks_dispatch_that_wins_after_ready_snapshot() {
    let repo = repo();
    let catalog = Catalog::open(&repo.path().join(".git/catalog.db")).unwrap();
    let plan = preview_experimental_routed_flow(&catalog, input(repo.path()), mission).unwrap();
    let reviewed = save_reviewed_flow_plan(&catalog, plan).unwrap();
    let run_id = staged_ref(&reviewed).run_id.0;
    let expected = catalog
        .get_flow_draft(&reviewed.draft_id)
        .unwrap()
        .unwrap()
        .plan_json
        .unwrap();
    let mut child = Command::new(std::env::current_exe().unwrap())
        .args(["--exact", "routed_flow_claim_owner_probe"])
        .env("PYTXO_TEST_CLAIM_PROBE", "1")
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::null())
        .spawn()
        .unwrap();
    let identity = pytxo_runner::process_start_identity(child.id())
        .unwrap()
        .unwrap();
    let store_path = PytxoConfig::default().db_path_at(repo.path());
    drop(PytxoStore::open(&store_path).unwrap());
    let store_path = fs::canonicalize(store_path).unwrap();
    let owner = RoutedFlowDispatchOwner {
        controller_pid: child.id(),
        controller_start_identity: identity,
        store_db_path: store_path.to_str().unwrap().into(),
        store_db_file_identity: Some(pytxo_runner::file_identity(&store_path).unwrap()),
    };
    let active_path = repo.path().join(".pytxo/data/active_run.json");
    let target = pytxo_orchestrate::flow::request_stop_experimental_routed_flow_with_test_race(
        &catalog,
        &reviewed.draft_id,
        &run_id,
        || {
            assert!(catalog
                .claim_routed_flow_dispatch(&reviewed.draft_id, &expected, &run_id, &owner)
                .unwrap());
            fs::create_dir_all(active_path.parent().unwrap()).unwrap();
            fs::write(
                &active_path,
                serde_json::to_vec(&serde_json::json!({
                    "run_id": run_id,
                    "repo_root": reviewed.domain_id,
                    "supervisor_pid": owner.controller_pid,
                    "supervisor_start_identity": owner.controller_start_identity
                }))
                .unwrap(),
            )
            .unwrap();
        },
    )
    .unwrap();
    assert!(target.is_some(), "Stop must return the newly claimed owner");
    assert_eq!(
        catalog
            .routed_flow_stop_requested(&reviewed.draft_id, &run_id)
            .unwrap(),
        Some(true)
    );
    child.kill().unwrap();
    child.wait().unwrap();
}

#[test]
#[cfg(windows)]
fn dead_owner_after_may_start_bit_settles_when_original_store_has_no_run() {
    let (_repo, catalog, reviewed, run_id, store_path, mut child, owner) =
        claimed_flow_with_child();
    assert!(catalog
        .mark_routed_flow_startup_may_have_started(&reviewed.draft_id, &run_id, &owner)
        .unwrap());
    child.kill().unwrap();
    child.wait().unwrap();
    assert!(list_flow_drafts_with_routed_recovery(&catalog)
        .unwrap()
        .iter()
        .any(|draft| draft.id == reviewed.draft_id && draft.status == "failed"));
    assert!(!reconcile_routed_flow_startup(&catalog, &reviewed.draft_id).unwrap());
    let store = PytxoStore::open(&store_path).unwrap();
    assert!(store.get_run(&run_id).unwrap().is_none());
    assert!(catalog.delete_flow_draft(&reviewed.draft_id).is_err());
}

#[test]
#[cfg(windows)]
fn dead_owner_with_markerless_starting_run_settles_exact_original_store() {
    let (repo, catalog, reviewed, run_id, store_path, mut child, owner) = claimed_flow_with_child();
    assert!(catalog
        .mark_routed_flow_startup_may_have_started(&reviewed.draft_id, &run_id, &owner)
        .unwrap());
    let store = PytxoStore::open(&store_path).unwrap();
    store
        .insert_starting_run_with_profile(&run_id, &reviewed.domain_id, Some("orbit"))
        .unwrap();
    drop(store);
    assert!(!repo.path().join(".pytxo/data/active_run.json").exists());
    child.kill().unwrap();
    child.wait().unwrap();
    assert!(list_flow_drafts_with_routed_recovery(&catalog)
        .unwrap()
        .iter()
        .any(|draft| draft.id == reviewed.draft_id && draft.status == "failed"));
    let store = PytxoStore::open(&store_path).unwrap();
    assert_eq!(
        store.get_run(&run_id).unwrap().unwrap().status,
        "failed_startup"
    );
}

#[test]
#[cfg(windows)]
fn stopped_recovery_uses_claimed_store_after_config_data_dir_changes() {
    let (repo, catalog, reviewed, run_id, store_path, mut child, owner) = claimed_flow_with_child();
    assert!(catalog
        .mark_routed_flow_startup_may_have_started(&reviewed.draft_id, &run_id, &owner)
        .unwrap());
    let store = PytxoStore::open(&store_path).unwrap();
    store
        .insert_starting_run_with_profile(&run_id, &reviewed.domain_id, Some("orbit"))
        .unwrap();
    drop(store);
    assert!(
        request_stop_experimental_routed_flow(&catalog, &reviewed.draft_id, &run_id)
            .unwrap()
            .is_none()
    );
    fs::write(
        repo.path().join("pytxo.toml"),
        "data_dir = '.pytxo/changed-data'\n",
    )
    .unwrap();
    child.kill().unwrap();
    child.wait().unwrap();
    assert!(list_flow_drafts_with_routed_recovery(&catalog)
        .unwrap()
        .iter()
        .any(|draft| draft.id == reviewed.draft_id && draft.status == "cancelled"));
    assert_eq!(
        PytxoStore::open(&store_path)
            .unwrap()
            .get_run(&run_id)
            .unwrap()
            .unwrap()
            .status,
        "cancelled"
    );
    assert!(!repo.path().join(".pytxo/changed-data/pytxo.db").exists());
}

#[test]
fn dead_owner_registered_without_attempt_cancels_mission_before_run_settlement() {
    let (_repo, catalog, reviewed, run_id, store_path, mut child, owner) =
        claimed_flow_with_child();
    assert!(catalog
        .mark_routed_flow_startup_may_have_started(&reviewed.draft_id, &run_id, &owner)
        .unwrap());
    let store = PytxoStore::open(&store_path).unwrap();
    store
        .insert_starting_run_with_profile(&run_id, &reviewed.domain_id, Some("orbit"))
        .unwrap();
    let registration = mission(
        &reviewed,
        &RunId(run_id.clone()),
        &staged_ref(&reviewed).plan_digest,
    )
    .unwrap();
    store.register_routing_mission(&registration).unwrap();
    drop(store);
    child.kill().unwrap();
    child.wait().unwrap();

    assert!(list_flow_drafts_with_routed_recovery(&catalog)
        .unwrap()
        .iter()
        .any(|draft| draft.id == reviewed.draft_id && draft.status == "failed"));
    let store = PytxoStore::open(&store_path).unwrap();
    assert_eq!(
        store.get_run(&run_id).unwrap().unwrap().status,
        "failed_startup"
    );
    assert!(store
        .routing_history(&RoutingScope {
            domain_id: DomainId(reviewed.domain_id),
            run_id: RunId(run_id),
        })
        .unwrap()
        .is_some_and(|history| history.cancelled
            && history.attempts.is_empty()
            && history.events.len() == 2
            && history.events[1].event_id == "controller.recovery.registered-no-attempt.v1"));
}

#[test]
fn registered_no_attempt_recovery_replays_its_exact_cancellation_only() {
    let (_repo, catalog, reviewed, run_id, store_path, mut child, owner) =
        claimed_flow_with_child();
    assert!(catalog
        .mark_routed_flow_startup_may_have_started(&reviewed.draft_id, &run_id, &owner)
        .unwrap());
    let store = PytxoStore::open(&store_path).unwrap();
    store
        .insert_starting_run_with_profile(&run_id, &reviewed.domain_id, Some("orbit"))
        .unwrap();
    let registration = mission(
        &reviewed,
        &RunId(run_id.clone()),
        &staged_ref(&reviewed).plan_digest,
    )
    .unwrap();
    store.register_routing_mission(&registration).unwrap();
    let scope = RoutingScope {
        domain_id: DomainId(reviewed.domain_id.clone()),
        run_id: RunId(run_id.clone()),
    };
    store
        .cancel_routing_mission(
            &scope,
            "controller.recovery.registered-no-attempt.v1",
            registration.authorization.cancel_epoch,
            123,
        )
        .unwrap();
    drop(store);
    child.kill().unwrap();
    child.wait().unwrap();

    assert!(list_flow_drafts_with_routed_recovery(&catalog)
        .unwrap()
        .iter()
        .any(|draft| draft.id == reviewed.draft_id && draft.status == "failed"));
    let store = PytxoStore::open(&store_path).unwrap();
    assert_eq!(
        store.get_run(&run_id).unwrap().unwrap().status,
        "failed_startup"
    );
    let history = store.routing_history(&scope).unwrap().unwrap();
    assert_eq!(history.events.len(), 2);
    assert!(history.cancelled);
}

#[test]
fn registered_no_attempt_recovery_holds_an_unrelated_cancellation() {
    let (_repo, catalog, reviewed, run_id, store_path, mut child, owner) =
        claimed_flow_with_child();
    assert!(catalog
        .mark_routed_flow_startup_may_have_started(&reviewed.draft_id, &run_id, &owner)
        .unwrap());
    let store = PytxoStore::open(&store_path).unwrap();
    store
        .insert_starting_run_with_profile(&run_id, &reviewed.domain_id, Some("orbit"))
        .unwrap();
    let registration = mission(
        &reviewed,
        &RunId(run_id.clone()),
        &staged_ref(&reviewed).plan_digest,
    )
    .unwrap();
    store.register_routing_mission(&registration).unwrap();
    store
        .cancel_routing_mission(
            &RoutingScope {
                domain_id: DomainId(reviewed.domain_id.clone()),
                run_id: RunId(run_id.clone()),
            },
            "different-controller-cancellation",
            registration.authorization.cancel_epoch,
            123,
        )
        .unwrap();
    drop(store);
    child.kill().unwrap();
    child.wait().unwrap();

    assert!(list_flow_drafts_with_routed_recovery(&catalog)
        .unwrap()
        .iter()
        .any(|draft| draft.id == reviewed.draft_id && draft.status == "recovery_required"));
    assert_eq!(
        PytxoStore::open(&store_path)
            .unwrap()
            .get_run(&run_id)
            .unwrap()
            .unwrap()
            .status,
        "starting"
    );
}

#[test]
fn registered_no_attempt_recovery_holds_a_foreign_active_marker() {
    let (repo, catalog, reviewed, run_id, store_path, mut child, owner) = claimed_flow_with_child();
    assert!(catalog
        .mark_routed_flow_startup_may_have_started(&reviewed.draft_id, &run_id, &owner)
        .unwrap());
    let store = PytxoStore::open(&store_path).unwrap();
    store
        .insert_starting_run_with_profile(&run_id, &reviewed.domain_id, Some("orbit"))
        .unwrap();
    store
        .register_routing_mission(
            &mission(
                &reviewed,
                &RunId(run_id.clone()),
                &staged_ref(&reviewed).plan_digest,
            )
            .unwrap(),
        )
        .unwrap();
    let active_path = repo.path().join(".pytxo/data/active_run.json");
    fs::create_dir_all(active_path.parent().unwrap()).unwrap();
    fs::write(
        &active_path,
        serde_json::to_vec(&serde_json::json!({
            "run_id": run_id,
            "repo_root": reviewed.domain_id,
            "supervisor_pid": owner.controller_pid,
            "supervisor_start_identity": "different-owner"
        }))
        .unwrap(),
    )
    .unwrap();
    drop(store);
    child.kill().unwrap();
    child.wait().unwrap();

    assert!(list_flow_drafts_with_routed_recovery(&catalog)
        .unwrap()
        .iter()
        .any(|draft| draft.id == reviewed.draft_id && draft.status == "recovery_required"));
    assert_eq!(
        PytxoStore::open(&store_path)
            .unwrap()
            .get_run(&run_id)
            .unwrap()
            .unwrap()
            .status,
        "starting"
    );
    assert!(active_path.exists());
}

#[test]
fn terminal_registered_recovery_does_not_clear_a_foreign_same_run_marker() {
    let (repo, catalog, reviewed, run_id, store_path, mut child, owner) = claimed_flow_with_child();
    assert!(catalog
        .mark_routed_flow_startup_may_have_started(&reviewed.draft_id, &run_id, &owner)
        .unwrap());
    let store = PytxoStore::open(&store_path).unwrap();
    store
        .insert_starting_run_with_profile(&run_id, &reviewed.domain_id, Some("orbit"))
        .unwrap();
    let registration = mission(
        &reviewed,
        &RunId(run_id.clone()),
        &staged_ref(&reviewed).plan_digest,
    )
    .unwrap();
    store.register_routing_mission(&registration).unwrap();
    let scope = RoutingScope {
        domain_id: DomainId(reviewed.domain_id.clone()),
        run_id: RunId(run_id.clone()),
    };
    store
        .cancel_routing_mission(
            &scope,
            "controller.recovery.registered-no-attempt.v1",
            registration.authorization.cancel_epoch,
            123,
        )
        .unwrap();
    assert!(store
        .finish_run_if_status(&run_id, "starting", "failed_startup")
        .unwrap());
    let active_path = repo.path().join(".pytxo/data/active_run.json");
    fs::create_dir_all(active_path.parent().unwrap()).unwrap();
    fs::write(
        &active_path,
        serde_json::to_vec(&serde_json::json!({
            "run_id": run_id,
            "repo_root": reviewed.domain_id,
            "supervisor_pid": owner.controller_pid,
            "supervisor_start_identity": "different-owner"
        }))
        .unwrap(),
    )
    .unwrap();
    drop(store);
    child.kill().unwrap();
    child.wait().unwrap();

    let drafts = list_flow_drafts_with_routed_recovery(&catalog).unwrap();
    assert!(
        drafts
            .iter()
            .any(|draft| draft.id == reviewed.draft_id && draft.status == "recovery_required"),
        "terminal replay drafts: {:?}",
        drafts
            .iter()
            .map(|draft| (&draft.id, &draft.status))
            .collect::<Vec<_>>()
    );
    assert!(active_path.exists());
    let store = PytxoStore::open(&store_path).unwrap();
    assert_eq!(
        store.get_run(&run_id).unwrap().unwrap().status,
        "failed_startup"
    );
    assert!(store.routing_history(&scope).unwrap().unwrap().cancelled);
}

#[test]
fn replaced_store_at_same_path_cannot_settle_a_post_start_claim() {
    let (repo, catalog, reviewed, run_id, store_path, mut child, owner) = claimed_flow_with_child();
    assert!(catalog
        .mark_routed_flow_startup_may_have_started(&reviewed.draft_id, &run_id, &owner)
        .unwrap());
    child.kill().unwrap();
    child.wait().unwrap();
    assert!(catalog
        .mark_routed_flow_owner_lost(&reviewed.draft_id, &run_id, &owner)
        .unwrap());

    let replacement_path = repo.path().join("replacement-store.db");
    let replacement = PytxoStore::open(&replacement_path).unwrap();
    replacement
        .insert_starting_run_with_profile(&run_id, &reviewed.domain_id, Some("orbit"))
        .unwrap();
    assert!(replacement
        .finish_run_if_status(&run_id, "starting", "failed_startup")
        .unwrap());
    drop(replacement);
    fs::rename(&store_path, repo.path().join("original-store.db")).unwrap();
    fs::rename(&replacement_path, &store_path).unwrap();

    assert!(!reconcile_routed_flow_startup(&catalog, &reviewed.draft_id).unwrap());
    assert_eq!(
        catalog
            .get_flow_draft(&reviewed.draft_id)
            .unwrap()
            .unwrap()
            .status,
        "recovery_required"
    );
}

#[test]
fn historical_claim_without_physical_store_identity_remains_held_after_start_bit() {
    let (_repo, catalog, reviewed, run_id, _store_path, mut child, owner) =
        claimed_flow_with_child();
    assert!(catalog
        .mark_routed_flow_startup_may_have_started(&reviewed.draft_id, &run_id, &owner)
        .unwrap());
    rusqlite::Connection::open(catalog.opened_path())
        .unwrap()
        .execute(
            "UPDATE flow_dispatch_claims SET store_db_file_identity=NULL WHERE draft_id=?1",
            [&reviewed.draft_id],
        )
        .unwrap();
    child.kill().unwrap();
    child.wait().unwrap();
    assert!(list_flow_drafts_with_routed_recovery(&catalog)
        .unwrap()
        .iter()
        .any(|draft| draft.id == reviewed.draft_id && draft.status == "recovery_required"));
    assert!(!reconcile_routed_flow_startup(&catalog, &reviewed.draft_id).unwrap());
}

#[test]
fn exact_dead_owner_marker_after_start_bit_can_settle_unregistered_run() {
    let (repo, catalog, reviewed, run_id, store_path, mut child, owner) = claimed_flow_with_child();
    assert!(catalog
        .mark_routed_flow_startup_may_have_started(&reviewed.draft_id, &run_id, &owner)
        .unwrap());
    let store = PytxoStore::open(&store_path).unwrap();
    store
        .insert_starting_run_with_profile(&run_id, &reviewed.domain_id, Some("orbit"))
        .unwrap();
    let active_path = repo.path().join(".pytxo/data/active_run.json");
    fs::create_dir_all(active_path.parent().unwrap()).unwrap();
    fs::write(
        &active_path,
        serde_json::to_vec(&serde_json::json!({
            "run_id": run_id,
            "repo_root": reviewed.domain_id,
            "supervisor_pid": owner.controller_pid,
            "supervisor_start_identity": owner.controller_start_identity
        }))
        .unwrap(),
    )
    .unwrap();
    child.kill().unwrap();
    child.wait().unwrap();
    assert!(list_flow_drafts_with_routed_recovery(&catalog)
        .unwrap()
        .iter()
        .any(|draft| draft.id == reviewed.draft_id && draft.status == "failed"));
    assert_eq!(
        store.get_run(&run_id).unwrap().unwrap().status,
        "failed_startup"
    );
    assert!(!active_path.exists());
    assert!(!reconcile_routed_flow_startup(&catalog, &reviewed.draft_id).unwrap());
}

#[test]
fn foreign_marker_after_start_bit_keeps_recovery_required() {
    let (repo, catalog, reviewed, run_id, _store_path, mut child, owner) =
        claimed_flow_with_child();
    assert!(catalog
        .mark_routed_flow_startup_may_have_started(&reviewed.draft_id, &run_id, &owner)
        .unwrap());
    let active_path = repo.path().join(".pytxo/data/active_run.json");
    fs::create_dir_all(active_path.parent().unwrap()).unwrap();
    fs::write(
        &active_path,
        serde_json::to_vec(&serde_json::json!({
            "run_id": run_id,
            "repo_root": reviewed.domain_id,
            "supervisor_pid": owner.controller_pid,
            "supervisor_start_identity": "different-owner"
        }))
        .unwrap(),
    )
    .unwrap();
    child.kill().unwrap();
    child.wait().unwrap();
    assert!(list_flow_drafts_with_routed_recovery(&catalog)
        .unwrap()
        .iter()
        .any(|draft| draft.id == reviewed.draft_id && draft.status == "recovery_required"));
    assert!(active_path.exists());
}

#[test]
fn ownerless_legacy_linked_flow_remains_unresolved() {
    let repo = repo();
    let catalog_path = repo.path().join(".git/catalog.db");
    let catalog = Catalog::open(&catalog_path).unwrap();
    let plan = preview_experimental_routed_flow(&catalog, input(repo.path()), mission).unwrap();
    let run_id = staged_ref(&plan).run_id.0;
    rusqlite::Connection::open(&catalog_path)
        .unwrap()
        .execute(
            "UPDATE flow_drafts SET status='dispatching', dispatched_run_id=?1 WHERE id=?2",
            rusqlite::params![run_id, plan.draft_id],
        )
        .unwrap();
    assert!(list_flow_drafts_with_routed_recovery(&catalog)
        .unwrap()
        .iter()
        .any(|draft| draft.id == plan.draft_id && draft.status == "dispatching"));
    assert!(catalog
        .routed_flow_dispatch_owner(&plan.draft_id, &run_id)
        .unwrap()
        .is_none());
}

fn input(repo: &std::path::Path) -> FlowDraftInput {
    FlowDraftInput {
        id: "routed-flow".into(),
        title: "Routed fixture".into(),
        mission_text: "Update src/lib.rs".into(),
        source: FlowSource::Text,
        domain_id: Some(repo.to_string_lossy().into_owned()),
        project_id: None,
        ade_id: None,
        max_workers: Some(1),
        verification_commands: vec!["echo verification-ok".into()],
    }
}

fn git_oid(repo: &std::path::Path, revision: &str) -> String {
    let output = Command::new("git")
        .args(["rev-parse", "--verify", revision])
        .current_dir(repo)
        .output()
        .unwrap();
    assert!(output.status.success());
    String::from_utf8(output.stdout).unwrap().trim().into()
}

fn actual_base(plan: &FlowPlan) -> BaseSnapshot {
    let repo = std::path::Path::new(&plan.domain_id);
    let tree = git_oid(repo, "HEAD^{tree}");
    BaseSnapshot {
        repository_identity: plan.domain_id.clone(),
        git_revision: git_oid(repo, "HEAD^{commit}"),
        snapshot_digest: canonical_digest(&(1_u32, "git-head-tree", tree), 1).unwrap(),
    }
}

fn profile(name: &str, backend: ExecutionBackend) -> RegisteredProfile {
    let execution = ExecutionProfile {
        schema_version: 1,
        canonicalization_version: 1,
        id: ProfileId(name.into()),
        revision: 1,
        harness_id: "fixture".into(),
        adapter_contract_version: "1".into(),
        adapter_digest: Digest::of_bytes(b"adapter"),
        requested_model: ModelIdentity {
            provider: "local".into(),
            model: name.into(),
            reasoning: None,
            revision: None,
        },
        skill_tool_bundle_digest: Digest::of_bytes(b"tools"),
        backend,
        capabilities: BTreeSet::from(["edit".into()]),
    };
    let binding = ProfileBinding {
        schema_version: 1,
        canonicalization_version: 1,
        id: BindingId(name.into()),
        revision: 1,
        profile_digest: execution.digest().unwrap(),
        credential_reference: Some("keychain:private-fixture-reference".into()),
        auth_owner: "private-fixture-owner".into(),
        billing_source_id: BillingSourceId("local".into()),
        billing_mode: BillingSourceMode::Local,
        endpoint_identity: "fixture".into(),
        trust_class: "local".into(),
        capacity_pool_ids: BTreeSet::from(["host".into()]),
    };
    RegisteredProfile {
        profile: execution,
        binding,
    }
}

fn mission(
    plan: &FlowPlan,
    run_id: &RunId,
    plan_digest: &Digest,
) -> anyhow::Result<RoutingMission> {
    let backend = ExecutionBackend::parse(&plan.execution_backend).unwrap();
    let profiles = vec![profile("everyday", backend), profile("strong", backend)];
    let targets: Vec<_> = profiles
        .iter()
        .map(|entry| RouteTarget {
            profile_id: entry.profile.id.clone(),
            binding_id: entry.binding.id.clone(),
        })
        .collect();
    let policy = RoutingPolicy {
        schema_version: 1,
        version: "rules-fixture".into(),
        mode: RoutingMode::Rules,
        everyday: targets[0].clone(),
        strong: targets[1].clone(),
        evaluated_manifest_digest: None,
        everyday_threshold_ppm: 800_000,
        unclear_ceiling_ppm: 100_000,
        advice_model: "unused".into(),
        advice_template: "unused".into(),
        disclosure_scope_digest: None,
        advisor_recipient: None,
    };
    let tasks = plan
        .tasks
        .iter()
        .map(|task| {
            let check_recipes =
                pytxo_orchestrate::flow::freeze_experimental_routed_checks(&task.id, &task.verify)?;
            let contract = TaskContract {
                schema_version: 1,
                canonicalization_version: 1,
                task_id: TaskId(task.id.clone()),
                revision: 1,
                plan_digest: plan_digest.clone(),
                base: actual_base(plan),
                goal: task.prompt.clone(),
                constraints: vec![],
                claim_roots: task.paths.clone(),
                dependencies: task.dependencies.iter().cloned().map(TaskId).collect(),
                task_kind: Some(TaskKind::LocalTransformation),
                task_kind_evidence: Some(Digest::of_bytes(b"kind")),
                required_capabilities: BTreeSet::from(["edit".into()]),
                checks: check_recipes
                    .iter()
                    .map(FrozenCheckRecipeV1::reference)
                    .collect::<pytxo_core::Result<Vec<_>>>()?,
                required_resources: BTreeSet::from(["host".into()]),
                skill_tool_bundle_digest: Digest::of_bytes(b"tools"),
                permission_profile: PermissionProfile::Orbit,
                required_egress: BTreeSet::new(),
                required_target: None,
                strong_only: false,
                cross_component_requirement: Some(false),
                context_complete: true,
                repeatable_symptom_supplied: None,
                specific_cause_hypothesis_supplied: None,
            };
            Ok(RegisteredTask {
                contract,
                attempt_budget_nano_usd: 0,
                check_recipes,
            })
        })
        .collect::<anyhow::Result<Vec<_>>>()?;
    let authorization = MissionAuthorization {
        schema_version: 1,
        domain_id: DomainId(plan.domain_id.clone()),
        run_id: run_id.clone(),
        plan_id: PlanId("fixture-plan".into()),
        plan_digest: plan_digest.clone(),
        revision: 1,
        cancel_epoch: 0,
        allowed_task_digests: tasks
            .iter()
            .map(|task| task.contract.digest())
            .collect::<std::result::Result<BTreeSet<_>, _>>()?,
        allowed_profiles: profiles
            .iter()
            .zip(targets)
            .map(|(entry, target)| ApprovedProfile {
                target,
                profile_digest: entry.profile.digest().unwrap(),
                binding_digest: entry.binding.digest().unwrap(),
            })
            .collect(),
        allowed_billing_sources: BTreeSet::from([BillingSourceId("local".into())]),
        allowed_billing_modes: BTreeSet::from([BillingSourceMode::Local]),
        permission_profile: PermissionProfile::Orbit,
        allowed_egress: BTreeSet::new(),
        minimum_model_identity: ModelIdentityLevel::HarnessReported,
        limits: MissionLimits {
            deadline_ms: u64::MAX,
            max_workers: u32::try_from(plan.max_workers)?,
            max_attempts: 2,
            max_spend_nano_usd: None,
            spend_guarantee: SpendGuarantee::RiskBounded,
        },
        policy_digest: policy.digest()?,
        live_advice_authorized: false,
        consent_revision: 1,
    };
    Ok(RoutingMission {
        authorization,
        policy,
        tasks,
        profiles,
    })
}

fn reviewed_claude_subscription_pair(
    plan: &FlowPlan,
    run_id: &RunId,
    plan_digest: &Digest,
) -> anyhow::Result<RoutingMission> {
    const ENDPOINT: &str = "claude-cli:claude-ai-subscription";
    let selected_home = std::env::var_os("PYTXO_ROUTED_CLAUDE_ACCOUNT_HOME")
        .map(std::path::PathBuf::from)
        .unwrap_or_else(std::env::temp_dir);
    let source_id =
        pytxo_orchestrate::routed_claude::claude_subscription_account_source_id(&selected_home)?;
    let mut reviewed = mission(plan, run_id, plan_digest)?;
    let task = &mut reviewed.tasks[0].contract;
    task.required_resources = BTreeSet::from(["account-claude".into()]);
    task.required_egress = BTreeSet::from([ENDPOINT.into()]);
    task.skill_tool_bundle_digest = Digest::of_bytes(b"claude-no-tools-one-file-json/3");
    for (entry, model) in reviewed.profiles.iter_mut().zip(["haiku", "sonnet"]) {
        entry.profile.harness_id = "claude".into();
        entry.profile.adapter_digest = Digest::of_bytes(b"pytxo-claude-one-file-proposal/3");
        entry.profile.skill_tool_bundle_digest = task.skill_tool_bundle_digest.clone();
        entry.profile.requested_model.provider = "anthropic".into();
        entry.profile.requested_model.model = model.into();
        entry.profile.capabilities = BTreeSet::from(["read".into(), "edit".into()]);
        entry.binding.profile_digest = entry.profile.digest()?;
        entry.binding.credential_reference = None;
        entry.binding.auth_owner = "Claude".into();
        entry.binding.billing_source_id = source_id.clone();
        entry.binding.billing_mode = BillingSourceMode::Subscription;
        entry.binding.endpoint_identity = ENDPOINT.into();
        entry.binding.trust_class = "vendor".into();
        entry.binding.capacity_pool_ids = task.required_resources.clone();
    }
    reviewed.authorization.allowed_task_digests = BTreeSet::from([task.digest()?]);
    reviewed.authorization.allowed_profiles = reviewed
        .profiles
        .iter()
        .map(|entry| {
            Ok(ApprovedProfile {
                target: RouteTarget {
                    profile_id: entry.profile.id.clone(),
                    binding_id: entry.binding.id.clone(),
                },
                profile_digest: entry.profile.digest()?,
                binding_digest: entry.binding.digest()?,
            })
        })
        .collect::<anyhow::Result<Vec<_>>>()?;
    reviewed.authorization.allowed_billing_sources = BTreeSet::from([source_id]);
    reviewed.authorization.allowed_billing_modes =
        BTreeSet::from([BillingSourceMode::Subscription]);
    reviewed.authorization.allowed_egress = task.required_egress.clone();
    reviewed.authorization.minimum_model_identity = ModelIdentityLevel::Requested;
    reviewed.authorization.limits.max_attempts = 1;
    Ok(reviewed)
}

#[test]
fn legacy_custom_claude_pair_is_rejected_before_dispatch() {
    let repo = repo();
    fs::write(
        repo.path().join("pytxo.toml"),
        "permission_profile = 'orbit'\nexecution_backend = 'subprocess'\nmax_agents = 1\n[blast]\nprefer_kernel_overlay = false\n",
    )
    .unwrap();
    assert!(Command::new("git")
        .args(["add", "pytxo.toml"])
        .current_dir(repo.path())
        .status()
        .unwrap()
        .success());
    assert!(Command::new("git")
        .args([
            "-c",
            "user.name=Test",
            "-c",
            "user.email=test@pytxo.local",
            "commit",
            "-qm",
            "routed policy",
        ])
        .current_dir(repo.path())
        .status()
        .unwrap()
        .success());
    let catalog = Catalog::open(&repo.path().join(".git/catalog.db")).unwrap();
    let error = preview_experimental_routed_flow(
        &catalog,
        input(repo.path()),
        reviewed_claude_subscription_pair,
    )
    .unwrap_err();
    assert!(
        error
            .to_string()
            .contains("Claude route exceeds the one-task subscription experiment"),
        "{error}"
    );
}

#[test]
#[cfg(all(windows, feature = "routed-test-faults"))]
#[ignore = "explicit subscription and current-source embedded-host beta journey"]
fn native_claude_proposal_reaches_review_and_apply() {
    let _stub = if std::env::var_os("PYTXO_TEST_ROUTED_CLAUDE_STUB").is_some() {
        let root = tempfile::tempdir().unwrap();
        let tools = root.path().join("tools");
        let account = root.path().join("account");
        let probes = root.path().join("probes");
        fs::create_dir_all(&tools).unwrap();
        fs::create_dir_all(&account).unwrap();
        fs::create_dir_all(&probes).unwrap();
        let cli = tools.join("claude.exe");
        fs::copy(env!("CARGO_BIN_EXE_pytxo-claude-routing-fixture"), &cli).unwrap();
        unsafe {
            std::env::set_var("PYTXO_EXPERIMENTAL_ROUTED_CLAUDE_PROPOSAL", "1");
            std::env::set_var("PYTXO_ROUTED_CLAUDE_EXE", &cli);
            std::env::set_var("PYTXO_ROUTED_CLAUDE_ACCOUNT_HOME", &account);
            std::env::set_var("PYTXO_ROUTED_CLAUDE_PROBE_ROOT", &probes);
        }
        Some(root)
    } else {
        None
    };
    assert_eq!(
        std::env::var("PYTXO_EXPERIMENTAL_ROUTED_CLAUDE_PROPOSAL").as_deref(),
        Ok("1")
    );
    assert!(std::env::var_os("PYTXO_ROUTED_CLAUDE_EXE").is_some());
    assert!(std::env::var_os("PYTXO_ROUTED_CLAUDE_ACCOUNT_HOME").is_some());
    assert!(std::env::var_os("PYTXO_ROUTED_CLAUDE_PROBE_ROOT").is_some());
    assert!(std::env::var_os("PYTXO_TEST_EMBEDDED_HOST").is_some());
    let repo = repo();
    fs::write(
        repo.path().join("pytxo.toml"),
        "permission_profile = 'orbit'\nexecution_backend = 'subprocess'\nmax_agents = 1\n[blast]\nprefer_kernel_overlay = false\n",
    )
    .unwrap();
    assert!(Command::new("git")
        .args(["add", "pytxo.toml"])
        .current_dir(repo.path())
        .status()
        .unwrap()
        .success());
    assert!(Command::new("git")
        .args([
            "-c",
            "user.name=Test",
            "-c",
            "user.email=test@pytxo.local",
            "commit",
            "-qm",
            "routed policy",
        ])
        .current_dir(repo.path())
        .status()
        .unwrap()
        .success());
    let catalog = Catalog::open(&repo.path().join(".git/catalog.db")).unwrap();
    catalog
        .configure_capacity_pool(&CapacityPoolConfig {
            resource_id: "account-claude".into(),
            capacity_units: 1,
            expected_revision: None,
            configured_at_ms: 100,
        })
        .unwrap();
    let mut draft = input(repo.path());
    draft.mission_text =
        "In src/lib.rs replace the file with exactly: pub fn value() -> i32 { 42 }".into();
    draft.verification_commands =
        vec![r"C:\Windows\System32\findstr.exe /C:42 src\lib.rs >NUL".into()];
    let selected_home =
        std::path::PathBuf::from(std::env::var_os("PYTXO_ROUTED_CLAUDE_ACCOUNT_HOME").unwrap());
    let preview = preview_experimental_claude_proposal_flow_with_facts(
        &catalog,
        draft,
        &selected_home,
        "account-claude",
        ReviewedDemandFacts {
            task_kind: TaskKind::LocalTransformation,
            context_complete: true,
            cross_component_requirement: Some(false),
            repeatable_symptom_supplied: None,
            specific_cause_hypothesis_supplied: None,
        },
    )
    .unwrap();
    let reviewed = save_reviewed_flow_plan(&catalog, preview).unwrap();
    let run_id = dispatch_experimental_routed_flow(&catalog, &reviewed.draft_id).unwrap();
    let store = PytxoStore::open(&PytxoConfig::default().db_path_at(repo.path())).unwrap();
    let status = store.get_run_status(&run_id).unwrap().unwrap().0;
    let scope = RoutingScope {
        domain_id: DomainId(reviewed.domain_id.clone()),
        run_id: RunId(run_id.clone()),
    };
    let history = store.routing_history(&scope).unwrap().unwrap();
    if status != "completed" {
        let attempt = history.attempts.first();
        eprintln!(
            "Claude routed failure: run={status}, attempt={:?}, failure={:?}, sealed={}, checks={}, owner={:?}, review={:?}",
            attempt.map(|row| row.state),
            attempt.and_then(|row| row.failure.as_ref()).map(|failure| failure.failure_class),
            attempt.is_some_and(|row| row.receipts.sealed_output.is_some()),
            attempt.is_some_and(|row| row.receipts.checks.is_some()),
            attempt.and_then(|row| store.launch_ownership(&row.attempt_id).ok().flatten()).map(|owner| owner.phase),
            store.get_run_contract(&run_id).ok().flatten().map(|contract| contract.apply_status),
        );
    }
    assert_eq!(status, "completed");
    assert_eq!(history.attempts.len(), 1);
    assert_eq!(history.attempts[0].state, AttemptState::Passed);
    assert_eq!(
        history.attempts[0].selected.profile.id,
        history.mission.policy.everyday.profile_id
    );
    assert!(history.attempts[0].receipts.process_identity.is_some());
    assert!(history.attempts[0].receipts.quiescence.is_some());
    assert!(history.attempts[0].receipts.sealed_output.is_some());
    assert!(history.attempts[0].receipts.checks.is_some());
    assert!(matches!(
        history.attempts[0].usage,
        RoutedUsage::Unknown { .. }
    ));
    let contract = store.get_run_contract(&run_id).unwrap().unwrap();
    let manifest = contract.prepared_manifest.unwrap();
    assert_eq!(manifest.files.len(), 1);
    assert_eq!(manifest.files[0].path, "src/lib.rs");
    assert!(manifest.candidate_verification.is_some());
    let digest = manifest.package_digest;
    assert_eq!(
        fs::read(repo.path().join("src/lib.rs")).unwrap(),
        b"pub fn value() {}\n"
    );
    apply_run_changes(None, Some(repo.path().to_path_buf()), &run_id, &digest).unwrap();
    assert_eq!(
        fs::read(repo.path().join("src/lib.rs")).unwrap(),
        b"pub fn value() -> i32 { 42 }\n"
    );
    assert_eq!(
        store
            .get_run_contract(&run_id)
            .unwrap()
            .unwrap()
            .apply_status,
        "applied"
    );
}

#[test]
#[cfg(all(windows, feature = "routed-test-faults"))]
#[ignore = "explicit local Claude-shaped stub and embedded-host repair journey"]
fn native_claude_failed_check_restarts_clean_once_on_strong() {
    run_native_claude_check_repair(false, false, false, false, false, false, false);
}

#[test]
#[cfg(all(windows, feature = "routed-test-faults"))]
#[ignore = "explicit local Claude-shaped stub and private checker receipt journey"]
fn native_claude_strong_clean_repair_consumes_owned_check_receipt_without_output() {
    run_native_claude_check_repair(false, false, false, false, false, false, true);
}

#[test]
#[cfg(all(windows, feature = "routed-test-faults"))]
#[ignore = "explicit local Claude-shaped stub and embedded-host Stop journey"]
fn native_claude_stop_after_failed_check_prevents_second_launch() {
    run_native_claude_check_repair(true, false, false, false, false, false, false);
}

#[test]
#[cfg(all(windows, feature = "routed-test-faults"))]
#[ignore = "explicit local Claude-shaped stub and Catalog Stop race journey"]
fn native_claude_catalog_stop_before_store_cancel_prevents_second_launch() {
    run_native_claude_check_repair(false, true, false, false, false, false, false);
}

#[test]
#[cfg(all(windows, feature = "routed-test-faults"))]
#[ignore = "explicit local Claude-shaped stub and checker integrity journey"]
fn native_claude_mutating_checker_does_not_trigger_paid_repair() {
    run_native_claude_check_repair(false, false, true, false, false, false, false);
}

#[test]
#[cfg(all(windows, feature = "routed-test-faults"))]
#[ignore = "explicit local Claude-shaped stub and dead-controller repair recovery"]
fn native_claude_prepared_second_attempt_recovers_without_relaunch() {
    run_native_claude_check_repair(false, false, false, true, false, false, false);
}

#[test]
#[cfg(all(windows, feature = "routed-test-faults"))]
#[ignore = "explicit local Claude-shaped stub and Stop/native-create race"]
fn native_claude_catalog_stop_before_strong_create_prevents_launch() {
    run_native_claude_check_repair(false, false, false, false, true, false, false);
}

#[test]
#[cfg(all(windows, feature = "routed-test-faults"))]
#[ignore = "explicit local Claude-shaped stub and Stop/checker-create race"]
fn native_claude_catalog_stop_before_checker_create_prevents_launch() {
    run_native_claude_check_repair(false, false, false, false, false, true, false);
}

#[cfg(all(windows, feature = "routed-test-faults"))]
fn run_native_claude_check_repair(
    stop_after_first: bool,
    catalog_stop_only: bool,
    mutate_checker: bool,
    crash_second: bool,
    stop_before_strong_create: bool,
    stop_before_checker_create: bool,
    receipt_repair: bool,
) {
    let root = tempfile::tempdir().unwrap();
    let tools = root.path().join("tools");
    let account = root.path().join("account");
    let probes = root.path().join("probes");
    fs::create_dir_all(&tools).unwrap();
    fs::create_dir_all(&account).unwrap();
    fs::create_dir_all(&probes).unwrap();
    let cli = tools.join("claude.exe");
    fs::copy(env!("CARGO_BIN_EXE_pytxo-claude-routing-fixture"), &cli).unwrap();
    unsafe {
        std::env::set_var("PYTXO_EXPERIMENTAL_ROUTED_CLAUDE_PROPOSAL", "1");
        std::env::set_var("PYTXO_EXPERIMENTAL_ROUTED_CLAUDE_REPAIR", "1");
        std::env::set_var("PYTXO_TEST_ROUTED_CLAUDE_SYNTHETIC_QUALIFICATION", "1");
        std::env::set_var("PYTXO_ROUTED_CLAUDE_EXE", &cli);
        std::env::set_var("PYTXO_ROUTED_CLAUDE_ACCOUNT_HOME", &account);
        std::env::set_var("PYTXO_ROUTED_CLAUDE_PROBE_ROOT", &probes);
        if stop_after_first {
            std::env::set_var("PYTXO_TEST_ROUTED_STOP_AFTER_REPAIRABLE_FAILURE", "1");
        } else {
            std::env::remove_var("PYTXO_TEST_ROUTED_STOP_AFTER_REPAIRABLE_FAILURE");
        }
        if catalog_stop_only {
            std::env::set_var(
                "PYTXO_TEST_ROUTED_CATALOG_STOP_AFTER_REPAIRABLE_FAILURE",
                "1",
            );
        } else {
            std::env::remove_var("PYTXO_TEST_ROUTED_CATALOG_STOP_AFTER_REPAIRABLE_FAILURE");
        }
        if crash_second {
            std::env::set_var("PYTXO_TEST_ROUTED_CRASH_AFTER_CLAUDE_REPAIR_ADMITTED", "1");
        } else {
            std::env::remove_var("PYTXO_TEST_ROUTED_CRASH_AFTER_CLAUDE_REPAIR_ADMITTED");
        }
        if stop_before_strong_create {
            std::env::set_var("PYTXO_TEST_ROUTED_CATALOG_STOP_BEFORE_STRONG_CREATE", "1");
        } else {
            std::env::remove_var("PYTXO_TEST_ROUTED_CATALOG_STOP_BEFORE_STRONG_CREATE");
        }
        if stop_before_checker_create {
            std::env::set_var("PYTXO_TEST_ROUTED_CATALOG_STOP_BEFORE_CHECKER_CREATE", "1");
        } else {
            std::env::remove_var("PYTXO_TEST_ROUTED_CATALOG_STOP_BEFORE_CHECKER_CREATE");
        }
    }
    assert!(std::env::var_os("PYTXO_TEST_EMBEDDED_HOST").is_some());
    let repo = repo();
    fs::write(
        repo.path().join("pytxo.toml"),
        "permission_profile = 'orbit'\nexecution_backend = 'subprocess'\nmax_agents = 1\n[blast]\nprefer_kernel_overlay = false\n",
    )
    .unwrap();
    assert!(Command::new("git")
        .args(["add", "pytxo.toml"])
        .current_dir(repo.path())
        .status()
        .unwrap()
        .success());
    assert!(Command::new("git")
        .args([
            "-c",
            "user.name=Test",
            "-c",
            "user.email=test@pytxo.local",
            "commit",
            "-qm",
            "routed repair policy",
        ])
        .current_dir(repo.path())
        .status()
        .unwrap()
        .success());
    let catalog = Catalog::open(&repo.path().join(".git/catalog.db")).unwrap();
    catalog
        .configure_capacity_pool(&CapacityPoolConfig {
            resource_id: "account-claude".into(),
            capacity_units: 1,
            expected_revision: None,
            configured_at_ms: 100,
        })
        .unwrap();
    let mut draft = input(repo.path());
    draft.mission_text = if receipt_repair {
        "pytxo-test-claude-receipt-repair: Fix the value in src/lib.rs so the frozen check passes."
    } else {
        "pytxo-test-claude-repair: In src/lib.rs replace the file with exactly: pub fn value() -> i32 { 42 }"
    }
    .into();
    draft.verification_commands = vec![if mutate_checker {
        r"echo checker-tampered>src\lib.rs & exit /b 1".into()
    } else {
        if receipt_repair {
            r"C:\Windows\System32\findstr.exe /C:42 src\lib.rs >NUL || (echo error: password=hunter2 & echo EXPECTED: 42 & exit /b 1)".into()
        } else {
            r"C:\Windows\System32\findstr.exe /C:42 src\lib.rs >NUL".into()
        }
    }];
    let preview = preview_experimental_claude_proposal_flow_with_facts(
        &catalog,
        draft,
        &account,
        "account-claude",
        ReviewedDemandFacts {
            task_kind: TaskKind::LocalTransformation,
            context_complete: true,
            cross_component_requirement: Some(false),
            repeatable_symptom_supplied: None,
            specific_cause_hypothesis_supplied: None,
        },
    )
    .unwrap();
    assert_eq!(
        preview
            .routing
            .as_ref()
            .unwrap()
            .authorization
            .limits
            .max_attempts,
        2
    );
    let reviewed = save_reviewed_flow_plan(&catalog, preview).unwrap();
    let expected_run_id = reviewed
        .routing
        .as_ref()
        .unwrap()
        .authorization
        .run_id
        .0
        .clone();
    let dispatch = if crash_second {
        assert!(std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            dispatch_experimental_routed_flow(&catalog, &reviewed.draft_id)
        }))
        .is_err());
        catalog
            .mark_routed_flow_recovery_required(&reviewed.draft_id, &expected_run_id)
            .unwrap();
        let owner = catalog
            .routed_flow_dispatch_owner(&reviewed.draft_id, &expected_run_id)
            .unwrap()
            .unwrap();
        rusqlite::Connection::open(repo.path().join(".git/catalog.db"))
            .unwrap()
            .execute(
                "UPDATE flow_dispatch_claims SET controller_start_identity=?1 WHERE draft_id=?2 AND run_id=?3 AND controller_pid=?4",
                rusqlite::params!["simulated-dead-controller", reviewed.draft_id, expected_run_id, owner.controller_pid],
            )
            .unwrap();
        let active_path = repo.path().join(".pytxo/data/active_run.json");
        let mut active: serde_json::Value =
            serde_json::from_slice(&fs::read(&active_path).unwrap()).unwrap();
        active["supervisor_start_identity"] = "simulated-dead-controller".into();
        fs::write(&active_path, serde_json::to_vec(&active).unwrap()).unwrap();
        assert!(reconcile_routed_flow_startup(&catalog, &reviewed.draft_id).unwrap());
        Ok(expected_run_id.clone())
    } else {
        dispatch_experimental_routed_flow(&catalog, &reviewed.draft_id)
    };
    let run_id = if catalog_stop_only || stop_before_strong_create || stop_before_checker_create {
        assert!(
            dispatch.is_err(),
            "Catalog Stop must win terminal acknowledgement"
        );
        expected_run_id
    } else {
        dispatch.unwrap()
    };
    let store = PytxoStore::open(&PytxoConfig::default().db_path_at(repo.path())).unwrap();
    let scope = RoutingScope {
        domain_id: DomainId(reviewed.domain_id.clone()),
        run_id: RunId(run_id.clone()),
    };
    let history = store.routing_history(&scope).unwrap().unwrap();
    if stop_before_checker_create {
        assert_eq!(
            store.get_run_status(&run_id).unwrap().unwrap().0,
            "cancelled"
        );
        assert!(history.cancelled);
        assert_eq!(history.attempts.len(), 1);
        assert_eq!(history.attempts[0].state, AttemptState::Cancelled);
        assert!(history.attempts[0].ownership_released);
        assert!(history.tasks[0].winner.is_none());
        assert!(catalog
            .unresolved_capacity_reservations()
            .unwrap()
            .is_empty());
        assert_eq!(
            fs::read(repo.path().join("src/lib.rs")).unwrap(),
            b"pub fn value() {}\n"
        );
        return;
    }
    if crash_second {
        assert_eq!(store.get_run_status(&run_id).unwrap().unwrap().0, "failed");
        assert!(history.cancelled);
        assert_eq!(history.attempts.len(), 2);
        assert_eq!(history.attempts[0].state, AttemptState::Failed);
        assert_eq!(history.attempts[1].state, AttemptState::FailedNoLaunch);
        assert!(history.attempts[1].ownership_released);
        assert!(history.tasks[0].winner.is_none());
        assert!(catalog
            .unresolved_capacity_reservations()
            .unwrap()
            .is_empty());
        assert_eq!(
            fs::read(repo.path().join("src/lib.rs")).unwrap(),
            b"pub fn value() {}\n"
        );
        return;
    }
    if stop_before_strong_create {
        assert_eq!(
            store.get_run_status(&run_id).unwrap().unwrap().0,
            "cancelled"
        );
        assert!(history.cancelled);
        assert_eq!(history.attempts.len(), 2);
        assert_eq!(history.attempts[0].state, AttemptState::Failed);
        assert_eq!(history.attempts[1].state, AttemptState::FailedNoLaunch);
        assert!(history.attempts[1].ownership_released);
        assert!(history.tasks[0].winner.is_none());
        assert!(catalog
            .unresolved_capacity_reservations()
            .unwrap()
            .is_empty());
        assert_eq!(
            fs::read(repo.path().join("src/lib.rs")).unwrap(),
            b"pub fn value() {}\n"
        );
        return;
    }
    if stop_after_first || catalog_stop_only {
        assert_eq!(
            store.get_run_status(&run_id).unwrap().unwrap().0,
            "cancelled"
        );
        assert!(history.cancelled);
        assert_eq!(history.attempts.len(), 1);
        assert_eq!(history.attempts[0].state, AttemptState::Failed);
        assert!(history.attempts[0].ownership_released);
        assert!(history.tasks[0].winner.is_none());
        assert_eq!(
            fs::read(repo.path().join("src/lib.rs")).unwrap(),
            b"pub fn value() {}\n"
        );
        return;
    }
    if mutate_checker {
        assert_eq!(store.get_run_status(&run_id).unwrap().unwrap().0, "failed");
        assert_eq!(history.attempts.len(), 1);
        assert_eq!(history.attempts[0].state, AttemptState::Failed);
        assert_eq!(
            history.attempts[0].failure.as_ref().unwrap().failure_class,
            AttemptFailureClass::Unknown
        );
        assert!(history.tasks[0].winner.is_none());
        assert_eq!(
            fs::read(repo.path().join("src/lib.rs")).unwrap(),
            b"pub fn value() {}\n"
        );
        return;
    }
    assert_eq!(
        store.get_run_status(&run_id).unwrap().unwrap().0,
        "completed"
    );
    assert_eq!(history.attempts.len(), 2);
    let first = &history.attempts[0];
    let second = &history.attempts[1];
    assert_eq!(first.ordinal, 1);
    assert_eq!(first.state, AttemptState::Failed);
    assert!(first.ownership_released);
    assert_eq!(
        first.selected.profile.id,
        history.mission.policy.everyday.profile_id
    );
    assert_eq!(
        first.failure.as_ref().unwrap().failure_class,
        AttemptFailureClass::Check
    );
    assert!(first
        .failure
        .as_ref()
        .unwrap()
        .actionable_evidence_digest
        .is_some());
    if receipt_repair {
        use pytxo_store::routing_private::{
            ControllerObservation, PrivateArtifactClaim, PrivateArtifactKind,
        };
        let checker = store
            .checker_ownership(&first.attempt_id, 1)
            .unwrap()
            .unwrap();
        let reference = checker.settlement_blob.as_ref().unwrap();
        let claim = PrivateArtifactClaim {
            scope: scope.clone(),
            task_id: first.task_id.clone(),
            attempt_id: first.attempt_id.clone(),
            reservation_id: first.capacity_reservation.clone(),
            kind: PrivateArtifactKind::ControllerReceipt,
            artifact_id: format!("{}:checker-1:result", first.attempt_id.0),
            event_id: format!("{}:checker-1:observed", first.attempt_id.0),
        };
        let receipt = store.read_controller_receipt(&claim, reference).unwrap();
        let ControllerObservation::NativeCheckerResult {
            exit_code: 1,
            native_outcome: pytxo_store::routing_private::CheckerNativeOutcome::Failed,
            ..
        } = receipt.observation
        else {
            panic!("owned failed checker receipt was not retained");
        };
    }
    assert_eq!(second.ordinal, 2);
    assert_eq!(second.predecessor.as_ref(), Some(&first.attempt_id));
    assert_eq!(second.state, AttemptState::Passed);
    assert_eq!(
        second.selected.profile.id,
        history.mission.policy.strong.profile_id
    );
    assert!(second.ownership_released);
    assert_eq!(
        history.tasks[0].winner.as_ref().unwrap().winning_attempt_id,
        second.attempt_id
    );
    let contract = store.get_run_contract(&run_id).unwrap().unwrap();
    let manifest = contract.prepared_manifest.unwrap();
    assert_eq!(manifest.files.len(), 1);
    assert_eq!(manifest.files[0].path, "src/lib.rs");
    assert_eq!(
        fs::read(repo.path().join("src/lib.rs")).unwrap(),
        b"pub fn value() {}\n"
    );
    apply_run_changes(
        None,
        Some(repo.path().to_path_buf()),
        &run_id,
        &manifest.package_digest,
    )
    .unwrap();
    assert_eq!(
        fs::read(repo.path().join("src/lib.rs")).unwrap(),
        b"pub fn value() -> i32 { 42 }\n"
    );
}

fn staged_ref(plan: &FlowPlan) -> StagedRoutingMissionRef {
    let review = plan.routing.as_ref().unwrap();
    StagedRoutingMissionRef {
        domain_id: review.authorization.domain_id.clone(),
        run_id: review.authorization.run_id.clone(),
        draft_id: plan.draft_id.clone(),
        plan_digest: review.authorization.plan_digest.clone(),
        mission_digest: review.mission_digest.clone(),
    }
}

fn shadow_mission(
    plan: &FlowPlan,
    run_id: &RunId,
    plan_digest: &Digest,
) -> anyhow::Result<RoutingMission> {
    let mut reviewed = mission(plan, run_id, plan_digest)?;
    reviewed.policy.mode = RoutingMode::Shadow;
    reviewed.policy.advice_model = pytxo_planner::advisor::MODEL_ID.into();
    reviewed.policy.advice_template =
        pytxo_orchestrate::flow::reviewed_routing_advisor_identity_for_task(
            &reviewed.tasks[0].contract,
        )?;
    reviewed.policy.disclosure_scope_digest =
        Some(pytxo_orchestrate::flow::reviewed_routing_advisor_disclosure_scope_digest());
    reviewed.authorization.policy_digest = reviewed.policy.digest()?;
    Ok(reviewed)
}

fn hosted_shadow_mission(
    plan: &FlowPlan,
    run_id: &RunId,
    plan_digest: &Digest,
) -> anyhow::Result<RoutingMission> {
    let mut reviewed = shadow_mission(plan, run_id, plan_digest)?;
    reviewed.policy.advisor_recipient = Some(pytxo_planner::advisor::HOSTED_RECIPIENT.into());
    reviewed.policy.disclosure_scope_digest = Some(pytxo_planner::advisor::hosted_scope_digest());
    reviewed.policy.advice_template =
        pytxo_orchestrate::flow::reviewed_hosted_routing_advisor_identity_for_task(
            &reviewed.tasks[0].contract,
        )?;
    reviewed.authorization.policy_digest = reviewed.policy.digest()?;
    Ok(reviewed)
}

#[test]
fn hosted_shadow_review_binds_exact_recipient_scope_and_packet() {
    let repo = repo();
    let catalog = Catalog::open(&repo.path().join(".git/catalog.db")).unwrap();
    let plan =
        preview_experimental_routed_flow(&catalog, input(repo.path()), hosted_shadow_mission)
            .unwrap();
    let hosted =
        pytxo_orchestrate::flow::preview_reviewed_hosted_advisor_packet(&catalog, &plan.draft_id)
            .unwrap();
    assert_eq!(
        hosted.recipient_identity,
        pytxo_planner::advisor::HOSTED_RECIPIENT
    );
    assert_eq!(
        hosted.scope_digest,
        pytxo_planner::advisor::hosted_scope_digest()
    );
    assert_eq!(hosted.packet_digest, Digest::of_bytes(&hosted.packet_body));
    assert_eq!(hosted.reviewed_consent_revision, 1);
    assert!(preview_experimental_routed_advisor_packet(&catalog, &plan.draft_id).is_err());
    assert!(preview_proposed_hosted_advisor_packet(&catalog, &plan.draft_id).is_err());
    let store =
        PytxoStore::open_existing_read_only(&PytxoConfig::default().db_path_at(repo.path()))
            .unwrap();
    let staged = store
        .load_staged_routing_mission(&staged_ref(&plan))
        .unwrap();
    assert_eq!(
        staged.policy.advisor_recipient.as_deref(),
        Some(hosted.recipient_identity.as_str())
    );
    assert_eq!(
        staged.policy.disclosure_scope_digest,
        Some(hosted.scope_digest)
    );
    assert_eq!(
        staged.authorization.policy_digest,
        staged.policy.digest().unwrap()
    );
}

#[test]
fn hosted_shadow_preview_cannot_dispatch_without_a_hosted_controller() {
    let repo = repo();
    let catalog = Catalog::open(&repo.path().join(".git/catalog.db")).unwrap();
    let plan =
        preview_experimental_routed_flow(&catalog, input(repo.path()), hosted_shadow_mission)
            .unwrap();
    assert_eq!(plan.status, FlowStatus::ReviewOnly);
    let saved = save_reviewed_flow_plan(&catalog, plan).unwrap();
    let error = dispatch_experimental_routed_flow(&catalog, &saved.draft_id).unwrap_err();
    assert!(error.to_string().contains("review-only hosted Shadow Flow"));
    let persisted = catalog.get_flow_draft(&saved.draft_id).unwrap().unwrap();
    assert!(persisted.dispatched_run_id.is_none());
    assert_eq!(persisted.status, "review_only");
}

#[test]
fn hosted_shadow_stop_before_claim_cancels_exact_review() {
    let repo = repo();
    let catalog = Catalog::open(&repo.path().join(".git/catalog.db")).unwrap();
    let plan =
        preview_experimental_routed_flow(&catalog, input(repo.path()), hosted_shadow_mission)
            .unwrap();
    let reviewed = save_reviewed_flow_plan(&catalog, plan).unwrap();
    let run_id = staged_ref(&reviewed).run_id.0;

    assert!(
        request_stop_experimental_routed_flow(&catalog, &reviewed.draft_id, &run_id)
            .unwrap()
            .is_none()
    );
    let stopped = catalog.get_flow_draft(&reviewed.draft_id).unwrap().unwrap();
    assert_eq!(stopped.status, "cancelled");
    assert!(stopped.dispatched_run_id.is_none());
    let owner = RoutedFlowDispatchOwner {
        controller_pid: std::process::id(),
        controller_start_identity: "test-owner".into(),
        store_db_path: "C:\\test\\pytxo.db".into(),
        store_db_file_identity: Some("test-store-identity".into()),
    };
    assert!(!catalog
        .claim_review_only_hosted_shadow_dispatch(
            &reviewed.draft_id,
            stopped.plan_json.as_deref().unwrap(),
            &run_id,
            &owner,
        )
        .unwrap());
}

#[test]
#[cfg(all(windows, feature = "routed-test-faults"))]
#[ignore = "requires the offline Claude stub and an exact MSI embedded host"]
fn native_claude_hosted_shadow_fake_client_keeps_rules_strong_and_applies() {
    use std::sync::atomic::{AtomicUsize, Ordering};
    use std::sync::Arc;

    use pytxo_orchestrate::{
        dispatch_experimental_hosted_shadow_with_client,
        preview_experimental_claude_hosted_shadow_flow_with_facts, HostedClientFuture,
        HostedEvaluationRequest, HostedShadowClient,
    };
    use pytxo_store::{HostedGrantIntent, HostedRemoteGrantReceipt};

    struct FakeHostedClient {
        workspace_id: String,
        sends: Arc<AtomicUsize>,
    }
    impl HostedShadowClient for FakeHostedClient {
        fn recipient_identity(&self) -> &str {
            pytxo_planner::advisor::HOSTED_RECIPIENT
        }
        fn account_id(&self) -> &str {
            "user_test"
        }
        fn link_origin(&self) -> &str {
            "https://link.pytxo.com"
        }
        fn workspace_id(&self) -> &str {
            &self.workspace_id
        }
        fn grant_revision(&self) -> u64 {
            1
        }
        fn evaluate<'a>(&'a self, request: &'a HostedEvaluationRequest) -> HostedClientFuture<'a> {
            Box::pin(async move {
                self.sends.fetch_add(1, Ordering::SeqCst);
                Ok(serde_json::to_vec(&serde_json::json!({
                    "schema_version": 1,
                    "evaluation_id": request.request_id,
                    "request_id": request.request_id,
                    "question_set_version": pytxo_planner::advisor::TEMPLATE_VERSION,
                    "model_id": pytxo_planner::advisor::MODEL_ID,
                    "choice": "everyday_fit",
                    "distribution": {"everyday_fit": 0.9, "strong_needed": 0.08, "unclear": 0.02},
                    "usage_status": "known",
                    "usage_receipt_id": "r_0123456789abcdef0123456789abcdef",
                    "input_tokens": 10,
                    "output_tokens": 1,
                    "cost_nano_usd": 420,
                    "packet_digest": request.packet_digest.0,
                }))
                .unwrap())
            })
        }
    }

    assert_eq!(
        std::env::var("PYTXO_TEST_ROUTED_CLAUDE_STUB").as_deref(),
        Ok("1")
    );
    let embedded_host = std::path::PathBuf::from(
        std::env::var_os("PYTXO_TEST_EMBEDDED_HOST").expect("exact packaged embedded host"),
    );
    let expected_host_sha256 = std::env::var("PYTXO_TEST_EMBEDDED_HOST_SHA256")
        .expect("set the extracted MSI host SHA-256 for this acceptance test");
    assert_eq!(
        Digest::of_bytes(&fs::read(&embedded_host).unwrap()).0,
        expected_host_sha256.to_ascii_lowercase(),
        "native test host differs from the separately inspected package"
    );
    let root = tempfile::tempdir().unwrap();
    let tools = root.path().join("tools");
    let account = root.path().join("account");
    let probes = root.path().join("probes");
    fs::create_dir_all(&tools).unwrap();
    fs::create_dir_all(&account).unwrap();
    fs::create_dir_all(&probes).unwrap();
    let cli = tools.join("claude.exe");
    fs::copy(env!("CARGO_BIN_EXE_pytxo-claude-routing-fixture"), &cli).unwrap();
    unsafe {
        std::env::set_var("PYTXO_EXPERIMENTAL_ROUTED_CLAUDE_PROPOSAL", "1");
        std::env::set_var("PYTXO_EXPERIMENTAL_ROUTED_HOSTED_SHADOW_REVIEW", "1");
        std::env::set_var("PYTXO_TEST_HOSTED_SHADOW_DISPATCH", "1");
        std::env::set_var("PYTXO_ROUTED_CLAUDE_EXE", &cli);
        std::env::set_var("PYTXO_ROUTED_CLAUDE_ACCOUNT_HOME", &account);
        std::env::set_var("PYTXO_ROUTED_CLAUDE_PROBE_ROOT", &probes);
    }
    let repo = repo();
    fs::write(
        repo.path().join("pytxo.toml"),
        "permission_profile = 'orbit'\nexecution_backend = 'subprocess'\nmax_agents = 1\n[blast]\nprefer_kernel_overlay = false\n",
    )
    .unwrap();
    assert!(Command::new("git")
        .args(["add", "pytxo.toml"])
        .current_dir(repo.path())
        .status()
        .unwrap()
        .success());
    assert!(Command::new("git")
        .args([
            "-c",
            "user.name=Test",
            "-c",
            "user.email=test@pytxo.local",
            "commit",
            "-qm",
            "routed policy"
        ])
        .current_dir(repo.path())
        .status()
        .unwrap()
        .success());
    let catalog = Catalog::open(&repo.path().join(".git/catalog.db")).unwrap();
    catalog
        .configure_capacity_pool(&CapacityPoolConfig {
            resource_id: "account-claude".into(),
            capacity_units: 1,
            expected_revision: None,
            configured_at_ms: 100,
        })
        .unwrap();
    let mut draft = input(repo.path());
    draft.mission_text =
        "In src/lib.rs replace the file with exactly: pub fn value() -> i32 { 42 }".into();
    draft.verification_commands =
        vec![r"C:\Windows\System32\findstr.exe /C:42 src\lib.rs >NUL".into()];
    let preview = preview_experimental_claude_hosted_shadow_flow_with_facts(
        &catalog,
        draft,
        &account,
        "account-claude",
        ReviewedDemandFacts {
            task_kind: TaskKind::Other,
            context_complete: true,
            cross_component_requirement: Some(false),
            repeatable_symptom_supplied: None,
            specific_cause_hypothesis_supplied: None,
        },
    )
    .unwrap();
    assert_eq!(preview.status, FlowStatus::ReviewOnly);
    let reviewed = save_reviewed_flow_plan(&catalog, preview).unwrap();
    assert!(dispatch_experimental_routed_flow(&catalog, &reviewed.draft_id).is_err());
    let packet =
        pytxo_orchestrate::preview_reviewed_hosted_advisor_packet(&catalog, &reviewed.draft_id)
            .unwrap();
    pytxo_orchestrate::enable_experimental_hosted_advisor_local_consent(
        &catalog,
        &reviewed.draft_id,
        &packet,
        0,
    )
    .unwrap();
    let pinned = catalog
        .routing_advisor_consent_store(&packet.domain_id)
        .unwrap()
        .unwrap();
    let store_path = PytxoConfig::default().db_path_at(repo.path());
    catalog
        .upsert_domain(
            &packet.domain_id,
            &packet.domain_id,
            &store_path.to_string_lossy(),
            None,
        )
        .unwrap();
    let workspace_id = catalog
        .ensure_hosted_workspace_id(
            &packet.domain_id,
            &pinned.store_db_path,
            &pinned.store_db_file_identity,
        )
        .unwrap();
    let intent = HostedGrantIntent {
        domain_id: packet.domain_id.clone(),
        workspace_id: workspace_id.clone(),
        account_id: "user_test".into(),
        link_origin: "https://link.pytxo.com".into(),
        recipient_identity: packet.recipient_identity.clone(),
        scope_digest: packet.scope_digest.0.clone(),
        store_db_file_identity: packet.store_db_file_identity.clone(),
        consent_revision: 1,
    };
    catalog.begin_hosted_grant(&intent).unwrap();
    catalog
        .confirm_hosted_grant(
            &intent,
            &HostedRemoteGrantReceipt {
                workspace_id,
                account_id: intent.account_id.clone(),
                link_origin: intent.link_origin.clone(),
                recipient_identity: intent.recipient_identity.clone(),
                scope_digest: intent.scope_digest.clone(),
                revision: 1,
                enabled: true,
            },
        )
        .unwrap();
    let sends = Arc::new(AtomicUsize::new(0));
    let run_id = dispatch_experimental_hosted_shadow_with_client(
        &catalog,
        &reviewed.draft_id,
        Arc::new(FakeHostedClient {
            workspace_id: intent.workspace_id,
            sends: Arc::clone(&sends),
        }),
    )
    .unwrap();
    assert_eq!(sends.load(Ordering::SeqCst), 1);
    let store = PytxoStore::open(&store_path).unwrap();
    let scope = RoutingScope {
        domain_id: DomainId(reviewed.domain_id.clone()),
        run_id: RunId(run_id.clone()),
    };
    let history = store.routing_history(&scope).unwrap().unwrap();
    assert_eq!(
        store.get_run_status(&run_id).unwrap().unwrap().0,
        "completed"
    );
    assert_eq!(history.attempts.len(), 1);
    assert_eq!(history.attempts[0].state, AttemptState::Passed);
    assert_eq!(
        history.attempts[0].selected.profile.id.0,
        "claude-subscription-sonnet"
    );
    assert_eq!(
        history.attempts[0].decision.advice_status,
        AdviceStatus::ShadowRecorded
    );
    let manifest = store
        .get_run_contract(&run_id)
        .unwrap()
        .unwrap()
        .prepared_manifest
        .unwrap();
    assert_eq!(
        fs::read_to_string(repo.path().join("src/lib.rs")).unwrap(),
        "pub fn value() {}\n"
    );
    apply_run_changes(
        None,
        Some(repo.path().to_path_buf()),
        &run_id,
        &manifest.package_digest,
    )
    .unwrap();
    assert!(fs::read_to_string(repo.path().join("src/lib.rs"))
        .unwrap()
        .contains("42"));
}

#[test]
fn hosted_shadow_review_rejects_fixture_scope_or_wrong_recipient() {
    for mutate in [0, 1, 2] {
        let repo = repo();
        let catalog = Catalog::open(&repo.path().join(".git/catalog.db")).unwrap();
        let result = preview_experimental_routed_flow(
            &catalog,
            input(repo.path()),
            |plan, run_id, plan_digest| {
                let mut mission = hosted_shadow_mission(plan, run_id, plan_digest)?;
                match mutate {
                    0 => mission.policy.disclosure_scope_digest = Some(
                        pytxo_orchestrate::flow::reviewed_routing_advisor_disclosure_scope_digest(),
                    ),
                    1 => {
                        mission.policy.advisor_recipient = Some("different-hosted-recipient".into())
                    }
                    _ => mission.policy.advice_template.push_str(":changed"),
                }
                mission.authorization.policy_digest = mission.policy.digest()?;
                Ok(mission)
            },
        );
        assert!(
            result.is_err(),
            "mutated hosted contract {mutate} was accepted"
        );
    }
}

#[test]
#[cfg(windows)]
fn hosted_local_consent_requires_exact_review_and_never_grants_fixture_or_link() {
    use pytxo_orchestrate::flow::{
        enable_experimental_hosted_advisor_local_consent,
        read_experimental_hosted_advisor_local_consent,
        revoke_experimental_hosted_advisor_local_consent,
    };

    let repo = repo();
    let catalog = Catalog::open(&repo.path().join(".git/catalog.db")).unwrap();
    let plan =
        preview_experimental_routed_flow(&catalog, input(repo.path()), hosted_shadow_mission)
            .unwrap();
    let saved = save_reviewed_flow_plan(&catalog, plan).unwrap();
    let packet =
        pytxo_orchestrate::flow::preview_reviewed_hosted_advisor_packet(&catalog, &saved.draft_id)
            .unwrap();
    let before =
        read_experimental_hosted_advisor_local_consent(&catalog, &packet.domain_id).unwrap();
    assert_eq!(before.revision, 0);
    assert!(!before.enabled);
    let enable = |inspected: &pytxo_orchestrate::flow::ReviewedHostedAdvisorPacketPreview,
                  revision| {
        enable_experimental_hosted_advisor_local_consent(
            &catalog,
            &saved.draft_id,
            inspected,
            revision,
        )
    };
    let mut wrong_recipient = packet.clone();
    wrong_recipient.recipient_identity = pytxo_orchestrate::flow::REVIEWED_ADVISOR_RECIPIENT.into();
    assert!(enable(&wrong_recipient, 0).is_err());
    let mut wrong_scope = packet.clone();
    wrong_scope.scope_digest = Digest::of_bytes(b"changed");
    assert!(enable(&wrong_scope, 0).is_err());
    assert_eq!(
        read_experimental_hosted_advisor_local_consent(&catalog, &packet.domain_id)
            .unwrap()
            .revision,
        0
    );

    let enabled = enable(&packet, 0).unwrap();
    assert_eq!(enabled.revision, 1);
    assert!(enabled.enabled && enabled.current_scope);
    assert_eq!(enabled.recipient_identity, packet.recipient_identity);
    assert!(enable(&packet, 0).is_err());
    assert_eq!(
        read_experimental_routed_advisor_consent(&catalog, &packet.domain_id)
            .unwrap()
            .revision,
        0,
        "hosted consent must not become the local fixture grant"
    );
    let store_path = PytxoConfig::default().db_path_at(repo.path());
    let store = PytxoStore::open_existing_read_only(&store_path).unwrap();
    let requests: i64 = rusqlite::Connection::open_with_flags(
        &store_path,
        rusqlite::OpenFlags::SQLITE_OPEN_READ_ONLY,
    )
    .unwrap()
    .query_row("SELECT COUNT(*) FROM routing_advisor_requests", [], |row| {
        row.get(0)
    })
    .unwrap();
    assert_eq!(requests, 0);
    assert!(store.get_run(&packet.run_id).unwrap().is_none());
    let revoked = revoke_experimental_hosted_advisor_local_consent(
        &catalog,
        &packet.domain_id,
        enabled.revision,
    )
    .unwrap();
    assert_eq!(revoked.revision, 2);
    assert!(!revoked.enabled && !revoked.current_scope);
}

#[test]
#[cfg(windows)]
fn hosted_revoke_recovers_store_commit_before_catalog_fence() {
    use pytxo_orchestrate::flow::{
        enable_experimental_hosted_advisor_local_consent, preview_reviewed_hosted_advisor_packet,
        revoke_experimental_hosted_advisor_local_consent,
    };
    use pytxo_store::{HostedGrantIntent, HostedGrantState, HostedRemoteGrantReceipt};

    let repo = repo();
    let catalog_path = repo.path().join(".git/catalog.db");
    let catalog = Catalog::open(&catalog_path).unwrap();
    let plan =
        preview_experimental_routed_flow(&catalog, input(repo.path()), hosted_shadow_mission)
            .unwrap();
    let saved = save_reviewed_flow_plan(&catalog, plan).unwrap();
    let packet = preview_reviewed_hosted_advisor_packet(&catalog, &saved.draft_id).unwrap();
    enable_experimental_hosted_advisor_local_consent(&catalog, &saved.draft_id, &packet, 0)
        .unwrap();
    let store_path = PytxoConfig::default().db_path_at(repo.path());
    let pinned = catalog
        .routing_advisor_consent_store(&packet.domain_id)
        .unwrap()
        .unwrap();
    catalog
        .upsert_domain(
            &packet.domain_id,
            &packet.domain_id,
            &store_path.to_string_lossy(),
            None,
        )
        .unwrap();
    let workspace_id = catalog
        .ensure_hosted_workspace_id(
            &packet.domain_id,
            &pinned.store_db_path,
            &pinned.store_db_file_identity,
        )
        .unwrap();
    let intent = HostedGrantIntent {
        domain_id: packet.domain_id.clone(),
        workspace_id: workspace_id.clone(),
        account_id: "user_one".into(),
        link_origin: "https://link.pytxo.com".into(),
        recipient_identity: packet.recipient_identity.clone(),
        scope_digest: packet.scope_digest.0.clone(),
        store_db_file_identity: packet.store_db_file_identity.clone(),
        consent_revision: 1,
    };
    catalog.begin_hosted_grant(&intent).unwrap();
    catalog
        .confirm_hosted_grant(
            &intent,
            &HostedRemoteGrantReceipt {
                workspace_id,
                account_id: intent.account_id.clone(),
                link_origin: intent.link_origin.clone(),
                recipient_identity: intent.recipient_identity.clone(),
                scope_digest: intent.scope_digest.clone(),
                revision: 1,
                enabled: true,
            },
        )
        .unwrap();
    let store = PytxoStore::open_existing_read_write(&store_path).unwrap();
    let committed = store
        .set_routing_hosted_advisor_consent(
            &DomainId(packet.domain_id.clone()),
            &packet.recipient_identity,
            1,
            false,
            None,
            u64::try_from(chrono::Utc::now().timestamp_millis()).unwrap(),
        )
        .unwrap();
    assert_eq!(committed.revision, 2);
    drop(store);
    drop(catalog);
    let catalog = Catalog::open(&catalog_path).unwrap();
    let stale = catalog
        .hosted_advisor_consent_fence(&packet.domain_id)
        .unwrap()
        .unwrap();
    assert!(stale.enabled);
    assert_eq!(stale.consent_revision, 1);
    assert_eq!(
        catalog
            .hosted_grant(&packet.domain_id)
            .unwrap()
            .unwrap()
            .state,
        HostedGrantState::Enabled
    );
    let recovered =
        revoke_experimental_hosted_advisor_local_consent(&catalog, &packet.domain_id, 1).unwrap();
    assert_eq!(recovered.revision, 2);
    assert!(!recovered.enabled);
    let fence = catalog
        .hosted_advisor_consent_fence(&packet.domain_id)
        .unwrap()
        .unwrap();
    assert!(!fence.enabled);
    assert_eq!(fence.consent_revision, 2);
    assert_eq!(
        catalog
            .hosted_grant(&packet.domain_id)
            .unwrap()
            .unwrap()
            .state,
        HostedGrantState::RevokePending
    );
}

#[test]
#[cfg(windows)]
fn hosted_local_consent_cannot_reopen_pending_link_revoke() {
    use pytxo_orchestrate::flow::{
        enable_experimental_hosted_advisor_local_consent, preview_reviewed_hosted_advisor_packet,
        revoke_experimental_hosted_advisor_local_consent,
    };
    use pytxo_store::{HostedGrantIntent, HostedRemoteGrantReceipt};

    let repo = repo();
    let catalog = Catalog::open(&repo.path().join(".git/catalog.db")).unwrap();
    let first =
        preview_experimental_routed_flow(&catalog, input(repo.path()), hosted_shadow_mission)
            .unwrap();
    let first = save_reviewed_flow_plan(&catalog, first).unwrap();
    let packet = preview_reviewed_hosted_advisor_packet(&catalog, &first.draft_id).unwrap();
    enable_experimental_hosted_advisor_local_consent(&catalog, &first.draft_id, &packet, 0)
        .unwrap();
    let pinned = catalog
        .routing_advisor_consent_store(&packet.domain_id)
        .unwrap()
        .unwrap();
    catalog
        .upsert_domain(
            &packet.domain_id,
            &packet.domain_id,
            &PytxoConfig::default()
                .db_path_at(repo.path())
                .to_string_lossy(),
            None,
        )
        .unwrap();
    let workspace_id = catalog
        .ensure_hosted_workspace_id(
            &packet.domain_id,
            &pinned.store_db_path,
            &pinned.store_db_file_identity,
        )
        .unwrap();
    let intent = HostedGrantIntent {
        domain_id: packet.domain_id.clone(),
        workspace_id: workspace_id.clone(),
        account_id: "user_one".into(),
        link_origin: "https://link.pytxo.com".into(),
        recipient_identity: packet.recipient_identity.clone(),
        scope_digest: packet.scope_digest.0.clone(),
        store_db_file_identity: packet.store_db_file_identity.clone(),
        consent_revision: 1,
    };
    catalog.begin_hosted_grant(&intent).unwrap();
    catalog
        .confirm_hosted_grant(
            &intent,
            &HostedRemoteGrantReceipt {
                workspace_id,
                account_id: intent.account_id.clone(),
                link_origin: intent.link_origin.clone(),
                recipient_identity: intent.recipient_identity.clone(),
                scope_digest: intent.scope_digest.clone(),
                revision: 1,
                enabled: true,
            },
        )
        .unwrap();
    revoke_experimental_hosted_advisor_local_consent(&catalog, &packet.domain_id, 1).unwrap();
    catalog
        .begin_hosted_grant_revoke(&packet.domain_id)
        .unwrap();

    let mut next_input = input(repo.path());
    next_input.id = "routed-flow-after-revoke".into();
    let second =
        preview_experimental_routed_flow(&catalog, next_input, hosted_shadow_mission).unwrap();
    let second = save_reviewed_flow_plan(&catalog, second).unwrap();
    let second_packet = preview_reviewed_hosted_advisor_packet(&catalog, &second.draft_id).unwrap();
    assert!(enable_experimental_hosted_advisor_local_consent(
        &catalog,
        &second.draft_id,
        &second_packet,
        2,
    )
    .is_err());
    let store =
        PytxoStore::open_existing_read_only(std::path::Path::new(&pinned.store_db_path)).unwrap();
    let local = store
        .routing_hosted_advisor_consent(
            &DomainId(packet.domain_id.clone()),
            &packet.recipient_identity,
        )
        .unwrap();
    assert!(!local.enabled);
    assert_eq!(local.revision, 2);
    assert!(
        !catalog
            .hosted_advisor_consent_fence(&packet.domain_id)
            .unwrap()
            .unwrap()
            .enabled
    );
}

#[test]
#[cfg(windows)]
fn hosted_local_consent_cannot_follow_a_replacement_store() {
    use pytxo_orchestrate::flow::{
        enable_experimental_hosted_advisor_local_consent,
        read_experimental_hosted_advisor_local_consent,
        revoke_experimental_hosted_advisor_local_consent,
    };

    let repo = repo();
    let catalog = Catalog::open(&repo.path().join(".git/catalog.db")).unwrap();
    let plan =
        preview_experimental_routed_flow(&catalog, input(repo.path()), hosted_shadow_mission)
            .unwrap();
    let saved = save_reviewed_flow_plan(&catalog, plan).unwrap();
    let packet =
        pytxo_orchestrate::flow::preview_reviewed_hosted_advisor_packet(&catalog, &saved.draft_id)
            .unwrap();
    let enabled =
        enable_experimental_hosted_advisor_local_consent(&catalog, &saved.draft_id, &packet, 0)
            .unwrap();
    let store_path = PytxoConfig::default().db_path_at(repo.path());
    let original_path = repo.path().join("original-hosted-consent.db");
    let replacement_path = repo.path().join("replacement-hosted-consent.db");
    drop(PytxoStore::open(&replacement_path).unwrap());
    fs::rename(&store_path, &original_path).unwrap();
    fs::rename(&replacement_path, &store_path).unwrap();
    assert!(read_experimental_hosted_advisor_local_consent(&catalog, &packet.domain_id).is_err());
    assert!(revoke_experimental_hosted_advisor_local_consent(
        &catalog,
        &packet.domain_id,
        enabled.revision,
    )
    .is_err());
    assert!(
        PytxoStore::open_existing_read_only(&original_path)
            .unwrap()
            .routing_hosted_advisor_consent(&DomainId(packet.domain_id), &packet.recipient_identity)
            .unwrap()
            .enabled
    );
}

#[test]
#[cfg(windows)]
fn hosted_local_consent_rejects_a_cloned_store_replaced_after_preview() {
    use pytxo_orchestrate::flow::{
        enable_experimental_hosted_advisor_local_consent, preview_reviewed_hosted_advisor_packet,
    };

    let repo = repo();
    let catalog = Catalog::open(&repo.path().join(".git/catalog.db")).unwrap();
    let plan =
        preview_experimental_routed_flow(&catalog, input(repo.path()), hosted_shadow_mission)
            .unwrap();
    let saved = save_reviewed_flow_plan(&catalog, plan).unwrap();
    let inspected = preview_reviewed_hosted_advisor_packet(&catalog, &saved.draft_id).unwrap();
    let store_path = PytxoConfig::default().db_path_at(repo.path());
    let original_path = repo.path().join(".git/original-hosted-review.db");
    let replacement_path = repo.path().join(".git/cloned-hosted-review.db");
    fs::copy(&store_path, &replacement_path).unwrap();
    fs::rename(&store_path, &original_path).unwrap();
    fs::rename(&replacement_path, &store_path).unwrap();
    let cloned = preview_reviewed_hosted_advisor_packet(&catalog, &saved.draft_id).unwrap();
    assert_eq!(cloned.packet_body, inspected.packet_body);
    assert_eq!(cloned.packet_digest, inspected.packet_digest);
    assert_ne!(
        cloned.store_db_file_identity,
        inspected.store_db_file_identity
    );
    assert!(enable_experimental_hosted_advisor_local_consent(
        &catalog,
        &saved.draft_id,
        &inspected,
        0,
    )
    .is_err());
    assert!(catalog
        .routing_advisor_consent_store(&inspected.domain_id)
        .unwrap()
        .is_none());
    assert!(
        !PytxoStore::open_existing_read_only(&store_path)
            .unwrap()
            .routing_hosted_advisor_consent(
                &DomainId(inspected.domain_id),
                &inspected.recipient_identity
            )
            .unwrap()
            .enabled
    );
}

#[test]
#[cfg(windows)]
fn hosted_local_consent_status_rejects_an_unpinned_store_grant() {
    use pytxo_orchestrate::flow::{
        preview_reviewed_hosted_advisor_packet, read_experimental_hosted_advisor_local_consent,
        revoke_experimental_hosted_advisor_local_consent,
    };

    let repo = repo();
    let catalog = Catalog::open(&repo.path().join(".git/catalog.db")).unwrap();
    let plan =
        preview_experimental_routed_flow(&catalog, input(repo.path()), hosted_shadow_mission)
            .unwrap();
    let saved = save_reviewed_flow_plan(&catalog, plan).unwrap();
    let packet = preview_reviewed_hosted_advisor_packet(&catalog, &saved.draft_id).unwrap();
    let store_path = PytxoConfig::default().db_path_at(repo.path());
    let store = PytxoStore::open_existing_read_write(&store_path).unwrap();
    store
        .set_routing_hosted_advisor_consent(
            &DomainId(packet.domain_id.clone()),
            &packet.recipient_identity,
            0,
            true,
            Some(packet.scope_digest),
            1,
        )
        .unwrap();
    assert!(catalog
        .routing_advisor_consent_store(&packet.domain_id)
        .unwrap()
        .is_none());
    assert!(read_experimental_hosted_advisor_local_consent(&catalog, &packet.domain_id).is_err());
    assert!(
        revoke_experimental_hosted_advisor_local_consent(&catalog, &packet.domain_id, 0).is_err()
    );
    assert!(catalog
        .routing_advisor_consent_store(&packet.domain_id)
        .unwrap()
        .is_none());
    assert!(read_experimental_hosted_advisor_local_consent(&catalog, &packet.domain_id).is_err());
}

#[test]
#[cfg(windows)]
fn hosted_status_does_not_treat_a_shared_store_pin_as_hosted_review() {
    use pytxo_orchestrate::flow::read_experimental_hosted_advisor_local_consent;

    let repo = repo();
    let catalog = Catalog::open(&repo.path().join(".git/catalog.db")).unwrap();
    let plan =
        preview_experimental_routed_flow(&catalog, input(repo.path()), shadow_mission).unwrap();
    let saved = save_reviewed_flow_plan(&catalog, plan).unwrap();
    let local = preview_experimental_routed_advisor_packet(&catalog, &saved.draft_id).unwrap();
    enable_experimental_routed_advisor_consent(
        &catalog,
        &saved.draft_id,
        &local.domain_id,
        &local.request_digest.0,
        &local.recipient_identity,
        0,
    )
    .unwrap();
    let store_path = PytxoConfig::default().db_path_at(repo.path());
    PytxoStore::open_existing_read_write(&store_path)
        .unwrap()
        .set_routing_hosted_advisor_consent(
            &DomainId(local.domain_id.clone()),
            pytxo_planner::advisor::HOSTED_RECIPIENT,
            0,
            true,
            Some(pytxo_planner::advisor::hosted_scope_digest()),
            1,
        )
        .unwrap();
    assert!(catalog
        .routing_advisor_consent_store(&local.domain_id)
        .unwrap()
        .is_some());
    assert!(catalog
        .hosted_advisor_consent_review(&local.domain_id)
        .unwrap()
        .is_none());
    assert!(read_experimental_hosted_advisor_local_consent(&catalog, &local.domain_id).is_err());
}

#[test]
#[cfg(windows)]
fn hosted_revoke_fence_rejects_an_old_enabled_row_restored_in_place() {
    use pytxo_orchestrate::flow::{
        enable_experimental_hosted_advisor_local_consent, preview_reviewed_hosted_advisor_packet,
        read_experimental_hosted_advisor_local_consent,
        revoke_experimental_hosted_advisor_local_consent,
    };

    let repo = repo();
    let catalog = Catalog::open(&repo.path().join(".git/catalog.db")).unwrap();
    let plan =
        preview_experimental_routed_flow(&catalog, input(repo.path()), hosted_shadow_mission)
            .unwrap();
    let saved = save_reviewed_flow_plan(&catalog, plan).unwrap();
    let packet = preview_reviewed_hosted_advisor_packet(&catalog, &saved.draft_id).unwrap();
    let granted =
        enable_experimental_hosted_advisor_local_consent(&catalog, &saved.draft_id, &packet, 0)
            .unwrap();
    assert_eq!(granted.revision, 1);
    let revoked = revoke_experimental_hosted_advisor_local_consent(
        &catalog,
        &packet.domain_id,
        granted.revision,
    )
    .unwrap();
    assert_eq!(revoked.revision, 2);
    assert_eq!(
        catalog
            .hosted_advisor_consent_fence(&packet.domain_id)
            .unwrap()
            .unwrap()
            .consent_revision,
        2
    );
    let store_path = PytxoConfig::default().db_path_at(repo.path());
    let original_identity = pytxo_runner::file_identity(&store_path).unwrap();
    rusqlite::Connection::open(&store_path)
        .unwrap()
        .execute(
            "UPDATE routing_hosted_advisor_consent SET revision = 1, enabled = 1,
                    scope_digest = ?1 WHERE domain_id = ?2 AND recipient_identity = ?3",
            rusqlite::params![
                packet.scope_digest.0,
                packet.domain_id,
                packet.recipient_identity
            ],
        )
        .unwrap();
    assert_eq!(
        pytxo_runner::file_identity(&store_path).unwrap(),
        original_identity
    );
    assert!(read_experimental_hosted_advisor_local_consent(&catalog, &packet.domain_id).is_err());
}

#[test]
#[cfg(windows)]
fn hosted_local_consent_revoke_before_grant_tombstones_a_pending_enable() {
    use pytxo_orchestrate::flow::{
        enable_experimental_hosted_advisor_local_consent, preview_reviewed_hosted_advisor_packet,
        read_experimental_hosted_advisor_local_consent,
        revoke_experimental_hosted_advisor_local_consent,
    };

    let repo = repo();
    let catalog = Catalog::open(&repo.path().join(".git/catalog.db")).unwrap();
    let plan =
        preview_experimental_routed_flow(&catalog, input(repo.path()), hosted_shadow_mission)
            .unwrap();
    let saved = save_reviewed_flow_plan(&catalog, plan).unwrap();
    let packet = preview_reviewed_hosted_advisor_packet(&catalog, &saved.draft_id).unwrap();
    let revoked =
        revoke_experimental_hosted_advisor_local_consent(&catalog, &packet.domain_id, 0).unwrap();
    assert_eq!(revoked.revision, 1);
    assert!(!revoked.enabled);
    assert_eq!(
        read_experimental_hosted_advisor_local_consent(&catalog, &packet.domain_id)
            .unwrap()
            .revision,
        1
    );
    assert!(catalog
        .routing_advisor_consent_store(&packet.domain_id)
        .unwrap()
        .is_some());
    assert!(enable_experimental_hosted_advisor_local_consent(
        &catalog,
        &saved.draft_id,
        &packet,
        0,
    )
    .is_err());
    assert!(
        !read_experimental_hosted_advisor_local_consent(&catalog, &packet.domain_id)
            .unwrap()
            .enabled
    );
}

#[test]
fn shadow_packet_preview_uses_only_reviewed_fixed_vocabulary_and_writes_no_routing_state() {
    let repo = repo();
    let catalog = Catalog::open(&repo.path().join(".git/catalog.db")).unwrap();
    let plan =
        preview_experimental_routed_flow(&catalog, input(repo.path()), shadow_mission).unwrap();
    let draft_before = catalog.get_flow_draft(&plan.draft_id).unwrap().unwrap();
    let review = plan.routing.as_ref().unwrap();
    let store_path = PytxoConfig::default().db_path_at(repo.path());
    let store = PytxoStore::open_existing_read_only(&store_path).unwrap();
    let scope = RoutingScope {
        domain_id: review.authorization.domain_id.clone(),
        run_id: review.authorization.run_id.clone(),
    };
    let consent_before = store.routing_advisor_consent(&scope.domain_id).unwrap();
    let requests = rusqlite::Connection::open_with_flags(
        &store_path,
        rusqlite::OpenFlags::SQLITE_OPEN_READ_ONLY,
    )
    .unwrap();
    let request_count = || -> i64 {
        requests
            .query_row("SELECT COUNT(*) FROM routing_advisor_requests", [], |row| {
                row.get(0)
            })
            .unwrap()
    };
    let requests_before = request_count();
    let preview = preview_experimental_routed_advisor_packet(&catalog, &plan.draft_id).unwrap();
    assert_eq!(preview.domain_id, plan.domain_id);
    assert_eq!(preview.run_id, review.authorization.run_id.0);
    assert_eq!(preview.task_id, plan.tasks[0].id);
    assert_eq!(
        preview.recipient_identity,
        pytxo_orchestrate::flow::REVIEWED_ADVISOR_RECIPIENT
    );
    assert_eq!(
        preview.request_digest,
        Digest::of_bytes(&preview.request_body)
    );
    let expected = pytxo_planner::advisor::AdvisorPacket::new(
        "Classify reviewed repository task using coarse facts",
        [
            "task_kind_local_transformation",
            "reviewed_checks",
            "single_component_claimed",
            "context_claimed_complete",
            "role_unrestricted",
            "no_required_egress",
            "repeatable_symptom_unknown",
            "specific_cause_hypothesis_unknown",
        ],
        "everyday execution role",
        "strong execution role",
    )
    .unwrap();
    assert_eq!(preview.packet_digest, expected.digest());
    assert_eq!(preview.request_body, expected.request_body().unwrap());
    let body = String::from_utf8(preview.request_body).unwrap();
    assert!(body.contains("task_kind_local_transformation"));
    for private in [
        plan.domain_id.as_str(),
        "src/lib.rs",
        "Update src/lib.rs",
        "Update",
        "Alice Smith",
        "salary 90000",
        "keychain:",
        "private-fixture",
    ] {
        assert!(!body.contains(private), "packet leaked {private}");
    }
    assert_eq!(
        catalog.get_flow_draft(&plan.draft_id).unwrap(),
        Some(draft_before)
    );
    assert!(store.get_run(&preview.run_id).unwrap().is_none());
    assert!(store.routing_history(&scope).unwrap().is_none());
    assert_eq!(
        store.routing_advisor_consent(&scope.domain_id).unwrap(),
        consent_before
    );
    assert_eq!(request_count(), requests_before);
}

#[test]
fn proposed_hosted_packet_is_read_only_and_matches_the_fixed_proxy_packet() {
    let repo = repo();
    let catalog = Catalog::open(&repo.path().join(".git/catalog.db")).unwrap();
    let plan =
        preview_experimental_routed_flow(&catalog, input(repo.path()), shadow_mission).unwrap();
    let draft_before = catalog.get_flow_draft(&plan.draft_id).unwrap();
    let store =
        PytxoStore::open_existing_read_only(&PytxoConfig::default().db_path_at(repo.path()))
            .unwrap();
    let domain = DomainId(plan.domain_id.clone());
    let consent_before = store.routing_advisor_consent(&domain).unwrap();
    let local = preview_experimental_routed_advisor_packet(&catalog, &plan.draft_id).unwrap();
    let proposed = preview_proposed_hosted_advisor_packet(&catalog, &plan.draft_id).unwrap();
    assert_eq!(
        proposed.source_review_recipient_identity,
        local.recipient_identity
    );
    assert_eq!(
        proposed.recipient_identity,
        pytxo_planner::advisor::HOSTED_RECIPIENT
    );
    assert_eq!(
        proposed.scope_digest,
        pytxo_planner::advisor::hosted_scope_digest()
    );
    assert_eq!(proposed.packet_digest, local.packet_digest);
    assert_eq!(proposed.wire_schema_version, 1);
    assert_eq!(proposed.decision_kind, "initial_demand");
    assert_eq!(
        proposed.question_set_version,
        pytxo_planner::advisor::TEMPLATE_VERSION
    );
    let packet: serde_json::Value = serde_json::from_slice(&proposed.packet_body).unwrap();
    let proxy_fixture: serde_json::Value = serde_json::from_str(include_str!(
        "../../../services/pytxo-proxy/tests/fixtures/proposed-hosted-packet-v1.json"
    ))
    .unwrap();
    assert_eq!(packet, proxy_fixture);
    assert_eq!(packet["schema_version"], 1);
    assert_eq!(
        packet["goal"],
        "Classify reviewed repository task using coarse facts"
    );
    assert_eq!(packet["everyday_role"], "everyday execution role");
    assert_eq!(packet["strong_role"], "strong execution role");
    assert_eq!(packet["features"].as_array().unwrap().len(), 8);
    assert_eq!(packet["features"][0], "task_kind_local_transformation");
    assert_eq!(packet["features"][1], "reviewed_checks");
    assert_eq!(packet["features"][2], "single_component_claimed");
    assert_eq!(packet["features"][3], "context_claimed_complete");
    assert_eq!(packet["features"][4], "role_unrestricted");
    assert_eq!(packet["features"][5], "no_required_egress");
    let body = String::from_utf8(proposed.packet_body).unwrap();
    for private in [
        plan.domain_id.as_str(),
        "src/lib.rs",
        "Alice Smith",
        "salary 90000",
    ] {
        assert!(!body.contains(private), "hosted packet leaked {private}");
    }
    assert_eq!(
        catalog.get_flow_draft(&plan.draft_id).unwrap(),
        draft_before
    );
    assert_eq!(
        store.routing_advisor_consent(&domain).unwrap(),
        consent_before
    );
}

#[test]
#[cfg(windows)]
fn local_shadow_consent_requires_exact_review_and_cas_then_revokes_durably() {
    let repo = repo();
    let catalog = Catalog::open(&repo.path().join(".git/catalog.db")).unwrap();
    let plan =
        preview_experimental_routed_flow(&catalog, input(repo.path()), shadow_mission).unwrap();
    let reviewed = save_reviewed_flow_plan(&catalog, plan).unwrap();
    let preview = preview_experimental_routed_advisor_packet(&catalog, &reviewed.draft_id).unwrap();
    let before = read_experimental_routed_advisor_consent(&catalog, &preview.domain_id).unwrap();
    assert_eq!(before.revision, 0);
    assert!(!before.enabled && !before.current_scope);
    assert_eq!(preview.reviewed_consent_revision, before.revision + 1);

    let enable = |domain: &str, digest: &str, recipient: &str, revision| {
        enable_experimental_routed_advisor_consent(
            &catalog,
            &reviewed.draft_id,
            domain,
            digest,
            recipient,
            revision,
        )
    };
    assert!(enable(
        "C:\\different-workspace",
        &preview.request_digest.0,
        &preview.recipient_identity,
        before.revision,
    )
    .is_err());
    assert!(enable(
        &preview.domain_id,
        &Digest::of_bytes(b"stale inspected packet").0,
        &preview.recipient_identity,
        before.revision,
    )
    .is_err());
    assert!(enable(
        &preview.domain_id,
        &preview.request_digest.0,
        "hosted-recipient-not-reviewed",
        before.revision,
    )
    .is_err());
    assert_eq!(
        read_experimental_routed_advisor_consent(&catalog, &preview.domain_id)
            .unwrap()
            .revision,
        0
    );

    let granted = enable(
        &preview.domain_id,
        &preview.request_digest.0,
        &preview.recipient_identity,
        before.revision,
    )
    .unwrap();
    assert_eq!(granted.revision, 1);
    assert!(granted.enabled && granted.current_scope);
    assert_eq!(granted.recipient_identity, preview.recipient_identity);
    assert!(enable(
        &preview.domain_id,
        &preview.request_digest.0,
        &preview.recipient_identity,
        before.revision,
    )
    .is_err());
    assert_eq!(
        read_experimental_routed_advisor_consent(&catalog, &preview.domain_id)
            .unwrap()
            .revision,
        granted.revision
    );

    let store_path = PytxoConfig::default().db_path_at(repo.path());
    let store = PytxoStore::open_existing_read_only(&store_path).unwrap();
    let old_stage = store
        .load_staged_routing_mission(&staged_ref(&reviewed))
        .unwrap();
    assert_eq!(old_stage.authorization.consent_revision, granted.revision);
    let db = rusqlite::Connection::open_with_flags(
        &store_path,
        rusqlite::OpenFlags::SQLITE_OPEN_READ_ONLY,
    )
    .unwrap();
    let request_count: i64 = db
        .query_row("SELECT COUNT(*) FROM routing_advisor_requests", [], |row| {
            row.get(0)
        })
        .unwrap();
    assert_eq!(request_count, 0);
    assert!(store.get_run(&preview.run_id).unwrap().is_none());

    // A config edit must not make the original grant disappear or redirect
    // revocation to a newly configured Store.
    fs::write(
        repo.path().join("pytxo.toml"),
        "data_dir = '.pytxo/changed-after-grant'\n",
    )
    .unwrap();
    let moved_config_status =
        read_experimental_routed_advisor_consent(&catalog, &preview.domain_id).unwrap();
    assert_eq!(moved_config_status.revision, granted.revision);
    assert!(moved_config_status.enabled);
    fs::write(repo.path().join("pytxo.toml"), "not valid [ TOML").unwrap();
    assert_eq!(
        read_experimental_routed_advisor_consent(&catalog, &preview.domain_id)
            .unwrap()
            .revision,
        granted.revision
    );

    assert!(revoke_experimental_routed_advisor_consent(
        &catalog,
        &preview.domain_id,
        before.revision
    )
    .is_err());
    let revoked =
        revoke_experimental_routed_advisor_consent(&catalog, &preview.domain_id, granted.revision)
            .unwrap();
    assert_eq!(revoked.revision, 2);
    assert!(!revoked.enabled && !revoked.current_scope);
    let reopened = read_experimental_routed_advisor_consent(&catalog, &preview.domain_id).unwrap();
    assert_eq!(reopened.revision, revoked.revision);
    assert!(!reopened.enabled);
    assert_ne!(old_stage.authorization.consent_revision, reopened.revision);
}

#[test]
#[cfg(windows)]
fn local_shadow_grant_rejects_a_review_frozen_to_the_wrong_consent_revision() {
    let repo = repo();
    let catalog = Catalog::open(&repo.path().join(".git/catalog.db")).unwrap();
    let plan = preview_experimental_routed_flow(
        &catalog,
        input(repo.path()),
        |plan, run_id, plan_digest| {
            let mut mission = shadow_mission(plan, run_id, plan_digest)?;
            mission.authorization.consent_revision = 9;
            Ok(mission)
        },
    )
    .unwrap();
    let reviewed = save_reviewed_flow_plan(&catalog, plan).unwrap();
    let preview = preview_experimental_routed_advisor_packet(&catalog, &reviewed.draft_id).unwrap();
    assert_eq!(preview.reviewed_consent_revision, 9);
    assert!(enable_experimental_routed_advisor_consent(
        &catalog,
        &reviewed.draft_id,
        &preview.domain_id,
        &preview.request_digest.0,
        &preview.recipient_identity,
        0,
    )
    .is_err());
    let consent = read_experimental_routed_advisor_consent(&catalog, &preview.domain_id).unwrap();
    assert_eq!(consent.revision, 0);
    assert!(!consent.enabled);
}

#[test]
#[cfg(windows)]
fn local_shadow_grant_cannot_be_redirected_to_a_replacement_store() {
    let repo = repo();
    let catalog = Catalog::open(&repo.path().join(".git/catalog.db")).unwrap();
    let plan =
        preview_experimental_routed_flow(&catalog, input(repo.path()), shadow_mission).unwrap();
    let reviewed = save_reviewed_flow_plan(&catalog, plan).unwrap();
    let preview = preview_experimental_routed_advisor_packet(&catalog, &reviewed.draft_id).unwrap();
    let granted = enable_experimental_routed_advisor_consent(
        &catalog,
        &reviewed.draft_id,
        &preview.domain_id,
        &preview.request_digest.0,
        &preview.recipient_identity,
        0,
    )
    .unwrap();
    let store_path = PytxoConfig::default().db_path_at(repo.path());
    let original_path = repo.path().join("original-consent-store.db");
    let replacement_path = repo.path().join("replacement-consent-store.db");
    drop(PytxoStore::open(&replacement_path).unwrap());
    fs::rename(&store_path, &original_path).unwrap();
    fs::rename(&replacement_path, &store_path).unwrap();

    assert!(read_experimental_routed_advisor_consent(&catalog, &preview.domain_id).is_err());
    assert!(revoke_experimental_routed_advisor_consent(
        &catalog,
        &preview.domain_id,
        granted.revision,
    )
    .is_err());
    assert!(
        PytxoStore::open_existing_read_only(&original_path)
            .unwrap()
            .routing_advisor_consent(&DomainId(preview.domain_id))
            .unwrap()
            .enabled
    );
}

#[test]
#[cfg(windows)]
fn local_shadow_grant_cannot_rebind_after_a_store_config_change() {
    let repo = repo();
    let catalog = Catalog::open(&repo.path().join(".git/catalog.db")).unwrap();
    let first =
        preview_experimental_routed_flow(&catalog, input(repo.path()), shadow_mission).unwrap();
    let reviewed = save_reviewed_flow_plan(&catalog, first).unwrap();
    let packet = preview_experimental_routed_advisor_packet(&catalog, &reviewed.draft_id).unwrap();
    let original = enable_experimental_routed_advisor_consent(
        &catalog,
        &reviewed.draft_id,
        &packet.domain_id,
        &packet.request_digest.0,
        &packet.recipient_identity,
        0,
    )
    .unwrap();
    fs::write(
        repo.path().join("pytxo.toml"),
        "data_dir = '.pytxo/new-routing-store'\npermission_profile = 'orbit'\nmax_agents = 1\n",
    )
    .unwrap();
    for args in [
        vec!["add", "pytxo.toml"],
        vec![
            "-c",
            "user.name=Test",
            "-c",
            "user.email=test@pytxo.local",
            "commit",
            "-qm",
            "move Store",
        ],
    ] {
        let output = Command::new("git")
            .args(args)
            .current_dir(repo.path())
            .output()
            .unwrap();
        assert!(
            output.status.success(),
            "{}",
            String::from_utf8_lossy(&output.stderr)
        );
    }
    let mut second_input = input(repo.path());
    second_input.id = "routed-flow-second".into();
    let second = preview_experimental_routed_flow(&catalog, second_input, shadow_mission).unwrap();
    let reviewed_second = save_reviewed_flow_plan(&catalog, second).unwrap();
    let packet_second =
        preview_experimental_routed_advisor_packet(&catalog, &reviewed_second.draft_id).unwrap();
    assert!(enable_experimental_routed_advisor_consent(
        &catalog,
        &reviewed_second.draft_id,
        &packet_second.domain_id,
        &packet_second.request_digest.0,
        &packet_second.recipient_identity,
        0,
    )
    .is_err());
    let still_original =
        read_experimental_routed_advisor_consent(&catalog, &packet.domain_id).unwrap();
    assert_eq!(still_original.revision, original.revision);
    assert!(still_original.enabled);
}

#[test]
fn shadow_review_freezes_exact_request_bytes_in_authorized_policy() {
    let repo = repo();
    let catalog = Catalog::open(&repo.path().join(".git/catalog.db")).unwrap();
    let plan =
        preview_experimental_routed_flow(&catalog, input(repo.path()), shadow_mission).unwrap();
    let preview = preview_experimental_routed_advisor_packet(&catalog, &plan.draft_id).unwrap();
    let store =
        PytxoStore::open_existing_read_only(&PytxoConfig::default().db_path_at(repo.path()))
            .unwrap();
    let mission = store
        .load_staged_routing_mission(&staged_ref(&plan))
        .unwrap();
    assert_eq!(
        mission.policy.advice_template,
        format!(
            "{}:{}",
            pytxo_orchestrate::flow::reviewed_routing_advisor_identity(),
            preview.request_digest.0
        )
    );
    assert_eq!(
        mission.authorization.policy_digest,
        mission.policy.digest().unwrap()
    );
}

#[test]
fn shadow_packet_preview_withholds_sensitive_reviewed_goal() {
    let repo = repo();
    let catalog = Catalog::open(&repo.path().join(".git/catalog.db")).unwrap();
    let mut request = input(repo.path());
    request.mission_text = "Update src/lib.rs to send Alice Smith salary 90000 to Bob".into();
    let plan = preview_experimental_routed_flow(&catalog, request, shadow_mission).unwrap();
    assert_eq!(
        plan.tasks[0].prompt,
        "Update src/lib.rs to send Alice Smith salary 90000 to Bob"
    );
    let preview = preview_experimental_routed_advisor_packet(&catalog, &plan.draft_id).unwrap();
    let body = String::from_utf8(preview.request_body).unwrap();
    for private in ["Alice", "Smith", "salary", "90000", "Bob"] {
        assert!(!body.contains(private), "packet leaked {private}");
    }
}

#[test]
fn shadow_review_rejects_wrong_exact_request_digest() {
    let repo = repo();
    let catalog = Catalog::open(&repo.path().join(".git/catalog.db")).unwrap();
    assert!(preview_experimental_routed_flow(
        &catalog,
        input(repo.path()),
        |plan, run_id, plan_digest| {
            let mut mission = shadow_mission(plan, run_id, plan_digest)?;
            mission.policy.advice_template = format!(
                "{}:{}",
                pytxo_orchestrate::flow::reviewed_routing_advisor_identity(),
                Digest::of_bytes(b"different outbound request").0
            );
            mission.authorization.policy_digest = mission.policy.digest()?;
            Ok(mission)
        },
    )
    .is_err());
}

#[test]
fn shadow_review_rejects_a_different_disclosure_scope() {
    let repo = repo();
    let catalog = Catalog::open(&repo.path().join(".git/catalog.db")).unwrap();
    assert!(preview_experimental_routed_flow(
        &catalog,
        input(repo.path()),
        |plan, run_id, plan_digest| {
            let mut mission = shadow_mission(plan, run_id, plan_digest)?;
            mission.policy.disclosure_scope_digest = Some(Digest::of_bytes(b"other-recipient"));
            mission.authorization.policy_digest = mission.policy.digest()?;
            Ok(mission)
        },
    )
    .is_err());
}

#[test]
fn advisor_packet_preview_rejects_rules_and_stale_public_or_private_review() {
    let repo = repo();
    let catalog = Catalog::open(&repo.path().join(".git/catalog.db")).unwrap();
    let rules = preview_experimental_routed_flow(&catalog, input(repo.path()), mission).unwrap();
    assert!(preview_experimental_routed_advisor_packet(&catalog, &rules.draft_id).is_err());
    assert!(preview_proposed_hosted_advisor_packet(&catalog, &rules.draft_id).is_err());
    let shadow =
        preview_experimental_routed_flow(&catalog, input(repo.path()), shadow_mission).unwrap();
    assert!(preview_experimental_routed_advisor_packet(&catalog, &shadow.draft_id).is_ok());
    let store_path = PytxoConfig::default().db_path_at(repo.path());
    let db = rusqlite::Connection::open(&store_path).unwrap();
    db.execute(
        "DELETE FROM routing_mission_stages WHERE run_id=?1",
        [&shadow.routing.as_ref().unwrap().authorization.run_id.0],
    )
    .unwrap();
    assert!(preview_experimental_routed_advisor_packet(&catalog, &shadow.draft_id).is_err());
    assert!(preview_proposed_hosted_advisor_packet(&catalog, &shadow.draft_id).is_err());
    drop(db);
    let rebuilt =
        preview_experimental_routed_flow(&catalog, input(repo.path()), shadow_mission).unwrap();
    let mut changed = rebuilt.clone();
    changed.tasks[0].prompt.push_str(" altered");
    let catalog_db = rusqlite::Connection::open(repo.path().join(".git/catalog.db")).unwrap();
    catalog_db
        .execute(
            "UPDATE flow_drafts SET plan_json=?1 WHERE id=?2",
            rusqlite::params![serde_json::to_string(&changed).unwrap(), rebuilt.draft_id],
        )
        .unwrap();
    assert!(preview_experimental_routed_advisor_packet(&catalog, &rebuilt.draft_id).is_err());
    assert!(preview_proposed_hosted_advisor_packet(&catalog, &rebuilt.draft_id).is_err());
}

#[test]
fn shadow_review_rejects_previous_wire_template() {
    let repo = repo();
    let catalog = Catalog::open(&repo.path().join(".git/catalog.db")).unwrap();
    assert!(preview_experimental_routed_flow(
        &catalog,
        input(repo.path()),
        |plan, run_id, plan_digest| {
            let mut mission = shadow_mission(plan, run_id, plan_digest)?;
            // The provider question key is unchanged. The reviewed template
            // identity must still change when outbound bytes can change.
            mission.policy.advice_template = pytxo_planner::advisor::TEMPLATE_VERSION.into();
            mission.authorization.policy_digest = mission.policy.digest()?;
            Ok(mission)
        },
    )
    .is_err());
}

#[test]
fn persisted_shadow_review_from_before_recipient_binding_requires_fresh_review() {
    let repo = repo();
    let catalog = Catalog::open(&repo.path().join(".git/catalog.db")).unwrap();
    let plan =
        preview_experimental_routed_flow(&catalog, input(repo.path()), shadow_mission).unwrap();
    let mut saved = save_reviewed_flow_plan(&catalog, plan).unwrap();
    let current = preview_experimental_routed_advisor_packet(&catalog, &saved.draft_id).unwrap();
    let store_path = PytxoConfig::default().db_path_at(repo.path());
    let store = PytxoStore::open_existing_read_only(&store_path).unwrap();
    let mut old_mission = store
        .load_staged_routing_mission(&staged_ref(&saved))
        .unwrap();
    old_mission.policy.advice_template = format!(
        "{}:coarse_goal_v2:{}",
        pytxo_planner::advisor::request_template_identity(),
        current.request_digest.0
    );
    old_mission.policy.disclosure_scope_digest = None;
    old_mission.authorization.policy_digest = old_mission.policy.digest().unwrap();
    let old_digest = canonical_digest(&old_mission, 1).unwrap();
    let review = saved.routing.as_mut().unwrap();
    review.authorization.policy_digest = old_mission.authorization.policy_digest.clone();
    review.mission_digest = old_digest.clone();

    // Simulate a previously persisted, internally consistent Shadow review.
    // Only the newly required recipient portion of the policy is absent.
    let private_db = rusqlite::Connection::open(&store_path).unwrap();
    private_db
        .execute(
            "UPDATE routing_mission_stages SET mission_digest=?1,mission_json=?2 WHERE run_id=?3",
            rusqlite::params![
                old_digest.0,
                serde_json::to_string(&old_mission).unwrap(),
                old_mission.authorization.run_id.0
            ],
        )
        .unwrap();
    let catalog_db = rusqlite::Connection::open(repo.path().join(".git/catalog.db")).unwrap();
    catalog_db
        .execute(
            "UPDATE flow_drafts SET plan_json=?1 WHERE id=?2",
            rusqlite::params![serde_json::to_string(&saved).unwrap(), saved.draft_id],
        )
        .unwrap();

    assert!(catalog.get_flow_draft(&saved.draft_id).unwrap().is_some());
    assert!(preview_experimental_routed_advisor_packet(&catalog, &saved.draft_id).is_err());
    assert!(dispatch_experimental_routed_flow(&catalog, &saved.draft_id).is_err());
    assert!(store
        .get_run(&old_mission.authorization.run_id.0)
        .unwrap()
        .is_none());
    let requests: i64 = private_db
        .query_row("SELECT COUNT(*) FROM routing_advisor_requests", [], |row| {
            row.get(0)
        })
        .unwrap();
    assert_eq!(requests, 0);

    let replacement =
        preview_experimental_routed_flow(&catalog, input(repo.path()), shadow_mission).unwrap();
    assert!(preview_experimental_routed_advisor_packet(&catalog, &replacement.draft_id).is_ok());
}

#[test]
fn routed_preview_binds_exact_two_wave_dependency_graph() {
    let repo = repo();
    fs::write(repo.path().join("seed.txt"), "").unwrap();
    fs::write(repo.path().join("result.txt"), "").unwrap();
    fs::write(
        repo.path().join("pytxo.toml"),
        "permission_profile = 'orbit'\nexecution_backend = 'subprocess'\nmax_agents = 1\n[blast]\nprefer_kernel_overlay = false\n[[task]]\nid = 'seed'\nagent = 'default'\npaths = ['seed.txt']\nverify = ['if exist seed.txt (exit /b 0) else (exit /b 1)']\n[[task]]\nid = 'result'\nagent = 'default'\npaths = ['result.txt']\ndepends_on = ['seed']\nverify = ['if exist result.txt (exit /b 0) else (exit /b 1)']\n",
    )
    .unwrap();
    for args in [
        vec!["add", "."],
        vec![
            "-c",
            "user.name=Test",
            "-c",
            "user.email=test@pytxo.local",
            "commit",
            "-qm",
            "two-wave fixture",
        ],
    ] {
        let output = Command::new("git")
            .args(args)
            .current_dir(repo.path())
            .output()
            .unwrap();
        assert!(
            output.status.success(),
            "{}",
            String::from_utf8_lossy(&output.stderr)
        );
    }
    let catalog = Catalog::open(&repo.path().join(".git/catalog.db")).unwrap();
    let mut request = input(repo.path());
    request.mission_text =
        "pytxo-local-fixture-v1:write-seed;pytxo-local-fixture-v1:write-result".into();
    request.verification_commands.clear();
    let plan = preview_experimental_routed_flow(&catalog, request, mission).unwrap();
    let mut shadow_request = input(repo.path());
    shadow_request.mission_text =
        "pytxo-local-fixture-v1:write-seed;pytxo-local-fixture-v1:write-result".into();
    shadow_request.verification_commands.clear();
    assert!(preview_experimental_routed_flow(&catalog, shadow_request, shadow_mission).is_err());
    assert_eq!(plan.tasks.len(), 2);
    assert!(preview_experimental_routed_advisor_packet(&catalog, &plan.draft_id).is_err());
    assert_eq!(
        plan.waves,
        [vec!["seed".to_owned()], vec!["result".to_owned()]]
    );
    assert_eq!(plan.tasks[1].dependencies, ["seed"]);
    let reviewed = plan.routing.as_ref().unwrap();
    assert_eq!(reviewed.authorization.allowed_task_digests.len(), 2);
    let stored = PytxoStore::open(&PytxoConfig::default().db_path_at(repo.path())).unwrap();
    let registered = stored
        .load_staged_routing_mission(&staged_ref(&plan))
        .unwrap();
    assert_eq!(
        registered.tasks[1].contract.dependencies,
        [TaskId("seed".into())]
    );
}

#[test]
fn routed_preview_binds_two_disjoint_siblings_in_one_wave() {
    let repo = repo();
    fs::write(repo.path().join("seed.txt"), "").unwrap();
    fs::write(repo.path().join("result.txt"), "").unwrap();
    fs::write(
        repo.path().join("pytxo.toml"),
        "permission_profile = 'orbit'\nexecution_backend = 'subprocess'\nmax_agents = 2\n[blast]\nprefer_kernel_overlay = false\n[[task]]\nid = 'seed'\nagent = 'default'\npaths = ['seed.txt']\nverify = ['if exist seed.txt (exit /b 0) else (exit /b 1)']\n[[task]]\nid = 'result'\nagent = 'default'\npaths = ['result.txt']\nverify = ['if exist result.txt (exit /b 0) else (exit /b 1)']\n",
    )
    .unwrap();
    for args in [
        vec!["add", "."],
        vec![
            "-c",
            "user.name=Test",
            "-c",
            "user.email=test@pytxo.local",
            "commit",
            "-qm",
            "sibling fixture",
        ],
    ] {
        let output = Command::new("git")
            .args(args)
            .current_dir(repo.path())
            .output()
            .unwrap();
        assert!(
            output.status.success(),
            "{}",
            String::from_utf8_lossy(&output.stderr)
        );
    }
    let catalog = Catalog::open(&repo.path().join(".git/catalog.db")).unwrap();
    let mut request = input(repo.path());
    request.mission_text =
        "pytxo-local-fixture-v1:write-seed;pytxo-local-fixture-v1:write-result".into();
    request.max_workers = Some(2);
    request.verification_commands.clear();
    let plan = preview_experimental_routed_flow(&catalog, request, mission).unwrap();
    assert_eq!(plan.max_workers, 2);
    assert_eq!(plan.waves, [vec!["seed".to_owned(), "result".to_owned()]]);
    assert!(plan.tasks.iter().all(|task| task.dependencies.is_empty()));
    let stored = PytxoStore::open(&PytxoConfig::default().db_path_at(repo.path())).unwrap();
    let registered = stored
        .load_staged_routing_mission(&staged_ref(&plan))
        .unwrap();
    assert!(registered
        .tasks
        .iter()
        .all(|task| task.contract.dependencies.is_empty()));
}

#[test]
fn routed_preview_stages_private_mission_durably_without_a_run_row() {
    let repo = repo();
    let catalog_path = repo.path().join(".git/catalog.db");
    let catalog = Catalog::open(&catalog_path).unwrap();
    let plan = preview_experimental_routed_flow(&catalog, input(repo.path()), mission).unwrap();
    let expected = mission(
        &plan,
        &plan.routing.as_ref().unwrap().authorization.run_id,
        &plan.routing.as_ref().unwrap().authorization.plan_digest,
    )
    .unwrap();
    drop(catalog);

    let reopened_catalog = Catalog::open(&catalog_path).unwrap();
    let saved = reopened_catalog
        .get_flow_draft(&plan.draft_id)
        .unwrap()
        .unwrap();
    let public_json = saved.plan_json.unwrap();
    assert!(!public_json.contains("credential_reference"));
    assert!(!public_json.contains("private-fixture-reference"));
    assert!(!public_json.contains("private-fixture-owner"));
    let saved_plan: FlowPlan = serde_json::from_str(&public_json).unwrap();
    assert_eq!(saved_plan.routing, plan.routing);

    let store_path = PytxoConfig::default().db_path_at(repo.path());
    let store = PytxoStore::open(&store_path).unwrap();
    assert_eq!(
        store
            .load_staged_routing_mission(&staged_ref(&saved_plan))
            .unwrap(),
        expected
    );
    assert!(store
        .get_run(&staged_ref(&saved_plan).run_id.0)
        .unwrap()
        .is_none());
    let private_json: String = rusqlite::Connection::open(store_path)
        .unwrap()
        .query_row(
            "SELECT mission_json FROM routing_mission_stages WHERE run_id=?1",
            [&staged_ref(&saved_plan).run_id.0],
            |row| row.get(0),
        )
        .unwrap();
    assert!(private_json.contains("private-fixture-reference"));
}

#[test]
fn routed_preview_rejects_a_builder_supplied_fictitious_git_base() {
    let repo = repo();
    let catalog = Catalog::open(&repo.path().join(".git/catalog.db")).unwrap();
    let error =
        preview_experimental_routed_flow(&catalog, input(repo.path()), |plan, run, digest| {
            let mut registration = mission(plan, run, digest)?;
            registration.tasks[0].contract.base.snapshot_digest =
                Digest::of_bytes(b"invented tree");
            registration.authorization.allowed_task_digests =
                BTreeSet::from([registration.tasks[0].contract.digest()?]);
            Ok(registration)
        })
        .unwrap_err();
    assert!(error.to_string().contains("Git base snapshot"), "{error:#}");
    assert!(catalog.get_flow_draft("routed-flow").unwrap().is_none());
}

#[test]
fn routed_preview_rejects_a_builder_supplied_fictitious_commit() {
    let repo = repo();
    let catalog = Catalog::open(&repo.path().join(".git/catalog.db")).unwrap();
    let error =
        preview_experimental_routed_flow(&catalog, input(repo.path()), |plan, run, digest| {
            let mut registration = mission(plan, run, digest)?;
            registration.tasks[0].contract.base.git_revision = "0".repeat(40);
            registration.authorization.allowed_task_digests =
                BTreeSet::from([registration.tasks[0].contract.digest()?]);
            Ok(registration)
        })
        .unwrap_err();
    assert!(error.to_string().contains("Git base snapshot"), "{error:#}");
    assert!(catalog.get_flow_draft("routed-flow").unwrap().is_none());
}

#[test]
fn routed_preview_rejects_a_profile_backend_outside_the_reviewed_transport() {
    let repo = repo();
    let catalog = Catalog::open(&repo.path().join(".git/catalog.db")).unwrap();
    let error =
        preview_experimental_routed_flow(&catalog, input(repo.path()), |plan, run, digest| {
            let mut registration = mission(plan, run, digest)?;
            registration.profiles[0].profile.backend = ExecutionBackend::Subprocess;
            registration.profiles[0].binding.profile_digest =
                registration.profiles[0].profile.digest()?;
            registration.authorization.allowed_profiles = registration
                .profiles
                .iter()
                .map(|entry| ApprovedProfile {
                    target: RouteTarget {
                        profile_id: entry.profile.id.clone(),
                        binding_id: entry.binding.id.clone(),
                    },
                    profile_digest: entry.profile.digest().unwrap(),
                    binding_digest: entry.binding.digest().unwrap(),
                })
                .collect();
            Ok(registration)
        })
        .unwrap_err();
    assert!(error.to_string().contains("execution backend"), "{error:#}");
    assert!(catalog.get_flow_draft("routed-flow").unwrap().is_none());
}

#[test]
fn routed_dispatch_rejects_a_new_head_before_claiming_the_reviewed_run() {
    let repo = repo();
    let catalog = Catalog::open(&repo.path().join(".git/catalog.db")).unwrap();
    let plan = preview_experimental_routed_flow(&catalog, input(repo.path()), mission).unwrap();
    let reviewed = save_reviewed_flow_plan(&catalog, plan).unwrap();
    let original_tree = git_oid(repo.path(), "HEAD^{tree}");
    assert!(Command::new("git")
        .args([
            "-c",
            "user.name=Test",
            "-c",
            "user.email=test@pytxo.local",
            "commit",
            "--allow-empty",
            "-qm",
            "new commit on the same tree",
        ])
        .current_dir(repo.path())
        .status()
        .unwrap()
        .success());
    assert_eq!(git_oid(repo.path(), "HEAD^{tree}"), original_tree);
    let error = dispatch_experimental_routed_flow(&catalog, &reviewed.draft_id).unwrap_err();
    assert!(error.to_string().contains("Git base snapshot"), "{error:#}");
    let draft = catalog.get_flow_draft(&reviewed.draft_id).unwrap().unwrap();
    assert_eq!(draft.status, "ready");
    assert!(draft.dispatched_run_id.is_none());
    assert!(
        PytxoStore::open(&PytxoConfig::default().db_path_at(repo.path()))
            .unwrap()
            .get_run(&staged_ref(&reviewed).run_id.0)
            .unwrap()
            .is_none()
    );
}

#[test]
fn replacing_a_routed_preview_leaves_an_inert_orphan_stage() {
    let repo = repo();
    let catalog = Catalog::open(&repo.path().join(".git/catalog.db")).unwrap();
    let old = preview_experimental_routed_flow(&catalog, input(repo.path()), mission).unwrap();
    let fresh = preview_experimental_routed_flow(&catalog, input(repo.path()), mission).unwrap();
    assert_ne!(staged_ref(&old).run_id, staged_ref(&fresh).run_id);
    let store = PytxoStore::open(&PytxoConfig::default().db_path_at(repo.path())).unwrap();
    assert!(store.load_staged_routing_mission(&staged_ref(&old)).is_ok());
    assert!(store
        .load_staged_routing_mission(&staged_ref(&fresh))
        .is_ok());
    assert!(dispatch_experimental_routed_flow(&catalog, &fresh.draft_id).is_err());
    assert!(store.get_run(&staged_ref(&old).run_id.0).unwrap().is_none());
    assert!(store
        .routing_history(&RoutingScope {
            domain_id: staged_ref(&old).domain_id,
            run_id: staged_ref(&old).run_id,
        })
        .unwrap()
        .is_none());
    assert_eq!(
        store
            .get_run(&staged_ref(&fresh).run_id.0)
            .unwrap()
            .unwrap()
            .status,
        "failed_startup"
    );
}

#[test]
fn missing_or_corrupt_private_stage_cannot_claim_ready_routed_flow() {
    for mutation in [
        "DELETE FROM routing_mission_stages",
        "UPDATE routing_mission_stages SET mission_json='{}'",
        "UPDATE routing_mission_stages SET mission_digest='stale'",
    ] {
        let repo = repo();
        let catalog = Catalog::open(&repo.path().join(".git/catalog.db")).unwrap();
        let plan = preview_experimental_routed_flow(&catalog, input(repo.path()), mission).unwrap();
        let store_path = PytxoConfig::default().db_path_at(repo.path());
        rusqlite::Connection::open(&store_path)
            .unwrap()
            .execute(mutation, [])
            .unwrap();
        let error = dispatch_experimental_routed_flow(&catalog, &plan.draft_id).unwrap_err();
        assert!(error.to_string().contains("routing stage"), "{error:#}");
        let saved = catalog.get_flow_draft(&plan.draft_id).unwrap().unwrap();
        assert_eq!(saved.status, "ready");
        assert!(saved.dispatched_run_id.is_none());
        let store = PytxoStore::open(&store_path).unwrap();
        assert!(store
            .get_run(&staged_ref(&plan).run_id.0)
            .unwrap()
            .is_none());
        assert!(!repo.path().join(".pytxo/data/active_run.json").exists());
    }
}

#[test]
fn changed_review_digest_cannot_use_an_older_private_stage() {
    let repo = repo();
    let catalog = Catalog::open(&repo.path().join(".git/catalog.db")).unwrap();
    let plan = preview_experimental_routed_flow(&catalog, input(repo.path()), mission).unwrap();
    let before = catalog
        .get_flow_draft(&plan.draft_id)
        .unwrap()
        .unwrap()
        .plan_json
        .unwrap();
    let mut forged = plan.clone();
    forged.routing.as_mut().unwrap().mission_digest = Digest::of_bytes(b"forged");
    assert!(catalog
        .replace_ready_flow_plan(
            &plan.draft_id,
            &before,
            &serde_json::to_string(&forged).unwrap()
        )
        .unwrap());
    let error = dispatch_experimental_routed_flow(&catalog, &plan.draft_id).unwrap_err();
    assert!(error.to_string().contains("routing stage"), "{error:#}");
    assert_eq!(
        catalog
            .get_flow_draft(&plan.draft_id)
            .unwrap()
            .unwrap()
            .status,
        "ready"
    );
    assert!(
        PytxoStore::open(&PytxoConfig::default().db_path_at(repo.path()))
            .unwrap()
            .get_run(&staged_ref(&plan).run_id.0)
            .unwrap()
            .is_none()
    );
}

#[test]
fn routed_preview_freezes_core_run_identity_and_review_cannot_replace_it() {
    let repo = repo();
    let catalog = Catalog::open(&repo.path().join(".git/catalog.db")).unwrap();
    let plan = preview_experimental_routed_flow(&catalog, input(repo.path()), mission).unwrap();
    assert_eq!(plan.status, FlowStatus::Ready, "{:?}", plan.blocked_reasons);
    let routed = plan.routing.as_ref().unwrap();
    assert_eq!(routed.authorization.run_id.0.len(), 36);
    let persisted = catalog.get_flow_draft(&plan.draft_id).unwrap().unwrap();
    let public_json = persisted.plan_json.as_deref().unwrap();
    assert!(!public_json.contains("credential_reference"));
    assert!(!public_json.contains("private-fixture-reference"));
    assert!(!public_json.contains("auth_owner"));
    assert!(!public_json.contains("private-fixture-owner"));
    assert!(!public_json.contains("executable"));
    let persisted: FlowPlan =
        serde_json::from_str(persisted.plan_json.as_deref().unwrap()).unwrap();
    assert_eq!(persisted.routing, plan.routing);
    assert!(!repo.path().join(".pytxo/data/active_run.json").exists());
    let store = PytxoStore::open(&PytxoConfig::default().db_path_at(repo.path())).unwrap();
    assert!(store
        .get_run(&routed.authorization.run_id.0)
        .unwrap()
        .is_none());

    let mut changed = plan.clone();
    changed.routing.as_mut().unwrap().authorization.run_id = RunId::new();
    assert!(save_reviewed_flow_plan(&catalog, changed).is_err());
    let mut removed = plan.clone();
    removed.routing = None;
    assert!(save_reviewed_flow_plan(&catalog, removed).is_err());
    let mut edited = plan.clone();
    edited.tasks[0].prompt.push_str(" and use a different goal");
    assert!(save_reviewed_flow_plan(&catalog, edited)
        .unwrap_err()
        .to_string()
        .contains("new experimental preview"));
    let mut edited_checks = plan.clone();
    edited_checks.tasks[0].verify[0].push_str(" --different");
    assert!(save_reviewed_flow_plan(&catalog, edited_checks).is_err());
    assert!(dispatch_flow(&catalog, &plan.draft_id).is_err());
    assert_eq!(
        catalog
            .get_flow_draft(&plan.draft_id)
            .unwrap()
            .unwrap()
            .status,
        "ready"
    );
}

#[tokio::test(flavor = "current_thread")]
async fn unavailable_routed_runtime_settles_exact_run_without_attempt_or_retry() {
    let repo = repo();
    let catalog = Catalog::open(&repo.path().join(".git/catalog.db")).unwrap();
    let plan = preview_experimental_routed_flow(&catalog, input(repo.path()), mission).unwrap();
    let reviewed = save_reviewed_flow_plan(&catalog, plan).unwrap();
    let routed = reviewed.routing.unwrap();
    let run_id = routed.authorization.run_id.0;
    drop(catalog);
    // Reopened Catalog/Store handles use only persisted review and stage data;
    // this does not exercise a separate OS controller process.
    let catalog = Catalog::open(&repo.path().join(".git/catalog.db")).unwrap();
    let error = dispatch_experimental_routed_flow(&catalog, &reviewed.draft_id).unwrap_err();
    assert!(
        error.to_string().contains("routed execution unavailable"),
        "{error:#}"
    );
    let store = PytxoStore::open(&PytxoConfig::default().db_path_at(repo.path())).unwrap();
    let row = store
        .get_run(&run_id)
        .unwrap()
        .expect("exact reviewed run exists");
    assert_eq!(row.status, "failed_startup");
    assert!(!repo.path().join(".pytxo/data/active_run.json").exists());
    assert!(store.list_agents_for_run(&run_id).unwrap().is_empty());
    let history = store
        .routing_history(&RoutingScope {
            domain_id: DomainId(reviewed.domain_id),
            run_id: RunId(run_id.clone()),
        })
        .unwrap()
        .expect("the exact private mission is registered after run claim");
    assert_eq!(history.mission.authorization.run_id.0, run_id);
    assert_eq!(
        canonical_digest(&history.mission, 1).unwrap(),
        routed.mission_digest
    );
    assert!(
        history.cancelled,
        "failed startup must revoke routing admission"
    );
    assert_eq!(history.cancel_epoch, 1);
    assert_eq!(history.tasks.len(), 1);
    assert_eq!(history.tasks[0].state, TaskRoutingState::Cancelled);
    assert!(history
        .events
        .iter()
        .any(|event| matches!(&event.event, RoutingControlEvent::Cancelled { .. })));
    assert!(history.attempts.is_empty());
    assert_eq!(
        catalog
            .get_flow_draft(&reviewed.draft_id)
            .unwrap()
            .unwrap()
            .status,
        "failed"
    );
    assert_eq!(
        catalog
            .get_flow_draft(&reviewed.draft_id)
            .unwrap()
            .unwrap()
            .dispatched_run_id
            .as_deref(),
        Some(run_id.as_str())
    );
    assert!(dispatch_experimental_routed_flow(&catalog, &reviewed.draft_id).is_err());
    assert_eq!(
        store.get_run(&run_id).unwrap().unwrap().status,
        "failed_startup"
    );
}

#[tokio::test(flavor = "current_thread")]
#[cfg(windows)]
async fn failed_routing_cancellation_preserves_active_ownership_for_recovery() {
    let repo = repo();
    let catalog = Catalog::open(&repo.path().join(".git/catalog.db")).unwrap();
    let plan = preview_experimental_routed_flow(&catalog, input(repo.path()), mission).unwrap();
    let run_id = staged_ref(&plan).run_id.0;
    let store_path = PytxoConfig::default().db_path_at(repo.path());
    rusqlite::Connection::open(&store_path)
        .unwrap()
        .execute_batch(
            "CREATE TRIGGER reject_cancel BEFORE UPDATE OF cancelled,cancel_epoch ON routing_missions
             BEGIN SELECT RAISE(ABORT, 'forced cancel failure'); END;",
        )
        .unwrap();

    let error = dispatch_experimental_routed_flow(&catalog, &plan.draft_id).unwrap_err();
    assert!(
        error.to_string().contains("routing cancellation"),
        "{error:#}"
    );
    let store = PytxoStore::open(&store_path).unwrap();
    assert_eq!(store.get_run(&run_id).unwrap().unwrap().status, "starting");
    assert!(repo.path().join(".pytxo/data/active_run.json").exists());
    assert_eq!(
        catalog
            .get_flow_draft(&plan.draft_id)
            .unwrap()
            .unwrap()
            .status,
        "recovery_required"
    );
    assert_eq!(
        catalog
            .get_flow_draft(&plan.draft_id)
            .unwrap()
            .unwrap()
            .dispatched_run_id
            .as_deref(),
        Some(run_id.as_str())
    );
    let history = store
        .routing_history(&RoutingScope {
            domain_id: DomainId(plan.domain_id.clone()),
            run_id: RunId(run_id.clone()),
        })
        .unwrap()
        .unwrap();
    assert!(history.attempts.is_empty());
    assert!(!history.cancelled);
    assert_eq!(history.tasks[0].state, TaskRoutingState::Ready);
    assert!(!reconcile_routed_flow_startup(&catalog, &plan.draft_id).unwrap());

    // Simulate a later stale-supervisor reconciliation that settles the run
    // without closing the registration/cancellation crash interval. Store and
    // Core must still reject admission from the inactive legacy run row.
    assert!(store
        .finish_run_if_status(
            &history.mission.authorization.run_id.0,
            "starting",
            "failed_startup"
        )
        .unwrap());
    let scope = RoutingScope {
        domain_id: history.mission.authorization.domain_id.clone(),
        run_id: history.mission.authorization.run_id.clone(),
    };
    let contract = &history.tasks[0].registration.contract;
    let facts = RoutingFacts {
        now_ms: 1,
        observed_at_ms: 1,
        expires_at_ms: 2,
        base: contract.base.clone(),
        plan_digest: contract.plan_digest.clone(),
        permission_profile: contract.permission_profile,
        observations: vec![],
        manual_target: None,
        packet_digest: None,
        advice_request_id: None,
    };
    let decision = store
        .preview_routing_decision(&scope, &contract.task_id, &facts, None)
        .unwrap();
    assert_eq!(
        decision.selection,
        RouteSelection::Blocked(RouteBlocker::ScopeDrift)
    );
    assert!(store
        .admit_routing_attempt(&AdmitRoutingAttempt {
            scope,
            event_id: "attempt-after-inactive-run".into(),
            task_id: contract.task_id.clone(),
            attempt_id: AttemptId("attempt-after-inactive-run".into()),
            agent_id: "agent-after-inactive-run".into(),
            facts,
            decision,
            capacity_reservation: "reservation-after-inactive-run".into(),
            input_manifest: BlobRef {
                digest: Digest::of_bytes(b"fixture input"),
                byte_length: 13,
            },
            handoff: None,
            advice_json: None,
            observation_event_id: None,
        })
        .is_err());
    rusqlite::Connection::open(&store_path)
        .unwrap()
        .execute_batch("DROP TRIGGER reject_cancel")
        .unwrap();
    pytxo_orchestrate::stop_exact(None, Some(repo.path().to_path_buf()), &run_id, false)
        .await
        .unwrap();
    assert!(list_flow_drafts_with_routed_recovery(&catalog)
        .unwrap()
        .iter()
        .any(|draft| draft.id == plan.draft_id && draft.status == "failed"));
    assert!(!reconcile_routed_flow_startup(&catalog, &plan.draft_id).unwrap());
    assert_eq!(
        catalog
            .get_flow_draft(&plan.draft_id)
            .unwrap()
            .unwrap()
            .status,
        "failed"
    );
}

#[tokio::test(flavor = "current_thread")]
#[cfg(windows)]
async fn failed_routing_registration_preserves_active_ownership_for_recovery() {
    let repo = repo();
    let catalog = Catalog::open(&repo.path().join(".git/catalog.db")).unwrap();
    let plan = preview_experimental_routed_flow(&catalog, input(repo.path()), mission).unwrap();
    let run_id = staged_ref(&plan).run_id.0;
    let store_path = PytxoConfig::default().db_path_at(repo.path());
    // The registration event updates the mission revision in the same Store
    // transaction. Force rollback at that point, after the run was claimed.
    rusqlite::Connection::open(&store_path)
        .unwrap()
        .execute_batch(
            "CREATE TRIGGER reject_registration BEFORE UPDATE OF revision ON routing_missions
             BEGIN SELECT RAISE(ABORT, 'forced registration failure'); END;",
        )
        .unwrap();

    let error = dispatch_experimental_routed_flow(&catalog, &plan.draft_id).unwrap_err();
    assert!(
        error.to_string().contains("registration failed"),
        "{error:#}"
    );
    let store = PytxoStore::open(&store_path).unwrap();
    assert_eq!(store.get_run(&run_id).unwrap().unwrap().status, "starting");
    assert!(repo.path().join(".pytxo/data/active_run.json").exists());
    assert_eq!(
        catalog
            .get_flow_draft(&plan.draft_id)
            .unwrap()
            .unwrap()
            .status,
        "recovery_required"
    );
    assert_eq!(
        catalog
            .get_flow_draft(&plan.draft_id)
            .unwrap()
            .unwrap()
            .dispatched_run_id
            .as_deref(),
        Some(run_id.as_str())
    );
    assert!(store
        .routing_history(&RoutingScope {
            domain_id: DomainId(plan.domain_id.clone()),
            run_id: RunId(run_id.clone()),
        })
        .unwrap()
        .is_none());
    assert!(!reconcile_routed_flow_startup(&catalog, &plan.draft_id).unwrap());
    rusqlite::Connection::open(&store_path)
        .unwrap()
        .execute_batch("DROP TRIGGER reject_registration")
        .unwrap();
    pytxo_orchestrate::stop_exact(None, Some(repo.path().to_path_buf()), &run_id, false)
        .await
        .unwrap();
    fs::write(
        repo.path().join("pytxo.toml"),
        "data_dir = '.pytxo/changed-after-claim'\n",
    )
    .unwrap();
    // Recovery follows the claim's original physical Store even if the live
    // configuration points elsewhere; it must not create a replacement Store.
    assert!(reconcile_routed_flow_startup(&catalog, &plan.draft_id).unwrap());
    assert!(!repo
        .path()
        .join(".pytxo/changed-after-claim/pytxo.db")
        .exists());
    assert_eq!(
        catalog
            .get_flow_draft(&plan.draft_id)
            .unwrap()
            .unwrap()
            .status,
        "failed"
    );
}

#[tokio::test(flavor = "current_thread")]
async fn routed_claim_failure_preserves_another_active_run() {
    let repo = repo();
    let catalog = Catalog::open(&repo.path().join(".git/catalog.db")).unwrap();
    let plan = preview_experimental_routed_flow(&catalog, input(repo.path()), mission).unwrap();
    let routed_id = plan
        .routing
        .as_ref()
        .unwrap()
        .authorization
        .run_id
        .0
        .clone();
    let store = PytxoStore::open(&PytxoConfig::default().db_path_at(repo.path())).unwrap();
    store
        .insert_starting_run_with_profile(
            "other-run",
            &repo.path().to_string_lossy(),
            Some("orbit"),
        )
        .unwrap();
    let marker = repo.path().join(".pytxo/data/active_run.json");
    fs::create_dir_all(marker.parent().unwrap()).unwrap();
    let identity = pytxo_runner::process_start_identity(std::process::id())
        .unwrap()
        .unwrap();
    let other_marker = serde_json::json!({
        "run_id": "other-run",
        "repo_root": repo.path().to_string_lossy(),
        "supervisor_pid": std::process::id(),
        "supervisor_start_identity": identity,
    })
    .to_string();
    fs::write(&marker, &other_marker).unwrap();

    let error = dispatch_experimental_routed_flow(&catalog, &plan.draft_id).unwrap_err();
    assert!(
        error.to_string().contains("already owns active run"),
        "{error:#}"
    );
    assert_eq!(fs::read_to_string(&marker).unwrap(), other_marker);
    assert_eq!(
        store.get_run("other-run").unwrap().unwrap().status,
        "starting"
    );
    assert_eq!(
        store.get_run(&routed_id).unwrap().unwrap().status,
        "failed_startup"
    );
    assert_eq!(
        catalog
            .get_flow_draft(&plan.draft_id)
            .unwrap()
            .unwrap()
            .status,
        "failed"
    );
    assert!(store.list_agents_for_run(&routed_id).unwrap().is_empty());
    assert!(store
        .routing_history(&RoutingScope {
            domain_id: DomainId(plan.domain_id),
            run_id: RunId(routed_id),
        })
        .unwrap()
        .is_none());
}

#[test]
fn experimental_dispatch_rejects_legacy_preview_without_claiming_it() {
    let repo = repo();
    let catalog = Catalog::open(&repo.path().join(".git/catalog.db")).unwrap();
    let routed = preview_experimental_routed_flow(&catalog, input(repo.path()), mission).unwrap();
    let before = catalog
        .get_flow_draft(&routed.draft_id)
        .unwrap()
        .unwrap()
        .plan_json
        .unwrap();
    let mut legacy = routed.clone();
    legacy.routing = None;
    // Persist a ready legacy-style preview to exercise the inverse gate without
    // depending on an installed vendor CLI or executing the legacy path.
    assert!(catalog
        .replace_ready_flow_plan(
            &routed.draft_id,
            &before,
            &serde_json::to_string(&legacy).unwrap()
        )
        .unwrap());
    let error = dispatch_experimental_routed_flow(&catalog, &legacy.draft_id).unwrap_err();
    assert!(error.to_string().contains("routed review"), "{error:#}");
    let draft = catalog.get_flow_draft(&legacy.draft_id).unwrap().unwrap();
    assert_eq!(draft.status, "ready");
    assert!(draft.dispatched_run_id.is_none());
}

#[test]
fn routed_preview_rejects_unsupported_multi_task_plan_before_claim() {
    let repo = repo();
    fs::write(repo.path().join("pytxo.toml"), "max_agents = 1\n[[task]]\nid = 'one'\nagent = 'default'\npaths = ['src/one.rs']\n[[task]]\nid = 'two'\nagent = 'default'\npaths = ['src/two.rs']\n").unwrap();
    for args in [
        vec!["add", "pytxo.toml"],
        vec![
            "-c",
            "user.name=Test",
            "-c",
            "user.email=test@pytxo.local",
            "commit",
            "-qm",
            "two tasks",
        ],
    ] {
        let output = Command::new("git")
            .args(args)
            .current_dir(repo.path())
            .output()
            .unwrap();
        assert!(
            output.status.success(),
            "{}",
            String::from_utf8_lossy(&output.stderr)
        );
    }
    let catalog = Catalog::open(&repo.path().join(".git/catalog.db")).unwrap();
    let mut request = input(repo.path());
    request.max_workers = Some(1);
    let error = preview_experimental_routed_flow(&catalog, request, mission).unwrap_err();
    assert!(error.to_string().contains("one task"), "{error:#}");
    assert!(catalog.get_flow_draft("routed-flow").unwrap().is_none());
}

#[test]
fn routed_preview_rejects_unbound_checks_even_when_count_matches() {
    let repo = repo();
    let catalog = Catalog::open(&repo.path().join(".git/catalog.db")).unwrap();
    let mut request = input(repo.path());
    request.verification_commands = vec!["cargo test --locked".into()];
    let error = preview_experimental_routed_flow(&catalog, request, |plan, run_id, digest| {
        let mut contract = mission(plan, run_id, digest)?;
        contract.tasks[0].contract.checks = vec![CheckRecipe {
            id: CheckId("different-check".into()),
            recipe_digest: Digest::of_bytes(b"different-command"),
        }];
        contract.authorization.allowed_task_digests =
            BTreeSet::from([contract.tasks[0].contract.digest()?]);
        Ok(contract)
    })
    .unwrap_err();
    assert!(error.to_string().contains("task contract"), "{error:#}");
    assert!(catalog.get_flow_draft("routed-flow").unwrap().is_none());
}

#[test]
fn routed_preview_binds_ordered_final_checks_before_private_stage() {
    let repo = repo();
    fs::write(
        repo.path().join("Cargo.toml"),
        "[package]\nname = \"fixture\"\nversion = \"0.1.0\"\n",
    )
    .unwrap();
    for args in [
        vec!["add", "Cargo.toml"],
        vec![
            "-c",
            "user.name=Test",
            "-c",
            "user.email=test@pytxo.local",
            "commit",
            "-qm",
            "add manifest",
        ],
    ] {
        assert!(Command::new("git")
            .args(args)
            .current_dir(repo.path())
            .status()
            .unwrap()
            .success());
    }
    let catalog = Catalog::open(&repo.path().join(".git/catalog.db")).unwrap();
    let mut request = input(repo.path());
    request.verification_commands = vec!["cargo test".into(), "echo extra".into()];
    let plan = preview_experimental_routed_flow(&catalog, request, mission).unwrap();
    assert_eq!(plan.tasks[0].verify, ["cargo test", "echo extra"]);
    let review = staged_ref(&plan);
    let store = PytxoStore::open(&PytxoConfig::default().db_path_at(repo.path())).unwrap();
    let frozen = store.load_staged_routing_mission(&review).unwrap();
    let task = &frozen.tasks[0];
    assert_eq!(task.check_recipes.len(), 2);
    assert_eq!(
        task.check_recipes[0].id.0,
        format!("{}:verify:0001", plan.tasks[0].id)
    );
    assert_eq!(task.check_recipes[1].command, "echo extra");
    assert_eq!(
        task.contract.checks[0],
        task.check_recipes[0].reference().unwrap()
    );
}

#[test]
fn routed_preview_rejects_self_consistent_but_changed_private_command() {
    let repo = repo();
    let catalog = Catalog::open(&repo.path().join(".git/catalog.db")).unwrap();
    let error =
        preview_experimental_routed_flow(&catalog, input(repo.path()), |plan, run, digest| {
            let mut changed = mission(plan, run, digest)?;
            changed.tasks[0].check_recipes[0].command = "echo substituted".into();
            changed.tasks[0].contract.checks[0] = changed.tasks[0].check_recipes[0].reference()?;
            changed.authorization.allowed_task_digests =
                BTreeSet::from([changed.tasks[0].contract.digest()?]);
            Ok(changed)
        })
        .unwrap_err();
    assert!(
        error.to_string().contains("routed task contract"),
        "{error:#}"
    );
    assert!(catalog.get_flow_draft("routed-flow").unwrap().is_none());
}

#[test]
fn routed_final_check_validator_rejects_unreviewable_lists() {
    for commands in [
        vec![],
        vec!["".into()],
        vec![" echo hi".into()],
        vec!["echo hi ".into()],
        vec!["echo hi\nother".into()],
        vec!["echo hi".into(), "echo hi".into()],
        vec!["x".repeat(4097)],
        vec!["echo hi".into(); 17],
    ] {
        assert!(
            pytxo_orchestrate::flow::freeze_experimental_routed_checks("task", &commands).is_err()
        );
    }
}

#[test]
fn edited_or_reordered_persisted_checks_cannot_claim_routed_dispatch() {
    for reorder in [false, true] {
        let repo = repo();
        let catalog = Catalog::open(&repo.path().join(".git/catalog.db")).unwrap();
        let mut request = input(repo.path());
        request.verification_commands = vec!["echo first".into(), "echo second".into()];
        let plan = preview_experimental_routed_flow(&catalog, request, mission).unwrap();
        let before = catalog
            .get_flow_draft(&plan.draft_id)
            .unwrap()
            .unwrap()
            .plan_json
            .unwrap();
        let mut forged = plan.clone();
        if reorder {
            forged.tasks[0].verify.swap(0, 1);
        } else {
            forged.tasks[0].verify[0] = "echo changed".into();
        }
        assert!(catalog
            .replace_ready_flow_plan(
                &plan.draft_id,
                &before,
                &serde_json::to_string(&forged).unwrap()
            )
            .unwrap());
        assert!(dispatch_experimental_routed_flow(&catalog, &plan.draft_id).is_err());
        assert_eq!(
            catalog
                .get_flow_draft(&plan.draft_id)
                .unwrap()
                .unwrap()
                .status,
            "ready"
        );
        let store = PytxoStore::open(&PytxoConfig::default().db_path_at(repo.path())).unwrap();
        assert!(store
            .get_run(&staged_ref(&plan).run_id.0)
            .unwrap()
            .is_none());
    }
}

#[test]
fn changed_frozen_shell_identity_fails_fresh_dispatch_probe_before_claim() {
    let repo = repo();
    let catalog = Catalog::open(&repo.path().join(".git/catalog.db")).unwrap();
    let plan = preview_experimental_routed_flow(&catalog, input(repo.path()), mission).unwrap();
    let store_path = PytxoConfig::default().db_path_at(repo.path());
    let store = PytxoStore::open(&store_path).unwrap();
    let mut forged_mission = store
        .load_staged_routing_mission(&staged_ref(&plan))
        .unwrap();
    forged_mission.tasks[0].check_recipes[0]
        .executor
        .shell
        .digest = Digest::of_bytes(b"a different shell file");
    forged_mission.tasks[0].contract.checks[0] = forged_mission.tasks[0].check_recipes[0]
        .reference()
        .unwrap();
    forged_mission.authorization.allowed_task_digests =
        BTreeSet::from([forged_mission.tasks[0].contract.digest().unwrap()]);
    let forged_digest = canonical_digest(&forged_mission, 1).unwrap();
    let conn = rusqlite::Connection::open(&store_path).unwrap();
    conn.execute(
        "UPDATE routing_mission_stages SET mission_json=?1,mission_digest=?2 WHERE run_id=?3",
        rusqlite::params![
            serde_json::to_string(&forged_mission).unwrap(),
            &forged_digest.0,
            &staged_ref(&plan).run_id.0,
        ],
    )
    .unwrap();
    let before = catalog
        .get_flow_draft(&plan.draft_id)
        .unwrap()
        .unwrap()
        .plan_json
        .unwrap();
    let mut forged_plan = plan.clone();
    forged_plan.routing.as_mut().unwrap().mission_digest = forged_digest;
    assert!(catalog
        .replace_ready_flow_plan(
            &plan.draft_id,
            &before,
            &serde_json::to_string(&forged_plan).unwrap()
        )
        .unwrap());
    let error = dispatch_experimental_routed_flow(&catalog, &plan.draft_id).unwrap_err();
    assert!(error.to_string().contains("task contract"), "{error:#}");
    assert_eq!(
        catalog
            .get_flow_draft(&plan.draft_id)
            .unwrap()
            .unwrap()
            .status,
        "ready"
    );
    assert!(store
        .get_run(&staged_ref(&plan).run_id.0)
        .unwrap()
        .is_none());
}

#[test]
fn routed_preview_rejects_non_orbit_or_multiple_worker_scope() {
    for (config, workers) in [
        ("permission_profile = 'galaxy'\n", 1),
        ("max_agents = 2\n", 2),
    ] {
        let repo = repo();
        fs::write(repo.path().join("pytxo.toml"), config).unwrap();
        for args in [
            vec!["add", "pytxo.toml"],
            vec![
                "-c",
                "user.name=Test",
                "-c",
                "user.email=test@pytxo.local",
                "commit",
                "-qm",
                "scope",
            ],
        ] {
            let output = Command::new("git")
                .args(args)
                .current_dir(repo.path())
                .output()
                .unwrap();
            assert!(
                output.status.success(),
                "{}",
                String::from_utf8_lossy(&output.stderr)
            );
        }
        let catalog = Catalog::open(&repo.path().join(".git/catalog.db")).unwrap();
        let mut request = input(repo.path());
        request.max_workers = Some(workers);
        let error = preview_experimental_routed_flow(&catalog, request, mission).unwrap_err();
        assert!(
            error
                .to_string()
                .contains("requires Orbit and the exact reviewed worker count"),
            "{error:#}"
        );
        assert!(catalog.get_flow_draft("routed-flow").unwrap().is_none());
    }
}

#[tokio::test(flavor = "current_thread")]
async fn routed_dispatch_rechecks_orbit_scope_before_claiming_draft() {
    let repo = repo();
    let catalog = Catalog::open(&repo.path().join(".git/catalog.db")).unwrap();
    let plan = preview_experimental_routed_flow(&catalog, input(repo.path()), mission).unwrap();
    let run_id = plan
        .routing
        .as_ref()
        .unwrap()
        .authorization
        .run_id
        .0
        .clone();
    fs::write(
        repo.path().join("pytxo.toml"),
        "permission_profile = 'galaxy'\n",
    )
    .unwrap();
    for args in [
        vec!["add", "pytxo.toml"],
        vec![
            "-c",
            "user.name=Test",
            "-c",
            "user.email=test@pytxo.local",
            "commit",
            "-qm",
            "raise permission",
        ],
    ] {
        let output = Command::new("git")
            .args(args)
            .current_dir(repo.path())
            .output()
            .unwrap();
        assert!(
            output.status.success(),
            "{}",
            String::from_utf8_lossy(&output.stderr)
        );
    }
    let error = dispatch_experimental_routed_flow(&catalog, &plan.draft_id).unwrap_err();
    assert!(
        error.to_string().contains("permission profile changed"),
        "{error:#}"
    );
    assert_eq!(
        catalog
            .get_flow_draft(&plan.draft_id)
            .unwrap()
            .unwrap()
            .status,
        "ready"
    );
    let store = PytxoStore::open(&PytxoConfig::default().db_path_at(repo.path())).unwrap();
    assert!(store.get_run(&run_id).unwrap().is_none());
    assert!(!repo.path().join(".pytxo/data/active_run.json").exists());
}

#[test]
fn routed_preview_rejects_deep_space_task_before_catalog_persistence() {
    let repo = repo();
    fs::write(
        repo.path().join("pytxo.toml"),
        "[[agent]]\nname = 'deep'\npermission_profile = 'deep_space'\n[[task]]\nid = 'one'\nagent = 'deep'\npaths = ['src/lib.rs']\n",
    )
    .unwrap();
    for args in [
        vec!["add", "pytxo.toml"],
        vec![
            "-c",
            "user.name=Test",
            "-c",
            "user.email=test@pytxo.local",
            "commit",
            "-qm",
            "agent override",
        ],
    ] {
        let output = Command::new("git")
            .args(args)
            .current_dir(repo.path())
            .output()
            .unwrap();
        assert!(
            output.status.success(),
            "{}",
            String::from_utf8_lossy(&output.stderr)
        );
    }
    let catalog = Catalog::open(&repo.path().join(".git/catalog.db")).unwrap();
    let error =
        preview_experimental_routed_flow(&catalog, input(repo.path()), mission).unwrap_err();
    assert!(
        error.to_string().contains("requires Orbit for every task"),
        "{error:#}"
    );
    assert!(catalog.get_flow_draft("routed-flow").unwrap().is_none());
    assert!(!repo.path().join(".pytxo/data/active_run.json").exists());
}

#[tokio::test(flavor = "current_thread")]
async fn routed_dispatch_rechecks_deep_space_agent_override_before_cas() {
    let repo = repo();
    let catalog = Catalog::open(&repo.path().join(".git/catalog.db")).unwrap();
    let plan = preview_experimental_routed_flow(&catalog, input(repo.path()), mission).unwrap();
    let run_id = plan
        .routing
        .as_ref()
        .unwrap()
        .authorization
        .run_id
        .0
        .clone();
    fs::write(
        repo.path().join("pytxo.toml"),
        format!(
            "[[agent]]\nname = '{}'\npermission_profile = 'deep_space'\n",
            plan.tasks[0].agent
        ),
    )
    .unwrap();
    for args in [
        vec!["add", "pytxo.toml"],
        vec![
            "-c",
            "user.name=Test",
            "-c",
            "user.email=test@pytxo.local",
            "commit",
            "-qm",
            "late agent override",
        ],
    ] {
        let output = Command::new("git")
            .args(args)
            .current_dir(repo.path())
            .output()
            .unwrap();
        assert!(
            output.status.success(),
            "{}",
            String::from_utf8_lossy(&output.stderr)
        );
    }
    let error = dispatch_experimental_routed_flow(&catalog, &plan.draft_id).unwrap_err();
    assert!(
        error.to_string().contains("requires Orbit for every task"),
        "{error:#}"
    );
    assert_eq!(
        catalog
            .get_flow_draft(&plan.draft_id)
            .unwrap()
            .unwrap()
            .status,
        "ready"
    );
    let store = PytxoStore::open(&PytxoConfig::default().db_path_at(repo.path())).unwrap();
    assert!(store.get_run(&run_id).unwrap().is_none());
    assert!(!repo.path().join(".pytxo/data/active_run.json").exists());
}
