//! Cross-project hypervisor catalog (`~/.pytxo/hypervisor.db`).
//!
//! A lightweight registry of every execution domain the local hypervisor has
//! seen, so the Reality Deck can show an "all projects" home without merging the
//! per-domain WAL streams ([[execution-domains]] v2). Each domain keeps its own
//! `pytxo.db`; this catalog only records where to find them.

use std::path::{Path, PathBuf};

use chrono::Utc;
use pytxo_core::{PytxoError, Result};
use rusqlite::{params, Connection, OptionalExtension};
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
BEGIN IMMEDIATE;
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
COMMIT;
"#;

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

pub struct Catalog {
    conn: Connection,
}

impl Catalog {
    pub fn open(path: &Path) -> Result<Self> {
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent).map_err(PytxoError::Io)?;
        }
        let conn = Connection::open(path).map_err(|e| PytxoError::Store(e.to_string()))?;
        conn.execute_batch("PRAGMA journal_mode=WAL;")
            .map_err(|e| PytxoError::Store(e.to_string()))?;
        conn.execute_batch(CATALOG_SCHEMA)
            .map_err(|e| PytxoError::Store(e.to_string()))?;
        migrate_catalog(&conn)?;
        Ok(Self { conn })
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
                    updated_at = excluded.updated_at",
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
            .map_err(|e| PytxoError::Store(e.to_string()))?;
        Ok(())
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
                 WHERE flow_drafts.status NOT IN ('dispatching', 'dispatched')",
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
                 WHERE flow_drafts.status NOT IN ('dispatching', 'dispatched')",
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
                "DELETE FROM flow_drafts WHERE id = ?1 AND status != 'dispatching'",
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
                "cannot delete a Flow draft while dispatching".into(),
            ));
        }
        Ok(())
    }

    /// Replace only the reviewed plan payload while the exact ready preview is still current.
    pub fn replace_ready_flow_plan(
        &self,
        id: &str,
        expected_plan_json: &str,
        reviewed_plan_json: &str,
    ) -> Result<bool> {
        self.conn
            .execute(
                "UPDATE flow_drafts SET plan_json = ?1, updated_at = ?2
                 WHERE id = ?3 AND status = 'ready' AND dispatched_run_id IS NULL
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
                    updated_at = ?2 WHERE id = ?3 AND status = 'dispatching'",
                params![run_id, Utc::now().to_rfc3339(), id],
            )
            .map_err(|e| PytxoError::Store(e.to_string()))?;
        if changed == 0 {
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

    pub fn mark_flow_dispatch_failed(&self, id: &str) -> Result<()> {
        self.conn
            .execute(
                "UPDATE flow_drafts SET status = 'failed', updated_at = ?1
                 WHERE id = ?2 AND status = 'dispatching'",
                params![Utc::now().to_rfc3339(), id],
            )
            .map_err(|e| PytxoError::Store(e.to_string()))?;
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

fn migrate_catalog(conn: &Connection) -> Result<()> {
    let version: i64 = conn
        .query_row("PRAGMA user_version", [], |row| row.get(0))
        .map_err(|e| PytxoError::Store(e.to_string()))?;
    if version < 1 {
        conn.execute_batch(FLOW_DRAFTS_MIGRATION)
            .map_err(|e| PytxoError::Store(e.to_string()))?;
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
        let loaded = cat.get_flow_draft("flow-1").unwrap().unwrap();
        assert_eq!(loaded.status, "dispatched");
        assert_eq!(loaded.dispatched_run_id.as_deref(), Some("run-42"));
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
        cat.insert_fleet_node("fr-1", "fix-api", "/repo/a", Some("run-a"), 0, "completed")
            .unwrap();
        cat.insert_fleet_node("fr-1", "deploy-web", "/repo/b", Some("run-b"), 1, "running")
            .unwrap();
        cat.update_fleet_node_status("fr-1", "deploy-web", "completed", Some("run-b"))
            .unwrap();
        cat.finish_fleet_run("fr-1", "completed", "2026-01-01T00:05:00Z")
            .unwrap();
        let runs = cat.list_fleet_runs(Some("api-then-web"), 10).unwrap();
        assert_eq!(runs.len(), 1);
        assert_eq!(runs[0].status, "completed");
        let nodes = cat.list_fleet_nodes("fr-1").unwrap();
        assert_eq!(nodes.len(), 2);
        assert_eq!(nodes[1].status, "completed");
    }
}
