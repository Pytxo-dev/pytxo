use std::fs;
use std::path::{Component, Path, PathBuf};

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

    let context_root = prepare_context_root(data_dir, run_id, agent_id)?;

    let profiler = ArbitrageProfiler::new(estimator);
    let mut manifest: Vec<ContextEntry> = Vec::new();
    let mut arbitrage = Vec::new();

    let engine = pytxo_core::PermissionEngine::new(permission_profile);
    // The materialization boundary owns this ceiling, including retry callers.
    let tier = engine.max_fidelity(tier);

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
            let out_path = context_output_path(&context_root, &rel)?;

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

            if let Some(parent) = out_path.parent() {
                fs::create_dir_all(parent).map_err(PytxoError::Io)?;
            }
            crate::change_set::replace_synced(&out_path, scaffold.content.as_bytes())?;

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

    let manifest_path = context_output_path(&context_root, "manifest.json")?;
    let json = serde_json::to_string_pretty(&manifest)
        .map_err(|e| PytxoError::Other(format!("context manifest: {e}")))?;
    crate::change_set::replace_synced(&manifest_path, json.as_bytes())?;

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

    let context_root = prepare_context_root(data_dir, run_id, agent_id)?;
    if let Some(existing) = bundle.context_dir.as_ref() {
        if fs::canonicalize(existing).map_err(PytxoError::Io)?
            != fs::canonicalize(&context_root).map_err(PytxoError::Io)?
        {
            return Err(PytxoError::Runner(
                "context bundle does not belong to this run and agent".into(),
            ));
        }
    }

    let manifest_path = context_output_path(&context_root, "manifest.json")?;
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
        context_output_path(&context_root, label)?;
        let canonical_root = fs::canonicalize(repo_root).map_err(PytxoError::Io)?;
        for pattern in &patterns {
            for file in resolve_pattern(repo_root, pattern)? {
                if !file.is_file() {
                    continue;
                }
                if !fs::canonicalize(&file)
                    .map_err(PytxoError::Io)?
                    .starts_with(&canonical_root)
                {
                    return Err(PytxoError::Runner(format!(
                        "read-only context source escaped its declared root: {}",
                        file.display()
                    )));
                }
                let rel = file
                    .strip_prefix(repo_root)
                    .unwrap_or(&file)
                    .to_string_lossy()
                    .replace('\\', "/");
                let scaffolded = format!("{label}/{rel}");
                let out_path = context_output_path(&context_root, &scaffolded)?;
                if manifest.iter().any(|e| e.scaffolded == scaffolded) {
                    continue;
                }
                let scaffold = read_scaffolded(&file, tier).or_else(|_| {
                    let raw = fs::read_to_string(&file).map_err(PytxoError::Io)?;
                    scaffold_source(&file, &raw, tier)
                })?;
                let sample = profiler.profile_file(&file, &scaffold, model);
                bundle.arbitrage.push(sample);
                if let Some(parent) = out_path.parent() {
                    fs::create_dir_all(parent).map_err(PytxoError::Io)?;
                }
                crate::change_set::replace_synced(&out_path, scaffold.content.as_bytes())?;
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

    let manifest_path = context_output_path(&context_root, "manifest.json")?;
    let json = serde_json::to_string_pretty(&manifest)
        .map_err(|e| PytxoError::Other(format!("context manifest: {e}")))?;
    crate::change_set::replace_synced(&manifest_path, json.as_bytes())?;
    bundle.context_dir = Some(context_root);
    bundle.fallback_count = manifest.iter().filter(|e| e.fallback_raw).count();
    Ok(bundle)
}

fn prepare_context_root(data_dir: &Path, run_id: &RunId, agent_id: &str) -> Result<PathBuf> {
    // The configured data directory is the trust anchor; links below it must not
    // redirect a generated run/agent directory. A configured data-dir link itself
    // still resolves to the operator's chosen storage location.
    fs::create_dir_all(data_dir).map_err(PytxoError::Io)?;
    let relative = format!("context/{}/{agent_id}", run_id.0);
    let context_root = context_output_path(data_dir, &relative)?;
    fs::create_dir_all(&context_root).map_err(PytxoError::Io)?;
    Ok(pytxo_core::strip_extended_path(context_root))
}

/// Keep materialized files and existing link targets under this context root.
/// Source-read authorization alone does not constrain a destination containing `..`.
fn context_output_path(context_root: &Path, relative: &str) -> Result<PathBuf> {
    let normalized = relative.replace('\\', "/");
    let relative_path = Path::new(&normalized);
    if normalized.contains(':')
        || normalized
            .split('/')
            .any(|part| part.is_empty() || part == "." || part == "..")
        || relative_path
            .components()
            .any(|part| !matches!(part, Component::Normal(_)))
    {
        return Err(PytxoError::Runner(format!(
            "unsafe context output path: {relative}"
        )));
    }
    let canonical_root = fs::canonicalize(context_root).map_err(PytxoError::Io)?;
    let mut output = canonical_root.clone();
    for part in relative_path.components() {
        output.push(part);
        match fs::symlink_metadata(&output) {
            Ok(metadata) => {
                if metadata.file_type().is_symlink()
                    || !fs::canonicalize(&output)
                        .map_err(PytxoError::Io)?
                        .starts_with(&canonical_root)
                {
                    return Err(PytxoError::Runner(format!(
                        "context output link escaped its context root: {}",
                        output.display()
                    )));
                }
            }
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => {}
            Err(error) => return Err(PytxoError::Io(error)),
        }
    }
    Ok(output)
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

    #[test]
    fn context_boundary_rejects_readonly_label_destination_escape() {
        let tmp = tempfile::tempdir().unwrap();
        let repo = tmp.path().join("repo");
        let data = tmp.path().join("data");
        fs::create_dir_all(&repo).unwrap();
        fs::write(repo.join("README.md"), "source contents\n").unwrap();
        let run_id = RunId::new();
        let outside = data.join("context").join(&run_id.0).join("escape");
        let absolute_outside = tmp.path().join("absolute-escape");
        for (label, destination) in [
            ("../escape".to_string(), outside),
            (
                absolute_outside.to_string_lossy().into_owned(),
                absolute_outside,
            ),
        ] {
            fs::create_dir_all(&destination).unwrap();
            let sentinel = destination.join("README.md");
            fs::write(&sentinel, "operator contents\n").unwrap();
            let result = extend_context_with_readonly_roots(
                ContextBundle {
                    context_dir: None,
                    arbitrage: vec![],
                    fallback_count: 0,
                },
                &data,
                &run_id,
                "agent",
                &[(label.clone(), repo.clone())],
                FidelityTier::Low,
                &ByteHeuristicEstimator,
                &ModelId("test-model".into()),
            );
            assert_eq!(
                fs::read_to_string(&sentinel).unwrap(),
                "operator contents\n",
                "context label {label} overwrote a file outside its context directory"
            );
            assert!(result.is_err(), "escaping label was accepted: {label}");
        }
    }

    #[test]
    fn context_boundary_rejects_traversal_in_source_relative_path() {
        let tmp = tempfile::tempdir().unwrap();
        let repo = tmp.path().join("repo");
        let data = tmp.path().join("data");
        fs::create_dir_all(&repo).unwrap();
        fs::write(repo.join("README.md"), "source contents\n").unwrap();
        let run_id = RunId::new();
        let result = prepare_agent_context(
            &repo,
            &data,
            &run_id,
            "agent",
            &["../repo/README.md".into()],
            true,
            FidelityTier::Low,
            &ByteHeuristicEstimator,
            &ModelId("test-model".into()),
        );
        assert!(
            result.is_err(),
            "source traversal escaped the context directory"
        );
        assert!(!data
            .join("context")
            .join(&run_id.0)
            .join("repo/README.md")
            .exists());
    }

    #[test]
    fn context_boundary_caps_deepspace_fidelity_at_materialization() {
        let tmp = tempfile::tempdir().unwrap();
        let repo = tmp.path().join("repo");
        fs::create_dir_all(&repo).unwrap();
        fs::write(
            repo.join("lib.rs"),
            "pub fn value() -> &'static str { \"implementation-only-marker\" }\n",
        )
        .unwrap();
        let bundle = prepare_agent_context_for_root(
            &repo,
            &tmp.path().join("data"),
            &RunId::new(),
            "agent",
            &["lib.rs".into()],
            true,
            FidelityTier::High,
            &ByteHeuristicEstimator,
            &ModelId("test-model".into()),
            None,
            None,
            "domain",
            false,
            PermissionProfile::DeepSpace,
        )
        .unwrap();
        let context = bundle.context_dir.unwrap();
        let manifest: Vec<ContextEntry> =
            serde_json::from_str(&fs::read_to_string(context.join("manifest.json")).unwrap())
                .unwrap();
        assert_eq!(manifest[0].fidelity, "low");
        assert!(!fs::read_to_string(context.join("lib.rs"))
            .unwrap()
            .contains("implementation-only-marker"));
    }

    #[test]
    fn context_boundary_preserves_safe_nested_readonly_context() {
        let tmp = tempfile::tempdir().unwrap();
        let repo = tmp.path().join("repo");
        fs::create_dir_all(&repo).unwrap();
        fs::write(repo.join("README.md"), "source contents\n").unwrap();
        let bundle = extend_context_with_readonly_roots(
            ContextBundle {
                context_dir: None,
                arbitrage: vec![],
                fallback_count: 0,
            },
            &tmp.path().join("data"),
            &RunId::new(),
            "agent",
            &[("shared/protos".into(), repo)],
            FidelityTier::Low,
            &ByteHeuristicEstimator,
            &ModelId("test-model".into()),
        )
        .unwrap();
        assert_eq!(
            fs::read_to_string(bundle.context_dir.unwrap().join("shared/protos/README.md"))
                .unwrap(),
            "source contents\n"
        );
    }

    #[cfg(unix)]
    fn link_directory(link: &Path, target: &Path) {
        std::os::unix::fs::symlink(target, link).unwrap();
    }

    #[cfg(windows)]
    fn link_directory(link: &Path, target: &Path) {
        let output = std::process::Command::new("cmd")
            .args(["/D", "/C", "mklink", "/J"])
            .arg(link)
            .arg(target)
            .output()
            .expect("create fixture junction");
        assert!(output.status.success(), "{output:?}");
    }

    #[test]
    fn context_boundary_rejects_existing_destination_link() {
        let tmp = tempfile::tempdir().unwrap();
        let repo = tmp.path().join("repo");
        let outside = tmp.path().join("outside");
        let data = tmp.path().join("data");
        let run_id = RunId::new();
        let context = data.join("context").join(&run_id.0).join("agent");
        fs::create_dir_all(&repo).unwrap();
        fs::create_dir_all(&outside).unwrap();
        fs::create_dir_all(&context).unwrap();
        fs::write(repo.join("README.md"), "source contents\n").unwrap();
        fs::write(outside.join("README.md"), "operator contents\n").unwrap();
        link_directory(&context.join("shared"), &outside);
        let result = extend_context_with_readonly_roots(
            ContextBundle {
                context_dir: Some(context),
                arbitrage: vec![],
                fallback_count: 0,
            },
            &data,
            &run_id,
            "agent",
            &[("shared".into(), repo)],
            FidelityTier::Low,
            &ByteHeuristicEstimator,
            &ModelId("test-model".into()),
        );
        assert_eq!(
            fs::read_to_string(outside.join("README.md")).unwrap(),
            "operator contents\n"
        );
        assert!(
            result.is_err(),
            "destination link escaped context confinement"
        );
    }

    #[test]
    fn context_boundary_rejects_readonly_source_link_escape() {
        let tmp = tempfile::tempdir().unwrap();
        let repo = tmp.path().join("repo");
        let outside = tmp.path().join("outside");
        fs::create_dir_all(&repo).unwrap();
        fs::create_dir_all(&outside).unwrap();
        fs::write(outside.join("README.md"), "outside source\n").unwrap();
        link_directory(&repo.join("linked"), &outside);
        let result = extend_context_with_readonly_roots(
            ContextBundle {
                context_dir: None,
                arbitrage: vec![],
                fallback_count: 0,
            },
            &tmp.path().join("data"),
            &RunId::new(),
            "agent",
            &[("shared".into(), repo)],
            FidelityTier::Low,
            &ByteHeuristicEstimator,
            &ModelId("test-model".into()),
        );
        assert!(
            result.is_err(),
            "read-only context followed an outside source link"
        );
    }

    #[test]
    fn context_boundary_rejects_context_root_and_ancestor_links() {
        for link_run_directory in [false, true] {
            let tmp = tempfile::tempdir().unwrap();
            let repo = tmp.path().join("repo");
            let outside = tmp.path().join("outside");
            let data = tmp.path().join("data");
            let run_id = RunId::new();
            let runs = data.join("context");
            let run = runs.join(&run_id.0);
            fs::create_dir_all(&repo).unwrap();
            fs::create_dir_all(&outside).unwrap();
            fs::create_dir_all(&runs).unwrap();
            let escaped_context = if link_run_directory {
                link_directory(&run, &outside);
                outside.join("agent")
            } else {
                fs::create_dir_all(&run).unwrap();
                link_directory(&run.join("agent"), &outside);
                outside.clone()
            };
            fs::create_dir_all(&escaped_context).unwrap();
            fs::write(repo.join("README.md"), "source contents\n").unwrap();
            fs::write(escaped_context.join("README.md"), "operator contents\n").unwrap();
            let result = prepare_agent_context(
                &repo,
                &data,
                &run_id,
                "agent",
                &["README.md".into()],
                true,
                FidelityTier::Low,
                &ByteHeuristicEstimator,
                &ModelId("test-model".into()),
            );
            assert_eq!(
                fs::read_to_string(escaped_context.join("README.md")).unwrap(),
                "operator contents\n",
                "a context root or ancestor link redirected materialization"
            );
            assert!(result.is_err());
        }
    }

    #[test]
    fn context_boundary_replaces_output_hardlinks_without_modifying_their_targets() {
        let tmp = tempfile::tempdir().unwrap();
        let repo = tmp.path().join("repo");
        let outside = tmp.path().join("outside");
        let data = tmp.path().join("data");
        let run_id = RunId::new();
        let context = data.join("context").join(&run_id.0).join("agent");
        fs::create_dir_all(&repo).unwrap();
        fs::create_dir_all(&outside).unwrap();
        fs::create_dir_all(&context).unwrap();
        fs::write(repo.join("README.md"), "source contents\n").unwrap();
        for file in ["README.md", "manifest.json"] {
            fs::write(outside.join(file), "operator contents\n").unwrap();
            fs::hard_link(outside.join(file), context.join(file)).unwrap();
        }
        let bundle = prepare_agent_context(
            &repo,
            &data,
            &run_id,
            "agent",
            &["README.md".into()],
            true,
            FidelityTier::Low,
            &ByteHeuristicEstimator,
            &ModelId("test-model".into()),
        )
        .unwrap();
        for file in ["README.md", "manifest.json"] {
            assert_eq!(
                fs::read_to_string(outside.join(file)).unwrap(),
                "operator contents\n",
                "materialization modified an outside hardlink target"
            );
        }
        assert_eq!(
            fs::read_to_string(bundle.context_dir.unwrap().join("README.md")).unwrap(),
            "source contents\n"
        );
    }

    #[test]
    fn context_boundary_preserves_explicitly_configured_data_directory_link() {
        let tmp = tempfile::tempdir().unwrap();
        let repo = tmp.path().join("repo");
        let storage = tmp.path().join("storage");
        let configured_data = tmp.path().join("configured-data");
        fs::create_dir_all(&repo).unwrap();
        fs::create_dir_all(&storage).unwrap();
        fs::write(repo.join("README.md"), "source contents\n").unwrap();
        link_directory(&configured_data, &storage);
        let bundle = prepare_agent_context(
            &repo,
            &configured_data,
            &RunId::new(),
            "agent",
            &["README.md".into()],
            true,
            FidelityTier::Low,
            &ByteHeuristicEstimator,
            &ModelId("test-model".into()),
        )
        .unwrap();
        let context = bundle.context_dir.unwrap();
        assert!(fs::canonicalize(&context)
            .unwrap()
            .starts_with(fs::canonicalize(storage).unwrap()));
        assert_eq!(
            fs::read_to_string(context.join("README.md")).unwrap(),
            "source contents\n"
        );
    }
}
