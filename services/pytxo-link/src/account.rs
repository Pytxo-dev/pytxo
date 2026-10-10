//! Optional Pytxo account: sign-in for the CLI and Desktop, shared receipts
//! and the public Pybench board. Local Pytxo never needs any of this.
//!
//! Sign-in is a browser handoff with PKCE: the signed-in website asks for a
//! one-use code bound to the client's `state` and `code_challenge`; the client
//! exchanges it with its `code_verifier` for an account credential (`pas1_…`,
//! 30 days). Only hashes of codes and credentials are stored. The credential
//! carries no entitlement, billing or Routing authority.

use std::collections::HashMap;
use std::sync::{Arc, Mutex};

use axum::extract::{Path, State};
use axum::http::{header, HeaderMap, HeaderValue, StatusCode};
use axum::Json;
use base64::engine::general_purpose::URL_SAFE_NO_PAD;
use base64::Engine;
use chrono::{DateTime, Duration, Utc};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use sqlx::PgPool;

use crate::state::AppState;

const CODE_PREFIX: &str = "pac1_";
const SESSION_PREFIX: &str = "pas1_";
const CODE_SECONDS: i64 = 120;
const SESSION_DAYS: i64 = 30;
const CODES_PER_MINUTE: usize = 5;
const SESSIONS_PER_ACCOUNT: usize = 10;
const MAX_PATHS: usize = 500;
const PUBLIC_ID_LEN: usize = 12;

// ------------------------------------------------------------------ storage

#[derive(Clone)]
pub enum AccountStore {
    Postgres(PgPool),
    /// Local development and tests only (never with LINK_REQUIRE_AUTH).
    Memory(Arc<Mutex<MemoryAccounts>>),
    Unavailable,
}

#[derive(Default)]
pub struct MemoryAccounts {
    codes: HashMap<[u8; 32], CodeRow>,
    sessions: HashMap<[u8; 32], SessionRow>,
    receipts: Vec<StoredItem>,
    bench: Vec<StoredItem>,
}

#[derive(Clone)]
struct CodeRow {
    account: String,
    state: [u8; 32],
    challenge: [u8; 32],
    created: DateTime<Utc>,
    expires: DateTime<Utc>,
    consumed: bool,
}

#[derive(Clone)]
struct SessionRow {
    account: String,
    /// Kept for parity with Postgres; not read in memory mode.
    _label: Option<String>,
    created: DateTime<Utc>,
    expires: DateTime<Utc>,
    revoked: bool,
}

#[derive(Clone)]
struct StoredItem {
    id: String,
    account: String,
    digest: String,
    payload: serde_json::Value,
    created: DateTime<Utc>,
    deleted: bool,
}

impl AccountStore {
    pub fn from_state_parts(db: Option<&PgPool>, require_auth: bool) -> Self {
        match (db, require_auth) {
            (Some(pool), _) => Self::Postgres(pool.clone()),
            (None, false) => Self::Memory(Arc::default()),
            (None, true) => Self::Unavailable,
        }
    }
}

// --------------------------------------------------------------- utilities

fn no_store() -> HeaderMap {
    let mut h = HeaderMap::new();
    h.insert(header::CACHE_CONTROL, HeaderValue::from_static("no-store"));
    h
}

fn bearer(headers: &HeaderMap) -> Option<&str> {
    headers
        .get(header::AUTHORIZATION)?
        .to_str()
        .ok()?
        .strip_prefix("Bearer ")
        .map(str::trim)
        .filter(|t| !t.is_empty() && t.len() <= 8192)
}

fn parse_random_bytes(value: &str) -> Option<[u8; 32]> {
    if value.len() != 43 {
        return None;
    }
    let bytes: [u8; 32] = URL_SAFE_NO_PAD.decode(value).ok()?.try_into().ok()?;
    (URL_SAFE_NO_PAD.encode(bytes) == value).then_some(bytes)
}

fn random_bytes() -> Result<[u8; 32], StatusCode> {
    let mut b = [0u8; 32];
    getrandom::getrandom(&mut b).map_err(|_| StatusCode::SERVICE_UNAVAILABLE)?;
    Ok(b)
}

fn secret(prefix: &str) -> Result<String, StatusCode> {
    Ok(format!(
        "{prefix}{}",
        URL_SAFE_NO_PAD.encode(random_bytes()?)
    ))
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

/// Short public id from random bytes: unguessable, URL-safe, no account info.
fn public_id(prefix: &str) -> Result<String, StatusCode> {
    const ALPHABET: &[u8] = b"0123456789abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ";
    let bytes = random_bytes()?;
    let tail: String = bytes
        .iter()
        .take(PUBLIC_ID_LEN)
        .map(|b| ALPHABET[(*b as usize) % ALPHABET.len()] as char)
        .collect();
    Ok(format!("{prefix}{tail}"))
}

/// The signed-in website user (Clerk session). Development Links without
/// JWKS accept `x-pytxo-dev-account` instead, never when auth is required.
async fn website_account(state: &AppState, headers: &HeaderMap) -> Result<String, StatusCode> {
    if let Some(jwks) = state.jwks.as_ref() {
        let token = bearer(headers).ok_or(StatusCode::UNAUTHORIZED)?;
        let claims = jwks.validate(token).await.ok_or(StatusCode::UNAUTHORIZED)?;
        let session_bound = claims
            .sid
            .as_deref()
            .is_some_and(|s| s.starts_with("sess_"));
        if !claims.sub.starts_with("user_") || !session_bound {
            return Err(StatusCode::UNAUTHORIZED);
        }
        return Ok(claims.sub);
    }
    if state.require_auth {
        return Err(StatusCode::SERVICE_UNAVAILABLE);
    }
    headers
        .get("x-pytxo-dev-account")
        .and_then(|v| v.to_str().ok())
        .filter(|v| v.starts_with("user_") && v.len() <= 64)
        .map(str::to_string)
        .ok_or(StatusCode::UNAUTHORIZED)
}

/// Account behind a `pas1_` credential (CLI/Desktop) or a website session.
async fn any_account(state: &AppState, headers: &HeaderMap) -> Result<String, StatusCode> {
    if let Some(token) = bearer(headers).filter(|t| t.starts_with(SESSION_PREFIX)) {
        return session_account(state, token)
            .await?
            .ok_or(StatusCode::UNAUTHORIZED);
    }
    website_account(state, headers).await
}

async fn session_account(state: &AppState, token: &str) -> Result<Option<String>, StatusCode> {
    if !is_secret(token, SESSION_PREFIX) {
        return Ok(None);
    }
    let h = hash(token);
    match &state.accounts {
        AccountStore::Unavailable => Err(StatusCode::SERVICE_UNAVAILABLE),
        AccountStore::Memory(m) => {
            let m = m.lock().map_err(|_| StatusCode::SERVICE_UNAVAILABLE)?;
            Ok(m.sessions
                .get(&h)
                .filter(|s| !s.revoked && s.expires > Utc::now())
                .map(|s| s.account.clone()))
        }
        AccountStore::Postgres(db) => sqlx::query_scalar(
            "UPDATE account_sessions SET last_used_at = clock_timestamp() \
             WHERE token_hash = $1 AND revoked_at IS NULL AND expires_at > clock_timestamp() \
             RETURNING account_id",
        )
        .bind(h.as_slice())
        .fetch_optional(db)
        .await
        .map_err(|_| StatusCode::SERVICE_UNAVAILABLE),
    }
}

// ------------------------------------------------------------------ sign-in

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub struct AuthorizeRequest {
    state: String,
    code_challenge: String,
}

#[derive(Serialize)]
pub struct AuthorizeResponse {
    code: String,
    expires_at: DateTime<Utc>,
}

pub async fn authorize(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(body): Json<AuthorizeRequest>,
) -> Result<(HeaderMap, Json<AuthorizeResponse>), StatusCode> {
    let account = website_account(&state, &headers).await?;
    let state_bytes = parse_random_bytes(&body.state).ok_or(StatusCode::BAD_REQUEST)?;
    let challenge = parse_random_bytes(&body.code_challenge).ok_or(StatusCode::BAD_REQUEST)?;
    let code = secret(CODE_PREFIX)?;
    let now = Utc::now();
    let expires_at = now + Duration::seconds(CODE_SECONDS);
    match &state.accounts {
        AccountStore::Unavailable => return Err(StatusCode::SERVICE_UNAVAILABLE),
        AccountStore::Memory(m) => {
            let mut m = m.lock().map_err(|_| StatusCode::SERVICE_UNAVAILABLE)?;
            let recent = m
                .codes
                .values()
                .filter(|c| c.account == account && c.created > now - Duration::minutes(1))
                .count();
            if recent >= CODES_PER_MINUTE {
                return Err(StatusCode::TOO_MANY_REQUESTS);
            }
            m.codes.insert(
                hash(&code),
                CodeRow {
                    account,
                    state: hash(&body.state),
                    challenge,
                    created: now,
                    expires: expires_at,
                    consumed: false,
                },
            );
            let _ = state_bytes;
        }
        AccountStore::Postgres(db) => {
            let recent: i64 = sqlx::query_scalar(
                "SELECT count(*) FROM account_codes WHERE account_id = $1 AND created_at > clock_timestamp() - interval '1 minute'",
            )
            .bind(&account)
            .fetch_one(db)
            .await
            .map_err(|_| StatusCode::SERVICE_UNAVAILABLE)?;
            if recent >= CODES_PER_MINUTE as i64 {
                return Err(StatusCode::TOO_MANY_REQUESTS);
            }
            sqlx::query(
                "INSERT INTO account_codes (code_hash, account_id, state_hash, challenge, expires_at) VALUES ($1, $2, $3, $4, $5)",
            )
            .bind(hash(&code).as_slice())
            .bind(&account)
            .bind(hash(&body.state).as_slice())
            .bind(challenge.as_slice())
            .bind(expires_at)
            .execute(db)
            .await
            .map_err(|_| StatusCode::SERVICE_UNAVAILABLE)?;
        }
    }
    Ok((no_store(), Json(AuthorizeResponse { code, expires_at })))
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ExchangeRequest {
    code: String,
    state: String,
    code_verifier: String,
    /// e.g. "cli on host" or "desktop"; shown in the account page.
    #[serde(default)]
    label: Option<String>,
}

#[derive(Serialize)]
pub struct ExchangeResponse {
    token: String,
    account_id: String,
    expires_at: DateTime<Utc>,
}

fn clean_label(label: Option<String>) -> Option<String> {
    label
        .map(|l| {
            l.chars()
                .filter(|c| !c.is_control())
                .take(48)
                .collect::<String>()
        })
        .filter(|l| !l.trim().is_empty())
}

pub async fn exchange(
    State(state): State<AppState>,
    Json(body): Json<ExchangeRequest>,
) -> Result<(HeaderMap, Json<ExchangeResponse>), StatusCode> {
    if !is_secret(&body.code, CODE_PREFIX)
        || parse_random_bytes(&body.state).is_none()
        || parse_random_bytes(&body.code_verifier).is_none()
    {
        return Err(StatusCode::BAD_REQUEST);
    }
    let code_hash = hash(&body.code);
    let state_hash = hash(&body.state);
    // PKCE S256 over the verifier's text, as in the Desktop routing bridge.
    let challenge = hash(&body.code_verifier);
    let token = secret(SESSION_PREFIX)?;
    let label = clean_label(body.label);
    let now = Utc::now();
    let expires_at = now + Duration::days(SESSION_DAYS);
    let account = match &state.accounts {
        AccountStore::Unavailable => return Err(StatusCode::SERVICE_UNAVAILABLE),
        AccountStore::Memory(m) => {
            let mut m = m.lock().map_err(|_| StatusCode::SERVICE_UNAVAILABLE)?;
            let row = m
                .codes
                .get_mut(&code_hash)
                .ok_or(StatusCode::UNAUTHORIZED)?;
            if row.consumed
                || row.expires <= now
                || row.state != state_hash
                || row.challenge != challenge
            {
                return Err(StatusCode::UNAUTHORIZED);
            }
            row.consumed = true;
            let account = row.account.clone();
            let mut active: Vec<([u8; 32], DateTime<Utc>)> = m
                .sessions
                .iter()
                .filter(|(_, s)| s.account == account && !s.revoked && s.expires > now)
                .map(|(k, s)| (*k, s.created))
                .collect();
            active.sort_by_key(|(_, created)| *created);
            while active.len() >= SESSIONS_PER_ACCOUNT {
                let (oldest, _) = active.remove(0);
                if let Some(s) = m.sessions.get_mut(&oldest) {
                    s.revoked = true;
                }
            }
            m.sessions.insert(
                hash(&token),
                SessionRow {
                    account: account.clone(),
                    _label: label,
                    created: now,
                    expires: expires_at,
                    revoked: false,
                },
            );
            account
        }
        AccountStore::Postgres(db) => {
            let mut tx = db
                .begin()
                .await
                .map_err(|_| StatusCode::SERVICE_UNAVAILABLE)?;
            let row: Option<(String, Vec<u8>, Vec<u8>, DateTime<Utc>, Option<DateTime<Utc>>)> = sqlx::query_as(
                "SELECT account_id, state_hash, challenge, expires_at, consumed_at FROM account_codes WHERE code_hash = $1 FOR UPDATE",
            )
            .bind(code_hash.as_slice())
            .fetch_optional(&mut *tx)
            .await
            .map_err(|_| StatusCode::SERVICE_UNAVAILABLE)?;
            let (account, s_hash, chal, expires, consumed) = row.ok_or(StatusCode::UNAUTHORIZED)?;
            if consumed.is_some() || expires <= now || s_hash != state_hash || chal != challenge {
                return Err(StatusCode::UNAUTHORIZED);
            }
            sqlx::query(
                "UPDATE account_codes SET consumed_at = clock_timestamp() WHERE code_hash = $1",
            )
            .bind(code_hash.as_slice())
            .execute(&mut *tx)
            .await
            .map_err(|_| StatusCode::SERVICE_UNAVAILABLE)?;
            // Keep the newest SESSIONS_PER_ACCOUNT - 1 active credentials.
            sqlx::query(
                "UPDATE account_sessions SET revoked_at = clock_timestamp() WHERE token_hash IN (\
                   SELECT token_hash FROM account_sessions \
                   WHERE account_id = $1 AND revoked_at IS NULL AND expires_at > clock_timestamp() \
                   ORDER BY created_at DESC OFFSET $2)",
            )
            .bind(&account)
            .bind((SESSIONS_PER_ACCOUNT - 1) as i64)
            .execute(&mut *tx)
            .await
            .map_err(|_| StatusCode::SERVICE_UNAVAILABLE)?;
            sqlx::query(
                "INSERT INTO account_sessions (token_hash, account_id, label, expires_at) VALUES ($1, $2, $3, $4)",
            )
            .bind(hash(&token).as_slice())
            .bind(&account)
            .bind(&label)
            .bind(expires_at)
            .execute(&mut *tx)
            .await
            .map_err(|_| StatusCode::SERVICE_UNAVAILABLE)?;
            tx.commit()
                .await
                .map_err(|_| StatusCode::SERVICE_UNAVAILABLE)?;
            account
        }
    };
    Ok((
        no_store(),
        Json(ExchangeResponse {
            token,
            account_id: account,
            expires_at,
        }),
    ))
}

#[derive(Serialize)]
pub struct SessionResponse {
    account_id: String,
    perks: [&'static str; 3],
}

pub async fn session(
    State(state): State<AppState>,
    headers: HeaderMap,
) -> Result<(HeaderMap, Json<SessionResponse>), StatusCode> {
    let token = bearer(&headers).ok_or(StatusCode::UNAUTHORIZED)?;
    let account = session_account(&state, token)
        .await?
        .ok_or(StatusCode::UNAUTHORIZED)?;
    Ok((
        no_store(),
        Json(SessionResponse {
            account_id: account,
            perks: ["share_receipts", "receipt_history", "bench_board"],
        }),
    ))
}

pub async fn sign_out(State(state): State<AppState>, headers: HeaderMap) -> StatusCode {
    let Some(token) = bearer(&headers).filter(|t| is_secret(t, SESSION_PREFIX)) else {
        return StatusCode::UNAUTHORIZED;
    };
    let h = hash(token);
    match &state.accounts {
        AccountStore::Unavailable => StatusCode::SERVICE_UNAVAILABLE,
        AccountStore::Memory(m) => match m.lock() {
            Ok(mut m) => {
                if let Some(s) = m.sessions.get_mut(&h) {
                    s.revoked = true;
                }
                StatusCode::NO_CONTENT
            }
            Err(_) => StatusCode::SERVICE_UNAVAILABLE,
        },
        AccountStore::Postgres(db) => {
            match sqlx::query("UPDATE account_sessions SET revoked_at = clock_timestamp() WHERE token_hash = $1 AND revoked_at IS NULL")
                .bind(h.as_slice())
                .execute(db)
                .await
            {
                Ok(_) => StatusCode::NO_CONTENT,
                Err(_) => StatusCode::SERVICE_UNAVAILABLE,
            }
        }
    }
}

// ----------------------------------------------------------------- payloads

/// Mirrors `Receipt::share_payload` in pytxo-core. Unknown fields are
/// rejected so the public page can never host arbitrary content.
#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct SharedReceipt {
    schema: String,
    id: String,
    kind: String,
    decision: String,
    mode: String,
    created_at: String,
    pytxo_version: String,
    inputs: u32,
    agents: Vec<String>,
    files_changed: u32,
    shared_files: u32,
    symbol_collisions: u32,
    conflicts: u32,
    checks: SharedChecks,
    duration_ms: u64,
    content_digest: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    paths: Option<Vec<String>>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct SharedChecks {
    combined: PassTotal,
    per_input: PassTotal,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct PassTotal {
    passed: u32,
    total: u32,
}

fn word(value: &str, max: usize) -> bool {
    !value.is_empty()
        && value.len() <= max
        && value
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || "._-:+".contains(c))
}

impl SharedReceipt {
    fn validate(&self) -> bool {
        let one_of = |v: &str, set: &[&str]| set.contains(&v);
        self.schema == "pytxo.receipt.share/v1"
            && word(&self.id, 64)
            && one_of(&self.kind, &["land", "apply"])
            && one_of(
                &self.decision,
                &["ready", "landed", "blocked", "stale", "applied", "refused"],
            )
            && one_of(&self.mode, &["shadow", "apply"])
            && DateTime::parse_from_rfc3339(&self.created_at).is_ok()
            && word(&self.pytxo_version, 32)
            && self.agents.len() <= 32
            && self.agents.iter().all(|a| word(a, 32))
            && self.inputs <= 1000
            && self.checks.combined.passed <= self.checks.combined.total
            && self.checks.per_input.passed <= self.checks.per_input.total
            && self.content_digest.starts_with("sha256:")
            && self.content_digest.len() == 71
            && self.content_digest[7..]
                .chars()
                .all(|c| c.is_ascii_hexdigit())
            && self.paths.as_ref().is_none_or(|p| {
                p.len() <= MAX_PATHS
                    && p.iter().all(|path| {
                        !path.is_empty() && path.len() <= 300 && !path.chars().any(char::is_control)
                    })
            })
    }
}

/// The public board keeps the summary numbers and machine identity only.
#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct SharedBench {
    schema: String,
    pytxo_version: String,
    created_at: String,
    os: String,
    arch: String,
    cpus: u32,
    scenarios: u32,
    correct: u32,
    naive_landed_broken: u32,
    pytxo_prevented: u32,
    false_refusals: u32,
    median_gate_ms: u64,
    #[serde(default)]
    signal_reduction_pct: Option<f64>,
}

impl SharedBench {
    fn validate(&self) -> bool {
        self.schema == "pybench.share/v1"
            && word(&self.pytxo_version, 32)
            && DateTime::parse_from_rfc3339(&self.created_at).is_ok()
            && word(&self.os, 16)
            && word(&self.arch, 16)
            && (1..=1024).contains(&self.cpus)
            && self.correct <= self.scenarios
            && self.scenarios <= 1000
            && self.pytxo_prevented <= self.naive_landed_broken
            && self
                .signal_reduction_pct
                .is_none_or(|p| (0.0..=100.0).contains(&p))
    }
}

// ----------------------------------------------------------------- receipts

#[derive(Serialize)]
pub struct Published {
    id: String,
    url: String,
}

async fn publish(
    state: &AppState,
    account: String,
    table: &'static str,
    prefix: &str,
    digest: String,
    payload: serde_json::Value,
) -> Result<String, StatusCode> {
    match &state.accounts {
        AccountStore::Unavailable => Err(StatusCode::SERVICE_UNAVAILABLE),
        AccountStore::Memory(m) => {
            let mut m = m.lock().map_err(|_| StatusCode::SERVICE_UNAVAILABLE)?;
            let items = if table == "shared_receipts" {
                &mut m.receipts
            } else {
                &mut m.bench
            };
            if let Some(existing) = items
                .iter()
                .find(|i| i.account == account && i.digest == digest && !i.deleted)
            {
                return Ok(existing.id.clone());
            }
            let id = public_id(prefix)?;
            items.push(StoredItem {
                id: id.clone(),
                account,
                digest,
                payload,
                created: Utc::now(),
                deleted: false,
            });
            Ok(id)
        }
        AccountStore::Postgres(db) => {
            let id = public_id(prefix)?;
            let sql = format!(
                "INSERT INTO {table} (id, account_id, content_digest, payload) VALUES ($1, $2, $3, $4) \
                 ON CONFLICT (account_id, content_digest) DO UPDATE SET deleted_at = NULL \
                 RETURNING id"
            );
            sqlx::query_scalar(&sql)
                .bind(&id)
                .bind(&account)
                .bind(&digest)
                .bind(&payload)
                .fetch_one(db)
                .await
                .map_err(|_| StatusCode::SERVICE_UNAVAILABLE)
        }
    }
}

pub async fn share_receipt(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(receipt): Json<SharedReceipt>,
) -> Result<(HeaderMap, Json<Published>), StatusCode> {
    let account = any_account(&state, &headers).await?;
    if !receipt.validate() {
        return Err(StatusCode::UNPROCESSABLE_ENTITY);
    }
    let payload = serde_json::to_value(&receipt).map_err(|_| StatusCode::UNPROCESSABLE_ENTITY)?;
    let id = publish(
        &state,
        account,
        "shared_receipts",
        "r_",
        receipt.content_digest.clone(),
        payload,
    )
    .await?;
    Ok((
        no_store(),
        Json(Published {
            url: format!("https://pytxo.com/r/{id}"),
            id,
        }),
    ))
}

#[derive(Serialize)]
pub struct PublicItem {
    id: String,
    created_at: DateTime<Utc>,
    payload: serde_json::Value,
}

pub async fn public_receipt(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> Result<Json<PublicItem>, StatusCode> {
    if !id.starts_with("r_")
        || id.len() != 2 + PUBLIC_ID_LEN
        || !id[2..].chars().all(|c| c.is_ascii_alphanumeric())
    {
        return Err(StatusCode::NOT_FOUND);
    }
    let found = match &state.accounts {
        AccountStore::Unavailable => return Err(StatusCode::SERVICE_UNAVAILABLE),
        AccountStore::Memory(m) => {
            let m = m.lock().map_err(|_| StatusCode::SERVICE_UNAVAILABLE)?;
            m.receipts
                .iter()
                .find(|r| r.id == id && !r.deleted)
                .map(|r| PublicItem { id: r.id.clone(), created_at: r.created, payload: r.payload.clone() })
        }
        AccountStore::Postgres(db) => sqlx::query_as::<_, (String, DateTime<Utc>, serde_json::Value)>(
            "SELECT id, created_at, payload FROM shared_receipts WHERE id = $1 AND deleted_at IS NULL",
        )
        .bind(&id)
        .fetch_optional(db)
        .await
        .map_err(|_| StatusCode::SERVICE_UNAVAILABLE)?
        .map(|(id, created_at, payload)| PublicItem { id, created_at, payload }),
    };
    found.map(Json).ok_or(StatusCode::NOT_FOUND)
}

pub async fn my_receipts(
    State(state): State<AppState>,
    headers: HeaderMap,
) -> Result<(HeaderMap, Json<Vec<PublicItem>>), StatusCode> {
    let account = any_account(&state, &headers).await?;
    let items = match &state.accounts {
        AccountStore::Unavailable => return Err(StatusCode::SERVICE_UNAVAILABLE),
        AccountStore::Memory(m) => {
            let m = m.lock().map_err(|_| StatusCode::SERVICE_UNAVAILABLE)?;
            let mut v: Vec<_> = m
                .receipts
                .iter()
                .filter(|r| r.account == account && !r.deleted)
                .map(|r| PublicItem { id: r.id.clone(), created_at: r.created, payload: r.payload.clone() })
                .collect();
            v.sort_by(|a, b| b.created_at.cmp(&a.created_at));
            v.truncate(200);
            v
        }
        AccountStore::Postgres(db) => sqlx::query_as::<_, (String, DateTime<Utc>, serde_json::Value)>(
            "SELECT id, created_at, payload FROM shared_receipts WHERE account_id = $1 AND deleted_at IS NULL ORDER BY created_at DESC LIMIT 200",
        )
        .bind(&account)
        .fetch_all(db)
        .await
        .map_err(|_| StatusCode::SERVICE_UNAVAILABLE)?
        .into_iter()
        .map(|(id, created_at, payload)| PublicItem { id, created_at, payload })
        .collect(),
    };
    Ok((no_store(), Json(items)))
}

pub async fn delete_receipt(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(id): Path<String>,
) -> StatusCode {
    let account = match any_account(&state, &headers).await {
        Ok(a) => a,
        Err(code) => return code,
    };
    match &state.accounts {
        AccountStore::Unavailable => StatusCode::SERVICE_UNAVAILABLE,
        AccountStore::Memory(m) => match m.lock() {
            Ok(mut m) => match m.receipts.iter_mut().find(|r| r.id == id && r.account == account && !r.deleted) {
                Some(r) => {
                    r.deleted = true;
                    StatusCode::NO_CONTENT
                }
                None => StatusCode::NOT_FOUND,
            },
            Err(_) => StatusCode::SERVICE_UNAVAILABLE,
        },
        AccountStore::Postgres(db) => match sqlx::query(
            "UPDATE shared_receipts SET deleted_at = clock_timestamp() WHERE id = $1 AND account_id = $2 AND deleted_at IS NULL",
        )
        .bind(&id)
        .bind(&account)
        .execute(db)
        .await
        {
            Ok(r) if r.rows_affected() == 1 => StatusCode::NO_CONTENT,
            Ok(_) => StatusCode::NOT_FOUND,
            Err(_) => StatusCode::SERVICE_UNAVAILABLE,
        },
    }
}

// -------------------------------------------------------------------- bench

pub async fn submit_bench(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(bench): Json<SharedBench>,
) -> Result<(HeaderMap, Json<Published>), StatusCode> {
    let account = any_account(&state, &headers).await?;
    if !bench.validate() {
        return Err(StatusCode::UNPROCESSABLE_ENTITY);
    }
    let payload = serde_json::to_value(&bench).map_err(|_| StatusCode::UNPROCESSABLE_ENTITY)?;
    let digest = format!("{:x}", Sha256::digest(payload.to_string().as_bytes()));
    let id = publish(&state, account, "bench_results", "b_", digest, payload).await?;
    Ok((
        no_store(),
        Json(Published {
            url: "https://pytxo.com/bench".into(),
            id,
        }),
    ))
}

pub async fn bench_board(
    State(state): State<AppState>,
) -> Result<Json<Vec<PublicItem>>, StatusCode> {
    let items = match &state.accounts {
        AccountStore::Unavailable => return Err(StatusCode::SERVICE_UNAVAILABLE),
        AccountStore::Memory(m) => {
            let m = m.lock().map_err(|_| StatusCode::SERVICE_UNAVAILABLE)?;
            let mut v: Vec<_> = m
                .bench
                .iter()
                .filter(|b| !b.deleted)
                .map(|b| PublicItem { id: b.id.clone(), created_at: b.created, payload: b.payload.clone() })
                .collect();
            v.sort_by(|a, b| b.created_at.cmp(&a.created_at));
            v.truncate(50);
            v
        }
        AccountStore::Postgres(db) => sqlx::query_as::<_, (String, DateTime<Utc>, serde_json::Value)>(
            "SELECT id, created_at, payload FROM bench_results WHERE deleted_at IS NULL ORDER BY created_at DESC LIMIT 50",
        )
        .fetch_all(db)
        .await
        .map_err(|_| StatusCode::SERVICE_UNAVAILABLE)?
        .into_iter()
        .map(|(id, created_at, payload)| PublicItem { id, created_at, payload })
        .collect(),
    };
    Ok(Json(items))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn secrets_and_public_ids_have_fixed_shapes() {
        let s = secret(SESSION_PREFIX).unwrap();
        assert!(is_secret(&s, SESSION_PREFIX));
        assert!(!is_secret(&s, CODE_PREFIX));
        let id = public_id("r_").unwrap();
        assert_eq!(id.len(), 2 + PUBLIC_ID_LEN);
    }

    #[test]
    fn labels_are_trimmed_and_control_free() {
        assert_eq!(
            clean_label(Some("cli\non host".into())).as_deref(),
            Some("clion host")
        );
        assert_eq!(clean_label(Some("   ".into())), None);
    }

    use axum::body::Body;
    use axum::http::Request;
    use http_body_util::BodyExt;
    use tower::ServiceExt;

    async fn call(
        app: &axum::Router,
        method: &str,
        uri: &str,
        auth: Option<(&str, &str)>,
        body: Option<serde_json::Value>,
    ) -> (StatusCode, serde_json::Value) {
        let mut req = Request::builder().method(method).uri(uri);
        if let Some((name, value)) = auth {
            req = req.header(name, value);
        }
        let req = match body {
            Some(b) => req
                .header("content-type", "application/json")
                .body(Body::from(b.to_string())),
            None => req.body(Body::empty()),
        }
        .unwrap();
        let res = app.clone().oneshot(req).await.unwrap();
        let status = res.status();
        let bytes = res.into_body().collect().await.unwrap().to_bytes();
        (
            status,
            serde_json::from_slice(&bytes).unwrap_or(serde_json::Value::Null),
        )
    }

    fn receipt_payload(digest_char: char) -> serde_json::Value {
        serde_json::json!({
            "schema": "pytxo.receipt.share/v1",
            "id": "rcpt_20261010T000000Z_deadbeef",
            "kind": "land", "decision": "blocked", "mode": "shadow",
            "created_at": "2026-10-10T00:00:00Z", "pytxo_version": "1.3.0",
            "inputs": 2, "agents": ["codex", "claude"], "files_changed": 2,
            "shared_files": 0, "symbol_collisions": 0, "conflicts": 0,
            "checks": { "combined": { "passed": 0, "total": 1 }, "per_input": { "passed": 2, "total": 2 } },
            "duration_ms": 4000,
            "content_digest": format!("sha256:{}", digest_char.to_string().repeat(64)),
        })
    }

    #[tokio::test]
    async fn sign_in_share_list_delete_and_board() {
        let app = crate::build_router(crate::contract_tests::test_state());
        let verifier = URL_SAFE_NO_PAD.encode([7u8; 32]);
        let challenge = URL_SAFE_NO_PAD.encode(Sha256::digest(verifier.as_bytes()));
        let state = URL_SAFE_NO_PAD.encode([9u8; 32]);
        let start = serde_json::json!({ "state": state, "code_challenge": challenge });

        // The website must be signed in to mint a code.
        let (status, _) = call(
            &app,
            "POST",
            "/v1/account/authorize",
            None,
            Some(start.clone()),
        )
        .await;
        assert_eq!(status, StatusCode::UNAUTHORIZED);
        let user = Some(("x-pytxo-dev-account", "user_alice"));
        let (status, body) = call(&app, "POST", "/v1/account/authorize", user, Some(start)).await;
        assert_eq!(status, StatusCode::OK);
        let code = body["code"].as_str().unwrap().to_string();

        // A wrong verifier fails without burning the code; the right one works once.
        let wrong = URL_SAFE_NO_PAD.encode([8u8; 32]);
        let bad_exchange =
            serde_json::json!({ "code": code, "state": state, "code_verifier": wrong });
        let (status, _) = call(
            &app,
            "POST",
            "/v1/account/exchange",
            None,
            Some(bad_exchange),
        )
        .await;
        assert_eq!(status, StatusCode::UNAUTHORIZED);
        let exchange = serde_json::json!({ "code": code, "state": state, "code_verifier": verifier, "label": "cli" });
        let (status, body) = call(
            &app,
            "POST",
            "/v1/account/exchange",
            None,
            Some(exchange.clone()),
        )
        .await;
        assert_eq!(status, StatusCode::OK);
        assert_eq!(body["account_id"], "user_alice");
        let token = format!("Bearer {}", body["token"].as_str().unwrap());
        let (status, _) = call(&app, "POST", "/v1/account/exchange", None, Some(exchange)).await;
        assert_eq!(status, StatusCode::UNAUTHORIZED, "codes are single use");

        let auth = Some(("authorization", token.as_str()));
        let (status, body) = call(&app, "GET", "/v1/account/session", auth, None).await;
        assert_eq!(status, StatusCode::OK);
        assert_eq!(body["account_id"], "user_alice");

        // Sharing is idempotent per digest; the public read never exposes the account.
        let (status, first) = call(
            &app,
            "POST",
            "/v1/receipts",
            auth,
            Some(receipt_payload('a')),
        )
        .await;
        assert_eq!(status, StatusCode::OK);
        let (_, again) = call(
            &app,
            "POST",
            "/v1/receipts",
            auth,
            Some(receipt_payload('a')),
        )
        .await;
        assert_eq!(first["id"], again["id"]);
        let id = first["id"].as_str().unwrap().to_string();
        assert!(first["url"].as_str().unwrap().ends_with(&id));
        let (status, public) = call(&app, "GET", &format!("/v1/receipts/{id}"), None, None).await;
        assert_eq!(status, StatusCode::OK);
        assert!(!public.to_string().contains("alice"));
        assert_eq!(public["payload"]["decision"], "blocked");

        // Extra fields and unsafe values are refused.
        let mut extra = receipt_payload('b');
        extra["html"] = serde_json::json!("<script>");
        let (status, _) = call(&app, "POST", "/v1/receipts", auth, Some(extra)).await;
        assert_eq!(status, StatusCode::UNPROCESSABLE_ENTITY);
        let mut bad = receipt_payload('c');
        bad["agents"] = serde_json::json!(["<b>codex</b>"]);
        let (status, _) = call(&app, "POST", "/v1/receipts", auth, Some(bad)).await;
        assert_eq!(status, StatusCode::UNPROCESSABLE_ENTITY);

        let (_, mine) = call(&app, "GET", "/v1/receipts", auth, None).await;
        assert_eq!(mine.as_array().unwrap().len(), 1);
        let other = Some(("x-pytxo-dev-account", "user_bob"));
        let (status, _) = call(&app, "DELETE", &format!("/v1/receipts/{id}"), other, None).await;
        assert_eq!(status, StatusCode::NOT_FOUND, "only the owner deletes");
        let (status, _) = call(&app, "DELETE", &format!("/v1/receipts/{id}"), auth, None).await;
        assert_eq!(status, StatusCode::NO_CONTENT);
        let (status, _) = call(&app, "GET", &format!("/v1/receipts/{id}"), None, None).await;
        assert_eq!(status, StatusCode::NOT_FOUND);

        let bench = serde_json::json!({
            "schema": "pybench.share/v1", "pytxo_version": "1.3.0", "created_at": "2026-10-10T00:00:00Z",
            "os": "windows", "arch": "x86_64", "cpus": 16, "scenarios": 12, "correct": 12,
            "naive_landed_broken": 4, "pytxo_prevented": 4, "false_refusals": 0, "median_gate_ms": 3000,
            "signal_reduction_pct": 82.9
        });
        let (status, _) = call(&app, "POST", "/v1/bench", auth, Some(bench)).await;
        assert_eq!(status, StatusCode::OK);
        let (_, board) = call(&app, "GET", "/v1/bench", None, None).await;
        assert_eq!(board[0]["payload"]["pytxo_prevented"], 4);
        assert!(!board.to_string().contains("alice"));

        // Signing out revokes the credential.
        let (status, _) = call(&app, "DELETE", "/v1/account/session", auth, None).await;
        assert_eq!(status, StatusCode::NO_CONTENT);
        let (status, _) = call(&app, "GET", "/v1/account/session", auth, None).await;
        assert_eq!(status, StatusCode::UNAUTHORIZED);
    }
}
