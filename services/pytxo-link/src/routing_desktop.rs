//! Experimental website-to-Desktop account return. Codes are single-use,
//! PKCE-bound and never carry a Clerk bearer through a custom app link.

use axum::extract::State;
use axum::http::{header, HeaderMap, HeaderValue, StatusCode};
use axum::Json;
use base64::engine::general_purpose::URL_SAFE_NO_PAD;
use base64::Engine;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

use crate::routing::signed_routing_account;
use crate::state::AppState;

const CODE_PREFIX: &str = "pdc1_";
const SESSION_PREFIX: &str = "pds1_";
const SESSION_SCOPE: &str = "routing:grants:v1";
const RECOVERY_SCOPE: &str = "routing:revoke:v1";
const CODE_SECONDS: i64 = 60;
const SESSION_HOURS: i64 = 24;
const CODES_PER_MINUTE: i64 = 3;

type AuthorizationRow = (
    String,
    Vec<u8>,
    Vec<u8>,
    String,
    DateTime<Utc>,
    Option<DateTime<Utc>>,
);

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub struct DesktopAuthorizationRequest {
    state: String,
    code_challenge: String,
    #[serde(default)]
    recovery_only: bool,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub struct DesktopExchangeRequest {
    code: String,
    state: String,
    code_verifier: String,
}

#[derive(Serialize)]
pub struct DesktopAuthorizationResponse {
    code: String,
    expires_at: DateTime<Utc>,
}

#[derive(Serialize)]
pub struct DesktopSessionResponse {
    account_id: String,
    expires_at: DateTime<Utc>,
    scope: String,
}

#[derive(Serialize)]
pub struct DesktopExchangeResponse {
    token: String,
    account_id: String,
    expires_at: DateTime<Utc>,
    scope: &'static str,
}

fn no_store() -> HeaderMap {
    let mut headers = HeaderMap::new();
    headers.insert(header::CACHE_CONTROL, HeaderValue::from_static("no-store"));
    headers
}

fn bearer(headers: &HeaderMap) -> Option<&str> {
    headers
        .get(header::AUTHORIZATION)?
        .to_str()
        .ok()?
        .strip_prefix("Bearer ")
        .filter(|token| !token.is_empty() && token.len() <= 8192)
}

fn parse_random_bytes(value: &str) -> Option<[u8; 32]> {
    if value.len() != 43 {
        return None;
    }
    let bytes: [u8; 32] = URL_SAFE_NO_PAD.decode(value).ok()?.try_into().ok()?;
    (URL_SAFE_NO_PAD.encode(bytes) == value).then_some(bytes)
}

fn random_secret(prefix: &str) -> Result<String, StatusCode> {
    let mut secret = [0u8; 32];
    getrandom::getrandom(&mut secret).map_err(|_| StatusCode::SERVICE_UNAVAILABLE)?;
    Ok(format!("{prefix}{}", URL_SAFE_NO_PAD.encode(secret)))
}

fn hash(value: &str) -> [u8; 32] {
    Sha256::digest(value.as_bytes()).into()
}

fn is_secret(token: &str, prefix: &str) -> bool {
    token
        .strip_prefix(prefix)
        .and_then(parse_random_bytes)
        .is_some()
}

/// Only the routing grants/token-issue endpoints call this. General Link auth,
/// entitlements and Ultra never inspect Desktop routing credentials.
async fn desktop_session(
    state: &AppState,
    token: &str,
    allow_recovery: bool,
) -> Result<Option<(String, DateTime<Utc>, String)>, StatusCode> {
    if !is_secret(token, SESSION_PREFIX) {
        return Ok(None);
    }
    if state.routing_token_audience.is_none() {
        return Err(StatusCode::SERVICE_UNAVAILABLE);
    }
    let db = state.db.as_ref().ok_or(StatusCode::SERVICE_UNAVAILABLE)?;
    let row: Option<(String, DateTime<Utc>, String)> = sqlx::query_as(
        "SELECT account_id, expires_at, scope FROM routing_desktop_sessions WHERE token_hash = $1 AND (scope = $2 OR ($3 AND scope = $4)) AND revoked_at IS NULL AND expires_at > clock_timestamp()",
    )
    .bind(hash(token).as_slice())
    .bind(SESSION_SCOPE)
    .bind(allow_recovery)
    .bind(RECOVERY_SCOPE)
    .fetch_optional(db)
    .await
    .map_err(|_| StatusCode::SERVICE_UNAVAILABLE)?;
    Ok(row)
}

pub(crate) async fn desktop_account(
    state: &AppState,
    token: &str,
) -> Result<Option<(String, DateTime<Utc>)>, StatusCode> {
    Ok(desktop_session(state, token, false)
        .await?
        .map(|(account, expiry, _)| (account, expiry)))
}

pub(crate) async fn desktop_account_for_revoke(
    state: &AppState,
    token: &str,
) -> Result<Option<(String, DateTime<Utc>)>, StatusCode> {
    Ok(desktop_session(state, token, true)
        .await?
        .map(|(account, expiry, _)| (account, expiry)))
}

pub async fn authorize(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(body): Json<DesktopAuthorizationRequest>,
) -> Result<(HeaderMap, Json<DesktopAuthorizationResponse>), StatusCode> {
    if !body.recovery_only && !state.routing_grant_experiment {
        return Err(StatusCode::SERVICE_UNAVAILABLE);
    }
    let (account, _) = signed_routing_account(&state, &headers).await?;
    parse_random_bytes(&body.state).ok_or(StatusCode::BAD_REQUEST)?;
    let challenge = parse_random_bytes(&body.code_challenge).ok_or(StatusCode::BAD_REQUEST)?;
    let db = state.db.as_ref().ok_or(StatusCode::SERVICE_UNAVAILABLE)?;
    let code = random_secret(CODE_PREFIX)?;
    let scope = if body.recovery_only {
        RECOVERY_SCOPE
    } else {
        SESSION_SCOPE
    };
    let mut tx = db
        .begin()
        .await
        .map_err(|_| StatusCode::SERVICE_UNAVAILABLE)?;
    if body.recovery_only {
        // A website with new connections switched off may still help an
        // existing Routing account reconcile a local grant. This is not an
        // account enrollment path for someone who has never used Routing.
        let established: bool = sqlx::query_scalar(
            "SELECT EXISTS(SELECT 1 FROM routing_account_gate WHERE account_id = $1)",
        )
        .bind(&account)
        .fetch_one(&mut *tx)
        .await
        .map_err(|_| StatusCode::SERVICE_UNAVAILABLE)?;
        if !established {
            return Err(StatusCode::FORBIDDEN);
        }
    }
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
    let recent: i64 = sqlx::query_scalar(
        "SELECT count(*) FROM routing_desktop_codes WHERE account_id = $1 AND created_at > clock_timestamp() - interval '1 minute'",
    )
    .bind(&account)
    .fetch_one(&mut *tx)
    .await
    .map_err(|_| StatusCode::SERVICE_UNAVAILABLE)?;
    if recent >= CODES_PER_MINUTE {
        return Err(StatusCode::TOO_MANY_REQUESTS);
    }
    let expires_at: DateTime<Utc> = sqlx::query_scalar(
        "INSERT INTO routing_desktop_codes (code_hash, account_id, state_hash, challenge, scope, expires_at) VALUES ($1,$2,$3,$4,$5,clock_timestamp() + $6 * interval '1 second') RETURNING expires_at",
    )
    .bind(hash(&code).as_slice())
    .bind(&account)
    .bind(hash(&body.state).as_slice())
    .bind(challenge.as_slice())
    .bind(scope)
    .bind(CODE_SECONDS)
    .fetch_one(&mut *tx)
    .await
    .map_err(|_| StatusCode::SERVICE_UNAVAILABLE)?;
    tx.commit()
        .await
        .map_err(|_| StatusCode::SERVICE_UNAVAILABLE)?;
    Ok((
        no_store(),
        Json(DesktopAuthorizationResponse { code, expires_at }),
    ))
}

pub async fn exchange(
    State(state): State<AppState>,
    Json(body): Json<DesktopExchangeRequest>,
) -> Result<(HeaderMap, Json<DesktopExchangeResponse>), StatusCode> {
    if state.routing_token_audience.is_none() {
        return Err(StatusCode::SERVICE_UNAVAILABLE);
    }
    if !is_secret(&body.code, CODE_PREFIX)
        || parse_random_bytes(&body.state).is_none()
        || parse_random_bytes(&body.code_verifier).is_none()
    {
        return Err(StatusCode::BAD_REQUEST);
    }
    let db = state.db.as_ref().ok_or(StatusCode::SERVICE_UNAVAILABLE)?;
    let challenge = hash(&body.code_verifier);
    let state_hash = hash(&body.state);
    let token = random_secret(SESSION_PREFIX)?;
    let mut tx = db
        .begin()
        .await
        .map_err(|_| StatusCode::SERVICE_UNAVAILABLE)?;
    // All account-scoped issue, exchange and revoke operations take the same
    // gate first. A lost-device revoke cannot race an older code into a new
    // native credential after the revoke has committed.
    let code_account: Option<String> =
        sqlx::query_scalar("SELECT account_id FROM routing_desktop_codes WHERE code_hash = $1")
            .bind(hash(&body.code).as_slice())
            .fetch_optional(&mut *tx)
            .await
            .map_err(|_| StatusCode::SERVICE_UNAVAILABLE)?;
    let code_account = code_account.ok_or(StatusCode::UNAUTHORIZED)?;
    sqlx::query("SELECT account_id FROM routing_account_gate WHERE account_id = $1 FOR UPDATE")
        .bind(&code_account)
        .fetch_one(&mut *tx)
        .await
        .map_err(|_| StatusCode::SERVICE_UNAVAILABLE)?;
    let row: Option<AuthorizationRow> =
        sqlx::query_as(
            "SELECT account_id, state_hash, challenge, scope, expires_at, consumed_at FROM routing_desktop_codes WHERE code_hash = $1 FOR UPDATE",
        )
        .bind(hash(&body.code).as_slice())
        .fetch_optional(&mut *tx)
        .await
        .map_err(|_| StatusCode::SERVICE_UNAVAILABLE)?;
    let (account, expected_state, expected_challenge, scope, expires, consumed) =
        row.ok_or(StatusCode::UNAUTHORIZED)?;
    let scope = match scope.as_str() {
        SESSION_SCOPE => SESSION_SCOPE,
        RECOVERY_SCOPE => RECOVERY_SCOPE,
        _ => return Err(StatusCode::UNAUTHORIZED),
    };
    // A code issued just before the experiment switch turns off must not
    // become a new full-scope Desktop credential after the switch changes.
    if scope == SESSION_SCOPE && !state.routing_grant_experiment {
        return Err(StatusCode::SERVICE_UNAVAILABLE);
    }
    let now: DateTime<Utc> = sqlx::query_scalar("SELECT clock_timestamp()")
        .fetch_one(&mut *tx)
        .await
        .map_err(|_| StatusCode::SERVICE_UNAVAILABLE)?;
    if expires <= now
        || consumed.is_some()
        || expected_state != state_hash
        || expected_challenge != challenge
    {
        return Err(StatusCode::UNAUTHORIZED);
    }
    sqlx::query(
        "UPDATE routing_desktop_codes SET consumed_at = clock_timestamp() WHERE code_hash = $1",
    )
    .bind(hash(&body.code).as_slice())
    .execute(&mut *tx)
    .await
    .map_err(|_| StatusCode::SERVICE_UNAVAILABLE)?;
    let expires_at: DateTime<Utc> = sqlx::query_scalar(
        "INSERT INTO routing_desktop_sessions (token_hash, account_id, scope, expires_at) VALUES ($1,$2,$3,clock_timestamp() + $4 * interval '1 hour') ON CONFLICT (account_id) DO UPDATE SET token_hash = EXCLUDED.token_hash, scope = EXCLUDED.scope, created_at = clock_timestamp(), expires_at = EXCLUDED.expires_at, revoked_at = NULL RETURNING expires_at",
    )
    .bind(hash(&token).as_slice())
    .bind(&account)
    .bind(scope)
    .bind(SESSION_HOURS)
    .fetch_one(&mut *tx)
    .await
    .map_err(|_| StatusCode::SERVICE_UNAVAILABLE)?;
    tx.commit()
        .await
        .map_err(|_| StatusCode::SERVICE_UNAVAILABLE)?;
    Ok((
        no_store(),
        Json(DesktopExchangeResponse {
            token,
            account_id: account,
            expires_at,
            scope,
        }),
    ))
}

pub async fn session_status(
    State(state): State<AppState>,
    headers: HeaderMap,
) -> Result<(HeaderMap, Json<DesktopSessionResponse>), StatusCode> {
    let token = bearer(&headers).ok_or(StatusCode::UNAUTHORIZED)?;
    let (account_id, expires_at, scope) = desktop_session(&state, token, true)
        .await?
        .ok_or(StatusCode::UNAUTHORIZED)?;
    Ok((
        no_store(),
        Json(DesktopSessionResponse {
            account_id,
            expires_at,
            scope,
        }),
    ))
}

pub async fn revoke_session(
    State(state): State<AppState>,
    headers: HeaderMap,
) -> Result<StatusCode, StatusCode> {
    let Some(token) = bearer(&headers).filter(|token| is_secret(token, SESSION_PREFIX)) else {
        return Err(StatusCode::UNAUTHORIZED);
    };
    let (account, _) = desktop_account_for_revoke(&state, token)
        .await?
        .ok_or(StatusCode::UNAUTHORIZED)?;
    let db = state.db.as_ref().ok_or(StatusCode::SERVICE_UNAVAILABLE)?;
    let mut tx = db
        .begin()
        .await
        .map_err(|_| StatusCode::SERVICE_UNAVAILABLE)?;
    lock_account(&mut tx, &account).await?;
    let still_current: bool = sqlx::query_scalar(
        "SELECT EXISTS(SELECT 1 FROM routing_desktop_sessions WHERE account_id = $1 AND token_hash = $2 AND revoked_at IS NULL AND expires_at > clock_timestamp())",
    )
    .bind(&account)
    .bind(hash(token).as_slice())
    .fetch_one(&mut *tx)
    .await
    .map_err(|_| StatusCode::SERVICE_UNAVAILABLE)?;
    if !still_current {
        return Err(StatusCode::UNAUTHORIZED);
    }
    revoke_account(&mut tx, &account).await?;
    tx.commit()
        .await
        .map_err(|_| StatusCode::SERVICE_UNAVAILABLE)?;
    Ok(StatusCode::NO_CONTENT)
}

/// Browser-authenticated lost-device recovery revokes every hosted routing
/// grant and issued evaluation token as well as the native credential.
pub async fn revoke_all(
    State(state): State<AppState>,
    headers: HeaderMap,
) -> Result<StatusCode, StatusCode> {
    let (account, _) = signed_routing_account(&state, &headers).await?;
    let db = state.db.as_ref().ok_or(StatusCode::SERVICE_UNAVAILABLE)?;
    let mut tx = db
        .begin()
        .await
        .map_err(|_| StatusCode::SERVICE_UNAVAILABLE)?;
    lock_account(&mut tx, &account).await?;
    revoke_account(&mut tx, &account).await?;
    tx.commit()
        .await
        .map_err(|_| StatusCode::SERVICE_UNAVAILABLE)?;
    Ok(StatusCode::NO_CONTENT)
}

async fn lock_account(
    tx: &mut sqlx::Transaction<'_, sqlx::Postgres>,
    account: &str,
) -> Result<(), StatusCode> {
    sqlx::query("INSERT INTO routing_account_gate(account_id) VALUES ($1) ON CONFLICT DO NOTHING")
        .bind(account)
        .execute(&mut **tx)
        .await
        .map_err(|_| StatusCode::SERVICE_UNAVAILABLE)?;
    sqlx::query("SELECT account_id FROM routing_account_gate WHERE account_id = $1 FOR UPDATE")
        .bind(account)
        .fetch_one(&mut **tx)
        .await
        .map_err(|_| StatusCode::SERVICE_UNAVAILABLE)?;
    Ok(())
}

async fn revoke_account(
    tx: &mut sqlx::Transaction<'_, sqlx::Postgres>,
    account: &str,
) -> Result<(), StatusCode> {
    // Lock order after the account gate: codes, native session, grant, token.
    // Claim admission separately checks the current grant before any send.
    sqlx::query("UPDATE routing_desktop_codes SET consumed_at = clock_timestamp() WHERE account_id = $1 AND consumed_at IS NULL")
        .bind(account).execute(&mut **tx).await.map_err(|_| StatusCode::SERVICE_UNAVAILABLE)?;
    sqlx::query("UPDATE routing_desktop_sessions SET revoked_at = clock_timestamp() WHERE account_id = $1 AND revoked_at IS NULL")
        .bind(account).execute(&mut **tx).await.map_err(|_| StatusCode::SERVICE_UNAVAILABLE)?;
    sqlx::query("UPDATE routing_workspace_grants SET enabled = FALSE, revision = revision + 1, updated_at = clock_timestamp() WHERE account_id = $1 AND enabled")
        .bind(account).execute(&mut **tx).await.map_err(|_| StatusCode::SERVICE_UNAVAILABLE)?;
    sqlx::query("UPDATE routing_session_tokens SET revoked_at = clock_timestamp() WHERE account_id = $1 AND revoked_at IS NULL")
        .bind(account).execute(&mut **tx).await.map_err(|_| StatusCode::SERVICE_UNAVAILABLE)?;
    Ok(())
}

/// This runs even when the experiment is disabled, so shutting off hosted
/// routing does not silently extend retention. Short batches avoid long locks.
pub(crate) async fn prune_expired(db: &sqlx::PgPool) -> Result<(), sqlx::Error> {
    sqlx::query(
        "DELETE FROM routing_desktop_codes WHERE code_hash IN (SELECT code_hash FROM routing_desktop_codes WHERE expires_at < clock_timestamp() - interval '1 day' ORDER BY expires_at LIMIT 1000 FOR UPDATE SKIP LOCKED)",
    )
    .execute(db)
    .await?;
    sqlx::query(
        "DELETE FROM routing_desktop_sessions WHERE token_hash IN (SELECT token_hash FROM routing_desktop_sessions WHERE expires_at < clock_timestamp() - interval '30 days' ORDER BY expires_at LIMIT 1000 FOR UPDATE SKIP LOCKED)",
    )
    .execute(db)
    .await?;
    Ok(())
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
    use serde_json::{json, Value};
    use tower::ServiceExt;

    #[test]
    fn pkce_inputs_are_canonical_and_scoped_secrets_are_distinct() {
        let verifier = URL_SAFE_NO_PAD.encode([7u8; 32]);
        assert_eq!(parse_random_bytes(&verifier), Some([7u8; 32]));
        assert!(parse_random_bytes("short").is_none());
        assert!(parse_random_bytes(&format!("{}=", verifier)).is_none());
        assert!(parse_random_bytes(&format!("+{}", &verifier[1..])).is_none());
        assert!(is_secret(&random_secret(CODE_PREFIX).unwrap(), CODE_PREFIX));
        assert!(is_secret(
            &random_secret(SESSION_PREFIX).unwrap(),
            SESSION_PREFIX
        ));
        assert!(!is_secret(
            &random_secret(CODE_PREFIX).unwrap(),
            SESSION_PREFIX
        ));
    }

    #[test]
    fn recovery_code_request_is_explicit_and_unknown_fields_are_rejected() {
        let normal: DesktopAuthorizationRequest = serde_json::from_value(json!({
            "state": "state", "code_challenge": "challenge"
        }))
        .unwrap();
        assert!(!normal.recovery_only);
        let recovery: DesktopAuthorizationRequest = serde_json::from_value(json!({
            "state": "state", "code_challenge": "challenge", "recovery_only": true
        }))
        .unwrap();
        assert!(recovery.recovery_only);
        assert!(
            serde_json::from_value::<DesktopAuthorizationRequest>(json!({
                "state": "state", "code_challenge": "challenge", "recovery_only": "true"
            }))
            .is_err()
        );
        assert!(
            serde_json::from_value::<DesktopAuthorizationRequest>(json!({
                "state": "state", "code_challenge": "challenge", "unused": 1
            }))
            .is_err()
        );
    }

    fn request(method: &str, uri: &str, bearer: Option<&str>, body: Value) -> Request<Body> {
        let mut builder = Request::builder()
            .method(method)
            .uri(uri)
            .header("content-type", "application/json");
        if let Some(token) = bearer {
            builder = builder.header("authorization", format!("Bearer {token}"));
        }
        builder.body(Body::from(body.to_string())).unwrap()
    }

    async fn json_body(response: axum::response::Response) -> Value {
        serde_json::from_slice(&response.into_body().collect().await.unwrap().to_bytes()).unwrap()
    }

    #[tokio::test]
    async fn bridge_stays_closed_without_experiment_configuration() {
        let state = crate::contract_tests::test_state();
        let random = URL_SAFE_NO_PAD.encode([3u8; 32]);
        let issued = crate::build_router(state.clone())
            .oneshot(request(
                "POST",
                "/v1/routing/desktop-authorizations",
                Some("shared-key"),
                json!({"state":random,"code_challenge":random}),
            ))
            .await
            .unwrap();
        assert_eq!(issued.status(), StatusCode::SERVICE_UNAVAILABLE);
        let exchanged = crate::build_router(state)
            .oneshot(request(
                "POST",
                "/v1/routing/desktop-exchange",
                None,
                json!({"code":random,"state":random,"code_verifier":random}),
            ))
            .await
            .unwrap();
        assert_eq!(exchanged.status(), StatusCode::SERVICE_UNAVAILABLE);
    }

    #[tokio::test]
    async fn signed_browser_pkce_exchange_is_one_use_and_desktop_scope_is_narrow() {
        let Ok(url) = std::env::var("PYTXO_LINK_TEST_DATABASE_URL") else {
            return;
        };
        let parsed = reqwest::Url::parse(&url).unwrap();
        assert_eq!(parsed.host_str(), Some("127.0.0.1"));
        assert!(parsed.path().ends_with("/pytxo_routing_test"));
        let db = crate::db::connect(&url).await.unwrap();
        let private = RsaPrivateKey::new(&mut OsRng, 2048).unwrap();
        let public = private.to_public_key();
        let validator = crate::jwt::JwksValidator::new(
            "https://unused-local-jwks.test/keys".into(),
            "https://issuer.test".into(),
        );
        validator.seed_key_for_test(
            "local-test",
            DecodingKey::from_rsa_components(
                &URL_SAFE_NO_PAD.encode(public.n().to_bytes_be()),
                &URL_SAFE_NO_PAD.encode(public.e().to_bytes_be()),
            )
            .unwrap(),
        );
        let mut header = Header::new(Algorithm::RS256);
        header.kid = Some("local-test".into());
        let mut random = [0u8; 8];
        getrandom::getrandom(&mut random).unwrap();
        let account = format!("user_desktop_bridge_{}", hex::encode(random));
        let signed = |sub: &str, audience: &str| {
            encode(
                &header,
                &json!({"iss":"https://issuer.test","aud":audience,"exp":Utc::now().timestamp()+180,"sub":sub,"sid":"sess_bridge"}),
                &EncodingKey::from_rsa_der(private.to_pkcs1_der().unwrap().as_bytes()),
            )
            .unwrap()
        };
        let jwt = signed(&account, "pytxo-routing");
        let mut state = crate::contract_tests::test_state();
        state.require_auth = true;
        state.routing_token_audience = Some("pytxo-routing".into());
        state.routing_grant_experiment = true;
        state.db = Some(db.clone());
        state.jwks = Some(std::sync::Arc::new(validator));
        let state_for_recheck = state.clone();
        let mut off_state = state.clone();
        off_state.routing_grant_experiment = false;
        let off_router = crate::build_router(off_state);
        let router = crate::build_router(state);
        let browser_state = URL_SAFE_NO_PAD.encode([11u8; 32]);
        let verifier = URL_SAFE_NO_PAD.encode([12u8; 32]);
        let challenge = URL_SAFE_NO_PAD.encode(hash(&verifier));
        let authorize_body = || json!({"state":browser_state,"code_challenge":challenge});
        let stopped_normal = off_router
            .clone()
            .oneshot(request(
                "POST",
                "/v1/routing/desktop-authorizations",
                Some(&jwt),
                authorize_body(),
            ))
            .await
            .unwrap();
        assert_eq!(stopped_normal.status(), StatusCode::SERVICE_UNAVAILABLE);
        let wrong_aud = router
            .clone()
            .oneshot(request(
                "POST",
                "/v1/routing/desktop-authorizations",
                Some(&signed(&account, "wrong")),
                authorize_body(),
            ))
            .await
            .unwrap();
        assert_eq!(wrong_aud.status(), StatusCode::UNAUTHORIZED);
        let first_recovery = router
            .clone()
            .oneshot(request(
                "POST",
                "/v1/routing/desktop-authorizations",
                Some(&jwt),
                json!({"state":browser_state,"code_challenge":challenge,"recovery_only":true}),
            ))
            .await
            .unwrap();
        assert_eq!(first_recovery.status(), StatusCode::FORBIDDEN);
        let authorized = router
            .clone()
            .oneshot(request(
                "POST",
                "/v1/routing/desktop-authorizations",
                Some(&jwt),
                authorize_body(),
            ))
            .await
            .unwrap();
        assert_eq!(authorized.status(), StatusCode::OK);
        assert_eq!(
            authorized.headers().get(header::CACHE_CONTROL).unwrap(),
            "no-store"
        );
        let issued = json_body(authorized).await;
        let code = issued["code"].as_str().unwrap();
        assert!(is_secret(code, CODE_PREFIX));
        let exchange_body = |state: &str, verifier: &str| {
            json!({
                "code":code,"state":state,"code_verifier":verifier,
            })
        };
        for bad in [
            exchange_body(&URL_SAFE_NO_PAD.encode([13u8; 32]), &verifier),
            exchange_body(&browser_state, &URL_SAFE_NO_PAD.encode([14u8; 32])),
        ] {
            let denied = router
                .clone()
                .oneshot(request("POST", "/v1/routing/desktop-exchange", None, bad))
                .await
                .unwrap();
            assert_eq!(denied.status(), StatusCode::UNAUTHORIZED);
        }
        let stopped_exchange = off_router
            .clone()
            .oneshot(request(
                "POST",
                "/v1/routing/desktop-exchange",
                None,
                exchange_body(&browser_state, &verifier),
            ))
            .await
            .unwrap();
        assert_eq!(stopped_exchange.status(), StatusCode::SERVICE_UNAVAILABLE);
        let first = router
            .clone()
            .oneshot(request(
                "POST",
                "/v1/routing/desktop-exchange",
                None,
                exchange_body(&browser_state, &verifier),
            ))
            .await
            .unwrap();
        assert_eq!(first.status(), StatusCode::OK);
        let credential = json_body(first).await["token"]
            .as_str()
            .unwrap()
            .to_string();
        assert!(is_secret(&credential, SESSION_PREFIX));
        let replay = router
            .clone()
            .oneshot(request(
                "POST",
                "/v1/routing/desktop-exchange",
                None,
                exchange_body(&browser_state, &verifier),
            ))
            .await
            .unwrap();
        assert_eq!(replay.status(), StatusCode::UNAUTHORIZED);
        let account_status = router
            .clone()
            .oneshot(request(
                "GET",
                "/v1/routing/desktop-session",
                Some(&credential),
                json!({}),
            ))
            .await
            .unwrap();
        assert_eq!(account_status.status(), StatusCode::OK);
        assert_eq!(json_body(account_status).await["account_id"], account);
        let general = router
            .clone()
            .oneshot(request(
                "GET",
                "/v1/entitlements/status",
                Some(&credential),
                json!({}),
            ))
            .await
            .unwrap();
        assert_ne!(general.status(), StatusCode::OK);
        let workspace = hex::encode([42u8; 16]);
        let grant = router.clone().oneshot(request(
            "POST", "/v1/routing/workspace-grants", Some(&credential),
            json!({"workspace_id":workspace,"recipient_identity":crate::routing::HOSTED_RECIPIENT,
                "scope_digest":hex::encode(crate::routing::hosted_scope_digest()),"expected_revision":0}),
        )).await.unwrap();
        assert_eq!(grant.status(), StatusCode::OK);
        let token = router
            .clone()
            .oneshot(request(
                "POST",
                "/v1/routing/tokens",
                Some(&credential),
                json!({"workspace_id":workspace,"expected_grant_revision":1}),
            ))
            .await
            .unwrap();
        assert_eq!(token.status(), StatusCode::OK);
        let eval_token = json_body(token).await["token"]
            .as_str()
            .unwrap()
            .to_string();
        assert_eq!(
            off_router
                .clone()
                .oneshot(request(
                    "GET",
                    &format!("/v1/routing/workspace-grants/{workspace}"),
                    Some(&credential),
                    json!({}),
                ))
                .await
                .unwrap()
                .status(),
            StatusCode::OK
        );
        assert_eq!(
            off_router
                .clone()
                .oneshot(request(
                    "POST",
                    "/v1/routing/workspace-grants",
                    Some(&credential),
                    json!({"workspace_id":workspace,"recipient_identity":crate::routing::HOSTED_RECIPIENT,
                        "scope_digest":hex::encode(crate::routing::hosted_scope_digest()),"expected_revision":1}),
                ))
                .await
                .unwrap()
                .status(),
            StatusCode::SERVICE_UNAVAILABLE
        );
        assert_eq!(
            off_router
                .clone()
                .oneshot(request(
                    "POST",
                    "/v1/routing/tokens",
                    Some(&credential),
                    json!({"workspace_id":workspace,"expected_grant_revision":1}),
                ))
                .await
                .unwrap()
                .status(),
            StatusCode::SERVICE_UNAVAILABLE
        );
        let (prevalidated_account, prevalidated_expiry) =
            desktop_account(&state_for_recheck, &credential)
                .await
                .unwrap()
                .unwrap();
        let mut stale_headers = HeaderMap::new();
        stale_headers.insert(
            header::AUTHORIZATION,
            HeaderValue::from_str(&format!("Bearer {credential}")).unwrap(),
        );
        let revoked = router
            .clone()
            .oneshot(request(
                "DELETE",
                "/v1/routing/desktop-sessions",
                Some(&jwt),
                json!({}),
            ))
            .await
            .unwrap();
        assert_eq!(revoked.status(), StatusCode::NO_CONTENT);
        let mut stale_tx = db.begin().await.unwrap();
        lock_account(&mut stale_tx, &prevalidated_account)
            .await
            .unwrap();
        assert_eq!(
            crate::routing::recheck_account_after_gate(
                &stale_headers,
                &prevalidated_account,
                prevalidated_expiry,
                &mut stale_tx,
                false,
            )
            .await,
            Err(StatusCode::UNAUTHORIZED)
        );
        stale_tx.rollback().await.unwrap();
        assert_eq!(
            router
                .clone()
                .oneshot(request(
                    "GET",
                    "/v1/routing/desktop-session",
                    Some(&credential),
                    json!({}),
                ))
                .await
                .unwrap()
                .status(),
            StatusCode::UNAUTHORIZED
        );
        assert_eq!(
            router
                .clone()
                .oneshot(request(
                    "GET",
                    &format!("/v1/routing/workspace-grants/{workspace}"),
                    Some(&jwt),
                    json!({}),
                ))
                .await
                .unwrap()
                .status(),
            StatusCode::OK
        );
        let grant_state = router
            .clone()
            .oneshot(request(
                "GET",
                &format!("/v1/routing/workspace-grants/{workspace}"),
                Some(&jwt),
                json!({}),
            ))
            .await
            .unwrap();
        assert!(!json_body(grant_state).await["enabled"].as_bool().unwrap());
        let stored_revoked: bool = sqlx::query_scalar(
            "SELECT revoked_at IS NOT NULL FROM routing_session_tokens WHERE token_hash = $1",
        )
        .bind(hash(&eval_token).as_slice())
        .fetch_one(&db)
        .await
        .unwrap();
        assert!(stored_revoked);
        let next_code = json_body(
            off_router
                .clone()
                .oneshot(request(
                    "POST",
                    "/v1/routing/desktop-authorizations",
                    Some(&jwt),
                    json!({"state":browser_state,"code_challenge":challenge,"recovery_only":true}),
                ))
                .await
                .unwrap(),
        )
        .await["code"]
            .as_str()
            .unwrap()
            .to_string();
        let concurrent_body =
            || json!({"code":next_code,"state":browser_state,"code_verifier":verifier});
        let (left, right) = tokio::join!(
            router.clone().oneshot(request(
                "POST",
                "/v1/routing/desktop-exchange",
                None,
                concurrent_body()
            )),
            router.clone().oneshot(request(
                "POST",
                "/v1/routing/desktop-exchange",
                None,
                concurrent_body()
            )),
        );
        let mut results = vec![left.unwrap(), right.unwrap()];
        results.sort_by_key(|response| response.status().as_u16());
        assert_eq!(results[0].status(), StatusCode::OK);
        assert_eq!(results[1].status(), StatusCode::UNAUTHORIZED);
        let second_credential = json_body(results.swap_remove(0)).await["token"]
            .as_str()
            .unwrap()
            .to_string();
        let recovery_status = router
            .clone()
            .oneshot(request(
                "GET",
                "/v1/routing/desktop-session",
                Some(&second_credential),
                json!({}),
            ))
            .await
            .unwrap();
        assert_eq!(recovery_status.status(), StatusCode::OK);
        assert_eq!(json_body(recovery_status).await["scope"], RECOVERY_SCOPE);
        assert!(desktop_account(&state_for_recheck, &second_credential)
            .await
            .unwrap()
            .is_none());
        let read_grant = router
            .clone()
            .oneshot(request(
                "GET",
                &format!("/v1/routing/workspace-grants/{workspace}"),
                Some(&second_credential),
                json!({}),
            ))
            .await
            .unwrap();
        assert_eq!(read_grant.status(), StatusCode::OK);
        let grant_revision = json_body(read_grant).await["revision"].as_i64().unwrap();
        assert_eq!(
            router
                .clone()
                .oneshot(request(
                    "POST",
                    "/v1/routing/workspace-grants",
                    Some(&second_credential),
                    json!({"workspace_id":workspace,"recipient_identity":crate::routing::HOSTED_RECIPIENT,
                        "scope_digest":hex::encode(crate::routing::hosted_scope_digest()),"expected_revision":grant_revision}),
                ))
                .await
                .unwrap()
                .status(),
            StatusCode::UNAUTHORIZED
        );
        assert_eq!(
            router
                .clone()
                .oneshot(request(
                    "POST",
                    "/v1/routing/tokens",
                    Some(&second_credential),
                    json!({"workspace_id":workspace,"expected_grant_revision":grant_revision}),
                ))
                .await
                .unwrap()
                .status(),
            StatusCode::UNAUTHORIZED
        );
        assert_eq!(
            off_router
                .clone()
                .oneshot(request(
                    "DELETE",
                    "/v1/routing/workspace-grants",
                    Some(&second_credential),
                    json!({"workspace_id":workspace,"expected_revision":grant_revision}),
                ))
                .await
                .unwrap()
                .status(),
            StatusCode::OK
        );
        let disconnected = router
            .clone()
            .oneshot(request(
                "DELETE",
                "/v1/routing/desktop-session",
                Some(&second_credential),
                json!({}),
            ))
            .await
            .unwrap();
        assert_eq!(disconnected.status(), StatusCode::NO_CONTENT);
        assert_eq!(
            router
                .clone()
                .oneshot(request(
                    "GET",
                    "/v1/routing/desktop-session",
                    Some(&second_credential),
                    json!({}),
                ))
                .await
                .unwrap()
                .status(),
            StatusCode::UNAUTHORIZED
        );
        let expiry_code = json_body(
            router
                .clone()
                .oneshot(request(
                    "POST",
                    "/v1/routing/desktop-authorizations",
                    Some(&jwt),
                    authorize_body(),
                ))
                .await
                .unwrap(),
        )
        .await["code"]
            .as_str()
            .unwrap()
            .to_string();
        sqlx::query("UPDATE routing_desktop_codes SET expires_at = clock_timestamp() - interval '1 second' WHERE code_hash = $1")
            .bind(hash(&expiry_code).as_slice()).execute(&db).await.unwrap();
        assert_eq!(
            router
                .clone()
                .oneshot(request(
                    "POST",
                    "/v1/routing/desktop-exchange",
                    None,
                    json!({"code":expiry_code,"state":browser_state,"code_verifier":verifier}),
                ))
                .await
                .unwrap()
                .status(),
            StatusCode::UNAUTHORIZED
        );
        sqlx::query("UPDATE routing_desktop_codes SET expires_at = clock_timestamp() - interval '25 hours' WHERE code_hash = $1")
            .bind(hash(&expiry_code).as_slice()).execute(&db).await.unwrap();
        prune_expired(&db).await.unwrap();
        let retained: i64 =
            sqlx::query_scalar("SELECT count(*) FROM routing_desktop_codes WHERE code_hash = $1")
                .bind(hash(&expiry_code).as_slice())
                .fetch_one(&db)
                .await
                .unwrap();
        assert_eq!(retained, 0);
        sqlx::query("DELETE FROM routing_desktop_codes WHERE account_id = $1")
            .bind(&account)
            .execute(&db)
            .await
            .unwrap();
        sqlx::query("DELETE FROM routing_desktop_sessions WHERE account_id = $1")
            .bind(&account)
            .execute(&db)
            .await
            .unwrap();
        sqlx::query("DELETE FROM routing_session_tokens WHERE account_id = $1")
            .bind(&account)
            .execute(&db)
            .await
            .unwrap();
        sqlx::query("DELETE FROM routing_workspace_grants WHERE account_id = $1")
            .bind(&account)
            .execute(&db)
            .await
            .unwrap();
        sqlx::query("DELETE FROM routing_account_gate WHERE account_id = $1")
            .bind(&account)
            .execute(&db)
            .await
            .unwrap();
    }
}
