use regex::{Regex, RegexBuilder};
use std::sync::LazyLock;

static RULES: LazyLock<Vec<(Regex, &'static str)>> = LazyLock::new(|| {
    let mut rules = vec![
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
    ];
    // The generic rules below stop the user name at a space, so the running
    // user's own home goes first, matched whole (`C:\Users\John Smith`).
    // ponytail: only this user's home; another user's profile name with a
    // space still leaks the part after it.
    let home = std::env::var(if cfg!(windows) { "USERPROFILE" } else { "HOME" });
    rules.extend(home.ok().and_then(|home| home_rule(&home, cfg!(windows))));
    rules.extend([
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
    ]);
    rules
});

/// `home` in any separator form (`\`, `/`, mixed, JSON-escaped); a drive home
/// also as Git Bash and WSL print it (`/c/Users/…`, `/mnt/c/Users/…`). `None`
/// for a relative or bare-root home, which would redact unrelated text.
fn home_rule(home: &str, ignore_case: bool) -> Option<(Regex, &'static str)> {
    let segments: Vec<&str> = Regex::new(r"[\\/]+")
        .expect("separators")
        .split(home.trim_end_matches(['\\', '/']))
        .collect();
    // Absolute and below the root: `/home/…` splits off an empty first
    // segment, `C:\Users\…` a drive, and at least one segment follows.
    let [root, _, ..] = segments.as_slice() else {
        return None;
    };
    let drive = match root.as_bytes() {
        [] => None,
        [letter, b':'] if letter.is_ascii_alphabetic() => Some(char::from(*letter)),
        _ => return None,
    };
    let mut parts: Vec<String> = segments.iter().map(|s| regex::escape(s)).collect();
    if let Some(letter) = drive {
        parts[0] = format!(r"(?:{letter}:|[\\/]+{letter})");
    }
    // Not followed by a word character, so a home of `C:\Users\Admin` leaves
    // `C:\Users\Administrator` to the generic rule; `$1` restores that character.
    let pattern = format!(r"{}(\W|$)", parts.join(r"[\\/]+"));
    let rule = RegexBuilder::new(&pattern)
        .case_insensitive(ignore_case)
        .build()
        .ok()?;
    Some((rule, "[REDACTED_PATH]$1"))
}

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

    const SPACED_HOME: &str = r"C:\Users\John Smith";

    fn redact_home(home: &str, ignore_case: bool, line: &str) -> String {
        let (rule, with) = home_rule(home, ignore_case).expect("home rule");
        rule.replace_all(line, with).into_owned()
    }

    #[test]
    fn redacts_own_home_with_a_space_in_any_separator_form() {
        for (line, expected) in [
            (
                r"C:\Users\John Smith\fw\app.ts",
                r"[REDACTED_PATH]\fw\app.ts",
            ),
            ("C:/Users/John Smith/fw", "[REDACTED_PATH]/fw"),
            (r"C:\Users/John Smith\fw", r"[REDACTED_PATH]\fw"),
            (r"c:\users\JOHN SMITH", "[REDACTED_PATH]"),
            (r"\\?\C:\Users\John Smith\fw", r"\\?\[REDACTED_PATH]\fw"),
            ("/c/Users/John Smith/fw", "[REDACTED_PATH]/fw"),
            ("/mnt/c/Users/John Smith/fw", "/mnt[REDACTED_PATH]/fw"),
            (
                r"PS C:\Users\John Smith> cargo test",
                "PS [REDACTED_PATH]> cargo test",
            ),
            // JSON doubles each backslash; the quotes and later fields survive.
            (
                r#"{"cwd":"C:\\Users\\John Smith\\fw","n":1}"#,
                r#"{"cwd":"[REDACTED_PATH]\\fw","n":1}"#,
            ),
            (
                r#"{"home":"C:/Users/John Smith"}"#,
                r#"{"home":"[REDACTED_PATH]"}"#,
            ),
        ] {
            assert_eq!(redact_home(SPACED_HOME, true, line), expected, "{line}");
        }
        assert_eq!(
            redact_home(
                "/home/John Smith/",
                false,
                r#"{"cwd":"/home/John Smith/fw"}"#
            ),
            r#"{"cwd":"[REDACTED_PATH]/fw"}"#
        );
    }

    #[test]
    fn keeps_paths_outside_the_home() {
        for line in [
            r"C:\Users\John Smithson\fw",
            r"D:\Users\John Smith\fw",
            r"C:\Projects\John Smith\fw",
            r"C:\Users\John",
            "John Smith",
        ] {
            assert_eq!(redact_home(SPACED_HOME, true, line), line);
        }
        // Case matters off Windows.
        let line = "/home/john smith/fw";
        assert_eq!(redact_home("/home/John Smith", false, line), line);
    }

    #[test]
    fn skips_relative_or_bare_root_homes() {
        for home in ["", "fw", "~/fw", "/", r"C:\", "C:", r"C:fw\x"] {
            assert!(home_rule(home, true).is_none(), "{home:?}");
        }
    }

    #[test]
    fn redacts_bearer() {
        let s = "Authorization: Bearer eyJhbGciOiJIUzI1NiJ9";
        let out = sanitize_line(s);
        assert!(!out.contains("eyJhbGci"));
    }
}
