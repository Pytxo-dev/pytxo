//! Optional, experimental account return for sponsored routing only. This
//! credential never enters Ultra's runtime session or worker environments.

use std::sync::Mutex;
use std::time::Duration;
use std::{io::Read, path::Path};

use base64::engine::general_purpose::URL_SAFE_NO_PAD;
use base64::Engine;
use chrono::{DateTime, Utc};
use keyring::Entry;
use pytxo_orchestrate::{
    preview_reviewed_hosted_advisor_packet, read_experimental_hosted_advisor_local_consent,
    revoke_experimental_hosted_advisor_local_consent,
};
use pytxo_planner::advisor::{hosted_scope_digest, HOSTED_RECIPIENT};
use pytxo_runner::FileIdentityGuard;
use pytxo_store::{
    Catalog, HostedAdvisorConsentReview, HostedGrantBinding, HostedGrantIntent, HostedGrantState,
    HostedRemoteGrantReceipt,
};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use tauri::{AppHandle, Emitter};
use tauri_plugin_shell::ShellExt;

use crate::ipc_auth::{focus_main_window, AUTH_CHANGED_EVENT, AUTH_ERROR_EVENT};
use crate::ipc_error::{map_io_err, map_orch_err, map_store_err, IpcResult, PytxoIpcError};
use crate::ipc_routing_hosted_http::HostedHttpClient;

const SERVICE: &str = "com.pytxo.reality-deck";
const PENDING_ACCOUNT: &str = "routing-bridge-pending";
const SESSION_ACCOUNT: &str = "routing-desktop-session";
const DEFAULT_SITE_ORIGIN: &str = "https://pytxo.com";
const DEFAULT_LINK_ORIGIN: &str = "https://link.pytxo.com";
const SCOPE: &str = "routing:grants:v1";
const RECOVERY_SCOPE: &str = "routing:revoke:v1";
const HOSTED_GRANT_EXPERIMENT: &str = "PYTXO_HOSTED_GRANT_EXPERIMENT";
// Serializes browser starts against an exchange already in flight. A second
// account return cannot finish out of order and overwrite the first session.
static PENDING_LOCK: Mutex<bool> = Mutex::new(false);

struct InFlightReset;

impl Drop for InFlightReset {
    fn drop(&mut self) {
        *PENDING_LOCK
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner()) = false;
    }
}

fn begin_account_mutation() -> IpcResult<InFlightReset> {
    let mut in_flight = PENDING_LOCK.lock().map_err(map_io_err)?;
    if *in_flight {
        return Err(auth_error(
            "Another Routing account operation is still completing.",
        ));
    }
    *in_flight = true;
    Ok(InFlightReset)
}

#[derive(Serialize, Deserialize)]
struct Pending {
    state: String,
    verifier: String,
    link_origin: String,
    created_at: i64,
    #[serde(default)]
    recovery_only: bool,
}

#[derive(Serialize, Deserialize)]
struct Session {
    token: String,
    account_id: String,
    expires_at: DateTime<Utc>,
    scope: String,
    link_origin: String,
}

#[derive(Deserialize)]
struct ExchangeReply {
    token: String,
    account_id: String,
    expires_at: DateTime<Utc>,
    scope: String,
}

#[derive(Deserialize)]
struct SessionStatusReply {
    account_id: String,
    expires_at: DateTime<Utc>,
    scope: String,
}

#[derive(Clone, Deserialize)]
#[serde(deny_unknown_fields)]
struct GrantReply {
    workspace_id: String,
    recipient_identity: String,
    scope_digest: String,
    revision: u64,
    enabled: bool,
}

fn checked_grant_receipt(
    session: &Session,
    intent: &HostedGrantIntent,
    reply: GrantReply,
) -> IpcResult<HostedRemoteGrantReceipt> {
    let valid_scope = reply.scope_digest.len() == 64
        && reply
            .scope_digest
            .bytes()
            .all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte));
    if session.account_id != intent.account_id
        || session.link_origin != intent.link_origin
        || reply.workspace_id != intent.workspace_id
        || reply.recipient_identity != intent.recipient_identity
        || !valid_scope
        || reply.revision == 0
    {
        return Err(auth_error(
            "Routing grant reply does not match the verified account and workspace.",
        ));
    }
    Ok(HostedRemoteGrantReceipt {
        workspace_id: reply.workspace_id,
        account_id: session.account_id.clone(),
        link_origin: session.link_origin.clone(),
        recipient_identity: reply.recipient_identity,
        scope_digest: reply.scope_digest,
        revision: reply.revision,
        enabled: reply.enabled,
    })
}

fn read_grant_reply(response: ureq::Response) -> IpcResult<GrantReply> {
    if response.status() != 200 {
        return Err(auth_error(
            "Pytxo Link returned an unexpected grant status.",
        ));
    }
    let mut body = Vec::new();
    response
        .into_reader()
        .take(4097)
        .read_to_end(&mut body)
        .map_err(map_io_err)?;
    if body.len() > 4096 {
        return Err(auth_error("Pytxo Link grant reply was too large."));
    }
    serde_json::from_slice(&body)
        .map_err(|_| auth_error("Pytxo Link returned an invalid grant reply."))
}

fn checked_grant_session() -> IpcResult<Session> {
    if !bridge_enabled()
        || std::env::var(HOSTED_GRANT_EXPERIMENT).as_deref() != Ok("1")
        || std::env::var("PYTXO_EXPERIMENTAL_ROUTED_HOSTED_SHADOW_REVIEW").as_deref() != Ok("1")
    {
        return Err(auth_error(
            "Experimental hosted Routing grants are disabled.",
        ));
    }
    let session = read_session()?;
    if session.scope != SCOPE {
        return Err(auth_error(
            "This Routing connection can only revoke existing grants. Finish revocation, then reconnect normally before creating a new grant.",
        ));
    }
    if session.link_origin
        != configured_origin("PYTXO_ROUTING_BRIDGE_LINK_ORIGIN", DEFAULT_LINK_ORIGIN)?
        || verify_remote(&session) != RemoteStatus::Verified
    {
        return Err(auth_error(
            "Routing account could not be freshly verified at its configured Link origin.",
        ));
    }
    Ok(session)
}

fn hosted_http_client_from_verified_session(
    session: &Session,
    binding: &HostedGrantBinding,
) -> IpcResult<HostedHttpClient> {
    if session.scope != SCOPE
        || session.account_id != binding.intent.account_id
        || session.link_origin != binding.intent.link_origin
        || binding.state != HostedGrantState::Enabled
        || binding.intent.recipient_identity != HOSTED_RECIPIENT
        || binding.intent.scope_digest != hosted_scope_digest().0
    {
        return Err(auth_error(
            "Hosted routing client differs from the verified account or reviewed grant.",
        ));
    }
    let revision = binding
        .remote_revision
        .ok_or_else(|| auth_error("Hosted routing grant has no confirmed revision."))?;
    HostedHttpClient::new(
        session.account_id.clone(),
        session.token.clone(),
        session.expires_at,
        session.link_origin.clone(),
        "https://proxy.pytxo.com".into(),
        binding.intent.workspace_id.clone(),
        revision,
    )
    .map_err(|_| auth_error("Hosted routing client configuration is invalid."))
}

/// A future explicit Shadow dispatch can obtain this client before claiming
/// its reviewed Flow. Neither normal Flow dispatch nor the UI calls it today.
#[allow(dead_code, reason = "hosted Shadow HTTP dispatch remains disabled")]
pub(crate) fn checked_hosted_http_client_for_review(
    catalog: &Catalog,
    draft_id: &str,
) -> IpcResult<HostedHttpClient> {
    let session = checked_grant_session()?;
    let preview =
        preview_reviewed_hosted_advisor_packet(catalog, draft_id).map_err(map_orch_err)?;
    let binding = catalog
        .hosted_grant(&preview.domain_id)
        .map_err(map_store_err)?
        .ok_or_else(|| auth_error("No confirmed hosted grant exists for this workspace."))?;
    let reviewed = HostedAdvisorConsentReview {
        domain_id: preview.domain_id,
        recipient_identity: preview.recipient_identity,
        draft_id: draft_id.into(),
        consent_revision: preview.reviewed_consent_revision,
        scope_digest: preview.scope_digest.0,
        packet_digest: preview.packet_digest.0,
        request_digest: preview.request_digest.0,
        store_db_file_identity: preview.store_db_file_identity,
    };
    catalog
        .confirmed_hosted_grant_for_send(&binding.intent, &reviewed)
        .map_err(map_store_err)?;
    let remote = link_grant_status(&session, &binding.intent)?
        .ok_or_else(|| auth_error("The hosted Link grant is absent."))?;
    if !remote.enabled
        || remote.revision != binding.remote_revision.unwrap_or(0)
        || remote.scope_digest != binding.intent.scope_digest
    {
        return Err(auth_error(
            "Hosted Link grant changed after the local review.",
        ));
    }
    hosted_http_client_from_verified_session(&session, &binding)
}

fn checked_revoke_session() -> IpcResult<Session> {
    // Turning an experiment off must never remove the ability to revoke a
    // previously recorded grant. Its original account and Link origin remain
    // pinned in the durable binding and are checked by the caller.
    let session = read_session()?;
    if verify_remote(&session) != RemoteStatus::Verified {
        return Err(auth_error(
            "Reconnect the original Routing account to finish hosted revocation.",
        ));
    }
    Ok(session)
}

fn link_grant_status(
    session: &Session,
    intent: &HostedGrantIntent,
) -> IpcResult<Option<HostedRemoteGrantReceipt>> {
    let endpoint = format!(
        "{}/v1/routing/workspace-grants/{}",
        session.link_origin, intent.workspace_id
    );
    let response = ureq::AgentBuilder::new()
        .timeout(Duration::from_secs(4))
        .redirects(0)
        .build()
        .get(&endpoint)
        .set("Authorization", &format!("Bearer {}", session.token))
        .call();
    match response {
        Ok(response) => {
            checked_grant_receipt(session, intent, read_grant_reply(response)?).map(Some)
        }
        Err(ureq::Error::Status(404, _)) => Ok(None),
        _ => Err(auth_error(
            "Pytxo Link grant status is unavailable; local state was retained.",
        )),
    }
}

fn link_grant_write(
    session: &Session,
    intent: &HostedGrantIntent,
    expected_revision: u64,
    enable: bool,
) -> IpcResult<HostedRemoteGrantReceipt> {
    let endpoint = format!("{}/v1/routing/workspace-grants", session.link_origin);
    let agent = ureq::AgentBuilder::new()
        .timeout(Duration::from_secs(4))
        .redirects(0)
        .build();
    let request = if enable {
        agent.post(&endpoint)
    } else {
        agent.delete(&endpoint)
    }
    .set("Authorization", &format!("Bearer {}", session.token))
    .set("Content-Type", "application/json");
    let body = if enable {
        serde_json::json!({
            "workspace_id": intent.workspace_id,
            "recipient_identity": intent.recipient_identity,
            "scope_digest": intent.scope_digest,
            "expected_revision": expected_revision,
        })
    } else {
        serde_json::json!({
            "workspace_id": intent.workspace_id,
            "expected_revision": expected_revision,
        })
    };
    let response = request.send_string(&body.to_string()).map_err(|_| {
        auth_error("Pytxo Link grant change is uncertain; local reconciliation state was retained.")
    })?;
    checked_grant_receipt(session, intent, read_grant_reply(response)?)
}

#[derive(Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum RemoteStatus {
    Absent,
    Verified,
    Unverified,
    Revoked,
}

#[derive(Serialize)]
pub struct RoutingAccountStatus {
    pub bridge_available: bool,
    pub credential_present: bool,
    pub account_id: Option<String>,
    pub expires_at: Option<DateTime<Utc>>,
    pub remote_status: RemoteStatus,
    pub recovery_only: bool,
}

#[derive(Serialize)]
pub struct RoutingDisconnectResult {
    pub locally_cleared: bool,
    pub remote_revoked: bool,
}

#[derive(Serialize)]
pub struct RoutingHostedGrantStatus {
    domain_id: String,
    workspace_id: String,
    account_id: String,
    link_origin: String,
    recipient_identity: String,
    scope_digest: String,
    state: &'static str,
    remote_revision: Option<u64>,
}

fn grant_status(binding: HostedGrantBinding) -> RoutingHostedGrantStatus {
    let state = match binding.state {
        HostedGrantState::GrantPending => "grant_pending",
        HostedGrantState::Enabled => "enabled",
        HostedGrantState::RevokePending => "revoke_pending",
        HostedGrantState::Revoked => "revoked",
    };
    RoutingHostedGrantStatus {
        domain_id: binding.intent.domain_id,
        workspace_id: binding.intent.workspace_id,
        account_id: binding.intent.account_id,
        link_origin: binding.intent.link_origin,
        recipient_identity: binding.intent.recipient_identity,
        scope_digest: binding.intent.scope_digest,
        state,
        remote_revision: binding.remote_revision,
    }
}

fn session_matches_unrevoked_grants(session: &Session, grants: &[HostedGrantBinding]) -> bool {
    grants.iter().all(|binding| {
        binding.intent.account_id == session.account_id
            && binding.intent.link_origin == session.link_origin
    })
}

fn auth_error(message: &str) -> PytxoIpcError {
    PytxoIpcError::new("routing_account", message)
}

fn random_b64() -> IpcResult<String> {
    let mut bytes = [0u8; 32];
    getrandom::getrandom(&mut bytes).map_err(map_io_err)?;
    Ok(URL_SAFE_NO_PAD.encode(bytes))
}

fn valid_b64_32(value: &str) -> bool {
    value.len() == 43
        && URL_SAFE_NO_PAD
            .decode(value)
            .ok()
            .is_some_and(|bytes| bytes.len() == 32 && URL_SAFE_NO_PAD.encode(bytes) == value)
}

fn checked_origin(value: &str) -> IpcResult<String> {
    let parsed =
        tauri::Url::parse(value).map_err(|_| auth_error("Invalid Routing bridge origin."))?;
    let secure = parsed.scheme() == "https"
        || (cfg!(debug_assertions)
            && parsed.scheme() == "http"
            && matches!(parsed.host_str(), Some("127.0.0.1" | "localhost")));
    if !secure
        || parsed.host_str().is_none()
        || !matches!(parsed.path(), "" | "/")
        || parsed.query().is_some()
        || parsed.fragment().is_some()
        || parsed.username() != ""
        || parsed.password().is_some()
    {
        return Err(auth_error("Invalid Routing bridge origin."));
    }
    Ok(parsed.as_str().trim_end_matches('/').to_string())
}

fn configured_origin(variable: &str, fallback: &str) -> IpcResult<String> {
    let value = std::env::var(variable).unwrap_or_else(|_| fallback.to_string());
    checked_origin(&value)
}

fn bridge_enabled_value(value: Option<&str>) -> bool {
    value == Some("1")
}

fn bridge_enabled() -> bool {
    bridge_enabled_value(
        std::env::var("PYTXO_ROUTING_BRIDGE_EXPERIMENT")
            .ok()
            .as_deref(),
    )
}

fn account_link_origin_for_start(
    bridge_available: bool,
    grants: &[HostedGrantBinding],
) -> IpcResult<String> {
    if let Some(first) = grants.first() {
        if grants.iter().any(|grant| {
            grant.intent.account_id != first.intent.account_id
                || grant.intent.link_origin != first.intent.link_origin
        }) {
            return Err(auth_error(
                "Recorded hosted grants have different accounts or Link origins; recovery needs operator review.",
            ));
        }
        return checked_origin(&first.intent.link_origin);
    }
    if !bridge_available {
        return Err(auth_error(
            "Experimental Routing account connection is unavailable.",
        ));
    }
    configured_origin("PYTXO_ROUTING_BRIDGE_LINK_ORIGIN", DEFAULT_LINK_ORIGIN)
}

fn account_return_url(site_origin: &str, state: &str, challenge: &str, recovery: bool) -> String {
    let recovery = if recovery { "&deck_recovery=1" } else { "" };
    format!(
        "{site_origin}/account?deck_callback=pytxo-deck&state={state}&code_challenge={challenge}{recovery}"
    )
}

fn account_link_recovery_only(
    bridge_available: bool,
    grants: &[HostedGrantBinding],
    recovery_requested: bool,
) -> bool {
    recovery_requested
        || (!bridge_available && !grants.is_empty())
        || (grants
            .iter()
            .any(|grant| grant.state == HostedGrantState::RevokePending)
            && !grants
                .iter()
                .any(|grant| grant.state == HostedGrantState::Enabled))
}

fn session_entry() -> IpcResult<Entry> {
    Entry::new(SERVICE, SESSION_ACCOUNT).map_err(map_io_err)
}

fn pending_entry() -> IpcResult<Entry> {
    Entry::new(SERVICE, PENDING_ACCOUNT).map_err(map_io_err)
}

fn read_session() -> IpcResult<Session> {
    let serialized = session_entry()?.get_password().map_err(map_io_err)?;
    parse_session(&serialized)
}

fn parse_session(serialized: &str) -> IpcResult<Session> {
    let session: Session = serde_json::from_str(serialized).map_err(map_io_err)?;
    if !session.token.starts_with("pds1_")
        || !valid_b64_32(&session.token[5..])
        || !session.account_id.starts_with("user_")
        || !matches!(session.scope.as_str(), SCOPE | RECOVERY_SCOPE)
        || checked_origin(&session.link_origin).is_err()
        || session.expires_at <= Utc::now()
    {
        return Err(auth_error(
            "Routing account credential is absent or expired.",
        ));
    }
    Ok(session)
}

fn credential_present(result: Result<String, keyring::Error>) -> bool {
    !matches!(result, Err(keyring::Error::NoEntry))
}

fn stored_credential_present() -> bool {
    // A keyring error is uncertain, not proof that no credential exists. Keep
    // Disconnect available so the operator can retry clearing it.
    session_entry()
        .map(|entry| credential_present(entry.get_password()))
        .unwrap_or(true)
}

fn status_matches_local(reply: &SessionStatusReply, session: &Session) -> bool {
    reply.account_id == session.account_id
        && reply.scope == session.scope
        && reply.expires_at == session.expires_at
        && reply.expires_at > Utc::now()
}

fn verify_remote(session: &Session) -> RemoteStatus {
    let endpoint = format!("{}/v1/routing/desktop-session", session.link_origin);
    match ureq::AgentBuilder::new()
        .timeout(Duration::from_secs(3))
        .redirects(0)
        .build()
        .get(&endpoint)
        .set("Authorization", &format!("Bearer {}", session.token))
        .call()
    {
        Ok(response) if response.status() == 200 => {
            match response.into_json::<SessionStatusReply>() {
                Ok(reply) if status_matches_local(&reply, session) => RemoteStatus::Verified,
                _ => RemoteStatus::Unverified,
            }
        }
        Err(ureq::Error::Status(401, _)) => RemoteStatus::Revoked,
        _ => RemoteStatus::Unverified,
    }
}

fn revoke_remote_session(session: &Session) -> bool {
    let endpoint = format!("{}/v1/routing/desktop-session", session.link_origin);
    ureq::AgentBuilder::new()
        .timeout(Duration::from_secs(5))
        .redirects(0)
        .build()
        .delete(&endpoint)
        .set("Authorization", &format!("Bearer {}", session.token))
        .call()
        .is_ok_and(|response| response.status() == 204)
}

fn existing_session_blocks_new_link(
    session: &Session,
    recovery_requested: bool,
    bridge_available: bool,
) -> bool {
    if session.scope == RECOVERY_SCOPE && !recovery_requested && bridge_available {
        return false;
    }
    verify_remote(session) != RemoteStatus::Revoked
}

#[tauri::command]
pub async fn routing_account_status() -> RoutingAccountStatus {
    let bridge_available = bridge_enabled();
    let recovery_needed = Catalog::open_default()
        .and_then(|catalog| catalog.unrevoked_hosted_grants())
        .is_ok_and(|grants| !grants.is_empty());
    match read_session() {
        Ok(session) => {
            let verification_session = Session {
                token: session.token.clone(),
                account_id: session.account_id.clone(),
                expires_at: session.expires_at,
                scope: session.scope.clone(),
                link_origin: session.link_origin.clone(),
            };
            let remote_status = if bridge_available || recovery_needed {
                tauri::async_runtime::spawn_blocking(move || verify_remote(&verification_session))
                    .await
                    .unwrap_or(RemoteStatus::Unverified)
            } else {
                RemoteStatus::Unverified
            };
            RoutingAccountStatus {
                bridge_available,
                credential_present: true,
                account_id: Some(session.account_id),
                expires_at: Some(session.expires_at),
                remote_status,
                recovery_only: session.scope == RECOVERY_SCOPE,
            }
        }
        Err(_) => {
            let present = stored_credential_present();
            RoutingAccountStatus {
                bridge_available,
                credential_present: present,
                account_id: None,
                expires_at: None,
                remote_status: if present {
                    RemoteStatus::Unverified
                } else {
                    RemoteStatus::Absent
                },
                recovery_only: false,
            }
        }
    }
}

fn prepared_hosted_intent(
    catalog: &Catalog,
    session: &Session,
    domain_id: &str,
) -> IpcResult<(HostedGrantIntent, FileIdentityGuard)> {
    let review = catalog
        .hosted_advisor_consent_review(domain_id)
        .map_err(map_store_err)?
        .ok_or_else(|| auth_error("A reviewed hosted Shadow packet is required."))?;
    let locator = catalog
        .routing_advisor_consent_store(domain_id)
        .map_err(map_store_err)?
        .ok_or_else(|| auth_error("The original hosted consent Store is unavailable."))?;
    let guard =
        FileIdentityGuard::acquire(Path::new(&locator.store_db_path)).map_err(map_orch_err)?;
    if guard.identity() != locator.store_db_file_identity
        || review.store_db_file_identity != locator.store_db_file_identity
    {
        return Err(auth_error("The reviewed hosted consent Store changed."));
    }
    let preview =
        preview_reviewed_hosted_advisor_packet(catalog, &review.draft_id).map_err(map_orch_err)?;
    let local =
        read_experimental_hosted_advisor_local_consent(catalog, domain_id).map_err(map_orch_err)?;
    if !local.enabled
        || !local.current_scope
        || !preview.recordable_shadow_context
        || local.revision != review.consent_revision
        || preview.domain_id != domain_id
        || preview.reviewed_consent_revision != review.consent_revision
        || preview.recipient_identity != review.recipient_identity
        || preview.scope_digest.0 != review.scope_digest
        || preview.store_db_file_identity != locator.store_db_file_identity
    {
        return Err(auth_error(
            "Hosted routing differs from the current reviewed local consent.",
        ));
    }
    let workspace_id = catalog
        .ensure_hosted_workspace_id(
            domain_id,
            &locator.store_db_path,
            &locator.store_db_file_identity,
        )
        .map_err(map_store_err)?;
    Ok((
        HostedGrantIntent {
            domain_id: domain_id.into(),
            workspace_id,
            account_id: session.account_id.clone(),
            link_origin: session.link_origin.clone(),
            recipient_identity: review.recipient_identity,
            scope_digest: review.scope_digest,
            store_db_file_identity: locator.store_db_file_identity,
            consent_revision: review.consent_revision,
        },
        guard,
    ))
}

fn confirm_remote_enabled_or_revoke(
    catalog: &Catalog,
    intent: &HostedGrantIntent,
    remote: &HostedRemoteGrantReceipt,
    cleanup: impl FnOnce() -> IpcResult<RoutingHostedGrantStatus>,
) -> IpcResult<RoutingHostedGrantStatus> {
    match catalog.confirm_hosted_grant(intent, remote) {
        Ok(binding) => Ok(grant_status(binding)),
        Err(error) => {
            let binding = catalog
                .hosted_grant(&intent.domain_id)
                .map_err(map_store_err)?;
            if binding.as_ref().is_some_and(|binding| {
                binding.intent == *intent && binding.state == HostedGrantState::RevokePending
            }) {
                return match cleanup() {
                    Ok(_) => Err(auth_error(
                        "Hosted consent was revoked while the Link grant was in flight; the remote grant was revoked.",
                    )),
                    Err(_) => Err(auth_error(
                        "Hosted consent was revoked while the Link grant was in flight; remote revocation is pending. Reconnect the original Routing account to finish it.",
                    )),
                };
            }
            Err(map_store_err(error))
        }
    }
}

fn enable_hosted_grant_blocking(domain_id: &str) -> IpcResult<RoutingHostedGrantStatus> {
    let session = checked_grant_session()?;
    let catalog = Catalog::open_default().map_err(map_store_err)?;
    let (intent, _store_guard) = prepared_hosted_intent(&catalog, &session, domain_id)?;
    let binding = catalog.begin_hosted_grant(&intent).map_err(map_store_err)?;
    let remote = link_grant_status(&session, &intent)?;
    let expected = match remote {
        Some(remote) => {
            if binding
                .remote_revision
                .is_some_and(|old| remote.revision < old)
            {
                return Err(auth_error(
                    "Link grant revision regressed; local state was retained.",
                ));
            }
            if remote.enabled {
                if remote.scope_digest != intent.scope_digest {
                    return Err(auth_error(
                        "Link grant scope differs from the reviewed packet.",
                    ));
                }
                return confirm_remote_enabled_or_revoke(&catalog, &intent, &remote, || {
                    revoke_hosted_grant_blocking(domain_id, intent.consent_revision)
                });
            }
            if binding.state == HostedGrantState::Enabled {
                return Err(auth_error(
                    "Link grant was disabled; a new reviewed opt-in is required.",
                ));
            }
            remote.revision
        }
        None => {
            if binding.remote_revision.is_some() || binding.state == HostedGrantState::Enabled {
                return Err(auth_error(
                    "A previously confirmed Link grant is absent; local state was retained.",
                ));
            }
            0
        }
    };
    // A GET followed by POST is a new external action. Re-read the physical
    // Store and reviewed packet while the original file remains pinned.
    let (fresh_intent, _fresh_guard) = prepared_hosted_intent(&catalog, &session, domain_id)?;
    if fresh_intent != intent {
        return Err(auth_error(
            "Hosted local consent changed before the Link grant request.",
        ));
    }
    let remote = link_grant_write(&session, &intent, expected, true)?;
    if !remote.enabled || remote.scope_digest != intent.scope_digest {
        return Err(auth_error(
            "Link did not confirm the reviewed hosted grant.",
        ));
    }
    confirm_remote_enabled_or_revoke(&catalog, &intent, &remote, || {
        revoke_hosted_grant_blocking(domain_id, intent.consent_revision)
    })
}

fn revoke_hosted_grant_blocking(
    domain_id: &str,
    expected_revision: u64,
) -> IpcResult<RoutingHostedGrantStatus> {
    let catalog = Catalog::open_default().map_err(map_store_err)?;
    let binding = catalog
        .hosted_grant(domain_id)
        .map_err(map_store_err)?
        .ok_or_else(|| auth_error("No hosted Link grant is recorded for this workspace."))?;
    if binding.state == HostedGrantState::Revoked {
        return Ok(grant_status(binding));
    }
    let fence = catalog
        .hosted_advisor_consent_fence(domain_id)
        .map_err(map_store_err)?
        .ok_or_else(|| auth_error("Hosted consent has no durable local fence."))?;
    if fence.enabled {
        match read_experimental_hosted_advisor_local_consent(&catalog, domain_id) {
            Ok(local) => {
                let current = local.enabled && local.revision == fence.consent_revision;
                let committed_without_fence =
                    !local.enabled && local.revision == fence.consent_revision.saturating_add(1);
                if !(current && expected_revision == fence.consent_revision
                    || committed_without_fence
                        && (expected_revision == local.revision
                            || expected_revision == fence.consent_revision))
                {
                    return Err(auth_error(
                        "Hosted local consent revision changed; inspect it before revoking.",
                    ));
                }
                revoke_experimental_hosted_advisor_local_consent(
                    &catalog,
                    domain_id,
                    fence.consent_revision,
                )
                .map_err(map_orch_err)?;
            }
            Err(_) => {
                // A removed or unreadable pinned Store must not strand a Link
                // grant. This revoke-only transaction fences Catalog sends
                // before any remote DELETE. Zero is reserved for the recovery
                // UI when no Store revision can be read; Catalog then CASes
                // against its own durable fence revision.
                let recovery_revision = if expected_revision == 0 {
                    fence.consent_revision
                } else {
                    expected_revision
                };
                catalog
                    .begin_hosted_grant_revoke_without_store(domain_id, recovery_revision)
                    .map_err(map_store_err)?;
            }
        }
    }
    let pending = catalog
        .begin_hosted_grant_revoke(domain_id)
        .map_err(map_store_err)?;
    // A failed session or remote DELETE cannot roll back the local fence.
    let session = checked_revoke_session()?;
    if session.account_id != pending.intent.account_id
        || session.link_origin != pending.intent.link_origin
    {
        return Err(auth_error(
            "Reconnect the original Routing account and Link origin to finish revocation.",
        ));
    }
    let remote = link_grant_status(&session, &pending.intent)?;
    let expected = match remote {
        Some(remote) => {
            if pending
                .remote_revision
                .is_some_and(|old| remote.revision < old)
            {
                return Err(auth_error(
                    "Link grant revision regressed; revocation remains pending.",
                ));
            }
            if !remote.enabled {
                return catalog
                    .confirm_hosted_grant_revoked(domain_id, &remote)
                    .map(grant_status)
                    .map_err(map_store_err);
            }
            remote.revision
        }
        None => {
            if pending.remote_revision.is_some() {
                return Err(auth_error(
                    "A previously confirmed Link grant is absent; revocation remains pending.",
                ));
            }
            0
        }
    };
    let remote = link_grant_write(&session, &pending.intent, expected, false)?;
    catalog
        .confirm_hosted_grant_revoked(domain_id, &remote)
        .map(grant_status)
        .map_err(map_store_err)
}

#[tauri::command]
pub fn routing_hosted_grant_status(
    domain_id: String,
) -> IpcResult<Option<RoutingHostedGrantStatus>> {
    Catalog::open_default()
        .and_then(|catalog| catalog.hosted_grant(&domain_id))
        .map(|binding| binding.map(grant_status))
        .map_err(map_store_err)
}

#[tauri::command]
pub fn routing_hosted_grants() -> IpcResult<Vec<RoutingHostedGrantStatus>> {
    Catalog::open_default()
        .and_then(|catalog| catalog.unrevoked_hosted_grants())
        .map(|bindings| bindings.into_iter().map(grant_status).collect())
        .map_err(map_store_err)
}

#[tauri::command]
pub async fn routing_hosted_grant_enable(domain_id: String) -> IpcResult<RoutingHostedGrantStatus> {
    let _reset = begin_account_mutation()?;
    tauri::async_runtime::spawn_blocking(move || enable_hosted_grant_blocking(&domain_id))
        .await
        .map_err(map_orch_err)?
}

#[tauri::command]
pub async fn routing_hosted_grant_revoke(
    domain_id: String,
    expected_revision: u64,
) -> IpcResult<RoutingHostedGrantStatus> {
    let _reset = begin_account_mutation()?;
    tauri::async_runtime::spawn_blocking(move || {
        revoke_hosted_grant_blocking(&domain_id, expected_revision)
    })
    .await
    .map_err(map_orch_err)?
}

#[tauri::command]
pub async fn routing_account_disconnect(app: AppHandle) -> IpcResult<RoutingDisconnectResult> {
    let _reset = begin_account_mutation()?;
    if !Catalog::open_default()
        .and_then(|catalog| catalog.unrevoked_hosted_grants())
        .map_err(map_store_err)?
        .is_empty()
    {
        return Err(auth_error("Revoke and reconcile experimental hosted workspace grants before disconnecting this Routing account."));
    }
    // A pending browser return cannot reconnect this Desktop after disconnect.
    match pending_entry()?.delete_credential() {
        Ok(()) | Err(keyring::Error::NoEntry) => {}
        Err(error) => return Err(map_io_err(error)),
    }
    let session = read_session().ok();
    let revoked = if let Some(session) = session {
        tauri::async_runtime::spawn_blocking(move || revoke_remote_session(&session))
            .await
            .unwrap_or(false)
    } else {
        false
    };
    match session_entry()?.delete_credential() {
        Ok(()) | Err(keyring::Error::NoEntry) => {}
        Err(error) => return Err(map_io_err(error)),
    }
    // Credential deletion is the outcome; an unavailable UI event must not
    // report it as failed after the keyring has already been cleared.
    let _ = app.emit(AUTH_CHANGED_EVENT, ());
    Ok(RoutingDisconnectResult {
        locally_cleared: true,
        remote_revoked: revoked,
    })
}

#[allow(deprecated)] // Existing Desktop shell plugin owns external-browser opening.
pub fn begin_account_link(app: &AppHandle, recovery_requested: bool) -> IpcResult<()> {
    let grants = Catalog::open_default()
        .and_then(|catalog| catalog.unrevoked_hosted_grants())
        .map_err(map_store_err)?;
    if recovery_requested && grants.is_empty() {
        return Err(auth_error("No hosted workspace grant needs revocation."));
    }
    let link_origin = account_link_origin_for_start(bridge_enabled(), &grants)?;
    let site_origin = configured_origin("PYTXO_ROUTING_BRIDGE_SITE_ORIGIN", DEFAULT_SITE_ORIGIN)?;
    let state = random_b64()?;
    let verifier = random_b64()?;
    let challenge = URL_SAFE_NO_PAD.encode(Sha256::digest(verifier.as_bytes()));
    let pending = Pending {
        state: state.clone(),
        verifier,
        link_origin,
        created_at: Utc::now().timestamp(),
        recovery_only: account_link_recovery_only(bridge_enabled(), &grants, recovery_requested),
    };
    let in_flight = PENDING_LOCK.lock().map_err(map_io_err)?;
    if *in_flight {
        return Err(auth_error("A Desktop account return is still completing."));
    }
    if let Ok(session) = read_session() {
        if existing_session_blocks_new_link(&session, recovery_requested, bridge_enabled()) {
            return Err(auth_error(
                "Disconnect the current Routing account before connecting another.",
            ));
        }
    }
    pending_entry()?
        .set_password(&serde_json::to_string(&pending).map_err(map_io_err)?)
        .map_err(map_io_err)?;
    let url = account_return_url(&site_origin, &state, &challenge, pending.recovery_only);
    if let Err(error) = app.shell().open(url, None) {
        let _ = pending_entry()?.delete_credential();
        return Err(map_io_err(error));
    }
    Ok(())
}

#[tauri::command]
pub fn routing_account_connect(app: AppHandle) -> IpcResult<()> {
    begin_account_link(&app, false)
}

#[tauri::command]
pub fn routing_account_reconnect_for_revocation(app: AppHandle) -> IpcResult<()> {
    begin_account_link(&app, true)
}

fn callback_parts(input: &str) -> IpcResult<(String, String)> {
    let url = tauri::Url::parse(input).map_err(|_| auth_error("Invalid account return link."))?;
    if url.scheme() != "pytxo-deck"
        || url.host_str() != Some("auth")
        || !matches!(url.path(), "" | "/")
        || url.fragment().is_some()
        || url.username() != ""
        || url.password().is_some()
    {
        return Err(auth_error("Invalid account return link."));
    }
    let mut state = None;
    let mut code = None;
    let mut count = 0;
    for (key, value) in url.query_pairs() {
        count += 1;
        match key.as_ref() {
            "state" if state.is_none() => state = Some(value.into_owned()),
            "code" if code.is_none() => code = Some(value.into_owned()),
            _ => return Err(auth_error("Invalid account return link.")),
        }
    }
    let state = state.ok_or_else(|| auth_error("Account return link is incomplete."))?;
    let code = code.ok_or_else(|| auth_error("Account return link is incomplete."))?;
    if count != 2
        || !valid_b64_32(&state)
        || !code.starts_with("pdc1_")
        || !valid_b64_32(&code[5..])
    {
        return Err(auth_error("Invalid account return link."));
    }
    Ok((state, code))
}

fn take_pending(state: &str) -> IpcResult<Pending> {
    let mut in_flight = PENDING_LOCK.lock().map_err(map_io_err)?;
    if *in_flight {
        return Err(auth_error("A Desktop account return is still completing."));
    }
    let entry = pending_entry()?;
    let serialized = entry
        .get_password()
        .map_err(|_| auth_error("No Desktop account request is pending."))?;
    let pending: Pending = serde_json::from_str(&serialized)
        .map_err(|_| auth_error("Desktop account request is invalid."))?;
    let age = Utc::now().timestamp() - pending.created_at;
    if !(0..=600).contains(&age) {
        let _ = entry.delete_credential();
        return Err(auth_error("Desktop account request expired. Start again."));
    }
    if pending.state != state
        || !valid_b64_32(&pending.verifier)
        || checked_origin(&pending.link_origin).is_err()
    {
        return Err(auth_error(
            "Desktop account return did not match this request.",
        ));
    }
    entry.delete_credential().map_err(map_io_err)?;
    *in_flight = true;
    Ok(pending)
}

fn exchange_at(
    link_origin: &str,
    state: &str,
    code: &str,
    verifier: &str,
    recovery_only: bool,
) -> IpcResult<Session> {
    let link_origin = checked_origin(link_origin)?;
    let endpoint = format!("{link_origin}/v1/routing/desktop-exchange");
    let result = ureq::AgentBuilder::new()
        .timeout(Duration::from_secs(5))
        .redirects(0)
        .build()
        .post(&endpoint)
        .set("Content-Type", "application/json")
        .send_json(serde_json::json!({
            "code": code,
            "state": state,
            "code_verifier": verifier,
        }))
        .map_err(|_| auth_error("Pytxo Link could not complete account return."))?;
    let reply: ExchangeReply = result
        .into_json()
        .map_err(|_| auth_error("Pytxo Link returned an invalid account response."))?;
    checked_exchange_session(reply, &link_origin, recovery_only)
}

fn checked_exchange_session(
    reply: ExchangeReply,
    link_origin: &str,
    recovery_only: bool,
) -> IpcResult<Session> {
    let link_origin = checked_origin(link_origin)?;
    if !reply.token.starts_with("pds1_")
        || !valid_b64_32(&reply.token[5..])
        || !reply.account_id.starts_with("user_")
        || reply.scope != if recovery_only { RECOVERY_SCOPE } else { SCOPE }
        || reply.expires_at <= Utc::now()
        || reply.expires_at > Utc::now() + chrono::Duration::hours(25)
    {
        return Err(auth_error(
            "Pytxo Link returned an invalid account response.",
        ));
    }
    Ok(Session {
        token: reply.token,
        account_id: reply.account_id,
        expires_at: reply.expires_at,
        scope: reply.scope,
        link_origin,
    })
}

pub fn handle_callback(app: &AppHandle, input: &str) -> IpcResult<()> {
    if !bridge_enabled() {
        let grants = Catalog::open_default()
            .and_then(|catalog| catalog.unrevoked_hosted_grants())
            .map_err(map_store_err)?;
        if grants.is_empty() {
            return Err(auth_error(
                "Experimental Routing account connection is unavailable.",
            ));
        }
    }
    let (state, code) = callback_parts(input)?;
    let pending = take_pending(&state)?;
    let app = app.clone();
    tauri::async_runtime::spawn_blocking(move || {
        let _reset = InFlightReset;
        let result = if !bridge_enabled() && !pending.recovery_only {
            Err(auth_error(
                "Experimental Routing account connection was turned off during this return.",
            ))
        } else {
            exchange_at(
            &pending.link_origin,
            &state,
            &code,
            &pending.verifier,
            pending.recovery_only,
        )
        .and_then(
            |session| {
                let grants = Catalog::open_default()
                    .and_then(|catalog| catalog.unrevoked_hosted_grants())
                    .map_err(map_store_err)?;
                if !bridge_enabled() && (!pending.recovery_only || grants.is_empty()) {
                    // The final pending grant may have been resolved while
                    // the browser exchange was in flight. Do not persist an
                    // otherwise unnecessary credential after the off switch.
                    // The exchanged token is only in this closure and is
                    // dropped. Link's session DELETE revokes the whole account,
                    // so it would be unsafe to use for a late return here.
                    return Err(auth_error(
                        "Experimental Routing account connection is unavailable; no workspace grant still needs recovery.",
                    ));
                }
                if !session_matches_unrevoked_grants(&session, &grants) {
                    return Err(auth_error("Reconnect the original Routing account and Link origin to finish pending workspace revocation."));
                }
                session_entry()?
                    .set_password(&serde_json::to_string(&session).map_err(map_io_err)?)
                    .map_err(map_io_err)
            },
        )
        };
        match result {
            Ok(()) => {
                let _ = app.emit(AUTH_CHANGED_EVENT, ());
            }
            Err(error) => {
                let _ = app.emit(
                    AUTH_ERROR_EVENT,
                    serde_json::json!({"message":error.message}),
                );
            }
        }
        focus_main_window(&app);
    });
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use pytxo_orchestrate::HostedShadowClient;
    use pytxo_store::HostedGrantIntent;
    use std::io::{Read, Write};
    use std::net::TcpListener;

    #[test]
    fn callback_accepts_only_exact_one_time_code_and_state() {
        let state = URL_SAFE_NO_PAD.encode([5u8; 32]);
        let code = format!("pdc1_{}", URL_SAFE_NO_PAD.encode([6u8; 32]));
        assert_eq!(
            callback_parts(&format!("pytxo-deck://auth?state={state}&code={code}")).unwrap(),
            (state.clone(), code.clone())
        );
        for url in [
            format!("pytxo://auth?state={state}&code={code}"),
            format!("pytxo-deck://evil?state={state}&code={code}"),
            format!("pytxo-deck://auth?state={state}&code={code}&token=secret"),
            format!("pytxo-deck://auth?state={state}&state={state}&code={code}"),
            format!("pytxo-deck://auth?state={state}&code={code}#fragment"),
            "pytxo-deck://auth?token=header.payload.signature".into(),
        ] {
            assert!(callback_parts(&url).is_err(), "{url}");
        }
    }

    #[test]
    fn bridge_origin_is_pinned_to_https_or_debug_loopback() {
        assert_eq!(
            checked_origin("https://link.pytxo.com/").unwrap(),
            "https://link.pytxo.com"
        );
        for value in [
            "https://link.pytxo.com.evil.test/path",
            "https://user:pass@link.pytxo.com",
            "https://link.pytxo.com/?next=evil",
            "http://remote.example",
        ] {
            assert!(checked_origin(value).is_err(), "{value}");
        }
        if cfg!(debug_assertions) {
            assert_eq!(
                checked_origin("http://127.0.0.1:15579").unwrap(),
                "http://127.0.0.1:15579"
            );
        }
    }

    #[test]
    fn account_return_requires_the_exact_experimental_switch() {
        assert!(bridge_enabled_value(Some("1")));
        for value in [None, Some("0"), Some("true"), Some("1 ")] {
            assert!(!bridge_enabled_value(value));
        }
    }

    #[test]
    fn hosted_http_client_requires_matching_verified_session_and_enabled_grant() {
        let session = Session {
            token: format!("pds1_{}", URL_SAFE_NO_PAD.encode([7u8; 32])),
            account_id: "user_bound".into(),
            expires_at: Utc::now() + chrono::Duration::minutes(4),
            scope: SCOPE.into(),
            link_origin: DEFAULT_LINK_ORIGIN.into(),
        };
        let binding = HostedGrantBinding {
            intent: HostedGrantIntent {
                domain_id: "C:/repo".into(),
                workspace_id: "0".repeat(32),
                account_id: session.account_id.clone(),
                link_origin: session.link_origin.clone(),
                recipient_identity: pytxo_planner::advisor::HOSTED_RECIPIENT.into(),
                scope_digest: pytxo_planner::advisor::hosted_scope_digest().0,
                store_db_file_identity: "file-a".into(),
                consent_revision: 2,
            },
            remote_revision: Some(7),
            state: HostedGrantState::Enabled,
        };
        let client = hosted_http_client_from_verified_session(&session, &binding).unwrap();
        assert_eq!(client.account_id(), "user_bound");
        assert_eq!(client.workspace_id(), binding.intent.workspace_id);
        assert_eq!(client.grant_revision(), 7);
        let mut wrong_account = binding.clone();
        wrong_account.intent.account_id = "user_other".into();
        assert!(hosted_http_client_from_verified_session(&session, &wrong_account).is_err());
        let mut revoked = binding;
        revoked.state = HostedGrantState::RevokePending;
        assert!(hosted_http_client_from_verified_session(&session, &revoked).is_err());
    }

    #[test]
    fn unresolved_grants_mark_the_browser_return_as_recovery() {
        let ordinary = account_return_url("https://pytxo.com", "state", "challenge", false);
        let recovery = account_return_url("https://pytxo.com", "state", "challenge", true);
        assert!(!ordinary.contains("deck_recovery"));
        assert!(recovery.ends_with("&deck_recovery=1"));
        let binding = HostedGrantBinding {
            intent: HostedGrantIntent {
                domain_id: "C:/repo".into(),
                workspace_id: "a".repeat(32),
                account_id: "user_one".into(),
                link_origin: DEFAULT_LINK_ORIGIN.into(),
                recipient_identity: "pytxo-hosted-routing/typesafe-systemone/v1".into(),
                scope_digest: "b".repeat(64),
                store_db_file_identity: "file-a".into(),
                consent_revision: 1,
            },
            remote_revision: Some(1),
            state: HostedGrantState::RevokePending,
        };
        assert!(account_link_recovery_only(
            true,
            std::slice::from_ref(&binding),
            false
        ));
        assert!(account_link_recovery_only(
            false,
            std::slice::from_ref(&binding),
            false
        ));
        assert!(!account_link_recovery_only(false, &[], false));
        let active = HostedGrantBinding {
            state: HostedGrantState::Enabled,
            ..binding.clone()
        };
        assert!(!account_link_recovery_only(
            true,
            std::slice::from_ref(&active),
            false
        ));
        assert!(account_link_recovery_only(
            true,
            std::slice::from_ref(&active),
            true
        ));
        assert!(!account_link_recovery_only(
            true,
            &[active.clone(), binding.clone()],
            false
        ));
        assert!(account_link_recovery_only(true, &[binding], true));
        let pending = HostedGrantBinding {
            state: HostedGrantState::GrantPending,
            ..active
        };
        assert!(!account_link_recovery_only(true, &[pending], false));
    }

    #[test]
    fn account_return_scope_must_match_the_pending_request() {
        let reply = |scope: &str| ExchangeReply {
            token: format!("pds1_{}", URL_SAFE_NO_PAD.encode([1u8; 32])),
            account_id: "user_one".into(),
            expires_at: Utc::now() + chrono::Duration::hours(1),
            scope: scope.into(),
        };
        assert!(checked_exchange_session(reply(SCOPE), DEFAULT_LINK_ORIGIN, false).is_ok());
        assert!(checked_exchange_session(reply(RECOVERY_SCOPE), DEFAULT_LINK_ORIGIN, true).is_ok());
        assert!(
            checked_exchange_session(reply(RECOVERY_SCOPE), DEFAULT_LINK_ORIGIN, false).is_err()
        );
        assert!(checked_exchange_session(reply(SCOPE), DEFAULT_LINK_ORIGIN, true).is_err());
        let recovered =
            checked_exchange_session(reply(RECOVERY_SCOPE), DEFAULT_LINK_ORIGIN, true).unwrap();
        assert!(parse_session(&serde_json::to_string(&recovered).unwrap()).is_ok());
    }

    #[test]
    fn hosted_grant_receipt_binds_verified_session_and_exact_workspace_identity() {
        let session = Session {
            token: format!("pds1_{}", URL_SAFE_NO_PAD.encode([1u8; 32])),
            account_id: "user_one".into(),
            expires_at: Utc::now() + chrono::Duration::hours(1),
            scope: SCOPE.into(),
            link_origin: DEFAULT_LINK_ORIGIN.into(),
        };
        let intent = HostedGrantIntent {
            domain_id: "C:/repo".into(),
            workspace_id: "a".repeat(32),
            account_id: session.account_id.clone(),
            link_origin: session.link_origin.clone(),
            recipient_identity: "pytxo-hosted-routing/typesafe-systemone/v1".into(),
            scope_digest: "b".repeat(64),
            store_db_file_identity: "file-a".into(),
            consent_revision: 1,
        };
        let good = GrantReply {
            workspace_id: intent.workspace_id.clone(),
            recipient_identity: intent.recipient_identity.clone(),
            scope_digest: intent.scope_digest.clone(),
            revision: 1,
            enabled: true,
        };
        let receipt = checked_grant_receipt(&session, &intent, good.clone()).unwrap();
        assert_eq!(receipt.account_id, "user_one");
        assert_eq!(receipt.link_origin, DEFAULT_LINK_ORIGIN);
        for bad in [
            GrantReply {
                workspace_id: "c".repeat(32),
                ..good.clone()
            },
            GrantReply {
                recipient_identity: "other".into(),
                ..good.clone()
            },
            GrantReply {
                scope_digest: "not-hex".into(),
                ..good.clone()
            },
            GrantReply {
                revision: 0,
                ..good.clone()
            },
        ] {
            assert!(checked_grant_receipt(&session, &intent, bad).is_err());
        }
        let mut other_account = intent.clone();
        other_account.account_id = "user_other".into();
        assert!(checked_grant_receipt(&session, &other_account, good.clone()).is_err());
        let mut other_origin = intent.clone();
        other_origin.link_origin = "https://other.test".into();
        assert!(checked_grant_receipt(&session, &other_origin, good).is_err());
    }

    #[test]
    fn local_revoke_before_link_enable_reply_triggers_remote_cleanup() {
        use pytxo_store::{HostedAdvisorConsentFence, HostedAdvisorConsentReview};

        let listener = TcpListener::bind("127.0.0.1:0").unwrap();
        let link_origin = format!("http://{}", listener.local_addr().unwrap());
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("hypervisor.db");
        let catalog = Catalog::open(&path).unwrap();
        catalog
            .upsert_domain("C:/repo", "C:/repo", "C:/repo/pytxo.db", None)
            .unwrap();
        catalog
            .bind_routing_advisor_consent_store("C:/repo", "C:/repo/pytxo.db", "file-a")
            .unwrap();
        let intent = HostedGrantIntent {
            domain_id: "C:/repo".into(),
            workspace_id: catalog
                .ensure_hosted_workspace_id("C:/repo", "C:/repo/pytxo.db", "file-a")
                .unwrap(),
            account_id: "user_one".into(),
            link_origin: link_origin.clone(),
            recipient_identity: "pytxo-hosted-routing/typesafe-systemone/v1".into(),
            scope_digest: "a".repeat(64),
            store_db_file_identity: "file-a".into(),
            consent_revision: 1,
        };
        catalog
            .record_hosted_advisor_consent_review(&HostedAdvisorConsentReview {
                domain_id: intent.domain_id.clone(),
                recipient_identity: intent.recipient_identity.clone(),
                draft_id: "reviewed-flow".into(),
                consent_revision: 1,
                scope_digest: intent.scope_digest.clone(),
                packet_digest: "b".repeat(64),
                request_digest: "c".repeat(64),
                store_db_file_identity: intent.store_db_file_identity.clone(),
            })
            .unwrap();
        catalog
            .advance_hosted_advisor_consent_fence(&HostedAdvisorConsentFence {
                domain_id: intent.domain_id.clone(),
                recipient_identity: intent.recipient_identity.clone(),
                consent_revision: 1,
                enabled: true,
                store_db_file_identity: intent.store_db_file_identity.clone(),
            })
            .unwrap();
        catalog.begin_hosted_grant(&intent).unwrap();
        let server_intent = intent.clone();
        let server = std::thread::spawn(move || {
            for (method, revision, enabled) in
                [("POST", 1, true), ("GET", 1, true), ("DELETE", 2, false)]
            {
                let (mut stream, _) = listener.accept().unwrap();
                stream
                    .set_read_timeout(Some(Duration::from_secs(3)))
                    .unwrap();
                let request = read_test_request(&mut stream);
                assert!(request.starts_with(&format!("{method} /v1/routing/workspace-grants")));
                if method != "GET" {
                    let body: serde_json::Value =
                        serde_json::from_str(request.split_once("\r\n\r\n").unwrap().1).unwrap();
                    assert_eq!(body["expected_revision"], revision - 1);
                    assert_eq!(body["workspace_id"], server_intent.workspace_id);
                }
                let reply = serde_json::json!({
                    "workspace_id": server_intent.workspace_id,
                    "recipient_identity": server_intent.recipient_identity,
                    "scope_digest": server_intent.scope_digest,
                    "revision": revision,
                    "enabled": enabled,
                })
                .to_string();
                stream
                    .write_all(
                        format!(
                            "HTTP/1.1 200 OK\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{reply}",
                            reply.len()
                        )
                        .as_bytes(),
                    )
                    .unwrap();
            }
        });

        let (before_post_tx, before_post_rx) = std::sync::mpsc::channel();
        let (resume_tx, resume_rx) = std::sync::mpsc::channel();
        let worker_path = path.clone();
        let worker_intent = intent.clone();
        let session = Session {
            token: format!("pds1_{}", URL_SAFE_NO_PAD.encode([1u8; 32])),
            account_id: intent.account_id.clone(),
            expires_at: Utc::now() + chrono::Duration::hours(1),
            scope: SCOPE.into(),
            link_origin,
        };
        let worker_session: Session =
            serde_json::from_str(&serde_json::to_string(&session).unwrap()).unwrap();
        let worker = std::thread::spawn(move || {
            let worker_catalog = Catalog::open(&worker_path).unwrap();
            before_post_tx.send(()).unwrap();
            resume_rx.recv().unwrap();
            let enabled = link_grant_write(&worker_session, &worker_intent, 0, true).unwrap();
            let mut delete_calls = 0;
            let result =
                confirm_remote_enabled_or_revoke(&worker_catalog, &worker_intent, &enabled, || {
                    delete_calls += 1;
                    Err(auth_error("simulated Link cleanup failure"))
                });
            (result.is_err(), delete_calls)
        });
        before_post_rx.recv().unwrap();
        catalog
            .advance_hosted_advisor_consent_fence(&HostedAdvisorConsentFence {
                domain_id: intent.domain_id.clone(),
                recipient_identity: intent.recipient_identity.clone(),
                consent_revision: 2,
                enabled: false,
                store_db_file_identity: intent.store_db_file_identity.clone(),
            })
            .unwrap();
        resume_tx.send(()).unwrap();
        assert_eq!(worker.join().unwrap(), (true, 1));
        assert_eq!(
            catalog
                .hosted_grant(&intent.domain_id)
                .unwrap()
                .unwrap()
                .state,
            HostedGrantState::RevokePending
        );
        let remote = link_grant_status(&session, &intent).unwrap().unwrap();
        assert!(remote.enabled);
        let disabled = link_grant_write(&session, &intent, remote.revision, false).unwrap();
        assert!(!disabled.enabled);
        catalog
            .confirm_hosted_grant_revoked(&intent.domain_id, &disabled)
            .unwrap();
        assert_eq!(
            catalog
                .hosted_grant(&intent.domain_id)
                .unwrap()
                .unwrap()
                .state,
            HostedGrantState::Revoked
        );
        server.join().unwrap();
    }

    #[test]
    fn expired_account_may_only_relink_to_original_pending_grant_owner() {
        let original = Session {
            token: format!("pds1_{}", URL_SAFE_NO_PAD.encode([1u8; 32])),
            account_id: "user_one".into(),
            expires_at: Utc::now() - chrono::Duration::hours(1),
            scope: SCOPE.into(),
            link_origin: DEFAULT_LINK_ORIGIN.into(),
        };
        assert!(parse_session(&serde_json::to_string(&original).unwrap()).is_err());
        let binding = HostedGrantBinding {
            intent: HostedGrantIntent {
                domain_id: "C:/repo".into(),
                workspace_id: "a".repeat(32),
                account_id: original.account_id.clone(),
                link_origin: original.link_origin.clone(),
                recipient_identity: "pytxo-hosted-routing/typesafe-systemone/v1".into(),
                scope_digest: "b".repeat(64),
                store_db_file_identity: "file-a".into(),
                consent_revision: 1,
            },
            remote_revision: Some(1),
            state: HostedGrantState::RevokePending,
        };
        let mut renewed = original;
        renewed.expires_at = Utc::now() + chrono::Duration::hours(1);
        assert_eq!(
            account_link_origin_for_start(false, std::slice::from_ref(&binding)).unwrap(),
            binding.intent.link_origin
        );
        assert!(account_link_origin_for_start(false, &[]).is_err());
        let mut other_origin = binding.clone();
        other_origin.intent.link_origin = "https://other.test".into();
        assert!(account_link_origin_for_start(false, &[binding.clone(), other_origin]).is_err());
        assert!(session_matches_unrevoked_grants(
            &renewed,
            std::slice::from_ref(&binding)
        ));
        renewed.account_id = "user_other".into();
        assert!(!session_matches_unrevoked_grants(
            &renewed,
            std::slice::from_ref(&binding)
        ));
        renewed.account_id = "user_one".into();
        renewed.link_origin = "https://other.test".into();
        assert!(!session_matches_unrevoked_grants(&renewed, &[binding]));
    }

    #[test]
    fn remotely_revoked_unexpired_session_can_start_relink() {
        let listener = TcpListener::bind("127.0.0.1:0").unwrap();
        let address = listener.local_addr().unwrap();
        let server = std::thread::spawn(move || {
            let (mut stream, _) = listener.accept().unwrap();
            stream
                .set_read_timeout(Some(Duration::from_secs(3)))
                .unwrap();
            let request = read_test_request(&mut stream);
            assert!(request.starts_with("GET /v1/routing/desktop-session HTTP/1.1"));
            stream
                .write_all(
                    b"HTTP/1.1 401 Unauthorized\r\nContent-Length: 0\r\nConnection: close\r\n\r\n",
                )
                .unwrap();
        });
        let session = Session {
            token: format!("pds1_{}", URL_SAFE_NO_PAD.encode([1u8; 32])),
            account_id: "user_one".into(),
            expires_at: Utc::now() + chrono::Duration::hours(1),
            scope: SCOPE.into(),
            link_origin: format!("http://{address}"),
        };
        assert!(!existing_session_blocks_new_link(&session, false, true));
        server.join().unwrap();
    }

    #[test]
    fn recovery_credential_can_be_replaced_by_full_scope_when_bridge_returns() {
        let session = Session {
            token: format!("pds1_{}", URL_SAFE_NO_PAD.encode([1u8; 32])),
            account_id: "user_one".into(),
            expires_at: Utc::now() + chrono::Duration::hours(1),
            scope: RECOVERY_SCOPE.into(),
            link_origin: DEFAULT_LINK_ORIGIN.into(),
        };
        assert!(!existing_session_blocks_new_link(&session, false, true));
    }

    #[test]
    fn uncertain_grant_revoke_sends_zero_revision_and_requires_disabled_reply() {
        let listener = TcpListener::bind("127.0.0.1:0").unwrap();
        let address = listener.local_addr().unwrap();
        let workspace_id = "a".repeat(32);
        let scope_digest = "b".repeat(64);
        let intent = HostedGrantIntent {
            domain_id: "C:/repo".into(),
            workspace_id: workspace_id.clone(),
            account_id: "user_one".into(),
            link_origin: format!("http://{address}"),
            recipient_identity: "pytxo-hosted-routing/typesafe-systemone/v1".into(),
            scope_digest: scope_digest.clone(),
            store_db_file_identity: "file-a".into(),
            consent_revision: 1,
        };
        let server = std::thread::spawn(move || {
            let (mut first, _) = listener.accept().unwrap();
            first
                .set_read_timeout(Some(Duration::from_secs(3)))
                .unwrap();
            let request = read_test_request(&mut first);
            assert!(request.starts_with(&format!(
                "GET /v1/routing/workspace-grants/{workspace_id} HTTP/1.1"
            )));
            assert!(request
                .to_ascii_lowercase()
                .contains("authorization: bearer pds1_"));
            first
                .write_all(
                    b"HTTP/1.1 404 Not Found\r\nContent-Length: 0\r\nConnection: close\r\n\r\n",
                )
                .unwrap();
            let (mut second, _) = listener.accept().unwrap();
            second
                .set_read_timeout(Some(Duration::from_secs(3)))
                .unwrap();
            let request = read_test_request(&mut second);
            assert!(request.starts_with("DELETE /v1/routing/workspace-grants HTTP/1.1"));
            let body = request.split_once("\r\n\r\n").unwrap().1;
            let body: serde_json::Value = serde_json::from_str(body).unwrap();
            assert_eq!(body["expected_revision"], 0);
            assert_eq!(body["workspace_id"], workspace_id);
            let reply = serde_json::json!({
                "workspace_id": workspace_id,
                "recipient_identity": "pytxo-hosted-routing/typesafe-systemone/v1",
                "scope_digest": scope_digest,
                "revision": 1,
                "enabled": false,
            })
            .to_string();
            second.write_all(format!(
                "HTTP/1.1 200 OK\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{reply}", reply.len()
            ).as_bytes()).unwrap();
        });
        let session = Session {
            token: format!("pds1_{}", URL_SAFE_NO_PAD.encode([1u8; 32])),
            account_id: "user_one".into(),
            expires_at: Utc::now() + chrono::Duration::hours(1),
            scope: SCOPE.into(),
            link_origin: intent.link_origin.clone(),
        };
        assert!(link_grant_status(&session, &intent).unwrap().is_none());
        let disabled = link_grant_write(&session, &intent, 0, false).unwrap();
        assert!(!disabled.enabled);
        assert_eq!(disabled.revision, 1);
        server.join().unwrap();
    }

    fn read_test_request(stream: &mut std::net::TcpStream) -> String {
        let mut bytes = Vec::new();
        let mut buffer = [0u8; 1024];
        loop {
            let count = stream.read(&mut buffer).unwrap();
            assert!(count > 0 && bytes.len() + count < 8192);
            bytes.extend_from_slice(&buffer[..count]);
            if let Some(headers_end) = bytes.windows(4).position(|window| window == b"\r\n\r\n") {
                let headers_end = headers_end + 4;
                let headers = String::from_utf8_lossy(&bytes[..headers_end]);
                let length = headers
                    .lines()
                    .find_map(|line| {
                        line.to_ascii_lowercase()
                            .strip_prefix("content-length: ")
                            .and_then(|value| value.parse::<usize>().ok())
                    })
                    .unwrap_or(0);
                if bytes.len() >= headers_end + length {
                    return String::from_utf8(bytes).unwrap();
                }
            }
        }
    }

    #[test]
    fn expired_or_malformed_local_credentials_stay_visible_for_disconnect() {
        let expired = Session {
            token: format!("pds1_{}", URL_SAFE_NO_PAD.encode([1u8; 32])),
            account_id: "user_one".into(),
            expires_at: Utc::now() - chrono::Duration::hours(1),
            scope: SCOPE.into(),
            link_origin: DEFAULT_LINK_ORIGIN.into(),
        };
        let serialized = serde_json::to_string(&expired).unwrap();
        assert!(parse_session(&serialized).is_err());
        assert!(parse_session("not-json").is_err());
        assert!(credential_present(Ok(serialized)));
        assert!(credential_present(Ok("not-json".into())));
        assert!(!credential_present(Err(keyring::Error::NoEntry)));
    }

    #[test]
    fn remote_status_requires_the_exact_account_scope_and_expiry() {
        let expires_at = Utc::now() + chrono::Duration::hours(1);
        let session = Session {
            token: format!("pds1_{}", URL_SAFE_NO_PAD.encode([1u8; 32])),
            account_id: "user_one".into(),
            expires_at,
            scope: SCOPE.into(),
            link_origin: DEFAULT_LINK_ORIGIN.into(),
        };
        let mut reply = SessionStatusReply {
            account_id: session.account_id.clone(),
            expires_at,
            scope: SCOPE.into(),
        };
        assert!(status_matches_local(&reply, &session));
        reply.account_id = "user_other".into();
        assert!(!status_matches_local(&reply, &session));
        reply.account_id = session.account_id.clone();
        reply.scope = "general".into();
        assert!(!status_matches_local(&reply, &session));
        reply.scope = SCOPE.into();
        reply.expires_at += chrono::Duration::seconds(1);
        assert!(!status_matches_local(&reply, &session));
    }

    #[test]
    fn remote_status_fails_closed_for_revocation_mismatch_and_redirect() {
        let expires_at = Utc::now() + chrono::Duration::hours(1);
        let account_id = "user_one";
        let valid_body = serde_json::json!({
            "account_id": account_id,
            "expires_at": expires_at,
            "scope": SCOPE,
        })
        .to_string();
        let wrong_account = serde_json::json!({
            "account_id": "user_other",
            "expires_at": expires_at,
            "scope": SCOPE,
        })
        .to_string();
        for (status, body, expected) in [
            (200, valid_body, RemoteStatus::Verified),
            (401, String::new(), RemoteStatus::Revoked),
            (200, wrong_account, RemoteStatus::Unverified),
            (302, String::new(), RemoteStatus::Unverified),
        ] {
            let listener = TcpListener::bind("127.0.0.1:0").unwrap();
            let address = listener.local_addr().unwrap();
            let server = std::thread::spawn(move || {
                let (mut stream, _) = listener.accept().unwrap();
                stream
                    .set_read_timeout(Some(Duration::from_secs(3)))
                    .unwrap();
                let mut request = [0u8; 4096];
                let count = stream.read(&mut request).unwrap();
                assert!(String::from_utf8_lossy(&request[..count])
                    .starts_with("GET /v1/routing/desktop-session HTTP/1.1"));
                let reason = match status {
                    200 => "OK",
                    401 => "Unauthorized",
                    _ => "Found",
                };
                let reply = format!(
                    "HTTP/1.1 {status} {reason}\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{body}",
                    body.len()
                );
                stream.write_all(reply.as_bytes()).unwrap();
            });
            let session = Session {
                token: format!("pds1_{}", URL_SAFE_NO_PAD.encode([1u8; 32])),
                account_id: account_id.into(),
                expires_at,
                scope: SCOPE.into(),
                link_origin: format!("http://{address}"),
            };
            assert_eq!(verify_remote(&session), expected);
            server.join().unwrap();
        }
    }
}
