-- Durable replay protection for Paddle entitlement webhooks.
CREATE TABLE IF NOT EXISTS paddle_webhook_events (
    event_id TEXT PRIMARY KEY,
    event_type TEXT NOT NULL,
    received_at TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE INDEX IF NOT EXISTS idx_paddle_webhook_events_received
    ON paddle_webhook_events(received_at DESC);

-- Server-side identity binding established only by subscription.created.
CREATE TABLE IF NOT EXISTS paddle_subscriptions (
    subscription_id TEXT PRIMARY KEY,
    customer_id TEXT NOT NULL,
    user_id TEXT NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE INDEX IF NOT EXISTS idx_paddle_subscriptions_customer
    ON paddle_subscriptions(customer_id);
