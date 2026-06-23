//! Pytxo Ultra managed-inference proxy (Phase 41, ADR-0019).
//!
//! Forwards provider API traffic so Ultra clients never hold provider keys locally.

mod metering;

use std::sync::Arc;

use axum::body::Body;
use axum::extract::{Request, State};
use axum::http::{HeaderMap, StatusCode};
use axum::response::{IntoResponse, Response};
use axum::routing::{any, get};
use axum::Router;
use bytes::Bytes;
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
}

#[derive(Debug, Deserialize)]
struct LinkEntitlements {
    tier: Option<String>,
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
        "agy" => std::env::var("AGY_UPSTREAM").unwrap_or_else(|_| "https://api.anthropic.com".into()),
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
        let mut req = state.client.get(&url).header("Authorization", format!("Bearer {bearer}"));
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

async fn health() -> &'static str {
    "ok"
}

async fn proxy_dispatch(State(state): State<Arc<AppState>>, request: Request) -> Response {
    let path = request.uri().path();
    if path == "/health" || path == "/" {
        return health().await.into_response();
    }
    let provider = path
        .trim_start_matches('/')
        .split('/')
        .next()
        .unwrap_or("");
    let allowed = [
        "anthropic", "openai", "google", "deepseek", "openrouter", "agy",
    ];
    if !allowed.contains(&provider) {
        return StatusCode::NOT_FOUND.into_response();
    }
    proxy_provider(State(state), provider.to_string(), request).await
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
            let status = StatusCode::from_u16(resp.status().as_u16()).unwrap_or(StatusCode::BAD_GATEWAY);
            let is_stream = resp
                .headers()
                .get("content-type")
                .and_then(|v| v.to_str().ok())
                .map(|ct| ct.contains("event-stream"))
                .unwrap_or(false);

            let bearer = headers
                .get("authorization")
                .and_then(|v| v.to_str().ok())
                .and_then(|s| s.strip_prefix("Bearer "))
                .unwrap_or("");
            let domain_id = headers
                .get("x-pytxo-domain-id")
                .and_then(|v| v.to_str().ok());
            let run_id = headers
                .get("x-pytxo-run-id")
                .and_then(|v| v.to_str().ok());
            let link_base = state.link_base.clone();

            if is_stream {
                let mut builder = Response::builder().status(status);
                if let Some(headers_out) = builder.headers_mut() {
                    for (k, v) in resp.headers().iter() {
                        if k != "transfer-encoding" {
                            headers_out.insert(k, v.clone());
                        }
                    }
                }
                let stream = resp.bytes_stream().map(|r| {
                    r.map(Bytes::from).map_err(|e| std::io::Error::other(e))
                });
                return builder
                    .body(Body::from_stream(stream))
                    .unwrap_or_else(|_| StatusCode::INTERNAL_SERVER_ERROR.into_response());
            }

            let resp_headers = resp.headers().clone();
            let body = match resp.bytes().await {
                Ok(b) => b,
                Err(e) => {
                    return (
                        StatusCode::BAD_GATEWAY,
                        format!("upstream read error: {e}"),
                    )
                        .into_response();
                }
            };

            if let Some(usage) = metering::parse_usage_json(&body, &provider) {
                info!(
                    provider = %provider,
                    tokens_in = usage.tokens_in,
                    tokens_out = usage.tokens_out,
                    "provider usage metered"
                );
                if let Some(base) = link_base.clone() {
                    let client = state.client.clone();
                    let provider_s = provider.clone();
                    let bearer_s = bearer.to_string();
                    let domain_s = domain_id.map(str::to_string);
                    let run_s = run_id.map(str::to_string);
                    let usage = usage.clone();
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
        Err(e) => (
            StatusCode::BAD_GATEWAY,
            format!("upstream error: {e}"),
        )
            .into_response(),
    }
}

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt()
        .with_env_filter(tracing_subscriber::EnvFilter::from_default_env())
        .init();

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
    });

    let app = Router::new()
        .route("/health", get(health))
        .fallback(any(proxy_dispatch))
        .with_state(state);

    let addr = listen_addr();
    let listener = tokio::net::TcpListener::bind(&addr)
        .await
        .expect("bind proxy");
    info!("pytxo-proxy listening on http://{addr}");
    axum::serve(listener, app).await.expect("serve proxy");
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn anthropic_upstream_is_configured() {
        assert_eq!(provider_upstream("anthropic"), "https://api.anthropic.com");
    }

    #[test]
    fn listen_addr_defaults_to_local() {
        std::env::remove_var("PORT");
        std::env::remove_var("PROXY_BIND");
        assert!(listen_addr().contains("8790"));
    }
}
