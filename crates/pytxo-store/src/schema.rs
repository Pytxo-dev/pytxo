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

pub const MIGRATION_005: &str = r#"
ALTER TABLE runs ADD COLUMN permission_profile TEXT;
"#;

/// Run-level review/apply contract. Keeping this normalized avoids widening the
/// hot run-list query while making one atomic apply state authoritative.
pub const MIGRATION_006: &str = r#"
CREATE TABLE IF NOT EXISTS run_contracts (
    run_id TEXT PRIMARY KEY,
    base_revision TEXT,
    plan_json TEXT NOT NULL,
    apply_status TEXT NOT NULL DEFAULT 'pending',
    apply_manifest_json TEXT,
    applied_at TEXT,
    enforcement_json TEXT NOT NULL,
    FOREIGN KEY (run_id) REFERENCES runs(id)
);

CREATE INDEX IF NOT EXISTS idx_run_contracts_apply_status
ON run_contracts(apply_status);
"#;

pub const MIGRATION_007: &str = r#"
ALTER TABLE run_contracts ADD COLUMN prepared_manifest_json TEXT;
ALTER TABLE run_contracts ADD COLUMN prepared_digest TEXT;
ALTER TABLE run_contracts ADD COLUMN prepared_at TEXT;
ALTER TABLE run_contracts ADD COLUMN last_apply_error_json TEXT;
ALTER TABLE run_contracts ADD COLUMN recovery_state TEXT;

CREATE TABLE domain_changes (
    sequence INTEGER PRIMARY KEY AUTOINCREMENT,
    entity_kind TEXT NOT NULL,
    entity_id TEXT NOT NULL,
    changed_at TEXT NOT NULL
);

CREATE INDEX idx_domain_changes_entity
ON domain_changes(entity_kind, entity_id, sequence);
"#;

/// Routing is opt-in. Legacy runs acquire no synthesized authority or attempts.
pub const MIGRATION_008: &str = r#"
CREATE TABLE routing_missions (
    run_id TEXT PRIMARY KEY REFERENCES runs(id),
    domain_id TEXT NOT NULL,
    registration_json TEXT NOT NULL,
    registration_digest TEXT NOT NULL,
    cancel_epoch INTEGER NOT NULL CHECK(cancel_epoch >= 0),
    cancelled INTEGER NOT NULL CHECK(cancelled IN (0,1)),
    revision INTEGER NOT NULL CHECK(revision >= 1)
);
CREATE TABLE routing_tasks (
    run_id TEXT NOT NULL REFERENCES routing_missions(run_id),
    task_id TEXT NOT NULL,
    revision INTEGER NOT NULL CHECK(revision >= 1),
    record_json TEXT NOT NULL,
    PRIMARY KEY(run_id,task_id)
);
CREATE TABLE routing_attempts (
    attempt_id TEXT PRIMARY KEY,
    agent_id TEXT NOT NULL UNIQUE,
    capacity_reservation TEXT NOT NULL UNIQUE,
    run_id TEXT NOT NULL,
    task_id TEXT NOT NULL,
    ordinal INTEGER NOT NULL CHECK(ordinal > 0),
    revision INTEGER NOT NULL CHECK(revision >= 1),
    record_json TEXT NOT NULL,
    UNIQUE(run_id,task_id,ordinal),
    FOREIGN KEY(run_id,task_id) REFERENCES routing_tasks(run_id,task_id)
);
CREATE TABLE routing_qualifications (
    run_id TEXT NOT NULL REFERENCES routing_missions(run_id),
    digest TEXT NOT NULL,
    qualification_json TEXT NOT NULL,
    PRIMARY KEY(run_id,digest)
);
CREATE TABLE routing_control_events (
    sequence INTEGER PRIMARY KEY AUTOINCREMENT,
    run_id TEXT NOT NULL REFERENCES routing_missions(run_id),
    domain_id TEXT NOT NULL,
    event_id TEXT NOT NULL,
    payload_digest TEXT NOT NULL,
    event_json TEXT NOT NULL,
    result_json TEXT NOT NULL,
    UNIQUE(run_id,event_id)
);
CREATE INDEX idx_routing_events_scope ON routing_control_events(domain_id,run_id,sequence);
"#;

/// Private reviewed mission bytes exist before the legacy run row. The run ID
/// is unique within a domain store and intentionally has no `runs` foreign key.
pub const MIGRATION_009: &str = r#"
CREATE TABLE routing_mission_stages (
    run_id TEXT PRIMARY KEY NOT NULL,
    domain_id TEXT NOT NULL,
    draft_id TEXT NOT NULL,
    plan_digest TEXT NOT NULL,
    mission_digest TEXT NOT NULL,
    mission_json TEXT NOT NULL
);
"#;

/// Private cross-database reservation intent. The Store row fences new claims
/// until exact Catalog release has been checked and closure is durable.
pub const MIGRATION_010: &str = r#"
CREATE TABLE routing_capacity_intents (
    reservation_id TEXT PRIMARY KEY NOT NULL,
    domain_id TEXT NOT NULL,
    run_id TEXT NOT NULL,
    task_id TEXT NOT NULL,
    attempt_id TEXT NOT NULL UNIQUE,
    creation_event_id TEXT NOT NULL,
    phase TEXT NOT NULL CHECK (phase IN ('prepared', 'reserve_may_have_started', 'release_proof_bound', 'closed')),
    revision INTEGER NOT NULL CHECK (revision >= 1),
    record_json TEXT NOT NULL,
    UNIQUE (run_id, creation_event_id),
    UNIQUE (run_id, task_id, attempt_id),
    FOREIGN KEY (run_id, task_id) REFERENCES routing_tasks(run_id, task_id)
);
CREATE INDEX idx_routing_capacity_intents_phase ON routing_capacity_intents(phase);
CREATE UNIQUE INDEX idx_routing_capacity_intents_unresolved_task
    ON routing_capacity_intents(run_id, task_id) WHERE phase <> 'closed';
"#;

/// Private retained routed bytes and one-use native creation intent. Neither
/// table is a worker launch permit or independent OS verification.
pub const MIGRATION_011: &str = r#"
CREATE TABLE routing_private_artifacts (
    artifact_id TEXT PRIMARY KEY NOT NULL,
    domain_id TEXT NOT NULL,
    run_id TEXT NOT NULL,
    task_id TEXT NOT NULL,
    attempt_id TEXT NOT NULL,
    reservation_id TEXT NOT NULL,
    kind TEXT NOT NULL CHECK(kind IN ('input_manifest','handoff_manifest','scoped_output','controller_receipt','no_worker_receipt')),
    event_id TEXT NOT NULL,
    digest TEXT NOT NULL,
    byte_length INTEGER NOT NULL CHECK(byte_length >= 0 AND byte_length <= 16777216),
    claim_json TEXT NOT NULL,
    bytes BLOB NOT NULL,
    UNIQUE(run_id,event_id),
    FOREIGN KEY(run_id,task_id) REFERENCES routing_tasks(run_id,task_id)
);
CREATE UNIQUE INDEX idx_routing_private_input_singleton
    ON routing_private_artifacts(run_id,task_id,attempt_id) WHERE kind='input_manifest';
CREATE UNIQUE INDEX idx_routing_private_handoff_singleton
    ON routing_private_artifacts(run_id,task_id,attempt_id) WHERE kind='handoff_manifest';
CREATE INDEX idx_routing_private_run ON routing_private_artifacts(run_id);
CREATE TABLE attempt_launch_ownership (
    attempt_id TEXT PRIMARY KEY NOT NULL REFERENCES routing_attempts(attempt_id),
    domain_id TEXT NOT NULL,
    run_id TEXT NOT NULL,
    task_id TEXT NOT NULL,
    reservation_id TEXT NOT NULL,
    launch_token TEXT NOT NULL,
    job_name TEXT,
    launch_nonce TEXT,
    pid INTEGER,
    start_identity TEXT,
    settlement_artifact_id TEXT REFERENCES routing_private_artifacts(artifact_id),
    phase TEXT NOT NULL CHECK(phase IN ('prepared','create_may_have_started','registered','settled','closed_no_launch')),
    revision INTEGER NOT NULL CHECK(revision >= 1),
    record_json TEXT NOT NULL,
    UNIQUE(run_id,launch_token),
    CHECK((phase='prepared' AND job_name IS NULL AND launch_nonce IS NULL AND pid IS NULL AND start_identity IS NULL AND settlement_artifact_id IS NULL)
       OR (phase='create_may_have_started' AND job_name IS NOT NULL AND launch_nonce IS NOT NULL AND pid IS NULL AND start_identity IS NULL AND settlement_artifact_id IS NULL)
       OR (phase='registered' AND job_name IS NOT NULL AND launch_nonce IS NOT NULL AND pid IS NOT NULL AND start_identity IS NOT NULL AND settlement_artifact_id IS NULL)
       OR (phase='settled' AND job_name IS NOT NULL AND launch_nonce IS NOT NULL AND pid IS NOT NULL AND start_identity IS NOT NULL AND settlement_artifact_id IS NOT NULL)
       OR (phase='closed_no_launch' AND job_name IS NULL AND launch_nonce IS NULL AND pid IS NULL AND start_identity IS NULL AND settlement_artifact_id IS NOT NULL))
);
CREATE UNIQUE INDEX idx_attempt_launch_job_name ON attempt_launch_ownership(job_name) WHERE job_name IS NOT NULL;
CREATE INDEX idx_attempt_launch_unsettled ON attempt_launch_ownership(phase) WHERE phase <> 'settled';
"#;

/// Checker processes have their own one-use ownership records. Worker ownership
/// remains in v11; changing that released table would break existing stores.
pub const MIGRATION_012: &str = r#"
CREATE TABLE attempt_checker_ownership (
    attempt_id TEXT NOT NULL REFERENCES routing_attempts(attempt_id),
    ordinal INTEGER NOT NULL CHECK(ordinal >= 1 AND ordinal <= 16),
    check_id TEXT NOT NULL,
    domain_id TEXT NOT NULL,
    run_id TEXT NOT NULL,
    task_id TEXT NOT NULL,
    reservation_id TEXT NOT NULL,
    launch_token TEXT NOT NULL,
    recipe_digest TEXT NOT NULL,
    sealed_view_digest TEXT NOT NULL,
    sealed_view_length INTEGER NOT NULL CHECK(sealed_view_length >= 0 AND sealed_view_length <= 16777216),
    prepare_event_id TEXT NOT NULL,
    job_name TEXT,
    launch_nonce TEXT,
    pid INTEGER,
    start_identity TEXT,
    settlement_artifact_id TEXT REFERENCES routing_private_artifacts(artifact_id),
    passed INTEGER CHECK(passed IN (0,1)),
    phase TEXT NOT NULL CHECK(phase IN ('prepared','create_may_have_started','registered','settled','closed_no_create')),
    revision INTEGER NOT NULL CHECK(revision >= 1),
    record_json TEXT NOT NULL,
    PRIMARY KEY(attempt_id,ordinal),
    UNIQUE(attempt_id,check_id),
    UNIQUE(run_id,prepare_event_id),
    CHECK((phase='prepared' AND job_name IS NULL AND launch_nonce IS NULL AND pid IS NULL AND start_identity IS NULL AND settlement_artifact_id IS NULL AND passed IS NULL)
       OR (phase='create_may_have_started' AND job_name IS NOT NULL AND launch_nonce IS NOT NULL AND pid IS NULL AND start_identity IS NULL AND settlement_artifact_id IS NULL AND passed IS NULL)
       OR (phase='registered' AND job_name IS NOT NULL AND launch_nonce IS NOT NULL AND pid IS NOT NULL AND start_identity IS NOT NULL AND settlement_artifact_id IS NULL AND passed IS NULL)
       OR (phase='settled' AND job_name IS NOT NULL AND launch_nonce IS NOT NULL AND pid IS NOT NULL AND start_identity IS NOT NULL AND settlement_artifact_id IS NOT NULL AND passed IS NOT NULL)
       OR (phase='closed_no_create' AND job_name IS NULL AND launch_nonce IS NULL AND pid IS NULL AND start_identity IS NULL AND settlement_artifact_id IS NOT NULL AND passed IS NULL))
);
CREATE UNIQUE INDEX idx_attempt_checker_job_name ON attempt_checker_ownership(job_name) WHERE job_name IS NOT NULL;
CREATE INDEX idx_attempt_checker_unsettled ON attempt_checker_ownership(phase) WHERE phase NOT IN ('settled','closed_no_create');
"#;

/// Per-domain, default-off routing disclosure consent and a one-send journal.
/// No redacted task packet or provider credential is retained here.
pub const MIGRATION_013: &str = r#"
CREATE TABLE routing_advisor_consent (
    domain_id TEXT PRIMARY KEY NOT NULL,
    revision INTEGER NOT NULL CHECK(revision >= 1),
    enabled INTEGER NOT NULL CHECK(enabled IN (0,1)),
    updated_at_ms INTEGER NOT NULL CHECK(updated_at_ms >= 0)
);
CREATE TABLE routing_advisor_requests (
    request_id TEXT PRIMARY KEY NOT NULL,
    domain_id TEXT NOT NULL,
    run_id TEXT NOT NULL REFERENCES routing_missions(run_id),
    task_id TEXT NOT NULL,
    ordinal INTEGER NOT NULL CHECK(ordinal > 0),
    policy_digest TEXT NOT NULL,
    packet_digest TEXT NOT NULL,
    consent_revision INTEGER NOT NULL CHECK(consent_revision >= 1),
    task_revision INTEGER NOT NULL CHECK(task_revision >= 1),
    task_state_revision INTEGER NOT NULL CHECK(task_state_revision >= 1),
    authorization_revision INTEGER NOT NULL CHECK(authorization_revision >= 1),
    cancel_epoch INTEGER NOT NULL CHECK(cancel_epoch >= 0),
    phase TEXT NOT NULL CHECK(phase IN ('prepared','sending_may_have_happened','completed','uncertain','late_receipt')),
    result_digest TEXT,
    created_at_ms INTEGER NOT NULL CHECK(created_at_ms >= 0),
    updated_at_ms INTEGER NOT NULL CHECK(updated_at_ms >= 0),
    UNIQUE(run_id,task_id,ordinal),
    FOREIGN KEY(run_id,task_id) REFERENCES routing_tasks(run_id,task_id),
    CHECK((phase IN ('prepared','sending_may_have_happened','uncertain') AND result_digest IS NULL)
       OR (phase IN ('completed','late_receipt') AND result_digest IS NOT NULL))
);
CREATE INDEX idx_routing_advisor_requests_scope ON routing_advisor_requests(domain_id,run_id,task_id);
"#;

/// A pre-scope grant is never valid for a newly reviewed disclosure recipient.
/// Existing request-journal rows remain intact for accounting and recovery.
pub const MIGRATION_014: &str = r#"
ALTER TABLE routing_advisor_consent ADD COLUMN scope_digest TEXT;
UPDATE routing_advisor_consent SET enabled=0 WHERE enabled=1;
"#;

/// Hosted disclosure has a separate recipient-scoped grant. Existing fixture
/// consent and journal entries remain readable but cannot authorize a hosted
/// send: their request recipient and scope columns stay NULL.
pub const MIGRATION_015: &str = r#"
CREATE TABLE routing_hosted_advisor_consent (
    domain_id TEXT NOT NULL,
    recipient_identity TEXT NOT NULL,
    revision INTEGER NOT NULL CHECK(revision >= 1),
    enabled INTEGER NOT NULL CHECK(enabled IN (0,1)),
    scope_digest TEXT,
    updated_at_ms INTEGER NOT NULL CHECK(updated_at_ms >= 0),
    PRIMARY KEY(domain_id,recipient_identity),
    CHECK((enabled=1 AND scope_digest IS NOT NULL) OR (enabled=0 AND scope_digest IS NULL))
);
ALTER TABLE routing_advisor_requests ADD COLUMN recipient_identity TEXT;
ALTER TABLE routing_advisor_requests ADD COLUMN scope_digest TEXT;
"#;
