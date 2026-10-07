-- Hosted consent is separate from the existing local no-network Shadow grant.
-- Opaque workspace IDs are random client identifiers, never repository paths.
CREATE TABLE routing_workspace_grants (
    account_id TEXT NOT NULL,
    workspace_id BYTEA NOT NULL CHECK (octet_length(workspace_id) = 16),
    recipient_identity TEXT NOT NULL,
    scope_digest BYTEA NOT NULL CHECK (octet_length(scope_digest) = 32),
    revision BIGINT NOT NULL CHECK (revision > 0),
    enabled BOOLEAN NOT NULL DEFAULT FALSE,
    updated_at TIMESTAMPTZ NOT NULL DEFAULT clock_timestamp(),
    PRIMARY KEY (account_id, workspace_id, recipient_identity)
);

-- Existing unscoped tokens and operations retain accounting history but can
-- no longer authorize a hosted send or recover a completed answer.
ALTER TABLE routing_session_tokens
    ADD COLUMN workspace_id BYTEA CHECK (workspace_id IS NULL OR octet_length(workspace_id) = 16),
    ADD COLUMN grant_revision BIGINT CHECK (grant_revision IS NULL OR grant_revision > 0),
    ADD COLUMN recipient_identity TEXT,
    ADD COLUMN scope_digest BYTEA CHECK (scope_digest IS NULL OR octet_length(scope_digest) = 32),
    ADD CONSTRAINT routing_session_tokens_grant_complete CHECK (
        (workspace_id IS NULL AND grant_revision IS NULL AND recipient_identity IS NULL AND scope_digest IS NULL)
        OR
        (workspace_id IS NOT NULL AND grant_revision IS NOT NULL AND recipient_identity IS NOT NULL AND scope_digest IS NOT NULL)
    );
ALTER TABLE routing_session_tokens DROP CONSTRAINT routing_session_tokens_account_id_key;
CREATE UNIQUE INDEX routing_session_tokens_workspace
    ON routing_session_tokens(account_id, workspace_id, recipient_identity)
    WHERE workspace_id IS NOT NULL;

ALTER TABLE routing_operations
    ADD COLUMN workspace_id BYTEA CHECK (workspace_id IS NULL OR octet_length(workspace_id) = 16),
    ADD COLUMN grant_revision BIGINT CHECK (grant_revision IS NULL OR grant_revision > 0),
    ADD COLUMN recipient_identity TEXT,
    ADD COLUMN scope_digest BYTEA CHECK (scope_digest IS NULL OR octet_length(scope_digest) = 32),
    ADD CONSTRAINT routing_operations_grant_complete CHECK (
        (workspace_id IS NULL AND grant_revision IS NULL AND recipient_identity IS NULL AND scope_digest IS NULL)
        OR
        (workspace_id IS NOT NULL AND grant_revision IS NOT NULL AND recipient_identity IS NOT NULL AND scope_digest IS NOT NULL)
    );
