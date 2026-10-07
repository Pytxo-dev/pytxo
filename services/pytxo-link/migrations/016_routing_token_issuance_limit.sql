-- Throttle short-lived sponsored tokens across all workspaces of one account.
ALTER TABLE routing_account_gate
    ADD COLUMN token_issue_window TIMESTAMPTZ NOT NULL DEFAULT date_trunc('minute', clock_timestamp()),
    ADD COLUMN token_issue_count INTEGER NOT NULL DEFAULT 0 CHECK (token_issue_count >= 0);
