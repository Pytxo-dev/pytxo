-- Keep both sides of model usage in the isolated sponsored ledger.
ALTER TABLE routing_operations
    ADD COLUMN actual_output_tokens BIGINT CHECK (actual_output_tokens >= 0);
