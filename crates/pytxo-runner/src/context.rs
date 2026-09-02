use std::fs;
use std::path::{Path, PathBuf};

use glob::glob;
use pytxo_core::{
    cloud_path_denied, content_hash, validate_cloud_content, ArbitrageSample, CacheLookup,
    CachePut, ContextCache, FidelityTier, ModelId, PytxoError, Result, RunId, TokenEstimator,
};
use pytxo_signal::{read_scaffolded, scaffold_source};

use crate::arbitrage::ArbitrageProfiler;

pub struct ContextBundle {
    pub context_dir: Option<PathBuf>,
    pub arbitrage: Vec<ArbitrageSample>,
    /// Count of scaffold operations that fell back to raw file content ([[signal-core]]).
    pub fallback_count: usize,
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
        pytxo_core::PermissionProfile::Orbit,
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
    permission_profile: pytxo_core::PermissionProfile,
) -> Result<ContextBundle> {
    if !enabled || path_patterns.is_empty() {
        return Ok(ContextBundle {
            context_dir: None,
            arbitrage: Vec::new(),
            fallback_count: 0,
        });
    }

    let context_root = data_dir.join("context").join(&run_id.0).join(agent_id);
    fs::create_dir_all(&context_root).map_err(PytxoError::Io)?;

    let profiler = ArbitrageProfiler::new(estimator);
    let mut manifest: Vec<ContextEntry> = Vec::new();
    let mut arbitrage = Vec::new();

    let engine = pytxo_core::PermissionEngine::new(permission_profile);

    for pattern in path_patterns {
        for file in resolve_pattern(repo_root, pattern)? {
            if !file.is_file() {
                continue;
            }
            if !engine.may_read(repo_root, &file, repo_root) {
                return Err(PytxoError::Runner(format!(
                    "read denied for {} profile: {}",
                    permission_profile.as_str(),
                    file.display()
                )));
            }
            let rel = file
                .strip_prefix(repo_root)
                .unwrap_or(&file)
                .to_string_lossy()
                .replace('\\', "/");

            let raw_bytes = fs::read(&file).map_err(PytxoError::Io)?;
            let hash = content_hash(&raw_bytes);
            // Cache is an egress boundary. A denied path or likely secret stays
            // entirely local regardless of the configured cache implementation.
            let cache_upload_safe = !cloud_path_denied(&rel)
                && validate_cloud_content(&rel, &String::from_utf8_lossy(&raw_bytes)).is_ok();

            let scaffold = if cache_enabled && cache_upload_safe {
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
        fallback_count: manifest.iter().filter(|e| e.fallback_raw).count(),
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
    bundle.fallback_count = manifest.iter().filter(|e| e.fallback_raw).count();
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

#[cfg(test)]
mod tests {
    use std::sync::Mutex;

    use pytxo_core::{ByteHeuristicEstimator, CachedScaffold, ContextCache, PermissionProfile};

    use super::*;

    #[derive(Default)]
    struct RecordingCache {
        gets: Mutex<Vec<String>>,
        puts: Mutex<Vec<String>>,
    }

    impl ContextCache for RecordingCache {
        fn get(&self, key: &CacheLookup) -> Result<Option<CachedScaffold>> {
            self.gets.lock().unwrap().push(key.path.clone());
            Ok(None)
        }

        fn put(&self, entry: &CachePut) -> Result<()> {
            self.puts.lock().unwrap().push(entry.path.clone());
            Ok(())
        }
    }

    #[test]
    fn cache_puts_only_allowed_secret_free_source() {
        let tmp = tempfile::tempdir().unwrap();
        let repo = tmp.path().join("repo");
        let data = tmp.path().join("data");
        fs::create_dir_all(repo.join("src")).unwrap();
        fs::write(repo.join(".env"), "OPENAI_API_KEY=not-uploaded").unwrap();
        fs::write(
            repo.join("src/credential.txt"),
            "GITHUB_TOKEN=ghp_abcdefghijklmnopqrstuvwxyz123456",
        )
        .unwrap();
        fs::write(repo.join("src/lib.rs"), "pub fn allowed() {}\n").unwrap();
        let cache = RecordingCache::default();
        let patterns = vec![
            ".env".to_string(),
            "src/credential.txt".to_string(),
            "src/lib.rs".to_string(),
        ];

        prepare_agent_context_for_root(
            &repo,
            &data,
            &RunId::new(),
            "agent",
            &patterns,
            true,
            FidelityTier::Low,
            &ByteHeuristicEstimator,
            &ModelId("test-model".into()),
            None,
            Some(&cache),
            "domain",
            true,
            PermissionProfile::Supernova,
        )
        .unwrap();

        assert_eq!(cache.gets.lock().unwrap().as_slice(), ["src/lib.rs"]);
        assert_eq!(cache.puts.lock().unwrap().as_slice(), ["src/lib.rs"]);
    }
}
