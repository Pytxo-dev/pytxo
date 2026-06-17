use serde::{Deserialize, Serialize};

use crate::cloud::CloudConfig;
use crate::{PytxoError, Result};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct CacheLookup {
    pub domain_id: String,
    pub path: String,
    pub fidelity: String,
    pub content_hash: String,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct CachedScaffold {
    pub content: String,
    pub scaffolded_bytes: usize,
    pub language: Option<String>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct CachePut {
    pub domain_id: String,
    pub path: String,
    pub fidelity: String,
    pub content_hash: String,
    pub content: String,
    pub scaffolded_bytes: usize,
    pub language: Option<String>,
}

pub trait ContextCache: Send + Sync {
    fn get(&self, key: &CacheLookup) -> Result<Option<CachedScaffold>>;
    fn put(&self, entry: &CachePut) -> Result<()>;
}

#[derive(Clone, Debug, Default)]
pub struct NoopContextCache;

impl ContextCache for NoopContextCache {
    fn get(&self, _: &CacheLookup) -> Result<Option<CachedScaffold>> {
        Ok(None)
    }
    fn put(&self, _: &CachePut) -> Result<()> {
        Ok(())
    }
}

#[derive(Clone, Debug)]
pub struct HttpContextCache {
    pub base_url: String,
}

impl HttpContextCache {
    pub fn from_config(cfg: &CloudConfig) -> Self {
        let base = if cfg.cache_url.trim().is_empty() {
            cfg.sandbox_url.as_str()
        } else {
            cfg.cache_url.as_str()
        };
        Self {
            base_url: base.trim_end_matches('/').to_string(),
        }
    }

    fn auth_headers(&self) -> Vec<(String, String)> {
        let mut h = vec![];
        if let Ok(token) = std::env::var("PYTXO_CLOUD_SESSION") {
            if !token.is_empty() {
                h.push(("Authorization".to_string(), format!("Bearer {token}")));
            }
        }
        h
    }
}

impl ContextCache for HttpContextCache {
    fn get(&self, key: &CacheLookup) -> Result<Option<CachedScaffold>> {
        #[cfg(feature = "cloud-http")]
        {
            let url = format!(
                "{}/cache/scaffold?domain_id={}&path={}&fidelity={}&content_hash={}",
                self.base_url,
                urlencoding_key(&key.domain_id),
                urlencoding_key(&key.path),
                urlencoding_key(&key.fidelity),
                urlencoding_key(&key.content_hash),
            );
            let agent = ureq::AgentBuilder::new().build();
            let mut req = agent.get(&url);
            for (k, v) in self.auth_headers() {
                req = req.set(&k, &v);
            }
            match req.call() {
                Ok(resp) if (200..300).contains(&resp.status()) => {
                    let text = resp.into_string().unwrap_or_default();
                    let hit: CachedScaffold = serde_json::from_str(&text)
                        .map_err(|e| PytxoError::Other(format!("cache parse: {e}")))?;
                    Ok(Some(hit))
                }
                Ok(resp) if resp.status() == 404 => Ok(None),
                Ok(resp) => Err(PytxoError::Other(format!(
                    "cache get: status {}",
                    resp.status()
                ))),
                Err(e) => Err(PytxoError::Other(format!("cache get: {e}"))),
            }
        }
        #[cfg(not(feature = "cloud-http"))]
        {
            let _ = key;
            Ok(None)
        }
    }

    fn put(&self, entry: &CachePut) -> Result<()> {
        #[cfg(feature = "cloud-http")]
        {
            let url = format!("{}/cache/scaffold", self.base_url);
            let agent = ureq::AgentBuilder::new().build();
            let mut req = agent.put(&url).set("Content-Type", "application/json");
            for (k, v) in self.auth_headers() {
                req = req.set(&k, &v);
            }
            let resp = req
                .send_json(entry)
                .map_err(|e| PytxoError::Other(format!("cache put: {e}")))?;
            if !(200..300).contains(&resp.status()) {
                return Err(PytxoError::Other(format!(
                    "cache put: status {}",
                    resp.status()
                )));
            }
        }
        #[cfg(not(feature = "cloud-http"))]
        {
            let _ = entry;
        }
        Ok(())
    }
}

fn urlencoding_key(s: &str) -> String {
    s.chars()
        .map(|c| match c {
            'A'..='Z' | 'a'..='z' | '0'..='9' | '-' | '_' | '.' | '~' => c.to_string(),
            _ => format!("%{:02X}", c as u8),
        })
        .collect()
}

pub fn content_hash(bytes: &[u8]) -> String {
    use std::collections::hash_map::DefaultHasher;
    use std::hash::{Hash, Hasher};
    let mut h = DefaultHasher::new();
    bytes.hash(&mut h);
    format!("{:016x}", h.finish())
}
