-- Bound reported output usage independently, and make the one-minute account
-- admission count efficient without relying on calendar-minute buckets.
ALTER TABLE routing_operations
    ADD COLUMN reserved_output_tokens BIGINT NOT NULL DEFAULT 0
        CHECK (reserved_output_tokens >= 0);
CREATE INDEX routing_operations_account_recent
    ON routing_operations(account_id, created_at DESC);
