//! Signal Core implementation — `tree-sitter` structural skeletons on the read path.

mod emit;
mod graph;
mod language;

use std::path::Path;

use pytxo_core::{FidelityTier, PytxoError, Result, ScaffoldResult, ScaffoldStats, SignalCore};

pub struct TreeSitterSignalCore;

impl Default for TreeSitterSignalCore {
    fn default() -> Self {
        Self
    }
}

impl SignalCore for TreeSitterSignalCore {
    fn scaffold(&self, path: &Path, source: &str, tier: FidelityTier) -> Result<ScaffoldResult> {
        if tier == FidelityTier::High {
            return Ok(raw_result(path, source));
        }

        let Some(lang) = language::detect(path) else {
            return Ok(raw_result(path, source));
        };

        let mut parser = tree_sitter::Parser::new();
        parser
            .set_language(&lang.grammar)
            .map_err(|e| PytxoError::Other(format!("signal parser: {e}")))?;

        let tree = parser
            .parse(source, None)
            .ok_or_else(|| PytxoError::Other(format!("signal parse failed: {}", path.display())))?;

        let content = emit::emit_skeleton(tree.root_node(), source, tier, lang.id);
        let original_bytes = source.len();
        let scaffolded_bytes = content.len();
        let token_reduction_pct = if original_bytes == 0 {
            0.0
        } else {
            ((original_bytes.saturating_sub(scaffolded_bytes)) as f64 / original_bytes as f64)
                * 100.0
        };

        Ok(ScaffoldResult {
            path: path.display().to_string(),
            content,
            stats: ScaffoldStats {
                original_bytes,
                scaffolded_bytes,
                language: Some(lang.id.to_string()),
                token_reduction_pct,
            },
            fallback_raw: false,
        })
    }
}

fn raw_result(path: &Path, source: &str) -> ScaffoldResult {
    let bytes = source.len();
    ScaffoldResult {
        path: path.display().to_string(),
        content: source.to_string(),
        stats: ScaffoldStats {
            original_bytes: bytes,
            scaffolded_bytes: bytes,
            language: None,
            token_reduction_pct: 0.0,
        },
        fallback_raw: true,
    }
}

/// Convenience entry for callers without trait objects.
pub fn scaffold_source(path: &Path, source: &str, tier: FidelityTier) -> Result<ScaffoldResult> {
    TreeSitterSignalCore.scaffold(path, source, tier)
}

/// Read a file from disk and return a scaffolded view (Signal Core read hook).
pub fn read_scaffolded(path: &Path, tier: FidelityTier) -> Result<ScaffoldResult> {
    TreeSitterSignalCore.read_scaffolded(path, tier)
}

pub use graph::{
    aggregate_modules, build_module_graph, build_structural_graph, enrich_symbol_nodes,
    graph_neighbor_paths, GraphEdge, GraphNode, StructuralGraph,
};

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;

    #[test]
    fn rust_skeleton_strips_function_bodies() {
        let src = r#"
use std::io;

pub fn hello(name: &str) -> String {
    let mut out = String::new();
    out.push_str(name);
    out
}

struct Widget {
    id: u64,
    label: String,
}
"#;
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("sample.rs");
        let mut f = std::fs::File::create(&path).unwrap();
        write!(f, "{src}").unwrap();

        let result = scaffold_source(&path, src, FidelityTier::Low).unwrap();
        assert!(!result.fallback_raw);
        assert!(result.content.contains("pub fn hello"));
        assert!(result.content.contains("/* … */"));
        assert!(!result.content.contains("push_str"));
        assert!(result.stats.scaffolded_bytes < result.stats.original_bytes);
    }

    #[test]
    fn high_fidelity_passthrough() {
        let src = "fn main() { println!(\"x\"); }";
        let path = std::path::Path::new("main.rs");
        let result = scaffold_source(path, src, FidelityTier::High).unwrap();
        assert!(result.fallback_raw);
        assert_eq!(result.content, src);
    }
}
