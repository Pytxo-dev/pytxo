use pytxo_core::routing::*;
use pytxo_core::{DomainId, ExecutionBackend, PermissionProfile, RunId, TaskId};
use pytxo_store::capacity::{
    CapacityBindRequest, CapacityBoundReleaseRequest, CapacityOwner, CapacityPoolConfig,
    CapacityReleaseEvidence, CapacityReleaseEvidenceKind, CapacityReleaseRequest,
    CapacityReservationRequest, CapacityResourceRequest,
};
use pytxo_store::routing_capacity_intent::{
    AdmittedNoLaunchRelease, AdmittedQuiescentRelease, CapacityIntentPhase,
    QuiescentCheckerReceipt, RoutingCapacityIntentRequest,
};
use pytxo_store::routing_checker::{
    CheckerCreateOutcome, CheckerCreateRequest, CheckerOwnershipPhase, CheckerOwnershipRecord,
    CheckerOwnershipRequest, CheckerProcessRegistration, CheckerSettlement,
};
use pytxo_store::routing_launch::{
    LaunchCreateOutcome, LaunchCreateRequest, LaunchOwnershipPhase, LaunchOwnershipRequest,
    LaunchProcessRegistration, LaunchSettlement, OwnedJobStopKind, OwnedJobStopPhase,
};
use pytxo_store::routing_private::{
    CheckerNativeOutcome, ControllerObservation, ControllerReceiptEnvelope, PrivateArtifactClaim,
    PrivateArtifactKind, ReceiptSource, RoutingArtifactError,
};
use pytxo_store::{routing::*, Catalog, PytxoStore, RoutedWorktreeInstance};
use std::collections::BTreeSet;

fn registration(count: usize) -> (RoutingMission, RoutingFacts, RoutingScope) {
    let (s, candidates, policy) = fixture();
    let mut auth = s.authorization;
    auth.limits.max_workers = 2;
    let tasks: Vec<_> = (0..count)
        .map(|i| {
            let mut contract = s.task.clone();
            contract.task_id = TaskId(format!("task{i}"));
            RegisteredTask {
                contract,
                attempt_budget_nano_usd: 60,
                check_recipes: vec![],
            }
        })
        .collect();
    auth.allowed_task_digests = tasks.iter().map(|t| t.contract.digest().unwrap()).collect();
    let scope = RoutingScope {
        domain_id: auth.domain_id.clone(),
        run_id: auth.run_id.clone(),
    };
    let facts = RoutingFacts {
        now_ms: 150,
        observed_at_ms: 100,
        expires_at_ms: 200,
        base: s.task.base,
        plan_digest: s.task.plan_digest,
        permission_profile: PermissionProfile::Orbit,
        observations: candidates.iter().map(|c| c.observation.clone()).collect(),
        manual_target: None,
        packet_digest: None,
        advice_request_id: None,
    };
    (
        RoutingMission {
            authorization: auth,
            policy,
            tasks,
            profiles: candidates
                .into_iter()
                .map(|c| RegisteredProfile {
                    profile: c.profile,
                    binding: c.binding,
                })
                .collect(),
        },
        facts,
        scope,
    )
}

#[test]
fn pre_recipe_v8_v9_json_and_digest_round_trip() {
    let registration_json = include_str!("fixtures/routing_pre_check_recipes_v8_registration.json");
    let task_json = include_str!("fixtures/routing_pre_check_recipes_v8_task_record.json");
    let stage_json = include_str!("fixtures/routing_pre_check_recipes_v9_stage.json");
    let digest = include_str!("fixtures/routing_pre_check_recipes_v8_mission_digest.txt");
    let registration_digest =
        include_str!("fixtures/routing_pre_check_recipes_v8_registration_digest.txt");
    assert_eq!(registration_json, stage_json);
    assert_eq!(digest, registration_digest);
    let mission: RoutingMission = serde_json::from_str(registration_json).unwrap();
    let task: RoutedTaskRecord = serde_json::from_str(task_json).unwrap();
    assert_eq!(serde_json::to_string(&mission).unwrap(), registration_json);
    assert_eq!(serde_json::to_string(&task).unwrap(), task_json);
    assert_eq!(canonical_digest(&mission, 1).unwrap().0, digest);
    assert_eq!(task.registration, mission.tasks[0]);

    let temp = tempfile::tempdir().unwrap();
    let path = temp.path().join("store.db");
    let db = PytxoStore::open(&path).unwrap();
    let scope = RoutingScope {
        domain_id: mission.authorization.domain_id.clone(),
        run_id: mission.authorization.run_id.clone(),
    };
    let reviewed = db.stage_routing_mission("golden-draft", &mission).unwrap();
    let conn = rusqlite::Connection::open(&path).unwrap();
    let staged_json: String = conn
        .query_row(
            "SELECT mission_json FROM routing_mission_stages WHERE run_id=?1",
            [&scope.run_id.0],
            |row| row.get(0),
        )
        .unwrap();
    assert_eq!(staged_json, stage_json);
    assert_eq!(reviewed.mission_digest.0, digest);
    drop(conn);
    drop(db);
    let db = PytxoStore::open(&path).unwrap();
    assert_eq!(db.load_staged_routing_mission(&reviewed).unwrap(), mission);
    db.insert_run(&scope.run_id.0, "repo").unwrap();
    db.register_routing_mission(&mission).unwrap();
    db.register_routing_mission(&mission).unwrap();
    let conn = rusqlite::Connection::open(&path).unwrap();
    let stored_registration_json: String = conn
        .query_row(
            "SELECT registration_json FROM routing_missions WHERE run_id=?1",
            [&scope.run_id.0],
            |row| row.get(0),
        )
        .unwrap();
    let stored_registration_digest: String = conn
        .query_row(
            "SELECT registration_digest FROM routing_missions WHERE run_id=?1",
            [&scope.run_id.0],
            |row| row.get(0),
        )
        .unwrap();
    let stored_task_json: String = conn
        .query_row(
            "SELECT record_json FROM routing_tasks WHERE run_id=?1 AND task_id='task0'",
            [&scope.run_id.0],
            |row| row.get(0),
        )
        .unwrap();
    assert_eq!(stored_registration_json, registration_json);
    assert_eq!(stored_registration_digest, digest);
    assert_eq!(stored_task_json, task_json);
    assert_eq!(
        db.routing_history(&scope).unwrap().unwrap().mission,
        mission
    );
    assert!(require_launchable_check_recipes(&mission).is_err());
}

fn frozen_fixture(command: &str, ordinal: u32) -> FrozenCheckRecipeV1 {
    let platform = if cfg!(windows) {
        CheckPlatform::Windows
    } else {
        CheckPlatform::Posix
    };
    FrozenCheckRecipeV1 {
        schema_version: 1,
        id: CheckId(format!("task0:verify:{ordinal:04}")),
        ordinal,
        command: command.into(),
        executor: FrozenCheckExecutorV1 {
            policy_version: 1,
            platform,
            shell: ExecutableIdentity {
                path: if cfg!(windows) {
                    "C:\\Windows\\System32\\cmd.exe".into()
                } else {
                    "/bin/sh".into()
                },
                version: "fixture".into(),
                digest: digest("shell-file"),
            },
            shell_args: if cfg!(windows) {
                vec!["/D".into(), "/C".into()]
            } else {
                vec!["-c".into()]
            },
            cwd_kind: CheckCwdKind::FreshSealedVerificationView,
            permission_profile: PermissionProfile::Orbit,
            stdin_closed: true,
            environment_policy_version: 1,
            network_policy_version: 1,
            timeout_ms: 120_000,
            max_stdout_bytes: 262_144,
            max_stderr_bytes: 262_144,
        },
    }
}

fn with_frozen_checks(commands: &[&str]) -> RoutingMission {
    let (mut mission, _, _) = registration(1);
    let recipes: Vec<_> = commands
        .iter()
        .enumerate()
        .map(|(index, command)| frozen_fixture(command, index as u32 + 1))
        .collect();
    mission.tasks[0].contract.checks = recipes
        .iter()
        .map(|recipe| recipe.reference().unwrap())
        .collect();
    mission.tasks[0].check_recipes = recipes;
    mission.authorization.allowed_task_digests =
        BTreeSet::from([mission.tasks[0].contract.digest().unwrap()]);
    mission
}

#[test]
fn frozen_check_references_bind_full_ordered_private_recipes() {
    let mission = with_frozen_checks(&["cargo test", "cargo clippy"]);
    require_launchable_check_recipes(&mission).unwrap();
    let temp = tempfile::tempdir().unwrap();
    let db = PytxoStore::open(&temp.path().join("store.db")).unwrap();
    let reviewed = db.stage_routing_mission("frozen-draft", &mission).unwrap();
    assert_eq!(db.load_staged_routing_mission(&reviewed).unwrap(), mission);
    let mut changed = mission.clone();
    changed.tasks[0].check_recipes.swap(0, 1);
    assert!(db.stage_routing_mission("changed-order", &changed).is_err());
    changed = mission.clone();
    changed.tasks[0].check_recipes[0]
        .command
        .push_str(" --changed");
    assert!(db
        .stage_routing_mission("changed-command", &changed)
        .is_err());
    changed = mission.clone();
    changed.tasks[0].check_recipes[0].executor.shell.digest = digest("changed-shell");
    assert!(db.stage_routing_mission("changed-shell", &changed).is_err());
    changed = mission.clone();
    changed.tasks[0].check_recipes[0]
        .executor
        .network_policy_version = 2;
    assert!(changed.tasks[0].check_recipes[0].reference().is_err());
    changed = mission.clone();
    changed.tasks[0].check_recipes.clear();
    assert!(require_launchable_check_recipes(&changed).is_err());
    let mut unknown = serde_json::to_value(&mission.tasks[0].check_recipes[0]).unwrap();
    unknown["unreviewed"] = serde_json::Value::Bool(true);
    assert!(serde_json::from_value::<FrozenCheckRecipeV1>(unknown).is_err());
}

#[test]
fn strict_registered_recipe_read_rejects_projection_or_digest_drift() {
    let mission = with_frozen_checks(&["cargo test"]);
    let temp = tempfile::tempdir().unwrap();
    let path = temp.path().join("store.db");
    let db = PytxoStore::open(&path).unwrap();
    let scope = RoutingScope {
        domain_id: mission.authorization.domain_id.clone(),
        run_id: mission.authorization.run_id.clone(),
    };
    db.insert_run(&scope.run_id.0, "repo").unwrap();
    db.register_routing_mission(&mission).unwrap();
    assert_eq!(
        db.load_registered_mission_with_recipe_integrity(&scope)
            .unwrap(),
        mission
    );
    let conn = rusqlite::Connection::open(&path).unwrap();
    conn.execute(
        "UPDATE routing_missions SET registration_digest=?1 WHERE run_id=?2",
        rusqlite::params![digest("wrong").0, scope.run_id.0],
    )
    .unwrap();
    assert!(db
        .load_registered_mission_with_recipe_integrity(&scope)
        .is_err());
    conn.execute(
        "UPDATE routing_missions SET registration_digest=?1 WHERE run_id=?2",
        rusqlite::params![canonical_digest(&mission, 1).unwrap().0, scope.run_id.0],
    )
    .unwrap();
    let record: String = conn
        .query_row(
            "SELECT record_json FROM routing_tasks WHERE run_id=?1 AND task_id='task0'",
            [&scope.run_id.0],
            |row| row.get(0),
        )
        .unwrap();
    let mut record: RoutedTaskRecord = serde_json::from_str(&record).unwrap();
    record.registration.check_recipes[0]
        .command
        .push_str(" --drift");
    conn.execute(
        "UPDATE routing_tasks SET record_json=?1 WHERE run_id=?2 AND task_id='task0'",
        rusqlite::params![serde_json::to_string(&record).unwrap(), scope.run_id.0],
    )
    .unwrap();
    assert!(db
        .load_registered_mission_with_recipe_integrity(&scope)
        .is_err());
    assert!(db.routing_history(&scope).unwrap().is_some());
}
fn setup(
    count: usize,
) -> (
    tempfile::TempDir,
    PytxoStore,
    RoutingMission,
    RoutingFacts,
    RoutingScope,
) {
    let temp = tempfile::tempdir().unwrap();
    let db = PytxoStore::open(&temp.path().join("store.db")).unwrap();
    let (mission, facts, scope) = registration(count);
    db.insert_run("run", "repo").unwrap();
    install(&db, &mission, &facts, &scope);
    (temp, db, mission, facts, scope)
}
fn install(db: &PytxoStore, m: &RoutingMission, f: &RoutingFacts, s: &RoutingScope) {
    db.register_routing_mission(m).unwrap();
    for (i, o) in f.observations.iter().enumerate() {
        db.register_routing_qualification(
            s,
            &format!("qual{i}"),
            o.qualification.as_ref().unwrap(),
        )
        .unwrap();
    }
}
fn request(db: &PytxoStore, s: &RoutingScope, f: &RoutingFacts, id: &str) -> AdmitRoutingAttempt {
    AdmitRoutingAttempt {
        scope: s.clone(),
        event_id: format!("admit-{id}"),
        task_id: TaskId(id.into()),
        attempt_id: AttemptId(format!("attempt-{id}")),
        agent_id: format!("agent-{id}"),
        facts: f.clone(),
        decision: db
            .preview_routing_decision(s, &TaskId(id.into()), f, None)
            .unwrap(),
        capacity_reservation: format!("capacity-{id}"),
        input_manifest: BlobRef {
            digest: digest("input"),
            byte_length: 5,
        },
        handoff: None,
        advice_json: None,
        observation_event_id: None,
    }
}

#[test]
fn admission_rejects_handoff_reference_without_retained_owned_bytes() {
    let (_, db, _, facts, scope) = setup(1);
    let mut admission = request(&db, &scope, &facts, "task0");
    admission.handoff = Some(BlobRef {
        digest: Digest::of_bytes(b"unretained handoff"),
        byte_length: b"unretained handoff".len() as u64,
    });
    assert!(db.admit_routing_attempt(&admission).is_err());
}

#[test]
fn admission_binds_retained_handoff_to_exact_attempt_and_reviewed_task() {
    let (temp, db, catalog, intent) = capacity_intent_fixture();
    db.register_capacity_intent(&catalog, &intent).unwrap();
    let input_claim = input_artifact_claim(&intent);
    let input = db.put_private_artifact(&input_claim, b"input").unwrap();
    let (_, facts, _) = registration(1);
    let snapshot = db
        .routing_snapshot(&intent.scope, &intent.task_id, &facts)
        .unwrap();
    let mut admission = request(&db, &intent.scope, &facts, "task0");
    admission.attempt_id = AttemptId(intent.reservation.attempt_id.clone());
    admission.capacity_reservation = intent.reservation.reservation_id.clone();
    admission.input_manifest = input;
    let handoff = PortableHandoffManifest {
        schema_version: 1,
        canonicalization_version: 1,
        manifest_digest: Digest::of_bytes(b"placeholder"),
        origin: HandoffOrigin {
            domain_id: intent.scope.domain_id.clone(),
            run_id: intent.scope.run_id.clone(),
            task_id: intent.task_id.clone(),
            attempt_id: admission.attempt_id.clone(),
            task_revision: snapshot.task.revision,
            plan_id: snapshot.plan_id.clone(),
            plan_digest: snapshot.task.plan_digest.clone(),
            authorization_revision: snapshot.authorization.revision,
        },
        task_contract_ref: snapshot.task.digest().unwrap(),
        base: snapshot.task.base.clone(),
        dependencies: snapshot.resolved_dependencies.clone(),
        repair_input: None,
        changes: vec![],
        evidence: vec![],
        requirements: HandoffRequirements {
            capabilities: snapshot.task.required_capabilities.clone(),
            skill_tool_bundle_digest: snapshot.task.skill_tool_bundle_digest.clone(),
        },
        notes: vec![],
    }
    .seal()
    .unwrap();
    let handoff_claim = pytxo_store::routing_private::handoff_manifest_claim(
        &intent.scope,
        &intent.task_id,
        &admission.attempt_id,
        &admission.capacity_reservation,
    );
    let handoff_bytes = serde_json::to_vec(&handoff).unwrap();
    admission.handoff = Some(
        db.put_private_artifact(&handoff_claim, &handoff_bytes)
            .unwrap(),
    );
    db.mark_capacity_reserve_may_have_started(&intent.reservation.reservation_id, "reserve-start")
        .unwrap();
    catalog.reserve_capacity(&intent.reservation).unwrap();
    let admitted = db.admit_routing_attempt(&admission).unwrap();
    assert_eq!(admitted.handoff, admission.handoff);
    assert_eq!(
        db.read_private_artifact(&handoff_claim, admitted.handoff.as_ref().unwrap())
            .unwrap(),
        handoff_bytes
    );
    assert_eq!(
        db.read_routing_attempt_handoff(&admitted).unwrap(),
        Some(handoff)
    );
    let conn = rusqlite::Connection::open(temp.path().join("store.db")).unwrap();
    conn.execute(
        "UPDATE routing_private_artifacts SET bytes=X'00' WHERE artifact_id=?1",
        [&handoff_claim.artifact_id],
    )
    .unwrap();
    assert!(db.read_routing_attempt_handoff(&admitted).is_err());
}

#[test]
fn admission_rejects_handoff_payloads_without_retained_blob_verification() {
    let (_, db, catalog, intent) = capacity_intent_fixture();
    db.register_capacity_intent(&catalog, &intent).unwrap();
    let input = db
        .put_private_artifact(&input_artifact_claim(&intent), b"input")
        .unwrap();
    let (_, facts, _) = registration(1);
    let snapshot = db
        .routing_snapshot(&intent.scope, &intent.task_id, &facts)
        .unwrap();
    let mut admission = request(&db, &intent.scope, &facts, "task0");
    admission.attempt_id = AttemptId(intent.reservation.attempt_id.clone());
    admission.capacity_reservation = intent.reservation.reservation_id.clone();
    admission.input_manifest = input;
    let handoff = PortableHandoffManifest {
        schema_version: 1,
        canonicalization_version: 1,
        manifest_digest: Digest::of_bytes(b"placeholder"),
        origin: HandoffOrigin {
            domain_id: intent.scope.domain_id.clone(),
            run_id: intent.scope.run_id.clone(),
            task_id: intent.task_id.clone(),
            attempt_id: admission.attempt_id.clone(),
            task_revision: snapshot.task.revision,
            plan_id: snapshot.plan_id.clone(),
            plan_digest: snapshot.task.plan_digest.clone(),
            authorization_revision: snapshot.authorization.revision,
        },
        task_contract_ref: snapshot.task.digest().unwrap(),
        base: snapshot.task.base.clone(),
        dependencies: snapshot.resolved_dependencies.clone(),
        repair_input: None,
        changes: vec![],
        evidence: vec![],
        requirements: HandoffRequirements {
            capabilities: snapshot.task.required_capabilities.clone(),
            skill_tool_bundle_digest: snapshot.task.skill_tool_bundle_digest.clone(),
        },
        notes: vec![ClaimedWorkerNote {
            text: "unverified note".into(),
            provenance: NoteProvenance::Claimed,
        }],
    }
    .seal()
    .unwrap();
    let claim = pytxo_store::routing_private::handoff_manifest_claim(
        &intent.scope,
        &intent.task_id,
        &admission.attempt_id,
        &admission.capacity_reservation,
    );
    admission.handoff = Some(
        db.put_private_artifact(&claim, &serde_json::to_vec(&handoff).unwrap())
            .unwrap(),
    );
    db.mark_capacity_reserve_may_have_started(&intent.reservation.reservation_id, "reserve-start")
        .unwrap();
    catalog.reserve_capacity(&intent.reservation).unwrap();
    assert!(db
        .admit_routing_attempt(&admission)
        .unwrap_err()
        .to_string()
        .contains("unsupported unretained payload references"));
}
fn step(
    db: &PytxoStore,
    s: &RoutingScope,
    f: &RoutingFacts,
    a: &RoutedAttemptRecord,
    to: AttemptState,
    receipts: RoutingReceipts,
) -> pytxo_core::Result<RoutedAttemptRecord> {
    let h = db.routing_history(s)?.unwrap();
    let task = h
        .tasks
        .iter()
        .find(|t| t.registration.contract.task_id == a.task_id)
        .unwrap();
    db.transition_routing_attempt(&TransitionRoutingAttempt {
        scope: s.clone(),
        event_id: format!("{}-{}-{}", a.attempt_id, a.revision, to.as_str()),
        attempt_id: a.attempt_id.clone(),
        expected_attempt_revision: a.revision,
        expected_task_revision: task.revision,
        to,
        facts: f.clone(),
        receipts,
        failure: None,
    })
}
fn advance(
    db: &PytxoStore,
    s: &RoutingScope,
    f: &RoutingFacts,
    mut a: RoutedAttemptRecord,
    to: AttemptState,
) -> RoutedAttemptRecord {
    let states = [
        AttemptState::Preparing,
        AttemptState::Launching,
        AttemptState::Running,
        AttemptState::Sealing,
        AttemptState::Verifying,
        AttemptState::Passed,
    ];
    for state in states {
        let receipts = match state {
            AttemptState::Preparing => RoutingReceipts {
                inputs: Some(digest("inputs")),
                ..Default::default()
            },
            AttemptState::Launching => RoutingReceipts {
                launch_checks: Some(digest("launch")),
                ..Default::default()
            },
            AttemptState::Running => RoutingReceipts {
                process_identity: Some(digest("process")),
                ..Default::default()
            },
            AttemptState::Sealing => RoutingReceipts {
                quiescence: Some(digest("quiet")),
                ..Default::default()
            },
            AttemptState::Verifying => RoutingReceipts {
                sealed_output: Some(digest("output")),
                ..Default::default()
            },
            AttemptState::Passed => RoutingReceipts {
                checks: Some(digest("checks")),
                ..Default::default()
            },
            _ => unreachable!(),
        };
        a = step(db, s, f, &a, state, receipts).unwrap();
        if state == to {
            break;
        }
    }
    a
}

#[test]
fn registered_recovery_scan_tracks_ready_no_attempt_and_cancellation() {
    let (_, db, _, _, scope) = setup(1);
    assert_eq!(
        db.unreconciled_registered_routing_scopes().unwrap(),
        vec![scope.clone()]
    );
    assert!(db
        .routing_history(&scope)
        .unwrap()
        .unwrap()
        .attempts
        .is_empty());
    db.cancel_routing_mission(&scope, "recover-ready", 0, 200)
        .unwrap();
    assert!(db
        .unreconciled_registered_routing_scopes()
        .unwrap()
        .is_empty());
}

fn capacity_intent_fixture() -> (
    tempfile::TempDir,
    PytxoStore,
    Catalog,
    RoutingCapacityIntentRequest,
) {
    let (temp, db, _, _, scope) = setup(1);
    let catalog = Catalog::open(&temp.path().join("catalog.db")).unwrap();
    catalog
        .configure_capacity_pool(&CapacityPoolConfig {
            resource_id: "fixture-slot".into(),
            capacity_units: 1,
            expected_revision: None,
            configured_at_ms: 100,
        })
        .unwrap();
    let request = RoutingCapacityIntentRequest {
        scope: scope.clone(),
        task_id: TaskId("task0".into()),
        reservation: CapacityReservationRequest {
            reservation_id: "fixture-reservation".into(),
            domain_id: scope.domain_id.0,
            run_id: scope.run_id.0,
            attempt_id: "fixture-attempt".into(),
            owner: CapacityOwner {
                process_id: 41,
                process_start_identity: "test-process-start".into(),
            },
            resources: vec![CapacityResourceRequest {
                resource_id: "fixture-slot".into(),
                units: 1,
            }],
            requested_at_ms: 101,
        },
        event_id: "intent.created.v1".into(),
    };
    (temp, db, catalog, request)
}

fn fixture_unused_proof() -> CapacityReleaseEvidence {
    CapacityReleaseEvidence {
        kind: CapacityReleaseEvidenceKind::KnownUnused,
        receipt_id: "fixture-unused-receipt".into(),
        evidence_digest: "sha256:fixture-unused-proof".into(),
        observed_at_ms: 110,
    }
}

fn input_artifact_claim(request: &RoutingCapacityIntentRequest) -> PrivateArtifactClaim {
    PrivateArtifactClaim {
        scope: request.scope.clone(),
        task_id: request.task_id.clone(),
        attempt_id: AttemptId(request.reservation.attempt_id.clone()),
        reservation_id: request.reservation.reservation_id.clone(),
        kind: PrivateArtifactKind::InputManifest,
        artifact_id: "fixture-input-artifact".into(),
        event_id: "fixture-input-created".into(),
    }
}

#[test]
fn private_artifact_retains_exact_bytes_and_rejects_scope_kind_replay_and_corruption() {
    let (temp, db, catalog, request) = capacity_intent_fixture();
    db.register_capacity_intent(&catalog, &request).unwrap();
    let claim = input_artifact_claim(&request);
    assert!(db
        .read_private_artifact(
            &claim,
            &BlobRef {
                digest: Digest::of_bytes(b"absent"),
                byte_length: 6,
            }
        )
        .is_err());
    let bytes = b"{\"task\":\"fixture\",\"revision\":1}";
    let blob = db.put_private_artifact(&claim, bytes).unwrap();
    assert_eq!(blob.digest, Digest::of_bytes(bytes));
    assert_eq!(blob.byte_length, bytes.len() as u64);
    assert_eq!(db.read_private_artifact(&claim, &blob).unwrap(), bytes);
    assert_eq!(
        db.read_private_artifact_matching_digest(&claim, &blob.digest)
            .unwrap(),
        bytes
    );
    assert!(db
        .read_private_artifact_matching_digest(&claim, &Digest::of_bytes(b"other"))
        .is_err());
    assert_eq!(db.put_private_artifact(&claim, bytes).unwrap(), blob);
    assert!(db.put_private_artifact(&claim, b"changed").is_err());
    let mut wrong = claim.clone();
    wrong.scope.domain_id = DomainId("other-domain".into());
    assert!(db.read_private_artifact(&wrong, &blob).is_err());
    assert!(db
        .read_private_artifact_matching_digest(&wrong, &blob.digest)
        .is_err());
    wrong = claim.clone();
    wrong.kind = PrivateArtifactKind::HandoffManifest;
    assert!(db.read_private_artifact(&wrong, &blob).is_err());
    wrong = claim.clone();
    wrong.event_id = "other-event".into();
    assert!(db.read_private_artifact(&wrong, &blob).is_err());
    let conn = rusqlite::Connection::open(temp.path().join("store.db")).unwrap();
    conn.execute(
        "UPDATE routing_private_artifacts SET bytes=X'00' WHERE artifact_id=?1",
        [&claim.artifact_id],
    )
    .unwrap();
    assert!(db.read_private_artifact(&claim, &blob).is_err());
    assert!(db
        .read_private_artifact_matching_digest(&claim, &blob.digest)
        .is_err());
    drop(catalog);
}

#[test]
fn private_artifact_requires_exact_pre_admission_intent_and_typed_quota() {
    let (_, db, _, _, scope) = setup(1);
    let claim = PrivateArtifactClaim {
        scope,
        task_id: TaskId("task0".into()),
        attempt_id: AttemptId("unadmitted-attempt".into()),
        reservation_id: "missing-reservation".into(),
        kind: PrivateArtifactKind::InputManifest,
        artifact_id: "without-intent".into(),
        event_id: "without-intent-event".into(),
    };
    assert!(db.put_private_artifact(&claim, b"input").is_err());

    let (_, db, catalog, request) = capacity_intent_fixture();
    db.register_capacity_intent(&catalog, &request).unwrap();
    let claim = input_artifact_claim(&request);
    let oversized = vec![0u8; 16 * 1024 * 1024 + 1];
    assert!(matches!(
        db.put_private_artifact(&claim, &oversized),
        Err(RoutingArtifactError::TooLarge { .. })
    ));
    assert!(db
        .capacity_intent(&request.reservation.reservation_id)
        .unwrap()
        .is_some());
    assert!(db
        .read_private_artifact(
            &claim,
            &BlobRef {
                digest: Digest::of_bytes(&oversized),
                byte_length: oversized.len() as u64,
            }
        )
        .is_err());
}

#[test]
fn private_artifact_child_readback() {
    let Ok(path) = std::env::var("PYTXO_PRIVATE_ARTIFACT_CHILD_PATH") else {
        return;
    };
    let claim: PrivateArtifactClaim =
        serde_json::from_str(&std::env::var("PYTXO_PRIVATE_ARTIFACT_CHILD_CLAIM").unwrap())
            .unwrap();
    let reference: BlobRef =
        serde_json::from_str(&std::env::var("PYTXO_PRIVATE_ARTIFACT_CHILD_REF").unwrap()).unwrap();
    let db = PytxoStore::open(std::path::Path::new(&path)).unwrap();
    assert_eq!(
        db.read_private_artifact(&claim, &reference).unwrap(),
        b"restart-exact-bytes"
    );
}

#[test]
fn private_artifact_bytes_survive_distinct_store_process_and_corrupt_quota_denies_write() {
    let (temp, db, catalog, request) = capacity_intent_fixture();
    db.register_capacity_intent(&catalog, &request).unwrap();
    let claim = input_artifact_claim(&request);
    let reference = db
        .put_private_artifact(&claim, b"restart-exact-bytes")
        .unwrap();
    drop(db);
    let path = temp.path().join("store.db");
    let output = std::process::Command::new(std::env::current_exe().unwrap())
        .arg("--exact")
        .arg("private_artifact_child_readback")
        .env("PYTXO_PRIVATE_ARTIFACT_CHILD_PATH", &path)
        .env(
            "PYTXO_PRIVATE_ARTIFACT_CHILD_CLAIM",
            serde_json::to_string(&claim).unwrap(),
        )
        .env(
            "PYTXO_PRIVATE_ARTIFACT_CHILD_REF",
            serde_json::to_string(&reference).unwrap(),
        )
        .output()
        .unwrap();
    assert!(
        output.status.success(),
        "distinct Store process failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    let db = PytxoStore::open(&path).unwrap();
    let conn = rusqlite::Connection::open(&path).unwrap();
    conn.execute(
        "UPDATE routing_private_artifacts SET bytes=X'00' WHERE artifact_id=?1",
        [&claim.artifact_id],
    )
    .unwrap();
    let mut next = claim.clone();
    next.artifact_id = "next-after-corrupt".into();
    next.event_id = "next-after-corrupt-event".into();
    next.kind = PrivateArtifactKind::HandoffManifest;
    assert!(db.put_private_artifact(&next, b"x").is_err());
    assert!(db
        .read_private_artifact(
            &next,
            &BlobRef {
                digest: Digest::of_bytes(b"x"),
                byte_length: 1
            }
        )
        .is_err());
}

#[test]
fn no_worker_receipt_requires_exact_cancelled_provisional_intent() {
    let (_, db, catalog, request) = capacity_intent_fixture();
    db.register_capacity_intent(&catalog, &request).unwrap();
    db.mark_capacity_reserve_may_have_started(
        &request.reservation.reservation_id,
        "reserve-for-no-worker",
    )
    .unwrap();
    catalog.reserve_capacity(&request.reservation).unwrap();
    let claim = PrivateArtifactClaim {
        scope: request.scope.clone(),
        task_id: request.task_id.clone(),
        attempt_id: AttemptId(request.reservation.attempt_id.clone()),
        reservation_id: request.reservation.reservation_id.clone(),
        kind: PrivateArtifactKind::NoWorkerReceipt,
        artifact_id: "no-worker-retained".into(),
        event_id: "no-worker-observed".into(),
    };
    let receipt = ControllerReceiptEnvelope {
        schema_version: 1,
        scope: claim.scope.clone(),
        task_id: claim.task_id.clone(),
        attempt_id: claim.attempt_id.clone(),
        reservation_id: claim.reservation_id.clone(),
        observed_at_ms: 120,
        evidence_id: claim.event_id.clone(),
        source: ReceiptSource::TrustedController,
        observation: ControllerObservation::NoWorker {
            cancellation_event_id: "cancel-no-worker".into(),
        },
    };
    assert!(db
        .put_no_worker_receipt(&catalog, &claim, &receipt)
        .is_err());
    db.cancel_routing_mission(&request.scope, "cancel-no-worker", 0, 121)
        .unwrap();
    let reference = db
        .put_no_worker_receipt(&catalog, &claim, &receipt)
        .unwrap();
    assert_eq!(
        db.read_controller_receipt(&claim, &reference).unwrap(),
        receipt
    );
    let mut wrong = claim.clone();
    wrong.event_id = "substituted".into();
    assert!(db.read_controller_receipt(&wrong, &reference).is_err());
}

#[test]
fn private_artifact_run_quota_is_typed_and_does_not_publish_failing_blob() {
    let (_, db, _, scope, _, _) = launch_ownership_fixture();
    let bytes = vec![7u8; 16 * 1024 * 1024];
    for index in 0..3 {
        let claim = PrivateArtifactClaim {
            scope: scope.clone(),
            task_id: TaskId("task0".into()),
            attempt_id: AttemptId("attempt-task0".into()),
            reservation_id: "launch-reservation".into(),
            kind: PrivateArtifactKind::ScopedOutput,
            artifact_id: format!("scoped-output-{index}"),
            event_id: format!("scoped-output-event-{index}"),
        };
        db.put_private_artifact(&claim, &bytes).unwrap();
    }
    let denied = PrivateArtifactClaim {
        scope,
        task_id: TaskId("task0".into()),
        attempt_id: AttemptId("attempt-task0".into()),
        reservation_id: "launch-reservation".into(),
        kind: PrivateArtifactKind::ScopedOutput,
        artifact_id: "scoped-output-over-quota".into(),
        event_id: "scoped-output-over-quota-event".into(),
    };
    assert!(matches!(
        db.put_private_artifact(&denied, &bytes),
        Err(RoutingArtifactError::RunQuotaExceeded { .. })
    ));
    assert!(db
        .read_private_artifact(
            &denied,
            &BlobRef {
                digest: Digest::of_bytes(&bytes),
                byte_length: bytes.len() as u64,
            }
        )
        .is_err());
}

#[test]
fn zero_byte_artifact_rows_and_oversized_ids_are_bounded() {
    let (temp, db, _, scope, _, _) = launch_ownership_fixture();
    let mut too_long = PrivateArtifactClaim {
        scope: scope.clone(),
        task_id: TaskId("task0".into()),
        attempt_id: AttemptId("attempt-task0".into()),
        reservation_id: "launch-reservation".into(),
        kind: PrivateArtifactKind::ScopedOutput,
        artifact_id: "x".repeat(257),
        event_id: "oversized-id".into(),
    };
    assert!(db.put_private_artifact(&too_long, b"").is_err());
    let mut conn = rusqlite::Connection::open(temp.path().join("store.db")).unwrap();
    let tx = conn.transaction().unwrap();
    let empty_digest = Digest::of_bytes(b"");
    // The launch fixture already retained one input manifest.
    for index in 0..4_095 {
        too_long.artifact_id = format!("empty-{index}");
        too_long.event_id = format!("empty-event-{index}");
        tx.execute(
            "INSERT INTO routing_private_artifacts
             (artifact_id,domain_id,run_id,task_id,attempt_id,reservation_id,kind,event_id,digest,byte_length,claim_json,bytes)
             VALUES (?1,?2,?3,?4,?5,?6,'scoped_output',?7,?8,0,?9,X'')",
            rusqlite::params![too_long.artifact_id, too_long.scope.domain_id.0,
                too_long.scope.run_id.0, too_long.task_id.0, too_long.attempt_id.0,
                too_long.reservation_id, too_long.event_id, empty_digest.0,
                serde_json::to_string(&too_long).unwrap()],
        ).unwrap();
    }
    tx.commit().unwrap();
    too_long.artifact_id = "empty-over-limit".into();
    too_long.event_id = "empty-over-limit-event".into();
    let rejected = db.put_private_artifact(&too_long, b"");
    assert!(
        matches!(
            rejected,
            Err(RoutingArtifactError::RunArtifactCountExceeded { held_count: 4_096 })
        ),
        "unexpected row-cap outcome: {rejected:?}"
    );
}

fn launch_ownership_fixture() -> (
    tempfile::TempDir,
    PytxoStore,
    Catalog,
    RoutingScope,
    RoutingFacts,
    RoutingCapacityIntentRequest,
) {
    launch_ownership_fixture_with_checks(&[], false)
}

fn launch_ownership_fixture_with_checks(
    commands: &[&str],
    dependent: bool,
) -> (
    tempfile::TempDir,
    PytxoStore,
    Catalog,
    RoutingScope,
    RoutingFacts,
    RoutingCapacityIntentRequest,
) {
    let temp = tempfile::tempdir().unwrap();
    let db = PytxoStore::open(&temp.path().join("store.db")).unwrap();
    let (mut mission, facts, scope) = registration(if dependent { 2 } else { 1 });
    if dependent {
        mission.tasks[1].contract.dependencies = vec![TaskId("task0".into())];
        let mut recipe = frozen_fixture("echo child", 1);
        recipe.id = CheckId("task1:verify:0001".into());
        mission.tasks[1].contract.checks = vec![recipe.reference().unwrap()];
        mission.tasks[1].check_recipes = vec![recipe];
        mission.authorization.limits.max_spend_nano_usd = Some(200);
    }
    if !commands.is_empty() {
        let recipes: Vec<_> = commands
            .iter()
            .enumerate()
            .map(|(index, command)| frozen_fixture(command, index as u32 + 1))
            .collect();
        mission.tasks[0].contract.checks = recipes
            .iter()
            .map(|recipe| recipe.reference().unwrap())
            .collect();
        mission.tasks[0].check_recipes = recipes;
    }
    mission.authorization.allowed_task_digests = mission
        .tasks
        .iter()
        .map(|task| task.contract.digest().unwrap())
        .collect();
    db.insert_run("run", "repo").unwrap();
    install(&db, &mission, &facts, &scope);
    let catalog = Catalog::open(&temp.path().join("catalog.db")).unwrap();
    catalog
        .configure_capacity_pool(&CapacityPoolConfig {
            resource_id: "launch-slot".into(),
            capacity_units: 1,
            expected_revision: None,
            configured_at_ms: 100,
        })
        .unwrap();
    let intent = RoutingCapacityIntentRequest {
        scope: scope.clone(),
        task_id: TaskId("task0".into()),
        reservation: CapacityReservationRequest {
            reservation_id: "launch-reservation".into(),
            domain_id: scope.domain_id.0.clone(),
            run_id: scope.run_id.0.clone(),
            attempt_id: "attempt-task0".into(),
            owner: CapacityOwner {
                process_id: 41,
                process_start_identity: "launch-fixture-owner".into(),
            },
            resources: vec![CapacityResourceRequest {
                resource_id: "launch-slot".into(),
                units: 1,
            }],
            requested_at_ms: 101,
        },
        event_id: "launch-intent-created".into(),
    };
    db.register_capacity_intent(&catalog, &intent).unwrap();
    let claim = PrivateArtifactClaim {
        scope: scope.clone(),
        task_id: TaskId("task0".into()),
        attempt_id: AttemptId("attempt-task0".into()),
        reservation_id: intent.reservation.reservation_id.clone(),
        kind: PrivateArtifactKind::InputManifest,
        artifact_id: "launch-input".into(),
        event_id: "launch-input-created".into(),
    };
    let input = db.put_private_artifact(&claim, b"retained-input").unwrap();
    db.mark_capacity_reserve_may_have_started("launch-reservation", "launch-reserve-may-start")
        .unwrap();
    catalog.reserve_capacity(&intent.reservation).unwrap();
    let mut admission = request(&db, &scope, &facts, "task0");
    admission.capacity_reservation = intent.reservation.reservation_id.clone();
    admission.input_manifest = input;
    let admitted = db.admit_routing_attempt(&admission).unwrap();
    advance(&db, &scope, &facts, admitted, AttemptState::Preparing);
    catalog
        .bind_capacity_reservation(&CapacityBindRequest {
            reservation_id: intent.reservation.reservation_id.clone(),
            attempt_id: intent.reservation.attempt_id.clone(),
            launch_token: "fixture-launch-token".into(),
            bound_at_ms: 102,
        })
        .unwrap();
    (temp, db, catalog, scope, facts, intent)
}

fn prepared_launch_request(scope: &RoutingScope) -> LaunchOwnershipRequest {
    LaunchOwnershipRequest {
        scope: scope.clone(),
        task_id: TaskId("task0".into()),
        attempt_id: AttemptId("attempt-task0".into()),
        reservation_id: "launch-reservation".into(),
        launch_token: "fixture-launch-token".into(),
        event_id: "launch-owner-prepared".into(),
    }
}

fn record_launch_checks(
    db: &PytxoStore,
    scope: &RoutingScope,
    facts: &RoutingFacts,
) -> RoutedAttemptRecord {
    let history = db.routing_history(scope).unwrap().unwrap();
    let preparing = history.attempts[0].clone();
    let checks = if let Some(owner) = db.launch_ownership(&preparing.attempt_id).unwrap() {
        let claim = PrivateArtifactClaim {
            scope: scope.clone(),
            task_id: preparing.task_id.clone(),
            attempt_id: preparing.attempt_id.clone(),
            reservation_id: preparing.capacity_reservation.clone(),
            kind: PrivateArtifactKind::ControllerReceipt,
            artifact_id: format!("launch-checks-{}", preparing.attempt_id.0),
            event_id: format!("launch-checks-observed-{}", preparing.attempt_id.0),
        };
        db.put_controller_receipt(
            &claim,
            &ControllerReceiptEnvelope {
                schema_version: 1,
                scope: scope.clone(),
                task_id: preparing.task_id.clone(),
                attempt_id: preparing.attempt_id.clone(),
                reservation_id: preparing.capacity_reservation.clone(),
                observed_at_ms: facts.now_ms,
                evidence_id: claim.event_id.clone(),
                source: ReceiptSource::TrustedController,
                observation: ControllerObservation::LaunchChecks {
                    mission_digest: canonical_digest(&history.mission, 1).unwrap(),
                    qualification_digest: preparing
                        .selected
                        .observation
                        .qualification
                        .as_ref()
                        .unwrap()
                        .digest()
                        .unwrap(),
                    launch_fingerprint: preparing.launch_fingerprint.clone(),
                    launch_token: owner.request.launch_token,
                },
            },
        )
        .unwrap()
        .digest
    } else {
        digest("launch-owner-checks")
    };
    step(
        db,
        scope,
        facts,
        &preparing,
        AttemptState::Launching,
        RoutingReceipts {
            launch_checks: Some(checks),
            ..Default::default()
        },
    )
    .unwrap()
}

#[test]
fn owned_launch_rejects_unretained_launch_check_digest() {
    let (_, db, catalog, scope, facts, _) = launch_ownership_fixture();
    db.prepare_launch_ownership(&catalog, &prepared_launch_request(&scope))
        .unwrap();
    let preparing = db.routing_history(&scope).unwrap().unwrap().attempts[0].clone();
    assert!(step(
        &db,
        &scope,
        &facts,
        &preparing,
        AttemptState::Launching,
        RoutingReceipts {
            launch_checks: Some(digest("unretained-authority-claim")),
            ..Default::default()
        },
    )
    .is_err());
}

#[test]
fn native_create_rechecks_retained_launch_check_bytes() {
    let (temp, db, catalog, scope, facts, _) = launch_ownership_fixture();
    db.prepare_launch_ownership(&catalog, &prepared_launch_request(&scope))
        .unwrap();
    let launching = record_launch_checks(&db, &scope, &facts);
    let digest = launching.receipts.launch_checks.unwrap();
    rusqlite::Connection::open(temp.path().join("store.db"))
        .unwrap()
        .execute(
            "UPDATE routing_private_artifacts SET bytes=?1 WHERE digest=?2",
            rusqlite::params![b"tampered".as_slice(), digest.0],
        )
        .unwrap();
    assert!(db
        .authorize_launch_create(&LaunchCreateRequest {
            scope,
            attempt_id: launching.attempt_id.clone(),
            event_id: "create-after-receipt-tamper".into(),
            job_name: "fixture-job".into(),
            launch_nonce: "fixture-nonce".into(),
        })
        .is_err());
    assert_eq!(
        db.launch_ownership(&launching.attempt_id)
            .unwrap()
            .unwrap()
            .phase,
        LaunchOwnershipPhase::Prepared
    );
}

#[test]
fn launch_check_receipt_cannot_substitute_mission_qualification_or_token() {
    let (_, db, catalog, scope, facts, _) = launch_ownership_fixture();
    let owner = db
        .prepare_launch_ownership(&catalog, &prepared_launch_request(&scope))
        .unwrap();
    let history = db.routing_history(&scope).unwrap().unwrap();
    let attempt = &history.attempts[0];
    for changed in ["mission", "qualification", "token"] {
        let claim = PrivateArtifactClaim {
            scope: scope.clone(),
            task_id: attempt.task_id.clone(),
            attempt_id: attempt.attempt_id.clone(),
            reservation_id: attempt.capacity_reservation.clone(),
            kind: PrivateArtifactKind::ControllerReceipt,
            artifact_id: format!("changed-launch-check-{changed}"),
            event_id: format!("changed-launch-check-event-{changed}"),
        };
        let mut mission_digest = canonical_digest(&history.mission, 1).unwrap();
        let mut qualification_digest = attempt
            .selected
            .observation
            .qualification
            .as_ref()
            .unwrap()
            .digest()
            .unwrap();
        let mut launch_token = owner.request.launch_token.clone();
        match changed {
            "mission" => mission_digest = digest("wrong-mission"),
            "qualification" => qualification_digest = digest("wrong-qualification"),
            "token" => launch_token = "wrong-launch-token".into(),
            _ => unreachable!(),
        }
        assert!(db
            .put_controller_receipt(
                &claim,
                &ControllerReceiptEnvelope {
                    schema_version: 1,
                    scope: scope.clone(),
                    task_id: attempt.task_id.clone(),
                    attempt_id: attempt.attempt_id.clone(),
                    reservation_id: attempt.capacity_reservation.clone(),
                    observed_at_ms: facts.now_ms,
                    evidence_id: claim.event_id.clone(),
                    source: ReceiptSource::TrustedController,
                    observation: ControllerObservation::LaunchChecks {
                        mission_digest,
                        qualification_digest,
                        launch_fingerprint: attempt.launch_fingerprint.clone(),
                        launch_token,
                    },
                },
            )
            .is_err());
    }
    assert_eq!(
        db.routing_history(&scope).unwrap().unwrap().attempts[0].state,
        AttemptState::Preparing
    );
}

#[test]
fn launch_owner_cannot_be_prepared_after_process_state() {
    let (_, db, catalog, scope, facts, _) = launch_ownership_fixture();
    let launching = record_launch_checks(&db, &scope, &facts);
    let running = step(
        &db,
        &scope,
        &facts,
        &launching,
        AttemptState::Running,
        RoutingReceipts {
            process_identity: Some(digest("claimed-process-without-owner")),
            ..Default::default()
        },
    )
    .unwrap();
    assert_eq!(running.state, AttemptState::Running);
    assert!(db
        .prepare_launch_ownership(&catalog, &prepared_launch_request(&scope))
        .is_err());
    assert!(db.launch_ownership(&running.attempt_id).unwrap().is_none());
}

#[test]
fn fresh_launch_create_cannot_follow_a_process_state() {
    let (_, db, catalog, scope, facts, _) = launch_ownership_fixture();
    db.prepare_launch_ownership(&catalog, &prepared_launch_request(&scope))
        .unwrap();
    let launching = record_launch_checks(&db, &scope, &facts);
    step(
        &db,
        &scope,
        &facts,
        &launching,
        AttemptState::Running,
        RoutingReceipts {
            process_identity: Some(digest("claimed-process-after-owner-prepared")),
            ..Default::default()
        },
    )
    .unwrap();
    let create = LaunchCreateRequest {
        scope: scope.clone(),
        attempt_id: AttemptId("attempt-task0".into()),
        event_id: "late-create-after-process-state".into(),
        job_name: "late-owned-job".into(),
        launch_nonce: "late-native-nonce".into(),
    };
    assert!(db.authorize_launch_create(&create).is_err());
    assert_eq!(
        db.launch_ownership(&create.attempt_id)
            .unwrap()
            .unwrap()
            .phase,
        LaunchOwnershipPhase::Prepared
    );
}

#[test]
fn launch_create_is_one_use_replay_never_grants_second_spawn() {
    let (temp, db, catalog, scope, facts, _) = launch_ownership_fixture();
    let prepared = db
        .prepare_launch_ownership(&catalog, &prepared_launch_request(&scope))
        .unwrap();
    assert_eq!(prepared.phase, LaunchOwnershipPhase::Prepared);
    assert_eq!(
        db.prepare_launch_ownership(&catalog, &prepared_launch_request(&scope))
            .unwrap(),
        prepared
    );
    record_launch_checks(&db, &scope, &facts);
    let create = LaunchCreateRequest {
        scope: scope.clone(),
        attempt_id: AttemptId("attempt-task0".into()),
        event_id: "launch-create-once".into(),
        job_name: "pytxo-owned-fixture".into(),
        launch_nonce: "nonce-once".into(),
    };
    assert!(matches!(
        db.authorize_launch_create(&create).unwrap(),
        LaunchCreateOutcome::Fresh(_)
    ));
    let failed_terminal = step(
        &db,
        &scope,
        &facts,
        &db.routing_history(&scope).unwrap().unwrap().attempts[0],
        AttemptState::FailedNoLaunch,
        RoutingReceipts {
            no_worker_created: Some(digest("claimed-no-worker")),
            ..Default::default()
        },
    );
    assert!(
        failed_terminal.is_err(),
        "unsettled native create retains domain ownership"
    );
    drop(db);
    let reopened = PytxoStore::open(&temp.path().join("store.db")).unwrap();
    assert_eq!(
        reopened
            .launch_ownership(&create.attempt_id)
            .unwrap()
            .unwrap()
            .phase,
        LaunchOwnershipPhase::CreateMayHaveStarted
    );
    assert!(matches!(
        reopened.authorize_launch_create(&create).unwrap(),
        LaunchCreateOutcome::Replay(_)
    ));
    let mut changed = create.clone();
    changed.launch_nonce = "different-nonce".into();
    assert!(reopened.authorize_launch_create(&changed).is_err());
    assert_eq!(
        reopened.unresolved_launch_ownership_scopes().unwrap(),
        vec![scope]
    );
}

#[test]
fn launch_create_rechecks_cancellation_and_corrupt_projection() {
    let (temp, db, catalog, scope, facts, _) = launch_ownership_fixture();
    db.prepare_launch_ownership(&catalog, &prepared_launch_request(&scope))
        .unwrap();
    record_launch_checks(&db, &scope, &facts);
    db.cancel_routing_mission(&scope, "cancel-before-create", 0, 120)
        .unwrap();
    let create = LaunchCreateRequest {
        scope: scope.clone(),
        attempt_id: AttemptId("attempt-task0".into()),
        event_id: "create-after-stop".into(),
        job_name: "job-after-stop".into(),
        launch_nonce: "nonce-after-stop".into(),
    };
    assert!(db.authorize_launch_create(&create).is_err());
    assert_eq!(
        db.launch_ownership(&create.attempt_id)
            .unwrap()
            .unwrap()
            .phase,
        LaunchOwnershipPhase::Prepared
    );
    let conn = rusqlite::Connection::open(temp.path().join("store.db")).unwrap();
    conn.execute(
        "UPDATE attempt_launch_ownership SET phase='settled' WHERE attempt_id=?1",
        [&create.attempt_id.0],
    )
    .unwrap_err();
    conn.execute(
        "UPDATE attempt_launch_ownership SET revision=revision+1 WHERE attempt_id=?1",
        [&create.attempt_id.0],
    )
    .unwrap();
    assert!(db.launch_ownership(&create.attempt_id).is_err());
    assert!(db.unresolved_launch_ownership_scopes().is_err());
}

#[test]
fn claim_scan_fails_closed_if_unsettled_launch_owner_loses_attempt_hold() {
    let (temp, db, catalog, scope, _, _) = launch_ownership_fixture();
    db.prepare_launch_ownership(&catalog, &prepared_launch_request(&scope))
        .unwrap();
    let mut attempt = db.routing_history(&scope).unwrap().unwrap().attempts[0].clone();
    attempt.state = AttemptState::FailedNoLaunch;
    attempt.ownership_released = true;
    let conn = rusqlite::Connection::open(temp.path().join("store.db")).unwrap();
    conn.execute(
        "UPDATE routing_attempts SET record_json=?1 WHERE attempt_id=?2",
        rusqlite::params![
            serde_json::to_string(&attempt).unwrap(),
            attempt.attempt_id.0
        ],
    )
    .unwrap();
    assert!(db.unresolved_routing_attempts().is_err());
    assert!(db.unresolved_launch_ownership_scopes().is_err());
}

#[test]
fn cancelled_prepared_launch_closes_only_with_retained_admitted_no_launch_receipt() {
    let (_, db, catalog, scope, facts, _) = launch_ownership_fixture();
    db.prepare_launch_ownership(&catalog, &prepared_launch_request(&scope))
        .unwrap();
    db.cancel_routing_mission(&scope, "cancel-prepared-launch", 0, 120)
        .unwrap();
    let claim = PrivateArtifactClaim {
        scope: scope.clone(),
        task_id: TaskId("task0".into()),
        attempt_id: AttemptId("attempt-task0".into()),
        reservation_id: "launch-reservation".into(),
        kind: PrivateArtifactKind::ControllerReceipt,
        artifact_id: "admitted-no-launch-receipt".into(),
        event_id: "admitted-no-launch-observation".into(),
    };
    let receipt = ControllerReceiptEnvelope {
        schema_version: 1,
        scope: scope.clone(),
        task_id: claim.task_id.clone(),
        attempt_id: claim.attempt_id.clone(),
        reservation_id: claim.reservation_id.clone(),
        observed_at_ms: 121,
        evidence_id: claim.event_id.clone(),
        source: ReceiptSource::TrustedController,
        observation: ControllerObservation::AdmittedNoLaunch {
            cancellation_event_id: "cancel-prepared-launch".into(),
        },
    };
    let mut wrong = receipt.clone();
    wrong.evidence_id = "substituted-event".into();
    assert!(db.put_controller_receipt(&claim, &wrong).is_err());
    let reference = db.put_controller_receipt(&claim, &receipt).unwrap();
    let settlement = LaunchSettlement {
        scope: scope.clone(),
        attempt_id: claim.attempt_id.clone(),
        event_id: "close-prepared-without-spawn".into(),
        receipt_claim: claim.clone(),
        receipt_ref: reference.clone(),
    };
    let closed = db
        .close_prepared_launch_without_worker(&settlement)
        .unwrap();
    assert_eq!(closed.phase, LaunchOwnershipPhase::ClosedNoLaunch);
    assert_eq!(
        db.close_prepared_launch_without_worker(&settlement)
            .unwrap(),
        closed
    );
    assert!(db.unresolved_launch_ownership_scopes().unwrap().is_empty());
    let failed = step(
        &db,
        &scope,
        &facts,
        &db.routing_history(&scope).unwrap().unwrap().attempts[0],
        AttemptState::FailedNoLaunch,
        RoutingReceipts {
            no_worker_created: Some(reference.digest),
            ..Default::default()
        },
    )
    .unwrap();
    assert!(failed.ownership_released);
}

#[test]
fn cancelled_launching_attempt_can_close_prepared_owner_but_never_claim_process() {
    let (temp, db, catalog, scope, facts, _) = launch_ownership_fixture();
    db.prepare_launch_ownership(&catalog, &prepared_launch_request(&scope))
        .unwrap();
    let launching = record_launch_checks(&db, &scope, &facts);
    db.cancel_routing_mission(&scope, "cancel-before-native-create", 0, 120)
        .unwrap();
    let claim = PrivateArtifactClaim {
        scope: scope.clone(),
        task_id: TaskId("task0".into()),
        attempt_id: launching.attempt_id.clone(),
        reservation_id: "launch-reservation".into(),
        kind: PrivateArtifactKind::ControllerReceipt,
        artifact_id: "launching-no-create-receipt".into(),
        event_id: "launching-no-create-observation".into(),
    };
    let receipt = ControllerReceiptEnvelope {
        schema_version: 1,
        scope: scope.clone(),
        task_id: claim.task_id.clone(),
        attempt_id: claim.attempt_id.clone(),
        reservation_id: claim.reservation_id.clone(),
        observed_at_ms: 121,
        evidence_id: claim.event_id.clone(),
        source: ReceiptSource::TrustedController,
        observation: ControllerObservation::AdmittedNoLaunch {
            cancellation_event_id: "cancel-before-native-create".into(),
        },
    };
    let reference = db.put_controller_receipt(&claim, &receipt).unwrap();
    let settlement = LaunchSettlement {
        scope: scope.clone(),
        attempt_id: launching.attempt_id.clone(),
        event_id: "close-launching-before-native-create".into(),
        receipt_claim: claim,
        receipt_ref: reference.clone(),
    };
    let closed = db
        .close_prepared_launch_without_worker(&settlement)
        .unwrap();
    assert_eq!(closed.phase, LaunchOwnershipPhase::ClosedNoLaunch);
    assert_eq!(
        db.close_prepared_launch_without_worker(&settlement)
            .unwrap(),
        closed
    );
    assert!(step(
        &db,
        &scope,
        &facts,
        &launching,
        AttemptState::Running,
        RoutingReceipts {
            process_identity: Some(digest("fabricated-process-after-closure")),
            ..Default::default()
        },
    )
    .is_err());
    assert!(step(
        &db,
        &scope,
        &facts,
        &launching,
        AttemptState::FailedNoLaunch,
        RoutingReceipts {
            no_worker_created: Some(digest("unrelated-no-worker-receipt")),
            ..Default::default()
        },
    )
    .is_err());
    assert_eq!(
        db.routing_history(&scope).unwrap().unwrap().attempts[0].state,
        AttemptState::Launching
    );
    let failed = step(
        &db,
        &scope,
        &facts,
        &launching,
        AttemptState::FailedNoLaunch,
        RoutingReceipts {
            no_worker_created: Some(reference.digest),
            ..Default::default()
        },
    )
    .unwrap();
    assert!(failed.ownership_released);
    let mut forged = failed;
    forged.state = AttemptState::Running;
    forged.ownership_released = false;
    forged.receipts.no_worker_created = None;
    forged.receipts.process_identity = Some(digest("forged-process-after-no-create"));
    let conn = rusqlite::Connection::open(temp.path().join("store.db")).unwrap();
    conn.execute(
        "UPDATE routing_attempts SET record_json=?1 WHERE attempt_id=?2",
        rusqlite::params![serde_json::to_string(&forged).unwrap(), forged.attempt_id.0],
    )
    .unwrap();
    assert!(db.unresolved_launch_ownership_scopes().is_err());
}

#[test]
fn recovered_pre_create_attempt_closes_only_as_reconciled_no_launch() {
    let (_, db, catalog, scope, facts, _) = launch_ownership_fixture();
    db.prepare_launch_ownership(&catalog, &prepared_launch_request(&scope))
        .unwrap();
    let launching = record_launch_checks(&db, &scope, &facts);
    let recovering = step(
        &db,
        &scope,
        &facts,
        &launching,
        AttemptState::RecoveryRequired,
        RoutingReceipts::default(),
    )
    .unwrap();
    db.cancel_routing_mission(&scope, "cancel-recovery-before-create", 0, 120)
        .unwrap();
    let claim = PrivateArtifactClaim {
        scope: scope.clone(),
        task_id: TaskId("task0".into()),
        attempt_id: recovering.attempt_id.clone(),
        reservation_id: "launch-reservation".into(),
        kind: PrivateArtifactKind::ControllerReceipt,
        artifact_id: "recovered-pre-create-receipt".into(),
        event_id: "recovered-pre-create-observation".into(),
    };
    let reference = db
        .put_controller_receipt(
            &claim,
            &ControllerReceiptEnvelope {
                schema_version: 1,
                scope: scope.clone(),
                task_id: claim.task_id.clone(),
                attempt_id: claim.attempt_id.clone(),
                reservation_id: claim.reservation_id.clone(),
                observed_at_ms: 121,
                evidence_id: claim.event_id.clone(),
                source: ReceiptSource::TrustedController,
                observation: ControllerObservation::AdmittedNoLaunch {
                    cancellation_event_id: "cancel-recovery-before-create".into(),
                },
            },
        )
        .unwrap();
    let settlement = LaunchSettlement {
        scope: scope.clone(),
        attempt_id: recovering.attempt_id.clone(),
        event_id: "close-recovered-before-create".into(),
        receipt_claim: claim,
        receipt_ref: reference.clone(),
    };
    assert_eq!(
        db.close_prepared_launch_without_worker(&settlement)
            .unwrap()
            .phase,
        LaunchOwnershipPhase::ClosedNoLaunch
    );
    assert!(step(
        &db,
        &scope,
        &facts,
        &recovering,
        AttemptState::Cancelled,
        RoutingReceipts {
            no_worker_created: Some(reference.digest.clone()),
            reconciliation: Some(digest("reconciled-pre-create")),
            quiescence: Some(digest("unneeded-quiescence")),
            ..Default::default()
        },
    )
    .is_err());
    assert!(step(
        &db,
        &scope,
        &facts,
        &recovering,
        AttemptState::FailedNoLaunch,
        RoutingReceipts {
            no_worker_created: Some(digest("wrong-recovery-receipt")),
            reconciliation: Some(digest("reconciled-pre-create")),
            ..Default::default()
        },
    )
    .is_err());
    let failed = step(
        &db,
        &scope,
        &facts,
        &recovering,
        AttemptState::FailedNoLaunch,
        RoutingReceipts {
            no_worker_created: Some(reference.digest),
            reconciliation: Some(digest("reconciled-pre-create")),
            ..Default::default()
        },
    )
    .unwrap();
    assert!(failed.ownership_released);
}

#[test]
fn admitted_no_launch_releases_bound_host_slot_only_after_exact_retained_proof() {
    let (temp, db, catalog, scope, facts, intent) = launch_ownership_fixture();
    let reservation_id = intent.reservation.reservation_id.as_str();
    db.prepare_launch_ownership(&catalog, &prepared_launch_request(&scope))
        .unwrap();
    let launching = record_launch_checks(&db, &scope, &facts);
    db.cancel_routing_mission(&scope, "cancel-before-capacity-release", 0, 120)
        .unwrap();
    let claim = PrivateArtifactClaim {
        scope: scope.clone(),
        task_id: TaskId("task0".into()),
        attempt_id: launching.attempt_id.clone(),
        reservation_id: reservation_id.into(),
        kind: PrivateArtifactKind::ControllerReceipt,
        artifact_id: "capacity-admitted-no-create".into(),
        event_id: "capacity-admitted-no-create-observation".into(),
    };
    let reference = db
        .put_controller_receipt(
            &claim,
            &ControllerReceiptEnvelope {
                schema_version: 1,
                scope: scope.clone(),
                task_id: claim.task_id.clone(),
                attempt_id: claim.attempt_id.clone(),
                reservation_id: claim.reservation_id.clone(),
                observed_at_ms: 121,
                evidence_id: claim.event_id.clone(),
                source: ReceiptSource::TrustedController,
                observation: ControllerObservation::AdmittedNoLaunch {
                    cancellation_event_id: "cancel-before-capacity-release".into(),
                },
            },
        )
        .unwrap();
    let admitted = AdmittedNoLaunchRelease {
        launch_token: "fixture-launch-token".into(),
        receipt_claim: claim.clone(),
        receipt_ref: reference.clone(),
    };
    let proof = CapacityReleaseEvidence {
        kind: CapacityReleaseEvidenceKind::KnownUnused,
        receipt_id: claim.event_id.clone(),
        evidence_digest: reference.digest.0.clone(),
        observed_at_ms: 121,
    };
    assert!(db
        .bind_admitted_no_launch_release_proof(
            &catalog,
            reservation_id,
            "bind-admitted-release",
            &proof,
            &admitted,
        )
        .is_err());
    db.close_prepared_launch_without_worker(&LaunchSettlement {
        scope: scope.clone(),
        attempt_id: claim.attempt_id.clone(),
        event_id: "close-before-capacity-release".into(),
        receipt_claim: claim,
        receipt_ref: reference.clone(),
    })
    .unwrap();
    assert!(db
        .bind_admitted_no_launch_release_proof(
            &catalog,
            reservation_id,
            "bind-admitted-release",
            &proof,
            &admitted,
        )
        .is_err());
    let terminal = step(
        &db,
        &scope,
        &facts,
        &launching,
        AttemptState::FailedNoLaunch,
        RoutingReceipts {
            no_worker_created: Some(reference.digest),
            ..Default::default()
        },
    )
    .unwrap();
    assert!(terminal.ownership_released);
    let mut wrong = proof.clone();
    wrong.evidence_digest = digest("wrong-capacity-release").0;
    assert!(db
        .bind_admitted_no_launch_release_proof(
            &catalog,
            reservation_id,
            "bind-admitted-release",
            &wrong,
            &admitted,
        )
        .is_err());
    let bound = db
        .bind_admitted_no_launch_release_proof(
            &catalog,
            reservation_id,
            "bind-admitted-release",
            &proof,
            &admitted,
        )
        .unwrap();
    assert_eq!(bound.phase, CapacityIntentPhase::ReleaseProofBound);
    assert_eq!(
        db.bind_admitted_no_launch_release_proof(
            &catalog,
            reservation_id,
            "bind-admitted-release",
            &proof,
            &admitted,
        )
        .unwrap(),
        bound
    );
    let mut wrong_token = admitted.clone();
    wrong_token.launch_token = "other-launch-token".into();
    assert!(db
        .bind_admitted_no_launch_release_proof(
            &catalog,
            reservation_id,
            "bind-admitted-release",
            &proof,
            &wrong_token,
        )
        .is_err());
    assert!(db
        .bind_admitted_no_launch_release_proof(
            &catalog,
            reservation_id,
            "changed-bind-event",
            &proof,
            &admitted,
        )
        .is_err());
    assert!(db
        .bind_capacity_release_proof(&catalog, reservation_id, "bind-admitted-release", &proof)
        .is_err());
    drop(db);
    let reopened = PytxoStore::open(&temp.path().join("store.db")).unwrap();
    assert_eq!(
        reopened.capacity_intent(reservation_id).unwrap().unwrap(),
        bound
    );
    let wrong_catalog = Catalog::open(&temp.path().join("other-capacity-catalog.db")).unwrap();
    assert!(reopened
        .release_admitted_no_launch_capacity(&wrong_catalog, reservation_id)
        .is_err());
    assert!(reopened
        .close_released_capacity_intent(&catalog, reservation_id, "close-too-early")
        .is_err());
    assert_eq!(
        catalog
            .capacity_pool_status("launch-slot")
            .unwrap()
            .unwrap()
            .held_units,
        1
    );
    let released = reopened
        .release_admitted_no_launch_capacity(&catalog, reservation_id)
        .unwrap();
    assert_eq!(released.release_evidence, Some(proof));
    assert_eq!(
        reopened
            .release_admitted_no_launch_capacity(&catalog, reservation_id)
            .unwrap(),
        released
    );
    drop(reopened);
    let after_release_restart = PytxoStore::open(&temp.path().join("store.db")).unwrap();
    let closed = after_release_restart
        .close_released_capacity_intent(&catalog, reservation_id, "close-admitted-release")
        .unwrap();
    assert_eq!(closed.phase, CapacityIntentPhase::Closed);
    assert_eq!(
        after_release_restart
            .close_released_capacity_intent(&catalog, reservation_id, "close-admitted-release")
            .unwrap(),
        closed
    );
    assert!(after_release_restart
        .unresolved_capacity_intent_scopes()
        .unwrap()
        .is_empty());
    assert_eq!(
        catalog
            .capacity_pool_status("launch-slot")
            .unwrap()
            .unwrap()
            .held_units,
        0
    );
    catalog
        .reserve_capacity(&CapacityReservationRequest {
            reservation_id: "after-admitted-no-launch".into(),
            domain_id: "second-domain".into(),
            run_id: "second-run".into(),
            attempt_id: "second-attempt".into(),
            owner: CapacityOwner {
                process_id: 42,
                process_start_identity: "second-owner".into(),
            },
            resources: vec![CapacityResourceRequest {
                resource_id: "launch-slot".into(),
                units: 1,
            }],
            requested_at_ms: 122,
        })
        .unwrap();
}

#[test]
fn prepared_no_launch_closure_refuses_attempt_that_claims_a_process() {
    let (_, db, catalog, scope, facts, _) = launch_ownership_fixture();
    db.prepare_launch_ownership(&catalog, &prepared_launch_request(&scope))
        .unwrap();
    let launching = record_launch_checks(&db, &scope, &facts);
    let running = step(
        &db,
        &scope,
        &facts,
        &launching,
        AttemptState::Running,
        RoutingReceipts {
            process_identity: Some(digest("process-claim-before-no-launch")),
            ..Default::default()
        },
    )
    .unwrap();
    db.cancel_routing_mission(&scope, "cancel-process-claim", 0, 120)
        .unwrap();
    let claim = PrivateArtifactClaim {
        scope: scope.clone(),
        task_id: TaskId("task0".into()),
        attempt_id: running.attempt_id.clone(),
        reservation_id: "launch-reservation".into(),
        kind: PrivateArtifactKind::ControllerReceipt,
        artifact_id: "false-no-launch-receipt".into(),
        event_id: "false-no-launch-observation".into(),
    };
    let receipt = ControllerReceiptEnvelope {
        schema_version: 1,
        scope: scope.clone(),
        task_id: claim.task_id.clone(),
        attempt_id: claim.attempt_id.clone(),
        reservation_id: claim.reservation_id.clone(),
        observed_at_ms: 121,
        evidence_id: claim.event_id.clone(),
        source: ReceiptSource::TrustedController,
        observation: ControllerObservation::AdmittedNoLaunch {
            cancellation_event_id: "cancel-process-claim".into(),
        },
    };
    let reference = db.put_controller_receipt(&claim, &receipt).unwrap();
    let settlement = LaunchSettlement {
        scope: scope.clone(),
        attempt_id: running.attempt_id.clone(),
        event_id: "close-false-no-launch".into(),
        receipt_claim: claim,
        receipt_ref: reference,
    };
    assert!(db
        .close_prepared_launch_without_worker(&settlement)
        .is_err());
    assert_eq!(
        db.launch_ownership(&running.attempt_id)
            .unwrap()
            .unwrap()
            .phase,
        LaunchOwnershipPhase::Prepared
    );
    assert!(step(
        &db,
        &scope,
        &facts,
        &running,
        AttemptState::Cancelled,
        RoutingReceipts {
            quiescence: Some(digest("unproven-quiescence")),
            ..Default::default()
        },
    )
    .is_err());
    assert_eq!(
        db.unresolved_launch_ownership_scopes().unwrap(),
        vec![scope]
    );
}

#[test]
fn registered_launch_settles_only_with_exact_retained_job_zero_receipt() {
    let (temp, db, catalog, scope, facts, _) = launch_ownership_fixture();
    db.prepare_launch_ownership(&catalog, &prepared_launch_request(&scope))
        .unwrap();
    record_launch_checks(&db, &scope, &facts);
    let create = LaunchCreateRequest {
        scope: scope.clone(),
        attempt_id: AttemptId("attempt-task0".into()),
        event_id: "native-create-event".into(),
        job_name: "owned-job-1".into(),
        launch_nonce: "native-nonce-1".into(),
    };
    assert!(matches!(
        db.authorize_launch_create(&create).unwrap(),
        LaunchCreateOutcome::Fresh(_)
    ));
    let register = LaunchProcessRegistration {
        scope: scope.clone(),
        attempt_id: create.attempt_id.clone(),
        event_id: "registered-native-process".into(),
        observed_job_name: create.job_name.clone(),
        pid: 5151,
        start_identity: "process-start-5151".into(),
    };
    let mut wrong_register = register.clone();
    wrong_register.observed_job_name = "wrong-job".into();
    assert!(db.register_launch_process(&wrong_register).is_err());
    assert_eq!(
        db.register_launch_process(&register).unwrap().phase,
        LaunchOwnershipPhase::Registered
    );
    let launching = db.routing_history(&scope).unwrap().unwrap().attempts[0].clone();
    let running = step(
        &db,
        &scope,
        &facts,
        &launching,
        AttemptState::Running,
        RoutingReceipts {
            process_identity: Some(digest("registered-process")),
            ..Default::default()
        },
    )
    .unwrap();
    assert!(step(
        &db,
        &scope,
        &facts,
        &running,
        AttemptState::Sealing,
        RoutingReceipts {
            quiescence: Some(digest("unretained-job-zero")),
            ..Default::default()
        },
    )
    .is_err());
    assert_eq!(
        db.register_launch_process(&register).unwrap().phase,
        LaunchOwnershipPhase::Registered
    );
    let claim = PrivateArtifactClaim {
        scope: scope.clone(),
        task_id: TaskId("task0".into()),
        attempt_id: create.attempt_id.clone(),
        reservation_id: "launch-reservation".into(),
        kind: PrivateArtifactKind::ControllerReceipt,
        artifact_id: "job-zero-receipt".into(),
        event_id: "job-zero-observation".into(),
    };
    let mut receipt = ControllerReceiptEnvelope {
        schema_version: 1,
        scope: scope.clone(),
        task_id: claim.task_id.clone(),
        attempt_id: claim.attempt_id.clone(),
        reservation_id: claim.reservation_id.clone(),
        observed_at_ms: 130,
        evidence_id: claim.event_id.clone(),
        source: ReceiptSource::OwnedJobObservation,
        observation: ControllerObservation::NativeJobZero {
            job_name: "wrong-job".into(),
            launch_nonce: create.launch_nonce.clone(),
            active_processes: 0,
        },
    };
    let wrong_reference = db.put_controller_receipt(&claim, &receipt).unwrap();
    let settlement = LaunchSettlement {
        scope: scope.clone(),
        attempt_id: create.attempt_id.clone(),
        event_id: "settle-native-job".into(),
        receipt_claim: claim.clone(),
        receipt_ref: wrong_reference,
    };
    assert!(db.settle_launch_ownership(&settlement).is_err());
    receipt.observation = ControllerObservation::NativeJobZero {
        job_name: create.job_name.clone(),
        launch_nonce: create.launch_nonce.clone(),
        active_processes: 0,
    };
    let mut valid_claim = claim.clone();
    valid_claim.artifact_id = "job-zero-correct".into();
    valid_claim.event_id = "job-zero-correct-event".into();
    receipt.evidence_id = valid_claim.event_id.clone();
    let reference = db.put_controller_receipt(&valid_claim, &receipt).unwrap();
    let valid = LaunchSettlement {
        receipt_claim: valid_claim,
        receipt_ref: reference,
        ..settlement
    };
    assert_eq!(
        db.settle_launch_ownership(&valid).unwrap().phase,
        LaunchOwnershipPhase::Settled
    );
    assert!(step(
        &db,
        &scope,
        &facts,
        &running,
        AttemptState::Sealing,
        RoutingReceipts {
            quiescence: Some(digest("wrong-job-zero")),
            ..Default::default()
        },
    )
    .is_err());
    let sealing = step(
        &db,
        &scope,
        &facts,
        &running,
        AttemptState::Sealing,
        RoutingReceipts {
            quiescence: Some(valid.receipt_ref.digest.clone()),
            ..Default::default()
        },
    )
    .unwrap();
    let verifying = step(
        &db,
        &scope,
        &facts,
        &sealing,
        AttemptState::Verifying,
        RoutingReceipts {
            sealed_output: Some(digest("controller-claimed-sealed-output")),
            ..Default::default()
        },
    )
    .unwrap();
    assert!(step(
        &db,
        &scope,
        &facts,
        &verifying,
        AttemptState::Passed,
        RoutingReceipts {
            checks: Some(digest("unowned-checks")),
            ..Default::default()
        },
    )
    .is_err());
    let failed = step(
        &db,
        &scope,
        &facts,
        &verifying,
        AttemptState::Failed,
        RoutingReceipts::default(),
    )
    .unwrap();
    assert!(failed.ownership_released);
    assert!(db.unresolved_launch_ownership_scopes().unwrap().is_empty());
    let conn = rusqlite::Connection::open(temp.path().join("store.db")).unwrap();
    conn.execute(
        "UPDATE routing_private_artifacts SET bytes=X'00' WHERE artifact_id=?1",
        [&valid.receipt_claim.artifact_id],
    )
    .unwrap();
    assert!(db.launch_ownership(&create.attempt_id).is_err());
    assert!(db.unresolved_routing_attempts().is_err());
}

#[test]
fn lost_owned_launch_row_cannot_restore_digest_only_sealing_or_hide_from_stop_scan() {
    let (temp, db, catalog, scope, facts, _) = launch_ownership_fixture();
    db.prepare_launch_ownership(&catalog, &prepared_launch_request(&scope))
        .unwrap();
    let launching = record_launch_checks(&db, &scope, &facts);
    db.authorize_launch_create(&LaunchCreateRequest {
        scope: scope.clone(),
        attempt_id: launching.attempt_id.clone(),
        event_id: "lost-owner-create".into(),
        job_name: "lost-owner-job".into(),
        launch_nonce: "lost-owner-nonce".into(),
    })
    .unwrap();
    db.register_launch_process(&LaunchProcessRegistration {
        scope: scope.clone(),
        attempt_id: launching.attempt_id.clone(),
        event_id: "lost-owner-register".into(),
        observed_job_name: "lost-owner-job".into(),
        pid: 6262,
        start_identity: "lost-owner-process-start".into(),
    })
    .unwrap();
    let running = step(
        &db,
        &scope,
        &facts,
        &launching,
        AttemptState::Running,
        RoutingReceipts {
            process_identity: Some(digest("lost-owner-process")),
            ..Default::default()
        },
    )
    .unwrap();
    let conn = rusqlite::Connection::open(temp.path().join("store.db")).unwrap();
    conn.execute(
        "DELETE FROM attempt_launch_ownership WHERE attempt_id=?1",
        [&running.attempt_id.0],
    )
    .unwrap();
    assert!(step(
        &db,
        &scope,
        &facts,
        &running,
        AttemptState::Sealing,
        RoutingReceipts {
            quiescence: Some(digest("forged-after-owner-loss")),
            ..Default::default()
        },
    )
    .is_err());
    assert!(db.unresolved_routing_attempts().is_err());
}

fn checker_verifying_fixture() -> (
    tempfile::TempDir,
    PytxoStore,
    Catalog,
    RoutingScope,
    RoutingFacts,
    BlobRef,
) {
    checker_verifying_fixture_with_dependent(false)
}

fn checker_verifying_fixture_with_dependent(
    dependent: bool,
) -> (
    tempfile::TempDir,
    PytxoStore,
    Catalog,
    RoutingScope,
    RoutingFacts,
    BlobRef,
) {
    let (temp, db, catalog, scope, facts, _) =
        launch_ownership_fixture_with_checks(&["echo first", "echo second"], dependent);
    db.prepare_launch_ownership(&catalog, &prepared_launch_request(&scope))
        .unwrap();
    record_launch_checks(&db, &scope, &facts);
    let create = LaunchCreateRequest {
        scope: scope.clone(),
        attempt_id: AttemptId("attempt-task0".into()),
        event_id: "checker-fixture-worker-create".into(),
        job_name: "checker-fixture-worker-job".into(),
        launch_nonce: "checker-fixture-worker-nonce".into(),
    };
    assert!(matches!(
        db.authorize_launch_create(&create).unwrap(),
        LaunchCreateOutcome::Fresh(_)
    ));
    db.register_launch_process(&LaunchProcessRegistration {
        scope: scope.clone(),
        attempt_id: create.attempt_id.clone(),
        event_id: "checker-fixture-worker-register".into(),
        observed_job_name: create.job_name.clone(),
        pid: 6161,
        start_identity: "checker-fixture-worker-start".into(),
    })
    .unwrap();
    let launching = db.routing_history(&scope).unwrap().unwrap().attempts[0].clone();
    let running = step(
        &db,
        &scope,
        &facts,
        &launching,
        AttemptState::Running,
        RoutingReceipts {
            process_identity: Some(digest("checker-fixture-worker-process")),
            ..Default::default()
        },
    )
    .unwrap();
    let claim = PrivateArtifactClaim {
        scope: scope.clone(),
        task_id: TaskId("task0".into()),
        attempt_id: create.attempt_id.clone(),
        reservation_id: "launch-reservation".into(),
        kind: PrivateArtifactKind::ControllerReceipt,
        artifact_id: "checker-fixture-job-zero".into(),
        event_id: "checker-fixture-job-zero-event".into(),
    };
    let receipt = ControllerReceiptEnvelope {
        schema_version: 1,
        scope: scope.clone(),
        task_id: claim.task_id.clone(),
        attempt_id: claim.attempt_id.clone(),
        reservation_id: claim.reservation_id.clone(),
        observed_at_ms: 130,
        evidence_id: claim.event_id.clone(),
        source: ReceiptSource::OwnedJobObservation,
        observation: ControllerObservation::NativeJobZero {
            job_name: create.job_name,
            launch_nonce: create.launch_nonce,
            active_processes: 0,
        },
    };
    let zero = db.put_controller_receipt(&claim, &receipt).unwrap();
    db.settle_launch_ownership(&LaunchSettlement {
        scope: scope.clone(),
        attempt_id: create.attempt_id,
        event_id: "checker-fixture-worker-settle".into(),
        receipt_claim: claim,
        receipt_ref: zero.clone(),
    })
    .unwrap();
    let sealing = step(
        &db,
        &scope,
        &facts,
        &running,
        AttemptState::Sealing,
        RoutingReceipts {
            quiescence: Some(zero.digest),
            ..Default::default()
        },
    )
    .unwrap();
    let output_claim = PrivateArtifactClaim {
        scope: scope.clone(),
        task_id: TaskId("task0".into()),
        attempt_id: AttemptId("attempt-task0".into()),
        reservation_id: "launch-reservation".into(),
        kind: PrivateArtifactKind::ScopedOutput,
        artifact_id: "checker-fixture-sealed-output".into(),
        event_id: "checker-fixture-sealed-output-event".into(),
    };
    let sealed = db
        .put_private_artifact(&output_claim, b"sealed fixture output")
        .unwrap();
    step(
        &db,
        &scope,
        &facts,
        &sealing,
        AttemptState::Verifying,
        RoutingReceipts {
            sealed_output: Some(sealed.digest.clone()),
            ..Default::default()
        },
    )
    .unwrap();
    (temp, db, catalog, scope, facts, sealed)
}

fn checker_request(
    db: &PytxoStore,
    scope: &RoutingScope,
    sealed: &BlobRef,
    ordinal: u32,
) -> CheckerOwnershipRequest {
    let history = db.routing_history(scope).unwrap().unwrap();
    let recipe = &history.tasks[0].registration.check_recipes[(ordinal - 1) as usize];
    CheckerOwnershipRequest {
        scope: scope.clone(),
        task_id: TaskId("task0".into()),
        attempt_id: AttemptId("attempt-task0".into()),
        reservation_id: "launch-reservation".into(),
        launch_token: "fixture-launch-token".into(),
        check_id: recipe.id.clone(),
        ordinal,
        recipe_digest: recipe.reference().unwrap().recipe_digest,
        sealed_view: sealed.clone(),
        prepare_event_id: format!("checker-prepare-{ordinal}"),
        now_ms: 150,
    }
}

#[test]
fn stop_snapshot_tracks_exact_worker_owner_through_restart_and_rejects_corruption() {
    let (temp, db, catalog, scope, facts, _) = launch_ownership_fixture();
    let attempt_id = AttemptId("attempt-task0".into());
    assert!(db
        .owned_job_stop_snapshot(&scope.domain_id, Some(&scope.run_id))
        .unwrap()
        .is_empty());
    db.prepare_launch_ownership(&catalog, &prepared_launch_request(&scope))
        .unwrap();
    let prepared = db
        .owned_job_stop_snapshot(&scope.domain_id, Some(&scope.run_id))
        .unwrap();
    assert_eq!(prepared.len(), 1);
    assert_eq!(prepared[0].phase, OwnedJobStopPhase::Prepared);
    assert_eq!(prepared[0].kind, OwnedJobStopKind::Worker);
    assert!(prepared[0].pid.is_none());
    assert!(db
        .owned_job_stop_snapshot(&scope.domain_id, Some(&RunId("other-run".into())))
        .unwrap()
        .is_empty());
    record_launch_checks(&db, &scope, &facts);
    let create = LaunchCreateRequest {
        scope: scope.clone(),
        attempt_id: attempt_id.clone(),
        event_id: "stop-snapshot-create".into(),
        job_name: "stop-snapshot-job".into(),
        launch_nonce: "stop-snapshot-nonce".into(),
    };
    assert!(matches!(
        db.authorize_launch_create(&create).unwrap(),
        LaunchCreateOutcome::Fresh(_)
    ));
    let ambiguous = db.owned_job_stop_snapshot(&scope.domain_id, None).unwrap();
    assert_eq!(ambiguous.len(), 1);
    assert_eq!(ambiguous[0].phase, OwnedJobStopPhase::CreateMayHaveStarted);
    assert_eq!(ambiguous[0].job_name.as_deref(), Some("stop-snapshot-job"));
    assert!(ambiguous[0].pid.is_none());
    db.register_launch_process(&LaunchProcessRegistration {
        scope: scope.clone(),
        attempt_id,
        event_id: "stop-snapshot-register".into(),
        observed_job_name: create.job_name,
        pid: 6721,
        start_identity: "stop-snapshot-start".into(),
    })
    .unwrap();
    drop(db);
    let reopened = PytxoStore::open(&temp.path().join("store.db")).unwrap();
    let registered = reopened
        .owned_job_stop_snapshot(&scope.domain_id, Some(&scope.run_id))
        .unwrap();
    assert_eq!(registered.len(), 1);
    assert_eq!(registered[0].phase, OwnedJobStopPhase::Registered);
    assert_eq!(registered[0].pid, Some(6721));
    assert_eq!(
        registered[0].start_identity.as_deref(),
        Some("stop-snapshot-start")
    );
    assert!(!format!("{:?}", registered[0]).contains("stop-snapshot-nonce"));
    let conn = rusqlite::Connection::open(temp.path().join("store.db")).unwrap();
    conn.execute(
        "UPDATE attempt_launch_ownership SET revision=revision+1 WHERE attempt_id='attempt-task0'",
        [],
    )
    .unwrap();
    assert!(reopened
        .owned_job_stop_snapshot(&scope.domain_id, Some(&scope.run_id))
        .is_err());
}

#[test]
fn stop_snapshot_includes_checker_without_unsettling_worker() {
    let (temp, db, catalog, scope, _, sealed) = checker_verifying_fixture();
    let first = checker_request(&db, &scope, &sealed, 1);
    db.prepare_checker_ownership(&catalog, &first).unwrap();
    let prepared = db
        .owned_job_stop_snapshot(&scope.domain_id, Some(&scope.run_id))
        .unwrap();
    assert_eq!(prepared.len(), 1);
    assert_eq!(prepared[0].phase, OwnedJobStopPhase::Prepared);
    assert_eq!(
        prepared[0].kind,
        OwnedJobStopKind::Checker {
            check_id: first.check_id.clone(),
            ordinal: 1,
        }
    );
    let create = CheckerCreateRequest {
        scope: scope.clone(),
        attempt_id: first.attempt_id.clone(),
        ordinal: 1,
        event_id: "stop-checker-create".into(),
        job_name: "stop-checker-job".into(),
        launch_nonce: "stop-checker-nonce".into(),
        now_ms: 150,
    };
    assert!(matches!(
        db.authorize_checker_create(&catalog, &create).unwrap(),
        CheckerCreateOutcome::Fresh(_)
    ));
    assert_eq!(
        db.owned_job_stop_snapshot(&scope.domain_id, None).unwrap()[0].phase,
        OwnedJobStopPhase::CreateMayHaveStarted
    );
    db.register_checker_process(&CheckerProcessRegistration {
        scope: scope.clone(),
        attempt_id: first.attempt_id.clone(),
        ordinal: 1,
        event_id: "stop-checker-register".into(),
        observed_job_name: create.job_name,
        pid: 7001,
        start_identity: "stop-checker-start".into(),
    })
    .unwrap();
    drop(db);
    let reopened = PytxoStore::open(&temp.path().join("store.db")).unwrap();
    let registered = reopened
        .owned_job_stop_snapshot(&scope.domain_id, Some(&scope.run_id))
        .unwrap();
    assert_eq!(registered.len(), 1);
    assert_eq!(registered[0].phase, OwnedJobStopPhase::Registered);
    assert_eq!(registered[0].pid, Some(7001));
    assert!(reopened
        .owned_job_stop_snapshot(&DomainId("other-domain".into()), None)
        .unwrap()
        .is_empty());
}

fn settle_fixture_checker(
    db: &PytxoStore,
    catalog: &Catalog,
    request: &CheckerOwnershipRequest,
    sealed: &BlobRef,
    exit_code: i32,
) -> CheckerOwnershipRecord {
    settle_fixture_checker_with_native(
        db,
        catalog,
        request,
        sealed,
        exit_code,
        CheckerNativeFields::default(),
    )
}

#[derive(Clone, Copy)]
struct CheckerNativeFields {
    payload_exit_code: Option<u32>,
    process_registered: bool,
    barrier_released: bool,
    native_outcome: CheckerNativeOutcome,
    output_truncated: bool,
    error_present: bool,
    view_intact: bool,
}

impl Default for CheckerNativeFields {
    fn default() -> Self {
        Self {
            payload_exit_code: Some(0),
            process_registered: true,
            barrier_released: true,
            native_outcome: CheckerNativeOutcome::Succeeded,
            output_truncated: false,
            error_present: false,
            view_intact: true,
        }
    }
}

fn settle_fixture_checker_with_native(
    db: &PytxoStore,
    catalog: &Catalog,
    request: &CheckerOwnershipRequest,
    sealed: &BlobRef,
    exit_code: i32,
    native: CheckerNativeFields,
) -> CheckerOwnershipRecord {
    let create = CheckerCreateRequest {
        scope: request.scope.clone(),
        attempt_id: request.attempt_id.clone(),
        ordinal: request.ordinal,
        event_id: format!("checker-create-{}", request.ordinal),
        job_name: format!("checker-job-{}", request.ordinal),
        launch_nonce: format!("checker-nonce-{}", request.ordinal),
        now_ms: 150,
    };
    assert!(matches!(
        db.authorize_checker_create(catalog, &create).unwrap(),
        CheckerCreateOutcome::Fresh(_)
    ));
    db.register_checker_process(&CheckerProcessRegistration {
        scope: request.scope.clone(),
        attempt_id: request.attempt_id.clone(),
        ordinal: request.ordinal,
        event_id: format!("checker-register-{}", request.ordinal),
        observed_job_name: create.job_name.clone(),
        pid: 7000 + request.ordinal,
        start_identity: format!("checker-start-{}", request.ordinal),
    })
    .unwrap();
    let claim = PrivateArtifactClaim {
        scope: request.scope.clone(),
        task_id: request.task_id.clone(),
        attempt_id: request.attempt_id.clone(),
        reservation_id: request.reservation_id.clone(),
        kind: PrivateArtifactKind::ControllerReceipt,
        artifact_id: format!("checker-result-{}", request.ordinal),
        event_id: format!("checker-result-event-{}", request.ordinal),
    };
    let result = ControllerReceiptEnvelope {
        schema_version: 1,
        scope: request.scope.clone(),
        task_id: request.task_id.clone(),
        attempt_id: request.attempt_id.clone(),
        reservation_id: request.reservation_id.clone(),
        observed_at_ms: 151,
        evidence_id: claim.event_id.clone(),
        source: ReceiptSource::OwnedJobObservation,
        observation: ControllerObservation::NativeCheckerResult {
            check_id: request.check_id.clone(),
            ordinal: request.ordinal,
            job_name: create.job_name,
            launch_nonce: create.launch_nonce,
            active_processes: 0,
            exit_code,
            payload_exit_code: native.payload_exit_code,
            process_registered: native.process_registered,
            barrier_released: native.barrier_released,
            native_outcome: native.native_outcome,
            stdout_complete: true,
            stderr_complete: true,
            output_truncated: native.output_truncated,
            error_present: native.error_present,
            sealed_view_before: sealed.clone(),
            sealed_view_after: native.view_intact.then(|| sealed.clone()),
        },
    };
    let reference = db.put_controller_receipt(&claim, &result).unwrap();
    db.settle_checker_ownership(&CheckerSettlement {
        scope: request.scope.clone(),
        attempt_id: request.attempt_id.clone(),
        ordinal: request.ordinal,
        event_id: format!("checker-settle-{}", request.ordinal),
        receipt_claim: claim,
        receipt_ref: reference,
    })
    .unwrap()
}

fn fixture_quiescent_context(
    db: &PytxoStore,
    scope: &RoutingScope,
    attempt_id: &AttemptId,
    checker_claim: PrivateArtifactClaim,
    checker_ref: BlobRef,
) -> AdmittedQuiescentRelease {
    let worker = db.launch_ownership(attempt_id).unwrap().unwrap();
    AdmittedQuiescentRelease {
        launch_token: worker.request.launch_token,
        worker_receipt_claim: PrivateArtifactClaim {
            scope: scope.clone(),
            task_id: TaskId("task0".into()),
            attempt_id: attempt_id.clone(),
            reservation_id: "launch-reservation".into(),
            kind: PrivateArtifactKind::ControllerReceipt,
            artifact_id: "checker-fixture-job-zero".into(),
            event_id: "checker-fixture-job-zero-event".into(),
        },
        worker_receipt_ref: worker.settlement_blob.unwrap(),
        checker_receipts: vec![QuiescentCheckerReceipt {
            ordinal: 1,
            receipt_claim: checker_claim,
            receipt_ref: checker_ref,
        }],
    }
}

#[test]
fn checker_bootstrap_zero_is_not_enough_to_pass_or_unlock_next_check() {
    let baseline = CheckerNativeFields::default();
    let cases = [
        (
            "payload_failed",
            CheckerNativeFields {
                payload_exit_code: Some(1),
                ..baseline
            },
        ),
        (
            "payload_missing",
            CheckerNativeFields {
                payload_exit_code: None,
                ..baseline
            },
        ),
        (
            "unregistered",
            CheckerNativeFields {
                process_registered: false,
                ..baseline
            },
        ),
        (
            "barrier_held",
            CheckerNativeFields {
                barrier_released: false,
                ..baseline
            },
        ),
        (
            "native_failed",
            CheckerNativeFields {
                native_outcome: CheckerNativeOutcome::Failed,
                ..baseline
            },
        ),
        (
            "truncated",
            CheckerNativeFields {
                output_truncated: true,
                ..baseline
            },
        ),
        (
            "error",
            CheckerNativeFields {
                error_present: true,
                ..baseline
            },
        ),
        (
            "view_unreadable_after_job_zero",
            CheckerNativeFields {
                native_outcome: CheckerNativeOutcome::Failed,
                error_present: true,
                view_intact: false,
                ..baseline
            },
        ),
    ];
    for (name, native) in cases {
        let (_, db, catalog, scope, _, sealed) = checker_verifying_fixture();
        let first = checker_request(&db, &scope, &sealed, 1);
        db.prepare_checker_ownership(&catalog, &first).unwrap();
        let result = settle_fixture_checker_with_native(&db, &catalog, &first, &sealed, 0, native);
        assert_eq!(result.passed, Some(false), "{name}");
        let second = checker_request(&db, &scope, &sealed, 2);
        assert!(
            db.prepare_checker_ownership(&catalog, &second).is_err(),
            "{name}"
        );
    }
}

#[test]
fn admitted_quiescent_release_binds_terminal_worker_and_checker_receipts() {
    let (temp, db, catalog, scope, facts, sealed) = checker_verifying_fixture();
    let reservation_id = "launch-reservation";
    let first = checker_request(&db, &scope, &sealed, 1);
    db.prepare_checker_ownership(&catalog, &first).unwrap();
    let checker = settle_fixture_checker(&db, &catalog, &first, &sealed, 1);
    assert_eq!(checker.passed, Some(false));
    let admitted = fixture_quiescent_context(
        &db,
        &scope,
        &first.attempt_id,
        PrivateArtifactClaim {
            scope: scope.clone(),
            task_id: first.task_id.clone(),
            attempt_id: first.attempt_id.clone(),
            reservation_id: reservation_id.into(),
            kind: PrivateArtifactKind::ControllerReceipt,
            artifact_id: "checker-result-1".into(),
            event_id: "checker-result-event-1".into(),
        },
        checker.settlement_blob.unwrap(),
    );
    let proof = CapacityReleaseEvidence {
        kind: CapacityReleaseEvidenceKind::QuiescenceReconciled,
        receipt_id: admitted.worker_receipt_claim.event_id.clone(),
        evidence_digest: canonical_digest(&admitted, 1).unwrap().0,
        observed_at_ms: 151,
    };
    assert!(db
        .release_admitted_quiescent_capacity(&catalog, reservation_id)
        .is_err());
    assert!(db
        .bind_admitted_quiescent_release_proof(
            &catalog,
            reservation_id,
            "bind-quiescent",
            &proof,
            &admitted,
        )
        .is_err());
    let verifying = db.routing_history(&scope).unwrap().unwrap().attempts[0].clone();
    let failed = step(
        &db,
        &scope,
        &facts,
        &verifying,
        AttemptState::Failed,
        RoutingReceipts::default(),
    )
    .unwrap();
    assert!(failed.ownership_released);
    let mut wrong_token = admitted.clone();
    wrong_token.launch_token = "wrong-token".into();
    assert!(db
        .bind_admitted_quiescent_release_proof(
            &catalog,
            reservation_id,
            "bind-quiescent",
            &proof,
            &wrong_token,
        )
        .is_err());
    let mut wrong_ref = admitted.clone();
    wrong_ref.checker_receipts[0].receipt_ref.digest = digest("wrong-checker-receipt");
    assert!(db
        .bind_admitted_quiescent_release_proof(
            &catalog,
            reservation_id,
            "bind-quiescent",
            &proof,
            &wrong_ref,
        )
        .is_err());
    let mut wrong_proof = proof.clone();
    wrong_proof.evidence_digest = digest("unrelated-quiescence").0;
    assert!(db
        .bind_admitted_quiescent_release_proof(
            &catalog,
            reservation_id,
            "bind-quiescent",
            &wrong_proof,
            &admitted,
        )
        .is_err());
    let bound = db
        .bind_admitted_quiescent_release_proof(
            &catalog,
            reservation_id,
            "bind-quiescent",
            &proof,
            &admitted,
        )
        .unwrap();
    assert_eq!(bound.phase, CapacityIntentPhase::ReleaseProofBound);
    assert_eq!(
        db.bind_admitted_quiescent_release_proof(
            &catalog,
            reservation_id,
            "bind-quiescent",
            &proof,
            &admitted,
        )
        .unwrap(),
        bound
    );
    assert!(db
        .bind_admitted_quiescent_release_proof(
            &catalog,
            reservation_id,
            "changed-bind-event",
            &proof,
            &admitted,
        )
        .is_err());
    drop(db);
    let reopened = PytxoStore::open(&temp.path().join("store.db")).unwrap();
    assert_eq!(
        reopened.capacity_intent(reservation_id).unwrap(),
        Some(bound)
    );
    assert!(reopened
        .close_released_capacity_intent(&catalog, reservation_id, "close-too-early")
        .is_err());
    let wrong_catalog = Catalog::open(&temp.path().join("wrong-catalog.db")).unwrap();
    assert!(reopened
        .release_admitted_quiescent_capacity(&wrong_catalog, reservation_id)
        .is_err());
    assert_eq!(
        catalog
            .capacity_pool_status("launch-slot")
            .unwrap()
            .unwrap()
            .held_units,
        1
    );
    let released = reopened
        .release_admitted_quiescent_capacity(&catalog, reservation_id)
        .unwrap();
    assert_eq!(released.release_evidence, Some(proof));
    assert_eq!(
        reopened
            .release_admitted_quiescent_capacity(&catalog, reservation_id)
            .unwrap(),
        released
    );
    assert_eq!(
        catalog
            .capacity_pool_status("launch-slot")
            .unwrap()
            .unwrap()
            .held_units,
        0
    );
    drop(reopened);
    let after_release_restart = PytxoStore::open(&temp.path().join("store.db")).unwrap();
    let closed = after_release_restart
        .close_released_capacity_intent(&catalog, reservation_id, "close-quiescent")
        .unwrap();
    assert_eq!(closed.phase, CapacityIntentPhase::Closed);
    assert_eq!(
        after_release_restart
            .close_released_capacity_intent(&catalog, reservation_id, "close-quiescent")
            .unwrap(),
        closed
    );
}

#[test]
fn admitted_quiescent_release_accepts_cancelled_checker_with_positive_no_create_receipt() {
    let (_, db, catalog, scope, facts, sealed) = checker_verifying_fixture();
    let first = checker_request(&db, &scope, &sealed, 1);
    db.prepare_checker_ownership(&catalog, &first).unwrap();
    db.cancel_routing_mission(&scope, "cancel-quiescent-checker", 0, 160)
        .unwrap();
    let claim = PrivateArtifactClaim {
        scope: scope.clone(),
        task_id: first.task_id.clone(),
        attempt_id: first.attempt_id.clone(),
        reservation_id: first.reservation_id.clone(),
        kind: PrivateArtifactKind::ControllerReceipt,
        artifact_id: "quiescent-checker-no-create".into(),
        event_id: "quiescent-checker-no-create-event".into(),
    };
    let reference = db
        .put_controller_receipt(
            &claim,
            &ControllerReceiptEnvelope {
                schema_version: 1,
                scope: scope.clone(),
                task_id: first.task_id.clone(),
                attempt_id: first.attempt_id.clone(),
                reservation_id: first.reservation_id.clone(),
                observed_at_ms: 161,
                evidence_id: claim.event_id.clone(),
                source: ReceiptSource::TrustedController,
                observation: ControllerObservation::CheckerNotCreated {
                    check_id: first.check_id.clone(),
                    ordinal: 1,
                    cancellation_event_id: "cancel-quiescent-checker".into(),
                },
            },
        )
        .unwrap();
    db.close_prepared_checker_without_create(&CheckerSettlement {
        scope: scope.clone(),
        attempt_id: first.attempt_id.clone(),
        ordinal: 1,
        event_id: "close-quiescent-checker".into(),
        receipt_claim: claim.clone(),
        receipt_ref: reference.clone(),
    })
    .unwrap();
    let verifying = db.routing_history(&scope).unwrap().unwrap().attempts[0].clone();
    let cancelled = step(
        &db,
        &scope,
        &facts,
        &verifying,
        AttemptState::Cancelled,
        RoutingReceipts::default(),
    )
    .unwrap();
    assert!(cancelled.ownership_released);
    let admitted = fixture_quiescent_context(&db, &scope, &first.attempt_id, claim, reference);
    let proof = CapacityReleaseEvidence {
        kind: CapacityReleaseEvidenceKind::QuiescenceReconciled,
        receipt_id: admitted.worker_receipt_claim.event_id.clone(),
        evidence_digest: canonical_digest(&admitted, 1).unwrap().0,
        observed_at_ms: 161,
    };
    db.bind_admitted_quiescent_release_proof(
        &catalog,
        "launch-reservation",
        "bind-cancelled-quiescent",
        &proof,
        &admitted,
    )
    .unwrap();
    db.release_admitted_quiescent_capacity(&catalog, "launch-reservation")
        .unwrap();
    assert_eq!(
        db.close_released_capacity_intent(
            &catalog,
            "launch-reservation",
            "close-cancelled-quiescent",
        )
        .unwrap()
        .phase,
        CapacityIntentPhase::Closed
    );
}

#[test]
fn admitted_quiescent_release_stays_held_if_a_settlement_receipt_disappears() {
    let (temp, db, catalog, scope, facts, sealed) = checker_verifying_fixture();
    let first = checker_request(&db, &scope, &sealed, 1);
    db.prepare_checker_ownership(&catalog, &first).unwrap();
    let checker = settle_fixture_checker(&db, &catalog, &first, &sealed, 1);
    let claim = PrivateArtifactClaim {
        scope: scope.clone(),
        task_id: first.task_id.clone(),
        attempt_id: first.attempt_id.clone(),
        reservation_id: first.reservation_id.clone(),
        kind: PrivateArtifactKind::ControllerReceipt,
        artifact_id: "checker-result-1".into(),
        event_id: "checker-result-event-1".into(),
    };
    let admitted = fixture_quiescent_context(
        &db,
        &scope,
        &first.attempt_id,
        claim,
        checker.settlement_blob.unwrap(),
    );
    let verifying = db.routing_history(&scope).unwrap().unwrap().attempts[0].clone();
    step(
        &db,
        &scope,
        &facts,
        &verifying,
        AttemptState::Failed,
        RoutingReceipts::default(),
    )
    .unwrap();
    let proof = CapacityReleaseEvidence {
        kind: CapacityReleaseEvidenceKind::QuiescenceReconciled,
        receipt_id: admitted.worker_receipt_claim.event_id.clone(),
        evidence_digest: canonical_digest(&admitted, 1).unwrap().0,
        observed_at_ms: 151,
    };
    db.bind_admitted_quiescent_release_proof(
        &catalog,
        "launch-reservation",
        "bind-then-corrupt-quiescence",
        &proof,
        &admitted,
    )
    .unwrap();
    let conn = rusqlite::Connection::open(temp.path().join("store.db")).unwrap();
    conn.execute(
        "UPDATE routing_private_artifacts SET bytes=X'00' WHERE artifact_id='checker-result-1'",
        [],
    )
    .unwrap();
    assert!(db
        .release_admitted_quiescent_capacity(&catalog, "launch-reservation")
        .is_err());
    assert_eq!(
        catalog
            .capacity_pool_status("launch-slot")
            .unwrap()
            .unwrap()
            .held_units,
        1
    );
    assert!(db.unresolved_capacity_intent_scopes().is_err());
}

#[test]
fn checker_owners_are_ordered_one_use_and_survive_reopen() {
    let (temp, db, catalog, scope, facts, sealed) = checker_verifying_fixture();
    let first = checker_request(&db, &scope, &sealed, 1);
    let second = checker_request(&db, &scope, &sealed, 2);
    assert!(db.prepare_checker_ownership(&catalog, &second).is_err());
    assert_eq!(
        db.prepare_checker_ownership(&catalog, &first)
            .unwrap()
            .phase,
        CheckerOwnershipPhase::Prepared
    );
    assert_eq!(
        db.prepare_checker_ownership(&catalog, &first)
            .unwrap()
            .phase,
        CheckerOwnershipPhase::Prepared
    );
    let mut changed = first.clone();
    changed.sealed_view = BlobRef {
        digest: digest("changed-view"),
        byte_length: sealed.byte_length,
    };
    assert!(db.prepare_checker_ownership(&catalog, &changed).is_err());
    let verifying = db.routing_history(&scope).unwrap().unwrap().attempts[0].clone();
    assert_eq!(verifying.owned_checker_count, 1);
    assert!(step(
        &db,
        &scope,
        &facts,
        &verifying,
        AttemptState::Failed,
        RoutingReceipts::default()
    )
    .is_err());
    let create = CheckerCreateRequest {
        scope: scope.clone(),
        attempt_id: first.attempt_id.clone(),
        ordinal: 1,
        event_id: "checker-create-1".into(),
        job_name: "checker-job-1".into(),
        launch_nonce: "checker-nonce-1".into(),
        now_ms: 150,
    };
    assert!(matches!(
        db.authorize_checker_create(&catalog, &create).unwrap(),
        CheckerCreateOutcome::Fresh(_)
    ));
    assert!(matches!(
        db.authorize_checker_create(&catalog, &create).unwrap(),
        CheckerCreateOutcome::Replay(_)
    ));
    let mut changed_create = create.clone();
    changed_create.job_name = "other-job".into();
    assert!(db
        .authorize_checker_create(&catalog, &changed_create)
        .is_err());
    assert!(db.prepare_checker_ownership(&catalog, &second).is_err());
    let reopened = PytxoStore::open(&temp.path().join("store.db")).unwrap();
    assert!(matches!(
        reopened
            .authorize_checker_create(&catalog, &create)
            .unwrap(),
        CheckerCreateOutcome::Replay(_)
    ));
    assert_eq!(
        reopened.unresolved_checker_ownership_scopes().unwrap(),
        vec![scope]
    );
}

#[test]
fn checker_success_publishes_only_the_exact_ordered_owned_aggregate() {
    let (temp, db, catalog, scope, mut facts, sealed) =
        checker_verifying_fixture_with_dependent(true);
    let first = checker_request(&db, &scope, &sealed, 1);
    db.prepare_checker_ownership(&catalog, &first).unwrap();
    assert_eq!(
        settle_fixture_checker(&db, &catalog, &first, &sealed, 0).passed,
        Some(true)
    );
    assert!(db.owned_checker_pass_digest(&first.attempt_id).is_err());
    let second = checker_request(&db, &scope, &sealed, 2);
    db.prepare_checker_ownership(&catalog, &second).unwrap();
    assert!(db.owned_checker_pass_digest(&first.attempt_id).is_err());
    assert_eq!(
        settle_fixture_checker(&db, &catalog, &second, &sealed, 0).passed,
        Some(true)
    );
    let verifying = db.routing_history(&scope).unwrap().unwrap().attempts[0].clone();
    facts.now_ms = 152;
    assert!(step(
        &db,
        &scope,
        &facts,
        &verifying,
        AttemptState::Passed,
        RoutingReceipts {
            checks: Some(digest("unverified-aggregate")),
            ..Default::default()
        }
    )
    .is_err());
    let aggregate = db.owned_checker_pass_digest(&first.attempt_id).unwrap();
    let pass_request = TransitionRoutingAttempt {
        scope: scope.clone(),
        event_id: format!(
            "{}-{}-{}",
            verifying.attempt_id,
            verifying.revision,
            AttemptState::Passed.as_str()
        ),
        attempt_id: verifying.attempt_id.clone(),
        expected_attempt_revision: verifying.revision,
        expected_task_revision: db.routing_history(&scope).unwrap().unwrap().tasks[0].revision,
        to: AttemptState::Passed,
        facts: facts.clone(),
        receipts: RoutingReceipts {
            checks: Some(aggregate.clone()),
            ..Default::default()
        },
        failure: None,
    };
    let passed = db.transition_routing_attempt(&pass_request).unwrap();
    assert!(passed.ownership_released);
    let trace = db
        .routing_benchmark_trace(&scope, &TaskId("task0".into()), &[7; 32])
        .unwrap()
        .unwrap();
    let exported = serde_json::to_string(&trace).unwrap();
    assert!(!exported.contains(&passed.receipts.sealed_output.as_ref().unwrap().0));
    assert!(!exported.contains(&passed.receipts.checks.as_ref().unwrap().0));
    assert_eq!(
        db.transition_routing_attempt(&pass_request).unwrap(),
        passed
    );
    let history = db.routing_history(&scope).unwrap().unwrap();
    assert_eq!(
        history.tasks[0]
            .winner
            .as_ref()
            .unwrap()
            .verification_receipt_digest,
        aggregate
    );
    let dependency = db
        .read_routing_dependency_output(&scope, &TaskId("task1".into()), &TaskId("task0".into()))
        .unwrap();
    assert_eq!(dependency.winner, history.tasks[0].winner.clone().unwrap());
    assert_eq!(dependency.sealed_output, sealed);
    assert_eq!(dependency.bytes, b"sealed fixture output");
    assert!(db
        .read_routing_dependency_output(&scope, &TaskId("task0".into()), &TaskId("task1".into()))
        .is_err());
    assert!(db
        .read_routing_dependency_output(&scope, &TaskId("task1".into()), &TaskId("missing".into()))
        .is_err());
    let conn = rusqlite::Connection::open(temp.path().join("store.db")).unwrap();
    let mut forged_consumer = history.tasks[0].clone();
    forged_consumer
        .registration
        .contract
        .dependencies
        .push(TaskId("task0".into()));
    conn.execute(
        "UPDATE routing_tasks SET record_json=?1 WHERE run_id=?2 AND task_id='task0'",
        rusqlite::params![
            serde_json::to_string(&forged_consumer).unwrap(),
            scope.run_id.0
        ],
    )
    .unwrap();
    let forged_edge_error = db
        .read_routing_dependency_output(&scope, &TaskId("task0".into()), &TaskId("task0".into()))
        .unwrap_err();
    assert!(
        forged_edge_error
            .to_string()
            .contains("registered task projection differs from mission"),
        "{forged_edge_error}"
    );
    conn.execute(
        "UPDATE routing_tasks SET record_json=?1 WHERE run_id=?2 AND task_id='task0'",
        rusqlite::params![
            serde_json::to_string(&history.tasks[0]).unwrap(),
            scope.run_id.0
        ],
    )
    .unwrap();
    let child_request = request(&db, &scope, &facts, "task1");
    conn.execute(
        "UPDATE routing_private_artifacts SET bytes=X'00' WHERE artifact_id='checker-fixture-sealed-output'",
        [],
    )
    .unwrap();
    let corrupt_parent_rejection = db.admit_routing_attempt(&child_request).unwrap_err();
    assert!(
        corrupt_parent_rejection
            .to_string()
            .contains("private artifact SQL projection or bytes changed"),
        "{corrupt_parent_rejection}"
    );
    assert!(db
        .read_routing_dependency_output(&scope, &TaskId("task1".into()), &TaskId("task0".into()))
        .is_err());
    conn.execute(
        "UPDATE routing_private_artifacts SET bytes=?1 WHERE artifact_id='checker-fixture-sealed-output'",
        [b"sealed fixture output".as_slice()],
    )
    .unwrap();
    assert!(db.unresolved_checker_ownership_scopes().unwrap().is_empty());
    drop(db);
    let reopened = PytxoStore::open(&temp.path().join("store.db")).unwrap();
    assert_eq!(
        reopened
            .read_routing_dependency_output(
                &scope,
                &TaskId("task1".into()),
                &TaskId("task0".into())
            )
            .unwrap()
            .bytes,
        b"sealed fixture output"
    );
    assert_eq!(
        reopened
            .owned_checker_pass_digest(&first.attempt_id)
            .unwrap(),
        aggregate
    );
    let retained_checker_bytes: Vec<u8> = conn
        .query_row(
            "SELECT bytes FROM routing_private_artifacts WHERE artifact_id='checker-result-2'",
            [],
            |row| row.get(0),
        )
        .unwrap();
    conn.execute(
        "UPDATE routing_private_artifacts SET bytes=X'00' WHERE artifact_id='checker-result-2'",
        [],
    )
    .unwrap();
    assert!(reopened
        .owned_checker_pass_digest(&first.attempt_id)
        .is_err());
    assert!(reopened.unresolved_routing_attempts().is_err());
    assert!(reopened.routing_display_summary(&scope).is_err());
    assert!(reopened.transition_routing_attempt(&pass_request).is_err());
    assert!(reopened
        .preview_routing_decision(&scope, &TaskId("task1".into()), &facts, None)
        .is_err());
    assert!(reopened.admit_routing_attempt(&child_request).is_err());
    conn.execute(
        "UPDATE routing_private_artifacts SET bytes=?1 WHERE artifact_id='checker-result-2'",
        [retained_checker_bytes],
    )
    .unwrap();
    assert!(reopened
        .read_routing_dependency_output(&scope, &TaskId("task1".into()), &TaskId("task0".into()))
        .is_ok());
    reopened
        .cancel_routing_mission(&scope, "cancel-retained-dependency", 0, facts.now_ms + 1)
        .unwrap();
    assert!(reopened
        .read_routing_dependency_output(&scope, &TaskId("task1".into()), &TaskId("task0".into()))
        .unwrap_err()
        .to_string()
        .contains("routed mission is cancelled"));
}

#[test]
fn failed_checker_cannot_unlock_next_check() {
    let (_, db, catalog, scope, facts, sealed) = checker_verifying_fixture();
    let first = checker_request(&db, &scope, &sealed, 1);
    db.prepare_checker_ownership(&catalog, &first).unwrap();
    assert_eq!(
        settle_fixture_checker(&db, &catalog, &first, &sealed, 1).passed,
        Some(false)
    );
    let second = checker_request(&db, &scope, &sealed, 2);
    assert!(db.prepare_checker_ownership(&catalog, &second).is_err());
    let verifying = db.routing_history(&scope).unwrap().unwrap().attempts[0].clone();
    assert!(step(
        &db,
        &scope,
        &facts,
        &verifying,
        AttemptState::Failed,
        RoutingReceipts::default()
    )
    .is_ok());
}

#[test]
fn lost_checker_row_cannot_erase_outstanding_native_ownership() {
    let (temp, db, catalog, scope, facts, sealed) = checker_verifying_fixture();
    let first = checker_request(&db, &scope, &sealed, 1);
    db.prepare_checker_ownership(&catalog, &first).unwrap();
    let conn = rusqlite::Connection::open(temp.path().join("store.db")).unwrap();
    conn.execute(
        "DELETE FROM attempt_checker_ownership WHERE attempt_id=?1 AND ordinal=1",
        [&first.attempt_id.0],
    )
    .unwrap();
    let verifying = db.routing_history(&scope).unwrap().unwrap().attempts[0].clone();
    assert!(step(
        &db,
        &scope,
        &facts,
        &verifying,
        AttemptState::Failed,
        RoutingReceipts::default()
    )
    .is_err());
    assert!(db.unresolved_routing_attempts().is_err());
}

#[test]
fn prepared_checker_closes_after_cancel_but_ambiguous_create_cannot_claim_no_create() {
    let (_, db, catalog, scope, facts, sealed) = checker_verifying_fixture();
    let first = checker_request(&db, &scope, &sealed, 1);
    db.prepare_checker_ownership(&catalog, &first).unwrap();
    let cancellation = "checker-cancel-event";
    db.cancel_routing_mission(&scope, cancellation, 0, 160)
        .unwrap();
    let claim = PrivateArtifactClaim {
        scope: scope.clone(),
        task_id: first.task_id.clone(),
        attempt_id: first.attempt_id.clone(),
        reservation_id: first.reservation_id.clone(),
        kind: PrivateArtifactKind::ControllerReceipt,
        artifact_id: "checker-no-create-artifact".into(),
        event_id: "checker-no-create-observation".into(),
    };
    let receipt = ControllerReceiptEnvelope {
        schema_version: 1,
        scope: scope.clone(),
        task_id: first.task_id.clone(),
        attempt_id: first.attempt_id.clone(),
        reservation_id: first.reservation_id.clone(),
        observed_at_ms: 161,
        evidence_id: claim.event_id.clone(),
        source: ReceiptSource::TrustedController,
        observation: ControllerObservation::CheckerNotCreated {
            check_id: first.check_id.clone(),
            ordinal: 1,
            cancellation_event_id: cancellation.into(),
        },
    };
    let reference = db.put_controller_receipt(&claim, &receipt).unwrap();
    let settlement = CheckerSettlement {
        scope: scope.clone(),
        attempt_id: first.attempt_id.clone(),
        ordinal: 1,
        event_id: "checker-no-create-close".into(),
        receipt_claim: claim,
        receipt_ref: reference,
    };
    assert_eq!(
        db.close_prepared_checker_without_create(&settlement)
            .unwrap()
            .phase,
        CheckerOwnershipPhase::ClosedNoCreate
    );
    let verifying = db.routing_history(&scope).unwrap().unwrap().attempts[0].clone();
    assert!(step(
        &db,
        &scope,
        &facts,
        &verifying,
        AttemptState::Cancelled,
        RoutingReceipts::default()
    )
    .is_ok());

    let (_, db, catalog, scope, _, sealed) = checker_verifying_fixture();
    let first = checker_request(&db, &scope, &sealed, 1);
    db.prepare_checker_ownership(&catalog, &first).unwrap();
    db.authorize_checker_create(
        &catalog,
        &CheckerCreateRequest {
            scope: scope.clone(),
            attempt_id: first.attempt_id.clone(),
            ordinal: 1,
            event_id: "ambiguous-checker-create".into(),
            job_name: "ambiguous-checker-job".into(),
            launch_nonce: "ambiguous-checker-nonce".into(),
            now_ms: 150,
        },
    )
    .unwrap();
    db.cancel_routing_mission(&scope, "ambiguous-checker-cancel", 0, 160)
        .unwrap();
    let claim = PrivateArtifactClaim {
        scope: scope.clone(),
        task_id: first.task_id.clone(),
        attempt_id: first.attempt_id.clone(),
        reservation_id: first.reservation_id.clone(),
        kind: PrivateArtifactKind::ControllerReceipt,
        artifact_id: "ambiguous-checker-no-create".into(),
        event_id: "ambiguous-checker-no-create-event".into(),
    };
    let receipt = ControllerReceiptEnvelope {
        schema_version: 1,
        scope: scope.clone(),
        task_id: first.task_id.clone(),
        attempt_id: first.attempt_id.clone(),
        reservation_id: first.reservation_id.clone(),
        observed_at_ms: 161,
        evidence_id: claim.event_id.clone(),
        source: ReceiptSource::TrustedController,
        observation: ControllerObservation::CheckerNotCreated {
            check_id: first.check_id,
            ordinal: 1,
            cancellation_event_id: "ambiguous-checker-cancel".into(),
        },
    };
    let reference = db.put_controller_receipt(&claim, &receipt).unwrap();
    assert!(db
        .close_prepared_checker_without_create(&CheckerSettlement {
            scope: scope.clone(),
            attempt_id: first.attempt_id,
            ordinal: 1,
            event_id: "ambiguous-checker-close".into(),
            receipt_claim: claim,
            receipt_ref: reference,
        })
        .is_err());
    assert_eq!(
        db.unresolved_checker_ownership_scopes().unwrap(),
        vec![scope]
    );
}

#[test]
fn checker_sql_and_retained_receipt_corruption_fail_closed() {
    let (temp, db, catalog, scope, _, sealed) = checker_verifying_fixture();
    let first = checker_request(&db, &scope, &sealed, 1);
    db.prepare_checker_ownership(&catalog, &first).unwrap();
    let settled = settle_fixture_checker(&db, &catalog, &first, &sealed, 0);
    let conn = rusqlite::Connection::open(temp.path().join("store.db")).unwrap();
    conn.execute(
        "UPDATE attempt_checker_ownership SET revision=revision+1 WHERE attempt_id=?1 AND ordinal=1",
        [&first.attempt_id.0],
    )
    .unwrap();
    assert!(db.checker_ownership(&first.attempt_id, 1).is_err());
    assert!(db.unresolved_routing_attempts().is_err());
    conn.execute(
        "UPDATE attempt_checker_ownership SET revision=revision-1 WHERE attempt_id=?1 AND ordinal=1",
        [&first.attempt_id.0],
    )
    .unwrap();
    conn.execute(
        "UPDATE routing_private_artifacts SET bytes=X'00' WHERE artifact_id=?1",
        [settled.settlement_artifact_id.unwrap()],
    )
    .unwrap();
    assert!(db.checker_ownership(&first.attempt_id, 1).is_err());
    assert!(db.unresolved_routing_attempts().is_err());
}

#[test]
fn checker_rejects_task_recipe_rewrite_before_prepare_and_fresh_create() {
    for prepare_first in [false, true] {
        let (temp, db, catalog, scope, _, sealed) = checker_verifying_fixture();
        let first = checker_request(&db, &scope, &sealed, 1);
        if prepare_first {
            db.prepare_checker_ownership(&catalog, &first).unwrap();
        }
        let mut task = db.routing_history(&scope).unwrap().unwrap().tasks[0].clone();
        task.registration.check_recipes[0].command = "echo unreviewed".into();
        task.registration.contract.checks[0] =
            task.registration.check_recipes[0].reference().unwrap();
        let conn = rusqlite::Connection::open(temp.path().join("store.db")).unwrap();
        conn.execute(
            "UPDATE routing_tasks SET record_json=?1 WHERE run_id=?2 AND task_id='task0'",
            rusqlite::params![serde_json::to_string(&task).unwrap(), scope.run_id.0],
        )
        .unwrap();
        if prepare_first {
            assert!(db
                .authorize_checker_create(
                    &catalog,
                    &CheckerCreateRequest {
                        scope: scope.clone(),
                        attempt_id: first.attempt_id.clone(),
                        ordinal: 1,
                        event_id: "checker-rewritten-create".into(),
                        job_name: "checker-rewritten-job".into(),
                        launch_nonce: "checker-rewritten-nonce".into(),
                        now_ms: 150,
                    },
                )
                .is_err());
            let phase: String = conn
                .query_row(
                    "SELECT phase FROM attempt_checker_ownership WHERE attempt_id=?1 AND ordinal=1",
                    [&first.attempt_id.0],
                    |row| row.get(0),
                )
                .unwrap();
            assert_eq!(phase, "prepared");
        } else {
            assert!(db.prepare_checker_ownership(&catalog, &first).is_err());
            assert!(db
                .checker_ownership(&first.attempt_id, 1)
                .unwrap()
                .is_none());
        }
    }
}

#[test]
fn checker_row_rewrite_cannot_authorize_unreviewed_recipe() {
    let (temp, db, catalog, scope, _, sealed) = checker_verifying_fixture();
    let first = checker_request(&db, &scope, &sealed, 1);
    let mut row = db.prepare_checker_ownership(&catalog, &first).unwrap();
    row.request.recipe_digest = digest("unreviewed-checker-recipe");
    let conn = rusqlite::Connection::open(temp.path().join("store.db")).unwrap();
    conn.execute(
        "UPDATE attempt_checker_ownership SET recipe_digest=?1,record_json=?2 WHERE attempt_id=?3 AND ordinal=1",
        rusqlite::params![
            row.request.recipe_digest.0,
            serde_json::to_string(&row).unwrap(),
            first.attempt_id.0
        ],
    )
    .unwrap();
    assert!(db
        .authorize_checker_create(
            &catalog,
            &CheckerCreateRequest {
                scope,
                attempt_id: first.attempt_id.clone(),
                ordinal: 1,
                event_id: "checker-unreviewed-create".into(),
                job_name: "checker-unreviewed-job".into(),
                launch_nonce: "checker-unreviewed-nonce".into(),
                now_ms: 150,
            },
        )
        .is_err());
    assert!(db.unresolved_routing_attempts().is_err());
}

#[test]
fn checker_owner_cannot_share_worker_job_name() {
    let (_, db, catalog, scope, _, sealed) = checker_verifying_fixture();
    let first = checker_request(&db, &scope, &sealed, 1);
    db.prepare_checker_ownership(&catalog, &first).unwrap();
    let worker_job = db
        .launch_ownership(&first.attempt_id)
        .unwrap()
        .unwrap()
        .job_name
        .unwrap();
    assert!(db
        .authorize_checker_create(
            &catalog,
            &CheckerCreateRequest {
                scope,
                attempt_id: first.attempt_id.clone(),
                ordinal: 1,
                event_id: "checker-colliding-create".into(),
                job_name: worker_job,
                launch_nonce: "checker-colliding-nonce".into(),
                now_ms: 150,
            },
        )
        .is_err());
    assert_eq!(
        db.checker_ownership(&first.attempt_id, 1)
            .unwrap()
            .unwrap()
            .phase,
        CheckerOwnershipPhase::Prepared
    );
}

#[test]
fn checker_row_survives_corruptly_decremented_attempt_marker() {
    let (temp, db, catalog, scope, facts, sealed) = checker_verifying_fixture();
    let first = checker_request(&db, &scope, &sealed, 1);
    db.prepare_checker_ownership(&catalog, &first).unwrap();
    let conn = rusqlite::Connection::open(temp.path().join("store.db")).unwrap();
    let mut attempt = db.routing_history(&scope).unwrap().unwrap().attempts[0].clone();
    attempt.owned_checker_count = 0;
    conn.execute(
        "UPDATE routing_attempts SET record_json=?1 WHERE attempt_id=?2",
        rusqlite::params![serde_json::to_string(&attempt).unwrap(), first.attempt_id.0],
    )
    .unwrap();
    assert!(db
        .authorize_checker_create(
            &catalog,
            &CheckerCreateRequest {
                scope: scope.clone(),
                attempt_id: first.attempt_id.clone(),
                ordinal: 1,
                event_id: "checker-marker-lost-create".into(),
                job_name: "checker-marker-lost-job".into(),
                launch_nonce: "checker-marker-lost-nonce".into(),
                now_ms: 150,
            },
        )
        .is_err());
    assert!(step(
        &db,
        &scope,
        &facts,
        &attempt,
        AttemptState::Failed,
        RoutingReceipts::default()
    )
    .is_err());
    assert!(db.unresolved_routing_attempts().is_err());
}

#[test]
fn checker_loses_authority_if_retained_sealed_view_disappears() {
    let (temp, db, catalog, scope, facts, sealed) = checker_verifying_fixture();
    let first = checker_request(&db, &scope, &sealed, 1);
    db.prepare_checker_ownership(&catalog, &first).unwrap();
    settle_fixture_checker(&db, &catalog, &first, &sealed, 0);
    let conn = rusqlite::Connection::open(temp.path().join("store.db")).unwrap();
    conn.execute(
        "DELETE FROM routing_private_artifacts WHERE artifact_id='checker-fixture-sealed-output'",
        [],
    )
    .unwrap();
    assert!(db.checker_ownership(&first.attempt_id, 1).is_err());
    let verifying = db.routing_history(&scope).unwrap().unwrap().attempts[0].clone();
    assert!(step(
        &db,
        &scope,
        &facts,
        &verifying,
        AttemptState::Failed,
        RoutingReceipts::default()
    )
    .is_err());
    assert!(db.unresolved_routing_attempts().is_err());
}

#[test]
fn settled_checker_cannot_change_worker_launch_token() {
    let (temp, db, catalog, scope, facts, sealed) = checker_verifying_fixture();
    let first = checker_request(&db, &scope, &sealed, 1);
    db.prepare_checker_ownership(&catalog, &first).unwrap();
    let mut row = settle_fixture_checker(&db, &catalog, &first, &sealed, 0);
    row.request.launch_token = "different-launch-token".into();
    let conn = rusqlite::Connection::open(temp.path().join("store.db")).unwrap();
    conn.execute(
        "UPDATE attempt_checker_ownership SET launch_token=?1,record_json=?2 WHERE attempt_id=?3 AND ordinal=1",
        rusqlite::params![row.request.launch_token, serde_json::to_string(&row).unwrap(), first.attempt_id.0],
    )
    .unwrap();
    assert!(db.checker_ownership(&first.attempt_id, 1).is_err());
    let verifying = db.routing_history(&scope).unwrap().unwrap().attempts[0].clone();
    assert!(step(
        &db,
        &scope,
        &facts,
        &verifying,
        AttemptState::Failed,
        RoutingReceipts::default()
    )
    .is_err());
    assert!(db.unresolved_routing_attempts().is_err());
}

#[test]
fn worker_fresh_create_rejects_preexisting_checker_job_name() {
    let (temp, db, catalog, scope, facts, _) = launch_ownership_fixture();
    let worker = prepared_launch_request(&scope);
    db.prepare_launch_ownership(&catalog, &worker).unwrap();
    record_launch_checks(&db, &scope, &facts);
    // A later worker in this domain must not take a Job already recorded for
    // any checker. A deliberately malformed row also has to block creation.
    let conn = rusqlite::Connection::open(temp.path().join("store.db")).unwrap();
    conn.execute(
        "INSERT INTO attempt_checker_ownership
         (attempt_id,ordinal,check_id,domain_id,run_id,task_id,reservation_id,launch_token,
          recipe_digest,sealed_view_digest,sealed_view_length,prepare_event_id,job_name,
          launch_nonce,pid,start_identity,settlement_artifact_id,passed,phase,revision,record_json)
         VALUES (?1,1,'fixture-check',?2,?3,'task0','launch-reservation','fixture-launch-token',
          ?4,?5,5,'fixture-checker-prepare','shared-native-job','checker-nonce',
          NULL,NULL,NULL,NULL,'create_may_have_started',1,'{}')",
        rusqlite::params![
            worker.attempt_id.0,
            scope.domain_id.0,
            scope.run_id.0,
            digest("fixture-recipe").0,
            digest("fixture-sealed-view").0
        ],
    )
    .unwrap();
    let outcome = db.authorize_launch_create(&LaunchCreateRequest {
        scope,
        attempt_id: worker.attempt_id,
        event_id: "worker-colliding-create".into(),
        job_name: "shared-native-job".into(),
        launch_nonce: "worker-nonce".into(),
    });
    assert!(outcome.unwrap_err().to_string().contains("collides"));
}

#[test]
fn capacity_intent_replays_exact_request_and_only_pre_call_can_close_without_catalog() {
    let (_, db, catalog, request) = capacity_intent_fixture();
    let created = db.register_capacity_intent(&catalog, &request).unwrap();
    assert_eq!(created.phase, CapacityIntentPhase::Prepared);
    assert_eq!(
        db.register_capacity_intent(&catalog, &request).unwrap(),
        created
    );
    let mut changed = request.clone();
    changed.reservation.owner.process_start_identity = "other-process".into();
    assert!(db.register_capacity_intent(&catalog, &changed).is_err());
    let mut duplicate_task = request.clone();
    duplicate_task.reservation.reservation_id = "second-reservation".into();
    duplicate_task.reservation.attempt_id = "second-attempt".into();
    duplicate_task.event_id = "second-intent-event".into();
    assert!(db
        .register_capacity_intent(&catalog, &duplicate_task)
        .is_err());
    assert_eq!(
        db.unresolved_capacity_intent_scopes().unwrap(),
        vec![request.scope.clone()]
    );
    let closed = db
        .close_unissued_capacity_intent(&request.reservation.reservation_id, "intent.preclose.v1")
        .unwrap();
    assert_eq!(closed.phase, CapacityIntentPhase::Closed);
    assert_eq!(
        db.close_unissued_capacity_intent(
            &request.reservation.reservation_id,
            "intent.preclose.v1"
        )
        .unwrap(),
        closed
    );
    assert!(db
        .mark_capacity_reserve_may_have_started(
            &request.reservation.reservation_id,
            "reserve-call.v1"
        )
        .is_err());
    assert!(db.unresolved_capacity_intent_scopes().unwrap().is_empty());
    // Closing releases the task slot, but the original event cannot create a
    // different reservation under the same run.
    duplicate_task.event_id = request.event_id.clone();
    assert!(db
        .register_capacity_intent(&catalog, &duplicate_task)
        .is_err());
}

#[test]
fn optional_admitted_release_contexts_preserve_existing_intent_json() {
    let (temp, db, catalog, request) = capacity_intent_fixture();
    let stored = db.register_capacity_intent(&catalog, &request).unwrap();
    let conn = rusqlite::Connection::open(temp.path().join("store.db")).unwrap();
    let raw: String = conn
        .query_row(
            "SELECT record_json FROM routing_capacity_intents WHERE reservation_id=?1",
            [&request.reservation.reservation_id],
            |row| row.get(0),
        )
        .unwrap();
    assert!(!raw.contains("admitted_no_launch_release"));
    assert!(!raw.contains("admitted_quiescent_release"));
    drop(conn);
    drop(db);
    let reopened = PytxoStore::open(&temp.path().join("store.db")).unwrap();
    assert_eq!(
        reopened
            .capacity_intent(&request.reservation.reservation_id)
            .unwrap(),
        Some(stored)
    );
}

#[test]
fn capacity_intent_may_started_requires_exact_released_catalog_and_bound_proof() {
    let (_, db, catalog, request) = capacity_intent_fixture();
    db.register_capacity_intent(&catalog, &request).unwrap();
    let started = db
        .mark_capacity_reserve_may_have_started(
            &request.reservation.reservation_id,
            "reserve-call.v1",
        )
        .unwrap();
    assert_eq!(started.phase, CapacityIntentPhase::ReserveMayHaveStarted);
    assert_eq!(
        db.mark_capacity_reserve_may_have_started(
            &request.reservation.reservation_id,
            "reserve-call.v1"
        )
        .unwrap(),
        started
    );
    assert!(db
        .close_unissued_capacity_intent(&request.reservation.reservation_id, "too-late")
        .is_err());
    assert!(db
        .close_released_capacity_intent(&catalog, &request.reservation.reservation_id, "too-early")
        .is_err());
    assert_eq!(
        db.unresolved_capacity_intent_scopes().unwrap(),
        vec![request.scope.clone()],
        "an absent Catalog row after the call may have started is ambiguous"
    );
    catalog.reserve_capacity(&request.reservation).unwrap();
    let proof = fixture_unused_proof();
    db.cancel_routing_mission(&request.scope, "fixture.cancel.v1", 0, 108)
        .unwrap();
    db.bind_capacity_release_proof(
        &catalog,
        &request.reservation.reservation_id,
        "proof.bound.v1",
        &proof,
    )
    .unwrap();
    assert!(db
        .close_released_capacity_intent(
            &catalog,
            &request.reservation.reservation_id,
            "before-release"
        )
        .is_err());
    catalog
        .release_capacity_reservation(&CapacityReleaseRequest {
            reservation_id: request.reservation.reservation_id.clone(),
            evidence: proof,
        })
        .unwrap();
    let closed = db
        .close_released_capacity_intent(
            &catalog,
            &request.reservation.reservation_id,
            "intent.closed.v1",
        )
        .unwrap();
    assert_eq!(closed.phase, CapacityIntentPhase::Closed);
    assert!(db.unresolved_capacity_intent_scopes().unwrap().is_empty());
}

#[test]
fn capacity_intent_reopens_across_ambiguous_call_and_each_release_write() {
    let (temp, db, catalog, request) = capacity_intent_fixture();
    db.register_capacity_intent(&catalog, &request).unwrap();
    db.mark_capacity_reserve_may_have_started(&request.reservation.reservation_id, "may-start")
        .unwrap();
    drop(db);
    drop(catalog);
    let db = PytxoStore::open(&temp.path().join("store.db")).unwrap();
    let catalog =
        Catalog::open_existing_for_capacity_recovery(&temp.path().join("catalog.db")).unwrap();
    assert!(catalog
        .capacity_reservation(&request.reservation.reservation_id)
        .unwrap()
        .is_none());
    assert!(db
        .close_unissued_capacity_intent(&request.reservation.reservation_id, "too-late")
        .is_err());
    assert_eq!(
        db.unresolved_capacity_intent_scopes().unwrap(),
        vec![request.scope.clone()]
    );
}

#[test]
fn capacity_intent_reopens_after_proof_and_after_catalog_release() {
    let (temp, db, catalog, request) = capacity_intent_fixture();
    db.register_capacity_intent(&catalog, &request).unwrap();
    db.mark_capacity_reserve_may_have_started(&request.reservation.reservation_id, "may-start")
        .unwrap();
    catalog.reserve_capacity(&request.reservation).unwrap();
    db.cancel_routing_mission(&request.scope, "cancel-before-proof-reopen", 0, 120)
        .unwrap();
    let proof = fixture_unused_proof();
    db.bind_capacity_release_proof(
        &catalog,
        &request.reservation.reservation_id,
        "bind-before-restart",
        &proof,
    )
    .unwrap();
    drop(db);
    drop(catalog);
    let db = PytxoStore::open(&temp.path().join("store.db")).unwrap();
    let catalog =
        Catalog::open_existing_for_capacity_recovery(&temp.path().join("catalog.db")).unwrap();
    let persisted = db
        .capacity_intent(&request.reservation.reservation_id)
        .unwrap()
        .unwrap();
    assert_eq!(persisted.phase, CapacityIntentPhase::ReleaseProofBound);
    assert_eq!(persisted.expected_release, Some(proof.clone()));
    catalog
        .release_capacity_reservation(&CapacityReleaseRequest {
            reservation_id: request.reservation.reservation_id.clone(),
            evidence: proof,
        })
        .unwrap();
    drop(db);
    drop(catalog);
    let db = PytxoStore::open(&temp.path().join("store.db")).unwrap();
    let catalog =
        Catalog::open_existing_for_capacity_recovery(&temp.path().join("catalog.db")).unwrap();
    db.close_released_capacity_intent(
        &catalog,
        &request.reservation.reservation_id,
        "close-after-restart",
    )
    .unwrap();
    assert!(db.unresolved_capacity_intent_scopes().unwrap().is_empty());
}

#[test]
fn capacity_intent_requires_live_authority_at_creation_and_before_catalog_call() {
    let (_, db, catalog, request) = capacity_intent_fixture();
    let mut unknown_task = request.clone();
    unknown_task.task_id = TaskId("unregistered".into());
    unknown_task.reservation.reservation_id = "unknown-task-reservation".into();
    assert!(db
        .register_capacity_intent(&catalog, &unknown_task)
        .is_err());
    db.register_capacity_intent(&catalog, &request).unwrap();
    db.cancel_routing_mission(&request.scope, "cancel-before-reserve", 0, 120)
        .unwrap();
    assert!(db
        .mark_capacity_reserve_may_have_started(&request.reservation.reservation_id, "reserve-call")
        .is_err());
    assert!(db
        .register_capacity_intent(&catalog, &unknown_task)
        .is_err());
    assert!(catalog
        .capacity_reservation(&request.reservation.reservation_id)
        .unwrap()
        .is_none());

    let (_, db, catalog, request) = capacity_intent_fixture();
    db.register_capacity_intent(&catalog, &request).unwrap();
    db.finish_run("run", "completed").unwrap();
    assert!(db
        .mark_capacity_reserve_may_have_started(&request.reservation.reservation_id, "reserve-call")
        .is_err());
    assert!(catalog
        .capacity_reservation(&request.reservation.reservation_id)
        .unwrap()
        .is_none());
}

#[test]
fn capacity_intent_refuses_task_revision_projection_drift_before_reserve() {
    let (temp, db, catalog, request) = capacity_intent_fixture();
    db.register_capacity_intent(&catalog, &request).unwrap();
    let conn = rusqlite::Connection::open(temp.path().join("store.db")).unwrap();
    conn.execute(
        "UPDATE routing_tasks SET revision=revision+1 WHERE run_id=?1 AND task_id=?2",
        [&request.scope.run_id.0, &request.task_id.0],
    )
    .unwrap();
    assert!(db
        .mark_capacity_reserve_may_have_started(&request.reservation.reservation_id, "reserve")
        .is_err());
    assert!(catalog
        .capacity_reservation(&request.reservation.reservation_id)
        .unwrap()
        .is_none());
}

#[test]
fn capacity_intent_rejects_corrupt_projection_and_mismatched_catalog() {
    let (temp, db, catalog, request) = capacity_intent_fixture();
    db.register_capacity_intent(&catalog, &request).unwrap();
    db.mark_capacity_reserve_may_have_started(
        &request.reservation.reservation_id,
        "call.may.start",
    )
    .unwrap();
    catalog.reserve_capacity(&request.reservation).unwrap();
    let other_catalog = Catalog::open(&temp.path().join("other-catalog.db")).unwrap();
    assert!(db
        .bind_capacity_release_proof(
            &other_catalog,
            &request.reservation.reservation_id,
            "proof.bound",
            &fixture_unused_proof(),
        )
        .is_err());
    let raw = rusqlite::Connection::open(temp.path().join("store.db")).unwrap();
    raw.execute(
        "UPDATE routing_capacity_intents SET phase='closed' WHERE reservation_id=?1",
        [&request.reservation.reservation_id],
    )
    .unwrap();
    assert!(db.unresolved_capacity_intent_scopes().is_err());
}

#[test]
fn capacity_intent_never_closes_after_bound_catalog_row_even_if_released() {
    let (_, db, catalog, request) = capacity_intent_fixture();
    db.register_capacity_intent(&catalog, &request).unwrap();
    db.mark_capacity_reserve_may_have_started(
        &request.reservation.reservation_id,
        "call.may.start",
    )
    .unwrap();
    catalog.reserve_capacity(&request.reservation).unwrap();
    catalog
        .bind_capacity_reservation(&CapacityBindRequest {
            reservation_id: request.reservation.reservation_id.clone(),
            attempt_id: request.reservation.attempt_id.clone(),
            launch_token: "never-consumed-token".into(),
            bound_at_ms: 103,
        })
        .unwrap();
    db.cancel_routing_mission(&request.scope, "cancel-before-proof", 0, 104)
        .unwrap();
    assert!(db
        .bind_capacity_release_proof(
            &catalog,
            &request.reservation.reservation_id,
            "proof.bound",
            &fixture_unused_proof(),
        )
        .is_err());
    assert!(catalog
        .release_capacity_reservation(&CapacityReleaseRequest {
            reservation_id: request.reservation.reservation_id.clone(),
            evidence: fixture_unused_proof(),
        })
        .is_err());
    catalog
        .release_bound_capacity_reservation(&CapacityBoundReleaseRequest {
            reservation_id: request.reservation.reservation_id.clone(),
            domain_id: request.scope.domain_id.0.clone(),
            run_id: request.scope.run_id.0.clone(),
            attempt_id: request.reservation.attempt_id.clone(),
            owner: request.reservation.owner.clone(),
            launch_token: "never-consumed-token".into(),
            evidence: fixture_unused_proof(),
        })
        .unwrap();
    assert!(db
        .close_released_capacity_intent(&catalog, &request.reservation.reservation_id, "close")
        .is_err());
    assert_eq!(
        db.unresolved_capacity_intent_scopes().unwrap(),
        vec![request.scope]
    );
}

#[test]
fn completed_passed_winner_does_not_release_unsettled_host_capacity() {
    let (temp, db, _, facts, scope) = setup(1);
    let catalog = Catalog::open(&temp.path().join("catalog.db")).unwrap();
    catalog
        .configure_capacity_pool(&CapacityPoolConfig {
            resource_id: "fixture-slot".into(),
            capacity_units: 1,
            expected_revision: None,
            configured_at_ms: 100,
        })
        .unwrap();
    let intent = RoutingCapacityIntentRequest {
        scope: scope.clone(),
        task_id: TaskId("task0".into()),
        reservation: CapacityReservationRequest {
            reservation_id: "winner-capacity".into(),
            domain_id: scope.domain_id.0.clone(),
            run_id: scope.run_id.0.clone(),
            attempt_id: "attempt-task0".into(),
            owner: CapacityOwner {
                process_id: 41,
                process_start_identity: "winner-fixture-owner".into(),
            },
            resources: vec![CapacityResourceRequest {
                resource_id: "fixture-slot".into(),
                units: 1,
            }],
            requested_at_ms: 101,
        },
        event_id: "winner-capacity-created".into(),
    };
    db.register_capacity_intent(&catalog, &intent).unwrap();
    db.mark_capacity_reserve_may_have_started("winner-capacity", "winner-reserve-started")
        .unwrap();
    catalog.reserve_capacity(&intent.reservation).unwrap();
    let mut admission = request(&db, &scope, &facts, "task0");
    admission.capacity_reservation = intent.reservation.reservation_id.clone();
    let admitted = db.admit_routing_attempt(&admission).unwrap();
    assert_eq!(admitted.capacity_reservation, "winner-capacity");
    let passed = advance(&db, &scope, &facts, admitted, AttemptState::Passed);
    assert!(passed.ownership_released);
    db.finish_run("run", "completed").unwrap();
    assert!(db
        .unreconciled_registered_routing_scopes()
        .unwrap()
        .is_empty());
    assert_eq!(db.unresolved_capacity_intent_scopes().unwrap(), vec![scope]);
    assert_eq!(
        catalog
            .capacity_pool_status("fixture-slot")
            .unwrap()
            .unwrap()
            .held_units,
        1,
        "domain ownership_released is not host Catalog release"
    );
}

#[test]
fn completed_registration_is_exempt_only_with_released_verified_winner() {
    let (_, db, _, _, scope) = setup(1);
    db.finish_run("run", "completed").unwrap();
    assert_eq!(
        db.unreconciled_registered_routing_scopes().unwrap(),
        vec![scope.clone()],
        "completed label alone cannot hide Ready routed work"
    );

    let (_, db, _, facts, scope) = setup(1);
    let admitted = db
        .admit_routing_attempt(&request(&db, &scope, &facts, "task0"))
        .unwrap();
    let passed = advance(&db, &scope, &facts, admitted, AttemptState::Passed);
    assert!(passed.ownership_released);
    db.finish_run("run", "completed").unwrap();
    assert!(
        db.unreconciled_registered_routing_scopes()
            .unwrap()
            .is_empty(),
        "a completed released winner must not block later claims"
    );
}

#[test]
fn registered_recovery_scan_rejects_malformed_private_rows() {
    for mutation in [
        "UPDATE routing_missions SET registration_json='{' WHERE run_id='run'",
        "UPDATE routing_missions SET registration_digest='bad' WHERE run_id='run'",
        "UPDATE routing_missions SET domain_id='wrong-domain' WHERE run_id='run'",
        "UPDATE routing_tasks SET record_json='{' WHERE run_id='run'",
        "UPDATE routing_tasks SET task_id='wrong-task' WHERE run_id='run'",
        "DELETE FROM routing_tasks WHERE run_id='run'",
        "UPDATE runs SET status=X'FF' WHERE id='run'",
    ] {
        let (temp, db, _, _, _) = setup(1);
        let conn = rusqlite::Connection::open(temp.path().join("store.db")).unwrap();
        conn.execute_batch(mutation).unwrap();
        assert!(
            db.unreconciled_registered_routing_scopes().is_err(),
            "scan accepted malformed row after {mutation}"
        );
    }
}

#[test]
fn completed_winner_projection_drift_fails_closed() {
    for drift in ["missing_attempt", "wrong_output", "wrong_check_receipt"] {
        let (temp, db, _, facts, scope) = setup(1);
        let admitted = db
            .admit_routing_attempt(&request(&db, &scope, &facts, "task0"))
            .unwrap();
        advance(&db, &scope, &facts, admitted, AttemptState::Passed);
        db.finish_run("run", "completed").unwrap();
        let mut task = db.routing_history(&scope).unwrap().unwrap().tasks.remove(0);
        let winner = task.winner.as_mut().unwrap();
        match drift {
            "missing_attempt" => winner.winning_attempt_id = AttemptId("unknown".into()),
            "wrong_output" => winner.output_digest = digest("different-output"),
            "wrong_check_receipt" => winner.verification_receipt_digest = digest("different-check"),
            _ => unreachable!(),
        }
        let conn = rusqlite::Connection::open(temp.path().join("store.db")).unwrap();
        conn.execute(
            "UPDATE routing_tasks SET record_json=?1 WHERE run_id='run' AND task_id='task0'",
            [serde_json::to_string(&task).unwrap()],
        )
        .unwrap();
        assert!(
            db.unreconciled_registered_routing_scopes().is_err(),
            "scan exempted {drift}"
        );
    }
}

#[test]
fn completed_winner_does_not_hide_nonterminal_attempt_with_forged_release() {
    let (temp, db, _, facts, scope) = setup(1);
    let admitted = db
        .admit_routing_attempt(&request(&db, &scope, &facts, "task0"))
        .unwrap();
    let passed = advance(&db, &scope, &facts, admitted, AttemptState::Passed);
    db.finish_run("run", "completed").unwrap();
    let mut extra = passed;
    extra.attempt_id = AttemptId("forged-extra".into());
    extra.agent_id = "forged-extra-agent".into();
    extra.capacity_reservation = "forged-extra-capacity".into();
    extra.ordinal = 2;
    extra.state = AttemptState::Running;
    extra.ownership_released = true;
    let conn = rusqlite::Connection::open(temp.path().join("store.db")).unwrap();
    conn.execute(
        "INSERT INTO routing_attempts (attempt_id,agent_id,capacity_reservation,run_id,task_id,ordinal,revision,record_json) VALUES (?1,?2,?3,?4,?5,?6,?7,?8)",
        rusqlite::params![
            &extra.attempt_id.0,
            &extra.agent_id,
            &extra.capacity_reservation,
            &scope.run_id.0,
            &extra.task_id.0,
            extra.ordinal,
            extra.revision,
            serde_json::to_string(&extra).unwrap(),
        ],
    )
    .unwrap();
    assert!(
        db.unreconciled_registered_routing_scopes().is_err(),
        "a nonterminal attempt cannot lose recovery ownership through a release flag"
    );
}

#[test]
fn completed_winner_exemption_rejects_sql_projection_drift() {
    for mutation in [
        "UPDATE routing_missions SET cancel_epoch=cancel_epoch+1 WHERE run_id='run'",
        "UPDATE routing_tasks SET revision=revision+1 WHERE run_id='run'",
        "UPDATE routing_attempts SET agent_id='different-agent' WHERE run_id='run'",
        "UPDATE routing_attempts SET capacity_reservation='different-reservation' WHERE run_id='run'",
        "UPDATE routing_attempts SET revision=revision+1 WHERE run_id='run'",
    ] {
        let (temp, db, _, facts, scope) = setup(1);
        let admitted = db
            .admit_routing_attempt(&request(&db, &scope, &facts, "task0"))
            .unwrap();
        advance(&db, &scope, &facts, admitted, AttemptState::Passed);
        db.finish_run("run", "completed").unwrap();
        let conn = rusqlite::Connection::open(temp.path().join("store.db")).unwrap();
        conn.execute_batch(mutation).unwrap();
        assert!(
            db.unreconciled_registered_routing_scopes().is_err(),
            "scan exempted SQL/JSON drift after {mutation}"
        );
    }
}

#[test]
fn registered_recovery_scan_preserves_nonzero_initial_cancel_epoch() {
    let temp = tempfile::tempdir().unwrap();
    let db = PytxoStore::open(&temp.path().join("store.db")).unwrap();
    let (mut mission, _, scope) = registration(1);
    mission.authorization.cancel_epoch = 7;
    db.insert_run(&scope.run_id.0, "repo").unwrap();
    db.register_routing_mission(&mission).unwrap();
    assert_eq!(
        db.unreconciled_registered_routing_scopes().unwrap(),
        vec![scope]
    );
}

#[test]
fn immutable_registration_reopens_and_does_not_synthesize_legacy_authority() {
    let (temp, db, m, _, scope) = setup(1);
    db.register_routing_mission(&m).unwrap();
    let mut changed = m.clone();
    changed.tasks[0].attempt_budget_nano_usd += 1;
    assert!(db.register_routing_mission(&changed).is_err());
    let original = db.routing_history(&scope).unwrap().unwrap();
    drop(db);
    let db = PytxoStore::open(&temp.path().join("store.db")).unwrap();
    assert_eq!(db.routing_history(&scope).unwrap().unwrap(), original);
    db.insert_run("legacy", "repo").unwrap();
    assert!(db
        .routing_history(&RoutingScope {
            domain_id: scope.domain_id,
            run_id: RunId("legacy".into())
        })
        .unwrap()
        .is_none());
}

#[test]
fn routing_revision_is_scoped_durable_and_changes_only_with_committed_events() {
    let temp = tempfile::tempdir().unwrap();
    let db = PytxoStore::open(&temp.path().join("store.db")).unwrap();
    let (mission, _, scope) = registration(1);
    db.insert_run(&scope.run_id.0, "repo").unwrap();
    assert_eq!(db.routing_revision(&scope).unwrap(), None);
    db.register_routing_mission(&mission).unwrap();
    let registered = db.routing_revision(&scope).unwrap().unwrap();
    assert_eq!(
        registered,
        db.routing_history(&scope)
            .unwrap()
            .unwrap()
            .routing_revision
    );
    db.register_routing_mission(&mission).unwrap();
    assert_eq!(db.routing_revision(&scope).unwrap(), Some(registered));
    assert!(db
        .routing_revision(&RoutingScope {
            domain_id: DomainId("another-domain".into()),
            run_id: scope.run_id.clone(),
        })
        .is_err());
    assert!(db
        .cancel_routing_mission(
            &scope,
            "desktop-revision-stale-stop",
            mission.authorization.cancel_epoch + 1,
            151,
        )
        .is_err());
    assert_eq!(db.routing_revision(&scope).unwrap(), Some(registered));
    db.cancel_routing_mission(
        &scope,
        "desktop-revision-stop",
        mission.authorization.cancel_epoch,
        151,
    )
    .unwrap();
    let cancelled = db.routing_revision(&scope).unwrap().unwrap();
    assert!(cancelled > registered);
    db.cancel_routing_mission(
        &scope,
        "desktop-revision-stop",
        mission.authorization.cancel_epoch,
        151,
    )
    .unwrap();
    assert_eq!(db.routing_revision(&scope).unwrap(), Some(cancelled));
    drop(db);
    let reopened = PytxoStore::open(&temp.path().join("store.db")).unwrap();
    assert_eq!(reopened.routing_revision(&scope).unwrap(), Some(cancelled));
    reopened.insert_run("legacy-revision", "repo").unwrap();
    assert_eq!(
        reopened
            .routing_revision(&RoutingScope {
                domain_id: scope.domain_id,
                run_id: RunId("legacy-revision".into()),
            })
            .unwrap(),
        None
    );
}

#[test]
fn routing_display_summary_is_scoped_and_omits_private_mission_bytes() {
    let temp = tempfile::tempdir().unwrap();
    let db = PytxoStore::open(&temp.path().join("store.db")).unwrap();
    let (mut mission, _, scope) = registration(1);
    mission.tasks[0].contract.goal = "PRIVATE_ROUTING_GOAL_SENTINEL".into();
    mission.authorization.allowed_task_digests = mission
        .tasks
        .iter()
        .map(|task| task.contract.digest().unwrap())
        .collect();
    mission.profiles[0].binding.credential_reference = Some("PRIVATE_CREDENTIAL_SENTINEL".into());
    mission.authorization.allowed_profiles[0].binding_digest =
        mission.profiles[0].binding.digest().unwrap();
    db.insert_run(&scope.run_id.0, "repo").unwrap();
    assert!(db.routing_display_summary(&scope).unwrap().is_none());
    db.register_routing_mission(&mission).unwrap();
    let summary = db.routing_display_summary(&scope).unwrap().unwrap();
    assert_eq!(
        summary.routing_revision,
        db.routing_revision(&scope).unwrap().unwrap().to_string()
    );
    assert_eq!(summary.tasks.len(), 1);
    assert_eq!(summary.tasks[0].state, TaskRoutingState::Ready);
    assert!(summary.tasks[0].attempts.is_empty());
    let serialized = serde_json::to_string(&summary).unwrap();
    for private in [
        "PRIVATE_ROUTING_GOAL_SENTINEL",
        "PRIVATE_CREDENTIAL_SENTINEL",
        "credential_reference",
        "auth_owner",
        "endpoint_identity",
        "advice_json",
        "registration_json",
    ] {
        assert!(!serialized.contains(private), "summary leaked {private}");
    }
    assert!(db
        .routing_display_summary(&RoutingScope {
            domain_id: DomainId("another-domain".into()),
            run_id: scope.run_id,
        })
        .is_err());
}

#[test]
fn registration_requires_two_distinct_reviewed_routing_roles() {
    let temp = tempfile::tempdir().unwrap();
    let db = PytxoStore::open(&temp.path().join("store.db")).unwrap();
    let (mut mission, _, scope) = registration(1);
    db.insert_run(&scope.run_id.0, "repo").unwrap();
    mission.policy.strong = mission.policy.everyday.clone();
    mission.authorization.policy_digest = mission.policy.digest().unwrap();
    assert!(db.register_routing_mission(&mission).is_err());

    let (mut mission, _, _) = registration(1);
    mission.tasks[0].contract.required_target = Some(RouteTarget {
        profile_id: ProfileId("third".into()),
        binding_id: BindingId("third".into()),
    });
    mission.authorization.allowed_task_digests = mission
        .tasks
        .iter()
        .map(|task| task.contract.digest().unwrap())
        .collect();
    assert!(db.register_routing_mission(&mission).is_err());
}

#[test]
fn routing_display_summary_tracks_durable_attempt_and_rejects_forged_scope() {
    let (temp, db, _, facts, scope) = setup(1);
    let admitted = db
        .admit_routing_attempt(&request(&db, &scope, &facts, "task0"))
        .unwrap();
    let summary = db.routing_display_summary(&scope).unwrap().unwrap();
    let task = &summary.tasks[0];
    assert_eq!(task.state, TaskRoutingState::Active);
    assert_eq!(
        task.current_attempt_id.as_deref(),
        Some(admitted.attempt_id.0.as_str())
    );
    assert_eq!(task.attempts.len(), 1);
    assert_eq!(task.attempts[0].ordinal, 1);
    assert_eq!(task.attempts[0].state, AttemptState::Admitted);
    assert_eq!(
        task.attempts[0].usage_status,
        RoutingDisplayUsageStatus::Unreported
    );
    assert!(!task.attempts[0].checks_receipt_recorded);
    assert!(task.last_decision.is_some());

    let conn = rusqlite::Connection::open(temp.path().join("store.db")).unwrap();
    let mut forged = serde_json::to_value(&admitted).unwrap();
    forged["scope"]["domain_id"] = serde_json::json!("another-domain");
    conn.execute(
        "UPDATE routing_attempts SET record_json=?1 WHERE attempt_id=?2",
        rusqlite::params![forged.to_string(), admitted.attempt_id.0],
    )
    .unwrap();
    assert!(db.routing_display_summary(&scope).is_err());
}

#[test]
fn routing_display_summary_rejects_sql_json_identity_drift() {
    let (temp, db, _, _, scope) = setup(2);
    let conn = rusqlite::Connection::open(temp.path().join("store.db")).unwrap();
    let first_task_json: String = conn
        .query_row(
            "SELECT record_json FROM routing_tasks WHERE run_id=?1 AND task_id='task0'",
            [&scope.run_id.0],
            |row| row.get(0),
        )
        .unwrap();
    conn.execute(
        "UPDATE routing_tasks SET record_json=?1 WHERE run_id=?2 AND task_id='task1'",
        rusqlite::params![first_task_json, scope.run_id.0],
    )
    .unwrap();
    assert!(db.routing_display_summary(&scope).is_err());

    // A second independent store proves that a matching JSON row still cannot
    // be attributed to a changed SQL agent identity.
    let (other_temp, other_db, _, other_facts, other_scope) = setup(1);
    let admitted = other_db
        .admit_routing_attempt(&request(&other_db, &other_scope, &other_facts, "task0"))
        .unwrap();
    let other_conn = rusqlite::Connection::open(other_temp.path().join("store.db")).unwrap();
    other_conn
        .execute(
            "UPDATE routing_attempts SET agent_id='forged-agent' WHERE attempt_id=?1",
            [&admitted.attempt_id.0],
        )
        .unwrap();
    assert!(other_db.routing_display_summary(&other_scope).is_err());
}

#[test]
fn routing_display_summary_rejects_success_without_a_winner_or_receipts() {
    let (temp, db, _, _, scope) = setup(1);
    let conn = rusqlite::Connection::open(temp.path().join("store.db")).unwrap();
    let mut task =
        serde_json::to_value(&db.routing_history(&scope).unwrap().unwrap().tasks[0]).unwrap();
    task["state"] = serde_json::json!("succeeded");
    conn.execute(
        "UPDATE routing_tasks SET record_json=?1 WHERE run_id=?2 AND task_id='task0'",
        rusqlite::params![task.to_string(), scope.run_id.0],
    )
    .unwrap();
    assert!(db.routing_display_summary(&scope).is_err());

    let (other_temp, other_db, _, facts, other_scope) = setup(1);
    let admitted = other_db
        .admit_routing_attempt(&request(&other_db, &other_scope, &facts, "task0"))
        .unwrap();
    let other_conn = rusqlite::Connection::open(other_temp.path().join("store.db")).unwrap();
    let mut forged = serde_json::to_value(&admitted).unwrap();
    forged["state"] = serde_json::json!("passed");
    forged["ownership_released"] = serde_json::json!(true);
    other_conn
        .execute(
            "UPDATE routing_attempts SET record_json=?1 WHERE attempt_id=?2",
            rusqlite::params![forged.to_string(), admitted.attempt_id.0],
        )
        .unwrap();
    assert!(other_db.routing_display_summary(&other_scope).is_err());
}

#[test]
fn routing_display_summary_rejects_attempt_decision_drift() {
    let (temp, db, mission, facts, scope) = setup(1);
    let admitted = db
        .admit_routing_attempt(&request(&db, &scope, &facts, "task0"))
        .unwrap();
    let mut forged = serde_json::to_value(&admitted).unwrap();
    forged["decision"]["selection"] =
        serde_json::to_value(RouteSelection::Selected(mission.policy.strong)).unwrap();
    let conn = rusqlite::Connection::open(temp.path().join("store.db")).unwrap();
    conn.execute(
        "UPDATE routing_attempts SET record_json=?1 WHERE attempt_id=?2",
        rusqlite::params![forged.to_string(), admitted.attempt_id.0],
    )
    .unwrap();
    assert!(db.routing_display_summary(&scope).is_err());
}

#[test]
fn private_stage_survives_reopen_before_run_row_and_exposes_only_review_identity() {
    let temp = tempfile::tempdir().unwrap();
    let path = temp.path().join("store.db");
    let db = PytxoStore::open(&path).unwrap();
    let (mut mission, _, scope) = registration(1);
    mission.profiles[0].binding.credential_reference = Some("vault://fixture-key".into());
    mission.authorization.allowed_profiles[0].binding_digest =
        mission.profiles[0].binding.digest().unwrap();

    let reviewed = db.stage_routing_mission("draft-one", &mission).unwrap();
    assert_eq!(reviewed.domain_id, scope.domain_id);
    assert_eq!(reviewed.run_id, scope.run_id);
    assert_eq!(reviewed.draft_id, "draft-one");
    assert_eq!(reviewed.plan_digest, mission.authorization.plan_digest);
    assert_eq!(
        reviewed.mission_digest,
        canonical_digest(&mission, 1).unwrap()
    );
    assert!(db.routing_history(&scope).unwrap().is_none());
    let public_json = serde_json::to_string(&reviewed).unwrap();
    assert!(!public_json.contains("vault://fixture-key"));
    assert!(!public_json.contains("Format one file"));
    drop(db);

    let conn = rusqlite::Connection::open(&path).unwrap();
    assert_eq!(
        conn.query_row("SELECT COUNT(*) FROM runs", [], |row| row.get::<_, i64>(0))
            .unwrap(),
        0
    );
    drop(conn);
    let reopened = PytxoStore::open(&path).unwrap();
    assert_eq!(
        reopened.load_staged_routing_mission(&reviewed).unwrap(),
        mission
    );
    assert!(reopened.routing_history(&scope).unwrap().is_none());
}

#[test]
fn private_stage_replay_requires_identical_draft_run_and_mission_bytes() {
    let temp = tempfile::tempdir().unwrap();
    let db = PytxoStore::open(&temp.path().join("store.db")).unwrap();
    let (mission, _, _) = registration(1);
    let reviewed = db.stage_routing_mission("draft-one", &mission).unwrap();
    assert_eq!(
        db.stage_routing_mission("draft-one", &mission).unwrap(),
        reviewed
    );

    let mut changed = mission.clone();
    changed.tasks[0].attempt_budget_nano_usd += 1;
    assert!(db.stage_routing_mission("draft-one", &changed).is_err());
    assert!(db.stage_routing_mission("draft-two", &mission).is_err());
    assert_eq!(db.load_staged_routing_mission(&reviewed).unwrap(), mission);
}

#[test]
fn private_stage_requires_review_before_run_row_but_allows_exact_late_replay() {
    let temp = tempfile::tempdir().unwrap();
    let db = PytxoStore::open(&temp.path().join("store.db")).unwrap();
    let (mission, _, _) = registration(1);
    db.insert_run(&mission.authorization.run_id.0, "repo")
        .unwrap();
    assert!(db.stage_routing_mission("late-draft", &mission).is_err());

    let mut staged = mission.clone();
    staged.authorization.run_id = RunId("reviewed-run".into());
    let reviewed = db.stage_routing_mission("reviewed-draft", &staged).unwrap();
    db.insert_run(&reviewed.run_id.0, "repo").unwrap();
    assert_eq!(
        db.stage_routing_mission("reviewed-draft", &staged).unwrap(),
        reviewed
    );
    db.register_routing_mission(&staged).unwrap();
}

#[test]
fn private_stage_load_rejects_each_stale_review_identity_and_corrupt_payload() {
    let temp = tempfile::tempdir().unwrap();
    let path = temp.path().join("store.db");
    let db = PytxoStore::open(&path).unwrap();
    let (mission, _, _) = registration(1);
    let reviewed = db.stage_routing_mission("draft-one", &mission).unwrap();

    let mut stale = reviewed.clone();
    stale.domain_id = DomainId("other-domain".into());
    assert!(db.load_staged_routing_mission(&stale).is_err());
    stale = reviewed.clone();
    stale.run_id = RunId("other-run".into());
    assert!(db.load_staged_routing_mission(&stale).is_err());
    stale = reviewed.clone();
    stale.draft_id = "other-draft".into();
    assert!(db.load_staged_routing_mission(&stale).is_err());
    stale = reviewed.clone();
    stale.plan_digest = digest("other-plan");
    assert!(db.load_staged_routing_mission(&stale).is_err());
    stale = reviewed.clone();
    stale.mission_digest = digest("other-mission");
    assert!(db.load_staged_routing_mission(&stale).is_err());

    let mut replacement = mission.clone();
    replacement.authorization.run_id = RunId("replacement-run".into());
    let replacement_review = db.stage_routing_mission("draft-one", &replacement).unwrap();
    let mut spliced = replacement_review.clone();
    spliced.run_id = reviewed.run_id.clone();
    assert!(db.load_staged_routing_mission(&spliced).is_err());
    assert_eq!(db.load_staged_routing_mission(&reviewed).unwrap(), mission);

    drop(db);
    let conn = rusqlite::Connection::open(&path).unwrap();
    let mut corrupted = mission.clone();
    corrupted.tasks[0].attempt_budget_nano_usd += 1;
    conn.execute(
        "UPDATE routing_mission_stages SET mission_json=?1 WHERE run_id=?2",
        rusqlite::params![
            serde_json::to_string(&corrupted).unwrap(),
            &reviewed.run_id.0
        ],
    )
    .unwrap();
    drop(conn);
    let reopened = PytxoStore::open(&path).unwrap();
    assert!(reopened.load_staged_routing_mission(&reviewed).is_err());
    assert_eq!(
        reopened
            .load_staged_routing_mission(&replacement_review)
            .unwrap(),
        replacement
    );
}

#[test]
fn private_stage_refuses_invalid_mission_before_persisting() {
    let temp = tempfile::tempdir().unwrap();
    let db = PytxoStore::open(&temp.path().join("store.db")).unwrap();
    let (mut mission, _, _) = registration(1);
    mission.tasks.push(mission.tasks[0].clone());
    assert!(db.stage_routing_mission("draft-one", &mission).is_err());
    let conn = rusqlite::Connection::open(temp.path().join("store.db")).unwrap();
    assert_eq!(
        conn.query_row("SELECT COUNT(*) FROM routing_mission_stages", [], |row| row
            .get::<_, i64>(0))
            .unwrap(),
        0
    );
}
#[test]
fn duplicate_admission_and_event_replay_are_atomic_across_connections() {
    let (temp, db, _, f, s) = setup(1);
    let r = request(&db, &s, &f, "task0");
    let other = PytxoStore::open(&temp.path().join("store.db")).unwrap();
    let a = db.admit_routing_attempt(&r).unwrap();
    assert_eq!(other.admit_routing_attempt(&r).unwrap(), a);
    let mut conflict = r.clone();
    conflict.input_manifest.digest = digest("different");
    assert!(other.admit_routing_attempt(&conflict).is_err());
    conflict = r.clone();
    conflict.event_id = "another-event".into();
    conflict.attempt_id = AttemptId("another-attempt".into());
    conflict.agent_id = "another-agent".into();
    assert!(other.admit_routing_attempt(&conflict).is_err());
    let h = other.routing_history(&s).unwrap().unwrap();
    assert_eq!(h.attempts.len(), 1);
    assert_eq!(h.tasks[0].next_ordinal, 2);
    assert_eq!(h.tasks[0].registration.contract.revision, 1);
    assert_eq!(h.tasks[0].revision, 2);
}
#[test]
fn concurrent_tasks_cannot_oversubscribe_reserved_spend() {
    let (temp, db, _, f, s) = setup(2);
    let r1 = request(&db, &s, &f, "task0");
    let r2 = request(&db, &s, &f, "task1");
    let path = temp.path().join("store.db");
    let barrier = std::sync::Arc::new(std::sync::Barrier::new(2));
    let handles: Vec<_> = [r1, r2]
        .into_iter()
        .map(|r| {
            let path = path.clone();
            let b = barrier.clone();
            std::thread::spawn(move || {
                let db = PytxoStore::open(&path).unwrap();
                b.wait();
                db.admit_routing_attempt(&r)
            })
        })
        .collect();
    assert_eq!(
        handles
            .into_iter()
            .filter(|h| h.thread().id() != std::thread::current().id())
            .map(|h| h.join().unwrap().is_ok() as usize)
            .sum::<usize>(),
        1
    );
    let h = db.routing_history(&s).unwrap().unwrap();
    assert_eq!(h.attempts.len(), 1);
    let loser = if h.attempts[0].task_id.0 == "task0" {
        "task1"
    } else {
        "task0"
    };
    assert!(db
        .admit_routing_attempt(&request(&db, &s, &f, loser))
        .is_err());
    let blocked = db
        .preview_routing_decision(&s, &TaskId(loser.into()), &f, None)
        .unwrap();
    assert_eq!(
        blocked.selection,
        RouteSelection::Blocked(RouteBlocker::BudgetExhausted)
    );
    db.record_routing_block(&RecordRoutingBlock {
        scope: s,
        event_id: "budget-block".into(),
        task_id: TaskId(loser.into()),
        facts: f,
        decision: blocked,
        advice_json: None,
        observation_event_id: None,
    })
    .unwrap();
}
#[test]
fn untrusted_qualification_stale_facts_and_scope_drift_do_not_admit() {
    let (temp, db, m, mut f, s) = setup(1);
    let mut fake = f.observations[0].qualification.clone().unwrap();
    fake.receipt_digest = digest("untrusted");
    f.observations[0].qualification = Some(fake);
    assert!(db
        .admit_routing_attempt(&request(&db, &s, &f, "task0"))
        .is_err());
    f.observations = registration(1).1.observations;
    f.now_ms = 201;
    assert!(db
        .admit_routing_attempt(&request(&db, &s, &f, "task0"))
        .is_err());
    f.now_ms = 150;
    f.base.snapshot_digest = digest("changed source");
    assert!(db
        .admit_routing_attempt(&request(&db, &s, &f, "task0"))
        .is_err());
    assert_eq!(db.routing_history(&s).unwrap().unwrap().attempts.len(), 0);
    drop((temp, m));
}
#[test]
fn cancellation_is_durable_and_prevents_late_winner_or_launch() {
    let (temp, db, _, f, s) = setup(1);
    let a = db
        .admit_routing_attempt(&request(&db, &s, &f, "task0"))
        .unwrap();
    let a = advance(&db, &s, &f, a, AttemptState::Verifying);
    assert_eq!(db.cancel_routing_mission(&s, "stop", 0, 151).unwrap(), 1);
    assert_eq!(db.cancel_routing_mission(&s, "stop", 0, 151).unwrap(), 1);
    assert!(step(
        &db,
        &s,
        &f,
        &a,
        AttemptState::Passed,
        RoutingReceipts {
            checks: Some(digest("checks")),
            ..Default::default()
        }
    )
    .is_err());
    let h = db.routing_history(&s).unwrap().unwrap();
    assert!(h.tasks[0].winner.is_none());
    assert!(!h.attempts[0].ownership_released);
    drop(db);
    let db = PytxoStore::open(&temp.path().join("store.db")).unwrap();
    assert_eq!(db.unresolved_routing_attempts().unwrap().len(), 1);
    let a = step(
        &db,
        &s,
        &f,
        &a,
        AttemptState::Cancelled,
        RoutingReceipts::default(),
    )
    .unwrap();
    assert!(a.ownership_released);
    assert!(db.unresolved_routing_attempts().unwrap().is_empty());
}
#[test]
fn launch_uncertainty_keeps_ownership_until_reconciliation() {
    let (temp, db, _, f, s) = setup(1);
    let a = db
        .admit_routing_attempt(&request(&db, &s, &f, "task0"))
        .unwrap();
    let a = advance(&db, &s, &f, a, AttemptState::Launching);
    let a = step(
        &db,
        &s,
        &f,
        &a,
        AttemptState::RecoveryRequired,
        RoutingReceipts::default(),
    )
    .unwrap();
    drop(db);
    let db = PytxoStore::open(&temp.path().join("store.db")).unwrap();
    assert_eq!(db.unresolved_routing_attempts().unwrap(), vec![a.clone()]);
    assert!(step(
        &db,
        &s,
        &f,
        &a,
        AttemptState::Launching,
        RoutingReceipts::default()
    )
    .is_err());
    assert!(step(
        &db,
        &s,
        &f,
        &a,
        AttemptState::FailedNoLaunch,
        RoutingReceipts {
            no_worker_created: Some(digest("none")),
            ..Default::default()
        }
    )
    .is_err());
    let a = step(
        &db,
        &s,
        &f,
        &a,
        AttemptState::FailedNoLaunch,
        RoutingReceipts {
            no_worker_created: Some(digest("none")),
            reconciliation: Some(digest("reconciled")),
            ..Default::default()
        },
    )
    .unwrap();
    assert!(a.ownership_released);
    assert!(step(
        &db,
        &s,
        &f,
        &a,
        AttemptState::Preparing,
        RoutingReceipts {
            inputs: Some(digest("inputs")),
            ..Default::default()
        }
    )
    .is_err());
}
#[test]
fn passed_winner_is_immutable_and_binds_dependency_outputs() {
    let temp = tempfile::tempdir().unwrap();
    let db = PytxoStore::open(&temp.path().join("db")).unwrap();
    db.insert_run("run", "repo").unwrap();
    let (mut m, f, s) = registration(2);
    m.tasks[1].contract.dependencies = vec![TaskId("task0".into())];
    m.authorization.allowed_task_digests = m
        .tasks
        .iter()
        .map(|t| t.contract.digest().unwrap())
        .collect();
    m.authorization.limits.max_spend_nano_usd = Some(200);
    install(&db, &m, &f, &s);
    assert!(db
        .admit_routing_attempt(&request(&db, &s, &f, "task1"))
        .is_err());
    let a = db
        .admit_routing_attempt(&request(&db, &s, &f, "task0"))
        .unwrap();
    let a = advance(&db, &s, &f, a, AttemptState::Passed);
    let child = db
        .admit_routing_attempt(&request(&db, &s, &f, "task1"))
        .unwrap();
    assert_eq!(child.dependencies[0].winning_attempt_id, a.attempt_id);
    assert_eq!(child.dependencies[0].output_digest, digest("output"));
    assert!(step(
        &db,
        &s,
        &f,
        &a,
        AttemptState::Failed,
        RoutingReceipts::default()
    )
    .is_err());
    assert!(db
        .admit_routing_attempt(&request(&db, &s, &f, "task0"))
        .is_err());
}

#[test]
fn dependency_snapshot_rejects_a_winner_borrowed_from_another_task() {
    let temp = tempfile::tempdir().unwrap();
    let db = PytxoStore::open(&temp.path().join("store.db")).unwrap();
    db.insert_run("run", "repo").unwrap();
    let (mut mission, facts, scope) = registration(3);
    mission.tasks[2].contract.dependencies = vec![TaskId("task0".into())];
    mission.authorization.allowed_task_digests = mission
        .tasks
        .iter()
        .map(|task| task.contract.digest().unwrap())
        .collect();
    mission.authorization.limits.max_spend_nano_usd = Some(200);
    install(&db, &mission, &facts, &scope);
    for task in ["task0", "task1"] {
        let admitted = db
            .admit_routing_attempt(&request(&db, &scope, &facts, task))
            .unwrap();
        advance(&db, &scope, &facts, admitted, AttemptState::Passed);
    }
    let history = db.routing_history(&scope).unwrap().unwrap();
    let child_request = request(&db, &scope, &facts, "task2");
    let mut prerequisite = history.tasks[0].clone();
    prerequisite.winner = history.tasks[1].winner.clone();
    let conn = rusqlite::Connection::open(temp.path().join("store.db")).unwrap();
    conn.execute(
        "UPDATE routing_tasks SET record_json=?1 WHERE run_id='run' AND task_id='task0'",
        [serde_json::to_string(&prerequisite).unwrap()],
    )
    .unwrap();
    assert!(db
        .routing_snapshot(&scope, &TaskId("task2".into()), &facts)
        .is_err());
    assert!(db.admit_routing_attempt(&child_request).is_err());
}
#[test]
fn unknown_cost_is_not_zero_and_settlement_requires_terminal_evidence() {
    let (_, db, _, f, s) = setup(2);
    let a = db
        .admit_routing_attempt(&request(&db, &s, &f, "task0"))
        .unwrap();
    assert!(db
        .settle_routing_usage(
            &s,
            "settle-early",
            &a.attempt_id,
            a.revision,
            &RoutedUsage::Known {
                nano_usd: 0,
                receipt: digest("cost")
            },
            150
        )
        .is_err());
    let a = step(
        &db,
        &s,
        &f,
        &a,
        AttemptState::FailedNoLaunch,
        RoutingReceipts {
            no_worker_created: Some(digest("none")),
            ..Default::default()
        },
    )
    .unwrap();
    let a = db
        .settle_routing_usage(
            &s,
            "unknown",
            &a.attempt_id,
            a.revision,
            &RoutedUsage::Unknown {
                reason: "missing metering".into(),
            },
            151,
        )
        .unwrap();
    assert!(db
        .routing_snapshot(&s, &TaskId("task1".into()), &f)
        .unwrap()
        .spent_and_reserved_nano_usd
        .is_none());
    assert!(db
        .admit_routing_attempt(&request(&db, &s, &f, "task1"))
        .is_err());
    db.settle_routing_usage(
        &s,
        "known",
        &a.attempt_id,
        a.revision,
        &RoutedUsage::Known {
            nano_usd: 10,
            receipt: digest("cost"),
        },
        152,
    )
    .unwrap();
    db.admit_routing_attempt(&request(&db, &s, &f, "task1"))
        .unwrap();
}
#[test]
fn journal_is_ordered_atomic_and_notifies_domain_cache() {
    let (temp, db, _, f, s) = setup(1);
    let h = db.routing_history(&s).unwrap().unwrap();
    db.admit_routing_attempt(&request(&db, &s, &f, "task0"))
        .unwrap();
    let after = db.routing_history(&s).unwrap().unwrap();
    assert!(after.routing_revision > h.routing_revision);
    assert!(after
        .events
        .windows(2)
        .all(|w| w[0].sequence < w[1].sequence));
    let sql = rusqlite::Connection::open(temp.path().join("store.db")).unwrap();
    let notifications: i64 = sql
        .query_row(
            "SELECT COUNT(*) FROM domain_changes WHERE entity_kind='routing'",
            [],
            |r| r.get(0),
        )
        .unwrap();
    assert_eq!(notifications as usize, after.events.len());
}

#[test]
fn selected_decision_is_observed_before_admission_without_granting_authority() {
    let (temp, db, _, facts, scope) = setup(1);
    let task_id = TaskId("task0".into());
    let advice_json = "PRIVATE_PACKET_SENTINEL".to_string();
    let decision = db
        .preview_routing_decision(&scope, &task_id, &facts, Some(&advice_json))
        .unwrap();
    assert!(matches!(decision.selection, RouteSelection::Selected(_)));
    let before = db.routing_history(&scope).unwrap().unwrap();
    let empty_trace = db
        .routing_benchmark_trace(&scope, &task_id, &[7; 32])
        .unwrap()
        .unwrap();
    assert!(!empty_trace.decision_links_complete);
    assert!(empty_trace.events.is_empty());
    let observation_request = ObserveRoutingDecision {
        scope: scope.clone(),
        event_id: "selected-before-capacity".into(),
        task_id: task_id.clone(),
        expected_cancel_epoch: before.cancel_epoch,
        expected_next_ordinal: 1,
        facts: facts.clone(),
        decision: decision.clone(),
        advice_json: Some(advice_json.clone()),
    };
    let observed = db.observe_routing_decision(&observation_request).unwrap();
    assert_eq!(observed.task_id, task_id);
    assert_eq!(observed.decision, decision);
    assert_eq!(observed.cancel_epoch, before.cancel_epoch);
    assert_eq!(
        db.observe_routing_decision(&observation_request).unwrap(),
        observed
    );
    let after = db.routing_history(&scope).unwrap().unwrap();
    assert_eq!(after.routing_revision, before.routing_revision + 1);
    assert_eq!(after.tasks, before.tasks);
    assert!(after.attempts.is_empty());
    assert!(matches!(
        &after.events.last().unwrap().event,
        RoutingControlEvent::DecisionObserved(recorded) if **recorded == observed
    ));
    assert!(!serde_json::to_string(&after.events)
        .unwrap()
        .contains(&advice_json));
    let display_before_admission = db.routing_display_summary(&scope).unwrap().unwrap();
    assert!(display_before_admission.tasks[0].pre_admission.is_some());
    assert!(!serde_json::to_string(&display_before_admission)
        .unwrap()
        .contains(&advice_json));
    let selected_trace = db
        .routing_benchmark_trace(&scope, &task_id, &[7; 32])
        .unwrap()
        .unwrap();
    assert!(!selected_trace.decision_links_complete);
    assert!(matches!(
        selected_trace.events[0].kind,
        RoutingBenchmarkEventKind::DecisionObserved { .. }
    ));
    assert!(!serde_json::to_string(&selected_trace)
        .unwrap()
        .contains(&advice_json));
    let private_scope_digest = canonical_digest(&scope, 1).unwrap();
    let private_task_digest = canonical_digest(&task_id, 1).unwrap();
    let serialized = serde_json::to_string(&selected_trace).unwrap();
    assert!(!serialized.contains(&private_scope_digest.0));
    assert!(!serialized.contains(&private_task_digest.0));
    assert!(!serialized.contains(&observed.advice_digest.as_ref().unwrap().0));
    let other_assignment = db
        .routing_benchmark_trace(&scope, &task_id, &[8; 32])
        .unwrap()
        .unwrap();
    assert_ne!(selected_trace.scope_digest, other_assignment.scope_digest);
    assert_ne!(selected_trace.events, other_assignment.events);

    let mut changed = observation_request.clone();
    changed.facts.now_ms += 1;
    assert!(db.observe_routing_decision(&changed).is_err());
    drop(db);
    let db = PytxoStore::open(&temp.path().join("store.db")).unwrap();
    assert_eq!(
        db.observe_routing_decision(&observation_request).unwrap(),
        observed
    );
    assert_eq!(
        db.routing_history(&scope).unwrap().unwrap().events.len(),
        after.events.len()
    );
    let mut unavailable_facts = facts.clone();
    for profile in &mut unavailable_facts.observations {
        profile.capacity_ready = false;
    }
    let mut failed_admission = request(&db, &scope, &facts, "task0");
    failed_admission.event_id = "admit-after-capacity-loss".into();
    failed_admission.facts = unavailable_facts;
    failed_admission.observation_event_id = Some("selected-before-capacity".into());
    failed_admission.advice_json = Some(advice_json.clone());
    assert!(db.admit_routing_attempt(&failed_admission).is_err());
    assert!(db
        .routing_history(&scope)
        .unwrap()
        .unwrap()
        .attempts
        .is_empty());
    assert!(db
        .routing_history(&scope)
        .unwrap()
        .unwrap()
        .events
        .iter()
        .any(|event| matches!(event.event, RoutingControlEvent::DecisionObserved(_))));
    assert!(db
        .admit_routing_attempt(&request(&db, &scope, &facts, "task0"))
        .is_err());
    let mut admission = request(&db, &scope, &facts, "task0");
    admission.observation_event_id = Some("selected-before-capacity".into());
    admission.advice_json = Some(advice_json.clone());
    db.admit_routing_attempt(&admission).unwrap();
    let admitted_trace = db
        .routing_benchmark_trace(&scope, &task_id, &[7; 32])
        .unwrap()
        .unwrap();
    assert!(admitted_trace.decision_links_complete);
    assert_eq!(admitted_trace.attempts.len(), 1);
    assert!(matches!(
        admitted_trace.attempts[0].usage,
        RoutingBenchmarkUsage::Unreported
    ));
    assert!(admitted_trace.events.iter().any(|event| matches!(
        event.kind,
        RoutingBenchmarkEventKind::DecisionResolved {
            outcome: RoutingBenchmarkResolution::Admitted { .. },
            ..
        }
    )));
    let public = serde_json::to_string(&admitted_trace).unwrap();
    assert!(!public.contains(&advice_json));
    assert!(!public.contains("agent-task0"));
    assert!(!public.contains("\"accepted\""));
    let display_after_admission = db.routing_display_summary(&scope).unwrap().unwrap();
    assert!(display_after_admission.tasks[0].pre_admission.is_none());
    let mut later = observation_request;
    later.event_id = "selected-after-admission".into();
    assert!(db.observe_routing_decision(&later).is_err());
}

#[test]
fn benchmark_trace_keeps_a_capacity_block_without_private_mission_or_fake_attempt() {
    let temp = tempfile::tempdir().unwrap();
    let db = PytxoStore::open(&temp.path().join("store.db")).unwrap();
    db.insert_run("run", "repo").unwrap();
    let (mut mission, facts, scope) = registration(1);
    mission.tasks[0].contract.goal = "PRIVATE_BENCHMARK_GOAL_SENTINEL".into();
    mission.authorization.allowed_task_digests =
        BTreeSet::from([mission.tasks[0].contract.digest().unwrap()]);
    install(&db, &mission, &facts, &scope);
    let task_id = TaskId("task0".into());
    let decision = db
        .preview_routing_decision(&scope, &task_id, &facts, None)
        .unwrap();
    db.observe_routing_decision(&ObserveRoutingDecision {
        scope: scope.clone(),
        event_id: "benchmark-capacity-observed".into(),
        task_id: task_id.clone(),
        expected_cancel_epoch: 0,
        expected_next_ordinal: 1,
        facts: facts.clone(),
        decision,
        advice_json: None,
    })
    .unwrap();
    db.close_unadmitted_routing_observation(&CloseUnadmittedRoutingObservation {
        scope: scope.clone(),
        observation_event_id: "benchmark-capacity-observed".into(),
        stage: PreAdmissionStop::CapacityUnavailable,
        controller_evidence_digest: Some(digest("capacity-evidence")),
        now_ms: facts.now_ms,
    })
    .unwrap();
    let trace = db
        .routing_benchmark_trace(&scope, &task_id, &[7; 32])
        .unwrap()
        .unwrap();
    assert_eq!(trace.schema_version, 3);
    let expected_alias = |raw: &Digest| {
        let mut bytes = b"pytxo-benchmark-alias-v1\0".to_vec();
        bytes.extend_from_slice(&[7; 32]);
        bytes.extend_from_slice(&6_u32.to_le_bytes());
        bytes.extend_from_slice(b"digest");
        bytes.extend_from_slice(raw.0.as_bytes());
        Digest::of_bytes(&bytes)
    };
    let role = |target: &RouteTarget| {
        mission
            .profiles
            .iter()
            .find(|registered| {
                registered.profile.id == target.profile_id
                    && registered.binding.id == target.binding_id
            })
            .unwrap()
    };
    let everyday = role(&mission.policy.everyday);
    let strong = role(&mission.policy.strong);
    let pair = canonical_digest(
        &[
            everyday.profile.digest().unwrap().0,
            everyday.binding.digest().unwrap().0,
            strong.profile.digest().unwrap().0,
            strong.binding.digest().unwrap().0,
        ],
        1,
    )
    .unwrap();
    assert_eq!(
        trace.run_pins.task_contract_digest,
        expected_alias(&mission.tasks[0].contract.digest().unwrap())
    );
    assert_eq!(
        trace.run_pins.snapshot_digest,
        expected_alias(&mission.tasks[0].contract.base.snapshot_digest)
    );
    assert_eq!(trace.run_pins.profile_pair_digest, expected_alias(&pair));
    assert_eq!(
        trace.run_pins.policy_digest,
        expected_alias(&mission.policy.digest().unwrap())
    );
    assert_eq!(
        trace.run_pins.adapter_digest,
        expected_alias(&everyday.profile.adapter_digest)
    );
    assert!(trace.decision_links_complete);
    assert!(trace.attempts.is_empty());
    assert!(trace.events.iter().any(|event| matches!(
        event.kind,
        RoutingBenchmarkEventKind::DecisionResolved {
            outcome: RoutingBenchmarkResolution::NotAdmitted {
                stage: PreAdmissionStop::CapacityUnavailable
            },
            ..
        }
    )));
    let exported = serde_json::to_string(&trace).unwrap();
    for private in [
        "PRIVATE_BENCHMARK_GOAL_SENTINEL",
        "task0",
        "repo",
        "\"run\"",
    ] {
        assert!(!exported.contains(private), "trace leaked {private}");
    }
    assert!(!exported.contains("\"accepted\""));
    assert!(db
        .routing_benchmark_trace(
            &RoutingScope {
                domain_id: DomainId("other-domain".into()),
                run_id: scope.run_id.clone(),
            },
            &task_id,
            &[7; 32],
        )
        .is_err());
    let mut projection = db.routing_history(&scope).unwrap().unwrap().tasks[0].clone();
    projection.registration.contract.goal = "forged projection goal".into();
    rusqlite::Connection::open(temp.path().join("store.db"))
        .unwrap()
        .execute(
            "UPDATE routing_tasks SET record_json=?1 WHERE run_id=?2 AND task_id=?3",
            rusqlite::params![
                serde_json::to_string(&projection).unwrap(),
                scope.run_id.0,
                task_id.0,
            ],
        )
        .unwrap();
    assert!(db
        .routing_benchmark_trace(&scope, &task_id, &[7; 32])
        .is_err());
}

#[test]
fn benchmark_trace_rejects_a_changed_journal_payload_digest() {
    let (temp, db, _, facts, scope) = setup(1);
    let task_id = TaskId("task0".into());
    let decision = db
        .preview_routing_decision(&scope, &task_id, &facts, None)
        .unwrap();
    db.observe_routing_decision(&ObserveRoutingDecision {
        scope: scope.clone(),
        event_id: "benchmark-digest-observation".into(),
        task_id: task_id.clone(),
        expected_cancel_epoch: 0,
        expected_next_ordinal: 1,
        facts: facts.clone(),
        decision,
        advice_json: None,
    })
    .unwrap();
    let conn = rusqlite::Connection::open(temp.path().join("store.db")).unwrap();
    let original_digest: String = conn
        .query_row(
            "SELECT payload_digest FROM routing_control_events WHERE event_id='benchmark-digest-observation'",
            [],
            |row| row.get(0),
        )
        .unwrap();
    conn.execute(
        "UPDATE routing_control_events SET payload_digest=?1 WHERE event_id='benchmark-digest-observation'",
        [digest("different-payload").0],
    )
    .unwrap();
    assert!(db
        .routing_benchmark_trace(&scope, &task_id, &[7; 32])
        .is_err());
    conn.execute(
        "UPDATE routing_control_events SET payload_digest=?1,result_json='{}' WHERE event_id='benchmark-digest-observation'",
        [original_digest],
    )
    .unwrap();
    assert!(db
        .routing_benchmark_trace(&scope, &task_id, &[7; 32])
        .is_err());
}

#[test]
fn benchmark_trace_rejects_admission_resolved_as_not_admitted() {
    let (temp, db, _, facts, scope) = setup(1);
    let task_id = TaskId("task0".into());
    let decision = db
        .preview_routing_decision(&scope, &task_id, &facts, None)
        .unwrap();
    db.observe_routing_decision(&ObserveRoutingDecision {
        scope: scope.clone(),
        event_id: "benchmark-resolution-observation".into(),
        task_id: task_id.clone(),
        expected_cancel_epoch: 0,
        expected_next_ordinal: 1,
        facts: facts.clone(),
        decision,
        advice_json: None,
    })
    .unwrap();
    let mut admission = request(&db, &scope, &facts, "task0");
    admission.observation_event_id = Some("benchmark-resolution-observation".into());
    db.admit_routing_attempt(&admission).unwrap();
    let conn = rusqlite::Connection::open(temp.path().join("store.db")).unwrap();
    let mut stmt = conn
        .prepare("SELECT event_id,event_json FROM routing_control_events WHERE run_id='run'")
        .unwrap();
    let rows = stmt
        .query_map([], |row| {
            Ok((row.get::<_, String>(0)?, row.get::<_, String>(1)?))
        })
        .unwrap();
    let (event_id, mut resolution) = rows
        .map(|row| row.unwrap())
        .find_map(|(id, json)| {
            let event: RoutingControlEvent = serde_json::from_str(&json).unwrap();
            matches!(event, RoutingControlEvent::DecisionResolved(_)).then_some((id, event))
        })
        .unwrap();
    if let RoutingControlEvent::DecisionResolved(ref mut value) = resolution {
        value.outcome = RoutingObservationOutcome::NotAdmitted {
            stage: PreAdmissionStop::Unknown,
            controller_evidence_digest: None,
        };
    }
    conn.execute(
        "UPDATE routing_control_events SET event_json=?1,payload_digest=?2 WHERE event_id=?3",
        rusqlite::params![
            serde_json::to_string(&resolution).unwrap(),
            canonical_digest(&resolution, 1).unwrap().0,
            event_id
        ],
    )
    .unwrap();
    assert!(db
        .routing_benchmark_trace(&scope, &task_id, &[7; 32])
        .is_err());
}

#[test]
fn benchmark_trace_rejects_an_admission_after_its_resolution() {
    let (temp, db, _, facts, scope) = setup(1);
    let task_id = TaskId("task0".into());
    let decision = db
        .preview_routing_decision(&scope, &task_id, &facts, None)
        .unwrap();
    db.observe_routing_decision(&ObserveRoutingDecision {
        scope: scope.clone(),
        event_id: "benchmark-order-observation".into(),
        task_id: task_id.clone(),
        expected_cancel_epoch: 0,
        expected_next_ordinal: 1,
        facts: facts.clone(),
        decision,
        advice_json: None,
    })
    .unwrap();
    let mut admission = request(&db, &scope, &facts, "task0");
    admission.observation_event_id = Some("benchmark-order-observation".into());
    db.admit_routing_attempt(&admission).unwrap();
    let conn = rusqlite::Connection::open(temp.path().join("store.db")).unwrap();
    conn.execute(
        "UPDATE routing_control_events SET sequence=sequence+100 WHERE event_id=?1",
        [&admission.event_id],
    )
    .unwrap();
    assert!(db
        .routing_benchmark_trace(&scope, &task_id, &[7; 32])
        .is_err());
}

#[test]
fn benchmark_trace_rejects_changed_observed_route_or_ordinal() {
    for drift in ["role", "ordinal"] {
        let (temp, db, _, facts, scope) = setup(1);
        let task_id = TaskId("task0".into());
        let decision = db
            .preview_routing_decision(&scope, &task_id, &facts, None)
            .unwrap();
        db.observe_routing_decision(&ObserveRoutingDecision {
            scope: scope.clone(),
            event_id: "benchmark-route-observation".into(),
            task_id: task_id.clone(),
            expected_cancel_epoch: 0,
            expected_next_ordinal: 1,
            facts: facts.clone(),
            decision,
            advice_json: None,
        })
        .unwrap();
        let mut admission = request(&db, &scope, &facts, "task0");
        admission.observation_event_id = Some("benchmark-route-observation".into());
        db.admit_routing_attempt(&admission).unwrap();
        assert!(
            db.routing_benchmark_trace(&scope, &task_id, &[7; 32])
                .unwrap()
                .unwrap()
                .decision_links_complete
        );
        let conn = rusqlite::Connection::open(temp.path().join("store.db")).unwrap();
        let json: String = conn
            .query_row(
                "SELECT event_json FROM routing_control_events WHERE event_id='benchmark-route-observation'",
                [],
                |row| row.get(0),
            )
            .unwrap();
        let mut event: RoutingControlEvent = serde_json::from_str(&json).unwrap();
        let RoutingControlEvent::DecisionObserved(ref mut observed) = event else {
            unreachable!();
        };
        match drift {
            "ordinal" => observed.next_ordinal += 1,
            "role" => {
                let policy = db.routing_history(&scope).unwrap().unwrap().mission.policy;
                let RouteSelection::Selected(ref target) = observed.decision.selection else {
                    unreachable!();
                };
                observed.decision.selection =
                    RouteSelection::Selected(if *target == policy.everyday {
                        policy.strong
                    } else {
                        policy.everyday
                    });
            }
            _ => unreachable!(),
        }
        conn.execute(
            "UPDATE routing_control_events SET event_json=?1,payload_digest=?2,result_json=?3 WHERE event_id='benchmark-route-observation'",
            rusqlite::params![
                serde_json::to_string(&event).unwrap(),
                canonical_digest(&event, 1).unwrap().0,
                match &event {
                    RoutingControlEvent::DecisionObserved(value) => {
                        serde_json::to_string(value).unwrap()
                    }
                    _ => unreachable!(),
                },
            ],
        )
        .unwrap();
        assert!(
            db.routing_benchmark_trace(&scope, &task_id, &[7; 32])
                .is_err(),
            "trace accepted changed {drift}"
        );
    }
}

#[test]
fn benchmark_trace_rejects_selected_profile_drift_from_the_admitted_decision() {
    let (temp, db, _, facts, scope) = setup(1);
    let task_id = TaskId("task0".into());
    let decision = db
        .preview_routing_decision(&scope, &task_id, &facts, None)
        .unwrap();
    db.observe_routing_decision(&ObserveRoutingDecision {
        scope: scope.clone(),
        event_id: "benchmark-profile-observation".into(),
        task_id: task_id.clone(),
        expected_cancel_epoch: 0,
        expected_next_ordinal: 1,
        facts: facts.clone(),
        decision,
        advice_json: None,
    })
    .unwrap();
    let mut admission = request(&db, &scope, &facts, "task0");
    admission.observation_event_id = Some("benchmark-profile-observation".into());
    db.admit_routing_attempt(&admission).unwrap();
    let history = db.routing_history(&scope).unwrap().unwrap();
    let mut attempt = history.attempts[0].clone();
    let other = history
        .mission
        .profiles
        .iter()
        .find(|profile| profile.profile.id != attempt.selected.profile.id)
        .unwrap();
    attempt.selected.profile = other.profile.clone();
    attempt.selected.binding = other.binding.clone();
    let conn = rusqlite::Connection::open(temp.path().join("store.db")).unwrap();
    conn.execute(
        "UPDATE routing_attempts SET record_json=?1 WHERE attempt_id=?2",
        rusqlite::params![
            serde_json::to_string(&attempt).unwrap(),
            attempt.attempt_id.0
        ],
    )
    .unwrap();
    assert!(db
        .routing_benchmark_trace(&scope, &task_id, &[7; 32])
        .is_err());
}

#[test]
fn benchmark_trace_rejects_changed_transition_and_usage_events() {
    let (temp, db, _, facts, scope) = setup(1);
    let task_id = TaskId("task0".into());
    let admitted = db
        .admit_routing_attempt(&request(&db, &scope, &facts, "task0"))
        .unwrap();
    let terminal = step(
        &db,
        &scope,
        &facts,
        &admitted,
        AttemptState::FailedNoLaunch,
        RoutingReceipts {
            no_worker_created: Some(digest("no-worker")),
            ..Default::default()
        },
    )
    .unwrap();
    db.settle_routing_usage(
        &scope,
        "benchmark-usage-settled",
        &terminal.attempt_id,
        terminal.revision,
        &RoutedUsage::Known {
            nano_usd: 0,
            receipt: digest("usage-receipt"),
        },
        facts.now_ms,
    )
    .unwrap();
    let transition_entry = db
        .routing_history(&scope)
        .unwrap()
        .unwrap()
        .events
        .into_iter()
        .find(|entry| {
            matches!(
                entry.event,
                RoutingControlEvent::Transitioned(ref value)
                    if value.to == AttemptState::FailedNoLaunch
            )
        })
        .unwrap();
    let conn = rusqlite::Connection::open(temp.path().join("store.db")).unwrap();
    for (event_id, mut event) in [
        (transition_entry.event_id, transition_entry.event),
        (
            "benchmark-usage-settled".into(),
            RoutingControlEvent::UsageSettled {
                attempt_id: terminal.attempt_id.clone(),
                expected_revision: terminal.revision,
                usage: RoutedUsage::Known {
                    nano_usd: 1,
                    receipt: digest("usage-receipt"),
                },
                now_ms: facts.now_ms,
            },
        ),
    ] {
        if let RoutingControlEvent::Transitioned(value) = &mut event {
            value.to = AttemptState::Passed;
        }
        let (original_json, original_digest): (String, String) = conn
            .query_row(
                "SELECT event_json,payload_digest FROM routing_control_events WHERE event_id=?1",
                [&event_id],
                |row| Ok((row.get(0)?, row.get(1)?)),
            )
            .unwrap();
        conn.execute(
            "UPDATE routing_control_events SET event_json=?1,payload_digest=?2 WHERE event_id=?3",
            rusqlite::params![
                serde_json::to_string(&event).unwrap(),
                canonical_digest(&event, 1).unwrap().0,
                event_id,
            ],
        )
        .unwrap();
        assert!(db
            .routing_benchmark_trace(&scope, &task_id, &[7; 32])
            .is_err());
        conn.execute(
            "UPDATE routing_control_events SET event_json=?1,payload_digest=?2 WHERE event_id=?3",
            rusqlite::params![original_json, original_digest, event_id],
        )
        .unwrap();
    }
}

#[test]
fn malformed_advice_on_unavailable_preferred_profile_is_still_invalid() {
    let temp = tempfile::tempdir().unwrap();
    let db = PytxoStore::open(&temp.path().join("store.db")).unwrap();
    db.insert_run("run", "repo").unwrap();
    let (mut mission, mut facts, scope) = registration(1);
    mission.policy.mode = RoutingMode::Shadow;
    mission.authorization.policy_digest = mission.policy.digest().unwrap();
    mission.tasks[0].contract.task_kind = Some(TaskKind::Other);
    mission.authorization.allowed_task_digests =
        [mission.tasks[0].contract.digest().unwrap()].into();
    facts.observations[1].auth_status = Readiness::Unavailable;
    facts.packet_digest = Some(digest("redacted-packet"));
    facts.advice_request_id = Some(AdviceRequestId("advisor-request".into()));
    install(&db, &mission, &facts, &scope);
    let task_id = TaskId("task0".into());
    let rules = db
        .preview_routing_decision(&scope, &task_id, &facts, None)
        .unwrap();
    assert_eq!(rules.reason, RouteReason::StrongDefault);
    assert_eq!(
        rules.selection,
        RouteSelection::Blocked(RouteBlocker::PreferredUnavailable)
    );
    let malformed = db
        .preview_routing_decision(&scope, &task_id, &facts, Some("{malformed"))
        .unwrap();
    assert_eq!(malformed.selection, rules.selection);
    assert_eq!(malformed.advice_status, AdviceStatus::InvalidOrStale);
}

#[test]
fn advisor_send_journal_requires_workspace_opt_in_and_marks_possible_send_once() {
    let temp = tempfile::tempdir().unwrap();
    let path = temp.path().join("store.db");
    let db = PytxoStore::open(&path).unwrap();
    db.insert_run("run", "repo").unwrap();
    let (mut mission, mut facts, scope) = registration(1);
    mission.policy.mode = RoutingMode::Shadow;
    mission.authorization.policy_digest = mission.policy.digest().unwrap();
    mission.tasks[0].contract.task_kind = Some(TaskKind::Other);
    mission.authorization.allowed_task_digests =
        [mission.tasks[0].contract.digest().unwrap()].into();
    facts.packet_digest = Some(digest("redacted-packet"));
    facts.advice_request_id = Some(AdviceRequestId("advisor-request".into()));
    install(&db, &mission, &facts, &scope);
    let task = TaskId("task0".into());
    assert!(db
        .prepare_routing_advisor_request(&scope, &task, &facts)
        .is_err());
    assert_eq!(
        db.routing_advisor_consent(&scope.domain_id)
            .unwrap()
            .revision,
        0
    );
    let consent = db
        .set_routing_advisor_consent(
            &scope.domain_id,
            0,
            true,
            Some(digest("reviewed-recipient-template-v1")),
            145,
        )
        .unwrap();
    assert_eq!(consent.revision, mission.authorization.consent_revision);
    let prepared = db
        .prepare_routing_advisor_request(&scope, &task, &facts)
        .unwrap();
    assert_eq!(prepared.phase, AdvisorSendPhase::Prepared);
    assert_eq!(
        db.prepare_routing_advisor_request(&scope, &task, &facts)
            .unwrap(),
        prepared
    );
    let mut conflicting = facts.clone();
    conflicting.advice_request_id = Some(AdviceRequestId("different-request".into()));
    assert!(db
        .prepare_routing_advisor_request(&scope, &task, &conflicting)
        .is_err());
    conflicting = facts.clone();
    conflicting.packet_digest = Some(digest("changed-packet"));
    assert!(db
        .prepare_routing_advisor_request(&scope, &task, &conflicting)
        .is_err());
    drop(db);
    let db = PytxoStore::open(&path).unwrap();
    let other = PytxoStore::open(&path).unwrap();
    assert!(db
        .mark_routing_advisor_send_may_have_happened(&scope, &prepared.request_id, &facts, 150)
        .unwrap());
    assert!(!other
        .mark_routing_advisor_send_may_have_happened(&scope, &prepared.request_id, &facts, 150)
        .unwrap());
    assert_eq!(
        other
            .routing_advisor_request(&scope, &prepared.request_id)
            .unwrap()
            .unwrap()
            .phase,
        AdvisorSendPhase::SendingMayHaveHappened
    );
    let mut second_packet = facts.clone();
    second_packet.advice_request_id = Some(AdviceRequestId("second-request".into()));
    second_packet.packet_digest = Some(digest("second-packet"));
    assert!(other
        .prepare_routing_advisor_request(&scope, &task, &second_packet)
        .is_err());
    let completed = other
        .settle_routing_advisor_request(&scope, &prepared.request_id, Some(digest("answer")), 160)
        .unwrap();
    assert_eq!(completed.phase, AdvisorSendPhase::Completed);
    assert_eq!(
        db.settle_routing_advisor_request(
            &scope,
            &prepared.request_id,
            Some(digest("answer")),
            160
        )
        .unwrap(),
        completed
    );
    assert!(db
        .settle_routing_advisor_request(&scope, &prepared.request_id, Some(digest("other")), 160)
        .is_err());
    assert!(!db
        .mark_routing_advisor_send_may_have_happened(&scope, &prepared.request_id, &facts, 170)
        .unwrap());
    let revoked = db
        .set_routing_advisor_consent(&scope.domain_id, consent.revision, false, None, 170)
        .unwrap();
    assert_eq!(revoked.revision, 2);
    assert!(db
        .prepare_routing_advisor_request(&scope, &task, &facts)
        .is_err());
}

#[test]
fn advisor_request_rejects_consent_for_another_recipient_and_template_scope() {
    let temp = tempfile::tempdir().unwrap();
    let db = PytxoStore::open(&temp.path().join("store.db")).unwrap();
    db.insert_run("run", "repo").unwrap();
    let (mut mission, mut facts, scope) = registration(1);
    mission.policy.mode = RoutingMode::Shadow;
    mission.policy.disclosure_scope_digest = Some(digest("reviewed-recipient-template-v1"));
    mission.authorization.policy_digest = mission.policy.digest().unwrap();
    facts.packet_digest = Some(digest("redacted-packet"));
    facts.advice_request_id = Some(AdviceRequestId("advisor-request".into()));
    install(&db, &mission, &facts, &scope);

    let consent = db
        .set_routing_advisor_consent(
            &scope.domain_id,
            0,
            true,
            Some(digest("different-recipient-or-template")),
            145,
        )
        .unwrap();
    assert_eq!(consent.revision, mission.authorization.consent_revision);
    assert!(db
        .prepare_routing_advisor_request(&scope, &TaskId("task0".into()), &facts)
        .is_err());
    assert!(db
        .routing_advisor_request(&scope, &AdviceRequestId("advisor-request".into()))
        .unwrap()
        .is_none());
}

#[test]
fn advisor_revocation_succeeds_after_clock_rollback() {
    let temp = tempfile::tempdir().unwrap();
    let db = PytxoStore::open(&temp.path().join("store.db")).unwrap();
    let domain = DomainId("clock-rollback".into());
    let enabled = db
        .set_routing_advisor_consent(
            &domain,
            0,
            true,
            Some(digest("reviewed-recipient-template-v1")),
            200,
        )
        .unwrap();
    let revoked = db
        .set_routing_advisor_consent(&domain, enabled.revision, false, None, 100)
        .unwrap();
    assert_eq!(revoked.revision, enabled.revision + 1);
    assert!(!revoked.enabled);
    assert_eq!(revoked.scope_digest, None);
    assert_eq!(revoked.updated_at_ms, 200);
    assert_eq!(db.routing_advisor_consent(&domain).unwrap(), revoked);
}

#[test]
fn hosted_advisor_consent_is_recipient_scoped_and_cannot_mutate_fixture_grant() {
    let temp = tempfile::tempdir().unwrap();
    let db = PytxoStore::open(&temp.path().join("store.db")).unwrap();
    let domain = DomainId("hosted-workspace".into());
    let hosted = "pytxo-hosted-routing/typesafe-systemone/v1";
    let other = "pytxo-hosted-routing/other/v1";
    let fixture = db
        .set_routing_advisor_consent(&domain, 0, true, Some(digest("fixture-scope")), 100)
        .unwrap();
    assert_eq!(
        db.routing_hosted_advisor_consent(&domain, hosted)
            .unwrap()
            .revision,
        0
    );

    let enabled = db
        .set_routing_hosted_advisor_consent(
            &domain,
            hosted,
            0,
            true,
            Some(digest("hosted-scope")),
            200,
        )
        .unwrap();
    assert_eq!(enabled.revision, 1);
    assert!(enabled.enabled);
    assert_eq!(db.routing_advisor_consent(&domain).unwrap(), fixture);
    assert_eq!(
        db.routing_hosted_advisor_consent(&domain, other)
            .unwrap()
            .revision,
        0
    );
    assert!(db
        .set_routing_hosted_advisor_consent(&domain, hosted, 0, false, None, 201,)
        .is_err());

    let revoked = db
        .set_routing_hosted_advisor_consent(&domain, hosted, 1, false, None, 100)
        .unwrap();
    assert_eq!(revoked.revision, 2);
    assert!(!revoked.enabled);
    assert_eq!(revoked.updated_at_ms, 200);
    assert_eq!(
        db.routing_hosted_advisor_consent(&domain, hosted).unwrap(),
        revoked
    );
    assert_eq!(db.routing_advisor_consent(&domain).unwrap(), fixture);
    assert!(db
        .set_routing_hosted_advisor_consent(
            &domain,
            hosted,
            1,
            true,
            Some(digest("hosted-scope")),
            201,
        )
        .is_err());
}

#[test]
fn hosted_advisor_request_binds_reviewed_recipient_scope_and_uuid_v7() {
    let temp = tempfile::tempdir().unwrap();
    let path = temp.path().join("store.db");
    let db = PytxoStore::open(&path).unwrap();
    db.insert_run("run", "repo").unwrap();
    let (mut mission, mut facts, scope) = registration(1);
    let recipient = "pytxo-hosted-routing/typesafe-systemone/v1";
    let scope_digest = digest("hosted-scope");
    mission.policy.mode = RoutingMode::Shadow;
    mission.policy.advisor_recipient = Some(recipient.into());
    mission.policy.disclosure_scope_digest = Some(scope_digest.clone());
    mission.authorization.policy_digest = mission.policy.digest().unwrap();
    mission.tasks[0].contract.task_kind = Some(TaskKind::Other);
    mission.authorization.allowed_task_digests =
        [mission.tasks[0].contract.digest().unwrap()].into();
    facts.packet_digest = Some(digest("reviewed-hosted-packet"));
    facts.advice_request_id = Some(AdviceRequestId(
        "00000000-0096-7000-8000-000000000001".into(),
    ));
    install(&db, &mission, &facts, &scope);
    let task = TaskId("task0".into());
    db.set_routing_advisor_consent(&scope.domain_id, 0, true, Some(scope_digest.clone()), 145)
        .unwrap();
    assert!(db
        .prepare_routing_advisor_request(&scope, &task, &facts)
        .is_err());
    assert!(db
        .prepare_routing_hosted_advisor_request(&scope, &task, &facts, recipient)
        .is_err());
    db.set_routing_hosted_advisor_consent(
        &scope.domain_id,
        recipient,
        0,
        true,
        Some(scope_digest.clone()),
        145,
    )
    .unwrap();
    let prepared = db
        .prepare_routing_hosted_advisor_request(&scope, &task, &facts, recipient)
        .unwrap();
    assert_eq!(prepared.recipient_identity.as_deref(), Some(recipient));
    assert_eq!(prepared.scope_digest, Some(scope_digest));
    assert_eq!(prepared.phase, AdvisorSendPhase::Prepared);
    assert_eq!(
        db.prepare_routing_hosted_advisor_request(&scope, &task, &facts, recipient)
            .unwrap(),
        prepared
    );
    let mut second_request = facts.clone();
    second_request.advice_request_id = Some(AdviceRequestId(
        "00000000-0096-7000-8000-000000000002".into(),
    ));
    assert!(db
        .prepare_routing_hosted_advisor_request(&scope, &task, &second_request, recipient)
        .is_err());
    assert!(db
        .routing_advisor_request(&scope, second_request.advice_request_id.as_ref().unwrap())
        .unwrap()
        .is_none());
    let mut non_v7 = facts.clone();
    non_v7.advice_request_id = Some(AdviceRequestId("advisor-request".into()));
    assert!(db
        .prepare_routing_hosted_advisor_request(&scope, &task, &non_v7, recipient)
        .is_err());
    drop(db);
    let db = PytxoStore::open(&path).unwrap();
    assert_eq!(
        db.routing_advisor_request(&scope, &prepared.request_id)
            .unwrap()
            .unwrap(),
        prepared
    );
    for wrong_scope in [
        RoutingScope {
            domain_id: scope.domain_id.clone(),
            run_id: RunId("other-run".into()),
        },
        RoutingScope {
            domain_id: DomainId("other-domain".into()),
            run_id: scope.run_id.clone(),
        },
    ] {
        assert!(db
            .mark_routing_advisor_send_may_have_happened(
                &wrong_scope,
                &prepared.request_id,
                &facts,
                150,
            )
            .is_err());
        assert_eq!(
            db.routing_advisor_request(&scope, &prepared.request_id)
                .unwrap()
                .unwrap()
                .phase,
            AdvisorSendPhase::Prepared
        );
    }
    assert!(db
        .mark_routing_advisor_send_may_have_happened(&scope, &prepared.request_id, &facts, 150)
        .unwrap());
    db.set_routing_hosted_advisor_consent(&scope.domain_id, recipient, 1, false, None, 150)
        .unwrap();
    assert!(db
        .prepare_routing_hosted_advisor_request(&scope, &task, &facts, recipient)
        .is_err());
}

#[test]
fn hosted_advisor_never_prepares_unrecordable_shadow_context() {
    let recipient = "pytxo-hosted-routing/typesafe-systemone/v1";
    for unrecordable in ["incomplete", "unknown_kind", "missing_evidence"] {
        let temp = tempfile::tempdir().unwrap();
        let path = temp.path().join("store.db");
        let db = PytxoStore::open(&path).unwrap();
        db.insert_run("run", "repo").unwrap();
        let (mut mission, mut facts, scope) = registration(1);
        let scope_digest = digest("hosted-scope");
        mission.policy.mode = RoutingMode::Shadow;
        mission.policy.advisor_recipient = Some(recipient.into());
        mission.policy.disclosure_scope_digest = Some(scope_digest.clone());
        mission.authorization.policy_digest = mission.policy.digest().unwrap();
        mission.tasks[0].contract.task_kind = Some(TaskKind::Other);
        match unrecordable {
            "incomplete" => mission.tasks[0].contract.context_complete = false,
            "unknown_kind" => mission.tasks[0].contract.task_kind = None,
            "missing_evidence" => mission.tasks[0].contract.task_kind_evidence = None,
            _ => unreachable!(),
        }
        mission.authorization.allowed_task_digests =
            [mission.tasks[0].contract.digest().unwrap()].into();
        facts.packet_digest = Some(digest("reviewed-hosted-packet"));
        facts.advice_request_id = Some(AdviceRequestId(
            "00000000-0096-7000-8000-000000000001".into(),
        ));
        install(&db, &mission, &facts, &scope);
        db.set_routing_hosted_advisor_consent(
            &scope.domain_id,
            recipient,
            0,
            true,
            Some(scope_digest),
            145,
        )
        .unwrap();
        let request_id = facts.advice_request_id.as_ref().unwrap();
        let task = TaskId("task0".into());
        assert!(
            db.prepare_routing_hosted_advisor_request(&scope, &task, &facts, recipient)
                .is_err(),
            "{unrecordable}"
        );
        assert!(
            db.routing_advisor_request(&scope, request_id)
                .unwrap()
                .is_none(),
            "{unrecordable}"
        );
        // An older controller may already have persisted a prepared row.
        // The final one-use mark must still refuse to spend on unusable advice.
        let conn = rusqlite::Connection::open(&path).unwrap();
        conn.execute(
            "INSERT INTO routing_advisor_requests(request_id,domain_id,run_id,task_id,ordinal,policy_digest,packet_digest,consent_revision,task_revision,task_state_revision,authorization_revision,cancel_epoch,phase,result_digest,created_at_ms,updated_at_ms,recipient_identity,scope_digest) VALUES (?1,?2,?3,?4,1,?5,?6,?7,1,1,?8,0,'prepared',NULL,150,150,?9,?10)",
            rusqlite::params![request_id.0,scope.domain_id.0,scope.run_id.0,task.0,mission.authorization.policy_digest.0,facts.packet_digest.as_ref().unwrap().0,mission.authorization.consent_revision,mission.authorization.revision,recipient,mission.policy.disclosure_scope_digest.as_ref().unwrap().0],
        ).unwrap();
        assert!(
            db.mark_routing_advisor_send_may_have_happened(&scope, request_id, &facts, 150)
                .is_err(),
            "{unrecordable}"
        );
        assert_eq!(
            db.routing_advisor_request(&scope, request_id)
                .unwrap()
                .unwrap()
                .phase,
            AdvisorSendPhase::Prepared,
            "{unrecordable}"
        );
    }
}

#[test]
fn hosted_advisor_request_cannot_prepare_a_live_decision() {
    let temp = tempfile::tempdir().unwrap();
    let path = temp.path().join("store.db");
    let db = PytxoStore::open(&path).unwrap();
    db.insert_run("run", "repo").unwrap();
    let (mut mission, mut facts, scope) = registration(1);
    let recipient = "pytxo-hosted-routing/typesafe-systemone/v1";
    let scope_digest = digest("hosted-live-scope");
    mission.policy.mode = RoutingMode::Live;
    mission.policy.advisor_recipient = Some(recipient.into());
    mission.policy.disclosure_scope_digest = Some(scope_digest.clone());
    mission.policy.evaluated_manifest_digest = Some(digest("evaluated-policy"));
    mission.authorization.policy_digest = mission.policy.digest().unwrap();
    mission.authorization.live_advice_authorized = true;
    mission.tasks[0].contract.task_kind = Some(TaskKind::Other);
    mission.authorization.allowed_task_digests =
        [mission.tasks[0].contract.digest().unwrap()].into();
    facts.packet_digest = Some(digest("reviewed-hosted-packet"));
    facts.advice_request_id = Some(AdviceRequestId(
        "00000000-0096-7000-8000-000000000001".into(),
    ));
    install(&db, &mission, &facts, &scope);
    db.set_routing_hosted_advisor_consent(
        &scope.domain_id,
        recipient,
        0,
        true,
        Some(scope_digest.clone()),
        145,
    )
    .unwrap();
    let request_id = facts.advice_request_id.as_ref().unwrap();
    let task = TaskId("task0".into());
    assert!(db
        .prepare_routing_hosted_advisor_request(&scope, &task, &facts, recipient)
        .is_err());
    assert!(db
        .routing_advisor_request(&scope, request_id)
        .unwrap()
        .is_none());
    // Even a forged prepared row from an older controller cannot pass the
    // final hosted send fence for a Live mission.
    let conn = rusqlite::Connection::open(&path).unwrap();
    conn.execute(
        "INSERT INTO routing_advisor_requests(request_id,domain_id,run_id,task_id,ordinal,policy_digest,packet_digest,consent_revision,task_revision,task_state_revision,authorization_revision,cancel_epoch,phase,result_digest,created_at_ms,updated_at_ms,recipient_identity,scope_digest) VALUES (?1,?2,?3,?4,1,?5,?6,?7,1,1,?8,0,'prepared',NULL,150,150,?9,?10)",
        rusqlite::params![request_id.0,scope.domain_id.0,scope.run_id.0,task.0,mission.authorization.policy_digest.0,facts.packet_digest.as_ref().unwrap().0,mission.authorization.consent_revision,mission.authorization.revision,recipient,scope_digest.0],
    ).unwrap();
    assert!(db
        .mark_routing_advisor_send_may_have_happened(&scope, request_id, &facts, 150)
        .is_err());
    assert_eq!(
        db.routing_advisor_request(&scope, request_id)
            .unwrap()
            .unwrap()
            .phase,
        AdvisorSendPhase::Prepared
    );
}

#[test]
fn advisor_never_sends_when_task_reservation_exceeds_remaining_budget() {
    let temp = tempfile::tempdir().unwrap();
    let db = PytxoStore::open(&temp.path().join("store.db")).unwrap();
    db.insert_run("run", "repo").unwrap();
    let (mut mission, mut facts, scope) = registration(2);
    mission.policy.mode = RoutingMode::Shadow;
    mission.authorization.policy_digest = mission.policy.digest().unwrap();
    mission.tasks[1].contract.task_kind = Some(TaskKind::Other);
    mission.authorization.allowed_task_digests = mission
        .tasks
        .iter()
        .map(|task| task.contract.digest().unwrap())
        .collect();
    facts.packet_digest = Some(digest("redacted-packet"));
    facts.advice_request_id = Some(AdviceRequestId("budget-blocked-advisor".into()));
    install(&db, &mission, &facts, &scope);
    db.set_routing_advisor_consent(
        &scope.domain_id,
        0,
        true,
        Some(digest("reviewed-recipient-template-v1")),
        145,
    )
    .unwrap();
    db.admit_routing_attempt(&request(&db, &scope, &facts, "task0"))
        .unwrap();
    let task = TaskId("task1".into());
    assert_eq!(
        db.preview_routing_decision(&scope, &task, &facts, None)
            .unwrap()
            .selection,
        RouteSelection::Blocked(RouteBlocker::BudgetExhausted)
    );
    assert!(db
        .prepare_routing_advisor_request(&scope, &task, &facts)
        .is_err());
    assert!(db
        .routing_advisor_request(&scope, facts.advice_request_id.as_ref().unwrap())
        .unwrap()
        .is_none());
}

#[test]
fn advisor_send_mark_rechecks_revocation_and_cancel_epoch() {
    let temp = tempfile::tempdir().unwrap();
    let db = PytxoStore::open(&temp.path().join("store.db")).unwrap();
    db.insert_run("run", "repo").unwrap();
    let (mut mission, mut facts, scope) = registration(1);
    mission.policy.mode = RoutingMode::Shadow;
    mission.authorization.policy_digest = mission.policy.digest().unwrap();
    mission.tasks[0].contract.task_kind = Some(TaskKind::Other);
    mission.authorization.allowed_task_digests =
        [mission.tasks[0].contract.digest().unwrap()].into();
    facts.packet_digest = Some(digest("redacted-packet"));
    facts.advice_request_id = Some(AdviceRequestId("advisor-request".into()));
    install(&db, &mission, &facts, &scope);
    db.set_routing_advisor_consent(
        &scope.domain_id,
        0,
        true,
        Some(digest("reviewed-recipient-template-v1")),
        145,
    )
    .unwrap();
    let task = TaskId("task0".into());
    let prepared = db
        .prepare_routing_advisor_request(&scope, &task, &facts)
        .unwrap();
    let mut stale = facts.clone();
    stale.now_ms = 500;
    stale.expires_at_ms = 600;
    assert!(db
        .mark_routing_advisor_send_may_have_happened(&scope, &prepared.request_id, &stale, 500)
        .is_err());
    db.set_routing_advisor_consent(&scope.domain_id, 1, false, None, 151)
        .unwrap();
    facts.now_ms = 152;
    assert!(db
        .mark_routing_advisor_send_may_have_happened(&scope, &prepared.request_id, &facts, 152)
        .is_err());
    assert_eq!(
        db.routing_advisor_request(&scope, &prepared.request_id)
            .unwrap()
            .unwrap()
            .phase,
        AdvisorSendPhase::Prepared
    );
    db.cancel_routing_mission(&scope, "advisor-cancel", 0, 153)
        .unwrap();
    facts.now_ms = 154;
    assert!(db
        .mark_routing_advisor_send_may_have_happened(&scope, &prepared.request_id, &facts, 154)
        .is_err());
}

#[test]
fn advisor_request_cannot_predate_workspace_consent() {
    let temp = tempfile::tempdir().unwrap();
    let db = PytxoStore::open(&temp.path().join("store.db")).unwrap();
    db.insert_run("run", "repo").unwrap();
    let (mut mission, mut facts, scope) = registration(1);
    mission.policy.mode = RoutingMode::Shadow;
    mission.authorization.policy_digest = mission.policy.digest().unwrap();
    mission.tasks[0].contract.task_kind = Some(TaskKind::Other);
    mission.authorization.allowed_task_digests =
        [mission.tasks[0].contract.digest().unwrap()].into();
    facts.packet_digest = Some(digest("redacted-packet"));
    facts.advice_request_id = Some(AdviceRequestId("advisor-after-consent".into()));
    install(&db, &mission, &facts, &scope);
    db.set_routing_advisor_consent(
        &scope.domain_id,
        0,
        true,
        Some(digest("reviewed-recipient-template-v1")),
        facts.now_ms + 1,
    )
    .unwrap();
    let task = TaskId("task0".into());
    assert!(db
        .prepare_routing_advisor_request(&scope, &task, &facts)
        .is_err());
    assert!(db
        .routing_advisor_request(&scope, facts.advice_request_id.as_ref().unwrap())
        .unwrap()
        .is_none());
    facts.now_ms += 2;
    let prepared = db
        .prepare_routing_advisor_request(&scope, &task, &facts)
        .unwrap();
    assert!(
        prepared.created_at_ms
            >= db
                .routing_advisor_consent(&scope.domain_id)
                .unwrap()
                .updated_at_ms
    );
    let conn = rusqlite::Connection::open(temp.path().join("store.db")).unwrap();
    conn.execute(
        "UPDATE routing_advisor_consent SET updated_at_ms=?1 WHERE domain_id=?2",
        rusqlite::params![facts.now_ms + 1, scope.domain_id.0],
    )
    .unwrap();
    assert!(db
        .mark_routing_advisor_send_may_have_happened(
            &scope,
            &prepared.request_id,
            &facts,
            facts.now_ms,
        )
        .is_err());
    conn.execute(
        "UPDATE routing_advisor_consent SET updated_at_ms=?1 WHERE domain_id=?2",
        rusqlite::params![facts.now_ms - 1, scope.domain_id.0],
    )
    .unwrap();
    assert!(db
        .mark_routing_advisor_send_may_have_happened(
            &scope,
            &prepared.request_id,
            &facts,
            facts.now_ms,
        )
        .unwrap());
}

#[test]
fn advisor_late_receipt_records_evidence_without_reopening_send() {
    let temp = tempfile::tempdir().unwrap();
    let db = PytxoStore::open(&temp.path().join("store.db")).unwrap();
    db.insert_run("run", "repo").unwrap();
    let (mut mission, mut facts, scope) = registration(1);
    mission.policy.mode = RoutingMode::Shadow;
    mission.authorization.policy_digest = mission.policy.digest().unwrap();
    mission.tasks[0].contract.task_kind = Some(TaskKind::Other);
    mission.authorization.allowed_task_digests =
        [mission.tasks[0].contract.digest().unwrap()].into();
    facts.packet_digest = Some(digest("redacted-packet"));
    facts.advice_request_id = Some(AdviceRequestId("advisor-request".into()));
    install(&db, &mission, &facts, &scope);
    db.set_routing_advisor_consent(
        &scope.domain_id,
        0,
        true,
        Some(digest("reviewed-recipient-template-v1")),
        145,
    )
    .unwrap();
    let prepared = db
        .prepare_routing_advisor_request(&scope, &TaskId("task0".into()), &facts)
        .unwrap();
    assert!(db
        .mark_routing_advisor_send_may_have_happened(&scope, &prepared.request_id, &facts, 150)
        .unwrap());
    let uncertain = db
        .settle_routing_advisor_request(&scope, &prepared.request_id, None, 152)
        .unwrap();
    assert_eq!(uncertain.phase, AdvisorSendPhase::Uncertain);
    let uncertain_trace = db
        .routing_benchmark_trace(&scope, &TaskId("task0".into()), &[7; 32])
        .unwrap()
        .unwrap();
    assert_eq!(uncertain_trace.advisor_requests.len(), 1);
    assert_eq!(
        uncertain_trace.advisor_requests[0].phase,
        AdvisorSendPhase::Uncertain
    );
    assert!(!uncertain_trace.decision_links_complete);
    let late = db
        .settle_routing_advisor_request(&scope, &prepared.request_id, Some(digest("receipt")), 160)
        .unwrap();
    assert_eq!(late.phase, AdvisorSendPhase::LateReceipt);
    assert_eq!(
        db.routing_benchmark_trace(&scope, &TaskId("task0".into()), &[7; 32])
            .unwrap()
            .unwrap()
            .advisor_requests[0]
            .phase,
        AdvisorSendPhase::LateReceipt
    );
    assert_eq!(
        db.settle_routing_advisor_request(
            &scope,
            &prepared.request_id,
            Some(digest("receipt")),
            160
        )
        .unwrap(),
        late
    );
    assert!(!db
        .mark_routing_advisor_send_may_have_happened(&scope, &prepared.request_id, &facts, 170)
        .unwrap());
    assert!(db
        .settle_routing_advisor_request(&scope, &prepared.request_id, Some(digest("changed")), 170)
        .is_err());
}

#[test]
fn jev_shadow_observation_requires_completed_journal_and_current_consent() {
    let temp = tempfile::tempdir().unwrap();
    let db = PytxoStore::open(&temp.path().join("store.db")).unwrap();
    db.insert_run("run", "repo").unwrap();
    let (mut mission, mut facts, scope) = registration(2);
    mission.policy.mode = RoutingMode::Shadow;
    mission.policy.advice_model = "jev-1.13.0".into();
    mission.policy.advice_template = "execution_demand_v1".into();
    mission.authorization.policy_digest = mission.policy.digest().unwrap();
    for registered in &mut mission.tasks {
        registered.contract.task_kind = Some(TaskKind::Other);
    }
    mission.authorization.allowed_task_digests = mission
        .tasks
        .iter()
        .map(|registered| registered.contract.digest().unwrap())
        .collect();
    facts.packet_digest = Some(digest("redacted-packet"));
    facts.advice_request_id = Some(AdviceRequestId("jev-shadow-request".into()));
    install(&db, &mission, &facts, &scope);
    db.set_routing_advisor_consent(
        &scope.domain_id,
        0,
        true,
        Some(digest("reviewed-recipient-template-v1")),
        145,
    )
    .unwrap();
    let task = TaskId("task0".into());
    let snapshot = db.routing_snapshot(&scope, &task, &facts).unwrap();
    let advice = AdviceEnvelope {
        schema_version: 1,
        request_id: facts.advice_request_id.clone().unwrap(),
        packet_digest: facts.packet_digest.clone().unwrap(),
        policy_version: mission.policy.version.clone(),
        template_version: mission.policy.advice_template.clone(),
        model_id: mission.policy.advice_model.clone(),
        task_revision: snapshot.task_revision,
        task_state_revision: snapshot.task_state_revision,
        authorization_revision: snapshot.authorization_revision,
        cancel_epoch: snapshot.cancel_epoch,
        consent_revision: snapshot.authorization.consent_revision,
        received_at_ms: 150,
        expires_at_ms: 180,
        packet_complete: true,
        choice: AdviceChoice::EverydayFit,
        distribution: AdviceDistribution {
            everyday_fit: 0.9,
            strong_needed: 0.08,
            unclear: 0.02,
        },
        usage_status: AdviceUsageStatus::Unknown,
        usage_receipt_id: None,
        elapsed_ms: 1,
    };
    let raw = serde_json::to_string(&advice).unwrap();
    let selected = db
        .preview_routing_decision(&scope, &task, &facts, Some(&raw))
        .unwrap();
    assert_eq!(selected.advice_status, AdviceStatus::ShadowRecorded);
    let observe = |event_id: &str| {
        db.observe_routing_decision(&ObserveRoutingDecision {
            scope: scope.clone(),
            event_id: event_id.into(),
            task_id: task.clone(),
            expected_cancel_epoch: snapshot.cancel_epoch,
            expected_next_ordinal: snapshot.next_ordinal,
            facts: facts.clone(),
            decision: selected.clone(),
            advice_json: Some(raw.clone()),
        })
    };
    assert!(observe("unbacked-jev").is_err());
    let malformed = "{not-a-jev-response";
    let invalid = db
        .preview_routing_decision(&scope, &task, &facts, Some(malformed))
        .unwrap();
    assert_eq!(invalid.advice_status, AdviceStatus::InvalidOrStale);
    assert!(db
        .observe_routing_decision(&ObserveRoutingDecision {
            scope: scope.clone(),
            event_id: "unbacked-malformed-jev".into(),
            task_id: task.clone(),
            expected_cancel_epoch: snapshot.cancel_epoch,
            expected_next_ordinal: snapshot.next_ordinal,
            facts: facts.clone(),
            decision: invalid,
            advice_json: Some(malformed.into()),
        })
        .is_err());
    let prepared = db
        .prepare_routing_advisor_request(&scope, &task, &facts)
        .unwrap();
    assert!(db
        .mark_routing_advisor_send_may_have_happened(&scope, &prepared.request_id, &facts, 150)
        .unwrap());
    assert!(observe("unsettled-jev").is_err());
    db.settle_routing_advisor_request(
        &scope,
        &prepared.request_id,
        Some(Digest::of_bytes(raw.as_bytes())),
        150,
    )
    .unwrap();
    let mut direct = request(&db, &scope, &facts, "task0");
    direct.decision = selected.clone();
    direct.advice_json = Some(raw.clone());
    assert!(db.admit_routing_attempt(&direct).is_err());
    assert_eq!(
        observe("journal-backed-jev").unwrap().shadow_choice,
        Some(AdviceChoice::EverydayFit)
    );
    // A sibling can have its own completed send. Exporting task0 must not
    // require task1's request to appear in task0's assignment-keyed trace.
    let sibling_task = TaskId("task1".into());
    let mut sibling_facts = facts.clone();
    sibling_facts.advice_request_id = Some(AdviceRequestId("jev-shadow-sibling".into()));
    sibling_facts.packet_digest = Some(digest("redacted-sibling-packet"));
    let sibling_snapshot = db
        .routing_snapshot(&scope, &sibling_task, &sibling_facts)
        .unwrap();
    let mut sibling_advice = advice.clone();
    sibling_advice.request_id = sibling_facts.advice_request_id.clone().unwrap();
    sibling_advice.packet_digest = sibling_facts.packet_digest.clone().unwrap();
    sibling_advice.task_revision = sibling_snapshot.task_revision;
    sibling_advice.task_state_revision = sibling_snapshot.task_state_revision;
    let sibling_raw = serde_json::to_string(&sibling_advice).unwrap();
    let sibling_prepared = db
        .prepare_routing_advisor_request(&scope, &sibling_task, &sibling_facts)
        .unwrap();
    assert!(db
        .mark_routing_advisor_send_may_have_happened(
            &scope,
            &sibling_prepared.request_id,
            &sibling_facts,
            150,
        )
        .unwrap());
    db.settle_routing_advisor_request(
        &scope,
        &sibling_prepared.request_id,
        Some(Digest::of_bytes(sibling_raw.as_bytes())),
        150,
    )
    .unwrap();
    let sibling_decision = db
        .preview_routing_decision(&scope, &sibling_task, &sibling_facts, Some(&sibling_raw))
        .unwrap();
    db.observe_routing_decision(&ObserveRoutingDecision {
        scope: scope.clone(),
        event_id: "journal-backed-sibling".into(),
        task_id: sibling_task.clone(),
        expected_cancel_epoch: sibling_snapshot.cancel_epoch,
        expected_next_ordinal: sibling_snapshot.next_ordinal,
        facts: sibling_facts,
        decision: sibling_decision,
        advice_json: Some(sibling_raw),
    })
    .unwrap();
    db.set_routing_advisor_consent(&scope.domain_id, 1, false, None, 151)
        .unwrap();
    assert!(observe("revoked-jev").is_err());
    let mut admitted = request(&db, &scope, &facts, "task0");
    admitted.decision = selected;
    admitted.advice_json = Some(raw);
    admitted.observation_event_id = Some("journal-backed-jev".into());
    assert_eq!(
        db.admit_routing_attempt(&admitted)
            .unwrap()
            .decision
            .advice_status,
        AdviceStatus::ShadowRecorded
    );
    let trace = db
        .routing_benchmark_trace(&scope, &task, &[7; 32])
        .unwrap()
        .unwrap();
    assert_eq!(trace.schema_version, 3);
    assert_eq!(trace.advisor_requests.len(), 1);
    assert_eq!(trace.advisor_requests[0].phase, AdvisorSendPhase::Completed);
    assert!(trace.events.iter().any(|event| matches!(
        &event.kind,
        RoutingBenchmarkEventKind::DecisionObserved {
            advice_request_digest: Some(request),
            advice_digest: Some(result),
            ..
        } if request == &trace.advisor_requests[0].request_digest
            && Some(result) == trace.advisor_requests[0].result_digest.as_ref()
    )));
    let exported = serde_json::to_string(&trace).unwrap();
    assert!(!exported.contains("redacted-packet"));
    assert!(!exported.contains("jev-shadow-request"));
    assert!(!exported.contains("created_at_ms"));
    assert!(!exported.contains("updated_at_ms"));
    assert!(!exported.contains("consent_revision"));
    let other_key_trace = db
        .routing_benchmark_trace(&scope, &task, &[8; 32])
        .unwrap()
        .unwrap();
    assert_ne!(
        trace.advisor_requests[0].request_digest,
        other_key_trace.advisor_requests[0].request_digest
    );
    assert_ne!(
        trace.advisor_requests[0].packet_digest,
        other_key_trace.advisor_requests[0].packet_digest
    );
    let conn = rusqlite::Connection::open(temp.path().join("store.db")).unwrap();
    conn.execute(
        "UPDATE routing_advisor_requests SET phase='uncertain',result_digest=NULL WHERE request_id=?1",
        [&prepared.request_id.0],
    )
    .unwrap();
    assert!(db.routing_benchmark_trace(&scope, &task, &[7; 32]).is_err());
}

#[test]
fn live_jev_applied_observation_requires_journal_and_admission_rechecks_consent() {
    let temp = tempfile::tempdir().unwrap();
    let db = PytxoStore::open(&temp.path().join("store.db")).unwrap();
    db.insert_run("run", "repo").unwrap();
    let (mut mission, mut facts, scope) = registration(1);
    mission.policy.mode = RoutingMode::Live;
    mission.policy.advice_model = "jev-1.13.0".into();
    mission.policy.advice_template = "execution_demand_v1".into();
    mission.policy.evaluated_manifest_digest = Some(digest("evaluated-policy"));
    mission.authorization.policy_digest = mission.policy.digest().unwrap();
    mission.authorization.live_advice_authorized = true;
    mission.tasks[0].contract.task_kind = Some(TaskKind::Diagnosis);
    mission.authorization.allowed_task_digests =
        [mission.tasks[0].contract.digest().unwrap()].into();
    facts.packet_digest = Some(digest("redacted-packet"));
    facts.advice_request_id = Some(AdviceRequestId("jev-live-request".into()));
    install(&db, &mission, &facts, &scope);
    db.set_routing_advisor_consent(
        &scope.domain_id,
        0,
        true,
        Some(digest("reviewed-recipient-template-v1")),
        145,
    )
    .unwrap();
    let task = TaskId("task0".into());
    let snapshot = db.routing_snapshot(&scope, &task, &facts).unwrap();
    let advice = AdviceEnvelope {
        schema_version: 1,
        request_id: facts.advice_request_id.clone().unwrap(),
        packet_digest: facts.packet_digest.clone().unwrap(),
        policy_version: mission.policy.version.clone(),
        template_version: mission.policy.advice_template.clone(),
        model_id: mission.policy.advice_model.clone(),
        task_revision: snapshot.task_revision,
        task_state_revision: snapshot.task_state_revision,
        authorization_revision: snapshot.authorization_revision,
        cancel_epoch: snapshot.cancel_epoch,
        consent_revision: snapshot.authorization.consent_revision,
        received_at_ms: 150,
        expires_at_ms: 180,
        packet_complete: true,
        choice: AdviceChoice::EverydayFit,
        distribution: AdviceDistribution {
            everyday_fit: 0.9,
            strong_needed: 0.08,
            unclear: 0.02,
        },
        usage_status: AdviceUsageStatus::Unknown,
        usage_receipt_id: None,
        elapsed_ms: 1,
    };
    let raw = serde_json::to_string(&advice).unwrap();
    let selected = db
        .preview_routing_decision(&scope, &task, &facts, Some(&raw))
        .unwrap();
    assert_eq!(selected.advice_status, AdviceStatus::Applied);
    assert_eq!(
        selected.selection,
        RouteSelection::Selected(mission.policy.everyday.clone())
    );
    let observe = |event_id: &str| {
        db.observe_routing_decision(&ObserveRoutingDecision {
            scope: scope.clone(),
            event_id: event_id.into(),
            task_id: task.clone(),
            expected_cancel_epoch: snapshot.cancel_epoch,
            expected_next_ordinal: snapshot.next_ordinal,
            facts: facts.clone(),
            decision: selected.clone(),
            advice_json: Some(raw.clone()),
        })
    };
    assert!(observe("unjournaled-live").is_err());
    let prepared = db
        .prepare_routing_advisor_request(&scope, &task, &facts)
        .unwrap();
    assert!(db
        .mark_routing_advisor_send_may_have_happened(&scope, &prepared.request_id, &facts, 150)
        .unwrap());
    db.settle_routing_advisor_request(
        &scope,
        &prepared.request_id,
        Some(Digest::of_bytes(raw.as_bytes())),
        150,
    )
    .unwrap();
    let observed = observe("journaled-live").unwrap();
    assert_eq!(
        observed.advice_request_id,
        Some(prepared.request_id.clone())
    );
    assert_eq!(observed.shadow_choice, None);
    let trace = db
        .routing_benchmark_trace(&scope, &task, &[7; 32])
        .unwrap()
        .unwrap();
    if let Some(path) = std::env::var_os("PYTXO_ROUTING_ADVISOR_BRIDGE_DIR") {
        let directory = std::fs::canonicalize(path).unwrap();
        let temporary_root = std::fs::canonicalize(std::env::temp_dir()).unwrap();
        assert!(directory.starts_with(&temporary_root) && directory != temporary_root);
        assert_eq!(std::fs::read_dir(&directory).unwrap().count(), 0);
        std::fs::write(
            directory.join("advisor-trace.json"),
            serde_json::to_vec(&trace).unwrap(),
        )
        .unwrap();
        std::fs::write(
            directory.join("private-packet.json"),
            serde_json::to_vec(&serde_json::json!({
                "packet_digest": facts.packet_digest,
                "alias_key_hex": "07".repeat(32),
            }))
            .unwrap(),
        )
        .unwrap();
    }
    assert!(trace.events.iter().any(|event| matches!(
        &event.kind,
        RoutingBenchmarkEventKind::DecisionObserved {
            decision,
            advice_request_digest: Some(request),
            advice_digest: Some(result),
            ..
        } if decision.advice_status == AdviceStatus::Applied
            && request == &trace.advisor_requests[0].request_digest
            && Some(result) == trace.advisor_requests[0].result_digest.as_ref()
    )));
    let mut admitted = request(&db, &scope, &facts, "task0");
    admitted.decision = selected.clone();
    admitted.advice_json = Some(raw.clone());
    admitted.observation_event_id = Some("journaled-live".into());
    assert_eq!(
        db.admit_routing_attempt(&admitted)
            .unwrap()
            .decision
            .advice_status,
        AdviceStatus::Applied
    );

    // A separate, otherwise identical run proves revocation between the
    // journaled observation and admission cannot retain Live influence.
    let revoked = PytxoStore::open(&temp.path().join("revoked.db")).unwrap();
    revoked.insert_run("run", "repo").unwrap();
    install(&revoked, &mission, &facts, &scope);
    revoked
        .set_routing_advisor_consent(
            &scope.domain_id,
            0,
            true,
            Some(digest("reviewed-recipient-template-v1")),
            145,
        )
        .unwrap();
    let prepared = revoked
        .prepare_routing_advisor_request(&scope, &task, &facts)
        .unwrap();
    assert!(revoked
        .mark_routing_advisor_send_may_have_happened(&scope, &prepared.request_id, &facts, 150)
        .unwrap());
    revoked
        .settle_routing_advisor_request(
            &scope,
            &prepared.request_id,
            Some(Digest::of_bytes(raw.as_bytes())),
            150,
        )
        .unwrap();
    revoked
        .observe_routing_decision(&ObserveRoutingDecision {
            scope: scope.clone(),
            event_id: "journaled-live".into(),
            task_id: task.clone(),
            expected_cancel_epoch: snapshot.cancel_epoch,
            expected_next_ordinal: snapshot.next_ordinal,
            facts: facts.clone(),
            decision: selected.clone(),
            advice_json: Some(raw.clone()),
        })
        .unwrap();
    revoked
        .set_routing_advisor_consent(&scope.domain_id, 1, false, None, 151)
        .unwrap();
    let mut admission = request(&revoked, &scope, &facts, "task0");
    admission.decision = selected;
    admission.advice_json = Some(raw);
    admission.observation_event_id = Some("journaled-live".into());
    assert!(revoked.admit_routing_attempt(&admission).is_err());
}

#[test]
fn malformed_shadow_advice_is_recorded_as_invalid_rules_fallback() {
    let temp = tempfile::tempdir().unwrap();
    let db = PytxoStore::open(&temp.path().join("store.db")).unwrap();
    db.insert_run("run", "repo").unwrap();
    let (mut mission, mut facts, scope) = registration(1);
    mission.policy.mode = RoutingMode::Shadow;
    mission.authorization.policy_digest = mission.policy.digest().unwrap();
    facts.packet_digest = Some(digest("redacted-packet"));
    facts.advice_request_id = Some(AdviceRequestId("advisor-request".into()));
    install(&db, &mission, &facts, &scope);
    let task_id = TaskId("task0".into());
    let snapshot = db.routing_snapshot(&scope, &task_id, &facts).unwrap();
    let rules = db
        .preview_routing_decision(&scope, &task_id, &facts, None)
        .unwrap();
    let malformed = "{PRIVATE_MALFORMED_ADVICE";
    let fallback = db
        .preview_routing_decision(&scope, &task_id, &facts, Some(malformed))
        .unwrap();
    assert_eq!(fallback.selection, rules.selection);
    assert_eq!(fallback.reason, rules.reason);
    assert_eq!(fallback.advice_status, AdviceStatus::InvalidOrStale);

    let observed = db
        .observe_routing_decision(&ObserveRoutingDecision {
            scope: scope.clone(),
            event_id: "invalid-shadow-response".into(),
            task_id: task_id.clone(),
            expected_cancel_epoch: snapshot.cancel_epoch,
            expected_next_ordinal: snapshot.next_ordinal,
            facts,
            decision: fallback,
            advice_json: Some(malformed.into()),
        })
        .unwrap();
    assert_eq!(observed.shadow_choice, None);
    assert!(observed.advice_digest.is_some());
    let trace = db
        .routing_benchmark_trace(&scope, &task_id, &[7; 32])
        .unwrap()
        .unwrap();
    assert!(trace.events.iter().any(|entry| matches!(
        &entry.kind,
        RoutingBenchmarkEventKind::DecisionObserved { decision, shadow_choice: None, .. }
            if decision.advice_status == AdviceStatus::InvalidOrStale
    )));
    assert!(
        !serde_json::to_string(&db.routing_history(&scope).unwrap().unwrap())
            .unwrap()
            .contains(malformed)
    );
}

#[test]
fn shadow_observation_keeps_bounded_jev_choice_separate_from_rules_route() {
    let temp = tempfile::tempdir().unwrap();
    let db = PytxoStore::open(&temp.path().join("store.db")).unwrap();
    db.insert_run("run", "repo").unwrap();
    let (mut mission, mut facts, scope) = registration(1);
    mission.policy.mode = RoutingMode::Shadow;
    mission.authorization.policy_digest = mission.policy.digest().unwrap();
    facts.packet_digest = Some(digest("redacted-packet"));
    facts.advice_request_id = Some(AdviceRequestId("advisor-request".into()));
    install(&db, &mission, &facts, &scope);
    let task_id = TaskId("task0".into());
    let snapshot = db.routing_snapshot(&scope, &task_id, &facts).unwrap();
    let advice = AdviceEnvelope {
        schema_version: 1,
        request_id: facts.advice_request_id.clone().unwrap(),
        packet_digest: facts.packet_digest.clone().unwrap(),
        policy_version: mission.policy.version.clone(),
        template_version: mission.policy.advice_template.clone(),
        model_id: mission.policy.advice_model.clone(),
        task_revision: snapshot.task_revision,
        task_state_revision: snapshot.task_state_revision,
        authorization_revision: snapshot.authorization_revision,
        cancel_epoch: snapshot.cancel_epoch,
        consent_revision: snapshot.authorization.consent_revision,
        received_at_ms: facts.now_ms - 10,
        expires_at_ms: facts.now_ms + 30,
        packet_complete: true,
        choice: AdviceChoice::EverydayFit,
        distribution: AdviceDistribution {
            everyday_fit: 0.9,
            strong_needed: 0.08,
            unclear: 0.02,
        },
        usage_status: AdviceUsageStatus::Unknown,
        usage_receipt_id: None,
        elapsed_ms: 40,
    };
    let raw_advice = serde_json::to_string(&advice).unwrap();
    let rules = db
        .preview_routing_decision(&scope, &task_id, &facts, None)
        .unwrap();
    let shadow = db
        .preview_routing_decision(&scope, &task_id, &facts, Some(&raw_advice))
        .unwrap();
    assert_eq!(rules.selection, shadow.selection);
    assert_eq!(shadow.advice_status, AdviceStatus::ShadowRecorded);
    let observed = db
        .observe_routing_decision(&ObserveRoutingDecision {
            scope: scope.clone(),
            event_id: "shadow-choice".into(),
            task_id,
            expected_cancel_epoch: snapshot.cancel_epoch,
            expected_next_ordinal: snapshot.next_ordinal,
            facts: facts.clone(),
            decision: shadow,
            advice_json: Some(raw_advice.clone()),
        })
        .unwrap();
    assert_eq!(observed.shadow_choice, Some(AdviceChoice::EverydayFit));
    assert_eq!(observed.advice_request_id, Some(advice.request_id.clone()));
    let event_json =
        serde_json::to_string(&db.routing_history(&scope).unwrap().unwrap().events).unwrap();
    assert!(!event_json.contains(&raw_advice));
    assert!(!event_json.contains("everyday_fit\":0.9"));
    let mut swapped = advice;
    swapped.choice = AdviceChoice::StrongNeeded;
    swapped.distribution = AdviceDistribution {
        everyday_fit: 0.08,
        strong_needed: 0.9,
        unclear: 0.02,
    };
    let swapped_json = serde_json::to_string(&swapped).unwrap();
    assert_eq!(
        db.preview_routing_decision(&scope, &TaskId("task0".into()), &facts, Some(&swapped_json))
            .unwrap(),
        observed.decision
    );
    let mut admission = request(&db, &scope, &facts, "task0");
    admission.decision = observed.decision.clone();
    admission.advice_json = Some(swapped_json);
    admission.observation_event_id = Some("shadow-choice".into());
    assert!(db.admit_routing_attempt(&admission).is_err());
    admission.advice_json = Some(raw_advice);
    db.admit_routing_attempt(&admission).unwrap();
    let original_event = db
        .routing_history(&scope)
        .unwrap()
        .unwrap()
        .events
        .into_iter()
        .find(|entry| entry.event_id == "shadow-choice")
        .unwrap()
        .event;
    let conn = rusqlite::Connection::open(temp.path().join("store.db")).unwrap();
    for drift in ["choice", "request"] {
        let mut event = original_event.clone();
        let RoutingControlEvent::DecisionObserved(ref mut value) = event else {
            unreachable!();
        };
        match drift {
            "choice" => value.shadow_choice = Some(AdviceChoice::StrongNeeded),
            "request" => value.advice_request_id = Some(AdviceRequestId("forged".into())),
            _ => unreachable!(),
        }
        let RoutingControlEvent::DecisionObserved(value) = &event else {
            unreachable!();
        };
        conn.execute(
            "UPDATE routing_control_events SET event_json=?1,payload_digest=?2,result_json=?3 WHERE event_id='shadow-choice'",
            rusqlite::params![
                serde_json::to_string(&event).unwrap(),
                canonical_digest(&event, 1).unwrap().0,
                serde_json::to_string(value).unwrap(),
            ],
        )
        .unwrap();
        assert!(
            db.routing_benchmark_trace(&scope, &TaskId("task0".into()), &[7; 32])
                .is_err(),
            "trace accepted changed shadow {drift}"
        );
    }
}

#[test]
fn observed_selection_links_to_one_atomic_admission_outcome() {
    let (temp, db, _, facts, scope) = setup(1);
    let task_id = TaskId("task0".into());
    let decision = db
        .preview_routing_decision(&scope, &task_id, &facts, None)
        .unwrap();
    let observed = db
        .observe_routing_decision(&ObserveRoutingDecision {
            scope: scope.clone(),
            event_id: "selection-for-admission".into(),
            task_id,
            expected_cancel_epoch: 0,
            expected_next_ordinal: 1,
            facts: facts.clone(),
            decision,
            advice_json: None,
        })
        .unwrap();
    let mut admission = request(&db, &scope, &facts, "task0");
    admission.observation_event_id = Some("selection-for-admission".into());
    let attempt = db.admit_routing_attempt(&admission).unwrap();
    assert_eq!(db.admit_routing_attempt(&admission).unwrap(), attempt);
    let history = db.routing_history(&scope).unwrap().unwrap();
    let resolved: Vec<_> = history
        .events
        .iter()
        .filter_map(|entry| match &entry.event {
            RoutingControlEvent::DecisionResolved(outcome) => Some(outcome.as_ref()),
            _ => None,
        })
        .collect();
    assert_eq!(resolved.len(), 1);
    assert_eq!(resolved[0].observation_event_id, "selection-for-admission");
    assert_eq!(
        resolved[0].outcome,
        RoutingObservationOutcome::Admitted {
            attempt_id: attempt.attempt_id.clone()
        }
    );
    assert_eq!(observed.decision, attempt.decision);
    drop(db);
    let db = PytxoStore::open(&temp.path().join("store.db")).unwrap();
    assert_eq!(db.admit_routing_attempt(&admission).unwrap(), attempt);
    assert!(db
        .close_unadmitted_routing_observation(&CloseUnadmittedRoutingObservation {
            scope: scope.clone(),
            observation_event_id: "selection-for-admission".into(),
            stage: PreAdmissionStop::CapacityUnavailable,
            controller_evidence_digest: None,
            now_ms: facts.now_ms + 1,
        })
        .is_err());
    let conn = rusqlite::Connection::open(temp.path().join("store.db")).unwrap();
    conn.execute(
        "UPDATE routing_control_events SET event_json='{}' WHERE event_id LIKE 'routing-observation-resolved:%'",
        [],
    )
    .unwrap();
    assert!(db.admit_routing_attempt(&admission).is_err());
    assert!(db.routing_display_summary(&scope).is_err());
}

#[test]
fn observed_selection_retains_one_typed_pre_admission_stop() {
    let (temp, db, _, facts, scope) = setup(1);
    let task_id = TaskId("task0".into());
    let decision = db
        .preview_routing_decision(&scope, &task_id, &facts, None)
        .unwrap();
    db.observe_routing_decision(&ObserveRoutingDecision {
        scope: scope.clone(),
        event_id: "selection-without-capacity".into(),
        task_id,
        expected_cancel_epoch: 0,
        expected_next_ordinal: 1,
        facts: facts.clone(),
        decision,
        advice_json: None,
    })
    .unwrap();
    let pending = db.routing_display_summary(&scope).unwrap().unwrap();
    let selected = pending.tasks[0].pre_admission.as_ref().unwrap();
    assert_eq!(selected.ordinal, 1);
    assert!(matches!(
        selected.outcome,
        RoutingDisplayPreAdmissionOutcome::Pending
    ));
    assert!(matches!(
        selected.decision.selection,
        RoutingDisplaySelection::Selected {
            role: RoutingDisplayRole::Everyday
        }
    ));
    assert!(pending.tasks[0].attempts.is_empty());
    let close = CloseUnadmittedRoutingObservation {
        scope: scope.clone(),
        observation_event_id: "selection-without-capacity".into(),
        stage: PreAdmissionStop::CapacityUnavailable,
        controller_evidence_digest: None,
        now_ms: facts.now_ms + 1,
    };
    let resolved = db.close_unadmitted_routing_observation(&close).unwrap();
    assert_eq!(
        db.close_unadmitted_routing_observation(&close).unwrap(),
        resolved
    );
    assert_eq!(
        resolved.outcome,
        RoutingObservationOutcome::NotAdmitted {
            stage: PreAdmissionStop::CapacityUnavailable,
            controller_evidence_digest: None,
        }
    );
    let stopped = db.routing_display_summary(&scope).unwrap().unwrap();
    assert_eq!(
        stopped.tasks[0].pre_admission.as_ref().unwrap().outcome,
        RoutingDisplayPreAdmissionOutcome::NotAdmitted {
            stage: PreAdmissionStop::CapacityUnavailable,
        }
    );
    let mut conflict = close.clone();
    conflict.stage = PreAdmissionStop::Superseded;
    assert!(db.close_unadmitted_routing_observation(&conflict).is_err());
    let mut admission = request(&db, &scope, &facts, "task0");
    admission.observation_event_id = Some(close.observation_event_id);
    assert!(db.admit_routing_attempt(&admission).is_err());
    assert!(db
        .admit_routing_attempt(&request(&db, &scope, &facts, "task0"))
        .is_err());
    drop(db);
    let db = PytxoStore::open(&temp.path().join("store.db")).unwrap();
    let history = db.routing_history(&scope).unwrap().unwrap();
    let reopened_summary = db.routing_display_summary(&scope).unwrap().unwrap();
    assert_eq!(
        reopened_summary.tasks[0].pre_admission,
        stopped.tasks[0].pre_admission
    );
    assert!(history.attempts.is_empty());
    assert_eq!(
        history
            .events
            .iter()
            .filter(|entry| matches!(entry.event, RoutingControlEvent::DecisionResolved(_)))
            .count(),
        1
    );
}

#[test]
fn selected_observation_requires_prior_outcome_before_same_ordinal_reobservation() {
    let (_, db, _, facts, scope) = setup(1);
    let task_id = TaskId("task0".into());
    let decision = db
        .preview_routing_decision(&scope, &task_id, &facts, None)
        .unwrap();
    let first = ObserveRoutingDecision {
        scope: scope.clone(),
        event_id: "first-selection".into(),
        task_id,
        expected_cancel_epoch: 0,
        expected_next_ordinal: 1,
        facts: facts.clone(),
        decision,
        advice_json: None,
    };
    db.observe_routing_decision(&first).unwrap();
    let mut second = first.clone();
    second.event_id = "second-selection".into();
    assert!(db.observe_routing_decision(&second).is_err());
    db.close_unadmitted_routing_observation(&CloseUnadmittedRoutingObservation {
        scope: scope.clone(),
        observation_event_id: first.event_id,
        stage: PreAdmissionStop::CapacityUnavailable,
        controller_evidence_digest: None,
        now_ms: facts.now_ms + 1,
    })
    .unwrap();
    db.observe_routing_decision(&second).unwrap();
    let mut admission = request(&db, &scope, &facts, "task0");
    admission.observation_event_id = Some(second.event_id);
    db.admit_routing_attempt(&admission).unwrap();
    let summary = db.routing_display_summary(&scope).unwrap().unwrap();
    assert_eq!(summary.tasks[0].attempts.len(), 1);
    assert_eq!(
        summary.tasks[0].pre_admission.as_ref().unwrap().outcome,
        RoutingDisplayPreAdmissionOutcome::NotAdmitted {
            stage: PreAdmissionStop::CapacityUnavailable,
        }
    );
}

#[test]
fn cancelled_selected_observation_closes_without_an_attempt_or_capacity_release() {
    let (_, db, _, facts, scope) = setup(1);
    let task_id = TaskId("task0".into());
    let decision = db
        .preview_routing_decision(&scope, &task_id, &facts, None)
        .unwrap();
    db.observe_routing_decision(&ObserveRoutingDecision {
        scope: scope.clone(),
        event_id: "selected-before-stop".into(),
        task_id,
        expected_cancel_epoch: 0,
        expected_next_ordinal: 1,
        facts: facts.clone(),
        decision,
        advice_json: None,
    })
    .unwrap();
    assert!(db
        .close_unadmitted_routing_observation(&CloseUnadmittedRoutingObservation {
            scope: scope.clone(),
            observation_event_id: "selected-before-stop".into(),
            stage: PreAdmissionStop::Cancelled,
            controller_evidence_digest: None,
            now_ms: facts.now_ms + 1,
        })
        .is_err());
    db.cancel_routing_mission(&scope, "cancel-selected", 0, facts.now_ms + 1)
        .unwrap();
    let cancelled_display = db.routing_display_summary(&scope).unwrap().unwrap();
    assert!(cancelled_display.cancelled);
    assert_eq!(
        cancelled_display.tasks[0].state,
        TaskRoutingState::Cancelled
    );
    assert_eq!(
        cancelled_display.tasks[0]
            .pre_admission
            .as_ref()
            .unwrap()
            .outcome,
        RoutingDisplayPreAdmissionOutcome::Pending
    );
    let resolved = db
        .close_unadmitted_routing_observation(&CloseUnadmittedRoutingObservation {
            scope: scope.clone(),
            observation_event_id: "selected-before-stop".into(),
            stage: PreAdmissionStop::Cancelled,
            controller_evidence_digest: None,
            now_ms: facts.now_ms + 1,
        })
        .unwrap();
    assert!(matches!(
        resolved.outcome,
        RoutingObservationOutcome::NotAdmitted {
            stage: PreAdmissionStop::Cancelled,
            ..
        }
    ));
    let history = db.routing_history(&scope).unwrap().unwrap();
    assert!(history.cancelled);
    assert!(history.attempts.is_empty());
}

#[test]
fn changed_selected_route_links_to_a_blocked_outcome_atomically() {
    let (temp, db, _, mut facts, scope) = setup(1);
    let task_id = TaskId("task0".into());
    let selected = db
        .preview_routing_decision(&scope, &task_id, &facts, None)
        .unwrap();
    db.observe_routing_decision(&ObserveRoutingDecision {
        scope: scope.clone(),
        event_id: "selection-before-capacity-loss".into(),
        task_id: task_id.clone(),
        expected_cancel_epoch: 0,
        expected_next_ordinal: 1,
        facts: facts.clone(),
        decision: selected,
        advice_json: None,
    })
    .unwrap();
    for observation in &mut facts.observations {
        observation.capacity_ready = false;
    }
    let blocked = db
        .preview_routing_decision(&scope, &task_id, &facts, None)
        .unwrap();
    assert!(!matches!(blocked.selection, RouteSelection::Selected(_)));
    let request = RecordRoutingBlock {
        scope: scope.clone(),
        event_id: "block-after-capacity-loss".into(),
        task_id,
        facts,
        decision: blocked,
        advice_json: None,
        observation_event_id: Some("selection-before-capacity-loss".into()),
    };
    let recorded = db.record_routing_block(&request).unwrap();
    assert_eq!(db.record_routing_block(&request).unwrap(), recorded);
    let history = db.routing_history(&scope).unwrap().unwrap();
    assert!(history.attempts.is_empty());
    assert!(history.events.iter().any(|entry| matches!(
        &entry.event,
        RoutingControlEvent::DecisionResolved(resolution)
            if resolution.observation_event_id == "selection-before-capacity-loss"
                && matches!(resolution.outcome, RoutingObservationOutcome::NotAdmitted {
                    stage: PreAdmissionStop::Superseded,
                    ..
                })
    )));
    drop(db);
    let db = PytxoStore::open(&temp.path().join("store.db")).unwrap();
    assert_eq!(db.record_routing_block(&request).unwrap(), recorded);
}

#[test]
fn recorded_block_reopens_with_reason_without_consuming_attempt_or_budget() {
    let (temp, db, _, mut f, s) = setup(1);
    f.observations[0].auth_status = Readiness::Unavailable;
    let decision = db
        .preview_routing_decision(&s, &TaskId("task0".into()), &f, None)
        .unwrap();
    let r = RecordRoutingBlock {
        scope: s.clone(),
        event_id: "blocked-auth".into(),
        task_id: TaskId("task0".into()),
        facts: f.clone(),
        decision: decision.clone(),
        advice_json: None,
        observation_event_id: None,
    };
    let t = db.record_routing_block(&r).unwrap();
    assert_eq!(t.state, TaskRoutingState::WaitingInput);
    assert_eq!(t.next_ordinal, 1);
    assert_eq!(db.record_routing_block(&r).unwrap(), t);
    drop(db);
    let db = PytxoStore::open(&temp.path().join("store.db")).unwrap();
    let h = db.routing_history(&s).unwrap().unwrap();
    assert_eq!(h.tasks[0].last_decision, Some(decision));
    assert!(h.attempts.is_empty());
    assert_eq!(
        db.routing_snapshot(&s, &t.registration.contract.task_id, &f)
            .unwrap()
            .spent_and_reserved_nano_usd,
        Some(0)
    );
    f.observations[0].auth_status = Readiness::Ready;
    let recovered = db
        .preview_routing_decision(&s, &TaskId("task0".into()), &f, None)
        .unwrap();
    assert!(matches!(recovered.selection, RouteSelection::Selected(_)));
    db.observe_routing_decision(&ObserveRoutingDecision {
        scope: s.clone(),
        event_id: "selected-after-input-recovers".into(),
        task_id: TaskId("task0".into()),
        expected_cancel_epoch: h.cancel_epoch,
        expected_next_ordinal: 1,
        facts: f.clone(),
        decision: recovered,
        advice_json: None,
    })
    .unwrap();
    let mut admission = request(&db, &s, &f, "task0");
    admission.observation_event_id = Some("selected-after-input-recovers".into());
    db.admit_routing_attempt(&admission).unwrap();
    assert!(db
        .record_routing_block(&RecordRoutingBlock {
            event_id: "cannot-hide-active".into(),
            ..r
        })
        .is_err());
}

#[test]
fn retry_is_strong_bounded_and_keeps_original_contract_deadline_and_failed_usage() {
    let temp = tempfile::tempdir().unwrap();
    let db = PytxoStore::open(&temp.path().join("db")).unwrap();
    db.insert_run("run", "repo").unwrap();
    let (mut m, f, s) = registration(1);
    m.authorization.limits.max_spend_nano_usd = Some(200);
    install(&db, &m, &f, &s);
    let mut initial = request(&db, &s, &f, "task0");
    db.observe_routing_decision(&ObserveRoutingDecision {
        scope: s.clone(),
        event_id: "initial-selection".into(),
        task_id: TaskId("task0".into()),
        expected_cancel_epoch: 0,
        expected_next_ordinal: 1,
        facts: f.clone(),
        decision: initial.decision.clone(),
        advice_json: None,
    })
    .unwrap();
    initial.observation_event_id = Some("initial-selection".into());
    let a = db.admit_routing_attempt(&initial).unwrap();
    let a = advance(&db, &s, &f, a, AttemptState::Verifying);
    let h = db.routing_history(&s).unwrap().unwrap();
    let a = db
        .transition_routing_attempt(&TransitionRoutingAttempt {
            scope: s.clone(),
            event_id: "check-failed".into(),
            attempt_id: a.attempt_id.clone(),
            expected_attempt_revision: a.revision,
            expected_task_revision: h.tasks[0].revision,
            to: AttemptState::Failed,
            facts: f.clone(),
            receipts: Default::default(),
            failure: Some(RepairEvidence {
                attempt_id: a.attempt_id.clone(),
                ordinal: 1,
                state: AttemptState::Failed,
                failure_class: AttemptFailureClass::Check,
                actionable_evidence_digest: Some(digest("actionable")),
            }),
        })
        .unwrap();
    let mut retry = request(&db, &s, &f, "task0");
    retry.event_id = "repair".into();
    retry.attempt_id = AttemptId("repair".into());
    retry.agent_id = "repair-agent".into();
    retry.capacity_reservation = "repair-slot".into();
    retry.handoff = Some(BlobRef {
        digest: digest("repair handoff"),
        byte_length: 1,
    });
    assert!(db.admit_routing_attempt(&retry).is_err());
    db.observe_routing_decision(&ObserveRoutingDecision {
        scope: s.clone(),
        event_id: "repair-selection".into(),
        task_id: TaskId("task0".into()),
        expected_cancel_epoch: 0,
        expected_next_ordinal: 2,
        facts: f.clone(),
        decision: retry.decision.clone(),
        advice_json: None,
    })
    .unwrap();
    retry.observation_event_id = Some("repair-selection".into());
    assert!(db.admit_routing_attempt(&retry).is_err());
    // This policy-only fixture has no retained repair bundle. A fabricated
    // handoff reference cannot accompany an otherwise eligible retry.
    retry.handoff = None;
    let b = db.admit_routing_attempt(&retry).unwrap();
    assert_eq!(b.ordinal, 2);
    assert_eq!(b.predecessor, Some(a.attempt_id));
    assert_eq!(b.selected.profile.id, ProfileId("strong".into()));
    let b = advance(&db, &s, &f, b, AttemptState::Passed);
    let h = db.routing_history(&s).unwrap().unwrap();
    assert_eq!(
        h.tasks[0].winner.as_ref().unwrap().winning_attempt_id,
        b.attempt_id
    );
    assert_eq!(h.tasks[0].registration.contract.revision, 1);
    assert_eq!(h.mission.authorization.limits.deadline_ms, 1000);
    assert_eq!(h.attempts.len(), 2);
    assert_eq!(h.attempts[0].usage, RoutedUsage::Unreported);
    let display = db.routing_display_summary(&s).unwrap().unwrap();
    assert_eq!(display.tasks[0].attempts.len(), 2);
    assert_eq!(
        display.tasks[0].attempts[0].decision.reason,
        a.decision.reason
    );
    assert_eq!(
        display.tasks[0].attempts[0].role,
        RoutingDisplayRole::Everyday
    );
    assert_eq!(
        display.tasks[0].attempts[1].decision.reason,
        RouteReason::StrongRepair
    );
    assert_eq!(
        display.tasks[0].attempts[1].role,
        RoutingDisplayRole::Strong
    );
    assert!(db
        .admit_routing_attempt(&request(&db, &s, &f, "task0"))
        .is_err());
}

#[test]
fn attempt_cap_counts_failed_startup_across_tasks() {
    let temp = tempfile::tempdir().unwrap();
    let db = PytxoStore::open(&temp.path().join("db")).unwrap();
    db.insert_run("run", "repo").unwrap();
    let (mut m, f, s) = registration(2);
    m.authorization.limits.max_attempts = 1;
    m.authorization.limits.max_spend_nano_usd = Some(200);
    install(&db, &m, &f, &s);
    let a = db
        .admit_routing_attempt(&request(&db, &s, &f, "task0"))
        .unwrap();
    step(
        &db,
        &s,
        &f,
        &a,
        AttemptState::FailedNoLaunch,
        RoutingReceipts {
            no_worker_created: Some(digest("none")),
            ..Default::default()
        },
    )
    .unwrap();
    assert!(db
        .admit_routing_attempt(&request(&db, &s, &f, "task1"))
        .is_err());
    assert_eq!(
        db.routing_snapshot(&s, &TaskId("task1".into()), &f)
            .unwrap()
            .admitted_attempts,
        1
    );
}

#[test]
fn terminal_legacy_run_cannot_publish_winner() {
    let (temp, db, _, f, s) = setup(1);
    let a = db
        .admit_routing_attempt(&request(&db, &s, &f, "task0"))
        .unwrap();
    let a = advance(&db, &s, &f, a, AttemptState::Verifying);
    let sql = rusqlite::Connection::open(temp.path().join("store.db")).unwrap();
    sql.execute("UPDATE runs SET status='failed' WHERE id='run'", [])
        .unwrap();
    assert!(step(
        &db,
        &s,
        &f,
        &a,
        AttemptState::Passed,
        RoutingReceipts {
            checks: Some(digest("checks")),
            ..Default::default()
        }
    )
    .is_err());
    assert!(db.routing_history(&s).unwrap().unwrap().tasks[0]
        .winner
        .is_none());
}

#[test]
fn receipt_from_before_launch_cannot_prove_later_quiescence_or_checks() {
    let (_temp, db, _, f, s) = setup(1);
    let a = db
        .admit_routing_attempt(&request(&db, &s, &f, "task0"))
        .unwrap();
    assert!(step(
        &db,
        &s,
        &f,
        &a,
        AttemptState::Preparing,
        RoutingReceipts {
            inputs: Some(digest("input")),
            quiescence: Some(digest("quiet-before-spawn")),
            checks: Some(digest("old-checks")),
            ..Default::default()
        }
    )
    .is_err());
}

#[test]
fn current_observation_cannot_change_admitted_launch_fingerprint() {
    let (_temp, db, _, mut f, s) = setup(1);
    let a = db
        .admit_routing_attempt(&request(&db, &s, &f, "task0"))
        .unwrap();
    let a = advance(&db, &s, &f, a, AttemptState::Preparing);
    f.observations[0].executable.digest = digest("replaced executable");
    assert!(step(
        &db,
        &s,
        &f,
        &a,
        AttemptState::Launching,
        RoutingReceipts {
            launch_checks: Some(digest("launch")),
            ..Default::default()
        }
    )
    .is_err());
}

#[test]
fn current_launch_shape_cannot_change_after_admission() {
    let (_temp, db, _, mut facts, scope) = setup(1);
    let admitted = db
        .admit_routing_attempt(&request(&db, &scope, &facts, "task0"))
        .unwrap();
    let preparing = advance(&db, &scope, &facts, admitted, AttemptState::Preparing);
    facts.observations[0]
        .launch
        .as_mut()
        .unwrap()
        .arguments_digest = digest("unreviewed-argv");
    assert!(step(
        &db,
        &scope,
        &facts,
        &preparing,
        AttemptState::Launching,
        RoutingReceipts {
            launch_checks: Some(digest("launch")),
            ..Default::default()
        }
    )
    .is_err());
}

#[test]
fn hosted_private_prompt_digest_cannot_change_after_admission() {
    let (_temp, db, _, mut facts, scope) = setup(1);
    let admitted = db
        .admit_routing_attempt(&request(&db, &scope, &facts, "task0"))
        .unwrap();
    let preparing = advance(&db, &scope, &facts, admitted, AttemptState::Preparing);
    facts.observations[0]
        .launch
        .as_mut()
        .unwrap()
        .private_stdin_digest = Some(digest("different private prompt"));
    assert!(step(
        &db,
        &scope,
        &facts,
        &preparing,
        AttemptState::Launching,
        RoutingReceipts {
            launch_checks: Some(digest("launch")),
            ..Default::default()
        }
    )
    .is_err());
}

#[test]
fn legacy_hosted_qualification_remains_in_history_but_cannot_route() {
    let temp = tempfile::tempdir().unwrap();
    let path = temp.path().join("store.db");
    let db = PytxoStore::open(&path).unwrap();
    db.insert_run("run", "repo").unwrap();
    let (mission, mut facts, scope) = registration(1);
    let profile = &mission.profiles[0];
    let observation = &mut facts.observations[0];
    let mut legacy = observation.qualification.clone().unwrap();
    legacy.launch_fingerprint = launch_fingerprint(
        &profile.profile,
        &profile.binding,
        &observation.executable,
        observation.launch.as_ref().unwrap(),
    )
    .unwrap();
    observation.qualification = Some(legacy.clone());
    install(&db, &mission, &facts, &scope);
    drop(db);

    let db = PytxoStore::open(&path).unwrap();
    let history = db.routing_history(&scope).unwrap().unwrap();
    assert!(history.events.iter().any(|entry| matches!(
        &entry.event,
        RoutingControlEvent::Qualified { qualification } if qualification == &legacy
    )));
    assert!(db
        .routing_snapshot(&scope, &TaskId("task0".into()), &facts)
        .unwrap()
        .qualification_digests
        .contains(&legacy.digest().unwrap()));
    assert_eq!(
        db.preview_routing_decision(&scope, &TaskId("task0".into()), &facts, None)
            .unwrap()
            .selection,
        RouteSelection::Blocked(RouteBlocker::PreferredUnavailable)
    );
    assert!(history.attempts.is_empty());
}

#[test]
fn routed_run_contract_insert_is_exact_and_never_replaces_review_authority() {
    let temp = tempfile::tempdir().unwrap();
    let db = PytxoStore::open(&temp.path().join("db")).unwrap();
    db.insert_starting_run_with_profile("routed-contract", "repo", Some("orbit"))
        .unwrap();
    db.insert_run_contract_once("routed-contract", "base", "plan-a", "orbit-receipt")
        .unwrap();
    db.insert_run_contract_once("routed-contract", "base", "plan-a", "orbit-receipt")
        .unwrap();
    assert!(db
        .insert_run_contract_once("routed-contract", "base", "plan-b", "orbit-receipt")
        .is_err());
    let original = db.get_run_contract("routed-contract").unwrap().unwrap();
    assert_eq!(original.plan_json.as_deref(), Some("plan-a"));
    assert_eq!(original.apply_status, "pending");
    db.save_run_contract_with_status(
        "routed-contract",
        Some("base"),
        "advanced-plan",
        "advanced-receipt",
        "non_flushable",
    )
    .unwrap();
    assert!(db
        .insert_run_contract_once("routed-contract", "base", "plan-a", "orbit-receipt")
        .is_err());
    let advanced = db.get_run_contract("routed-contract").unwrap().unwrap();
    assert_eq!(advanced.plan_json.as_deref(), Some("advanced-plan"));
    assert_eq!(advanced.apply_status, "non_flushable");
}

#[test]
fn routed_review_publication_requires_a_registered_owned_winner() {
    let temp = tempfile::tempdir().unwrap();
    let db = PytxoStore::open(&temp.path().join("db")).unwrap();
    for (run_id, expected) in [("routed-ready", "starting"), ("routed-recovered", "failed")] {
        db.insert_starting_run_with_profile(run_id, "repo", Some("orbit"))
            .unwrap();
        db.insert_run_contract_once(run_id, "base", "plan", "orbit-receipt")
            .unwrap();
        if expected == "failed" {
            assert!(db
                .finish_run_if_status(run_id, "starting", "failed")
                .unwrap());
        }
        assert!(db.begin_run_preparation(run_id).unwrap());
        let manifest = pytxo_core::PreparedRunManifest {
            version: 2,
            run_id: run_id.into(),
            base_revision: "base".into(),
            prepared_at: "2026-09-25T00:00:00Z".into(),
            package_digest: "sha256:fixture-ready".into(),
            summary: Default::default(),
            files: vec![],
            candidate_verification: None,
        };
        let wrong = if expected == "failed" {
            "starting"
        } else {
            "failed"
        };
        assert!(db
            .finish_routed_review_and_run(run_id, &manifest, wrong)
            .is_err());
        assert_eq!(db.get_run_status(run_id).unwrap().unwrap().0, expected);
        assert_eq!(
            db.get_run_contract(run_id).unwrap().unwrap().apply_status,
            "preparing"
        );
        assert!(db
            .finish_routed_review_and_run(run_id, &manifest, expected)
            .is_err());
        assert_eq!(db.get_run_status(run_id).unwrap().unwrap().0, expected);
        let contract = db.get_run_contract(run_id).unwrap().unwrap();
        assert_eq!(contract.apply_status, "preparing");
        assert!(contract.prepared_manifest.is_none());
        assert!(db
            .finish_routed_review_and_run(run_id, &manifest, expected)
            .is_err());
    }
}

#[test]
fn generic_review_publication_cannot_bypass_routed_authority() {
    let temp = tempfile::tempdir().unwrap();
    let db = PytxoStore::open(&temp.path().join("db")).unwrap();
    let (mission, _, scope) = registration(1);
    let run_id = &scope.run_id.0;
    db.insert_starting_run_with_profile(run_id, "repo", Some("orbit"))
        .unwrap();
    db.insert_run_contract_once(run_id, "base", "plan", "orbit-receipt")
        .unwrap();
    db.register_routing_mission(&mission).unwrap();
    assert!(db
        .save_run_contract_with_status(
            run_id,
            Some("changed-base"),
            "changed-plan",
            "changed-receipt",
            "non_flushable",
        )
        .is_err());
    let before = db.get_run_contract(run_id).unwrap().unwrap();
    assert_eq!(before.plan_json.as_deref(), Some("plan"));
    assert_eq!(before.apply_status, "pending");
    assert!(db.begin_run_preparation(run_id).unwrap());
    let manifest = pytxo_core::PreparedRunManifest {
        version: 2,
        run_id: run_id.clone(),
        base_revision: "base".into(),
        prepared_at: "2026-09-25T00:00:00Z".into(),
        package_digest: "sha256:fixture-ready".into(),
        summary: Default::default(),
        files: vec![],
        candidate_verification: None,
    };
    assert!(db.finish_run_preparation(run_id, &manifest).is_err());
    assert_eq!(db.get_run_status(run_id).unwrap().unwrap().0, "starting");
    let contract = db.get_run_contract(run_id).unwrap().unwrap();
    assert_eq!(contract.apply_status, "preparing");
    assert!(contract.prepared_manifest.is_none());
}

#[test]
fn routed_worktree_instance_is_durable_exact_and_singleton() {
    let temp = tempfile::tempdir().unwrap();
    let db = PytxoStore::open(&temp.path().join("db")).unwrap();
    db.insert_starting_run_with_profile("instance-run", "repo", Some("orbit"))
        .unwrap();
    db.insert_agent_with_root(
        "actor",
        "instance-run",
        "task",
        1,
        Some("original-worktree"),
        "fixture",
        None,
    )
    .unwrap();
    let instance = RoutedWorktreeInstance {
        path: "original-worktree".into(),
        git_file_identity: "windows-file-id:1:3".into(),
    };
    assert!(db
        .record_routed_worktree_instance("actor", "wrong-run", "task", &instance)
        .is_err());
    assert!(db
        .record_routed_worktree_instance("actor", "instance-run", "wrong-task", &instance)
        .is_err());
    assert!(db
        .record_routed_worktree_instance(
            "actor",
            "instance-run",
            "task",
            &RoutedWorktreeInstance {
                path: "different-worktree".into(),
                ..instance.clone()
            },
        )
        .is_err());
    db.record_routed_worktree_instance("actor", "instance-run", "task", &instance)
        .unwrap();
    db.record_routed_worktree_instance("actor", "instance-run", "task", &instance)
        .unwrap();
    assert_eq!(
        db.routed_worktree_instance("actor", "instance-run", "task")
            .unwrap(),
        Some(instance.clone())
    );
    assert!(db
        .record_routed_worktree_instance(
            "actor",
            "instance-run",
            "task",
            &RoutedWorktreeInstance {
                git_file_identity: "windows-file-id:1:4".into(),
                ..instance.clone()
            },
        )
        .is_err());
    assert!(db
        .routed_worktree_instance("actor", "instance-run", "wrong-task")
        .is_err());
    db.append_event("actor", "routed-worktree-instance", "{}")
        .unwrap();
    assert!(db
        .routed_worktree_instance("actor", "instance-run", "task")
        .is_err());
}

#[test]
fn routed_winner_workspace_projection_is_one_way_and_identity_bound() {
    let temp = tempfile::tempdir().unwrap();
    let db = PytxoStore::open(&temp.path().join("db")).unwrap();
    db.insert_starting_run_with_profile("routed-review", "repo", Some("orbit"))
        .unwrap();
    db.insert_agent_with_root(
        "actor",
        "routed-review",
        "task",
        1,
        Some("original-worktree"),
        "fixture",
        None,
    )
    .unwrap();
    assert!(db
        .promote_routed_winner_workspace(
            "actor",
            "another-run",
            "task",
            "original-worktree",
            "sealed-view",
        )
        .is_err());
    db.promote_routed_winner_workspace(
        "actor",
        "routed-review",
        "task",
        "original-worktree",
        "sealed-view",
    )
    .unwrap();
    db.promote_routed_winner_workspace(
        "actor",
        "routed-review",
        "task",
        "original-worktree",
        "sealed-view",
    )
    .unwrap();
    assert!(db
        .promote_routed_winner_workspace(
            "actor",
            "routed-review",
            "task",
            "original-worktree",
            "different-view",
        )
        .is_err());
    assert_eq!(
        db.get_agent("actor")
            .unwrap()
            .unwrap()
            .worktree_path
            .as_deref(),
        Some("sealed-view")
    );
    assert_eq!(
        db.routed_original_worktree("actor", "routed-review")
            .unwrap()
            .as_deref(),
        Some("original-worktree")
    );
    db.insert_run_contract_once("routed-review", "base", "plan", "orbit-receipt")
        .unwrap();
    db.finish_agent("actor", Some(0), "completed").unwrap();
    assert!(db
        .finish_run_if_status("routed-review", "starting", "failed")
        .unwrap());
    assert!(db.begin_run_preparation("routed-review").unwrap());
    assert!(db
        .refresh_routed_winner_workspace(
            "actor",
            "routed-review",
            "task",
            "wrong-view",
            "fresh-view"
        )
        .is_err());
    db.refresh_routed_winner_workspace(
        "actor",
        "routed-review",
        "task",
        "sealed-view",
        "fresh-view",
    )
    .unwrap();
    assert_eq!(
        db.get_agent("actor")
            .unwrap()
            .unwrap()
            .worktree_path
            .as_deref(),
        Some("fresh-view")
    );
    assert_eq!(
        db.routed_original_worktree("actor", "routed-review")
            .unwrap()
            .as_deref(),
        Some("original-worktree")
    );
    assert!(db
        .refresh_routed_winner_workspace(
            "actor",
            "routed-review",
            "task",
            "sealed-view",
            "another-view"
        )
        .is_err());
    db.append_event("actor", "routed-original-worktree", "forged-worktree")
        .unwrap();
    assert!(db
        .routed_original_worktree("actor", "routed-review")
        .is_err());
}

#[test]
fn routed_refresh_before_first_projection_retains_original_workspace() {
    let temp = tempfile::tempdir().unwrap();
    let db = PytxoStore::open(&temp.path().join("db")).unwrap();
    db.insert_starting_run_with_profile("pre-projection", "repo", Some("orbit"))
        .unwrap();
    db.insert_agent_with_root(
        "actor",
        "pre-projection",
        "task",
        1,
        Some("original-worktree"),
        "fixture",
        None,
    )
    .unwrap();
    db.insert_run_contract_once("pre-projection", "base", "plan", "orbit-receipt")
        .unwrap();
    db.finish_agent("actor", Some(0), "completed").unwrap();
    assert!(db
        .finish_run_if_status("pre-projection", "starting", "failed")
        .unwrap());
    assert!(db.begin_run_preparation("pre-projection").unwrap());
    db.refresh_routed_winner_workspace(
        "actor",
        "pre-projection",
        "task",
        "original-worktree",
        "fresh-view",
    )
    .unwrap();
    assert_eq!(
        db.routed_original_worktree("actor", "pre-projection")
            .unwrap()
            .as_deref(),
        Some("original-worktree")
    );
    assert_eq!(
        db.get_agent("actor")
            .unwrap()
            .unwrap()
            .worktree_path
            .as_deref(),
        Some("fresh-view")
    );
}

#[test]
fn dependency_failure_blocks_descendants_and_cannot_be_overwritten_by_wait() {
    let temp = tempfile::tempdir().unwrap();
    let db = PytxoStore::open(&temp.path().join("db")).unwrap();
    db.insert_run("run", "repo").unwrap();
    let (mut m, f, s) = registration(2);
    m.tasks[1].contract.dependencies = vec![TaskId("task0".into())];
    m.authorization.allowed_task_digests = m
        .tasks
        .iter()
        .map(|t| t.contract.digest().unwrap())
        .collect();
    install(&db, &m, &f, &s);
    let a = db
        .admit_routing_attempt(&request(&db, &s, &f, "task0"))
        .unwrap();
    step(
        &db,
        &s,
        &f,
        &a,
        AttemptState::FailedNoLaunch,
        RoutingReceipts {
            no_worker_created: Some(digest("none")),
            ..Default::default()
        },
    )
    .unwrap();
    let h = db.routing_history(&s).unwrap().unwrap();
    assert_eq!(h.tasks[1].state, TaskRoutingState::BlockedDependency);
    let decision = db
        .preview_routing_decision(&s, &TaskId("task1".into()), &f, None)
        .unwrap();
    assert!(db
        .record_routing_block(&RecordRoutingBlock {
            scope: s,
            event_id: "wait".into(),
            task_id: TaskId("task1".into()),
            facts: f,
            decision,
            advice_json: None,
            observation_event_id: None,
        })
        .is_err());
}

#[test]
fn transition_replay_returns_original_result_and_conflicts_on_changed_payload() {
    let (_temp, db, _, f, s) = setup(1);
    let a = db
        .admit_routing_attempt(&request(&db, &s, &f, "task0"))
        .unwrap();
    let task = db.routing_history(&s).unwrap().unwrap().tasks[0].clone();
    let r = TransitionRoutingAttempt {
        scope: s.clone(),
        event_id: "prepare-once".into(),
        attempt_id: a.attempt_id,
        expected_attempt_revision: a.revision,
        expected_task_revision: task.revision,
        to: AttemptState::Preparing,
        facts: f,
        receipts: RoutingReceipts {
            inputs: Some(digest("input")),
            ..Default::default()
        },
        failure: None,
    };
    let a = db.transition_routing_attempt(&r).unwrap();
    assert_eq!(db.transition_routing_attempt(&r).unwrap(), a);
    let mut changed = r.clone();
    changed.receipts.inputs = Some(digest("different"));
    assert!(db.transition_routing_attempt(&changed).is_err());
    changed = r;
    changed.event_id = "stale-CAS".into();
    assert!(db.transition_routing_attempt(&changed).is_err());
}

#[test]
fn actual_cost_overrun_is_retained_and_blocks_remaining_admission() {
    let (_temp, db, _, f, s) = setup(2);
    let a = db
        .admit_routing_attempt(&request(&db, &s, &f, "task0"))
        .unwrap();
    let a = step(
        &db,
        &s,
        &f,
        &a,
        AttemptState::FailedNoLaunch,
        RoutingReceipts {
            no_worker_created: Some(digest("none")),
            ..Default::default()
        },
    )
    .unwrap();
    let usage = RoutedUsage::Known {
        nano_usd: 500,
        receipt: digest("provider-overrun"),
    };
    let a = db
        .settle_routing_usage(&s, "overrun", &a.attempt_id, a.revision, &usage, 151)
        .unwrap();
    assert_eq!(a.usage, usage);
    assert_eq!(
        db.routing_snapshot(&s, &TaskId("task1".into()), &f)
            .unwrap()
            .spent_and_reserved_nano_usd,
        Some(500)
    );
    assert!(db
        .admit_routing_attempt(&request(&db, &s, &f, "task1"))
        .is_err());
}

fn launch_budget_after_settlement(usage: RoutedUsage, expected_total: Option<u64>, allowed: bool) {
    let temp = tempfile::tempdir().unwrap();
    let path = temp.path().join("launch-budget.db");
    let db = PytxoStore::open(&path).unwrap();
    db.insert_run("run", "repo").unwrap();
    let (mut mission, mut facts, scope) = registration(2);
    mission.authorization.limits.max_spend_nano_usd = Some(150);
    install(&db, &mission, &facts, &scope);
    let a = db
        .admit_routing_attempt(&request(&db, &scope, &facts, "task0"))
        .unwrap();
    let b = db
        .admit_routing_attempt(&request(&db, &scope, &facts, "task1"))
        .unwrap();
    let b = advance(&db, &scope, &facts, b, AttemptState::Preparing);
    let a = advance(&db, &scope, &facts, a, AttemptState::Passed);
    db.settle_routing_usage(
        &scope,
        "settled-before-second-launch",
        &a.attempt_id,
        a.revision,
        &usage,
        151,
    )
    .unwrap();
    facts.now_ms = 152;
    let before = db.routing_history(&scope).unwrap().unwrap();
    assert_eq!(
        db.routing_snapshot(&scope, &b.task_id, &facts)
            .unwrap()
            .spent_and_reserved_nano_usd,
        expected_total
    );
    let sql = rusqlite::Connection::open(&path).unwrap();
    let before_notifications: u64 = sql
        .query_row("SELECT COUNT(*) FROM domain_changes", [], |r| r.get(0))
        .unwrap();
    let result = step(
        &db,
        &scope,
        &facts,
        &b,
        AttemptState::Launching,
        RoutingReceipts {
            launch_checks: Some(digest("launch")),
            ..Default::default()
        },
    );
    if allowed {
        let launched = result.unwrap();
        assert_eq!(launched.state, AttemptState::Launching);
        assert_eq!(launched.reserved_nano_usd, 60);
        assert!(!launched.ownership_released);
        assert_eq!(
            db.routing_snapshot(&scope, &b.task_id, &facts)
                .unwrap()
                .spent_and_reserved_nano_usd,
            expected_total
        );
        assert_eq!(
            db.routing_history(&scope).unwrap().unwrap().events.len(),
            before.events.len() + 1
        );
    } else {
        assert!(
            result.is_err(),
            "underfunded or unknown full ledger must deny an admitted launch"
        );
        // No projection revision, receipt, launch event, routing revision, or
        // notification may survive the denied transaction.
        assert_eq!(db.routing_history(&scope).unwrap().unwrap(), before);
        assert_eq!(
            sql.query_row("SELECT COUNT(*) FROM domain_changes", [], |r| r
                .get::<_, u64>(0))
                .unwrap(),
            before_notifications
        );
        assert_eq!(db.unresolved_routing_attempts().unwrap(), vec![b]);
    }
}

#[test]
fn launch_budget_denies_actual_overrun_after_both_attempts_were_admitted() {
    launch_budget_after_settlement(
        RoutedUsage::Known {
            nano_usd: 100,
            receipt: digest("actual-overrun"),
        },
        Some(160),
        false,
    );
}

#[test]
fn launch_budget_allows_full_ledger_exactly_at_cap_without_doublecounting() {
    launch_budget_after_settlement(
        RoutedUsage::Known {
            nano_usd: 90,
            receipt: digest("actual-at-cap"),
        },
        Some(150),
        true,
    );
}

#[test]
fn launch_budget_denies_unknown_cost_and_keeps_the_other_reservation() {
    launch_budget_after_settlement(
        RoutedUsage::Unknown {
            reason: "no final provider receipt".into(),
        },
        None,
        false,
    );
}

#[test]
fn launch_budget_denies_estimated_overrun_and_keeps_the_other_reservation() {
    launch_budget_after_settlement(
        RoutedUsage::Estimated {
            nano_usd: 100,
            receipt: digest("estimated-overrun"),
        },
        Some(160),
        false,
    );
}

#[test]
fn launch_budget_keeps_full_reservation_despite_low_estimate() {
    launch_budget_after_settlement(
        RoutedUsage::Estimated {
            nano_usd: 20,
            receipt: digest("low-estimate"),
        },
        Some(120),
        true,
    );
}

#[test]
fn launch_budget_denies_overflow_without_releasing_reservations() {
    launch_budget_after_settlement(
        RoutedUsage::Known {
            nano_usd: u64::MAX,
            receipt: digest("overflow-cost"),
        },
        None,
        false,
    );
}

#[test]
fn capacity_reference_and_agent_identity_cannot_be_reused() {
    let temp = tempfile::tempdir().unwrap();
    let db = PytxoStore::open(&temp.path().join("db")).unwrap();
    db.insert_run("run", "repo").unwrap();
    let (mut m, f, s) = registration(2);
    m.authorization.limits.max_spend_nano_usd = Some(200);
    install(&db, &m, &f, &s);
    let a = db
        .admit_routing_attempt(&request(&db, &s, &f, "task0"))
        .unwrap();
    let mut r = request(&db, &s, &f, "task1");
    r.capacity_reservation = a.capacity_reservation;
    assert!(db.admit_routing_attempt(&r).is_err());
    r.capacity_reservation = "fresh-capacity".into();
    r.agent_id = a.agent_id;
    assert!(db.admit_routing_attempt(&r).is_err());
    r.agent_id = "legacy-agent".into();
    let sql = rusqlite::Connection::open(temp.path().join("db")).unwrap();
    sql.execute("INSERT INTO agents(id,run_id,task_id,wave,cmd,status) VALUES ('legacy-agent','run','legacy-task',0,'legacy','failed')",[]).unwrap();
    assert!(db.admit_routing_attempt(&r).is_err());
    assert_eq!(db.routing_history(&s).unwrap().unwrap().attempts.len(), 1);
}

#[test]
fn concurrent_cancel_and_pass_have_one_serializable_winner_boundary() {
    let (temp, db, _, f, s) = setup(1);
    let a = db
        .admit_routing_attempt(&request(&db, &s, &f, "task0"))
        .unwrap();
    let a = advance(&db, &s, &f, a, AttemptState::Verifying);
    let task = db.routing_history(&s).unwrap().unwrap().tasks[0].clone();
    let r = TransitionRoutingAttempt {
        scope: s.clone(),
        event_id: "race-pass".into(),
        attempt_id: a.attempt_id,
        expected_attempt_revision: a.revision,
        expected_task_revision: task.revision,
        to: AttemptState::Passed,
        facts: f,
        receipts: RoutingReceipts {
            checks: Some(digest("checks")),
            ..Default::default()
        },
        failure: None,
    };
    let b = std::sync::Arc::new(std::sync::Barrier::new(2));
    let path = temp.path().join("store.db");
    let scope = s.clone();
    let barrier = b.clone();
    let cancel = std::thread::spawn(move || {
        let db = PytxoStore::open(&path).unwrap();
        barrier.wait();
        db.cancel_routing_mission(&scope, "race-stop", 0, 151)
    });
    b.wait();
    let published = db.transition_routing_attempt(&r).is_ok();
    cancel.join().unwrap().unwrap();
    let h = db.routing_history(&s).unwrap().unwrap();
    assert!(h.cancelled);
    assert_eq!(h.tasks[0].winner.is_some(), published);
    if published {
        let passed = h
            .events
            .iter()
            .find(|e| e.event_id == "race-pass")
            .unwrap()
            .sequence;
        let stopped = h
            .events
            .iter()
            .find(|e| e.event_id == "race-stop")
            .unwrap()
            .sequence;
        assert!(passed < stopped);
    }
}

#[test]
fn estimates_cannot_release_unproven_spend_or_clear_unknown_cost() {
    let (_temp, db, _, f, s) = setup(2);
    let a = db
        .admit_routing_attempt(&request(&db, &s, &f, "task0"))
        .unwrap();
    let a = step(
        &db,
        &s,
        &f,
        &a,
        AttemptState::FailedNoLaunch,
        RoutingReceipts {
            no_worker_created: Some(digest("none")),
            ..Default::default()
        },
    )
    .unwrap();
    let a = db
        .settle_routing_usage(
            &s,
            "estimate-large",
            &a.attempt_id,
            a.revision,
            &RoutedUsage::Estimated {
                nano_usd: 120,
                receipt: digest("estimate"),
            },
            151,
        )
        .unwrap();
    assert!(db
        .settle_routing_usage(
            &s,
            "estimate-shrink",
            &a.attempt_id,
            a.revision,
            &RoutedUsage::Estimated {
                nano_usd: 5,
                receipt: digest("lower-estimate")
            },
            152
        )
        .is_err());
    let a = db
        .settle_routing_usage(
            &s,
            "usage-unknown",
            &a.attempt_id,
            a.revision,
            &RoutedUsage::Unknown {
                reason: "lost final metering".into(),
            },
            152,
        )
        .unwrap();
    assert!(db
        .settle_routing_usage(
            &s,
            "unknown-to-estimate",
            &a.attempt_id,
            a.revision,
            &RoutedUsage::Estimated {
                nano_usd: 5,
                receipt: digest("guess")
            },
            153
        )
        .is_err());
}

fn digest(label: &str) -> Digest {
    Digest::of_bytes(label.as_bytes())
}
fn set(values: &[&str]) -> BTreeSet<String> {
    values.iter().map(|v| (*v).into()).collect()
}

fn candidate(name: &str) -> ProfileCandidate {
    let profile = ExecutionProfile {
        schema_version: 1,
        canonicalization_version: 1,
        id: ProfileId(name.into()),
        revision: 1,
        harness_id: name.into(),
        adapter_contract_version: "1".into(),
        adapter_digest: digest("adapter"),
        requested_model: ModelIdentity {
            provider: "vendor".into(),
            model: name.into(),
            reasoning: Some("medium".into()),
            revision: None,
        },
        skill_tool_bundle_digest: digest("tools"),
        backend: ExecutionBackend::Pty,
        capabilities: set(&["edit", "check"]),
    };
    let binding = ProfileBinding {
        schema_version: 1,
        canonicalization_version: 1,
        id: BindingId(name.into()),
        revision: 1,
        profile_digest: profile.digest().unwrap(),
        credential_reference: Some("keychain:worker".into()),
        auth_owner: "local-user".into(),
        billing_source_id: BillingSourceId("subscription".into()),
        billing_mode: BillingSourceMode::Subscription,
        endpoint_identity: "vendor-cli".into(),
        trust_class: "vendor".into(),
        capacity_pool_ids: set(&["host-worker", "account"]),
    };
    let launch = LaunchContract {
        schema_version: 1,
        transport: LaunchTransport::HostPty,
        host: Some(ExecutableIdentity {
            path: "C:/tools/pytxo-attempt-host.exe".into(),
            version: "1.0".into(),
            digest: digest("host"),
        }),
        dependencies: vec![],
        arguments_digest: digest("argv-template"),
        environment_policy_digest: digest("environment-policy"),
        working_directory_policy: "reviewed_attempt_worktree_v1".into(),
        stdin_delivery: StdinDelivery::PrivateHostPipe,
        private_stdin_digest: Some(digest("private-stdin")),
        output_protocol: "pytxo-attempt-host/1".into(),
        argument_lowering: "rust-std-command-windows-structured-argv/v1".into(),
        barrier_timeout_ms: Some(5_000),
        execution_timeout_ms: 30_000,
        settlement_timeout_ms: 5_000,
        output_limit_bytes: 4096,
    };
    let observation = ProfileObservation {
        schema_version: 1,
        profile_digest: profile.digest().unwrap(),
        binding_digest: binding.digest().unwrap(),
        executable: ExecutableIdentity {
            path: format!("C:/tools/{name}.exe"),
            version: "1.0".into(),
            digest: digest("exe"),
        },
        launch: Some(launch.clone()),
        observed_at_ms: 100,
        expires_at_ms: 200,
        auth_status: Readiness::Ready,
        dispatch_supported: true,
        qualification: Some(AdapterQualification {
            receipt_digest: digest("qualified"),
            launch_fingerprint: qualification_fingerprint(
                &profile,
                &binding,
                &ExecutableIdentity {
                    path: format!("C:/tools/{name}.exe"),
                    version: "1.0".into(),
                    digest: digest("exe"),
                },
                &launch,
            )
            .unwrap(),
            permission_profile: PermissionProfile::Orbit,
            tool_probe_passed: true,
            cancellation_probe_passed: true,
            quiescence_probe_passed: true,
            capabilities: set(&["edit", "check"]),
            allowed_egress: BTreeSet::new(),
            capacity_pool_ids: set(&["host-worker", "account"]),
        }),
        requested_model: profile.requested_model.clone(),
        reported_model: Some(profile.requested_model.clone()),
        model_identity_level: ModelIdentityLevel::HarnessReported,
        metering_support: MeteringSupport::Unknown,
        hard_spend_limit_verified: false,
        capacity_ready: true,
    };
    ProfileCandidate {
        profile,
        binding,
        observation,
    }
}

fn fixture() -> (RoutingSnapshot, Vec<ProfileCandidate>, RoutingPolicy) {
    let candidates = vec![candidate("everyday"), candidate("strong")];
    let policy = RoutingPolicy {
        schema_version: 1,
        version: "rules-1".into(),
        mode: RoutingMode::Rules,
        everyday: candidates[0].target(),
        strong: candidates[1].target(),
        evaluated_manifest_digest: None,
        everyday_threshold_ppm: 800_000,
        unclear_ceiling_ppm: 100_000,
        advice_model: "pinned".into(),
        advice_template: "rubric-1".into(),
        disclosure_scope_digest: Some(digest("reviewed-recipient-template-v1")),
        advisor_recipient: None,
    };
    let task = TaskContract {
        schema_version: 1,
        canonicalization_version: 1,
        task_id: TaskId("task".into()),
        revision: 1,
        plan_digest: digest("plan"),
        base: BaseSnapshot {
            repository_identity: "repo".into(),
            git_revision: "abc".into(),
            snapshot_digest: digest("base"),
        },
        goal: "Format one file".into(),
        constraints: vec!["Keep behavior".into()],
        claim_roots: vec!["src".into()],
        dependencies: vec![],
        task_kind: Some(TaskKind::Formatting),
        task_kind_evidence: Some(digest("reviewed-kind")),
        required_capabilities: set(&["edit", "check"]),
        checks: vec![CheckRecipe {
            id: CheckId("check".into()),
            recipe_digest: digest("check"),
        }],
        required_resources: set(&["host-worker"]),
        skill_tool_bundle_digest: digest("tools"),
        permission_profile: PermissionProfile::Orbit,
        required_egress: BTreeSet::new(),
        required_target: None,
        strong_only: false,
        cross_component_requirement: Some(false),
        context_complete: true,
        repeatable_symptom_supplied: None,
        specific_cause_hypothesis_supplied: None,
    };
    let authorization = MissionAuthorization {
        schema_version: 1,
        domain_id: DomainId("domain".into()),
        run_id: RunId("run".into()),
        plan_id: PlanId("plan".into()),
        plan_digest: task.plan_digest.clone(),
        revision: 1,
        cancel_epoch: 0,
        allowed_task_digests: BTreeSet::from([task.digest().unwrap()]),
        allowed_profiles: candidates
            .iter()
            .map(|c| ApprovedProfile {
                target: c.target(),
                profile_digest: c.profile.digest().unwrap(),
                binding_digest: c.binding.digest().unwrap(),
            })
            .collect(),
        allowed_billing_sources: BTreeSet::from([BillingSourceId("subscription".into())]),
        allowed_billing_modes: BTreeSet::from([BillingSourceMode::Subscription]),
        permission_profile: PermissionProfile::Orbit,
        allowed_egress: BTreeSet::new(),
        minimum_model_identity: ModelIdentityLevel::HarnessReported,
        limits: MissionLimits {
            deadline_ms: 1000,
            max_workers: 1,
            max_attempts: 2,
            max_spend_nano_usd: Some(100),
            spend_guarantee: SpendGuarantee::RiskBounded,
        },
        policy_digest: policy.digest().unwrap(),
        live_advice_authorized: false,
        consent_revision: 1,
    };
    let snapshot = RoutingSnapshot {
        domain_id: authorization.domain_id.clone(),
        run_id: authorization.run_id.clone(),
        plan_id: authorization.plan_id.clone(),
        task,
        authorization,
        task_revision: 1,
        task_state_revision: 1,
        authorization_revision: 1,
        cancel_epoch: 0,
        next_ordinal: 1,
        previous_attempt: None,
        resolved_dependencies: vec![],
        qualification_digests: candidates
            .iter()
            .map(|c| {
                c.observation
                    .qualification
                    .as_ref()
                    .unwrap()
                    .digest()
                    .unwrap()
            })
            .collect(),
        now_ms: 150,
        cancelled: false,
        scope_valid: true,
        dependencies_ready: true,
        ownership_resolved: true,
        active_workers: 0,
        admitted_attempts: 0,
        spent_and_reserved_nano_usd: Some(0),
        manual_target: None,
        packet_digest: Some(digest("packet")),
        advice_request_id: Some(AdviceRequestId("request".into())),
    };
    (snapshot, candidates, policy)
}
