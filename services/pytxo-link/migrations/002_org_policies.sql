CREATE TABLE IF NOT EXISTS org_policies (
    org_id TEXT PRIMARY KEY,
    default_permission_profile TEXT,
    shared_trusted_domains JSONB NOT NULL DEFAULT '[]'::jsonb,
    updated_at TIMESTAMPTZ NOT NULL DEFAULT now()
);
