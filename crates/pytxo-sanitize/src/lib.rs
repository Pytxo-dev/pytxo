use regex::Regex;
use std::sync::LazyLock;

static RULES: LazyLock<Vec<(Regex, &'static str)>> = LazyLock::new(|| {
    vec![
        // Anthropic API keys and Claude subscription OAuth tokens
        // (`sk-ant-api03-…`, `sk-ant-oat01-…`) carry dashes the generic rule misses.
        (
            Regex::new(r"sk-ant-[a-z0-9]+-[A-Za-z0-9_\-]{16,}").expect("anthropic key"),
            "[REDACTED_API_KEY]",
        ),
        (
            Regex::new(r"(?i)sk-[a-zA-Z0-9]{20,}").expect("sk pattern"),
            "[REDACTED_API_KEY]",
        ),
        (
            Regex::new(r"(?i)bearer\s+[a-zA-Z0-9._\-]+").expect("bearer"),
            "Bearer [REDACTED]",
        ),
        // Vendor token files and OAuth replies (Codex `auth.json`, refresh responses).
        (
            Regex::new(r#"(?i)"((?:access|refresh|id)_token)"\s*:\s*"[^"]*""#).expect("json token"),
            r#""$1":"[REDACTED]""#,
        ),
        (
            Regex::new(r"eyJ[A-Za-z0-9_\-]{8,}\.[A-Za-z0-9_\-]{8,}\.[A-Za-z0-9_\-]{8,}")
                .expect("jwt"),
            "[REDACTED_TOKEN]",
        ),
        (
            Regex::new(r"(?i)(api[_-]?key|token)\s*[:=]\s*\S+").expect("api key"),
            "$1=[REDACTED]",
        ),
        // Any drive, any case, either separator, and JSON-escaped runs
        // (`C:\Users\…`, `c:/users/…`, `C:\\Users\\…`). The name stops at `"`,
        // which no OS allows in user names, so JSON around the path survives.
        (
            Regex::new(r#"(?i)[a-z]:[\\/]+users[\\/]+[^\\/\s"]+"#).expect("win path"),
            "[REDACTED_PATH]",
        ),
        // Case-sensitive, so URL paths such as `/users/<login>` survive.
        (
            Regex::new(r#"/Users/[^/\s"]+"#).expect("mac home"),
            "[REDACTED_PATH]",
        ),
        (
            Regex::new(r#"/home/[^/\s"]+"#).expect("unix home"),
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
    fn redacts_vendor_cli_credentials() {
        for (line, secret) in [
            (
                "key sk-ant-api03-AbCdEfGhIjKlMnOpQrStUv_wx-yz",
                "AbCdEfGhIjKlMnOp",
            ),
            (
                "CLAUDE_CODE_OAUTH_TOKEN sk-ant-oat01-ZyXwVuTsRqPoNmLkJi-12",
                "ZyXwVuTsRqPoNm",
            ),
            (r#"{"refresh_token": "rt_9f8e7d6c5b4a"}"#, "rt_9f8e7d6c5b4a"),
            (
                "id eyJhbGciOiJSUzI1NiJ9.eyJlbWFpbCI6ImFAYi5jIn0.c2lnbmF0dXJlLXZhbHVl",
                "eyJlbWFpbCI6",
            ),
        ] {
            let out = sanitize_line(line);
            assert!(!out.contains(secret), "{line} -> {out}");
        }
    }

    #[test]
    fn redacts_windows_profile_paths_for_any_user_name() {
        // The old class `[^\\s]` also stopped at the letter `s`.
        let out = sanitize_line(r"C:\Users\sam\.pytxo\agent-accounts\claude\work");
        assert!(!out.contains("sam"), "{out}");
    }

    #[test]
    fn redacts_user_paths_in_any_separator_case_or_os_form() {
        for (line, expected) in [
            // Node, Git and many agents print Windows paths with forward slashes.
            ("C:/Users/sam/fw/app.ts", "[REDACTED_PATH]/fw/app.ts"),
            (r"C:\Users/sam/fw", "[REDACTED_PATH]/fw"),
            (r"C:/Users\sam\fw", r"[REDACTED_PATH]\fw"),
            (r"d:\users\sam\fw", r"[REDACTED_PATH]\fw"),
            ("E:/USERS/sam", "[REDACTED_PATH]"),
            // PTY output keeps its color codes.
            (
                "\x1b[36mC:/Users/sam/fw\x1b[0m",
                "\x1b[36m[REDACTED_PATH]/fw\x1b[0m",
            ),
            // JSON doubles each backslash; the closing quote must survive.
            (
                r#"{"cwd":"C:\\Users\\sam\\fw"}"#,
                r#"{"cwd":"[REDACTED_PATH]\\fw"}"#,
            ),
            (
                r#"{"home":"C:/Users/sam","n":1}"#,
                r#"{"home":"[REDACTED_PATH]","n":1}"#,
            ),
            ("/Users/sam/fw/app.ts", "[REDACTED_PATH]/fw/app.ts"),
            (
                r#"{"cwd":"/home/sam","n":1}"#,
                r#"{"cwd":"[REDACTED_PATH]","n":1}"#,
            ),
        ] {
            assert_eq!(sanitize_line(line), expected, "{line}");
        }
    }

    #[test]
    fn keeps_paths_without_a_user_segment() {
        for line in [
            "C:/tools/node/node.exe",
            r"C:\Program Files\Git\bin\git.exe",
            "C:/UsersData/cache",
            r"dir C:\Users",
            "/usr/local/bin/cargo",
            "https://api.github.com/users/octocat",
        ] {
            assert_eq!(sanitize_line(line), line);
        }
    }

    #[test]
    fn redacts_bearer() {
        let s = "Authorization: Bearer eyJhbGciOiJIUzI1NiJ9";
        let out = sanitize_line(s);
        assert!(!out.contains("eyJhbGci"));
    }
}
