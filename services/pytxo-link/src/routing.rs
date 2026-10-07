//! Experimental, account-bound routing session credentials. This module does
//! not admit an evaluation or call an inference provider.

use axum::extract::{Path, State};
use axum::http::{header, HeaderMap, HeaderValue, StatusCode};
use axum::Json;
use base64::engine::general_purpose::URL_SAFE_NO_PAD;
use base64::Engine;
use chrono::{DateTime, Utc};
pub(crate) use pytxo_planner::advisor::HOSTED_RECIPIENT;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

use crate::state::AppState;

const SCOPE: &str = "routing:evaluate:v1";
const TOKEN_PREFIX: &str = "pr1_";
const MAX_WORKSPACE_GRANTS_PER_ACCOUNT: i64 = 64;
pub(crate) fn hosted_scope_digest() -> [u8; 32] {
    hex::decode(pytxo_planner::advisor::hosted_scope_digest().0)
        .expect("trusted hosted scope digest is hex")
        .try_into()
        .expect("trusted hosted scope digest is 32 bytes")
}

#[derive(Serialize)]
pub struct RoutingTokenResponse {
    token: String,
    token_type: &'static str,
    scope: &'static str,
    expires_at: DateTime<Utc>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub struct WorkspaceGrantRequest {
    workspace_id: String,
    recipient_identity: String,
    scope_digest: String,
    expected_revision: u64,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub struct WorkspaceRevokeRequest {
    workspace_id: String,
    expected_revision: u64,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RoutingTokenRequest {
    workspace_id: String,
    expected_grant_revision: u64,
}

#[derive(Serialize)]
pub struct WorkspaceGrantResponse {
    workspace_id: String,
    recipient_identity: &'static str,
    scope_digest: String,
    revision: i64,
    enabled: bool,
}

fn parse_workspace_id(value: &str) -> Result<[u8; 16], StatusCode> {
    if value.len() != 32
        || !value
            .bytes()
            .all(|byte| byte.is_ascii_hexdigit() && !byte.is_ascii_uppercase())
    {
        return Err(StatusCode::BAD_REQUEST);
    }
    hex::decode(value)
        .map_err(|_| StatusCode::BAD_REQUEST)?
        .try_into()
        .map_err(|_| StatusCode::BAD_REQUEST)
}

fn revision(value: u64) -> Result<i64, StatusCode> {
    i64::try_from(value).map_err(|_| StatusCode::BAD_REQUEST)
}

pub(crate) async fn signed_routing_account(
    state: &AppState,
    headers: &HeaderMap,
) -> Result<(String, DateTime<Utc>), StatusCode> {
    let (Some(audience), Some(jwks), Some(_)) = (
        state.routing_token_audience.as_deref(),
        state.jwks.as_ref(),
        state.db.as_ref(),
    ) else {
        return Err(StatusCode::SERVICE_UNAVAILABLE);
    };
    let session = bearer(headers).ok_or(StatusCode::UNAUTHORIZED)?;
    let claims = jwks
        .validate_for_routing(session, audience)
        .await
        .ok_or(StatusCode::UNAUTHORIZED)?;
    let expiry = DateTime::<Utc>::from_timestamp(claims.exp, 0).ok_or(StatusCode::UNAUTHORIZED)?;
    if expiry <= Utc::now() {
        return Err(StatusCode::UNAUTHORIZED);
    }
    Ok((claims.sub, expiry))
}

async fn routing_account(
    state: &AppState,
    headers: &HeaderMap,
) -> Result<(String, DateTime<Utc>), StatusCode> {
    if let Some(token) = bearer(headers) {
        if let Some(account) = crate::routing_desktop::desktop_account(state, token).await? {
            return Ok(account);
        }
    }
    signed_routing_account(state, headers).await
}

async fn routing_account_for_revoke(
    state: &AppState,
    headers: &HeaderMap,
) -> Result<(String, DateTime<Utc>), StatusCode> {
    if let Some(token) = bearer(headers) {
        if let Some(account) =
            crate::routing_desktop::desktop_account_for_revoke(state, token).await?
        {
            return Ok(account);
        }
    }
    signed_routing_account(state, headers).await
}

/// A routing identity checked before waiting on the account gate may have
/// been revoked by the time it can mutate a grant or mint a token. Recheck
/// the scoped Desktop credential and expiry inside that serialized section.
pub(crate) async fn recheck_account_after_gate(
    headers: &HeaderMap,
    account: &str,
    expiry: DateTime<Utc>,
    tx: &mut sqlx::Transaction<'_, sqlx::Postgres>,
    allow_recovery: bool,
) -> Result<(), StatusCode> {
    let now: DateTime<Utc> = sqlx::query_scalar("SELECT clock_timestamp()")
        .fetch_one(&mut **tx)
        .await
        .map_err(|_| StatusCode::SERVICE_UNAVAILABLE)?;
    if expiry <= now {
        return Err(StatusCode::UNAUTHORIZED);
    }
    if let Some(token) = bearer(headers).filter(|token| token.starts_with("pds1_")) {
        let valid: bool = sqlx::query_scalar(
            "SELECT EXISTS(SELECT 1 FROM routing_desktop_sessions WHERE account_id = $1 AND token_hash = $2 AND (scope = 'routing:grants:v1' OR ($3 AND scope = 'routing:revoke:v1')) AND revoked_at IS NULL AND expires_at > clock_timestamp())",
        )
        .bind(account)
        .bind(token_hash(token).as_slice())
        .bind(allow_recovery)
        .fetch_one(&mut **tx)
        .await
        .map_err(|_| StatusCode::SERVICE_UNAVAILABLE)?;
        if !valid {
            return Err(StatusCode::UNAUTHORIZED);
        }
    }
    Ok(())
}

fn grant_response(
    workspace_id: String,
    scope_digest: String,
    revision: i64,
    enabled: bool,
) -> (HeaderMap, Json<WorkspaceGrantResponse>) {
    (
        no_store_headers(),
        Json(WorkspaceGrantResponse {
            workspace_id,
            recipient_identity: HOSTED_RECIPIENT,
            scope_digest,
            revision,
            enabled,
        }),
    )
}

pub async fn workspace_grant_status(
    State(state): State<AppState>,
    Path(workspace_id): Path<String>,
    headers: HeaderMap,
) -> Result<(HeaderMap, Json<WorkspaceGrantResponse>), StatusCode> {
    let (account, _) = routing_account_for_revoke(&state, &headers).await?;
    let workspace = parse_workspace_id(&workspace_id)?;
    let pool = state.db.as_ref().ok_or(StatusCode::SERVICE_UNAVAILABLE)?;
    let stored: Option<(i64, bool, Vec<u8>)> = sqlx::query_as(
        "SELECT revision, enabled, scope_digest FROM routing_workspace_grants WHERE account_id = $1 AND workspace_id = $2 AND recipient_identity = $3",
    )
    .bind(&account).bind(workspace.as_slice()).bind(HOSTED_RECIPIENT)
    .fetch_optional(pool).await.map_err(|_| StatusCode::SERVICE_UNAVAILABLE)?;
    let (revision, enabled, scope) = stored.ok_or(StatusCode::NOT_FOUND)?;
    Ok(grant_response(
        workspace_id,
        hex::encode(scope),
        revision,
        enabled,
    ))
}

pub async fn enable_workspace_grant(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(body): Json<WorkspaceGrantRequest>,
) -> Result<(HeaderMap, Json<WorkspaceGrantResponse>), StatusCode> {
    if !state.routing_grant_experiment {
        return Err(StatusCode::SERVICE_UNAVAILABLE);
    }
    let (account, expiry) = routing_account(&state, &headers).await?;
    let workspace = parse_workspace_id(&body.workspace_id)?;
    let expected = revision(body.expected_revision)?;
    if body.recipient_identity != HOSTED_RECIPIENT
        || body.scope_digest != hex::encode(hosted_scope_digest())
    {
        return Err(StatusCode::BAD_REQUEST);
    }
    let pool = state.db.as_ref().ok_or(StatusCode::SERVICE_UNAVAILABLE)?;
    let mut tx = pool
        .begin()
        .await
        .map_err(|_| StatusCode::SERVICE_UNAVAILABLE)?;
    sqlx::query("INSERT INTO routing_account_gate(account_id) VALUES ($1) ON CONFLICT DO NOTHING")
        .bind(&account)
        .execute(&mut *tx)
        .await
        .map_err(|_| StatusCode::SERVICE_UNAVAILABLE)?;
    sqlx::query("SELECT account_id FROM routing_account_gate WHERE account_id = $1 FOR UPDATE")
        .bind(&account)
        .fetch_one(&mut *tx)
        .await
        .map_err(|_| StatusCode::SERVICE_UNAVAILABLE)?;
    recheck_account_after_gate(&headers, &account, expiry, &mut tx, false).await?;
    let old: Option<(i64, bool, Vec<u8>)> = sqlx::query_as(
        "SELECT revision, enabled, scope_digest FROM routing_workspace_grants WHERE account_id = $1 AND workspace_id = $2 AND recipient_identity = $3 FOR UPDATE",
    )
    .bind(&account).bind(workspace.as_slice()).bind(HOSTED_RECIPIENT)
    .fetch_optional(&mut *tx).await.map_err(|_| StatusCode::SERVICE_UNAVAILABLE)?;
    let next = match old.as_ref() {
        Some((current, enabled, scope)) if *current == expected => {
            if *enabled && *scope == hosted_scope_digest() {
                *current
            } else {
                current.checked_add(1).ok_or(StatusCode::CONFLICT)?
            }
        }
        None if expected == 0 => {
            let count: i64 = sqlx::query_scalar(
                "SELECT count(*) FROM routing_workspace_grants WHERE account_id = $1",
            )
            .bind(&account)
            .fetch_one(&mut *tx)
            .await
            .map_err(|_| StatusCode::SERVICE_UNAVAILABLE)?;
            if count >= MAX_WORKSPACE_GRANTS_PER_ACCOUNT {
                return Err(StatusCode::TOO_MANY_REQUESTS);
            }
            1
        }
        _ => return Err(StatusCode::CONFLICT),
    };
    if old.is_some() {
        sqlx::query("UPDATE routing_workspace_grants SET revision = $4, enabled = TRUE, scope_digest = $5, updated_at = clock_timestamp() WHERE account_id = $1 AND workspace_id = $2 AND recipient_identity = $3")
            .bind(&account).bind(workspace.as_slice()).bind(HOSTED_RECIPIENT).bind(next)
            .bind(hosted_scope_digest().as_slice())
            .execute(&mut *tx).await.map_err(|_| StatusCode::SERVICE_UNAVAILABLE)?;
    } else {
        sqlx::query("INSERT INTO routing_workspace_grants (account_id, workspace_id, recipient_identity, scope_digest, revision, enabled) VALUES ($1,$2,$3,$4,1,TRUE)")
            .bind(&account).bind(workspace.as_slice()).bind(HOSTED_RECIPIENT)
            .bind(hosted_scope_digest().as_slice())
            .execute(&mut *tx).await.map_err(|_| StatusCode::CONFLICT)?;
    }
    tx.commit()
        .await
        .map_err(|_| StatusCode::SERVICE_UNAVAILABLE)?;
    Ok(grant_response(
        body.workspace_id,
        hex::encode(hosted_scope_digest()),
        next,
        true,
    ))
}

pub async fn revoke_workspace_grant(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(body): Json<WorkspaceRevokeRequest>,
) -> Result<(HeaderMap, Json<WorkspaceGrantResponse>), StatusCode> {
    let (account, expiry) = routing_account_for_revoke(&state, &headers).await?;
    let workspace = parse_workspace_id(&body.workspace_id)?;
    let expected = revision(body.expected_revision)?;
    let pool = state.db.as_ref().ok_or(StatusCode::SERVICE_UNAVAILABLE)?;
    let mut tx = pool
        .begin()
        .await
        .map_err(|_| StatusCode::SERVICE_UNAVAILABLE)?;
    sqlx::query("INSERT INTO routing_account_gate(account_id) VALUES ($1) ON CONFLICT DO NOTHING")
        .bind(&account)
        .execute(&mut *tx)
        .await
        .map_err(|_| StatusCode::SERVICE_UNAVAILABLE)?;
    sqlx::query("SELECT account_id FROM routing_account_gate WHERE account_id = $1 FOR UPDATE")
        .bind(&account)
        .fetch_one(&mut *tx)
        .await
        .map_err(|_| StatusCode::SERVICE_UNAVAILABLE)?;
    recheck_account_after_gate(&headers, &account, expiry, &mut tx, true).await?;
    let old: Option<(i64, bool, Vec<u8>)> = sqlx::query_as(
        "SELECT revision, enabled, scope_digest FROM routing_workspace_grants WHERE account_id = $1 AND workspace_id = $2 AND recipient_identity = $3 FOR UPDATE",
    )
    .bind(&account).bind(workspace.as_slice()).bind(HOSTED_RECIPIENT)
    .fetch_optional(&mut *tx).await.map_err(|_| StatusCode::SERVICE_UNAVAILABLE)?;
    let Some((current, enabled, scope)) = old else {
        if expected != 0 {
            return Err(StatusCode::CONFLICT);
        }
        let count: i64 = sqlx::query_scalar(
            "SELECT count(*) FROM routing_workspace_grants WHERE account_id = $1",
        )
        .bind(&account)
        .fetch_one(&mut *tx)
        .await
        .map_err(|_| StatusCode::SERVICE_UNAVAILABLE)?;
        if count >= MAX_WORKSPACE_GRANTS_PER_ACCOUNT {
            return Err(StatusCode::TOO_MANY_REQUESTS);
        }
        sqlx::query("INSERT INTO routing_workspace_grants (account_id, workspace_id, recipient_identity, scope_digest, revision, enabled) VALUES ($1,$2,$3,$4,1,FALSE)")
            .bind(&account).bind(workspace.as_slice()).bind(HOSTED_RECIPIENT)
            .bind(hosted_scope_digest().as_slice())
            .execute(&mut *tx).await.map_err(|_| StatusCode::CONFLICT)?;
        tx.commit()
            .await
            .map_err(|_| StatusCode::SERVICE_UNAVAILABLE)?;
        return Ok(grant_response(
            body.workspace_id,
            hex::encode(hosted_scope_digest()),
            1,
            false,
        ));
    };
    if current != expected {
        return Err(StatusCode::CONFLICT);
    }
    let next = if enabled {
        current.checked_add(1).ok_or(StatusCode::CONFLICT)?
    } else {
        current
    };
    if enabled {
        sqlx::query("UPDATE routing_workspace_grants SET revision = $4, enabled = FALSE, updated_at = clock_timestamp() WHERE account_id = $1 AND workspace_id = $2 AND recipient_identity = $3")
            .bind(&account).bind(workspace.as_slice()).bind(HOSTED_RECIPIENT).bind(next)
            .execute(&mut *tx).await.map_err(|_| StatusCode::SERVICE_UNAVAILABLE)?;
    }
    tx.commit()
        .await
        .map_err(|_| StatusCode::SERVICE_UNAVAILABLE)?;
    Ok(grant_response(
        body.workspace_id,
        hex::encode(scope),
        next,
        false,
    ))
}

fn bearer(headers: &HeaderMap) -> Option<&str> {
    headers
        .get(header::AUTHORIZATION)?
        .to_str()
        .ok()?
        .strip_prefix("Bearer ")
        .filter(|token| !token.is_empty() && token.len() <= 8192)
}

fn fresh_token() -> Result<String, StatusCode> {
    let mut secret = [0u8; 32];
    getrandom::getrandom(&mut secret).map_err(|_| StatusCode::SERVICE_UNAVAILABLE)?;
    Ok(format!("{TOKEN_PREFIX}{}", URL_SAFE_NO_PAD.encode(secret)))
}

fn token_hash(token: &str) -> [u8; 32] {
    Sha256::digest(token.as_bytes()).into()
}

fn is_routing_token(token: &str) -> bool {
    token
        .strip_prefix(TOKEN_PREFIX)
        .and_then(|encoded| URL_SAFE_NO_PAD.decode(encoded).ok())
        .is_some_and(|secret| secret.len() == 32)
}

fn no_store_headers() -> HeaderMap {
    let mut headers = HeaderMap::new();
    headers.insert(header::CACHE_CONTROL, HeaderValue::from_static("no-store"));
    headers
}

pub async fn issue_token(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(body): Json<RoutingTokenRequest>,
) -> Result<(HeaderMap, Json<RoutingTokenResponse>), StatusCode> {
    if !state.routing_grant_experiment {
        return Err(StatusCode::SERVICE_UNAVAILABLE);
    }
    // A local/dev Link, shared service key, or missing database never mints a
    // sponsored credential. Ordinary entitlements do not grant this scope.
    let (account, session_expiry) = routing_account(&state, &headers).await?;
    let db = state.db.as_ref().ok_or(StatusCode::SERVICE_UNAVAILABLE)?;
    let workspace = parse_workspace_id(&body.workspace_id)?;
    let expected = revision(body.expected_grant_revision)?;
    if expected <= 0 {
        return Err(StatusCode::BAD_REQUEST);
    }
    let token = fresh_token()?;
    let digest = token_hash(&token);
    let mut tx = db
        .begin()
        .await
        .map_err(|_| StatusCode::SERVICE_UNAVAILABLE)?;
    sqlx::query("INSERT INTO routing_account_gate(account_id) VALUES ($1) ON CONFLICT DO NOTHING")
        .bind(&account)
        .execute(&mut *tx)
        .await
        .map_err(|_| StatusCode::SERVICE_UNAVAILABLE)?;
    let (window, count): (DateTime<Utc>, i32) = sqlx::query_as(
        "SELECT token_issue_window, token_issue_count FROM routing_account_gate WHERE account_id = $1 FOR UPDATE",
    ).bind(&account).fetch_one(&mut *tx).await.map_err(|_| StatusCode::SERVICE_UNAVAILABLE)?;
    let now: DateTime<Utc> = sqlx::query_scalar("SELECT clock_timestamp()")
        .fetch_one(&mut *tx)
        .await
        .map_err(|_| StatusCode::SERVICE_UNAVAILABLE)?;
    recheck_account_after_gate(&headers, &account, session_expiry, &mut tx, false).await?;
    let new_window = window <= now - chrono::Duration::minutes(1);
    if !new_window && count >= 10 {
        return Err(StatusCode::TOO_MANY_REQUESTS);
    }
    let grant: Option<(i64, bool, Vec<u8>)> = sqlx::query_as(
        "SELECT revision, enabled, scope_digest FROM routing_workspace_grants WHERE account_id = $1 AND workspace_id = $2 AND recipient_identity = $3 FOR UPDATE",
    ).bind(&account).bind(workspace.as_slice()).bind(HOSTED_RECIPIENT)
        .fetch_optional(&mut *tx).await.map_err(|_| StatusCode::SERVICE_UNAVAILABLE)?;
    if !grant.is_some_and(|(current, enabled, scope)| {
        enabled && current == expected && scope == hosted_scope_digest()
    }) {
        return Err(StatusCode::FORBIDDEN);
    }
    let expires_at: DateTime<Utc> = sqlx::query_scalar(
        r#"INSERT INTO routing_session_tokens (token_hash, account_id, scope, created_at, expires_at, workspace_id, grant_revision, recipient_identity, scope_digest)
           SELECT $1, $2, $3, clock_timestamp(),
                  LEAST(clock_timestamp() + interval '5 minutes', $4::timestamptz), $5, $6, $7, $8
           WHERE $4::timestamptz > clock_timestamp()
           ON CONFLICT (account_id, workspace_id, recipient_identity) WHERE workspace_id IS NOT NULL DO UPDATE SET
               token_hash = EXCLUDED.token_hash, scope = EXCLUDED.scope,
               created_at = clock_timestamp(),
               expires_at = LEAST(clock_timestamp() + interval '5 minutes', $4::timestamptz),
               revoked_at = NULL, grant_revision = EXCLUDED.grant_revision,
               scope_digest = EXCLUDED.scope_digest
           WHERE (routing_session_tokens.revoked_at IS NOT NULL
               OR routing_session_tokens.grant_revision <> EXCLUDED.grant_revision
               OR routing_session_tokens.created_at <= clock_timestamp() - interval '30 seconds')
               AND $4::timestamptz > clock_timestamp()
           RETURNING expires_at"#,
    )
    .bind(digest.as_slice())
    .bind(&account)
    .bind(SCOPE)
    .bind(session_expiry)
    .bind(workspace.as_slice())
    .bind(expected)
    .bind(HOSTED_RECIPIENT)
    .bind(hosted_scope_digest().as_slice())
    .fetch_optional(&mut *tx)
    .await
    .map_err(|_| StatusCode::SERVICE_UNAVAILABLE)?
    .ok_or(StatusCode::TOO_MANY_REQUESTS)?;
    sqlx::query("UPDATE routing_account_gate SET token_issue_window = CASE WHEN $2 THEN $3 ELSE token_issue_window END, token_issue_count = CASE WHEN $2 THEN 1 ELSE token_issue_count + 1 END WHERE account_id = $1")
        .bind(&account).bind(new_window).bind(now)
        .execute(&mut *tx).await.map_err(|_| StatusCode::SERVICE_UNAVAILABLE)?;
    tx.commit()
        .await
        .map_err(|_| StatusCode::SERVICE_UNAVAILABLE)?;
    Ok((
        no_store_headers(),
        Json(RoutingTokenResponse {
            token,
            token_type: "Bearer",
            scope: SCOPE,
            expires_at,
        }),
    ))
}

pub async fn revoke_token(State(state): State<AppState>, headers: HeaderMap) -> StatusCode {
    let (Some(_), Some(db)) = (state.routing_token_audience.as_ref(), state.db.as_ref()) else {
        return StatusCode::SERVICE_UNAVAILABLE;
    };
    let Some(token) = bearer(&headers).filter(|token| is_routing_token(token)) else {
        return StatusCode::UNAUTHORIZED;
    };
    let digest = token_hash(token);
    match sqlx::query(
        "UPDATE routing_session_tokens SET revoked_at = clock_timestamp() WHERE token_hash = $1 AND revoked_at IS NULL",
    )
    .bind(digest.as_slice())
    .execute(db)
    .await
    {
        Ok(_) => StatusCode::NO_CONTENT,
        Err(_) => StatusCode::SERVICE_UNAVAILABLE,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::body::Body;
    use axum::http::Request;
    use base64::engine::general_purpose::URL_SAFE_NO_PAD;
    use base64::Engine;
    use http_body_util::BodyExt;
    use jsonwebtoken::{encode, Algorithm, DecodingKey, EncodingKey, Header};
    use rand::rngs::OsRng;
    use rsa::pkcs1::EncodeRsaPrivateKey;
    use rsa::traits::PublicKeyParts;
    use rsa::RsaPrivateKey;
    use tower::ServiceExt;

    #[test]
    fn routing_token_is_random_and_hashes_without_storing_plaintext() {
        let first = fresh_token().unwrap();
        let second = fresh_token().unwrap();
        assert_ne!(first, second);
        assert!(is_routing_token(&first));
        assert_eq!(token_hash(&first), token_hash(&first));
        assert_ne!(token_hash(&first), token_hash(&second));
        assert!(!is_routing_token("shared-api-key"));
        assert!(!is_routing_token("pr1_a"));
    }

    #[tokio::test]
    async fn route_stays_closed_with_dev_auth_and_no_database() {
        let mut state = crate::contract_tests::test_state();
        state.api_key = Some("shared-key".into());
        state.routing_token_audience = Some("pytxo-routing".into());
        for (uri, method, body) in [
            ("/v1/routing/tokens", "POST", serde_json::json!({"workspace_id":"00".repeat(16), "expected_grant_revision":1}).to_string()),
            ("/v1/routing/tokens", "DELETE", "{}".into()),
            ("/v1/routing/workspace-grants", "POST", serde_json::json!({"workspace_id":"00".repeat(16), "recipient_identity":HOSTED_RECIPIENT, "scope_digest":hex::encode(hosted_scope_digest()), "expected_revision":0}).to_string()),
            ("/v1/routing/workspace-grants", "DELETE", serde_json::json!({"workspace_id":"00".repeat(16), "expected_revision":1}).to_string()),
        ] {
            let response = crate::build_router(state.clone()).oneshot(
                Request::builder().method(method).uri(uri)
                    .header("authorization", "Bearer shared-key")
                    .header("x-pytxo-user-id", "user_forged")
                    .header("content-type", "application/json")
                    .body(Body::from(body)).unwrap()
            ).await.unwrap();
            assert_eq!(response.status(), StatusCode::SERVICE_UNAVAILABLE, "{method} {uri}");
        }
    }

    #[tokio::test]
    async fn signed_session_issue_rotate_revoke_uses_postgres() {
        let Ok(url) = std::env::var("PYTXO_LINK_TEST_DATABASE_URL") else {
            return;
        };
        let parsed = reqwest::Url::parse(&url).unwrap();
        assert_eq!(parsed.host_str(), Some("127.0.0.1"));
        assert!(parsed.path().ends_with("/pytxo_routing_test"));
        let pool = crate::db::connect(&url).await.unwrap();
        let private = RsaPrivateKey::new(&mut OsRng, 2048).unwrap();
        let public = private.to_public_key();
        let n = URL_SAFE_NO_PAD.encode(public.n().to_bytes_be());
        let e = URL_SAFE_NO_PAD.encode(public.e().to_bytes_be());
        let validator = crate::jwt::JwksValidator::new(
            "https://unused-local-jwks.test/keys".into(),
            "https://issuer.test".into(),
        );
        validator.seed_key_for_test(
            "local-test",
            DecodingKey::from_rsa_components(&n, &e).unwrap(),
        );
        let der = private.to_pkcs1_der().unwrap();
        let mut header = Header::new(Algorithm::RS256);
        header.kid = Some("local-test".into());
        let mut random = [0u8; 8];
        getrandom::getrandom(&mut random).unwrap();
        let account = format!("user_routing_issue_{}", hex::encode(random));
        let jwt = encode(
            &header,
            &serde_json::json!({
                "iss":"https://issuer.test",
                "aud":"pytxo-routing",
                "exp": Utc::now().timestamp() + 120,
                "sub": account,
                "sid": "sess_test"
            }),
            &EncodingKey::from_rsa_der(der.as_bytes()),
        )
        .unwrap();
        let mut state = crate::contract_tests::test_state();
        state.db = Some(pool.clone());
        state.jwks = Some(std::sync::Arc::new(validator));
        state.routing_token_audience = Some("pytxo-routing".into());
        state.routing_grant_experiment = true;
        let mut workspace = [0u8; 16];
        getrandom::getrandom(&mut workspace).unwrap();
        let workspace_id = hex::encode(workspace);
        let scope_digest = hex::encode(hosted_scope_digest());
        let status_request = |bearer: &str, workspace: &str| {
            Request::builder()
                .method("GET")
                .uri(format!("/v1/routing/workspace-grants/{workspace}"))
                .header("authorization", format!("Bearer {bearer}"))
                .body(Body::empty())
                .unwrap()
        };
        let absent = crate::build_router(state.clone())
            .oneshot(status_request(&jwt, &workspace_id))
            .await
            .unwrap();
        assert_eq!(absent.status(), StatusCode::NOT_FOUND);
        // A lost first POST reply may leave the client unable to tell whether
        // the POST is still in flight. DELETE at revision zero must create a
        // tombstone so that an old POST cannot arrive after local revocation.
        let mut uncertain_workspace = [0u8; 16];
        getrandom::getrandom(&mut uncertain_workspace).unwrap();
        let uncertain_workspace = hex::encode(uncertain_workspace);
        let revoke_uncertain = |expected_revision: u64| {
            Request::builder()
                .method("DELETE")
                .uri("/v1/routing/workspace-grants")
                .header("authorization", format!("Bearer {jwt}"))
                .header("content-type", "application/json")
                .body(Body::from(
                    serde_json::json!({
                        "workspace_id": uncertain_workspace,
                        "expected_revision": expected_revision,
                    })
                    .to_string(),
                ))
                .unwrap()
        };
        let tombstone = crate::build_router(state.clone())
            .oneshot(revoke_uncertain(0))
            .await
            .unwrap();
        assert_eq!(tombstone.status(), StatusCode::OK);
        let tombstone: serde_json::Value =
            serde_json::from_slice(&tombstone.into_body().collect().await.unwrap().to_bytes())
                .unwrap();
        assert_eq!(tombstone["revision"], 1);
        assert_eq!(tombstone["enabled"], false);
        let delayed_post = Request::builder()
            .method("POST")
            .uri("/v1/routing/workspace-grants")
            .header("authorization", format!("Bearer {jwt}"))
            .header("content-type", "application/json")
            .body(Body::from(
                serde_json::json!({
                    "workspace_id": uncertain_workspace,
                    "recipient_identity": HOSTED_RECIPIENT,
                    "scope_digest": scope_digest,
                    "expected_revision": 0,
                })
                .to_string(),
            ))
            .unwrap();
        assert_eq!(
            crate::build_router(state.clone())
                .oneshot(delayed_post)
                .await
                .unwrap()
                .status(),
            StatusCode::CONFLICT
        );
        let repeated = crate::build_router(state.clone())
            .oneshot(revoke_uncertain(1))
            .await
            .unwrap();
        assert_eq!(repeated.status(), StatusCode::OK);
        let grant_request = |recipient: &str, scope: &str| {
            Request::builder()
                .method("POST")
                .uri("/v1/routing/workspace-grants")
                .header("authorization", format!("Bearer {jwt}"))
                .header("content-type", "application/json")
                .body(Body::from(
                    serde_json::json!({
                        "workspace_id": workspace_id,
                        "recipient_identity": recipient,
                        "scope_digest": scope,
                        "expected_revision": 0
                    })
                    .to_string(),
                ))
                .unwrap()
        };
        let wrong = crate::build_router(state.clone())
            .oneshot(grant_request("unreviewed-recipient", &scope_digest))
            .await
            .unwrap();
        assert_eq!(wrong.status(), StatusCode::BAD_REQUEST);
        let wrong_scope = crate::build_router(state.clone())
            .oneshot(grant_request(HOSTED_RECIPIENT, &"00".repeat(32)))
            .await
            .unwrap();
        assert_eq!(wrong_scope.status(), StatusCode::BAD_REQUEST);
        let granted = crate::build_router(state.clone())
            .oneshot(grant_request(HOSTED_RECIPIENT, &scope_digest))
            .await
            .unwrap();
        assert_eq!(granted.status(), StatusCode::OK);
        // Reconcile a committed POST when its response never reaches the client.
        let grant_status = crate::build_router(state.clone())
            .oneshot(status_request(&jwt, &workspace_id))
            .await
            .unwrap();
        assert_eq!(grant_status.status(), StatusCode::OK);
        assert_eq!(
            grant_status.headers().get(header::CACHE_CONTROL).unwrap(),
            "no-store"
        );
        let grant_status: serde_json::Value =
            serde_json::from_slice(&grant_status.into_body().collect().await.unwrap().to_bytes())
                .unwrap();
        assert_eq!(grant_status["revision"], 1);
        assert_eq!(grant_status["enabled"], true);
        assert_eq!(grant_status["scope_digest"], scope_digest);
        let other_jwt = encode(
            &header,
            &serde_json::json!({
                "iss":"https://issuer.test", "aud":"pytxo-routing",
                "exp": Utc::now().timestamp() + 120, "sub": format!("{account}_other"),
                "sid":"sess_other"
            }),
            &EncodingKey::from_rsa_der(der.as_bytes()),
        )
        .unwrap();
        assert_eq!(
            crate::build_router(state.clone())
                .oneshot(status_request(&other_jwt, &workspace_id))
                .await
                .unwrap()
                .status(),
            StatusCode::NOT_FOUND
        );
        let request = |method, bearer: &str| {
            Request::builder()
                .method(method)
                .uri("/v1/routing/tokens")
                .header("authorization", format!("Bearer {bearer}"))
                .header("content-type", "application/json")
                .body(Body::from(
                    serde_json::json!({
                        "workspace_id": workspace_id,
                        "expected_grant_revision": 1
                    })
                    .to_string(),
                ))
                .unwrap()
        };
        let denied = crate::build_router(state.clone())
            .oneshot(request("POST", "shared-key"))
            .await
            .unwrap();
        assert_eq!(denied.status(), StatusCode::UNAUTHORIZED);
        let first = crate::build_router(state.clone())
            .oneshot(request("POST", &jwt))
            .await
            .unwrap();
        assert_eq!(first.status(), StatusCode::OK);
        assert_eq!(
            first.headers().get(header::CACHE_CONTROL).unwrap(),
            "no-store"
        );
        let bytes = first.into_body().collect().await.unwrap().to_bytes();
        let payload: serde_json::Value = serde_json::from_slice(&bytes).unwrap();
        let token = payload["token"].as_str().unwrap().to_string();
        let stored: i64 = sqlx::query_scalar(
            "SELECT count(*) FROM routing_session_tokens WHERE account_id = $1 AND token_hash = $2",
        )
        .bind(&account)
        .bind(token_hash(&token).as_slice())
        .fetch_one(&pool)
        .await
        .unwrap();
        assert_eq!(stored, 1);
        let cooldown = crate::build_router(state.clone())
            .oneshot(request("POST", &jwt))
            .await
            .unwrap();
        assert_eq!(cooldown.status(), StatusCode::TOO_MANY_REQUESTS);
        let revoked = crate::build_router(state.clone())
            .oneshot(request("DELETE", &token))
            .await
            .unwrap();
        assert_eq!(revoked.status(), StatusCode::NO_CONTENT);
        let again = crate::build_router(state.clone())
            .oneshot(request("POST", &jwt))
            .await
            .unwrap();
        assert_eq!(again.status(), StatusCode::OK);
        let mut other_workspace = [0u8; 16];
        getrandom::getrandom(&mut other_workspace).unwrap();
        let other_workspace = hex::encode(other_workspace);
        let other_grant = crate::build_router(state.clone())
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/v1/routing/workspace-grants")
                    .header("authorization", format!("Bearer {jwt}"))
                    .header("content-type", "application/json")
                    .body(Body::from(
                        serde_json::json!({
                            "workspace_id": other_workspace, "recipient_identity": HOSTED_RECIPIENT,
                            "scope_digest": scope_digest, "expected_revision": 0
                        })
                        .to_string(),
                    ))
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(other_grant.status(), StatusCode::OK);
        let other_token = crate::build_router(state.clone())
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/v1/routing/tokens")
                    .header("authorization", format!("Bearer {jwt}"))
                    .header("content-type", "application/json")
                    .body(Body::from(
                        serde_json::json!({
                            "workspace_id": other_workspace, "expected_grant_revision": 1
                        })
                        .to_string(),
                    ))
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(other_token.status(), StatusCode::OK);
        sqlx::query("UPDATE routing_workspace_grants SET scope_digest = $4 WHERE account_id = $1 AND workspace_id = $2 AND recipient_identity = $3")
            .bind(&account).bind(hex::decode(&other_workspace).unwrap())
            .bind(HOSTED_RECIPIENT).bind([0u8; 32].as_slice())
            .execute(&pool).await.unwrap();
        let old_scope_status = crate::build_router(state.clone())
            .oneshot(status_request(&jwt, &other_workspace))
            .await
            .unwrap();
        assert_eq!(old_scope_status.status(), StatusCode::OK);
        let old_scope_status: serde_json::Value = serde_json::from_slice(
            &old_scope_status
                .into_body()
                .collect()
                .await
                .unwrap()
                .to_bytes(),
        )
        .unwrap();
        assert_eq!(old_scope_status["scope_digest"], "00".repeat(32));
        let refreshed_scope = crate::build_router(state.clone())
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/v1/routing/workspace-grants")
                    .header("authorization", format!("Bearer {jwt}"))
                    .header("content-type", "application/json")
                    .body(Body::from(
                        serde_json::json!({
                            "workspace_id":other_workspace, "recipient_identity":HOSTED_RECIPIENT,
                            "scope_digest":scope_digest, "expected_revision":1
                        })
                        .to_string(),
                    ))
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(refreshed_scope.status(), StatusCode::OK);
        let refreshed_scope: serde_json::Value = serde_json::from_slice(
            &refreshed_scope
                .into_body()
                .collect()
                .await
                .unwrap()
                .to_bytes(),
        )
        .unwrap();
        assert_eq!(refreshed_scope["revision"], 2);
        let workspace_tokens: i64 = sqlx::query_scalar("SELECT count(*) FROM routing_session_tokens WHERE account_id = $1 AND workspace_id IS NOT NULL")
            .bind(&account).fetch_one(&pool).await.unwrap();
        assert_eq!(workspace_tokens, 2);
        let revoke_grant = crate::build_router(state.clone())
            .oneshot(
                Request::builder()
                    .method("DELETE")
                    .uri("/v1/routing/workspace-grants")
                    .header("authorization", format!("Bearer {jwt}"))
                    .header("content-type", "application/json")
                    .body(Body::from(
                        serde_json::json!({
                            "workspace_id": workspace_id, "expected_revision": 1
                        })
                        .to_string(),
                    ))
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(revoke_grant.status(), StatusCode::OK);
        let revoked_body: serde_json::Value =
            serde_json::from_slice(&revoke_grant.into_body().collect().await.unwrap().to_bytes())
                .unwrap();
        assert_eq!(revoked_body["revision"], 2);
        assert_eq!(revoked_body["enabled"], false);
        // A committed DELETE is recoverable without guessing the new revision.
        let revoked_status = crate::build_router(state.clone())
            .oneshot(status_request(&jwt, &workspace_id))
            .await
            .unwrap();
        assert_eq!(revoked_status.status(), StatusCode::OK);
        let revoked_status: serde_json::Value = serde_json::from_slice(
            &revoked_status
                .into_body()
                .collect()
                .await
                .unwrap()
                .to_bytes(),
        )
        .unwrap();
        assert_eq!(revoked_status["revision"], 2);
        assert_eq!(revoked_status["enabled"], false);
        let denied_issue = crate::build_router(state.clone())
            .oneshot(request("POST", &jwt))
            .await
            .unwrap();
        assert_eq!(denied_issue.status(), StatusCode::FORBIDDEN);
        let stale_regrant = crate::build_router(state.clone())
            .oneshot(grant_request(HOSTED_RECIPIENT, &scope_digest))
            .await
            .unwrap();
        assert_eq!(stale_regrant.status(), StatusCode::CONFLICT);
        // Retain revision tombstones, but bound total rows per account.
        sqlx::query("INSERT INTO routing_workspace_grants (account_id, workspace_id, recipient_identity, scope_digest, revision, enabled) SELECT $1, decode(md5($1 || ':' || n::text), 'hex'), $2, $3, 1, FALSE FROM generate_series(1, 62) AS n")
            .bind(&account).bind(HOSTED_RECIPIENT).bind(hosted_scope_digest().as_slice())
            .execute(&pool).await.unwrap();
        let mut over_limit_workspace = [0u8; 16];
        getrandom::getrandom(&mut over_limit_workspace).unwrap();
        let over_limit = crate::build_router(state.clone())
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/v1/routing/workspace-grants")
                    .header("authorization", format!("Bearer {jwt}"))
                    .header("content-type", "application/json")
                    .body(Body::from(
                        serde_json::json!({
                            "workspace_id":hex::encode(over_limit_workspace),
                            "recipient_identity":HOSTED_RECIPIENT,
                            "scope_digest":scope_digest, "expected_revision":0
                        })
                        .to_string(),
                    ))
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(over_limit.status(), StatusCode::TOO_MANY_REQUESTS);
        sqlx::query("DELETE FROM routing_session_tokens WHERE account_id = $1")
            .bind(&account)
            .execute(&pool)
            .await
            .unwrap();
        sqlx::query("DELETE FROM routing_workspace_grants WHERE account_id = $1")
            .bind(&account)
            .execute(&pool)
            .await
            .unwrap();
    }
}
