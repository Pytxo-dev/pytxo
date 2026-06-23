//! Pytxo Link â€” billing reconcile, entitlements, and Paddle webhooks.



mod auth;
mod audit;
mod db;
mod entitlements;
mod inference;
mod jwt;
mod paddle;
mod runs;
mod state;
mod telemetry;

use std::sync::Arc;



use axum::extract::{Path, State};

use axum::http::{HeaderMap, StatusCode};

use axum::routing::{get, post, put};

use axum::{Json, Router};

use entitlements::{
    AdminUpsertBody, EntitlementStatusResponse, EntitlementStore, OrgPolicyResponse, Tier,
};
use jwt::JwksValidator;

use serde::Serialize;
use serde_json::Value;

use state::{AppState, RunEndBody, RunStartBody};

use tracing::info;



async fn health() -> &'static str {

    "ok"

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

    let tier = Tier::parse(&body.tier);

    let max_agents = body.max_agents.unwrap_or_else(|| tier.max_agents());

    let cloud_enabled = body

        .cloud_enabled

        .unwrap_or_else(|| matches!(tier, Tier::Max | Tier::Ultra));

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

#[derive(Serialize)]
struct OrgSeatsResponse {
    org_id: String,
    seats_total: usize,
    seats_used: usize,
    seats_available: usize,
}

/// Enterprise seat management stub (Phase 49). Postgres ledger lands in a follow-up migration.
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

async fn org_seats(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(org_id): Path<String>,
) -> Result<Json<OrgSeatsResponse>, StatusCode> {
    let _subject = auth::authorized(&headers, &state)
        .await
        .ok_or(StatusCode::UNAUTHORIZED)?;
    let seats_total = std::env::var("LINK_ORG_SEATS_DEFAULT")
        .ok()
        .and_then(|s| s.parse().ok())
        .unwrap_or(10);
    let seats_used = if state.db.is_some() { 1 } else { 0 };
    Ok(Json(OrgSeatsResponse {
        org_id,
        seats_total,
        seats_used,
        seats_available: seats_total.saturating_sub(seats_used),
    }))
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
    if let Ok(secret) = std::env::var("PADDLE_WEBHOOK_SECRET") {
        if !secret.is_empty() {
            let sig = headers
                .get("paddle-signature")
                .and_then(|v| v.to_str().ok())
                .unwrap_or("");
            let raw = std::str::from_utf8(&body).unwrap_or("");
            if !paddle::verify_paddle_signature(raw, sig, &secret) {
                return StatusCode::UNAUTHORIZED;
            }
        }
    }
    let parsed: paddle::PaddleWebhook = match serde_json::from_slice(&body) {
        Ok(v) => v,
        Err(_) => return StatusCode::BAD_REQUEST,
    };
    if paddle::handle_paddle_webhook(&state.entitlements, &parsed).await {
        StatusCode::OK
    } else {
        StatusCode::BAD_REQUEST
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

            "/v1/orgs/{org_id}/policy": { "get": { "summary": "Team default permission profile" } },

            "/v1/runs/start": { "post": { "summary": "Reconcile run start" } },

            "/v1/runs/end": { "post": { "summary": "Reconcile run end with usage" } },

            "/v1/inference/usage": { "post": { "summary": "Proxy-reported provider token usage" } },

            "/v1/orgs/{org_id}/audit": { "get": { "summary": "Org audit log" } },

            "/v1/webhooks/paddle": { "post": { "summary": "Paddle subscription webhooks" } }

        }

    }))

}



#[tokio::main]

async fn main() {

    telemetry::init();

    let require_auth = std::env::var("LINK_REQUIRE_AUTH")

        .map(|v| v == "1" || v.eq_ignore_ascii_case("true"))

        .unwrap_or(false);



    let (entitlements, db, runs) = if let Ok(url) = std::env::var("DATABASE_URL") {
        if !url.is_empty() {
            let pool = db::connect(&url)
                .await
                .expect("postgres connect");
            let pool_clone = pool.clone();
            (
                EntitlementStore::postgres(pool.clone()),
                Some(pool_clone),
                state::RunLedger::postgres(pool),
            )
        } else {
            (
                EntitlementStore::memory(),
                None,
                state::RunLedger::memory(),
            )
        }
    } else {
        (
            EntitlementStore::memory(),
            None,
            state::RunLedger::memory(),
        )
    };



    let jwks = match (

        std::env::var("CLERK_JWKS_URL").ok().filter(|s| !s.is_empty()),

        std::env::var("CLERK_ISSUER").ok().filter(|s| !s.is_empty()),

    ) {

        (Some(jwks_url), Some(issuer)) => Some(Arc::new(JwksValidator::new(jwks_url, issuer))),

        _ => None,

    };



    let state = AppState {

        api_key: std::env::var("LINK_API_KEY").ok().filter(|s| !s.is_empty()),

        admin_key: std::env::var("LINK_ADMIN_KEY").ok().filter(|s| !s.is_empty()),

        require_auth,

        jwks,

        entitlements,

        db,

        runs,

    };



    if require_auth && state.api_key.is_none() && state.jwks.is_none() {

        tracing::warn!("LINK_REQUIRE_AUTH=1 but neither LINK_API_KEY nor CLERK_JWKS_URL is set");

    }



    let app = build_router(state);

    let addr = listen_addr();
    let listener = tokio::net::TcpListener::bind(&addr)
        .await
        .expect("bind link service");
    info!("pytxo-link listening on http://{addr}");
    axum::serve(listener, app)
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

fn build_router(state: AppState) -> Router {
    Router::new()
        .route("/health", get(health))

        .route("/openapi.json", get(openapi))

        .route("/v1/entitlements/status", get(entitlements_status))

        .route(

            "/v1/admin/entitlements/{user_id}",
            put(admin_upsert_entitlement),
        )
        .route("/v1/orgs/{org_id}/policy", get(org_policy))
        .route("/v1/orgs/{org_id}/seats", get(org_seats))
        .route("/v1/orgs/{org_id}/audit", get(org_audit))
        .route("/v1/inference/usage", post(inference_usage))

        .route("/v1/runs/start", post(runs_start))

        .route("/v1/runs/end", post(runs_end))

        .route("/v1/webhooks/paddle", post(paddle_webhook))

        .with_state(state)
}

#[cfg(test)]
mod contract_tests {
    use super::*;
    use axum::body::Body;
    use axum::http::{Request, StatusCode};
    use http_body_util::BodyExt;
    use serde_json::Value;
    use tower::ServiceExt;

    fn test_state() -> AppState {
        AppState {
            api_key: None,
            admin_key: Some("test-admin".into()),
            require_auth: false,
            jwks: None,
            entitlements: EntitlementStore::memory(),
            db: None,
            runs: state::RunLedger::memory(),
        }
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
            "/v1/runs/start",
            "/v1/runs/end",
            "/v1/webhooks/paddle",
        ] {
            assert!(paths.contains_key(path), "missing path {path}");
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
}

