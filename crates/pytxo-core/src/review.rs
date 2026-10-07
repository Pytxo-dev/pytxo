use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};

use crate::{routing::Digest, PytxoError, Result};

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum PreparedRunFileKind {
    Add,
    Modify,
    Delete,
}

#[derive(Clone, Debug, Default, Deserialize, Eq, PartialEq, Serialize)]
pub struct PreparedRunSummary {
    pub added: u64,
    pub modified: u64,
    pub deleted: u64,
    pub bytes: u64,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct PreparedBlobChunk {
    pub offset: u64,
    pub length: u64,
    pub sha256: String,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct PreparedRunFile {
    pub path: String,
    pub kind: PreparedRunFileKind,
    pub before_sha256: Option<String>,
    pub after_sha256: Option<String>,
    pub byte_count: u64,
    pub task_id: String,
    pub agent_id: String,
    pub blob_digest: Option<String>,
    /// Portable Unix mode metadata. Windows packages carry `None` explicitly.
    #[serde(default)]
    pub before_mode: Option<u32>,
    /// Prepared target mode for additions/modifications on Unix.
    #[serde(default)]
    pub after_mode: Option<u32>,
    /// Exact immutable preimage size for review streaming.
    #[serde(default)]
    pub before_byte_count: u64,
    /// Exact immutable target size for review streaming.
    #[serde(default)]
    pub after_byte_count: u64,
    /// `None` when there is no preimage; otherwise an honest text/binary marker.
    #[serde(default)]
    pub before_is_binary: Option<bool>,
    /// `None` for deletions; otherwise an honest text/binary marker.
    #[serde(default)]
    pub after_is_binary: Option<bool>,
    /// Digest-covered fixed chunks for bounded exact preimage reads.
    #[serde(default)]
    pub before_chunks: Vec<PreparedBlobChunk>,
    /// Digest-covered fixed chunks for bounded exact target reads.
    #[serde(default)]
    pub after_chunks: Vec<PreparedBlobChunk>,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct PreparedRunManifest {
    pub version: u32,
    pub run_id: String,
    pub base_revision: String,
    pub prepared_at: String,
    pub package_digest: String,
    pub summary: PreparedRunSummary,
    pub files: Vec<PreparedRunFile>,
    /// Absent on legacy packages; task checks never imply candidate verification.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub candidate_verification: Option<CandidateVerificationEvidence>,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct RunApplyError {
    pub at: String,
    pub code: String,
    pub message: String,
    pub attempt_id: Option<String>,
    pub rollback_confirmed: bool,
}

/// Exact included repository source. Excluded outputs and external dependencies are not attested.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct CandidateInventoryFile {
    pub path: String,
    pub sha256: String,
    pub mode: Option<u32>,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct CandidateCheckEvidence {
    pub task_id: String,
    pub command: String,
    pub effective_profile: String,
    pub passed: bool,
    pub enforcement: serde_json::Value,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct CandidateVerificationEvidence {
    pub version: u32,
    pub base_inventory: Vec<CandidateInventoryFile>,
    pub candidate_inventory: Vec<CandidateInventoryFile>,
    pub exclusions: Vec<String>,
    pub checks: Vec<CandidateCheckEvidence>,
    pub verified_at: String,
}

/// Digest of the exact manifest bytes with its digest field empty. This is
/// shared by Review publication and Apply so they cannot disagree about the
/// identity of a candidate package.
pub fn prepared_manifest_digest(manifest: &PreparedRunManifest) -> Result<String> {
    let mut unsigned = manifest.clone();
    unsigned.package_digest.clear();
    let bytes = serde_json::to_vec(&unsigned)
        .map_err(|error| PytxoError::Runner(format!("serialize prepared manifest: {error}")))?;
    Ok(Digest::of_bytes(&bytes).0)
}

/// Pure candidate evidence validation. The Runner additionally checks actual
/// repository bytes and protected paths before Apply.
pub fn require_candidate_verification_contract(
    manifest: &PreparedRunManifest,
) -> Result<&CandidateVerificationEvidence> {
    let evidence = manifest
        .candidate_verification
        .as_ref()
        .filter(|e| {
            manifest.version == 3
                && e.version == 1
                && !e.checks.is_empty()
                && e.checks.iter().all(|c| {
                    c.passed
                        && !c.command.trim().is_empty()
                        && !c.task_id.is_empty()
                        && matches!(
                            c.effective_profile.to_ascii_lowercase().as_str(),
                            "orbit" | "galaxy"
                        )
                        && c.enforcement.is_object()
                })
        })
        .ok_or_else(|| {
            PytxoError::Runner(
                "exact candidate is not verified; configure required checks and refresh review"
                    .into(),
            )
        })?;
    let mut expected = BTreeMap::new();
    for entry in &evidence.base_inventory {
        if expected.insert(entry.path.clone(), entry.clone()).is_some() {
            return Err(PytxoError::Runner(
                "candidate base inventory contains duplicate paths".into(),
            ));
        }
    }
    for file in &manifest.files {
        let before = expected.get(&file.path);
        if before.map(|f| &f.sha256) != file.before_sha256.as_ref()
            || before.and_then(|f| f.mode) != file.before_mode
        {
            return Err(PytxoError::Runner(
                "candidate base inventory disagrees with prepared preimages".into(),
            ));
        }
        if let Some(digest) = &file.after_sha256 {
            expected.insert(
                file.path.clone(),
                CandidateInventoryFile {
                    path: file.path.clone(),
                    sha256: digest.clone(),
                    mode: file.after_mode,
                },
            );
        } else {
            expected.remove(&file.path);
        }
    }
    if expected.into_values().collect::<Vec<_>>() != evidence.candidate_inventory {
        return Err(PytxoError::Runner(
            "candidate inventory does not match exact prepared composition".into(),
        ));
    }
    Ok(evidence)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn verified_manifest() -> PreparedRunManifest {
        let mut manifest = PreparedRunManifest {
            version: 3,
            run_id: "run-1".into(),
            base_revision: "base".into(),
            prepared_at: "now".into(),
            package_digest: String::new(),
            summary: PreparedRunSummary::default(),
            files: Vec::new(),
            candidate_verification: Some(CandidateVerificationEvidence {
                version: 1,
                base_inventory: Vec::new(),
                candidate_inventory: Vec::new(),
                exclusions: Vec::new(),
                checks: vec![CandidateCheckEvidence {
                    task_id: "task-1".into(),
                    command: "check".into(),
                    effective_profile: "orbit".into(),
                    passed: true,
                    enforcement: serde_json::json!({}),
                }],
                verified_at: "now".into(),
            }),
        };
        manifest.package_digest = prepared_manifest_digest(&manifest).unwrap();
        manifest
    }

    #[test]
    fn candidate_contract_requires_checks_and_binds_manifest_digest() {
        let manifest = verified_manifest();
        assert!(require_candidate_verification_contract(&manifest).is_ok());
        assert_eq!(
            prepared_manifest_digest(&manifest).unwrap(),
            manifest.package_digest
        );

        let mut changed = manifest.clone();
        changed
            .candidate_verification
            .as_mut()
            .unwrap()
            .checks
            .clear();
        assert!(require_candidate_verification_contract(&changed).is_err());
        assert_ne!(
            prepared_manifest_digest(&changed).unwrap(),
            manifest.package_digest
        );

        let mut changed = manifest;
        changed.candidate_verification.as_mut().unwrap().checks[0].passed = false;
        assert!(require_candidate_verification_contract(&changed).is_err());
    }
}
