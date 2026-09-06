use std::path::Path;

use chrono::{DateTime, Utc};
use pytxo_core::{PreparedRunManifest, PytxoError, Result, RunApplyError};
use rusqlite::{params, Connection, OptionalExtension, Transaction};
use serde::Serialize;

use crate::migrate::apply_migrations;

#[derive(Clone, Debug, Serialize)]
pub struct RunRecord {
    pub id: String,
    pub started_at: DateTime<Utc>,
    pub finished_at: Option<DateTime<Utc>>,
    pub status: String,
    pub repo_root: String,
    pub estimated_tokens_in: Option<i64>,
    pub estimated_tokens_out: Option<i64>,
    pub estimated_cost_usd: Option<f64>,
    pub permission_profile: Option<String>,
}

#[derive(Clone, Debug, Serialize)]
pub struct RunContractRecord {
    pub run_id: String,
    pub base_revision: Option<String>,
    pub plan_json: Option<String>,
    pub apply_status: String,
    pub apply_manifest_json: Option<String>,
    pub applied_at: Option<String>,
    pub enforcement_json: Option<String>,
    pub prepared_manifest: Option<PreparedRunManifest>,
    pub prepared_digest: Option<String>,
    pub prepared_at: Option<String>,
    pub last_apply_error: Option<RunApplyError>,
    pub recovery_state: Option<String>,
}

#[derive(Clone, Debug, Serialize)]
pub struct DomainChange {
    pub sequence: i64,
    pub entity_kind: String,
    pub entity_id: String,
    pub changed_at: String,
}

#[derive(Clone, Debug, Serialize)]
pub struct DomainChangesPage {
    pub changes: Vec<DomainChange>,
    pub next_cursor: i64,
    pub has_more: bool,
    pub cursor_gap: bool,
}

#[derive(Clone, Debug, Serialize)]
pub struct AgentRecord {
    pub id: String,
    pub run_id: String,
    pub task_id: String,
    pub wave: i32,
    pub worktree_path: Option<String>,
    pub cmd: String,
    pub exit_code: Option<i32>,
    pub status: String,
    /// Modular project root label ([[ADR-0011-modular-project-manifest]]).
    #[serde(default)]
    pub root_id: Option<String>,
}

#[derive(Clone, Debug, Serialize)]
pub struct EventRecord {
    pub id: i64,
    pub agent_id: String,
    pub ts: DateTime<Utc>,
    pub kind: String,
    pub payload: String,
}

/// Aggregated run health for a single execution domain ([[execution-domains]] dashboard).
#[derive(Clone, Debug, Default, Serialize)]
pub struct DomainRunSummary {
    pub active_runs: usize,
    pub latest_run_id: Option<String>,
    pub latest_run_status: Option<String>,
    pub latest_started_at: Option<String>,
}

pub struct PytxoStore {
    pub(crate) conn: Connection,
}

impl PytxoStore {
    pub fn open(path: &Path) -> Result<Self> {
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent).map_err(PytxoError::Io)?;
        }
        let conn = Connection::open(path).map_err(|e| PytxoError::Store(e.to_string()))?;
        conn.execute_batch("PRAGMA journal_mode=WAL;")
            .map_err(|e| PytxoError::Store(e.to_string()))?;
        apply_migrations(&conn)?;
        Ok(Self { conn })
    }

    pub fn insert_run(&self, id: &str, repo_root: &str) -> Result<()> {
        self.insert_run_with_profile(id, repo_root, None)
    }

    pub fn insert_run_with_profile(
        &self,
        id: &str,
        repo_root: &str,
        permission_profile: Option<&str>,
    ) -> Result<()> {
        self.insert_run_with_profile_and_status(id, repo_root, permission_profile, "running")
    }

    pub fn insert_starting_run_with_profile(
        &self,
        id: &str,
        repo_root: &str,
        permission_profile: Option<&str>,
    ) -> Result<()> {
        self.insert_run_with_profile_and_status(id, repo_root, permission_profile, "starting")
    }

    fn insert_run_with_profile_and_status(
        &self,
        id: &str,
        repo_root: &str,
        permission_profile: Option<&str>,
        status: &str,
    ) -> Result<()> {
        let now = Utc::now().to_rfc3339();
        let tx = self.conn.unchecked_transaction().map_err(store_error)?;
        tx
            .execute(
                "INSERT INTO runs (id, started_at, status, repo_root, permission_profile) VALUES (?1, ?2, ?3, ?4, ?5)",
                params![id, now, status, repo_root, permission_profile],
            )
            .map_err(store_error)?;
        append_domain_change(&tx, "run", id)?;
        tx.commit().map_err(store_error)?;
        Ok(())
    }

    /// Tag a run with its owning project and (optional) primary root label
    /// ([[ADR-0011-modular-project-manifest]]).
    pub fn tag_run_project(
        &self,
        run_id: &str,
        project_id: &str,
        root_id: Option<&str>,
    ) -> Result<()> {
        let tx = self.conn.unchecked_transaction().map_err(store_error)?;
        tx.execute(
            "UPDATE runs SET project_id = ?1, root_id = ?2 WHERE id = ?3",
            params![project_id, root_id, run_id],
        )
        .map_err(store_error)?;
        append_domain_change(&tx, "run", run_id)?;
        tx.commit().map_err(store_error)?;
        Ok(())
    }

    /// Project id a run belongs to, if it was dispatched under a project.
    pub fn run_project(&self, run_id: &str) -> Result<Option<String>> {
        let mut stmt = self
            .conn
            .prepare("SELECT project_id FROM runs WHERE id = ?1")
            .map_err(|e| PytxoError::Store(e.to_string()))?;
        let mut rows = stmt
            .query_map(params![run_id], |row| row.get::<_, Option<String>>(0))
            .map_err(|e| PytxoError::Store(e.to_string()))?;
        match rows.next() {
            Some(r) => r.map_err(|e| PytxoError::Store(e.to_string())),
            None => Ok(None),
        }
    }

    pub fn finish_run(&self, id: &str, status: &str) -> Result<()> {
        let now = Utc::now().to_rfc3339();
        let tx = self.conn.unchecked_transaction().map_err(store_error)?;
        tx.execute(
            "UPDATE runs SET finished_at = ?1, status = ?2 WHERE id = ?3",
            params![now, status, id],
        )
        .map_err(store_error)?;
        append_domain_change(&tx, "run", id)?;
        tx.commit().map_err(store_error)?;
        Ok(())
    }

    /// Finalize a worker-owned run only while it is still active.
    ///
    /// Explicit cancellation deliberately uses `finish_run` so an operator can
    /// transition a running run to `cancelled`; this guarded transition keeps a
    /// worker that unwinds afterward from overwriting that terminal state.
    pub fn finish_run_if_running(&self, id: &str, status: &str) -> Result<bool> {
        self.finish_run_if_status(id, "running", status)
    }

    pub fn finish_run_if_status(
        &self,
        id: &str,
        expected_status: &str,
        status: &str,
    ) -> Result<bool> {
        let now = Utc::now().to_rfc3339();
        let tx = self.conn.unchecked_transaction().map_err(store_error)?;
        let changed = tx
            .execute(
                "UPDATE runs SET finished_at = ?1, status = ?2 WHERE id = ?3 AND status = ?4",
                params![now, status, id, expected_status],
            )
            .map_err(store_error)?;
        if changed > 0 {
            append_domain_change(&tx, "run", id)?;
        }
        tx.commit().map_err(store_error)?;
        Ok(changed > 0)
    }

    pub fn mark_run_running(&self, id: &str) -> Result<bool> {
        let tx = self.conn.unchecked_transaction().map_err(store_error)?;
        let changed = tx
            .execute(
                "UPDATE runs SET status = 'running' WHERE id = ?1 AND status = 'starting'",
                params![id],
            )
            .map_err(store_error)?;
        if changed > 0 {
            append_domain_change(&tx, "run", id)?;
        }
        tx.commit().map_err(store_error)?;
        Ok(changed > 0)
    }

    pub fn save_run_contract(
        &self,
        run_id: &str,
        base_revision: &str,
        plan_json: &str,
        enforcement_json: &str,
    ) -> Result<()> {
        self.save_run_contract_with_status(
            run_id,
            Some(base_revision),
            plan_json,
            enforcement_json,
            "pending",
        )
    }

    pub fn save_run_contract_with_status(
        &self,
        run_id: &str,
        base_revision: Option<&str>,
        plan_json: &str,
        enforcement_json: &str,
        apply_status: &str,
    ) -> Result<()> {
        if !matches!(
            apply_status,
            "pending" | "non_flushable" | "not_applicable" | "unsupported"
        ) {
            return Err(PytxoError::Store(format!(
                "invalid initial run apply status: {apply_status}"
            )));
        }
        let tx = self.conn.unchecked_transaction().map_err(store_error)?;
        tx.execute(
            "INSERT INTO run_contracts (
                    run_id, base_revision, plan_json, apply_status, enforcement_json
                 ) VALUES (?1, ?2, ?3, ?4, ?5)
                 ON CONFLICT(run_id) DO UPDATE SET
                    base_revision = excluded.base_revision,
                    plan_json = excluded.plan_json,
                    apply_status = excluded.apply_status,
                    apply_manifest_json = NULL,
                    applied_at = NULL,
                    enforcement_json = excluded.enforcement_json",
            params![
                run_id,
                base_revision,
                plan_json,
                apply_status,
                enforcement_json
            ],
        )
        .map_err(store_error)?;
        append_domain_change(&tx, "contract", run_id)?;
        tx.commit().map_err(store_error)?;
        Ok(())
    }

    pub fn get_run_contract(&self, run_id: &str) -> Result<Option<RunContractRecord>> {
        self.conn
            .query_row(
                "SELECT run_id, base_revision, plan_json, apply_status,
                        apply_manifest_json, applied_at, enforcement_json,
                        prepared_manifest_json, prepared_digest, prepared_at,
                        last_apply_error_json, recovery_state
                 FROM run_contracts WHERE run_id = ?1",
                params![run_id],
                |row| {
                    let prepared_json: Option<String> = row.get(7)?;
                    let error_json: Option<String> = row.get(10)?;
                    Ok(RunContractRecord {
                        run_id: row.get(0)?,
                        base_revision: row.get(1)?,
                        plan_json: row.get(2)?,
                        apply_status: row.get(3)?,
                        apply_manifest_json: row.get(4)?,
                        applied_at: row.get(5)?,
                        enforcement_json: row.get(6)?,
                        prepared_manifest: prepared_json
                            .and_then(|json| serde_json::from_str(&json).ok()),
                        prepared_digest: row.get(8)?,
                        prepared_at: row.get(9)?,
                        last_apply_error: error_json
                            .and_then(|json| serde_json::from_str(&json).ok()),
                        recovery_state: row.get(11)?,
                    })
                },
            )
            .optional()
            .map_err(|e| PytxoError::Store(e.to_string()))
    }

    pub fn list_run_contract_ids_with_status(&self, status: &str) -> Result<Vec<String>> {
        let mut statement = self
            .conn
            .prepare("SELECT run_id FROM run_contracts WHERE apply_status = ?1 ORDER BY run_id")
            .map_err(store_error)?;
        let rows = statement
            .query_map(params![status], |row| row.get(0))
            .map_err(store_error)?;
        rows.collect::<std::result::Result<Vec<_>, _>>()
            .map_err(store_error)
    }

    pub fn begin_run_preparation(&self, run_id: &str) -> Result<bool> {
        self.transition_contract(
            run_id,
            &["pending", "stale", "review_failed"],
            "preparing",
            None,
        )
    }

    pub fn finish_run_preparation(
        &self,
        run_id: &str,
        manifest: &PreparedRunManifest,
    ) -> Result<()> {
        let manifest_json = serde_json::to_string(manifest)
            .map_err(|error| PytxoError::Store(error.to_string()))?;
        let tx = self.conn.unchecked_transaction().map_err(store_error)?;
        let changed = tx
            .execute(
                "UPDATE run_contracts
                 SET apply_status='ready', prepared_manifest_json=?1,
                     prepared_digest=?2, prepared_at=?3,
                     base_revision=?4, last_apply_error_json=NULL, recovery_state=NULL
                 WHERE run_id=?5 AND apply_status='preparing'",
                params![
                    manifest_json,
                    manifest.package_digest,
                    manifest.prepared_at,
                    manifest.base_revision,
                    run_id
                ],
            )
            .map_err(store_error)?;
        if changed != 1 {
            return Err(PytxoError::Store(format!(
                "run review was not preparing: {run_id}"
            )));
        }
        append_domain_change(&tx, "contract", run_id)?;
        tx.commit().map_err(store_error)?;
        Ok(())
    }

    pub fn fail_run_preparation(&self, run_id: &str, error: &RunApplyError) -> Result<()> {
        self.set_contract_outcome(run_id, "review_failed", Some(error), None, &["preparing"])
    }

    pub fn discard_run_review(&self, run_id: &str) -> Result<bool> {
        self.transition_contract(
            run_id,
            &["ready", "stale", "review_failed"],
            "discarded",
            None,
        )
    }

    pub fn finish_run_apply_error(
        &self,
        run_id: &str,
        status: &str,
        error: &RunApplyError,
        recovery_state: Option<&str>,
    ) -> Result<()> {
        if !matches!(
            status,
            "ready" | "stale" | "review_failed" | "recovery_required"
        ) {
            return Err(PytxoError::Store(format!(
                "invalid failed apply status: {status}"
            )));
        }
        self.set_contract_outcome(run_id, status, Some(error), recovery_state, &["applying"])
    }

    pub fn finish_run_recovery_error(
        &self,
        run_id: &str,
        status: &str,
        error: &RunApplyError,
        recovery_state: Option<&str>,
    ) -> Result<()> {
        if !matches!(status, "ready" | "recovery_required") {
            return Err(PytxoError::Store(format!(
                "invalid recovery status: {status}"
            )));
        }
        self.set_contract_outcome(
            run_id,
            status,
            Some(error),
            recovery_state,
            &["applying", "recovery_required"],
        )
    }

    fn set_contract_outcome(
        &self,
        run_id: &str,
        status: &str,
        error: Option<&RunApplyError>,
        recovery_state: Option<&str>,
        expected: &[&str],
    ) -> Result<()> {
        let error_json = error
            .map(serde_json::to_string)
            .transpose()
            .map_err(|error| PytxoError::Store(error.to_string()))?;
        let tx = self.conn.unchecked_transaction().map_err(store_error)?;
        let placeholders = expected.iter().map(|_| "?").collect::<Vec<_>>().join(",");
        let sql = format!(
            "UPDATE run_contracts SET apply_status=?1, last_apply_error_json=?2, recovery_state=?3
             WHERE run_id=?4 AND apply_status IN ({placeholders})"
        );
        let mut values: Vec<rusqlite::types::Value> = vec![
            status.to_owned().into(),
            error_json.into(),
            recovery_state.map(str::to_owned).into(),
            run_id.to_owned().into(),
        ];
        values.extend(expected.iter().map(|value| (*value).to_owned().into()));
        let changed = tx
            .execute(&sql, rusqlite::params_from_iter(values))
            .map_err(store_error)?;
        if changed != 1 {
            return Err(PytxoError::Store(format!(
                "invalid run contract transition for {run_id}"
            )));
        }
        append_domain_change(&tx, "contract", run_id)?;
        tx.commit().map_err(store_error)?;
        Ok(())
    }

    fn transition_contract(
        &self,
        run_id: &str,
        expected: &[&str],
        status: &str,
        recovery_state: Option<&str>,
    ) -> Result<bool> {
        let tx = self.conn.unchecked_transaction().map_err(store_error)?;
        let placeholders = expected.iter().map(|_| "?").collect::<Vec<_>>().join(",");
        let sql = format!(
            "UPDATE run_contracts SET apply_status=?1, recovery_state=?2
             WHERE run_id=?3 AND apply_status IN ({placeholders})"
        );
        let mut values: Vec<rusqlite::types::Value> = vec![
            status.to_owned().into(),
            recovery_state.map(str::to_owned).into(),
            run_id.to_owned().into(),
        ];
        values.extend(expected.iter().map(|value| (*value).to_owned().into()));
        let changed = tx
            .execute(&sql, rusqlite::params_from_iter(values))
            .map_err(store_error)?;
        if changed == 1 {
            append_domain_change(&tx, "contract", run_id)?;
        }
        tx.commit().map_err(store_error)?;
        Ok(changed == 1)
    }

    pub fn changes_since(&self, cursor: i64, limit: usize) -> Result<DomainChangesPage> {
        let limit = limit.clamp(1, 1_000);
        let (oldest, newest): (Option<i64>, Option<i64>) = self
            .conn
            .query_row(
                "SELECT MIN(sequence), MAX(sequence) FROM domain_changes",
                [],
                |row| Ok((row.get(0)?, row.get(1)?)),
            )
            .map_err(store_error)?;
        let boundary = newest.unwrap_or(0);
        let cursor_gap =
            cursor > boundary || oldest.is_some_and(|oldest| cursor > 0 && cursor < oldest - 1);
        if cursor_gap {
            return Ok(DomainChangesPage {
                changes: Vec::new(),
                next_cursor: boundary,
                has_more: false,
                cursor_gap: true,
            });
        }
        let mut statement = self
            .conn
            .prepare(
                "SELECT sequence, entity_kind, entity_id, changed_at
                 FROM domain_changes WHERE sequence > ?1
                 ORDER BY sequence LIMIT ?2",
            )
            .map_err(store_error)?;
        let mut changes = statement
            .query_map(params![cursor, (limit + 1) as i64], |row| {
                Ok(DomainChange {
                    sequence: row.get(0)?,
                    entity_kind: row.get(1)?,
                    entity_id: row.get(2)?,
                    changed_at: row.get(3)?,
                })
            })
            .map_err(store_error)?
            .collect::<std::result::Result<Vec<_>, _>>()
            .map_err(store_error)?;
        let has_more = changes.len() > limit;
        changes.truncate(limit);
        let next_cursor = changes.last().map_or(cursor, |change| change.sequence);
        Ok(DomainChangesPage {
            changes,
            next_cursor,
            has_more,
            cursor_gap,
        })
    }

    pub fn claim_run_apply(&self, run_id: &str) -> Result<bool> {
        let tx = self.conn.unchecked_transaction().map_err(store_error)?;
        let changed = tx
            .execute(
                "UPDATE run_contracts SET apply_status = 'applying'
                 WHERE run_id = ?1
                   AND apply_status = 'ready'
                   AND prepared_manifest_json IS NOT NULL
                   AND EXISTS (
                       SELECT 1 FROM runs
                       WHERE runs.id = run_contracts.run_id
                         AND runs.status = 'completed'
                   )",
                params![run_id],
            )
            .map_err(store_error)?;
        if changed == 1 {
            append_domain_change(&tx, "contract", run_id)?;
        }
        tx.commit().map_err(store_error)?;
        Ok(changed == 1)
    }

    pub fn finish_run_apply(
        &self,
        run_id: &str,
        apply_status: &str,
        manifest_json: Option<&str>,
    ) -> Result<()> {
        if !matches!(
            apply_status,
            "applied" | "ready" | "stale" | "recovery_required"
        ) {
            return Err(PytxoError::Store(format!(
                "invalid run apply status: {apply_status}"
            )));
        }
        let applied_at = (apply_status == "applied").then(|| Utc::now().to_rfc3339());
        let tx = self.conn.unchecked_transaction().map_err(store_error)?;
        let changed = tx
            .execute(
                "UPDATE run_contracts
                 SET apply_status = ?1, apply_manifest_json = ?2, applied_at = ?3
                 WHERE run_id = ?4 AND apply_status = 'applying'",
                params![apply_status, manifest_json, applied_at, run_id],
            )
            .map_err(store_error)?;
        if changed != 1 {
            return Err(PytxoError::Store(format!(
                "run apply was not claimed: {run_id}"
            )));
        }
        append_domain_change(&tx, "contract", run_id)?;
        tx.commit().map_err(store_error)?;
        Ok(())
    }

    pub fn finish_run_recovery_apply(&self, run_id: &str, manifest_json: &str) -> Result<()> {
        let tx = self.conn.unchecked_transaction().map_err(store_error)?;
        let changed = tx
            .execute(
                "UPDATE run_contracts
                 SET apply_status = 'applied', apply_manifest_json = ?1,
                     applied_at = ?2, last_apply_error_json = NULL,
                     recovery_state = NULL
                 WHERE run_id = ?3
                   AND apply_status IN ('applying', 'recovery_required')",
                params![manifest_json, Utc::now().to_rfc3339(), run_id],
            )
            .map_err(store_error)?;
        if changed != 1 {
            return Err(PytxoError::Store(format!(
                "run recovery was not pending: {run_id}"
            )));
        }
        append_domain_change(&tx, "contract", run_id)?;
        tx.commit().map_err(store_error)?;
        Ok(())
    }

    pub fn insert_agent(
        &self,
        id: &str,
        run_id: &str,
        task_id: &str,
        wave: u32,
        worktree_path: Option<&str>,
        cmd: &str,
    ) -> Result<()> {
        self.insert_agent_with_root(id, run_id, task_id, wave, worktree_path, cmd, None)
    }

    /// Insert an agent tagged with its modular-project root label
    /// ([[ADR-0011-modular-project-manifest]]). `root_id` is `None` for single-root runs.
    #[allow(clippy::too_many_arguments)]
    pub fn insert_agent_with_root(
        &self,
        id: &str,
        run_id: &str,
        task_id: &str,
        wave: u32,
        worktree_path: Option<&str>,
        cmd: &str,
        root_id: Option<&str>,
    ) -> Result<()> {
        let tx = self.conn.unchecked_transaction().map_err(store_error)?;
        tx.execute(
            "INSERT INTO agents (id, run_id, task_id, wave, worktree_path, cmd, status, root_id)
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6, 'running', ?7)",
            params![
                id,
                run_id,
                task_id,
                wave as i32,
                worktree_path,
                cmd,
                root_id
            ],
        )
        .map_err(store_error)?;
        append_domain_change(&tx, "agent", id)?;
        tx.commit().map_err(store_error)?;
        Ok(())
    }

    pub fn finish_agent(&self, id: &str, exit_code: Option<i32>, status: &str) -> Result<()> {
        let tx = self.conn.unchecked_transaction().map_err(store_error)?;
        tx.execute(
            "UPDATE agents SET exit_code = ?1, status = ?2 WHERE id = ?3",
            params![exit_code, status, id],
        )
        .map_err(store_error)?;
        append_domain_change(&tx, "agent", id)?;
        tx.commit().map_err(store_error)?;
        Ok(())
    }

    pub fn set_agent_workspace(&self, id: &str, workspace: Option<&str>) -> Result<()> {
        let tx = self.conn.unchecked_transaction().map_err(store_error)?;
        tx.execute(
            "UPDATE agents SET worktree_path = ?1 WHERE id = ?2",
            params![workspace, id],
        )
        .map_err(store_error)?;
        append_domain_change(&tx, "agent", id)?;
        tx.commit().map_err(store_error)?;
        Ok(())
    }

    pub fn append_event(&self, agent_id: &str, kind: &str, payload: &str) -> Result<()> {
        self.append_event_with_change(agent_id, kind, payload, None)
    }

    pub fn append_approval_event(
        &self,
        agent_id: &str,
        approval_id: &str,
        kind: &str,
        payload: &str,
    ) -> Result<()> {
        self.append_event_with_change(agent_id, kind, payload, Some(("approval", approval_id)))
    }

    fn append_event_with_change(
        &self,
        agent_id: &str,
        kind: &str,
        payload: &str,
        entity_change: Option<(&str, &str)>,
    ) -> Result<()> {
        let now = Utc::now().to_rfc3339();
        let tx = self.conn.unchecked_transaction().map_err(store_error)?;
        tx.execute(
            "INSERT INTO events (agent_id, ts, kind, payload) VALUES (?1, ?2, ?3, ?4)",
            params![agent_id, now, kind, payload],
        )
        .map_err(store_error)?;
        let event_id = tx.last_insert_rowid().to_string();
        append_domain_change(&tx, "event", &event_id)?;
        if let Some((entity_kind, entity_id)) = entity_change {
            append_domain_change(&tx, entity_kind, entity_id)?;
        }
        tx.commit().map_err(store_error)?;
        Ok(())
    }

    pub fn list_runs(&self, limit: usize) -> Result<Vec<RunRecord>> {
        let mut stmt = self
            .conn
            .prepare(
                "SELECT id, started_at, finished_at, status, repo_root,
                        estimated_tokens_in, estimated_tokens_out, estimated_cost_usd,
                        permission_profile
                 FROM runs ORDER BY started_at DESC LIMIT ?1",
            )
            .map_err(|e| PytxoError::Store(e.to_string()))?;

        let rows = stmt
            .query_map(params![limit as i64], |row| {
                Ok(RunRecord {
                    id: row.get(0)?,
                    started_at: parse_dt(row.get::<_, String>(1)?),
                    finished_at: row.get::<_, Option<String>>(2)?.map(parse_dt),
                    status: row.get(3)?,
                    repo_root: row.get(4)?,
                    estimated_tokens_in: row.get(5)?,
                    estimated_tokens_out: row.get(6)?,
                    estimated_cost_usd: row.get(7)?,
                    permission_profile: row.get(8)?,
                })
            })
            .map_err(|e| PytxoError::Store(e.to_string()))?;

        rows.collect::<std::result::Result<Vec<_>, _>>()
            .map_err(|e| PytxoError::Store(e.to_string()))
    }

    pub fn get_run(&self, run_id: &str) -> Result<Option<RunRecord>> {
        self.conn
            .query_row(
                "SELECT id, started_at, finished_at, status, repo_root,
                        estimated_tokens_in, estimated_tokens_out, estimated_cost_usd,
                        permission_profile
                 FROM runs WHERE id = ?1",
                params![run_id],
                |row| {
                    Ok(RunRecord {
                        id: row.get(0)?,
                        started_at: parse_dt(row.get::<_, String>(1)?),
                        finished_at: row.get::<_, Option<String>>(2)?.map(parse_dt),
                        status: row.get(3)?,
                        repo_root: row.get(4)?,
                        estimated_tokens_in: row.get(5)?,
                        estimated_tokens_out: row.get(6)?,
                        estimated_cost_usd: row.get(7)?,
                        permission_profile: row.get(8)?,
                    })
                },
            )
            .optional()
            .map_err(|e| PytxoError::Store(e.to_string()))
    }

    pub fn arbitrage_by_agent(&self, run_id: &str) -> Result<Vec<(String, i64, i64, i64)>> {
        let mut stmt = self
            .conn
            .prepare(
                "SELECT agent_id,
                        COALESCE(SUM(saved_tokens), 0),
                        COUNT(DISTINCT path),
                        COALESCE(SUM(fallback_raw), 0)
                 FROM arbitrage_samples WHERE run_id = ?1 GROUP BY agent_id",
            )
            .map_err(|e| PytxoError::Store(e.to_string()))?;
        let rows = stmt
            .query_map(params![run_id], |row| {
                Ok((row.get(0)?, row.get(1)?, row.get(2)?, row.get(3)?))
            })
            .map_err(|e| PytxoError::Store(e.to_string()))?;
        rows.collect::<std::result::Result<Vec<_>, _>>()
            .map_err(|e| PytxoError::Store(e.to_string()))
    }

    /// Distinct edited paths for a run: (path, agent_id).
    pub fn arbitrage_paths_for_run(&self, run_id: &str) -> Result<Vec<(String, String)>> {
        let mut stmt = self
            .conn
            .prepare("SELECT DISTINCT path, agent_id FROM arbitrage_samples WHERE run_id = ?1")
            .map_err(|e| PytxoError::Store(e.to_string()))?;
        let rows = stmt
            .query_map(params![run_id], |row| Ok((row.get(0)?, row.get(1)?)))
            .map_err(|e| PytxoError::Store(e.to_string()))?;
        rows.collect::<std::result::Result<Vec<_>, _>>()
            .map_err(|e| PytxoError::Store(e.to_string()))
    }

    pub fn get_agent(&self, id: &str) -> Result<Option<AgentRecord>> {
        let mut stmt = self
            .conn
            .prepare(
                "SELECT id, run_id, task_id, wave, worktree_path, cmd, exit_code, status, root_id
                 FROM agents WHERE id = ?1",
            )
            .map_err(|e| PytxoError::Store(e.to_string()))?;
        let mut rows = stmt
            .query_map(params![id], |row| {
                Ok(AgentRecord {
                    id: row.get(0)?,
                    run_id: row.get(1)?,
                    task_id: row.get(2)?,
                    wave: row.get(3)?,
                    worktree_path: row.get(4)?,
                    cmd: row.get(5)?,
                    exit_code: row.get(6)?,
                    status: row.get(7)?,
                    root_id: row.get(8)?,
                })
            })
            .map_err(|e| PytxoError::Store(e.to_string()))?;
        rows.next()
            .transpose()
            .map_err(|e| PytxoError::Store(e.to_string()))
    }

    pub fn list_agents_for_run(&self, run_id: &str) -> Result<Vec<AgentRecord>> {
        let mut stmt = self
            .conn
            .prepare(
                "SELECT id, run_id, task_id, wave, worktree_path, cmd, exit_code, status, root_id
                 FROM agents WHERE run_id = ?1 ORDER BY wave, id",
            )
            .map_err(|e| PytxoError::Store(e.to_string()))?;

        let rows = stmt
            .query_map(params![run_id], |row| {
                Ok(AgentRecord {
                    id: row.get(0)?,
                    run_id: row.get(1)?,
                    task_id: row.get(2)?,
                    wave: row.get(3)?,
                    worktree_path: row.get(4)?,
                    cmd: row.get(5)?,
                    exit_code: row.get(6)?,
                    status: row.get(7)?,
                    root_id: row.get(8)?,
                })
            })
            .map_err(|e| PytxoError::Store(e.to_string()))?;

        rows.collect::<std::result::Result<Vec<_>, _>>()
            .map_err(|e| PytxoError::Store(e.to_string()))
    }

    pub fn list_events(&self, agent_id: &str, tail: usize) -> Result<Vec<EventRecord>> {
        let mut stmt = self
            .conn
            .prepare(
                "SELECT id, agent_id, ts, kind, payload FROM events
                 WHERE agent_id = ?1 ORDER BY id DESC LIMIT ?2",
            )
            .map_err(|e| PytxoError::Store(e.to_string()))?;

        let rows = stmt
            .query_map(params![agent_id, tail as i64], |row| {
                Ok(EventRecord {
                    id: row.get(0)?,
                    agent_id: row.get(1)?,
                    ts: parse_dt(row.get::<_, String>(2)?),
                    kind: row.get(3)?,
                    payload: row.get(4)?,
                })
            })
            .map_err(|e| PytxoError::Store(e.to_string()))?;

        let mut events: Vec<EventRecord> = rows
            .collect::<std::result::Result<Vec<_>, _>>()
            .map_err(|e| PytxoError::Store(e.to_string()))?;
        events.reverse();
        Ok(events)
    }

    pub fn latest_run(&self) -> Result<Option<RunRecord>> {
        Ok(self.list_runs(1)?.into_iter().next())
    }

    pub fn update_run_cost(
        &self,
        run_id: &str,
        tokens_in: i64,
        tokens_out: i64,
        cost_usd: f64,
    ) -> Result<()> {
        let tx = self.conn.unchecked_transaction().map_err(store_error)?;
        tx.execute(
            "UPDATE runs SET estimated_tokens_in = ?1, estimated_tokens_out = ?2,
                 estimated_cost_usd = ?3 WHERE id = ?4",
            params![tokens_in, tokens_out, cost_usd, run_id],
        )
        .map_err(store_error)?;
        append_domain_change(&tx, "run", run_id)?;
        tx.commit().map_err(store_error)?;
        Ok(())
    }

    pub fn tail_events_after(
        &self,
        agent_id: &str,
        after_id: i64,
        limit: usize,
    ) -> Result<Vec<EventRecord>> {
        let mut stmt = self
            .conn
            .prepare(
                "SELECT id, agent_id, ts, kind, payload FROM events
                 WHERE agent_id = ?1 AND id > ?2 ORDER BY id ASC LIMIT ?3",
            )
            .map_err(|e| PytxoError::Store(e.to_string()))?;

        let rows = stmt
            .query_map(params![agent_id, after_id, limit as i64], |row| {
                Ok(EventRecord {
                    id: row.get(0)?,
                    agent_id: row.get(1)?,
                    ts: parse_dt(row.get::<_, String>(2)?),
                    kind: row.get(3)?,
                    payload: row.get(4)?,
                })
            })
            .map_err(|e| PytxoError::Store(e.to_string()))?;

        rows.collect::<std::result::Result<Vec<_>, _>>()
            .map_err(|e| PytxoError::Store(e.to_string()))
    }

    /// MCP tool audit rows for a run (`events.kind = mcp-tool`).
    pub fn list_mcp_audit_for_run(&self, run_id: &str) -> Result<Vec<EventRecord>> {
        let mut stmt = self
            .conn
            .prepare(
                "SELECT e.id, e.agent_id, e.ts, e.kind, e.payload
                 FROM events e
                 JOIN agents a ON e.agent_id = a.id
                 WHERE a.run_id = ?1 AND e.kind = 'mcp-tool'
                 ORDER BY e.id ASC",
            )
            .map_err(|e| PytxoError::Store(e.to_string()))?;
        let rows = stmt
            .query_map(params![run_id], |row| {
                Ok(EventRecord {
                    id: row.get(0)?,
                    agent_id: row.get(1)?,
                    ts: parse_dt(row.get::<_, String>(2)?),
                    kind: row.get(3)?,
                    payload: row.get(4)?,
                })
            })
            .map_err(|e| PytxoError::Store(e.to_string()))?;
        rows.collect::<std::result::Result<Vec<_>, _>>()
            .map_err(|e| PytxoError::Store(e.to_string()))
    }

    /// Terminal statuses for domain runs.
    pub fn is_terminal_run_status(status: &str) -> bool {
        matches!(
            status,
            "completed" | "failed" | "failed_startup" | "cancelled"
        )
    }

    pub fn get_run_status(&self, run_id: &str) -> Result<Option<(String, Option<String>)>> {
        let mut stmt = self
            .conn
            .prepare("SELECT status, finished_at FROM runs WHERE id = ?1")
            .map_err(|e| PytxoError::Store(e.to_string()))?;
        let mut rows = stmt
            .query_map(params![run_id], |row| {
                Ok((row.get::<_, String>(0)?, row.get::<_, Option<String>>(1)?))
            })
            .map_err(|e| PytxoError::Store(e.to_string()))?;
        match rows.next() {
            Some(r) => r.map(Some).map_err(|e| PytxoError::Store(e.to_string())),
            None => Ok(None),
        }
    }

    pub fn domain_run_summary(&self) -> Result<DomainRunSummary> {
        let active: i64 = self
            .conn
            .query_row(
                "SELECT COUNT(*) FROM runs WHERE status IN ('starting', 'running')",
                [],
                |row| row.get(0),
            )
            .map_err(|e| PytxoError::Store(e.to_string()))?;

        let latest = self
            .conn
            .query_row(
                "SELECT id, status, started_at FROM runs ORDER BY started_at DESC LIMIT 1",
                [],
                |row| {
                    Ok((
                        row.get::<_, String>(0)?,
                        row.get::<_, String>(1)?,
                        row.get::<_, String>(2)?,
                    ))
                },
            )
            .ok();

        Ok(DomainRunSummary {
            active_runs: active as usize,
            latest_run_id: latest.as_ref().map(|(id, _, _)| id.clone()),
            latest_run_status: latest.as_ref().map(|(_, s, _)| s.clone()),
            latest_started_at: latest.map(|(_, _, t)| t),
        })
    }
}

fn store_error(error: rusqlite::Error) -> PytxoError {
    PytxoError::Store(error.to_string())
}

fn append_domain_change(tx: &Transaction<'_>, entity_kind: &str, entity_id: &str) -> Result<()> {
    tx.execute(
        "INSERT INTO domain_changes(entity_kind, entity_id, changed_at) VALUES (?1, ?2, ?3)",
        params![entity_kind, entity_id, Utc::now().to_rfc3339()],
    )
    .map_err(store_error)?;
    Ok(())
}

fn parse_dt(s: String) -> DateTime<Utc> {
    DateTime::parse_from_rfc3339(&s)
        .map(|d| d.with_timezone(&Utc))
        .unwrap_or_else(|_| Utc::now())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn round_trip_run_and_agent() {
        let dir = tempfile::tempdir().unwrap();
        let db = dir.path().join("pytxo.db");
        let store = PytxoStore::open(&db).unwrap();
        store.insert_run("run-1", "/tmp/repo").unwrap();
        store
            .insert_agent("a1", "run-1", "task-a", 0, None, "echo hi")
            .unwrap();
        store.finish_agent("a1", Some(0), "completed").unwrap();
        store.finish_run("run-1", "completed").unwrap();

        let agents = store.list_agents_for_run("run-1").unwrap();
        assert_eq!(agents.len(), 1);
        assert_eq!(agents[0].exit_code, Some(0));
    }

    #[test]
    fn finalizes_a_running_run_atomically() {
        let dir = tempfile::tempdir().unwrap();
        let store = PytxoStore::open(&dir.path().join("pytxo.db")).unwrap();
        store.insert_run("run-1", "/tmp/repo").unwrap();

        assert!(store.finish_run_if_running("run-1", "completed").unwrap());

        let (status, finished_at) = store.get_run_status("run-1").unwrap().unwrap();
        assert_eq!(status, "completed");
        assert!(finished_at.is_some());
    }

    #[test]
    fn running_only_finalization_preserves_explicit_cancellation() {
        let dir = tempfile::tempdir().unwrap();
        let store = PytxoStore::open(&dir.path().join("pytxo.db")).unwrap();
        store.insert_run("run-1", "/tmp/repo").unwrap();
        store.finish_run("run-1", "cancelled").unwrap();

        assert!(!store.finish_run_if_running("run-1", "failed").unwrap());

        let (status, finished_at) = store.get_run_status("run-1").unwrap().unwrap();
        assert_eq!(status, "cancelled");
        assert!(finished_at.is_some());
    }

    #[test]
    fn persists_and_claims_one_run_level_apply_contract() {
        let dir = tempfile::tempdir().unwrap();
        let store = PytxoStore::open(&dir.path().join("pytxo.db")).unwrap();
        store
            .insert_run_with_profile("run-1", "/tmp/repo", Some("orbit"))
            .unwrap();
        store
            .save_run_contract(
                "run-1",
                "abc123",
                r#"{"waves":[[]]}"#,
                r#"{"effective_profile":"orbit"}"#,
            )
            .unwrap();
        assert!(store.begin_run_preparation("run-1").unwrap());
        store
            .finish_run_preparation(
                "run-1",
                &PreparedRunManifest {
                    candidate_verification: None,
                    version: 1,
                    run_id: "run-1".into(),
                    base_revision: "abc123".into(),
                    prepared_at: "2026-07-31T00:00:00Z".into(),
                    package_digest: "prepared-digest".into(),
                    summary: Default::default(),
                    files: Vec::new(),
                },
            )
            .unwrap();
        assert!(store.finish_run_if_running("run-1", "completed").unwrap());

        let contract = store.get_run_contract("run-1").unwrap().unwrap();
        assert_eq!(contract.base_revision.as_deref(), Some("abc123"));
        assert_eq!(contract.plan_json.as_deref(), Some(r#"{"waves":[[]]}"#));
        assert_eq!(contract.apply_status, "ready");
        assert_eq!(
            contract.enforcement_json.as_deref(),
            Some(r#"{"effective_profile":"orbit"}"#)
        );

        assert!(store.claim_run_apply("run-1").unwrap());
        assert!(!store.claim_run_apply("run-1").unwrap());
        store
            .finish_run_apply("run-1", "applied", Some(r#"{"paths":["src/lib.rs"]}"#))
            .unwrap();

        let applied = store.get_run_contract("run-1").unwrap().unwrap();
        assert_eq!(applied.apply_status, "applied");
        assert_eq!(
            applied.apply_manifest_json.as_deref(),
            Some(r#"{"paths":["src/lib.rs"]}"#)
        );
        assert!(applied.applied_at.is_some());
    }

    #[test]
    fn contract_preparation_and_domain_changes_are_durable_and_bounded() {
        let dir = tempfile::tempdir().unwrap();
        let store = PytxoStore::open(&dir.path().join("pytxo.db")).unwrap();
        store
            .insert_run_with_profile("run-delta", "/tmp/repo", Some("orbit"))
            .unwrap();
        store
            .save_run_contract(
                "run-delta",
                "abc123",
                r#"{"waves":[[]]}"#,
                r#"{"effective_profile":"orbit"}"#,
            )
            .unwrap();

        let manifest = pytxo_core::PreparedRunManifest {
            candidate_verification: None,
            version: 1,
            run_id: "run-delta".into(),
            base_revision: "abc123".into(),
            prepared_at: "2026-07-31T00:00:00Z".into(),
            package_digest: "digest".into(),
            summary: Default::default(),
            files: Vec::new(),
        };
        assert!(store.begin_run_preparation("run-delta").unwrap());
        store
            .finish_run_preparation("run-delta", &manifest)
            .unwrap();

        let contract = store.get_run_contract("run-delta").unwrap().unwrap();
        assert_eq!(contract.apply_status, "ready");
        assert_eq!(contract.prepared_digest.as_deref(), Some("digest"));
        assert_eq!(contract.prepared_manifest.as_ref(), Some(&manifest));

        let first_page = store.changes_since(0, 2).unwrap();
        assert_eq!(first_page.changes.len(), 2);
        assert!(first_page.has_more);
        assert!(!first_page.cursor_gap);
        let second_page = store.changes_since(first_page.next_cursor, 100).unwrap();
        assert!(second_page
            .changes
            .iter()
            .any(|change| change.entity_kind == "contract" && change.entity_id == "run-delta"));
    }

    #[test]
    fn concurrent_apply_claim_retry_and_discard_follow_contract_states() {
        let dir = tempfile::tempdir().unwrap();
        let db_path = dir.path().join("pytxo.db");
        let store = PytxoStore::open(&db_path).unwrap();
        store.insert_run("run-race", "/tmp/repo").unwrap();
        store
            .save_run_contract("run-race", "base", "{}", "{}")
            .unwrap();
        assert!(store.begin_run_preparation("run-race").unwrap());
        store
            .finish_run_preparation(
                "run-race",
                &PreparedRunManifest {
                    candidate_verification: None,
                    version: 1,
                    run_id: "run-race".into(),
                    base_revision: "base".into(),
                    prepared_at: "2026-07-31T00:00:00Z".into(),
                    package_digest: "digest".into(),
                    summary: Default::default(),
                    files: Vec::new(),
                },
            )
            .unwrap();
        store.finish_run("run-race", "completed").unwrap();
        drop(store);

        let barrier = std::sync::Arc::new(std::sync::Barrier::new(2));
        let threads = (0..2)
            .map(|_| {
                let path = db_path.clone();
                let barrier = barrier.clone();
                std::thread::spawn(move || {
                    let store = PytxoStore::open(&path).unwrap();
                    barrier.wait();
                    store.claim_run_apply("run-race").unwrap()
                })
            })
            .collect::<Vec<_>>();
        let claims = threads
            .into_iter()
            .map(|thread| thread.join().unwrap())
            .collect::<Vec<_>>();
        assert_eq!(claims.iter().filter(|claimed| **claimed).count(), 1);

        let store = PytxoStore::open(&db_path).unwrap();
        let error = RunApplyError {
            at: "2026-07-31T00:00:01Z".into(),
            code: "copy_failed".into(),
            message: "rolled back".into(),
            attempt_id: Some("attempt-1".into()),
            rollback_confirmed: true,
        };
        store
            .finish_run_apply_error("run-race", "ready", &error, Some("rolled_back"))
            .unwrap();
        assert!(store.claim_run_apply("run-race").unwrap());
        store
            .finish_run_apply_error("run-race", "ready", &error, Some("rolled_back"))
            .unwrap();
        assert!(store.discard_run_review("run-race").unwrap());
        assert_eq!(
            store
                .get_run_contract("run-race")
                .unwrap()
                .unwrap()
                .apply_status,
            "discarded"
        );
    }

    #[test]
    fn explicit_recovery_transitions_are_authoritative_and_audited() {
        let dir = tempfile::tempdir().unwrap();
        let store = PytxoStore::open(&dir.path().join("pytxo.db")).unwrap();
        let run_id = "run-recovery-transition";
        store.insert_run(run_id, "/tmp/repo").unwrap();
        store.save_run_contract(run_id, "base", "{}", "{}").unwrap();
        assert!(store.begin_run_preparation(run_id).unwrap());
        store
            .finish_run_preparation(
                run_id,
                &PreparedRunManifest {
                    candidate_verification: None,
                    version: 1,
                    run_id: run_id.into(),
                    base_revision: "base".into(),
                    prepared_at: "2026-08-01T00:00:00Z".into(),
                    package_digest: "digest".into(),
                    summary: Default::default(),
                    files: vec![],
                },
            )
            .unwrap();
        store.finish_run(run_id, "completed").unwrap();
        let error = RunApplyError {
            at: "2026-08-01T00:01:00Z".into(),
            code: "recovery_unprovable".into(),
            message: "recovery required".into(),
            attempt_id: Some("attempt-1".into()),
            rollback_confirmed: false,
        };
        assert!(store.claim_run_apply(run_id).unwrap());
        store
            .finish_run_apply_error(run_id, "recovery_required", &error, Some("unprovable"))
            .unwrap();
        let rolled_back = RunApplyError {
            rollback_confirmed: true,
            message: "rolled back".into(),
            ..error.clone()
        };
        store
            .finish_run_recovery_error(run_id, "ready", &rolled_back, Some("rolled_back"))
            .unwrap();
        assert_eq!(
            store
                .get_run_contract(run_id)
                .unwrap()
                .unwrap()
                .apply_status,
            "ready"
        );

        assert!(store.claim_run_apply(run_id).unwrap());
        store
            .finish_run_apply_error(run_id, "recovery_required", &error, Some("unprovable"))
            .unwrap();
        store
            .finish_run_recovery_apply(run_id, r#"{"transaction_id":"attempt-1","changes":[]}"#)
            .unwrap();
        let applied = store.get_run_contract(run_id).unwrap().unwrap();
        assert_eq!(applied.apply_status, "applied");
        assert!(applied.applied_at.is_some());
        assert!(applied.last_apply_error.is_none());
        assert!(applied.recovery_state.is_none());
        assert!(store
            .changes_since(0, 100)
            .unwrap()
            .changes
            .iter()
            .any(|change| change.entity_kind == "contract" && change.entity_id == run_id));
    }

    #[test]
    fn approval_event_and_delta_are_written_in_one_store_operation() {
        let dir = tempfile::tempdir().unwrap();
        let store = PytxoStore::open(&dir.path().join("pytxo.db")).unwrap();
        store.insert_run("run-approval", "/tmp/repo").unwrap();
        store
            .insert_agent(
                "approval-resolver",
                "run-approval",
                "resolve",
                0,
                None,
                "internal",
            )
            .unwrap();
        let cursor = store.changes_since(0, 100).unwrap().next_cursor;

        store
            .append_approval_event(
                "approval-resolver",
                "approval-123",
                "hitl-resolve",
                r#"{"decision":"approved"}"#,
            )
            .unwrap();

        let page = store.changes_since(cursor, 100).unwrap();
        assert!(page
            .changes
            .iter()
            .any(|change| change.entity_kind == "approval" && change.entity_id == "approval-123"));
        assert!(page
            .changes
            .iter()
            .any(|change| change.entity_kind == "event"));
    }

    #[test]
    fn cursor_beyond_replacement_database_max_requests_reset() {
        let dir = tempfile::tempdir().unwrap();
        let database = dir.path().join("pytxo.db");
        let old_cursor = {
            let store = PytxoStore::open(&database).unwrap();
            store.insert_run("old-run", "/tmp/repo").unwrap();
            store
                .insert_agent("old-agent", "old-run", "task", 0, None, "internal")
                .unwrap();
            store.changes_since(0, 100).unwrap().next_cursor
        };
        std::fs::remove_file(&database).unwrap();

        let replacement = PytxoStore::open(&database).unwrap();
        let empty_reset = replacement.changes_since(old_cursor, 100).unwrap();
        assert!(empty_reset.cursor_gap);
        assert!(empty_reset.changes.is_empty());
        assert_eq!(empty_reset.next_cursor, 0);
        let empty_settled = replacement
            .changes_since(empty_reset.next_cursor, 100)
            .unwrap();
        assert!(!empty_settled.cursor_gap);
        assert_eq!(empty_settled.next_cursor, 0);

        replacement.insert_run("new-run", "/tmp/repo").unwrap();
        let page = replacement.changes_since(old_cursor, 100).unwrap();
        assert!(page.cursor_gap);
        assert!(page.changes.is_empty());
        assert_eq!(page.next_cursor, 1);

        let settled = replacement.changes_since(page.next_cursor, 100).unwrap();
        assert!(!settled.cursor_gap);
        assert!(settled.changes.is_empty());
        assert_eq!(settled.next_cursor, 1);
    }
}
