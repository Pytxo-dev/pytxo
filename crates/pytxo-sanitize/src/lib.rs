use regex::Regex;
use std::sync::LazyLock;

static RULES: LazyLock<Vec<(Regex, &'static str)>> = LazyLock::new(|| {
    vec![
        (
            Regex::new(r"(?i)sk-[a-zA-Z0-9]{20,}").expect("sk pattern"),
            "[REDACTED_API_KEY]",
        ),
        (
            Regex::new(r"(?i)bearer\s+[a-zA-Z0-9._\-]+").expect("bearer"),
            "Bearer [REDACTED]",
        ),
        (
            Regex::new(r"(?i)(api[_-]?key|token)\s*[:=]\s*\S+").expect("api key"),
            "$1=[REDACTED]",
        ),
        (
            Regex::new(r"[A-Za-z]:\\Users\\[^\\s]+").expect("win path"),
            "[REDACTED_PATH]",
        ),
        (
            Regex::new(r"/home/[^/\s]+").expect("unix home"),
            "[REDACTED_PATH]",
        ),
    ]
});

pub fn sanitize_line(input: &str) -> String {
    let mut out = input.to_string();
    for (re, replacement) in RULES.iter() {
        out = re.replace_all(&out, *replacement).to_string();
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn redacts_openai_style_key() {
        let s = "error: sk-abcdefghijklmnopqrstuvwxyz1234567890";
        let out = sanitize_line(s);
        assert!(!out.contains("sk-abcdefghijklmnopqrst"));
        assert!(out.contains("[REDACTED_API_KEY]"));
    }

    #[test]
    fn redacts_bearer() {
        let s = "Authorization: Bearer eyJhbGciOiJIUzI1NiJ9";
        let out = sanitize_line(s);
        assert!(!out.contains("eyJhbGci"));
    }
}
