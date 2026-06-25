//! Request body size and per-IP rate limits (Phase 56, ADR-0027).

use std::collections::HashMap;
use std::net::SocketAddr;
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

use axum::extract::ConnectInfo;
use axum::http::{Request, StatusCode};
use axum::middleware::Next;
use axum::response::Response;

const DEFAULT_MAX_BODY: usize = 1024 * 1024;
const DEFAULT_RATE_PER_MIN: usize = 120;
const RATE_WINDOW: Duration = Duration::from_secs(60);

#[derive(Clone)]
pub struct RateLimiter {
    hits: Arc<Mutex<HashMap<String, Vec<Instant>>>>,
    max_per_window: usize,
}

impl RateLimiter {
    pub fn from_env() -> Self {
        let max_per_window = std::env::var("LINK_RATE_LIMIT_PER_MIN")
            .ok()
            .and_then(|s| s.parse().ok())
            .unwrap_or(DEFAULT_RATE_PER_MIN);
        Self {
            hits: Arc::new(Mutex::new(HashMap::new())),
            max_per_window,
        }
    }

    fn allow(&self, key: &str) -> bool {
        let now = Instant::now();
        let mut map = self.hits.lock().expect("rate limiter lock");
        let entries = map.entry(key.to_string()).or_default();
        entries.retain(|t| now.duration_since(*t) < RATE_WINDOW);
        if entries.len() >= self.max_per_window {
            return false;
        }
        entries.push(now);
        true
    }
}

pub fn max_body_bytes() -> usize {
    std::env::var("LINK_MAX_BODY_BYTES")
        .ok()
        .and_then(|s| s.parse().ok())
        .unwrap_or(DEFAULT_MAX_BODY)
}

pub async fn rate_limit_middleware(
    ConnectInfo(peer): ConnectInfo<SocketAddr>,
    axum::extract::State(limiter): axum::extract::State<RateLimiter>,
    request: Request<axum::body::Body>,
    next: Next,
) -> Result<Response, StatusCode> {
    let key = peer.ip().to_string();
    if !limiter.allow(&key) {
        return Err(StatusCode::TOO_MANY_REQUESTS);
    }
    Ok(next.run(request).await)
}
