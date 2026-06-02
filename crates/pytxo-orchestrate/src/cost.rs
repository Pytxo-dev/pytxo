use regex::Regex;
use std::sync::LazyLock;

static TOKEN_IN: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"(?i)(?:input|prompt)\s*tokens?\s*[:=]\s*(\d+)").unwrap());
static TOKEN_OUT: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"(?i)(?:output|completion)\s*tokens?\s*[:=]\s*(\d+)").unwrap());
static COST_USD: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"(?i)\$\s*([0-9]+(?:\.[0-9]+)?)").unwrap());

#[derive(Clone, Debug, Default)]
pub struct CostEstimate {
    pub tokens_in: i64,
    pub tokens_out: i64,
    pub cost_usd: f64,
}

pub fn parse_cost_from_lines(lines: &[&str]) -> CostEstimate {
    let mut est = CostEstimate::default();
    for line in lines {
        if let Some(c) = TOKEN_IN.captures(line) {
            if let Ok(n) = c[1].parse::<i64>() {
                est.tokens_in = est.tokens_in.max(n);
            }
        }
        if let Some(c) = TOKEN_OUT.captures(line) {
            if let Ok(n) = c[1].parse::<i64>() {
                est.tokens_out = est.tokens_out.max(n);
            }
        }
        if let Some(c) = COST_USD.captures(line) {
            if let Ok(n) = c[1].parse::<f64>() {
                est.cost_usd = est.cost_usd.max(n);
            }
        }
    }
    est
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_token_lines() {
        let lines = ["input tokens: 1200", "output tokens: 340"];
        let est = parse_cost_from_lines(&lines);
        assert_eq!(est.tokens_in, 1200);
        assert_eq!(est.tokens_out, 340);
    }
}
