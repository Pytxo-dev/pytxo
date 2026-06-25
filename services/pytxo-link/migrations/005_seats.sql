-- Phase 59: enterprise org seat ledger

CREATE TABLE IF NOT EXISTS org_seats (
    org_id TEXT PRIMARY KEY,
    seats_total INTEGER NOT NULL DEFAULT 10,
    updated_at TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE INDEX IF NOT EXISTS idx_entitlements_org ON entitlements (org_id) WHERE org_id IS NOT NULL;
