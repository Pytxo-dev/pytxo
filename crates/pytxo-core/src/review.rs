use serde::{Deserialize, Serialize};

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
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct RunApplyError {
    pub at: String,
    pub code: String,
    pub message: String,
    pub attempt_id: Option<String>,
    pub rollback_confirmed: bool,
}
