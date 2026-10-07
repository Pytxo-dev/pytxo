//! Isolated sponsored routing path. It never enters Ultra provider forwarding,
//! wallet metering, subscription checks, or bearer-key rate buckets.

use std::future::Future;
use std::pin::Pin;
use std::sync::Arc;
use std::time::Duration;

use axum::body;
use axum::extract::{Request, State};
use axum::http::{header, StatusCode};
use axum::response::{IntoResponse, Response};
use axum::Json;
use base64::engine::general_purpose::URL_SAFE_NO_PAD;
use base64::Engine;
use pytxo_core::routing::{AdviceChoice, AdviceDistribution, Digest};
use pytxo_planner::advisor::{
    parse_response, AdvisorPacket, JevHttpTransport, UnavailableReason, MODEL_ID, TEMPLATE_VERSION,
};
use ring::aead::{self, Aad, LessSafeKey, Nonce, UnboundKey};
use ring::rand::{SecureRandom, SystemRandom};
use serde::{Deserialize, Serialize};

use crate::AppState;

const MAX_CLIENT_BYTES: usize = 16 * 1024;
const MAX_LINK_REPLY_BYTES: usize = 2 * 1024;
const FIXED_GOAL: &str = "Classify reviewed repository task using coarse facts";
const EVERYDAY_ROLE: &str = "everyday execution role";
const STRONG_ROLE: &str = "strong execution role";
// Local Shadow consent names a different, no-network recipient. Keep real
// paid dispatch closed until a hosted workspace grant is verified at Link.
const HOSTED_CONSENT_GATE_COMPLETE: bool = false;

fn pinned_price(value: Option<&str>, positive: bool) -> Option<i64> {
    value?
        .parse::<i64>()
        .ok()
        .filter(|price| if positive { *price > 0 } else { *price >= 0 })
}

type UpstreamFuture<'a> =
    Pin<Box<dyn Future<Output = Result<Vec<u8>, UnavailableReason>> + Send + 'a>>;

pub trait RoutingUpstream: Send + Sync {
    fn send<'a>(&'a self, body: &'a [u8]) -> UpstreamFuture<'a>;
}

impl RoutingUpstream for JevHttpTransport {
    fn send<'a>(&'a self, body: &'a [u8]) -> UpstreamFuture<'a> {
        Box::pin(self.send_fixed(body))
    }
}

#[derive(Clone)]
pub struct HostedRouting {
    client: reqwest::Client,
    link_base: String,
    service_key: String,
    upstream: Arc<dyn RoutingUpstream>,
    rate_card_revision: String,
    input_nano_usd_per_token: i64,
    output_nano_usd_per_token: i64,
    recovery: RecoveryCipher,
}

#[derive(Clone)]
struct RecoveryCipher {
    key: [u8; 32],
    revision: String,
    previous: Option<([u8; 32], String)>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
struct RecoveryBlob {
    key_revision: String,
    nonce: String,
    ciphertext: String,
}

#[derive(Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
struct Delivery {
    schema_version: u32,
    evaluation_id: String,
    request_id: String,
    question_set_version: String,
    model_id: String,
    choice: AdviceChoice,
    distribution: AdviceDistribution,
    usage_status: String,
    usage_receipt_id: String,
    input_tokens: i64,
    output_tokens: i64,
    cost_nano_usd: i64,
    packet_digest: String,
}

impl RecoveryCipher {
    fn aad(request_id: &str, payload_digest: &str) -> Vec<u8> {
        format!("pytxo-routing-answer/v1\0{request_id}\0{payload_digest}").into_bytes()
    }

    fn seal(&self, delivery: &Delivery, payload_digest: &str) -> Result<RecoveryBlob, ()> {
        let mut nonce = [0u8; 12];
        SystemRandom::new().fill(&mut nonce).map_err(|_| ())?;
        let key = LessSafeKey::new(UnboundKey::new(&aead::AES_256_GCM, &self.key).map_err(|_| ())?);
        let mut plaintext = serde_json::to_vec(delivery).map_err(|_| ())?;
        // Leave room for base64 and the Link envelope under its 2 KiB cap.
        if plaintext.len() > 1024 {
            return Err(());
        }
        key.seal_in_place_append_tag(
            Nonce::assume_unique_for_key(nonce),
            Aad::from(Self::aad(&delivery.request_id, payload_digest)),
            &mut plaintext,
        )
        .map_err(|_| ())?;
        Ok(RecoveryBlob {
            key_revision: self.revision.clone(),
            nonce: URL_SAFE_NO_PAD.encode(nonce),
            ciphertext: URL_SAFE_NO_PAD.encode(plaintext),
        })
    }

    fn open(
        &self,
        blob: &RecoveryBlob,
        request_id: &str,
        payload_digest: &str,
    ) -> Result<Delivery, ()> {
        let key_bytes = if blob.key_revision == self.revision {
            &self.key
        } else if let Some((key, revision)) = self.previous.as_ref() {
            if blob.key_revision != *revision {
                return Err(());
            }
            key
        } else {
            return Err(());
        };
        let nonce: [u8; 12] = URL_SAFE_NO_PAD
            .decode(&blob.nonce)
            .map_err(|_| ())?
            .try_into()
            .map_err(|_| ())?;
        let mut ciphertext = URL_SAFE_NO_PAD.decode(&blob.ciphertext).map_err(|_| ())?;
        if !(17..=2048).contains(&ciphertext.len()) {
            return Err(());
        }
        let key = LessSafeKey::new(UnboundKey::new(&aead::AES_256_GCM, key_bytes).map_err(|_| ())?);
        let bytes = key
            .open_in_place(
                Nonce::assume_unique_for_key(nonce),
                Aad::from(Self::aad(request_id, payload_digest)),
                &mut ciphertext,
            )
            .map_err(|_| ())?;
        let delivery: Delivery = serde_json::from_slice(bytes).map_err(|_| ())?;
        if delivery.schema_version != 1
            || delivery.request_id != request_id
            || delivery.evaluation_id != request_id
            || delivery.packet_digest.len() != 64
            || delivery.question_set_version != TEMPLATE_VERSION
            || delivery.model_id != MODEL_ID
            || delivery.usage_status != "known"
            || delivery.input_tokens < 0
            || delivery.output_tokens < 0
            || delivery.cost_nano_usd < 0
        {
            return Err(());
        }
        Ok(delivery)
    }
}

impl HostedRouting {
    pub fn from_env() -> Result<Option<Self>, &'static str> {
        if std::env::var("PROXY_ROUTING_PAID_SEND_EXPERIMENT")
            .ok()
            .as_deref()
            != Some("1")
        {
            return Ok(None);
        }
        if !HOSTED_CONSENT_GATE_COMPLETE {
            return Err("hosted routing recipient consent gate is not implemented");
        }
        let link_base =
            std::env::var("LINK_BASE_URL").map_err(|_| "hosted routing needs LINK_BASE_URL")?;
        let parsed = reqwest::Url::parse(&link_base).map_err(|_| "invalid LINK_BASE_URL")?;
        if parsed.scheme() != "https"
            && !(parsed.scheme() == "http" && parsed.host_str() == Some("127.0.0.1"))
        {
            return Err("hosted routing requires HTTPS Link or loopback test Link");
        }
        let service_key = std::env::var("LINK_ROUTING_SERVICE_KEY")
            .map_err(|_| "hosted routing needs its own Link service key")?;
        if service_key.len() < 32 {
            return Err("hosted routing Link service key must be at least 32 bytes");
        }
        let jev_key = std::env::var("TYPESAFE_ROUTING_API_KEY")
            .map_err(|_| "hosted routing needs its own Jev key")?;
        if jev_key == service_key
            || std::env::var("LINK_API_KEY").ok().as_deref() == Some(service_key.as_str())
        {
            return Err("hosted routing keys must be separate from Link service and client keys");
        }
        let upstream =
            JevHttpTransport::new(jev_key).map_err(|_| "invalid hosted Jev configuration")?;
        let rate_card_revision = std::env::var("ROUTING_RATE_CARD_REVISION")
            .map_err(|_| "hosted routing needs a pinned rate card")?;
        if rate_card_revision.is_empty()
            || rate_card_revision.len() > 64
            || !rate_card_revision
                .bytes()
                .all(|b| b.is_ascii_alphanumeric() || b == b'-' || b == b'_' || b == b'/')
        {
            return Err("invalid hosted routing rate card revision");
        }
        let input_nano_usd_per_token = pinned_price(
            std::env::var("ROUTING_INPUT_NANO_USD_PER_TOKEN")
                .ok()
                .as_deref(),
            true,
        )
        .ok_or("hosted routing needs pinned input price")?;
        let output_nano_usd_per_token = pinned_price(
            std::env::var("ROUTING_OUTPUT_NANO_USD_PER_TOKEN")
                .ok()
                .as_deref(),
            false,
        )
        .ok_or("hosted routing needs pinned output price")?;
        let recovery_key = std::env::var("PROXY_ROUTING_RECOVERY_KEY")
            .map_err(|_| "hosted routing needs a 32-byte answer recovery key")?;
        let recovery_key: [u8; 32] = hex::decode(recovery_key)
            .map_err(|_| "answer recovery key must be hex")?
            .try_into()
            .map_err(|_| "answer recovery key must be 32 bytes")?;
        let recovery_revision = std::env::var("PROXY_ROUTING_RECOVERY_KEY_REVISION")
            .map_err(|_| "hosted routing needs an answer key revision")?;
        if recovery_revision.is_empty()
            || recovery_revision.len() > 64
            || !recovery_revision
                .bytes()
                .all(|b| b.is_ascii_alphanumeric() || matches!(b, b'-' | b'_' | b'/'))
        {
            return Err("invalid answer recovery key revision");
        }
        let previous = match (
            std::env::var("PROXY_ROUTING_RECOVERY_PREVIOUS_KEY").ok(),
            std::env::var("PROXY_ROUTING_RECOVERY_PREVIOUS_KEY_REVISION").ok(),
        ) {
            (None, None) => None,
            (Some(key), Some(revision)) => {
                let key: [u8; 32] = hex::decode(key)
                    .map_err(|_| "previous answer recovery key must be hex")?
                    .try_into()
                    .map_err(|_| "previous answer recovery key must be 32 bytes")?;
                if revision.is_empty()
                    || revision == recovery_revision
                    || revision.len() > 64
                    || !revision
                        .bytes()
                        .all(|b| b.is_ascii_alphanumeric() || matches!(b, b'-' | b'_' | b'/'))
                {
                    return Err("invalid previous answer recovery key revision");
                }
                Some((key, revision))
            }
            _ => return Err("previous answer recovery key and revision must be paired"),
        };
        let client = reqwest::Client::builder()
            .redirect(reqwest::redirect::Policy::none())
            .retry(reqwest::retry::never())
            .no_proxy()
            .timeout(Duration::from_secs(2))
            .build()
            .map_err(|_| "failed to construct hosted routing Link client")?;
        Ok(Some(Self {
            client,
            link_base,
            service_key,
            upstream: Arc::new(upstream),
            rate_card_revision,
            input_nano_usd_per_token,
            output_nano_usd_per_token,
            recovery: RecoveryCipher {
                key: recovery_key,
                revision: recovery_revision,
                previous,
            },
        }))
    }

    async fn link(
        &self,
        action: &str,
        token: Option<&str>,
        payload: serde_json::Value,
    ) -> Result<LinkReply, StatusCode> {
        let url = format!(
            "{}/internal/routing/{action}",
            self.link_base.trim_end_matches('/')
        );
        let mut request = self
            .client
            .post(url)
            .header("x-pytxo-routing-service", &self.service_key)
            .json(&payload);
        if let Some(token) = token {
            request = request.bearer_auth(token);
        }
        let mut response = request
            .send()
            .await
            .map_err(|_| StatusCode::SERVICE_UNAVAILABLE)?;
        if !response.status().is_success() {
            return Err(match response.status().as_u16() {
                400 => StatusCode::BAD_REQUEST,
                401 => StatusCode::UNAUTHORIZED,
                409 => StatusCode::CONFLICT,
                429 => StatusCode::TOO_MANY_REQUESTS,
                _ => StatusCode::SERVICE_UNAVAILABLE,
            });
        }
        let mut bytes = Vec::new();
        while let Some(chunk) = response
            .chunk()
            .await
            .map_err(|_| StatusCode::SERVICE_UNAVAILABLE)?
        {
            if chunk.len() > MAX_LINK_REPLY_BYTES - bytes.len() {
                return Err(StatusCode::SERVICE_UNAVAILABLE);
            }
            bytes.extend_from_slice(&chunk);
        }
        let reply: LinkReply =
            serde_json::from_slice(&bytes).map_err(|_| StatusCode::SERVICE_UNAVAILABLE)?;
        Ok(reply)
    }
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct LinkReply {
    request_id: String,
    state: String,
    new_reservation: bool,
    owner_nonce: Option<String>,
    recovery: Option<RecoveryBlob>,
}

#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct WireRequest {
    schema_version: u32,
    request_id: String,
    decision_kind: String,
    question_set_version: String,
    packet: WirePacket,
}

#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct WirePacket {
    schema_version: u32,
    goal: String,
    features: Vec<String>,
    everyday_role: String,
    strong_role: String,
}

fn validate_wire(wire: &WireRequest) -> Result<AdvisorPacket, StatusCode> {
    if wire.schema_version != 1
        || wire.decision_kind != "initial_demand"
        || wire.question_set_version != TEMPLATE_VERSION
        || wire.packet.schema_version != 1
        || wire.packet.goal != FIXED_GOAL
        || wire.packet.everyday_role != EVERYDAY_ROLE
        || wire.packet.strong_role != STRONG_ROLE
        || wire.packet.features.len() != 8
    {
        return Err(StatusCode::BAD_REQUEST);
    }
    let choices: [&[&str]; 8] = [
        &[
            "task_kind_documentation",
            "task_kind_formatting",
            "task_kind_rename",
            "task_kind_local_transformation",
            "task_kind_diagnosis",
            "task_kind_architecture",
            "task_kind_unspecified",
        ],
        &["no_reviewed_checks", "reviewed_checks"],
        &[
            "cross_component_required",
            "single_component_claimed",
            "cross_component_unknown",
        ],
        &["context_claimed_complete", "context_incomplete"],
        &["strong_only", "role_unrestricted"],
        &["no_required_egress", "egress_required"],
        &[
            "repeatable_symptom_supplied",
            "repeatable_symptom_not_supplied",
            "repeatable_symptom_unknown",
        ],
        &[
            "specific_cause_hypothesis_supplied",
            "specific_cause_hypothesis_not_supplied",
            "specific_cause_hypothesis_unknown",
        ],
    ];
    if !wire
        .packet
        .features
        .iter()
        .enumerate()
        .all(|(index, feature)| choices[index].contains(&feature.as_str()))
    {
        return Err(StatusCode::BAD_REQUEST);
    }
    AdvisorPacket::new(
        &wire.packet.goal,
        wire.packet.features.iter().map(String::as_str),
        &wire.packet.everyday_role,
        &wire.packet.strong_role,
    )
    .map_err(|_| StatusCode::BAD_REQUEST)
}

fn unavailable(status: StatusCode, reason: &'static str) -> Response {
    (status, Json(serde_json::json!({"reason": reason}))).into_response()
}

pub async fn evaluate(State(state): State<Arc<AppState>>, request: Request) -> Response {
    let Some(hosted) = state.hosted_routing.as_ref() else {
        return unavailable(
            StatusCode::SERVICE_UNAVAILABLE,
            "hosted_routing_not_enabled",
        );
    };
    let token = request
        .headers()
        .get(header::AUTHORIZATION)
        .and_then(|value| value.to_str().ok())
        .and_then(|value| value.strip_prefix("Bearer "))
        .filter(|value| value.len() == 47 && value.starts_with("pr1_"));
    let Some(token) = token.map(str::to_owned) else {
        return unavailable(StatusCode::UNAUTHORIZED, "routing_token_required");
    };
    if request.headers().contains_key(header::CONTENT_ENCODING)
        || request
            .headers()
            .get(header::CONTENT_TYPE)
            .and_then(|value| value.to_str().ok())
            .is_none_or(|value| value != "application/json")
    {
        return unavailable(StatusCode::UNSUPPORTED_MEDIA_TYPE, "unsupported_encoding");
    }
    let bytes = match body::to_bytes(request.into_body(), MAX_CLIENT_BYTES).await {
        Ok(bytes) => bytes,
        Err(_) => return unavailable(StatusCode::PAYLOAD_TOO_LARGE, "routing_packet_too_large"),
    };
    let mut decoder = serde_json::Deserializer::from_slice(&bytes);
    let wire = match WireRequest::deserialize(&mut decoder) {
        Ok(wire) if decoder.end().is_ok() => wire,
        _ => return unavailable(StatusCode::BAD_REQUEST, "invalid_routing_packet"),
    };
    let packet = match validate_wire(&wire) {
        Ok(packet) => packet,
        Err(status) => return unavailable(status, "invalid_routing_packet"),
    };
    let body = match packet.request_body() {
        Ok(body) => body,
        Err(_) => return unavailable(StatusCode::BAD_REQUEST, "invalid_routing_packet"),
    };
    let canonical = match serde_json::to_vec(&wire) {
        Ok(bytes) => bytes,
        Err(_) => return unavailable(StatusCode::BAD_REQUEST, "invalid_routing_packet"),
    };
    let payload_digest = Digest::of_bytes(&canonical);
    let reserved = hosted
        .link(
            "reserve",
            Some(&token),
            serde_json::json!({"request_id": wire.request_id, "payload_sha256": payload_digest.0}),
        )
        .await;
    let Ok(reserved) = reserved else {
        return unavailable(reserved.err().unwrap(), "routing_admission_unavailable");
    };
    if reserved.request_id != wire.request_id
        || (reserved.new_reservation && reserved.state != "reserved")
        || !matches!(
            reserved.state.as_str(),
            "reserved"
                | "sending"
                | "completed"
                | "rejected_before_send"
                | "uncertain"
                | "settled_at_ceiling"
        )
    {
        return unavailable(StatusCode::SERVICE_UNAVAILABLE, "routing_admission_schema");
    }
    if reserved.state != "reserved" {
        if reserved.state == "completed" {
            if let Some(recovery) = reserved.recovery.as_ref() {
                if let Ok(delivery) =
                    hosted
                        .recovery
                        .open(recovery, &wire.request_id, &payload_digest.0)
                {
                    if delivery.packet_digest == packet.digest().0 {
                        return (
                            StatusCode::OK,
                            [(header::CACHE_CONTROL, "no-store")],
                            Json(delivery),
                        )
                            .into_response();
                    }
                }
                return unavailable(
                    StatusCode::SERVICE_UNAVAILABLE,
                    "routing_recovery_unavailable",
                );
            }
        }
        return unavailable(StatusCode::CONFLICT, "routing_operation_already_used");
    }
    // A reserved operation is cheap to abandon, but the claim and everything
    // after it must have one owner independent of the client connection.
    let hosted = hosted.clone();
    let completion = tokio::spawn(async move {
        let claimed = hosted
            .link(
                "claim",
                Some(&token),
                serde_json::json!({"request_id": wire.request_id}),
            )
            .await;
        let Ok(claimed) = claimed else {
            return unavailable(claimed.err().unwrap(), "routing_claim_unavailable");
        };
        let Some(owner_nonce) = claimed
            .owner_nonce
            .filter(|_| claimed.state == "sending" && claimed.request_id == wire.request_id)
        else {
            return unavailable(StatusCode::CONFLICT, "routing_operation_not_claimed");
        };
        // The client can stop waiting at two seconds. This owner still has a
        // five-second upstream deadline and attempts durable settlement.
        let upstream =
            tokio::time::timeout(Duration::from_secs(5), hosted.upstream.send(&body)).await;
        let bytes = match upstream {
            Ok(Ok(bytes)) => bytes,
            _ => {
                let _ = hosted
                    .link(
                        "uncertain",
                        None,
                        serde_json::json!({
                            "request_id": wire.request_id, "owner_nonce": owner_nonce,
                        }),
                    )
                    .await;
                return unavailable(
                    StatusCode::SERVICE_UNAVAILABLE,
                    "routing_upstream_uncertain",
                );
            }
        };
        let observation = match parse_response(&bytes) {
            Ok(observation) => observation,
            Err(_) => {
                let _ = hosted
                    .link(
                        "uncertain",
                        None,
                        serde_json::json!({
                            "request_id": wire.request_id, "owner_nonce": owner_nonce,
                        }),
                    )
                    .await;
                return unavailable(StatusCode::SERVICE_UNAVAILABLE, "routing_response_invalid");
            }
        };
        let input_tokens = i64::try_from(observation.usage.input_tokens).ok();
        let output_tokens = i64::try_from(observation.usage.output_tokens).ok();
        let cost = input_tokens.zip(output_tokens).and_then(|(input, output)| {
            input
                .checked_mul(hosted.input_nano_usd_per_token)
                .and_then(|part| {
                    output
                        .checked_mul(hosted.output_nano_usd_per_token)
                        .and_then(|other| part.checked_add(other))
                })
        });
        let (Some(input_tokens), Some(output_tokens), Some(cost)) =
            (input_tokens, output_tokens, cost)
        else {
            let _ = hosted
                .link(
                    "uncertain",
                    None,
                    serde_json::json!({
                        "request_id": wire.request_id, "owner_nonce": owner_nonce,
                    }),
                )
                .await;
            return unavailable(StatusCode::SERVICE_UNAVAILABLE, "routing_usage_invalid");
        };
        let receipt_id = format!("r_{}", &Digest::of_bytes(&bytes).0[..32]);
        let delivery = Delivery {
            schema_version: 1,
            evaluation_id: wire.request_id.clone(),
            request_id: wire.request_id.clone(),
            question_set_version: TEMPLATE_VERSION.into(),
            model_id: MODEL_ID.into(),
            choice: observation.choice,
            distribution: observation.distribution,
            usage_status: "known".into(),
            usage_receipt_id: receipt_id.clone(),
            input_tokens,
            output_tokens,
            cost_nano_usd: cost,
            packet_digest: packet.digest().0,
        };
        let recovery = match hosted.recovery.seal(&delivery, &payload_digest.0) {
            Ok(recovery) => recovery,
            Err(_) => {
                let _ = hosted
                    .link(
                        "uncertain",
                        None,
                        serde_json::json!({
                            "request_id": wire.request_id, "owner_nonce": owner_nonce,
                        }),
                    )
                    .await;
                return unavailable(
                    StatusCode::SERVICE_UNAVAILABLE,
                    "routing_recovery_unavailable",
                );
            }
        };
        let settlement = serde_json::json!({
            "request_id": wire.request_id,
            "owner_nonce": owner_nonce,
            "actual_input_tokens": input_tokens,
            "actual_output_tokens": output_tokens,
            "actual_nano_usd": cost,
            "rate_card_revision": hosted.rate_card_revision,
            "receipt_id": receipt_id,
            "recovery": recovery,
        });
        let mut settled = hosted.link("settle", None, settlement.clone()).await;
        // This repeats only an idempotent Link ledger write, never the provider
        // send. Use the identical encrypted blob and owner nonce on retry.
        if matches!(settled, Err(StatusCode::SERVICE_UNAVAILABLE)) {
            settled = hosted.link("settle", None, settlement).await;
        }
        if !settled
            .is_ok_and(|reply| reply.state == "completed" && reply.request_id == wire.request_id)
        {
            return unavailable(
                StatusCode::SERVICE_UNAVAILABLE,
                "routing_settlement_unavailable",
            );
        }
        (
            StatusCode::OK,
            [(header::CACHE_CONTROL, "no-store")],
            Json(delivery),
        )
            .into_response()
    });
    match tokio::time::timeout(Duration::from_secs(2), completion).await {
        Ok(Ok(response)) => response,
        _ => unavailable(StatusCode::SERVICE_UNAVAILABLE, "routing_client_deadline"),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::body::Body;
    use axum::http::{HeaderMap, Request};
    use axum::routing::post;
    use axum::Router;
    use std::collections::HashMap;
    use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};
    use std::sync::Mutex;
    use tower::ServiceExt;

    #[test]
    fn output_price_may_be_explicitly_free_but_input_must_be_positive() {
        assert_eq!(pinned_price(Some("0"), false), Some(0));
        assert_eq!(pinned_price(Some("0"), true), None);
        assert_eq!(pinned_price(None, false), None);
        assert_eq!(pinned_price(Some("bad"), false), None);
        assert_eq!(pinned_price(Some("-1"), false), None);
    }

    #[test]
    fn staged_flow_proposal_fixture_passes_the_actual_proxy_validator() {
        // The routed Flow seam test compares its persisted review projection
        // with this shared fixture. Validate that exact semantic packet here
        // without a Link admission, hosted grant, or upstream send.
        let packet: WirePacket = serde_json::from_str(include_str!(
            "../tests/fixtures/proposed-hosted-packet-v1.json"
        ))
        .unwrap();
        let wire = WireRequest {
            schema_version: 1,
            request_id: "01994a00-0000-7abc-8abc-0123456789ab".into(),
            decision_kind: "initial_demand".into(),
            question_set_version: TEMPLATE_VERSION.into(),
            packet,
        };
        assert!(validate_wire(&wire).is_ok());
    }

    #[derive(Default)]
    struct FakeLink {
        operations: Mutex<HashMap<String, (String, Option<RecoveryBlob>)>>,
        fail_settle_once: AtomicBool,
    }

    fn reply(
        request_id: &str,
        state: &str,
        new_reservation: bool,
        nonce: Option<&str>,
        recovery: Option<&RecoveryBlob>,
    ) -> Json<serde_json::Value> {
        Json(serde_json::json!({
            "request_id": request_id,
            "state": state,
            "new_reservation": new_reservation,
            "owner_nonce": nonce,
            "recovery": recovery,
        }))
    }

    async fn fake_reserve(
        State(state): State<Arc<FakeLink>>,
        headers: HeaderMap,
        Json(body): Json<serde_json::Value>,
    ) -> Response {
        if headers
            .get("x-pytxo-routing-service")
            .and_then(|value| value.to_str().ok())
            != Some("service-secret-at-least-32-bytes-long")
            || headers
                .get(header::AUTHORIZATION)
                .and_then(|value| value.to_str().ok())
                != Some("Bearer pr1_1234567890123456789012345678901234567890123")
        {
            return StatusCode::UNAUTHORIZED.into_response();
        }
        let request_id = body["request_id"].as_str().unwrap();
        let mut ops = state.operations.lock().unwrap();
        let previous = ops.get(request_id).cloned();
        if previous.is_none() {
            ops.insert(request_id.into(), ("reserved".into(), None));
        }
        let (state, recovery) = ops.get(request_id).unwrap();
        reply(
            request_id,
            state,
            previous.is_none(),
            None,
            recovery.as_ref(),
        )
        .into_response()
    }

    async fn fake_claim(
        State(state): State<Arc<FakeLink>>,
        Json(body): Json<serde_json::Value>,
    ) -> Json<serde_json::Value> {
        let request_id = body["request_id"].as_str().unwrap();
        let mut ops = state.operations.lock().unwrap();
        let current = ops.get(request_id).cloned().unwrap_or_default();
        if current.0 == "reserved" {
            ops.insert(request_id.into(), ("sending".into(), None));
            reply(request_id, "sending", false, Some(&"ab".repeat(32)), None)
        } else {
            reply(request_id, &current.0, false, None, current.1.as_ref())
        }
    }

    async fn fake_settle(
        State(state): State<Arc<FakeLink>>,
        Json(body): Json<serde_json::Value>,
    ) -> Response {
        if state.fail_settle_once.swap(false, Ordering::SeqCst) {
            return StatusCode::SERVICE_UNAVAILABLE.into_response();
        }
        let request_id = body["request_id"].as_str().unwrap();
        state.operations.lock().unwrap().insert(
            request_id.into(),
            (
                "completed".into(),
                serde_json::from_value(body["recovery"].clone()).ok(),
            ),
        );
        reply(request_id, "completed", false, None, None).into_response()
    }

    async fn fake_uncertain(
        State(state): State<Arc<FakeLink>>,
        Json(body): Json<serde_json::Value>,
    ) -> Json<serde_json::Value> {
        let request_id = body["request_id"].as_str().unwrap();
        state
            .operations
            .lock()
            .unwrap()
            .insert(request_id.into(), ("uncertain".into(), None));
        reply(request_id, "uncertain", false, None, None)
    }

    struct CountingUpstream(AtomicUsize, Duration);
    impl RoutingUpstream for CountingUpstream {
        fn send<'a>(&'a self, _body: &'a [u8]) -> UpstreamFuture<'a> {
            Box::pin(async move {
                self.0.fetch_add(1, Ordering::SeqCst);
                tokio::time::sleep(self.1).await;
                Ok(br#"{"model":"jev-1.13.0","answers":{"execution_demand_v2":{"type":"choice","choice":"everyday_fit","probabilities":{"everyday_fit":0.8,"strong_needed":0.1,"unclear":0.1},"confidence":0.7}},"usage":{"input_tokens":42,"output_tokens":3}}"#.to_vec())
            })
        }
    }

    async fn test_app(
        delay: Duration,
    ) -> (
        Router,
        Arc<CountingUpstream>,
        Arc<FakeLink>,
        tokio::task::JoinHandle<()>,
    ) {
        let fake = Arc::new(FakeLink::default());
        let link = Router::new()
            .route("/internal/routing/reserve", post(fake_reserve))
            .route("/internal/routing/claim", post(fake_claim))
            .route("/internal/routing/settle", post(fake_settle))
            .route("/internal/routing/uncertain", post(fake_uncertain))
            .with_state(fake.clone());
        let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
        let base = format!("http://{}", listener.local_addr().unwrap());
        let server = tokio::spawn(async move {
            axum::serve(listener, link).await.unwrap();
        });
        let upstream = Arc::new(CountingUpstream(AtomicUsize::new(0), delay));
        let hosted = HostedRouting {
            client: reqwest::Client::builder().no_proxy().build().unwrap(),
            link_base: base,
            service_key: "service-secret-at-least-32-bytes-long".into(),
            upstream: upstream.clone(),
            rate_card_revision: "test-v1".into(),
            input_nano_usd_per_token: 4,
            output_nano_usd_per_token: 20,
            recovery: RecoveryCipher {
                key: [7u8; 32],
                revision: "test-key/v1".into(),
                previous: None,
            },
        };
        let state = Arc::new(AppState {
            client: reqwest::Client::new(),
            require_auth: false,
            link_base: None,
            link_api_key: None,
            rate_limiter: Arc::new(crate::RateLimiter::from_env()),
            hosted_routing: Some(hosted),
        });
        (crate::build_router(state), upstream, fake, server)
    }

    fn valid_request() -> Vec<u8> {
        serde_json::to_vec(&serde_json::json!({
            "schema_version": 1,
            "request_id": "01994a00-0000-7abc-8abc-0123456789ab",
            "decision_kind": "initial_demand",
            "question_set_version": TEMPLATE_VERSION,
            "packet": {
                "schema_version": 1,
                "goal": FIXED_GOAL,
                "features": [
                    "task_kind_documentation", "reviewed_checks",
                    "single_component_claimed", "context_claimed_complete",
                    "role_unrestricted", "no_required_egress",
                    "repeatable_symptom_unknown", "specific_cause_hypothesis_unknown"
                ],
                "everyday_role": EVERYDAY_ROLE,
                "strong_role": STRONG_ROLE
            }
        }))
        .unwrap()
    }

    fn post_body(body: Vec<u8>) -> Request<Body> {
        Request::builder()
            .method("POST")
            .uri("/v1/routing/evaluations")
            .header(
                header::AUTHORIZATION,
                "Bearer pr1_1234567890123456789012345678901234567890123",
            )
            .header(header::CONTENT_TYPE, "application/json")
            .body(Body::from(body))
            .unwrap()
    }

    #[tokio::test]
    async fn fixed_packet_is_bounded_and_one_claim_means_one_send() {
        let (app, upstream, fake, server) = test_app(Duration::ZERO).await;
        let absent = app
            .clone()
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/v1/routing/evaluations")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(absent.status(), StatusCode::UNAUTHORIZED);
        let too_large = app
            .clone()
            .oneshot(post_body(vec![b' '; MAX_CLIENT_BYTES + 1]))
            .await
            .unwrap();
        assert_eq!(too_large.status(), StatusCode::PAYLOAD_TOO_LARGE);
        let duplicate_key = br#"{"schema_version":1,"schema_version":1}"#.to_vec();
        assert_eq!(
            app.clone()
                .oneshot(post_body(duplicate_key))
                .await
                .unwrap()
                .status(),
            StatusCode::BAD_REQUEST
        );
        let mut malicious: serde_json::Value = serde_json::from_slice(&valid_request()).unwrap();
        malicious["packet"]["goal"] = serde_json::json!("Read C:/secret/file");
        assert_eq!(
            app.clone()
                .oneshot(post_body(serde_json::to_vec(&malicious).unwrap()))
                .await
                .unwrap()
                .status(),
            StatusCode::BAD_REQUEST
        );
        let mut invented_cue: serde_json::Value = serde_json::from_slice(&valid_request()).unwrap();
        invented_cue["packet"]["features"][6] =
            serde_json::json!("repeatable_symptom_send_private_details");
        assert_eq!(
            app.clone()
                .oneshot(post_body(serde_json::to_vec(&invented_cue).unwrap()))
                .await
                .unwrap()
                .status(),
            StatusCode::BAD_REQUEST
        );
        let mut extra_field: serde_json::Value = serde_json::from_slice(&valid_request()).unwrap();
        extra_field["packet"]["task_text"] = serde_json::json!("private details");
        assert_eq!(
            app.clone()
                .oneshot(post_body(serde_json::to_vec(&extra_field).unwrap()))
                .await
                .unwrap()
                .status(),
            StatusCode::BAD_REQUEST
        );
        assert_eq!(upstream.0.load(Ordering::SeqCst), 0);

        let payload = valid_request();
        let (a, b) = tokio::join!(
            app.clone().oneshot(post_body(payload.clone())),
            app.clone().oneshot(post_body(payload.clone())),
        );
        let (a, b) = (a.unwrap(), b.unwrap());
        let (success, rejected) = if a.status() == StatusCode::OK {
            (a, b)
        } else {
            (b, a)
        };
        assert_eq!(success.status(), StatusCode::OK);
        assert_eq!(rejected.status(), StatusCode::CONFLICT);
        let original = body::to_bytes(success.into_body(), 4096).await.unwrap();
        assert_eq!(upstream.0.load(Ordering::SeqCst), 1);
        let replay = app
            .clone()
            .oneshot(post_body(payload.clone()))
            .await
            .unwrap();
        assert_eq!(replay.status(), StatusCode::OK);
        assert_eq!(
            replay.headers().get(header::CACHE_CONTROL).unwrap(),
            "no-store"
        );
        assert_eq!(
            body::to_bytes(replay.into_body(), 4096).await.unwrap(),
            original
        );
        assert_eq!(upstream.0.load(Ordering::SeqCst), 1);
        assert_eq!(fake.operations.lock().unwrap().len(), 1);
        let request_id: String = serde_json::from_slice::<serde_json::Value>(&payload).unwrap()
            ["request_id"]
            .as_str()
            .unwrap()
            .into();
        fake.operations
            .lock()
            .unwrap()
            .get_mut(&request_id)
            .unwrap()
            .1
            .as_mut()
            .unwrap()
            .ciphertext = URL_SAFE_NO_PAD.encode([1u8; 40]);
        assert_eq!(
            app.oneshot(post_body(payload)).await.unwrap().status(),
            StatusCode::SERVICE_UNAVAILABLE
        );
        assert_eq!(upstream.0.load(Ordering::SeqCst), 1);
        server.abort();
    }

    #[tokio::test]
    async fn client_deadline_does_not_cancel_claimed_settlement() {
        let (app, upstream, fake, server) = test_app(Duration::from_millis(2200)).await;
        let packet = valid_request();
        let response = app
            .clone()
            .oneshot(post_body(packet.clone()))
            .await
            .unwrap();
        assert_eq!(response.status(), StatusCode::SERVICE_UNAVAILABLE);
        assert_eq!(upstream.0.load(Ordering::SeqCst), 1);
        tokio::time::sleep(Duration::from_millis(450)).await;
        let request_id: String = serde_json::from_slice::<serde_json::Value>(&packet).unwrap()
            ["request_id"]
            .as_str()
            .unwrap()
            .into();
        assert_eq!(fake.operations.lock().unwrap()[&request_id].0, "completed");
        let replay = app.oneshot(post_body(packet)).await.unwrap();
        assert_eq!(replay.status(), StatusCode::OK);
        assert_eq!(upstream.0.load(Ordering::SeqCst), 1);
        server.abort();
    }

    #[tokio::test]
    async fn identical_link_settlement_retry_never_resends_provider_request() {
        let (app, upstream, fake, server) = test_app(Duration::ZERO).await;
        fake.fail_settle_once.store(true, Ordering::SeqCst);
        let response = app.oneshot(post_body(valid_request())).await.unwrap();
        assert_eq!(response.status(), StatusCode::OK);
        assert_eq!(upstream.0.load(Ordering::SeqCst), 1);
        assert_eq!(fake.operations.lock().unwrap().len(), 1);
        server.abort();
    }

    #[test]
    fn previous_recovery_key_can_read_unexpired_receipt() {
        let old = RecoveryCipher {
            key: [1u8; 32],
            revision: "old/v1".into(),
            previous: None,
        };
        let delivery = Delivery {
            schema_version: 1,
            evaluation_id: "request".into(),
            request_id: "request".into(),
            question_set_version: TEMPLATE_VERSION.into(),
            model_id: MODEL_ID.into(),
            choice: AdviceChoice::Unclear,
            distribution: AdviceDistribution {
                everyday_fit: 0.2,
                strong_needed: 0.3,
                unclear: 0.5,
            },
            usage_status: "known".into(),
            usage_receipt_id: "receipt".into(),
            input_tokens: 4,
            output_tokens: 2,
            cost_nano_usd: 12,
            packet_digest: "a".repeat(64),
        };
        let blob = old.seal(&delivery, &"b".repeat(64)).unwrap();
        let rotated = RecoveryCipher {
            key: [2u8; 32],
            revision: "new/v2".into(),
            previous: Some(([1u8; 32], "old/v1".into())),
        };
        assert!(rotated.open(&blob, "request", &"b".repeat(64)).is_ok());
        assert!(rotated.open(&blob, "other", &"b".repeat(64)).is_err());
        assert!(rotated.open(&blob, "request", &"c".repeat(64)).is_err());
        assert!(RecoveryCipher {
            previous: None,
            ..rotated
        }
        .open(&blob, "request", &"b".repeat(64))
        .is_err());
    }
}
