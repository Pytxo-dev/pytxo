//! Failure classifier v1 ([[closed-loop-fidelity]]).
//!
//! Maps an agent's failure output (rustc/cargo/test diagnostics and similar) to
//! the subset of its task path patterns that the failure actually implicates, so
//! the closed loop re-scaffolds only those files at higher fidelity instead of the
//! whole task surface.

use pytxo_core::normalize_claim_path;

/// Source-file extensions worth treating as implicated diagnostics.
const CODE_EXTS: &[&str] = &[
    "rs", "ts", "tsx", "js", "jsx", "py", "go", "java", "rb", "c", "h", "cpp", "hpp", "cc", "cs",
    "swift", "kt", "scala", "php", "lua", "sql", "toml", "json", "yaml", "yml",
];

/// Return the subset of `candidates` (task/agent path patterns) that the failure
/// output implicates. Empty means "no confident subset" — the caller should fall
/// back to escalating all candidate paths.
pub fn implicated_paths(output: &str, candidates: &[String]) -> Vec<String> {
    if candidates.is_empty() {
        return Vec::new();
    }
    let mentions = extract_file_mentions(output);
    if mentions.is_empty() {
        return Vec::new();
    }
    let mut out: Vec<String> = Vec::new();
    for cand in candidates {
        if mentions.iter().any(|m| mention_matches_pattern(m, cand)) && !out.contains(cand) {
            out.push(cand.clone());
        }
    }
    out
}

/// Pull file-like tokens out of compiler/test output. Handles rustc `-->` markers,
/// `path:line:col` forms, and bare paths ending in a known code extension.
fn extract_file_mentions(output: &str) -> Vec<String> {
    let mut mentions = Vec::new();
    for raw_token in output.split(|c: char| c.is_whitespace() || c == '(' || c == ')' || c == '"') {
        let token = raw_token.trim_matches(|c: char| matches!(c, ',' | '`' | '\'' | '[' | ']'));
        if token.is_empty() {
            continue;
        }
        // Strip a trailing :line:col (or :line) suffix from diagnostics.
        let path_part = strip_line_col(token);
        let normalized = normalize_claim_path(path_part);
        if has_code_ext(&normalized) {
            mentions.push(normalized);
        }
    }
    mentions
}

fn strip_line_col(token: &str) -> &str {
    let mut end = token.len();
    let mut seen = 0;
    // Peel up to two trailing ":<digits>" groups.
    while seen < 2 {
        let head = &token[..end];
        match head.rfind(':') {
            Some(idx)
                if idx + 1 < head.len() && head[idx + 1..].bytes().all(|b| b.is_ascii_digit()) =>
            {
                end = idx;
                seen += 1;
            }
            _ => break,
        }
    }
    &token[..end]
}

fn has_code_ext(path: &str) -> bool {
    match path.rsplit_once('.') {
        Some((_, ext)) => CODE_EXTS.contains(&ext.to_ascii_lowercase().as_str()),
        None => false,
    }
}

/// True when a mentioned file is covered by a candidate path pattern.
fn mention_matches_pattern(mention: &str, pattern: &str) -> bool {
    let pat = normalize_claim_path(pattern);
    if mention == pat {
        return true;
    }
    let base = pat
        .trim_end_matches("/**")
        .trim_end_matches("/*")
        .trim_end_matches('*')
        .trim_end_matches('/');
    if base.is_empty() {
        return false;
    }
    // Glob/dir patterns: the mention lives under the base. Exact-file patterns:
    // tolerate the mention being a suffix of the pattern path or vice versa.
    mention.starts_with(base) || mention.ends_with(&pat) || pat.ends_with(mention)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rustc_marker_implicates_single_file() {
        let out = "error[E0308]: mismatched types\n  --> src/parser.rs:42:9\n";
        let candidates = vec!["src/parser.rs".to_string(), "src/lexer.rs".to_string()];
        assert_eq!(implicated_paths(out, &candidates), vec!["src/parser.rs"]);
    }

    #[test]
    fn glob_pattern_matches_file_under_dir() {
        let out = "FAIL tests/api/users.ts:10";
        let candidates = vec!["tests/**".to_string(), "src/**".to_string()];
        assert_eq!(implicated_paths(out, &candidates), vec!["tests/**"]);
    }

    #[test]
    fn no_recognizable_paths_returns_empty() {
        let out = "thread 'main' panicked at internal error";
        let candidates = vec!["src/lib.rs".to_string()];
        assert!(implicated_paths(out, &candidates).is_empty());
    }

    #[test]
    fn windows_separators_are_normalized() {
        let out = "error: see src\\db\\schema.rs:3:1 for details";
        let candidates = vec!["src/db/schema.rs".to_string()];
        assert_eq!(implicated_paths(out, &candidates), vec!["src/db/schema.rs"]);
    }
}
