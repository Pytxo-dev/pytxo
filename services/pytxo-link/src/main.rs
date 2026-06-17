//! Pytxo Link â€” billing reconcile, entitlements, and Paddle webhooks.



mod auth;

mod db;

mod entitlements;

mod jwt;

mod paddle;

mod state;
mod telemetry;

use std::sync::{Arc, Mutex};



use axum::extract::{Path, State};

use axum::http::{HeaderMap, StatusCode};

use axum::routing::{get, post, put};

use axum::{Json, Router};

use entitlements::{
    AdminUpsertBody, EntitlementStatusResponse, EntitlementStore, OrgPolicyResponse, Tier,
};
use jwt::JwksValidator;

use serde_json::Value;

use state::{AppState, RunEndBody, RunRecord, RunStartBody};

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



async fn runs_start(

    State(state): State<AppState>,

    headers: HeaderMap,

    Json(body): Json<RunStartBody>,

) -> StatusCode {

    if auth::authorized(&headers, &state).await.is_none() {

        return StatusCode::UNAUTHORIZED;

    }

    info!(run_id = %body.run_id, domain = %body.domain_id, "v1/runs/start");

    state.runs.lock().unwrap().insert(

        body.run_id.clone(),

        RunRecord {

            domain_id: body.domain_id,

            started: true,

            ended: false,

            usage: None,

        },

    );

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

    let mut runs = state.runs.lock().unwrap();

    let entry = runs.entry(body.run_id).or_insert(RunRecord {

        domain_id: body.domain_id.clone(),

        started: false,

        ended: false,

        usage: None,

    });

    entry.ended = true;

    entry.usage = Some(body.usage);

    StatusCode::OK

}



async fn paddle_webhook(

    State(state): State<AppState>,

    headers: HeaderMap,

    Json(body): Json<paddle::PaddleWebhook>,

) -> StatusCode {

    if let Some(secret) = std::env::var("PADDLE_WEBHOOK_SECRET").ok() {

        let sig = headers

            .get("paddle-signature")

            .and_then(|v| v.to_str().ok())

            .unwrap_or("");

        if sig.is_empty()

            && !secret.is_empty()

            && std::env::var("LINK_REQUIRE_AUTH").ok().as_deref() == Some("1")

        {

            return StatusCode::UNAUTHORIZED;

        }

    }

    if paddle::handle_paddle_webhook(&state.entitlements, &body).await {

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



    let (entitlements, db) = if let Ok(url) = std::env::var("DATABASE_URL") {

        if !url.is_empty() {

            let pool = db::connect(&url)

                .await

                .expect("postgres connect");

            let pool_clone = pool.clone();

            (EntitlementStore::postgres(pool), Some(pool_clone))

        } else {

            (EntitlementStore::memory(), None)

        }

    } else {

        (EntitlementStore::memory(), None)

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

        runs: Arc::new(Mutex::new(std::collections::HashMap::new())),

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
            runs: Arc::new(Mutex::new(std::collections::HashMap::new())),
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
}

