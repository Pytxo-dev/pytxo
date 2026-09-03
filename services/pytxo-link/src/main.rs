//! Pytxo Link — billing reconcile, entitlements, and Paddle webhooks.

mod audit;
mod auth;
mod db;
mod entitlements;
mod inference;
mod jwt;
mod limits;
mod paddle;
mod runs;
mod seats;
mod state;
mod telemetry;
mod wallet;

use std::sync::Arc;

use axum::extract::{DefaultBodyLimit, Path, State};

use axum::http::{HeaderMap, StatusCode};

use axum::routing::{get, post, put};

use axum::middleware;

use entitlements::{
    AdminUpsertBody, EntitlementStatusResponse, EntitlementStore, OrgPolicyPutBody,
    OrgPolicyResponse, Tier,
};
use jwt::JwksValidator;

use axum::{Json, Router};
use serde_json::Value;
use std::net::SocketAddr;

use state::{AppState, RunEndBody, RunStartBody};

use tracing::info;

async fn health() -> Json<telemetry::HealthBody> {
    Json(telemetry::health_body())
}

async fn entitlements_status(
    State(state): State<AppState>,

    headers: HeaderMap,
) -> Result<Json<EntitlementStatusResponse>, StatusCode> {
    let subject = auth::authorized(&headers, &state)
        .await
        .ok_or(StatusCode::UNAUTHORIZED)?;

    let user_id = if subject.user_id == "api-key" {
        headers
            .get("x-pytxo-user-id")
            .and_then(|v| v.to_str().ok())
            .unwrap_or("anonymous")
    } else {
        &subject.user_id
    };

    let record = state.entitlements.get(user_id).await;

    Ok(Json(record.into()))
}

async fn admin_upsert_entitlement(
    State(state): State<AppState>,

    headers: HeaderMap,

    Path(user_id): Path<String>,

    Json(body): Json<AdminUpsertBody>,
) -> StatusCode {
    if !auth::admin_authorized(&headers, &state) {
        return StatusCode::UNAUTHORIZED;
    }

    let existing = state.entitlements.get(&user_id).await;
    if let Some(org_id) = &body.org_id {
        if let Some(pool) = state.db.as_ref() {
            let same_org = existing.org_id.as_deref() == Some(org_id.as_str());
            if !same_org {
                match seats::get(pool, org_id).await {
                    Ok(info) if info.seats_available == 0 => {
                        tracing::warn!(org_id = %org_id, user_id = %user_id, "org seat limit reached");
                        return StatusCode::CONFLICT;
                    }
                    Err(e) => {
                        tracing::error!(error = %e, "seat availability check failed");
                        return StatusCode::INTERNAL_SERVER_ERROR;
                    }
                    _ => {}
                }
            }
        }
    }

    let tier = Tier::parse(&body.tier);

    let max_agents = body.max_agents.unwrap_or_else(|| tier.max_agents());

    let cloud_enabled = body
        .cloud_enabled
        .unwrap_or(matches!(tier, Tier::Max | Tier::Ultra));

    let org_id_audit = body.org_id.clone();
    let record = entitlements::EntitlementRecord {
        user_id: user_id.clone(),

        clerk_user_id: body.clerk_user_id,

        org_id: body.org_id,

        tier,

        max_agents,

        cloud_enabled,
    };

    match state.entitlements.upsert(record).await {
        Ok(()) => {
            info!(user_id = %user_id, tier = %tier.as_str(), "admin entitlement upsert");

            if let Some(pool) = state.db.as_ref() {
                let actor = headers
                    .get("x-pytxo-actor")
                    .and_then(|v| v.to_str().ok())
                    .unwrap_or("admin");
                let detail = serde_json::json!({
                    "user_id": user_id,
                    "tier": tier.as_str(),
                    "max_agents": max_agents,
                    "cloud_enabled": cloud_enabled,
                });
                let _ = audit::append(
                    pool,
                    org_id_audit.as_deref(),
                    actor,
                    "entitlement.upsert",
                    detail,
                )
                .await;
            }

            StatusCode::OK
        }

        Err(e) => {
            tracing::error!(error = %e, "entitlement upsert failed");

            StatusCode::INTERNAL_SERVER_ERROR
        }
    }
}

async fn org_policy_put(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(org_id): Path<String>,
    Json(body): Json<OrgPolicyPutBody>,
) -> Result<Json<OrgPolicyResponse>, StatusCode> {
    if !auth::admin_authorized(&headers, &state) {
        return Err(StatusCode::UNAUTHORIZED);
    }
    let pool = state.db.as_ref().ok_or(StatusCode::NOT_FOUND)?;
    entitlements::upsert_org_policy(pool, &org_id, &body)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    let actor = headers
        .get("x-pytxo-actor")
        .and_then(|v| v.to_str().ok())
        .unwrap_or("admin");
    let detail = serde_json::json!({
        "org_id": org_id,
        "default_permission_profile": body.default_permission_profile,
        "shared_trusted_domains": body.shared_trusted_domains,
    });
    let _ = audit::append(pool, Some(&org_id), actor, "org.policy.update", detail).await;
    entitlements::get_org_policy(pool, &org_id)
        .await
        .map(Json)
        .ok_or(StatusCode::NOT_FOUND)
}

async fn admin_org_seats_put(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(org_id): Path<String>,
    Json(body): Json<seats::AdminSeatsPutBody>,
) -> Result<Json<seats::OrgSeatsResponse>, StatusCode> {
    if !auth::admin_authorized(&headers, &state) {
        return Err(StatusCode::UNAUTHORIZED);
    }
    let pool = state.db.as_ref().ok_or(StatusCode::NOT_FOUND)?;
    seats::upsert_total(pool, &org_id, body.seats_total)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    let actor = headers
        .get("x-pytxo-actor")
        .and_then(|v| v.to_str().ok())
        .unwrap_or("admin");
    let detail = serde_json::json!({
        "org_id": org_id,
        "seats_total": body.seats_total,
    });
    let _ = audit::append(pool, Some(&org_id), actor, "org.seats.update", detail).await;
    seats::get(pool, &org_id)
        .await
        .map(Json)
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)
}

async fn org_policy(
    State(state): State<AppState>,

    headers: HeaderMap,

    Path(org_id): Path<String>,
) -> Result<Json<OrgPolicyResponse>, StatusCode> {
    let _subject = auth::authorized(&headers, &state)
        .await
        .ok_or(StatusCode::UNAUTHORIZED)?;

    let pool = state.db.as_ref().ok_or(StatusCode::NOT_FOUND)?;

    let policy = entitlements::get_org_policy(pool, &org_id)
        .await
        .ok_or(StatusCode::NOT_FOUND)?;

    Ok(Json(policy))
}

async fn org_seats(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(org_id): Path<String>,
) -> Result<Json<seats::OrgSeatsResponse>, StatusCode> {
    let _subject = auth::authorized(&headers, &state)
        .await
        .ok_or(StatusCode::UNAUTHORIZED)?;
    let Some(pool) = state.db.as_ref() else {
        let seats_total = seats::default_seats_total();
        return Ok(Json(seats::OrgSeatsResponse {
            org_id,
            seats_total,
            seats_used: 0,
            seats_available: seats_total,
        }));
    };
    seats::get(pool, &org_id)
        .await
        .map(Json)
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)
}

async fn org_audit(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(org_id): Path<String>,
) -> Result<Json<Vec<audit::AuditEntry>>, StatusCode> {
    let _subject = auth::authorized(&headers, &state)
        .await
        .ok_or(StatusCode::UNAUTHORIZED)?;
    let pool = state.db.as_ref().ok_or(StatusCode::NOT_FOUND)?;
    audit::list_for_org(pool, &org_id, 50)
        .await
        .map(Json)
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)
}

async fn wallet_balance(
    State(state): State<AppState>,
    headers: HeaderMap,
) -> Result<Json<wallet::WalletBalanceResponse>, StatusCode> {
    let subject = auth::authorized(&headers, &state)
        .await
        .ok_or(StatusCode::UNAUTHORIZED)?;
    let user_id = if subject.user_id == "api-key" {
        headers
            .get("x-pytxo-user-id")
            .and_then(|v| v.to_str().ok())
            .unwrap_or("anonymous")
    } else {
        &subject.user_id
    };
    if let Some(pool) = state.db.as_ref() {
        match wallet::balance_for_user(pool, user_id).await {
            Ok(balance) => Ok(Json(balance)),
            Err(e) => {
                tracing::warn!(?e, %user_id, "wallet balance db query failed; returning initial balance");
                Ok(Json(wallet::balance_memory(0)))
            }
        }
    } else {
        Ok(Json(wallet::balance_memory(0)))
    }
}

async fn inference_usage(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(body): Json<inference::InferenceUsageBody>,
) -> StatusCode {
    let subject = match auth::authorized(&headers, &state).await {
        Some(s) => s,
        None => return StatusCode::UNAUTHORIZED,
    };
    let user_id = if subject.user_id == "api-key" {
        headers
            .get("x-pytxo-user-id")
            .and_then(|v| v.to_str().ok())
            .unwrap_or("anonymous")
            .to_string()
    } else {
        subject.user_id.clone()
    };
    let Some(pool) = state.db.as_ref() else {
        info!(
            user_id = %user_id,
            provider = %body.provider,
            tokens_in = body.tokens_in,
            tokens_out = body.tokens_out,
            "inference usage (memory mode)"
        );
        return StatusCode::OK;
    };
    match inference::record(pool, &user_id, &body).await {
        Ok(()) => StatusCode::OK,
        Err(e) => {
            tracing::error!(error = %e, "inference usage record failed");
            StatusCode::INTERNAL_SERVER_ERROR
        }
    }
}

async fn runs_start(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(body): Json<RunStartBody>,
) -> StatusCode {
    if auth::authorized(&headers, &state).await.is_none() {
        return StatusCode::UNAUTHORIZED;
    }
    info!(run_id = %body.run_id, domain = %body.domain_id, "v1/runs/start");
    match &state.runs {
        state::RunLedger::Postgres(store) => {
            if store.start(&body).await.is_err() {
                return StatusCode::INTERNAL_SERVER_ERROR;
            }
        }
        state::RunLedger::Memory(store) => store.start(&body),
    }
    StatusCode::OK
}

async fn runs_end(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(body): Json<RunEndBody>,
) -> StatusCode {
    if auth::authorized(&headers, &state).await.is_none() {
        return StatusCode::UNAUTHORIZED;
    }
    info!(run_id = %body.run_id, saved = body.usage.saved_tokens, "v1/runs/end");
    match &state.runs {
        state::RunLedger::Postgres(store) => {
            if store.end(&body).await.is_err() {
                return StatusCode::INTERNAL_SERVER_ERROR;
            }
        }
        state::RunLedger::Memory(store) => store.end(&body),
    }
    StatusCode::OK
}

async fn paddle_webhook(
    State(state): State<AppState>,
    headers: HeaderMap,
    body: axum::body::Bytes,
) -> StatusCode {
    let Some(secret) = state.paddle_webhook_secret.as_deref() else {
        return StatusCode::SERVICE_UNAVAILABLE;
    };
    let sig = headers
        .get("paddle-signature")
        .and_then(|v| v.to_str().ok())
        .unwrap_or("");
    let raw = std::str::from_utf8(&body).unwrap_or("");
    if !paddle::verify_paddle_signature(raw, sig, secret) {
        return StatusCode::UNAUTHORIZED;
    }
    let parsed: paddle::PaddleWebhook = match serde_json::from_slice(&body) {
        Ok(v) => v,
        Err(_) => return StatusCode::BAD_REQUEST,
    };
    let (Some(events), Some(prices)) = (state.paddle_events.as_ref(), state.paddle_prices.as_ref())
    else {
        return StatusCode::SERVICE_UNAVAILABLE;
    };
    match paddle::handle_paddle_webhook(events, &state.entitlements, prices, &parsed).await {
        Ok(paddle::PaddleWebhookOutcome::Applied) => StatusCode::OK,
        // Paddle retries deliveries. A durable duplicate is an idempotent
        // success, not a signal to retry the already-applied side effect.
        Ok(paddle::PaddleWebhookOutcome::Duplicate) => StatusCode::OK,
        Ok(paddle::PaddleWebhookOutcome::Rejected) => StatusCode::BAD_REQUEST,
        Err(error) => {
            tracing::error!(error = %error, event_id = %parsed.event_id, "paddle webhook transaction failed");
            StatusCode::INTERNAL_SERVER_ERROR
        }
    }
}

async fn openapi() -> Json<Value> {
    Json(serde_json::json!({

        "openapi": "3.1.0",

        "info": { "title": "Pytxo Link", "version": "0.3.3" },

        "paths": {

            "/health": { "get": { "summary": "Health check" } },

            "/v1/entitlements/status": { "get": { "summary": "Current tier and agent limits" } },

            "/v1/admin/entitlements/{user_id}": { "put": { "summary": "Admin provisioning" } },

            "/v1/orgs/{org_id}/policy": {
                "get": { "summary": "Team default permission profile" },
                "put": { "summary": "Admin org policy write" }
            },

            "/v1/admin/orgs/{org_id}/seats": { "put": { "summary": "Admin seat total update" } },

            "/v1/orgs/{org_id}/seats": { "get": { "summary": "Org seat usage" } },

            "/v1/runs/start": { "post": { "summary": "Reconcile run start" } },

            "/v1/runs/end": { "post": { "summary": "Reconcile run end with usage" } },

            "/v1/inference/usage": { "post": { "summary": "Proxy-reported provider token usage" } },

            "/v1/wallet/balance": { "get": { "summary": "Ultra wallet balance (microcredits)" } },

            "/v1/orgs/{org_id}/audit": { "get": { "summary": "Org audit log" } },

            "/v1/webhooks/paddle": { "post": { "summary": "Paddle subscription webhooks" } }

        }

    }))
}

#[tokio::main]

async fn main() {
    telemetry::init();

    let require_auth_override = std::env::var("LINK_REQUIRE_AUTH")
        .map(|v| v == "1" || v.eq_ignore_ascii_case("true"))
        .ok();

    let (entitlements, db, runs, paddle_events) = if let Ok(url) = std::env::var("DATABASE_URL") {
        if !url.is_empty() {
            let pool = db::connect(&url).await.expect("postgres connect");
            let pool_clone = pool.clone();
            (
                EntitlementStore::postgres(pool.clone()),
                Some(pool_clone),
                state::RunLedger::postgres(pool.clone()),
                Some(paddle::PaddleEventStore::postgres(pool)),
            )
        } else {
            (
                EntitlementStore::memory(),
                None,
                state::RunLedger::memory(),
                None,
            )
        }
    } else {
        (
            EntitlementStore::memory(),
            None,
            state::RunLedger::memory(),
            None,
        )
    };

    let jwks = match (
        std::env::var("CLERK_JWKS_URL")
            .ok()
            .filter(|s| !s.is_empty()),
        std::env::var("CLERK_ISSUER").ok().filter(|s| !s.is_empty()),
    ) {
        (Some(jwks_url), Some(issuer)) => Some(Arc::new(JwksValidator::new(jwks_url, issuer))),

        _ => None,
    };

    let api_key = std::env::var("LINK_API_KEY").ok().filter(|s| !s.is_empty());
    let require_auth = require_auth_override.unwrap_or(api_key.is_some() || jwks.is_some());

    let state = AppState {
        api_key,

        admin_key: std::env::var("LINK_ADMIN_KEY")
            .ok()
            .filter(|s| !s.is_empty()),

        require_auth,

        jwks,

        entitlements,

        db,

        runs,

        paddle_webhook_secret: std::env::var("PADDLE_WEBHOOK_SECRET")
            .ok()
            .filter(|value| !value.is_empty()),

        paddle_events,

        paddle_prices: paddle::PaddlePriceCatalog::from_env(),
    };

    let addr = listen_addr();
    validate_startup_security(
        &addr,
        require_auth,
        state.api_key.is_some() || state.jwks.is_some(),
    )
    .expect("refusing insecure Pytxo Link startup");

    let app = apply_service_layers(build_router(state));

    let listener = tokio::net::TcpListener::bind(&addr)
        .await
        .expect("bind link service");
    info!("pytxo-link listening on http://{addr}");
    axum::serve(
        listener,
        app.into_make_service_with_connect_info::<SocketAddr>(),
    )
    .await
    .expect("serve link service");
}

/// Railway injects `PORT`; local dev uses `LINK_BIND` or 127.0.0.1:8787.
fn listen_addr() -> String {
    if let Ok(port) = std::env::var("PORT") {
        if !port.is_empty() {
            return format!("0.0.0.0:{port}");
        }
    }
    std::env::var("LINK_BIND").unwrap_or_else(|_| "127.0.0.1:8787".into())
}

fn validate_startup_security(
    addr: &str,
    require_auth: bool,
    has_authenticator: bool,
) -> Result<(), String> {
    let socket: SocketAddr = addr
        .parse()
        .map_err(|_| format!("LINK_BIND must be an IP socket address, got {addr:?}"))?;
    if !socket.ip().is_loopback() && !require_auth {
        return Err(format!("public bind {addr} requires LINK_REQUIRE_AUTH=1"));
    }
    if require_auth && !has_authenticator {
        return Err(
            "LINK_REQUIRE_AUTH=1 requires LINK_API_KEY or both CLERK_JWKS_URL and CLERK_ISSUER"
                .into(),
        );
    }
    Ok(())
}

fn build_router(state: AppState) -> Router {
    Router::new()
        .route("/health", get(health))
        .route("/openapi.json", get(openapi))
        .route("/v1/entitlements/status", get(entitlements_status))
        .route(
            "/v1/admin/entitlements/{user_id}",
            put(admin_upsert_entitlement),
        )
        .route(
            "/v1/orgs/{org_id}/policy",
            get(org_policy).put(org_policy_put),
        )
        .route("/v1/admin/orgs/{org_id}/seats", put(admin_org_seats_put))
        .route("/v1/orgs/{org_id}/seats", get(org_seats))
        .route("/v1/orgs/{org_id}/audit", get(org_audit))
        .route("/v1/inference/usage", post(inference_usage))
        .route("/v1/wallet/balance", get(wallet_balance))
        .route("/v1/runs/start", post(runs_start))
        .route("/v1/runs/end", post(runs_end))
        .route("/v1/webhooks/paddle", post(paddle_webhook))
        .with_state(state)
}

fn apply_service_layers(router: Router) -> Router {
    let rate_limiter = limits::RateLimiter::from_env();
    router
        .layer(DefaultBodyLimit::max(limits::max_body_bytes()))
        .layer(middleware::from_fn_with_state(
            rate_limiter,
            limits::rate_limit_middleware,
        ))
}

#[cfg(test)]
mod contract_tests {
    use super::*;
    use axum::body::Body;
    use axum::http::{Request, StatusCode};
    use hmac::{Hmac, Mac};
    use http_body_util::BodyExt;
    use serde_json::Value;
    use sha2::Sha256;
    use std::time::{SystemTime, UNIX_EPOCH};
    use tower::ServiceExt;

    const TEST_PADDLE_SECRET: &str = "test-paddle-secret";

    #[test]
    fn startup_security_allows_only_authenticated_public_binds() {
        assert!(validate_startup_security("127.0.0.1:8787", false, false).is_ok());
        assert!(validate_startup_security("[::1]:8787", false, false).is_ok());
        assert!(validate_startup_security("0.0.0.0:8787", true, true).is_ok());
        assert!(validate_startup_security("0.0.0.0:8787", false, false).is_err());
        assert!(validate_startup_security("0.0.0.0:8787", true, false).is_err());
    }

    fn test_state() -> AppState {
        AppState {
            api_key: None,
            admin_key: Some("test-admin".into()),
            require_auth: false,
            jwks: None,
            entitlements: EntitlementStore::memory(),
            db: None,
            runs: state::RunLedger::memory(),
            paddle_webhook_secret: None,
            paddle_events: Some(paddle::PaddleEventStore::memory()),
            paddle_prices: Some(paddle::PaddlePriceCatalog::test()),
        }
    }

    fn paddle_state() -> AppState {
        AppState {
            paddle_webhook_secret: Some(TEST_PADDLE_SECRET.into()),
            ..test_state()
        }
    }

    fn signed_paddle_request(body: &'static str) -> Request<Body> {
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();
        let payload = format!("{timestamp}:{body}");
        let mut mac = Hmac::<Sha256>::new_from_slice(TEST_PADDLE_SECRET.as_bytes()).unwrap();
        mac.update(payload.as_bytes());
        let signature = format!(
            "ts={timestamp};h1={}",
            hex::encode(mac.finalize().into_bytes())
        );
        Request::builder()
            .method("POST")
            .uri("/v1/webhooks/paddle")
            .header("content-type", "application/json")
            .header("paddle-signature", signature)
            .body(Body::from(body))
            .unwrap()
    }

    #[tokio::test]
    async fn health_returns_json_uptime() {
        let app = build_router(test_state());
        let response = app
            .oneshot(
                Request::builder()
                    .uri("/health")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(response.status(), StatusCode::OK);
        let body = response.into_body().collect().await.unwrap().to_bytes();
        let doc: Value = serde_json::from_slice(&body).unwrap();
        assert_eq!(doc["status"], "ok");
        assert_eq!(doc["service"], "pytxo-link");
        assert!(doc["uptime_secs"].is_number());
    }

    #[tokio::test]
    async fn openapi_lists_required_paths() {
        let app = build_router(test_state());
        let response = app
            .oneshot(
                Request::builder()
                    .uri("/openapi.json")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(response.status(), StatusCode::OK);
        let body = response.into_body().collect().await.unwrap().to_bytes();
        let doc: Value = serde_json::from_slice(&body).unwrap();
        let paths = doc["paths"].as_object().unwrap();
        for path in [
            "/health",
            "/v1/entitlements/status",
            "/v1/admin/entitlements/{user_id}",
            "/v1/orgs/{org_id}/policy",
            "/v1/admin/orgs/{org_id}/seats",
            "/v1/orgs/{org_id}/seats",
            "/v1/orgs/{org_id}/audit",
            "/v1/runs/start",
            "/v1/runs/end",
            "/v1/wallet/balance",
            "/v1/webhooks/paddle",
        ] {
            assert!(paths.contains_key(path), "missing path {path}");
        }
    }

    #[tokio::test]
    async fn wallet_balance_returns_initial_in_memory_mode() {
        let app = build_router(test_state());
        let response = app
            .oneshot(
                Request::builder()
                    .uri("/v1/wallet/balance")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(response.status(), StatusCode::OK);
        let body = response.into_body().collect().await.unwrap().to_bytes();
        let doc: Value = serde_json::from_slice(&body).unwrap();
        assert!(doc["balance_microcredits"].as_i64().unwrap() > 0);
    }

    #[tokio::test]
    async fn runs_ledger_idempotent_across_retry_simulation() {
        let app = build_router(test_state());
        let start_body = r#"{"domain_id":"d-restart","run_id":"r-restart"}"#;
        let end_body = r#"{"domain_id":"d-restart","run_id":"r-restart","usage":{"tokens_in_billed":10,"tokens_in_sent":5,"tokens_out":2,"saved_tokens":3,"cost_micro_usd":100}}"#;

        for _ in 0..2 {
            let response = app
                .clone()
                .oneshot(
                    Request::builder()
                        .method("POST")
                        .uri("/v1/runs/start")
                        .header("content-type", "application/json")
                        .body(Body::from(start_body))
                        .unwrap(),
                )
                .await
                .unwrap();
            assert_eq!(response.status(), StatusCode::OK);
        }

        for _ in 0..2 {
            let response = app
                .clone()
                .oneshot(
                    Request::builder()
                        .method("POST")
                        .uri("/v1/runs/end")
                        .header("content-type", "application/json")
                        .body(Body::from(end_body))
                        .unwrap(),
                )
                .await
                .unwrap();
            assert_eq!(response.status(), StatusCode::OK);
        }
    }

    #[tokio::test]
    async fn admin_upsert_requires_key() {
        let app = build_router(test_state());
        let response = app
            .oneshot(
                Request::builder()
                    .method("PUT")
                    .uri("/v1/admin/entitlements/user-1")
                    .header("content-type", "application/json")
                    .body(Body::from(r#"{"tier":"pro"}"#))
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
    }

    #[tokio::test]
    async fn runs_start_end_memory_ledger() {
        let app = build_router(test_state());
        let start = app
            .clone()
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/v1/runs/start")
                    .header("content-type", "application/json")
                    .body(Body::from(r#"{"domain_id":"d1","run_id":"r1"}"#))
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(start.status(), StatusCode::OK);
        let end = app
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/v1/runs/end")
                    .header("content-type", "application/json")
                    .body(Body::from(
                        r#"{"domain_id":"d1","run_id":"r1","usage":{"tokens_in_billed":10,"tokens_in_sent":5,"tokens_out":2,"saved_tokens":3,"cost_micro_usd":100}}"#,
                    ))
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(end.status(), StatusCode::OK);
    }

    #[tokio::test]
    async fn paddle_webhook_rejects_requests_when_secret_is_missing() {
        let app = build_router(test_state());
        let response = app
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/v1/webhooks/paddle")
                    .header("content-type", "application/json")
                    .body(Body::from(
                        r#"{"event_id":"evt-missing-secret","event_type":"subscription.updated","data":{"custom_data":{"user_id":"user-1","tier":"ultra"}}}"#,
                    ))
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(response.status(), StatusCode::SERVICE_UNAVAILABLE);
    }

    #[tokio::test]
    async fn paddle_webhook_rejects_unsupported_event_type() {
        let body = r#"{"event_id":"evt-unsupported","event_type":"transaction.completed","data":{"custom_data":{"user_id":"user-1","tier":"ultra"}}}"#;
        let response = build_router(paddle_state())
            .oneshot(signed_paddle_request(body))
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::BAD_REQUEST);
    }

    #[tokio::test]
    async fn paddle_webhook_rejects_duplicate_event_id() {
        let app = build_router(paddle_state());
        let body = r#"{"event_id":"evt-duplicate","event_type":"subscription.created","data":{"id":"sub-duplicate","customer_id":"ctm-duplicate","items":[{"price":{"id":"pri-pro"}}],"custom_data":{"user_id":"user-1"}}}"#;

        let first = app
            .clone()
            .oneshot(signed_paddle_request(body))
            .await
            .unwrap();
        let replay = app.oneshot(signed_paddle_request(body)).await.unwrap();

        assert_eq!(first.status(), StatusCode::OK);
        assert_eq!(replay.status(), StatusCode::OK);
    }

    #[tokio::test]
    async fn paddle_created_rejects_missing_subscription_identifiers() {
        let body = r#"{"event_id":"evt-missing-identifiers","event_type":"subscription.created","data":{"custom_data":{"user_id":"user-1","tier":"ultra"}}}"#;
        let response = build_router(paddle_state())
            .oneshot(signed_paddle_request(body))
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::BAD_REQUEST);
    }

    #[tokio::test]
    async fn paddle_created_rejects_unknown_price_even_when_custom_tier_is_ultra() {
        let body = r#"{"event_id":"evt-unknown-price","event_type":"subscription.created","data":{"id":"sub-1","customer_id":"ctm-1","items":[{"price":{"id":"pri-attacker"}}],"custom_data":{"user_id":"user-1","tier":"ultra"}}}"#;
        let response = build_router(paddle_state())
            .oneshot(signed_paddle_request(body))
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::BAD_REQUEST);
    }

    #[tokio::test]
    async fn paddle_updated_rejects_conflicting_custom_user() {
        let state = paddle_state();
        let entitlements = state.entitlements.clone();
        let app = build_router(state);
        let created = r#"{"event_id":"evt-created-binding","event_type":"subscription.created","data":{"id":"sub-bound","customer_id":"ctm-bound","items":[{"price":{"id":"pri-pro"}}],"custom_data":{"user_id":"user-1"}}}"#;
        let updated = r#"{"event_id":"evt-updated-conflict","event_type":"subscription.updated","data":{"id":"sub-bound","customer_id":"ctm-bound","items":[{"price":{"id":"pri-ultra"}}],"custom_data":{"user_id":"attacker"}}}"#;

        let first = app
            .clone()
            .oneshot(signed_paddle_request(created))
            .await
            .unwrap();
        let conflict = app.oneshot(signed_paddle_request(updated)).await.unwrap();

        assert_eq!(first.status(), StatusCode::OK);
        assert_eq!(conflict.status(), StatusCode::BAD_REQUEST);
        assert_eq!(entitlements.get("user-1").await.tier, Tier::Pro);
        assert_eq!(entitlements.get("attacker").await.tier, Tier::Core);
    }

    #[tokio::test]
    async fn paddle_created_requires_user_binding() {
        let body = r#"{"event_id":"evt-missing-user","event_type":"subscription.created","data":{"id":"sub-missing-user","customer_id":"ctm-missing-user","items":[{"price":{"id":"pri-pro"}}],"custom_data":{}}}"#;
        let response = build_router(paddle_state())
            .oneshot(signed_paddle_request(body))
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::BAD_REQUEST);
    }

    #[tokio::test]
    async fn paddle_updated_uses_persisted_binding_and_catalog_price() {
        let state = paddle_state();
        let entitlements = state.entitlements.clone();
        let app = build_router(state);
        let created = r#"{"event_id":"evt-created-upgrade","event_type":"subscription.created","data":{"id":"sub-upgrade","customer_id":"ctm-upgrade","items":[{"price":{"id":"pri-pro"}}],"custom_data":{"user_id":"user-upgrade"}}}"#;
        let updated = r#"{"event_id":"evt-updated-upgrade","event_type":"subscription.updated","data":{"id":"sub-upgrade","customer_id":"ctm-upgrade","items":[{"price":{"id":"pri-ultra"}}]}}"#;

        let created_response = app
            .clone()
            .oneshot(signed_paddle_request(created))
            .await
            .unwrap();
        let updated_response = app.oneshot(signed_paddle_request(updated)).await.unwrap();

        assert_eq!(created_response.status(), StatusCode::OK);
        assert_eq!(updated_response.status(), StatusCode::OK);
        assert_eq!(entitlements.get("user-upgrade").await.tier, Tier::Ultra);
    }

    #[tokio::test]
    async fn paddle_updated_rejects_unbound_or_conflicting_customer() {
        let app = build_router(paddle_state());
        let unbound = r#"{"event_id":"evt-unbound","event_type":"subscription.updated","data":{"id":"sub-unbound","customer_id":"ctm-unbound","items":[{"price":{"id":"pri-ultra"}}]}}"#;
        let created = r#"{"event_id":"evt-created-customer","event_type":"subscription.created","data":{"id":"sub-customer","customer_id":"ctm-correct","items":[{"price":{"id":"pri-pro"}}],"custom_data":{"user_id":"user-customer"}}}"#;
        let conflicting = r#"{"event_id":"evt-conflicting-customer","event_type":"subscription.updated","data":{"id":"sub-customer","customer_id":"ctm-attacker","items":[{"price":{"id":"pri-ultra"}}]}}"#;

        let unbound_response = app
            .clone()
            .oneshot(signed_paddle_request(unbound))
            .await
            .unwrap();
        let created_response = app
            .clone()
            .oneshot(signed_paddle_request(created))
            .await
            .unwrap();
        let conflict_response = app
            .oneshot(signed_paddle_request(conflicting))
            .await
            .unwrap();

        assert_eq!(unbound_response.status(), StatusCode::BAD_REQUEST);
        assert_eq!(created_response.status(), StatusCode::OK);
        assert_eq!(conflict_response.status(), StatusCode::BAD_REQUEST);
    }

    #[tokio::test]
    async fn paddle_canceled_downgrades_only_the_persisted_user() {
        let state = paddle_state();
        let entitlements = state.entitlements.clone();
        let app = build_router(state);
        let created = r#"{"event_id":"evt-created-cancel","event_type":"subscription.created","data":{"id":"sub-cancel","customer_id":"ctm-cancel","items":[{"price":{"id":"pri-ultra"}}],"custom_data":{"user_id":"user-cancel"}}}"#;
        let canceled = r#"{"event_id":"evt-canceled","event_type":"subscription.canceled","data":{"id":"sub-cancel","customer_id":"ctm-cancel","custom_data":{"user_id":"user-cancel","tier":"ultra"}}}"#;

        let created_response = app
            .clone()
            .oneshot(signed_paddle_request(created))
            .await
            .unwrap();
        let canceled_response = app.oneshot(signed_paddle_request(canceled)).await.unwrap();

        assert_eq!(created_response.status(), StatusCode::OK);
        assert_eq!(canceled_response.status(), StatusCode::OK);
        assert_eq!(entitlements.get("user-cancel").await.tier, Tier::Core);
    }
}
