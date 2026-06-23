//! Provider usage extraction and Link reporting (Phase 41).

use serde_json::Value;

#[derive(Clone, Debug, Default)]
pub struct ProviderUsage {
    pub tokens_in: u64,
    pub tokens_out: u64,
    pub model: Option<String>,
}

pub fn parse_usage_json(body: &[u8], provider: &str) -> Option<ProviderUsage> {
    let v: Value = serde_json::from_slice(body).ok()?;
    match provider {
        "anthropic" | "agy" => {
            let usage = v.get("usage")?;
            Some(ProviderUsage {
                tokens_in: usage.get("input_tokens")?.as_u64()?,
                tokens_out: usage.get("output_tokens")?.as_u64()?,
                model: v
                    .get("model")
                    .and_then(|m| m.as_str())
                    .map(str::to_string),
            })
        }
        "openai" | "deepseek" | "openrouter" => {
            let usage = v.get("usage")?;
            Some(ProviderUsage {
                tokens_in: usage.get("prompt_tokens")?.as_u64()?,
                tokens_out: usage.get("completion_tokens")?.as_u64()?,
                model: v
                    .get("model")
                    .and_then(|m| m.as_str())
                    .map(str::to_string),
            })
        }
        "google" => {
            let usage = v
                .pointer("/usageMetadata")
                .or_else(|| v.get("usageMetadata"))?;
            Some(ProviderUsage {
                tokens_in: usage.get("promptTokenCount")?.as_u64()?,
                tokens_out: usage.get("candidatesTokenCount")?.as_u64()?,
                model: None,
            })
        }
        _ => None,
    }
}

pub async fn report_to_link(
    client: &reqwest::Client,
    link_base: &str,
    bearer: &str,
    provider: &str,
    usage: &ProviderUsage,
    domain_id: Option<&str>,
    run_id: Option<&str>,
) {
    let url = format!(
        "{}/v1/inference/usage",
        link_base.trim_end_matches('/')
    );
    let body = serde_json::json!({
        "provider": provider,
        "model": usage.model,
        "tokens_in": usage.tokens_in,
        "tokens_out": usage.tokens_out,
        "domain_id": domain_id,
        "run_id": run_id,
        "cost_micro_usd": 0,
    });
    let mut req = client
        .post(&url)
        .header("Authorization", format!("Bearer {bearer}"))
        .json(&body);
    if let Ok(uid) = std::env::var("PYTXO_ULTRA_USER_ID") {
        if !uid.is_empty() {
            req = req.header("x-pytxo-user-id", uid);
        }
    }
    if let Err(e) = req.send().await {
        tracing::warn!(error = %e, "failed to report inference usage to Link");
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_anthropic_usage() {
        let body = br#"{"model":"claude-3","usage":{"input_tokens":10,"output_tokens":5}}"#;
        let u = parse_usage_json(body, "anthropic").unwrap();
        assert_eq!(u.tokens_in, 10);
        assert_eq!(u.tokens_out, 5);
    }

    #[test]
    fn parses_openai_usage() {
        let body = br#"{"model":"gpt-4","usage":{"prompt_tokens":20,"completion_tokens":8}}"#;
        let u = parse_usage_json(body, "openai").unwrap();
        assert_eq!(u.tokens_in, 20);
        assert_eq!(u.tokens_out, 8);
    }
}
