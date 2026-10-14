//! Which named symbols an input changed in one file, from its own merge base.
//! Names (not line numbers) are compared across inputs, so inputs branched
//! from different commits still line up.

use std::collections::BTreeSet;
use std::path::Path;

use pytxo_signal::{innermost_symbol, symbol_spans};

use crate::git::Git;

const MAX_LINES: usize = 50_000;

pub fn touched_symbols(git: &Git, merge_base: &str, commit: &str, path: &str) -> BTreeSet<String> {
    let mut names = BTreeSet::new();
    let Ok(diff) = git.run(&[
        "diff",
        "-U0",
        "--no-renames",
        "--no-color",
        "--no-ext-diff",
        merge_base,
        commit,
        "--",
        path,
    ]) else {
        return names;
    };
    let (old_lines, new_lines) = changed_lines(&diff);
    let file = Path::new(path);
    for (rev, lines) in [(merge_base, old_lines), (commit, new_lines)] {
        if lines.is_empty() {
            continue;
        }
        let Some(source) = git.show(rev, path) else {
            continue;
        };
        let spans = symbol_spans(file, &source);
        if spans.is_empty() {
            continue;
        }
        for line in lines.into_iter().take(MAX_LINES) {
            if let Some(span) = innermost_symbol(&spans, line) {
                names.insert(span.name.clone());
            }
        }
    }
    names
}

/// Old-side and new-side changed line numbers from `-U0` hunk headers.
pub fn changed_lines(diff: &str) -> (Vec<usize>, Vec<usize>) {
    let (mut old, mut new) = (Vec::new(), Vec::new());
    for line in diff.lines().filter(|l| l.starts_with("@@ ")) {
        let mut parts = line.split_whitespace().skip(1);
        let (Some(o), Some(n)) = (parts.next(), parts.next()) else {
            continue;
        };
        if let Some((start, len)) = range(o.trim_start_matches('-')) {
            old.extend(start..start + len);
        }
        if let Some((start, len)) = range(n.trim_start_matches('+')) {
            new.extend(start..start + len);
        }
    }
    (old, new)
}

fn range(spec: &str) -> Option<(usize, usize)> {
    let (start, len) = match spec.split_once(',') {
        Some((s, l)) => (s.parse().ok()?, l.parse().ok()?),
        None => (spec.parse().ok()?, 1),
    };
    (len > 0).then_some((start, len))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn hunk_headers_to_lines() {
        let diff = "diff --git a/x b/x\n@@ -3,2 +3 @@ fn a\n-x\n-y\n+z\n@@ -10,0 +11,2 @@\n+p\n+q\n@@ -20 +22,0 @@\n-r\n";
        let (old, new) = changed_lines(diff);
        assert_eq!(old, [3, 4, 20]);
        assert_eq!(new, [3, 11, 12]);
    }
}
