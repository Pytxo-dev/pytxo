-- An operation pins the numeric rates that settlement must use. A revision
-- label alone cannot protect a funded reservation from mismatched services.
ALTER TABLE routing_operations
    ADD COLUMN input_nano_usd_per_token BIGINT NOT NULL DEFAULT 0
        CHECK (input_nano_usd_per_token >= 0),
    ADD COLUMN output_nano_usd_per_token BIGINT NOT NULL DEFAULT 0
        CHECK (output_nano_usd_per_token >= 0);
