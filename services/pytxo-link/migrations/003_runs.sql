-- Phase 40: authoritative run ledger for Ultra reconcile (ADR-0021).
CREATE TABLE IF NOT EXISTS runs (
    run_id TEXT PRIMARY KEY,
    domain_id TEXT NOT NULL,
    idempotency_key TEXT UNIQUE,
    status TEXT NOT NULL DEFAULT 'started',
    started_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    ended_at TIMESTAMPTZ,
    usage_json JSONB
);

CREATE INDEX IF NOT EXISTS idx_runs_domain ON runs(domain_id);
CREATE INDEX IF NOT EXISTS idx_runs_started ON runs(started_at DESC);
