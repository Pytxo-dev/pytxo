-- A browser session may authorize one native Desktop, but never places a
-- reusable bearer in a custom-scheme URL. Both secrets are stored as hashes.
CREATE TABLE routing_desktop_codes (
    code_hash BYTEA PRIMARY KEY CHECK (octet_length(code_hash) = 32),
    account_id TEXT NOT NULL,
    state_hash BYTEA NOT NULL CHECK (octet_length(state_hash) = 32),
    challenge BYTEA NOT NULL CHECK (octet_length(challenge) = 32),
    created_at TIMESTAMPTZ NOT NULL DEFAULT clock_timestamp(),
    expires_at TIMESTAMPTZ NOT NULL,
    consumed_at TIMESTAMPTZ
);
CREATE INDEX routing_desktop_codes_account_time
    ON routing_desktop_codes(account_id, created_at);

-- One scoped Desktop credential per account is a deliberate v1 limit. A new
-- sign-in rotates the previous one; it has no entitlement or Ultra authority.
CREATE TABLE routing_desktop_sessions (
    token_hash BYTEA PRIMARY KEY CHECK (octet_length(token_hash) = 32),
    account_id TEXT NOT NULL UNIQUE,
    scope TEXT NOT NULL CHECK (scope = 'routing:grants:v1'),
    created_at TIMESTAMPTZ NOT NULL DEFAULT clock_timestamp(),
    expires_at TIMESTAMPTZ NOT NULL,
    revoked_at TIMESTAMPTZ
);
CREATE INDEX routing_desktop_sessions_expiry
    ON routing_desktop_sessions(expires_at);
