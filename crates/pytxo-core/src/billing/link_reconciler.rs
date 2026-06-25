use serde_json::{json, Value};

use crate::billing::{BillingReconciler, RunUsageTotals};
use crate::moat::DomainId;
use crate::{PytxoError, Result, RunId};

/// HTTP Pytxo Link reconciler. Local dev uses [`NoopBillingReconciler`].
///
/// The Link service itself lives in a **separate private repo** ([[github-organization]]);
/// this client builds the request envelopes (endpoint + JSON body). The actual
/// transport is attached when the Link service is deployed — the monorepo stays
/// network-free and offline-testable.
#[derive(Clone, Debug)]
pub struct HttpBillingReconciler {
    pub base_url: String,
}

impl HttpBillingReconciler {
    pub fn new(base_url: impl Into<String>) -> Self {
        Self {
            base_url: base_url.into(),
        }
    }

    /// Build a fully-qualified endpoint, joining without duplicate slashes.
    pub fn endpoint(&self, path: &str) -> String {
        format!(
            "{}/{}",
            self.base_url.trim_end_matches('/'),
            path.trim_start_matches('/')
        )
    }

    /// JSON body for `POST /runs/start`.
    pub fn run_start_body(&self, domain_id: &DomainId, run_id: &RunId) -> Value {
        json!({
            "domain_id": domain_id.as_str(),
            "run_id": run_id.0,
            "idempotency_key": format!("{}:{}", domain_id.as_str(), run_id.0),
        })
    }

    /// JSON body for `POST /runs/end` (usage reconciliation).
    pub fn run_end_body(
        &self,
        domain_id: &DomainId,
        run_id: &RunId,
        totals: &RunUsageTotals,
    ) -> Value {
        json!({
            "domain_id": domain_id.as_str(),
            "run_id": run_id.0,
            "idempotency_key": format!("{}:{}", domain_id.as_str(), run_id.0),
            "usage": {
                "tokens_in_billed": totals.tokens_in_billed,
                "tokens_in_sent": totals.tokens_in_sent,
                "tokens_out": totals.tokens_out,
                "saved_tokens": totals.saved_tokens,
                "cost_micro_usd": totals.cost_micro_usd,
            }
        })
    }

    /// POST JSON to the Link service. Without the `link-http` feature this is a
    /// ping-only stub so the monorepo stays offline-testable by default.
    fn send(&self, endpoint: &str, body: &Value) -> Result<()> {
        self.ping()?;
        #[cfg(feature = "link-http")]
        {
            let mut req = ureq::post(endpoint).set("Content-Type", "application/json");
            if let Ok(token) = std::env::var("PYTXO_ULTRA_SESSION") {
                if !token.is_empty() {
                    req = req.set("Authorization", &format!("Bearer {token}"));
                }
            }
            let resp = req
                .send_json(body.clone())
                .map_err(|e| PytxoError::Other(format!("link http: {e}")))?;
            let status = resp.status();
            if !(200..300).contains(&status) {
                return Err(PytxoError::Other(format!(
                    "link http {endpoint}: status {status}"
                )));
            }
        }
        #[cfg(not(feature = "link-http"))]
        {
            let _ = (endpoint, body);
        }
        Ok(())
    }

    pub fn ping(&self) -> Result<()> {
        if self.base_url.trim().is_empty() {
            return Err(PytxoError::Other("link base_url empty".into()));
        }
        #[cfg(feature = "link-http")]
        {
            let health = self.endpoint("health");
            let resp = ureq::get(&health)
                .call()
                .map_err(|e| PytxoError::Other(format!("link health: {e}")))?;
            if resp.status() != 200 {
                return Err(PytxoError::Other(format!(
                    "link health {}: status {}",
                    health,
                    resp.status()
                )));
            }
            let body = resp
                .into_string()
                .map_err(|e| PytxoError::Other(format!("link health body: {e}")))?;
            if !crate::service_health_ok(&body) {
                return Err(PytxoError::Other(format!(
                    "link health unexpected body: {body}"
                )));
            }
        }
        Ok(())
    }
}

impl BillingReconciler for HttpBillingReconciler {
    fn reconcile_run_start(&self, domain_id: &DomainId, run_id: &RunId) -> Result<()> {
        let endpoint = self.endpoint("v1/runs/start");
        let body = self.run_start_body(domain_id, run_id);
        self.send(&endpoint, &body)
    }

    fn reconcile_run_end(
        &self,
        domain_id: &DomainId,
        run_id: &RunId,
        totals: &RunUsageTotals,
    ) -> Result<()> {
        let endpoint = self.endpoint("v1/runs/end");
        let body = self.run_end_body(domain_id, run_id, totals);
        self.send(&endpoint, &body)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ping_requires_base_url() {
        let bad = HttpBillingReconciler::new("");
        assert!(bad.ping().is_err());
    }

    #[test]
    fn endpoint_joins_without_double_slash() {
        let r = HttpBillingReconciler::new("https://link.pytxo.com/");
        assert_eq!(
            r.endpoint("v1/runs/start"),
            "https://link.pytxo.com/v1/runs/start"
        );
        assert_eq!(
            r.endpoint("v1/runs/end"),
            "https://link.pytxo.com/v1/runs/end"
        );
    }

    #[test]
    fn bodies_have_expected_shape() {
        let r = HttpBillingReconciler::new("https://link.pytxo.com");
        let domain = DomainId("/repo/a".into());
        let run = RunId("run-123".into());
        let start = r.run_start_body(&domain, &run);
        assert_eq!(start["domain_id"], "/repo/a");
        assert_eq!(start["run_id"], "run-123");
        assert_eq!(start["idempotency_key"], "/repo/a:run-123");

        let totals = RunUsageTotals {
            tokens_in_billed: 100,
            tokens_in_sent: 60,
            tokens_out: 40,
            saved_tokens: 40,
            cost_micro_usd: 1234,
        };
        let end = r.run_end_body(&domain, &run, &totals);
        assert_eq!(end["usage"]["tokens_in_billed"], 100);
        assert_eq!(end["usage"]["saved_tokens"], 40);
        assert_eq!(end["usage"]["cost_micro_usd"], 1234);
    }

    #[cfg(feature = "link-http")]
    #[test]
    fn http_send_hits_mock_server() {
        use std::io::{Read, Write};
        use std::net::TcpListener;
        use std::sync::atomic::{AtomicBool, Ordering};
        use std::sync::Arc;
        use std::thread;
        use std::time::{Duration, Instant};

        let hit = Arc::new(AtomicBool::new(false));
        let hit2 = Arc::clone(&hit);
        let listener = TcpListener::bind("127.0.0.1:0").unwrap();
        let port = listener.local_addr().unwrap().port();
        let base = format!("http://127.0.0.1:{port}");
        let server = thread::spawn(move || {
            listener.set_nonblocking(true).ok();
            let deadline = Instant::now() + Duration::from_secs(2);
            let mut requests = 0u32;
            while Instant::now() < deadline && requests < 2 {
                if let Ok((mut stream, _)) = listener.accept() {
                    let mut buf = vec![0u8; 8192];
                    let n = stream.read(&mut buf).unwrap_or(0);
                    if n > 0 {
                        let req = String::from_utf8_lossy(&buf[..n]);
                        requests += 1;
                        if req.contains("v1/runs/start") && req.contains("run-xyz") {
                            hit2.store(true, Ordering::SeqCst);
                        }
                        let resp = if req.contains("/health") {
                            "HTTP/1.1 200 OK\r\nContent-Length: 2\r\n\r\nok"
                        } else {
                            "HTTP/1.1 200 OK\r\nContent-Length: 0\r\n\r\n"
                        };
                        let _ = stream.write_all(resp.as_bytes());
                    }
                }
                thread::sleep(Duration::from_millis(10));
            }
        });

        let r = HttpBillingReconciler::new(&base);
        let domain = DomainId("/repo/mock".into());
        let run = RunId("run-xyz".into());
        r.reconcile_run_start(&domain, &run).unwrap();
        server.join().unwrap();
        assert!(hit.load(Ordering::SeqCst));
    }
}
