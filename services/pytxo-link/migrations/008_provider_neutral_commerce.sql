-- Provider-neutral commerce events and subscription grants. Payment providers
-- are ingress adapters; Pytxo entitlement capabilities remain Link-owned.
CREATE TABLE IF NOT EXISTS billing_events (
    provider TEXT NOT NULL,
    event_id TEXT NOT NULL,
    event_type TEXT NOT NULL,
    occurred_at TIMESTAMPTZ NOT NULL,
    subscription_id TEXT NOT NULL,
    outcome TEXT NOT NULL,
    received_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    PRIMARY KEY (provider, event_id)
);

CREATE INDEX IF NOT EXISTS idx_billing_events_subscription
    ON billing_events(provider, subscription_id, occurred_at DESC);

CREATE TABLE IF NOT EXISTS billing_subscriptions (
    provider TEXT NOT NULL,
    subscription_id TEXT NOT NULL,
    customer_id TEXT NOT NULL,
    user_id TEXT NOT NULL,
    plan_key TEXT NOT NULL,
    lifecycle_state TEXT NOT NULL,
    access_until TIMESTAMPTZ,
    last_event_at TIMESTAMPTZ NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    PRIMARY KEY (provider, subscription_id)
);

CREATE INDEX IF NOT EXISTS idx_billing_subscriptions_user
    ON billing_subscriptions(user_id, lifecycle_state);

CREATE TABLE IF NOT EXISTS entitlement_grants (
    source TEXT NOT NULL,
    grant_id TEXT NOT NULL,
    user_id TEXT NOT NULL,
    tier TEXT NOT NULL,
    max_agents INT NOT NULL,
    cloud_enabled BOOLEAN NOT NULL,
    active BOOLEAN NOT NULL,
    valid_until TIMESTAMPTZ,
    updated_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    PRIMARY KEY (source, grant_id)
);

CREATE INDEX IF NOT EXISTS idx_entitlement_grants_user
    ON entitlement_grants(user_id, active);

-- Carry the existing Paddle identity bindings and replay keys forward. The old
-- tables remain intact for rollback during the dual-provider window.
INSERT INTO billing_events
    (provider, event_id, event_type, occurred_at, subscription_id, outcome, received_at)
SELECT 'paddle', event_id, event_type, received_at, 'legacy-unknown', 'applied', received_at
FROM paddle_webhook_events
ON CONFLICT (provider, event_id) DO NOTHING;

WITH latest_paid_subscription AS (
    SELECT DISTINCT ON (subscriptions.user_id)
           subscriptions.subscription_id, subscriptions.customer_id,
           subscriptions.user_id, subscriptions.created_at, subscriptions.updated_at,
           entitlements.tier, entitlements.max_agents, entitlements.cloud_enabled
    FROM paddle_subscriptions AS subscriptions
    JOIN entitlements ON entitlements.user_id = subscriptions.user_id
    WHERE entitlements.tier IN ('pro', 'max', 'ultra')
    ORDER BY subscriptions.user_id, subscriptions.updated_at DESC,
             subscriptions.subscription_id DESC
)
INSERT INTO billing_subscriptions
    (provider, subscription_id, customer_id, user_id, plan_key, lifecycle_state,
     last_event_at, created_at, updated_at)
SELECT 'paddle', subscription_id, customer_id, user_id, tier, 'active',
       updated_at, created_at, updated_at
FROM latest_paid_subscription
ON CONFLICT (provider, subscription_id) DO NOTHING;

INSERT INTO entitlement_grants
    (source, grant_id, user_id, tier, max_agents, cloud_enabled, active, updated_at)
SELECT 'paddle', subscriptions.subscription_id, subscriptions.user_id,
       entitlements.tier, entitlements.max_agents, entitlements.cloud_enabled,
       TRUE, subscriptions.updated_at
FROM billing_subscriptions AS subscriptions
JOIN entitlements ON entitlements.user_id = subscriptions.user_id
WHERE subscriptions.provider = 'paddle'
  AND entitlements.tier IN ('pro', 'max', 'ultra')
ON CONFLICT (source, grant_id) DO NOTHING;

-- Existing entitlements with no Paddle binding are treated as the current
-- administrative grant so later admin updates replace, rather than stack with,
-- this migrated state.
INSERT INTO entitlement_grants
    (source, grant_id, user_id, tier, max_agents, cloud_enabled, active, updated_at)
SELECT 'admin', entitlements.user_id, entitlements.user_id, entitlements.tier,
       entitlements.max_agents, entitlements.cloud_enabled, TRUE, entitlements.updated_at
FROM entitlements
WHERE entitlements.tier <> 'core'
  AND NOT EXISTS (
    SELECT 1 FROM entitlement_grants
    WHERE entitlement_grants.source = 'paddle'
      AND entitlement_grants.user_id = entitlements.user_id
  )
ON CONFLICT (source, grant_id) DO NOTHING;
