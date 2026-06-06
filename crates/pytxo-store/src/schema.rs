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

pub const MIGRATION_003: &str = r#"
CREATE TABLE IF NOT EXISTS wallet_accounts (
    domain_id TEXT PRIMARY KEY,
    balance_microcredits INTEGER NOT NULL,
    reserved_microcredits INTEGER NOT NULL DEFAULT 0,
    synced_at TEXT
);

CREATE TABLE IF NOT EXISTS wallet_reservations (
    id TEXT PRIMARY KEY,
    domain_id TEXT NOT NULL,
    run_id TEXT NOT NULL,
    amount_microcredits INTEGER NOT NULL,
    status TEXT NOT NULL,
    created_at TEXT NOT NULL
);

CREATE TABLE IF NOT EXISTS arbitrage_samples (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    run_id TEXT NOT NULL,
    agent_id TEXT NOT NULL,
    domain_id TEXT NOT NULL,
    path TEXT NOT NULL,
    raw_bytes INTEGER NOT NULL,
    scaffolded_bytes INTEGER NOT NULL,
    raw_tokens_in INTEGER NOT NULL,
    sent_tokens_in INTEGER NOT NULL,
    saved_tokens INTEGER NOT NULL,
    reduction_pct REAL NOT NULL,
    fallback_raw INTEGER NOT NULL,
    ts TEXT NOT NULL
);

CREATE TABLE IF NOT EXISTS usage_records (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    run_id TEXT NOT NULL,
    agent_id TEXT NOT NULL,
    domain_id TEXT NOT NULL,
    model TEXT NOT NULL,
    tokens_in_billed INTEGER NOT NULL,
    tokens_in_sent INTEGER NOT NULL,
    tokens_out INTEGER NOT NULL,
    cost_micro_usd INTEGER NOT NULL,
    source TEXT NOT NULL,
    ts TEXT NOT NULL
);

CREATE INDEX IF NOT EXISTS idx_arbitrage_run ON arbitrage_samples(run_id);
CREATE INDEX IF NOT EXISTS idx_usage_run ON usage_records(run_id);
"#;

/// Modular projects ([[ADR-0011-modular-project-manifest]]): tag runs/agents with
/// the owning project and path root so the Deck can filter by project and folder.
pub const MIGRATION_004: &str = r#"
ALTER TABLE runs ADD COLUMN project_id TEXT;
ALTER TABLE runs ADD COLUMN root_id TEXT;
ALTER TABLE agents ADD COLUMN root_id TEXT;
"#;
