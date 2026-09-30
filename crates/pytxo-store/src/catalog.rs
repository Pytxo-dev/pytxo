//! Cross-project hypervisor catalog (`~/.pytxo/hypervisor.db`).
//!
//! A lightweight registry of every execution domain the local hypervisor has
//! seen, so the Reality Deck can show an "all projects" home without merging the
//! per-domain WAL streams ([[execution-domains]] v2). Each domain keeps its own
//! `pytxo.db`; this catalog only records where to find them.

use std::path::{Path, PathBuf};

use chrono::Utc;
use pytxo_core::{PytxoError, Result};
use rusqlite::{params, Connection, OptionalExtension, Transaction, TransactionBehavior};
use serde::{Deserialize, Serialize};

const CATALOG_SCHEMA: &str = r#"
CREATE TABLE IF NOT EXISTS domains (
    domain_id TEXT PRIMARY KEY,
    repo_root TEXT NOT NULL,
    db_path TEXT NOT NULL,
    project_id TEXT,
    status TEXT NOT NULL DEFAULT 'active',
    updated_at TEXT NOT NULL
);

CREATE TABLE IF NOT EXISTS fleet_runs (
    id TEXT PRIMARY KEY,
    fleet_id TEXT NOT NULL,
    started_at TEXT NOT NULL,
    finished_at TEXT,
    status TEXT NOT NULL
);

CREATE TABLE IF NOT EXISTS fleet_nodes (
    fleet_run_id TEXT NOT NULL,
    node_id TEXT NOT NULL,
    domain_id TEXT NOT NULL,
    domain_run_id TEXT,
    wave INTEGER NOT NULL,
    status TEXT NOT NULL,
    PRIMARY KEY (fleet_run_id, node_id)
);

CREATE INDEX IF NOT EXISTS idx_fleet_runs_fleet_id ON fleet_runs(fleet_id);
CREATE INDEX IF NOT EXISTS idx_fleet_nodes_run ON fleet_nodes(fleet_run_id);

CREATE TABLE IF NOT EXISTS project_roots (
    project_id TEXT NOT NULL,
    label TEXT NOT NULL,
    path TEXT NOT NULL,
    read_only INTEGER NOT NULL DEFAULT 0,
    primary_root INTEGER NOT NULL DEFAULT 0,
    permission_profile TEXT,
    updated_at TEXT NOT NULL,
    PRIMARY KEY (project_id, label)
);

CREATE INDEX IF NOT EXISTS idx_project_roots_project ON project_roots(project_id);
"#;

const FLOW_DRAFTS_MIGRATION: &str = r#"
CREATE TABLE IF NOT EXISTS flow_drafts (
    id TEXT PRIMARY KEY,
    title TEXT NOT NULL,
    mission_text TEXT NOT NULL,
    source TEXT NOT NULL CHECK (source IN ('text', 'voice')),
    domain_id TEXT,
    project_id TEXT,
    status TEXT NOT NULL,
    plan_json TEXT,
    dispatched_run_id TEXT,
    created_at TEXT NOT NULL,
    updated_at TEXT NOT NULL
);
CREATE INDEX IF NOT EXISTS idx_flow_drafts_updated_at ON flow_drafts(updated_at DESC);
PRAGMA user_version = 1;
"#;

const ROUTED_FLOW_DISPATCH_CLAIMS_TABLE_V3: &str = r#"
CREATE TABLE flow_dispatch_claims (
    draft_id TEXT PRIMARY KEY REFERENCES flow_drafts(id) ON DELETE CASCADE,
    run_id TEXT NOT NULL UNIQUE,
    controller_pid INTEGER NOT NULL CHECK (controller_pid > 0),
    controller_start_identity TEXT NOT NULL CHECK (length(controller_start_identity) > 0),
    store_db_path TEXT NOT NULL CHECK (length(store_db_path) > 0)
)"#;

const ROUTED_FLOW_DISPATCH_CLAIMS_V4: &str = "ALTER TABLE flow_dispatch_claims
    ADD COLUMN startup_may_have_started INTEGER NOT NULL DEFAULT 0
    CHECK (startup_may_have_started IN (0, 1))";

const ROUTED_FLOW_DISPATCH_CLAIMS_V5: &str = "ALTER TABLE flow_dispatch_claims
    ADD COLUMN store_db_file_identity TEXT
    CHECK (store_db_file_identity IS NULL OR length(store_db_file_identity) > 0)";

const ROUTED_FLOW_DISPATCH_CLAIMS_V9: &str = "ALTER TABLE flow_dispatch_claims
    ADD COLUMN stop_requested INTEGER NOT NULL DEFAULT 0
    CHECK (stop_requested IN (0, 1))";

const ROUTING_ADVISOR_CONSENT_STORES_V6: &str = "CREATE TABLE routing_advisor_consent_stores (
    domain_id TEXT PRIMARY KEY CHECK (length(domain_id) > 0),
    store_db_path TEXT NOT NULL CHECK (length(store_db_path) > 0),
    store_db_file_identity TEXT NOT NULL CHECK (length(store_db_file_identity) > 0)
)";

const HOSTED_WORKSPACE_IDS_V7: &str = "CREATE TABLE hosted_workspace_ids (
    domain_id TEXT PRIMARY KEY NOT NULL CHECK (length(domain_id) > 0),
    workspace_id TEXT NOT NULL UNIQUE CHECK (length(workspace_id) = 32 AND workspace_id NOT GLOB '*[^0-9a-f]*')
)";

const HOSTED_ADVISOR_CONSENT_REVIEWS_V8: &str =
    "CREATE TABLE IF NOT EXISTS routing_hosted_advisor_consent_reviews (
    domain_id TEXT PRIMARY KEY CHECK (length(domain_id) > 0),
    recipient_identity TEXT NOT NULL CHECK (length(recipient_identity) > 0),
    draft_id TEXT NOT NULL CHECK (length(draft_id) > 0),
    consent_revision INTEGER NOT NULL CHECK (consent_revision > 0),
    scope_digest TEXT NOT NULL CHECK (length(scope_digest) = 64),
    packet_digest TEXT NOT NULL CHECK (length(packet_digest) = 64),
    request_digest TEXT NOT NULL CHECK (length(request_digest) = 64),
    store_db_file_identity TEXT NOT NULL CHECK (length(store_db_file_identity) > 0)
)";

const HOSTED_ADVISOR_CONSENT_FENCES_V8: &str =
    "CREATE TABLE IF NOT EXISTS routing_hosted_advisor_consent_fences (
    domain_id TEXT PRIMARY KEY CHECK (length(domain_id) > 0),
    recipient_identity TEXT NOT NULL CHECK (length(recipient_identity) > 0),
    consent_revision INTEGER NOT NULL CHECK (consent_revision > 0),
    enabled INTEGER NOT NULL CHECK (enabled IN (0, 1)),
    store_db_file_identity TEXT NOT NULL CHECK (length(store_db_file_identity) > 0)
)";

const HOSTED_ROUTING_GRANTS_V10: &str = "CREATE TABLE IF NOT EXISTS hosted_routing_grants (
    domain_id TEXT PRIMARY KEY NOT NULL CHECK (length(domain_id) > 0),
    workspace_id TEXT NOT NULL CHECK (length(workspace_id) = 32 AND workspace_id NOT GLOB '*[^0-9a-f]*'),
    account_id TEXT NOT NULL CHECK (length(account_id) > 0),
    link_origin TEXT NOT NULL CHECK (length(link_origin) > 0),
    recipient_identity TEXT NOT NULL CHECK (length(recipient_identity) > 0),
    scope_digest TEXT NOT NULL CHECK (length(scope_digest) = 64),
    store_db_file_identity TEXT NOT NULL CHECK (length(store_db_file_identity) > 0),
    consent_revision INTEGER NOT NULL CHECK (consent_revision > 0),
    remote_revision INTEGER CHECK (remote_revision IS NULL OR remote_revision >= 0),
    state TEXT NOT NULL CHECK (state IN ('grant_pending', 'enabled', 'revoke_pending', 'revoked'))
        CHECK (state != 'enabled' OR remote_revision IS NOT NULL),
    updated_at_ms INTEGER NOT NULL CHECK (updated_at_ms > 0)
)";

#[derive(Clone, Debug, Serialize)]
pub struct ProjectRootRecord {
    pub project_id: String,
    pub label: String,
    pub path: String,
    pub read_only: bool,
    pub primary: bool,
    pub permission_profile: Option<String>,
    pub updated_at: String,
}

#[derive(Clone, Debug, Serialize)]
pub struct CatalogEntry {
    pub domain_id: String,
    pub repo_root: String,
    pub db_path: String,
    pub project_id: Option<String>,
    pub status: String,
    pub updated_at: String,
}

#[derive(Clone, Debug, Serialize)]
pub struct FleetRunRecord {
    pub id: String,
    pub fleet_id: String,
    pub started_at: String,
    pub finished_at: Option<String>,
    pub status: String,
}

#[derive(Clone, Debug, Serialize)]
pub struct FleetNodeRecord {
    pub fleet_run_id: String,
    pub node_id: String,
    pub domain_id: String,
    pub domain_run_id: Option<String>,
    pub wave: i32,
    pub status: String,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct FlowDraftRecord {
    pub id: String,
    pub title: String,
    pub mission_text: String,
    pub source: String,
    pub domain_id: Option<String>,
    pub project_id: Option<String>,
    pub status: String,
    pub plan_json: Option<String>,
    pub dispatched_run_id: Option<String>,
    pub created_at: String,
    pub updated_at: String,
}

/// Durable owner evidence for the interval between the reviewed Flow claim and
/// the first domain run record. This is never permission to start a worker.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RoutedFlowDispatchOwner {
    pub controller_pid: u32,
    pub controller_start_identity: String,
    pub store_db_path: String,
    /// None only for claims written before Catalog v5. They cannot prove that
    /// a later file at the same path is the original Store.
    pub store_db_file_identity: Option<String>,
}

/// Immutable location of a workspace routing grant. The regular domain
/// catalog entry may change when pytxo.toml changes; revocation must not.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RoutingAdvisorConsentStoreLocator {
    pub store_db_path: String,
    pub store_db_file_identity: String,
}

/// Proof that a specific hosted consent revision followed one persisted
/// recipient-bound Flow review. A shared Store locator alone is not proof.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct HostedAdvisorConsentReview {
    pub domain_id: String,
    pub recipient_identity: String,
    pub draft_id: String,
    pub consent_revision: u64,
    pub scope_digest: String,
    pub packet_digest: String,
    pub request_digest: String,
    pub store_db_file_identity: String,
}

/// Latest local hosted consent transition, including revocations. A prior
/// grant receipt cannot become current again if Store rows are rolled back.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct HostedAdvisorConsentFence {
    pub domain_id: String,
    pub recipient_identity: String,
    pub consent_revision: u64,
    pub enabled: bool,
    pub store_db_file_identity: String,
}

pub struct Catalog {
    pub(crate) conn: Connection,
    opened_path: PathBuf,
}

impl Catalog {
    pub fn open(path: &Path) -> Result<Self> {
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent).map_err(PytxoError::Io)?;
        }
        let conn = Connection::open(path).map_err(|e| PytxoError::Store(e.to_string()))?;
        conn.busy_timeout(std::time::Duration::from_secs(5))
            .map_err(|e| PytxoError::Store(e.to_string()))?;
        conn.execute_batch("PRAGMA foreign_keys=ON; PRAGMA journal_mode=WAL;")
            .map_err(|e| PytxoError::Store(e.to_string()))?;
        conn.execute_batch(CATALOG_SCHEMA)
            .map_err(|e| PytxoError::Store(e.to_string()))?;
        migrate_catalog(&conn)?;
        let opened_path = std::fs::canonicalize(path).map_err(PytxoError::Io)?;
        Ok(Self { conn, opened_path })
    }

    /// Open an existing capacity Catalog without creating or migrating a file.
    /// A missing replacement path is uncertainty, never proof that a reserve
    /// call did not happen in the Catalog originally opened by the controller.
    pub fn open_existing_for_capacity_recovery(path: &Path) -> Result<Self> {
        let opened_path = std::fs::canonicalize(path).map_err(PytxoError::Io)?;
        if !opened_path.is_file() {
            return Err(PytxoError::Store(
                "capacity Catalog path is not a file".into(),
            ));
        }
        let conn =
            Connection::open_with_flags(&opened_path, rusqlite::OpenFlags::SQLITE_OPEN_READ_WRITE)
                .map_err(|error| PytxoError::Store(error.to_string()))?;
        conn.busy_timeout(std::time::Duration::from_secs(5))
            .map_err(|error| PytxoError::Store(error.to_string()))?;
        let version: i64 = conn
            .query_row("PRAGMA user_version", [], |row| row.get(0))
            .map_err(|error| PytxoError::Store(error.to_string()))?;
        if !matches!(version, 2..=10) {
            return Err(PytxoError::Store(format!(
                "capacity Catalog has incompatible schema version {version}"
            )));
        }
        verify_flow_catalog_schema(&conn)?;
        crate::capacity::verify_catalog_capacity(&conn)?;
        if version >= 3 {
            verify_routed_flow_dispatch_claims_schema(&conn, version)?;
        }
        if version >= 6 {
            verify_routing_advisor_consent_store_schema(&conn)?;
        }
        if version >= 7 {
            verify_hosted_workspace_ids_schema(&conn)?;
        }
        if version >= 8 {
            verify_hosted_advisor_consent_reviews_schema(&conn)?;
            verify_hosted_advisor_consent_fences_schema(&conn)?;
        }
        if version >= 10 {
            verify_hosted_routing_grants_schema(&conn)?;
        }
        Ok(Self { conn, opened_path })
    }

    /// Canonical locator for this opened Catalog. It is a private lookup hint,
    /// not proof that a later file at this path is the same database.
    pub fn opened_path(&self) -> &Path {
        &self.opened_path
    }

    /// Pin the original Store before changing a routing grant. A later config
    /// update cannot silently redirect status or revocation to another file.
    pub fn bind_routing_advisor_consent_store(
        &self,
        domain_id: &str,
        store_db_path: &str,
        store_db_file_identity: &str,
    ) -> Result<RoutingAdvisorConsentStoreLocator> {
        if domain_id.is_empty() || store_db_path.is_empty() || store_db_file_identity.is_empty() {
            return Err(PytxoError::Store(
                "routing advisor consent Store locator is incomplete".into(),
            ));
        }
        self.conn
            .execute(
                "INSERT INTO routing_advisor_consent_stores
                 (domain_id, store_db_path, store_db_file_identity)
                 VALUES (?1, ?2, ?3) ON CONFLICT(domain_id) DO NOTHING",
                params![domain_id, store_db_path, store_db_file_identity],
            )
            .map_err(|error| PytxoError::Store(error.to_string()))?;
        let pinned = self
            .routing_advisor_consent_store(domain_id)?
            .ok_or_else(|| {
                PytxoError::Store("routing advisor consent Store was not pinned".into())
            })?;
        if pinned.store_db_path != store_db_path
            || pinned.store_db_file_identity != store_db_file_identity
        {
            return Err(PytxoError::Store(
                "routing advisor consent Store differs from the pinned file".into(),
            ));
        }
        Ok(pinned)
    }

    pub fn routing_advisor_consent_store(
        &self,
        domain_id: &str,
    ) -> Result<Option<RoutingAdvisorConsentStoreLocator>> {
        self.conn
            .query_row(
                "SELECT store_db_path, store_db_file_identity
                 FROM routing_advisor_consent_stores WHERE domain_id = ?1",
                params![domain_id],
                |row| {
                    Ok(RoutingAdvisorConsentStoreLocator {
                        store_db_path: row.get(0)?,
                        store_db_file_identity: row.get(1)?,
                    })
                },
            )
            .optional()
            .map_err(|error| PytxoError::Store(error.to_string()))
    }

    /// Record only a completed, reviewed hosted grant. The caller must first
    /// commit the separate Store consent CAS; readers still check that Store's
    /// current revision and the persisted Flow review before treating it as
    /// enabled. A delayed older receipt cannot replace a newer revision.
    pub fn record_hosted_advisor_consent_review(
        &self,
        review: &HostedAdvisorConsentReview,
    ) -> Result<()> {
        let valid_digest = |value: &str| {
            value.len() == 64
                && value
                    .bytes()
                    .all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte))
        };
        if review.domain_id.is_empty()
            || review.recipient_identity.is_empty()
            || review.draft_id.is_empty()
            || review.consent_revision == 0
            || review.store_db_file_identity.is_empty()
            || !valid_digest(&review.scope_digest)
            || !valid_digest(&review.packet_digest)
            || !valid_digest(&review.request_digest)
        {
            return Err(PytxoError::Store(
                "hosted advisor consent review is incomplete".into(),
            ));
        }
        let locator = self
            .routing_advisor_consent_store(&review.domain_id)?
            .ok_or_else(|| PytxoError::Store("hosted advisor Store is not pinned".into()))?;
        if locator.store_db_file_identity != review.store_db_file_identity {
            return Err(PytxoError::Store(
                "hosted advisor reviewed Store identity differs from pin".into(),
            ));
        }
        let revision = i64::try_from(review.consent_revision)
            .map_err(|_| PytxoError::Store("hosted advisor revision is too large".into()))?;
        self.conn
            .execute(
                "INSERT INTO routing_hosted_advisor_consent_reviews
                 (domain_id, recipient_identity, draft_id, consent_revision, scope_digest,
                  packet_digest, request_digest, store_db_file_identity)
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)
                 ON CONFLICT(domain_id) DO UPDATE SET
                   recipient_identity=excluded.recipient_identity,
                   draft_id=excluded.draft_id,
                   consent_revision=excluded.consent_revision,
                   scope_digest=excluded.scope_digest,
                   packet_digest=excluded.packet_digest,
                   request_digest=excluded.request_digest,
                   store_db_file_identity=excluded.store_db_file_identity
                 WHERE excluded.consent_revision > routing_hosted_advisor_consent_reviews.consent_revision",
                params![
                    review.domain_id,
                    review.recipient_identity,
                    review.draft_id,
                    revision,
                    review.scope_digest,
                    review.packet_digest,
                    review.request_digest,
                    review.store_db_file_identity
                ],
            )
            .map_err(|error| PytxoError::Store(error.to_string()))?;
        if self
            .hosted_advisor_consent_review(&review.domain_id)?
            .as_ref()
            != Some(review)
        {
            return Err(PytxoError::Store(
                "hosted advisor consent review revision conflict".into(),
            ));
        }
        Ok(())
    }

    pub fn hosted_advisor_consent_review(
        &self,
        domain_id: &str,
    ) -> Result<Option<HostedAdvisorConsentReview>> {
        let row: Option<(String, String, i64, String, String, String, String)> = self
            .conn
            .query_row(
                "SELECT recipient_identity, draft_id, consent_revision, scope_digest,
                        packet_digest, request_digest, store_db_file_identity
                 FROM routing_hosted_advisor_consent_reviews WHERE domain_id = ?1",
                params![domain_id],
                |row| {
                    Ok((
                        row.get(0)?,
                        row.get(1)?,
                        row.get(2)?,
                        row.get(3)?,
                        row.get(4)?,
                        row.get(5)?,
                        row.get(6)?,
                    ))
                },
            )
            .optional()
            .map_err(|error| PytxoError::Store(error.to_string()))?;
        row.map(
            |(
                recipient_identity,
                draft_id,
                revision,
                scope_digest,
                packet_digest,
                request_digest,
                store_db_file_identity,
            )| {
                Ok(HostedAdvisorConsentReview {
                    domain_id: domain_id.into(),
                    recipient_identity,
                    draft_id,
                    consent_revision: u64::try_from(revision).map_err(|_| {
                        PytxoError::Store("hosted advisor review revision is invalid".into())
                    })?,
                    scope_digest,
                    packet_digest,
                    request_digest,
                    store_db_file_identity,
                })
            },
        )
        .transpose()
    }

    pub fn advance_hosted_advisor_consent_fence(
        &self,
        fence: &HostedAdvisorConsentFence,
    ) -> Result<()> {
        if fence.domain_id.is_empty()
            || fence.recipient_identity.is_empty()
            || fence.consent_revision == 0
            || fence.store_db_file_identity.is_empty()
        {
            return Err(PytxoError::Store(
                "hosted advisor consent fence is incomplete".into(),
            ));
        }
        let tx = Transaction::new_unchecked(&self.conn, TransactionBehavior::Immediate)
            .map_err(|error| PytxoError::Store(error.to_string()))?;
        let pinned_identity: Option<String> = tx
            .query_row(
                "SELECT store_db_file_identity FROM routing_advisor_consent_stores WHERE domain_id = ?1",
                params![fence.domain_id],
                |row| row.get(0),
            )
            .optional()
            .map_err(|error| PytxoError::Store(error.to_string()))?;
        let pinned_identity = pinned_identity
            .ok_or_else(|| PytxoError::Store("hosted advisor Store is not pinned".into()))?;
        if pinned_identity != fence.store_db_file_identity {
            return Err(PytxoError::Store(
                "hosted advisor consent fence differs from Store pin".into(),
            ));
        }
        if fence.enabled {
            let unrevoked: bool = tx
                .query_row(
                    "SELECT EXISTS(SELECT 1 FROM hosted_routing_grants
                     WHERE domain_id = ?1 AND state != 'revoked')",
                    params![fence.domain_id],
                    |row| row.get(0),
                )
                .map_err(|error| PytxoError::Store(error.to_string()))?;
            if unrevoked {
                return Err(PytxoError::Store(
                    "revoke the existing hosted Link grant before enabling a new consent".into(),
                ));
            }
        }
        let revision = i64::try_from(fence.consent_revision)
            .map_err(|_| PytxoError::Store("hosted advisor revision is too large".into()))?;
        tx.execute(
                "INSERT INTO routing_hosted_advisor_consent_fences
                 (domain_id, recipient_identity, consent_revision, enabled, store_db_file_identity)
                 VALUES (?1, ?2, ?3, ?4, ?5)
                 ON CONFLICT(domain_id) DO UPDATE SET
                   recipient_identity=excluded.recipient_identity,
                   consent_revision=excluded.consent_revision,
                   enabled=excluded.enabled,
                   store_db_file_identity=excluded.store_db_file_identity
                 WHERE excluded.consent_revision > routing_hosted_advisor_consent_fences.consent_revision",
                params![
                    fence.domain_id,
                    fence.recipient_identity,
                    revision,
                    fence.enabled,
                    fence.store_db_file_identity
                ],
            )
            .map_err(|error| PytxoError::Store(error.to_string()))?;
        let observed: Option<(String, i64, bool, String)> = tx
            .query_row(
                "SELECT recipient_identity, consent_revision, enabled, store_db_file_identity
                 FROM routing_hosted_advisor_consent_fences WHERE domain_id = ?1",
                params![fence.domain_id],
                |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?, row.get(3)?)),
            )
            .optional()
            .map_err(|error| PytxoError::Store(error.to_string()))?;
        if observed
            != Some((
                fence.recipient_identity.clone(),
                revision,
                fence.enabled,
                fence.store_db_file_identity.clone(),
            ))
        {
            return Err(PytxoError::Store(
                "hosted advisor consent fence revision conflict".into(),
            ));
        }
        if !fence.enabled {
            // A Link enable POST may already be in flight. Persist its remote
            // cleanup obligation in the same transaction as the local fence;
            // an enabled reply can no longer leave a misleading grant_pending
            // row after consent was revoked.
            let updated_at_ms = Utc::now().timestamp_millis();
            if updated_at_ms <= 0 {
                return Err(PytxoError::Store("hosted grant clock is invalid".into()));
            }
            tx.execute(
                "UPDATE hosted_routing_grants SET state = 'revoke_pending', updated_at_ms = ?2
                 WHERE domain_id = ?1 AND state IN ('grant_pending','enabled')",
                params![fence.domain_id, updated_at_ms],
            )
            .map_err(|error| PytxoError::Store(error.to_string()))?;
        }
        tx.commit()
            .map_err(|error| PytxoError::Store(error.to_string()))?;
        Ok(())
    }

    pub fn hosted_advisor_consent_fence(
        &self,
        domain_id: &str,
    ) -> Result<Option<HostedAdvisorConsentFence>> {
        let row: Option<(String, i64, bool, String)> = self
            .conn
            .query_row(
                "SELECT recipient_identity, consent_revision, enabled, store_db_file_identity
                 FROM routing_hosted_advisor_consent_fences WHERE domain_id = ?1",
                params![domain_id],
                |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?, row.get(3)?)),
            )
            .optional()
            .map_err(|error| PytxoError::Store(error.to_string()))?;
        row.map(
            |(recipient_identity, revision, enabled, store_db_file_identity)| {
                Ok(HostedAdvisorConsentFence {
                    domain_id: domain_id.into(),
                    recipient_identity,
                    consent_revision: u64::try_from(revision).map_err(|_| {
                        PytxoError::Store("hosted advisor fence revision is invalid".into())
                    })?,
                    enabled,
                    store_db_file_identity,
                })
            },
        )
        .transpose()
    }

    pub fn routing_advisor_consent_domains(&self) -> Result<Vec<String>> {
        let mut statement = self
            .conn
            .prepare("SELECT domain_id FROM routing_advisor_consent_stores ORDER BY domain_id")
            .map_err(|error| PytxoError::Store(error.to_string()))?;
        let rows = statement
            .query_map([], |row| row.get(0))
            .map_err(|error| PytxoError::Store(error.to_string()))?;
        rows.collect::<std::result::Result<Vec<_>, _>>()
            .map_err(|error| PytxoError::Store(error.to_string()))
    }

    /// Read a retained address only for revocation. An address from this path
    /// does not prove that the currently registered Store is its original
    /// owner; callers preparing a hosted grant must use the checked issuer.
    pub fn hosted_workspace_id_for_revocation(&self, domain_id: &str) -> Result<Option<String>> {
        let id: Option<String> = self
            .conn
            .query_row(
                "SELECT workspace_id FROM hosted_workspace_ids WHERE domain_id = ?1",
                params![domain_id],
                |row| row.get(0),
            )
            .optional()
            .map_err(|error| PytxoError::Store(error.to_string()))?;
        if id
            .as_deref()
            .is_some_and(|id| !valid_hosted_workspace_id(id))
        {
            return Err(PytxoError::Store(
                "hosted workspace identity is malformed".into(),
            ));
        }
        Ok(id)
    }

    /// Assign exactly one opaque address to an active, physically checked
    /// Store. The caller must obtain `store_db_file_identity` from the opened
    /// Store file under its file guard, not from mutable Catalog configuration.
    /// The immediate transaction serializes competing issuers and checks the
    /// original pinned Store owner before an old address can be reused.
    pub fn ensure_hosted_workspace_id(
        &self,
        domain_id: &str,
        store_db_path: &str,
        store_db_file_identity: &str,
    ) -> Result<String> {
        let mut random = [0_u8; 16];
        getrandom::getrandom(&mut random).map_err(|error| PytxoError::Store(error.to_string()))?;
        let candidate = random
            .iter()
            .fold(String::with_capacity(32), |mut id, byte| {
                use std::fmt::Write;
                write!(&mut id, "{byte:02x}").expect("formatting into String cannot fail");
                id
            });
        let tx = Transaction::new_unchecked(&self.conn, TransactionBehavior::Immediate)
            .map_err(|error| PytxoError::Store(error.to_string()))?;
        let registered: Option<String> = tx
            .query_row(
                "SELECT db_path FROM domains WHERE domain_id = ?1",
                params![domain_id],
                |row| row.get(0),
            )
            .optional()
            .map_err(|error| PytxoError::Store(error.to_string()))?;
        let pinned: Option<(String, String)> = tx
            .query_row(
                "SELECT store_db_path, store_db_file_identity
                 FROM routing_advisor_consent_stores WHERE domain_id = ?1",
                params![domain_id],
                |row| Ok((row.get(0)?, row.get(1)?)),
            )
            .optional()
            .map_err(|error| PytxoError::Store(error.to_string()))?;
        // Desktop registers the configured Windows path while the consent
        // pin records std::fs::canonicalize's extended-length path. Accept
        // only the same resolved file; the caller separately holds and checks
        // its physical file-identity guard before a remote grant.
        let registered_matches = registered.as_deref().is_some_and(|registered_path| {
            registered_path == store_db_path
                || std::fs::canonicalize(registered_path)
                    .is_ok_and(|canonical| canonical == Path::new(store_db_path))
        });
        if store_db_path.is_empty()
            || store_db_file_identity.is_empty()
            || !registered_matches
            || !pinned.as_ref().is_some_and(|(path, identity)| {
                path == store_db_path && identity == store_db_file_identity
            })
        {
            return Err(PytxoError::Store(
                "hosted workspace identity requires the original registered Store".into(),
            ));
        }
        tx.execute(
            "INSERT INTO hosted_workspace_ids (domain_id, workspace_id)
             VALUES (?1, ?2) ON CONFLICT(domain_id) DO NOTHING",
            params![domain_id, candidate],
        )
        .map_err(|error| PytxoError::Store(error.to_string()))?;
        let id: String = tx
            .query_row(
                "SELECT workspace_id FROM hosted_workspace_ids WHERE domain_id = ?1",
                params![domain_id],
                |row| row.get(0),
            )
            .map_err(|error| PytxoError::Store(error.to_string()))?;
        if !valid_hosted_workspace_id(&id) {
            return Err(PytxoError::Store(
                "hosted workspace identity is malformed".into(),
            ));
        }
        tx.commit()
            .map_err(|error| PytxoError::Store(error.to_string()))?;
        Ok(id)
    }

    /// Open the default catalog at `~/.pytxo/hypervisor.db`.
    pub fn open_default() -> Result<Self> {
        let path = default_catalog_path()
            .ok_or_else(|| PytxoError::Store("cannot resolve home directory".into()))?;
        Self::open(&path)
    }

    /// Insert or refresh a domain entry.
    pub fn upsert_domain(
        &self,
        domain_id: &str,
        repo_root: &str,
        db_path: &str,
        project_id: Option<&str>,
    ) -> Result<()> {
        let now = Utc::now().to_rfc3339();
        self.conn
            .execute(
                "INSERT INTO domains (domain_id, repo_root, db_path, project_id, status, updated_at)
                 VALUES (?1, ?2, ?3, ?4, 'active', ?5)
                 ON CONFLICT(domain_id) DO UPDATE SET
                     repo_root = excluded.repo_root,
                     db_path = excluded.db_path,
                     project_id = COALESCE(excluded.project_id, domains.project_id),
                     status = 'active',
                     updated_at = excluded.updated_at",
                params![domain_id, repo_root, db_path, project_id, now],
            )
            .map_err(|e| PytxoError::Store(e.to_string()))?;
        Ok(())
    }

    /// Remove one domain reference from the global catalog. This only deletes the
    /// catalog row (`~/.pytxo/hypervisor.db`); it never touches the repository on
    /// disk or the domain's own per-repo store/WAL.
    pub fn delete_domain(&self, domain_id: &str) -> Result<bool> {
        let changed = self
            .conn
            .execute(
                "DELETE FROM domains WHERE domain_id = ?1",
                params![domain_id],
            )
            .map_err(|e| PytxoError::Store(e.to_string()))?;
        Ok(changed > 0)
    }

    pub fn list_domains(&self) -> Result<Vec<CatalogEntry>> {
        let mut stmt = self
            .conn
            .prepare(
                "SELECT domain_id, repo_root, db_path, project_id, status, updated_at
                 FROM domains ORDER BY updated_at DESC",
            )
            .map_err(|e| PytxoError::Store(e.to_string()))?;
        let rows = stmt
            .query_map([], |row| {
                Ok(CatalogEntry {
                    domain_id: row.get(0)?,
                    repo_root: row.get(1)?,
                    db_path: row.get(2)?,
                    project_id: row.get(3)?,
                    status: row.get(4)?,
                    updated_at: row.get(5)?,
                })
            })
            .map_err(|e| PytxoError::Store(e.to_string()))?;
        rows.collect::<std::result::Result<Vec<_>, _>>()
            .map_err(|e| PytxoError::Store(e.to_string()))
    }

    /// Modular projects v2: persist path roots in the hypervisor catalog (Phase 66).
    pub fn upsert_project_root(
        &self,
        project_id: &str,
        label: &str,
        path: &str,
        read_only: bool,
        primary: bool,
        permission_profile: Option<&str>,
    ) -> Result<()> {
        let now = Utc::now().to_rfc3339();
        self.conn
            .execute(
                "INSERT INTO project_roots (project_id, label, path, read_only, primary_root, permission_profile, updated_at)
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)
                 ON CONFLICT(project_id, label) DO UPDATE SET
                     path = excluded.path,
                     read_only = excluded.read_only,
                     primary_root = excluded.primary_root,
                     permission_profile = excluded.permission_profile,
                     updated_at = excluded.updated_at",
                params![
                    project_id,
                    label,
                    path,
                    read_only as i32,
                    primary as i32,
                    permission_profile,
                    now
                ],
            )
            .map_err(|e| PytxoError::Store(e.to_string()))?;
        Ok(())
    }

    pub fn list_project_roots(&self, project_id: &str) -> Result<Vec<ProjectRootRecord>> {
        let mut stmt = self
            .conn
            .prepare(
                "SELECT project_id, label, path, read_only, primary_root, permission_profile, updated_at
                 FROM project_roots WHERE project_id = ?1 ORDER BY primary_root DESC, label",
            )
            .map_err(|e| PytxoError::Store(e.to_string()))?;
        let rows = stmt
            .query_map(params![project_id], |row| {
                Ok(ProjectRootRecord {
                    project_id: row.get(0)?,
                    label: row.get(1)?,
                    path: row.get(2)?,
                    read_only: row.get::<_, i32>(3)? != 0,
                    primary: row.get::<_, i32>(4)? != 0,
                    permission_profile: row.get(5)?,
                    updated_at: row.get(6)?,
                })
            })
            .map_err(|e| PytxoError::Store(e.to_string()))?;
        rows.collect::<std::result::Result<Vec<_>, _>>()
            .map_err(|e| PytxoError::Store(e.to_string()))
    }

    /// Count every active fleet, including gaps between individual domain runs.
    pub fn active_fleet_count(&self) -> Result<usize> {
        self.conn
            .query_row(
                "SELECT COUNT(*) FROM fleet_runs WHERE status='running'",
                [],
                |row| row.get::<_, i64>(0),
            )
            .map(|count| count as usize)
            .map_err(|error| PytxoError::Store(error.to_string()))
    }

    /// Record a new fleet run (hypervisor-level cross-repo DAG).
    pub fn insert_fleet_run(&self, id: &str, fleet_id: &str, started_at: &str) -> Result<()> {
        self.conn
            .execute(
                "INSERT INTO fleet_runs (id, fleet_id, started_at, status) VALUES (?1, ?2, ?3, 'running')",
                params![id, fleet_id, started_at],
            )
            .map_err(|e| PytxoError::Store(e.to_string()))?;
        Ok(())
    }

    pub fn finish_fleet_run(&self, id: &str, status: &str, finished_at: &str) -> Result<()> {
        self.conn
            .execute(
                "UPDATE fleet_runs SET finished_at = ?1, status = ?2 WHERE id = ?3",
                params![finished_at, status, id],
            )
            .map_err(|e| PytxoError::Store(e.to_string()))?;
        Ok(())
    }

    pub fn insert_fleet_node(
        &self,
        fleet_run_id: &str,
        node_id: &str,
        domain_id: &str,
        domain_run_id: Option<&str>,
        wave: i32,
        status: &str,
    ) -> Result<()> {
        self.conn
            .execute(
                "INSERT INTO fleet_nodes (fleet_run_id, node_id, domain_id, domain_run_id, wave, status)
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6)
                 ON CONFLICT(fleet_run_id, node_id) DO UPDATE SET
                     domain_id = excluded.domain_id,
                     domain_run_id = excluded.domain_run_id,
                     wave = excluded.wave,
                     status = excluded.status",
                params![fleet_run_id, node_id, domain_id, domain_run_id, wave, status],
            )
            .map_err(|e| PytxoError::Store(e.to_string()))?;
        Ok(())
    }

    pub fn update_fleet_node_status(
        &self,
        fleet_run_id: &str,
        node_id: &str,
        status: &str,
        domain_run_id: Option<&str>,
    ) -> Result<()> {
        self.conn
            .execute(
                "UPDATE fleet_nodes SET status = ?1, domain_run_id = COALESCE(?2, domain_run_id)
                 WHERE fleet_run_id = ?3 AND node_id = ?4",
                params![status, domain_run_id, fleet_run_id, node_id],
            )
            .map_err(|e| PytxoError::Store(e.to_string()))?;
        Ok(())
    }

    pub fn list_fleet_runs(
        &self,
        fleet_id: Option<&str>,
        limit: usize,
    ) -> Result<Vec<FleetRunRecord>> {
        let sql = if fleet_id.is_some() {
            "SELECT id, fleet_id, started_at, finished_at, status FROM fleet_runs
             WHERE fleet_id = ?1 ORDER BY started_at DESC LIMIT ?2"
        } else {
            "SELECT id, fleet_id, started_at, finished_at, status FROM fleet_runs
             ORDER BY started_at DESC LIMIT ?1"
        };
        let mut stmt = self
            .conn
            .prepare(sql)
            .map_err(|e| PytxoError::Store(e.to_string()))?;
        let rows = if let Some(fid) = fleet_id {
            stmt.query_map(params![fid, limit as i64], map_fleet_run)
        } else {
            stmt.query_map(params![limit as i64], map_fleet_run)
        }
        .map_err(|e| PytxoError::Store(e.to_string()))?;
        rows.collect::<std::result::Result<Vec<_>, _>>()
            .map_err(|e| PytxoError::Store(e.to_string()))
    }

    pub fn list_fleet_nodes(&self, fleet_run_id: &str) -> Result<Vec<FleetNodeRecord>> {
        let mut stmt = self
            .conn
            .prepare(
                "SELECT fleet_run_id, node_id, domain_id, domain_run_id, wave, status
                 FROM fleet_nodes WHERE fleet_run_id = ?1 ORDER BY wave, node_id",
            )
            .map_err(|e| PytxoError::Store(e.to_string()))?;
        let rows = stmt
            .query_map(params![fleet_run_id], |row| {
                Ok(FleetNodeRecord {
                    fleet_run_id: row.get(0)?,
                    node_id: row.get(1)?,
                    domain_id: row.get(2)?,
                    domain_run_id: row.get(3)?,
                    wave: row.get(4)?,
                    status: row.get(5)?,
                })
            })
            .map_err(|e| PytxoError::Store(e.to_string()))?;
        rows.collect::<std::result::Result<Vec<_>, _>>()
            .map_err(|e| PytxoError::Store(e.to_string()))
    }

    pub fn upsert_flow_draft(&self, draft: &FlowDraftRecord) -> Result<()> {
        self.conn
            .execute(
                "INSERT INTO flow_drafts
                 (id, title, mission_text, source, domain_id, project_id, status, plan_json,
                  dispatched_run_id, created_at, updated_at)
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11)
                 ON CONFLICT(id) DO UPDATE SET
                    title = excluded.title, mission_text = excluded.mission_text,
                    source = excluded.source, domain_id = excluded.domain_id,
                    project_id = excluded.project_id, status = excluded.status,
                    plan_json = excluded.plan_json, dispatched_run_id = excluded.dispatched_run_id,
                    updated_at = excluded.updated_at
                 WHERE flow_drafts.dispatched_run_id IS NULL",
                params![
                    draft.id,
                    draft.title,
                    draft.mission_text,
                    draft.source,
                    draft.domain_id,
                    draft.project_id,
                    draft.status,
                    draft.plan_json,
                    draft.dispatched_run_id,
                    draft.created_at,
                    draft.updated_at
                ],
            )
            .map_err(|e| PytxoError::Store(e.to_string()))
            .and_then(|changed| {
                if changed == 1 {
                    Ok(())
                } else {
                    Err(PytxoError::Store(
                        "cannot replace a Flow draft linked to a reviewed run".into(),
                    ))
                }
            })
    }

    /// Persist editable intent without accepting execution-owned state and without reopening a
    /// dispatching or dispatched Flow.
    pub fn upsert_flow_draft_intent(&self, draft: &FlowDraftRecord) -> Result<bool> {
        self.conn
            .execute(
                "INSERT INTO flow_drafts
                 (id, title, mission_text, source, domain_id, project_id, status, plan_json,
                  dispatched_run_id, created_at, updated_at)
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6, 'draft', NULL, NULL, ?7, ?8)
                 ON CONFLICT(id) DO UPDATE SET
                    title = excluded.title, mission_text = excluded.mission_text,
                    source = excluded.source, domain_id = excluded.domain_id,
                    project_id = excluded.project_id, status = 'draft', plan_json = NULL,
                    dispatched_run_id = NULL, updated_at = excluded.updated_at
                 WHERE flow_drafts.status NOT IN ('dispatching', 'dispatched')
                   AND flow_drafts.dispatched_run_id IS NULL",
                params![
                    draft.id,
                    draft.title,
                    draft.mission_text,
                    draft.source,
                    draft.domain_id,
                    draft.project_id,
                    draft.created_at,
                    draft.updated_at
                ],
            )
            .map(|changed| changed == 1)
            .map_err(|e| PytxoError::Store(e.to_string()))
    }

    /// Persist a Rust-produced preview unless dispatch has already claimed this draft ID.
    pub fn upsert_flow_preview(&self, draft: &FlowDraftRecord) -> Result<bool> {
        self.conn
            .execute(
                "INSERT INTO flow_drafts
                 (id, title, mission_text, source, domain_id, project_id, status, plan_json,
                  dispatched_run_id, created_at, updated_at)
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, NULL, ?9, ?10)
                 ON CONFLICT(id) DO UPDATE SET
                    title = excluded.title, mission_text = excluded.mission_text,
                    source = excluded.source, domain_id = excluded.domain_id,
                    project_id = excluded.project_id, status = excluded.status,
                    plan_json = excluded.plan_json, dispatched_run_id = NULL,
                    updated_at = excluded.updated_at
                 WHERE flow_drafts.status NOT IN ('dispatching', 'dispatched')
                   AND flow_drafts.dispatched_run_id IS NULL",
                params![
                    draft.id,
                    draft.title,
                    draft.mission_text,
                    draft.source,
                    draft.domain_id,
                    draft.project_id,
                    draft.status,
                    draft.plan_json,
                    draft.created_at,
                    draft.updated_at
                ],
            )
            .map(|changed| changed == 1)
            .map_err(|e| PytxoError::Store(e.to_string()))
    }

    pub fn get_flow_draft(&self, id: &str) -> Result<Option<FlowDraftRecord>> {
        self.conn
            .query_row(
                "SELECT id, title, mission_text, source, domain_id, project_id, status,
                    plan_json, dispatched_run_id, created_at, updated_at
             FROM flow_drafts WHERE id = ?1",
                params![id],
                map_flow_draft,
            )
            .optional()
            .map_err(|e| PytxoError::Store(e.to_string()))
    }

    pub fn list_flow_drafts(&self) -> Result<Vec<FlowDraftRecord>> {
        let mut stmt = self
            .conn
            .prepare(
                "SELECT id, title, mission_text, source, domain_id, project_id, status,
                    plan_json, dispatched_run_id, created_at, updated_at
             FROM flow_drafts ORDER BY updated_at DESC, id ASC",
            )
            .map_err(|e| PytxoError::Store(e.to_string()))?;
        let rows = stmt
            .query_map([], map_flow_draft)
            .map_err(|e| PytxoError::Store(e.to_string()))?;
        rows.collect::<std::result::Result<Vec<_>, _>>()
            .map_err(|e| PytxoError::Store(e.to_string()))
    }

    pub fn delete_flow_draft(&self, id: &str) -> Result<()> {
        let changed = self
            .conn
            .execute(
                "DELETE FROM flow_drafts WHERE id = ?1 AND status != 'dispatching'
                    AND (status = 'dispatched' OR dispatched_run_id IS NULL)",
                params![id],
            )
            .map_err(|e| PytxoError::Store(e.to_string()))?;
        if changed == 0
            && self
                .conn
                .query_row(
                    "SELECT EXISTS(SELECT 1 FROM flow_drafts WHERE id = ?1)",
                    params![id],
                    |row| row.get::<_, bool>(0),
                )
                .map_err(|e| PytxoError::Store(e.to_string()))?
        {
            return Err(PytxoError::Store(
                "cannot delete a Flow draft while dispatching or linked to a recovering run".into(),
            ));
        }
        Ok(())
    }

    /// Replace only the reviewed plan payload while the exact preview is still current.
    pub fn replace_ready_flow_plan(
        &self,
        id: &str,
        expected_plan_json: &str,
        reviewed_plan_json: &str,
    ) -> Result<bool> {
        self.conn
            .execute(
                "UPDATE flow_drafts SET plan_json = ?1, updated_at = ?2
                 WHERE id = ?3 AND status IN ('ready', 'review_only') AND dispatched_run_id IS NULL
                   AND plan_json = ?4",
                params![
                    reviewed_plan_json,
                    Utc::now().to_rfc3339(),
                    id,
                    expected_plan_json
                ],
            )
            .map(|changed| changed == 1)
            .map_err(|e| PytxoError::Store(e.to_string()))
    }

    pub fn mark_flow_dispatched(&self, id: &str, run_id: &str) -> Result<()> {
        let changed = self
            .conn
            .execute(
                "UPDATE flow_drafts SET status = 'dispatched', dispatched_run_id = ?1,
                    updated_at = ?2 WHERE id = ?3 AND status = 'dispatching'
                    AND (dispatched_run_id IS NULL OR dispatched_run_id = ?1)
                    AND NOT EXISTS (SELECT 1 FROM flow_dispatch_claims
                        WHERE draft_id = ?3 AND run_id = ?1 AND stop_requested = 1)",
                params![run_id, Utc::now().to_rfc3339(), id],
            )
            .map_err(|e| PytxoError::Store(e.to_string()))?;
        if changed == 0 {
            let existing = self.get_flow_draft(id)?;
            if existing.as_ref().is_some_and(|row| {
                row.status == "dispatched" && row.dispatched_run_id.as_deref() == Some(run_id)
            }) {
                return Ok(());
            }
            return Err(PytxoError::Store(format!("flow draft not found: {id}")));
        }
        Ok(())
    }

    /// Atomically claim a ready preview for dispatch. Exactly one concurrent caller can win.
    pub fn claim_flow_dispatch(&self, id: &str, expected_plan_json: &str) -> Result<bool> {
        self.conn
            .execute(
                "UPDATE flow_drafts SET status = 'dispatching', updated_at = ?1
                 WHERE id = ?2 AND status = 'ready' AND dispatched_run_id IS NULL
                   AND plan_json = ?3",
                params![Utc::now().to_rfc3339(), id, expected_plan_json],
            )
            .map(|changed| changed == 1)
            .map_err(|e| PytxoError::Store(e.to_string()))
    }

    /// Claim the reviewed routed RunId in the same Catalog write that claims
    /// the ready Flow. A later startup error retains this exact association.
    pub fn claim_routed_flow_dispatch(
        &self,
        id: &str,
        expected_plan_json: &str,
        run_id: &str,
        owner: &RoutedFlowDispatchOwner,
    ) -> Result<bool> {
        self.claim_routed_flow_dispatch_from_status(id, expected_plan_json, run_id, owner, "ready")
    }

    /// The hosted Shadow experiment has a separately reviewed, non-runnable
    /// draft state. Only its explicit controller may make this exact claim;
    /// ordinary routed dispatch continues to accept `ready` alone.
    pub fn claim_review_only_hosted_shadow_dispatch(
        &self,
        id: &str,
        expected_plan_json: &str,
        run_id: &str,
        owner: &RoutedFlowDispatchOwner,
    ) -> Result<bool> {
        self.claim_routed_flow_dispatch_from_status(
            id,
            expected_plan_json,
            run_id,
            owner,
            "review_only",
        )
    }

    fn claim_routed_flow_dispatch_from_status(
        &self,
        id: &str,
        expected_plan_json: &str,
        run_id: &str,
        owner: &RoutedFlowDispatchOwner,
        expected_status: &str,
    ) -> Result<bool> {
        if run_id.trim().is_empty() {
            return Err(PytxoError::Store("reviewed routed RunId is empty".into()));
        }
        if owner.controller_pid == 0
            || owner.controller_start_identity.trim().is_empty()
            || owner.store_db_path.trim().is_empty()
            || owner
                .store_db_file_identity
                .as_deref()
                .is_none_or(|identity| identity.trim().is_empty())
        {
            return Err(PytxoError::Store(
                "routed Flow dispatch owner evidence is incomplete".into(),
            ));
        }
        let tx = Transaction::new_unchecked(&self.conn, TransactionBehavior::Immediate)
            .map_err(|error| PytxoError::Store(error.to_string()))?;
        let changed = tx
            .execute(
                "UPDATE flow_drafts SET status = 'dispatching', dispatched_run_id = ?1,
                 updated_at = ?2 WHERE id = ?3 AND status = ?5
                     AND dispatched_run_id IS NULL AND plan_json = ?4",
                params![
                    run_id,
                    Utc::now().to_rfc3339(),
                    id,
                    expected_plan_json,
                    expected_status
                ],
            )
            .map_err(|error| PytxoError::Store(error.to_string()))?;
        if changed == 0 {
            return Ok(false);
        }
        tx.execute(
            "INSERT INTO flow_dispatch_claims
             (draft_id, run_id, controller_pid, controller_start_identity,
              store_db_path, store_db_file_identity)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
            params![
                id,
                run_id,
                i64::from(owner.controller_pid),
                owner.controller_start_identity,
                owner.store_db_path,
                owner.store_db_file_identity
            ],
        )
        .map_err(|error| PytxoError::Store(error.to_string()))?;
        tx.commit()
            .map_err(|error| PytxoError::Store(error.to_string()))?;
        Ok(true)
    }

    /// An older linked draft without an owner record is uncertain, not dead.
    pub fn routed_flow_dispatch_owner(
        &self,
        id: &str,
        run_id: &str,
    ) -> Result<Option<RoutedFlowDispatchOwner>> {
        let row: Option<(i64, String, String, Option<String>)> = self
            .conn
            .query_row(
                "SELECT controller_pid, controller_start_identity, store_db_path,
                        store_db_file_identity
                 FROM flow_dispatch_claims WHERE draft_id = ?1 AND run_id = ?2",
                params![id, run_id],
                |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?, row.get(3)?)),
            )
            .optional()
            .map_err(|error| PytxoError::Store(error.to_string()))?;
        row.map(
            |(pid, controller_start_identity, store_db_path, store_db_file_identity)| {
                Ok(RoutedFlowDispatchOwner {
                    controller_pid: u32::try_from(pid).map_err(|_| {
                        PytxoError::Store("routed Flow owner PID is invalid".into())
                    })?,
                    controller_start_identity,
                    store_db_path,
                    store_db_file_identity,
                })
            },
        )
        .transpose()
    }

    /// This Catalog write precedes the first call that can reserve a domain
    /// run. A crash with this bit still zero proves that this controller never
    /// invoked startup; a crash after setting it is ambiguous.
    pub fn mark_routed_flow_startup_may_have_started(
        &self,
        id: &str,
        run_id: &str,
        owner: &RoutedFlowDispatchOwner,
    ) -> Result<bool> {
        self.conn
            .execute(
                "UPDATE flow_dispatch_claims SET startup_may_have_started = 1
                 WHERE draft_id = ?1 AND run_id = ?2 AND controller_pid = ?3
                   AND controller_start_identity = ?4 AND store_db_path = ?5
                   AND store_db_file_identity IS ?6
                   AND startup_may_have_started = 0
                   AND EXISTS (SELECT 1 FROM flow_drafts WHERE id = ?1
                               AND status = 'dispatching' AND dispatched_run_id = ?2)",
                params![
                    id,
                    run_id,
                    i64::from(owner.controller_pid),
                    owner.controller_start_identity,
                    owner.store_db_path,
                    owner.store_db_file_identity
                ],
            )
            .map(|changed| changed == 1)
            .map_err(|error| PytxoError::Store(error.to_string()))
    }

    pub fn routed_flow_startup_may_have_started(
        &self,
        id: &str,
        run_id: &str,
    ) -> Result<Option<bool>> {
        self.conn
            .query_row(
                "SELECT startup_may_have_started FROM flow_dispatch_claims
                 WHERE draft_id = ?1 AND run_id = ?2",
                params![id, run_id],
                |row| row.get(0),
            )
            .optional()
            .map_err(|error| PytxoError::Store(error.to_string()))
    }

    /// Win the Ready-to-Dispatch race before acknowledging a pre-claim Stop.
    /// If dispatch won, the caller must acquire the claimed domain's native
    /// launch gate before recording a Stop against that active claim.
    pub fn cancel_ready_routed_flow(&self, id: &str, expected_plan_json: &str) -> Result<bool> {
        self.cancel_reviewed_routed_flow_from_status(id, expected_plan_json, "ready")
    }

    /// Fence the experimental hosted Shadow claim before its controller owns a
    /// Run. The caller must have checked the exact reviewed `review_only` plan.
    pub fn cancel_review_only_hosted_shadow_flow(
        &self,
        id: &str,
        expected_plan_json: &str,
    ) -> Result<bool> {
        self.cancel_reviewed_routed_flow_from_status(id, expected_plan_json, "review_only")
    }

    fn cancel_reviewed_routed_flow_from_status(
        &self,
        id: &str,
        expected_plan_json: &str,
        expected_status: &str,
    ) -> Result<bool> {
        if expected_plan_json.is_empty() {
            return Ok(false);
        }
        self.conn
            .execute(
                "UPDATE flow_drafts SET status = 'cancelled', updated_at = ?1
                 WHERE id = ?2 AND status = ?4 AND dispatched_run_id IS NULL
                   AND plan_json = ?3",
                params![
                    Utc::now().to_rfc3339(),
                    id,
                    expected_plan_json,
                    expected_status
                ],
            )
            .map(|changed| changed == 1)
            .map_err(|error| PytxoError::Store(error.to_string()))
    }

    /// Record an exact reviewed Stop before a routed claim, or durably fence an
    /// existing claim. A request is not proof that a probe or worker is stopped.
    pub fn request_routed_flow_stop(
        &self,
        id: &str,
        expected_plan_json: &str,
        run_id: &str,
    ) -> Result<bool> {
        if run_id.trim().is_empty() || expected_plan_json.is_empty() {
            return Ok(false);
        }
        let tx = Transaction::new_unchecked(&self.conn, TransactionBehavior::Immediate)
            .map_err(|error| PytxoError::Store(error.to_string()))?;
        let stopped_before_claim = tx
            .execute(
                "UPDATE flow_drafts SET status = 'cancelled', updated_at = ?1
                 WHERE id = ?2 AND status = 'ready' AND dispatched_run_id IS NULL
                   AND plan_json = ?3",
                params![Utc::now().to_rfc3339(), id, expected_plan_json],
            )
            .map_err(|error| PytxoError::Store(error.to_string()))?
            == 1;
        let requested_during_dispatch = if stopped_before_claim {
            false
        } else {
            tx.execute(
                "UPDATE flow_dispatch_claims SET stop_requested = 1
                 WHERE draft_id = ?1 AND run_id = ?2
                   AND EXISTS (SELECT 1 FROM flow_drafts WHERE id = ?1
                       AND status = 'dispatching' AND dispatched_run_id = ?2
                       AND plan_json = ?3)",
                params![id, run_id, expected_plan_json],
            )
            .map_err(|error| PytxoError::Store(error.to_string()))?
                == 1
        };
        let already_cancelled = if stopped_before_claim || requested_during_dispatch {
            false
        } else {
            tx.query_row(
                "SELECT EXISTS(SELECT 1 FROM flow_drafts WHERE id = ?1
                    AND status = 'cancelled' AND plan_json = ?2)",
                params![id, expected_plan_json],
                |row| row.get::<_, bool>(0),
            )
            .map_err(|error| PytxoError::Store(error.to_string()))?
        };
        tx.commit()
            .map_err(|error| PytxoError::Store(error.to_string()))?;
        Ok(stopped_before_claim || requested_during_dispatch || already_cancelled)
    }

    /// None is an absent or mismatched claim, not permission to proceed.
    pub fn routed_flow_stop_requested(&self, id: &str, run_id: &str) -> Result<Option<bool>> {
        self.conn
            .query_row(
                "SELECT stop_requested FROM flow_dispatch_claims
                 WHERE draft_id = ?1 AND run_id = ?2",
                params![id, run_id],
                |row| row.get(0),
            )
            .optional()
            .map_err(|error| PytxoError::Store(error.to_string()))
    }

    /// Only the dispatch controller calls this after every owned preflight
    /// probe is quiescent and before reserving any domain run.
    pub fn mark_routed_flow_preflight_stopped(&self, id: &str, run_id: &str) -> Result<()> {
        let changed = self
            .conn
            .execute(
                "UPDATE flow_drafts SET status = 'cancelled', updated_at = ?1
                 WHERE id = ?2 AND status = 'dispatching' AND dispatched_run_id = ?3
                   AND EXISTS (SELECT 1 FROM flow_dispatch_claims
                       WHERE draft_id = ?2 AND run_id = ?3 AND stop_requested = 1)",
                params![Utc::now().to_rfc3339(), id, run_id],
            )
            .map_err(|error| PytxoError::Store(error.to_string()))?;
        if changed == 1
            || self.get_flow_draft(id)?.as_ref().is_some_and(|draft| {
                draft.status == "cancelled" && draft.dispatched_run_id.as_deref() == Some(run_id)
            })
        {
            return Ok(());
        }
        Err(PytxoError::Store(
            "routed preflight Stop did not match an owned dispatch claim".into(),
        ))
    }

    /// The controller proved the exact reviewed startup is quiescent after a
    /// durable Stop. This also settles a linked recovery row; it never infers
    /// quiescence from the Stop bit alone.
    pub fn mark_routed_flow_stopped_after_quiescence(&self, id: &str, run_id: &str) -> Result<()> {
        let changed = self
            .conn
            .execute(
                "UPDATE flow_drafts SET status = 'cancelled', updated_at = ?1
                 WHERE id = ?2 AND status IN ('dispatching', 'recovery_required')
                   AND dispatched_run_id = ?3
                   AND EXISTS (SELECT 1 FROM flow_dispatch_claims
                       WHERE draft_id = ?2 AND run_id = ?3 AND stop_requested = 1)",
                params![Utc::now().to_rfc3339(), id, run_id],
            )
            .map_err(|error| PytxoError::Store(error.to_string()))?;
        if changed == 1
            || self.get_flow_draft(id)?.as_ref().is_some_and(|draft| {
                draft.status == "cancelled" && draft.dispatched_run_id.as_deref() == Some(run_id)
            })
        {
            return Ok(());
        }
        Err(PytxoError::Store(
            "routed Stop settlement did not match a linked requested run".into(),
        ))
    }

    /// Called only after the OS confirms the original controller PID and start
    /// identity no longer identify a live process. The exact owner and RunId
    /// must still match when the Catalog transition commits.
    pub fn mark_routed_flow_owner_lost(
        &self,
        id: &str,
        run_id: &str,
        owner: &RoutedFlowDispatchOwner,
    ) -> Result<bool> {
        self.conn
            .execute(
                "UPDATE flow_drafts SET status = 'recovery_required', updated_at = ?1
                 WHERE id = ?2 AND status = 'dispatching' AND dispatched_run_id = ?3
                   AND EXISTS (SELECT 1 FROM flow_dispatch_claims
                               WHERE draft_id = ?2 AND run_id = ?3 AND controller_pid = ?4
                                 AND controller_start_identity = ?5 AND store_db_path = ?6
                                 AND store_db_file_identity IS ?7)",
                params![
                    Utc::now().to_rfc3339(),
                    id,
                    run_id,
                    i64::from(owner.controller_pid),
                    owner.controller_start_identity,
                    owner.store_db_path,
                    owner.store_db_file_identity
                ],
            )
            .map(|changed| changed == 1)
            .map_err(|error| PytxoError::Store(error.to_string()))
    }

    /// A routed startup may have written private ownership before returning an
    /// error. Preserve the reviewed RunId until an exact recovery reconciles it.
    pub fn mark_routed_flow_recovery_required(&self, id: &str, run_id: &str) -> Result<()> {
        let changed = self
            .conn
            .execute(
                "UPDATE flow_drafts SET status = 'recovery_required', updated_at = ?1
                 WHERE id = ?2 AND status = 'dispatching' AND dispatched_run_id = ?3",
                params![Utc::now().to_rfc3339(), id, run_id],
            )
            .map_err(|e| PytxoError::Store(e.to_string()))?;
        if changed == 1
            || self.get_flow_draft(id)?.as_ref().is_some_and(|row| {
                row.status == "recovery_required"
                    && row.dispatched_run_id.as_deref() == Some(run_id)
            })
        {
            return Ok(());
        }
        Err(PytxoError::Store(format!(
            "routed Flow recovery is not linked to reviewed run: {id}"
        )))
    }

    /// Only a controller that has durably cancelled the private mission and
    /// released the active run can classify this linked startup as failed.
    pub fn mark_routed_flow_startup_failed(&self, id: &str, run_id: &str) -> Result<()> {
        let changed = self
            .conn
            .execute(
                "UPDATE flow_drafts SET status = 'failed', updated_at = ?1
                 WHERE id = ?2 AND status = 'dispatching' AND dispatched_run_id = ?3",
                params![Utc::now().to_rfc3339(), id, run_id],
            )
            .map_err(|e| PytxoError::Store(e.to_string()))?;
        if changed == 1
            || self.get_flow_draft(id)?.as_ref().is_some_and(|row| {
                row.status == "failed" && row.dispatched_run_id.as_deref() == Some(run_id)
            })
        {
            return Ok(());
        }
        Err(PytxoError::Store(format!(
            "routed Flow startup failure is not linked to reviewed run: {id}"
        )))
    }

    /// Finish a previously ambiguous startup only after the controller has
    /// checked the exact terminal run and released all of its owned surfaces.
    pub fn mark_routed_flow_recovered_failed(&self, id: &str, run_id: &str) -> Result<()> {
        let changed = self
            .conn
            .execute(
                "UPDATE flow_drafts SET status = 'failed', updated_at = ?1
                 WHERE id = ?2 AND status = 'recovery_required' AND dispatched_run_id = ?3",
                params![Utc::now().to_rfc3339(), id, run_id],
            )
            .map_err(|e| PytxoError::Store(e.to_string()))?;
        if changed == 1
            || self.get_flow_draft(id)?.as_ref().is_some_and(|row| {
                row.status == "failed" && row.dispatched_run_id.as_deref() == Some(run_id)
            })
        {
            return Ok(());
        }
        Err(PytxoError::Store(format!(
            "routed Flow recovery settlement is not linked to reviewed run: {id}"
        )))
    }

    /// Reconcile a reviewed routed dispatch whose run settled before the
    /// Catalog acknowledgement. The caller must first prove the exact run is
    /// terminal and every Store/Catalog/native ownership surface is closed.
    pub fn mark_routed_flow_recovered_dispatched(&self, id: &str, run_id: &str) -> Result<()> {
        let changed = self
            .conn
            .execute(
                "UPDATE flow_drafts SET status = 'dispatched', updated_at = ?1
                 WHERE id = ?2 AND status IN ('dispatching','recovery_required')
                   AND dispatched_run_id = ?3",
                params![Utc::now().to_rfc3339(), id, run_id],
            )
            .map_err(|e| PytxoError::Store(e.to_string()))?;
        if changed == 1
            || self.get_flow_draft(id)?.as_ref().is_some_and(|row| {
                row.status == "dispatched" && row.dispatched_run_id.as_deref() == Some(run_id)
            })
        {
            return Ok(());
        }
        Err(PytxoError::Store(format!(
            "routed Flow terminal recovery is not linked to reviewed run: {id}"
        )))
    }

    pub fn mark_flow_dispatch_failed(&self, id: &str) -> Result<()> {
        let changed = self
            .conn
            .execute(
                "UPDATE flow_drafts SET status = 'failed', updated_at = ?1
                 WHERE id = ?2 AND status = 'dispatching' AND dispatched_run_id IS NULL",
                params![Utc::now().to_rfc3339(), id],
            )
            .map_err(|e| PytxoError::Store(e.to_string()))?;
        if changed == 0
            && self
                .get_flow_draft(id)?
                .as_ref()
                .is_some_and(|row| row.status == "dispatching" && row.dispatched_run_id.is_some())
        {
            return Err(PytxoError::Store(
                "linked routed Flow requires exact startup outcome".into(),
            ));
        }
        Ok(())
    }
}

fn map_fleet_run(row: &rusqlite::Row<'_>) -> rusqlite::Result<FleetRunRecord> {
    Ok(FleetRunRecord {
        id: row.get(0)?,
        fleet_id: row.get(1)?,
        started_at: row.get(2)?,
        finished_at: row.get(3)?,
        status: row.get(4)?,
    })
}

/// Default catalog path: `~/.pytxo/hypervisor.db`.
///
/// `PYTXO_HOME` takes priority over `HOME`/`USERPROFILE` so tests and other
/// isolated invocations never write into the operator's real catalog. Prefer
/// this override (rather than mutating `HOME`) in new test code; it cannot
/// race with unrelated env var reads on other threads.
pub fn default_catalog_path() -> Option<PathBuf> {
    std::env::var_os("PYTXO_HOME")
        .or_else(|| std::env::var_os("HOME"))
        .or_else(|| std::env::var_os("USERPROFILE"))
        .map(PathBuf::from)
        .map(|h| h.join(".pytxo").join("hypervisor.db"))
}

fn valid_hosted_workspace_id(value: &str) -> bool {
    value.len() == 32
        && value
            .bytes()
            .all(|byte| byte.is_ascii_hexdigit() && !byte.is_ascii_uppercase())
}

fn migrate_catalog(conn: &Connection) -> Result<()> {
    // Read the version while holding the schema writer lock. Two controllers
    // opening a v4 Catalog must not both run the v5 ALTER from a stale read.
    let tx = Transaction::new_unchecked(conn, TransactionBehavior::Immediate)
        .map_err(|error| PytxoError::Store(error.to_string()))?;
    let version: i64 = tx
        .query_row("PRAGMA user_version", [], |row| row.get(0))
        .map_err(|e| PytxoError::Store(e.to_string()))?;
    match version {
        0 => tx
            .execute_batch(FLOW_DRAFTS_MIGRATION)
            .map_err(|e| PytxoError::Store(e.to_string()))?,
        1..=10 => {}
        _ => {
            return Err(PytxoError::Store(format!(
                "unsupported catalog schema version {version}"
            )));
        }
    }
    verify_flow_catalog_schema(&tx)?;
    if version < 2 {
        crate::capacity::migrate_catalog_capacity(&tx)?;
    } else {
        crate::capacity::verify_catalog_capacity(&tx)?;
    }
    if version < 3 {
        tx.execute_batch(ROUTED_FLOW_DISPATCH_CLAIMS_TABLE_V3)
            .map_err(|error| PytxoError::Store(error.to_string()))?;
        verify_routed_flow_dispatch_claims_schema(&tx, 3)?;
        tx.pragma_update(None, "user_version", 3)
            .map_err(|error| PytxoError::Store(error.to_string()))?;
    } else {
        verify_routed_flow_dispatch_claims_schema(&tx, version)?;
    }
    if version < 4 {
        tx.execute_batch(ROUTED_FLOW_DISPATCH_CLAIMS_V4)
            .map_err(|error| PytxoError::Store(error.to_string()))?;
        // A v3 owner may already have entered startup. Backfill uncertainty,
        // while new v4 claims use the column's zero default until the exact
        // pre-start transition is durably recorded.
        tx.execute(
            "UPDATE flow_dispatch_claims SET startup_may_have_started = 1",
            [],
        )
        .map_err(|error| PytxoError::Store(error.to_string()))?;
        verify_routed_flow_dispatch_claims_schema(&tx, 4)?;
        tx.pragma_update(None, "user_version", 4)
            .map_err(|error| PytxoError::Store(error.to_string()))?;
    }
    if version < 5 {
        tx.execute_batch(ROUTED_FLOW_DISPATCH_CLAIMS_V5)
            .map_err(|error| PytxoError::Store(error.to_string()))?;
        // Historical claims retain NULL: a path is insufficient to recover
        // post-startup ambiguity after replacement of the Store file.
        verify_routed_flow_dispatch_claims_schema(&tx, 5)?;
        tx.pragma_update(None, "user_version", 5)
            .map_err(|error| PytxoError::Store(error.to_string()))?;
    }
    if version < 6 {
        tx.execute_batch(ROUTING_ADVISOR_CONSENT_STORES_V6)
            .map_err(|error| PytxoError::Store(error.to_string()))?;
        verify_routing_advisor_consent_store_schema(&tx)?;
        tx.pragma_update(None, "user_version", 6)
            .map_err(|error| PytxoError::Store(error.to_string()))?;
    } else {
        verify_routing_advisor_consent_store_schema(&tx)?;
    }
    if version < 7 {
        tx.execute_batch(HOSTED_WORKSPACE_IDS_V7)
            .map_err(|error| PytxoError::Store(error.to_string()))?;
        verify_hosted_workspace_ids_schema(&tx)?;
        tx.pragma_update(None, "user_version", 7)
            .map_err(|error| PytxoError::Store(error.to_string()))?;
    } else {
        verify_hosted_workspace_ids_schema(&tx)?;
    }
    if version < 8 {
        tx.execute_batch(HOSTED_ADVISOR_CONSENT_REVIEWS_V8)
            .map_err(|error| PytxoError::Store(error.to_string()))?;
        tx.execute_batch(HOSTED_ADVISOR_CONSENT_FENCES_V8)
            .map_err(|error| PytxoError::Store(error.to_string()))?;
        verify_hosted_advisor_consent_reviews_schema(&tx)?;
        verify_hosted_advisor_consent_fences_schema(&tx)?;
        tx.pragma_update(None, "user_version", 8)
            .map_err(|error| PytxoError::Store(error.to_string()))?;
    } else {
        verify_hosted_advisor_consent_reviews_schema(&tx)?;
        verify_hosted_advisor_consent_fences_schema(&tx)?;
    }
    if version < 9 {
        tx.execute_batch(ROUTED_FLOW_DISPATCH_CLAIMS_V9)
            .map_err(|error| PytxoError::Store(error.to_string()))?;
        verify_routed_flow_dispatch_claims_schema(&tx, 9)?;
        tx.pragma_update(None, "user_version", 9)
            .map_err(|error| PytxoError::Store(error.to_string()))?;
    } else {
        verify_routed_flow_dispatch_claims_schema(&tx, 9)?;
    }
    if version < 10 {
        tx.execute_batch(HOSTED_ROUTING_GRANTS_V10)
            .map_err(|error| PytxoError::Store(error.to_string()))?;
        verify_hosted_routing_grants_schema(&tx)?;
        tx.pragma_update(None, "user_version", 10)
            .map_err(|error| PytxoError::Store(error.to_string()))?;
    } else {
        verify_hosted_routing_grants_schema(&tx)?;
    }
    tx.commit()
        .map_err(|error| PytxoError::Store(error.to_string()))
}

fn verify_hosted_routing_grants_schema(conn: &Connection) -> Result<()> {
    let actual: Option<String> = conn
        .query_row(
            "SELECT sql FROM sqlite_master WHERE type='table' AND name='hosted_routing_grants'",
            [],
            |row| row.get(0),
        )
        .optional()
        .map_err(|error| PytxoError::Store(error.to_string()))?;
    let expected_sql = HOSTED_ROUTING_GRANTS_V10.replacen("IF NOT EXISTS ", "", 1);
    if actual.as_deref() != Some(expected_sql.as_str()) {
        return Err(PytxoError::Store(
            "catalog hosted routing grant schema is missing or malformed".into(),
        ));
    }
    Ok(())
}

fn verify_hosted_advisor_consent_reviews_schema(conn: &Connection) -> Result<()> {
    let actual: Option<String> = conn
        .query_row(
            "SELECT sql FROM sqlite_master WHERE type='table' AND name='routing_hosted_advisor_consent_reviews'",
            [],
            |row| row.get(0),
        )
        .optional()
        .map_err(|error| PytxoError::Store(error.to_string()))?;
    // SQLite normalizes CREATE TABLE IF NOT EXISTS in sqlite_master.
    let expected_sql = HOSTED_ADVISOR_CONSENT_REVIEWS_V8.replacen("IF NOT EXISTS ", "", 1);
    if actual.as_deref() != Some(expected_sql.as_str()) {
        return Err(PytxoError::Store(
            "catalog hosted advisor consent review schema is missing or malformed".into(),
        ));
    }
    Ok(())
}

fn verify_hosted_advisor_consent_fences_schema(conn: &Connection) -> Result<()> {
    let actual: Option<String> = conn
        .query_row(
            "SELECT sql FROM sqlite_master WHERE type='table' AND name='routing_hosted_advisor_consent_fences'",
            [],
            |row| row.get(0),
        )
        .optional()
        .map_err(|error| PytxoError::Store(error.to_string()))?;
    let expected_sql = HOSTED_ADVISOR_CONSENT_FENCES_V8.replacen("IF NOT EXISTS ", "", 1);
    if actual.as_deref() != Some(expected_sql.as_str()) {
        return Err(PytxoError::Store(
            "catalog hosted advisor consent fence schema is missing or malformed".into(),
        ));
    }
    Ok(())
}

fn verify_hosted_workspace_ids_schema(conn: &Connection) -> Result<()> {
    let actual: Option<String> = conn
        .query_row(
            "SELECT sql FROM sqlite_master WHERE type='table' AND name='hosted_workspace_ids'",
            [],
            |row| row.get(0),
        )
        .optional()
        .map_err(|error| PytxoError::Store(error.to_string()))?;
    if actual.as_deref() != Some(HOSTED_WORKSPACE_IDS_V7) {
        return Err(PytxoError::Store(
            "catalog hosted workspace identity schema is missing or malformed".into(),
        ));
    }
    Ok(())
}

fn verify_routing_advisor_consent_store_schema(conn: &Connection) -> Result<()> {
    let actual: Option<String> = conn
        .query_row(
            "SELECT sql FROM sqlite_master WHERE type='table' AND name='routing_advisor_consent_stores'",
            [],
            |row| row.get(0),
        )
        .optional()
        .map_err(|error| PytxoError::Store(error.to_string()))?;
    if actual.as_deref() != Some(ROUTING_ADVISOR_CONSENT_STORES_V6) {
        return Err(PytxoError::Store(
            "catalog routing advisor consent Store schema is missing or malformed".into(),
        ));
    }
    Ok(())
}

fn verify_routed_flow_dispatch_claims_schema(conn: &Connection, version: i64) -> Result<()> {
    let expected =
        Connection::open_in_memory().map_err(|error| PytxoError::Store(error.to_string()))?;
    expected
        .execute_batch(ROUTED_FLOW_DISPATCH_CLAIMS_TABLE_V3)
        .map_err(|error| PytxoError::Store(error.to_string()))?;
    if version >= 4 {
        expected
            .execute_batch(ROUTED_FLOW_DISPATCH_CLAIMS_V4)
            .map_err(|error| PytxoError::Store(error.to_string()))?;
    }
    if version >= 5 {
        expected
            .execute_batch(ROUTED_FLOW_DISPATCH_CLAIMS_V5)
            .map_err(|error| PytxoError::Store(error.to_string()))?;
    }
    if version >= 9 {
        expected
            .execute_batch(ROUTED_FLOW_DISPATCH_CLAIMS_V9)
            .map_err(|error| PytxoError::Store(error.to_string()))?;
    }
    let expected_sql: String = expected
        .query_row(
            "SELECT sql FROM sqlite_master WHERE type='table' AND name='flow_dispatch_claims'",
            [],
            |row| row.get(0),
        )
        .map_err(|error| PytxoError::Store(error.to_string()))?;
    let actual_sql: Option<String> = conn
        .query_row(
            "SELECT sql FROM sqlite_master WHERE type='table' AND name='flow_dispatch_claims'",
            [],
            |row| row.get(0),
        )
        .optional()
        .map_err(|error| PytxoError::Store(error.to_string()))?;
    if actual_sql.as_deref() != Some(expected_sql.as_str()) {
        return Err(PytxoError::Store(
            "catalog routed Flow dispatch claim schema is missing or malformed".into(),
        ));
    }
    Ok(())
}

fn verify_flow_catalog_schema(conn: &Connection) -> Result<()> {
    let expected =
        Connection::open_in_memory().map_err(|error| PytxoError::Store(error.to_string()))?;
    expected
        .execute_batch(FLOW_DRAFTS_MIGRATION)
        .map_err(|error| PytxoError::Store(error.to_string()))?;
    for (kind, name) in [
        ("table", "flow_drafts"),
        ("index", "idx_flow_drafts_updated_at"),
    ] {
        let expected_sql: String = expected
            .query_row(
                "SELECT sql FROM sqlite_master WHERE type=?1 AND name=?2",
                params![kind, name],
                |row| row.get(0),
            )
            .map_err(|error| PytxoError::Store(error.to_string()))?;
        let actual_sql: Option<String> = conn
            .query_row(
                "SELECT sql FROM sqlite_master WHERE type=?1 AND name=?2",
                params![kind, name],
                |row| row.get(0),
            )
            .optional()
            .map_err(|error| PytxoError::Store(error.to_string()))?;
        if actual_sql.as_deref() != Some(expected_sql.as_str()) {
            return Err(PytxoError::Store(format!(
                "catalog Flow {kind} {name} is missing or malformed"
            )));
        }
    }
    Ok(())
}

fn map_flow_draft(row: &rusqlite::Row<'_>) -> rusqlite::Result<FlowDraftRecord> {
    Ok(FlowDraftRecord {
        id: row.get(0)?,
        title: row.get(1)?,
        mission_text: row.get(2)?,
        source: row.get(3)?,
        domain_id: row.get(4)?,
        project_id: row.get(5)?,
        status: row.get(6)?,
        plan_json: row.get(7)?,
        dispatched_run_id: row.get(8)?,
        created_at: row.get(9)?,
        updated_at: row.get(10)?,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use rusqlite::Connection;

    fn owner() -> RoutedFlowDispatchOwner {
        RoutedFlowDispatchOwner {
            controller_pid: 42,
            controller_start_identity: "windows-filetime:1".into(),
            store_db_path: "C:/reviewed/store.db".into(),
            store_db_file_identity: Some("windows-file-id:1:2".into()),
        }
    }

    fn draft(id: &str, title: &str, updated_at: &str) -> FlowDraftRecord {
        FlowDraftRecord {
            id: id.into(),
            title: title.into(),
            mission_text: format!("mission {id}"),
            source: "text".into(),
            domain_id: Some("/repo/a".into()),
            project_id: None,
            status: "draft".into(),
            plan_json: None,
            dispatched_run_id: None,
            created_at: "2026-01-01T00:00:00Z".into(),
            updated_at: updated_at.into(),
        }
    }

    #[test]
    fn advisor_consent_locator_is_durable_and_cannot_silently_rebind() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("hypervisor.db");
        let catalog = Catalog::open(&path).unwrap();
        let original = catalog
            .bind_routing_advisor_consent_store("/repo/a", "/original/pytxo.db", "file-1")
            .unwrap();
        assert_eq!(
            original,
            RoutingAdvisorConsentStoreLocator {
                store_db_path: "/original/pytxo.db".into(),
                store_db_file_identity: "file-1".into(),
            }
        );
        assert!(catalog
            .bind_routing_advisor_consent_store("/repo/a", "/new/pytxo.db", "file-2")
            .is_err());
        assert!(catalog
            .bind_routing_advisor_consent_store("/repo/a", "/original/pytxo.db", "file-2")
            .is_err());
        catalog
            .upsert_domain("/repo/a", "/repo/a", "/new/pytxo.db", None)
            .unwrap();
        assert!(catalog.delete_domain("/repo/a").unwrap());
        assert_eq!(
            catalog.routing_advisor_consent_domains().unwrap(),
            vec!["/repo/a"]
        );
        drop(catalog);
        let reopened = Catalog::open(&path).unwrap();
        assert_eq!(
            reopened.routing_advisor_consent_store("/repo/a").unwrap(),
            Some(original.clone())
        );
        assert_eq!(
            reopened.routing_advisor_consent_domains().unwrap(),
            vec!["/repo/a"]
        );
    }

    #[test]
    fn hosted_review_receipt_requires_pin_and_never_regresses_revision() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("hypervisor.db");
        let catalog = Catalog::open(&path).unwrap();
        let mut review = HostedAdvisorConsentReview {
            domain_id: "/repo/a".into(),
            recipient_identity: "pytxo-hosted-routing/jev/v1".into(),
            draft_id: "reviewed-flow".into(),
            consent_revision: 1,
            scope_digest: "a".repeat(64),
            packet_digest: "b".repeat(64),
            request_digest: "c".repeat(64),
            store_db_file_identity: "file-1".into(),
        };
        assert!(catalog
            .record_hosted_advisor_consent_review(&review)
            .is_err());
        catalog
            .bind_routing_advisor_consent_store("/repo/a", "/repo/a/pytxo.db", "file-1")
            .unwrap();
        catalog
            .record_hosted_advisor_consent_review(&review)
            .unwrap();
        let mut fence = HostedAdvisorConsentFence {
            domain_id: review.domain_id.clone(),
            recipient_identity: review.recipient_identity.clone(),
            consent_revision: 1,
            enabled: true,
            store_db_file_identity: review.store_db_file_identity.clone(),
        };
        catalog
            .advance_hosted_advisor_consent_fence(&fence)
            .unwrap();
        fence.consent_revision = 2;
        fence.enabled = false;
        catalog
            .advance_hosted_advisor_consent_fence(&fence)
            .unwrap();
        let older_fence = HostedAdvisorConsentFence {
            consent_revision: 1,
            enabled: true,
            ..fence.clone()
        };
        assert!(catalog
            .advance_hosted_advisor_consent_fence(&older_fence)
            .is_err());
        assert_eq!(
            catalog.hosted_advisor_consent_fence("/repo/a").unwrap(),
            Some(fence)
        );
        catalog
            .record_hosted_advisor_consent_review(&review)
            .unwrap();
        let original = review.clone();
        review.packet_digest = "d".repeat(64);
        assert!(catalog
            .record_hosted_advisor_consent_review(&review)
            .is_err());
        assert_eq!(
            catalog.hosted_advisor_consent_review("/repo/a").unwrap(),
            Some(original.clone())
        );
        review.consent_revision = 2;
        catalog
            .record_hosted_advisor_consent_review(&review)
            .unwrap();
        assert_eq!(
            catalog.hosted_advisor_consent_review("/repo/a").unwrap(),
            Some(review.clone())
        );
        assert!(catalog
            .record_hosted_advisor_consent_review(&original)
            .is_err());
        drop(catalog);
        assert_eq!(
            Catalog::open(&path)
                .unwrap()
                .hosted_advisor_consent_review("/repo/a")
                .unwrap(),
            Some(review)
        );
    }

    #[test]
    fn hosted_review_v7_migration_and_malformed_v8_fail_closed() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("hypervisor.db");
        drop(Catalog::open(&path).unwrap());
        let conn = Connection::open(&path).unwrap();
        conn.execute_batch(
            "DROP TABLE routing_hosted_advisor_consent_reviews;
             ALTER TABLE flow_dispatch_claims DROP COLUMN stop_requested;
             PRAGMA user_version = 7;",
        )
        .unwrap();
        drop(conn);
        let catalog = Catalog::open(&path).unwrap();
        assert!(catalog
            .hosted_advisor_consent_review("/repo/a")
            .unwrap()
            .is_none());
        drop(catalog);
        let conn = Connection::open(&path).unwrap();
        let version: i64 = conn
            .query_row("PRAGMA user_version", [], |row| row.get(0))
            .unwrap();
        assert_eq!(version, 10);
        conn.execute_batch(
            "DROP TABLE routing_hosted_advisor_consent_reviews;
             CREATE TABLE routing_hosted_advisor_consent_reviews (domain_id TEXT PRIMARY KEY);",
        )
        .unwrap();
        drop(conn);
        assert!(Catalog::open(&path).is_err());
        assert!(Catalog::open_existing_for_capacity_recovery(&path).is_err());
    }

    #[test]
    fn hosted_workspace_identity_is_opaque_stable_and_issued_once_across_connections() {
        use std::sync::{Arc, Barrier};

        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("hypervisor.db");
        let catalog = Catalog::open(&path).unwrap();
        catalog
            .upsert_domain("/repo/a", "/repo/a", "/repo/a/pytxo.db", None)
            .unwrap();
        catalog
            .upsert_domain("/repo/b", "/repo/b", "/repo/b/pytxo.db", None)
            .unwrap();
        assert!(catalog
            .ensure_hosted_workspace_id("/repo/a", "/repo/a/pytxo.db", "file-a")
            .is_err());
        catalog
            .bind_routing_advisor_consent_store("/repo/a", "/repo/a/pytxo.db", "file-a")
            .unwrap();
        catalog
            .bind_routing_advisor_consent_store("/repo/b", "/repo/b/pytxo.db", "file-b")
            .unwrap();
        assert!(catalog
            .ensure_hosted_workspace_id("/repo/unknown", "/repo/a/pytxo.db", "file-a")
            .is_err());
        assert_eq!(
            catalog
                .hosted_workspace_id_for_revocation("/repo/unknown")
                .unwrap(),
            None
        );
        drop(catalog);

        let barrier = Arc::new(Barrier::new(3));
        let workers = (0..2)
            .map(|_| {
                let barrier = Arc::clone(&barrier);
                let path = path.clone();
                std::thread::spawn(move || {
                    let catalog = Catalog::open(&path).unwrap();
                    barrier.wait();
                    catalog
                        .ensure_hosted_workspace_id("/repo/a", "/repo/a/pytxo.db", "file-a")
                        .unwrap()
                })
            })
            .collect::<Vec<_>>();
        barrier.wait();
        let ids = workers
            .into_iter()
            .map(|worker| worker.join().unwrap())
            .collect::<Vec<_>>();
        assert_eq!(ids[0], ids[1]);
        assert_eq!(ids[0].len(), 32);
        assert!(ids[0]
            .bytes()
            .all(|byte| byte.is_ascii_hexdigit() && !byte.is_ascii_uppercase()));
        assert!(!ids[0].contains("repo"));

        let catalog = Catalog::open(&path).unwrap();
        assert_eq!(
            catalog
                .hosted_workspace_id_for_revocation("/repo/a")
                .unwrap(),
            Some(ids[0].clone())
        );
        let other = catalog
            .ensure_hosted_workspace_id("/repo/b", "/repo/b/pytxo.db", "file-b")
            .unwrap();
        assert_ne!(ids[0], other);
        catalog
            .upsert_domain("/repo/a", "/moved/a", "/moved/a/pytxo.db", None)
            .unwrap();
        assert!(catalog
            .ensure_hosted_workspace_id("/repo/a", "/moved/a/pytxo.db", "file-a")
            .is_err());
        assert!(catalog.delete_domain("/repo/a").unwrap());
        assert_eq!(
            catalog
                .hosted_workspace_id_for_revocation("/repo/a")
                .unwrap(),
            Some(ids[0].clone())
        );
        assert!(catalog
            .ensure_hosted_workspace_id("/repo/a", "/repo/a/pytxo.db", "file-a")
            .is_err());
        catalog
            .upsert_domain("/repo/a", "/repo/a", "/repo/a/pytxo.db", None)
            .unwrap();
        assert!(catalog
            .ensure_hosted_workspace_id("/repo/a", "/repo/a/pytxo.db", "replacement-file")
            .is_err());
        assert_eq!(
            catalog
                .ensure_hosted_workspace_id("/repo/a", "/repo/a/pytxo.db", "file-a")
                .unwrap(),
            ids[0]
        );
        assert_eq!(
            catalog.routing_advisor_consent_domains().unwrap(),
            vec!["/repo/a", "/repo/b"]
        );
    }

    #[test]
    fn hosted_workspace_v6_migration_preserves_domains_and_rejects_malformed_v7() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("hypervisor.db");
        let catalog = Catalog::open(&path).unwrap();
        catalog
            .upsert_domain("/repo/a", "/repo/a", "/repo/a/pytxo.db", None)
            .unwrap();
        drop(catalog);
        let conn = Connection::open(&path).unwrap();
        conn.execute_batch(
            "DROP TABLE hosted_workspace_ids;
             ALTER TABLE flow_dispatch_claims DROP COLUMN stop_requested;
             PRAGMA user_version = 6;",
        )
        .unwrap();
        drop(conn);

        let upgraded = Catalog::open(&path).unwrap();
        assert_eq!(upgraded.list_domains().unwrap()[0].domain_id, "/repo/a");
        assert_eq!(
            upgraded
                .hosted_workspace_id_for_revocation("/repo/a")
                .unwrap(),
            None
        );
        upgraded
            .bind_routing_advisor_consent_store("/repo/a", "/repo/a/pytxo.db", "file-a")
            .unwrap();
        assert_eq!(
            upgraded
                .ensure_hosted_workspace_id("/repo/a", "/repo/a/pytxo.db", "file-a")
                .unwrap()
                .len(),
            32
        );
        drop(upgraded);

        let conn = Connection::open(&path).unwrap();
        let version: i64 = conn
            .query_row("PRAGMA user_version", [], |row| row.get(0))
            .unwrap();
        assert_eq!(version, 10);
        conn.execute_batch(
            "DROP TABLE hosted_workspace_ids;
             CREATE TABLE hosted_workspace_ids (domain_id TEXT PRIMARY KEY, workspace_id TEXT);",
        )
        .unwrap();
        drop(conn);
        assert!(Catalog::open(&path).is_err());
        assert!(Catalog::open_existing_for_capacity_recovery(&path).is_err());
    }

    #[test]
    fn flow_schema_is_migrated_without_audio_storage() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("hypervisor.db");
        Catalog::open(&path).unwrap();
        Catalog::open(&path).unwrap();

        let conn = Connection::open(path).unwrap();
        let version: i64 = conn
            .query_row("PRAGMA user_version", [], |row| row.get(0))
            .unwrap();
        assert!(version >= 1);
        let schema: String = conn
            .query_row(
                "SELECT group_concat(sql, ' ') FROM sqlite_master WHERE sql IS NOT NULL",
                [],
                |row| row.get(0),
            )
            .unwrap();
        assert!(schema.contains("flow_drafts"));
        assert!(!schema.to_ascii_lowercase().contains("audio"));
        assert!(!schema.to_ascii_lowercase().contains("blob"));
    }

    #[test]
    fn flow_migration_preserves_legacy_catalog_rows() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("hypervisor.db");
        let conn = Connection::open(&path).unwrap();
        conn.execute_batch(CATALOG_SCHEMA).unwrap();
        conn.execute(
            "INSERT INTO domains (domain_id, repo_root, db_path, status, updated_at)
             VALUES ('legacy', '/legacy', '/legacy/pytxo.db', 'active', '2026-01-01Z')",
            [],
        )
        .unwrap();
        drop(conn);

        let catalog = Catalog::open(&path).unwrap();
        assert_eq!(catalog.list_domains().unwrap()[0].domain_id, "legacy");
        drop(catalog);
        let reopened = Catalog::open(&path).unwrap();
        assert_eq!(reopened.list_domains().unwrap().len(), 1);
    }

    #[test]
    fn flow_crud_preserves_created_at_and_orders_by_updated_at() {
        let dir = tempfile::tempdir().unwrap();
        let cat = Catalog::open(&dir.path().join("hypervisor.db")).unwrap();
        cat.upsert_flow_draft(&draft("older", "Older", "2026-01-02T00:00:00Z"))
            .unwrap();
        cat.upsert_flow_draft(&draft("newer", "Newer", "2026-01-03T00:00:00Z"))
            .unwrap();

        let mut updated = draft("older", "Updated", "2026-01-04T00:00:00Z");
        updated.created_at = "2099-01-01T00:00:00Z".into();
        cat.upsert_flow_draft(&updated).unwrap();
        let loaded = cat.get_flow_draft("older").unwrap().unwrap();
        assert_eq!(loaded.title, "Updated");
        assert_eq!(loaded.created_at, "2026-01-01T00:00:00Z");
        assert_eq!(cat.list_flow_drafts().unwrap()[0].id, "older");

        cat.delete_flow_draft("older").unwrap();
        cat.delete_flow_draft("older").unwrap();
        assert!(cat.get_flow_draft("older").unwrap().is_none());
    }

    #[test]
    fn flow_dispatch_linkage_is_persisted() {
        let dir = tempfile::tempdir().unwrap();
        let cat = Catalog::open(&dir.path().join("hypervisor.db")).unwrap();
        cat.upsert_flow_draft(&draft("flow-1", "Flow", "2026-01-02T00:00:00Z"))
            .unwrap();
        let mut ready = draft("flow-1", "Flow", "2026-01-02T00:00:00Z");
        ready.status = "ready".into();
        ready.plan_json = Some("{}".into());
        cat.upsert_flow_draft(&ready).unwrap();
        let expected = ready.plan_json.as_deref().unwrap_or("");
        assert!(cat.claim_flow_dispatch("flow-1", expected).unwrap());
        assert!(!cat.claim_flow_dispatch("flow-1", expected).unwrap());
        cat.mark_flow_dispatched("flow-1", "run-42").unwrap();
        cat.mark_flow_dispatched("flow-1", "run-42").unwrap();
        assert!(cat.mark_flow_dispatched("flow-1", "run-43").is_err());
        let loaded = cat.get_flow_draft("flow-1").unwrap().unwrap();
        assert_eq!(loaded.status, "dispatched");
        assert_eq!(loaded.dispatched_run_id.as_deref(), Some("run-42"));
        cat.delete_flow_draft("flow-1").unwrap();
        assert!(cat.get_flow_draft("flow-1").unwrap().is_none());
    }

    #[test]
    fn routed_claim_binds_exact_run_before_startup_and_preserves_it_on_failure() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("hypervisor.db");
        let cat = Catalog::open(&path).unwrap();
        let mut ready = draft("routed-flow", "Flow", "2026-01-02T00:00:00Z");
        ready.status = "ready".into();
        ready.plan_json = Some("{\"reviewed\":true}".into());
        cat.upsert_flow_draft(&ready).unwrap();
        assert!(!cat
            .claim_routed_flow_dispatch("routed-flow", "{\"reviewed\":false}", "run-42", &owner())
            .unwrap());
        assert!(cat
            .claim_routed_flow_dispatch("routed-flow", "{\"reviewed\":true}", "run-42", &owner())
            .unwrap());
        assert!(!cat
            .claim_routed_flow_dispatch("routed-flow", "{\"reviewed\":true}", "run-43", &owner())
            .unwrap());
        let claimed = cat.get_flow_draft("routed-flow").unwrap().unwrap();
        assert_eq!(claimed.status, "dispatching");
        assert_eq!(claimed.dispatched_run_id.as_deref(), Some("run-42"));
        assert_eq!(
            cat.routed_flow_dispatch_owner("routed-flow", "run-42")
                .unwrap(),
            Some(owner())
        );
        assert!(cat
            .routed_flow_dispatch_owner("routed-flow", "run-43")
            .unwrap()
            .is_none());
        assert!(cat.mark_flow_dispatched("routed-flow", "run-43").is_err());
        assert!(cat.mark_flow_dispatch_failed("routed-flow").is_err());
        assert!(cat
            .mark_routed_flow_startup_failed("routed-flow", "run-43")
            .is_err());
        cat.mark_routed_flow_startup_failed("routed-flow", "run-42")
            .unwrap();
        let mut replacement = ready.clone();
        replacement.title = "replacement after failure".into();
        assert!(!cat.upsert_flow_draft_intent(&replacement).unwrap());
        assert!(!cat.upsert_flow_preview(&replacement).unwrap());
        assert!(cat.upsert_flow_draft(&replacement).is_err());
        assert!(cat.delete_flow_draft("routed-flow").is_err());
        drop(cat);
        let reopened = Catalog::open(&path).unwrap();
        let failed = reopened.get_flow_draft("routed-flow").unwrap().unwrap();
        assert_eq!(failed.status, "failed");
        assert_eq!(failed.dispatched_run_id.as_deref(), Some("run-42"));

        let mut another = draft("routed-success", "Flow", "2026-01-02T00:00:00Z");
        another.status = "ready".into();
        another.plan_json = Some("{\"reviewed\":true}".into());
        reopened.upsert_flow_draft(&another).unwrap();
        assert!(reopened
            .claim_routed_flow_dispatch("routed-success", "{\"reviewed\":true}", "run-44", &owner())
            .unwrap());
        reopened
            .mark_flow_dispatched("routed-success", "run-44")
            .unwrap();
        reopened
            .mark_flow_dispatched("routed-success", "run-44")
            .unwrap();
        assert!(reopened
            .mark_flow_dispatched("routed-success", "run-45")
            .is_err());
    }

    #[test]
    fn hosted_shadow_claim_is_single_use_and_cannot_widen_ready_dispatch() {
        let dir = tempfile::tempdir().unwrap();
        let cat = Catalog::open(&dir.path().join("hypervisor.db")).unwrap();
        let mut reviewed = draft("hosted-shadow", "Hosted Shadow", "2026-01-02T00:00:00Z");
        reviewed.status = "review_only".into();
        reviewed.plan_json = Some("{\"hosted\":true}".into());
        cat.upsert_flow_draft(&reviewed).unwrap();

        assert!(!cat
            .claim_routed_flow_dispatch("hosted-shadow", "{\"hosted\":true}", "run-1", &owner())
            .unwrap());
        assert!(!cat
            .claim_review_only_hosted_shadow_dispatch(
                "hosted-shadow",
                "{\"hosted\":false}",
                "run-1",
                &owner(),
            )
            .unwrap());
        assert!(cat
            .claim_review_only_hosted_shadow_dispatch(
                "hosted-shadow",
                "{\"hosted\":true}",
                "run-1",
                &owner(),
            )
            .unwrap());
        assert!(!cat
            .claim_review_only_hosted_shadow_dispatch(
                "hosted-shadow",
                "{\"hosted\":true}",
                "run-2",
                &owner(),
            )
            .unwrap());
        let claimed = cat.get_flow_draft("hosted-shadow").unwrap().unwrap();
        assert_eq!(claimed.status, "dispatching");
        assert_eq!(claimed.dispatched_run_id.as_deref(), Some("run-1"));
        assert_eq!(
            cat.routed_flow_dispatch_owner("hosted-shadow", "run-1")
                .unwrap(),
            Some(owner())
        );
    }

    #[test]
    fn hosted_shadow_stop_and_claim_have_one_catalog_order() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("hypervisor.db");
        let stop = Catalog::open(&path).unwrap();
        for id in ["stop-wins", "dispatch-wins"] {
            let mut reviewed = draft(id, "Hosted Shadow", "2026-01-02T00:00:00Z");
            reviewed.status = "review_only".into();
            reviewed.plan_json = Some("reviewed-hosted-plan".into());
            stop.upsert_flow_draft(&reviewed).unwrap();
        }
        let dispatch = Catalog::open(&path).unwrap();

        assert!(!stop
            .cancel_review_only_hosted_shadow_flow("stop-wins", "stale-plan")
            .unwrap());
        assert!(stop
            .cancel_review_only_hosted_shadow_flow("stop-wins", "reviewed-hosted-plan")
            .unwrap());
        assert!(!dispatch
            .claim_review_only_hosted_shadow_dispatch(
                "stop-wins",
                "reviewed-hosted-plan",
                "run-1",
                &owner(),
            )
            .unwrap());

        assert!(dispatch
            .claim_review_only_hosted_shadow_dispatch(
                "dispatch-wins",
                "reviewed-hosted-plan",
                "run-2",
                &owner(),
            )
            .unwrap());
        assert!(!stop
            .cancel_review_only_hosted_shadow_flow("dispatch-wins", "reviewed-hosted-plan")
            .unwrap());
        assert!(stop
            .request_routed_flow_stop("dispatch-wins", "reviewed-hosted-plan", "run-2")
            .unwrap());
        assert_eq!(
            stop.routed_flow_stop_requested("dispatch-wins", "run-2")
                .unwrap(),
            Some(true)
        );
    }

    #[test]
    fn routed_stop_request_is_durable_exact_and_prevents_a_late_claim() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("hypervisor.db");
        let cat = Catalog::open(&path).unwrap();
        for id in ["before", "during", "after"] {
            let mut ready = draft(id, "Flow", "2026-01-02T00:00:00Z");
            ready.status = "ready".into();
            ready.plan_json = Some("reviewed".into());
            cat.upsert_flow_draft(&ready).unwrap();
        }
        assert!(!cat
            .request_routed_flow_stop("before", "other", "run-1")
            .unwrap());
        assert!(cat
            .request_routed_flow_stop("before", "reviewed", "run-1")
            .unwrap());
        assert!(!cat
            .claim_routed_flow_dispatch("before", "reviewed", "run-1", &owner())
            .unwrap());
        assert_eq!(
            cat.get_flow_draft("before").unwrap().unwrap().status,
            "cancelled"
        );
        assert!(cat
            .claim_routed_flow_dispatch("during", "reviewed", "run-2", &owner())
            .unwrap());
        assert!(!cat
            .request_routed_flow_stop("during", "reviewed", "wrong-run")
            .unwrap());
        assert!(cat
            .request_routed_flow_stop("during", "reviewed", "run-2")
            .unwrap());
        assert_eq!(
            cat.routed_flow_stop_requested("during", "run-2").unwrap(),
            Some(true)
        );
        assert_eq!(
            cat.routed_flow_stop_requested("during", "wrong-run")
                .unwrap(),
            None
        );
        drop(cat);
        let reopened = Catalog::open(&path).unwrap();
        assert_eq!(
            reopened
                .routed_flow_stop_requested("during", "run-2")
                .unwrap(),
            Some(true)
        );
        assert!(reopened.mark_flow_dispatched("during", "run-2").is_err());
        reopened
            .mark_routed_flow_preflight_stopped("during", "run-2")
            .unwrap();
        assert_eq!(
            reopened.get_flow_draft("during").unwrap().unwrap().status,
            "cancelled"
        );
        assert!(reopened
            .claim_routed_flow_dispatch("after", "reviewed", "run-3", &owner())
            .unwrap());
        assert!(reopened
            .request_routed_flow_stop("after", "reviewed", "run-3")
            .unwrap());
        reopened
            .mark_routed_flow_stopped_after_quiescence("after", "run-3")
            .unwrap();
        assert_eq!(
            reopened.get_flow_draft("after").unwrap().unwrap().status,
            "cancelled"
        );
    }

    #[test]
    fn ready_stop_compare_and_swap_reports_when_dispatch_won_the_race() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("hypervisor.db");
        let stop = Catalog::open(&path).unwrap();
        for id in ["stop-wins", "dispatch-wins"] {
            let mut ready = draft(id, "Flow", "2026-01-02T00:00:00Z");
            ready.status = "ready".into();
            ready.plan_json = Some("reviewed".into());
            stop.upsert_flow_draft(&ready).unwrap();
        }
        let dispatch = Catalog::open(&path).unwrap();

        assert!(!stop.cancel_ready_routed_flow("stop-wins", "stale").unwrap());
        assert!(stop
            .cancel_ready_routed_flow("stop-wins", "reviewed")
            .unwrap());
        assert!(!dispatch
            .claim_routed_flow_dispatch("stop-wins", "reviewed", "run-1", &owner())
            .unwrap());

        // Stop observed Ready, but dispatch won before Stop's conditional write.
        assert_eq!(
            stop.get_flow_draft("dispatch-wins")
                .unwrap()
                .unwrap()
                .status,
            "ready"
        );
        assert!(dispatch
            .claim_routed_flow_dispatch("dispatch-wins", "reviewed", "run-2", &owner())
            .unwrap());
        assert!(!stop
            .cancel_ready_routed_flow("dispatch-wins", "reviewed")
            .unwrap());
        assert_eq!(
            stop.routed_flow_stop_requested("dispatch-wins", "run-2")
                .unwrap(),
            Some(false)
        );
    }

    #[test]
    fn routed_owner_claim_is_atomic_and_lost_owner_requires_exact_compare_and_swap() {
        let dir = tempfile::tempdir().unwrap();
        let cat = Catalog::open(&dir.path().join("hypervisor.db")).unwrap();
        for id in ["first", "second"] {
            let mut ready = draft(id, "Flow", "2026-01-02T00:00:00Z");
            ready.status = "ready".into();
            ready.plan_json = Some("reviewed".into());
            cat.upsert_flow_draft(&ready).unwrap();
        }
        let mut incomplete = owner();
        incomplete.controller_start_identity.clear();
        assert!(cat
            .claim_routed_flow_dispatch("first", "reviewed", "run-1", &incomplete)
            .is_err());
        let mut missing_store_identity = owner();
        missing_store_identity.store_db_file_identity = None;
        assert!(cat
            .claim_routed_flow_dispatch("first", "reviewed", "run-1", &missing_store_identity)
            .is_err());
        assert_eq!(
            cat.get_flow_draft("first").unwrap().unwrap().status,
            "ready"
        );
        assert!(cat
            .claim_routed_flow_dispatch("first", "reviewed", "run-1", &owner())
            .unwrap());
        assert!(cat
            .claim_routed_flow_dispatch("second", "reviewed", "run-1", &owner())
            .is_err());
        assert_eq!(
            cat.get_flow_draft("second").unwrap().unwrap().status,
            "ready"
        );
        let mut wrong_owner = owner();
        wrong_owner.controller_start_identity = "reused-pid".into();
        assert_eq!(
            cat.routed_flow_startup_may_have_started("first", "run-1")
                .unwrap(),
            Some(false)
        );
        assert!(!cat
            .mark_routed_flow_startup_may_have_started("first", "run-2", &owner())
            .unwrap());
        assert!(!cat
            .mark_routed_flow_startup_may_have_started("first", "run-1", &wrong_owner)
            .unwrap());
        wrong_owner = owner();
        wrong_owner.store_db_file_identity = Some("different-file".into());
        assert!(!cat
            .mark_routed_flow_startup_may_have_started("first", "run-1", &wrong_owner)
            .unwrap());
        assert!(cat
            .mark_routed_flow_startup_may_have_started("first", "run-1", &owner())
            .unwrap());
        assert!(!cat
            .mark_routed_flow_startup_may_have_started("first", "run-1", &owner())
            .unwrap());
        assert_eq!(
            cat.routed_flow_startup_may_have_started("first", "run-1")
                .unwrap(),
            Some(true)
        );
        assert!(!cat
            .mark_routed_flow_owner_lost("first", "run-1", &wrong_owner)
            .unwrap());
        assert!(!cat
            .mark_routed_flow_owner_lost("first", "run-2", &owner())
            .unwrap());
        assert_eq!(
            cat.get_flow_draft("first").unwrap().unwrap().status,
            "dispatching"
        );
        assert!(cat
            .mark_routed_flow_owner_lost("first", "run-1", &owner())
            .unwrap());
        assert!(!cat
            .mark_routed_flow_owner_lost("first", "run-1", &owner())
            .unwrap());
        assert_eq!(
            cat.get_flow_draft("first").unwrap().unwrap().status,
            "recovery_required"
        );
    }

    #[test]
    fn concurrent_routed_claims_commit_one_run_and_one_matching_owner() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("hypervisor.db");
        let cat = Catalog::open(&path).unwrap();
        let mut ready = draft("competing", "Flow", "2026-01-02T00:00:00Z");
        ready.status = "ready".into();
        ready.plan_json = Some("reviewed".into());
        cat.upsert_flow_draft(&ready).unwrap();
        drop(cat);
        let catalogs = [Catalog::open(&path).unwrap(), Catalog::open(&path).unwrap()];
        let barrier = std::sync::Arc::new(std::sync::Barrier::new(2));
        let handles = catalogs
            .into_iter()
            .zip(["run-a", "run-b"])
            .map(|(cat, run_id)| {
                let barrier = barrier.clone();
                std::thread::spawn(move || {
                    barrier.wait();
                    (
                        run_id,
                        cat.claim_routed_flow_dispatch("competing", "reviewed", run_id, &owner())
                            .unwrap(),
                    )
                })
            })
            .collect::<Vec<_>>();
        let contenders = handles
            .into_iter()
            .map(|handle| handle.join().unwrap())
            .collect::<Vec<_>>();
        assert_eq!(contenders.iter().filter(|(_, won)| *won).count(), 1);
        let winner = contenders.iter().find(|(_, won)| *won).unwrap().0;
        let cat = Catalog::open(&path).unwrap();
        let row = cat.get_flow_draft("competing").unwrap().unwrap();
        assert_eq!(row.status, "dispatching");
        assert_eq!(row.dispatched_run_id.as_deref(), Some(winner));
        assert_eq!(
            cat.routed_flow_dispatch_owner("competing", winner).unwrap(),
            Some(owner())
        );
        let count: i64 = cat
            .conn
            .query_row("SELECT COUNT(*) FROM flow_dispatch_claims", [], |row| {
                row.get(0)
            })
            .unwrap();
        assert_eq!(count, 1);
    }

    #[test]
    fn routed_recovery_keeps_exact_run_link_and_rejects_replacement() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("hypervisor.db");
        let cat = Catalog::open(&path).unwrap();
        let mut ready = draft("recovering-flow", "Flow", "2026-01-02T00:00:00Z");
        ready.status = "ready".into();
        ready.plan_json = Some("{\"reviewed\":true}".into());
        cat.upsert_flow_draft(&ready).unwrap();
        assert!(cat
            .claim_routed_flow_dispatch(
                "recovering-flow",
                "{\"reviewed\":true}",
                "run-42",
                &owner()
            )
            .unwrap());
        assert!(cat
            .mark_routed_flow_recovery_required("recovering-flow", "run-43")
            .is_err());
        assert_eq!(
            cat.get_flow_draft("recovering-flow")
                .unwrap()
                .unwrap()
                .status,
            "dispatching"
        );
        cat.mark_routed_flow_recovery_required("recovering-flow", "run-42")
            .unwrap();
        cat.mark_routed_flow_recovery_required("recovering-flow", "run-42")
            .unwrap();
        assert!(cat
            .mark_routed_flow_startup_failed("recovering-flow", "run-42")
            .is_err());
        assert!(!cat
            .claim_routed_flow_dispatch(
                "recovering-flow",
                "{\"reviewed\":true}",
                "run-42",
                &owner()
            )
            .unwrap());
        assert!(cat
            .mark_flow_dispatched("recovering-flow", "run-42")
            .is_err());
        let mut replacement = ready.clone();
        replacement.title = "replacement".into();
        assert!(!cat.upsert_flow_draft_intent(&replacement).unwrap());
        assert!(!cat.upsert_flow_preview(&replacement).unwrap());
        assert!(cat.upsert_flow_draft(&replacement).is_err());
        assert!(cat.delete_flow_draft("recovering-flow").is_err());
        drop(cat);
        let reopened = Catalog::open(&path).unwrap();
        let recovering = reopened.get_flow_draft("recovering-flow").unwrap().unwrap();
        assert_eq!(recovering.status, "recovery_required");
        assert_eq!(recovering.dispatched_run_id.as_deref(), Some("run-42"));
        assert!(reopened
            .mark_routed_flow_recovered_failed("recovering-flow", "run-43")
            .is_err());
        reopened
            .mark_routed_flow_recovered_failed("recovering-flow", "run-42")
            .unwrap();
        reopened
            .mark_routed_flow_recovered_failed("recovering-flow", "run-42")
            .unwrap();
        assert_eq!(
            reopened
                .get_flow_draft("recovering-flow")
                .unwrap()
                .unwrap()
                .status,
            "failed"
        );
    }

    #[test]
    fn routed_terminal_ack_recovery_preserves_reviewed_run_identity() {
        let dir = tempfile::tempdir().unwrap();
        let cat = Catalog::open(&dir.path().join("hypervisor.db")).unwrap();
        for (draft_id, recovering) in [("terminal-direct", false), ("terminal-retry", true)] {
            let run_id = if recovering { "run-43" } else { "run-42" };
            let mut ready = draft(draft_id, "Flow", "2026-01-02T00:00:00Z");
            ready.status = "ready".into();
            ready.plan_json = Some("{\"reviewed\":true}".into());
            cat.upsert_flow_draft(&ready).unwrap();
            assert!(cat
                .claim_routed_flow_dispatch(draft_id, "{\"reviewed\":true}", run_id, &owner())
                .unwrap());
            if recovering {
                cat.mark_routed_flow_recovery_required(draft_id, run_id)
                    .unwrap();
            }
            assert!(cat
                .mark_routed_flow_recovered_dispatched(draft_id, "run-other")
                .is_err());
            cat.mark_routed_flow_recovered_dispatched(draft_id, run_id)
                .unwrap();
            cat.mark_routed_flow_recovered_dispatched(draft_id, run_id)
                .unwrap();
            let row = cat.get_flow_draft(draft_id).unwrap().unwrap();
            assert_eq!(row.status, "dispatched");
            assert_eq!(row.dispatched_run_id.as_deref(), Some(run_id));
            assert!(cat
                .mark_routed_flow_recovered_dispatched(draft_id, "run-other")
                .is_err());
        }
    }

    #[test]
    fn dispatch_claim_blocks_stale_review_save_and_delete() {
        let dir = tempfile::tempdir().unwrap();
        let cat = Catalog::open(&dir.path().join("hypervisor.db")).unwrap();
        let mut ready = draft("flow-race", "Flow", "2026-01-02T00:00:00Z");
        ready.status = "ready".into();
        ready.plan_json = Some("{\"version\":1}".into());
        cat.upsert_flow_draft(&ready).unwrap();
        assert!(cat
            .claim_flow_dispatch("flow-race", "{\"version\":1}")
            .unwrap());
        assert!(!cat
            .replace_ready_flow_plan("flow-race", "{\"version\":1}", "{\"version\":2}")
            .unwrap());
        assert!(cat.delete_flow_draft("flow-race").is_err());
        let mut reopened = ready.clone();
        reopened.mission_text = "changed after claim".into();
        assert!(!cat.upsert_flow_draft_intent(&reopened).unwrap());
        assert!(!cat.upsert_flow_preview(&reopened).unwrap());
        assert_eq!(
            cat.get_flow_draft("flow-race").unwrap().unwrap().status,
            "dispatching"
        );
    }

    #[test]
    fn dispatch_claim_is_bound_to_the_exact_reviewed_snapshot() {
        let dir = tempfile::tempdir().unwrap();
        let cat = Catalog::open(&dir.path().join("hypervisor.db")).unwrap();
        let mut ready = draft("flow-snapshot", "Flow", "2026-01-02T00:00:00Z");
        ready.status = "ready".into();
        ready.plan_json = Some("{\"version\":1}".into());
        cat.upsert_flow_draft(&ready).unwrap();
        assert!(cat
            .replace_ready_flow_plan("flow-snapshot", "{\"version\":1}", "{\"version\":2}")
            .unwrap());
        assert!(!cat
            .claim_flow_dispatch("flow-snapshot", "{\"version\":1}")
            .unwrap());
        assert!(cat
            .claim_flow_dispatch("flow-snapshot", "{\"version\":2}")
            .unwrap());
    }

    #[test]
    fn upsert_is_idempotent_and_lists() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("hypervisor.db");
        let cat = Catalog::open(&path).unwrap();
        cat.upsert_domain("/repo/a", "/repo/a", "/repo/a/.pytxo/data/pytxo.db", None)
            .unwrap();
        cat.upsert_domain(
            "/repo/a",
            "/repo/a",
            "/repo/a/.pytxo/data/pytxo.db",
            Some("proj-1"),
        )
        .unwrap();
        cat.upsert_domain("/repo/b", "/repo/b", "/repo/b/.pytxo/data/pytxo.db", None)
            .unwrap();
        let list = cat.list_domains().unwrap();
        assert_eq!(list.len(), 2);
        let a = list.iter().find(|e| e.domain_id == "/repo/a").unwrap();
        assert_eq!(a.project_id.as_deref(), Some("proj-1"));
    }

    #[test]
    fn project_roots_round_trip() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("hypervisor.db");
        let cat = Catalog::open(&path).unwrap();
        cat.upsert_project_root("acme", "api", "/repo/api", false, true, Some("orbit"))
            .unwrap();
        cat.upsert_project_root("acme", "web", "/repo/web", true, false, None)
            .unwrap();
        let roots = cat.list_project_roots("acme").unwrap();
        assert_eq!(roots.len(), 2);
        assert!(roots.iter().any(|r| r.label == "api" && r.primary));
    }

    #[test]
    fn fleet_run_lifecycle() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("hypervisor.db");
        let cat = Catalog::open(&path).unwrap();
        cat.insert_fleet_run("fr-1", "api-then-web", "2026-01-01T00:00:00Z")
            .unwrap();
        assert_eq!(cat.active_fleet_count().unwrap(), 1);
        cat.insert_fleet_node("fr-1", "fix-api", "/repo/a", Some("run-a"), 0, "completed")
            .unwrap();
        cat.insert_fleet_node("fr-1", "deploy-web", "/repo/b", Some("run-b"), 1, "running")
            .unwrap();
        cat.update_fleet_node_status("fr-1", "deploy-web", "completed", Some("run-b"))
            .unwrap();
        cat.finish_fleet_run("fr-1", "completed", "2026-01-01T00:05:00Z")
            .unwrap();
        assert_eq!(cat.active_fleet_count().unwrap(), 0);
        let runs = cat.list_fleet_runs(Some("api-then-web"), 10).unwrap();
        assert_eq!(runs.len(), 1);
        assert_eq!(runs[0].status, "completed");
        let nodes = cat.list_fleet_nodes("fr-1").unwrap();
        assert_eq!(nodes.len(), 2);
        assert_eq!(nodes[1].status, "completed");
    }
}
