//! Shared `/health` body parsing for Pytxo HTTP services (ADR-0027).

/// Returns true when the body is legacy plain `ok` or JSON `{"status":"ok",...}`.
pub fn response_ok(body: &str) -> bool {
    let trimmed = body.trim();
    if trimmed == "ok" {
        return true;
    }
    if let Ok(v) = serde_json::from_str::<serde_json::Value>(trimmed) {
        if let Some(status) = v.get("status").and_then(|s| s.as_str()) {
            return status == "ok";
        }
    }
    false
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn accepts_plain_ok() {
        assert!(response_ok("ok"));
    }

    #[test]
    fn accepts_json_contract() {
        assert!(response_ok(r#"{"status":"ok","uptime_secs":42,"service":"pytxo-link"}"#));
    }

    #[test]
    fn rejects_error_body() {
        assert!(!response_ok(r#"{"status":"degraded"}"#));
    }
}
