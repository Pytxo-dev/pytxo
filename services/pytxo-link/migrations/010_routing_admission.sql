-- Sponsored routing has its own ledger. No subscription entitlement, Ultra
-- wallet, or provider usage table can grant an admission here.
CREATE TABLE routing_funding (
    singleton BOOLEAN PRIMARY KEY DEFAULT TRUE CHECK (singleton),
    enabled BOOLEAN NOT NULL DEFAULT FALSE,
    ceiling_nano_usd BIGINT NOT NULL DEFAULT 0 CHECK (ceiling_nano_usd >= 0),
    spent_nano_usd BIGINT NOT NULL DEFAULT 0 CHECK (spent_nano_usd >= 0),
    held_nano_usd BIGINT NOT NULL DEFAULT 0 CHECK (held_nano_usd >= 0),
    disabled_reason TEXT
);
INSERT INTO routing_funding (singleton) VALUES (TRUE);

CREATE TABLE routing_account_gate (
    account_id TEXT PRIMARY KEY,
    rate_window TIMESTAMPTZ NOT NULL DEFAULT date_trunc('minute', now()),
    rate_used INTEGER NOT NULL DEFAULT 0 CHECK (rate_used >= 0),
    active INTEGER NOT NULL DEFAULT 0 CHECK (active >= 0)
);

CREATE TABLE routing_account_month (
    account_id TEXT NOT NULL,
    utc_month DATE NOT NULL,
    held_input_tokens BIGINT NOT NULL DEFAULT 0 CHECK (held_input_tokens >= 0),
    consumed_input_tokens BIGINT NOT NULL DEFAULT 0 CHECK (consumed_input_tokens >= 0),
    PRIMARY KEY (account_id, utc_month)
);

CREATE TABLE routing_operations (
    account_id TEXT NOT NULL,
    request_id TEXT NOT NULL,
    token_hash BYTEA NOT NULL,
    payload_mac BYTEA NOT NULL,
    mac_revision TEXT NOT NULL,
    state TEXT NOT NULL CHECK (state IN
        ('reserved', 'sending', 'completed', 'rejected_before_send',
         'uncertain', 'settled_at_ceiling')),
    owner_nonce BYTEA,
    utc_month DATE NOT NULL,
    rate_card_revision TEXT NOT NULL,
    reserved_input_tokens BIGINT NOT NULL CHECK (reserved_input_tokens > 0),
    reserved_nano_usd BIGINT NOT NULL CHECK (reserved_nano_usd > 0),
    actual_input_tokens BIGINT,
    actual_nano_usd BIGINT,
    receipt_id TEXT,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    send_claimed_at TIMESTAMPTZ,
    settled_at TIMESTAMPTZ,
    PRIMARY KEY (account_id, request_id)
);
CREATE INDEX routing_operations_stale_sending
    ON routing_operations(send_claimed_at) WHERE state = 'sending';
CREATE UNIQUE INDEX routing_operations_owner_nonce
    ON routing_operations(owner_nonce) WHERE owner_nonce IS NOT NULL;
CREATE INDEX routing_operations_tombstone_age
    ON routing_operations(created_at);
