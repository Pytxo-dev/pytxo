use pytxo_core::{PytxoError, Result};
use rusqlite::Connection;

use crate::schema::{
    MIGRATION_001, MIGRATION_002, MIGRATION_003, MIGRATION_004, MIGRATION_005, MIGRATION_006,
    MIGRATION_007,
};

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
        version = 4;
    }

    if version < 5 {
        for stmt in MIGRATION_005.split(';').filter(|s| !s.trim().is_empty()) {
            let _ = conn.execute(stmt.trim(), []);
        }
        conn.pragma_update(None, "user_version", 5)
            .map_err(|e| PytxoError::Store(e.to_string()))?;
        version = 5;
    }

    if version < 6 {
        conn.execute_batch(MIGRATION_006)
            .map_err(|e| PytxoError::Store(e.to_string()))?;
        conn.pragma_update(None, "user_version", 6)
            .map_err(|e| PytxoError::Store(e.to_string()))?;
        version = 6;
    }

    if version < 7 {
        apply_migration_007(conn, MIGRATION_007)?;
    }

    Ok(())
}

fn apply_migration_007(conn: &Connection, sql: &str) -> Result<()> {
    let transaction = conn
        .unchecked_transaction()
        .map_err(|error| PytxoError::Store(error.to_string()))?;
    transaction
        .execute_batch(sql)
        .map_err(|error| PytxoError::Store(error.to_string()))?;
    transaction
        .pragma_update(None, "user_version", 7)
        .map_err(|error| PytxoError::Store(error.to_string()))?;
    transaction
        .commit()
        .map_err(|error| PytxoError::Store(error.to_string()))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn migration_seven_upgrades_an_existing_v6_contract_database() {
        let conn = Connection::open_in_memory().unwrap();
        conn.execute_batch(crate::schema::MIGRATION_001).unwrap();
        for migration in [crate::schema::MIGRATION_002, crate::schema::MIGRATION_004] {
            for stmt in migration.split(';').filter(|stmt| !stmt.trim().is_empty()) {
                conn.execute(stmt.trim(), []).unwrap();
            }
        }
        conn.execute_batch(crate::schema::MIGRATION_003).unwrap();
        conn.execute(
            crate::schema::MIGRATION_005.trim().trim_end_matches(';'),
            [],
        )
        .unwrap();
        conn.execute_batch(crate::schema::MIGRATION_006).unwrap();
        conn.pragma_update(None, "user_version", 6).unwrap();

        apply_migrations(&conn).unwrap();

        let version: i32 = conn
            .pragma_query_value(None, "user_version", |row| row.get(0))
            .unwrap();
        assert_eq!(version, 7);
        let columns: Vec<String> = conn
            .prepare("PRAGMA table_info(run_contracts)")
            .unwrap()
            .query_map([], |row| row.get(1))
            .unwrap()
            .collect::<std::result::Result<_, _>>()
            .unwrap();
        assert!(columns.contains(&"prepared_manifest_json".to_string()));
        let table_exists: i64 = conn
            .query_row(
                "SELECT COUNT(*) FROM sqlite_master WHERE type='table' AND name='domain_changes'",
                [],
                |row| row.get(0),
            )
            .unwrap();
        assert_eq!(table_exists, 1);
    }

    #[test]
    fn migration_seven_rolls_back_partial_schema_and_can_retry() {
        let conn = Connection::open_in_memory().unwrap();
        conn.execute_batch(crate::schema::MIGRATION_001).unwrap();
        for migration in [crate::schema::MIGRATION_002, crate::schema::MIGRATION_004] {
            for stmt in migration.split(';').filter(|stmt| !stmt.trim().is_empty()) {
                conn.execute(stmt.trim(), []).unwrap();
            }
        }
        conn.execute_batch(crate::schema::MIGRATION_003).unwrap();
        conn.execute(
            crate::schema::MIGRATION_005.trim().trim_end_matches(';'),
            [],
        )
        .unwrap();
        conn.execute_batch(crate::schema::MIGRATION_006).unwrap();
        conn.pragma_update(None, "user_version", 6).unwrap();

        let broken = format!(
            "{}\nALTER TABLE missing_table ADD COLUMN impossible TEXT;",
            crate::schema::MIGRATION_007
        );
        apply_migration_007(&conn, &broken).expect_err("migration must fail atomically");

        let version: i32 = conn
            .pragma_query_value(None, "user_version", |row| row.get(0))
            .unwrap();
        assert_eq!(version, 6);
        let columns: Vec<String> = conn
            .prepare("PRAGMA table_info(run_contracts)")
            .unwrap()
            .query_map([], |row| row.get(1))
            .unwrap()
            .collect::<std::result::Result<_, _>>()
            .unwrap();
        assert!(!columns.contains(&"prepared_manifest_json".to_string()));
        let table_exists: i64 = conn
            .query_row(
                "SELECT COUNT(*) FROM sqlite_master WHERE type='table' AND name='domain_changes'",
                [],
                |row| row.get(0),
            )
            .unwrap();
        assert_eq!(table_exists, 0);

        apply_migrations(&conn).expect("clean retry");
        let version: i32 = conn
            .pragma_query_value(None, "user_version", |row| row.get(0))
            .unwrap();
        assert_eq!(version, 7);
    }
}
