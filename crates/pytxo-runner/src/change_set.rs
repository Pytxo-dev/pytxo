use std::collections::{BTreeMap, BTreeSet};
use std::fs::File;
use std::path::{Component, Path, PathBuf};

use pytxo_core::{
    PreparedBlobChunk, PreparedRunFile, PreparedRunFileKind, PreparedRunManifest,
    PreparedRunSummary, PytxoError, Result,
};
use sha2::{Digest, Sha256};

const REVIEW_CONTENT_CHUNK_BYTES: usize = 64 * 1024;

/// Process-scoped exclusive mutation lease for one execution domain.
///
/// The lock file is durable, while ownership is an OS file lock: a crashed
/// process cannot orphan it. Callers must hold this across preimage checks,
/// journal mutation/reconciliation, and durable contract settlement.
#[derive(Debug)]
pub struct ExecutionDomainMutationLease {
    file: File,
}

impl ExecutionDomainMutationLease {
    pub fn try_acquire(data_dir: &Path) -> Result<Self> {
        use fs2::FileExt;
        use std::io::{Seek, Write};

        let apply_root = data_dir.join("apply");
        create_dir_all_synced(&apply_root)?;
        let lock_path = apply_root.join("domain-mutation.lock");
        let mut file = std::fs::OpenOptions::new()
            .create(true)
            .truncate(false)
            .read(true)
            .write(true)
            .open(&lock_path)
            .map_err(PytxoError::Io)?;
        file.try_lock_exclusive().map_err(|error| {
            PytxoError::Runner(format!(
                "execution domain mutation is busy; retry after Apply/recovery settles: {error}"
            ))
        })?;
        file.set_len(0).map_err(PytxoError::Io)?;
        file.seek(std::io::SeekFrom::Start(0))
            .map_err(PytxoError::Io)?;
        writeln!(
            file,
            "pid={} acquired_at={}",
            std::process::id(),
            chrono::Utc::now().to_rfc3339()
        )
        .map_err(PytxoError::Io)?;
        file.sync_all().map_err(PytxoError::Io)?;
        sync_directory(&apply_root)?;
        Ok(Self { file })
    }
}

impl Drop for ExecutionDomainMutationLease {
    fn drop(&mut self) {
        let _ = fs2::FileExt::unlock(&self.file);
    }
}

#[derive(Clone, Debug)]
pub struct AgentWorkspaceInput {
    pub agent_id: String,
    pub task_id: String,
    pub workspace_path: PathBuf,
    pub claims: Vec<String>,
    pub depends_on: Vec<String>,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, serde::Deserialize, serde::Serialize)]
#[serde(rename_all = "snake_case")]
pub enum RunChangeKind {
    Add,
    Modify,
    Delete,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ReviewContentSide {
    Before,
    After,
}

impl ReviewContentSide {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Before => "before",
            Self::After => "after",
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ReviewContentChunk {
    pub digest: String,
    pub byte_count: u64,
    pub binary: bool,
    pub offset: u64,
    pub length: u64,
    pub next_offset: u64,
    pub complete: bool,
    pub bytes: Vec<u8>,
}

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct ReviewReadMetrics {
    pub bytes_read: u64,
    pub read_operations: u64,
}

/// Deterministic test seam for proving that exact-content reads fail closed.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ReviewReadFaultPoint {
    CorruptAfterOpen,
    CorruptAfterStream,
}

#[derive(Clone, Debug)]
pub struct RunChange {
    pub path: String,
    pub kind: RunChangeKind,
    pub source_agent_id: String,
    pub source_task_id: String,
    pub source_path: Option<PathBuf>,
    pub base_digest: Option<String>,
    pub result_digest: Option<String>,
    pub base_mode: Option<u32>,
    pub result_mode: Option<u32>,
}

#[derive(Clone, Debug, Default)]
pub struct PreparedRunChangeSet {
    pub changes: Vec<RunChange>,
}

#[derive(Default)]
struct FileInventory {
    files: BTreeMap<String, PathBuf>,
    symlinks: BTreeMap<String, PathBuf>,
}

#[derive(Clone, Debug, Eq, PartialEq, serde::Deserialize, serde::Serialize)]
pub struct AppliedRunChange {
    pub path: String,
    pub kind: RunChangeKind,
    pub source_agent_id: String,
    pub source_task_id: String,
    pub base_digest: Option<String>,
    pub result_digest: Option<String>,
}

#[derive(Clone, Debug, Eq, PartialEq, serde::Deserialize, serde::Serialize)]
pub struct RunApplyManifest {
    pub transaction_id: String,
    pub changes: Vec<AppliedRunChange>,
}

#[derive(Clone, Debug, serde::Deserialize, serde::Serialize)]
struct ApplyJournal {
    version: u32,
    run_id: String,
    attempt_id: String,
    created_at: String,
    phase: String,
    completed: usize,
    in_flight: Option<usize>,
    in_flight_stage: Option<JournalOperationStage>,
    operations: Vec<JournalOperation>,
    created_directories: Vec<JournalDirectory>,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ApplyFaultPoint {
    InterruptAfterJournalPrepared,
    DriftBeforeMutation(usize),
    InterruptAfterDirectoryIntent(usize),
    InterruptAfterTempWrite(usize),
    InterruptAfterDestinationRemoval(usize),
    InterruptAfterRename(usize),
    FailAfterMutation(usize),
    CorruptBlobBeforeMutation(usize),
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum RecoveryOutcome {
    NothingToDo,
    RolledBack { attempt_id: String },
    Committed(RunApplyManifest),
    RecoveryRequired { attempt_id: Option<String> },
}

#[derive(Clone, Debug, serde::Deserialize, serde::Serialize)]
struct JournalOperation {
    path: String,
    kind: PreparedRunFileKind,
    before_sha256: Option<String>,
    after_sha256: Option<String>,
    #[serde(default)]
    before_mode: Option<u32>,
    #[serde(default)]
    after_mode: Option<u32>,
    blob_digest: Option<String>,
    source_agent_id: String,
    source_task_id: String,
    temporary_path: Option<String>,
    existed: bool,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, serde::Deserialize, serde::Serialize)]
#[serde(rename_all = "snake_case")]
enum JournalOperationStage {
    Started,
    TempWritten,
    DestinationRemoved,
    Renamed,
}

#[derive(Clone, Debug, serde::Deserialize, serde::Serialize)]
struct JournalDirectory {
    path: String,
    stage: JournalDirectoryStage,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, serde::Deserialize, serde::Serialize)]
#[serde(rename_all = "snake_case")]
enum JournalDirectoryStage {
    Intent,
    Created,
}

impl ApplyJournal {
    fn apply_manifest(&self) -> RunApplyManifest {
        RunApplyManifest {
            transaction_id: self.attempt_id.clone(),
            changes: self
                .operations
                .iter()
                .map(|operation| AppliedRunChange {
                    path: operation.path.clone(),
                    kind: match operation.kind {
                        PreparedRunFileKind::Add => RunChangeKind::Add,
                        PreparedRunFileKind::Modify => RunChangeKind::Modify,
                        PreparedRunFileKind::Delete => RunChangeKind::Delete,
                    },
                    source_agent_id: operation.source_agent_id.clone(),
                    source_task_id: operation.source_task_id.clone(),
                    base_digest: operation.before_sha256.clone(),
                    result_digest: operation.after_sha256.clone(),
                })
                .collect(),
        }
    }
}

pub fn prepare_review_package(
    repo_root: &Path,
    data_dir: &Path,
    run_id: &str,
    base_revision: &str,
    workspaces: &[AgentWorkspaceInput],
    sparse_exclude: &[String],
) -> Result<PreparedRunManifest> {
    validate_run_id(run_id)?;
    let change_set = prepare_run_change_set(repo_root, workspaces, sparse_exclude)?;
    let reviews_root = data_dir.join("reviews");
    std::fs::create_dir_all(&reviews_root).map_err(PytxoError::Io)?;
    let temporary = reviews_root.join(format!(".tmp-{run_id}-{}", uuid::Uuid::new_v4()));
    let blobs = temporary.join("blobs");
    std::fs::create_dir_all(&blobs).map_err(PytxoError::Io)?;

    let result = (|| -> Result<PreparedRunManifest> {
        let mut summary = PreparedRunSummary::default();
        let mut files = Vec::with_capacity(change_set.changes.len());
        for change in &change_set.changes {
            let kind = match change.kind {
                RunChangeKind::Add => {
                    summary.added += 1;
                    PreparedRunFileKind::Add
                }
                RunChangeKind::Modify => {
                    summary.modified += 1;
                    PreparedRunFileKind::Modify
                }
                RunChangeKind::Delete => {
                    summary.deleted += 1;
                    PreparedRunFileKind::Delete
                }
            };
            let (before_byte_count, before_is_binary, before_chunks) =
                if let Some(digest) = &change.base_digest {
                    let target = repo_root.join(validated_change_path(&change.path)?);
                    let bytes = std::fs::read(&target).map_err(PytxoError::Io)?;
                    if sha256_bytes(&bytes) != *digest {
                        return Err(PytxoError::Runner(format!(
                            "repository source changed during review preparation for {}",
                            change.path
                        )));
                    }
                    let blob = blobs.join(digest);
                    if !blob.exists() {
                        write_synced(&blob, &bytes)?;
                    }
                    (
                        bytes.len() as u64,
                        Some(is_binary_content(&bytes)),
                        prepared_blob_chunks(&bytes),
                    )
                } else {
                    (0, None, vec![])
                };
            let (after_byte_count, after_is_binary, after_chunks, blob_digest) =
                if let Some(source) = &change.source_path {
                    let bytes = std::fs::read(source).map_err(PytxoError::Io)?;
                    let digest = sha256_bytes(&bytes);
                    if Some(&digest) != change.result_digest.as_ref() {
                        return Err(PytxoError::Runner(format!(
                            "agent result changed during review preparation for {}",
                            change.path
                        )));
                    }
                    let blob = blobs.join(&digest);
                    if !blob.exists() {
                        write_synced(&blob, &bytes)?;
                    }
                    summary.bytes += bytes.len() as u64;
                    (
                        bytes.len() as u64,
                        Some(is_binary_content(&bytes)),
                        prepared_blob_chunks(&bytes),
                        Some(digest),
                    )
                } else {
                    (0, None, vec![], None)
                };
            let byte_count = if blob_digest.is_some() {
                after_byte_count
            } else {
                before_byte_count
            };
            files.push(PreparedRunFile {
                path: change.path.clone(),
                kind,
                before_sha256: change.base_digest.clone(),
                after_sha256: change.result_digest.clone(),
                byte_count,
                task_id: change.source_task_id.clone(),
                agent_id: change.source_agent_id.clone(),
                blob_digest,
                before_mode: change.base_mode,
                after_mode: change.result_mode,
                before_byte_count,
                after_byte_count,
                before_is_binary,
                after_is_binary,
                before_chunks,
                after_chunks,
            });
        }
        let mut manifest = PreparedRunManifest {
            version: 2,
            run_id: run_id.to_owned(),
            base_revision: base_revision.to_owned(),
            prepared_at: chrono::Utc::now().to_rfc3339(),
            package_digest: String::new(),
            summary,
            files,
        };
        manifest.package_digest = manifest_digest(&manifest)?;
        write_synced(
            &temporary.join("manifest.json"),
            &serde_json::to_vec_pretty(&manifest)
                .map_err(|error| PytxoError::Runner(error.to_string()))?,
        )?;
        sync_directory(&blobs)?;
        sync_directory(&temporary)?;

        let final_path = reviews_root.join(run_id);
        if final_path.exists() {
            let retired = reviews_root.join(format!(".old-{run_id}-{}", uuid::Uuid::new_v4()));
            std::fs::rename(&final_path, &retired).map_err(PytxoError::Io)?;
            if let Err(error) = std::fs::rename(&temporary, &final_path) {
                let _ = std::fs::rename(&retired, &final_path);
                return Err(PytxoError::Io(error));
            }
            let _ = std::fs::remove_dir_all(retired);
        } else {
            std::fs::rename(&temporary, &final_path).map_err(PytxoError::Io)?;
        }
        sync_directory(&reviews_root)?;
        Ok(manifest)
    })();
    if result.is_err() {
        let _ = std::fs::remove_dir_all(&temporary);
    }
    result
}

pub fn load_review_manifest(data_dir: &Path, run_id: &str) -> Result<PreparedRunManifest> {
    validate_run_id(run_id)?;
    let root = data_dir.join("reviews").join(run_id);
    let bytes = std::fs::read(root.join("manifest.json")).map_err(PytxoError::Io)?;
    let manifest: PreparedRunManifest = serde_json::from_slice(&bytes)
        .map_err(|error| PytxoError::Runner(format!("invalid prepared manifest: {error}")))?;
    if manifest.run_id != run_id {
        return Err(PytxoError::Runner(
            "prepared manifest identity mismatch".into(),
        ));
    }
    if manifest.version == 1 {
        return Err(PytxoError::Runner(
            "prepared review package version 1 requires an explicit refresh for Pytxo v1.1".into(),
        ));
    }
    if manifest.version != 2 {
        return Err(PytxoError::Runner(format!(
            "unsupported prepared review package version {}; refresh the review",
            manifest.version
        )));
    }
    if manifest_digest(&manifest)? != manifest.package_digest {
        return Err(PytxoError::Runner(
            "prepared manifest digest mismatch".into(),
        ));
    }
    let mut paths = BTreeSet::new();
    for file in &manifest.files {
        validated_change_path(&file.path)?;
        if !paths.insert(file.path.as_str()) {
            return Err(PytxoError::Runner(format!(
                "prepared manifest contains duplicate path {}",
                file.path
            )));
        }
        if let Some(digest) = &file.before_sha256 {
            validate_sha256(digest)?;
            if file.before_is_binary.is_none() {
                return Err(PytxoError::Runner(format!(
                    "prepared before-review metadata is incomplete for {}",
                    file.path
                )));
            }
            validate_prepared_chunks(
                &file.before_chunks,
                file.before_byte_count,
                &file.path,
                "before",
            )?;
        } else if file.before_byte_count != 0
            || file.before_is_binary.is_some()
            || !file.before_chunks.is_empty()
        {
            return Err(PytxoError::Runner(format!(
                "prepared before-review metadata is invalid for {}",
                file.path
            )));
        }
        match (&file.kind, &file.blob_digest) {
            (PreparedRunFileKind::Delete, None)
                if file.after_sha256.is_none()
                    && file.after_byte_count == 0
                    && file.after_is_binary.is_none()
                    && file.after_chunks.is_empty()
                    && file.byte_count == file.before_byte_count => {}
            (PreparedRunFileKind::Add | PreparedRunFileKind::Modify, Some(digest)) => {
                validate_sha256(digest)?;
                if file.after_sha256.as_ref() != Some(digest)
                    || file.after_is_binary.is_none()
                    || file.byte_count != file.after_byte_count
                {
                    return Err(PytxoError::Runner(format!(
                        "prepared after-review metadata mismatch for {}",
                        file.path
                    )));
                }
                validate_prepared_chunks(
                    &file.after_chunks,
                    file.after_byte_count,
                    &file.path,
                    "after",
                )?;
            }
            _ => {
                return Err(PytxoError::Runner(format!(
                    "prepared manifest has invalid blob reference for {}",
                    file.path
                )))
            }
        }
    }
    Ok(manifest)
}

pub fn load_review_package(data_dir: &Path, run_id: &str) -> Result<PreparedRunManifest> {
    let manifest = load_review_manifest(data_dir, run_id)?;
    for file in &manifest.files {
        for chunk in &file.before_chunks {
            read_review_content_chunk(
                data_dir,
                &manifest,
                &file.path,
                ReviewContentSide::Before,
                chunk.offset,
                REVIEW_CONTENT_CHUNK_BYTES,
            )?;
        }
        for chunk in &file.after_chunks {
            read_review_content_chunk(
                data_dir,
                &manifest,
                &file.path,
                ReviewContentSide::After,
                chunk.offset,
                REVIEW_CONTENT_CHUNK_BYTES,
            )?;
        }
    }
    Ok(manifest)
}

pub fn read_review_content_chunk(
    data_dir: &Path,
    manifest: &PreparedRunManifest,
    path: &str,
    side: ReviewContentSide,
    offset: u64,
    limit: usize,
) -> Result<ReviewContentChunk> {
    read_review_content_chunk_internal(data_dir, manifest, path, side, offset, limit, None, None)
}

pub fn read_review_content_chunk_with_fault(
    data_dir: &Path,
    manifest: &PreparedRunManifest,
    path: &str,
    side: ReviewContentSide,
    offset: u64,
    limit: usize,
    fault: Option<ReviewReadFaultPoint>,
) -> Result<ReviewContentChunk> {
    read_review_content_chunk_internal(data_dir, manifest, path, side, offset, limit, fault, None)
}

pub fn read_review_content_chunk_with_metrics(
    data_dir: &Path,
    manifest: &PreparedRunManifest,
    path: &str,
    side: ReviewContentSide,
    offset: u64,
    limit: usize,
    metrics: &mut ReviewReadMetrics,
) -> Result<ReviewContentChunk> {
    read_review_content_chunk_internal(
        data_dir,
        manifest,
        path,
        side,
        offset,
        limit,
        None,
        Some(metrics),
    )
}

#[allow(clippy::too_many_arguments)]
fn read_review_content_chunk_internal(
    data_dir: &Path,
    manifest: &PreparedRunManifest,
    path: &str,
    side: ReviewContentSide,
    offset: u64,
    limit: usize,
    fault: Option<ReviewReadFaultPoint>,
    mut metrics: Option<&mut ReviewReadMetrics>,
) -> Result<ReviewContentChunk> {
    use std::io::{Read, Seek};

    if limit == 0 || limit > REVIEW_CONTENT_CHUNK_BYTES {
        return Err(PytxoError::Runner(format!(
            "review content chunk limit must be 1..={REVIEW_CONTENT_CHUNK_BYTES}"
        )));
    }
    validate_run_id(&manifest.run_id)?;
    if manifest.version != 2 || manifest_digest(manifest)? != manifest.package_digest {
        return Err(PytxoError::Runner(
            "prepared manifest changed before exact-content read".into(),
        ));
    }
    let file = manifest
        .files
        .iter()
        .find(|candidate| candidate.path == path)
        .ok_or_else(|| PytxoError::Runner("path is not in the prepared package".into()))?;
    let (digest, byte_count, binary, chunks) = match side {
        ReviewContentSide::Before => (
            file.before_sha256.as_ref(),
            file.before_byte_count,
            file.before_is_binary,
            file.before_chunks.as_slice(),
        ),
        ReviewContentSide::After => (
            file.after_sha256.as_ref(),
            file.after_byte_count,
            file.after_is_binary,
            file.after_chunks.as_slice(),
        ),
    };
    let digest = digest.ok_or_else(|| {
        PytxoError::Runner(format!(
            "{} content does not exist for {path}",
            side.as_str()
        ))
    })?;
    validate_sha256(digest)?;
    let binary = binary.ok_or_else(|| {
        PytxoError::Runner(format!(
            "{} content metadata is unavailable for {path}",
            side.as_str()
        ))
    })?;
    if offset > byte_count {
        return Err(PytxoError::Runner(
            "content offset is past the exact prepared file".into(),
        ));
    }

    if byte_count == 0 {
        return Ok(ReviewContentChunk {
            digest: digest.clone(),
            byte_count,
            binary,
            offset,
            length: 0,
            next_offset: 0,
            complete: true,
            bytes: vec![],
        });
    }
    let prepared_chunk = chunks
        .iter()
        .find(|chunk| chunk.offset == offset)
        .ok_or_else(|| {
            PytxoError::Runner(format!(
                "content offset is not a digest-covered chunk boundary for {path}"
            ))
        })?;
    if prepared_chunk.length > limit as u64 {
        return Err(PytxoError::Runner(format!(
            "content limit is smaller than the digest-covered chunk for {path}"
        )));
    }

    let blob_path = data_dir
        .join("reviews")
        .join(&manifest.run_id)
        .join("blobs")
        .join(digest);
    let mut blob = File::open(&blob_path).map_err(PytxoError::Io)?;
    if fault == Some(ReviewReadFaultPoint::CorruptAfterOpen) {
        std::fs::write(&blob_path, b"injected review read corruption").map_err(PytxoError::Io)?;
    }
    blob.seek(std::io::SeekFrom::Start(offset))
        .map_err(PytxoError::Io)?;
    let mut bytes = vec![0_u8; prepared_chunk.length as usize];
    let mut filled = 0_usize;
    if let Some(metrics) = metrics.as_deref_mut() {
        metrics.read_operations += 1;
    }
    while filled < bytes.len() {
        let read = blob.read(&mut bytes[filled..]).map_err(PytxoError::Io)?;
        if read == 0 {
            break;
        }
        filled += read;
        if let Some(metrics) = metrics.as_deref_mut() {
            metrics.bytes_read += read as u64;
        }
    }
    bytes.truncate(filled);
    if bytes.len() as u64 != prepared_chunk.length || sha256_bytes(&bytes) != prepared_chunk.sha256
    {
        return Err(PytxoError::Runner(format!(
            "prepared {}-chunk digest mismatch for {path}",
            side.as_str()
        )));
    }
    if fault == Some(ReviewReadFaultPoint::CorruptAfterStream) {
        std::fs::write(&blob_path, b"injected post-stream corruption").map_err(PytxoError::Io)?;
    }
    let next_offset = offset + prepared_chunk.length;
    Ok(ReviewContentChunk {
        digest: digest.clone(),
        byte_count,
        binary,
        offset,
        length: bytes.len() as u64,
        next_offset,
        complete: next_offset == byte_count,
        bytes,
    })
}

pub fn apply_prepared_review(
    repo_root: &Path,
    data_dir: &Path,
    manifest: &PreparedRunManifest,
) -> Result<RunApplyManifest> {
    let lease = ExecutionDomainMutationLease::try_acquire(data_dir)?;
    apply_prepared_review_with_fault_under_lease(repo_root, data_dir, manifest, &lease, None)
}

pub fn apply_prepared_review_with_fault(
    repo_root: &Path,
    data_dir: &Path,
    manifest: &PreparedRunManifest,
    fault: Option<ApplyFaultPoint>,
) -> Result<RunApplyManifest> {
    let lease = ExecutionDomainMutationLease::try_acquire(data_dir)?;
    apply_prepared_review_with_fault_under_lease(repo_root, data_dir, manifest, &lease, fault)
}

pub fn apply_prepared_review_under_lease(
    repo_root: &Path,
    data_dir: &Path,
    manifest: &PreparedRunManifest,
    lease: &ExecutionDomainMutationLease,
) -> Result<RunApplyManifest> {
    apply_prepared_review_with_fault_under_lease(repo_root, data_dir, manifest, lease, None)
}

pub fn apply_prepared_review_with_fault_under_lease(
    repo_root: &Path,
    data_dir: &Path,
    manifest: &PreparedRunManifest,
    _lease: &ExecutionDomainMutationLease,
    fault: Option<ApplyFaultPoint>,
) -> Result<RunApplyManifest> {
    let durable = load_review_package(data_dir, &manifest.run_id)?;
    if &durable != manifest {
        return Err(PytxoError::Runner(
            "requested manifest differs from durable review package".into(),
        ));
    }
    verify_manifest_preimages(repo_root, manifest)?;
    let attempt_id = uuid::Uuid::new_v4().to_string();
    let attempt_root = data_dir
        .join("apply")
        .join(&manifest.run_id)
        .join(&attempt_id);
    let backup_root = attempt_root.join("backups");
    create_dir_all_synced(&backup_root)?;
    let operations = manifest
        .files
        .iter()
        .enumerate()
        .map(|(index, file)| {
            let relative = validated_change_path(&file.path)?;
            Ok(JournalOperation {
                path: file.path.clone(),
                kind: file.kind,
                before_sha256: file.before_sha256.clone(),
                after_sha256: file.after_sha256.clone(),
                before_mode: file.before_mode,
                after_mode: file.after_mode,
                blob_digest: file.blob_digest.clone(),
                source_agent_id: file.agent_id.clone(),
                source_task_id: file.task_id.clone(),
                temporary_path: (file.kind != PreparedRunFileKind::Delete).then(|| {
                    let name = format!(".pytxo-apply-{attempt_id}-{index}.tmp");
                    normalize_relative(&relative.with_file_name(name))
                }),
                existed: file.before_sha256.is_some(),
            })
        })
        .collect::<Result<Vec<_>>>()?;
    for operation in &operations {
        if operation.existed {
            let relative = validated_change_path(&operation.path)?;
            let backup = backup_root.join(&relative);
            if let Some(parent) = backup.parent() {
                create_dir_all_synced(parent)?;
            }
            let bytes = std::fs::read(repo_root.join(&relative)).map_err(PytxoError::Io)?;
            write_synced(&backup, &bytes)?;
        }
    }
    sync_directory(&backup_root)?;
    let mut journal = ApplyJournal {
        version: 1,
        run_id: manifest.run_id.clone(),
        attempt_id: attempt_id.clone(),
        created_at: chrono::Utc::now().to_rfc3339(),
        phase: "prepared".into(),
        completed: 0,
        in_flight: None,
        in_flight_stage: None,
        operations,
        created_directories: Vec::new(),
    };
    persist_journal(&attempt_root, &journal)?;
    if fault == Some(ApplyFaultPoint::InterruptAfterJournalPrepared) {
        return Err(PytxoError::Runner(
            "injected interruption after journal preparation".into(),
        ));
    }
    journal.phase = "mutating".into();
    persist_journal(&attempt_root, &journal)?;

    for index in 0..journal.operations.len() {
        journal.in_flight = Some(index);
        journal.in_flight_stage = Some(JournalOperationStage::Started);
        persist_journal(&attempt_root, &journal)?;
        let operation = journal.operations[index].clone();
        if fault == Some(ApplyFaultPoint::CorruptBlobBeforeMutation(index + 1)) {
            if let Some(digest) = &operation.blob_digest {
                std::fs::write(
                    data_dir
                        .join("reviews")
                        .join(&manifest.run_id)
                        .join("blobs")
                        .join(digest),
                    b"injected post-validation blob corruption\n",
                )
                .map_err(PytxoError::Io)?;
            }
        }
        if fault == Some(ApplyFaultPoint::DriftBeforeMutation(index + 1)) {
            let relative = validated_change_path(&operation.path)?;
            let target = repo_root.join(relative);
            if let Some(parent) = target.parent() {
                create_dir_all_synced(parent)?;
            }
            std::fs::write(&target, b"injected source drift\n").map_err(PytxoError::Io)?;
            std::fs::OpenOptions::new()
                .write(true)
                .open(&target)
                .and_then(|file| file.sync_all())
                .map_err(PytxoError::Io)?;
        }
        if let Err(error) = verify_operation_preimage(repo_root, &operation) {
            journal.in_flight = None;
            journal.in_flight_stage = None;
            persist_journal(&attempt_root, &journal)?;
            let rollback = rollback_journal(repo_root, &attempt_root, &mut journal);
            return match rollback {
                Ok(()) => Err(error),
                Err(rollback_error) => Err(PytxoError::Runner(format!(
                    "{error}; rollback after preimage drift failed: {rollback_error}"
                ))),
            };
        }
        let interrupted = match apply_journal_operation(
            repo_root,
            data_dir,
            &manifest.run_id,
            &attempt_root,
            &mut journal,
            index,
            fault,
        ) {
            Ok(interrupted) => interrupted,
            Err(error) => {
                let rollback = rollback_journal(repo_root, &attempt_root, &mut journal);
                return match rollback {
                    Ok(()) => Err(PytxoError::Runner(format!(
                        "prepared run apply failed and was rolled back: {error}"
                    ))),
                    Err(rollback_error) => Err(PytxoError::Runner(format!(
                        "prepared run apply failed: {error}; rollback failed: {rollback_error}"
                    ))),
                };
            }
        };
        if interrupted {
            return Err(PytxoError::Runner(format!(
                "injected interruption during mutation {}",
                index + 1
            )));
        }
        journal.completed = index + 1;
        journal.in_flight = None;
        journal.in_flight_stage = None;
        persist_journal(&attempt_root, &journal)?;
        if fault == Some(ApplyFaultPoint::FailAfterMutation(index + 1)) {
            let injected = PytxoError::Runner(format!(
                "injected apply failure after mutation {}",
                index + 1
            ));
            rollback_journal(repo_root, &attempt_root, &mut journal)?;
            return Err(injected);
        }
    }
    journal.phase = "committed".into();
    persist_journal(&attempt_root, &journal)?;
    Ok(journal.apply_manifest())
}

pub fn reconcile_apply_journals(
    repo_root: &Path,
    data_dir: &Path,
    run_id: &str,
) -> Result<RecoveryOutcome> {
    let lease = ExecutionDomainMutationLease::try_acquire(data_dir)?;
    reconcile_apply_journals_under_lease(repo_root, data_dir, run_id, &lease)
}

pub fn reconcile_apply_journals_under_lease(
    repo_root: &Path,
    data_dir: &Path,
    run_id: &str,
    _lease: &ExecutionDomainMutationLease,
) -> Result<RecoveryOutcome> {
    validate_run_id(run_id)?;
    let run_root = data_dir.join("apply").join(run_id);
    if !run_root.exists() {
        return Ok(RecoveryOutcome::NothingToDo);
    }
    let mut attempts = std::fs::read_dir(&run_root)
        .map_err(PytxoError::Io)?
        .collect::<std::result::Result<Vec<_>, _>>()
        .map_err(PytxoError::Io)?;
    attempts.sort_by_key(|entry| entry.file_name());
    let mut recovered_attempt = None;
    let mut committed_manifest = None;
    for entry in attempts {
        let attempt_root = entry.path();
        if !attempt_root.is_dir() {
            continue;
        }
        let bytes = match std::fs::read(attempt_root.join("journal.json")) {
            Ok(bytes) => bytes,
            Err(_) => {
                return Ok(RecoveryOutcome::RecoveryRequired {
                    attempt_id: entry.file_name().to_str().map(str::to_owned),
                })
            }
        };
        let mut journal: ApplyJournal = match serde_json::from_slice(&bytes) {
            Ok(journal) => journal,
            Err(_) => {
                return Ok(RecoveryOutcome::RecoveryRequired {
                    attempt_id: entry.file_name().to_str().map(str::to_owned),
                })
            }
        };
        if journal.run_id != run_id || journal.version != 1 {
            return Ok(RecoveryOutcome::RecoveryRequired {
                attempt_id: Some(journal.attempt_id),
            });
        }
        match journal.phase.as_str() {
            "committed" => committed_manifest = Some(journal.apply_manifest()),
            "rolled_back" => continue,
            "prepared" | "mutating" => {
                let attempt_id = journal.attempt_id.clone();
                if rollback_journal(repo_root, &attempt_root, &mut journal).is_err() {
                    return Ok(RecoveryOutcome::RecoveryRequired {
                        attempt_id: Some(attempt_id),
                    });
                }
                match journal_preimages_restored(repo_root, &journal) {
                    Ok(true) => recovered_attempt = Some(attempt_id),
                    Ok(false) | Err(_) => {
                        return Ok(RecoveryOutcome::RecoveryRequired {
                            attempt_id: Some(attempt_id),
                        })
                    }
                }
            }
            _ => {
                return Ok(RecoveryOutcome::RecoveryRequired {
                    attempt_id: Some(journal.attempt_id),
                })
            }
        }
    }
    Ok(if let Some(attempt_id) = recovered_attempt {
        RecoveryOutcome::RolledBack { attempt_id }
    } else if let Some(manifest) = committed_manifest {
        RecoveryOutcome::Committed(manifest)
    } else {
        RecoveryOutcome::NothingToDo
    })
}

pub fn apply_attempt_ids(data_dir: &Path, run_id: &str) -> Result<BTreeSet<String>> {
    validate_run_id(run_id)?;
    let run_root = data_dir.join("apply").join(run_id);
    if !run_root.exists() {
        return Ok(BTreeSet::new());
    }
    let attempts = std::fs::read_dir(run_root)
        .map_err(PytxoError::Io)?
        .collect::<std::result::Result<Vec<_>, _>>()
        .map_err(PytxoError::Io)?;
    let mut attempt_ids = BTreeSet::new();
    for entry in attempts {
        if entry.file_type().map_err(PytxoError::Io)?.is_dir() {
            attempt_ids.insert(entry.file_name().to_string_lossy().into_owned());
        }
    }
    Ok(attempt_ids)
}

pub fn prepare_run_change_set(
    repo_root: &Path,
    workspaces: &[AgentWorkspaceInput],
    sparse_exclude: &[String],
) -> Result<PreparedRunChangeSet> {
    let base_inventory = collect_inventory(repo_root, sparse_exclude)?;
    let task_dependencies = task_dependencies(workspaces)?;
    let mut changes = BTreeMap::<String, RunChange>::new();

    for workspace in workspaces {
        validate_claims(&workspace.claims)?;
        let workspace_inventory = collect_inventory(&workspace.workspace_path, sparse_exclude)?;
        let paths: BTreeSet<_> = base_inventory
            .files
            .keys()
            .chain(workspace_inventory.files.keys())
            .chain(base_inventory.symlinks.keys())
            .chain(workspace_inventory.symlinks.keys())
            .cloned()
            .collect();

        for path in paths {
            let base_symlink = base_inventory.symlinks.get(&path);
            let workspace_symlink = workspace_inventory.symlinks.get(&path);
            if base_symlink.is_some() || workspace_symlink.is_some() {
                return Err(PytxoError::Runner(format!(
                    "run Apply does not support symlink path {path}"
                )));
            }

            let before = base_inventory.files.get(&path);
            let after = workspace_inventory.files.get(&path);
            let kind = match (before, after) {
                (None, Some(_)) => Some(RunChangeKind::Add),
                (Some(_), None) => Some(RunChangeKind::Delete),
                (Some(before), Some(after)) => {
                    let before_bytes = std::fs::read(before).map_err(PytxoError::Io)?;
                    let after_bytes = std::fs::read(after).map_err(PytxoError::Io)?;
                    (before_bytes != after_bytes || file_mode(before)? != file_mode(after)?)
                        .then_some(RunChangeKind::Modify)
                }
                (None, None) => None,
            };
            if let Some(kind) = kind {
                let candidate = RunChange {
                    path,
                    kind,
                    source_agent_id: workspace.agent_id.clone(),
                    source_task_id: workspace.task_id.clone(),
                    source_path: after.cloned(),
                    base_digest: before.map(|path| file_digest(path)).transpose()?,
                    result_digest: after.map(|path| file_digest(path)).transpose()?,
                    base_mode: before.map(|path| file_mode(path)).transpose()?.flatten(),
                    result_mode: after.map(|path| file_mode(path)).transpose()?.flatten(),
                };
                let is_claimed = workspace
                    .claims
                    .iter()
                    .any(|claim| path_matches_claim(&candidate.path, claim));
                if !is_claimed {
                    if let Some(existing) = changes.get(&candidate.path) {
                        let is_dependency_output = task_depends_on(
                            &candidate.source_task_id,
                            &existing.source_task_id,
                            &task_dependencies,
                        );
                        if is_dependency_output && changes_are_equivalent(existing, &candidate)? {
                            continue;
                        }
                    }
                    return Err(PytxoError::Runner(format!(
                        "agent {} edited {} outside declared claims",
                        workspace.agent_id, candidate.path
                    )));
                }
                match changes.get(&candidate.path) {
                    None => {
                        changes.insert(candidate.path.clone(), candidate);
                    }
                    Some(existing)
                        if changes_are_equivalent(existing, &candidate)?
                            && (task_depends_on(
                                &candidate.source_task_id,
                                &existing.source_task_id,
                                &task_dependencies,
                            ) || task_depends_on(
                                &existing.source_task_id,
                                &candidate.source_task_id,
                                &task_dependencies,
                            )) => {}
                    Some(existing)
                        if task_depends_on(
                            &candidate.source_task_id,
                            &existing.source_task_id,
                            &task_dependencies,
                        ) =>
                    {
                        changes.insert(candidate.path.clone(), candidate);
                    }
                    Some(existing)
                        if task_depends_on(
                            &existing.source_task_id,
                            &candidate.source_task_id,
                            &task_dependencies,
                        ) => {}
                    Some(existing) => {
                        return Err(PytxoError::Runner(format!(
                            "run change conflict on {} between independent tasks {} and {}",
                            candidate.path, existing.source_task_id, candidate.source_task_id
                        )));
                    }
                }
            }
        }
    }

    Ok(PreparedRunChangeSet {
        changes: changes.into_values().collect(),
    })
}

fn validate_run_id(run_id: &str) -> Result<()> {
    if run_id.is_empty()
        || run_id
            .chars()
            .any(|character| !(character.is_ascii_alphanumeric() || matches!(character, '-' | '_')))
    {
        return Err(PytxoError::Runner(format!(
            "unsafe review run id: {run_id}"
        )));
    }
    Ok(())
}

fn manifest_digest(manifest: &PreparedRunManifest) -> Result<String> {
    let mut unsigned = manifest.clone();
    unsigned.package_digest.clear();
    let bytes =
        serde_json::to_vec(&unsigned).map_err(|error| PytxoError::Runner(error.to_string()))?;
    Ok(sha256_bytes(&bytes))
}

fn sha256_bytes(bytes: &[u8]) -> String {
    format!("{:x}", Sha256::digest(bytes))
}

fn prepared_blob_chunks(bytes: &[u8]) -> Vec<PreparedBlobChunk> {
    bytes
        .chunks(REVIEW_CONTENT_CHUNK_BYTES)
        .enumerate()
        .map(|(index, chunk)| PreparedBlobChunk {
            offset: (index * REVIEW_CONTENT_CHUNK_BYTES) as u64,
            length: chunk.len() as u64,
            sha256: sha256_bytes(chunk),
        })
        .collect()
}

fn is_binary_content(bytes: &[u8]) -> bool {
    bytes.contains(&0) || std::str::from_utf8(bytes).is_err()
}

fn validate_sha256(digest: &str) -> Result<()> {
    if digest.len() != 64 || !digest.bytes().all(|byte| byte.is_ascii_hexdigit()) {
        return Err(PytxoError::Runner(format!(
            "invalid content-addressed blob digest: {digest}"
        )));
    }
    Ok(())
}

fn validate_prepared_chunks(
    chunks: &[PreparedBlobChunk],
    byte_count: u64,
    path: &str,
    side: &str,
) -> Result<()> {
    if byte_count == 0 {
        if chunks.is_empty() {
            return Ok(());
        }
    } else if chunks.is_empty() {
        return Err(PytxoError::Runner(format!(
            "prepared {side}-review for {path} has no digest-covered chunks; refresh the review"
        )));
    }
    let mut expected_offset = 0_u64;
    for (index, chunk) in chunks.iter().enumerate() {
        validate_sha256(&chunk.sha256)?;
        let final_chunk = index + 1 == chunks.len();
        if chunk.offset != expected_offset
            || chunk.length == 0
            || chunk.length > REVIEW_CONTENT_CHUNK_BYTES as u64
            || (!final_chunk && chunk.length != REVIEW_CONTENT_CHUNK_BYTES as u64)
        {
            return Err(PytxoError::Runner(format!(
                "prepared {side}-review chunk metadata mismatch for {path}; refresh the review"
            )));
        }
        expected_offset = expected_offset.checked_add(chunk.length).ok_or_else(|| {
            PytxoError::Runner(format!(
                "prepared {side}-review chunk length overflow for {path}"
            ))
        })?;
    }
    if expected_offset != byte_count {
        return Err(PytxoError::Runner(format!(
            "prepared {side}-review chunks do not cover {path}; refresh the review"
        )));
    }
    Ok(())
}

fn write_synced(path: &Path, bytes: &[u8]) -> Result<()> {
    if let Some(parent) = path.parent() {
        create_dir_all_synced(parent)?;
    }
    let mut options = std::fs::OpenOptions::new();
    options.create_new(true).write(true);
    let mut file = options.open(path).map_err(PytxoError::Io)?;
    std::io::Write::write_all(&mut file, bytes).map_err(PytxoError::Io)?;
    file.sync_all().map_err(PytxoError::Io)?;
    drop(file);
    if let Some(parent) = path.parent() {
        sync_directory(parent)?;
    }
    Ok(())
}

fn replace_synced(path: &Path, bytes: &[u8]) -> Result<()> {
    let temporary = path.with_extension(format!("tmp-{}", uuid::Uuid::new_v4()));
    write_synced(&temporary, bytes)?;
    std::fs::rename(&temporary, path).map_err(PytxoError::Io)?;
    if let Some(parent) = path.parent() {
        sync_directory(parent)?;
    }
    Ok(())
}

fn sync_directory(path: &Path) -> Result<()> {
    #[cfg(unix)]
    {
        std::fs::File::open(path)
            .and_then(|directory| directory.sync_all())
            .map_err(PytxoError::Io)?;
    }
    #[cfg(windows)]
    {
        use std::os::windows::fs::OpenOptionsExt;

        const FILE_FLAG_BACKUP_SEMANTICS: u32 = 0x0200_0000;
        // Windows directory flush support varies by filesystem and driver. We
        // make a best-effort flush for process-crash durability, but this is
        // deliberately not a claim of power-loss ACID behavior.
        if let Ok(directory) = std::fs::OpenOptions::new()
            .read(true)
            .custom_flags(FILE_FLAG_BACKUP_SEMANTICS)
            .open(path)
        {
            let _ = directory.sync_all();
        }
    }
    Ok(())
}

fn create_dir_all_synced(path: &Path) -> Result<()> {
    let mut missing = Vec::new();
    let mut cursor = path;
    while !cursor.exists() {
        missing.push(cursor.to_path_buf());
        cursor = cursor.parent().ok_or_else(|| {
            PytxoError::Runner(format!(
                "directory has no existing ancestor: {}",
                path.display()
            ))
        })?;
    }
    if !cursor.is_dir() {
        return Err(PytxoError::Runner(format!(
            "directory ancestor is not a directory: {}",
            cursor.display()
        )));
    }
    for directory in missing.into_iter().rev() {
        std::fs::create_dir(&directory).map_err(PytxoError::Io)?;
        if let Some(parent) = directory.parent() {
            sync_directory(parent)?;
        }
        sync_directory(&directory)?;
    }
    Ok(())
}

fn current_path_digest(path: &Path) -> Result<Option<String>> {
    match std::fs::symlink_metadata(path) {
        Ok(metadata) if metadata.is_file() && !metadata.file_type().is_symlink() => {
            Ok(Some(file_digest(path)?))
        }
        Ok(_) => Err(PytxoError::Runner(format!(
            "apply target is not a regular file: {}",
            path.display()
        ))),
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(None),
        Err(error) => Err(PytxoError::Io(error)),
    }
}

fn current_path_mode(path: &Path) -> Result<Option<u32>> {
    match std::fs::symlink_metadata(path) {
        Ok(metadata) if metadata.is_file() && !metadata.file_type().is_symlink() => file_mode(path),
        Ok(_) => Err(PytxoError::Runner(format!(
            "apply target is not a regular file: {}",
            path.display()
        ))),
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(None),
        Err(error) => Err(PytxoError::Io(error)),
    }
}

fn file_mode(path: &Path) -> Result<Option<u32>> {
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let metadata = std::fs::symlink_metadata(path).map_err(PytxoError::Io)?;
        Ok(Some(metadata.permissions().mode() & 0o7777))
    }
    #[cfg(windows)]
    {
        let _ = path;
        Ok(None)
    }
}

fn set_file_mode(path: &Path, mode: Option<u32>) -> Result<()> {
    #[cfg(unix)]
    if let Some(mode) = mode {
        use std::os::unix::fs::PermissionsExt;
        std::fs::set_permissions(path, std::fs::Permissions::from_mode(mode))
            .map_err(PytxoError::Io)?;
    }
    #[cfg(windows)]
    {
        let _ = (path, mode);
    }
    Ok(())
}

fn verify_manifest_preimages(repo_root: &Path, manifest: &PreparedRunManifest) -> Result<()> {
    for file in &manifest.files {
        let relative = validated_change_path(&file.path)?;
        validate_target_ancestry(repo_root, &relative)?;
        let target = repo_root.join(relative);
        if current_path_digest(&target)? != file.before_sha256
            || current_path_mode(&target)? != file.before_mode
        {
            return Err(PytxoError::Runner(format!(
                "{} changed since review; refresh the stale review",
                file.path
            )));
        }
    }
    Ok(())
}

fn verify_operation_preimage(repo_root: &Path, operation: &JournalOperation) -> Result<()> {
    let relative = validated_change_path(&operation.path)?;
    validate_target_ancestry(repo_root, &relative)?;
    let target = repo_root.join(relative);
    if current_path_digest(&target)? != operation.before_sha256
        || current_path_mode(&target)? != operation.before_mode
    {
        return Err(PytxoError::Runner(format!(
            "{} changed since review immediately before mutation",
            operation.path
        )));
    }
    Ok(())
}

fn validate_target_ancestry(repo_root: &Path, relative: &Path) -> Result<()> {
    let mut current = repo_root.to_path_buf();
    let components = relative.components().collect::<Vec<_>>();
    for component in components.iter().take(components.len().saturating_sub(1)) {
        let Component::Normal(component) = component else {
            return Err(PytxoError::Runner("unsafe apply path ancestry".into()));
        };
        current.push(component);
        match std::fs::symlink_metadata(&current) {
            Ok(metadata) if metadata.is_dir() && !metadata.file_type().is_symlink() => {}
            Ok(_) => {
                return Err(PytxoError::Runner(format!(
                    "apply path parent is not a regular directory: {}",
                    current.display()
                )))
            }
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => break,
            Err(error) => return Err(PytxoError::Io(error)),
        }
    }
    Ok(())
}

fn persist_journal(attempt_root: &Path, journal: &ApplyJournal) -> Result<()> {
    create_dir_all_synced(attempt_root)?;
    let bytes = serde_json::to_vec_pretty(journal)
        .map_err(|error| PytxoError::Runner(error.to_string()))?;
    replace_synced(&attempt_root.join("journal.json"), &bytes)
}

fn apply_journal_operation(
    repo_root: &Path,
    data_dir: &Path,
    run_id: &str,
    attempt_root: &Path,
    journal: &mut ApplyJournal,
    index: usize,
    fault: Option<ApplyFaultPoint>,
) -> Result<bool> {
    let operation = journal.operations[index].clone();
    let relative = validated_change_path(&operation.path)?;
    validate_target_ancestry(repo_root, &relative)?;
    let target = repo_root.join(&relative);
    match operation.kind {
        PreparedRunFileKind::Delete => {
            std::fs::remove_file(&target).map_err(PytxoError::Io)?;
            if let Some(parent) = target.parent() {
                sync_directory(parent)?;
            }
            journal.in_flight_stage = Some(JournalOperationStage::DestinationRemoved);
            persist_journal(attempt_root, journal)?;
            if fault == Some(ApplyFaultPoint::InterruptAfterDestinationRemoval(index + 1)) {
                return Ok(true);
            }
        }
        PreparedRunFileKind::Add | PreparedRunFileKind::Modify => {
            if let Some(parent) = relative.parent() {
                if create_apply_directories(repo_root, parent, attempt_root, journal, index, fault)?
                {
                    return Ok(true);
                }
            }
            let digest = operation.blob_digest.as_ref().ok_or_else(|| {
                PytxoError::Runner(format!("missing prepared blob for {}", operation.path))
            })?;
            let bytes = std::fs::read(
                data_dir
                    .join("reviews")
                    .join(run_id)
                    .join("blobs")
                    .join(digest),
            )
            .map_err(PytxoError::Io)?;
            if sha256_bytes(&bytes) != *digest || operation.after_sha256.as_ref() != Some(digest) {
                return Err(PytxoError::Runner(format!(
                    "prepared blob digest mismatch for {} immediately before mutation",
                    operation.path
                )));
            }
            let temporary_relative =
                validated_change_path(operation.temporary_path.as_deref().ok_or_else(|| {
                    PytxoError::Runner(format!("missing journal temp path for {}", operation.path))
                })?)?;
            validate_target_ancestry(repo_root, &temporary_relative)?;
            let temporary = repo_root.join(temporary_relative);
            write_synced(&temporary, &bytes)?;
            set_file_mode(&temporary, operation.after_mode)?;
            journal.in_flight_stage = Some(JournalOperationStage::TempWritten);
            persist_journal(attempt_root, journal)?;
            if fault == Some(ApplyFaultPoint::InterruptAfterTempWrite(index + 1)) {
                return Ok(true);
            }
            if target.exists() {
                std::fs::remove_file(&target).map_err(PytxoError::Io)?;
            }
            if let Some(parent) = target.parent() {
                sync_directory(parent)?;
            }
            journal.in_flight_stage = Some(JournalOperationStage::DestinationRemoved);
            persist_journal(attempt_root, journal)?;
            if fault == Some(ApplyFaultPoint::InterruptAfterDestinationRemoval(index + 1)) {
                return Ok(true);
            }
            std::fs::rename(&temporary, &target).map_err(PytxoError::Io)?;
            if let Some(parent) = target.parent() {
                sync_directory(parent)?;
            }
            journal.in_flight_stage = Some(JournalOperationStage::Renamed);
            persist_journal(attempt_root, journal)?;
            if fault == Some(ApplyFaultPoint::InterruptAfterRename(index + 1)) {
                return Ok(true);
            }
        }
    }
    Ok(false)
}

fn create_apply_directories(
    repo_root: &Path,
    relative_parent: &Path,
    attempt_root: &Path,
    journal: &mut ApplyJournal,
    operation_index: usize,
    fault: Option<ApplyFaultPoint>,
) -> Result<bool> {
    if relative_parent.as_os_str().is_empty() {
        return Ok(false);
    }
    validate_target_ancestry(repo_root, &relative_parent.join(".sentinel"))?;
    let mut missing = Vec::new();
    let mut cursor = relative_parent;
    while !cursor.as_os_str().is_empty() && !repo_root.join(cursor).exists() {
        missing.push(cursor.to_path_buf());
        cursor = cursor.parent().unwrap_or_else(|| Path::new(""));
    }
    for relative in missing.into_iter().rev() {
        let directory = repo_root.join(&relative);
        journal.created_directories.push(JournalDirectory {
            path: normalize_relative(&relative),
            stage: JournalDirectoryStage::Intent,
        });
        persist_journal(attempt_root, journal)?;
        if fault
            == Some(ApplyFaultPoint::InterruptAfterDirectoryIntent(
                operation_index + 1,
            ))
        {
            return Ok(true);
        }
        match std::fs::create_dir(&directory) {
            Ok(()) => {
                if let Some(parent) = directory.parent() {
                    sync_directory(parent)?;
                }
                sync_directory(&directory)?;
                journal
                    .created_directories
                    .last_mut()
                    .expect("directory intent exists")
                    .stage = JournalDirectoryStage::Created;
                persist_journal(attempt_root, journal)?;
            }
            Err(error) if error.kind() == std::io::ErrorKind::AlreadyExists => {
                let metadata = std::fs::symlink_metadata(&directory).map_err(PytxoError::Io)?;
                if !metadata.is_dir() || metadata.file_type().is_symlink() {
                    return Err(PytxoError::Runner(format!(
                        "apply parent is not a regular directory: {}",
                        directory.display()
                    )));
                }
            }
            Err(error) => return Err(PytxoError::Io(error)),
        }
    }
    Ok(false)
}

fn rollback_journal(
    repo_root: &Path,
    attempt_root: &Path,
    journal: &mut ApplyJournal,
) -> Result<()> {
    let touched = journal
        .in_flight
        .map_or(journal.completed, |index| journal.completed.max(index + 1));
    validate_rollback_state(repo_root, attempt_root, journal, touched)?;

    for operation in journal.operations[..touched].iter().rev() {
        let relative = validated_change_path(&operation.path)?;
        validate_target_ancestry(repo_root, &relative)?;
        let target = repo_root.join(&relative);
        if let Some(temporary) = &operation.temporary_path {
            let temporary_relative = validated_change_path(temporary)?;
            validate_target_ancestry(repo_root, &temporary_relative)?;
            let temporary = repo_root.join(temporary_relative);
            if current_path_digest(&temporary)?.is_some() {
                std::fs::remove_file(&temporary).map_err(PytxoError::Io)?;
                if let Some(parent) = temporary.parent() {
                    sync_directory(parent)?;
                }
            }
        }
        if current_path_digest(&target)? == operation.before_sha256
            && current_path_mode(&target)? == operation.before_mode
        {
            continue;
        }
        if operation.existed {
            let bytes = std::fs::read(attempt_root.join("backups").join(&relative))
                .map_err(PytxoError::Io)?;
            if target.exists() {
                std::fs::remove_file(&target).map_err(PytxoError::Io)?;
            }
            if let Some(parent) = target.parent() {
                create_dir_all_synced(parent)?;
            }
            replace_synced(&target, &bytes)?;
            set_file_mode(&target, operation.before_mode)?;
            std::fs::OpenOptions::new()
                .write(true)
                .open(&target)
                .and_then(|file| file.sync_all())
                .map_err(PytxoError::Io)?;
        } else if target.exists() {
            std::fs::remove_file(&target).map_err(PytxoError::Io)?;
            if let Some(parent) = target.parent() {
                sync_directory(parent)?;
            }
        }
    }
    for directory in journal
        .created_directories
        .iter()
        .rev()
        .filter(|directory| directory.stage == JournalDirectoryStage::Created)
    {
        let relative = validated_change_path(&directory.path)?;
        validate_target_ancestry(repo_root, &relative.join(".sentinel"))?;
        let path = repo_root.join(relative);
        if path.is_dir()
            && std::fs::read_dir(&path)
                .map_err(PytxoError::Io)?
                .next()
                .is_none()
        {
            std::fs::remove_dir(&path).map_err(PytxoError::Io)?;
            if let Some(parent) = path.parent() {
                sync_directory(parent)?;
            }
        }
    }
    journal.phase = "rolled_back".into();
    journal.in_flight = None;
    journal.in_flight_stage = None;
    persist_journal(attempt_root, journal)
}

fn validate_rollback_state(
    repo_root: &Path,
    attempt_root: &Path,
    journal: &ApplyJournal,
    touched: usize,
) -> Result<()> {
    for (index, operation) in journal.operations[..touched].iter().enumerate() {
        let relative = validated_change_path(&operation.path)?;
        validate_target_ancestry(repo_root, &relative)?;
        let target = repo_root.join(&relative);
        let current = current_path_digest(&target)?;
        let current_mode = current_path_mode(&target)?;
        let is_completed = index < journal.completed;
        let target_safe = if is_completed {
            current == operation.after_sha256 && current_mode == operation.after_mode
        } else {
            match operation.kind {
                PreparedRunFileKind::Add => {
                    current.is_none()
                        || (current == operation.after_sha256
                            && current_mode == operation.after_mode)
                }
                PreparedRunFileKind::Modify => {
                    current.is_none()
                        || (current == operation.before_sha256
                            && current_mode == operation.before_mode)
                        || (current == operation.after_sha256
                            && current_mode == operation.after_mode)
                }
                PreparedRunFileKind::Delete => {
                    current.is_none()
                        || (current == operation.before_sha256
                            && current_mode == operation.before_mode)
                }
            }
        };
        if !target_safe {
            return Err(PytxoError::Runner(format!(
                "recovery refused to overwrite post-crash drift at {}",
                operation.path
            )));
        }

        if operation.existed {
            let backup = attempt_root.join("backups").join(&relative);
            if current_path_digest(&backup)? != operation.before_sha256 {
                return Err(PytxoError::Runner(format!(
                    "recovery backup digest mismatch for {}",
                    operation.path
                )));
            }
        }
        if let Some(temporary) = &operation.temporary_path {
            let temporary_relative = validated_change_path(temporary)?;
            validate_target_ancestry(repo_root, &temporary_relative)?;
            let temporary_digest = current_path_digest(&repo_root.join(temporary_relative))?;
            if temporary_digest.is_some() && temporary_digest != operation.after_sha256 {
                return Err(PytxoError::Runner(format!(
                    "recovery temp digest mismatch for {}",
                    operation.path
                )));
            }
        }
    }
    Ok(())
}

fn journal_preimages_restored(repo_root: &Path, journal: &ApplyJournal) -> Result<bool> {
    for operation in &journal.operations {
        let relative = validated_change_path(&operation.path)?;
        validate_target_ancestry(repo_root, &relative)?;
        let target = repo_root.join(relative);
        if current_path_digest(&target)? != operation.before_sha256
            || current_path_mode(&target)? != operation.before_mode
        {
            return Ok(false);
        }
    }
    Ok(true)
}

fn file_digest(path: &Path) -> Result<String> {
    let metadata = std::fs::symlink_metadata(path).map_err(PytxoError::Io)?;
    if metadata.file_type().is_symlink() || !metadata.is_file() {
        return Err(PytxoError::Runner(format!(
            "run change source must be a regular file: {}",
            path.display()
        )));
    }
    let bytes = std::fs::read(path).map_err(PytxoError::Io)?;
    let digest = Sha256::digest(bytes);
    Ok(format!("{digest:x}"))
}

fn validated_change_path(path: &str) -> Result<PathBuf> {
    let path = PathBuf::from(path);
    if path.as_os_str().is_empty()
        || path.is_absolute()
        || path
            .components()
            .any(|component| !matches!(component, Component::Normal(_)))
    {
        return Err(PytxoError::Runner(format!(
            "unsafe run change path: {}",
            path.display()
        )));
    }
    let normalized = normalize_relative(&path);
    if is_protected_path(&normalized) {
        return Err(PytxoError::Runner(format!(
            "protected run change path: {}",
            path.display()
        )));
    }
    Ok(path)
}

fn task_dependencies(
    workspaces: &[AgentWorkspaceInput],
) -> Result<BTreeMap<String, BTreeSet<String>>> {
    let mut dependencies = BTreeMap::new();
    for workspace in workspaces {
        if dependencies
            .insert(
                workspace.task_id.clone(),
                workspace.depends_on.iter().cloned().collect(),
            )
            .is_some()
        {
            return Err(PytxoError::Runner(format!(
                "duplicate task id in run change set: {}",
                workspace.task_id
            )));
        }
    }
    Ok(dependencies)
}

fn task_depends_on(
    task_id: &str,
    ancestor_id: &str,
    dependencies: &BTreeMap<String, BTreeSet<String>>,
) -> bool {
    let mut pending = vec![task_id];
    let mut visited = BTreeSet::new();
    while let Some(current) = pending.pop() {
        if !visited.insert(current) {
            continue;
        }
        let Some(direct) = dependencies.get(current) else {
            continue;
        };
        if direct.contains(ancestor_id) {
            return true;
        }
        pending.extend(direct.iter().map(String::as_str));
    }
    false
}

fn changes_are_equivalent(left: &RunChange, right: &RunChange) -> Result<bool> {
    Ok(left.kind == right.kind
        && left.result_digest == right.result_digest
        && left.result_mode == right.result_mode)
}

fn collect_inventory(root: &Path, sparse_exclude: &[String]) -> Result<FileInventory> {
    let mut inventory = FileInventory::default();
    collect_files_inner(root, root, sparse_exclude, &mut inventory)?;
    Ok(inventory)
}

fn collect_files_inner(
    root: &Path,
    directory: &Path,
    sparse_exclude: &[String],
    inventory: &mut FileInventory,
) -> Result<()> {
    for entry in std::fs::read_dir(directory).map_err(PytxoError::Io)? {
        let entry = entry.map_err(PytxoError::Io)?;
        let path = entry.path();
        let relative = path.strip_prefix(root).map_err(|error| {
            PytxoError::Runner(format!("change-set path escaped root: {error}"))
        })?;
        let normalized = normalize_relative(relative);
        if is_protected_path(&normalized) && !is_canonical_protected_path(&normalized) {
            return Err(PytxoError::Runner(format!(
                "protected run change path: {}",
                relative.display()
            )));
        }
        if path_is_ignored(relative, sparse_exclude) {
            continue;
        }
        let metadata = std::fs::symlink_metadata(&path).map_err(PytxoError::Io)?;
        if metadata.file_type().is_symlink() {
            inventory
                .symlinks
                .insert(normalize_relative(relative), path);
        } else if metadata.is_dir() {
            collect_files_inner(root, &path, sparse_exclude, inventory)?;
        } else if metadata.is_file() {
            inventory.files.insert(normalize_relative(relative), path);
        } else {
            return Err(PytxoError::Runner(format!(
                "run Apply does not support special filesystem node {}",
                relative.display()
            )));
        }
    }
    Ok(())
}

fn validate_claims(claims: &[String]) -> Result<()> {
    for claim in claims {
        let path = Path::new(claim);
        let normalized = claim.replace('\\', "/");
        if normalized.trim().is_empty()
            || normalized.trim() == "."
            || path.is_absolute()
            || normalized
                .split('/')
                .any(|component| matches!(component, "." | ".."))
            || is_protected_path(&normalized)
        {
            return Err(PytxoError::Runner(format!(
                "unsafe run change-set claim: {claim}"
            )));
        }
        if claim
            .chars()
            .any(|character| matches!(character, '*' | '?' | '[' | ']'))
            && glob::Pattern::new(&claim.replace('\\', "/")).is_err()
        {
            return Err(PytxoError::Runner(format!(
                "invalid run change-set glob claim: {claim}"
            )));
        }
    }
    Ok(())
}

fn path_matches_claim(path: &str, claim: &str) -> bool {
    let claim = claim
        .replace('\\', "/")
        .trim_start_matches("./")
        .trim_end_matches('/')
        .to_string();
    if claim
        .chars()
        .any(|character| matches!(character, '*' | '?' | '[' | ']'))
    {
        return glob::Pattern::new(&claim).is_ok_and(|pattern| pattern.matches(path));
    }
    path == claim || path.starts_with(&format!("{claim}/"))
}

fn path_is_ignored(relative: &Path, sparse_exclude: &[String]) -> bool {
    let normalized = normalize_relative(relative);
    if is_protected_path(&normalized) {
        return true;
    }
    sparse_exclude.iter().any(|excluded| {
        let excluded = excluded.replace('\\', "/").trim_matches('/').to_string();
        !excluded.is_empty()
            && (normalized == excluded || normalized.starts_with(&format!("{excluded}/")))
    })
}

fn normalize_relative(path: &Path) -> String {
    path.to_string_lossy().replace('\\', "/")
}

fn is_protected_path(path: &str) -> bool {
    let first = path.split('/').next().unwrap_or_default();
    #[cfg(windows)]
    {
        first.eq_ignore_ascii_case(".git") || first.eq_ignore_ascii_case(".pytxo")
    }
    #[cfg(not(windows))]
    {
        first == ".git" || first == ".pytxo"
    }
}

fn is_canonical_protected_path(path: &str) -> bool {
    let first = path.split('/').next().unwrap_or_default();
    first == ".git" || first == ".pytxo"
}
