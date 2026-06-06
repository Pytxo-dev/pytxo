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
}
