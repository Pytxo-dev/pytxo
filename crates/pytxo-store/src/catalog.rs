//! Cross-project hypervisor catalog (`~/.pytxo/hypervisor.db`).
//!
//! A lightweight registry of every execution domain the local hypervisor has
//! seen, so the Reality Deck can show an "all projects" home without merging the
//! per-domain WAL streams ([[execution-domains]] v2). Each domain keeps its own
//! `pytxo.db`; this catalog only records where to find them.

use std::path::{Path, PathBuf};

use chrono::Utc;
use pytxo_core::{PytxoError, Result};
use rusqlite::{params, Connection};
use serde::Serialize;

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
"#;

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

    pub fn list_fleet_runs(&self, fleet_id: Option<&str>, limit: usize) -> Result<Vec<FleetRunRecord>> {
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
pub fn default_catalog_path() -> Option<PathBuf> {
    std::env::var_os("HOME")
        .or_else(|| std::env::var_os("USERPROFILE"))
        .map(PathBuf::from)
        .map(|h| h.join(".pytxo").join("hypervisor.db"))
}

#[cfg(test)]
mod tests {
    use super::*;

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
