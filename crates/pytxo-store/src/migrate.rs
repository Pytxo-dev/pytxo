use pytxo_core::{PytxoError, Result};
use rusqlite::Connection;

use crate::schema::{
    MIGRATION_001, MIGRATION_002, MIGRATION_003, MIGRATION_004, MIGRATION_005, MIGRATION_006,
    MIGRATION_007,
};

const SCHEMA_001: &[(&str, &[&str])] = &[
    (
        "runs",
        &["id", "started_at", "finished_at", "status", "repo_root"],
    ),
    (
        "agents",
        &[
            "id",
            "run_id",
            "task_id",
            "wave",
            "worktree_path",
            "cmd",
            "exit_code",
            "status",
        ],
    ),
    ("events", &["id", "agent_id", "ts", "kind", "payload"]),
];

const SCHEMA_003: &[(&str, &[&str])] = &[
    (
        "wallet_accounts",
        &[
            "domain_id",
            "balance_microcredits",
            "reserved_microcredits",
            "synced_at",
        ],
    ),
    (
        "wallet_reservations",
        &[
            "id",
            "domain_id",
            "run_id",
            "amount_microcredits",
            "status",
            "created_at",
        ],
    ),
    (
        "arbitrage_samples",
        &[
            "id",
            "run_id",
            "agent_id",
            "domain_id",
            "path",
            "raw_bytes",
            "scaffolded_bytes",
            "raw_tokens_in",
            "sent_tokens_in",
            "saved_tokens",
            "reduction_pct",
            "fallback_raw",
            "ts",
        ],
    ),
    (
        "usage_records",
        &[
            "id",
            "run_id",
            "agent_id",
            "domain_id",
            "model",
            "tokens_in_billed",
            "tokens_in_sent",
            "tokens_out",
            "cost_micro_usd",
            "source",
            "ts",
        ],
    ),
];

const SCHEMA_006: &[(&str, &[&str])] = &[(
    "run_contracts",
    &[
        "run_id",
        "base_revision",
        "plan_json",
        "apply_status",
        "apply_manifest_json",
        "applied_at",
        "enforcement_json",
    ],
)];

const SCHEMA_007: &[(&str, &[&str])] = &[
    (
        "run_contracts",
        &[
            "prepared_manifest_json",
            "prepared_digest",
            "prepared_at",
            "last_apply_error_json",
            "recovery_state",
        ],
    ),
    (
        "domain_changes",
        &["sequence", "entity_kind", "entity_id", "changed_at"],
    ),
];

pub fn apply_migrations(conn: &Connection) -> Result<()> {
    let mut version: i32 = conn
        .pragma_query_value(None, "user_version", |row| row.get(0))
        .map_err(store_error)?;

    if version < 1 {
        apply_batch_migration(conn, 1, MIGRATION_001, SCHEMA_001)?;
        version = 1;
    }
    if version < 2 {
        apply_column_migration(conn, 2, MIGRATION_002)?;
        version = 2;
    }
    if version < 3 {
        apply_batch_migration(conn, 3, MIGRATION_003, SCHEMA_003)?;
        version = 3;
    }
    if version < 4 {
        apply_column_migration(conn, 4, MIGRATION_004)?;
        version = 4;
    }
    if version < 5 {
        apply_column_migration(conn, 5, MIGRATION_005)?;
        version = 5;
    }
    if version < 6 {
        apply_batch_migration(conn, 6, MIGRATION_006, SCHEMA_006)?;
        version = 6;
    }
    if version < 7 {
        apply_migration_007(conn, MIGRATION_007)?;
    }
    verify_latest_schema(conn)
}

fn apply_batch_migration(
    conn: &Connection,
    target_version: i32,
    sql: &str,
    required_schema: &[(&str, &[&str])],
) -> Result<()> {
    let transaction = conn.unchecked_transaction().map_err(store_error)?;
    transaction.execute_batch(sql).map_err(store_error)?;
    verify_schema(&transaction, required_schema)?;
    transaction
        .pragma_update(None, "user_version", target_version)
        .map_err(store_error)?;
    transaction.commit().map_err(store_error)
}

fn apply_column_migration(conn: &Connection, target_version: i32, sql: &str) -> Result<()> {
    let columns = parse_add_columns(target_version, sql)?;
    let transaction = conn.unchecked_transaction().map_err(store_error)?;
    for (table, column, definition) in &columns {
        if !table_exists(&transaction, table)? {
            return Err(PytxoError::Store(format!(
                "migration {target_version} requires missing table '{table}'"
            )));
        }
        if column_exists(&transaction, table, column)? {
            continue;
        }
        let sql = format!(
            "ALTER TABLE {} ADD COLUMN {} {definition}",
            quote_identifier(table),
            quote_identifier(column)
        );
        transaction.execute(&sql, []).map_err(store_error)?;
    }

    for (table, column, _) in &columns {
        if !column_exists(&transaction, table, column)? {
            return Err(PytxoError::Store(format!(
                "migration {target_version} did not create required column '{table}.{column}'"
            )));
        }
    }
    transaction
        .pragma_update(None, "user_version", target_version)
        .map_err(store_error)?;
    transaction.commit().map_err(store_error)
}

fn parse_add_columns(target_version: i32, sql: &str) -> Result<Vec<(String, String, String)>> {
    sql.split(';')
        .filter(|statement| !statement.trim().is_empty())
        .map(|statement| {
            let tokens: Vec<_> = statement.split_whitespace().collect();
            match tokens.as_slice() {
                [alter, table_keyword, table, add, column_keyword, column, definition]
                    if alter.eq_ignore_ascii_case("ALTER")
                        && table_keyword.eq_ignore_ascii_case("TABLE")
                        && add.eq_ignore_ascii_case("ADD")
                        && column_keyword.eq_ignore_ascii_case("COLUMN") =>
                {
                    Ok((
                        (*table).to_string(),
                        (*column).to_string(),
                        (*definition).to_string(),
                    ))
                }
                _ => Err(PytxoError::Store(format!(
                    "migration {target_version} contains an unsupported schema statement"
                ))),
            }
        })
        .collect()
}

fn apply_migration_007(conn: &Connection, sql: &str) -> Result<()> {
    apply_batch_migration(conn, 7, sql, SCHEMA_007)
}

fn verify_schema(conn: &Connection, required_schema: &[(&str, &[&str])]) -> Result<()> {
    for (table, columns) in required_schema {
        if !table_exists(conn, table)? {
            return Err(PytxoError::Store(format!(
                "migration schema verification failed: missing table '{table}'"
            )));
        }
        for column in *columns {
            if !column_exists(conn, table, column)? {
                return Err(PytxoError::Store(format!(
                    "migration schema verification failed: missing column '{table}.{column}'"
                )));
            }
        }
    }
    Ok(())
}

fn verify_latest_schema(conn: &Connection) -> Result<()> {
    for required in [SCHEMA_001, SCHEMA_003, SCHEMA_006, SCHEMA_007] {
        verify_schema(conn, required)?;
    }
    for (version, sql) in [(2, MIGRATION_002), (4, MIGRATION_004), (5, MIGRATION_005)] {
        for (table, column, _) in parse_add_columns(version, sql)? {
            if !column_exists(conn, &table, &column)? {
                return Err(PytxoError::Store(format!(
                    "latest schema verification failed: missing column '{table}.{column}'"
                )));
            }
        }
    }
    Ok(())
}

fn table_exists(conn: &Connection, table: &str) -> Result<bool> {
    conn.query_row(
        "SELECT EXISTS(SELECT 1 FROM sqlite_master WHERE type = 'table' AND name = ?1)",
        [table],
        |row| row.get(0),
    )
    .map_err(store_error)
}

fn column_exists(conn: &Connection, table: &str, column: &str) -> Result<bool> {
    let sql = format!("PRAGMA table_info({})", quote_identifier(table));
    let mut statement = conn.prepare(&sql).map_err(store_error)?;
    let mut rows = statement.query([]).map_err(store_error)?;
    while let Some(row) = rows.next().map_err(store_error)? {
        let existing: String = row.get(1).map_err(store_error)?;
        if existing == column {
            return Ok(true);
        }
    }
    Ok(false)
}

fn quote_identifier(identifier: &str) -> String {
    format!("\"{}\"", identifier.replace('"', "\"\""))
}

fn store_error(error: rusqlite::Error) -> PytxoError {
    PytxoError::Store(error.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn set_version(conn: &Connection, version: i32) {
        conn.pragma_update(None, "user_version", version).unwrap();
    }

    fn apply_historical_column_sql(conn: &Connection, sql: &str) {
        for statement in sql.split(';').filter(|stmt| !stmt.trim().is_empty()) {
            conn.execute(statement.trim(), []).unwrap();
        }
    }

    fn released_schema_fixture(version: i32) -> Connection {
        let conn = Connection::open_in_memory().unwrap();
        if version >= 1 {
            conn.execute_batch(crate::schema::MIGRATION_001).unwrap();
        }
        if version >= 2 {
            apply_historical_column_sql(&conn, crate::schema::MIGRATION_002);
        }
        if version >= 3 {
            conn.execute_batch(crate::schema::MIGRATION_003).unwrap();
        }
        if version >= 4 {
            apply_historical_column_sql(&conn, crate::schema::MIGRATION_004);
        }
        if version >= 5 {
            apply_historical_column_sql(&conn, crate::schema::MIGRATION_005);
        }
        if version >= 6 {
            conn.execute_batch(crate::schema::MIGRATION_006).unwrap();
        }
        set_version(&conn, version);
        conn
    }

    #[test]
    fn every_released_schema_upgrades_to_latest() {
        for source_version in 0..=6 {
            let conn = released_schema_fixture(source_version);
            apply_migrations(&conn)
                .unwrap_or_else(|error| panic!("upgrade from v{source_version} failed: {error}"));

            let version: i32 = conn
                .pragma_query_value(None, "user_version", |row| row.get(0))
                .unwrap();
            assert_eq!(version, 7, "source version {source_version}");
            verify_schema(&conn, SCHEMA_001).unwrap();
            verify_schema(&conn, SCHEMA_003).unwrap();
            verify_schema(&conn, SCHEMA_006).unwrap();
            verify_schema(&conn, SCHEMA_007).unwrap();
        }
    }

    #[test]
    fn column_migration_tolerates_only_a_verified_existing_column() {
        let conn = released_schema_fixture(1);
        conn.execute(
            "ALTER TABLE runs ADD COLUMN estimated_tokens_in INTEGER",
            [],
        )
        .unwrap();

        apply_migrations(&conn).unwrap();

        let version: i32 = conn
            .pragma_query_value(None, "user_version", |row| row.get(0))
            .unwrap();
        assert_eq!(version, 7);
        for (table, column, _) in parse_add_columns(2, MIGRATION_002).unwrap() {
            assert!(column_exists(&conn, &table, &column).unwrap());
        }
    }

    #[test]
    fn column_migration_failure_rolls_back_schema_and_version() {
        let conn = released_schema_fixture(1);
        conn.execute_batch("DROP TABLE agents").unwrap();

        apply_migrations(&conn).expect_err("missing table must fail migration");

        let version: i32 = conn
            .pragma_query_value(None, "user_version", |row| row.get(0))
            .unwrap();
        assert_eq!(version, 1);
        assert!(!column_exists(&conn, "runs", "estimated_tokens_in").unwrap());
    }

    #[test]
    fn malformed_existing_schema_does_not_advance_version() {
        let conn = Connection::open_in_memory().unwrap();
        conn.execute_batch("CREATE TABLE runs (id TEXT PRIMARY KEY)")
            .unwrap();

        apply_migrations(&conn).expect_err("schema verification must fail closed");

        let version: i32 = conn
            .pragma_query_value(None, "user_version", |row| row.get(0))
            .unwrap();
        assert_eq!(version, 0);
        assert!(!table_exists(&conn, "agents").unwrap());
    }

    #[test]
    fn falsely_advanced_current_version_fails_schema_verification() {
        let conn = Connection::open_in_memory().unwrap();
        conn.execute_batch("CREATE TABLE runs (id TEXT PRIMARY KEY)")
            .unwrap();
        set_version(&conn, 7);

        apply_migrations(&conn).expect_err("current version with missing schema must fail closed");

        let version: i32 = conn
            .pragma_query_value(None, "user_version", |row| row.get(0))
            .unwrap();
        assert_eq!(version, 7);
    }

    #[test]
    fn migration_seven_rolls_back_partial_schema_and_can_retry() {
        let conn = released_schema_fixture(6);
        let broken = format!(
            "{}\nALTER TABLE missing_table ADD COLUMN impossible TEXT;",
            crate::schema::MIGRATION_007
        );
        apply_migration_007(&conn, &broken).expect_err("migration must fail atomically");

        let version: i32 = conn
            .pragma_query_value(None, "user_version", |row| row.get(0))
            .unwrap();
        assert_eq!(version, 6);
        for column in [
            "prepared_manifest_json",
            "prepared_digest",
            "prepared_at",
            "last_apply_error_json",
            "recovery_state",
        ] {
            assert!(!column_exists(&conn, "run_contracts", column).unwrap());
        }
        assert!(!table_exists(&conn, "domain_changes").unwrap());

        apply_migrations(&conn).expect("clean retry");
        let version: i32 = conn
            .pragma_query_value(None, "user_version", |row| row.get(0))
            .unwrap();
        assert_eq!(version, 7);
    }
}
