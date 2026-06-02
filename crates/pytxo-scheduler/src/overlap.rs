use pytxo_core::{ConflictPair, Task};

/// Two path patterns overlap if they could touch the same file.
/// Conservative rules: exact match, one is prefix of another, or glob bases collide.
pub fn paths_overlap(a: &str, b: &str) -> bool {
    let a = normalize_pattern(a);
    let b = normalize_pattern(b);
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
    p.replace('\\', "/").trim_start_matches("./").to_string()
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
}
