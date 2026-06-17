CREATE TABLE IF NOT EXISTS entitlements (
    user_id TEXT PRIMARY KEY,
    clerk_user_id TEXT,
    org_id TEXT,
    tier TEXT NOT NULL DEFAULT 'core',
    max_agents INT NOT NULL DEFAULT 3,
    cloud_enabled BOOLEAN NOT NULL DEFAULT false,
    updated_at TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE INDEX IF NOT EXISTS idx_entitlements_clerk ON entitlements(clerk_user_id);
