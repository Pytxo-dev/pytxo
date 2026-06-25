//! Provider usage extraction and Link reporting (Phase 41/52).

use axum::http::HeaderMap;
use pytxo_core::{ModelId, StaticPriceTable, TokenCounts};
use serde_json::Value;

#[derive(Clone, Debug, Default)]
pub struct ProviderUsage {
    pub tokens_in: u64,
    pub tokens_out: u64,
    pub model: Option<String>,
}

pub fn parse_usage_from_response(
    headers: &HeaderMap,
    body: &[u8],
    provider: &str,
) -> Option<ProviderUsage> {
    parse_usage_json(body, provider).or_else(|| parse_usage_headers(headers, provider))
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

/// Best-effort token hints from provider rate-limit headers.
pub fn parse_usage_headers(headers: &HeaderMap, provider: &str) -> Option<ProviderUsage> {
    match provider {
        "anthropic" | "agy" => {
            let limit = header_u64(headers, "anthropic-ratelimit-input-tokens-limit")?;
            let remaining = header_u64(headers, "anthropic-ratelimit-input-tokens-remaining")?;
            let out_limit = header_u64(headers, "anthropic-ratelimit-output-tokens-limit");
            let out_remaining =
                header_u64(headers, "anthropic-ratelimit-output-tokens-remaining");
            Some(ProviderUsage {
                tokens_in: limit.saturating_sub(remaining),
                tokens_out: out_limit
                    .zip(out_remaining)
                    .map(|(l, r)| l.saturating_sub(r))
                    .unwrap_or(0),
                model: None,
            })
        }
        "openai" | "deepseek" | "openrouter" => {
            let limit = header_u64(headers, "x-ratelimit-limit-tokens")?;
            let remaining = header_u64(headers, "x-ratelimit-remaining-tokens")?;
            Some(ProviderUsage {
                tokens_in: limit.saturating_sub(remaining),
                tokens_out: 0,
                model: None,
            })
        }
        _ => None,
    }
}

fn header_u64(headers: &HeaderMap, name: &str) -> Option<u64> {
    headers
        .get(name)
        .and_then(|v| v.to_str().ok())
        .and_then(|s| s.split(',').next())
        .and_then(|s| s.trim().parse().ok())
}

/// Accumulate usage from SSE `data:` lines (streaming completions).
pub fn parse_usage_from_sse(buffer: &[u8], provider: &str) -> Option<ProviderUsage> {
    let text = std::str::from_utf8(buffer).ok()?;
    let mut usage = ProviderUsage::default();
    let mut found = false;

    for line in text.lines() {
        let Some(payload) = line.strip_prefix("data: ") else {
            continue;
        };
        let payload = payload.trim();
        if payload.is_empty() || payload == "[DONE]" {
            continue;
        }
        let v: Value = match serde_json::from_str(payload) {
            Ok(v) => v,
            Err(_) => continue,
        };
        if let Some(parsed) = parse_usage_json(payload.as_bytes(), provider) {
            usage.tokens_in = usage.tokens_in.max(parsed.tokens_in);
            usage.tokens_out = usage.tokens_out.max(parsed.tokens_out);
            if usage.model.is_none() {
                usage.model = parsed.model;
            }
            found = true;
        }
        if let Some(delta) = v.pointer("/usage") {
            if let Some(input) = delta
                .get("input_tokens")
                .or_else(|| delta.get("prompt_tokens"))
                .and_then(|x| x.as_u64())
            {
                usage.tokens_in = usage.tokens_in.max(input);
                found = true;
            }
            if let Some(output) = delta
                .get("output_tokens")
                .or_else(|| delta.get("completion_tokens"))
                .and_then(|x| x.as_u64())
            {
                usage.tokens_out = usage.tokens_out.max(output);
                found = true;
            }
        }
        if let Some(model) = v.get("model").and_then(|m| m.as_str()) {
            usage.model = Some(model.to_string());
        }
    }

    if found {
        Some(usage)
    } else {
        None
    }
}

pub fn cost_micro_usd(usage: &ProviderUsage, provider: &str) -> i64 {
    let model_name = usage
        .model
        .clone()
        .unwrap_or_else(|| default_model_for_provider(provider));
    let model = ModelId::new(model_name);
    let table = StaticPriceTable;
    table.cost_micro_usd(
        &model,
        &TokenCounts {
            tokens_in: usage.tokens_in,
            tokens_out: usage.tokens_out,
        },
    )
}

fn default_model_for_provider(provider: &str) -> String {
    match provider {
        "anthropic" | "agy" => "claude-sonnet-4".into(),
        "openai" => "gpt-4".into(),
        "google" => "gemini-2.0-flash".into(),
        "deepseek" => "deepseek-chat".into(),
        "openrouter" => "openrouter/auto".into(),
        _ => "generic".into(),
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
    let cost = cost_micro_usd(usage, provider);
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
        "cost_micro_usd": cost,
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
    use axum::http::HeaderMap;

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

    #[test]
    fn parses_sse_anthropic_message_delta() {
        let body = concat!(
            "data: {\"type\":\"message_delta\",\"usage\":{\"output_tokens\":7}}\n\n",
            "data: {\"type\":\"message_delta\",\"usage\":{\"input_tokens\":15,\"output_tokens\":9}}\n"
        );
        let u = parse_usage_from_sse(body.as_bytes(), "anthropic").unwrap();
        assert_eq!(u.tokens_in, 15);
        assert_eq!(u.tokens_out, 9);
    }

    #[test]
    fn cost_is_nonzero_for_tokens() {
        let usage = ProviderUsage {
            tokens_in: 1000,
            tokens_out: 500,
            model: Some("claude-sonnet-4".into()),
        };
        assert!(cost_micro_usd(&usage, "anthropic") > 0);
    }

    #[test]
    fn header_fallback_when_body_empty() {
        let mut headers = HeaderMap::new();
        headers.insert(
            "anthropic-ratelimit-input-tokens-limit",
            "1000".parse().unwrap(),
        );
        headers.insert(
            "anthropic-ratelimit-input-tokens-remaining",
            "900".parse().unwrap(),
        );
        let u = parse_usage_headers(&headers, "anthropic").unwrap();
        assert_eq!(u.tokens_in, 100);
    }
}
