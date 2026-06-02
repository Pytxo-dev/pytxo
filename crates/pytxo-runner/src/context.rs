use std::fs;
use std::path::{Path, PathBuf};

use glob::glob;
use pytxo_core::{FidelityTier, PytxoError, Result, RunId};
use pytxo_signal::{read_scaffolded, scaffold_source};

/// Materialize Signal Core context for an agent under `.pytxo/context/{run}/{agent}/`.
pub fn prepare_agent_context(
    repo_root: &Path,
    data_dir: &Path,
    run_id: &RunId,
    agent_id: &str,
    path_patterns: &[String],
    enabled: bool,
    tier: FidelityTier,
) -> Result<Option<PathBuf>> {
    if !enabled || path_patterns.is_empty() {
        return Ok(None);
    }

    let context_root = data_dir.join("context").join(&run_id.0).join(agent_id);
    fs::create_dir_all(&context_root).map_err(PytxoError::Io)?;

    let mut manifest: Vec<ContextEntry> = Vec::new();

    for pattern in path_patterns {
        for file in resolve_pattern(repo_root, pattern)? {
            if !file.is_file() {
                continue;
            }
            let rel = file
                .strip_prefix(repo_root)
                .unwrap_or(&file)
                .to_string_lossy()
                .replace('\\', "/");

            let scaffold = read_scaffolded(&file, tier).or_else(|_| {
                let raw = fs::read_to_string(&file).map_err(PytxoError::Io)?;
                scaffold_source(&file, &raw, tier)
            })?;

            let out_path = context_root.join(&rel);
            if let Some(parent) = out_path.parent() {
                fs::create_dir_all(parent).map_err(PytxoError::Io)?;
            }
            fs::write(&out_path, &scaffold.content).map_err(PytxoError::Io)?;

            manifest.push(ContextEntry {
                source: rel.clone(),
                scaffolded: rel,
                token_reduction_pct: scaffold.stats.token_reduction_pct,
                fallback_raw: scaffold.fallback_raw,
            });
        }
    }

    let manifest_path = context_root.join("manifest.json");
    let json = serde_json::to_string_pretty(&manifest)
        .map_err(|e| PytxoError::Other(format!("context manifest: {e}")))?;
    fs::write(&manifest_path, json).map_err(PytxoError::Io)?;

    Ok(Some(context_root))
}

#[derive(serde::Serialize)]
struct ContextEntry {
    source: String,
    scaffolded: String,
    token_reduction_pct: f64,
    fallback_raw: bool,
}

fn resolve_pattern(repo_root: &Path, pattern: &str) -> Result<Vec<PathBuf>> {
    let normalized = pattern.replace('\\', "/");
    let full_pattern = repo_root.join(&normalized);
    let pattern_str = full_pattern.to_string_lossy().replace('\\', "/");

    if normalized.contains('*') {
        let mut files = Vec::new();
        for entry in glob(&pattern_str).map_err(|e| PytxoError::Runner(format!("glob: {e}")))? {
            let path = entry.map_err(|e| PytxoError::Runner(format!("glob entry: {e}")))?;
            files.push(path);
        }
        return Ok(files);
    }

    let path = repo_root.join(&normalized);
    if path.exists() {
        Ok(vec![path])
    } else {
        Ok(Vec::new())
    }
}
