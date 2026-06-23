-- Phase 41/49: inference usage ledger + org audit log

CREATE TABLE IF NOT EXISTS inference_usage (
    id BIGSERIAL PRIMARY KEY,
    user_id TEXT NOT NULL,
    domain_id TEXT,
    run_id TEXT,
    provider TEXT NOT NULL,
    model TEXT,
    tokens_in BIGINT NOT NULL DEFAULT 0,
    tokens_out BIGINT NOT NULL DEFAULT 0,
    cost_micro_usd BIGINT NOT NULL DEFAULT 0,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE INDEX IF NOT EXISTS idx_inference_usage_user ON inference_usage (user_id, created_at DESC);

CREATE TABLE IF NOT EXISTS audit_log (
    id BIGSERIAL PRIMARY KEY,
    org_id TEXT,
    actor_id TEXT NOT NULL,
    action TEXT NOT NULL,
    detail JSONB,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE INDEX IF NOT EXISTS idx_audit_log_org ON audit_log (org_id, created_at DESC);
