use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

use jsonwebtoken::{decode, decode_header, Algorithm, DecodingKey, Validation};
use serde::Deserialize;

#[derive(Clone)]
pub struct JwksValidator {
    jwks_url: String,
    issuer: String,
    cache: Arc<Mutex<JwksCache>>,
}

struct JwksCache {
    keys: HashMap<String, DecodingKey>,
    fetched_at: Option<Instant>,
}

#[derive(Debug, Deserialize)]
struct JwksResponse {
    keys: Vec<JwkKey>,
}

#[derive(Debug, Deserialize)]
struct JwkKey {
    kid: Option<String>,
    kty: String,
    n: Option<String>,
    e: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct ClerkClaims {
    pub sub: String,
    pub sid: Option<String>,
    pub exp: i64,
}

impl JwksValidator {
    pub fn new(jwks_url: String, issuer: String) -> Self {
        Self {
            jwks_url,
            issuer,
            cache: Arc::new(Mutex::new(JwksCache {
                keys: HashMap::new(),
                fetched_at: None,
            })),
        }
    }

    #[cfg(test)]
    pub(crate) fn seed_key_for_test(&self, kid: &str, key: DecodingKey) {
        let mut cache = self.cache.lock().unwrap();
        cache.keys.insert(kid.into(), key);
        cache.fetched_at = Some(Instant::now());
    }

    pub async fn validate(&self, token: &str) -> Option<ClerkClaims> {
        self.validate_with_audience(token, None).await
    }

    /// Sponsored routing only accepts a real signed account session for the
    /// configured audience; the permissive general Link path is not reused.
    pub async fn validate_for_routing(&self, token: &str, audience: &str) -> Option<ClerkClaims> {
        if !self.routing_source_secure() || audience.trim().is_empty() || token.len() > 8192 {
            return None;
        }
        let claims = tokio::time::timeout(
            Duration::from_secs(3),
            self.validate_with_audience(token, Some(audience)),
        )
        .await
        .ok()??;
        if !claims.sub.starts_with("user_")
            || claims.sub.len() <= 5
            || !claims
                .sid
                .as_deref()
                .is_some_and(|sid| sid.starts_with("sess_") && sid.len() > 5)
        {
            return None;
        }
        Some(claims)
    }

    pub fn routing_source_secure(&self) -> bool {
        [&self.jwks_url, &self.issuer].into_iter().all(|value| {
            reqwest::Url::parse(value)
                .ok()
                .is_some_and(|url| url.scheme() == "https" && url.host_str().is_some())
        })
    }

    async fn validate_with_audience(
        &self,
        token: &str,
        audience: Option<&str>,
    ) -> Option<ClerkClaims> {
        let header = decode_header(token).ok()?;
        if header.alg != Algorithm::RS256 {
            return None;
        }
        let kid = header.kid?;
        self.ensure_keys().await.ok()?;
        let key = {
            let cache = self.cache.lock().unwrap();
            cache.keys.get(&kid).cloned()?
        };
        let mut validation = Validation::new(Algorithm::RS256);
        validation.set_issuer(&[self.issuer.as_str()]);
        if let Some(audience) = audience {
            validation.leeway = 0;
            validation.set_audience(&[audience]);
            validation.set_required_spec_claims(&["exp", "iss", "aud", "sub", "sid"]);
        }
        let data = decode::<ClerkClaims>(token, &key, &validation).ok()?;
        Some(data.claims)
    }

    async fn ensure_keys(&self) -> Result<(), String> {
        let stale = {
            let cache = self.cache.lock().unwrap();
            cache
                .fetched_at
                .map(|t| t.elapsed() > Duration::from_secs(3600))
                .unwrap_or(true)
        };
        if !stale {
            return Ok(());
        }
        let resp = reqwest::get(&self.jwks_url)
            .await
            .map_err(|e| e.to_string())?
            .json::<JwksResponse>()
            .await
            .map_err(|e| e.to_string())?;
        let mut keys = HashMap::new();
        for jwk in resp.keys {
            if jwk.kty != "RSA" {
                continue;
            }
            let kid = jwk.kid.unwrap_or_default();
            if kid.is_empty() {
                continue;
            }
            let n = jwk.n.ok_or_else(|| "missing n".to_string())?;
            let e = jwk.e.ok_or_else(|| "missing e".to_string())?;
            let key = DecodingKey::from_rsa_components(&n, &e).map_err(|e| e.to_string())?;
            keys.insert(kid, key);
        }
        let mut cache = self.cache.lock().unwrap();
        cache.keys = keys;
        cache.fetched_at = Some(Instant::now());
        Ok(())
    }
}

#[cfg(test)]
mod routing_tests {
    use super::*;
    use base64::engine::general_purpose::URL_SAFE_NO_PAD;
    use base64::Engine;
    use jsonwebtoken::{encode, EncodingKey, Header};
    use rand::rngs::OsRng;
    use rsa::pkcs1::EncodeRsaPrivateKey;
    use rsa::traits::PublicKeyParts;
    use rsa::RsaPrivateKey;
    use serde_json::{json, Value};
    use std::time::{SystemTime, UNIX_EPOCH};

    #[tokio::test]
    async fn routing_requires_signed_session_issuer_audience_expiry_and_identity() {
        let private = RsaPrivateKey::new(&mut OsRng, 2048).unwrap();
        let public = private.to_public_key();
        let n = URL_SAFE_NO_PAD.encode(public.n().to_bytes_be());
        let e = URL_SAFE_NO_PAD.encode(public.e().to_bytes_be());
        let key = DecodingKey::from_rsa_components(&n, &e).unwrap();
        let validator = JwksValidator::new(
            "https://unused-local-jwks.test/keys".into(),
            "https://issuer.test".into(),
        );
        assert!(validator.routing_source_secure());
        assert!(
            !JwksValidator::new("http://jwks.test".into(), "https://issuer.test".into())
                .routing_source_secure()
        );
        {
            let mut cache = validator.cache.lock().unwrap();
            cache.keys.insert("local-test".into(), key);
            cache.fetched_at = Some(Instant::now());
        }
        let der = private.to_pkcs1_der().unwrap();
        let encoding_key = EncodingKey::from_rsa_der(der.as_bytes());
        let mut header = Header::new(Algorithm::RS256);
        header.kid = Some("local-test".into());
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();
        let valid = json!({
            "iss": "https://issuer.test",
            "aud": "pytxo-routing",
            "exp": now + 120,
            "sub": "user_123",
            "sid": "sess_123"
        });
        let sign = |claims: &Value| encode(&header, claims, &encoding_key).unwrap();
        assert_eq!(
            validator
                .validate_for_routing(&sign(&valid), "pytxo-routing")
                .await
                .unwrap()
                .sub,
            "user_123"
        );
        for (field, replacement) in [
            ("iss", json!("https://other.test")),
            ("aud", json!("other-service")),
            ("exp", json!(now - 1)),
            ("sub", json!("api-key")),
            ("sub", json!("")),
            ("sid", json!("")),
        ] {
            let mut altered = valid.clone();
            altered[field] = replacement;
            assert!(
                validator
                    .validate_for_routing(&sign(&altered), "pytxo-routing")
                    .await
                    .is_none(),
                "field {field} must be checked"
            );
        }
        let mut missing_sid = valid.clone();
        missing_sid.as_object_mut().unwrap().remove("sid");
        assert!(validator
            .validate_for_routing(&sign(&missing_sid), "pytxo-routing")
            .await
            .is_none());
        assert!(validator
            .validate_for_routing("shared-key", "pytxo-routing")
            .await
            .is_none());
    }
}
