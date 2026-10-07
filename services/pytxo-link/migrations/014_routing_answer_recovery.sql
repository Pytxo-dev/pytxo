-- Retain only an encrypted, minimal advisory response for a bounded client
-- delivery retry. The proxy owns the encryption key; Link never stores a raw
-- task packet or plaintext advice. Accounting tombstones outlive this receipt.
ALTER TABLE routing_operations
    ADD COLUMN recovery_ciphertext BYTEA,
    ADD COLUMN recovery_nonce BYTEA,
    ADD COLUMN recovery_key_revision TEXT,
    ADD COLUMN recovery_expires_at TIMESTAMPTZ;

ALTER TABLE routing_operations
    ADD CONSTRAINT routing_recovery_all_or_none CHECK (
        (recovery_ciphertext IS NULL AND recovery_nonce IS NULL
         AND recovery_key_revision IS NULL AND recovery_expires_at IS NULL)
        OR
        (recovery_ciphertext IS NOT NULL AND recovery_nonce IS NOT NULL
         AND recovery_key_revision IS NOT NULL AND recovery_expires_at IS NOT NULL)
    );

CREATE INDEX routing_operations_recovery_expiry
    ON routing_operations(recovery_expires_at)
    WHERE recovery_expires_at IS NOT NULL;
