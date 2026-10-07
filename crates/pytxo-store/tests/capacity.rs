use std::sync::{Arc, Barrier};

use pytxo_store::capacity::{
    CapacityBindRequest, CapacityBoundReleaseRequest, CapacityOwner, CapacityPoolConfig,
    CapacityRecoveryRequest, CapacityReleaseEvidence, CapacityReleaseEvidenceKind,
    CapacityReleaseRequest, CapacityReservationRequest, CapacityReservationState,
    CapacityResourceRequest,
};
use pytxo_store::{Catalog, FlowDraftRecord};
use rusqlite::Connection;

fn pool(resource_id: &str, capacity_units: u64) -> CapacityPoolConfig {
    CapacityPoolConfig {
        resource_id: resource_id.into(),
        capacity_units,
        expected_revision: None,
        configured_at_ms: 100,
    }
}

fn request(
    reservation_id: &str,
    domain_id: &str,
    run_id: &str,
    attempt_id: &str,
    resources: &[(&str, u64)],
) -> CapacityReservationRequest {
    CapacityReservationRequest {
        reservation_id: reservation_id.into(),
        domain_id: domain_id.into(),
        run_id: run_id.into(),
        attempt_id: attempt_id.into(),
        owner: CapacityOwner {
            process_id: 41,
            process_start_identity: "host-boot-7:process-start-99".into(),
        },
        resources: resources
            .iter()
            .map(|(resource_id, units)| CapacityResourceRequest {
                resource_id: (*resource_id).into(),
                units: *units,
            })
            .collect(),
        requested_at_ms: 101,
    }
}

fn known_unused(receipt_id: &str) -> CapacityReleaseRequest {
    CapacityReleaseRequest {
        reservation_id: "reservation-a".into(),
        evidence: CapacityReleaseEvidence {
            kind: CapacityReleaseEvidenceKind::KnownUnused,
            receipt_id: receipt_id.into(),
            evidence_digest: format!("sha256:{receipt_id}"),
            observed_at_ms: 110,
        },
    }
}

fn quiescence(reservation_id: &str, receipt_id: &str) -> CapacityReleaseRequest {
    CapacityReleaseRequest {
        reservation_id: reservation_id.into(),
        evidence: CapacityReleaseEvidence {
            kind: CapacityReleaseEvidenceKind::QuiescenceReconciled,
            receipt_id: receipt_id.into(),
            evidence_digest: format!("sha256:{receipt_id}"),
            observed_at_ms: 130,
        },
    }
}

fn bound_release(proof: CapacityReleaseRequest, launch_token: &str) -> CapacityBoundReleaseRequest {
    CapacityBoundReleaseRequest {
        reservation_id: proof.reservation_id,
        domain_id: "domain-a".into(),
        run_id: "run-a".into(),
        attempt_id: "attempt-a".into(),
        owner: CapacityOwner {
            process_id: 41,
            process_start_identity: "host-boot-7:process-start-99".into(),
        },
        launch_token: launch_token.into(),
        evidence: proof.evidence,
    }
}

fn generated_v1_catalog(path: &std::path::Path) -> Connection {
    drop(Catalog::open(path).unwrap());
    let conn = Connection::open(path).unwrap();
    conn.execute_batch(
        "DROP TABLE flow_dispatch_claims;
         DROP TABLE routing_advisor_consent_stores;
         DROP TABLE hosted_workspace_ids;
         DROP TABLE capacity_reservation_resources;
         DROP TABLE capacity_reservations;
         DROP TABLE capacity_pools;
         PRAGMA user_version = 1;",
    )
    .unwrap();
    conn
}

#[test]
fn separate_connections_compete_for_one_shared_account_pool() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("catalog.db");
    Catalog::open(&path)
        .unwrap()
        .configure_capacity_pool(&pool("account:vendor:equivalent-a", 1))
        .unwrap();

    let barrier = Arc::new(Barrier::new(3));
    let handles = [("codex-profile", "run-a"), ("claude-profile", "run-b")]
        .into_iter()
        .map(|(profile_label, run_id)| {
            let path = path.clone();
            let barrier = barrier.clone();
            std::thread::spawn(move || {
                let catalog = Catalog::open(&path).unwrap();
                barrier.wait();
                catalog.reserve_capacity(&request(
                    &format!("reservation-{profile_label}"),
                    &format!("domain-{profile_label}"),
                    run_id,
                    &format!("attempt-{profile_label}"),
                    &[("account:vendor:equivalent-a", 1)],
                ))
            })
        })
        .collect::<Vec<_>>();
    barrier.wait();

    let outcomes = handles
        .into_iter()
        .map(|handle| handle.join().unwrap())
        .collect::<Vec<_>>();
    assert_eq!(outcomes.iter().filter(|result| result.is_ok()).count(), 1);
    assert_eq!(outcomes.iter().filter(|result| result.is_err()).count(), 1);

    let status = Catalog::open(&path)
        .unwrap()
        .capacity_pool_status("account:vendor:equivalent-a")
        .unwrap()
        .unwrap();
    assert_eq!(status.capacity_units, 1);
    assert_eq!(status.held_units, 1);
}

#[test]
fn reserve_is_all_or_nothing_across_the_sorted_resource_set() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("catalog.db");
    let catalog = Catalog::open(&path).unwrap();
    catalog
        .configure_capacity_pool(&pool("account:shared", 1))
        .unwrap();
    catalog
        .configure_capacity_pool(&pool("gpu:local-0", 1))
        .unwrap();
    catalog
        .reserve_capacity(&request(
            "reservation-existing",
            "domain-existing",
            "run-existing",
            "attempt-existing",
            &[("account:shared", 1)],
        ))
        .unwrap();

    let denied = request(
        "reservation-denied",
        "domain-new",
        "run-new",
        "attempt-new",
        &[("gpu:local-0", 1), ("account:shared", 1)],
    );
    assert!(catalog.reserve_capacity(&denied).is_err());
    assert!(catalog
        .capacity_reservation("reservation-denied")
        .unwrap()
        .is_none());
    let gpu = catalog
        .capacity_pool_status("gpu:local-0")
        .unwrap()
        .unwrap();
    assert_eq!(gpu.held_units, 0);
}

#[test]
fn identical_reserve_replays_but_identity_or_content_conflicts() {
    let dir = tempfile::tempdir().unwrap();
    let catalog = Catalog::open(&dir.path().join("catalog.db")).unwrap();
    catalog
        .configure_capacity_pool(&pool("worker:routed", 3))
        .unwrap();
    let original = request(
        "reservation-a",
        "domain-a",
        "run-a",
        "attempt-a",
        &[("worker:routed", 1)],
    );
    let first = catalog.reserve_capacity(&original).unwrap();
    assert_eq!(catalog.reserve_capacity(&original).unwrap(), first);

    let mut changed_owner = original.clone();
    changed_owner.owner.process_start_identity = "different-start".into();
    assert!(catalog.reserve_capacity(&changed_owner).is_err());

    let mut changed_content = original.clone();
    changed_content.resources[0].units = 2;
    assert!(catalog.reserve_capacity(&changed_content).is_err());

    let mut reused_attempt = original;
    reused_attempt.reservation_id = "reservation-other".into();
    assert!(catalog.reserve_capacity(&reused_attempt).is_err());
    assert_eq!(
        catalog
            .capacity_pool_status("worker:routed")
            .unwrap()
            .unwrap()
            .held_units,
        1
    );
}

#[test]
fn invalid_or_unconfigured_resource_requests_fail_closed() {
    let dir = tempfile::tempdir().unwrap();
    let catalog = Catalog::open(&dir.path().join("catalog.db")).unwrap();
    catalog
        .configure_capacity_pool(&pool("worker:routed", 2))
        .unwrap();

    let cases = [
        request(
            "missing",
            "domain-missing",
            "run-missing",
            "attempt-missing",
            &[("worker:missing", 1)],
        ),
        request(
            "zero",
            "domain-zero",
            "run-zero",
            "attempt-zero",
            &[("worker:routed", 0)],
        ),
        request(
            "too-large",
            "domain-large",
            "run-large",
            "attempt-large",
            &[("worker:routed", 3)],
        ),
        request(
            "overflow",
            "domain-overflow",
            "run-overflow",
            "attempt-overflow",
            &[("worker:routed", u64::MAX)],
        ),
        request(
            "duplicate",
            "domain-duplicate",
            "run-duplicate",
            "attempt-duplicate",
            &[("worker:routed", 1), ("worker:routed", 1)],
        ),
        request("empty", "domain-empty", "run-empty", "attempt-empty", &[]),
    ];
    for invalid in cases {
        assert!(
            catalog.reserve_capacity(&invalid).is_err(),
            "invalid request {} was accepted",
            invalid.reservation_id
        );
    }
    assert!(catalog
        .configure_capacity_pool(&pool("too-wide", u64::MAX))
        .is_err());
    assert!(catalog
        .unresolved_capacity_reservations()
        .unwrap()
        .is_empty());
}

#[test]
fn bind_is_scoped_to_original_attempt_and_one_immutable_launch_token() {
    let dir = tempfile::tempdir().unwrap();
    let catalog = Catalog::open(&dir.path().join("catalog.db")).unwrap();
    catalog
        .configure_capacity_pool(&pool("worker:routed", 1))
        .unwrap();
    catalog
        .reserve_capacity(&request(
            "reservation-a",
            "domain-a",
            "run-a",
            "attempt-a",
            &[("worker:routed", 1)],
        ))
        .unwrap();

    assert!(catalog
        .bind_capacity_reservation(&CapacityBindRequest {
            reservation_id: "reservation-a".into(),
            attempt_id: "attempt-other".into(),
            launch_token: "launch-a".into(),
            bound_at_ms: 105,
        })
        .is_err());
    let binding = CapacityBindRequest {
        reservation_id: "reservation-a".into(),
        attempt_id: "attempt-a".into(),
        launch_token: "launch-a".into(),
        bound_at_ms: 106,
    };
    let bound = catalog.bind_capacity_reservation(&binding).unwrap();
    assert_eq!(bound.state, CapacityReservationState::Bound);
    assert_eq!(catalog.bind_capacity_reservation(&binding).unwrap(), bound);

    let mut conflicting = binding;
    conflicting.launch_token = "launch-b".into();
    assert!(catalog.bind_capacity_reservation(&conflicting).is_err());
}

#[test]
fn bound_never_launched_reservation_accepts_positive_known_unused_evidence() {
    let dir = tempfile::tempdir().unwrap();
    let catalog = Catalog::open(&dir.path().join("catalog.db")).unwrap();
    catalog
        .configure_capacity_pool(&pool("worker:routed", 1))
        .unwrap();
    catalog
        .reserve_capacity(&request(
            "reservation-a",
            "domain-a",
            "run-a",
            "attempt-a",
            &[("worker:routed", 1)],
        ))
        .unwrap();
    catalog
        .bind_capacity_reservation(&CapacityBindRequest {
            reservation_id: "reservation-a".into(),
            attempt_id: "attempt-a".into(),
            launch_token: "launch-never-consumed".into(),
            bound_at_ms: 105,
        })
        .unwrap();

    let mut invalid = known_unused("positive-no-launch");
    invalid.evidence.evidence_digest.clear();
    assert!(catalog.release_capacity_reservation(&invalid).is_err());
    assert_eq!(
        catalog
            .capacity_pool_status("worker:routed")
            .unwrap()
            .unwrap()
            .held_units,
        1
    );

    let proof = known_unused("positive-no-launch");
    assert!(catalog.release_capacity_reservation(&proof).is_err());
    assert_eq!(
        catalog
            .capacity_pool_status("worker:routed")
            .unwrap()
            .unwrap()
            .held_units,
        1
    );
    let exact = bound_release(proof.clone(), "launch-never-consumed");
    let mut wrong_domain = exact.clone();
    wrong_domain.domain_id = "domain-b".into();
    let mut wrong_run = exact.clone();
    wrong_run.run_id = "run-b".into();
    let mut wrong_attempt = exact.clone();
    wrong_attempt.attempt_id = "attempt-b".into();
    let mut wrong_process = exact.clone();
    wrong_process.owner.process_id += 1;
    let mut wrong_start = exact.clone();
    wrong_start.owner.process_start_identity = "different-process-start".into();
    for wrong in [
        wrong_domain,
        wrong_run,
        wrong_attempt,
        wrong_process,
        wrong_start,
    ] {
        assert!(catalog.release_bound_capacity_reservation(&wrong).is_err());
    }
    assert!(catalog
        .release_bound_capacity_reservation(&bound_release(proof.clone(), "wrong-token"))
        .is_err());
    let released = catalog.release_bound_capacity_reservation(&exact).unwrap();
    assert_eq!(released.state, CapacityReservationState::Released);
    assert_eq!(
        catalog.release_bound_capacity_reservation(&exact).unwrap(),
        released
    );
    assert!(catalog.release_capacity_reservation(&proof).is_err());
    assert!(catalog
        .release_bound_capacity_reservation(&bound_release(
            known_unused("changed-no-launch-proof"),
            "launch-never-consumed"
        ))
        .is_err());
}

#[test]
fn release_requires_positive_state_appropriate_evidence_and_is_immutable() {
    let dir = tempfile::tempdir().unwrap();
    let catalog = Catalog::open(&dir.path().join("catalog.db")).unwrap();
    catalog
        .configure_capacity_pool(&pool("worker:routed", 2))
        .unwrap();
    catalog
        .reserve_capacity(&request(
            "reservation-a",
            "domain-a",
            "run-a",
            "attempt-a",
            &[("worker:routed", 1)],
        ))
        .unwrap();

    assert!(catalog
        .release_capacity_reservation(&quiescence("reservation-a", "receipt-wrong-state"))
        .is_err());
    let released = catalog
        .release_capacity_reservation(&known_unused("receipt-unused"))
        .unwrap();
    assert_eq!(released.state, CapacityReservationState::Released);
    assert_eq!(
        catalog
            .release_capacity_reservation(&known_unused("receipt-unused"))
            .unwrap(),
        released
    );
    assert!(catalog
        .release_capacity_reservation(&known_unused("receipt-different"))
        .is_err());
}

#[test]
fn recovery_required_survives_reopen_and_holds_until_positive_release() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("catalog.db");
    let catalog = Catalog::open(&path).unwrap();
    catalog
        .configure_capacity_pool(&pool("worker:routed", 1))
        .unwrap();
    catalog
        .reserve_capacity(&request(
            "reservation-recovery",
            "domain-a",
            "run-a",
            "attempt-a",
            &[("worker:routed", 1)],
        ))
        .unwrap();
    catalog
        .bind_capacity_reservation(&CapacityBindRequest {
            reservation_id: "reservation-recovery".into(),
            attempt_id: "attempt-a".into(),
            launch_token: "launch-a".into(),
            bound_at_ms: 102,
        })
        .unwrap();
    catalog
        .mark_capacity_recovery_required(&CapacityRecoveryRequest {
            reservation_id: "reservation-recovery".into(),
            evidence_id: "owner-inaccessible".into(),
            reason: "domain store could not be inspected".into(),
            observed_at_ms: 103,
        })
        .unwrap();
    drop(catalog);

    let reopened = Catalog::open(&path).unwrap();
    let held = reopened.unresolved_capacity_reservations().unwrap();
    assert_eq!(held.len(), 1);
    assert_eq!(held[0].state, CapacityReservationState::RecoveryRequired);
    assert!(reopened
        .reserve_capacity(&request(
            "reservation-blocked",
            "domain-b",
            "run-b",
            "attempt-b",
            &[("worker:routed", 1)],
        ))
        .is_err());
    let mut invalid_no_launch = CapacityReleaseRequest {
        reservation_id: "reservation-recovery".into(),
        ..known_unused("receipt-no-launch")
    };
    invalid_no_launch.evidence.receipt_id.clear();
    assert!(reopened
        .release_capacity_reservation(&invalid_no_launch)
        .is_err());
    assert_eq!(
        reopened.unresolved_capacity_reservations().unwrap().len(),
        1
    );
    let proof = CapacityReleaseRequest {
        reservation_id: "reservation-recovery".into(),
        ..known_unused("receipt-no-launch")
    };
    assert!(reopened.release_capacity_reservation(&proof).is_err());
    reopened
        .release_bound_capacity_reservation(&bound_release(proof, "launch-a"))
        .unwrap();
    assert!(reopened
        .unresolved_capacity_reservations()
        .unwrap()
        .is_empty());
}

#[test]
fn recovery_catalog_open_requires_existing_file_and_preserves_canonical_locator() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("catalog.db");
    assert!(Catalog::open_existing_for_capacity_recovery(&path).is_err());
    assert!(
        !path.exists(),
        "recovery must not create a replacement Catalog"
    );

    let catalog = Catalog::open(&path).unwrap();
    let expected = path.canonicalize().unwrap();
    assert_eq!(catalog.opened_path(), expected);
    drop(catalog);
    let reopened = Catalog::open_existing_for_capacity_recovery(&path).unwrap();
    assert_eq!(reopened.opened_path(), expected);
}

#[test]
fn quiescence_releases_bound_and_recovery_required_reservations() {
    for requires_recovery in [false, true] {
        let dir = tempfile::tempdir().unwrap();
        let catalog = Catalog::open(&dir.path().join("catalog.db")).unwrap();
        catalog
            .configure_capacity_pool(&pool("worker:routed", 1))
            .unwrap();
        catalog
            .reserve_capacity(&request(
                "reservation-a",
                "domain-a",
                "run-a",
                "attempt-a",
                &[("worker:routed", 1)],
            ))
            .unwrap();
        catalog
            .bind_capacity_reservation(&CapacityBindRequest {
                reservation_id: "reservation-a".into(),
                attempt_id: "attempt-a".into(),
                launch_token: "launch-a".into(),
                bound_at_ms: 102,
            })
            .unwrap();
        if requires_recovery {
            catalog
                .mark_capacity_recovery_required(&CapacityRecoveryRequest {
                    reservation_id: "reservation-a".into(),
                    evidence_id: "owner-inaccessible".into(),
                    reason: "worker state needs reconciliation".into(),
                    observed_at_ms: 103,
                })
                .unwrap();
        }
        assert_eq!(
            catalog
                .capacity_pool_status("worker:routed")
                .unwrap()
                .unwrap()
                .held_units,
            1
        );

        let proof = quiescence("reservation-a", "worker-and-descendants-quiescent");
        assert!(catalog.release_capacity_reservation(&proof).is_err());
        let exact = bound_release(proof.clone(), "launch-a");
        let released = catalog.release_bound_capacity_reservation(&exact).unwrap();
        assert_eq!(released.state, CapacityReservationState::Released);
        assert_eq!(released.release_evidence.as_ref(), Some(&proof.evidence));
        assert_eq!(
            catalog
                .capacity_pool_status("worker:routed")
                .unwrap()
                .unwrap()
                .held_units,
            0
        );
        assert_eq!(
            catalog.release_bound_capacity_reservation(&exact).unwrap(),
            released
        );
        assert!(catalog
            .release_bound_capacity_reservation(&bound_release(
                quiescence("reservation-a", "different-receipt"),
                "launch-a"
            ))
            .is_err());
    }
}

#[test]
fn provisional_recovery_accepts_positive_never_launched_evidence() {
    let dir = tempfile::tempdir().unwrap();
    let catalog = Catalog::open(&dir.path().join("catalog.db")).unwrap();
    catalog
        .configure_capacity_pool(&pool("worker:routed", 1))
        .unwrap();
    catalog
        .reserve_capacity(&request(
            "reservation-recovery",
            "domain-a",
            "run-a",
            "attempt-a",
            &[("worker:routed", 1)],
        ))
        .unwrap();
    catalog
        .mark_capacity_recovery_required(&CapacityRecoveryRequest {
            reservation_id: "reservation-recovery".into(),
            evidence_id: "domain-store-inaccessible".into(),
            reason: "admission outcome initially unknown".into(),
            observed_at_ms: 103,
        })
        .unwrap();

    assert_eq!(catalog.unresolved_capacity_reservations().unwrap().len(), 1);
    let released = catalog
        .release_capacity_reservation(&CapacityReleaseRequest {
            reservation_id: "reservation-recovery".into(),
            ..known_unused("positive-never-launched")
        })
        .unwrap();
    assert_eq!(released.state, CapacityReservationState::Released);
}

#[test]
fn age_alone_never_reclaims_a_provisional_reservation() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("catalog.db");
    let catalog = Catalog::open(&path).unwrap();
    catalog
        .configure_capacity_pool(&pool("worker:routed", 1))
        .unwrap();
    let mut old = request(
        "reservation-old",
        "domain-old",
        "run-old",
        "attempt-old",
        &[("worker:routed", 1)],
    );
    old.requested_at_ms = 1;
    catalog.reserve_capacity(&old).unwrap();
    drop(catalog);

    let reopened = Catalog::open(&path).unwrap();
    assert!(reopened
        .reserve_capacity(&request(
            "reservation-new",
            "domain-new",
            "run-new",
            "attempt-new",
            &[("worker:routed", 1)],
        ))
        .is_err());
    assert_eq!(
        reopened.unresolved_capacity_reservations().unwrap().len(),
        1
    );
}

#[test]
fn lowering_configuration_below_occupancy_preserves_existing_holds() {
    let dir = tempfile::tempdir().unwrap();
    let catalog = Catalog::open(&dir.path().join("catalog.db")).unwrap();
    catalog
        .configure_capacity_pool(&pool("worker:routed", 2))
        .unwrap();
    catalog
        .reserve_capacity(&request(
            "reservation-a",
            "domain-a",
            "run-a",
            "attempt-a",
            &[("worker:routed", 2)],
        ))
        .unwrap();

    catalog
        .configure_capacity_pool(&CapacityPoolConfig {
            expected_revision: Some(1),
            configured_at_ms: 120,
            ..pool("worker:routed", 1)
        })
        .unwrap();
    let status = catalog
        .capacity_pool_status("worker:routed")
        .unwrap()
        .unwrap();
    assert_eq!(status.capacity_units, 1);
    assert_eq!(status.held_units, 2);
    assert_eq!(status.available_units, 0);
    assert_eq!(status.revision, 2);
    assert!(catalog
        .configure_capacity_pool(&CapacityPoolConfig {
            capacity_units: 10,
            expected_revision: Some(1),
            configured_at_ms: 121,
            ..pool("worker:routed", 1)
        })
        .is_err());
    assert_eq!(
        catalog
            .capacity_pool_status("worker:routed")
            .unwrap()
            .unwrap()
            .capacity_units,
        1
    );
    assert!(catalog
        .reserve_capacity(&request(
            "reservation-b",
            "domain-b",
            "run-b",
            "attempt-b",
            &[("worker:routed", 1)],
        ))
        .is_err());
}

#[test]
fn catalog_v1_migrates_to_v8_without_changing_flow_data() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("catalog.db");
    let conn = generated_v1_catalog(&path);
    conn.execute_batch(
        "INSERT INTO flow_drafts VALUES (
            'flow-kept', 'Kept', 'mission', 'text', '/repo/a', NULL, 'draft',
            NULL, NULL, '2026-01-01Z', '2026-01-02Z'
        );",
    )
    .unwrap();
    drop(conn);

    let catalog = Catalog::open(&path).unwrap();
    assert_eq!(
        catalog.get_flow_draft("flow-kept").unwrap().unwrap().title,
        "Kept"
    );
    catalog
        .configure_capacity_pool(&pool("worker:routed", 1))
        .unwrap();
    drop(catalog);

    let conn = Connection::open(path).unwrap();
    let version: i64 = conn
        .query_row("PRAGMA user_version", [], |row| row.get(0))
        .unwrap();
    assert_eq!(version, 10);
}

#[test]
fn capacity_recovery_opens_v2_without_migrating_and_normal_open_upgrades_to_v8() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("catalog.db");
    drop(Catalog::open(&path).unwrap());
    let conn = Connection::open(&path).unwrap();
    conn.execute_batch("DROP TABLE flow_dispatch_claims; DROP TABLE routing_advisor_consent_stores; DROP TABLE hosted_workspace_ids; PRAGMA user_version = 2;")
        .unwrap();
    drop(conn);
    drop(Catalog::open_existing_for_capacity_recovery(&path).unwrap());
    let conn = Connection::open(&path).unwrap();
    let version: i64 = conn
        .query_row("PRAGMA user_version", [], |row| row.get(0))
        .unwrap();
    assert_eq!(version, 2);
    drop(conn);
    drop(Catalog::open(&path).unwrap());
    let conn = Connection::open(&path).unwrap();
    let version: i64 = conn
        .query_row("PRAGMA user_version", [], |row| row.get(0))
        .unwrap();
    assert_eq!(version, 10);
}

#[test]
fn v3_owner_claims_upgrade_to_v8_without_losing_exact_owner() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("catalog.db");
    drop(Catalog::open(&path).unwrap());
    let conn = Connection::open(&path).unwrap();
    conn.execute_batch(
        "DROP TABLE flow_dispatch_claims; DROP TABLE routing_advisor_consent_stores; DROP TABLE hosted_workspace_ids;",
    )
    .unwrap();
    conn.execute_batch(
        r#"CREATE TABLE flow_dispatch_claims (
    draft_id TEXT PRIMARY KEY REFERENCES flow_drafts(id) ON DELETE CASCADE,
    run_id TEXT NOT NULL UNIQUE,
    controller_pid INTEGER NOT NULL CHECK (controller_pid > 0),
    controller_start_identity TEXT NOT NULL CHECK (length(controller_start_identity) > 0),
    store_db_path TEXT NOT NULL CHECK (length(store_db_path) > 0)
)"#,
    )
    .unwrap();
    conn.execute_batch(
        "INSERT INTO flow_drafts VALUES (
             'held', 'Held', 'mission', 'text', '/repo/a', NULL, 'dispatching',
             'reviewed', 'run-held', '2026-01-01Z', '2026-01-02Z'
         );
         INSERT INTO flow_dispatch_claims VALUES (
             'held', 'run-held', 42, 'windows-filetime:123', 'C:/reviewed/store.db'
         );
         PRAGMA user_version = 3;",
    )
    .unwrap();
    drop(conn);
    drop(Catalog::open_existing_for_capacity_recovery(&path).unwrap());
    let conn = Connection::open(&path).unwrap();
    let version: i64 = conn
        .query_row("PRAGMA user_version", [], |row| row.get(0))
        .unwrap();
    assert_eq!(version, 3);
    drop(conn);
    let conn = Connection::open(&path).unwrap();
    conn.execute_batch(
        "CREATE TRIGGER reject_v4_backfill BEFORE UPDATE ON flow_dispatch_claims
         BEGIN SELECT RAISE(ABORT, 'forced backfill failure'); END;",
    )
    .unwrap();
    drop(conn);
    assert!(Catalog::open(&path).is_err());
    let conn = Connection::open(&path).unwrap();
    let version: i64 = conn
        .query_row("PRAGMA user_version", [], |row| row.get(0))
        .unwrap();
    assert_eq!(version, 3);
    let bit_columns: i64 = conn
        .query_row(
            "SELECT COUNT(*) FROM pragma_table_info('flow_dispatch_claims')
             WHERE name='startup_may_have_started'",
            [],
            |row| row.get(0),
        )
        .unwrap();
    assert_eq!(bit_columns, 0);
    conn.execute_batch("DROP TRIGGER reject_v4_backfill;")
        .unwrap();
    drop(conn);
    let catalog = Catalog::open(&path).unwrap();
    let owner = catalog
        .routed_flow_dispatch_owner("held", "run-held")
        .unwrap()
        .unwrap();
    assert_eq!(owner.controller_pid, 42);
    assert_eq!(owner.controller_start_identity, "windows-filetime:123");
    assert_eq!(owner.store_db_path, "C:/reviewed/store.db");
    assert_eq!(owner.store_db_file_identity, None);
    assert_eq!(
        catalog
            .routed_flow_startup_may_have_started("held", "run-held")
            .unwrap(),
        Some(true)
    );
    drop(catalog);
    let conn = Connection::open(&path).unwrap();
    let version: i64 = conn
        .query_row("PRAGMA user_version", [], |row| row.get(0))
        .unwrap();
    assert_eq!(version, 10);
}

#[test]
fn v4_owner_claim_migrates_as_physically_unverified() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("catalog.db");
    drop(Catalog::open(&path).unwrap());
    let conn = Connection::open(&path).unwrap();
    conn.execute_batch(
        "DROP TABLE routing_advisor_consent_stores;
         DROP TABLE hosted_workspace_ids;
         ALTER TABLE flow_dispatch_claims DROP COLUMN stop_requested;
         ALTER TABLE flow_dispatch_claims DROP COLUMN store_db_file_identity;
         INSERT INTO flow_drafts VALUES (
             'held-v4', 'Held', 'mission', 'text', '/repo/a', NULL, 'dispatching',
             'reviewed', 'run-v4', '2026-01-01Z', '2026-01-02Z'
         );
         INSERT INTO flow_dispatch_claims
             (draft_id,run_id,controller_pid,controller_start_identity,store_db_path,startup_may_have_started)
             VALUES ('held-v4','run-v4',42,'windows-filetime:123','C:/reviewed/store.db',1);
         PRAGMA user_version = 4;",
    )
    .unwrap();
    drop(conn);
    drop(Catalog::open_existing_for_capacity_recovery(&path).unwrap());
    let catalog = Catalog::open(&path).unwrap();
    let owner = catalog
        .routed_flow_dispatch_owner("held-v4", "run-v4")
        .unwrap()
        .unwrap();
    assert_eq!(owner.store_db_path, "C:/reviewed/store.db");
    assert_eq!(owner.store_db_file_identity, None);
    assert_eq!(
        catalog
            .routed_flow_startup_may_have_started("held-v4", "run-v4")
            .unwrap(),
        Some(true)
    );
    drop(catalog);
    let conn = Connection::open(&path).unwrap();
    let version: i64 = conn
        .query_row("PRAGMA user_version", [], |row| row.get(0))
        .unwrap();
    assert_eq!(version, 10);
}

#[test]
fn concurrent_v4_catalog_openers_complete_one_schema_upgrade() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("catalog.db");
    drop(Catalog::open(&path).unwrap());
    let conn = Connection::open(&path).unwrap();
    conn.execute_batch(
        "DROP TABLE routing_advisor_consent_stores;
         DROP TABLE hosted_workspace_ids;
         ALTER TABLE flow_dispatch_claims DROP COLUMN stop_requested;
         ALTER TABLE flow_dispatch_claims DROP COLUMN store_db_file_identity;
         PRAGMA user_version = 4;",
    )
    .unwrap();
    drop(conn);

    let barrier = Arc::new(Barrier::new(9));
    let handles = (0..8)
        .map(|_| {
            let path = path.clone();
            let barrier = barrier.clone();
            std::thread::spawn(move || {
                barrier.wait();
                Catalog::open(&path)
            })
        })
        .collect::<Vec<_>>();
    barrier.wait();
    for handle in handles {
        handle.join().unwrap().unwrap();
    }
    let conn = Connection::open(path).unwrap();
    let version: i64 = conn
        .query_row("PRAGMA user_version", [], |row| row.get(0))
        .unwrap();
    assert_eq!(version, 10);
}

#[test]
fn malformed_current_and_future_catalogs_fail_closed() {
    let dir = tempfile::tempdir().unwrap();
    let malformed = dir.path().join("malformed.db");
    let conn = generated_v1_catalog(&malformed);
    conn.execute_batch(
        "CREATE TABLE capacity_pools (resource_id TEXT PRIMARY KEY);
        PRAGMA user_version = 2;",
    )
    .unwrap();
    drop(conn);
    assert!(Catalog::open(&malformed).is_err());

    let future = dir.path().join("future.db");
    Connection::open(&future)
        .unwrap()
        .execute_batch("PRAGMA user_version = 11;")
        .unwrap();
    assert!(Catalog::open(&future).is_err());
}

#[test]
fn malformed_v5_owner_claim_schema_fails_closed() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("malformed-claims.db");
    drop(Catalog::open(&path).unwrap());
    let conn = Connection::open(&path).unwrap();
    conn.execute_batch("DROP TABLE flow_dispatch_claims; CREATE TABLE flow_dispatch_claims (draft_id TEXT PRIMARY KEY);")
        .unwrap();
    drop(conn);
    assert!(Catalog::open(&path).is_err());
    assert!(Catalog::open_existing_for_capacity_recovery(&path).is_err());
}

#[test]
fn current_catalog_rejects_case_changed_capacity_state_literals() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("case-changed-capacity.db");
    drop(Catalog::open(&path).unwrap());
    let conn = Connection::open(&path).unwrap();
    conn.execute_batch("PRAGMA writable_schema=ON;").unwrap();
    let changed = conn
        .execute(
            "UPDATE sqlite_master
             SET sql=replace(sql, '''bound''', '''BOUND''')
             WHERE type='table' AND name='capacity_reservations'",
            [],
        )
        .unwrap();
    assert_eq!(changed, 1);
    conn.execute_batch("PRAGMA writable_schema=OFF;").unwrap();
    drop(conn);

    assert!(Catalog::open(&path).is_err());
}

#[test]
fn v1_catalog_rejects_case_changed_flow_source_literals() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("case-changed-flow.db");
    let conn = generated_v1_catalog(&path);
    conn.execute_batch("PRAGMA writable_schema=ON;").unwrap();
    let changed = conn
        .execute(
            "UPDATE sqlite_master
             SET sql=replace(sql, '''text'', ''voice''', '''TEXT'', ''VOICE''')
             WHERE type='table' AND name='flow_drafts'",
            [],
        )
        .unwrap();
    assert_eq!(changed, 1);
    conn.execute_batch("PRAGMA writable_schema=OFF;").unwrap();
    drop(conn);

    assert!(Catalog::open(&path).is_err());
    let conn = Connection::open(path).unwrap();
    let version: i64 = conn
        .query_row("PRAGMA user_version", [], |row| row.get(0))
        .unwrap();
    assert_eq!(version, 1);
    let capacity_objects: i64 = conn
        .query_row(
            "SELECT COUNT(*) FROM sqlite_master WHERE name LIKE 'capacity_%'",
            [],
            |row| row.get(0),
        )
        .unwrap();
    assert_eq!(capacity_objects, 0);
}

#[test]
fn negative_catalog_version_fails_without_rewriting_the_version() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("negative.db");
    let conn = generated_v1_catalog(&path);
    conn.execute_batch("PRAGMA user_version = -1;").unwrap();
    drop(conn);

    assert!(Catalog::open(&path).is_err());
    let conn = Connection::open(path).unwrap();
    let version: i64 = conn
        .query_row("PRAGMA user_version", [], |row| row.get(0))
        .unwrap();
    assert_eq!(version, -1);
}

#[test]
fn malformed_v1_capacity_migration_rolls_back_and_preserves_flow_data() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("malformed-v1.db");
    let conn = generated_v1_catalog(&path);
    conn.execute_batch(
        "INSERT INTO flow_drafts VALUES (
            'flow-kept', 'Kept', 'mission', 'text', '/repo/a', NULL, 'draft',
            NULL, NULL, '2026-01-01Z', '2026-01-02Z'
        );
        CREATE TABLE capacity_pools (resource_id TEXT PRIMARY KEY);
        ",
    )
    .unwrap();
    drop(conn);

    assert!(Catalog::open(&path).is_err());
    let conn = Connection::open(path).unwrap();
    let version: i64 = conn
        .query_row("PRAGMA user_version", [], |row| row.get(0))
        .unwrap();
    assert_eq!(version, 1);
    let flow_title: String = conn
        .query_row(
            "SELECT title FROM flow_drafts WHERE id='flow-kept'",
            [],
            |row| row.get(0),
        )
        .unwrap();
    assert_eq!(flow_title, "Kept");
    let reservations_created: bool = conn
        .query_row(
            "SELECT EXISTS(SELECT 1 FROM sqlite_master
             WHERE type='table' AND name='capacity_reservations')",
            [],
            |row| row.get(0),
        )
        .unwrap();
    assert!(!reservations_created);
    let malformed_pool_columns: i64 = conn
        .query_row(
            "SELECT COUNT(*) FROM pragma_table_info('capacity_pools')",
            [],
            |row| row.get(0),
        )
        .unwrap();
    assert_eq!(malformed_pool_columns, 1);
}

#[test]
fn flow_crud_still_works_after_capacity_migration() {
    let dir = tempfile::tempdir().unwrap();
    let catalog = Catalog::open(&dir.path().join("catalog.db")).unwrap();
    let flow = FlowDraftRecord {
        id: "flow-after-v2".into(),
        title: "Flow remains intact".into(),
        mission_text: "Keep Flow behavior".into(),
        source: "text".into(),
        domain_id: Some("/repo/a".into()),
        project_id: None,
        status: "draft".into(),
        plan_json: None,
        dispatched_run_id: None,
        created_at: "2026-01-01Z".into(),
        updated_at: "2026-01-01Z".into(),
    };
    catalog.upsert_flow_draft(&flow).unwrap();
    assert_eq!(catalog.get_flow_draft(&flow.id).unwrap(), Some(flow));
}
