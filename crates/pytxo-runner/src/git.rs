use std::collections::{BTreeMap, BTreeSet};
use std::io::{Read, Write};
use std::path::{Path, PathBuf};
use std::process::Stdio;

use base64::Engine;
use pytxo_core::routing::{canonical_digest, BaseSnapshot, Digest};
use pytxo_core::{canonical_repo_root, strip_extended_path, PytxoError, Result};

pub fn create_worktree(repo_root: &Path, worktree_path: &Path, branch: &str) -> Result<()> {
    if worktree_path.exists() {
        return Err(PytxoError::Runner(format!(
            "worktree path already exists: {}",
            worktree_path.display()
        )));
    }
    if let Some(parent) = worktree_path.parent() {
        std::fs::create_dir_all(parent)?;
    }

    git(
        repo_root,
        &[
            "worktree",
            "add",
            "-b",
            branch,
            &path_to_git(worktree_path),
            "HEAD",
        ],
    )?;

    Ok(())
}

/// Prepare detached worktree metadata at the reviewed commit without checking
/// out files. Checkout may execute hooks and filters, so a routed controller
/// must materialize inputs later under owned process/sandbox authority.
pub fn prepare_reviewed_worktree(
    repo_root: &Path,
    worktree_path: &Path,
    base: &BaseSnapshot,
) -> Result<()> {
    let repo = canonical_repo_root(repo_root).map_err(PytxoError::Io)?;
    if !worktree_path.is_absolute()
        || base.repository_identity != repo.to_string_lossy()
        || !matches!(base.git_revision.len(), 40 | 64)
        || !base
            .git_revision
            .bytes()
            .all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte))
    {
        return Err(PytxoError::Runner(
            "reviewed worktree Git base identity is invalid".into(),
        ));
    }
    let observed_root = reviewed_git(&repo, &["rev-parse", "--show-toplevel"])?;
    if canonical_repo_root(Path::new(&observed_root)).map_err(PytxoError::Io)? != repo {
        return Err(PytxoError::Runner(
            "reviewed worktree Git repository identity changed".into(),
        ));
    }
    let commit_revision = format!("{}^{{commit}}", base.git_revision);
    if reviewed_git(&repo, &["rev-parse", "--verify", &commit_revision])? != base.git_revision {
        return Err(PytxoError::Runner(
            "reviewed worktree base is not an exact commit".into(),
        ));
    }
    let tree_revision = format!("{}^{{tree}}", base.git_revision);
    let tree = reviewed_git(&repo, &["rev-parse", "--verify", &tree_revision])?;
    if tree.len() != base.git_revision.len()
        || canonical_digest(&(1_u32, "git-head-tree", &tree), 1)
            .map_err(|error| PytxoError::Runner(error.to_string()))?
            != base.snapshot_digest
    {
        return Err(PytxoError::Runner(
            "reviewed worktree Git tree identity changed".into(),
        ));
    }
    if worktree_path.exists() {
        return Err(PytxoError::Runner(format!(
            "worktree path already exists: {}",
            worktree_path.display()
        )));
    }
    if let Some(parent) = worktree_path.parent() {
        std::fs::create_dir_all(parent)?;
    }
    reviewed_git(
        &repo,
        &[
            "worktree",
            "add",
            "--detach",
            "--no-checkout",
            &path_to_git(worktree_path),
            &base.git_revision,
        ],
    )?;
    let observed_worktree = reviewed_git(worktree_path, &["rev-parse", "--show-toplevel"])?;
    if canonical_repo_root(Path::new(&observed_worktree)).map_err(PytxoError::Io)?
        != canonical_repo_root(worktree_path).map_err(PytxoError::Io)?
        || reviewed_git(worktree_path, &["rev-parse", "--verify", "HEAD^{commit}"])?
            != base.git_revision
        || reviewed_git(worktree_path, &["rev-parse", "--verify", "HEAD^{tree}"])? != tree
    {
        // Preserve the created path for explicit recovery. An ambiguous
        // preparation must never be retried as a fresh worktree.
        return Err(PytxoError::Runner(
            "reviewed worktree changed during creation; ownership requires recovery".into(),
        ));
    }
    verify_reviewed_worktree_registration(&repo, worktree_path, base)?;
    Ok(())
}

const MAX_REVIEWED_INPUT_BYTES: usize = 16 * 1024 * 1024;
const MAX_REVIEWED_INPUT_FILES: usize = 4_096;
const MAX_TREE_LIST_BYTES: usize = 2 * 1024 * 1024;
const MAX_SEALED_OUTPUT_BYTES: usize = 9 * 1024 * 1024;

/// Git object bytes, without checkout filters, are the input authority for the
/// first routed local adapter. This manifest contains no file contents; its
/// exact serialized bytes belong in the private, per-domain input artifact.
/// The immutable Git objects must still be present at materialization time.
#[derive(Clone, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ReviewedInputManifest {
    pub schema_version: u32,
    pub base: BaseSnapshot,
    pub files: Vec<ReviewedInputFile>,
}

#[derive(Clone, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ReviewedInputFile {
    pub path: String,
    pub git_blob_oid: String,
    pub executable: bool,
    pub content_digest: Digest,
    pub byte_length: u64,
}

/// Exact private bytes captured only after an owned worker's Job is observed
/// at zero. The controller must retain the serialized snapshot as its scoped
/// output artifact before preparing a fresh independent verification view.
#[derive(Clone, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SealedOutputSnapshot {
    pub schema_version: u32,
    pub base: BaseSnapshot,
    pub files: Vec<SealedOutputFile>,
}

#[derive(Clone, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SealedOutputFile {
    pub path: String,
    pub content_base64: String,
    pub content_digest: Digest,
    pub byte_length: u64,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub mode: Option<u32>,
}

/// Capture the whole physical file set of the exact registered worktree. This
/// does not itself prove Job quiescence or authorize a routing transition.
pub fn capture_sealed_output(
    repo_root: &Path,
    worktree_path: &Path,
    input: &ReviewedInputManifest,
    claim_roots: &[String],
) -> Result<SealedOutputSnapshot> {
    capture_sealed_output_against(repo_root, worktree_path, input, None, claim_roots)
}

/// Seal a bounded text proposal without writing it into the worker's tree.
/// The owned worker must already be settled at Job zero. This function proves
/// that its entire physical tree still equals the reviewed input, then changes
/// exactly one existing claimed file in the private snapshot. The caller must
/// retain that snapshot before running independent checks.
pub fn seal_one_existing_claimed_text_proposal(
    repo_root: &Path,
    worktree_path: &Path,
    input: &ReviewedInputManifest,
    claim: &str,
    proposed: &[u8],
) -> Result<SealedOutputSnapshot> {
    const MAX_PROPOSAL_BYTES: usize = 16 * 1024;
    if proposed.len() > MAX_PROPOSAL_BYTES
        || proposed.contains(&0)
        || std::str::from_utf8(proposed).is_err()
    {
        return Err(invalid_reviewed_input(
            "one-file proposal is not bounded UTF-8 text",
        ));
    }
    validate_reviewed_path(claim)?;
    if input.files.iter().filter(|file| file.path == claim).count() != 1 {
        return Err(invalid_reviewed_input(
            "one-file proposal has no existing reviewed claim",
        ));
    }
    let mut snapshot = capture_sealed_output(repo_root, worktree_path, input, &[claim.into()])?;
    if snapshot.files.len() != input.files.len() {
        return Err(invalid_reviewed_input(
            "proposal worker changed the reviewed file set",
        ));
    }
    let originals: BTreeMap<_, _> = input
        .files
        .iter()
        .map(|file| (file.path.as_str(), file))
        .collect();
    for file in &snapshot.files {
        let original = originals
            .get(file.path.as_str())
            .ok_or_else(|| invalid_reviewed_input("proposal worker added a file"))?;
        let expected_mode = if cfg!(unix) {
            Some(if original.executable { 0o755 } else { 0o644 })
        } else {
            None
        };
        if file.content_digest != original.content_digest
            || file.byte_length != original.byte_length
            || file.mode != expected_mode
        {
            return Err(invalid_reviewed_input(
                "proposal worker changed a reviewed input",
            ));
        }
    }
    let target = snapshot
        .files
        .iter_mut()
        .find(|file| file.path == claim)
        .ok_or_else(|| invalid_reviewed_input("one-file proposal claim disappeared"))?;
    let original_bytes = base64::engine::general_purpose::STANDARD
        .decode(&target.content_base64)
        .map_err(|_| invalid_reviewed_input("proposal source encoding changed"))?;
    if original_bytes.len() > MAX_PROPOSAL_BYTES
        || original_bytes.contains(&0)
        || std::str::from_utf8(&original_bytes).is_err()
        || original_bytes == proposed
    {
        return Err(invalid_reviewed_input(
            "one-file proposal source is not a changed bounded text file",
        ));
    }
    target.content_base64 = base64::engine::general_purpose::STANDARD.encode(proposed);
    target.content_digest = Digest::of_bytes(proposed);
    target.byte_length = proposed.len() as u64;
    validate_sealed_output(&snapshot)?;
    Ok(snapshot)
}

/// The child may change only its own reviewed claims. Inherited predecessor
/// bytes form the baseline, including predecessor-created and deleted paths.
/// The caller must obtain `inherited` from Core's checked winner read.
pub fn capture_sealed_output_after_dependency(
    repo_root: &Path,
    worktree_path: &Path,
    input: &ReviewedInputManifest,
    inherited: &SealedOutputSnapshot,
    claim_roots: &[String],
) -> Result<SealedOutputSnapshot> {
    capture_sealed_output_against(
        repo_root,
        worktree_path,
        input,
        Some(inherited),
        claim_roots,
    )
}

fn capture_sealed_output_against(
    repo_root: &Path,
    worktree_path: &Path,
    input: &ReviewedInputManifest,
    inherited: Option<&SealedOutputSnapshot>,
    claim_roots: &[String],
) -> Result<SealedOutputSnapshot> {
    if capture_reviewed_inputs(repo_root, &input.base)? != *input || claim_roots.is_empty() {
        return Err(invalid_reviewed_input(
            "sealed output has no reviewed input or claim",
        ));
    }
    if let Some(inherited) = inherited {
        validate_dependency_output(input, inherited)?;
    }
    for claim in claim_roots {
        validate_reviewed_path(claim)?;
    }
    let root = verify_reviewed_worktree_registration(repo_root, worktree_path, &input.base)?;
    let files = read_plain_tree(&root, true)?;
    verify_reviewed_worktree_registration(repo_root, worktree_path, &input.base)?;
    let originals: BTreeMap<_, _> = match inherited {
        Some(inherited) => inherited
            .files
            .iter()
            .map(|file| {
                (
                    file.path.as_str(),
                    (&file.content_digest, file.byte_length, file.mode),
                )
            })
            .collect(),
        None => input
            .files
            .iter()
            .map(|file| {
                (
                    file.path.as_str(),
                    (
                        &file.content_digest,
                        file.byte_length,
                        if cfg!(unix) {
                            Some(if file.executable { 0o755 } else { 0o644 })
                        } else {
                            None
                        },
                    ),
                )
            })
            .collect(),
    };
    let current: BTreeSet<_> = files.iter().map(|file| file.path.as_str()).collect();
    for file in &files {
        if originals
            .get(file.path.as_str())
            .is_none_or(|(digest, length, mode)| {
                **digest != file.content_digest || *length != file.byte_length || file.mode != *mode
            })
            && !claim_roots
                .iter()
                .any(|claim| path_is_claimed(&file.path, claim))
        {
            return Err(invalid_reviewed_input("worker changed an unclaimed path"));
        }
    }
    for original_path in originals.keys() {
        if !current.contains(original_path)
            && !claim_roots
                .iter()
                .any(|claim| path_is_claimed(original_path, claim))
        {
            return Err(invalid_reviewed_input("worker deleted an unclaimed path"));
        }
    }
    let snapshot = SealedOutputSnapshot {
        schema_version: 2,
        base: input.base.clone(),
        files,
    };
    validate_sealed_output(&snapshot)?;
    Ok(snapshot)
}

fn path_is_claimed(path: &str, claim: &str) -> bool {
    path == claim
        || path
            .strip_prefix(claim)
            .is_some_and(|rest| rest.starts_with('/'))
}

/// Copy retained bytes into an empty plain directory for an owned checker.
/// Git checkout, hooks and filters never run in the verification view.
pub fn create_plain_verification_view(
    data_dir: &Path,
    run_id: &str,
    attempt_id: &str,
) -> Result<std::path::PathBuf> {
    for id in [run_id, attempt_id] {
        if id.is_empty()
            || !id
                .bytes()
                .all(|byte| byte.is_ascii_alphanumeric() || byte == b'-' || byte == b'_')
        {
            return Err(invalid_reviewed_input(
                "verification view identity is not a plain path component",
            ));
        }
    }
    let root = std::fs::symlink_metadata(data_dir)?;
    if !root.file_type().is_dir() || is_reparse(&root) {
        return Err(invalid_reviewed_input(
            "verification data directory is redirected",
        ));
    }
    let mut view = data_dir.to_path_buf();
    for component in ["routed-verification", run_id, attempt_id] {
        view.push(component);
        match std::fs::create_dir(&view) {
            Ok(()) => {}
            Err(error) if error.kind() == std::io::ErrorKind::AlreadyExists => {}
            Err(error) => return Err(error.into()),
        }
        let metadata = std::fs::symlink_metadata(&view)?;
        if !metadata.file_type().is_dir() || is_reparse(&metadata) {
            return Err(invalid_reviewed_input(
                "verification view ancestor is redirected",
            ));
        }
    }
    Ok(view)
}

/// Retire only this run's private verification views after Apply or discard.
pub fn remove_routed_verification_views(data_dir: &Path, run_id: &str) -> Result<()> {
    if run_id.is_empty()
        || !run_id
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || byte == b'-' || byte == b'_')
    {
        return Err(invalid_reviewed_input(
            "verification view identity is not a plain path component",
        ));
    }
    let root = data_dir.join("routed-verification");
    let run = root.join(run_id);
    if !run.exists() {
        return Ok(());
    }
    for path in [data_dir, root.as_path(), run.as_path()] {
        let metadata = std::fs::symlink_metadata(path)?;
        if !metadata.file_type().is_dir() || is_reparse(&metadata) {
            return Err(invalid_reviewed_input(
                "verification view cleanup path is redirected",
            ));
        }
    }
    read_plain_tree(&run, false)?;
    std::fs::remove_dir_all(run)?;
    Ok(())
}

pub fn materialize_sealed_output(view_root: &Path, snapshot: &SealedOutputSnapshot) -> Result<()> {
    validate_sealed_output(snapshot)?;
    let metadata = std::fs::symlink_metadata(view_root)?;
    if !metadata.file_type().is_dir()
        || is_reparse(&metadata)
        || std::fs::read_dir(view_root)?.next().is_some()
    {
        return Err(invalid_reviewed_input(
            "verification view is not an empty plain directory",
        ));
    }
    write_snapshot_files(view_root, snapshot)?;
    verify_sealed_output_view(view_root, snapshot)
}

fn write_snapshot_files(view_root: &Path, snapshot: &SealedOutputSnapshot) -> Result<()> {
    for file in &snapshot.files {
        let bytes = base64::engine::general_purpose::STANDARD
            .decode(&file.content_base64)
            .map_err(|_| invalid_reviewed_input("sealed output encoding changed"))?;
        let mut current = view_root.to_path_buf();
        for component in file.path.split('/').take(file.path.matches('/').count()) {
            current = current.join(component);
            if !current.exists() {
                std::fs::create_dir(&current)?;
            }
            let metadata = std::fs::symlink_metadata(&current)?;
            if !metadata.file_type().is_dir() || is_reparse(&metadata) {
                return Err(invalid_reviewed_input(
                    "verification view directory changed",
                ));
            }
        }
        let path = view_root.join(file.path.replace('/', std::path::MAIN_SEPARATOR_STR));
        let mut output = std::fs::OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(&path)?;
        output.write_all(&bytes)?;
        output.sync_all()?;
        crate::change_set::set_file_mode(&path, file.mode)?;
    }
    Ok(())
}

/// Rehash every file before and after each checker. A changed view cannot
/// satisfy a retained checker receipt even if the checker exits successfully.
pub fn verify_sealed_output_view(view_root: &Path, snapshot: &SealedOutputSnapshot) -> Result<()> {
    validate_sealed_output(snapshot)?;
    let metadata = std::fs::symlink_metadata(view_root)?;
    if !metadata.file_type().is_dir() || is_reparse(&metadata) {
        return Err(invalid_reviewed_input("verification view root changed"));
    }
    if read_plain_tree(view_root, false)? != snapshot.files {
        return Err(invalid_reviewed_input("verification view bytes changed"));
    }
    Ok(())
}

fn validate_sealed_output(snapshot: &SealedOutputSnapshot) -> Result<()> {
    if !matches!(snapshot.schema_version, 1 | 2)
        || snapshot.files.is_empty()
        || snapshot.files.len() > MAX_REVIEWED_INPUT_FILES
        || !snapshot.base.snapshot_digest.is_valid()
        || snapshot.base.repository_identity.is_empty()
        || !matches!(snapshot.base.git_revision.len(), 40 | 64)
        || !snapshot
            .base
            .git_revision
            .bytes()
            .all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte))
    {
        return Err(invalid_reviewed_input("sealed output shape is invalid"));
    }
    let mut total = 0_usize;
    let mut paths = BTreeSet::new();
    for file in &snapshot.files {
        validate_reviewed_path(&file.path)?;
        let folded = file.path.to_ascii_lowercase();
        if folded
            .match_indices('/')
            .any(|(index, _)| paths.contains(&folded[..index]))
            || paths
                .range(folded.clone()..)
                .next()
                .is_some_and(|existing: &String| existing.starts_with(&format!("{folded}/")))
        {
            return Err(invalid_reviewed_input(
                "sealed output file and directory collide",
            ));
        }
        if !paths.insert(folded) {
            return Err(invalid_reviewed_input("sealed output path collides"));
        }
        let bytes = base64::engine::general_purpose::STANDARD
            .decode(&file.content_base64)
            .map_err(|_| invalid_reviewed_input("sealed output encoding changed"))?;
        total = total
            .checked_add(bytes.len())
            .ok_or_else(|| invalid_reviewed_input("sealed output size overflow"))?;
        if total > MAX_SEALED_OUTPUT_BYTES
            || file.byte_length != bytes.len() as u64
            || file.content_digest != Digest::of_bytes(&bytes)
            || file.mode.is_some_and(|mode| mode > 0o7777)
            || (cfg!(unix) && file.mode.is_none())
            || (snapshot.schema_version == 1 && file.mode.is_some())
        {
            return Err(invalid_reviewed_input(
                "sealed output bytes or size changed",
            ));
        }
    }
    if serde_json::to_vec(snapshot)
        .map_err(|error| invalid_reviewed_input(&error.to_string()))?
        .len()
        > MAX_REVIEWED_INPUT_BYTES
    {
        return Err(invalid_reviewed_input(
            "sealed output artifact exceeds Store limit",
        ));
    }
    Ok(())
}

fn read_plain_tree(root: &Path, skip_root_git: bool) -> Result<Vec<SealedOutputFile>> {
    let mut files = Vec::new();
    let mut total = 0_usize;
    let mut directories = vec![root.to_path_buf()];
    let mut observed_directories = BTreeSet::new();
    while let Some(directory) = directories.pop() {
        for entry in std::fs::read_dir(directory)? {
            let entry = entry?;
            let metadata = std::fs::symlink_metadata(entry.path())?;
            if skip_root_git && entry.path() == root.join(".git") {
                if !metadata.file_type().is_file() || is_reparse(&metadata) {
                    return Err(invalid_reviewed_input(
                        "worker Git link changed while sealing",
                    ));
                }
                continue;
            }
            if is_reparse(&metadata) {
                return Err(invalid_reviewed_input(
                    "sealed output contains a reparse point",
                ));
            }
            if metadata.file_type().is_dir() {
                let relative = entry
                    .path()
                    .strip_prefix(root)
                    .map_err(|_| {
                        invalid_reviewed_input("sealed output directory escaped its root")
                    })?
                    .to_string_lossy()
                    .replace('\\', "/");
                validate_reviewed_path(&relative)?;
                if relative.matches('/').count() >= 64
                    || !observed_directories.insert(relative)
                    || observed_directories.len() > MAX_REVIEWED_INPUT_FILES
                {
                    return Err(invalid_reviewed_input(
                        "sealed output has too many directories",
                    ));
                }
                directories.push(entry.path());
                continue;
            }
            if !metadata.file_type().is_file() || files.len() >= MAX_REVIEWED_INPUT_FILES {
                return Err(invalid_reviewed_input(
                    "sealed output has unsupported entries",
                ));
            }
            let relative = entry
                .path()
                .strip_prefix(root)
                .map_err(|_| invalid_reviewed_input("sealed output escaped its root"))?
                .to_string_lossy()
                .replace('\\', "/");
            validate_reviewed_path(&relative)?;
            let remaining = MAX_SEALED_OUTPUT_BYTES - total;
            if metadata.len() > remaining as u64 {
                return Err(invalid_reviewed_input(
                    "sealed output exceeds fixture limit",
                ));
            }
            let mut bytes = Vec::new();
            std::fs::File::open(entry.path())?
                .take(remaining as u64 + 1)
                .read_to_end(&mut bytes)?;
            if bytes.len() > remaining || bytes.len() as u64 != metadata.len() {
                return Err(invalid_reviewed_input(
                    "sealed output file changed while reading",
                ));
            }
            total = total
                .checked_add(bytes.len())
                .ok_or_else(|| invalid_reviewed_input("sealed output size overflow"))?;
            if total > MAX_SEALED_OUTPUT_BYTES {
                return Err(invalid_reviewed_input(
                    "sealed output exceeds fixture limit",
                ));
            }
            files.push(SealedOutputFile {
                path: relative,
                content_base64: base64::engine::general_purpose::STANDARD.encode(&bytes),
                content_digest: Digest::of_bytes(&bytes),
                byte_length: bytes.len() as u64,
                mode: crate::change_set::file_mode(&entry.path())?,
            });
        }
    }
    files.sort_by(|a, b| a.path.cmp(&b.path));
    let expected_directories = parent_directories(files.iter().map(|file| file.path.as_str()));
    if observed_directories != expected_directories {
        return Err(invalid_reviewed_input(
            "sealed output contains an empty directory",
        ));
    }
    Ok(files)
}

fn parent_directories<'a>(paths: impl IntoIterator<Item = &'a str>) -> BTreeSet<String> {
    let mut directories = BTreeSet::new();
    for path in paths {
        for (index, _) in path.match_indices('/') {
            directories.insert(path[..index].to_owned());
        }
    }
    directories
}

#[cfg(all(test, unix))]
mod sealed_mode_tests {
    use super::*;
    use std::os::unix::fs::PermissionsExt;

    #[test]
    fn sealed_executable_mode_survives_plain_view_and_is_rechecked() {
        let source = tempfile::tempdir().unwrap();
        let script = source.path().join("run.sh");
        std::fs::write(&script, b"#!/bin/sh\nexit 0\n").unwrap();
        std::fs::set_permissions(&script, std::fs::Permissions::from_mode(0o755)).unwrap();
        let snapshot = SealedOutputSnapshot {
            schema_version: 2,
            base: BaseSnapshot {
                repository_identity: "repo".into(),
                git_revision: "a".repeat(40),
                snapshot_digest: Digest::of_bytes(b"base"),
            },
            files: read_plain_tree(source.path(), false).unwrap(),
        };
        assert_eq!(snapshot.files[0].mode, Some(0o755));
        let view = tempfile::tempdir().unwrap();
        materialize_sealed_output(view.path(), &snapshot).unwrap();
        assert_eq!(
            crate::change_set::file_mode(&view.path().join("run.sh")).unwrap(),
            Some(0o755)
        );
        std::fs::set_permissions(
            view.path().join("run.sh"),
            std::fs::Permissions::from_mode(0o644),
        )
        .unwrap();
        assert!(verify_sealed_output_view(view.path(), &snapshot).is_err());
    }
}

/// Capture a complete, bounded Git tree. Symlinks, submodules, nonportable
/// paths and large trees are ineligible for this first local fixture. `cat-file`
/// reads stored blobs directly and never runs smudge/clean filters or hooks.
pub fn capture_reviewed_inputs(
    repo_root: &Path,
    base: &BaseSnapshot,
) -> Result<ReviewedInputManifest> {
    let repo = verify_reviewed_base(repo_root, base)?;
    let tree_output = reviewed_git_output_limited(
        &repo,
        &["ls-tree", "-r", "-z", "--full-tree", &base.git_revision],
        MAX_TREE_LIST_BYTES,
    )?;
    if tree_output.len() > MAX_TREE_LIST_BYTES || tree_output.last() != Some(&0) {
        return Err(invalid_reviewed_input(
            "Git tree listing is invalid or too large",
        ));
    }
    let mut files = Vec::new();
    let mut paths = BTreeSet::new();
    for record in tree_output[..tree_output.len() - 1].split(|byte| *byte == 0) {
        if record.is_empty() || files.len() >= MAX_REVIEWED_INPUT_FILES {
            return Err(invalid_reviewed_input(
                "Git tree has too many or empty entries",
            ));
        }
        let Some(tab) = record.iter().position(|byte| *byte == b'\t') else {
            return Err(invalid_reviewed_input("Git tree entry has no path"));
        };
        let metadata = std::str::from_utf8(&record[..tab])
            .map_err(|_| invalid_reviewed_input("Git tree entry metadata is not UTF-8"))?;
        let path = std::str::from_utf8(&record[tab + 1..])
            .map_err(|_| invalid_reviewed_input("Git tree path is not UTF-8"))?;
        validate_reviewed_path(path)?;
        let folded = path.to_ascii_lowercase();
        if folded
            .match_indices('/')
            .any(|(index, _)| paths.contains(&folded[..index]))
            || paths
                .range(folded.clone()..)
                .next()
                .is_some_and(|existing: &String| existing.starts_with(&format!("{folded}/")))
        {
            return Err(invalid_reviewed_input(
                "Git tree has a case-colliding file and directory",
            ));
        }
        if !paths.insert(folded) {
            return Err(invalid_reviewed_input("Git tree has a case-colliding path"));
        }
        let mut fields = metadata.split(' ');
        let (Some(mode), Some(kind), Some(oid), None) =
            (fields.next(), fields.next(), fields.next(), fields.next())
        else {
            return Err(invalid_reviewed_input("Git tree entry metadata is invalid"));
        };
        if kind != "blob" || !matches!(mode, "100644" | "100755") {
            return Err(invalid_reviewed_input(
                "Git tree has an unsupported entry type",
            ));
        }
        if oid.len() != base.git_revision.len()
            || !oid
                .bytes()
                .all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte))
        {
            return Err(invalid_reviewed_input("Git tree blob identity is invalid"));
        }
        files.push(ReviewedInputFile {
            path: path.to_owned(),
            git_blob_oid: oid.to_owned(),
            executable: mode == "100755",
            content_digest: Digest::of_bytes(&[]),
            byte_length: 0,
        });
    }
    if files.is_empty() {
        return Err(invalid_reviewed_input(
            "Git tree has no regular input files",
        ));
    }
    let contents = reviewed_git_blobs(&repo, &files)?;
    for (file, bytes) in files.iter_mut().zip(contents) {
        file.content_digest = Digest::of_bytes(&bytes);
        file.byte_length = bytes.len() as u64;
    }
    Ok(ReviewedInputManifest {
        schema_version: 1,
        base: base.clone(),
        files,
    })
}

/// The current routed Review and Apply use physical primary-checkout preimages,
/// while worker inputs come from Git blobs. Until those two baselines are
/// modeled separately, admit only a checkout whose tracked bytes are identical
/// to the reviewed blobs. Git's clean status alone does not establish this on
/// checkouts transformed by core.autocrlf or checkout filters.
pub fn require_exact_reviewed_checkout(
    repo_root: &Path,
    base: &BaseSnapshot,
    sparse_exclude: &[String],
) -> Result<()> {
    let manifest = capture_reviewed_inputs(repo_root, base)?;
    let root = canonical_repo_root(repo_root).map_err(PytxoError::Io)?;
    for file in &manifest.files {
        let mut physical = root.clone();
        let mut components = Path::new(&file.path).components().peekable();
        while let Some(component) = components.next() {
            let std::path::Component::Normal(part) = component else {
                return Err(invalid_reviewed_input("reviewed checkout path is invalid"));
            };
            physical.push(part);
            let metadata = std::fs::symlink_metadata(&physical)?;
            if is_reparse(&metadata)
                || (components.peek().is_some() && !metadata.file_type().is_dir())
                || (components.peek().is_none() && !metadata.file_type().is_file())
            {
                return Err(invalid_reviewed_input(
                    "reviewed checkout contains a redirected or unsupported tracked path",
                ));
            }
            if components.peek().is_none() {
                if metadata.len() != file.byte_length {
                    return Err(invalid_reviewed_input(
                        "reviewed checkout bytes differ from Git blobs; disable checkout normalization such as core.autocrlf and re-check out before using this experimental route",
                    ));
                }
                let bytes = std::fs::read(&physical)?;
                if bytes.len() as u64 != file.byte_length
                    || Digest::of_bytes(&bytes) != file.content_digest
                    || (cfg!(unix)
                        && crate::change_set::file_mode(&physical)?
                            .is_none_or(|mode| (mode & 0o111 != 0) != file.executable))
                {
                    return Err(invalid_reviewed_input(
                        "reviewed checkout bytes or modes differ from Git blobs; disable checkout normalization such as core.autocrlf and re-check out before using this experimental route",
                    ));
                }
            }
        }
    }
    let physical = crate::change_set::collect_inventory(&root, sparse_exclude)?;
    if !physical.symlinks.is_empty() {
        return Err(invalid_reviewed_input(
            "reviewed checkout contains an included symlink",
        ));
    }
    let included_git: BTreeSet<_> = manifest
        .files
        .iter()
        .filter(|file| !crate::change_set::path_is_ignored(Path::new(&file.path), sparse_exclude))
        .map(|file| file.path.as_str())
        .collect();
    let included_physical: BTreeSet<_> = physical.files.keys().map(String::as_str).collect();
    if included_git != included_physical {
        return Err(invalid_reviewed_input(
            "reviewed checkout has an included physical file outside the Git snapshot; remove or exclude ignored files before using this experimental route",
        ));
    }
    Ok(())
}

/// Read one existing claimed text file from the exact reviewed Git blob. The
/// proposal prompt uses this source, never a mutable worker worktree read.
pub fn read_reviewed_claim_text(
    repo_root: &Path,
    input: &ReviewedInputManifest,
    claim: &str,
) -> Result<String> {
    const MAX_SOURCE_BYTES: u64 = 8 * 1024;
    validate_reviewed_path(claim)?;
    if capture_reviewed_inputs(repo_root, &input.base)? != *input {
        return Err(invalid_reviewed_input("reviewed source manifest changed"));
    }
    let file = input
        .files
        .iter()
        .find(|file| file.path == claim)
        .ok_or_else(|| invalid_reviewed_input("reviewed text claim does not exist"))?;
    if file.byte_length > MAX_SOURCE_BYTES {
        return Err(invalid_reviewed_input(
            "reviewed text claim exceeds prompt bound",
        ));
    }
    let repo = verify_reviewed_base(repo_root, &input.base)?;
    let mut contents = reviewed_git_blobs(&repo, std::slice::from_ref(file))?;
    let bytes = contents
        .pop()
        .ok_or_else(|| invalid_reviewed_input("reviewed blob is absent"))?;
    if bytes.len() as u64 != file.byte_length
        || Digest::of_bytes(&bytes) != file.content_digest
        || bytes.contains(&0)
    {
        return Err(invalid_reviewed_input(
            "reviewed text claim bytes changed or are binary",
        ));
    }
    String::from_utf8(bytes).map_err(|_| invalid_reviewed_input("reviewed text claim is not UTF-8"))
}

fn reviewed_git_blobs(repo_root: &Path, files: &[ReviewedInputFile]) -> Result<Vec<Vec<u8>>> {
    if files.is_empty() || files.len() > MAX_REVIEWED_INPUT_FILES {
        return Err(invalid_reviewed_input("Git input file count is invalid"));
    }
    let mut requests = Vec::with_capacity(files.len() * 65);
    for file in files {
        requests.extend(file.git_blob_oid.as_bytes());
        requests.push(b'\n');
    }
    let limit = MAX_REVIEWED_INPUT_BYTES
        .checked_add(files.len() * 100)
        .ok_or_else(|| invalid_reviewed_input("Git batch output limit overflow"))?;
    let batch = reviewed_git_output_limited_with_input(
        repo_root,
        &["cat-file", "--batch"],
        limit,
        Some(&requests),
    )?;
    let mut cursor = 0_usize;
    let mut total_bytes = 0_usize;
    let mut contents = Vec::with_capacity(files.len());
    for file in files {
        let header_end = batch[cursor..]
            .iter()
            .position(|byte| *byte == b'\n')
            .and_then(|offset| cursor.checked_add(offset))
            .ok_or_else(|| invalid_reviewed_input("Git batch response has no header"))?;
        let header = std::str::from_utf8(&batch[cursor..header_end])
            .map_err(|_| invalid_reviewed_input("Git batch header is not UTF-8"))?;
        let mut fields = header.split(' ');
        let (Some(oid), Some(kind), Some(length), None) =
            (fields.next(), fields.next(), fields.next(), fields.next())
        else {
            return Err(invalid_reviewed_input("Git batch header is invalid"));
        };
        if oid != file.git_blob_oid || kind != "blob" {
            return Err(invalid_reviewed_input("Git batch object identity changed"));
        }
        let length = length
            .parse::<usize>()
            .map_err(|_| invalid_reviewed_input("Git batch blob length is invalid"))?;
        total_bytes = total_bytes
            .checked_add(length)
            .ok_or_else(|| invalid_reviewed_input("Git input size overflow"))?;
        if total_bytes > MAX_REVIEWED_INPUT_BYTES {
            return Err(invalid_reviewed_input(
                "Git input exceeds the local fixture limit",
            ));
        }
        let start = header_end + 1;
        let end = start
            .checked_add(length)
            .ok_or_else(|| invalid_reviewed_input("Git batch length overflow"))?;
        if batch.get(end) != Some(&b'\n') {
            return Err(invalid_reviewed_input("Git batch blob framing changed"));
        }
        contents.push(batch[start..end].to_vec());
        cursor = end + 1;
    }
    if cursor != batch.len() {
        return Err(invalid_reviewed_input(
            "Git batch returned extra object bytes",
        ));
    }
    Ok(contents)
}

/// Materialize exact retained Git inputs into an inert reviewed worktree. All
/// object bytes and manifest entries are checked before the first file write.
/// The caller must own this fresh worktree and retain it for recovery if a
/// write or final physical-byte check fails; this is never an Apply operation.
pub fn materialize_reviewed_inputs(
    repo_root: &Path,
    worktree_path: &Path,
    manifest: &ReviewedInputManifest,
) -> Result<()> {
    if manifest.schema_version != 1
        || capture_reviewed_inputs(repo_root, &manifest.base)? != *manifest
    {
        return Err(invalid_reviewed_input(
            "retained Git input manifest changed",
        ));
    }
    let worker = verify_reviewed_worktree_registration(repo_root, worktree_path, &manifest.base)?;
    let mut root_entries = std::fs::read_dir(&worker)?;
    let only_git = root_entries.next().transpose()?.is_some_and(|entry| {
        entry.file_name() == ".git"
            && std::fs::symlink_metadata(entry.path())
                .is_ok_and(|metadata| metadata.file_type().is_file() && !is_reparse(&metadata))
    });
    if !only_git || root_entries.next().transpose()?.is_some() {
        return Err(invalid_reviewed_input(
            "inert worktree already contains files",
        ));
    }
    // Collect all bytes first. A missing or changed Git object cannot leave a
    // partially materialized worktree, and no checkout filter is executed.
    let contents = reviewed_git_blobs(repo_root, &manifest.files)?;
    for (file, bytes) in manifest.files.iter().zip(&contents) {
        if bytes.len() as u64 != file.byte_length || Digest::of_bytes(bytes) != file.content_digest
        {
            return Err(invalid_reviewed_input("retained Git input bytes changed"));
        }
    }
    for (entry, bytes) in manifest.files.iter().zip(contents) {
        let path = worker.join(entry.path.replace('/', std::path::MAIN_SEPARATOR_STR));
        let mut current = worker.clone();
        for component in entry.path.split('/').take(entry.path.matches('/').count()) {
            current = current.join(component);
            if !current.exists() {
                std::fs::create_dir(&current)?;
            }
            let metadata = std::fs::symlink_metadata(&current)?;
            if !metadata.file_type().is_dir() || is_reparse(&metadata) {
                return Err(invalid_reviewed_input(
                    "input directory is not a plain directory",
                ));
            }
        }
        let mut output = std::fs::OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(&path)?;
        output.write_all(&bytes)?;
        output.sync_all()?;
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            std::fs::set_permissions(
                &path,
                std::fs::Permissions::from_mode(if entry.executable { 0o755 } else { 0o644 }),
            )?;
        }
    }
    verify_materialized_reviewed_inputs(repo_root, &worker, manifest)
}

fn validate_dependency_output(
    input: &ReviewedInputManifest,
    inherited: &SealedOutputSnapshot,
) -> Result<()> {
    validate_sealed_output(inherited)?;
    if input.schema_version != 1 || inherited.base != input.base {
        return Err(invalid_reviewed_input(
            "dependency output has a different reviewed base",
        ));
    }
    Ok(())
}

/// Materialize one Core-verified predecessor's complete retained tree into a
/// fresh inert worktree. This checks physical bytes, not dependency authority;
/// the caller must first resolve the exact passed winner from the domain Store.
pub fn materialize_dependency_output(
    repo_root: &Path,
    worktree_path: &Path,
    input: &ReviewedInputManifest,
    inherited: &SealedOutputSnapshot,
) -> Result<()> {
    if capture_reviewed_inputs(repo_root, &input.base)? != *input {
        return Err(invalid_reviewed_input(
            "retained Git input manifest changed",
        ));
    }
    validate_dependency_output(input, inherited)?;
    let worker = verify_reviewed_worktree_registration(repo_root, worktree_path, &input.base)?;
    let mut root_entries = std::fs::read_dir(&worker)?;
    let only_git = root_entries.next().transpose()?.is_some_and(|entry| {
        entry.file_name() == ".git"
            && std::fs::symlink_metadata(entry.path())
                .is_ok_and(|metadata| metadata.file_type().is_file() && !is_reparse(&metadata))
    });
    if !only_git || root_entries.next().transpose()?.is_some() {
        return Err(invalid_reviewed_input(
            "inert worktree already contains files",
        ));
    }
    write_snapshot_files(&worker, inherited)?;
    verify_materialized_dependency_output(repo_root, &worker, input, inherited)
}

/// Check the exact inherited tree immediately before releasing a child worker.
/// The dependency winner and its output digest must be rechecked by Core.
pub fn verify_materialized_dependency_output(
    repo_root: &Path,
    worktree_path: &Path,
    input: &ReviewedInputManifest,
    inherited: &SealedOutputSnapshot,
) -> Result<()> {
    if capture_reviewed_inputs(repo_root, &input.base)? != *input {
        return Err(invalid_reviewed_input(
            "retained Git input manifest changed",
        ));
    }
    validate_dependency_output(input, inherited)?;
    let worker = verify_reviewed_worktree_registration(repo_root, worktree_path, &input.base)?;
    if read_plain_tree(&worker, true)? != inherited.files {
        return Err(invalid_reviewed_input("dependency input bytes changed"));
    }
    verify_reviewed_worktree_registration(repo_root, worktree_path, &input.base)?;
    Ok(())
}

/// Check the complete physical file set immediately before a routed launch.
/// The caller must first read this exact manifest from the domain Store; this
/// method checks Git and bytes, not Store authority or launch ownership.
pub fn verify_materialized_reviewed_inputs(
    repo_root: &Path,
    worktree_path: &Path,
    manifest: &ReviewedInputManifest,
) -> Result<()> {
    if manifest.schema_version != 1
        || manifest.files.is_empty()
        || capture_reviewed_inputs(repo_root, &manifest.base)? != *manifest
    {
        return Err(invalid_reviewed_input(
            "retained Git input manifest changed",
        ));
    }
    let root = verify_reviewed_worktree_registration(repo_root, worktree_path, &manifest.base)?;
    let expected: BTreeMap<_, _> = manifest
        .files
        .iter()
        .map(|file| (file.path.as_str(), file))
        .collect();
    let expected_directories =
        parent_directories(manifest.files.iter().map(|file| file.path.as_str()));
    let mut seen = BTreeSet::new();
    let mut directories = vec![root.clone()];
    while let Some(directory) = directories.pop() {
        for entry in std::fs::read_dir(directory)? {
            let entry = entry?;
            let metadata = std::fs::symlink_metadata(entry.path())?;
            if entry.path() == root.join(".git") {
                if !metadata.file_type().is_file() || is_reparse(&metadata) {
                    return Err(invalid_reviewed_input("worktree Git link changed"));
                }
                continue;
            }
            if is_reparse(&metadata) {
                return Err(invalid_reviewed_input(
                    "materialized input contains a reparse point",
                ));
            }
            if metadata.file_type().is_dir() {
                let relative = entry
                    .path()
                    .strip_prefix(&root)
                    .map_err(|_| {
                        invalid_reviewed_input("materialized input directory left its worktree")
                    })?
                    .to_string_lossy()
                    .replace('\\', "/");
                if !expected_directories.contains(&relative) {
                    return Err(invalid_reviewed_input(
                        "materialized input has an extra directory",
                    ));
                }
                directories.push(entry.path());
                continue;
            }
            if !metadata.file_type().is_file() {
                return Err(invalid_reviewed_input(
                    "materialized input is not a regular file",
                ));
            }
            let relative = entry
                .path()
                .strip_prefix(&root)
                .map_err(|_| invalid_reviewed_input("materialized input left its worktree"))?
                .to_string_lossy()
                .replace('\\', "/");
            let Some(file) = expected.get(relative.as_str()) else {
                return Err(invalid_reviewed_input(
                    "materialized input has an extra file",
                ));
            };
            if metadata.len() != file.byte_length {
                return Err(invalid_reviewed_input("materialized input size changed"));
            }
            #[cfg(unix)]
            {
                use std::os::unix::fs::PermissionsExt;
                let expected_mode = if file.executable { 0o755 } else { 0o644 };
                if metadata.permissions().mode() & 0o7777 != expected_mode {
                    return Err(invalid_reviewed_input("materialized input mode changed"));
                }
            }
            let mut bytes = Vec::new();
            std::fs::File::open(entry.path())?
                .take(file.byte_length + 1)
                .read_to_end(&mut bytes)?;
            if bytes.len() as u64 != file.byte_length
                || Digest::of_bytes(&bytes) != file.content_digest
            {
                return Err(invalid_reviewed_input("materialized input bytes changed"));
            }
            seen.insert(relative);
        }
    }
    if seen.len() != manifest.files.len() {
        return Err(invalid_reviewed_input(
            "materialized input is missing files",
        ));
    }
    verify_reviewed_worktree_registration(repo_root, worktree_path, &manifest.base)?;
    Ok(())
}

fn reviewed_git_directory(root: &Path, argument: &str) -> Result<PathBuf> {
    let location = reviewed_git(root, &["rev-parse", argument])?;
    let location = Path::new(&location);
    let location = if location.is_absolute() {
        location.to_path_buf()
    } else {
        root.join(location)
    };
    canonical_repo_root(&location).map_err(PytxoError::Io)
}

fn verify_reviewed_worktree_registration(
    repo_root: &Path,
    worktree_path: &Path,
    base: &BaseSnapshot,
) -> Result<PathBuf> {
    let repo = canonical_repo_root(repo_root).map_err(PytxoError::Io)?;
    if !worktree_path.is_absolute() || base.repository_identity != repo.to_string_lossy() {
        return Err(invalid_reviewed_input(
            "worktree is outside reviewed repository identity",
        ));
    }
    let worker_metadata = std::fs::symlink_metadata(worktree_path)?;
    if !worker_metadata.file_type().is_dir() || is_reparse(&worker_metadata) {
        return Err(invalid_reviewed_input("worktree is not a plain directory"));
    }
    let worker = canonical_repo_root(worktree_path).map_err(PytxoError::Io)?;
    let link = worker.join(".git");
    let link_metadata = std::fs::symlink_metadata(&link)?;
    if !link_metadata.file_type().is_file() || is_reparse(&link_metadata) {
        return Err(invalid_reviewed_input(
            "worktree Git link is not a plain file",
        ));
    }
    let observed_root = reviewed_git(&worker, &["rev-parse", "--show-toplevel"])?;
    if canonical_repo_root(Path::new(&observed_root)).map_err(PytxoError::Io)? != worker
        || reviewed_git(&worker, &["rev-parse", "--verify", "HEAD^{commit}"])? != base.git_revision
    {
        return Err(invalid_reviewed_input(
            "worktree differs from reviewed base",
        ));
    }
    let source_common = reviewed_git_directory(&repo, "--git-common-dir")?;
    let worker_common = reviewed_git_directory(&worker, "--git-common-dir")?;
    if worker_common != source_common {
        return Err(invalid_reviewed_input(
            "worktree has a foreign Git common directory",
        ));
    }
    let worker_admin = reviewed_git_directory(&worker, "--absolute-git-dir")?;
    let admin_parent = worker_admin
        .parent()
        .ok_or_else(|| invalid_reviewed_input("worktree has no Git administration parent"))?;
    if canonical_repo_root(admin_parent).map_err(PytxoError::Io)?
        != canonical_repo_root(&source_common.join("worktrees")).map_err(PytxoError::Io)?
    {
        return Err(invalid_reviewed_input(
            "worktree Git registration has a foreign owner",
        ));
    }
    let backpointer_file = worker_admin.join("gitdir");
    if std::fs::symlink_metadata(&backpointer_file)?.len() > 4_096 {
        return Err(invalid_reviewed_input(
            "worktree Git backpointer is too large",
        ));
    }
    let backpointer = std::fs::read_to_string(backpointer_file)?;
    let backpointer = Path::new(backpointer.trim());
    let backpointer = if backpointer.is_absolute() {
        backpointer.to_path_buf()
    } else {
        worker_admin.join(backpointer)
    };
    if canonical_repo_root(&backpointer).map_err(PytxoError::Io)?
        != canonical_repo_root(&link).map_err(PytxoError::Io)?
    {
        return Err(invalid_reviewed_input(
            "worktree Git registration backpointer changed",
        ));
    }
    Ok(worker)
}

fn validate_reviewed_path(path: &str) -> Result<()> {
    if path.len() > 2_048 || !path.is_ascii() || path.starts_with('/') {
        return Err(invalid_reviewed_input("Git input path is unsupported"));
    }
    for part in path.split('/') {
        if part.is_empty()
            || part.len() > 240
            || matches!(part, "." | "..")
            || part.eq_ignore_ascii_case(".git")
            || part.eq_ignore_ascii_case(".pytxo")
            || part.ends_with(['.', ' '])
            || part.bytes().any(|byte| {
                byte < 32 || matches!(byte, b'\\' | b':' | b'*' | b'?' | b'"' | b'<' | b'>' | b'|')
            })
        {
            return Err(invalid_reviewed_input("Git input path is unsafe"));
        }
        let stem = part.split('.').next().unwrap_or("").to_ascii_uppercase();
        if matches!(stem.as_str(), "CON" | "PRN" | "AUX" | "NUL")
            || (stem.len() == 4
                && (stem.starts_with("COM") || stem.starts_with("LPT"))
                && matches!(stem.as_bytes()[3], b'1'..=b'9'))
        {
            return Err(invalid_reviewed_input(
                "Git input path is a reserved device name",
            ));
        }
    }
    Ok(())
}

pub(crate) fn is_reparse(metadata: &std::fs::Metadata) -> bool {
    if metadata.file_type().is_symlink() {
        return true;
    }
    #[cfg(windows)]
    {
        use std::os::windows::fs::MetadataExt;
        if metadata.file_attributes() & 0x400 != 0 {
            return true;
        }
    }
    false
}

fn invalid_reviewed_input(message: &str) -> PytxoError {
    PytxoError::Runner(message.into())
}

fn verify_reviewed_base(repo_root: &Path, base: &BaseSnapshot) -> Result<PathBuf> {
    let repo = canonical_repo_root(repo_root).map_err(PytxoError::Io)?;
    if base.repository_identity != repo.to_string_lossy()
        || !matches!(base.git_revision.len(), 40 | 64)
        || !base
            .git_revision
            .bytes()
            .all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte))
    {
        return Err(invalid_reviewed_input(
            "reviewed Git base identity is invalid",
        ));
    }
    let observed_root = reviewed_git(&repo, &["rev-parse", "--show-toplevel"])?;
    if canonical_repo_root(Path::new(&observed_root)).map_err(PytxoError::Io)? != repo {
        return Err(invalid_reviewed_input(
            "reviewed Git repository identity changed",
        ));
    }
    if reviewed_git(&repo, &["rev-parse", "--verify", "HEAD^{commit}"])? != base.git_revision {
        // Capture and materialization both require the current reviewed HEAD.
        return Err(invalid_reviewed_input("reviewed Git HEAD changed"));
    }
    let tree_revision = format!("{}^{{tree}}", base.git_revision);
    let tree = reviewed_git(&repo, &["rev-parse", "--verify", &tree_revision])?;
    if canonical_digest(&(1_u32, "git-head-tree", &tree), 1)
        .map_err(|error| invalid_reviewed_input(&error.to_string()))?
        != base.snapshot_digest
    {
        return Err(invalid_reviewed_input("reviewed Git tree changed"));
    }
    Ok(repo)
}

fn reviewed_git(repo_root: &Path, args: &[&str]) -> Result<String> {
    Ok(
        String::from_utf8_lossy(&reviewed_git_output(repo_root, args)?)
            .trim()
            .to_owned(),
    )
}

fn reviewed_git_output(repo_root: &Path, args: &[&str]) -> Result<Vec<u8>> {
    reviewed_git_output_limited(repo_root, args, MAX_REVIEWED_INPUT_BYTES)
}

fn reviewed_git_output_limited(repo_root: &Path, args: &[&str], limit: usize) -> Result<Vec<u8>> {
    reviewed_git_output_limited_with_input(repo_root, args, limit, None)
}

fn reviewed_git_output_limited_with_input(
    repo_root: &Path,
    args: &[&str],
    limit: usize,
    input: Option<&[u8]>,
) -> Result<Vec<u8>> {
    let mut command = pytxo_core::background_command(reviewed_git_program()?);
    // Git configuration can execute hooks or fsmonitor helpers even when a
    // routed operation is intended to be metadata-only. The hook directory is
    // a fresh, absent absolute path; no repository hook is consulted.
    let inert_hooks =
        std::env::temp_dir().join(format!("pytxo-disabled-git-hooks-{}", uuid::Uuid::new_v4()));
    if !inert_hooks.is_absolute() || inert_hooks.exists() {
        return Err(PytxoError::Runner("cannot isolate routed Git hooks".into()));
    }
    let hook_config = format!("core.hooksPath={}", path_to_git(&inert_hooks));
    command
        .args(["-c", &hook_config, "-c", "core.fsmonitor=false"])
        .args(args)
        .current_dir(repo_root);
    for (key, _) in std::env::vars_os() {
        if key
            .to_string_lossy()
            .to_ascii_uppercase()
            .starts_with("GIT_")
        {
            command.env_remove(key);
        }
    }
    command.env("GIT_NO_LAZY_FETCH", "1");
    command.env("GIT_NO_REPLACE_OBJECTS", "1");
    command.env("GIT_OPTIONAL_LOCKS", "0");
    command.stdin(if input.is_some() {
        Stdio::piped()
    } else {
        Stdio::null()
    });
    let mut child = command
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .map_err(|error| PytxoError::Runner(format!("spawn routed git: {error}")))?;
    let input_writer = input.map(|bytes| {
        let mut stdin = child.stdin.take().expect("requested routed Git stdin pipe");
        let bytes = bytes.to_vec();
        std::thread::spawn(move || stdin.write_all(&bytes))
    });
    let mut stdout = child
        .stdout
        .take()
        .ok_or_else(|| invalid_reviewed_input("routed Git stdout pipe is missing"))?;
    let mut stderr = child
        .stderr
        .take()
        .ok_or_else(|| invalid_reviewed_input("routed Git stderr pipe is missing"))?;
    let stderr_reader = std::thread::spawn(move || -> std::io::Result<Vec<u8>> {
        let mut prefix = Vec::new();
        stderr.by_ref().take(65_536).read_to_end(&mut prefix)?;
        std::io::copy(&mut stderr, &mut std::io::sink())?;
        Ok(prefix)
    });
    let (stdout_sender, stdout_receiver) = std::sync::mpsc::sync_channel(1);
    let _stdout_reader = std::thread::spawn(move || {
        let mut output = Vec::new();
        let result = stdout
            .by_ref()
            .take(u64::try_from(limit).unwrap_or(u64::MAX).saturating_add(1))
            .read_to_end(&mut output)
            .map(|_| output);
        let _ = stdout_sender.send(result);
    });
    let deadline = std::time::Instant::now() + std::time::Duration::from_secs(30);
    let mut output_result = None;
    let mut timed_out = false;
    let mut child_status = None;
    let status = loop {
        if output_result.is_none() {
            if let Ok(result) = stdout_receiver.try_recv() {
                output_result = Some(result);
            }
        }
        if output_result
            .as_ref()
            .is_some_and(|result| result.as_ref().map_or(true, |output| output.len() > limit))
        {
            let _ = child.kill();
            break child.wait()?;
        }
        if child_status.is_none() {
            child_status = child.try_wait()?;
        }
        if let Some(status) = child_status.filter(|_| output_result.is_some()) {
            break status;
        }
        if std::time::Instant::now() >= deadline {
            timed_out = true;
            let _ = child.kill();
            break child.wait()?;
        }
        std::thread::sleep(std::time::Duration::from_millis(10));
    };
    let output = if timed_out {
        Vec::new()
    } else {
        output_result.ok_or_else(|| invalid_reviewed_input("routed Git stdout reader failed"))??
    };
    if let Some(writer) = input_writer {
        writer
            .join()
            .map_err(|_| invalid_reviewed_input("routed Git input writer failed"))??;
    }
    let error_output = stderr_reader
        .join()
        .map_err(|_| invalid_reviewed_input("routed Git stderr reader failed"))??;
    if timed_out {
        return Err(invalid_reviewed_input("routed Git operation timed out"));
    }
    if output.len() > limit {
        return Err(invalid_reviewed_input("routed Git output exceeds limit"));
    }
    if !status.success() {
        return Err(PytxoError::Runner(format!(
            "routed git {} failed: {}",
            args.join(" "),
            String::from_utf8_lossy(&error_output)
        )));
    }
    Ok(output)
}

fn reviewed_git_program() -> Result<PathBuf> {
    #[cfg(windows)]
    {
        static PROGRAM: std::sync::OnceLock<std::result::Result<PathBuf, String>> =
            std::sync::OnceLock::new();
        PROGRAM
            .get_or_init(|| {
                let path = std::env::var_os("PATH").ok_or("routed Git PATH is missing")?;
                for directory in std::env::split_paths(&path) {
                    if !directory.is_absolute() {
                        continue;
                    }
                    let shim = directory.join("git.exe");
                    if !std::fs::symlink_metadata(&shim).is_ok_and(|metadata| {
                        metadata.file_type().is_file() && !is_reparse(&metadata)
                    }) {
                        continue;
                    }
                    let direct = if directory
                        .file_name()
                        .is_some_and(|name| name.to_string_lossy().eq_ignore_ascii_case("cmd"))
                    {
                        directory
                            .parent()
                            .map(|root| root.join("mingw64").join("bin").join("git.exe"))
                    } else {
                        None
                    };
                    if let Some(program) = direct {
                        if std::fs::symlink_metadata(&program).is_ok_and(|metadata| {
                            metadata.file_type().is_file() && !is_reparse(&metadata)
                        }) {
                            return canonical_repo_root(&program)
                                .map_err(|error| error.to_string());
                        }
                    }
                }
                Err("no direct Git for Windows executable was found beside a PATH cmd shim".into())
            })
            .as_ref()
            .cloned()
            .map_err(|message| invalid_reviewed_input(message))
    }
    #[cfg(not(windows))]
    {
        Ok(PathBuf::from("git"))
    }
}

pub fn remove_worktree(
    repo_root: &Path,
    worktree_path: &Path,
    branch: &str,
    force: bool,
) -> Result<()> {
    if worktree_path.exists() {
        let path = path_to_git(worktree_path);
        if force {
            let _ = git(repo_root, &["worktree", "remove", "--force", &path]);
        } else {
            let _ = git(repo_root, &["worktree", "remove", &path]);
        }
    }
    let _ = git(repo_root, &["branch", "-D", branch]);
    Ok(())
}

/// Remove only a detached worktree still registered to this repository and
/// exact reviewed commit. Used after a routed candidate is Applied/discarded.
pub fn remove_reviewed_worktree_if_owned(
    repo_root: &Path,
    worktree_path: &Path,
    base: &BaseSnapshot,
) -> Result<()> {
    if !worktree_path.exists() {
        return Ok(());
    }
    verify_reviewed_worktree_registration(repo_root, worktree_path, base)?;
    read_plain_tree(worktree_path, true)?;
    let path = path_to_git(worktree_path);
    reviewed_git(repo_root, &["worktree", "remove", "--force", &path])?;
    if worktree_path.exists() {
        return Err(invalid_reviewed_input(
            "owned worktree remains after cleanup",
        ));
    }
    Ok(())
}

pub fn branch_name(run_id: &str, agent_id: &str) -> String {
    format!("pytxo/{run_id}/{agent_id}")
}

pub fn worktree_path(base: &Path, run_id: &str, agent_id: &str) -> PathBuf {
    base.join(run_id).join(agent_id)
}

/// Merge an agent worktree branch into the repo's current HEAD (Blast Shield approve).
pub fn merge_agent_branch(repo_root: &Path, branch: &str) -> Result<()> {
    if branch.is_empty() {
        return Ok(());
    }
    git(
        repo_root,
        &[
            "merge",
            branch,
            "--no-edit",
            "-m",
            "pytxo blast-shield flush",
        ],
    )?;
    let _ = git(repo_root, &["branch", "-d", branch]);
    Ok(())
}

fn git(repo_root: &Path, args: &[&str]) -> Result<String> {
    let output = pytxo_core::background_command("git")
        .args(args)
        .current_dir(repo_root)
        .output()
        .map_err(|e| PytxoError::Runner(format!("spawn git: {e}")))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(PytxoError::Runner(format!(
            "git {} failed: {stderr}",
            args.join(" ")
        )));
    }
    Ok(String::from_utf8_lossy(&output.stdout).trim().to_string())
}

fn path_to_git(path: &Path) -> String {
    strip_extended_path(path.to_path_buf())
        .to_string_lossy()
        .to_string()
}
