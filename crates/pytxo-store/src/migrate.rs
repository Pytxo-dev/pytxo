use pytxo_core::{PytxoError, Result};
use rusqlite::Connection;

use crate::schema::{MIGRATION_001, MIGRATION_002, MIGRATION_003, MIGRATION_004};

pub fn apply_migrations(conn: &Connection) -> Result<()> {
    let mut version: i32 = conn
        .pragma_query_value(None, "user_version", |row| row.get(0))
        .map_err(|e| PytxoError::Store(e.to_string()))?;

    if version < 1 {
        conn.execute_batch(MIGRATION_001)
            .map_err(|e| PytxoError::Store(e.to_string()))?;
        conn.pragma_update(None, "user_version", 1)
            .map_err(|e| PytxoError::Store(e.to_string()))?;
        version = 1;
    }

    if version < 2 {
        for stmt in MIGRATION_002.split(';').filter(|s| !s.trim().is_empty()) {
            let _ = conn.execute(stmt.trim(), []);
        }
        conn.pragma_update(None, "user_version", 2)
            .map_err(|e| PytxoError::Store(e.to_string()))?;
        version = 2;
    }

    if version < 3 {
        conn.execute_batch(MIGRATION_003)
            .map_err(|e| PytxoError::Store(e.to_string()))?;
        conn.pragma_update(None, "user_version", 3)
            .map_err(|e| PytxoError::Store(e.to_string()))?;
        version = 3;
    }

    if version < 4 {
        // ALTER statements are not idempotent; ignore "duplicate column" on re-run.
        for stmt in MIGRATION_004.split(';').filter(|s| !s.trim().is_empty()) {
            let _ = conn.execute(stmt.trim(), []);
        }
        conn.pragma_update(None, "user_version", 4)
            .map_err(|e| PytxoError::Store(e.to_string()))?;
    }

    Ok(())
}
