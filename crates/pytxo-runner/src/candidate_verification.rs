//! Combined-candidate source attestation for Orbit/Galaxy, one repository domain.
//! Commands run through the existing bounded verifier; this module never spawns processes.
use std::path::{Path, PathBuf};

use crate::change_set::{
    collect_inventory, file_mode, load_review_package, manifest_digest, replace_synced,
    set_file_mode, sha256_bytes, validated_change_path, verify_manifest_preimages,
};
use pytxo_core::{
    CandidateCheckEvidence, CandidateInventoryFile, CandidateVerificationEvidence,
    PreparedRunFileKind, PreparedRunManifest, PytxoError, Result,
};

// This source snapshot has no Git history. Stop Git's ancestor discovery from
// reaching the primary repository, including when a check changes directories.
// A gitdir beneath this regular file cannot resolve to a real Git directory.
const GIT_BOUNDARY: &str = "gitdir: .git/pytxo-candidate-has-no-git-context\n";

fn reject(message: &str) -> PytxoError {
    PytxoError::Runner(message.into())
}

fn inventory(root: &Path, exclusions: &[String]) -> Result<Vec<CandidateInventoryFile>> {
    let nodes = collect_inventory(root, exclusions)?;
    if !nodes.symlinks.is_empty() {
        return Err(reject("candidate verification refuses included symlinks"));
    }
    nodes
        .files
        .into_iter()
        .map(|(path, absolute)| {
            Ok(CandidateInventoryFile {
                path,
                sha256: sha256_bytes(&std::fs::read(&absolute).map_err(PytxoError::Io)?),
                mode: file_mode(&absolute)?,
            })
        })
        .collect()
}

fn persist(data_dir: &Path, manifest: &mut PreparedRunManifest) -> Result<()> {
    manifest.package_digest = manifest_digest(manifest)?;
    replace_synced(
        &data_dir
            .join("reviews")
            .join(&manifest.run_id)
            .join("manifest.json"),
        &serde_json::to_vec_pretty(manifest).map_err(|e| reject(&e.to_string()))?,
    )
}

/// Holds a frozen composition while its ordered recipe is executed by the existing verifier.
/// Drop removes only this generated candidate directory, including on cancellation/failure.
pub struct CandidateVerification {
    repo_root: PathBuf,
    data_dir: PathBuf,
    workspace: PathBuf,
    manifest: PreparedRunManifest,
    base: Vec<CandidateInventoryFile>,
    candidate: Vec<CandidateInventoryFile>,
    exclusions: Vec<String>,
}

impl CandidateVerification {
    pub fn prepare(
        repo_root: &Path,
        data_dir: &Path,
        manifest: &PreparedRunManifest,
        exclusions: &[String],
    ) -> Result<Self> {
        if load_review_package(data_dir, &manifest.run_id)? != *manifest {
            return Err(reject("candidate package changed before verification"));
        }
        verify_manifest_preimages(repo_root, manifest)?;
        let mut exclusions = exclusions.to_vec();
        exclusions.extend([".git".into(), ".pytxo".into()]);
        exclusions.sort();
        exclusions.dedup();
        let base = inventory(repo_root, &exclusions)?;
        let workspace = data_dir
            .join("candidate-verification")
            .join(uuid::Uuid::new_v4().to_string());
        std::fs::create_dir_all(&workspace).map_err(PytxoError::Io)?;
        let mut guard = Self {
            repo_root: repo_root.into(),
            data_dir: data_dir.into(),
            workspace,
            manifest: manifest.clone(),
            base,
            candidate: vec![],
            exclusions,
        };
        // Promote before running commands: an interrupted or failed candidate cannot be
        // mistaken for a legacy package and applied through the low-level API.
        guard.manifest.version = 3;
        guard.manifest.candidate_verification = None;
        persist(data_dir, &mut guard.manifest)?;
        for entry in &guard.base {
            let relative = validated_change_path(&entry.path)?;
            let bytes = std::fs::read(repo_root.join(&relative)).map_err(PytxoError::Io)?;
            if sha256_bytes(&bytes) != entry.sha256
                || file_mode(&repo_root.join(&relative))? != entry.mode
            {
                return Err(reject("repository changed while copying candidate inputs"));
            }
            let target = guard.workspace.join(relative);
            if let Some(parent) = target.parent() {
                std::fs::create_dir_all(parent).map_err(PytxoError::Io)?;
            }
            std::fs::write(&target, bytes).map_err(PytxoError::Io)?;
            set_file_mode(&target, entry.mode)?;
        }
        if inventory(repo_root, &guard.exclusions)? != guard.base {
            return Err(reject("repository changed while copying candidate inputs"));
        }
        for file in &guard.manifest.files {
            let target = guard.workspace.join(validated_change_path(&file.path)?);
            match file.kind {
                PreparedRunFileKind::Delete => {
                    std::fs::remove_file(target).map_err(PytxoError::Io)?
                }
                _ => {
                    let digest = file
                        .blob_digest
                        .as_ref()
                        .ok_or_else(|| reject("candidate target has no blob"))?;
                    let bytes = std::fs::read(
                        data_dir
                            .join("reviews")
                            .join(&manifest.run_id)
                            .join("blobs")
                            .join(digest),
                    )
                    .map_err(PytxoError::Io)?;
                    if sha256_bytes(&bytes) != *digest {
                        return Err(reject("candidate immutable blob changed"));
                    }
                    if let Some(parent) = target.parent() {
                        std::fs::create_dir_all(parent).map_err(PytxoError::Io)?;
                    }
                    std::fs::write(&target, bytes).map_err(PytxoError::Io)?;
                    set_file_mode(&target, file.after_mode)?;
                }
            }
        }
        guard.candidate = inventory(&guard.workspace, &guard.exclusions)?;
        std::fs::write(guard.workspace.join(".git"), GIT_BOUNDARY).map_err(PytxoError::Io)?;
        Ok(guard)
    }

    pub fn workspace_root(&self) -> &Path {
        &self.workspace
    }

    /// Call after EVERY command, including a failed command, before another command runs.
    pub fn check_unchanged(&self) -> Result<()> {
        let git_boundary = self.workspace.join(".git");
        let boundary_is_file = std::fs::symlink_metadata(&git_boundary)
            .map(|metadata| metadata.file_type().is_file())
            .unwrap_or(false);
        if !boundary_is_file
            || std::fs::read(&git_boundary).ok().as_deref() != Some(GIT_BOUNDARY.as_bytes())
        {
            return Err(reject(
                "candidate verifier changed its Git discovery boundary",
            ));
        }
        if inventory(&self.workspace, &self.exclusions)? != self.candidate {
            return Err(reject(
                "candidate verifier changed included source; verification is invalid",
            ));
        }
        Ok(())
    }

    pub fn finish(mut self, checks: Vec<CandidateCheckEvidence>) -> Result<PreparedRunManifest> {
        self.check_unchanged()?;
        if inventory(&self.repo_root, &self.exclusions)? != self.base {
            return Err(reject(
                "repository inputs changed during candidate verification",
            ));
        }
        if load_review_package(&self.data_dir, &self.manifest.run_id)? != self.manifest {
            return Err(reject("candidate package changed during verification"));
        }
        self.manifest.candidate_verification = Some(CandidateVerificationEvidence {
            version: 1,
            base_inventory: self.base.clone(),
            candidate_inventory: self.candidate.clone(),
            exclusions: self.exclusions.clone(),
            checks,
            verified_at: chrono::Utc::now().to_rfc3339(),
        });
        require_candidate_verification(&self.manifest)?;
        persist(&self.data_dir, &mut self.manifest)?;
        Ok(self.manifest.clone())
    }
}

impl Drop for CandidateVerification {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.workspace);
    }
}

/// Rebase the frozen targets onto current preimages for explicit re-verification.
/// Unchanged operator files come from today's primary tree, never stale worker copies.
pub fn refresh_frozen_review_package(
    repo_root: &Path,
    data_dir: &Path,
    manifest: &PreparedRunManifest,
    base_revision: &str,
    exclusions: &[String],
) -> Result<PreparedRunManifest> {
    if load_review_package(data_dir, &manifest.run_id)? != *manifest {
        return Err(reject("frozen review changed before refresh"));
    }
    let base = inventory(repo_root, exclusions)?;
    let scratch = data_dir
        .join("candidate-refresh")
        .join(uuid::Uuid::new_v4().to_string());
    std::fs::create_dir_all(&scratch).map_err(PytxoError::Io)?;
    let result = (|| {
        let mut groups = std::collections::BTreeMap::new();
        for file in &manifest.files {
            groups
                .entry((file.task_id.clone(), file.agent_id.clone()))
                .or_insert_with(Vec::new)
                .push(file);
        }
        let mut inputs = Vec::new();
        for (index, ((task_id, agent_id), files)) in groups.into_iter().enumerate() {
            let workspace = scratch.join(index.to_string());
            std::fs::create_dir_all(&workspace).map_err(PytxoError::Io)?;
            for entry in &base {
                let relative = validated_change_path(&entry.path)?;
                let source = repo_root.join(&relative);
                let bytes = std::fs::read(&source).map_err(PytxoError::Io)?;
                if sha256_bytes(&bytes) != entry.sha256 || file_mode(&source)? != entry.mode {
                    return Err(reject("repository changed while refreshing candidate"));
                }
                let target = workspace.join(relative);
                if let Some(parent) = target.parent() {
                    std::fs::create_dir_all(parent).map_err(PytxoError::Io)?;
                }
                std::fs::write(&target, bytes).map_err(PytxoError::Io)?;
                set_file_mode(&target, entry.mode)?;
            }
            let mut claims = Vec::new();
            for file in files {
                claims.push(file.path.clone());
                let target = workspace.join(validated_change_path(&file.path)?);
                if let Some(digest) = &file.blob_digest {
                    let bytes = std::fs::read(
                        data_dir
                            .join("reviews")
                            .join(&manifest.run_id)
                            .join("blobs")
                            .join(digest),
                    )
                    .map_err(PytxoError::Io)?;
                    if sha256_bytes(&bytes) != *digest {
                        return Err(reject("frozen target changed during refresh"));
                    }
                    if let Some(parent) = target.parent() {
                        std::fs::create_dir_all(parent).map_err(PytxoError::Io)?;
                    }
                    std::fs::write(&target, bytes).map_err(PytxoError::Io)?;
                    set_file_mode(&target, file.after_mode)?;
                } else if target.exists() {
                    std::fs::remove_file(&target).map_err(PytxoError::Io)?;
                }
            }
            inputs.push(crate::AgentWorkspaceInput {
                agent_id,
                task_id,
                workspace_path: workspace,
                claims,
                depends_on: vec![],
            });
        }
        if inventory(repo_root, exclusions)? != base {
            return Err(reject("repository changed during candidate refresh"));
        }
        crate::prepare_review_package(
            repo_root,
            data_dir,
            &manifest.run_id,
            base_revision,
            &inputs,
            exclusions,
        )
    })();
    let _ = std::fs::remove_dir_all(&scratch);
    result
}

/// Public Apply surfaces must require this even for readable legacy v2 packages.
pub fn require_candidate_verification(
    manifest: &PreparedRunManifest,
) -> Result<&CandidateVerificationEvidence> {
    let evidence = pytxo_core::require_candidate_verification_contract(manifest)?;
    for entry in &evidence.base_inventory {
        validated_change_path(&entry.path)?;
    }
    Ok(evidence)
}

pub(crate) fn verify_candidate_base(
    repo_root: &Path,
    manifest: &PreparedRunManifest,
) -> Result<()> {
    if manifest.version == 3 {
        let e = require_candidate_verification(manifest)?;
        if inventory(repo_root, &e.exclusions)? != e.base_inventory {
            return Err(reject(
                "candidate base inventory drifted; refresh and verify review",
            ));
        }
    }
    Ok(())
}

pub(crate) fn verify_candidate_poststate(
    repo_root: &Path,
    manifest: &PreparedRunManifest,
) -> Result<()> {
    if manifest.version == 3 {
        let e = require_candidate_verification(manifest)?;
        if inventory(repo_root, &e.exclusions)? != e.candidate_inventory {
            return Err(reject(
                "observed repository post-state differs from verified candidate",
            ));
        }
    }
    Ok(())
}
