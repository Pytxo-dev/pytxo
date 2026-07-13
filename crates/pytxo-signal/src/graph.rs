//! Structural import graph for Reality Deck topology (Phase 27).

use std::collections::{HashMap, HashSet};
use std::path::{Path, PathBuf};

use serde::Serialize;

use crate::language;
#[derive(Clone, Debug, Serialize)]
pub struct GraphNode {
    pub id: String,
    pub label: String,
    pub edited: bool,
    pub root_id: Option<String>,
}

#[derive(Clone, Debug, Serialize)]
pub struct GraphEdge {
    pub from: String,
    pub to: String,
}

#[derive(Clone, Debug, Serialize, Default)]
pub struct StructuralGraph {
    #[serde(default = "default_graph_version")]
    pub version: u32,
    pub nodes: Vec<GraphNode>,
    pub edges: Vec<GraphEdge>,
}

fn default_graph_version() -> u32 {
    2
}

/// Build a lightweight module/file graph from edited paths and on-disk imports.
pub fn build_structural_graph(
    repo_root: &Path,
    edited: &[(String, String, Option<String>)],
) -> StructuralGraph {
    let mut nodes: HashMap<String, GraphNode> = HashMap::new();
    let mut edges: Vec<GraphEdge> = Vec::new();
    let mut seen_edges: HashSet<(String, String)> = HashSet::new();

    for (path, _agent_id, root_id) in edited {
        let rel = normalize_rel(path);
        nodes.entry(rel.clone()).or_insert_with(|| GraphNode {
            id: rel.clone(),
            label: Path::new(&rel)
                .file_name()
                .map(|s| s.to_string_lossy().into_owned())
                .unwrap_or_else(|| rel.clone()),
            edited: true,
            root_id: root_id.clone(),
        });
        let abs = repo_root.join(&rel);
        if let Ok(raw) = std::fs::read_to_string(&abs) {
            let targets = extract_import_targets_tree_sitter(&raw, &abs)
                .unwrap_or_else(|| extract_import_targets_regex(&raw, &rel));
            for target in targets {
                let to = resolve_import(repo_root, &rel, &target);
                nodes.entry(to.clone()).or_insert_with(|| GraphNode {
                    id: to.clone(),
                    label: Path::new(&to)
                        .file_name()
                        .map(|s| s.to_string_lossy().into_owned())
                        .unwrap_or_else(|| to.clone()),
                    edited: false,
                    root_id: root_id.clone(),
                });
                if seen_edges.insert((rel.clone(), to.clone())) {
                    edges.push(GraphEdge {
                        from: rel.clone(),
                        to,
                    });
                }
            }
        }
    }

    StructuralGraph {
        version: default_graph_version(),
        nodes: nodes.into_values().collect(),
        edges,
    }
}

fn module_key(path: &str) -> String {
    let p = Path::new(path);
    p.parent()
        .filter(|parent| !parent.as_os_str().is_empty())
        .map(|parent| normalize_rel(&parent.to_string_lossy()))
        .unwrap_or_else(|| normalize_rel(path))
}

/// Roll file-level nodes up to module directories for Deck topology (Phase 36).
pub fn aggregate_modules(mut graph: StructuralGraph) -> StructuralGraph {
    let mut modules: HashMap<String, GraphNode> = HashMap::new();
    for node in graph.nodes.drain(..) {
        let key = module_key(&node.id);
        let entry = modules.entry(key.clone()).or_insert_with(|| GraphNode {
            id: key.clone(),
            label: Path::new(&key)
                .file_name()
                .map(|s| s.to_string_lossy().into_owned())
                .unwrap_or_else(|| key.clone()),
            edited: false,
            root_id: node.root_id.clone(),
        });
        if node.edited {
            entry.edited = true;
        }
    }
    let mut seen_edges: HashSet<(String, String)> = HashSet::new();
    let mut edges = Vec::new();
    for edge in graph.edges {
        let from = module_key(&edge.from);
        let to = module_key(&edge.to);
        if from != to && seen_edges.insert((from.clone(), to.clone())) {
            edges.push(GraphEdge { from, to });
        }
    }
    StructuralGraph {
        version: default_graph_version(),
        nodes: modules.into_values().collect(),
        edges,
    }
}

/// Build graph and aggregate to module-level nodes.
pub fn build_module_graph(
    repo_root: &Path,
    edited: &[(String, String, Option<String>)],
) -> StructuralGraph {
    aggregate_modules(build_structural_graph(repo_root, edited))
}

/// Emit symbol-level nodes (functions) for edited files (Phase 47).
pub fn enrich_symbol_nodes(mut graph: StructuralGraph, repo_root: &Path) -> StructuralGraph {
    let mut extra_nodes = Vec::new();
    for node in &graph.nodes {
        if !node.edited {
            continue;
        }
        let abs = repo_root.join(&node.id);
        let Ok(raw) = std::fs::read_to_string(&abs) else {
            continue;
        };
        for sym in extract_symbol_names_tree_sitter(&raw, &abs) {
            let id = format!("{}::{sym}", node.id);
            extra_nodes.push(GraphNode {
                id: id.clone(),
                label: sym,
                edited: true,
                root_id: node.root_id.clone(),
            });
        }
    }
    graph.nodes.extend(extra_nodes);
    graph.version = 3;
    graph
}

fn extract_symbol_names_tree_sitter(source: &str, abs_path: &Path) -> Vec<String> {
    let lang = match language::detect(abs_path) {
        Some(l) => l,
        None => return Vec::new(),
    };
    let mut parser = tree_sitter::Parser::new();
    if parser.set_language(&lang.grammar).is_err() {
        return Vec::new();
    }
    let tree = match parser.parse(source, None) {
        Some(t) => t,
        None => return Vec::new(),
    };
    let mut out = Vec::new();
    walk_symbol_nodes(tree.root_node(), source, lang.id, &mut out);
    out
}

fn walk_symbol_nodes(
    node: tree_sitter::Node<'_>,
    source: &str,
    lang_id: &str,
    out: &mut Vec<String>,
) {
    let is_symbol = match lang_id {
        "rust" => matches!(
            node.kind(),
            "function_item" | "struct_item" | "enum_item" | "trait_item"
        ),
        "typescript" | "javascript" => {
            matches!(node.kind(), "function_declaration" | "method_definition")
        }
        _ => false,
    };
    if is_symbol {
        if let Some(name) = node.child_by_field_name("name") {
            if let Ok(s) = name.utf8_text(source.as_bytes()) {
                if !s.is_empty() {
                    out.push(s.to_string());
                }
            }
        }
    }
    for i in 0..node.child_count() {
        if let Some(child) = node.child(i as u32) {
            walk_symbol_nodes(child, source, lang_id, out);
        }
    }
}

/// Neighbor file paths from structural import edges for closed-loop v3 escalation.
pub fn graph_neighbor_paths(
    repo_root: &Path,
    edited: &[(String, String, Option<String>)],
    seeds: &[String],
) -> Vec<String> {
    let graph = build_structural_graph(repo_root, edited);
    let seed_set: HashSet<String> = seeds.iter().map(|s| normalize_rel(s)).collect();
    let mut out: Vec<String> = seeds.to_vec();
    for edge in &graph.edges {
        if seed_set.contains(&edge.from) && !out.contains(&edge.to) {
            out.push(edge.to.clone());
        }
        if seed_set.contains(&edge.to) && !out.contains(&edge.from) {
            out.push(edge.from.clone());
        }
    }
    out
}

fn normalize_rel(path: &str) -> String {
    path.replace('\\', "/").trim_start_matches("./").to_string()
}

fn extract_import_targets_tree_sitter(source: &str, abs_path: &Path) -> Option<Vec<String>> {
    let lang = language::detect(abs_path)?;
    let mut parser = tree_sitter::Parser::new();
    parser.set_language(&lang.grammar).ok()?;
    let tree = parser.parse(source, None)?;
    let mut out = Vec::new();
    walk_import_nodes(tree.root_node(), source, lang.id, &mut out);
    if out.is_empty() {
        None
    } else {
        Some(out)
    }
}

fn walk_import_nodes(
    node: tree_sitter::Node<'_>,
    source: &str,
    lang_id: &str,
    out: &mut Vec<String>,
) {
    match lang_id {
        "rust" if matches!(node.kind(), "use_declaration" | "extern_crate_item") => {
            if let Some(sym) = node
                .child_by_field_name("argument")
                .and_then(|n| n.utf8_text(source.as_bytes()).ok())
            {
                let path = sym.split([':', '{']).next().unwrap_or(sym).trim();
                if !path.is_empty() {
                    out.push(path.replace("::", "/"));
                }
            } else if let Ok(text) = node.utf8_text(source.as_bytes()) {
                if let Some(rest) = text.strip_prefix("use ") {
                    let sym = rest.split([':', ';', '{']).next().unwrap_or(rest).trim();
                    if !sym.is_empty() {
                        out.push(sym.replace("::", "/"));
                    }
                }
            }
        }
        "typescript" | "javascript" if node.kind() == "import_statement" => {
            for i in 0..node.child_count() {
                if let Some(child) = node.child(i as u32) {
                    if child.kind() == "string" {
                        if let Ok(s) = child.utf8_text(source.as_bytes()) {
                            let t = s.trim_matches(['"', '\'', '`']);
                            if !t.is_empty() {
                                out.push(t.to_string());
                            }
                        }
                    }
                }
            }
        }
        _ => {}
    }
    for i in 0..node.child_count() {
        if let Some(child) = node.child(i as u32) {
            walk_import_nodes(child, source, lang_id, out);
        }
    }
}

fn extract_import_targets_regex(source: &str, from_path: &str) -> Vec<String> {
    let ext = Path::new(from_path)
        .extension()
        .and_then(|e| e.to_str())
        .unwrap_or("");
    let mut out = Vec::new();
    for line in source.lines().take(400) {
        let trimmed = line.trim();
        match ext {
            "rs" if trimmed.starts_with("use ") => {
                if let Some(m) = trimmed.strip_prefix("use ") {
                    let sym = m.split([':', ';', '{']).next().unwrap_or(m).trim();
                    if !sym.is_empty() {
                        out.push(sym.replace("::", "/"));
                    }
                }
            }
            "ts" | "tsx" | "js" | "jsx" if trimmed.starts_with("import ") => {
                if let Some(start) = trimmed.find(['\'', '"']) {
                    let q = trimmed.as_bytes()[start] as char;
                    if let Some(rest) = trimmed.get(start + 1..) {
                        if let Some(end) = rest.find(q) {
                            out.push(rest[..end].to_string());
                        }
                    }
                }
            }
            "py" if trimmed.starts_with("import ") || trimmed.starts_with("from ") => {
                let sym = trimmed
                    .trim_start_matches("from ")
                    .trim_start_matches("import ")
                    .split_whitespace()
                    .next()
                    .unwrap_or("")
                    .trim_matches('.');
                if !sym.is_empty() {
                    out.push(sym.replace('.', "/"));
                }
            }
            _ => {}
        }
    }
    out
}

fn resolve_import(repo_root: &Path, from_rel: &str, target: &str) -> String {
    if target.starts_with("crate/") || target == "crate" {
        let rest = target.strip_prefix("crate/").unwrap_or("");
        let candidate = if rest.is_empty() {
            repo_root.join("src/lib.rs")
        } else {
            let mut base = repo_root.join("src").join(rest);
            if base.extension().is_none() {
                base.set_extension("rs");
            }
            base
        };
        if candidate.exists() {
            return normalize_rel(
                &candidate
                    .strip_prefix(repo_root)
                    .unwrap_or(&candidate)
                    .to_string_lossy(),
            );
        }
    }
    if target.starts_with('.') {
        let from_dir = Path::new(from_rel).parent().unwrap_or(Path::new(""));
        let joined = from_dir.join(target);
        let mut canon = PathBuf::new();
        for comp in joined.components() {
            match comp {
                std::path::Component::ParentDir => {
                    canon.pop();
                }
                std::path::Component::Normal(c) => canon.push(c),
                _ => {}
            }
        }
        return normalize_rel(&canon.to_string_lossy());
    }
    if target.starts_with('/') || target.contains(':') {
        return normalize_rel(target);
    }
    let candidate = repo_root.join(target);
    if candidate.exists() {
        return normalize_rel(
            &candidate
                .strip_prefix(repo_root)
                .unwrap_or(&candidate)
                .to_string_lossy(),
        );
    }
    normalize_rel(target)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn builds_nodes_for_edited_paths() {
        let dir = tempfile::tempdir().unwrap();
        let repo = dir.path();
        std::fs::create_dir_all(repo.join("src")).unwrap();
        std::fs::write(repo.join("src/lib.rs"), "pub fn hi() {}\n").unwrap();
        let g = build_structural_graph(repo, &[("src/lib.rs".into(), "agent-0".into(), None)]);
        assert_eq!(g.nodes.len(), 1);
        assert!(g.nodes[0].edited);
    }

    #[test]
    fn rust_import_edges_via_tree_sitter() {
        let dir = tempfile::tempdir().unwrap();
        let repo = dir.path();
        std::fs::create_dir_all(repo.join("src")).unwrap();
        std::fs::write(repo.join("src/lib.rs"), "pub fn hi() {}\n").unwrap();
        std::fs::write(
            repo.join("src/main.rs"),
            "use crate::lib;\nfn main() { lib::hi(); }\n",
        )
        .unwrap();
        let g = build_structural_graph(repo, &[("src/main.rs".into(), "agent-0".into(), None)]);
        assert!(g
            .edges
            .iter()
            .any(|e| e.from.contains("main") && e.to.contains("lib")));
    }

    #[test]
    fn symbol_nodes_for_rust_functions() {
        let dir = tempfile::tempdir().unwrap();
        let repo = dir.path();
        std::fs::create_dir_all(repo.join("src")).unwrap();
        std::fs::write(
            repo.join("src/lib.rs"),
            "pub fn alpha() {}\npub fn beta() {}\n",
        )
        .unwrap();
        let g = enrich_symbol_nodes(
            build_structural_graph(repo, &[("src/lib.rs".into(), "agent-0".into(), None)]),
            repo,
        );
        assert!(g.nodes.iter().any(|n| n.id.contains("alpha")));
        assert!(g.nodes.iter().any(|n| n.id.contains("beta")));
        assert_eq!(g.version, 3);
    }

    #[test]
    fn aggregates_to_module_nodes() {
        let dir = tempfile::tempdir().unwrap();
        let repo = dir.path();
        std::fs::create_dir_all(repo.join("src")).unwrap();
        std::fs::write(repo.join("src/a.rs"), "pub fn a() {}\n").unwrap();
        std::fs::write(repo.join("src/b.rs"), "pub fn b() {}\n").unwrap();
        let g = build_module_graph(
            repo,
            &[
                ("src/a.rs".into(), "agent-0".into(), None),
                ("src/b.rs".into(), "agent-1".into(), None),
            ],
        );
        assert_eq!(g.nodes.len(), 1);
        assert_eq!(g.nodes[0].id, "src");
    }
}
