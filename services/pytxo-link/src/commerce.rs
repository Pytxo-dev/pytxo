#[cfg(test)]
use std::collections::{HashMap, HashSet};
#[cfg(test)]
use std::sync::{Arc, Mutex};

use chrono::{DateTime, Utc};
use sqlx::{PgPool, Postgres, Transaction};

use crate::entitlements::{
    lock_entitlement_in_transaction, recompute_entitlement_in_transaction,
    upsert_grant_in_transaction, EntitlementGrant, EntitlementStore, Tier,
};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum CommerceProvider {
    Paddle,
    Dodo,
}

impl CommerceProvider {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Paddle => "paddle",
            Self::Dodo => "dodo",
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum PlanKey {
    Pro,
    Max,
    Ultra,
}

impl PlanKey {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Pro => "pro",
            Self::Max => "max",
            Self::Ultra => "ultra",
        }
    }

    fn parse(value: &str) -> Option<Self> {
        match value {
            "pro" => Some(Self::Pro),
            "max" => Some(Self::Max),
            "ultra" => Some(Self::Ultra),
            _ => None,
        }
    }

    pub fn tier(self) -> Tier {
        match self {
            Self::Pro => Tier::Pro,
            Self::Max => Tier::Max,
            Self::Ultra => Tier::Ultra,
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum SubscriptionLifecycle {
    Active,
    Grace,
    Suspended,
    Ended,
}

impl SubscriptionLifecycle {
    fn as_str(self) -> &'static str {
        match self {
            Self::Active => "active",
            Self::Grace => "grace",
            Self::Suspended => "suspended",
            Self::Ended => "ended",
        }
    }

    fn parse(value: &str) -> Option<Self> {
        match value {
            "active" => Some(Self::Active),
            "grace" => Some(Self::Grace),
            "suspended" => Some(Self::Suspended),
            "ended" => Some(Self::Ended),
            _ => None,
        }
    }

    fn grants_access(self) -> bool {
        matches!(self, Self::Active | Self::Grace)
    }
}

/// Provider-neutral subscription state accepted by Link's business layer.
/// Provider payloads and user-supplied tier metadata never cross this boundary.
#[derive(Clone, Debug)]
pub struct SubscriptionEvent {
    pub provider: CommerceProvider,
    pub event_id: String,
    pub event_type: String,
    pub occurred_at: DateTime<Utc>,
    pub subscription_id: String,
    pub customer_id: String,
    pub establishes_binding: bool,
    pub user_id: Option<String>,
    pub plan: Option<PlanKey>,
    pub lifecycle: SubscriptionLifecycle,
    pub access_until: Option<DateTime<Utc>>,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ReconcileOutcome {
    Applied,
    Duplicate,
    IgnoredStale,
    Rejected,
}

#[derive(Clone)]
pub enum CommerceStore {
    Postgres(PgPool),
    #[cfg(test)]
    Memory(Arc<Mutex<MemoryCommerceState>>),
}

#[cfg(test)]
#[derive(Default)]
pub(crate) struct MemoryCommerceState {
    events: HashSet<(String, String)>,
    subscriptions: HashMap<(String, String), StoredSubscription>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct StoredSubscription {
    customer_id: String,
    user_id: String,
    plan: PlanKey,
    lifecycle: SubscriptionLifecycle,
    access_until: Option<DateTime<Utc>>,
    last_event_at: DateTime<Utc>,
}

impl CommerceStore {
    pub fn postgres(pool: PgPool) -> Self {
        Self::Postgres(pool)
    }

    #[cfg(test)]
    pub fn memory() -> Self {
        Self::Memory(Arc::new(Mutex::new(MemoryCommerceState::default())))
    }

    pub async fn reconcile(
        &self,
        _entitlements: &EntitlementStore,
        event: &SubscriptionEvent,
    ) -> Result<ReconcileOutcome, sqlx::Error> {
        if !valid_event(event) {
            return Ok(ReconcileOutcome::Rejected);
        }
        match self {
            Self::Postgres(pool) => reconcile_postgres(pool, event).await,
            #[cfg(test)]
            Self::Memory(store) => reconcile_memory(store, _entitlements, event).await,
        }
    }
}

fn valid_event(event: &SubscriptionEvent) -> bool {
    !event.event_id.trim().is_empty()
        && !event.event_type.trim().is_empty()
        && !event.subscription_id.trim().is_empty()
        && !event.customer_id.trim().is_empty()
        && event
            .user_id
            .as_ref()
            .is_none_or(|id| !id.trim().is_empty())
}

#[cfg(test)]
async fn reconcile_memory(
    store: &Arc<Mutex<MemoryCommerceState>>,
    entitlements: &EntitlementStore,
    event: &SubscriptionEvent,
) -> Result<ReconcileOutcome, sqlx::Error> {
    let provider = event.provider.as_str().to_string();
    let event_key = (provider.clone(), event.event_id.clone());
    let subscription_key = (provider.clone(), event.subscription_id.clone());

    let (stored, previous) = {
        let mut state = store.lock().unwrap();
        if state.events.contains(&event_key) {
            return Ok(ReconcileOutcome::Duplicate);
        }
        let previous = state.subscriptions.get(&subscription_key).cloned();
        let Some(stored) = next_subscription(previous.as_ref(), event) else {
            state.events.insert(event_key);
            return Ok(ReconcileOutcome::Rejected);
        };
        if previous
            .as_ref()
            .is_some_and(|current| should_ignore_as_stale(current, &stored))
        {
            state.events.insert(event_key);
            return Ok(ReconcileOutcome::IgnoredStale);
        }
        state.events.insert(event_key.clone());
        state
            .subscriptions
            .insert(subscription_key.clone(), stored.clone());
        (stored, previous)
    };

    let grant = grant_for(event.provider, &event.subscription_id, &stored);
    if let Err(error) = entitlements.apply_grant(grant).await {
        let mut state = store.lock().unwrap();
        state.events.remove(&event_key);
        if let Some(previous) = previous {
            state.subscriptions.insert(subscription_key, previous);
        } else {
            state.subscriptions.remove(&subscription_key);
        }
        return Err(error);
    }
    Ok(ReconcileOutcome::Applied)
}

fn next_subscription(
    current: Option<&StoredSubscription>,
    event: &SubscriptionEvent,
) -> Option<StoredSubscription> {
    if let Some(current) = current {
        if current.customer_id != event.customer_id
            || event
                .user_id
                .as_deref()
                .is_some_and(|user_id| user_id != current.user_id)
        {
            return None;
        }
        let plan = event.plan.unwrap_or(current.plan);
        return Some(StoredSubscription {
            customer_id: current.customer_id.clone(),
            user_id: current.user_id.clone(),
            plan,
            lifecycle: event.lifecycle,
            access_until: event.access_until,
            last_event_at: event.occurred_at,
        });
    }

    if !event.establishes_binding || !event.lifecycle.grants_access() {
        return None;
    }
    Some(StoredSubscription {
        customer_id: event.customer_id.clone(),
        user_id: event.user_id.as_ref()?.trim().to_string(),
        plan: event.plan?,
        lifecycle: event.lifecycle,
        access_until: event.access_until,
        last_event_at: event.occurred_at,
    })
}

fn should_ignore_as_stale(current: &StoredSubscription, proposed: &StoredSubscription) -> bool {
    if proposed.last_event_at < current.last_event_at {
        return true;
    }
    if proposed.last_event_at > current.last_event_at {
        return false;
    }

    // Equal provider timestamps have no trustworthy order. Apply only a state
    // that cannot increase access; otherwise keep the committed row.
    let lifecycle_not_wider =
        lifecycle_rank(proposed.lifecycle) <= lifecycle_rank(current.lifecycle);
    let plan_not_higher = plan_rank(proposed.plan) <= plan_rank(current.plan);
    let access_not_extended = match (current.access_until, proposed.access_until) {
        (None, None) | (None, Some(_)) => true,
        (Some(_), None) => false,
        (Some(current), Some(proposed)) => proposed <= current,
    };
    !(lifecycle_not_wider && plan_not_higher && access_not_extended)
}

fn lifecycle_rank(lifecycle: SubscriptionLifecycle) -> u8 {
    match lifecycle {
        SubscriptionLifecycle::Ended | SubscriptionLifecycle::Suspended => 0,
        SubscriptionLifecycle::Grace => 1,
        SubscriptionLifecycle::Active => 2,
    }
}

fn plan_rank(plan: PlanKey) -> u8 {
    match plan {
        PlanKey::Pro => 1,
        PlanKey::Max => 2,
        PlanKey::Ultra => 3,
    }
}

fn grant_for(
    provider: CommerceProvider,
    subscription_id: &str,
    subscription: &StoredSubscription,
) -> EntitlementGrant {
    let tier = subscription.plan.tier();
    EntitlementGrant {
        source: provider.as_str().to_string(),
        grant_id: subscription_id.to_string(),
        user_id: subscription.user_id.clone(),
        tier,
        max_agents: tier.max_agents(),
        cloud_enabled: matches!(tier, Tier::Max | Tier::Ultra),
        active: subscription.lifecycle.grants_access(),
        valid_until: subscription.access_until,
    }
}

async fn reconcile_postgres(
    pool: &PgPool,
    event: &SubscriptionEvent,
) -> Result<ReconcileOutcome, sqlx::Error> {
    let mut transaction = pool.begin().await?;
    let provider = event.provider.as_str();
    let inserted = sqlx::query(
        r#"
        INSERT INTO billing_events
            (provider, event_id, event_type, occurred_at, subscription_id, outcome)
        VALUES ($1, $2, $3, $4, $5, 'received')
        ON CONFLICT (provider, event_id) DO NOTHING
        "#,
    )
    .bind(provider)
    .bind(&event.event_id)
    .bind(&event.event_type)
    .bind(event.occurred_at)
    .bind(&event.subscription_id)
    .execute(&mut *transaction)
    .await?
    .rows_affected();
    if inserted == 0 {
        transaction.rollback().await?;
        return Ok(ReconcileOutcome::Duplicate);
    }

    let mut current =
        fetch_subscription(&mut transaction, provider, &event.subscription_id).await?;
    let created = if current.is_none() {
        if !event.establishes_binding || !event.lifecycle.grants_access() {
            mark_event_outcome(&mut transaction, provider, &event.event_id, "rejected").await?;
            transaction.commit().await?;
            return Ok(ReconcileOutcome::Rejected);
        }
        let (Some(user_id), Some(plan)) = (event.user_id.as_deref(), event.plan) else {
            mark_event_outcome(&mut transaction, provider, &event.event_id, "rejected").await?;
            transaction.commit().await?;
            return Ok(ReconcileOutcome::Rejected);
        };
        sqlx::query(
            r#"
            INSERT INTO billing_subscriptions
                (provider, subscription_id, customer_id, user_id, plan_key, lifecycle_state,
                 access_until, last_event_at, updated_at)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, now())
            ON CONFLICT (provider, subscription_id) DO NOTHING
            "#,
        )
        .bind(provider)
        .bind(&event.subscription_id)
        .bind(&event.customer_id)
        .bind(user_id.trim())
        .bind(plan.as_str())
        .bind(event.lifecycle.as_str())
        .bind(event.access_until)
        .bind(event.occurred_at)
        .execute(&mut *transaction)
        .await?
        .rows_affected()
            == 1
    } else {
        false
    };
    if current.is_none() {
        current = fetch_subscription(&mut transaction, provider, &event.subscription_id).await?;
    }
    let Some(current) = current else {
        mark_event_outcome(&mut transaction, provider, &event.event_id, "rejected").await?;
        transaction.commit().await?;
        return Ok(ReconcileOutcome::Rejected);
    };
    if current.customer_id != event.customer_id
        || event
            .user_id
            .as_deref()
            .is_some_and(|user_id| user_id.trim() != current.user_id)
    {
        mark_event_outcome(&mut transaction, provider, &event.event_id, "rejected").await?;
        transaction.commit().await?;
        return Ok(ReconcileOutcome::Rejected);
    }
    let Some(stored) = next_subscription(Some(&current), event) else {
        mark_event_outcome(&mut transaction, provider, &event.event_id, "rejected").await?;
        transaction.commit().await?;
        return Ok(ReconcileOutcome::Rejected);
    };
    if !created && should_ignore_as_stale(&current, &stored) {
        mark_event_outcome(&mut transaction, provider, &event.event_id, "ignored_stale").await?;
        transaction.commit().await?;
        return Ok(ReconcileOutcome::IgnoredStale);
    }
    sqlx::query(
        r#"
        UPDATE billing_subscriptions
        SET plan_key = $3,
            lifecycle_state = $4,
            access_until = $5,
            last_event_at = $6,
            updated_at = now()
        WHERE provider = $1 AND subscription_id = $2
        "#,
    )
    .bind(provider)
    .bind(&event.subscription_id)
    .bind(stored.plan.as_str())
    .bind(stored.lifecycle.as_str())
    .bind(stored.access_until)
    .bind(stored.last_event_at)
    .execute(&mut *transaction)
    .await?;

    let grant = grant_for(event.provider, &event.subscription_id, &stored);
    lock_entitlement_in_transaction(&mut transaction, &stored.user_id).await?;
    upsert_grant_in_transaction(&mut transaction, &grant).await?;
    recompute_entitlement_in_transaction(&mut transaction, &stored.user_id).await?;
    mark_event_outcome(&mut transaction, provider, &event.event_id, "applied").await?;
    transaction.commit().await?;
    Ok(ReconcileOutcome::Applied)
}

async fn mark_event_outcome(
    transaction: &mut Transaction<'_, Postgres>,
    provider: &str,
    event_id: &str,
    outcome: &str,
) -> Result<(), sqlx::Error> {
    sqlx::query("UPDATE billing_events SET outcome = $3 WHERE provider = $1 AND event_id = $2")
        .bind(provider)
        .bind(event_id)
        .bind(outcome)
        .execute(&mut **transaction)
        .await?;
    Ok(())
}

async fn fetch_subscription(
    transaction: &mut Transaction<'_, Postgres>,
    provider: &str,
    subscription_id: &str,
) -> Result<Option<StoredSubscription>, sqlx::Error> {
    let row = sqlx::query_as::<_, SubscriptionRow>(
        r#"
        SELECT customer_id, user_id, plan_key, lifecycle_state, access_until, last_event_at
        FROM billing_subscriptions
        WHERE provider = $1 AND subscription_id = $2
        FOR UPDATE
        "#,
    )
    .bind(provider)
    .bind(subscription_id)
    .fetch_optional(&mut **transaction)
    .await?;
    Ok(row.and_then(SubscriptionRow::into_subscription))
}

#[derive(sqlx::FromRow)]
struct SubscriptionRow {
    customer_id: String,
    user_id: String,
    plan_key: String,
    lifecycle_state: String,
    access_until: Option<DateTime<Utc>>,
    last_event_at: DateTime<Utc>,
}

impl SubscriptionRow {
    fn into_subscription(self) -> Option<StoredSubscription> {
        Some(StoredSubscription {
            customer_id: self.customer_id,
            user_id: self.user_id,
            plan: PlanKey::parse(&self.plan_key)?,
            lifecycle: SubscriptionLifecycle::parse(&self.lifecycle_state)?,
            access_until: self.access_until,
            last_event_at: self.last_event_at,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn event(
        provider: CommerceProvider,
        id: &str,
        subscription_id: &str,
        user_id: Option<&str>,
        plan: Option<PlanKey>,
        lifecycle: SubscriptionLifecycle,
        occurred_at: &str,
    ) -> SubscriptionEvent {
        SubscriptionEvent {
            provider,
            event_id: id.into(),
            event_type: "subscription.test".into(),
            occurred_at: occurred_at.parse().unwrap(),
            subscription_id: subscription_id.into(),
            customer_id: format!("customer-{subscription_id}"),
            establishes_binding: true,
            user_id: user_id.map(str::to_string),
            plan,
            lifecycle,
            access_until: None,
        }
    }

    #[tokio::test]
    async fn duplicate_and_stale_events_are_idempotent() {
        let store = CommerceStore::memory();
        let entitlements = EntitlementStore::memory();
        let current = event(
            CommerceProvider::Dodo,
            "evt-current",
            "sub-1",
            Some("user-1"),
            Some(PlanKey::Pro),
            SubscriptionLifecycle::Active,
            "2026-09-15T10:00:00Z",
        );
        assert_eq!(
            store.reconcile(&entitlements, &current).await.unwrap(),
            ReconcileOutcome::Applied
        );
        assert_eq!(
            store.reconcile(&entitlements, &current).await.unwrap(),
            ReconcileOutcome::Duplicate
        );
        let stale_upgrade = event(
            CommerceProvider::Dodo,
            "evt-stale",
            "sub-1",
            None,
            Some(PlanKey::Ultra),
            SubscriptionLifecycle::Active,
            "2026-09-15T09:00:00Z",
        );
        assert_eq!(
            store
                .reconcile(&entitlements, &stale_upgrade)
                .await
                .unwrap(),
            ReconcileOutcome::IgnoredStale
        );
        assert_eq!(entitlements.get("user-1").await.tier, Tier::Pro);
    }

    #[tokio::test]
    async fn equal_timestamps_fail_closed_but_allow_access_restriction() {
        let store = CommerceStore::memory();
        let entitlements = EntitlementStore::memory();
        let current = event(
            CommerceProvider::Dodo,
            "evt-current-equal",
            "sub-equal",
            Some("user-equal"),
            Some(PlanKey::Pro),
            SubscriptionLifecycle::Active,
            "2026-09-15T10:00:00Z",
        );
        store.reconcile(&entitlements, &current).await.unwrap();

        let upgrade = event(
            CommerceProvider::Dodo,
            "evt-upgrade-equal",
            "sub-equal",
            None,
            Some(PlanKey::Ultra),
            SubscriptionLifecycle::Active,
            "2026-09-15T10:00:00Z",
        );
        assert_eq!(
            store.reconcile(&entitlements, &upgrade).await.unwrap(),
            ReconcileOutcome::IgnoredStale
        );
        assert_eq!(entitlements.get("user-equal").await.tier, Tier::Pro);

        let ended = event(
            CommerceProvider::Dodo,
            "evt-ended-equal",
            "sub-equal",
            None,
            None,
            SubscriptionLifecycle::Ended,
            "2026-09-15T10:00:00Z",
        );
        assert_eq!(
            store.reconcile(&entitlements, &ended).await.unwrap(),
            ReconcileOutcome::Applied
        );
        assert_eq!(entitlements.get("user-equal").await.tier, Tier::Core);
    }

    #[tokio::test]
    async fn ending_one_provider_preserves_another_provider_grant() {
        let store = CommerceStore::memory();
        let entitlements = EntitlementStore::memory();
        let paddle = event(
            CommerceProvider::Paddle,
            "evt-paddle",
            "sub-paddle",
            Some("user-1"),
            Some(PlanKey::Pro),
            SubscriptionLifecycle::Active,
            "2026-09-15T08:00:00Z",
        );
        let dodo = event(
            CommerceProvider::Dodo,
            "evt-dodo",
            "sub-dodo",
            Some("user-1"),
            Some(PlanKey::Ultra),
            SubscriptionLifecycle::Active,
            "2026-09-15T09:00:00Z",
        );
        store.reconcile(&entitlements, &paddle).await.unwrap();
        store.reconcile(&entitlements, &dodo).await.unwrap();

        let ended = event(
            CommerceProvider::Dodo,
            "evt-dodo-ended",
            "sub-dodo",
            None,
            None,
            SubscriptionLifecycle::Ended,
            "2026-09-15T10:00:00Z",
        );
        store.reconcile(&entitlements, &ended).await.unwrap();
        assert_eq!(entitlements.get("user-1").await.tier, Tier::Pro);
    }

    #[tokio::test]
    async fn unbound_terminal_event_cannot_create_an_identity_binding() {
        let store = CommerceStore::memory();
        let entitlements = EntitlementStore::memory();
        let ended = event(
            CommerceProvider::Dodo,
            "evt-ended",
            "sub-unknown",
            Some("attacker"),
            Some(PlanKey::Ultra),
            SubscriptionLifecycle::Ended,
            "2026-09-15T10:00:00Z",
        );
        assert_eq!(
            store.reconcile(&entitlements, &ended).await.unwrap(),
            ReconcileOutcome::Rejected
        );
        assert_eq!(entitlements.get("attacker").await.tier, Tier::Core);
    }

    #[tokio::test]
    async fn later_active_event_cannot_establish_a_missing_binding() {
        let store = CommerceStore::memory();
        let entitlements = EntitlementStore::memory();
        let mut updated = event(
            CommerceProvider::Dodo,
            "evt-renewed",
            "sub-unknown",
            Some("user-1"),
            Some(PlanKey::Ultra),
            SubscriptionLifecycle::Active,
            "2026-09-15T10:00:00Z",
        );
        updated.establishes_binding = false;

        assert_eq!(
            store.reconcile(&entitlements, &updated).await.unwrap(),
            ReconcileOutcome::Rejected
        );
        assert_eq!(entitlements.get("user-1").await.tier, Tier::Core);
    }
}
