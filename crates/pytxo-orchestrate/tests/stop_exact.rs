use std::collections::BTreeSet;
use std::fs;
use std::fs::OpenOptions;
use std::process::Command;
use std::time::Duration;

use fs2::FileExt;
use pytxo_core::routing::*;
use pytxo_core::{DomainId, ExecutionBackend, PermissionProfile, PytxoConfig, RunId, TaskId};
use pytxo_orchestrate::{dispatch, stop, stop_exact, trust_repo, RunOptions};
use pytxo_runner::{
    process_matches, process_start_identity, registry_path, ProcessEntry, ProcessRegistryFile,
};
use pytxo_store::{
    capacity::{
        CapacityOwner, CapacityPoolConfig, CapacityReleaseEvidence, CapacityReleaseEvidenceKind,
        CapacityReleaseRequest, CapacityReservationRequest, CapacityResourceRequest,
    },
    routing::{
        RegisteredProfile, RegisteredTask, RoutedAttemptRecord, RoutedUsage, RoutingMission,
        RoutingReceipts, RoutingScope,
    },
    routing_capacity_intent::RoutingCapacityIntentRequest,
    Catalog, PytxoStore,
};
use tempfile::tempdir;

const ISOLATED_SCENARIO_SENTINEL: &str = "PYTXO_STOP_EXACT_ISOLATED_SCENARIO";
const ISOLATED_RECOVERY_SENTINEL: &str = "PYTXO_STOP_RECOVERY_ISOLATED_SCENARIO";
const ISOLATED_CRASH_RECONCILE_SENTINEL: &str = "PYTXO_STOP_CRASH_RECONCILE_SCENARIO";
const MARKERLESS_RESTART_REPO: &str = "PYTXO_STOP_MARKERLESS_RESTART_REPO";
const CROSS_PROCESS_GATE_REPO: &str = "PYTXO_STOP_CROSS_PROCESS_GATE_REPO";
const CROSS_PROCESS_GATE_READY: &str = "PYTXO_STOP_CROSS_PROCESS_GATE_READY";
const MARKERLESS_CAPACITY_RESTART_REPO: &str = "PYTXO_STOP_MARKERLESS_CAPACITY_RESTART_REPO";
const RELEASE_CAPACITY_RESTART_REPO: &str = "PYTXO_STOP_RELEASE_CAPACITY_RESTART_REPO";

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
    if cfg!(windows) {
        fs::write(
            path.join("block.ps1"),
            "$child = Start-Process powershell -ArgumentList @('-NoProfile','-NonInteractive','-Command','Start-Sleep -Seconds 120') -PassThru\nSet-Content -Path 'child.pid' -Value $child.Id\nWait-Process -Id $child.Id\n",
        )
        .expect("write blocking PowerShell fixture");
    } else {
        fs::write(
            path.join("block.sh"),
            "#!/bin/sh\nsleep 120 &\necho $! > child.pid\nwait\n",
        )
        .expect("write blocking shell fixture");
    }
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
        "powershell -NoProfile -File block.ps1"
    } else {
        "sh block.sh"
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

fn with_capacity_test_gate<T>(repo: &std::path::Path, action: impl FnOnce() -> T) -> T {
    let data_dir = repo.join(PytxoConfig::default().data_dir);
    fs::create_dir_all(&data_dir).unwrap();
    let lock = OpenOptions::new()
        .create(true)
        .truncate(false)
        .read(true)
        .write(true)
        .open(data_dir.join("active_run.json.lock"))
        .unwrap();
    lock.lock_exclusive().unwrap();
    let result = action();
    drop(lock);
    result
}

fn issue_no_worker_reservation_once(
    repo: &std::path::Path,
    store: &PytxoStore,
    catalog: &Catalog,
    request: &RoutingCapacityIntentRequest,
) -> anyhow::Result<()> {
    with_capacity_test_gate(repo, || {
        let intent = store
            .capacity_intent(&request.reservation.reservation_id)?
            .ok_or_else(|| anyhow::anyhow!("capacity intent missing before reserve"))?;
        anyhow::ensure!(
            intent.phase == pytxo_store::routing_capacity_intent::CapacityIntentPhase::Prepared,
            "reserve may already have started; exact readback or recovery is required"
        );
        store.mark_capacity_reserve_may_have_started(
            &request.reservation.reservation_id,
            "reserve-may-start",
        )?;
        catalog.reserve_capacity(&request.reservation)?;
        Ok(())
    })
}

struct TrackedSleepChild {
    child: std::process::Child,
    pid: u32,
    identity: String,
}

impl Drop for TrackedSleepChild {
    fn drop(&mut self) {
        let _ = self.child.kill();
        let _ = self.child.wait();
    }
}

fn spawn_tracked_sleep_child(
    repo: &std::path::Path,
    run_id: &str,
    workspace: &std::path::Path,
) -> TrackedSleepChild {
    let child = if cfg!(windows) {
        Command::new("powershell")
            .args([
                "-NoProfile",
                "-NonInteractive",
                "-Command",
                "Start-Sleep -Seconds 120",
            ])
            .spawn()
            .unwrap()
    } else {
        Command::new("sh")
            .args(["-c", "sleep 120"])
            .spawn()
            .unwrap()
    };
    let pid = child.id();
    let identity = process_start_identity(pid)
        .unwrap()
        .expect("live child identity");
    ProcessRegistryFile::update(
        &registry_path(&repo.join(PytxoConfig::default().data_dir)),
        |registry| {
            registry.push(ProcessEntry {
                run_id: run_id.into(),
                repo_root: repo.to_string_lossy().into_owned(),
                agent_key: format!("{run_id}:fixture"),
                pid,
                start_identity: Some(identity.clone()),
                worktree_path: workspace.to_string_lossy().into_owned(),
                branch: "fixture".into(),
            });
            Ok(())
        },
    )
    .unwrap();
    TrackedSleepChild {
        child,
        pid,
        identity,
    }
}

fn registered_ready_mission(
    repo: &std::path::Path,
    run_id: &str,
) -> (RoutingMission, RoutingScope) {
    let domain_id = DomainId::from_repo_root(repo).expect("domain ID");
    let plan_digest = Digest::of_bytes(b"stop-test-plan");
    let profiles = ["everyday", "strong"]
        .into_iter()
        .map(|name| {
            let profile = ExecutionProfile {
                schema_version: 1,
                canonicalization_version: 1,
                id: ProfileId(name.into()),
                revision: 1,
                harness_id: "fixture".into(),
                adapter_contract_version: "1".into(),
                adapter_digest: Digest::of_bytes(b"fixture-adapter"),
                requested_model: ModelIdentity {
                    provider: "local".into(),
                    model: name.into(),
                    reasoning: None,
                    revision: None,
                },
                skill_tool_bundle_digest: Digest::of_bytes(b"fixture-tools"),
                backend: ExecutionBackend::Subprocess,
                capabilities: BTreeSet::from(["edit".into()]),
            };
            let binding = ProfileBinding {
                schema_version: 1,
                canonicalization_version: 1,
                id: BindingId(name.into()),
                revision: 1,
                profile_digest: profile.digest().unwrap(),
                credential_reference: None,
                auth_owner: "fixture-owner".into(),
                billing_source_id: BillingSourceId("local".into()),
                billing_mode: BillingSourceMode::Local,
                endpoint_identity: "fixture".into(),
                trust_class: "local".into(),
                capacity_pool_ids: BTreeSet::from(["host".into()]),
            };
            RegisteredProfile { profile, binding }
        })
        .collect::<Vec<_>>();
    let policy = RoutingPolicy {
        schema_version: 1,
        version: "stop-test-rules".into(),
        mode: RoutingMode::Rules,
        everyday: RouteTarget {
            profile_id: profiles[0].profile.id.clone(),
            binding_id: profiles[0].binding.id.clone(),
        },
        strong: RouteTarget {
            profile_id: profiles[1].profile.id.clone(),
            binding_id: profiles[1].binding.id.clone(),
        },
        evaluated_manifest_digest: None,
        everyday_threshold_ppm: 800_000,
        unclear_ceiling_ppm: 100_000,
        advice_model: "unused".into(),
        advice_template: "unused".into(),
        disclosure_scope_digest: None,
        advisor_recipient: None,
    };
    let task = RegisteredTask {
        contract: TaskContract {
            schema_version: 1,
            canonicalization_version: 1,
            task_id: TaskId("task".into()),
            revision: 1,
            plan_digest: plan_digest.clone(),
            base: BaseSnapshot {
                repository_identity: domain_id.0.clone(),
                git_revision: "fixture".into(),
                snapshot_digest: Digest::of_bytes(b"fixture-base"),
            },
            goal: "update fixture".into(),
            constraints: vec![],
            claim_roots: vec!["README.md".into()],
            dependencies: vec![],
            task_kind: Some(TaskKind::LocalTransformation),
            task_kind_evidence: Some(Digest::of_bytes(b"kind")),
            required_capabilities: BTreeSet::from(["edit".into()]),
            checks: vec![],
            required_resources: BTreeSet::from(["host".into()]),
            skill_tool_bundle_digest: Digest::of_bytes(b"fixture-tools"),
            permission_profile: PermissionProfile::Orbit,
            required_egress: BTreeSet::new(),
            required_target: None,
            strong_only: false,
            cross_component_requirement: Some(false),
            context_complete: true,
            repeatable_symptom_supplied: None,
            specific_cause_hypothesis_supplied: None,
        },
        attempt_budget_nano_usd: 0,
        check_recipes: vec![],
    };
    let authorization = MissionAuthorization {
        schema_version: 1,
        domain_id: domain_id.clone(),
        run_id: RunId(run_id.into()),
        plan_id: PlanId("stop-test-plan".into()),
        plan_digest,
        revision: 1,
        cancel_epoch: 0,
        allowed_task_digests: BTreeSet::from([task.contract.digest().unwrap()]),
        allowed_profiles: profiles
            .iter()
            .map(|entry| ApprovedProfile {
                target: RouteTarget {
                    profile_id: entry.profile.id.clone(),
                    binding_id: entry.binding.id.clone(),
                },
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
            max_workers: 1,
            max_attempts: 2,
            max_spend_nano_usd: None,
            spend_guarantee: SpendGuarantee::RiskBounded,
        },
        policy_digest: policy.digest().unwrap(),
        live_advice_authorized: false,
        consent_revision: 1,
    };
    let scope = RoutingScope {
        domain_id,
        run_id: authorization.run_id.clone(),
    };
    (
        RoutingMission {
            authorization,
            policy,
            tasks: vec![task],
            profiles,
        },
        scope,
    )
}

fn seed_registered_capacity_intent(
    repo: &std::path::Path,
    run_id: &str,
) -> (PytxoStore, Catalog, RoutingCapacityIntentRequest) {
    let config = PytxoConfig::default();
    let store = PytxoStore::open(&config.db_path_at(repo)).unwrap();
    store
        .insert_starting_run_with_profile(run_id, &repo.to_string_lossy(), Some("orbit"))
        .unwrap();
    let (mission, scope) = registered_ready_mission(repo, run_id);
    store.register_routing_mission(&mission).unwrap();
    let catalog = Catalog::open(&repo.join("capacity-catalog.db")).unwrap();
    catalog
        .configure_capacity_pool(&CapacityPoolConfig {
            resource_id: "fixture-host".into(),
            capacity_units: 1,
            expected_revision: None,
            configured_at_ms: 100,
        })
        .unwrap();
    let request = RoutingCapacityIntentRequest {
        scope: scope.clone(),
        task_id: TaskId("task".into()),
        reservation: CapacityReservationRequest {
            reservation_id: format!("reservation-{run_id}"),
            domain_id: scope.domain_id.0,
            run_id: scope.run_id.0,
            attempt_id: format!("attempt-{run_id}"),
            owner: CapacityOwner {
                process_id: 41,
                process_start_identity: "capacity-fixture-owner".into(),
            },
            resources: vec![CapacityResourceRequest {
                resource_id: "fixture-host".into(),
                units: 1,
            }],
            requested_at_ms: 101,
        },
        event_id: format!("intent-{run_id}"),
    };
    with_capacity_test_gate(repo, || {
        store.register_capacity_intent(&catalog, &request).unwrap();
    });
    (store, catalog, request)
}

/// Seed a durable ambiguous actor directly. The Stop test is about recovery
/// preflight, not admission or a claim that a worker was launched.
fn seed_unresolved_attempt(repo: &std::path::Path, scope: &RoutingScope, mission: &RoutingMission) {
    let selected = &mission.profiles[0];
    let target = RouteTarget {
        profile_id: selected.profile.id.clone(),
        binding_id: selected.binding.id.clone(),
    };
    let record = RoutedAttemptRecord {
        scope: scope.clone(),
        task_id: TaskId("task".into()),
        attempt_id: AttemptId("ambiguous-attempt".into()),
        agent_id: "ambiguous-agent".into(),
        ordinal: 1,
        predecessor: None,
        state: AttemptState::RecoveryRequired,
        revision: 1,
        selected: ProfileCandidate {
            profile: selected.profile.clone(),
            binding: selected.binding.clone(),
            observation: ProfileObservation {
                schema_version: 1,
                profile_digest: selected.profile.digest().unwrap(),
                binding_digest: selected.binding.digest().unwrap(),
                executable: ExecutableIdentity {
                    path: "fixture".into(),
                    version: "1".into(),
                    digest: Digest::of_bytes(b"fixture-executable"),
                },
                launch: None,
                observed_at_ms: 1,
                expires_at_ms: u64::MAX,
                auth_status: Readiness::Ready,
                dispatch_supported: true,
                qualification: None,
                requested_model: selected.profile.requested_model.clone(),
                reported_model: None,
                model_identity_level: ModelIdentityLevel::HarnessReported,
                metering_support: MeteringSupport::Unknown,
                hard_spend_limit_verified: false,
                capacity_ready: true,
            },
        },
        launch_fingerprint: Digest::of_bytes(b"ambiguous-launch"),
        decision: RouteDecision {
            task_state_revision: 1,
            selection: RouteSelection::Selected(target),
            reason: RouteReason::MechanicalEveryday,
            advice_status: AdviceStatus::NotUsed,
            snapshot_digest: Digest::of_bytes(b"snapshot"),
            catalog_digest: Digest::of_bytes(b"catalog"),
            policy_digest: mission.policy.digest().unwrap(),
        },
        capacity_reservation: "ambiguous-hold".into(),
        input_manifest: BlobRef {
            digest: Digest::of_bytes(b"input"),
            byte_length: 5,
        },
        handoff: None,
        dependencies: vec![],
        admitted_at_ms: 1,
        updated_at_ms: 1,
        cancel_epoch: 0,
        reserved_nano_usd: 0,
        usage: RoutedUsage::Unreported,
        receipts: RoutingReceipts::default(),
        failure: None,
        owned_launch_required: false,
        owned_checker_count: 0,
        ownership_released: false,
    };
    let config = PytxoConfig::default();
    let connection = rusqlite::Connection::open(config.db_path_at(repo)).unwrap();
    connection.execute(
        "INSERT INTO routing_attempts(attempt_id,agent_id,capacity_reservation,run_id,task_id,ordinal,revision,record_json) VALUES (?1,?2,?3,?4,?5,1,1,?6)",
        rusqlite::params![
            record.attempt_id.0,
            record.agent_id,
            record.capacity_reservation,
            scope.run_id.0,
            record.task_id.0,
            serde_json::to_string(&record).unwrap(),
        ],
    ).unwrap();
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

#[test]
fn exact_stop_waits_for_domain_gate_and_preserves_a_newer_marker() {
    let temp = tempdir().expect("tempdir");
    let repo = temp.path().to_path_buf();
    let config = PytxoConfig::default();
    let store = PytxoStore::open(&config.db_path_at(&repo)).expect("open store");
    store
        .insert_run("run-a", &repo.to_string_lossy())
        .expect("insert A");
    store
        .insert_run("run-b", &repo.to_string_lossy())
        .expect("insert B");
    let marker = write_active_run(&repo, "run-a");
    let lock_path = marker.with_file_name("active_run.json.lock");
    let lock = OpenOptions::new()
        .create(true)
        .truncate(false)
        .read(true)
        .write(true)
        .open(lock_path)
        .expect("open domain gate");
    lock.lock_exclusive().expect("hold domain gate");

    let (started_tx, started_rx) = std::sync::mpsc::channel();
    let (finished_tx, finished_rx) = std::sync::mpsc::channel();
    let worker = std::thread::spawn(move || {
        let runtime = tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .expect("runtime");
        started_tx.send(()).expect("signal start");
        finished_tx
            .send(runtime.block_on(stop_exact(None, Some(repo), "run-a", false)))
            .expect("send stop result");
    });
    started_rx
        .recv_timeout(Duration::from_secs(5))
        .expect("start");
    assert!(
        finished_rx
            .recv_timeout(Duration::from_millis(150))
            .is_err(),
        "exact Stop must wait for the domain gate before observing A"
    );

    fs::write(
        &marker,
        serde_json::json!({
            "run_id": "run-b",
            "repo_root": temp.path().to_string_lossy(),
        })
        .to_string(),
    )
    .expect("replace active marker while holding gate");
    drop(lock);
    let error = finished_rx
        .recv_timeout(Duration::from_secs(5))
        .expect("exact Stop returned")
        .expect_err("A must be rejected after B claims the domain");
    assert!(error.to_string().contains("run-b"));
    worker.join().expect("worker thread");
    assert!(fs::read_to_string(&marker).unwrap().contains("run-b"));
    assert_eq!(store.get_run_status("run-a").unwrap().unwrap().0, "running");
    assert_eq!(store.get_run_status("run-b").unwrap().unwrap().0, "running");
}

#[test]
fn ordinary_stop_waits_for_domain_gate_before_cancelling() {
    let temp = tempdir().expect("tempdir");
    let repo = temp.path().to_path_buf();
    let config = PytxoConfig::default();
    let store = PytxoStore::open(&config.db_path_at(&repo)).expect("store");
    store
        .insert_run("run-a", &repo.to_string_lossy())
        .expect("run");
    let marker = write_active_run(&repo, "run-a");
    let lock = OpenOptions::new()
        .create(true)
        .truncate(false)
        .read(true)
        .write(true)
        .open(marker.with_file_name("active_run.json.lock"))
        .expect("gate file");
    lock.lock_exclusive().expect("hold gate");
    let (started_tx, started_rx) = std::sync::mpsc::channel();
    let (finished_tx, finished_rx) = std::sync::mpsc::channel();
    let worker = std::thread::spawn(move || {
        let runtime = tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .expect("runtime");
        started_tx.send(()).expect("started");
        finished_tx
            .send(runtime.block_on(stop(None, Some(repo), false, false)))
            .expect("finished");
    });
    started_rx
        .recv_timeout(Duration::from_secs(5))
        .expect("start");
    assert!(
        finished_rx
            .recv_timeout(Duration::from_millis(150))
            .is_err(),
        "ordinary Stop must wait before observing the active marker"
    );
    assert_eq!(store.get_run_status("run-a").unwrap().unwrap().0, "running");
    drop(lock);
    finished_rx
        .recv_timeout(Duration::from_secs(5))
        .expect("stop result")
        .expect("stop success");
    worker.join().expect("worker");
    assert_eq!(
        store.get_run_status("run-a").unwrap().unwrap().0,
        "cancelled"
    );
}

#[test]
fn exact_stop_obeys_domain_gate_across_processes() {
    if let Some(repo) = std::env::var_os(CROSS_PROCESS_GATE_REPO) {
        let ready = std::env::var_os(CROSS_PROCESS_GATE_READY).expect("ready marker");
        fs::write(ready, b"ready").expect("signal child is about to Stop");
        tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .unwrap()
            .block_on(stop_exact(None, Some(repo.into()), "cross-process", false))
            .expect("child Stop");
        return;
    }
    let temp = tempdir().unwrap();
    let repo = temp.path();
    let config = PytxoConfig::default();
    let store = PytxoStore::open(&config.db_path_at(repo)).unwrap();
    store
        .insert_run("cross-process", &repo.to_string_lossy())
        .unwrap();
    let marker = write_active_run(repo, "cross-process");
    let lock = OpenOptions::new()
        .create(true)
        .truncate(false)
        .read(true)
        .write(true)
        .open(marker.with_file_name("active_run.json.lock"))
        .unwrap();
    lock.lock_exclusive().unwrap();
    let ready = temp.path().join("stop-child-ready");
    let mut child = Command::new(std::env::current_exe().unwrap())
        .arg("exact_stop_obeys_domain_gate_across_processes")
        .arg("--exact")
        .arg("--nocapture")
        .env(CROSS_PROCESS_GATE_REPO, repo)
        .env(CROSS_PROCESS_GATE_READY, &ready)
        .spawn()
        .unwrap();
    assert!(
        tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .unwrap()
            .block_on(wait_until(|| ready.exists())),
        "child never reached Stop"
    );
    std::thread::sleep(Duration::from_millis(150));
    assert!(
        child.try_wait().unwrap().is_none(),
        "child bypassed file gate"
    );
    assert!(marker.exists());
    drop(lock);
    assert!(child.wait().unwrap().success());
    assert!(!marker.exists());
    assert_eq!(
        store.get_run_status("cross-process").unwrap().unwrap().0,
        "cancelled"
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

#[tokio::test]
async fn late_stop_fences_incomplete_completed_run_without_reclassification() {
    let temp = tempdir().unwrap();
    let repo = temp.path();
    let config = PytxoConfig::default();
    let store = PytxoStore::open(&config.db_path_at(repo)).unwrap();
    let (mission, scope) = registered_ready_mission(repo, "already-completed");
    store
        .insert_run("already-completed", &repo.to_string_lossy())
        .unwrap();
    store.register_routing_mission(&mission).unwrap();
    store.finish_run("already-completed", "completed").unwrap();
    let marker = write_active_run(repo, "already-completed");

    stop_exact(None, Some(repo.to_path_buf()), "already-completed", false)
        .await
        .unwrap();
    assert_eq!(
        store
            .get_run_status("already-completed")
            .unwrap()
            .unwrap()
            .0,
        "completed"
    );
    let history = store.routing_history(&scope).unwrap().unwrap();
    assert!(history.cancelled);
    assert_eq!(history.cancel_epoch, 1);
    assert!(history.attempts.is_empty());
    assert!(!marker.exists());
}

async fn assert_routed_stop_cleanup_preserves_registered_worktree(all: bool) {
    let temp = tempdir().unwrap();
    let repo = temp.path();
    init_git_repo(repo);
    let run_id = "routed-stale-registry";
    let config = PytxoConfig::default();
    let store = PytxoStore::open(&config.db_path_at(repo)).unwrap();
    store.insert_run(run_id, &repo.to_string_lossy()).unwrap();
    let (mission, scope) = registered_ready_mission(repo, run_id);
    store.register_routing_mission(&mission).unwrap();
    store
        .cancel_routing_mission(&scope, "already-settled", 0, 120)
        .unwrap();
    let worktree = repo.join(&config.worktree_dir).join(run_id).join("attempt");
    fs::create_dir_all(worktree.parent().unwrap()).unwrap();
    let added = Command::new("git")
        .args(["worktree", "add", "--detach"])
        .arg(&worktree)
        .arg("HEAD")
        .current_dir(repo)
        .output()
        .unwrap();
    assert!(
        added.status.success(),
        "{}",
        String::from_utf8_lossy(&added.stderr)
    );
    fs::write(worktree.join("keep.txt"), b"user-worktree").unwrap();
    write_active_run(repo, run_id);
    let child = spawn_tracked_sleep_child(repo, run_id, &worktree);

    if all {
        stop(None, Some(repo.to_path_buf()), true, true)
            .await
            .unwrap();
    } else {
        stop_exact(None, Some(repo.to_path_buf()), run_id, true)
            .await
            .unwrap();
    }
    assert!(!process_matches(child.pid, &child.identity).unwrap());
    assert_eq!(
        fs::read(worktree.join("keep.txt")).unwrap(),
        b"user-worktree",
        "Stop must not use generic force cleanup for any routed run"
    );
}

#[tokio::test]
async fn routed_exact_stop_cleanup_preserves_registered_worktree() {
    assert_routed_stop_cleanup_preserves_registered_worktree(false).await;
}

#[tokio::test]
async fn routed_stop_all_cleanup_preserves_registered_worktree() {
    assert_routed_stop_cleanup_preserves_registered_worktree(true).await;
}

#[tokio::test]
async fn late_stop_terminates_lingering_process_without_reclassifying_completed_run() {
    struct ChildGuard(std::process::Child);
    impl Drop for ChildGuard {
        fn drop(&mut self) {
            let _ = self.0.kill();
            let _ = self.0.wait();
        }
    }
    let temp = tempdir().unwrap();
    let repo = temp.path();
    let config = PytxoConfig::default();
    let data_dir = repo.join(&config.data_dir);
    let store = PytxoStore::open(&config.db_path_at(repo)).unwrap();
    store
        .insert_run("completed-with-child", &repo.to_string_lossy())
        .unwrap();
    store
        .finish_run("completed-with-child", "completed")
        .unwrap();
    let marker = write_active_run(repo, "completed-with-child");
    let child = if cfg!(windows) {
        Command::new("powershell")
            .args([
                "-NoProfile",
                "-NonInteractive",
                "-Command",
                "Start-Sleep -Seconds 120",
            ])
            .spawn()
            .unwrap()
    } else {
        Command::new("sh")
            .args(["-c", "sleep 120"])
            .spawn()
            .unwrap()
    };
    let mut child = ChildGuard(child);
    let pid = child.0.id();
    let identity = process_start_identity(pid).unwrap().unwrap();
    ProcessRegistryFile::update(&registry_path(&data_dir), |registry| {
        registry.push(ProcessEntry {
            run_id: "completed-with-child".into(),
            repo_root: repo.to_string_lossy().into_owned(),
            agent_key: "completed-with-child:fixture".into(),
            pid,
            start_identity: Some(identity.clone()),
            worktree_path: String::new(),
            branch: String::new(),
        });
        Ok(())
    })
    .unwrap();

    stop_exact(
        None,
        Some(repo.to_path_buf()),
        "completed-with-child",
        false,
    )
    .await
    .unwrap();
    assert!(!process_matches(pid, &identity).unwrap());
    assert_eq!(
        store
            .get_run_status("completed-with-child")
            .unwrap()
            .unwrap()
            .0,
        "completed"
    );
    assert!(!marker.exists());
    let _ = child.0.wait();
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

#[tokio::test]
async fn stop_all_cancels_active_run_between_child_processes() {
    let temp = tempdir().unwrap();
    let config = PytxoConfig::default();
    let store = PytxoStore::open(&config.db_path_at(temp.path())).unwrap();
    store
        .insert_run("between-checks", &temp.path().to_string_lossy())
        .unwrap();
    let marker = write_active_run(temp.path(), "between-checks");
    stop(None, Some(temp.path().to_path_buf()), true, false)
        .await
        .unwrap();
    assert_eq!(
        store.get_run_status("between-checks").unwrap().unwrap().0,
        "cancelled"
    );
    assert!(!marker.exists());
    let registry =
        ProcessRegistryFile::load(&registry_path(&temp.path().join(&config.data_dir))).unwrap();
    assert!(registry.cancelled_runs.contains(&"between-checks".into()));
    assert!(registry.entries.is_empty());
}

#[tokio::test]
async fn stop_all_fences_registered_ready_mission_once_without_a_process() {
    let temp = tempdir().unwrap();
    let repo = temp.path();
    let config = PytxoConfig::default();
    let store = PytxoStore::open(&config.db_path_at(repo)).unwrap();
    let (mission, scope) = registered_ready_mission(repo, "route-ready");
    store
        .insert_run("route-ready", &repo.to_string_lossy())
        .unwrap();
    store.register_routing_mission(&mission).unwrap();
    let registration_events = store.routing_history(&scope).unwrap().unwrap().events.len();
    let marker = write_active_run(repo, "route-ready");

    stop(None, Some(repo.to_path_buf()), true, false)
        .await
        .unwrap();
    let history = store.routing_history(&scope).unwrap().unwrap();
    assert!(history.cancelled);
    assert_eq!(history.cancel_epoch, 1);
    assert_eq!(history.attempts.len(), 0);
    assert_eq!(history.events.len(), registration_events + 1);
    assert!(!marker.exists());
    let registry = ProcessRegistryFile::load(&registry_path(&repo.join(&config.data_dir))).unwrap();
    assert!(registry.cancelled_runs.contains(&"route-ready".to_string()));
    assert!(registry.entries.is_empty());

    stop(None, Some(repo.to_path_buf()), true, false)
        .await
        .unwrap();
    let replay = store.routing_history(&scope).unwrap().unwrap();
    assert_eq!(
        replay.cancel_epoch, 1,
        "Stop replay must not advance the epoch"
    );
    assert_eq!(replay.events.len(), registration_events + 1);
}

#[test]
fn stop_all_keeps_ownership_after_epoch_failure_but_terminates_known_process() {
    struct ChildGuard(std::process::Child);
    impl Drop for ChildGuard {
        fn drop(&mut self) {
            let _ = self.0.kill();
            let _ = self.0.wait();
        }
    }
    let temp = tempdir().unwrap();
    let repo = temp.path();
    let config = PytxoConfig::default();
    let data_dir = repo.join(&config.data_dir);
    let store = PytxoStore::open(&config.db_path_at(repo)).unwrap();
    let (mission, scope) = registered_ready_mission(repo, "route-partial");
    store
        .insert_run("route-partial", &repo.to_string_lossy())
        .unwrap();
    store.register_routing_mission(&mission).unwrap();
    let marker = write_active_run(repo, "route-partial");
    let workspace = temp.path().join("owned-workspace");
    fs::create_dir(&workspace).unwrap();
    fs::write(workspace.join("keep.txt"), "retain on partial failure").unwrap();
    let child = if cfg!(windows) {
        Command::new("powershell")
            .args([
                "-NoProfile",
                "-NonInteractive",
                "-Command",
                "Start-Sleep -Seconds 120",
            ])
            .spawn()
            .unwrap()
    } else {
        Command::new("sh")
            .args(["-c", "sleep 120"])
            .spawn()
            .unwrap()
    };
    let mut child = ChildGuard(child);
    let pid = child.0.id();
    let identity = process_start_identity(pid)
        .unwrap()
        .expect("live child identity");
    ProcessRegistryFile::update(&registry_path(&data_dir), |registry| {
        registry.push(ProcessEntry {
            run_id: "route-partial".into(),
            repo_root: repo.to_string_lossy().into_owned(),
            agent_key: "route-partial:fixture".into(),
            pid,
            start_identity: Some(identity.clone()),
            worktree_path: workspace.to_string_lossy().into_owned(),
            branch: "fixture".into(),
        });
        Ok(())
    })
    .unwrap();
    let conn = rusqlite::Connection::open(config.db_path_at(repo)).unwrap();
    conn.execute(
        "UPDATE routing_missions SET cancel_epoch=?1 WHERE run_id=?2",
        rusqlite::params![i64::MAX, "route-partial"],
    )
    .unwrap();
    let runtime = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .unwrap();
    let error = runtime
        .block_on(stop(None, Some(repo.to_path_buf()), true, true))
        .expect_err("Store epoch write must fail");
    assert!(error.to_string().contains("partially failed"));
    assert!(marker.exists(), "partial Stop retains active ownership");
    assert!(
        workspace.join("keep.txt").exists(),
        "partial Stop retains workspace"
    );
    assert!(
        !process_matches(pid, &identity).unwrap(),
        "known child must terminate"
    );
    assert!(ProcessRegistryFile::load(&registry_path(&data_dir))
        .unwrap()
        .cancelled_runs
        .contains(&"route-partial".to_string()));
    assert!(!store.routing_history(&scope).unwrap().unwrap().cancelled);
    let _ = child.0.wait();

    conn.execute(
        "UPDATE routing_missions SET cancel_epoch=0 WHERE run_id=?1",
        ["route-partial"],
    )
    .unwrap();
    runtime
        .block_on(stop(None, Some(repo.to_path_buf()), true, false))
        .expect("idempotent retry after Store repair");
    assert_eq!(
        store.routing_history(&scope).unwrap().unwrap().cancel_epoch,
        1
    );
    assert!(!marker.exists());
}

#[tokio::test]
async fn ordinary_stop_terminates_known_process_when_store_cannot_open() {
    struct ChildGuard(std::process::Child);
    impl Drop for ChildGuard {
        fn drop(&mut self) {
            let _ = self.0.kill();
            let _ = self.0.wait();
        }
    }
    let temp = tempdir().unwrap();
    let repo = temp.path();
    let config = PytxoConfig::default();
    let data_dir = repo.join(&config.data_dir);
    let marker = write_active_run(repo, "store-unavailable");
    fs::create_dir(config.db_path_at(repo)).unwrap();
    let workspace = repo.join("owned-workspace");
    fs::create_dir(&workspace).unwrap();
    fs::write(workspace.join("keep.txt"), "retain on partial failure").unwrap();
    let child = if cfg!(windows) {
        Command::new("powershell")
            .args([
                "-NoProfile",
                "-NonInteractive",
                "-Command",
                "Start-Sleep -Seconds 120",
            ])
            .spawn()
            .unwrap()
    } else {
        Command::new("sh")
            .args(["-c", "sleep 120"])
            .spawn()
            .unwrap()
    };
    let mut child = ChildGuard(child);
    let pid = child.0.id();
    let identity = process_start_identity(pid)
        .unwrap()
        .expect("live child identity");
    ProcessRegistryFile::update(&registry_path(&data_dir), |registry| {
        registry.push(ProcessEntry {
            run_id: "store-unavailable".into(),
            repo_root: repo.to_string_lossy().into_owned(),
            agent_key: "store-unavailable:fixture".into(),
            pid,
            start_identity: Some(identity.clone()),
            worktree_path: workspace.to_string_lossy().into_owned(),
            branch: "fixture".into(),
        });
        Ok(())
    })
    .unwrap();

    let error = stop(None, Some(repo.to_path_buf()), false, true)
        .await
        .expect_err("Store open must fail");
    assert!(error.to_string().contains("Store unavailable"), "{error:#}");
    assert!(marker.exists(), "partial Stop retains active ownership");
    assert!(
        workspace.join("keep.txt").exists(),
        "partial Stop retains workspace"
    );
    assert!(
        !process_matches(pid, &identity).unwrap(),
        "known child must terminate"
    );
    assert!(ProcessRegistryFile::load(&registry_path(&data_dir))
        .unwrap()
        .cancelled_runs
        .contains(&"store-unavailable".to_string()));
    let _ = child.0.wait();
}

#[tokio::test]
async fn stop_all_terminates_known_process_when_store_cannot_open() {
    let temp = tempdir().unwrap();
    let repo = temp.path();
    let config = PytxoConfig::default();
    let data_dir = repo.join(&config.data_dir);
    let marker = write_active_run(repo, "all-store-unavailable");
    fs::create_dir(config.db_path_at(repo)).unwrap();
    let workspace = repo.join("owned-workspace");
    fs::create_dir(&workspace).unwrap();
    fs::write(workspace.join("keep.txt"), "retain on partial failure").unwrap();
    let mut child = spawn_tracked_sleep_child(repo, "all-store-unavailable", &workspace);

    let error = stop(None, Some(repo.to_path_buf()), true, true)
        .await
        .expect_err("Store open must leave routed ownership unconfirmed");
    assert!(error.to_string().contains("Store unavailable"), "{error:#}");
    assert!(marker.exists(), "partial Stop-all retains active ownership");
    assert!(workspace.join("keep.txt").exists());
    assert!(
        !process_matches(child.pid, &child.identity).unwrap(),
        "known registry child must terminate despite Store failure"
    );
    assert!(ProcessRegistryFile::load(&registry_path(&data_dir))
        .unwrap()
        .cancelled_runs
        .contains(&"all-store-unavailable".to_string()));
    let _ = child.child.wait();
}

#[tokio::test]
async fn corrupt_registry_does_not_block_store_routing_fence() {
    for all in [false, true] {
        let temp = tempdir().unwrap();
        let repo = temp.path();
        let config = PytxoConfig::default();
        let store = PytxoStore::open(&config.db_path_at(repo)).unwrap();
        let run_id = if all { "corrupt-all" } else { "corrupt-exact" };
        let (mission, scope) = registered_ready_mission(repo, run_id);
        store.insert_run(run_id, &repo.to_string_lossy()).unwrap();
        store.register_routing_mission(&mission).unwrap();
        let marker = write_active_run(repo, run_id);
        fs::write(registry_path(&repo.join(&config.data_dir)), b"{").unwrap();

        let error = stop(None, Some(repo.to_path_buf()), all, false)
            .await
            .expect_err("corrupt registry must retain partial Stop status");
        assert!(
            error.to_string().contains("registry"),
            "missing registry failure: {error:#}"
        );
        assert!(
            store.routing_history(&scope).unwrap().unwrap().cancelled,
            "Store mission must be fenced despite legacy registry corruption"
        );
        assert!(marker.exists(), "partial Stop must retain active ownership");
    }
}

#[tokio::test]
async fn stop_all_terminates_known_process_after_status_query_failure() {
    struct ChildGuard(std::process::Child);
    impl Drop for ChildGuard {
        fn drop(&mut self) {
            let _ = self.0.kill();
            let _ = self.0.wait();
        }
    }
    let temp = tempdir().unwrap();
    let repo = temp.path();
    let config = PytxoConfig::default();
    let data_dir = repo.join(&config.data_dir);
    let store = PytxoStore::open(&config.db_path_at(repo)).unwrap();
    store
        .insert_run("status-query-fault", &repo.to_string_lossy())
        .unwrap();
    let (mission, scope) = registered_ready_mission(repo, "status-query-fault");
    store.register_routing_mission(&mission).unwrap();
    let marker = write_active_run(repo, "status-query-fault");
    let workspace = repo.join("owned-workspace");
    fs::create_dir(&workspace).unwrap();
    fs::write(workspace.join("keep.txt"), "retain on partial failure").unwrap();
    let child = if cfg!(windows) {
        Command::new("powershell")
            .args([
                "-NoProfile",
                "-NonInteractive",
                "-Command",
                "Start-Sleep -Seconds 120",
            ])
            .spawn()
            .unwrap()
    } else {
        Command::new("sh")
            .args(["-c", "sleep 120"])
            .spawn()
            .unwrap()
    };
    let mut child = ChildGuard(child);
    let pid = child.0.id();
    let identity = process_start_identity(pid)
        .unwrap()
        .expect("live child identity");
    ProcessRegistryFile::update(&registry_path(&data_dir), |registry| {
        registry.push(ProcessEntry {
            run_id: "status-query-fault".into(),
            repo_root: repo.to_string_lossy().into_owned(),
            agent_key: "status-query-fault:fixture".into(),
            pid,
            start_identity: Some(identity.clone()),
            worktree_path: workspace.to_string_lossy().into_owned(),
            branch: "fixture".into(),
        });
        Ok(())
    })
    .unwrap();
    let conn = rusqlite::Connection::open(config.db_path_at(repo)).unwrap();
    conn.execute_batch("UPDATE runs SET status = X'FF' WHERE id = 'status-query-fault'")
        .unwrap();

    let error = stop(None, Some(repo.to_path_buf()), true, true)
        .await
        .expect_err("run status query must fail");
    assert!(error.to_string().contains("partially failed"), "{error:#}");
    assert!(store.routing_history(&scope).unwrap().unwrap().cancelled);
    assert!(marker.exists(), "partial Stop retains active ownership");
    assert!(
        workspace.join("keep.txt").exists(),
        "partial Stop retains workspace"
    );
    assert!(
        !process_matches(pid, &identity).unwrap(),
        "known child must terminate"
    );
    assert!(ProcessRegistryFile::load(&registry_path(&data_dir))
        .unwrap()
        .cancelled_runs
        .contains(&"status-query-fault".to_string()));
    let _ = child.0.wait();
}

#[tokio::test]
async fn stop_all_terminates_known_process_after_attempt_store_read_failure() {
    let temp = tempdir().unwrap();
    let repo = temp.path();
    let config = PytxoConfig::default();
    let data_dir = repo.join(&config.data_dir);
    let store = PytxoStore::open(&config.db_path_at(repo)).unwrap();
    store
        .insert_run("legacy-known", &repo.to_string_lossy())
        .unwrap();
    let marker = write_active_run(repo, "legacy-known");
    let workspace = repo.join("legacy-workspace");
    fs::create_dir(&workspace).unwrap();
    fs::write(
        workspace.join("keep.txt"),
        "retain while Store is uncertain",
    )
    .unwrap();
    let child = spawn_tracked_sleep_child(repo, "legacy-known", &workspace);

    let (mission, scope) = registered_ready_mission(repo, "route-corrupt-attempt");
    store
        .insert_run("route-corrupt-attempt", &repo.to_string_lossy())
        .unwrap();
    store.register_routing_mission(&mission).unwrap();
    seed_unresolved_attempt(repo, &scope, &mission);
    let conn = rusqlite::Connection::open(config.db_path_at(repo)).unwrap();
    conn.execute(
        "UPDATE routing_attempts SET record_json='{' WHERE run_id=?1",
        ["route-corrupt-attempt"],
    )
    .unwrap();

    let error = stop(None, Some(repo.to_path_buf()), true, true)
        .await
        .expect_err("malformed attempt must retain routed ownership");
    assert!(error.to_string().contains("partially failed"), "{error:#}");
    assert!(marker.exists(), "unknown routed ownership retains marker");
    assert!(
        workspace.join("keep.txt").exists(),
        "unknown ownership retains workspace"
    );
    assert!(
        !process_matches(child.pid, &child.identity).unwrap(),
        "known legacy child must terminate"
    );
    assert!(ProcessRegistryFile::load(&registry_path(&data_dir))
        .unwrap()
        .cancelled_runs
        .contains(&"legacy-known".to_string()));
    let route_epoch: (i64, i64) = conn
        .query_row(
            "SELECT cancel_epoch,cancelled FROM routing_missions WHERE run_id=?1",
            ["route-corrupt-attempt"],
            |row| Ok((row.get(0)?, row.get(1)?)),
        )
        .unwrap();
    assert_eq!(route_epoch, (0, 0), "unknown route was not falsely fenced");
}

#[tokio::test]
async fn stop_all_keeps_known_child_safe_after_registration_scan_failure() {
    let temp = tempdir().unwrap();
    let repo = temp.path();
    let config = PytxoConfig::default();
    let data_dir = repo.join(&config.data_dir);
    let store = PytxoStore::open(&config.db_path_at(repo)).unwrap();
    store
        .insert_run("legacy-known-registration", &repo.to_string_lossy())
        .unwrap();
    let marker = write_active_run(repo, "legacy-known-registration");
    let workspace = repo.join("legacy-registration-workspace");
    fs::create_dir(&workspace).unwrap();
    fs::write(
        workspace.join("keep.txt"),
        "retain while registration is uncertain",
    )
    .unwrap();
    let child = spawn_tracked_sleep_child(repo, "legacy-known-registration", &workspace);
    let (mission, _) = registered_ready_mission(repo, "route-bad-registration");
    store
        .insert_run("route-bad-registration", &repo.to_string_lossy())
        .unwrap();
    store.register_routing_mission(&mission).unwrap();
    let conn = rusqlite::Connection::open(config.db_path_at(repo)).unwrap();
    conn.execute(
        "UPDATE routing_missions SET registration_json='{' WHERE run_id=?1",
        ["route-bad-registration"],
    )
    .unwrap();

    let error = stop(None, Some(repo.to_path_buf()), true, true)
        .await
        .expect_err("malformed registration must retain recovery ownership");
    assert!(error.to_string().contains("partially failed"), "{error:#}");
    assert!(error.to_string().contains("registration"), "{error:#}");
    assert!(
        marker.exists(),
        "unknown registration retains active marker"
    );
    assert!(
        workspace.join("keep.txt").exists(),
        "unknown registration retains workspace"
    );
    assert!(
        !process_matches(child.pid, &child.identity).unwrap(),
        "known legacy child must terminate"
    );
    assert!(ProcessRegistryFile::load(&registry_path(&data_dir))
        .unwrap()
        .cancelled_runs
        .contains(&"legacy-known-registration".to_string()));
    let route_epoch: (i64, i64) = conn
        .query_row(
            "SELECT cancel_epoch,cancelled FROM routing_missions WHERE run_id=?1",
            ["route-bad-registration"],
            |row| Ok((row.get(0)?, row.get(1)?)),
        )
        .unwrap();
    assert_eq!(
        route_epoch,
        (0, 0),
        "corrupt registration was not falsely fenced"
    );
}

#[tokio::test]
async fn stop_all_terminates_known_child_after_capacity_intent_scan_failure() {
    let temp = tempdir().unwrap();
    let repo = temp.path();
    let config = PytxoConfig::default();
    let store = PytxoStore::open(&config.db_path_at(repo)).unwrap();
    store
        .insert_run("known-capacity-child", &repo.to_string_lossy())
        .unwrap();
    let marker = write_active_run(repo, "known-capacity-child");
    let workspace = repo.join("known-capacity-workspace");
    fs::create_dir(&workspace).unwrap();
    fs::write(workspace.join("keep.txt"), "retain on uncertain capacity").unwrap();
    let child = spawn_tracked_sleep_child(repo, "known-capacity-child", &workspace);
    let (route_store, _catalog, request) =
        seed_registered_capacity_intent(repo, "corrupt-capacity-intent");
    drop(route_store);
    let conn = rusqlite::Connection::open(config.db_path_at(repo)).unwrap();
    conn.execute(
        "UPDATE routing_capacity_intents SET record_json='{' WHERE reservation_id=?1",
        [&request.reservation.reservation_id],
    )
    .unwrap();

    let error = stop(None, Some(repo.to_path_buf()), true, true)
        .await
        .expect_err("corrupt capacity ownership must retain recovery");
    assert!(error.to_string().contains("capacity intent"), "{error:#}");
    assert!(marker.exists());
    assert!(workspace.join("keep.txt").exists());
    assert!(!process_matches(child.pid, &child.identity).unwrap());
    assert!(
        ProcessRegistryFile::load(&registry_path(&repo.join(config.data_dir)))
            .unwrap()
            .cancelled_runs
            .contains(&"known-capacity-child".to_string())
    );
}

#[test]
fn markerless_capacity_intent_fences_second_process_until_closed() {
    if let Some(repo) = std::env::var_os(MARKERLESS_CAPACITY_RESTART_REPO) {
        tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .unwrap()
            .block_on(run_markerless_capacity_recovery(std::path::PathBuf::from(
                repo,
            )));
        return;
    }
    let temp = tempdir().unwrap();
    let repo = temp.path();
    init_git_repo(repo);
    let (store, _catalog, request) = seed_registered_capacity_intent(repo, "markerless-capacity");
    store
        .cancel_routing_mission(&request.scope, "cancel-capacity-mission", 0, 120)
        .unwrap();
    store
        .finish_run_if_status("markerless-capacity", "starting", "cancelled")
        .unwrap();
    drop(store);
    let home = tempdir().unwrap();
    let status = Command::new(std::env::current_exe().unwrap())
        .arg("markerless_capacity_intent_fences_second_process_until_closed")
        .arg("--exact")
        .arg("--nocapture")
        .env("PYTXO_HOME", home.path())
        .env(
            "PYTXO_TRUST_STORE",
            home.path().join("trusted-domains.json"),
        )
        .env(MARKERLESS_CAPACITY_RESTART_REPO, repo)
        .status()
        .unwrap();
    assert!(status.success(), "second-process capacity recovery failed");
}

async fn run_markerless_capacity_recovery(repo: std::path::PathBuf) {
    trust_repo(&repo, PermissionProfile::Orbit).unwrap();
    let config = PytxoConfig::default();
    let store = PytxoStore::open(&config.db_path_at(&repo)).unwrap();
    let denied_payload = repo.join("capacity-denied-payload.txt");
    let denied = dispatch(RunOptions {
        agents: 1,
        cmd: format!("echo should-not-run > \"{}\"", denied_payload.display()),
        config: None,
        dry_run: false,
        keep_worktrees: false,
        repo: Some(repo.clone()),
        execution: Some(ExecutionBackend::Subprocess),
        project: None,
        tasks: None,
        task_cmd_template: None,
        task_prompts: None,
    })
    .expect_err("pending markerless intent must block a new claim");
    assert!(denied.to_string().contains("capacity intent"), "{denied:#}");
    assert!(!denied_payload.exists());
    assert!(!repo.join(&config.data_dir).join("active_run.json").exists());
    let stopped = stop(None, Some(repo.clone()), true, false)
        .await
        .expect_err("Stop-all cannot pretend a pending capacity intent is released");
    assert!(
        stopped.to_string().contains("capacity intent"),
        "{stopped:#}"
    );
    assert_eq!(store.unresolved_capacity_intent_scopes().unwrap().len(), 1);
    with_capacity_test_gate(&repo, || {
        store
            .close_unissued_capacity_intent("reservation-markerless-capacity", "precall-close")
            .unwrap();
    });
    assert!(store
        .unresolved_capacity_intent_scopes()
        .unwrap()
        .is_empty());
    let (_, newer_id) = dispatch(RunOptions {
        agents: 1,
        cmd: "echo later-claim".into(),
        config: None,
        dry_run: false,
        keep_worktrees: false,
        repo: Some(repo.clone()),
        execution: Some(ExecutionBackend::Subprocess),
        project: None,
        tasks: None,
        task_cmd_template: None,
        task_prompts: None,
    })
    .expect("new claim should proceed after exact pre-call closure");
    assert!(
        wait_until(|| store
            .get_run_status(&newer_id.0)
            .unwrap()
            .is_some_and(|(status, _)| PytxoStore::is_terminal_run_status(&status)))
        .await,
        "later legacy fixture run should settle"
    );
}

#[test]
fn capacity_reserve_release_reconciles_in_second_process() {
    if let Some(repo) = std::env::var_os(RELEASE_CAPACITY_RESTART_REPO) {
        tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .unwrap()
            .block_on(run_second_process_capacity_release(
                std::path::PathBuf::from(repo),
            ));
        return;
    }
    let temp = tempdir().unwrap();
    let repo = temp.path();
    init_git_repo(repo);
    let (store, catalog, request) = seed_registered_capacity_intent(repo, "capacity-release");
    issue_no_worker_reservation_once(repo, &store, &catalog, &request).unwrap();
    assert!(
        issue_no_worker_reservation_once(repo, &store, &catalog, &request).is_err(),
        "a replayed may-started event cannot issue a second reserve"
    );
    store
        .cancel_routing_mission(&request.scope, "capacity-release-cancel", 0, 120)
        .unwrap();
    store
        .finish_run_if_status("capacity-release", "starting", "cancelled")
        .unwrap();
    drop(store);
    drop(catalog);
    let home = tempdir().unwrap();
    let status = Command::new(std::env::current_exe().unwrap())
        .arg("capacity_reserve_release_reconciles_in_second_process")
        .arg("--exact")
        .arg("--nocapture")
        .env("PYTXO_HOME", home.path())
        .env(
            "PYTXO_TRUST_STORE",
            home.path().join("trusted-domains.json"),
        )
        .env(RELEASE_CAPACITY_RESTART_REPO, repo)
        .status()
        .unwrap();
    assert!(status.success(), "second-process exact release failed");
}

async fn run_second_process_capacity_release(repo: std::path::PathBuf) {
    trust_repo(&repo, PermissionProfile::Orbit).unwrap();
    let config = PytxoConfig::default();
    let store = PytxoStore::open(&config.db_path_at(&repo)).unwrap();
    let reservation_id = "reservation-capacity-release";
    let intent = store.capacity_intent(reservation_id).unwrap().unwrap();
    assert_eq!(store.unresolved_capacity_intent_scopes().unwrap().len(), 1);
    let missing = repo.join("missing-capacity-catalog.db");
    assert!(Catalog::open_existing_for_capacity_recovery(&missing).is_err());
    assert!(!missing.exists());
    let denied_payload = repo.join("unreleased-capacity-payload.txt");
    let denied = dispatch(RunOptions {
        agents: 1,
        cmd: format!("echo should-not-run > \"{}\"", denied_payload.display()),
        config: None,
        dry_run: false,
        keep_worktrees: false,
        repo: Some(repo.clone()),
        execution: Some(ExecutionBackend::Subprocess),
        project: None,
        tasks: None,
        task_cmd_template: None,
        task_prompts: None,
    })
    .expect_err("host hold must deny a new claim after restart");
    assert!(denied.to_string().contains("capacity intent"), "{denied:#}");
    assert!(!denied_payload.exists());
    let catalog = Catalog::open_existing_for_capacity_recovery(&intent.catalog_locator).unwrap();
    assert!(
        issue_no_worker_reservation_once(&repo, &store, &catalog, &intent.request).is_err(),
        "restart cannot reissue reserve from MayStarted"
    );
    let proof = CapacityReleaseEvidence {
        kind: CapacityReleaseEvidenceKind::KnownUnused,
        receipt_id: "no-worker-recovery-receipt".into(),
        evidence_digest: "sha256:no-worker-test-attestation".into(),
        observed_at_ms: 130,
    };
    with_capacity_test_gate(&repo, || {
        store
            .bind_capacity_release_proof(&catalog, reservation_id, "bind-no-worker-proof", &proof)
            .unwrap();
        catalog
            .release_capacity_reservation(&CapacityReleaseRequest {
                reservation_id: reservation_id.into(),
                evidence: proof.clone(),
            })
            .unwrap();
        store
            .close_released_capacity_intent(&catalog, reservation_id, "close-released-intent")
            .unwrap();
    });
    assert!(store
        .unresolved_capacity_intent_scopes()
        .unwrap()
        .is_empty());
    let (_, newer_id) = dispatch(RunOptions {
        agents: 1,
        cmd: "echo after-capacity-release".into(),
        config: None,
        dry_run: false,
        keep_worktrees: false,
        repo: Some(repo.clone()),
        execution: Some(ExecutionBackend::Subprocess),
        project: None,
        tasks: None,
        task_cmd_template: None,
        task_prompts: None,
    })
    .expect("new claim should proceed after exact Catalog release and Store closure");
    assert!(
        wait_until(|| store
            .get_run_status(&newer_id.0)
            .unwrap()
            .is_some_and(|(status, _)| PytxoStore::is_terminal_run_status(&status)))
        .await,
        "later legacy fixture run should settle"
    );
}

#[tokio::test]
async fn stop_all_cancels_markerless_incomplete_completed_registration() {
    let temp = tempdir().unwrap();
    let repo = temp.path();
    let config = PytxoConfig::default();
    let store = PytxoStore::open(&config.db_path_at(repo)).unwrap();
    let (mission, scope) = registered_ready_mission(repo, "completed-ready-markerless");
    store
        .insert_run("completed-ready-markerless", &repo.to_string_lossy())
        .unwrap();
    store.register_routing_mission(&mission).unwrap();
    store
        .finish_run("completed-ready-markerless", "completed")
        .unwrap();

    stop(None, Some(repo.to_path_buf()), true, false)
        .await
        .expect("Stop-all must fence incomplete completed routing work");
    let history = store.routing_history(&scope).unwrap().unwrap();
    assert!(history.cancelled);
    assert_eq!(history.cancel_epoch, 1);
    assert!(history.attempts.is_empty());
    assert_eq!(
        store
            .get_run_status("completed-ready-markerless")
            .unwrap()
            .unwrap()
            .0,
        "completed"
    );
    assert!(
        ProcessRegistryFile::load(&registry_path(&repo.join(&config.data_dir)))
            .unwrap()
            .cancelled_runs
            .contains(&"completed-ready-markerless".to_string())
    );
}

#[tokio::test]
async fn ordinary_stop_terminates_known_process_after_status_store_read_failure() {
    let temp = tempdir().unwrap();
    let repo = temp.path();
    let config = PytxoConfig::default();
    let data_dir = repo.join(&config.data_dir);
    let store = PytxoStore::open(&config.db_path_at(repo)).unwrap();
    store
        .insert_run("ordinary-status-fault", &repo.to_string_lossy())
        .unwrap();
    let (mission, scope) = registered_ready_mission(repo, "ordinary-status-fault");
    store.register_routing_mission(&mission).unwrap();
    let marker = write_active_run(repo, "ordinary-status-fault");
    let workspace = repo.join("ordinary-workspace");
    fs::create_dir(&workspace).unwrap();
    fs::write(
        workspace.join("keep.txt"),
        "retain while Store is uncertain",
    )
    .unwrap();
    let child = spawn_tracked_sleep_child(repo, "ordinary-status-fault", &workspace);
    let conn = rusqlite::Connection::open(config.db_path_at(repo)).unwrap();
    conn.execute_batch("UPDATE runs SET status = X'FF' WHERE id = 'ordinary-status-fault'")
        .unwrap();

    let error = stop(None, Some(repo.to_path_buf()), false, true)
        .await
        .expect_err("invalid run status must require recovery");
    assert!(error.to_string().contains("unconfirmed"), "{error:#}");
    assert!(store.routing_history(&scope).unwrap().unwrap().cancelled);
    assert!(marker.exists(), "unknown Store status retains marker");
    assert!(
        workspace.join("keep.txt").exists(),
        "unknown status retains workspace"
    );
    assert!(
        !process_matches(child.pid, &child.identity).unwrap(),
        "known child must terminate"
    );
    assert!(ProcessRegistryFile::load(&registry_path(&data_dir))
        .unwrap()
        .cancelled_runs
        .contains(&"ordinary-status-fault".to_string()));
}

#[tokio::test]
async fn exact_stop_terminates_known_process_after_history_store_read_failure() {
    let temp = tempdir().unwrap();
    let repo = temp.path();
    let config = PytxoConfig::default();
    let data_dir = repo.join(&config.data_dir);
    let store = PytxoStore::open(&config.db_path_at(repo)).unwrap();
    let (mission, _) = registered_ready_mission(repo, "history-fault");
    store
        .insert_run("history-fault", &repo.to_string_lossy())
        .unwrap();
    store.register_routing_mission(&mission).unwrap();
    store.finish_run("history-fault", "failed_startup").unwrap();
    let marker = write_active_run(repo, "history-fault");
    let workspace = repo.join("history-workspace");
    fs::create_dir(&workspace).unwrap();
    fs::write(
        workspace.join("keep.txt"),
        "retain while Store is uncertain",
    )
    .unwrap();
    let child = spawn_tracked_sleep_child(repo, "history-fault", &workspace);
    let conn = rusqlite::Connection::open(config.db_path_at(repo)).unwrap();
    conn.execute(
        "UPDATE routing_missions SET registration_json='{' WHERE run_id=?1",
        ["history-fault"],
    )
    .unwrap();

    let error = stop_exact(None, Some(repo.to_path_buf()), "history-fault", true)
        .await
        .expect_err("malformed registered history must require recovery");
    assert!(error.to_string().contains("unconfirmed"), "{error:#}");
    assert!(marker.exists(), "unknown mission history retains marker");
    assert!(
        workspace.join("keep.txt").exists(),
        "unknown history retains workspace"
    );
    assert!(
        !process_matches(child.pid, &child.identity).unwrap(),
        "known child must terminate"
    );
    assert!(ProcessRegistryFile::load(&registry_path(&data_dir))
        .unwrap()
        .cancelled_runs
        .contains(&"history-fault".to_string()));
    assert_eq!(
        store.get_run_status("history-fault").unwrap().unwrap().0,
        "failed_startup"
    );
    let route_epoch: (i64, i64) = conn
        .query_row(
            "SELECT cancel_epoch,cancelled FROM routing_missions WHERE run_id=?1",
            ["history-fault"],
            |row| Ok((row.get(0)?, row.get(1)?)),
        )
        .unwrap();
    assert_eq!(
        route_epoch,
        (0, 0),
        "unknown mission was not falsely fenced"
    );
}

#[test]
fn stop_all_finds_missing_marker_unresolved_attempt_and_keeps_recovery_owned() {
    if std::env::var_os(ISOLATED_RECOVERY_SENTINEL).is_some() {
        tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .unwrap()
            .block_on(run_missing_marker_recovery_scenario());
        return;
    }
    let home = tempdir().unwrap();
    let status = Command::new(std::env::current_exe().unwrap())
        .arg("stop_all_finds_missing_marker_unresolved_attempt_and_keeps_recovery_owned")
        .arg("--exact")
        .arg("--nocapture")
        .env("PYTXO_HOME", home.path())
        .env(
            "PYTXO_TRUST_STORE",
            home.path().join("trusted-domains.json"),
        )
        .env(ISOLATED_RECOVERY_SENTINEL, "1")
        .status()
        .unwrap();
    assert!(status.success(), "isolated recovery scenario failed");
}

#[test]
fn markerless_ready_registration_is_recovered_across_processes() {
    if let Some(repo) = std::env::var_os(MARKERLESS_RESTART_REPO) {
        tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .unwrap()
            .block_on(run_markerless_restart_recovery_scenario(
                std::path::PathBuf::from(repo),
            ));
        return;
    }
    let temp = tempdir().unwrap();
    let repo = temp.path();
    init_git_repo(repo);
    let config = PytxoConfig::default();
    let store = PytxoStore::open(&config.db_path_at(repo)).unwrap();
    let (mission, scope) = registered_ready_mission(repo, "route-markerless-ready");
    store
        .insert_starting_run_with_profile(
            "route-markerless-ready",
            &repo.to_string_lossy(),
            Some("orbit"),
        )
        .unwrap();
    store.register_routing_mission(&mission).unwrap();
    let marker = write_active_run(repo, "route-markerless-ready");
    fs::remove_file(&marker).unwrap();
    drop(store);

    let home = tempdir().unwrap();
    let status = Command::new(std::env::current_exe().unwrap())
        .arg("markerless_ready_registration_is_recovered_across_processes")
        .arg("--exact")
        .arg("--nocapture")
        .env("PYTXO_HOME", home.path())
        .env(
            "PYTXO_TRUST_STORE",
            home.path().join("trusted-domains.json"),
        )
        .env(MARKERLESS_RESTART_REPO, repo)
        .status()
        .unwrap();
    assert!(status.success(), "fresh-process recovery failed");
    let reopened = PytxoStore::open(&config.db_path_at(repo)).unwrap();
    let history = reopened.routing_history(&scope).unwrap().unwrap();
    assert!(history.cancelled);
    assert_eq!(history.cancel_epoch, 1);
    assert!(history.attempts.is_empty());
    assert_eq!(
        reopened
            .get_run_status("route-markerless-ready")
            .unwrap()
            .unwrap()
            .0,
        "cancelled"
    );
}

async fn run_markerless_restart_recovery_scenario(repo: std::path::PathBuf) {
    trust_repo(&repo, PermissionProfile::Orbit).unwrap();
    let config = PytxoConfig::default();
    let store = PytxoStore::open(&config.db_path_at(&repo)).unwrap();
    let denied_payload = repo.join("denied-claim-payload.txt");
    let scope = RoutingScope {
        domain_id: DomainId::from_repo_root(&repo).unwrap(),
        run_id: RunId("route-markerless-ready".into()),
    };
    let claim_error = match dispatch(RunOptions {
        agents: 1,
        cmd: format!("echo should-not-run > \"{}\"", denied_payload.display()),
        config: None,
        dry_run: false,
        keep_worktrees: false,
        repo: Some(repo.clone()),
        execution: Some(ExecutionBackend::Subprocess),
        project: None,
        tasks: None,
        task_cmd_template: None,
        task_prompts: None,
    }) {
        Err(error) => error,
        Ok(_) => {
            let _ = stop(None, Some(repo.clone()), true, false).await;
            panic!("markerless registered mission allowed a new claim");
        }
    };
    assert!(
        claim_error.to_string().contains("route-markerless-ready"),
        "{claim_error:#}"
    );
    assert!(!denied_payload.exists(), "rejected claim ran its payload");
    assert!(
        store
            .routing_history(&scope)
            .unwrap()
            .unwrap()
            .attempts
            .is_empty(),
        "rejected claim cannot create a routed attempt"
    );
    assert!(!repo.join(&config.data_dir).join("active_run.json").exists());
    assert!(
        ProcessRegistryFile::load(&registry_path(&repo.join(&config.data_dir)))
            .unwrap()
            .entries
            .is_empty()
    );

    stop(None, Some(repo.clone()), true, false)
        .await
        .expect("fresh process must cancel markerless Ready mission");
    let history = store.routing_history(&scope).unwrap().unwrap();
    assert!(history.cancelled);
    assert_eq!(history.cancel_epoch, 1);
    assert_eq!(
        store
            .get_run_status("route-markerless-ready")
            .unwrap()
            .unwrap()
            .0,
        "cancelled"
    );

    let (_, newer_id) = dispatch(RunOptions {
        agents: 1,
        cmd: "echo later-claim".into(),
        config: None,
        dry_run: false,
        keep_worktrees: false,
        repo: Some(repo.clone()),
        execution: Some(ExecutionBackend::Subprocess),
        project: None,
        tasks: None,
        task_cmd_template: None,
        task_prompts: None,
    })
    .expect("new claim must proceed after durable cancellation");
    if !wait_until(|| {
        store
            .get_run_status(&newer_id.0)
            .unwrap()
            .is_some_and(|(status, _)| PytxoStore::is_terminal_run_status(&status))
    })
    .await
    {
        let _ = stop(None, Some(repo), true, false).await;
        panic!("post-recovery legacy fixture run did not settle");
    }
}

async fn run_missing_marker_recovery_scenario() {
    let temp = tempdir().unwrap();
    let repo = temp.path();
    init_git_repo(repo);
    trust_repo(repo, PermissionProfile::Orbit).unwrap();
    let config = PytxoConfig::default();
    let store = PytxoStore::open(&config.db_path_at(repo)).unwrap();
    let (mission, scope) = registered_ready_mission(repo, "route-ambiguous");
    store
        .insert_run("route-ambiguous", &repo.to_string_lossy())
        .unwrap();
    store.register_routing_mission(&mission).unwrap();
    seed_unresolved_attempt(repo, &scope, &mission);

    let error = stop(None, Some(repo.to_path_buf()), true, false)
        .await
        .expect_err("unresolved actor must retain recovery ownership");
    assert!(error.to_string().contains("unresolved routed attempt"));
    let history = store.routing_history(&scope).unwrap().unwrap();
    assert!(history.cancelled);
    assert_eq!(history.cancel_epoch, 1);
    assert!(!history.attempts[0].ownership_released);
    let registry = ProcessRegistryFile::load(&registry_path(&repo.join(&config.data_dir))).unwrap();
    assert!(registry
        .cancelled_runs
        .contains(&"route-ambiguous".to_string()));
    assert!(!repo.join(&config.data_dir).join("active_run.json").exists());

    let claim_error = dispatch(RunOptions {
        agents: 1,
        cmd: "echo should-not-run".into(),
        config: None,
        dry_run: false,
        keep_worktrees: false,
        repo: Some(repo.to_path_buf()),
        execution: None,
        project: None,
        tasks: None,
        task_cmd_template: None,
        task_prompts: None,
    })
    .expect_err("unresolved routed ownership must block a new domain claim");
    assert!(claim_error
        .to_string()
        .contains("unresolved routed attempt"));
}

#[test]
fn new_claim_defers_registered_ready_mission_until_explicit_stop() {
    if std::env::var_os(ISOLATED_CRASH_RECONCILE_SENTINEL).is_some() {
        tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .unwrap()
            .block_on(run_crashed_ready_registration_scenario());
        return;
    }
    let home = tempdir().unwrap();
    let status = Command::new(std::env::current_exe().unwrap())
        .arg("new_claim_defers_registered_ready_mission_until_explicit_stop")
        .arg("--exact")
        .arg("--nocapture")
        .env("PYTXO_HOME", home.path())
        .env(
            "PYTXO_TRUST_STORE",
            home.path().join("trusted-domains.json"),
        )
        .env(ISOLATED_CRASH_RECONCILE_SENTINEL, "1")
        .status()
        .unwrap();
    assert!(status.success(), "isolated crash reconciliation failed");
}

async fn run_crashed_ready_registration_scenario() {
    let temp = tempdir().unwrap();
    let repo = temp.path();
    init_git_repo(repo);
    trust_repo(repo, PermissionProfile::Orbit).unwrap();
    let config = PytxoConfig::default();
    let store = PytxoStore::open(&config.db_path_at(repo)).unwrap();
    let (mission, scope) = registered_ready_mission(repo, "route-crashed-starting");
    store
        .insert_starting_run_with_profile(
            "route-crashed-starting",
            &repo.to_string_lossy(),
            Some("orbit"),
        )
        .unwrap();
    store.register_routing_mission(&mission).unwrap();
    write_active_run(repo, "route-crashed-starting");

    let refused = dispatch(RunOptions {
        agents: 1,
        cmd: "echo newer-run".into(),
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
    .expect_err("new claim must leave the registered routed mission to exact recovery");
    assert!(refused.to_string().contains("routed Flow recovery"));
    let history = store.routing_history(&scope).unwrap().unwrap();
    assert!(!history.cancelled);
    assert_eq!(history.cancel_epoch, 0);
    assert!(history.attempts.is_empty());
    assert_eq!(
        store
            .get_run_status("route-crashed-starting")
            .unwrap()
            .unwrap()
            .0,
        "starting"
    );
    assert!(
        !ProcessRegistryFile::load(&registry_path(&repo.join(&config.data_dir)))
            .unwrap()
            .cancelled_runs
            .contains(&"route-crashed-starting".to_string())
    );
    stop_exact(
        None,
        Some(repo.to_path_buf()),
        "route-crashed-starting",
        false,
    )
    .await
    .expect("explicit Stop fences the registered mission");
    assert!(store.routing_history(&scope).unwrap().unwrap().cancelled);
    let (_, newer_id) = dispatch(RunOptions {
        agents: 1,
        cmd: "echo newer-run".into(),
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
    .expect("new claim may proceed after explicit Stop");
    if let Ok(active) = fs::read_to_string(repo.join(&config.data_dir).join("active_run.json")) {
        assert!(
            !active.contains("route-crashed-starting"),
            "new run must have a distinct active identity"
        );
    }
    if !wait_until(|| {
        store
            .get_run_status(&newer_id.0)
            .unwrap()
            .is_some_and(|(status, _)| PytxoStore::is_terminal_run_status(&status))
    })
    .await
    {
        let _ = stop(None, Some(repo.to_path_buf()), true, false).await;
        panic!("new fixture run did not settle");
    }
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
        execution: None,
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
                .and_then(|registry| registry.for_run(&run_id.0).first().cloned().cloned())
                .is_some_and(|entry| {
                    entry.start_identity.is_some()
                        && std::path::Path::new(&entry.worktree_path)
                            .join("child.pid")
                            .exists()
                });
            status_is_running && child_is_persisted
        })
        .await;
        if !started {
            let status = PytxoStore::open(&db_path)
                .ok()
                .and_then(|store| store.get_run_status(&run_id.0).ok().flatten());
            let entries = ProcessRegistryFile::load(&registry_path(&data_dir))
                .map(|registry| registry.for_run(&run_id.0).into_iter().cloned().collect::<Vec<_>>())
                .map_err(|error| format!("load timeout registry: {error}"))?;
            let details = entries
                .iter()
                .map(|entry| {
                    format!(
                        "pid={} identity={} child_pid_file={} worktree={}",
                        entry.pid,
                        entry.start_identity.is_some(),
                        std::path::Path::new(&entry.worktree_path).join("child.pid").exists(),
                        entry.worktree_path
                    )
                })
                .collect::<Vec<_>>()
                .join("; ");
            return Err(format!(
                "timed out waiting for the dispatched blocking child; status={status:?}; registry=[{details}]"
            ));
        }

        let root_entry: ProcessEntry = ProcessRegistryFile::load(&registry_path(&data_dir))
            .map_err(|error| format!("load live registry: {error}"))?
            .for_run(&run_id.0)
            .first()
            .cloned()
            .cloned()
            .ok_or_else(|| "live PTY process identity was not durable".to_string())?;
        let root_identity = root_entry
            .start_identity
            .clone()
            .ok_or_else(|| "live PTY entry has no process start identity".to_string())?;
        if !process_matches(root_entry.pid, &root_identity)
            .map_err(|error| format!("verify live PTY identity: {error}"))?
        {
            return Err("durable PTY identity does not match the live process".into());
        }
        let descendant_pid =
            fs::read_to_string(std::path::Path::new(&root_entry.worktree_path).join("child.pid"))
                .map_err(|error| format!("read descendant pid: {error}"))?
                .trim()
                .parse::<u32>()
                .map_err(|error| format!("parse descendant pid: {error}"))?;
        let descendant_identity = process_start_identity(descendant_pid)
            .map_err(|error| format!("read descendant identity: {error}"))?
            .ok_or_else(|| "PTY descendant was not live before Stop".to_string())?;

        stop_exact(None, Some(repo.to_path_buf()), &run_id.0, false)
            .await
            .map_err(|error| format!("stop the exact active run: {error}"))?;

        if process_matches(root_entry.pid, &root_identity)
            .map_err(|error| format!("verify PTY root exit: {error}"))?
        {
            return Err("Stop returned while the PTY root was still live".into());
        }
        if process_matches(descendant_pid, &descendant_identity)
            .map_err(|error| format!("verify PTY descendant exit: {error}"))?
        {
            return Err("Stop returned while a PTY descendant was still live".into());
        }
        if !ProcessRegistryFile::load(&registry_path(&data_dir))
            .map_err(|error| format!("load registry after Stop: {error}"))?
            .for_run(&run_id.0)
            .is_empty()
        {
            return Err("confirmed process identity remained in the live registry".into());
        }

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
