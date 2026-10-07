//! Staged routing-only HTTP client. No Desktop dispatch path constructs it yet.

use std::time::Duration;

use base64::engine::general_purpose::URL_SAFE_NO_PAD;
use base64::Engine;
use chrono::{DateTime, Utc};
use pytxo_orchestrate::{HostedClientFuture, HostedEvaluationRequest, HostedShadowClient};
use pytxo_planner::advisor::{
    parse_hosted_evaluation_receipt, UnavailableReason, HOSTED_RECIPIENT, TEMPLATE_VERSION,
};
use reqwest::{header, redirect, Client, Response, StatusCode, Url};
use serde::Deserialize;

const PROXY_ORIGIN: &str = "https://proxy.pytxo.com";
const MAX_TOKEN_REPLY_BYTES: usize = 1024;
const MAX_EVALUATION_REPLY_BYTES: usize = 2048;
const MAX_EVALUATION_REQUEST_BYTES: usize = 16 * 1024;

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct TokenReply {
    token: String,
    token_type: String,
    scope: String,
    expires_at: DateTime<Utc>,
}

fn secret_has_32_bytes(value: &str, prefix: &str) -> bool {
    value.strip_prefix(prefix).is_some_and(|encoded| {
        encoded.len() == 43
            && URL_SAFE_NO_PAD
                .decode(encoded)
                .ok()
                .is_some_and(|bytes| bytes.len() == 32 && URL_SAFE_NO_PAD.encode(bytes) == encoded)
    })
}

fn checked_origin(value: &str, proxy: bool) -> Result<String, UnavailableReason> {
    let url = Url::parse(value).map_err(|_| UnavailableReason::InvalidConfiguration)?;
    let loopback = cfg!(debug_assertions)
        && url.scheme() == "http"
        && matches!(url.host_str(), Some("127.0.0.1" | "localhost"));
    if (url.scheme() != "https" && !loopback)
        || (proxy && value != PROXY_ORIGIN && !loopback)
        || url.host_str().is_none()
        || !matches!(url.path(), "" | "/")
        || url.query().is_some()
        || url.fragment().is_some()
        || !url.username().is_empty()
        || url.password().is_some()
    {
        return Err(UnavailableReason::InvalidConfiguration);
    }
    Ok(url.as_str().trim_end_matches('/').to_owned())
}

async fn read_json_bounded(
    mut response: Response,
    max_bytes: usize,
) -> Result<Vec<u8>, UnavailableReason> {
    if response
        .headers()
        .get(header::CONTENT_TYPE)
        .and_then(|value| value.to_str().ok())
        .and_then(|value| value.split(';').next())
        != Some("application/json")
    {
        return Err(UnavailableReason::Schema);
    }
    let mut bytes = Vec::new();
    while let Some(chunk) = response
        .chunk()
        .await
        .map_err(|_| UnavailableReason::Network)?
    {
        if chunk.len() > max_bytes - bytes.len() {
            return Err(UnavailableReason::Schema);
        }
        bytes.extend_from_slice(&chunk);
    }
    Ok(bytes)
}

fn request_matches_binding(
    request: &HostedEvaluationRequest,
    workspace_id: &str,
    revision: u64,
) -> bool {
    if request.workspace_id != workspace_id
        || request.grant_revision != revision
        || request.body.is_empty()
        || request.body.len() > MAX_EVALUATION_REQUEST_BYTES
        || !request.packet_digest.is_valid()
    {
        return false;
    }
    let Ok(wire) = serde_json::from_slice::<serde_json::Value>(&request.body) else {
        return false;
    };
    wire.get("schema_version") == Some(&serde_json::json!(1))
        && wire.get("request_id").and_then(serde_json::Value::as_str)
            == Some(request.request_id.as_str())
        && wire
            .get("decision_kind")
            .and_then(serde_json::Value::as_str)
            == Some("initial_demand")
        && wire
            .get("question_set_version")
            .and_then(serde_json::Value::as_str)
            == Some(TEMPLATE_VERSION)
        && wire.get("packet").is_some_and(serde_json::Value::is_object)
}

/// Holds only the routing Desktop session, never a worker credential. The
/// caller must first verify the session and bind this client to a confirmed
/// reviewed grant; Link repeats that check when it issues and claims a token.
pub(crate) struct HostedHttpClient {
    http: Client,
    account_id: String,
    desktop_session_token: String,
    link_origin: String,
    proxy_origin: String,
    workspace_id: String,
    grant_revision: u64,
}

impl HostedHttpClient {
    #[allow(
        clippy::too_many_arguments,
        reason = "all binding fields are checked together"
    )]
    pub(crate) fn new(
        account_id: String,
        desktop_session_token: String,
        session_expires_at: DateTime<Utc>,
        link_origin: String,
        proxy_origin: String,
        workspace_id: String,
        grant_revision: u64,
    ) -> Result<Self, UnavailableReason> {
        if !account_id.starts_with("user_")
            || !secret_has_32_bytes(&desktop_session_token, "pds1_")
            || session_expires_at <= Utc::now()
            || workspace_id.len() != 32
            || !workspace_id
                .bytes()
                .all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte))
            || grant_revision == 0
        {
            return Err(UnavailableReason::InvalidConfiguration);
        }
        let link_origin = checked_origin(&link_origin, false)?;
        let proxy_origin = checked_origin(&proxy_origin, true)?;
        let http = Client::builder()
            .redirect(redirect::Policy::none())
            .no_proxy()
            .retry(reqwest::retry::never())
            .timeout(Duration::from_millis(1800))
            .connect_timeout(Duration::from_millis(500))
            .build()
            .map_err(|_| UnavailableReason::InvalidConfiguration)?;
        Ok(Self {
            http,
            account_id,
            desktop_session_token,
            link_origin,
            proxy_origin,
            workspace_id,
            grant_revision,
        })
    }

    async fn evaluate_once(
        &self,
        request: &HostedEvaluationRequest,
    ) -> Result<Vec<u8>, UnavailableReason> {
        if !request_matches_binding(request, &self.workspace_id, self.grant_revision) {
            return Err(UnavailableReason::InvalidPacket);
        }
        let issue = self
            .http
            .post(format!("{}/v1/routing/tokens", self.link_origin))
            .bearer_auth(&self.desktop_session_token)
            .json(&serde_json::json!({
                "workspace_id": self.workspace_id,
                "expected_grant_revision": self.grant_revision,
            }))
            .send()
            .await
            .map_err(|_| UnavailableReason::Network)?;
        if issue.status() != StatusCode::OK {
            return Err(match issue.status() {
                StatusCode::UNAUTHORIZED | StatusCode::FORBIDDEN => {
                    UnavailableReason::Authentication
                }
                StatusCode::TOO_MANY_REQUESTS => UnavailableReason::NoPermit,
                _ => UnavailableReason::Upstream,
            });
        }
        let token_bytes = read_json_bounded(issue, MAX_TOKEN_REPLY_BYTES).await?;
        let token: TokenReply =
            serde_json::from_slice(&token_bytes).map_err(|_| UnavailableReason::Schema)?;
        if !secret_has_32_bytes(&token.token, "pr1_")
            || token.token_type != "Bearer"
            || token.scope != "routing:evaluate:v1"
            || token.expires_at <= Utc::now()
        {
            return Err(UnavailableReason::Schema);
        }
        let evaluation = self
            .http
            .post(format!("{}/v1/routing/evaluations", self.proxy_origin))
            .bearer_auth(&token.token)
            .header(header::CONTENT_TYPE, "application/json")
            .body(request.body.clone())
            .send()
            .await
            .map_err(|_| UnavailableReason::Network)?;
        if evaluation.status() != StatusCode::OK {
            return Err(match evaluation.status() {
                StatusCode::UNAUTHORIZED | StatusCode::FORBIDDEN => {
                    UnavailableReason::Authentication
                }
                StatusCode::TOO_MANY_REQUESTS => UnavailableReason::NoPermit,
                _ => UnavailableReason::Upstream,
            });
        }
        let bytes = read_json_bounded(evaluation, MAX_EVALUATION_REPLY_BYTES).await?;
        parse_hosted_evaluation_receipt(&bytes, &request.request_id, &request.packet_digest)?;
        Ok(bytes)
    }
}

impl HostedShadowClient for HostedHttpClient {
    fn recipient_identity(&self) -> &str {
        HOSTED_RECIPIENT
    }

    fn account_id(&self) -> &str {
        &self.account_id
    }

    fn link_origin(&self) -> &str {
        &self.link_origin
    }

    fn workspace_id(&self) -> &str {
        &self.workspace_id
    }

    fn grant_revision(&self) -> u64 {
        self.grant_revision
    }

    fn evaluate<'a>(&'a self, request: &'a HostedEvaluationRequest) -> HostedClientFuture<'a> {
        Box::pin(self.evaluate_once(request))
    }
}

#[cfg(test)]
mod tests {
    use std::sync::atomic::{AtomicU8, AtomicUsize, Ordering};
    use std::sync::Arc;

    use axum::body::Bytes;
    use axum::extract::State;
    use axum::http::{header, HeaderMap, StatusCode};
    use axum::response::{IntoResponse, Response};
    use axum::routing::post;
    use axum::{Json, Router};
    use chrono::{Duration, Utc};
    use pytxo_orchestrate::{HostedEvaluationRequest, HostedShadowClient};
    use pytxo_planner::advisor::{build_hosted_evaluation_request, AdvisorPacket};
    use serde_json::json;

    use super::HostedHttpClient;

    const REQUEST_ID: &str = "0199e4af-aaaa-7abc-8abc-0123456789ab";
    const WORKSPACE: &str = "000102030405060708090a0b0c0d0e0f";

    #[derive(Clone, Default)]
    struct Calls {
        tokens: Arc<AtomicUsize>,
        evaluations: Arc<AtomicUsize>,
        token_mode: Arc<AtomicU8>,
        evaluation_mode: Arc<AtomicU8>,
        redirects_followed: Arc<AtomicUsize>,
    }

    async fn token(
        State(calls): State<Calls>,
        headers: HeaderMap,
        Json(body): Json<serde_json::Value>,
    ) -> Response {
        calls.tokens.fetch_add(1, Ordering::SeqCst);
        assert_eq!(
            headers.get("authorization").unwrap(),
            &format!("Bearer pds1_{}", "A".repeat(43))
        );
        assert_eq!(
            body,
            json!({"workspace_id":WORKSPACE,"expected_grant_revision":7})
        );
        match calls.token_mode.load(Ordering::SeqCst) {
            1 => return StatusCode::UNAUTHORIZED.into_response(),
            2 => return (StatusCode::FOUND, [(header::LOCATION, "/trap")]).into_response(),
            3 => tokio::time::sleep(std::time::Duration::from_secs(2)).await,
            4 => tokio::time::sleep(std::time::Duration::from_millis(1200)).await,
            _ => {}
        }
        (StatusCode::OK, Json(json!({"token":format!("pr1_{}", "A".repeat(43)),"token_type":"Bearer","scope":"routing:evaluate:v1","expires_at":Utc::now()+Duration::minutes(3)}))).into_response()
    }

    async fn evaluate(State(calls): State<Calls>, headers: HeaderMap, body: Bytes) -> Response {
        calls.evaluations.fetch_add(1, Ordering::SeqCst);
        assert_eq!(
            headers.get("authorization").unwrap(),
            &format!("Bearer pr1_{}", "A".repeat(43))
        );
        let wire: serde_json::Value = serde_json::from_slice(&body).unwrap();
        assert_eq!(wire["request_id"], REQUEST_ID);
        match calls.evaluation_mode.load(Ordering::SeqCst) {
            1 => return (StatusCode::OK, Json(json!({"choice":"strong_needed"}))).into_response(),
            2 => {
                return (
                    StatusCode::OK,
                    [(header::CONTENT_TYPE, "application/json")],
                    vec![b'X'; 2049],
                )
                    .into_response()
            }
            3 => tokio::time::sleep(std::time::Duration::from_millis(1200)).await,
            _ => {}
        }
        (
            StatusCode::OK,
            Json(json!({
                "schema_version":1,
                "evaluation_id":REQUEST_ID,
                "request_id":REQUEST_ID,
                "question_set_version":"execution_demand_v2",
                "model_id":"jev-1.13.0",
                "choice":"strong_needed",
                "distribution":{"everyday_fit":0.1,"strong_needed":0.8,"unclear":0.1},
                "usage_status":"known",
                "usage_receipt_id":format!("r_{}", "a".repeat(32)),
                "input_tokens":42,
                "output_tokens":3,
                "cost_nano_usd":228,
                "packet_digest":packet().digest().0,
            })),
        )
            .into_response()
    }

    async fn trap(State(calls): State<Calls>) -> StatusCode {
        calls.redirects_followed.fetch_add(1, Ordering::SeqCst);
        StatusCode::OK
    }

    async fn server() -> (String, Calls, tokio::task::JoinHandle<()>) {
        let calls = Calls::default();
        let router = Router::new()
            .route("/v1/routing/tokens", post(token))
            .route("/v1/routing/evaluations", post(evaluate))
            .route("/trap", post(trap))
            .with_state(calls.clone());
        let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
        let origin = format!("http://{}", listener.local_addr().unwrap());
        let task = tokio::spawn(async move {
            axum::serve(listener, router).await.unwrap();
        });
        (origin, calls, task)
    }

    fn packet() -> AdvisorPacket {
        AdvisorPacket::new(
            "Classify reviewed repository task using coarse facts",
            ["task_kind_diagnosis"],
            "everyday execution role",
            "strong execution role",
        )
        .unwrap()
    }

    fn request(workspace_id: &str) -> HostedEvaluationRequest {
        let packet = packet();
        HostedEvaluationRequest {
            body: build_hosted_evaluation_request(REQUEST_ID, &packet).unwrap(),
            request_id: REQUEST_ID.into(),
            packet_digest: packet.digest(),
            workspace_id: workspace_id.into(),
            grant_revision: 7,
        }
    }

    fn client(origin: &str) -> HostedHttpClient {
        HostedHttpClient::new(
            "user_test".into(),
            format!("pds1_{}", "A".repeat(43)),
            Utc::now() + Duration::minutes(4),
            origin.into(),
            origin.into(),
            WORKSPACE.into(),
            7,
        )
        .unwrap()
    }

    #[tokio::test]
    async fn exact_workspace_token_is_issued_once_before_one_evaluation() {
        let (origin, calls, task) = server().await;
        let reply = client(&origin).evaluate(&request(WORKSPACE)).await.unwrap();
        assert_eq!(
            serde_json::from_slice::<serde_json::Value>(&reply).unwrap()["choice"],
            "strong_needed"
        );
        assert_eq!(calls.tokens.load(Ordering::SeqCst), 1);
        assert_eq!(calls.evaluations.load(Ordering::SeqCst), 1);
        task.abort();
    }

    #[tokio::test]
    async fn wrong_workspace_never_requests_a_token_or_evaluation() {
        let (origin, calls, task) = server().await;
        assert!(client(&origin)
            .evaluate(&request("ffffffffffffffffffffffffffffffff"))
            .await
            .is_err());
        assert_eq!(calls.tokens.load(Ordering::SeqCst), 0);
        assert_eq!(calls.evaluations.load(Ordering::SeqCst), 0);
        task.abort();
    }

    #[tokio::test]
    async fn wrong_revision_never_requests_a_token_or_evaluation() {
        let (origin, calls, task) = server().await;
        let mut request = request(WORKSPACE);
        request.grant_revision = 8;
        assert!(client(&origin).evaluate(&request).await.is_err());
        assert_eq!(calls.tokens.load(Ordering::SeqCst), 0);
        assert_eq!(calls.evaluations.load(Ordering::SeqCst), 0);
        task.abort();
    }

    #[tokio::test]
    async fn revoked_session_at_link_cannot_reach_proxy() {
        let (origin, calls, task) = server().await;
        calls.token_mode.store(1, Ordering::SeqCst);
        assert!(client(&origin).evaluate(&request(WORKSPACE)).await.is_err());
        assert_eq!(calls.tokens.load(Ordering::SeqCst), 1);
        assert_eq!(calls.evaluations.load(Ordering::SeqCst), 0);
        task.abort();
    }

    #[tokio::test]
    async fn token_redirect_is_not_followed() {
        let (origin, calls, task) = server().await;
        calls.token_mode.store(2, Ordering::SeqCst);
        assert!(client(&origin).evaluate(&request(WORKSPACE)).await.is_err());
        assert_eq!(calls.tokens.load(Ordering::SeqCst), 1);
        assert_eq!(calls.evaluations.load(Ordering::SeqCst), 0);
        assert_eq!(calls.redirects_followed.load(Ordering::SeqCst), 0);
        task.abort();
    }

    #[tokio::test]
    async fn malformed_or_oversized_proxy_reply_is_not_returned_as_advice() {
        let (origin, calls, task) = server().await;
        for mode in [1, 2] {
            calls.evaluation_mode.store(mode, Ordering::SeqCst);
            assert!(client(&origin).evaluate(&request(WORKSPACE)).await.is_err());
        }
        assert_eq!(calls.tokens.load(Ordering::SeqCst), 2);
        assert_eq!(calls.evaluations.load(Ordering::SeqCst), 2);
        task.abort();
    }

    #[tokio::test]
    async fn token_timeout_never_starts_proxy_evaluation() {
        let (origin, calls, task) = server().await;
        calls.token_mode.store(3, Ordering::SeqCst);
        assert!(client(&origin).evaluate(&request(WORKSPACE)).await.is_err());
        assert_eq!(calls.tokens.load(Ordering::SeqCst), 1);
        assert_eq!(calls.evaluations.load(Ordering::SeqCst), 0);
        task.abort();
    }

    #[tokio::test]
    async fn outer_deadline_cancels_two_slow_successful_hops_without_another_send() {
        let (origin, calls, task) = server().await;
        calls.token_mode.store(4, Ordering::SeqCst);
        calls.evaluation_mode.store(3, Ordering::SeqCst);
        let client = client(&origin);
        let request = request(WORKSPACE);
        let observed =
            tokio::time::timeout(std::time::Duration::from_secs(2), client.evaluate(&request))
                .await;
        assert!(observed.is_err());
        assert_eq!(calls.tokens.load(Ordering::SeqCst), 1);
        assert_eq!(calls.evaluations.load(Ordering::SeqCst), 1);
        task.abort();
    }

    #[test]
    fn expired_session_and_untrusted_proxy_origin_fail_before_client_creation() {
        let session = format!("pds1_{}", "A".repeat(43));
        let expired = HostedHttpClient::new(
            "user_test".into(),
            session.clone(),
            Utc::now() - Duration::seconds(1),
            "https://link.pytxo.com".into(),
            "https://proxy.pytxo.com".into(),
            WORKSPACE.into(),
            7,
        );
        assert!(expired.is_err());
        let wrong_proxy = HostedHttpClient::new(
            "user_test".into(),
            session,
            Utc::now() + Duration::minutes(4),
            "https://link.pytxo.com".into(),
            "https://attacker.example".into(),
            WORKSPACE.into(),
            7,
        );
        assert!(wrong_proxy.is_err());
    }
}
