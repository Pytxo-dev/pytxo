-- Repair: ensure inference_usage exists if migration 004 checksum was repaired without re-applying DDL.

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
