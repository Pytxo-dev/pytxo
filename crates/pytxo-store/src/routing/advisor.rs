//! Local disclosure consent and the client-side one-send journal. These rows
//! authorize no model choice, worker launch, spending, or repository Apply.
//! Only a trusted user action may call the consent setter; there is no caller
//! from repository configuration or the current routing fixture.

use rusqlite::{params, Connection, OptionalExtension};

use super::*;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RoutingAdvisorConsent {
    pub revision: u64,
    pub enabled: bool,
    pub scope_digest: Option<Digest>,
    pub updated_at_ms: u64,
}

const HOSTED_RECIPIENT_V1: &str = "pytxo-hosted-routing/typesafe-systemone/v1";

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum AdvisorSendPhase {
    Prepared,
    SendingMayHaveHappened,
    Completed,
    Uncertain,
    LateReceipt,
}

impl AdvisorSendPhase {
    fn as_str(self) -> &'static str {
        match self {
            Self::Prepared => "prepared",
            Self::SendingMayHaveHappened => "sending_may_have_happened",
            Self::Completed => "completed",
            Self::Uncertain => "uncertain",
            Self::LateReceipt => "late_receipt",
        }
    }

    fn parse(value: &str) -> Result<Self> {
        match value {
            "prepared" => Ok(Self::Prepared),
            "sending_may_have_happened" => Ok(Self::SendingMayHaveHappened),
            "completed" => Ok(Self::Completed),
            "uncertain" => Ok(Self::Uncertain),
            "late_receipt" => Ok(Self::LateReceipt),
            _ => fail("invalid routed advisor send phase"),
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RoutingAdvisorRequest {
    pub scope: RoutingScope,
    pub task_id: TaskId,
    pub request_id: AdviceRequestId,
    /// NULL only for the historical no-network fixture journal.
    pub recipient_identity: Option<String>,
    pub scope_digest: Option<Digest>,
    pub ordinal: u32,
    pub policy_digest: Digest,
    pub packet_digest: Digest,
    pub consent_revision: u64,
    pub task_revision: u64,
    pub task_state_revision: u64,
    pub authorization_revision: u64,
    pub cancel_epoch: u64,
    pub phase: AdvisorSendPhase,
    pub result_digest: Option<Digest>,
    pub created_at_ms: u64,
    pub updated_at_ms: u64,
}

pub(super) fn load_consent(conn: &Connection, domain: &DomainId) -> Result<RoutingAdvisorConsent> {
    let row: Option<(u64, bool, Option<String>, u64)> = conn
        .query_row(
            "SELECT revision,enabled,scope_digest,updated_at_ms FROM routing_advisor_consent WHERE domain_id=?1",
            [&domain.0],
            |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?, row.get(3)?)),
        )
        .optional()
        .map_err(err)?;
    let consent = match row {
        Some((revision, enabled, scope_digest, updated_at_ms)) => RoutingAdvisorConsent {
            revision,
            enabled,
            scope_digest: scope_digest.map(Digest),
            updated_at_ms,
        },
        None => RoutingAdvisorConsent {
            revision: 0,
            enabled: false,
            scope_digest: None,
            updated_at_ms: 0,
        },
    };
    require(
        consent.enabled == consent.scope_digest.is_some()
            && consent.scope_digest.as_ref().is_none_or(Digest::is_valid),
        "invalid advisor consent scope",
    )?;
    Ok(consent)
}

pub(super) fn load_hosted_consent(
    conn: &Connection,
    domain: &DomainId,
    recipient: &str,
) -> Result<RoutingAdvisorConsent> {
    require(
        !domain.0.trim().is_empty() && !recipient.trim().is_empty(),
        "empty hosted advisor consent identity",
    )?;
    let row: Option<(u64, bool, Option<String>, u64)> = conn
        .query_row(
            "SELECT revision,enabled,scope_digest,updated_at_ms FROM routing_hosted_advisor_consent WHERE domain_id=?1 AND recipient_identity=?2",
            params![domain.0, recipient],
            |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?, row.get(3)?)),
        )
        .optional()
        .map_err(err)?;
    let consent = match row {
        Some((revision, enabled, scope_digest, updated_at_ms)) => RoutingAdvisorConsent {
            revision,
            enabled,
            scope_digest: scope_digest.map(Digest),
            updated_at_ms,
        },
        None => RoutingAdvisorConsent {
            revision: 0,
            enabled: false,
            scope_digest: None,
            updated_at_ms: 0,
        },
    };
    require(
        consent.enabled == consent.scope_digest.is_some()
            && consent.scope_digest.as_ref().is_none_or(Digest::is_valid),
        "invalid hosted advisor consent scope",
    )?;
    Ok(consent)
}

pub(super) fn load_request(
    conn: &Connection,
    request_id: &AdviceRequestId,
) -> Result<Option<RoutingAdvisorRequest>> {
    let row = conn
        .query_row(
            "SELECT domain_id,run_id,task_id,ordinal,policy_digest,packet_digest,consent_revision,task_revision,task_state_revision,authorization_revision,cancel_epoch,phase,result_digest,created_at_ms,updated_at_ms,recipient_identity,scope_digest FROM routing_advisor_requests WHERE request_id=?1",
            [&request_id.0],
            |row| {
                Ok((
                    row.get::<_, String>(0)?,
                    row.get::<_, String>(1)?,
                    row.get::<_, String>(2)?,
                    row.get::<_, u32>(3)?,
                    row.get::<_, String>(4)?,
                    row.get::<_, String>(5)?,
                    row.get::<_, u64>(6)?,
                    row.get::<_, u64>(7)?,
                    row.get::<_, u64>(8)?,
                    row.get::<_, u64>(9)?,
                    row.get::<_, u64>(10)?,
                    row.get::<_, String>(11)?,
                    row.get::<_, Option<String>>(12)?,
                    row.get::<_, u64>(13)?,
                    row.get::<_, u64>(14)?,
                    row.get::<_, Option<String>>(15)?,
                    row.get::<_, Option<String>>(16)?,
                ))
            },
        )
        .optional()
        .map_err(err)?;
    row.map(
        |(
            domain,
            run,
            task,
            ordinal,
            policy,
            packet,
            consent,
            task_revision,
            task_state_revision,
            authorization_revision,
            cancel_epoch,
            phase,
            result,
            created,
            updated,
            recipient,
            scope,
        )| {
            let phase = AdvisorSendPhase::parse(&phase)?;
            let result_digest = result.map(Digest);
            let scope_digest = scope.map(Digest);
            require(
                Digest(policy.clone()).is_valid()
                    && Digest(packet.clone()).is_valid()
                    && (recipient.is_none() && scope_digest.is_none()
                        || recipient.as_deref() == Some(HOSTED_RECIPIENT_V1)
                            && scope_digest.as_ref().is_some_and(Digest::is_valid))
                    && result_digest.as_ref().is_none_or(Digest::is_valid)
                    && (matches!(
                        phase,
                        AdvisorSendPhase::Completed | AdvisorSendPhase::LateReceipt
                    ) == result_digest.is_some())
                    && created <= updated,
                "invalid routed advisor request row",
            )?;
            Ok(RoutingAdvisorRequest {
                scope: RoutingScope {
                    domain_id: DomainId(domain),
                    run_id: RunId(run),
                },
                task_id: TaskId(task),
                request_id: request_id.clone(),
                recipient_identity: recipient,
                scope_digest,
                ordinal,
                policy_digest: Digest(policy),
                packet_digest: Digest(packet),
                consent_revision: consent,
                task_revision,
                task_state_revision,
                authorization_revision,
                cancel_epoch,
                phase,
                result_digest,
                created_at_ms: created,
                updated_at_ms: updated,
            })
        },
    )
    .transpose()
}

pub(super) fn load_requests_for_task(
    conn: &Connection,
    scope: &RoutingScope,
    task_id: &TaskId,
) -> Result<Vec<RoutingAdvisorRequest>> {
    let mut statement = conn
        .prepare(
            "SELECT request_id FROM routing_advisor_requests WHERE domain_id=?1 AND run_id=?2 AND task_id=?3 ORDER BY ordinal",
        )
        .map_err(err)?;
    let ids = statement
        .query_map(
            params![scope.domain_id.0, scope.run_id.0, task_id.0],
            |row| row.get::<_, String>(0),
        )
        .map_err(err)?
        .collect::<std::result::Result<Vec<_>, _>>()
        .map_err(err)?;
    ids.into_iter()
        .map(|id| {
            let request = load_request(conn, &AdviceRequestId(id))?
                .ok_or_else(|| error("benchmark advisor request disappeared"))?;
            require(
                request.scope == *scope && request.task_id == *task_id,
                "benchmark advisor request identity changed",
            )?;
            Ok(request)
        })
        .collect()
}

fn require_same_request(
    existing: &RoutingAdvisorRequest,
    expected: &RoutingAdvisorRequest,
) -> Result<()> {
    require(
        existing.scope == expected.scope
            && existing.task_id == expected.task_id
            && existing.request_id == expected.request_id
            && existing.recipient_identity == expected.recipient_identity
            && existing.scope_digest == expected.scope_digest
            && existing.ordinal == expected.ordinal
            && existing.policy_digest == expected.policy_digest
            && existing.packet_digest == expected.packet_digest
            && existing.consent_revision == expected.consent_revision
            && existing.task_revision == expected.task_revision
            && existing.task_state_revision == expected.task_state_revision
            && existing.authorization_revision == expected.authorization_revision
            && existing.cancel_epoch == expected.cancel_epoch,
        "routed advisor request identity conflict",
    )
}

fn advisor_mode_authorized(history: &RoutingHistory) -> bool {
    match history.mission.policy.mode {
        RoutingMode::Shadow => true,
        RoutingMode::Live => {
            history.mission.authorization.live_advice_authorized
                && history
                    .mission
                    .policy
                    .evaluated_manifest_digest
                    .as_ref()
                    .is_some_and(Digest::is_valid)
        }
        RoutingMode::Disabled | RoutingMode::Rules => false,
    }
}

fn hosted_request_id_is_current(request_id: &AdviceRequestId, now_ms: u64) -> bool {
    let Ok(uuid) = uuid::Uuid::parse_str(&request_id.0) else {
        return false;
    };
    if uuid.get_version() != Some(uuid::Version::SortRand)
        || uuid.hyphenated().to_string() != request_id.0
    {
        return false;
    }
    let created_ms = uuid.as_bytes()[..6]
        .iter()
        .fold(0u64, |value, byte| (value << 8) | u64::from(*byte));
    created_ms <= now_ms.saturating_add(300_000) && now_ms <= created_ms.saturating_add(86_400_000)
}

impl PytxoStore {
    /// Recovery may treat a freshly registered mission as pre-admission only
    /// when no advisor send was prepared for this exact scope.
    pub fn has_routing_advisor_requests(&self, scope: &RoutingScope) -> Result<bool> {
        self.conn
            .query_row(
                "SELECT EXISTS(SELECT 1 FROM routing_advisor_requests WHERE domain_id=?1 AND run_id=?2)",
                params![scope.domain_id.0, scope.run_id.0],
                |row| row.get(0),
            )
            .map_err(err)
    }

    /// Recovery inspects every request in the exact Run, including requests
    /// for an unexpected task. It must never infer no-send from a task-only
    /// query that silently omits another durable request.
    pub fn routing_advisor_requests_for_recovery(
        &self,
        scope: &RoutingScope,
    ) -> Result<Vec<RoutingAdvisorRequest>> {
        let mut statement = self
            .conn
            .prepare(
                "SELECT request_id FROM routing_advisor_requests WHERE domain_id=?1 AND run_id=?2 ORDER BY task_id,ordinal",
            )
            .map_err(err)?;
        let ids = statement
            .query_map(params![scope.domain_id.0, scope.run_id.0], |row| {
                row.get::<_, String>(0)
            })
            .map_err(err)?
            .collect::<std::result::Result<Vec<_>, _>>()
            .map_err(err)?;
        ids.into_iter()
            .map(|id| {
                let request = load_request(&self.conn, &AdviceRequestId(id))?
                    .ok_or_else(|| error("recovery advisor request disappeared"))?;
                require(request.scope == *scope, "recovery advisor scope changed")?;
                Ok(request)
            })
            .collect()
    }

    /// Only dead-controller recovery calls this after proving no attempt or
    /// worker exists. A prepared row may not have been sent; `uncertain` is a
    /// conservative terminal no-retry state for that ambiguity.
    pub fn abandon_prepared_routing_advisor_request(
        &self,
        scope: &RoutingScope,
        request_id: &AdviceRequestId,
        now_ms: u64,
    ) -> Result<RoutingAdvisorRequest> {
        let tx = immediate(&self.conn)?;
        let mut request =
            load_request(&tx, request_id)?.ok_or_else(|| error("advisor request missing"))?;
        require(request.scope == *scope, "advisor request scope mismatch")?;
        if request.phase == AdvisorSendPhase::Uncertain {
            return Ok(request);
        }
        require(
            request.phase == AdvisorSendPhase::Prepared && request.result_digest.is_none(),
            "only an unsent prepared advisor request can be abandoned",
        )?;
        let updated_at_ms = now_ms.max(request.updated_at_ms);
        require(
            tx.execute(
                "UPDATE routing_advisor_requests SET phase='uncertain',updated_at_ms=?1 WHERE request_id=?2 AND phase='prepared'",
                params![updated_at_ms, request_id.0],
            )
            .map_err(err)?
                == 1,
            "prepared advisor abandonment CAS conflict",
        )?;
        tx.commit().map_err(err)?;
        request.phase = AdvisorSendPhase::Uncertain;
        request.updated_at_ms = updated_at_ms;
        Ok(request)
    }

    /// Default is disabled. This setter must be reached only from an explicit,
    /// trusted workspace action, never from project TOML, worker output or Jev.
    pub fn set_routing_advisor_consent(
        &self,
        domain: &DomainId,
        expected_revision: u64,
        enabled: bool,
        scope_digest: Option<Digest>,
        now_ms: u64,
    ) -> Result<RoutingAdvisorConsent> {
        require(!domain.0.trim().is_empty(), "empty advisor consent domain")?;
        require(
            enabled == scope_digest.is_some() && scope_digest.as_ref().is_none_or(Digest::is_valid),
            "advisor consent requires exact disclosure scope",
        )?;
        let tx = immediate(&self.conn)?;
        let current = load_consent(&tx, domain)?;
        require(
            current.revision == expected_revision && (!enabled || now_ms >= current.updated_at_ms),
            "stale advisor consent revision",
        )?;
        // A clock rollback must never prevent revocation. Keep the display
        // timestamp monotone while the CAS revision carries authority.
        let updated_at_ms = now_ms.max(current.updated_at_ms);
        let next = current
            .revision
            .checked_add(1)
            .ok_or_else(|| error("advisor consent revision overflow"))?;
        if current.revision == 0 {
            tx.execute(
                "INSERT INTO routing_advisor_consent(domain_id,revision,enabled,scope_digest,updated_at_ms) VALUES (?1,?2,?3,?4,?5)",
                params![domain.0, next, enabled, scope_digest.as_ref().map(|digest| &digest.0), updated_at_ms],
            ).map_err(err)?;
        } else {
            require(
                tx.execute(
                    "UPDATE routing_advisor_consent SET revision=?1,enabled=?2,scope_digest=?3,updated_at_ms=?4 WHERE domain_id=?5 AND revision=?6",
                    params![next, enabled, scope_digest.as_ref().map(|digest| &digest.0), updated_at_ms, domain.0, expected_revision],
                ).map_err(err)? == 1,
                "advisor consent CAS conflict",
            )?;
        }
        tx.commit().map_err(err)?;
        Ok(RoutingAdvisorConsent {
            revision: next,
            enabled,
            scope_digest,
            updated_at_ms,
        })
    }

    pub fn routing_advisor_consent(&self, domain: &DomainId) -> Result<RoutingAdvisorConsent> {
        load_consent(&self.conn, domain)
    }

    /// Hosted disclosure uses its own revision and exact recipient; enabling
    /// it cannot mutate or inherit the local no-network fixture grant.
    pub fn set_routing_hosted_advisor_consent(
        &self,
        domain: &DomainId,
        recipient: &str,
        expected_revision: u64,
        enabled: bool,
        scope_digest: Option<Digest>,
        now_ms: u64,
    ) -> Result<RoutingAdvisorConsent> {
        require(
            !domain.0.trim().is_empty() && recipient == HOSTED_RECIPIENT_V1,
            "unsupported hosted advisor recipient",
        )?;
        require(
            enabled == scope_digest.is_some() && scope_digest.as_ref().is_none_or(Digest::is_valid),
            "hosted advisor consent requires exact disclosure scope",
        )?;
        let tx = immediate(&self.conn)?;
        let current = load_hosted_consent(&tx, domain, recipient)?;
        require(
            current.revision == expected_revision && (!enabled || now_ms >= current.updated_at_ms),
            "stale hosted advisor consent revision",
        )?;
        let updated_at_ms = now_ms.max(current.updated_at_ms);
        let next = current
            .revision
            .checked_add(1)
            .ok_or_else(|| error("hosted advisor consent revision overflow"))?;
        if current.revision == 0 {
            tx.execute(
                "INSERT INTO routing_hosted_advisor_consent(domain_id,recipient_identity,revision,enabled,scope_digest,updated_at_ms) VALUES (?1,?2,?3,?4,?5,?6)",
                params![domain.0, recipient, next, enabled, scope_digest.as_ref().map(|digest| &digest.0), updated_at_ms],
            ).map_err(err)?;
        } else {
            require(
                tx.execute(
                    "UPDATE routing_hosted_advisor_consent SET revision=?1,enabled=?2,scope_digest=?3,updated_at_ms=?4 WHERE domain_id=?5 AND recipient_identity=?6 AND revision=?7",
                    params![next, enabled, scope_digest.as_ref().map(|digest| &digest.0), updated_at_ms, domain.0, recipient, expected_revision],
                ).map_err(err)? == 1,
                "hosted advisor consent CAS conflict",
            )?;
        }
        tx.commit().map_err(err)?;
        Ok(RoutingAdvisorConsent {
            revision: next,
            enabled,
            scope_digest,
            updated_at_ms,
        })
    }

    pub fn routing_hosted_advisor_consent(
        &self,
        domain: &DomainId,
        recipient: &str,
    ) -> Result<RoutingAdvisorConsent> {
        load_hosted_consent(&self.conn, domain, recipient)
    }

    /// Persist the exact context before a future controller may attempt a send.
    /// This method performs no network operation and stores no packet bytes.
    pub fn prepare_routing_advisor_request(
        &self,
        scope: &RoutingScope,
        task_id: &TaskId,
        facts: &RoutingFacts,
    ) -> Result<RoutingAdvisorRequest> {
        self.prepare_routing_advisor_request_for(scope, task_id, facts, None)
    }

    /// Prepared hosted requests are still local journal rows, not permission
    /// to call a provider. The caller must separately pass the send fence.
    pub fn prepare_routing_hosted_advisor_request(
        &self,
        scope: &RoutingScope,
        task_id: &TaskId,
        facts: &RoutingFacts,
        recipient: &str,
    ) -> Result<RoutingAdvisorRequest> {
        require(
            recipient == HOSTED_RECIPIENT_V1,
            "unsupported hosted advisor recipient",
        )?;
        self.prepare_routing_advisor_request_for(scope, task_id, facts, Some(recipient))
    }

    fn prepare_routing_advisor_request_for(
        &self,
        scope: &RoutingScope,
        task_id: &TaskId,
        facts: &RoutingFacts,
        recipient: Option<&str>,
    ) -> Result<RoutingAdvisorRequest> {
        let tx = immediate(&self.conn)?;
        let history = require_history(&tx, scope)?;
        let consent = match recipient {
            Some(recipient) => load_hosted_consent(&tx, &scope.domain_id, recipient)?,
            None => load_consent(&tx, &scope.domain_id)?,
        };
        require(
            consent.enabled
                && history.mission.policy.advisor_recipient.as_deref() == recipient
                && (recipient.is_none() || history.mission.policy.mode == RoutingMode::Shadow)
                && consent.revision == history.mission.authorization.consent_revision
                && consent.scope_digest == history.mission.policy.disclosure_scope_digest
                && facts.now_ms >= consent.updated_at_ms
                && advisor_mode_authorized(&history),
            "hosted advisor is not consented for this reviewed mode",
        )?;
        let request_id = facts
            .advice_request_id
            .as_ref()
            .ok_or_else(|| error("missing advisor request identity"))?;
        let packet_digest = facts
            .packet_digest
            .as_ref()
            .ok_or_else(|| error("missing advisor packet digest"))?;
        require(
            !request_id.0.trim().is_empty()
                && request_id.0.len() <= 128
                && recipient.is_none_or(|_| hosted_request_id_is_current(request_id, facts.now_ms))
                && packet_digest.is_valid(),
            "invalid advisor request identity",
        )?;
        let snapshot = snapshot(&tx, &history, task_id, facts)?;
        let catalog = build_eligible_catalog(&snapshot, &candidates(&history.mission, facts)?);
        // Match Store's final reservation-aware preview, not only Core's
        // policy result. An advisor send has no value when this task cannot
        // reserve its frozen per-attempt budget.
        let rules = decision(&history, &snapshot, facts, None)?;
        let everyday = &history.mission.policy.everyday;
        let strong = &history.mission.policy.strong;
        require(
            snapshot.scope_valid
                && !snapshot.cancelled
                && recipient.is_none_or(|_| snapshot.has_recordable_advice_context())
                && snapshot.next_ordinal == 1
                && snapshot.ownership_resolved
                && history.mission.authorization.consent_revision > 0
                && rules.reason == RouteReason::StrongDefault
                && rules.selection == RouteSelection::Selected(strong.clone())
                && catalog
                    .eligible
                    .iter()
                    .filter(|candidate| candidate.target() == *everyday)
                    .count()
                    == 1
                && catalog
                    .eligible
                    .iter()
                    .filter(|candidate| candidate.target() == *strong)
                    .count()
                    == 1,
            "advisor is not needed for an eligible ambiguous initial decision",
        )?;
        let expected = RoutingAdvisorRequest {
            scope: scope.clone(),
            task_id: task_id.clone(),
            request_id: request_id.clone(),
            recipient_identity: recipient.map(str::to_owned),
            scope_digest: recipient.and(consent.scope_digest.clone()),
            ordinal: snapshot.next_ordinal,
            policy_digest: rules.policy_digest,
            packet_digest: packet_digest.clone(),
            consent_revision: consent.revision,
            task_revision: snapshot.task_revision,
            task_state_revision: snapshot.task_state_revision,
            authorization_revision: snapshot.authorization_revision,
            cancel_epoch: snapshot.cancel_epoch,
            phase: AdvisorSendPhase::Prepared,
            result_digest: None,
            created_at_ms: facts.now_ms,
            updated_at_ms: facts.now_ms,
        };
        if let Some(existing) = load_request(&tx, request_id)? {
            require_same_request(&existing, &expected)?;
            return Ok(existing);
        }
        tx.execute(
            "INSERT INTO routing_advisor_requests(request_id,domain_id,run_id,task_id,ordinal,policy_digest,packet_digest,consent_revision,task_revision,task_state_revision,authorization_revision,cancel_epoch,phase,result_digest,created_at_ms,updated_at_ms,recipient_identity,scope_digest) VALUES (?1,?2,?3,?4,?5,?6,?7,?8,?9,?10,?11,?12,'prepared',NULL,?13,?13,?14,?15)",
            params![request_id.0,scope.domain_id.0,scope.run_id.0,task_id.0,expected.ordinal,expected.policy_digest.0,expected.packet_digest.0,expected.consent_revision,expected.task_revision,expected.task_state_revision,expected.authorization_revision,expected.cancel_epoch,facts.now_ms,expected.recipient_identity,expected.scope_digest.as_ref().map(|digest| &digest.0)],
        ).map_err(err)?;
        tx.commit().map_err(err)?;
        Ok(expected)
    }

    /// Returns true only for the one caller that committed the may-send mark.
    /// False means no second send is permitted, including after a crash.
    pub fn mark_routing_advisor_send_may_have_happened(
        &self,
        scope: &RoutingScope,
        request_id: &AdviceRequestId,
        facts: &RoutingFacts,
        now_ms: u64,
    ) -> Result<bool> {
        let tx = immediate(&self.conn)?;
        let request =
            load_request(&tx, request_id)?.ok_or_else(|| error("advisor request missing"))?;
        require(request.scope == *scope, "advisor request scope mismatch")?;
        if request.phase != AdvisorSendPhase::Prepared {
            return Ok(false);
        }
        let history = require_history(&tx, scope)?;
        let task = find_task(&history, &request.task_id)?;
        let consent = match request.recipient_identity.as_deref() {
            Some(recipient) => load_hosted_consent(&tx, &scope.domain_id, recipient)?,
            None => load_consent(&tx, &scope.domain_id)?,
        };
        require(
            facts.now_ms == now_ms
                && history.mission.policy.advisor_recipient == request.recipient_identity
                && (request.recipient_identity.is_none()
                    || history.mission.policy.mode == RoutingMode::Shadow)
                && (request.recipient_identity.is_none() && request.scope_digest.is_none()
                    || request.recipient_identity.as_deref() == Some(HOSTED_RECIPIENT_V1)
                        && request.scope_digest == history.mission.policy.disclosure_scope_digest
                        && hosted_request_id_is_current(request_id, now_ms))
                && facts.advice_request_id.as_ref() == Some(request_id)
                && facts.packet_digest.as_ref() == Some(&request.packet_digest),
            "advisor send facts changed since preparation",
        )?;
        let snapshot = snapshot(&tx, &history, &request.task_id, facts)?;
        let catalog = build_eligible_catalog(&snapshot, &candidates(&history.mission, facts)?);
        let rules = decision(&history, &snapshot, facts, None)?;
        let everyday = &history.mission.policy.everyday;
        let strong = &history.mission.policy.strong;
        let run_status: String = tx
            .query_row(
                "SELECT status FROM runs WHERE id=?1",
                [&scope.run_id.0],
                |row| row.get(0),
            )
            .map_err(err)?;
        require(
            consent.enabled
                && consent.revision == request.consent_revision
                && consent.scope_digest == history.mission.policy.disclosure_scope_digest
                && now_ms >= consent.updated_at_ms
                && history.mission.authorization.consent_revision == consent.revision
                && history.mission.authorization.revision == request.authorization_revision
                && advisor_mode_authorized(&history)
                && history.mission.policy.digest().map_err(err)? == request.policy_digest
                && history.cancel_epoch == request.cancel_epoch
                && !history.cancelled
                && task.registration.contract.revision == request.task_revision
                && task.revision == request.task_state_revision
                && task.next_ordinal == request.ordinal
                && task.state == TaskRoutingState::Ready
                && task.winner.is_none()
                && snapshot.scope_valid
                && (request.recipient_identity.is_none()
                    || snapshot.has_recordable_advice_context())
                && snapshot.ownership_resolved
                && rules.reason == RouteReason::StrongDefault
                && rules.selection == RouteSelection::Selected(strong.clone())
                && rules.policy_digest == request.policy_digest
                && catalog
                    .eligible
                    .iter()
                    .filter(|candidate| candidate.target() == *everyday)
                    .count()
                    == 1
                && catalog
                    .eligible
                    .iter()
                    .filter(|candidate| candidate.target() == *strong)
                    .count()
                    == 1
                && matches!(run_status.as_str(), "starting" | "running")
                && now_ms >= request.created_at_ms
                && now_ms < history.mission.authorization.limits.deadline_ms,
            "advisor send context is no longer current",
        )?;
        require(
            tx.execute(
                "UPDATE routing_advisor_requests SET phase='sending_may_have_happened',updated_at_ms=?1 WHERE request_id=?2 AND phase='prepared'",
                params![now_ms, request_id.0],
            ).map_err(err)? == 1,
            "advisor send mark CAS conflict",
        )?;
        tx.commit().map_err(err)?;
        Ok(true)
    }

    /// A response digest is for a complete observed response. `None` means
    /// sending may have billed upstream but its result/usage is uncertain. A
    /// later attributable receipt may settle evidence, never revive routing.
    pub fn settle_routing_advisor_request(
        &self,
        scope: &RoutingScope,
        request_id: &AdviceRequestId,
        result_digest: Option<Digest>,
        now_ms: u64,
    ) -> Result<RoutingAdvisorRequest> {
        let tx = immediate(&self.conn)?;
        let mut request =
            load_request(&tx, request_id)?.ok_or_else(|| error("advisor request missing"))?;
        require(
            request.scope == *scope && now_ms >= request.updated_at_ms,
            "advisor settlement identity mismatch",
        )?;
        require(
            result_digest.as_ref().is_none_or(Digest::is_valid),
            "invalid advisor result digest",
        )?;
        let phase = match (request.phase, result_digest.is_some()) {
            (AdvisorSendPhase::Uncertain | AdvisorSendPhase::LateReceipt, true) => {
                AdvisorSendPhase::LateReceipt
            }
            (_, true) => AdvisorSendPhase::Completed,
            (_, false) => AdvisorSendPhase::Uncertain,
        };
        if request.phase == phase && request.result_digest == result_digest {
            return Ok(request);
        }
        require(
            request.phase == AdvisorSendPhase::SendingMayHaveHappened
                || (request.phase == AdvisorSendPhase::Uncertain
                    && phase == AdvisorSendPhase::LateReceipt),
            "advisor send already settled or never started",
        )?;
        require(
            tx.execute(
                "UPDATE routing_advisor_requests SET phase=?1,result_digest=?2,updated_at_ms=?3 WHERE request_id=?4 AND phase=?5",
                params![phase.as_str(), result_digest.as_ref().map(|digest| &digest.0), now_ms, request_id.0, request.phase.as_str()],
            ).map_err(err)? == 1,
            "advisor settlement CAS conflict",
        )?;
        tx.commit().map_err(err)?;
        request.phase = phase;
        request.result_digest = result_digest;
        request.updated_at_ms = now_ms;
        Ok(request)
    }

    pub fn routing_advisor_request(
        &self,
        scope: &RoutingScope,
        request_id: &AdviceRequestId,
    ) -> Result<Option<RoutingAdvisorRequest>> {
        let request = load_request(&self.conn, request_id)?;
        if let Some(request) = &request {
            require(request.scope == *scope, "advisor request scope mismatch")?;
        }
        Ok(request)
    }
}
