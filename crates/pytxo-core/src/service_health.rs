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

/// Provider ids listed in proxy health JSON `providers_configured` (empty if absent or invalid).
pub fn providers_configured(body: &str) -> Vec<String> {
    let Ok(v) = serde_json::from_str::<serde_json::Value>(body.trim()) else {
        return Vec::new();
    };
    v.get("providers_configured")
        .and_then(|a| a.as_array())
        .map(|arr| {
            arr.iter()
                .filter_map(|x| x.as_str().map(str::to_string))
                .collect()
        })
        .unwrap_or_default()
}

/// True when `provider` appears in `providers_configured` on a health JSON body.
pub fn health_lists_provider(body: &str, provider: &str) -> bool {
    providers_configured(body)
        .iter()
        .any(|p| p == provider)
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

    #[test]
    fn parses_providers_configured() {
        let body = r#"{"status":"ok","providers_configured":["deepseek","openai"]}"#;
        assert!(health_lists_provider(body, "deepseek"));
        assert!(!health_lists_provider(body, "anthropic"));
    }
}
