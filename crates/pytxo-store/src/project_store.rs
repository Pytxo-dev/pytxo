//! Project-scoped telemetry registry (`~/.pytxo/projects/<id>/pytxo.db`, Phase 25).

use std::path::Path;

use pytxo_core::{PermissionProfile, ProjectManifest, PytxoError, Result};
use rusqlite::{params, Connection};

const PROJECT_SCHEMA: &str = r#"
CREATE TABLE IF NOT EXISTS roots (
    label TEXT PRIMARY KEY,
    path TEXT NOT NULL,
    read_only INTEGER NOT NULL DEFAULT 0,
    permission_profile TEXT,
    primary_flag INTEGER NOT NULL DEFAULT 0
);
"#;

pub struct ProjectStore {
    conn: Connection,
}

impl ProjectStore {
    pub fn open(path: &Path) -> Result<Self> {
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent).map_err(PytxoError::Io)?;
        }
        let conn = Connection::open(path).map_err(|e| PytxoError::Store(e.to_string()))?;
        conn.execute_batch("PRAGMA journal_mode=WAL;")
            .map_err(|e| PytxoError::Store(e.to_string()))?;
        conn.execute_batch(PROJECT_SCHEMA)
            .map_err(|e| PytxoError::Store(e.to_string()))?;
        Ok(Self { conn })
    }

    /// Upsert manifest roots into the project DB.
    pub fn sync_roots(&self, manifest: &ProjectManifest) -> Result<()> {
        let tx = self
            .conn
            .unchecked_transaction()
            .map_err(|e| PytxoError::Store(e.to_string()))?;
        tx.execute("DELETE FROM roots", [])
            .map_err(|e| PytxoError::Store(e.to_string()))?;
        for r in &manifest.roots {
            let profile = r
                .permission_profile
                .map(|p| p.as_str().to_string());
            tx.execute(
                "INSERT INTO roots (label, path, read_only, permission_profile, primary_flag)
                 VALUES (?1, ?2, ?3, ?4, ?5)",
                params![
                    r.effective_label(),
                    r.path.to_string_lossy(),
                    r.read_only as i32,
                    profile,
                    r.primary as i32,
                ],
            )
            .map_err(|e| PytxoError::Store(e.to_string()))?;
        }
        tx.commit().map_err(|e| PytxoError::Store(e.to_string()))?;
        Ok(())
    }

    pub fn list_roots(
        &self,
    ) -> Result<Vec<(String, String, bool, bool, Option<PermissionProfile>)>> {
        let mut stmt = self
            .conn
            .prepare(
                "SELECT label, path, read_only, permission_profile, primary_flag FROM roots ORDER BY primary_flag DESC, label",
            )
            .map_err(|e| PytxoError::Store(e.to_string()))?;
        let rows = stmt
            .query_map([], |row| {
                let profile: Option<String> = row.get(3)?;
                Ok((
                    row.get(0)?,
                    row.get(1)?,
                    row.get::<_, i32>(2)? != 0,
                    row.get::<_, i32>(4)? != 0,
                    profile.and_then(|s| PermissionProfile::parse(&s)),
                ))
            })
            .map_err(|e| PytxoError::Store(e.to_string()))?;
        rows.collect::<std::result::Result<Vec<_>, _>>()
            .map_err(|e| PytxoError::Store(e.to_string()))
    }
}
