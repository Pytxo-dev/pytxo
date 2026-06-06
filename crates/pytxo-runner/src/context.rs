use std::fs;
use std::path::{Path, PathBuf};

use glob::glob;
use pytxo_core::{
    content_hash, ArbitrageSample, CacheLookup, CachePut, ContextCache, FidelityTier, ModelId,
    PytxoError, Result, RunId, TokenEstimator,
};
use pytxo_signal::{read_scaffolded, scaffold_source};

use crate::arbitrage::ArbitrageProfiler;

pub struct ContextBundle {
    pub context_dir: Option<PathBuf>,
    pub arbitrage: Vec<ArbitrageSample>,
}

/// Materialize Signal Core context for an agent under `.pytxo/context/{run}/{agent}/`.
#[allow(clippy::too_many_arguments)]
pub fn prepare_agent_context(
    repo_root: &Path,
    data_dir: &Path,
    run_id: &RunId,
    agent_id: &str,
    path_patterns: &[String],
    enabled: bool,
    tier: FidelityTier,
    estimator: &dyn TokenEstimator,
    model: &ModelId,
) -> Result<ContextBundle> {
    prepare_agent_context_for_root(
        repo_root,
        data_dir,
        run_id,
        agent_id,
        path_patterns,
        enabled,
        tier,
        estimator,
        model,
        None,
        None,
        "",
        false,
    )
}

/// Like [`prepare_agent_context`], tagging each manifest entry with a modular
/// project `root_id` ([[ADR-0011-modular-project-manifest]]).
#[allow(clippy::too_many_arguments)]
pub fn prepare_agent_context_for_root(
    repo_root: &Path,
    data_dir: &Path,
    run_id: &RunId,
    agent_id: &str,
    path_patterns: &[String],
    enabled: bool,
    tier: FidelityTier,
    estimator: &dyn TokenEstimator,
    model: &ModelId,
    root_id: Option<&str>,
    cache: Option<&dyn ContextCache>,
    domain_id: &str,
    cache_enabled: bool,
) -> Result<ContextBundle> {
    if !enabled || path_patterns.is_empty() {
        return Ok(ContextBundle {
            context_dir: None,
            arbitrage: Vec::new(),
        });
    }

    let context_root = data_dir.join("context").join(&run_id.0).join(agent_id);
    fs::create_dir_all(&context_root).map_err(PytxoError::Io)?;

    let profiler = ArbitrageProfiler::new(estimator);
    let mut manifest: Vec<ContextEntry> = Vec::new();
    let mut arbitrage = Vec::new();

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

            let raw_bytes = fs::read(&file).map_err(PytxoError::Io)?;
            let hash = content_hash(&raw_bytes);

            let scaffold = if cache_enabled {
                if let Some(cache_client) = cache {
                    let lookup = CacheLookup {
                        domain_id: domain_id.to_string(),
                        path: rel.clone(),
                        fidelity: tier.as_str().to_string(),
                        content_hash: hash.clone(),
                    };
                    if let Ok(Some(cached)) = cache_client.get(&lookup) {
                        pytxo_core::ScaffoldResult {
                            path: rel.clone(),
                            content: cached.content,
                            stats: pytxo_core::ScaffoldStats {
                                original_bytes: raw_bytes.len(),
                                scaffolded_bytes: cached.scaffolded_bytes,
                                language: cached.language,
                                token_reduction_pct: 0.0,
                            },
                            fallback_raw: false,
                        }
                    } else {
                        let local = scaffold_file(&file, tier)?;
                        let _ = cache_client.put(&CachePut {
                            domain_id: domain_id.to_string(),
                            path: rel.clone(),
                            fidelity: tier.as_str().to_string(),
                            content_hash: hash,
                            content: local.content.clone(),
                            scaffolded_bytes: local.content.len(),
                            language: cached_language_hint(&file),
                        });
                        local
                    }
                } else {
                    scaffold_file(&file, tier)?
                }
            } else {
                scaffold_file(&file, tier)?
            };

            let sample = profiler.profile_file(&file, &scaffold, model);
            arbitrage.push(sample);

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
                fidelity: tier.as_str().to_string(),
                root_id: root_id.map(str::to_string),
                bytes_scaffolded: scaffold.content.len(),
            });
        }
    }

    let manifest_path = context_root.join("manifest.json");
    let json = serde_json::to_string_pretty(&manifest)
        .map_err(|e| PytxoError::Other(format!("context manifest: {e}")))?;
    fs::write(&manifest_path, json).map_err(PytxoError::Io)?;

    Ok(ContextBundle {
        context_dir: Some(context_root),
        arbitrage,
    })
}

/// Default globs when pulling read-only project roots into agent context.
const READONLY_CONTEXT_PATTERNS: &[&str] = &["**/*.proto", "**/*.rs", "**/*.md", "**/*.json"];

/// Scaffold files from additional read-only project roots into an existing context
/// bundle ([[ADR-0011-modular-project-manifest]] cross-root protos).
#[allow(clippy::too_many_arguments)]
pub fn extend_context_with_readonly_roots(
    mut bundle: ContextBundle,
    data_dir: &Path,
    run_id: &RunId,
    agent_id: &str,
    readonly_roots: &[(String, PathBuf)],
    tier: FidelityTier,
    estimator: &dyn TokenEstimator,
    model: &ModelId,
) -> Result<ContextBundle> {
    if readonly_roots.is_empty() {
        return Ok(bundle);
    }

    let context_root = match bundle.context_dir {
        Some(ref p) => p.clone(),
        None => data_dir.join("context").join(&run_id.0).join(agent_id),
    };
    fs::create_dir_all(&context_root).map_err(PytxoError::Io)?;

    let manifest_path = context_root.join("manifest.json");
    let mut manifest: Vec<ContextEntry> = if manifest_path.exists() {
        let raw = fs::read_to_string(&manifest_path).map_err(PytxoError::Io)?;
        serde_json::from_str(&raw).unwrap_or_default()
    } else {
        Vec::new()
    };

    let profiler = ArbitrageProfiler::new(estimator);
    let patterns: Vec<String> = READONLY_CONTEXT_PATTERNS
        .iter()
        .map(|s| (*s).to_string())
        .collect();

    for (label, repo_root) in readonly_roots {
        for pattern in &patterns {
            for file in resolve_pattern(repo_root, pattern)? {
                if !file.is_file() {
                    continue;
                }
                let rel = file
                    .strip_prefix(repo_root)
                    .unwrap_or(&file)
                    .to_string_lossy()
                    .replace('\\', "/");
                let scaffolded = format!("{label}/{rel}");
                if manifest.iter().any(|e| e.scaffolded == scaffolded) {
                    continue;
                }
                let scaffold = read_scaffolded(&file, tier).or_else(|_| {
                    let raw = fs::read_to_string(&file).map_err(PytxoError::Io)?;
                    scaffold_source(&file, &raw, tier)
                })?;
                let sample = profiler.profile_file(&file, &scaffold, model);
                bundle.arbitrage.push(sample);
                let out_path = context_root.join(&scaffolded);
                if let Some(parent) = out_path.parent() {
                    fs::create_dir_all(parent).map_err(PytxoError::Io)?;
                }
                fs::write(&out_path, &scaffold.content).map_err(PytxoError::Io)?;
                manifest.push(ContextEntry {
                    source: format!("{label}:{rel}"),
                    scaffolded,
                    token_reduction_pct: scaffold.stats.token_reduction_pct,
                    fallback_raw: scaffold.fallback_raw,
                    fidelity: tier.as_str().to_string(),
                    root_id: Some(label.clone()),
                    bytes_scaffolded: scaffold.content.len(),
                });
            }
        }
    }

    let manifest_path = context_root.join("manifest.json");
    let json = serde_json::to_string_pretty(&manifest)
        .map_err(|e| PytxoError::Other(format!("context manifest: {e}")))?;
    fs::write(&manifest_path, json).map_err(PytxoError::Io)?;
    bundle.context_dir = Some(context_root);
    Ok(bundle)
}

#[derive(serde::Serialize, serde::Deserialize)]
struct ContextEntry {
    source: String,
    scaffolded: String,
    token_reduction_pct: f64,
    fallback_raw: bool,
    /// Fidelity tier used to scaffold this entry ([[closed-loop-fidelity]]).
    fidelity: String,
    /// Modular project root label ([[ADR-0011-modular-project-manifest]]).
    #[serde(skip_serializing_if = "Option::is_none")]
    root_id: Option<String>,
    /// Bytes written for the scaffolded copy.
    bytes_scaffolded: usize,
}

fn scaffold_file(file: &Path, tier: FidelityTier) -> Result<pytxo_core::ScaffoldResult> {
    read_scaffolded(file, tier).or_else(|_| {
        let raw = fs::read_to_string(file).map_err(PytxoError::Io)?;
        scaffold_source(file, &raw, tier)
    })
}

fn cached_language_hint(file: &Path) -> Option<String> {
    file.extension()
        .and_then(|e| e.to_str())
        .map(|e| e.to_ascii_lowercase())
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
