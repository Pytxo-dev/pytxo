//! Structural telemetry graph IPC (Phase 27).

use std::path::{Path, PathBuf};
use std::process::Command;

use pytxo_signal::StructuralGraph;

use crate::resolve_repo_root;

const MAX_WORKSPACE_FILES: usize = 200;

/// Build a Signal structural graph for an active or recent run.
pub fn structural_graph(
    repo: Option<PathBuf>,
    run_id: &str,
) -> anyhow::Result<StructuralGraph> {
    let repo_root = resolve_repo_root(repo.as_deref())?;
    let (_, store) = crate::open_store(None, Some(repo_root.clone()))?;
    let paths = store.arbitrage_paths_for_run(run_id)?;
    let agents = store.list_agents_for_run(run_id)?;
    let root_by_agent: std::collections::HashMap<String, Option<String>> = agents
        .into_iter()
        .map(|a| (a.id, a.root_id))
        .collect();
    let edited: Vec<(String, String, Option<String>)> = paths
        .into_iter()
        .map(|(path, agent_id)| {
            let root_id = root_by_agent.get(&agent_id).cloned().flatten();
            (path, agent_id, root_id)
        })
        .collect();
    let g = pytxo_signal::build_structural_graph(&repo_root, &edited);
    Ok(pytxo_signal::enrich_symbol_nodes(g, &repo_root))
}

/// Preview import graph for the workspace before any agent run (Reality Deck idle state).
pub fn workspace_structural_graph(repo: Option<PathBuf>) -> anyhow::Result<StructuralGraph> {
    let repo_root = resolve_repo_root(repo.as_deref())?;
    let paths = collect_workspace_source_files(&repo_root)?;
    let edited: Vec<(String, String, Option<String>)> = paths
        .into_iter()
        .map(|p| (p, String::new(), None))
        .collect();
    let mut g = pytxo_signal::build_structural_graph(&repo_root, &edited);
    for node in &mut g.nodes {
        node.edited = false;
    }
    Ok(g)
}

fn collect_workspace_source_files(repo_root: &Path) -> anyhow::Result<Vec<String>> {
    if let Ok(out) = Command::new("git")
        .args(["ls-files", "*.rs", "*.ts", "*.tsx", "*.js", "*.jsx"])
        .current_dir(repo_root)
        .output()
    {
        if out.status.success() {
            let files: Vec<String> = String::from_utf8_lossy(&out.stdout)
                .lines()
                .filter(|l| !l.is_empty())
                .take(MAX_WORKSPACE_FILES)
                .map(str::to_string)
                .collect();
            if !files.is_empty() {
                return Ok(files);
            }
        }
    }

    let mut found = Vec::new();
    for sub in ["src", "crates", "apps", "services"] {
        let base = repo_root.join(sub);
        if !base.is_dir() {
            continue;
        }
        walk_source_dir(&base, repo_root, &mut found);
        if found.len() >= MAX_WORKSPACE_FILES {
            break;
        }
    }
    found.truncate(MAX_WORKSPACE_FILES);
    Ok(found)
}

fn walk_source_dir(dir: &Path, repo_root: &Path, out: &mut Vec<String>) {
    if out.len() >= MAX_WORKSPACE_FILES {
        return;
    }
    let entries = match std::fs::read_dir(dir) {
        Ok(e) => e,
        Err(_) => return,
    };
    for entry in entries.flatten() {
        if out.len() >= MAX_WORKSPACE_FILES {
            return;
        }
        let path = entry.path();
        if path.is_dir() {
            if path.file_name().is_some_and(|n| n == "node_modules" || n == "target" || n == ".git")
            {
                continue;
            }
            walk_source_dir(&path, repo_root, out);
        } else if is_source_file(&path) {
            if let Ok(rel) = path.strip_prefix(repo_root) {
                out.push(rel.to_string_lossy().replace('\\', "/"));
            }
        }
    }
}

fn is_source_file(path: &Path) -> bool {
    matches!(
        path.extension().and_then(|e| e.to_str()),
        Some("rs" | "ts" | "tsx" | "js" | "jsx")
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn is_source_file_recognizes_rust() {
        assert!(is_source_file(Path::new("foo.rs")));
        assert!(!is_source_file(Path::new("foo.md")));
    }
}
