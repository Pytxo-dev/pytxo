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

    pub async fn validate(&self, token: &str) -> Option<ClerkClaims> {
        let header = decode_header(token).ok()?;
        let kid = header.kid?;
        self.ensure_keys().await.ok()?;
        let key = {
            let cache = self.cache.lock().unwrap();
            cache.keys.get(&kid).cloned()?
        };
        let mut validation = Validation::new(Algorithm::RS256);
        validation.set_issuer(&[self.issuer.as_str()]);
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
