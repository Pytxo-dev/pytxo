pub const MIGRATION_001: &str = r#"
CREATE TABLE IF NOT EXISTS runs (
    id TEXT PRIMARY KEY,
    started_at TEXT NOT NULL,
    finished_at TEXT,
    status TEXT NOT NULL,
    repo_root TEXT NOT NULL
);

CREATE TABLE IF NOT EXISTS agents (
    id TEXT PRIMARY KEY,
    run_id TEXT NOT NULL,
    task_id TEXT NOT NULL,
    wave INTEGER NOT NULL,
    worktree_path TEXT,
    cmd TEXT NOT NULL,
    exit_code INTEGER,
    status TEXT NOT NULL,
    FOREIGN KEY (run_id) REFERENCES runs(id)
);

CREATE TABLE IF NOT EXISTS events (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    agent_id TEXT NOT NULL,
    ts TEXT NOT NULL,
    kind TEXT NOT NULL,
    payload TEXT NOT NULL,
    FOREIGN KEY (agent_id) REFERENCES agents(id)
);

CREATE INDEX IF NOT EXISTS idx_agents_run ON agents(run_id);
CREATE INDEX IF NOT EXISTS idx_events_agent ON events(agent_id);
"#;

pub const MIGRATION_002: &str = r#"
ALTER TABLE runs ADD COLUMN estimated_tokens_in INTEGER;
ALTER TABLE runs ADD COLUMN estimated_tokens_out INTEGER;
ALTER TABLE runs ADD COLUMN estimated_cost_usd REAL;
ALTER TABLE agents ADD COLUMN estimated_tokens_in INTEGER;
ALTER TABLE agents ADD COLUMN estimated_tokens_out INTEGER;
"#;
