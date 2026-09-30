use pytxo_store::{
    Catalog, HostedAdvisorConsentFence, HostedAdvisorConsentReview, HostedGrantIntent,
    HostedGrantState, HostedRemoteGrantReceipt,
};
use rusqlite::Connection;

#[test]
fn catalog_has_durable_hosted_grant_reconciliation_schema() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("hypervisor.db");
    let _catalog = Catalog::open(&path).unwrap();
    let conn = Connection::open(&path).unwrap();
    let version: i64 = conn
        .query_row("PRAGMA user_version", [], |row| row.get(0))
        .unwrap();
    assert_eq!(version, 10);
    let sql: String = conn
        .query_row(
            "SELECT sql FROM sqlite_master WHERE type='table' AND name='hosted_routing_grants'",
            [],
            |row| row.get(0),
        )
        .unwrap();
    for column in [
        "domain_id",
        "workspace_id",
        "account_id",
        "link_origin",
        "consent_revision",
        "remote_revision",
        "state",
    ] {
        assert!(sql.contains(column), "missing {column}: {sql}");
    }
    assert!(
        conn.execute(
            "INSERT INTO hosted_routing_grants (
                domain_id, workspace_id, account_id, link_origin, recipient_identity,
                scope_digest, store_db_file_identity, consent_revision, remote_revision,
                state, updated_at_ms
             ) VALUES ('bad', ?1, 'user_one', 'https://link.pytxo.com', 'hosted', ?2,
                       'file-a', 1, NULL, 'enabled', 1)",
            rusqlite::params!["0".repeat(32), "a".repeat(64)],
        )
        .is_err(),
        "enabled grant needs a confirmed remote revision"
    );
}

fn trusted_review(catalog: &Catalog) -> HostedGrantIntent {
    catalog
        .upsert_domain("/repo/a", "/repo/a", "/repo/a/pytxo.db", None)
        .unwrap();
    catalog
        .bind_routing_advisor_consent_store("/repo/a", "/repo/a/pytxo.db", "file-a")
        .unwrap();
    let workspace_id = catalog
        .ensure_hosted_workspace_id("/repo/a", "/repo/a/pytxo.db", "file-a")
        .unwrap();
    let recipient_identity = "pytxo-hosted-routing/typesafe-systemone/v1";
    catalog
        .record_hosted_advisor_consent_review(&HostedAdvisorConsentReview {
            domain_id: "/repo/a".into(),
            recipient_identity: recipient_identity.into(),
            draft_id: "reviewed-flow".into(),
            consent_revision: 1,
            scope_digest: "a".repeat(64),
            packet_digest: "b".repeat(64),
            request_digest: "c".repeat(64),
            store_db_file_identity: "file-a".into(),
        })
        .unwrap();
    catalog
        .advance_hosted_advisor_consent_fence(&HostedAdvisorConsentFence {
            domain_id: "/repo/a".into(),
            recipient_identity: recipient_identity.into(),
            consent_revision: 1,
            enabled: true,
            store_db_file_identity: "file-a".into(),
        })
        .unwrap();
    HostedGrantIntent {
        domain_id: "/repo/a".into(),
        workspace_id,
        account_id: "user_one".into(),
        link_origin: "https://link.pytxo.com".into(),
        recipient_identity: recipient_identity.into(),
        scope_digest: "a".repeat(64),
        store_db_file_identity: "file-a".into(),
        consent_revision: 1,
    }
}

fn receipt(intent: &HostedGrantIntent, revision: u64, enabled: bool) -> HostedRemoteGrantReceipt {
    HostedRemoteGrantReceipt {
        workspace_id: intent.workspace_id.clone(),
        account_id: intent.account_id.clone(),
        link_origin: intent.link_origin.clone(),
        recipient_identity: intent.recipient_identity.clone(),
        scope_digest: intent.scope_digest.clone(),
        revision,
        enabled,
    }
}

#[test]
fn hosted_send_gate_requires_the_exact_current_review_and_confirmed_grant() {
    let dir = tempfile::tempdir().unwrap();
    let catalog = Catalog::open(&dir.path().join("hypervisor.db")).unwrap();
    let intent = trusted_review(&catalog);
    let review = catalog
        .hosted_advisor_consent_review(&intent.domain_id)
        .unwrap()
        .unwrap();

    assert!(catalog
        .confirmed_hosted_grant_for_send(&intent, &review)
        .is_err());
    catalog.begin_hosted_grant(&intent).unwrap();
    assert!(catalog
        .confirmed_hosted_grant_for_send(&intent, &review)
        .is_err());
    let enabled = catalog
        .confirm_hosted_grant(&intent, &receipt(&intent, 1, true))
        .unwrap();
    assert_eq!(
        catalog
            .confirmed_hosted_grant_for_send(&intent, &review)
            .unwrap(),
        enabled
    );

    for changed in [
        HostedAdvisorConsentReview {
            packet_digest: "d".repeat(64),
            ..review.clone()
        },
        HostedAdvisorConsentReview {
            request_digest: "d".repeat(64),
            ..review.clone()
        },
        HostedAdvisorConsentReview {
            draft_id: "another-flow".into(),
            ..review.clone()
        },
        HostedAdvisorConsentReview {
            store_db_file_identity: "replacement".into(),
            ..review.clone()
        },
    ] {
        assert!(catalog
            .confirmed_hosted_grant_for_send(&intent, &changed)
            .is_err());
    }
    let mut switched = intent.clone();
    switched.account_id = "user_other".into();
    assert!(catalog
        .confirmed_hosted_grant_for_send(&switched, &review)
        .is_err());

    catalog
        .advance_hosted_advisor_consent_fence(&HostedAdvisorConsentFence {
            domain_id: intent.domain_id.clone(),
            recipient_identity: intent.recipient_identity.clone(),
            consent_revision: 2,
            enabled: false,
            store_db_file_identity: intent.store_db_file_identity.clone(),
        })
        .unwrap();
    assert!(catalog
        .confirmed_hosted_grant_for_send(&intent, &review)
        .is_err());
}

#[test]
fn hosted_grant_intent_and_revoke_survive_restart_without_account_switch() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("hypervisor.db");
    let catalog = Catalog::open(&path).unwrap();
    let intent = trusted_review(&catalog);
    let pending = catalog.begin_hosted_grant(&intent).unwrap();
    assert_eq!(pending.state, HostedGrantState::GrantPending);
    assert_eq!(pending.remote_revision, None);
    let mut switched = intent.clone();
    switched.account_id = "user_other".into();
    assert!(catalog.begin_hosted_grant(&switched).is_err());
    drop(catalog);

    let catalog = Catalog::open(&path).unwrap();
    assert_eq!(
        catalog.hosted_grant(&intent.domain_id).unwrap().unwrap(),
        pending
    );
    let enabled = catalog
        .confirm_hosted_grant(&intent, &receipt(&intent, 1, true))
        .unwrap();
    assert_eq!(enabled.state, HostedGrantState::Enabled);
    assert_eq!(enabled.remote_revision, Some(1));
    catalog
        .advance_hosted_advisor_consent_fence(&HostedAdvisorConsentFence {
            domain_id: intent.domain_id.clone(),
            recipient_identity: intent.recipient_identity.clone(),
            consent_revision: 2,
            enabled: false,
            store_db_file_identity: intent.store_db_file_identity.clone(),
        })
        .unwrap();
    assert!(catalog
        .confirm_hosted_grant(&intent, &receipt(&intent, 1, true))
        .is_err());
    let revoking = catalog
        .begin_hosted_grant_revoke(&intent.domain_id)
        .unwrap();
    assert_eq!(revoking.state, HostedGrantState::RevokePending);
    drop(catalog);

    let catalog = Catalog::open(&path).unwrap();
    assert_eq!(
        catalog.hosted_grant(&intent.domain_id).unwrap().unwrap(),
        revoking
    );
    assert!(catalog
        .confirm_hosted_grant_revoked(&intent.domain_id, &receipt(&intent, 2, true))
        .is_err());
    let revoked = catalog
        .confirm_hosted_grant_revoked(&intent.domain_id, &receipt(&intent, 2, false))
        .unwrap();
    assert_eq!(revoked.state, HostedGrantState::Revoked);
    assert_eq!(revoked.remote_revision, Some(2));
}

#[test]
fn local_revoke_while_remote_enable_is_in_flight_retains_revoke_intent() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("hypervisor.db");
    let catalog = Catalog::open(&path).unwrap();
    let intent = trusted_review(&catalog);
    catalog.begin_hosted_grant(&intent).unwrap();

    // The Link POST has passed the caller's last local check but has not
    // returned. A separate local revoke commits before its enabled reply.
    catalog
        .advance_hosted_advisor_consent_fence(&HostedAdvisorConsentFence {
            domain_id: intent.domain_id.clone(),
            recipient_identity: intent.recipient_identity.clone(),
            consent_revision: 2,
            enabled: false,
            store_db_file_identity: intent.store_db_file_identity.clone(),
        })
        .unwrap();
    assert_eq!(
        catalog
            .hosted_grant(&intent.domain_id)
            .unwrap()
            .unwrap()
            .state,
        HostedGrantState::RevokePending
    );
    assert!(catalog
        .confirm_hosted_grant(&intent, &receipt(&intent, 1, true))
        .is_err());
    drop(catalog);

    let catalog = Catalog::open(&path).unwrap();
    assert_eq!(
        catalog
            .hosted_grant(&intent.domain_id)
            .unwrap()
            .unwrap()
            .state,
        HostedGrantState::RevokePending
    );
    assert_eq!(
        catalog
            .confirm_hosted_grant_revoked(&intent.domain_id, &receipt(&intent, 2, false))
            .unwrap()
            .state,
        HostedGrantState::Revoked
    );
}

#[test]
fn unavailable_workspace_store_can_be_fenced_for_revoke_only() {
    let dir = tempfile::tempdir().unwrap();
    let catalog = Catalog::open(&dir.path().join("hypervisor.db")).unwrap();
    let intent = trusted_review(&catalog);
    catalog.begin_hosted_grant(&intent).unwrap();
    catalog
        .confirm_hosted_grant(&intent, &receipt(&intent, 1, true))
        .unwrap();
    assert!(catalog
        .begin_hosted_grant_revoke_without_store(&intent.domain_id, 0)
        .is_err());
    assert!(
        catalog
            .hosted_advisor_consent_fence(&intent.domain_id)
            .unwrap()
            .unwrap()
            .enabled
    );
    let pending = catalog
        .begin_hosted_grant_revoke_without_store(&intent.domain_id, 1)
        .unwrap();
    assert_eq!(pending.state, HostedGrantState::RevokePending);
    let fence = catalog
        .hosted_advisor_consent_fence(&intent.domain_id)
        .unwrap()
        .unwrap();
    assert!(!fence.enabled);
    assert_eq!(fence.consent_revision, 2);
    assert!(catalog
        .confirm_hosted_grant(&intent, &receipt(&intent, 2, true))
        .is_err());
    assert!(catalog
        .begin_hosted_grant_revoke_without_store(&intent.domain_id, 1)
        .is_err());
    assert_eq!(
        catalog
            .confirm_hosted_grant_revoked(&intent.domain_id, &receipt(&intent, 2, false))
            .unwrap()
            .state,
        HostedGrantState::Revoked
    );
}

#[test]
fn pending_remote_revoke_blocks_fresh_local_enable_fence() {
    let dir = tempfile::tempdir().unwrap();
    let catalog = Catalog::open(&dir.path().join("hypervisor.db")).unwrap();
    let intent = trusted_review(&catalog);
    catalog.begin_hosted_grant(&intent).unwrap();
    catalog
        .confirm_hosted_grant(&intent, &receipt(&intent, 1, true))
        .unwrap();
    catalog
        .advance_hosted_advisor_consent_fence(&HostedAdvisorConsentFence {
            domain_id: intent.domain_id.clone(),
            recipient_identity: intent.recipient_identity.clone(),
            consent_revision: 2,
            enabled: false,
            store_db_file_identity: intent.store_db_file_identity.clone(),
        })
        .unwrap();
    catalog
        .begin_hosted_grant_revoke(&intent.domain_id)
        .unwrap();
    let attempted_reenable = HostedAdvisorConsentFence {
        domain_id: intent.domain_id.clone(),
        recipient_identity: intent.recipient_identity.clone(),
        consent_revision: 3,
        enabled: true,
        store_db_file_identity: intent.store_db_file_identity.clone(),
    };
    assert!(catalog
        .advance_hosted_advisor_consent_fence(&attempted_reenable)
        .is_err());
    assert_eq!(
        catalog
            .hosted_advisor_consent_fence(&intent.domain_id)
            .unwrap()
            .unwrap()
            .consent_revision,
        2
    );
    assert_eq!(
        catalog
            .confirm_hosted_grant_revoked(&intent.domain_id, &receipt(&intent, 2, false))
            .unwrap()
            .state,
        HostedGrantState::Revoked
    );
    catalog
        .advance_hosted_advisor_consent_fence(&attempted_reenable)
        .unwrap();
}

#[test]
fn grant_receipt_must_bind_account_workspace_recipient_scope_and_revision() {
    let dir = tempfile::tempdir().unwrap();
    let catalog = Catalog::open(&dir.path().join("hypervisor.db")).unwrap();
    let intent = trusted_review(&catalog);
    catalog.begin_hosted_grant(&intent).unwrap();
    let good = receipt(&intent, 1, true);
    for bad in [
        HostedRemoteGrantReceipt {
            account_id: "user_other".into(),
            ..good.clone()
        },
        HostedRemoteGrantReceipt {
            link_origin: "https://other.test".into(),
            ..good.clone()
        },
        HostedRemoteGrantReceipt {
            workspace_id: "b".repeat(32),
            ..good.clone()
        },
        HostedRemoteGrantReceipt {
            recipient_identity: "other".into(),
            ..good.clone()
        },
        HostedRemoteGrantReceipt {
            scope_digest: "b".repeat(64),
            ..good.clone()
        },
        HostedRemoteGrantReceipt {
            revision: 0,
            ..good.clone()
        },
        HostedRemoteGrantReceipt {
            enabled: false,
            ..good.clone()
        },
    ] {
        assert!(
            catalog.confirm_hosted_grant(&intent, &bad).is_err(),
            "accepted {bad:?}"
        );
    }
    assert_eq!(
        catalog
            .hosted_grant(&intent.domain_id)
            .unwrap()
            .unwrap()
            .state,
        HostedGrantState::GrantPending
    );
    catalog.confirm_hosted_grant(&intent, &good).unwrap();
}

#[test]
fn regrant_preserves_remote_revision_after_confirmed_revoke() {
    let dir = tempfile::tempdir().unwrap();
    let catalog = Catalog::open(&dir.path().join("hypervisor.db")).unwrap();
    let old = trusted_review(&catalog);
    catalog.begin_hosted_grant(&old).unwrap();
    catalog
        .confirm_hosted_grant(&old, &receipt(&old, 1, true))
        .unwrap();
    catalog
        .advance_hosted_advisor_consent_fence(&HostedAdvisorConsentFence {
            domain_id: old.domain_id.clone(),
            recipient_identity: old.recipient_identity.clone(),
            consent_revision: 2,
            enabled: false,
            store_db_file_identity: old.store_db_file_identity.clone(),
        })
        .unwrap();
    catalog.begin_hosted_grant_revoke(&old.domain_id).unwrap();
    catalog
        .confirm_hosted_grant_revoked(&old.domain_id, &receipt(&old, 2, false))
        .unwrap();
    catalog
        .record_hosted_advisor_consent_review(&HostedAdvisorConsentReview {
            domain_id: old.domain_id.clone(),
            recipient_identity: old.recipient_identity.clone(),
            draft_id: "new-reviewed-flow".into(),
            consent_revision: 3,
            scope_digest: old.scope_digest.clone(),
            packet_digest: "b".repeat(64),
            request_digest: "c".repeat(64),
            store_db_file_identity: old.store_db_file_identity.clone(),
        })
        .unwrap();
    catalog
        .advance_hosted_advisor_consent_fence(&HostedAdvisorConsentFence {
            domain_id: old.domain_id.clone(),
            recipient_identity: old.recipient_identity.clone(),
            consent_revision: 3,
            enabled: true,
            store_db_file_identity: old.store_db_file_identity.clone(),
        })
        .unwrap();
    let mut new = old.clone();
    new.consent_revision = 3;
    let pending = catalog.begin_hosted_grant(&new).unwrap();
    assert_eq!(pending.remote_revision, Some(2));
    assert_eq!(pending.state, HostedGrantState::GrantPending);
    assert!(catalog
        .confirm_hosted_grant(&new, &receipt(&new, 1, true))
        .is_err());
    let enabled = catalog
        .confirm_hosted_grant(&new, &receipt(&new, 3, true))
        .unwrap();
    assert_eq!(enabled.remote_revision, Some(3));
}

#[test]
fn uncertain_first_post_stays_revoke_pending_until_bound_disabled_tombstone() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("hypervisor.db");
    let catalog = Catalog::open(&path).unwrap();
    let intent = trusted_review(&catalog);
    catalog.begin_hosted_grant(&intent).unwrap();
    catalog
        .advance_hosted_advisor_consent_fence(&HostedAdvisorConsentFence {
            domain_id: intent.domain_id.clone(),
            recipient_identity: intent.recipient_identity.clone(),
            consent_revision: 2,
            enabled: false,
            store_db_file_identity: intent.store_db_file_identity.clone(),
        })
        .unwrap();
    let pending = catalog
        .begin_hosted_grant_revoke(&intent.domain_id)
        .unwrap();
    assert_eq!(pending.remote_revision, None);
    assert_eq!(pending.state, HostedGrantState::RevokePending);
    let mut wrong = receipt(&intent, 1, false);
    wrong.account_id = "user_other".into();
    assert!(catalog
        .confirm_hosted_grant_revoked(&intent.domain_id, &wrong)
        .is_err());
    drop(catalog);
    let catalog = Catalog::open(&path).unwrap();
    assert_eq!(
        catalog.hosted_grant(&intent.domain_id).unwrap().unwrap(),
        pending
    );
    let mut scope_changed = receipt(&intent, 1, false);
    scope_changed.scope_digest = "b".repeat(64);
    let revoked = catalog
        .confirm_hosted_grant_revoked(&intent.domain_id, &scope_changed)
        .unwrap();
    assert_eq!(revoked.state, HostedGrantState::Revoked);
    assert_eq!(revoked.remote_revision, Some(1));
}
