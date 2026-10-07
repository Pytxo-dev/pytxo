use pytxo_core::{ConflictPair, Task};

/// Two path patterns overlap if they could touch the same file.
/// Conservative rules: exact match, one is prefix of another, or glob bases collide.
pub fn paths_overlap(a: &str, b: &str) -> bool {
    let a = normalize_pattern(a);
    let b = normalize_pattern(b);
    // Win32 path equality is case-insensitive by default. Its non-ASCII
    // upcase table is not Rust's Unicode case fold. Dot-only components and
    // possible short-name aliases also need filesystem resolution, so these
    // uncertain claims cannot share a wave.
    #[cfg(windows)]
    if [a.as_str(), b.as_str()].iter().any(|path| {
        !path.is_ascii()
            || path.split('/').any(|segment| {
                segment.is_empty()
                    || segment.contains(':')
                    || segment.trim_end_matches(['.', ' ']).is_empty()
                    || segment
                        .as_bytes()
                        .windows(2)
                        .any(|pair| pair[0] == b'~' && pair[1].is_ascii_digit())
            })
    }) {
        return true;
    }
    if a == b {
        return true;
    }
    if a.starts_with(&b) || b.starts_with(&a) {
        return true;
    }
    // package.json vs package.json / src/** vs src/a.ts
    let a_base = strip_glob(&a);
    let b_base = strip_glob(&b);
    a_base == b_base || a_base.starts_with(&b_base) || b_base.starts_with(&a_base)
}

fn normalize_pattern(p: &str) -> String {
    let normalized = p.replace('\\', "/");
    let normalized = normalized.trim_start_matches("./");
    #[cfg(windows)]
    {
        normalized
            .split('/')
            .map(|segment| segment.trim_end_matches(['.', ' ']).to_ascii_lowercase())
            .collect::<Vec<_>>()
            .join("/")
    }
    #[cfg(not(windows))]
    {
        normalized.to_string()
    }
}

fn strip_glob(p: &str) -> String {
    p.trim_end_matches("/**")
        .trim_end_matches("/*")
        .trim_end_matches('*')
        .to_string()
}

pub fn tasks_overlap(a: &Task, b: &Task) -> bool {
    for pa in &a.paths {
        for pb in &b.paths {
            if paths_overlap(pa, pb) {
                return true;
            }
        }
    }
    false
}

pub fn shared_paths(a: &Task, b: &Task) -> Vec<String> {
    let mut out = Vec::new();
    for pa in &a.paths {
        for pb in &b.paths {
            if paths_overlap(pa, pb) {
                let label = if pa == pb {
                    pa.clone()
                } else {
                    format!("{pa} ~ {pb}")
                };
                if !out.contains(&label) {
                    out.push(label);
                }
            }
        }
    }
    out
}

pub fn find_conflicts(tasks: &[Task]) -> Vec<ConflictPair> {
    let mut conflicts = Vec::new();
    for i in 0..tasks.len() {
        for j in (i + 1)..tasks.len() {
            if tasks_overlap(&tasks[i], &tasks[j]) {
                conflicts.push(ConflictPair {
                    task_a: tasks[i].id.clone(),
                    task_b: tasks[j].id.clone(),
                    paths: shared_paths(&tasks[i], &tasks[j]),
                });
            }
        }
    }
    conflicts
}

pub fn find_cross_root_conflicts(tasks: &[Task]) -> Vec<ConflictPair> {
    let mut conflicts = Vec::new();
    for i in 0..tasks.len() {
        for j in (i + 1)..tasks.len() {
            let root_a = tasks[i].root.as_deref().unwrap_or("");
            let root_b = tasks[j].root.as_deref().unwrap_or("");
            if root_a == root_b || root_a.is_empty() || root_b.is_empty() {
                continue;
            }
            if tasks_overlap(&tasks[i], &tasks[j]) {
                conflicts.push(ConflictPair {
                    task_a: tasks[i].id.clone(),
                    task_b: tasks[j].id.clone(),
                    paths: shared_paths(&tasks[i], &tasks[j]),
                });
            }
        }
    }
    conflicts
}

#[cfg(test)]
mod tests {
    use super::*;
    use pytxo_core::TaskId;

    fn task(id: &str, paths: &[&str]) -> Task {
        Task {
            id: TaskId(id.to_string()),
            agent: "builder".into(),
            paths: paths.iter().map(|s| (*s).to_string()).collect(),
            depends_on: Vec::new(),
            root: None,
            signal_fidelity: None,
            verify: vec![],
        }
    }

    #[test]
    fn package_json_conflicts() {
        let a = task("a", &["package.json"]);
        let b = task("b", &["package.json"]);
        assert!(tasks_overlap(&a, &b));
    }

    #[test]
    fn disjoint_paths_ok() {
        let a = task("a", &["src/a.ts"]);
        let b = task("b", &["src/b.ts"]);
        assert!(!tasks_overlap(&a, &b));
    }

    #[test]
    #[cfg(windows)]
    fn windows_case_aliases_cannot_share_a_wave() {
        let a = task("a", &["src/Foo.rs"]);
        let b = task("b", &["src/foo.rs"]);
        assert!(tasks_overlap(&a, &b));
        assert!(paths_overlap("./SRC\\Foo.rs", "src/foo.rs"));
        assert!(paths_overlap("src/Foo.rs.", "src/foo.rs"));
        assert!(paths_overlap("src/Dir.\\file.rs", "src/dir/file.rs"));
        assert!(paths_overlap("src/Dir \\file.rs", "src/dir/file.rs"));
        assert!(paths_overlap("src/./foo.rs", "src/foo.rs"));
        assert!(paths_overlap("C:src\\foo.rs", "src/foo.rs"));
    }

    #[test]
    #[cfg(windows)]
    fn unresolved_unicode_claims_do_not_run_concurrently() {
        assert!(paths_overlap("src/Ä.rs", "docs/readme.md"));
    }
}
