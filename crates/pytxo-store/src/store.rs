use std::path::Path;

use chrono::{DateTime, Utc};
use pytxo_core::{PytxoError, Result};
use rusqlite::{params, Connection};
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

#[derive(Clone, Debug)]
pub struct EventRecord {
    pub id: i64,
    pub agent_id: String,
    pub ts: DateTime<Utc>,
    pub kind: String,
    pub payload: String,
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
        let now = Utc::now().to_rfc3339();
        self.conn
            .execute(
                "INSERT INTO runs (id, started_at, status, repo_root, permission_profile) VALUES (?1, ?2, 'running', ?3, ?4)",
                params![id, now, repo_root, permission_profile],
            )
            .map_err(|e| PytxoError::Store(e.to_string()))?;
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
        self.conn
            .execute(
                "UPDATE runs SET project_id = ?1, root_id = ?2 WHERE id = ?3",
                params![project_id, root_id, run_id],
            )
            .map_err(|e| PytxoError::Store(e.to_string()))?;
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
        self.conn
            .execute(
                "UPDATE runs SET finished_at = ?1, status = ?2 WHERE id = ?3",
                params![now, status, id],
            )
            .map_err(|e| PytxoError::Store(e.to_string()))?;
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
        self.conn
            .execute(
                "INSERT INTO agents (id, run_id, task_id, wave, worktree_path, cmd, status, root_id)
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6, 'running', ?7)",
                params![id, run_id, task_id, wave as i32, worktree_path, cmd, root_id],
            )
            .map_err(|e| PytxoError::Store(e.to_string()))?;
        Ok(())
    }

    pub fn finish_agent(&self, id: &str, exit_code: Option<i32>, status: &str) -> Result<()> {
        self.conn
            .execute(
                "UPDATE agents SET exit_code = ?1, status = ?2 WHERE id = ?3",
                params![exit_code, status, id],
            )
            .map_err(|e| PytxoError::Store(e.to_string()))?;
        Ok(())
    }

    pub fn append_event(&self, agent_id: &str, kind: &str, payload: &str) -> Result<()> {
        let now = Utc::now().to_rfc3339();
        self.conn
            .execute(
                "INSERT INTO events (agent_id, ts, kind, payload) VALUES (?1, ?2, ?3, ?4)",
                params![agent_id, now, kind, payload],
            )
            .map_err(|e| PytxoError::Store(e.to_string()))?;
        Ok(())
    }

    pub fn list_runs(&self, limit: usize) -> Result<Vec<RunRecord>> {
        let mut stmt = self
            .conn
            .prepare(
                "SELECT id, started_at, finished_at, status, repo_root,
                        estimated_tokens_in, estimated_tokens_out, estimated_cost_usd
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
                })
            })
            .map_err(|e| PytxoError::Store(e.to_string()))?;

        rows.collect::<std::result::Result<Vec<_>, _>>()
            .map_err(|e| PytxoError::Store(e.to_string()))
    }

    /// Per-agent Signal Core arbitrage totals for a run: (agent_id, saved_tokens, edited_path_count).
    /// Feeds the Reality Deck topology graph ([[reality-deck-visual-system]]).
    pub fn arbitrage_by_agent(&self, run_id: &str) -> Result<Vec<(String, i64, i64)>> {
        let mut stmt = self
            .conn
            .prepare(
                "SELECT agent_id, COALESCE(SUM(saved_tokens), 0), COUNT(DISTINCT path)
                 FROM arbitrage_samples WHERE run_id = ?1 GROUP BY agent_id",
            )
            .map_err(|e| PytxoError::Store(e.to_string()))?;
        let rows = stmt
            .query_map(params![run_id], |row| {
                Ok((row.get(0)?, row.get(1)?, row.get(2)?))
            })
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
        self.conn
            .execute(
                "UPDATE runs SET estimated_tokens_in = ?1, estimated_tokens_out = ?2,
                 estimated_cost_usd = ?3 WHERE id = ?4",
                params![tokens_in, tokens_out, cost_usd, run_id],
            )
            .map_err(|e| PytxoError::Store(e.to_string()))?;
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
}
