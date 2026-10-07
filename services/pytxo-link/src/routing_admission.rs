//! Experimental sponsored-routing admission ledger. This module never calls
//! a model. Funding starts disabled in the migration and all failures close
//! the hosted path while the local rules router remains available.

use axum::extract::State;
use axum::http::{header, HeaderMap, StatusCode};
use axum::Json;
use base64::engine::general_purpose::URL_SAFE_NO_PAD;
use base64::Engine;
use chrono::{DateTime, Duration, NaiveDate, Utc};
use hmac::{Hmac, Mac};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use sqlx::{PgPool, Postgres, Transaction};

use crate::routing::{hosted_scope_digest, HOSTED_RECIPIENT};
use crate::state::AppState;

type HmacSha256 = Hmac<Sha256>;
const TOKEN_SCOPE: &str = "routing:evaluate:v1";
const MONTHLY_INPUT_LIMIT: i64 = 3_000_000;
const REQUESTS_PER_MINUTE: i32 = 10;
const MAX_ACTIVE: i32 = 2;

fn pinned_rate(value: Option<&str>) -> Option<i64> {
    value?.parse::<i64>().ok().filter(|rate| *rate >= 0)
}

#[derive(Clone)]
pub struct RoutingAdmissionConfig {
    service_key_digest: [u8; 32],
    payload_mac_key: [u8; 32],
    pub verified_input_ceiling: i64,
    pub verified_output_ceiling: i64,
    pub verified_total_nano_usd_ceiling: i64,
    pub rate_card_revision: String,
    pub input_nano_usd_per_token: i64,
    pub output_nano_usd_per_token: i64,
}

impl RoutingAdmissionConfig {
    pub fn from_env(enabled: bool) -> Result<Option<Self>, &'static str> {
        if !enabled {
            return Ok(None);
        }
        let service_key = std::env::var("LINK_ROUTING_SERVICE_KEY")
            .map_err(|_| "routing admission needs its own service key")?;
        if service_key.len() < 32 {
            return Err("routing service key must be at least 32 bytes");
        }
        for shared_name in ["LINK_API_KEY", "LINK_ADMIN_KEY", "TYPESAFE_ROUTING_API_KEY"] {
            if std::env::var(shared_name).ok().as_deref() == Some(service_key.as_str()) {
                return Err("routing service key must differ from client, admin and provider keys");
            }
        }
        let mac_hex = std::env::var("LINK_ROUTING_PAYLOAD_MAC_KEY")
            .map_err(|_| "routing admission needs a 32-byte MAC key")?;
        let mac_bytes = hex::decode(mac_hex).map_err(|_| "routing MAC key must be hex")?;
        let payload_mac_key: [u8; 32] = mac_bytes
            .try_into()
            .map_err(|_| "routing MAC key must be 32 bytes")?;
        // An operator must separately verify the all-in billable ceiling,
        // including output/framing, before setting these values. Zero means
        // the service may authenticate but cannot admit a paid send.
        let verified_input_ceiling = std::env::var("ROUTING_VERIFIED_INPUT_CEILING")
            .ok()
            .and_then(|value| value.parse::<i64>().ok())
            .unwrap_or(0);
        let verified_output_ceiling = std::env::var("ROUTING_VERIFIED_OUTPUT_CEILING")
            .ok()
            .and_then(|value| value.parse::<i64>().ok())
            .unwrap_or(0);
        let verified_total_nano_usd_ceiling =
            std::env::var("ROUTING_VERIFIED_TOTAL_NANO_USD_CEILING")
                .ok()
                .and_then(|value| value.parse::<i64>().ok())
                .unwrap_or(0);
        let rate_card_revision = std::env::var("ROUTING_RATE_CARD_REVISION").unwrap_or_default();
        // An absent or malformed rate is distinct from a pinned free output
        // rate. Both prices must be explicitly supplied by the operator.
        let rate = |name| pinned_rate(std::env::var(name).ok().as_deref()).unwrap_or(-1);
        Ok(Some(Self {
            service_key_digest: Sha256::digest(service_key.as_bytes()).into(),
            payload_mac_key,
            verified_input_ceiling,
            verified_output_ceiling,
            verified_total_nano_usd_ceiling,
            rate_card_revision,
            input_nano_usd_per_token: rate("ROUTING_INPUT_NANO_USD_PER_TOKEN"),
            output_nano_usd_per_token: rate("ROUTING_OUTPUT_NANO_USD_PER_TOKEN"),
        }))
    }

    fn has_verified_bounds(&self) -> bool {
        self.verified_input_ceiling > 0
            && self.verified_input_ceiling <= MONTHLY_INPUT_LIMIT
            && self.verified_output_ceiling > 0
            && self.verified_total_nano_usd_ceiling > 0
            && !self.rate_card_revision.is_empty()
            && self.input_nano_usd_per_token > 0
            && self.output_nano_usd_per_token >= 0
            && self
                .verified_input_ceiling
                .checked_mul(self.input_nano_usd_per_token)
                .and_then(|input| {
                    self.verified_output_ceiling
                        .checked_mul(self.output_nano_usd_per_token)
                        .and_then(|output| input.checked_add(output))
                })
                .is_some_and(|minimum| minimum <= self.verified_total_nano_usd_ceiling)
    }

    fn authenticates(&self, headers: &HeaderMap) -> bool {
        let Some(candidate) = headers
            .get("x-pytxo-routing-service")
            .and_then(|value| value.to_str().ok())
        else {
            return false;
        };
        let digest: [u8; 32] = Sha256::digest(candidate.as_bytes()).into();
        digest
            .iter()
            .zip(self.service_key_digest.iter())
            .fold(0u8, |difference, (left, right)| difference | (left ^ right))
            == 0
    }

    fn mac(&self, account_id: &str, payload_digest: &[u8; 32]) -> [u8; 32] {
        let mut mac =
            HmacSha256::new_from_slice(&self.payload_mac_key).expect("HMAC accepts 32-byte keys");
        mac.update(account_id.as_bytes());
        mac.update(&[0]);
        mac.update(payload_digest);
        mac.finalize().into_bytes().into()
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AdmissionError {
    BadRequest,
    Unauthorized,
    Conflict,
    Limited,
    Unavailable,
}

impl AdmissionError {
    fn status(self) -> StatusCode {
        match self {
            Self::BadRequest => StatusCode::BAD_REQUEST,
            Self::Unauthorized => StatusCode::UNAUTHORIZED,
            Self::Conflict => StatusCode::CONFLICT,
            Self::Limited => StatusCode::TOO_MANY_REQUESTS,
            Self::Unavailable => StatusCode::SERVICE_UNAVAILABLE,
        }
    }
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ReserveBody {
    request_id: String,
    payload_sha256: String,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ClaimBody {
    request_id: String,
}

#[derive(Clone, Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SettleBody {
    request_id: String,
    owner_nonce: String,
    actual_input_tokens: i64,
    actual_output_tokens: i64,
    actual_nano_usd: i64,
    rate_card_revision: String,
    receipt_id: String,
    recovery: RecoveryBlob,
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct RecoveryBlob {
    key_revision: String,
    nonce: String,
    ciphertext: String,
}

impl RecoveryBlob {
    fn decode(&self) -> Result<(Vec<u8>, Vec<u8>), AdmissionError> {
        if self.key_revision.is_empty()
            || self.key_revision.len() > 64
            || !self
                .key_revision
                .bytes()
                .all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'-' | b'_' | b'/'))
        {
            return Err(AdmissionError::BadRequest);
        }
        let nonce = URL_SAFE_NO_PAD
            .decode(&self.nonce)
            .map_err(|_| AdmissionError::BadRequest)?;
        let ciphertext = URL_SAFE_NO_PAD
            .decode(&self.ciphertext)
            .map_err(|_| AdmissionError::BadRequest)?;
        if nonce.len() != 12 || !(17..=2048).contains(&ciphertext.len()) {
            return Err(AdmissionError::BadRequest);
        }
        Ok((nonce, ciphertext))
    }

    fn from_columns(revision: String, nonce: Vec<u8>, ciphertext: Vec<u8>) -> Self {
        Self {
            key_revision: revision,
            nonce: URL_SAFE_NO_PAD.encode(nonce),
            ciphertext: URL_SAFE_NO_PAD.encode(ciphertext),
        }
    }
}

#[derive(sqlx::FromRow)]
struct SettlementRow {
    state: String,
    reserved_input_tokens: i64,
    reserved_output_tokens: i64,
    reserved_nano_usd: i64,
    rate_card_revision: String,
    input_nano_usd_per_token: i64,
    output_nano_usd_per_token: i64,
    actual_input_tokens: Option<i64>,
    actual_output_tokens: Option<i64>,
    actual_nano_usd: Option<i64>,
    receipt_id: Option<String>,
    recovery_ciphertext: Option<Vec<u8>>,
    recovery_nonce: Option<Vec<u8>>,
    recovery_key_revision: Option<String>,
}

#[derive(sqlx::FromRow)]
struct RecoveryRow {
    payload_mac: Vec<u8>,
    state: String,
    workspace_id: Option<Vec<u8>>,
    grant_revision: Option<i64>,
    recipient_identity: Option<String>,
    scope_digest: Option<Vec<u8>>,
    recovery_ciphertext: Option<Vec<u8>>,
    recovery_nonce: Option<Vec<u8>>,
    recovery_key_revision: Option<String>,
    recovery_live: bool,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct GrantBinding {
    pub workspace_id: [u8; 16],
    pub revision: i64,
    pub recipient_identity: String,
    pub scope_digest: [u8; 32],
}

#[derive(sqlx::FromRow)]
struct TokenBindingRow {
    account_id: String,
    workspace_id: Option<Vec<u8>>,
    grant_revision: Option<i64>,
    recipient_identity: Option<String>,
    scope_digest: Option<Vec<u8>>,
}

#[derive(sqlx::FromRow)]
struct LockedTokenRow {
    account_id: String,
    scope: String,
    expires_at: DateTime<Utc>,
    not_revoked: bool,
    workspace_id: Option<Vec<u8>>,
    grant_revision: Option<i64>,
    recipient_identity: Option<String>,
    scope_digest: Option<Vec<u8>>,
}

#[derive(sqlx::FromRow)]
struct OperationBindingRow {
    account_id: String,
    utc_month: NaiveDate,
    workspace_id: Option<Vec<u8>>,
    grant_revision: Option<i64>,
    recipient_identity: Option<String>,
    scope_digest: Option<Vec<u8>>,
}

impl GrantBinding {
    fn from_columns(
        workspace_id: Option<Vec<u8>>,
        revision: Option<i64>,
        recipient_identity: Option<String>,
        scope_digest: Option<Vec<u8>>,
    ) -> Option<Self> {
        Some(Self {
            workspace_id: workspace_id?.try_into().ok()?,
            revision: revision?,
            recipient_identity: recipient_identity?,
            scope_digest: scope_digest?.try_into().ok()?,
        })
    }

    fn current_scope(&self) -> bool {
        self.recipient_identity == HOSTED_RECIPIENT && self.scope_digest == hosted_scope_digest()
    }
}

#[derive(sqlx::FromRow)]
struct StaleOperation {
    state: String,
    reserved_input_tokens: i64,
    reserved_nano_usd: i64,
    created_at: DateTime<Utc>,
    send_claimed_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Serialize)]
pub struct AdmissionReply {
    request_id: String,
    state: String,
    new_reservation: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    owner_nonce: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    recovery: Option<RecoveryBlob>,
}

fn routing_token(headers: &HeaderMap) -> Result<&str, AdmissionError> {
    let token = headers
        .get(header::AUTHORIZATION)
        .and_then(|value| value.to_str().ok())
        .and_then(|value| value.strip_prefix("Bearer "))
        .ok_or(AdmissionError::Unauthorized)?;
    if token.len() != 47 || !token.starts_with("pr1_") {
        return Err(AdmissionError::Unauthorized);
    }
    Ok(token)
}

fn token_hash(token: &str) -> [u8; 32] {
    Sha256::digest(token.as_bytes()).into()
}

fn service_context(
    state: &AppState,
    headers: &HeaderMap,
) -> Result<(PgPool, RoutingAdmissionConfig), AdmissionError> {
    let config = state
        .routing_admission
        .as_ref()
        .ok_or(AdmissionError::Unavailable)?;
    if !config.authenticates(headers) {
        return Err(AdmissionError::Unauthorized);
    }
    let pool = state.db.as_ref().ok_or(AdmissionError::Unavailable)?;
    Ok((pool.clone(), config.clone()))
}

fn request_timestamp(request_id: &str, now: DateTime<Utc>) -> Result<(), AdmissionError> {
    let bytes = request_id.as_bytes();
    if bytes.len() != 36
        || bytes[8] != b'-'
        || bytes[13] != b'-'
        || bytes[18] != b'-'
        || bytes[23] != b'-'
        || bytes[14] != b'7'
        || !matches!(bytes[19], b'8' | b'9' | b'a' | b'b')
        || !bytes
            .iter()
            .enumerate()
            .all(|(index, byte)| matches!(index, 8 | 13 | 18 | 23) || byte.is_ascii_hexdigit())
    {
        return Err(AdmissionError::BadRequest);
    }
    let millis = i64::from_str_radix(&format!("{}{}", &request_id[..8], &request_id[9..13]), 16)
        .map_err(|_| AdmissionError::BadRequest)?;
    let timestamp =
        DateTime::<Utc>::from_timestamp_millis(millis).ok_or(AdmissionError::BadRequest)?;
    if timestamp < now - Duration::hours(24) || timestamp > now + Duration::minutes(5) {
        return Err(AdmissionError::BadRequest);
    }
    Ok(())
}

async fn current_token(
    pool: &PgPool,
    digest: &[u8; 32],
) -> Result<(String, Option<GrantBinding>), AdmissionError> {
    let row: TokenBindingRow = sqlx::query_as(
        "SELECT account_id, workspace_id, grant_revision, recipient_identity, scope_digest FROM routing_session_tokens WHERE token_hash = $1",
    )
    .bind(digest.as_slice())
    .fetch_optional(pool)
    .await
    .map_err(|_| AdmissionError::Unavailable)?
    .ok_or(AdmissionError::Unauthorized)?;
    Ok((
        row.account_id,
        GrantBinding::from_columns(
            row.workspace_id,
            row.grant_revision,
            row.recipient_identity,
            row.scope_digest,
        ),
    ))
}

async fn lock_live_grant(
    tx: &mut Transaction<'_, Postgres>,
    account: &str,
    binding: &GrantBinding,
) -> Result<bool, AdmissionError> {
    if !binding.current_scope() {
        return Ok(false);
    }
    let row: Option<(i64, bool, Vec<u8>)> = sqlx::query_as(
        "SELECT revision, enabled, scope_digest FROM routing_workspace_grants WHERE account_id = $1 AND workspace_id = $2 AND recipient_identity = $3 FOR UPDATE",
    )
    .bind(account).bind(binding.workspace_id.as_slice()).bind(&binding.recipient_identity)
    .fetch_optional(&mut **tx).await.map_err(|_| AdmissionError::Unavailable)?;
    Ok(row.is_some_and(|(revision, enabled, scope)| {
        enabled && revision == binding.revision && scope == binding.scope_digest
    }))
}

async fn lock_funding(
    tx: &mut Transaction<'_, Postgres>,
) -> Result<(bool, i64, i64, i64), AdmissionError> {
    sqlx::query_as(
        "SELECT enabled, ceiling_nano_usd, spent_nano_usd, held_nano_usd FROM routing_funding WHERE singleton = TRUE FOR UPDATE",
    )
    .fetch_one(&mut **tx)
    .await
    .map_err(|_| AdmissionError::Unavailable)
}

async fn lock_gate(
    tx: &mut Transaction<'_, Postgres>,
    account: &str,
) -> Result<i32, AdmissionError> {
    sqlx::query("INSERT INTO routing_account_gate (account_id) VALUES ($1) ON CONFLICT DO NOTHING")
        .bind(account)
        .execute(&mut **tx)
        .await
        .map_err(|_| AdmissionError::Unavailable)?;
    sqlx::query_scalar("SELECT active FROM routing_account_gate WHERE account_id = $1 FOR UPDATE")
        .bind(account)
        .fetch_one(&mut **tx)
        .await
        .map_err(|_| AdmissionError::Unavailable)
}

async fn lock_month(
    tx: &mut Transaction<'_, Postgres>,
    account: &str,
    month: NaiveDate,
) -> Result<(i64, i64), AdmissionError> {
    sqlx::query(
        "INSERT INTO routing_account_month (account_id, utc_month) VALUES ($1, $2) ON CONFLICT DO NOTHING",
    )
    .bind(account).bind(month).execute(&mut **tx).await.map_err(|_| AdmissionError::Unavailable)?;
    sqlx::query_as(
        "SELECT held_input_tokens, consumed_input_tokens FROM routing_account_month WHERE account_id = $1 AND utc_month = $2 FOR UPDATE",
    )
    .bind(account).bind(month)
    .fetch_one(&mut **tx)
    .await
    .map_err(|_| AdmissionError::Unavailable)
}

async fn lock_live_token(
    tx: &mut Transaction<'_, Postgres>,
    account: &str,
    digest: &[u8; 32],
    binding: &GrantBinding,
) -> Result<bool, AdmissionError> {
    let row: Option<LockedTokenRow> =
        sqlx::query_as(
            "SELECT account_id, scope, expires_at, revoked_at IS NULL AS not_revoked, workspace_id, grant_revision, recipient_identity, scope_digest FROM routing_session_tokens WHERE token_hash = $1 FOR UPDATE",
        )
        .bind(digest.as_slice()).fetch_optional(&mut **tx).await.map_err(|_| AdmissionError::Unavailable)?;
    let clock: DateTime<Utc> = sqlx::query_scalar("SELECT clock_timestamp()")
        .fetch_one(&mut **tx)
        .await
        .map_err(|_| AdmissionError::Unavailable)?;
    Ok(row.is_some_and(|row| {
        row.account_id == account
            && row.scope == TOKEN_SCOPE
            && row.expires_at > clock
            && row.not_revoked
            && GrantBinding::from_columns(
                row.workspace_id,
                row.grant_revision,
                row.recipient_identity,
                row.scope_digest,
            )
            .as_ref()
                == Some(binding)
    }))
}

async fn admission_month(tx: &mut Transaction<'_, Postgres>) -> Result<NaiveDate, AdmissionError> {
    sqlx::query_scalar("SELECT date_trunc('month', clock_timestamp() AT TIME ZONE 'UTC')::date")
        .fetch_one(&mut **tx)
        .await
        .map_err(|_| AdmissionError::Unavailable)
}

pub async fn reserve(
    pool: &PgPool,
    config: &RoutingAdmissionConfig,
    token: &str,
    request_id: &str,
    payload_digest: [u8; 32],
) -> Result<AdmissionReply, AdmissionError> {
    let digest = token_hash(token);
    let (account, binding) = current_token(pool, &digest).await?;
    let mut tx = pool
        .begin()
        .await
        .map_err(|_| AdmissionError::Unavailable)?;
    let (enabled, ceiling, spent, held) = lock_funding(&mut tx).await?;
    let now: DateTime<Utc> = sqlx::query_scalar("SELECT clock_timestamp()")
        .fetch_one(&mut *tx)
        .await
        .map_err(|_| AdmissionError::Unavailable)?;
    request_timestamp(request_id, now)?;
    let active = lock_gate(&mut tx, &account).await?;
    let month = admission_month(&mut tx).await?;
    let (month_held, month_consumed) = lock_month(&mut tx, &account, month).await?;
    let binding = binding.ok_or(AdmissionError::Unauthorized)?;
    if !lock_live_grant(&mut tx, &account, &binding).await?
        || !lock_live_token(&mut tx, &account, &digest, &binding).await?
    {
        return Err(AdmissionError::Unauthorized);
    }
    let mac = config.mac(&account, &payload_digest);
    let old: Option<RecoveryRow> = sqlx::query_as(
        "SELECT payload_mac, state, workspace_id, grant_revision, recipient_identity, scope_digest, recovery_ciphertext, recovery_nonce, recovery_key_revision, COALESCE(recovery_expires_at > clock_timestamp(), FALSE) AS recovery_live FROM routing_operations WHERE account_id = $1 AND request_id = $2 FOR UPDATE",
    )
    .bind(&account).bind(request_id).fetch_optional(&mut *tx).await
    .map_err(|_| AdmissionError::Unavailable)?;
    if let Some(old) = old {
        if GrantBinding::from_columns(
            old.workspace_id,
            old.grant_revision,
            old.recipient_identity,
            old.scope_digest,
        )
        .as_ref()
            != Some(&binding)
        {
            return Err(AdmissionError::Conflict);
        }
        if old.payload_mac != mac {
            return Err(AdmissionError::Conflict);
        }
        let recovery = if old.state == "completed" && old.recovery_live {
            match (
                old.recovery_key_revision,
                old.recovery_nonce,
                old.recovery_ciphertext,
            ) {
                (Some(revision), Some(nonce), Some(ciphertext)) => {
                    Some(RecoveryBlob::from_columns(revision, nonce, ciphertext))
                }
                _ => None,
            }
        } else {
            None
        };
        return Ok(AdmissionReply {
            request_id: request_id.into(),
            state: old.state,
            new_reservation: false,
            owner_nonce: None,
            recovery,
        });
    }
    if !enabled || ceiling <= 0 || !config.has_verified_bounds() {
        return Err(AdmissionError::Unavailable);
    }
    let recent: i64 = sqlx::query_scalar(
        "SELECT count(*) FROM routing_operations WHERE account_id = $1 AND created_at > clock_timestamp() - interval '1 minute'",
    )
    .bind(&account).fetch_one(&mut *tx).await.map_err(|_| AdmissionError::Unavailable)?;
    if recent >= i64::from(REQUESTS_PER_MINUTE) || active >= MAX_ACTIVE {
        return Err(AdmissionError::Limited);
    }
    if month_held
        .checked_add(month_consumed)
        .and_then(|v| v.checked_add(config.verified_input_ceiling))
        .is_none_or(|v| v > MONTHLY_INPUT_LIMIT)
        || spent
            .checked_add(held)
            .and_then(|v| v.checked_add(config.verified_total_nano_usd_ceiling))
            .is_none_or(|v| v > ceiling)
    {
        return Err(AdmissionError::Limited);
    }
    sqlx::query(
        "INSERT INTO routing_operations (account_id, request_id, token_hash, payload_mac, mac_revision, state, utc_month, rate_card_revision, reserved_input_tokens, reserved_output_tokens, reserved_nano_usd, input_nano_usd_per_token, output_nano_usd_per_token, created_at, workspace_id, grant_revision, recipient_identity, scope_digest) VALUES ($1,$2,$3,$4,'hmac-sha256/v1','reserved',$5,$6,$7,$8,$9,$10,$11,clock_timestamp(),$12,$13,$14,$15)",
    )
    .bind(&account).bind(request_id).bind(digest.as_slice()).bind(mac.as_slice())
    .bind(month).bind(&config.rate_card_revision)
    .bind(config.verified_input_ceiling).bind(config.verified_output_ceiling)
    .bind(config.verified_total_nano_usd_ceiling)
    .bind(config.input_nano_usd_per_token).bind(config.output_nano_usd_per_token)
    .bind(binding.workspace_id.as_slice()).bind(binding.revision)
    .bind(&binding.recipient_identity).bind(binding.scope_digest.as_slice())
    .execute(&mut *tx).await.map_err(|_| AdmissionError::Unavailable)?;
    sqlx::query("UPDATE routing_account_gate SET active = active + 1 WHERE account_id = $1")
        .bind(&account)
        .execute(&mut *tx)
        .await
        .map_err(|_| AdmissionError::Unavailable)?;
    sqlx::query(
        "UPDATE routing_account_month SET held_input_tokens = held_input_tokens + $3 WHERE account_id = $1 AND utc_month = $2",
    )
    .bind(&account).bind(month).bind(config.verified_input_ceiling)
    .execute(&mut *tx).await.map_err(|_| AdmissionError::Unavailable)?;
    sqlx::query(
        "UPDATE routing_funding SET held_nano_usd = held_nano_usd + $1 WHERE singleton = TRUE",
    )
    .bind(config.verified_total_nano_usd_ceiling)
    .execute(&mut *tx)
    .await
    .map_err(|_| AdmissionError::Unavailable)?;
    tx.commit().await.map_err(|_| AdmissionError::Unavailable)?;
    Ok(AdmissionReply {
        request_id: request_id.into(),
        state: "reserved".into(),
        new_reservation: true,
        owner_nonce: None,
        recovery: None,
    })
}

async fn release_reservation(
    tx: &mut Transaction<'_, Postgres>,
    account: &str,
    month: NaiveDate,
    input_tokens: i64,
    nano_usd: i64,
) -> Result<(), AdmissionError> {
    sqlx::query(
        "UPDATE routing_funding SET held_nano_usd = held_nano_usd - $1 WHERE singleton = TRUE",
    )
    .bind(nano_usd)
    .execute(&mut **tx)
    .await
    .map_err(|_| AdmissionError::Unavailable)?;
    sqlx::query("UPDATE routing_account_gate SET active = active - 1 WHERE account_id = $1")
        .bind(account)
        .execute(&mut **tx)
        .await
        .map_err(|_| AdmissionError::Unavailable)?;
    sqlx::query(
        "UPDATE routing_account_month SET held_input_tokens = held_input_tokens - $3 WHERE account_id = $1 AND utc_month = $2",
    )
    .bind(account).bind(month).bind(input_tokens)
    .execute(&mut **tx).await.map_err(|_| AdmissionError::Unavailable)?;
    Ok(())
}

pub async fn claim(
    pool: &PgPool,
    config: &RoutingAdmissionConfig,
    token: &str,
    request_id: &str,
) -> Result<AdmissionReply, AdmissionError> {
    let digest = token_hash(token);
    let operation: OperationBindingRow = sqlx::query_as(
        "SELECT account_id, utc_month, workspace_id, grant_revision, recipient_identity, scope_digest FROM routing_operations WHERE token_hash = $1 AND request_id = $2",
    )
    .bind(digest.as_slice()).bind(request_id)
    .fetch_optional(pool).await.map_err(|_| AdmissionError::Unavailable)?
    .ok_or(AdmissionError::Unauthorized)?;
    let account = operation.account_id;
    let month = operation.utc_month;
    let binding = GrantBinding::from_columns(
        operation.workspace_id,
        operation.grant_revision,
        operation.recipient_identity,
        operation.scope_digest,
    );
    let mut tx = pool
        .begin()
        .await
        .map_err(|_| AdmissionError::Unavailable)?;
    let (funding_enabled, _, _, _) = lock_funding(&mut tx).await?;
    let _ = lock_gate(&mut tx, &account).await?;
    let _ = lock_month(&mut tx, &account, month).await?;
    let grant_live = if let Some(binding) = binding.as_ref() {
        lock_live_grant(&mut tx, &account, binding).await?
    } else {
        false
    };
    let token_live = if let Some(binding) = binding.as_ref() {
        lock_live_token(&mut tx, &account, &digest, binding).await?
    } else {
        false
    };
    let (state, input_tokens, output_tokens, nano_usd, revision, input_rate, output_rate):
        (String, i64, i64, i64, String, i64, i64) = sqlx::query_as(
        "SELECT state, reserved_input_tokens, reserved_output_tokens, reserved_nano_usd, rate_card_revision, input_nano_usd_per_token, output_nano_usd_per_token FROM routing_operations WHERE account_id = $1 AND request_id = $2 FOR UPDATE",
    )
    .bind(&account).bind(request_id).fetch_one(&mut *tx).await
    .map_err(|_| AdmissionError::Unavailable)?;
    if state != "reserved" {
        return Ok(AdmissionReply {
            request_id: request_id.into(),
            state,
            new_reservation: false,
            owner_nonce: None,
            recovery: None,
        });
    }
    let bounds_match = config.has_verified_bounds()
        && input_tokens == config.verified_input_ceiling
        && output_tokens == config.verified_output_ceiling
        && nano_usd == config.verified_total_nano_usd_ceiling
        && revision == config.rate_card_revision
        && input_rate == config.input_nano_usd_per_token
        && output_rate == config.output_nano_usd_per_token;
    if !funding_enabled || !grant_live || !token_live || !bounds_match {
        release_reservation(&mut tx, &account, month, input_tokens, nano_usd).await?;
        sqlx::query(
            "UPDATE routing_operations SET state = 'rejected_before_send', settled_at = clock_timestamp() WHERE account_id = $1 AND request_id = $2",
        )
        .bind(&account).bind(request_id).execute(&mut *tx).await
        .map_err(|_| AdmissionError::Unavailable)?;
        tx.commit().await.map_err(|_| AdmissionError::Unavailable)?;
        return Ok(AdmissionReply {
            request_id: request_id.into(),
            state: "rejected_before_send".into(),
            new_reservation: false,
            owner_nonce: None,
            recovery: None,
        });
    }
    let mut nonce = [0u8; 32];
    getrandom::getrandom(&mut nonce).map_err(|_| AdmissionError::Unavailable)?;
    sqlx::query(
        "UPDATE routing_operations SET state = 'sending', owner_nonce = $3, send_claimed_at = clock_timestamp() WHERE account_id = $1 AND request_id = $2 AND state = 'reserved'",
    )
    .bind(&account).bind(request_id).bind(nonce.as_slice())
    .execute(&mut *tx).await.map_err(|_| AdmissionError::Unavailable)?;
    tx.commit().await.map_err(|_| AdmissionError::Unavailable)?;
    Ok(AdmissionReply {
        request_id: request_id.into(),
        state: "sending".into(),
        new_reservation: false,
        owner_nonce: Some(hex::encode(nonce)),
        recovery: None,
    })
}

fn decode_nonce(nonce: &str) -> Result<[u8; 32], AdmissionError> {
    hex::decode(nonce)
        .map_err(|_| AdmissionError::BadRequest)?
        .try_into()
        .map_err(|_| AdmissionError::BadRequest)
}

async fn owned_operation(
    pool: &PgPool,
    request_id: &str,
    nonce: &[u8; 32],
) -> Result<(String, NaiveDate), AdmissionError> {
    sqlx::query_as(
        "SELECT account_id, utc_month FROM routing_operations WHERE request_id = $1 AND owner_nonce = $2",
    )
    .bind(request_id).bind(nonce.as_slice())
    .fetch_optional(pool).await.map_err(|_| AdmissionError::Unavailable)?
    .ok_or(AdmissionError::Unauthorized)
}

pub async fn mark_uncertain(
    pool: &PgPool,
    request_id: &str,
    owner_nonce: &str,
) -> Result<AdmissionReply, AdmissionError> {
    let nonce = decode_nonce(owner_nonce)?;
    let (account, month) = owned_operation(pool, request_id, &nonce).await?;
    let mut tx = pool
        .begin()
        .await
        .map_err(|_| AdmissionError::Unavailable)?;
    let _ = lock_funding(&mut tx).await?;
    let _ = lock_gate(&mut tx, &account).await?;
    let _ = lock_month(&mut tx, &account, month).await?;
    let state: String = sqlx::query_scalar(
        "SELECT state FROM routing_operations WHERE account_id = $1 AND request_id = $2 AND owner_nonce = $3 FOR UPDATE",
    )
    .bind(&account).bind(request_id).bind(nonce.as_slice())
    .fetch_one(&mut *tx).await.map_err(|_| AdmissionError::Unavailable)?;
    if state == "sending" {
        sqlx::query(
            "UPDATE routing_operations SET state = 'uncertain'     WHERE account_id = $1 AND request_id = $2",
        )
        .bind(&account).bind(request_id).execute(&mut *tx).await
        .map_err(|_| AdmissionError::Unavailable)?;
        tx.commit().await.map_err(|_| AdmissionError::Unavailable)?;
        return Ok(AdmissionReply {
            request_id: request_id.into(),
            state: "uncertain".into(),
            new_reservation: false,
            owner_nonce: None,
            recovery: None,
        });
    }
    Ok(AdmissionReply {
        request_id: request_id.into(),
        state,
        new_reservation: false,
        owner_nonce: None,
        recovery: None,
    })
}

pub async fn settle(pool: &PgPool, body: &SettleBody) -> Result<AdmissionReply, AdmissionError> {
    if body.actual_input_tokens < 0
        || body.actual_output_tokens < 0
        || body.actual_nano_usd < 0
        || body.receipt_id.is_empty()
        || body.receipt_id.len() > 128
        || !body
            .receipt_id
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || byte == b'-' || byte == b'_')
    {
        return Err(AdmissionError::BadRequest);
    }
    let (recovery_nonce, recovery_ciphertext) = body.recovery.decode()?;
    let nonce = decode_nonce(&body.owner_nonce)?;
    let (account, month) = owned_operation(pool, &body.request_id, &nonce).await?;
    let mut tx = pool
        .begin()
        .await
        .map_err(|_| AdmissionError::Unavailable)?;
    let _ = lock_funding(&mut tx).await?;
    let _ = lock_gate(&mut tx, &account).await?;
    let _ = lock_month(&mut tx, &account, month).await?;
    let row: SettlementRow = sqlx::query_as(
        "SELECT state, reserved_input_tokens, reserved_output_tokens, reserved_nano_usd, rate_card_revision, input_nano_usd_per_token, output_nano_usd_per_token, actual_input_tokens, actual_output_tokens, actual_nano_usd, receipt_id, recovery_ciphertext, recovery_nonce, recovery_key_revision FROM routing_operations WHERE account_id = $1 AND request_id = $2 AND owner_nonce = $3 FOR UPDATE",
    )
    .bind(&account).bind(&body.request_id).bind(nonce.as_slice())
    .fetch_one(&mut *tx).await.map_err(|_| AdmissionError::Unavailable)?;
    if row.state == "completed" {
        if row.actual_input_tokens != Some(body.actual_input_tokens)
            || row.actual_output_tokens != Some(body.actual_output_tokens)
            || row.actual_nano_usd != Some(body.actual_nano_usd)
            || row.receipt_id.as_deref() != Some(body.receipt_id.as_str())
            || row.rate_card_revision != body.rate_card_revision
            || row.recovery_ciphertext.as_deref() != Some(recovery_ciphertext.as_slice())
            || row.recovery_nonce.as_deref() != Some(recovery_nonce.as_slice())
            || row.recovery_key_revision.as_deref() != Some(body.recovery.key_revision.as_str())
        {
            return Err(AdmissionError::Conflict);
        }
        return Ok(AdmissionReply {
            request_id: body.request_id.clone(),
            state: row.state,
            new_reservation: false,
            owner_nonce: None,
            recovery: None,
        });
    }
    if !matches!(
        row.state.as_str(),
        "sending" | "uncertain" | "settled_at_ceiling"
    ) {
        return Err(AdmissionError::Conflict);
    }
    if body.rate_card_revision != row.rate_card_revision {
        return Err(AdmissionError::Conflict);
    }
    let expected_cost = body
        .actual_input_tokens
        .checked_mul(row.input_nano_usd_per_token)
        .and_then(|input| {
            body.actual_output_tokens
                .checked_mul(row.output_nano_usd_per_token)
                .and_then(|output| input.checked_add(output))
        });
    if row.input_nano_usd_per_token <= 0
        || row.output_nano_usd_per_token < 0
        || expected_cost != Some(body.actual_nano_usd)
    {
        return Err(AdmissionError::Conflict);
    }
    let was_ceiling = row.state == "settled_at_ceiling";
    if !was_ceiling {
        release_reservation(
            &mut tx,
            &account,
            month,
            row.reserved_input_tokens,
            row.reserved_nano_usd,
        )
        .await?;
    }
    let cost_delta = if was_ceiling {
        body.actual_nano_usd - row.reserved_nano_usd
    } else {
        body.actual_nano_usd
    };
    let input_delta = if was_ceiling {
        body.actual_input_tokens - row.reserved_input_tokens
    } else {
        body.actual_input_tokens
    };
    sqlx::query(
        "UPDATE routing_funding SET spent_nano_usd = spent_nano_usd + $1, enabled = enabled AND $2 <= $3 AND $4 <= $5 AND $6 <= $7, disabled_reason = CASE WHEN $2 > $3 OR $4 > $5 OR $6 > $7 THEN 'billable_usage_over_reserve' ELSE disabled_reason END WHERE singleton = TRUE",
    )
    .bind(cost_delta).bind(body.actual_nano_usd).bind(row.reserved_nano_usd)
    .bind(body.actual_input_tokens).bind(row.reserved_input_tokens)
    .bind(body.actual_output_tokens).bind(row.reserved_output_tokens)
    .execute(&mut *tx).await.map_err(|_| AdmissionError::Unavailable)?;
    sqlx::query(
        "UPDATE routing_account_month SET consumed_input_tokens = consumed_input_tokens + $3 WHERE account_id = $1 AND utc_month = $2",
    )
    .bind(&account).bind(month).bind(input_delta)
    .execute(&mut *tx).await.map_err(|_| AdmissionError::Unavailable)?;
    sqlx::query(
        "UPDATE routing_operations SET state = 'completed', actual_input_tokens = $4, actual_output_tokens = $5, actual_nano_usd = $6, receipt_id = $7, recovery_nonce = $8, recovery_ciphertext = $9, recovery_key_revision = $10, recovery_expires_at = clock_timestamp() + interval '24 hours', settled_at = clock_timestamp() WHERE account_id = $1 AND request_id = $2 AND owner_nonce = $3",
    )
    .bind(&account).bind(&body.request_id).bind(nonce.as_slice())
    .bind(body.actual_input_tokens).bind(body.actual_output_tokens)
    .bind(body.actual_nano_usd).bind(&body.receipt_id)
    .bind(&recovery_nonce).bind(&recovery_ciphertext).bind(&body.recovery.key_revision)
    .execute(&mut *tx).await.map_err(|_| AdmissionError::Unavailable)?;
    tx.commit().await.map_err(|_| AdmissionError::Unavailable)?;
    Ok(AdmissionReply {
        request_id: body.request_id.clone(),
        state: "completed".into(),
        new_reservation: false,
        owner_nonce: None,
        recovery: None,
    })
}

/// Conservatively close operations that lost their proxy owner. The provider
/// is never retried. UUIDv7 age checks prevent replay after tombstone pruning.
pub async fn reconcile_stale(pool: &PgPool) -> Result<usize, AdmissionError> {
    let candidates: Vec<(String, String, NaiveDate)> = sqlx::query_as(
        "SELECT account_id, request_id, utc_month FROM routing_operations WHERE (state = 'reserved' AND created_at < clock_timestamp() - interval '2 minutes') OR (state IN ('sending','uncertain') AND send_claimed_at < clock_timestamp() - interval '10 minutes') ORDER BY created_at LIMIT 100",
    )
    .fetch_all(pool).await.map_err(|_| AdmissionError::Unavailable)?;
    let mut closed = 0usize;
    for (account, request_id, month) in candidates {
        let mut tx = pool
            .begin()
            .await
            .map_err(|_| AdmissionError::Unavailable)?;
        let _ = lock_funding(&mut tx).await?;
        let _ = lock_gate(&mut tx, &account).await?;
        let _ = lock_month(&mut tx, &account, month).await?;
        let row: Option<StaleOperation> =
            sqlx::query_as(
                "SELECT state, reserved_input_tokens, reserved_nano_usd, created_at, send_claimed_at         FROM routing_operations WHERE account_id = $1 AND request_id = $2 FOR UPDATE",
            )
            .bind(&account).bind(&request_id).fetch_optional(&mut *tx).await
            .map_err(|_| AdmissionError::Unavailable)?;
        let Some(row) = row else {
            continue;
        };
        let now: DateTime<Utc> = sqlx::query_scalar("SELECT clock_timestamp()")
            .fetch_one(&mut *tx)
            .await
            .map_err(|_| AdmissionError::Unavailable)?;
        let reserved_stale = row.state == "reserved" && row.created_at < now - Duration::minutes(2);
        let sending_stale = matches!(row.state.as_str(), "sending" | "uncertain")
            && row
                .send_claimed_at
                .is_some_and(|time| time < now - Duration::minutes(10));
        if !reserved_stale && !sending_stale {
            continue;
        }
        release_reservation(
            &mut tx,
            &account,
            month,
            row.reserved_input_tokens,
            row.reserved_nano_usd,
        )
        .await?;
        if sending_stale {
            sqlx::query(
                "UPDATE routing_funding SET spent_nano_usd = spent_nano_usd + $1         WHERE singleton = TRUE",
            )
            .bind(row.reserved_nano_usd).execute(&mut *tx).await.map_err(|_| AdmissionError::Unavailable)?;
            sqlx::query(
                "UPDATE routing_account_month SET consumed_input_tokens = consumed_input_tokens + $3         WHERE account_id = $1 AND utc_month = $2",
            )
            .bind(&account).bind(month).bind(row.reserved_input_tokens)
            .execute(&mut *tx).await.map_err(|_| AdmissionError::Unavailable)?;
        }
        let next = if reserved_stale {
            "rejected_before_send"
        } else {
            "settled_at_ceiling"
        };
        sqlx::query(
            "UPDATE routing_operations SET state = $3, settled_at = clock_timestamp() WHERE account_id = $1 AND request_id = $2",
        )
        .bind(&account).bind(&request_id).bind(next)
        .execute(&mut *tx).await.map_err(|_| AdmissionError::Unavailable)?;
        tx.commit().await.map_err(|_| AdmissionError::Unavailable)?;
        closed += 1;
    }
    // The operation key is still replay-protected by its UUIDv7 timestamp
    // after this bounded tombstone retention window ends.
    sqlx::query("UPDATE routing_operations SET recovery_ciphertext = NULL, recovery_nonce = NULL, recovery_key_revision = NULL, recovery_expires_at = NULL WHERE recovery_expires_at < clock_timestamp()")
        .execute(pool)
        .await
        .map_err(|_| AdmissionError::Unavailable)?;
    sqlx::query("DELETE FROM routing_operations WHERE settled_at < now() - interval '90 days'")
        .execute(pool)
        .await
        .map_err(|_| AdmissionError::Unavailable)?;
    Ok(closed)
}

pub async fn claim_route(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(body): Json<ClaimBody>,
) -> Result<Json<AdmissionReply>, StatusCode> {
    let (pool, config) = service_context(&state, &headers).map_err(AdmissionError::status)?;
    let token = routing_token(&headers).map_err(AdmissionError::status)?;
    claim(&pool, &config, token, &body.request_id)
        .await
        .map(Json)
        .map_err(AdmissionError::status)
}

pub async fn uncertain_route(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(body): Json<ClaimOwnerBody>,
) -> Result<Json<AdmissionReply>, StatusCode> {
    let (pool, _) = service_context(&state, &headers).map_err(AdmissionError::status)?;
    mark_uncertain(&pool, &body.request_id, &body.owner_nonce)
        .await
        .map(Json)
        .map_err(AdmissionError::status)
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ClaimOwnerBody {
    request_id: String,
    owner_nonce: String,
}

pub async fn settle_route(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(body): Json<SettleBody>,
) -> Result<Json<AdmissionReply>, StatusCode> {
    let (pool, _) = service_context(&state, &headers).map_err(AdmissionError::status)?;
    settle(&pool, &body)
        .await
        .map(Json)
        .map_err(AdmissionError::status)
}

pub async fn reserve_route(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(body): Json<ReserveBody>,
) -> Result<Json<AdmissionReply>, StatusCode> {
    let (pool, config) = service_context(&state, &headers).map_err(AdmissionError::status)?;
    let token = routing_token(&headers).map_err(AdmissionError::status)?;
    let raw = hex::decode(&body.payload_sha256).map_err(|_| StatusCode::BAD_REQUEST)?;
    let digest: [u8; 32] = raw.try_into().map_err(|_| StatusCode::BAD_REQUEST)?;
    reserve(&pool, &config, token, &body.request_id, digest)
        .await
        .map(Json)
        .map_err(AdmissionError::status)
}

#[cfg(test)]
mod tests {
    use super::*;
    use base64::engine::general_purpose::URL_SAFE_NO_PAD;
    use base64::Engine;

    #[test]
    fn verified_free_output_rate_requires_a_pinned_value_and_usage_ceiling() {
        let mut config = RoutingAdmissionConfig {
            service_key_digest: [1; 32],
            payload_mac_key: [2; 32],
            verified_input_ceiling: 100,
            verified_output_ceiling: 5,
            verified_total_nano_usd_ceiling: 500,
            rate_card_revision: "jev-input-only/v1".into(),
            input_nano_usd_per_token: 4,
            output_nano_usd_per_token: 0,
        };
        assert!(config.has_verified_bounds());
        config.verified_output_ceiling = 0;
        assert!(!config.has_verified_bounds());
        config.verified_output_ceiling = 5;
        config.output_nano_usd_per_token = -1;
        assert!(!config.has_verified_bounds());
        assert_eq!(pinned_rate(Some("0")), Some(0));
        assert_eq!(pinned_rate(None), None);
        assert_eq!(pinned_rate(Some("bad")), None);
        assert_eq!(pinned_rate(Some("-1")), None);
    }

    fn test_recovery() -> RecoveryBlob {
        RecoveryBlob {
            key_revision: "test-key/v1".into(),
            nonce: URL_SAFE_NO_PAD.encode([9u8; 12]),
            ciphertext: URL_SAFE_NO_PAD.encode([8u8; 32]),
        }
    }

    #[test]
    fn uuid_v7_age_and_structure_are_required() {
        // Version and timestamp structure alone do not make an opaque v4 ID
        // valid for the replay window.
        let now = DateTime::<Utc>::from_timestamp_millis(1_800_000_000_000).unwrap();
        let prefix = format!("{:012x}", now.timestamp_millis());
        let good = format!("{}-{}-7abc-8abc-0123456789ab", &prefix[..8], &prefix[8..]);
        assert!(request_timestamp(&good, now).is_ok());
        assert_eq!(
            request_timestamp(&good.replace("-7abc-", "-4abc-"), now),
            Err(AdmissionError::BadRequest)
        );
        let old = now - Duration::hours(25);
        let old_prefix = format!("{:012x}", old.timestamp_millis());
        let old_id = format!(
            "{}-{}-7abc-8abc-0123456789ab",
            &old_prefix[..8],
            &old_prefix[8..]
        );
        assert_eq!(
            request_timestamp(&old_id, now),
            Err(AdmissionError::BadRequest)
        );
    }

    #[tokio::test]
    async fn legacy_token_without_hosted_workspace_grant_cannot_reserve() {
        let Ok(url) = std::env::var("PYTXO_LINK_TEST_DATABASE_URL") else {
            return;
        };
        let parsed = reqwest::Url::parse(&url).unwrap();
        assert_eq!(parsed.host_str(), Some("127.0.0.1"));
        assert!(parsed.path().ends_with("/pytxo_routing_test"));
        let pool = crate::db::connect(&url).await.unwrap();
        let mut secret = [0u8; 32];
        getrandom::getrandom(&mut secret).unwrap();
        let token = format!("pr1_{}", URL_SAFE_NO_PAD.encode(secret));
        let account = format!("user_unscoped_{}", hex::encode(&secret[..8]));
        sqlx::query("INSERT INTO routing_session_tokens (token_hash, account_id, scope, expires_at) VALUES ($1,$2,$3,now()+interval '5 minutes')")
            .bind(token_hash(&token).as_slice()).bind(&account).bind(TOKEN_SCOPE)
            .execute(&pool).await.unwrap();
        sqlx::query("UPDATE routing_funding SET enabled = TRUE, ceiling_nano_usd = 1000, spent_nano_usd = 0, held_nano_usd = 0, disabled_reason = NULL WHERE singleton = TRUE")
            .execute(&pool).await.unwrap();
        let config = RoutingAdmissionConfig {
            service_key_digest: [1; 32],
            payload_mac_key: [2; 32],
            verified_input_ceiling: 100,
            verified_output_ceiling: 5,
            verified_total_nano_usd_ceiling: 500,
            rate_card_revision: "test-fixed-price/v1".into(),
            input_nano_usd_per_token: 4,
            output_nano_usd_per_token: 20,
        };
        let now = Utc::now();
        let prefix = format!("{:012x}", now.timestamp_millis());
        let request_id = format!("{}-{}-7abc-8abc-0123456789ab", &prefix[..8], &prefix[8..]);
        let result = reserve(&pool, &config, &token, &request_id, [3; 32]).await;
        sqlx::query("DELETE FROM routing_operations WHERE account_id = $1")
            .bind(&account)
            .execute(&pool)
            .await
            .unwrap();
        sqlx::query("DELETE FROM routing_account_month WHERE account_id = $1")
            .bind(&account)
            .execute(&pool)
            .await
            .unwrap();
        sqlx::query("DELETE FROM routing_account_gate WHERE account_id = $1")
            .bind(&account)
            .execute(&pool)
            .await
            .unwrap();
        sqlx::query("DELETE FROM routing_session_tokens WHERE account_id = $1")
            .bind(&account)
            .execute(&pool)
            .await
            .unwrap();
        sqlx::query("UPDATE routing_funding SET enabled = FALSE, ceiling_nano_usd = 0, spent_nano_usd = 0, held_nano_usd = 0 WHERE singleton = TRUE")
            .execute(&pool).await.unwrap();
        assert_eq!(result.err(), Some(AdmissionError::Unauthorized));
    }

    #[tokio::test]
    async fn postgres_reserve_claim_revoke_and_settle_are_atomic() {
        let Ok(url) = std::env::var("PYTXO_LINK_TEST_DATABASE_URL") else {
            return;
        };
        let parsed = reqwest::Url::parse(&url).unwrap();
        assert_eq!(parsed.host_str(), Some("127.0.0.1"));
        assert!(parsed.path().ends_with("/pytxo_routing_test"));
        let pool = crate::db::connect(&url).await.unwrap();
        let second_pool = PgPool::connect(&url).await.unwrap();
        let mut secret = [0u8; 32];
        getrandom::getrandom(&mut secret).unwrap();
        let token = format!("pr1_{}", URL_SAFE_NO_PAD.encode(secret));
        let digest = token_hash(&token);
        let account = format!("user_routing_test_{}", hex::encode(&secret[..8]));
        let workspace = &secret[..16];
        sqlx::query("INSERT INTO routing_workspace_grants (account_id, workspace_id, recipient_identity, scope_digest, revision, enabled) VALUES ($1,$2,$3,$4,1,TRUE)")
            .bind(&account).bind(workspace).bind(HOSTED_RECIPIENT)
            .bind(hosted_scope_digest().as_slice()).execute(&pool).await.unwrap();
        sqlx::query(
            "INSERT INTO routing_session_tokens (token_hash, account_id, scope, expires_at, workspace_id, grant_revision, recipient_identity, scope_digest) VALUES ($1,$2,$3,now()+interval '5 minutes',$4,1,$5,$6)",
        )
        .bind(digest.as_slice()).bind(&account).bind(TOKEN_SCOPE)
        .bind(workspace).bind(HOSTED_RECIPIENT).bind(hosted_scope_digest().as_slice())
        .execute(&pool).await.unwrap();
        sqlx::query(
            "UPDATE routing_funding SET enabled = TRUE, ceiling_nano_usd = 2000,     spent_nano_usd = 0, held_nano_usd = 0, disabled_reason = NULL WHERE singleton = TRUE",
        )
        .execute(&pool).await.unwrap();
        let config = RoutingAdmissionConfig {
            service_key_digest: [1; 32],
            payload_mac_key: [2; 32],
            verified_input_ceiling: 100,
            verified_output_ceiling: 5,
            verified_total_nano_usd_ceiling: 500,
            rate_card_revision: "test-fixed-price/v1".into(),
            input_nano_usd_per_token: 4,
            output_nano_usd_per_token: 20,
        };
        let now = Utc::now();
        let prefix = format!("{:012x}", now.timestamp_millis());
        let request = |suffix: u8| {
            format!(
                "{}-{}-7abc-8abc-0123456789a{:x}",
                &prefix[..8],
                &prefix[8..],
                suffix
            )
        };
        let request_id = request(1);
        let (first, duplicate) = tokio::join!(
            reserve(&pool, &config, &token, &request_id, [3; 32]),
            reserve(&second_pool, &config, &token, &request_id, [3; 32]),
        );
        let (first, duplicate) = (first.unwrap(), duplicate.unwrap());
        assert_ne!(first.new_reservation, duplicate.new_reservation);
        assert_eq!(
            reserve(&pool, &config, &token, &request_id, [4; 32])
                .await
                .err(),
            Some(AdmissionError::Conflict)
        );
        let held: i64 = sqlx::query_scalar("SELECT held_nano_usd FROM routing_funding")
            .fetch_one(&pool)
            .await
            .unwrap();
        assert_eq!(held, 500);
        let (claim_a, claim_b) = tokio::join!(
            claim(&pool, &config, &token, &request_id),
            claim(&second_pool, &config, &token, &request_id),
        );
        let (claim_a, claim_b) = (claim_a.unwrap(), claim_b.unwrap());
        let owner = claim_a.owner_nonce.or(claim_b.owner_nonce).unwrap();
        assert_eq!(
            claim(&pool, &config, &token, &request_id)
                .await
                .unwrap()
                .owner_nonce,
            None
        );
        let settlement = SettleBody {
            request_id: request_id.clone(),
            owner_nonce: owner.clone(),
            actual_input_tokens: 80,
            actual_output_tokens: 4,
            actual_nano_usd: 400,
            rate_card_revision: "test-fixed-price/v1".into(),
            receipt_id: "test-receipt-1".into(),
            recovery: test_recovery(),
        };
        let mut wrong_cost = settlement.clone();
        wrong_cost.actual_nano_usd = 399;
        assert_eq!(
            settle(&pool, &wrong_cost).await.err(),
            Some(AdmissionError::Conflict)
        );
        assert_eq!(settle(&pool, &settlement).await.unwrap().state, "completed");
        assert_eq!(settle(&pool, &settlement).await.unwrap().state, "completed");
        let recovered = reserve(&pool, &config, &token, &request_id, [3; 32])
            .await
            .unwrap();
        assert_eq!(recovered.state, "completed");
        assert_eq!(recovered.recovery, Some(test_recovery()));
        // A different workspace on the same account must not recover this
        // still-live ciphertext, even with the same request and payload.
        let mut other_workspace = [0u8; 16];
        getrandom::getrandom(&mut other_workspace).unwrap();
        let mut other_secret = [0u8; 32];
        getrandom::getrandom(&mut other_secret).unwrap();
        let other_token = format!("pr1_{}", URL_SAFE_NO_PAD.encode(other_secret));
        sqlx::query("INSERT INTO routing_workspace_grants (account_id, workspace_id, recipient_identity, scope_digest, revision, enabled) VALUES ($1,$2,$3,$4,1,TRUE)")
            .bind(&account).bind(other_workspace.as_slice()).bind(HOSTED_RECIPIENT)
            .bind(hosted_scope_digest().as_slice()).execute(&pool).await.unwrap();
        sqlx::query("INSERT INTO routing_session_tokens (token_hash, account_id, scope, expires_at, workspace_id, grant_revision, recipient_identity, scope_digest) VALUES ($1,$2,$3,now()+interval '5 minutes',$4,1,$5,$6)")
            .bind(token_hash(&other_token).as_slice()).bind(&account).bind(TOKEN_SCOPE)
            .bind(other_workspace.as_slice()).bind(HOSTED_RECIPIENT)
            .bind(hosted_scope_digest().as_slice()).execute(&pool).await.unwrap();
        assert_eq!(
            reserve(&pool, &config, &other_token, &request_id, [3; 32])
                .await
                .err(),
            Some(AdmissionError::Conflict)
        );
        let mut rotated_secret = [0u8; 32];
        getrandom::getrandom(&mut rotated_secret).unwrap();
        let rotated_token = format!("pr1_{}", URL_SAFE_NO_PAD.encode(rotated_secret));
        sqlx::query("UPDATE routing_session_tokens SET token_hash = $2 WHERE token_hash = $1")
            .bind(digest.as_slice())
            .bind(token_hash(&rotated_token).as_slice())
            .execute(&pool)
            .await
            .unwrap();
        assert_eq!(
            reserve(&pool, &config, &rotated_token, &request_id, [3; 32])
                .await
                .unwrap()
                .recovery,
            Some(test_recovery())
        );
        sqlx::query("UPDATE routing_session_tokens SET token_hash = $2 WHERE token_hash = $1")
            .bind(token_hash(&rotated_token).as_slice())
            .bind(digest.as_slice())
            .execute(&pool)
            .await
            .unwrap();
        sqlx::query("UPDATE routing_funding SET enabled = FALSE WHERE singleton = TRUE")
            .execute(&pool)
            .await
            .unwrap();
        let mut changed_bounds = config.clone();
        changed_bounds.verified_input_ceiling = 0;
        assert_eq!(
            reserve(&pool, &changed_bounds, &token, &request_id, [3; 32])
                .await
                .unwrap()
                .recovery,
            Some(test_recovery())
        );
        sqlx::query("UPDATE routing_funding SET enabled = TRUE WHERE singleton = TRUE")
            .execute(&pool)
            .await
            .unwrap();
        let mut changed_recovery = settlement.clone();
        changed_recovery.recovery.ciphertext = URL_SAFE_NO_PAD.encode([7u8; 32]);
        assert_eq!(
            settle(&pool, &changed_recovery).await.err(),
            Some(AdmissionError::Conflict)
        );
        sqlx::query("UPDATE routing_operations SET recovery_expires_at = clock_timestamp() - interval '1 second' WHERE account_id = $1 AND request_id = $2")
            .bind(&account).bind(&request_id).execute(&pool).await.unwrap();
        let expired = reserve(&pool, &config, &token, &request_id, [3; 32])
            .await
            .unwrap();
        assert_eq!(expired.state, "completed");
        assert!(expired.recovery.is_none());
        reconcile_stale(&pool).await.unwrap();
        let stored_ciphertext: Option<Vec<u8>> = sqlx::query_scalar("SELECT recovery_ciphertext FROM routing_operations WHERE account_id = $1 AND request_id = $2")
            .bind(&account).bind(&request_id).fetch_one(&pool).await.unwrap();
        assert!(stored_ciphertext.is_none());
        let (spent, held): (i64, i64) =
            sqlx::query_as("SELECT spent_nano_usd, held_nano_usd FROM routing_funding")
                .fetch_one(&pool)
                .await
                .unwrap();
        assert_eq!((spent, held), (400, 0));

        let rotate_id = request(2);
        assert!(
            reserve(&pool, &config, &token, &rotate_id, [5; 32])
                .await
                .unwrap()
                .new_reservation
        );
        let mut next_secret = [0u8; 32];
        getrandom::getrandom(&mut next_secret).unwrap();
        let next_token = format!("pr1_{}", URL_SAFE_NO_PAD.encode(next_secret));
        sqlx::query(
            "UPDATE routing_session_tokens SET token_hash = $3, created_at = now() WHERE account_id = $1 AND workspace_id = $2",
        )
        .bind(&account).bind(workspace).bind(token_hash(&next_token).as_slice())
        .execute(&pool).await.unwrap();
        assert_eq!(
            claim(&pool, &config, &token, &rotate_id)
                .await
                .unwrap()
                .state,
            "rejected_before_send"
        );
        let (spent, held): (i64, i64) =
            sqlx::query_as("SELECT spent_nano_usd, held_nano_usd FROM routing_funding")
                .fetch_one(&pool)
                .await
                .unwrap();
        assert_eq!((spent, held), (400, 0));
        sqlx::query("UPDATE routing_session_tokens SET revoked_at = now() WHERE account_id = $1 AND workspace_id = $2")
            .bind(&account)
            .bind(workspace)
            .execute(&pool)
            .await
            .unwrap();
        assert_eq!(
            reserve(&pool, &config, &next_token, &request(3), [6; 32])
                .await
                .err(),
            Some(AdmissionError::Unauthorized)
        );
        sqlx::query("UPDATE routing_session_tokens SET revoked_at = NULL WHERE account_id = $1 AND workspace_id = $2")
            .bind(&account)
            .bind(workspace)
            .execute(&pool)
            .await
            .unwrap();
        let stale_reserved = request(4);
        reserve(&pool, &config, &next_token, &stale_reserved, [7; 32])
            .await
            .unwrap();
        sqlx::query(
            "UPDATE routing_operations SET created_at = now() - interval '3 minutes' WHERE account_id = $1 AND request_id = $2",
        )
        .bind(&account).bind(&stale_reserved).execute(&pool).await.unwrap();
        assert_eq!(reconcile_stale(&pool).await.unwrap(), 1);
        assert_eq!(
            claim(&pool, &config, &next_token, &stale_reserved)
                .await
                .unwrap()
                .state,
            "rejected_before_send"
        );
        let uncertain_id = request(5);
        reserve(&pool, &config, &next_token, &uncertain_id, [8; 32])
            .await
            .unwrap();
        let uncertain_owner = claim(&pool, &config, &next_token, &uncertain_id)
            .await
            .unwrap()
            .owner_nonce
            .unwrap();
        assert_eq!(
            mark_uncertain(&pool, &uncertain_id, &uncertain_owner)
                .await
                .unwrap()
                .state,
            "uncertain"
        );
        sqlx::query(
            "UPDATE routing_operations SET send_claimed_at = now() - interval '11 minutes' WHERE account_id = $1 AND request_id = $2",
        )
        .bind(&account).bind(&uncertain_id).execute(&pool).await.unwrap();
        assert_eq!(reconcile_stale(&pool).await.unwrap(), 1);
        let state: String = sqlx::query_scalar(
            "SELECT state FROM routing_operations WHERE account_id = $1 AND request_id = $2",
        )
        .bind(&account)
        .bind(&uncertain_id)
        .fetch_one(&pool)
        .await
        .unwrap();
        assert_eq!(state, "settled_at_ceiling");
        let late = SettleBody {
            request_id: uncertain_id.clone(),
            owner_nonce: uncertain_owner,
            actual_input_tokens: 110,
            actual_output_tokens: 8,
            actual_nano_usd: 600,
            rate_card_revision: "test-fixed-price/v1".into(),
            receipt_id: "test-late-attributable-receipt".into(),
            recovery: test_recovery(),
        };
        assert_eq!(settle(&pool, &late).await.unwrap().state, "completed");
        let (enabled, spent, held): (bool, i64, i64) =
            sqlx::query_as("SELECT enabled, spent_nano_usd, held_nano_usd FROM routing_funding")
                .fetch_one(&pool)
                .await
                .unwrap();
        assert_eq!((enabled, spent, held), (false, 1000, 0));

        // A claim queued behind the funding lock must compare token expiry
        // with post-lock database wall time, not transaction-start time.
        sqlx::query("UPDATE routing_funding SET enabled = TRUE WHERE singleton = TRUE")
            .execute(&pool)
            .await
            .unwrap();
        let expiring_id = request(6);
        reserve(&pool, &config, &next_token, &expiring_id, [9; 32])
            .await
            .unwrap();
        let mut hold = pool.begin().await.unwrap();
        sqlx::query("SELECT singleton FROM routing_funding WHERE singleton = TRUE FOR UPDATE")
            .fetch_one(&mut *hold)
            .await
            .unwrap();
        sqlx::query(
            "UPDATE routing_session_tokens SET expires_at = clock_timestamp() + interval '250 milliseconds' WHERE account_id = $1 AND workspace_id = $2",
        )
        .bind(&account).bind(workspace).execute(&second_pool).await.unwrap();
        let claim_pool = second_pool.clone();
        let claim_config = config.clone();
        let claim_token = next_token.clone();
        let claim_id = expiring_id.clone();
        let waiting = tokio::spawn(async move {
            claim(&claim_pool, &claim_config, &claim_token, &claim_id).await
        });
        tokio::time::sleep(std::time::Duration::from_millis(400)).await;
        hold.commit().await.unwrap();
        assert_eq!(
            waiting.await.unwrap().unwrap().state,
            "rejected_before_send"
        );
        sqlx::query(
            "UPDATE routing_session_tokens SET expires_at = clock_timestamp() + interval '5 minutes' WHERE account_id = $1 AND workspace_id = $2",
        )
        .bind(&account).bind(workspace).execute(&pool).await.unwrap();
        // A grant revoked while claim waits on its row wins the send barrier.
        let revoking_id = request(7);
        reserve(&pool, &config, &next_token, &revoking_id, [10; 32])
            .await
            .unwrap();
        let mut grant_hold = pool.begin().await.unwrap();
        sqlx::query("UPDATE routing_workspace_grants SET enabled = FALSE, revision = 2 WHERE account_id = $1 AND workspace_id = $2 AND recipient_identity = $3")
            .bind(&account).bind(workspace).bind(HOSTED_RECIPIENT)
            .execute(&mut *grant_hold).await.unwrap();
        let waiting_pool = second_pool.clone();
        let waiting_config = config.clone();
        let waiting_token = next_token.clone();
        let waiting_id = revoking_id.clone();
        let waiting_claim = tokio::spawn(async move {
            claim(&waiting_pool, &waiting_config, &waiting_token, &waiting_id).await
        });
        tokio::time::sleep(std::time::Duration::from_millis(100)).await;
        assert!(!waiting_claim.is_finished());
        grant_hold.commit().await.unwrap();
        assert_eq!(
            waiting_claim.await.unwrap().unwrap().state,
            "rejected_before_send"
        );
        assert_eq!(
            reserve(&pool, &config, &next_token, &request_id, [3; 32])
                .await
                .err(),
            Some(AdmissionError::Unauthorized),
            "revocation must also stop completed-answer recovery"
        );
        assert_eq!(
            reserve(&pool, &config, &other_token, &uncertain_id, [8; 32])
                .await
                .err(),
            Some(AdmissionError::Conflict),
            "another active workspace cannot recover a live completed answer after revocation"
        );
        sqlx::query("UPDATE routing_workspace_grants SET enabled = TRUE, revision = 3 WHERE account_id = $1 AND workspace_id = $2 AND recipient_identity = $3")
            .bind(&account).bind(workspace).bind(HOSTED_RECIPIENT)
            .execute(&pool).await.unwrap();
        assert_eq!(
            reserve(&pool, &config, &next_token, &request_id, [3; 32])
                .await
                .err(),
            Some(AdmissionError::Unauthorized)
        );
        sqlx::query("UPDATE routing_session_tokens SET grant_revision = 3 WHERE account_id = $1 AND workspace_id = $2")
            .bind(&account).bind(workspace).execute(&pool).await.unwrap();
        assert_eq!(
            reserve(&pool, &config, &next_token, &uncertain_id, [8; 32])
                .await
                .err(),
            Some(AdmissionError::Conflict),
            "a regrant cannot recover an older grant revision's answer"
        );
        sqlx::query(
            "INSERT INTO routing_operations (account_id, request_id, token_hash, payload_mac, mac_revision, state, utc_month, rate_card_revision, reserved_input_tokens, reserved_output_tokens, reserved_nano_usd, input_nano_usd_per_token, output_nano_usd_per_token, created_at, settled_at) SELECT $1, 'rolling-seed-' || n::text, $2, $3, 'test', 'rejected_before_send', date_trunc('month', clock_timestamp() AT TIME ZONE 'UTC')::date, 'test-fixed-price/v1', 100, 5, 500, 4, 20, clock_timestamp() - interval '40 seconds', clock_timestamp() FROM generate_series(1,10) AS n",
        )
        .bind(&account).bind(token_hash(&next_token).as_slice()).bind([1u8; 32].as_slice())
        .execute(&pool).await.unwrap();
        assert_eq!(
            reserve(&pool, &config, &next_token, &request(8), [11; 32])
                .await
                .err(),
            Some(AdmissionError::Limited)
        );

        sqlx::query("DELETE FROM routing_operations WHERE account_id = $1")
            .bind(&account)
            .execute(&pool)
            .await
            .unwrap();
        sqlx::query("DELETE FROM routing_account_month WHERE account_id = $1")
            .bind(&account)
            .execute(&pool)
            .await
            .unwrap();
        sqlx::query("DELETE FROM routing_account_gate WHERE account_id = $1")
            .bind(&account)
            .execute(&pool)
            .await
            .unwrap();
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
        sqlx::query(
            "UPDATE routing_funding SET enabled = FALSE, ceiling_nano_usd = 0,     spent_nano_usd = 0, held_nano_usd = 0 WHERE singleton = TRUE",
        ).execute(&pool).await.unwrap();
    }
}
