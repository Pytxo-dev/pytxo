-- Optional Pytxo account: CLI/Desktop sign-in codes and credentials, shared
-- receipts and the public Pybench board. Codes and credentials are stored as
-- SHA-256 hashes only. Account credentials carry no entitlement, billing or
-- Routing authority; Routing keeps its own tables (017/019).
CREATE TABLE account_codes (
    code_hash BYTEA PRIMARY KEY CHECK (octet_length(code_hash) = 32),
    account_id TEXT NOT NULL,
    state_hash BYTEA NOT NULL CHECK (octet_length(state_hash) = 32),
    challenge BYTEA NOT NULL CHECK (octet_length(challenge) = 32),
    created_at TIMESTAMPTZ NOT NULL DEFAULT clock_timestamp(),
    expires_at TIMESTAMPTZ NOT NULL,
    consumed_at TIMESTAMPTZ
);
CREATE INDEX account_codes_account_time ON account_codes(account_id, created_at);

CREATE TABLE account_sessions (
    token_hash BYTEA PRIMARY KEY CHECK (octet_length(token_hash) = 32),
    account_id TEXT NOT NULL,
    label TEXT CHECK (label IS NULL OR char_length(label) <= 48),
    created_at TIMESTAMPTZ NOT NULL DEFAULT clock_timestamp(),
    expires_at TIMESTAMPTZ NOT NULL,
    last_used_at TIMESTAMPTZ,
    revoked_at TIMESTAMPTZ
);
CREATE INDEX account_sessions_account ON account_sessions(account_id, created_at DESC);

CREATE TABLE shared_receipts (
    id TEXT PRIMARY KEY CHECK (id ~ '^r_[0-9A-Za-z]{12}$'),
    account_id TEXT NOT NULL,
    content_digest TEXT NOT NULL,
    payload JSONB NOT NULL CHECK (octet_length(payload::text) <= 65536),
    created_at TIMESTAMPTZ NOT NULL DEFAULT clock_timestamp(),
    deleted_at TIMESTAMPTZ,
    UNIQUE (account_id, content_digest)
);
CREATE INDEX shared_receipts_account ON shared_receipts(account_id, created_at DESC);

CREATE TABLE bench_results (
    id TEXT PRIMARY KEY CHECK (id ~ '^b_[0-9A-Za-z]{12}$'),
    account_id TEXT NOT NULL,
    content_digest TEXT NOT NULL,
    payload JSONB NOT NULL CHECK (octet_length(payload::text) <= 8192),
    created_at TIMESTAMPTZ NOT NULL DEFAULT clock_timestamp(),
    deleted_at TIMESTAMPTZ,
    UNIQUE (account_id, content_digest)
);
CREATE INDEX bench_results_created ON bench_results(created_at DESC);
