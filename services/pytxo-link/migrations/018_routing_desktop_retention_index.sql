-- Expired one-use codes are pruned in bounded batches without scanning the
-- full bridge table or changing the already-applied migration 017.
CREATE INDEX routing_desktop_codes_expiry
    ON routing_desktop_codes(expires_at);
