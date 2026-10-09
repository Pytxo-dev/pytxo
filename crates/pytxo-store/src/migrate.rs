use pytxo_core::{PytxoError, Result};
use rusqlite::{Connection, OptionalExtension};

use crate::schema::{
    MIGRATION_001, MIGRATION_002, MIGRATION_003, MIGRATION_004, MIGRATION_005, MIGRATION_006,
    MIGRATION_007, MIGRATION_008, MIGRATION_009, MIGRATION_010, MIGRATION_011, MIGRATION_012,
    MIGRATION_013, MIGRATION_014, MIGRATION_015,
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

    if !(0..=15).contains(&version) {
        return Err(PytxoError::Store(format!(
            "unsupported store schema version {version}"
        )));
    }

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
        version = 7;
    }
    // A falsely advanced v7 must not acquire a v8 stamp before old schema checks.
    verify_previous_schema(conn)?;
    if version < 8 {
        apply_migration_008(conn, MIGRATION_008)?;
        version = 8;
    }
    // A falsely advanced v8 must not acquire a v9 stamp before routing checks.
    verify_routing_schema(conn)?;
    if version < 9 {
        apply_migration_009(conn, MIGRATION_009)?;
        version = 9;
    }
    if version < 10 {
        apply_migration_010(conn, MIGRATION_010)?;
        version = 10;
    }
    verify_v10_schema(conn)?;
    if version < 11 {
        apply_migration_011(conn, MIGRATION_011)?;
    }
    verify_v11_schema(conn)?;
    if version < 12 {
        apply_migration_012(conn, MIGRATION_012)?;
    }
    verify_v12_schema(conn)?;
    if version < 13 {
        apply_migration_013(conn, MIGRATION_013)?;
    }
    // Steps 14 and 15 check their source schema under the step's lock: another
    // connection may have moved the store past it since `version` was read.
    if version < 14 {
        apply_migration_014(conn, MIGRATION_014)?;
        version = 14;
    }
    if version < 15 {
        apply_migration_015(conn, MIGRATION_015)?;
    }
    verify_latest_schema(conn)
}

/// Opens one migration step's write transaction. Two connections can open a new
/// store at once and both read an old `user_version` before either migrates; the
/// immediate lock serializes them, and the step is skipped when the version read
/// under that lock shows another connection already applied it.
fn begin_step(conn: &Connection, target_version: i32) -> Result<Option<rusqlite::Transaction<'_>>> {
    let tx = rusqlite::Transaction::new_unchecked(conn, rusqlite::TransactionBehavior::Immediate)
        .map_err(store_error)?;
    let current: i32 = tx
        .pragma_query_value(None, "user_version", |row| row.get(0))
        .map_err(store_error)?;
    if current >= target_version {
        tx.commit().map_err(store_error)?;
        return Ok(None);
    }
    Ok(Some(tx))
}

fn apply_migration_008(conn: &Connection, sql: &str) -> Result<()> {
    let Some(tx) = begin_step(conn, 8)? else {
        return Ok(());
    };
    tx.execute_batch(sql).map_err(store_error)?;
    verify_previous_schema(&tx)?;
    verify_routing_schema(&tx)?;
    tx.pragma_update(None, "user_version", 8)
        .map_err(store_error)?;
    tx.commit().map_err(store_error)
}

fn apply_migration_009(conn: &Connection, sql: &str) -> Result<()> {
    let Some(tx) = begin_step(conn, 9)? else {
        return Ok(());
    };
    tx.execute_batch(sql).map_err(store_error)?;
    verify_v9_schema(&tx)?;
    tx.pragma_update(None, "user_version", 9)
        .map_err(store_error)?;
    tx.commit().map_err(store_error)
}

fn apply_migration_010(conn: &Connection, sql: &str) -> Result<()> {
    let Some(tx) = begin_step(conn, 10)? else {
        return Ok(());
    };
    tx.execute_batch(sql).map_err(store_error)?;
    verify_v10_schema(&tx)?;
    tx.pragma_update(None, "user_version", 10)
        .map_err(store_error)?;
    tx.commit().map_err(store_error)
}

fn apply_migration_011(conn: &Connection, sql: &str) -> Result<()> {
    let Some(tx) = begin_step(conn, 11)? else {
        return Ok(());
    };
    tx.execute_batch(sql).map_err(store_error)?;
    verify_v11_schema(&tx)?;
    tx.pragma_update(None, "user_version", 11)
        .map_err(store_error)?;
    tx.commit().map_err(store_error)
}

fn apply_migration_012(conn: &Connection, sql: &str) -> Result<()> {
    let Some(tx) = begin_step(conn, 12)? else {
        return Ok(());
    };
    tx.execute_batch(sql).map_err(store_error)?;
    verify_v12_schema(&tx)?;
    tx.pragma_update(None, "user_version", 12)
        .map_err(store_error)?;
    tx.commit().map_err(store_error)
}

fn apply_migration_013(conn: &Connection, sql: &str) -> Result<()> {
    let Some(tx) = begin_step(conn, 13)? else {
        return Ok(());
    };
    tx.execute_batch(sql).map_err(store_error)?;
    verify_v13_schema(&tx)?;
    tx.pragma_update(None, "user_version", 13)
        .map_err(store_error)?;
    tx.commit().map_err(store_error)
}

fn apply_migration_014(conn: &Connection, sql: &str) -> Result<()> {
    let Some(tx) = begin_step(conn, 14)? else {
        return Ok(());
    };
    verify_v13_schema(&tx)?;
    tx.execute_batch(sql).map_err(store_error)?;
    verify_v14_schema(&tx)?;
    tx.pragma_update(None, "user_version", 14)
        .map_err(store_error)?;
    tx.commit().map_err(store_error)
}

fn apply_migration_015(conn: &Connection, sql: &str) -> Result<()> {
    let Some(tx) = begin_step(conn, 15)? else {
        return Ok(());
    };
    verify_v14_schema(&tx)?;
    tx.execute_batch(sql).map_err(store_error)?;
    verify_latest_schema(&tx)?;
    tx.pragma_update(None, "user_version", 15)
        .map_err(store_error)?;
    tx.commit().map_err(store_error)
}

/// Compare the new tables/indexes with the canonical SQLite-parsed schema,
/// including column types, keys, uniqueness, FK and CHECK constraints. Checking
/// column names alone would accept a malformed pre-stamped version 8 database.
fn verify_routing_schema(conn: &Connection) -> Result<()> {
    let expected = Connection::open_in_memory().map_err(store_error)?;
    expected.execute_batch(MIGRATION_008).map_err(store_error)?;
    let mut stmt = expected.prepare("SELECT type,name,sql FROM sqlite_master WHERE name LIKE 'routing_%' OR name='idx_routing_events_scope'").map_err(store_error)?;
    let rows = stmt
        .query_map([], |r| {
            Ok((
                r.get::<_, String>(0)?,
                r.get::<_, String>(1)?,
                r.get::<_, String>(2)?,
            ))
        })
        .map_err(store_error)?;
    for row in rows {
        let (kind, name, sql) = row.map_err(store_error)?;
        let actual: String = conn
            .query_row(
                "SELECT sql FROM sqlite_master WHERE type=?1 AND name=?2",
                rusqlite::params![kind, name],
                |r| r.get(0),
            )
            .map_err(store_error)?;
        if sql != actual {
            return Err(PytxoError::Store(format!(
                "malformed routing schema: {name}"
            )));
        }
    }
    Ok(())
}

/// Compare the new private table with the canonical SQLite-parsed definition,
/// including its primary key and NOT NULL constraints.
fn verify_staging_schema(conn: &Connection) -> Result<()> {
    let expected = Connection::open_in_memory().map_err(store_error)?;
    expected.execute_batch(MIGRATION_009).map_err(store_error)?;
    let expected_sql: String = expected
        .query_row(
            "SELECT sql FROM sqlite_master WHERE type='table' AND name='routing_mission_stages'",
            [],
            |r| r.get(0),
        )
        .map_err(store_error)?;
    let actual_sql: String = conn
        .query_row(
            "SELECT sql FROM sqlite_master WHERE type='table' AND name='routing_mission_stages'",
            [],
            |r| r.get(0),
        )
        .map_err(store_error)?;
    if expected_sql != actual_sql {
        return Err(PytxoError::Store(
            "malformed private routing stage schema".into(),
        ));
    }
    Ok(())
}

fn verify_capacity_intent_schema(conn: &Connection) -> Result<()> {
    let expected = Connection::open_in_memory().map_err(store_error)?;
    expected.execute_batch(MIGRATION_010).map_err(store_error)?;
    for (kind, name) in [
        ("table", "routing_capacity_intents"),
        ("index", "idx_routing_capacity_intents_phase"),
        ("index", "idx_routing_capacity_intents_unresolved_task"),
    ] {
        let expected_sql: String = expected
            .query_row(
                "SELECT sql FROM sqlite_master WHERE type=?1 AND name=?2",
                rusqlite::params![kind, name],
                |row| row.get(0),
            )
            .map_err(store_error)?;
        let actual_sql: Option<String> = conn
            .query_row(
                "SELECT sql FROM sqlite_master WHERE type=?1 AND name=?2",
                rusqlite::params![kind, name],
                |row| row.get(0),
            )
            .optional()
            .map_err(store_error)?;
        if actual_sql.as_deref() != Some(expected_sql.as_str()) {
            return Err(PytxoError::Store(format!(
                "malformed private capacity intent {kind} {name}"
            )));
        }
    }
    Ok(())
}

fn verify_private_artifact_schema(conn: &Connection) -> Result<()> {
    let expected = Connection::open_in_memory().map_err(store_error)?;
    expected.execute_batch(MIGRATION_011).map_err(store_error)?;
    for (kind, name) in [
        ("table", "routing_private_artifacts"),
        ("index", "idx_routing_private_input_singleton"),
        ("index", "idx_routing_private_handoff_singleton"),
        ("index", "idx_routing_private_run"),
        ("table", "attempt_launch_ownership"),
        ("index", "idx_attempt_launch_job_name"),
        ("index", "idx_attempt_launch_unsettled"),
    ] {
        let expected_sql: String = expected
            .query_row(
                "SELECT sql FROM sqlite_master WHERE type=?1 AND name=?2",
                rusqlite::params![kind, name],
                |row| row.get(0),
            )
            .map_err(store_error)?;
        let actual_sql: Option<String> = conn
            .query_row(
                "SELECT sql FROM sqlite_master WHERE type=?1 AND name=?2",
                rusqlite::params![kind, name],
                |row| row.get(0),
            )
            .optional()
            .map_err(store_error)?;
        if actual_sql.as_deref() != Some(expected_sql.as_str()) {
            return Err(PytxoError::Store(format!(
                "malformed private routed schema {kind} {name}"
            )));
        }
    }
    Ok(())
}

fn apply_batch_migration(
    conn: &Connection,
    target_version: i32,
    sql: &str,
    required_schema: &[(&str, &[&str])],
) -> Result<()> {
    let Some(transaction) = begin_step(conn, target_version)? else {
        return Ok(());
    };
    transaction.execute_batch(sql).map_err(store_error)?;
    verify_schema(&transaction, required_schema)?;
    transaction
        .pragma_update(None, "user_version", target_version)
        .map_err(store_error)?;
    transaction.commit().map_err(store_error)
}

fn apply_column_migration(conn: &Connection, target_version: i32, sql: &str) -> Result<()> {
    let columns = parse_add_columns(target_version, sql)?;
    let Some(transaction) = begin_step(conn, target_version)? else {
        return Ok(());
    };
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
    verify_v12_schema(conn)?;
    verify_advisor_schema(conn, true, true)
}

fn verify_v14_schema(conn: &Connection) -> Result<()> {
    verify_v12_schema(conn)?;
    verify_advisor_schema(conn, true, false)
}

fn verify_v13_schema(conn: &Connection) -> Result<()> {
    verify_v12_schema(conn)?;
    verify_advisor_schema(conn, false, false)
}

fn verify_v12_schema(conn: &Connection) -> Result<()> {
    verify_v11_schema(conn)?;
    verify_checker_ownership_schema(conn)
}

fn verify_advisor_schema(conn: &Connection, scoped: bool, hosted: bool) -> Result<()> {
    let expected = Connection::open_in_memory().map_err(store_error)?;
    expected.execute_batch(MIGRATION_013).map_err(store_error)?;
    if scoped {
        expected.execute_batch(MIGRATION_014).map_err(store_error)?;
    }
    if hosted {
        expected.execute_batch(MIGRATION_015).map_err(store_error)?;
    }
    let mut objects = vec![
        ("table", "routing_advisor_consent"),
        ("table", "routing_advisor_requests"),
        ("index", "idx_routing_advisor_requests_scope"),
    ];
    if hosted {
        objects.push(("table", "routing_hosted_advisor_consent"));
    }
    for (kind, name) in objects {
        let expected_sql: String = expected
            .query_row(
                "SELECT sql FROM sqlite_master WHERE type=?1 AND name=?2",
                rusqlite::params![kind, name],
                |row| row.get(0),
            )
            .map_err(store_error)?;
        let actual_sql: Option<String> = conn
            .query_row(
                "SELECT sql FROM sqlite_master WHERE type=?1 AND name=?2",
                rusqlite::params![kind, name],
                |row| row.get(0),
            )
            .optional()
            .map_err(store_error)?;
        if actual_sql.as_deref() != Some(expected_sql.as_str()) {
            return Err(PytxoError::Store(format!(
                "malformed routing advisor schema {kind} {name}"
            )));
        }
    }
    Ok(())
}

fn verify_v11_schema(conn: &Connection) -> Result<()> {
    verify_v10_schema(conn)?;
    verify_private_artifact_schema(conn)
}

fn verify_checker_ownership_schema(conn: &Connection) -> Result<()> {
    let expected = Connection::open_in_memory().map_err(store_error)?;
    expected.execute_batch(MIGRATION_012).map_err(store_error)?;
    for (kind, name) in [
        ("table", "attempt_checker_ownership"),
        ("index", "idx_attempt_checker_job_name"),
        ("index", "idx_attempt_checker_unsettled"),
    ] {
        let expected_sql: String = expected
            .query_row(
                "SELECT sql FROM sqlite_master WHERE type=?1 AND name=?2",
                rusqlite::params![kind, name],
                |row| row.get(0),
            )
            .map_err(store_error)?;
        let actual_sql: Option<String> = conn
            .query_row(
                "SELECT sql FROM sqlite_master WHERE type=?1 AND name=?2",
                rusqlite::params![kind, name],
                |row| row.get(0),
            )
            .optional()
            .map_err(store_error)?;
        if actual_sql.as_deref() != Some(expected_sql.as_str()) {
            return Err(PytxoError::Store(format!(
                "malformed private checker schema {kind} {name}"
            )));
        }
    }
    Ok(())
}

fn verify_v10_schema(conn: &Connection) -> Result<()> {
    verify_v9_schema(conn)?;
    verify_capacity_intent_schema(conn)
}

fn verify_v9_schema(conn: &Connection) -> Result<()> {
    verify_previous_schema(conn)?;
    verify_routing_schema(conn)?;
    verify_staging_schema(conn)
}

fn verify_previous_schema(conn: &Connection) -> Result<()> {
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
        if version >= 7 {
            conn.execute_batch(crate::schema::MIGRATION_007).unwrap();
        }
        if version >= 8 {
            conn.execute_batch(crate::schema::MIGRATION_008).unwrap();
        }
        if version >= 9 {
            conn.execute_batch(crate::schema::MIGRATION_009).unwrap();
        }
        if version >= 10 {
            conn.execute_batch(crate::schema::MIGRATION_010).unwrap();
        }
        if version >= 11 {
            conn.execute_batch(crate::schema::MIGRATION_011).unwrap();
        }
        set_version(&conn, version);
        conn
    }

    fn v14_schema_fixture() -> Connection {
        let conn = released_schema_fixture(11);
        apply_migration_012(&conn, MIGRATION_012).unwrap();
        apply_migration_013(&conn, MIGRATION_013).unwrap();
        apply_migration_014(&conn, MIGRATION_014).unwrap();
        conn
    }

    #[test]
    fn every_released_schema_upgrades_to_latest() {
        for source_version in 0..=11 {
            let conn = released_schema_fixture(source_version);
            apply_migrations(&conn)
                .unwrap_or_else(|error| panic!("upgrade from v{source_version} failed: {error}"));

            let version: i32 = conn
                .pragma_query_value(None, "user_version", |row| row.get(0))
                .unwrap();
            assert_eq!(version, 15, "source version {source_version}");
            verify_schema(&conn, SCHEMA_001).unwrap();
            verify_schema(&conn, SCHEMA_003).unwrap();
            verify_schema(&conn, SCHEMA_006).unwrap();
            verify_schema(&conn, SCHEMA_007).unwrap();
            verify_routing_schema(&conn).unwrap();
            verify_staging_schema(&conn).unwrap();
            verify_capacity_intent_schema(&conn).unwrap();
            verify_private_artifact_schema(&conn).unwrap();
            verify_checker_ownership_schema(&conn).unwrap();
            verify_advisor_schema(&conn, true, true).unwrap();
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
        assert_eq!(version, 15);
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
        assert_eq!(version, 15);
    }

    #[test]
    fn routing_migration_preserves_legacy_data_and_rolls_back_on_failure() {
        let conn = released_schema_fixture(7);
        conn.execute("INSERT INTO runs(id,started_at,status,repo_root) VALUES ('legacy','old','failed','repo')",[]).unwrap();
        conn.execute("INSERT INTO run_contracts(run_id,plan_json,enforcement_json,prepared_digest,recovery_state) VALUES ('legacy','{}','{}','keep-exactly','recovery_required')",[]).unwrap();
        let broken = format!("{MIGRATION_008} INSERT INTO nonexistent VALUES (1);");
        assert!(apply_migration_008(&conn, &broken).is_err());
        assert_eq!(
            conn.pragma_query_value(None, "user_version", |r| r.get::<_, i32>(0))
                .unwrap(),
            7
        );
        assert!(!table_exists(&conn, "routing_missions").unwrap());
        apply_migrations(&conn).unwrap();
        assert_eq!(
            conn.query_row(
                "SELECT prepared_digest FROM run_contracts WHERE run_id='legacy'",
                [],
                |r| r.get::<_, String>(0)
            )
            .unwrap(),
            "keep-exactly"
        );
        assert_eq!(
            conn.query_row("SELECT COUNT(*) FROM routing_missions", [], |r| r
                .get::<_, u64>(0))
                .unwrap(),
            0
        );
    }

    #[test]
    fn advanced_routing_schema_rejects_missing_constraints_and_future_version() {
        let conn = released_schema_fixture(7);
        conn.execute_batch(
            &MIGRATION_008.replace("agent_id TEXT NOT NULL UNIQUE", "agent_id TEXT NOT NULL"),
        )
        .unwrap();
        set_version(&conn, 8);
        assert!(apply_migrations(&conn).is_err());
        set_version(&conn, 14);
        assert!(apply_migrations(&conn).is_err());
        assert_eq!(
            conn.pragma_query_value(None, "user_version", |r| r.get::<_, i32>(0))
                .unwrap(),
            14
        );
    }

    #[test]
    fn routing_schema_rejects_token_boundary_type_corruption() {
        let conn = released_schema_fixture(7);
        conn.execute_batch(&MIGRATION_008.replace(
            "cancel_epoch INTEGER NOT NULL",
            "cancel_epoch INTEGERNOT NULL",
        ))
        .unwrap();
        set_version(&conn, 8);
        let not_null: i64 = conn
            .query_row(
                "SELECT \"notnull\" FROM pragma_table_info('routing_missions')
                 WHERE name='cancel_epoch'",
                [],
                |row| row.get(0),
            )
            .unwrap();
        assert_eq!(not_null, 0, "fixture must weaken the NOT NULL constraint");
        assert!(apply_migrations(&conn).is_err());
        assert_eq!(
            conn.pragma_query_value(None, "user_version", |row| row.get::<_, i32>(0))
                .unwrap(),
            8
        );
    }

    #[test]
    fn falsely_advanced_eight_is_not_repaired_into_apparent_success() {
        let conn = released_schema_fixture(7);
        set_version(&conn, 8);
        assert!(apply_migrations(&conn).is_err());
        assert!(!table_exists(&conn, "routing_missions").unwrap());
    }

    #[test]
    fn staging_migration_rolls_back_atomically_and_preserves_prior_data() {
        let conn = released_schema_fixture(8);
        conn.execute("INSERT INTO runs(id,started_at,status,repo_root) VALUES ('legacy','old','failed','repo')",[]).unwrap();
        let broken = format!("{MIGRATION_009} INSERT INTO nonexistent VALUES (1);");
        assert!(apply_migration_009(&conn, &broken).is_err());
        assert_eq!(
            conn.pragma_query_value(None, "user_version", |r| r.get::<_, i32>(0))
                .unwrap(),
            8
        );
        assert!(!table_exists(&conn, "routing_mission_stages").unwrap());

        apply_migrations(&conn).unwrap();
        assert_eq!(
            conn.pragma_query_value(None, "user_version", |r| r.get::<_, i32>(0))
                .unwrap(),
            15
        );
        assert_eq!(
            conn.query_row("SELECT status FROM runs WHERE id='legacy'", [], |r| r
                .get::<_, String>(0))
                .unwrap(),
            "failed"
        );
    }

    #[test]
    fn falsely_advanced_nine_and_weakened_stage_schema_fail_closed() {
        let conn = released_schema_fixture(8);
        set_version(&conn, 9);
        assert!(apply_migrations(&conn).is_err());

        let conn = released_schema_fixture(8);
        conn.execute_batch(&MIGRATION_009.replace("run_id TEXT PRIMARY KEY", "run_id TEXT"))
            .unwrap();
        set_version(&conn, 9);
        assert!(apply_migrations(&conn).is_err());
        assert_eq!(
            conn.pragma_query_value(None, "user_version", |r| r.get::<_, i32>(0))
                .unwrap(),
            9
        );
    }

    #[test]
    fn capacity_intent_migration_is_atomic_and_rejects_false_version_ten() {
        let conn = released_schema_fixture(9);
        let broken = format!("{MIGRATION_010} INSERT INTO nonexistent VALUES (1);");
        assert!(apply_migration_010(&conn, &broken).is_err());
        assert!(!table_exists(&conn, "routing_capacity_intents").unwrap());
        assert_eq!(
            conn.pragma_query_value(None, "user_version", |row| row.get::<_, i32>(0))
                .unwrap(),
            9
        );
        apply_migrations(&conn).unwrap();
        verify_capacity_intent_schema(&conn).unwrap();

        let conn = released_schema_fixture(9);
        set_version(&conn, 10);
        assert!(apply_migrations(&conn).is_err());
        assert!(!table_exists(&conn, "routing_capacity_intents").unwrap());
    }

    #[test]
    fn private_artifact_migration_rolls_back_and_rejects_false_version_eleven() {
        let conn = released_schema_fixture(10);
        let broken = format!("{MIGRATION_011} INSERT INTO nonexistent VALUES (1);");
        assert!(apply_migration_011(&conn, &broken).is_err());
        assert!(!table_exists(&conn, "routing_private_artifacts").unwrap());
        assert!(!table_exists(&conn, "attempt_launch_ownership").unwrap());
        assert_eq!(
            conn.pragma_query_value(None, "user_version", |r| r.get::<_, i32>(0))
                .unwrap(),
            10
        );
        apply_migrations(&conn).unwrap();
        verify_private_artifact_schema(&conn).unwrap();

        let conn = released_schema_fixture(10);
        set_version(&conn, 11);
        assert!(apply_migrations(&conn).is_err());
    }

    #[test]
    fn checker_ownership_migration_rolls_back_and_rejects_false_version_twelve() {
        let conn = released_schema_fixture(11);
        let broken = format!("{MIGRATION_012} INSERT INTO nonexistent VALUES (1);");
        assert!(apply_migration_012(&conn, &broken).is_err());
        assert!(!table_exists(&conn, "attempt_checker_ownership").unwrap());
        assert_eq!(
            conn.pragma_query_value(None, "user_version", |row| row.get::<_, i32>(0))
                .unwrap(),
            11
        );
        apply_migrations(&conn).unwrap();
        verify_checker_ownership_schema(&conn).unwrap();

        let false_stamp = released_schema_fixture(11);
        set_version(&false_stamp, 12);
        assert!(apply_migrations(&false_stamp).is_err());
        assert!(!table_exists(&false_stamp, "attempt_checker_ownership").unwrap());
    }

    #[test]
    fn advisor_migration_rolls_back_and_rejects_false_version_thirteen() {
        let conn = released_schema_fixture(11);
        apply_migration_012(&conn, MIGRATION_012).unwrap();
        let broken = format!("{MIGRATION_013} INSERT INTO nonexistent VALUES (1);");
        assert!(apply_migration_013(&conn, &broken).is_err());
        assert!(!table_exists(&conn, "routing_advisor_consent").unwrap());
        assert_eq!(
            conn.pragma_query_value(None, "user_version", |row| row.get::<_, i32>(0))
                .unwrap(),
            12
        );
        apply_migrations(&conn).unwrap();
        verify_advisor_schema(&conn, true, true).unwrap();

        let false_stamp = released_schema_fixture(11);
        apply_migration_012(&false_stamp, MIGRATION_012).unwrap();
        set_version(&false_stamp, 13);
        assert!(apply_migrations(&false_stamp).is_err());
        assert!(!table_exists(&false_stamp, "routing_advisor_requests").unwrap());
    }

    #[test]
    fn disclosure_scope_migration_disables_unscoped_grants() {
        let conn = released_schema_fixture(11);
        apply_migration_012(&conn, MIGRATION_012).unwrap();
        apply_migration_013(&conn, MIGRATION_013).unwrap();
        conn.execute(
            "INSERT INTO routing_advisor_consent(domain_id,revision,enabled,updated_at_ms) VALUES ('old-domain',1,1,145)",
            [],
        )
        .unwrap();
        apply_migrations(&conn).unwrap();
        let row: (u64, bool, Option<String>) = conn
            .query_row(
                "SELECT revision,enabled,scope_digest FROM routing_advisor_consent WHERE domain_id='old-domain'",
                [],
                |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?)),
            )
            .unwrap();
        assert_eq!(row, (1, false, None));
        assert_eq!(
            conn.pragma_query_value(None, "user_version", |row| row.get::<_, i32>(0))
                .unwrap(),
            15
        );
    }

    #[test]
    fn disclosure_scope_migration_rolls_back_a_failed_upgrade() {
        let conn = released_schema_fixture(11);
        apply_migration_012(&conn, MIGRATION_012).unwrap();
        apply_migration_013(&conn, MIGRATION_013).unwrap();
        conn.execute(
            "INSERT INTO routing_advisor_consent(domain_id,revision,enabled,updated_at_ms) VALUES ('old-domain',1,1,145)",
            [],
        )
        .unwrap();
        let broken = format!("{MIGRATION_014} INSERT INTO nonexistent VALUES (1);");
        assert!(apply_migration_014(&conn, &broken).is_err());
        assert!(!column_exists(&conn, "routing_advisor_consent", "scope_digest").unwrap());
        let still_enabled: bool = conn
            .query_row(
                "SELECT enabled FROM routing_advisor_consent WHERE domain_id='old-domain'",
                [],
                |row| row.get(0),
            )
            .unwrap();
        assert!(still_enabled);
        assert_eq!(
            conn.pragma_query_value(None, "user_version", |row| row.get::<_, i32>(0))
                .unwrap(),
            13
        );
        apply_migrations(&conn).unwrap();
        verify_latest_schema(&conn).unwrap();
    }

    #[test]
    fn hosted_journal_migration_preserves_v14_review_and_send_history() {
        let conn = v14_schema_fixture();
        let mission_json =
            include_str!("../tests/fixtures/routing_pre_check_recipes_v8_registration.json");
        let task_json =
            include_str!("../tests/fixtures/routing_pre_check_recipes_v8_task_record.json");
        let mission_digest =
            include_str!("../tests/fixtures/routing_pre_check_recipes_v8_mission_digest.txt")
                .trim();
        let mission: crate::routing::RoutingMission = serde_json::from_str(mission_json).unwrap();
        assert_eq!(serde_json::to_string(&mission).unwrap(), mission_json);
        assert_eq!(
            pytxo_core::routing::canonical_digest(&mission, 1)
                .unwrap()
                .0,
            mission_digest
        );
        let policy_digest = mission.policy.digest().unwrap();
        let packet_digest = pytxo_core::routing::Digest::of_bytes(b"v14-packet");
        let result_digest = pytxo_core::routing::Digest::of_bytes(b"v14-response");
        conn.execute("INSERT INTO runs(id,started_at,status,repo_root) VALUES ('run','old','running','repo')", []).unwrap();
        conn.execute("INSERT INTO routing_missions(run_id,domain_id,registration_json,registration_digest,cancel_epoch,cancelled,revision) VALUES ('run','domain',?1,?2,0,0,1)", rusqlite::params![mission_json,mission_digest]).unwrap();
        conn.execute("INSERT INTO routing_tasks(run_id,task_id,revision,record_json) VALUES ('run','task0',1,?1)", [task_json]).unwrap();
        conn.execute("INSERT INTO routing_mission_stages(run_id,domain_id,draft_id,plan_digest,mission_digest,mission_json) VALUES ('run','domain','draft',?1,?2,?3)", rusqlite::params![mission.authorization.plan_digest.0,mission_digest,mission_json]).unwrap();
        for (ordinal, phase) in [
            "prepared",
            "sending_may_have_happened",
            "completed",
            "uncertain",
            "late_receipt",
        ]
        .iter()
        .enumerate()
        {
            let result =
                matches!(*phase, "completed" | "late_receipt").then_some(result_digest.0.as_str());
            conn.execute("INSERT INTO routing_advisor_requests(request_id,domain_id,run_id,task_id,ordinal,policy_digest,packet_digest,consent_revision,task_revision,task_state_revision,authorization_revision,cancel_epoch,phase,result_digest,created_at_ms,updated_at_ms) VALUES (?1,'domain','run','task0',?2,?3,?4,1,1,1,1,0,?5,?6,100,120)", rusqlite::params![format!("old-{ordinal}"),ordinal+1,policy_digest.0,packet_digest.0,phase,result]).unwrap();
        }
        apply_migrations(&conn).unwrap();
        assert_eq!(
            conn.pragma_query_value(None, "user_version", |row| row.get::<_, i32>(0))
                .unwrap(),
            15
        );
        let stored: (String,String) = conn.query_row("SELECT registration_json,registration_digest FROM routing_missions WHERE run_id='run'", [], |row| Ok((row.get(0)?,row.get(1)?))).unwrap();
        assert_eq!(stored, (mission_json.into(), mission_digest.into()));
        let staged: String = conn
            .query_row(
                "SELECT mission_json FROM routing_mission_stages WHERE run_id='run'",
                [],
                |row| row.get(0),
            )
            .unwrap();
        assert_eq!(staged, mission_json);
        let db = crate::PytxoStore { conn };
        let scope = crate::routing::RoutingScope {
            domain_id: pytxo_core::DomainId("domain".into()),
            run_id: pytxo_core::RunId("run".into()),
        };
        let staged = crate::routing::StagedRoutingMissionRef {
            domain_id: scope.domain_id.clone(),
            run_id: scope.run_id.clone(),
            draft_id: "draft".into(),
            plan_digest: mission.authorization.plan_digest.clone(),
            mission_digest: pytxo_core::routing::Digest(mission_digest.into()),
        };
        assert_eq!(db.load_staged_routing_mission(&staged).unwrap(), mission);
        assert_eq!(
            db.routing_history(&scope).unwrap().unwrap().mission,
            mission
        );
        for ordinal in 0..5 {
            let row = db
                .routing_advisor_request(
                    &scope,
                    &pytxo_core::routing::AdviceRequestId(format!("old-{ordinal}")),
                )
                .unwrap()
                .unwrap();
            assert_eq!(row.recipient_identity, None);
            assert_eq!(row.scope_digest, None);
        }
        let sent = pytxo_core::routing::AdviceRequestId("old-1".into());
        assert_eq!(
            db.settle_routing_advisor_request(&scope, &sent, None, 121)
                .unwrap()
                .phase,
            crate::routing::AdvisorSendPhase::Uncertain
        );
        assert_eq!(
            db.routing_hosted_advisor_consent(
                &scope.domain_id,
                "pytxo-hosted-routing/typesafe-systemone/v1"
            )
            .unwrap()
            .revision,
            0
        );
    }

    #[test]
    fn hosted_journal_migration_rolls_back_and_rejects_false_v15_stamp() {
        let conn = v14_schema_fixture();
        let broken = format!("{MIGRATION_015} INSERT INTO nonexistent VALUES (1);");
        assert!(apply_migration_015(&conn, &broken).is_err());
        assert!(!table_exists(&conn, "routing_hosted_advisor_consent").unwrap());
        assert!(!column_exists(&conn, "routing_advisor_requests", "recipient_identity").unwrap());
        assert_eq!(
            conn.pragma_query_value(None, "user_version", |row| row.get::<_, i32>(0))
                .unwrap(),
            14
        );
        set_version(&conn, 15);
        assert!(apply_migrations(&conn).is_err());
        set_version(&conn, 14);
        apply_migrations(&conn).unwrap();
        verify_latest_schema(&conn).unwrap();
    }
}
