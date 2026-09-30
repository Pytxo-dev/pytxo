-- Experimental routing sessions are account-bound and have no relation to
-- subscription/API billing or the managed-inference wallet.
CREATE TABLE routing_session_tokens (
    token_hash BYTEA PRIMARY KEY,
    account_id TEXT NOT NULL UNIQUE,
    scope TEXT NOT NULL CHECK (scope = 'routing:evaluate:v1'),
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    expires_at TIMESTAMPTZ NOT NULL,
    revoked_at TIMESTAMPTZ
);

CREATE INDEX routing_session_tokens_expiry ON routing_session_tokens(expires_at);
