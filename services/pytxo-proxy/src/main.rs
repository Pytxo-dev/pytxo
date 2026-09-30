//! Pytxo Ultra managed-inference proxy (Phase 41, ADR-0019).

//!

//! Forwards provider API traffic so Ultra clients never hold provider keys locally.

mod metering;
mod routing;
mod telemetry;

use std::collections::HashMap;

use std::pin::Pin;

use std::sync::{Arc, Mutex};

use std::task::{Context, Poll};

use std::time::Instant;

use axum::body::Body;

use axum::extract::{Request, State};

use axum::http::{HeaderMap, StatusCode};

use axum::response::{IntoResponse, Json, Response};

use axum::routing::{any, get, post};

use axum::Router;

use bytes::Bytes;

use futures_util::Stream;

use futures_util::StreamExt;

use reqwest::header::{HeaderName, HeaderValue};

use serde::Deserialize;

use tracing::info;

#[derive(Clone)]

struct AppState {
    client: reqwest::Client,

    require_auth: bool,

    link_base: Option<String>,

    link_api_key: Option<String>,

    rate_limiter: Arc<RateLimiter>,
    hosted_routing: Option<routing::HostedRouting>,
}

#[derive(Debug, Deserialize)]

struct LinkEntitlements {
    tier: Option<String>,
}

struct TokenBucket {
    tokens: f64,

    last_refill: Instant,

    rate: f64,

    burst: f64,
}

impl TokenBucket {
    fn new(rate: f64, burst: f64) -> Self {
        Self {
            tokens: burst,

            last_refill: Instant::now(),

            rate,

            burst,
        }
    }

    fn refill(&mut self) {
        let now = Instant::now();

        let elapsed = now.duration_since(self.last_refill).as_secs_f64();

        self.tokens = (self.tokens + elapsed * self.rate).min(self.burst);

        self.last_refill = now;
    }

    fn try_consume(&mut self, amount: f64) -> bool {
        self.refill();

        if self.tokens >= amount {
            self.tokens -= amount;

            true
        } else {
            false
        }
    }
}

struct RateLimiter {
    buckets: Mutex<HashMap<String, TokenBucket>>,

    rate: f64,

    burst: f64,
}

impl RateLimiter {
    fn from_env() -> Self {
        let rate = std::env::var("PROXY_RATE_LIMIT_RPS")
            .ok()
            .and_then(|s| s.parse().ok())
            .unwrap_or(60.0);

        let burst = std::env::var("PROXY_RATE_BURST")
            .ok()
            .and_then(|s| s.parse().ok())
            .unwrap_or(10.0);

        Self {
            buckets: Mutex::new(HashMap::new()),

            rate,

            burst,
        }
    }

    fn allow(&self, key: &str) -> bool {
        let mut buckets = self.buckets.lock().unwrap();

        let bucket = buckets
            .entry(key.to_string())
            .or_insert_with(|| TokenBucket::new(self.rate, self.burst));

        bucket.try_consume(1.0)
    }
}

struct MeteringStream<S> {
    inner: S,

    buffer: Vec<u8>,

    provider: String,

    bearer: String,

    domain_id: Option<String>,

    run_id: Option<String>,

    link_base: Option<String>,

    client: reqwest::Client,

    finished: bool,
}

impl<S> Stream for MeteringStream<S>
where
    S: Stream<Item = Result<Bytes, std::io::Error>> + Unpin,
{
    type Item = Result<Bytes, std::io::Error>;

    fn poll_next(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        let this = self.as_mut().get_mut();

        match Pin::new(&mut this.inner).poll_next(cx) {
            Poll::Ready(Some(Ok(chunk))) => {
                this.buffer.extend_from_slice(&chunk);

                Poll::Ready(Some(Ok(chunk)))
            }

            Poll::Ready(Some(Err(e))) => Poll::Ready(Some(Err(e))),

            Poll::Ready(None) => {
                if !this.finished {
                    this.finished = true;

                    if let Some(usage) =
                        metering::parse_usage_from_sse(&this.buffer, &this.provider).or_else(|| {
                            metering::parse_usage_from_response(
                                &HeaderMap::new(),
                                &this.buffer,
                                &this.provider,
                            )
                        })
                    {
                        info!(

                            provider = %this.provider,

                            tokens_in = usage.tokens_in,

                            tokens_out = usage.tokens_out,

                            "stream usage metered"

                        );

                        if let Some(base) = this.link_base.clone() {
                            let client = this.client.clone();

                            let provider = this.provider.clone();

                            let bearer = this.bearer.clone();

                            let domain_id = this.domain_id.clone();

                            let run_id = this.run_id.clone();

                            tokio::spawn(async move {
                                metering::report_to_link(
                                    &client,
                                    &base,
                                    &bearer,
                                    &provider,
                                    &usage,
                                    domain_id.as_deref(),
                                    run_id.as_deref(),
                                )
                                .await;
                            });
                        }
                    }
                }

                Poll::Ready(None)
            }

            Poll::Pending => Poll::Pending,
        }
    }
}

fn listen_addr() -> String {
    if let Ok(port) = std::env::var("PORT") {
        if !port.is_empty() {
            return format!("0.0.0.0:{port}");
        }
    }

    std::env::var("PROXY_BIND").unwrap_or_else(|_| "127.0.0.1:8790".into())
}

fn provider_upstream(provider: &str) -> String {
    match provider {
        "anthropic" => "https://api.anthropic.com".into(),

        "openai" => "https://api.openai.com".into(),

        "google" => "https://generativelanguage.googleapis.com".into(),

        "deepseek" => "https://api.deepseek.com".into(),

        "openrouter" => "https://openrouter.ai/api".into(),

        "agy" => {
            std::env::var("AGY_UPSTREAM").unwrap_or_else(|_| "https://api.anthropic.com".into())
        }

        _ => String::new(),
    }
}

fn provider_api_key(provider: &str) -> Option<String> {
    let key = match provider {
        "anthropic" | "agy" => std::env::var("ANTHROPIC_API_KEY").ok(),

        "openai" => std::env::var("OPENAI_API_KEY").ok(),

        "google" => std::env::var("GOOGLE_API_KEY").ok(),

        "deepseek" => std::env::var("DEEPSEEK_API_KEY").ok(),

        "openrouter" => std::env::var("OPENROUTER_API_KEY").ok(),

        _ => None,
    };

    key.filter(|k| !k.trim().is_empty())
}

async fn authorized(state: &AppState, headers: &HeaderMap) -> bool {
    if !state.require_auth {
        return true;
    }

    let bearer = headers
        .get("authorization")
        .and_then(|v| v.to_str().ok())
        .and_then(|s| s.strip_prefix("Bearer "))
        .unwrap_or("");

    if bearer.is_empty() {
        return false;
    }

    if let Some(expected) = &state.link_api_key {
        if bearer == expected {
            return true;
        }
    }

    if let Some(base) = &state.link_base {
        let url = format!("{}/v1/entitlements/status", base.trim_end_matches('/'));

        let mut req = state
            .client
            .get(&url)
            .header("Authorization", format!("Bearer {bearer}"));

        if let Ok(uid) = std::env::var("PYTXO_ULTRA_USER_ID") {
            if !uid.is_empty() {
                req = req.header("x-pytxo-user-id", uid);
            }
        }

        if let Ok(resp) = req.send().await {
            if resp.status().is_success() {
                if let Ok(body) = resp.json::<LinkEntitlements>().await {
                    return matches!(
                        body.tier.as_deref(),
                        Some("ultra") | Some("max") | Some("pro")
                    );
                }
            }
        }

        return false;
    }

    !bearer.is_empty()
}

fn bearer_key(headers: &HeaderMap) -> String {
    headers
        .get("authorization")
        .and_then(|v| v.to_str().ok())
        .and_then(|s| s.strip_prefix("Bearer "))
        .unwrap_or("anonymous")
        .to_string()
}

async fn health() -> Json<telemetry::HealthBody> {
    Json(telemetry::health_body())
}

async fn proxy_dispatch(State(state): State<Arc<AppState>>, request: Request) -> Response {
    let path = request.uri().path();

    // The sponsored namespace never enters the paid provider pass-through.
    if path == "/v1/routing" || path.starts_with("/v1/routing/") {
        return StatusCode::NOT_FOUND.into_response();
    }

    if path == "/health" || path == "/" {
        return health().await.into_response();
    }

    let provider = path.trim_start_matches('/').split('/').next().unwrap_or("");

    let allowed = [
        "anthropic",
        "openai",
        "google",
        "deepseek",
        "openrouter",
        "agy",
    ];

    if !allowed.contains(&provider) {
        return StatusCode::NOT_FOUND.into_response();
    }

    proxy_provider(State(state), provider.to_string(), request).await
}

fn report_usage_async(
    state: &AppState,

    provider: &str,

    usage: metering::ProviderUsage,

    bearer: &str,

    domain_id: Option<&str>,

    run_id: Option<&str>,
) {
    if let Some(base) = state.link_base.clone() {
        let client = state.client.clone();

        let provider_s = provider.to_string();

        let bearer_s = bearer.to_string();

        let domain_s = domain_id.map(str::to_string);

        let run_s = run_id.map(str::to_string);

        tokio::spawn(async move {
            metering::report_to_link(
                &client,
                &base,
                &bearer_s,
                &provider_s,
                &usage,
                domain_s.as_deref(),
                run_s.as_deref(),
            )
            .await;
        });
    }
}

async fn proxy_provider(
    State(state): State<Arc<AppState>>,

    provider: String,

    request: Request,
) -> Response {
    let headers = request.headers().clone();

    if !authorized(&state, &headers).await {
        return StatusCode::UNAUTHORIZED.into_response();
    }

    let rate_key = bearer_key(&headers);

    if !state.rate_limiter.allow(&rate_key) {
        return (
            StatusCode::TOO_MANY_REQUESTS,
            "rate limit exceeded for bearer token",
        )
            .into_response();
    }

    let upstream_base = provider_upstream(&provider);

    if upstream_base.is_empty() {
        return StatusCode::NOT_FOUND.into_response();
    }

    let Some(api_key) = provider_api_key(&provider) else {
        return (
            StatusCode::SERVICE_UNAVAILABLE,
            format!("provider {provider} not configured on proxy"),
        )
            .into_response();
    };

    let path_and_query = request
        .uri()
        .path_and_query()
        .map(|p| p.as_str())
        .unwrap_or("/");

    let suffix = path_and_query
        .strip_prefix(&format!("/{provider}"))
        .unwrap_or(path_and_query);

    let target = format!(
        "{}{}",
        upstream_base.trim_end_matches('/'),
        if suffix.starts_with('/') {
            suffix.to_string()
        } else {
            format!("/{suffix}")
        }
    );

    info!(provider = %provider, target = %target, "proxy forward");

    let method = request.method().clone();

    let body_bytes = match axum::body::to_bytes(request.into_body(), usize::MAX).await {
        Ok(b) => b,

        Err(_) => return StatusCode::BAD_REQUEST.into_response(),
    };

    let mut req = state.client.request(method, &target).body(body_bytes);

    req = req.header("Authorization", format!("Bearer {api_key}"));

    if provider == "anthropic" || provider == "agy" {
        req = req.header("x-api-key", &api_key);

        req = req.header("anthropic-version", "2023-06-01");
    }

    for (name, value) in headers.iter() {
        let n = name.as_str();

        if n.eq_ignore_ascii_case("host")
            || n.eq_ignore_ascii_case("authorization")
            || n.eq_ignore_ascii_case("content-length")
        {
            continue;
        }

        if let (Ok(hn), Ok(hv)) = (
            HeaderName::from_bytes(n.as_bytes()),
            HeaderValue::from_bytes(value.as_bytes()),
        ) {
            req = req.header(hn, hv);
        }
    }

    match req.send().await {
        Ok(resp) => {
            let status =
                StatusCode::from_u16(resp.status().as_u16()).unwrap_or(StatusCode::BAD_GATEWAY);

            let is_stream = resp
                .headers()
                .get("content-type")
                .and_then(|v| v.to_str().ok())
                .map(|ct| ct.contains("event-stream"))
                .unwrap_or(false);

            let bearer = bearer_key(&headers);

            let domain_id = headers
                .get("x-pytxo-domain-id")
                .and_then(|v| v.to_str().ok())
                .map(str::to_string);

            let run_id = headers
                .get("x-pytxo-run-id")
                .and_then(|v| v.to_str().ok())
                .map(str::to_string);

            if is_stream {
                let mut builder = Response::builder().status(status);

                if let Some(headers_out) = builder.headers_mut() {
                    for (k, v) in resp.headers().iter() {
                        if k != "transfer-encoding" {
                            headers_out.insert(k, v.clone());
                        }
                    }
                }

                let stream = MeteringStream {
                    inner: resp
                        .bytes_stream()
                        .map(|r| r.map_err(std::io::Error::other)),

                    buffer: Vec::new(),

                    provider: provider.clone(),

                    bearer: bearer.clone(),

                    domain_id: domain_id.clone(),

                    run_id: run_id.clone(),

                    link_base: state.link_base.clone(),

                    client: state.client.clone(),

                    finished: false,
                };

                return builder
                    .body(Body::from_stream(stream))
                    .unwrap_or_else(|_| StatusCode::INTERNAL_SERVER_ERROR.into_response());
            }

            let resp_headers = resp.headers().clone();

            let body = match resp.bytes().await {
                Ok(b) => b,

                Err(e) => {
                    return (StatusCode::BAD_GATEWAY, format!("upstream read error: {e}"))
                        .into_response();
                }
            };

            if let Some(usage) =
                metering::parse_usage_from_response(&resp_headers, &body, &provider)
            {
                info!(

                    provider = %provider,

                    tokens_in = usage.tokens_in,

                    tokens_out = usage.tokens_out,

                    "provider usage metered"

                );

                report_usage_async(
                    &state,
                    &provider,
                    usage,
                    &bearer,
                    domain_id.as_deref(),
                    run_id.as_deref(),
                );
            }

            let mut builder = Response::builder().status(status);

            if let Some(headers_out) = builder.headers_mut() {
                for (k, v) in resp_headers.iter() {
                    if k != "transfer-encoding" {
                        headers_out.insert(k, v.clone());
                    }
                }
            }

            builder
                .body(Body::from(body))
                .unwrap_or_else(|_| StatusCode::INTERNAL_SERVER_ERROR.into_response())
        }

        Err(e) => (StatusCode::BAD_GATEWAY, format!("upstream error: {e}")).into_response(),
    }
}

#[tokio::main]

async fn main() {
    telemetry::init();

    let require_auth = std::env::var("PROXY_REQUIRE_AUTH")
        .map(|v| v == "1" || v.eq_ignore_ascii_case("true"))
        .unwrap_or(true);

    let state = Arc::new(AppState {
        client: reqwest::Client::new(),

        require_auth,

        link_base: std::env::var("LINK_BASE_URL")
            .ok()
            .filter(|s| !s.is_empty())
            .or_else(|| std::env::var("LINK_ADMIN_URL").ok()),

        link_api_key: std::env::var("LINK_API_KEY").ok().filter(|s| !s.is_empty()),

        rate_limiter: Arc::new(RateLimiter::from_env()),
        hosted_routing: routing::HostedRouting::from_env()
            .expect("refusing incomplete experimental hosted routing configuration"),
    });

    let app = build_router(state);

    let addr = listen_addr();

    let listener = tokio::net::TcpListener::bind(&addr)
        .await
        .expect("bind proxy");

    info!("pytxo-proxy listening on http://{addr}");

    axum::serve(listener, app).await.expect("serve proxy");
}

fn build_router(state: Arc<AppState>) -> Router {
    Router::new()
        .route("/health", get(health))
        .route("/v1/routing/evaluations", post(routing::evaluate))
        .fallback(any(proxy_dispatch))
        .with_state(state)
}

#[cfg(test)]
mod tests {

    use super::*;
    use axum::body::Body;
    use axum::http::Request;
    use tower::ServiceExt;

    #[tokio::test]
    async fn sponsored_routing_never_falls_through_paid_proxy_auth() {
        let state = Arc::new(AppState {
            client: reqwest::Client::new(),
            require_auth: false,
            link_base: None,
            link_api_key: Some("paid-key".into()),
            rate_limiter: Arc::new(RateLimiter::from_env()),
            hosted_routing: None,
        });
        for (method, path, expected) in [
            (
                "POST",
                "/v1/routing/evaluations",
                StatusCode::SERVICE_UNAVAILABLE,
            ),
            (
                "GET",
                "/v1/routing/evaluations",
                StatusCode::METHOD_NOT_ALLOWED,
            ),
            ("POST", "/v1/routing/anything-else", StatusCode::NOT_FOUND),
        ] {
            let response = build_router(state.clone())
                .oneshot(
                    Request::builder()
                        .method(method)
                        .uri(path)
                        .header("authorization", "Bearer paid-key")
                        .body(Body::empty())
                        .unwrap(),
                )
                .await
                .unwrap();
            assert_eq!(response.status(), expected, "{method} {path}");
        }
    }

    #[test]

    fn anthropic_upstream_is_configured() {
        assert_eq!(provider_upstream("anthropic"), "https://api.anthropic.com");
    }

    #[test]

    fn deepseek_upstream_is_configured() {
        assert_eq!(provider_upstream("deepseek"), "https://api.deepseek.com");
    }

    #[test]

    fn health_lists_deepseek_when_key_set() {
        std::env::set_var("DEEPSEEK_API_KEY", "test-key-not-real");

        let providers = telemetry::configured_providers();

        assert!(providers.contains(&"deepseek"));

        std::env::remove_var("DEEPSEEK_API_KEY");
    }

    #[test]

    fn listen_addr_defaults_to_local() {
        std::env::remove_var("PORT");

        std::env::remove_var("PROXY_BIND");

        assert!(listen_addr().contains("8790"));
    }

    #[test]

    fn rate_limiter_blocks_burst() {
        let limiter = RateLimiter {
            buckets: Mutex::new(HashMap::new()),

            rate: 1.0,

            burst: 2.0,
        };

        assert!(limiter.allow("user-a"));

        assert!(limiter.allow("user-a"));

        assert!(!limiter.allow("user-a"));

        assert!(limiter.allow("user-b"));
    }
}
