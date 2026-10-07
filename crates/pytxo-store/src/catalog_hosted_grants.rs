//! Durable, account-bound reconciliation state for experimental hosted routing.
//! This ledger is not send authority: callers must recheck the physical Store,
//! reviewed packet, local consent, remote session and Link grant before a send.

use chrono::Utc;
use pytxo_core::{PytxoError, Result};
use rusqlite::{params, Connection, OptionalExtension, Transaction, TransactionBehavior};

use crate::{Catalog, HostedAdvisorConsentReview};

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct HostedGrantIntent {
    pub domain_id: String,
    pub workspace_id: String,
    pub account_id: String,
    pub link_origin: String,
    pub recipient_identity: String,
    pub scope_digest: String,
    pub store_db_file_identity: String,
    pub consent_revision: u64,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum HostedGrantState {
    GrantPending,
    Enabled,
    RevokePending,
    Revoked,
}

impl HostedGrantState {
    fn parse(value: &str) -> Result<Self> {
        match value {
            "grant_pending" => Ok(Self::GrantPending),
            "enabled" => Ok(Self::Enabled),
            "revoke_pending" => Ok(Self::RevokePending),
            "revoked" => Ok(Self::Revoked),
            _ => Err(error("hosted grant state is invalid")),
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct HostedGrantBinding {
    pub intent: HostedGrantIntent,
    pub remote_revision: Option<u64>,
    pub state: HostedGrantState,
}

/// A response observed through the verified account session at the pinned Link
/// origin. Link's JSON supplies workspace/recipient/scope/revision/enabled;
/// the client supplies the session's verified account and request origin.
/// Callers must validate all fields before constructing this receipt.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct HostedRemoteGrantReceipt {
    pub workspace_id: String,
    pub account_id: String,
    pub link_origin: String,
    pub recipient_identity: String,
    pub scope_digest: String,
    pub revision: u64,
    pub enabled: bool,
}

type GrantRow = (
    String,
    String,
    String,
    String,
    String,
    String,
    i64,
    Option<i64>,
    String,
);

fn error(message: &str) -> PytxoError {
    PytxoError::Store(message.into())
}

fn valid_hex(value: &str, length: usize) -> bool {
    value.len() == length
        && value
            .bytes()
            .all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte))
}

fn valid_intent(intent: &HostedGrantIntent) -> bool {
    !intent.domain_id.is_empty()
        && valid_hex(&intent.workspace_id, 32)
        && intent.account_id.starts_with("user_")
        && intent.account_id.len() > 5
        && !intent.link_origin.is_empty()
        && !intent.recipient_identity.is_empty()
        && valid_hex(&intent.scope_digest, 64)
        && !intent.store_db_file_identity.is_empty()
        && intent.consent_revision > 0
}

fn checked_revision(revision: u64) -> Result<i64> {
    i64::try_from(revision).map_err(|_| error("hosted grant revision is too large"))
}

fn receipt_identity_matches(
    intent: &HostedGrantIntent,
    receipt: &HostedRemoteGrantReceipt,
) -> bool {
    receipt.workspace_id == intent.workspace_id
        && receipt.account_id == intent.account_id
        && receipt.link_origin == intent.link_origin
        && receipt.recipient_identity == intent.recipient_identity
        && valid_hex(&receipt.scope_digest, 64)
        && receipt.revision > 0
}

fn receipt_matches(intent: &HostedGrantIntent, receipt: &HostedRemoteGrantReceipt) -> bool {
    receipt_identity_matches(intent, receipt) && receipt.scope_digest == intent.scope_digest
}

fn now_ms() -> Result<i64> {
    let time = Utc::now().timestamp_millis();
    if time <= 0 {
        return Err(error("hosted grant clock is invalid"));
    }
    Ok(time)
}

fn load(conn: &Connection, domain_id: &str) -> Result<Option<HostedGrantBinding>> {
    let row: Option<GrantRow> = conn
        .query_row(
            "SELECT workspace_id, account_id, link_origin, recipient_identity, scope_digest,
                    store_db_file_identity, consent_revision, remote_revision, state
             FROM hosted_routing_grants WHERE domain_id = ?1",
            params![domain_id],
            |row| {
                Ok((
                    row.get(0)?,
                    row.get(1)?,
                    row.get(2)?,
                    row.get(3)?,
                    row.get(4)?,
                    row.get(5)?,
                    row.get(6)?,
                    row.get(7)?,
                    row.get(8)?,
                ))
            },
        )
        .optional()
        .map_err(|error| PytxoError::Store(error.to_string()))?;
    row.map(
        |(
            workspace_id,
            account_id,
            link_origin,
            recipient_identity,
            scope_digest,
            store_db_file_identity,
            consent_revision,
            remote_revision,
            state,
        )| {
            Ok(HostedGrantBinding {
                intent: HostedGrantIntent {
                    domain_id: domain_id.into(),
                    workspace_id,
                    account_id,
                    link_origin,
                    recipient_identity,
                    scope_digest,
                    store_db_file_identity,
                    consent_revision: u64::try_from(consent_revision)
                        .map_err(|_| error("hosted grant consent revision is invalid"))?,
                },
                remote_revision: remote_revision
                    .map(|revision| {
                        u64::try_from(revision)
                            .map_err(|_| error("hosted grant remote revision is invalid"))
                    })
                    .transpose()?,
                state: HostedGrantState::parse(&state)?,
            })
        },
    )
    .transpose()
}

fn current_review_matches(tx: &Transaction<'_>, intent: &HostedGrantIntent) -> Result<bool> {
    let workspace: Option<String> = tx
        .query_row(
            "SELECT workspace_id FROM hosted_workspace_ids WHERE domain_id = ?1",
            params![intent.domain_id],
            |row| row.get(0),
        )
        .optional()
        .map_err(|error| PytxoError::Store(error.to_string()))?;
    let pin: Option<String> = tx
        .query_row(
            "SELECT store_db_file_identity FROM routing_advisor_consent_stores WHERE domain_id = ?1",
            params![intent.domain_id],
            |row| row.get(0),
        )
        .optional()
        .map_err(|error| PytxoError::Store(error.to_string()))?;
    let review: Option<(String, i64, String, String)> = tx
        .query_row(
            "SELECT recipient_identity, consent_revision, scope_digest, store_db_file_identity
             FROM routing_hosted_advisor_consent_reviews WHERE domain_id = ?1",
            params![intent.domain_id],
            |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?, row.get(3)?)),
        )
        .optional()
        .map_err(|error| PytxoError::Store(error.to_string()))?;
    let fence: Option<(String, i64, bool, String)> = tx
        .query_row(
            "SELECT recipient_identity, consent_revision, enabled, store_db_file_identity
             FROM routing_hosted_advisor_consent_fences WHERE domain_id = ?1",
            params![intent.domain_id],
            |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?, row.get(3)?)),
        )
        .optional()
        .map_err(|error| PytxoError::Store(error.to_string()))?;
    let revision = checked_revision(intent.consent_revision)?;
    Ok(workspace.as_deref() == Some(intent.workspace_id.as_str())
        && pin.as_deref() == Some(intent.store_db_file_identity.as_str())
        && review.is_some_and(|(recipient, current, scope, identity)| {
            recipient == intent.recipient_identity
                && current == revision
                && scope == intent.scope_digest
                && identity == intent.store_db_file_identity
        })
        && fence.is_some_and(|(recipient, current, enabled, identity)| {
            recipient == intent.recipient_identity
                && current == revision
                && enabled
                && identity == intent.store_db_file_identity
        }))
}

fn revoke_fence_matches(tx: &Transaction<'_>, binding: &HostedGrantBinding) -> Result<bool> {
    let fence: Option<(String, i64, bool, String)> = tx
        .query_row(
            "SELECT recipient_identity, consent_revision, enabled, store_db_file_identity
             FROM routing_hosted_advisor_consent_fences WHERE domain_id = ?1",
            params![binding.intent.domain_id],
            |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?, row.get(3)?)),
        )
        .optional()
        .map_err(|error| PytxoError::Store(error.to_string()))?;
    Ok(
        fence.is_some_and(|(recipient, revision, enabled, identity)| {
            recipient == binding.intent.recipient_identity
                && revision > i64::try_from(binding.intent.consent_revision).unwrap_or(i64::MAX)
                && !enabled
                && identity == binding.intent.store_db_file_identity
        }),
    )
}

impl Catalog {
    pub fn hosted_grant(&self, domain_id: &str) -> Result<Option<HostedGrantBinding>> {
        load(&self.conn, domain_id)
    }

    /// Recheck the exact reviewed packet and locally confirmed Link grant in
    /// one Catalog transaction. The caller must also hold the original Store
    /// file guard, check Store consent, and obtain a current Link token/claim.
    /// A revoke after this read can still race an external request; Link's
    /// claim transaction is the remote ordering boundary.
    pub fn confirmed_hosted_grant_for_send(
        &self,
        intent: &HostedGrantIntent,
        review: &HostedAdvisorConsentReview,
    ) -> Result<HostedGrantBinding> {
        if !valid_intent(intent)
            || review.domain_id != intent.domain_id
            || review.recipient_identity != intent.recipient_identity
            || review.consent_revision != intent.consent_revision
            || review.scope_digest != intent.scope_digest
            || review.store_db_file_identity != intent.store_db_file_identity
            || review.draft_id.is_empty()
            || !valid_hex(&review.packet_digest, 64)
            || !valid_hex(&review.request_digest, 64)
        {
            return Err(error("hosted send differs from reviewed grant identity"));
        }
        let tx = Transaction::new_unchecked(&self.conn, TransactionBehavior::Immediate)
            .map_err(|error| PytxoError::Store(error.to_string()))?;
        if !current_review_matches(&tx, intent)? {
            return Err(error("hosted send is no longer locally consented"));
        }
        let actual: Option<(String, String, i64, String, String, String, String)> = tx
            .query_row(
                "SELECT recipient_identity, draft_id, consent_revision, scope_digest,
                        packet_digest, request_digest, store_db_file_identity
                 FROM routing_hosted_advisor_consent_reviews WHERE domain_id = ?1",
                [&intent.domain_id],
                |row| {
                    Ok((
                        row.get(0)?,
                        row.get(1)?,
                        row.get(2)?,
                        row.get(3)?,
                        row.get(4)?,
                        row.get(5)?,
                        row.get(6)?,
                    ))
                },
            )
            .optional()
            .map_err(|error| PytxoError::Store(error.to_string()))?;
        if actual
            != Some((
                review.recipient_identity.clone(),
                review.draft_id.clone(),
                checked_revision(review.consent_revision)?,
                review.scope_digest.clone(),
                review.packet_digest.clone(),
                review.request_digest.clone(),
                review.store_db_file_identity.clone(),
            ))
        {
            return Err(error("hosted send packet review changed"));
        }
        let binding =
            load(&tx, &intent.domain_id)?.ok_or_else(|| error("hosted send has no Link grant"))?;
        if binding.intent != *intent
            || binding.state != HostedGrantState::Enabled
            || binding.remote_revision.is_none_or(|revision| revision == 0)
        {
            return Err(error("hosted send Link grant is not confirmed"));
        }
        tx.commit()
            .map_err(|error| PytxoError::Store(error.to_string()))?;
        Ok(binding)
    }

    /// Persist intent before any remote POST. Replays must have the same
    /// account, recipient, Store and reviewed consent revision.
    pub fn begin_hosted_grant(&self, intent: &HostedGrantIntent) -> Result<HostedGrantBinding> {
        if !valid_intent(intent) {
            return Err(error("hosted grant intent is invalid"));
        }
        let tx = Transaction::new_unchecked(&self.conn, TransactionBehavior::Immediate)
            .map_err(|error| PytxoError::Store(error.to_string()))?;
        if !current_review_matches(&tx, intent)? {
            return Err(error("hosted grant differs from reviewed local consent"));
        }
        match load(&tx, &intent.domain_id)? {
            Some(existing) if existing.intent == *intent => {
                if !matches!(
                    existing.state,
                    HostedGrantState::GrantPending | HostedGrantState::Enabled
                ) {
                    return Err(error("hosted grant is pending revocation or revoked"));
                }
            }
            Some(existing)
                if existing.state == HostedGrantState::Revoked
                    && intent.consent_revision > existing.intent.consent_revision
                    && intent.workspace_id == existing.intent.workspace_id
                    && intent.account_id == existing.intent.account_id
                    && intent.link_origin == existing.intent.link_origin
                    && intent.recipient_identity == existing.intent.recipient_identity
                    && intent.scope_digest == existing.intent.scope_digest
                    && intent.store_db_file_identity == existing.intent.store_db_file_identity =>
            {
                tx.execute(
                    "UPDATE hosted_routing_grants SET account_id = ?2, link_origin = ?3,
                     recipient_identity = ?4, scope_digest = ?5, consent_revision = ?6,
                     state = 'grant_pending', updated_at_ms = ?7
                     WHERE domain_id = ?1 AND state = 'revoked'",
                    params![
                        intent.domain_id,
                        intent.account_id,
                        intent.link_origin,
                        intent.recipient_identity,
                        intent.scope_digest,
                        checked_revision(intent.consent_revision)?,
                        now_ms()?
                    ],
                )
                .map_err(|error| PytxoError::Store(error.to_string()))?;
            }
            Some(_) => return Err(error("hosted grant account or review changed")),
            None => {
                tx.execute(
                    "INSERT INTO hosted_routing_grants (
                        domain_id, workspace_id, account_id, link_origin, recipient_identity,
                        scope_digest, store_db_file_identity, consent_revision, remote_revision,
                        state, updated_at_ms
                     ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, NULL, 'grant_pending', ?9)",
                    params![
                        intent.domain_id,
                        intent.workspace_id,
                        intent.account_id,
                        intent.link_origin,
                        intent.recipient_identity,
                        intent.scope_digest,
                        intent.store_db_file_identity,
                        checked_revision(intent.consent_revision)?,
                        now_ms()?
                    ],
                )
                .map_err(|error| PytxoError::Store(error.to_string()))?;
            }
        }
        let binding = load(&tx, &intent.domain_id)?
            .ok_or_else(|| error("hosted grant intent was not retained"))?;
        tx.commit()
            .map_err(|error| PytxoError::Store(error.to_string()))?;
        Ok(binding)
    }

    /// The remote reply is evidence of grant state, never authority to send.
    pub fn confirm_hosted_grant(
        &self,
        intent: &HostedGrantIntent,
        receipt: &HostedRemoteGrantReceipt,
    ) -> Result<HostedGrantBinding> {
        if !receipt.enabled || !receipt_matches(intent, receipt) {
            return Err(error("hosted remote grant receipt differs from intent"));
        }
        let tx = Transaction::new_unchecked(&self.conn, TransactionBehavior::Immediate)
            .map_err(|error| PytxoError::Store(error.to_string()))?;
        if !current_review_matches(&tx, intent)? {
            return Err(error("hosted grant is no longer locally consented"));
        }
        let existing =
            load(&tx, &intent.domain_id)?.ok_or_else(|| error("hosted grant intent is absent"))?;
        if existing.intent != *intent {
            return Err(error("hosted grant account or review changed"));
        }
        match existing.state {
            HostedGrantState::GrantPending => {
                if existing
                    .remote_revision
                    .is_some_and(|old| receipt.revision <= old)
                {
                    return Err(error("hosted remote grant revision did not advance"));
                }
                tx.execute(
                    "UPDATE hosted_routing_grants SET remote_revision = ?2, state = 'enabled',
                     updated_at_ms = ?3 WHERE domain_id = ?1 AND state = 'grant_pending'",
                    params![
                        intent.domain_id,
                        checked_revision(receipt.revision)?,
                        now_ms()?
                    ],
                )
                .map_err(|error| PytxoError::Store(error.to_string()))?;
            }
            HostedGrantState::Enabled if existing.remote_revision == Some(receipt.revision) => {}
            _ => return Err(error("hosted grant cannot be confirmed from this state")),
        }
        let binding = load(&tx, &intent.domain_id)?
            .ok_or_else(|| error("hosted grant confirmation was not retained"))?;
        tx.commit()
            .map_err(|error| PytxoError::Store(error.to_string()))?;
        Ok(binding)
    }

    /// Local consent must be fenced first. A failed remote DELETE leaves this
    /// state durable so a later controller can reconcile with the same account.
    pub fn begin_hosted_grant_revoke(&self, domain_id: &str) -> Result<HostedGrantBinding> {
        let tx = Transaction::new_unchecked(&self.conn, TransactionBehavior::Immediate)
            .map_err(|error| PytxoError::Store(error.to_string()))?;
        let existing = load(&tx, domain_id)?.ok_or_else(|| error("hosted grant is absent"))?;
        if !revoke_fence_matches(&tx, &existing)? {
            return Err(error("hosted grant local revoke fence is absent"));
        }
        match existing.state {
            HostedGrantState::GrantPending | HostedGrantState::Enabled => {
                tx.execute(
                    "UPDATE hosted_routing_grants SET state = 'revoke_pending', updated_at_ms = ?2
                     WHERE domain_id = ?1 AND state IN ('grant_pending','enabled')",
                    params![domain_id, now_ms()?],
                )
                .map_err(|error| PytxoError::Store(error.to_string()))?;
            }
            HostedGrantState::RevokePending => {}
            HostedGrantState::Revoked => return Err(error("hosted grant is already revoked")),
        }
        let binding =
            load(&tx, domain_id)?.ok_or_else(|| error("hosted revoke intent was not retained"))?;
        tx.commit()
            .map_err(|error| PytxoError::Store(error.to_string()))?;
        Ok(binding)
    }

    /// Emergency, revoke-only path when the pinned workspace Store cannot be
    /// opened. Fence future hosted sends and retain remote DELETE intent in
    /// one Catalog transaction. The physical Store remains unverified and
    /// must be inspected before any later grant is enabled.
    pub fn begin_hosted_grant_revoke_without_store(
        &self,
        domain_id: &str,
        expected_revision: u64,
    ) -> Result<HostedGrantBinding> {
        let tx = Transaction::new_unchecked(&self.conn, TransactionBehavior::Immediate)
            .map_err(|error| PytxoError::Store(error.to_string()))?;
        let existing = load(&tx, domain_id)?.ok_or_else(|| error("hosted grant is absent"))?;
        if !matches!(
            existing.state,
            HostedGrantState::GrantPending | HostedGrantState::Enabled
        ) {
            return Err(error("hosted grant is not active for emergency revoke"));
        }
        let revision = checked_revision(expected_revision)?;
        let next = checked_revision(
            expected_revision
                .checked_add(1)
                .ok_or_else(|| error("hosted revoke revision overflow"))?,
        )?;
        let changed = tx
            .execute(
                "UPDATE routing_hosted_advisor_consent_fences
                 SET consent_revision = ?2, enabled = 0
                 WHERE domain_id = ?1 AND consent_revision = ?3 AND enabled = 1
                   AND recipient_identity = ?4 AND store_db_file_identity = ?5",
                params![
                    domain_id,
                    next,
                    revision,
                    existing.intent.recipient_identity,
                    existing.intent.store_db_file_identity
                ],
            )
            .map_err(|error| PytxoError::Store(error.to_string()))?;
        if changed != 1 {
            return Err(error(
                "hosted emergency revoke fence is stale or mismatched",
            ));
        }
        tx.execute(
            "UPDATE hosted_routing_grants SET state = 'revoke_pending', updated_at_ms = ?2
             WHERE domain_id = ?1 AND state IN ('grant_pending','enabled')",
            params![domain_id, now_ms()?],
        )
        .map_err(|error| PytxoError::Store(error.to_string()))?;
        let binding = load(&tx, domain_id)?
            .ok_or_else(|| error("hosted emergency revoke intent was not retained"))?;
        if !revoke_fence_matches(&tx, &binding)? || binding.state != HostedGrantState::RevokePending
        {
            return Err(error("hosted emergency revoke did not fence the grant"));
        }
        tx.commit()
            .map_err(|error| PytxoError::Store(error.to_string()))?;
        Ok(binding)
    }

    pub fn confirm_hosted_grant_revoked(
        &self,
        domain_id: &str,
        receipt: &HostedRemoteGrantReceipt,
    ) -> Result<HostedGrantBinding> {
        let tx = Transaction::new_unchecked(&self.conn, TransactionBehavior::Immediate)
            .map_err(|error| PytxoError::Store(error.to_string()))?;
        let existing = load(&tx, domain_id)?.ok_or_else(|| error("hosted grant is absent"))?;
        if !revoke_fence_matches(&tx, &existing)? {
            return Err(error("hosted grant local revoke fence is absent"));
        }
        if receipt.enabled || !receipt_identity_matches(&existing.intent, receipt) {
            return Err(error("hosted remote revoke receipt differs from intent"));
        }
        if existing
            .remote_revision
            .is_some_and(|old| receipt.revision < old)
        {
            return Err(error("hosted remote revoke revision regressed"));
        }
        match existing.state {
            HostedGrantState::RevokePending => {
                tx.execute(
                    "UPDATE hosted_routing_grants SET remote_revision = ?2,
                     state = 'revoked', updated_at_ms = ?3
                     WHERE domain_id = ?1 AND state = 'revoke_pending'",
                    params![domain_id, checked_revision(receipt.revision)?, now_ms()?],
                )
                .map_err(|error| PytxoError::Store(error.to_string()))?;
            }
            HostedGrantState::Revoked if existing.remote_revision == Some(receipt.revision) => {}
            _ => {
                return Err(error(
                    "hosted grant cannot be marked revoked from this state",
                ))
            }
        }
        let binding =
            load(&tx, domain_id)?.ok_or_else(|| error("hosted revoke receipt was not retained"))?;
        tx.commit()
            .map_err(|error| PytxoError::Store(error.to_string()))?;
        Ok(binding)
    }

    pub fn unrevoked_hosted_grants(&self) -> Result<Vec<HostedGrantBinding>> {
        let mut statement = self
            .conn
            .prepare("SELECT domain_id FROM hosted_routing_grants WHERE state != 'revoked' ORDER BY domain_id")
            .map_err(|error| PytxoError::Store(error.to_string()))?;
        let domains = statement
            .query_map([], |row| row.get::<_, String>(0))
            .map_err(|error| PytxoError::Store(error.to_string()))?
            .collect::<std::result::Result<Vec<_>, _>>()
            .map_err(|error| PytxoError::Store(error.to_string()))?;
        domains
            .iter()
            .map(|domain| {
                self.hosted_grant(domain)?
                    .ok_or_else(|| error("listed hosted grant is absent"))
            })
            .collect()
    }
}
