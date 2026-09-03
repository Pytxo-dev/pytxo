//! Deterministic overlay-upper delta for cloud hybrid sync ([[delta-sync]]).

use std::collections::BTreeMap;
use std::fs::{self, File, OpenOptions};
use std::io::Read;
use std::path::Path;

use crate::cloud::{cloud_path_denied, content_hash, validate_cloud_content, SyncFile};
use crate::{PytxoError, Result};

/// Sorted, hash-fingerprinted file delta from an overlay upper layer.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct OverlayDelta {
    pub files: Vec<SyncFile>,
    /// Stable fingerprint over relative path + content hashes (sorted).
    pub fingerprint: String,
}

/// Walk `upper` and emit files whose bytes differ from `base` (or are absent in base).
///
/// Paths are repo-relative, sorted lexicographically for deterministic wire order.
pub fn delta_from_overlay_upper(base: &Path, upper: &Path) -> Result<OverlayDelta> {
    if !upper.is_dir() {
        return Ok(OverlayDelta {
            files: Vec::new(),
            fingerprint: content_hash(b"empty-upper"),
        });
    }

    let mut by_path: BTreeMap<String, SyncFile> = BTreeMap::new();
    walk_upper(base, upper, upper, &mut by_path)?;

    let files: Vec<SyncFile> = by_path.into_values().collect();
    let fingerprint = fingerprint_delta(&files);
    Ok(OverlayDelta { files, fingerprint })
}

fn walk_upper(
    base: &Path,
    upper_root: &Path,
    current: &Path,
    out: &mut BTreeMap<String, SyncFile>,
) -> Result<()> {
    for entry in fs::read_dir(current)
        .map_err(|e| PytxoError::Other(format!("delta readdir {}: {e}", current.display())))?
    {
        let entry = entry.map_err(|e| PytxoError::Other(format!("delta entry: {e}")))?;
        let path = entry.path();
        let rel = path
            .strip_prefix(upper_root)
            .map_err(|e| PytxoError::Other(format!("delta rel: {e}")))?;
        let rel_key = rel.to_string_lossy().replace('\\', "/");
        if cloud_path_denied(&rel_key) {
            continue;
        }
        let file_type = entry
            .file_type()
            .map_err(|e| PytxoError::Other(e.to_string()))?;
        if file_type.is_dir() {
            walk_upper(base, upper_root, &path, out)?;
            continue;
        }
        if file_type.is_symlink() {
            return Err(PytxoError::CloudPolicy(format!(
                "cloud upload denied for symlink: {rel_key}"
            )));
        }
        if !file_type.is_file() {
            return Err(PytxoError::CloudPolicy(format!(
                "cloud upload denied for special file: {rel_key}"
            )));
        }
        let Some(upper_bytes) = read_regular_file_no_follow(&path, &rel_key)? else {
            continue;
        };
        let base_path = base.join(rel);
        let changed = match fs::read(&base_path) {
            Ok(base_bytes) => base_bytes != upper_bytes,
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => true,
            Err(e) => return Err(PytxoError::Other(format!("delta base read: {e}"))),
        };
        if changed {
            let content = String::from_utf8_lossy(&upper_bytes).into_owned();
            validate_cloud_content(&rel_key, &content)?;
            out.insert(
                rel_key.clone(),
                SyncFile {
                    path: rel_key,
                    content,
                },
            );
        }
    }
    Ok(())
}

fn fingerprint_delta(files: &[SyncFile]) -> String {
    let mut acc = String::new();
    for f in files {
        acc.push_str(&f.path);
        acc.push('\0');
        acc.push_str(&content_hash(f.content.as_bytes()));
        acc.push('\n');
    }
    content_hash(acc.as_bytes())
}

fn path_excluded(rel: &str, sparse_exclude: &[String]) -> bool {
    let first = rel.split('/').next().unwrap_or(rel);
    sparse_exclude.iter().any(|e| e == first)
}

/// Walk `repo_root` and collect text files for initial cloud `sync_delta`, skipping
/// top-level names in `sparse_exclude` ([[blast-shield]] Phase 33).
pub fn collect_sync_paths(repo_root: &Path, sparse_exclude: &[String]) -> Result<Vec<SyncFile>> {
    let mut by_path: BTreeMap<String, SyncFile> = BTreeMap::new();
    if repo_root.is_dir() {
        walk_repo_sync(repo_root, repo_root, sparse_exclude, &mut by_path)?;
    }
    Ok(by_path.into_values().collect())
}

fn walk_repo_sync(
    repo_root: &Path,
    current: &Path,
    sparse_exclude: &[String],
    out: &mut BTreeMap<String, SyncFile>,
) -> Result<()> {
    for entry in fs::read_dir(current)
        .map_err(|e| PytxoError::Other(format!("sync readdir {}: {e}", current.display())))?
    {
        let entry = entry.map_err(|e| PytxoError::Other(format!("sync entry: {e}")))?;
        let path = entry.path();
        let rel = path
            .strip_prefix(repo_root)
            .map_err(|e| PytxoError::Other(format!("sync rel: {e}")))?;
        let rel_key = rel.to_string_lossy().replace('\\', "/");
        if path_excluded(&rel_key, sparse_exclude) || cloud_path_denied(&rel_key) {
            continue;
        }
        let file_type = entry
            .file_type()
            .map_err(|e| PytxoError::Other(e.to_string()))?;
        if file_type.is_dir() {
            walk_repo_sync(repo_root, &path, sparse_exclude, out)?;
            continue;
        }
        if file_type.is_symlink() {
            return Err(PytxoError::CloudPolicy(format!(
                "cloud upload denied for symlink: {rel_key}"
            )));
        }
        if !file_type.is_file() {
            return Err(PytxoError::CloudPolicy(format!(
                "cloud upload denied for special file: {rel_key}"
            )));
        }
        let Some(bytes) = read_regular_file_no_follow(&path, &rel_key)? else {
            continue;
        };
        if bytes.contains(&0) {
            continue;
        }
        let content = String::from_utf8_lossy(&bytes).into_owned();
        validate_cloud_content(&rel_key, &content)?;
        out.insert(
            rel_key.clone(),
            SyncFile {
                path: rel_key,
                content,
            },
        );
    }
    Ok(())
}

const MAX_CLOUD_FILE_BYTES: u64 = 512 * 1024;

fn open_no_follow(path: &Path) -> std::io::Result<File> {
    let mut options = OpenOptions::new();
    options.read(true);
    #[cfg(unix)]
    {
        use std::os::unix::fs::OpenOptionsExt;
        options.custom_flags(libc::O_NOFOLLOW);
    }
    #[cfg(windows)]
    {
        use std::os::windows::fs::OpenOptionsExt;
        // Open the reparse point itself so the handle metadata check rejects it.
        options.custom_flags(0x0020_0000);
    }
    options.open(path)
}

/// Read outbound bytes from the validated file handle, never from the path
/// after validation. This closes the final-component symlink replacement race.
fn read_regular_file_no_follow(path: &Path, rel_key: &str) -> Result<Option<Vec<u8>>> {
    let mut file = open_no_follow(path).map_err(|error| {
        PytxoError::CloudPolicy(format!(
            "cloud upload denied while opening {rel_key} without following links: {error}"
        ))
    })?;
    let metadata = file.metadata().map_err(PytxoError::Io)?;
    if !metadata.file_type().is_file() {
        return Err(PytxoError::CloudPolicy(format!(
            "cloud upload denied for non-regular file: {rel_key}"
        )));
    }
    if metadata.len() > MAX_CLOUD_FILE_BYTES {
        return Ok(None);
    }
    let mut bytes = Vec::with_capacity(metadata.len() as usize);
    file.read_to_end(&mut bytes).map_err(PytxoError::Io)?;
    Ok(Some(bytes))
}

/// Returns true when `worktree` looks like an overlay upper directory.
pub fn is_overlay_upper(worktree: &Path) -> bool {
    worktree.to_string_lossy().contains("overlay-")
        || worktree.file_name().is_some_and(|n| n == "upper")
}

/// Cloud hybrid hook: incremental sync from overlay upper when active ([[delta-sync]], Phase 46).
///
/// Returns `None` when `worktree` is not an overlay upper — callers should fall back to
/// [`collect_sync_paths`] for initial sandbox seeding.
pub fn overlay_upper_cloud_delta(base: &Path, worktree: &Path) -> Option<Result<OverlayDelta>> {
    if is_overlay_upper(worktree) {
        Some(delta_from_overlay_upper(base, worktree))
    } else {
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    #[test]
    fn delta_detects_new_and_changed_files() {
        let tmp = tempfile::tempdir().unwrap();
        let base = tmp.path().join("repo");
        let upper = tmp.path().join("upper");
        fs::create_dir_all(base.join("src")).unwrap();
        fs::create_dir_all(upper.join("src")).unwrap();
        fs::write(base.join("README.md"), "base\n").unwrap();
        fs::write(upper.join("README.md"), "changed\n").unwrap();
        fs::write(base.join("src/a.rs"), "fn a() {}\n").unwrap();
        fs::write(upper.join("src/a.rs"), "fn a() {}\n").unwrap();
        fs::write(upper.join("src/new.rs"), "fn new() {}\n").unwrap();

        let delta = delta_from_overlay_upper(&base, &upper).unwrap();
        let paths: Vec<_> = delta.files.iter().map(|f| f.path.as_str()).collect();
        assert_eq!(paths, vec!["README.md", "src/new.rs"]);
        assert!(!delta.fingerprint.is_empty());
    }

    #[test]
    fn delta_order_is_deterministic() {
        let tmp = tempfile::tempdir().unwrap();
        let base = tmp.path().join("base");
        let upper = tmp.path().join("upper");
        fs::create_dir_all(&base).unwrap();
        fs::create_dir_all(upper.join("z")).unwrap();
        fs::create_dir_all(upper.join("a")).unwrap();
        fs::write(upper.join("z/z.txt"), "z").unwrap();
        fs::write(upper.join("a/a.txt"), "a").unwrap();

        let d1 = delta_from_overlay_upper(&base, &upper).unwrap();
        let d2 = delta_from_overlay_upper(&base, &upper).unwrap();
        assert_eq!(d1, d2);
        assert_eq!(
            d1.files.iter().map(|f| f.path.as_str()).collect::<Vec<_>>(),
            vec!["a/a.txt", "z/z.txt"]
        );
    }

    #[test]
    fn collect_sync_paths_skips_sparse_exclude() {
        let tmp = tempfile::tempdir().unwrap();
        let repo = tmp.path().join("repo");
        fs::create_dir_all(repo.join("node_modules")).unwrap();
        fs::create_dir_all(repo.join("src")).unwrap();
        fs::write(repo.join("README.md"), "hi\n").unwrap();
        fs::write(repo.join("node_modules/x.js"), "x").unwrap();
        fs::write(repo.join("src/lib.rs"), "fn main() {}\n").unwrap();
        let files = collect_sync_paths(&repo, &["node_modules".into()]).unwrap();
        let paths: Vec<_> = files.iter().map(|f| f.path.as_str()).collect();
        assert!(paths.contains(&"README.md"));
        assert!(paths.contains(&"src/lib.rs"));
        assert!(!paths.iter().any(|p| p.starts_with("node_modules")));
    }

    #[test]
    fn collect_sync_paths_omits_denied_paths_but_keeps_allowed_source() {
        let tmp = tempfile::tempdir().unwrap();
        let repo = tmp.path().join("repo");
        fs::create_dir_all(repo.join(".pytxo/data")).unwrap();
        fs::create_dir_all(repo.join("nested")).unwrap();
        fs::create_dir_all(repo.join("src")).unwrap();
        fs::write(repo.join(".env"), "OPENAI_API_KEY=not-uploaded").unwrap();
        fs::write(repo.join(".pytxo/data/run.db"), "private control data").unwrap();
        fs::write(repo.join("nested/private.pem"), "private key bytes").unwrap();
        fs::write(repo.join("src/lib.rs"), "pub fn allowed() {}\n").unwrap();

        let files = collect_sync_paths(&repo, &[]).unwrap();
        assert_eq!(files.len(), 1);
        assert_eq!(files[0].path, "src/lib.rs");
        assert_eq!(files[0].content, "pub fn allowed() {}\n");
    }

    #[test]
    fn collect_sync_paths_fails_closed_on_secret_content() {
        let tmp = tempfile::tempdir().unwrap();
        let repo = tmp.path().join("repo");
        fs::create_dir_all(repo.join("src")).unwrap();
        fs::write(repo.join("src/lib.rs"), "pub fn allowed() {}\n").unwrap();
        fs::write(
            repo.join("src/config.txt"),
            "GITHUB_TOKEN=ghp_abcdefghijklmnopqrstuvwxyz123456",
        )
        .unwrap();

        let error = collect_sync_paths(&repo, &[]).unwrap_err();
        assert!(error.to_string().contains("likely GitHub token"));
    }

    #[test]
    fn overlay_delta_omits_denied_paths_and_fails_closed_on_secret_content() {
        let tmp = tempfile::tempdir().unwrap();
        let base = tmp.path().join("base");
        let upper = tmp.path().join("upper");
        fs::create_dir_all(&base).unwrap();
        fs::create_dir_all(upper.join(".pytxo")).unwrap();
        fs::create_dir_all(upper.join("src")).unwrap();
        fs::write(upper.join(".pytxo/state.json"), "hidden").unwrap();
        fs::write(upper.join("src/lib.rs"), "pub fn allowed() {}\n").unwrap();

        let delta = delta_from_overlay_upper(&base, &upper).unwrap();
        assert_eq!(delta.files.len(), 1);
        assert_eq!(delta.files[0].path, "src/lib.rs");

        fs::write(
            upper.join("src/credential.txt"),
            "AWS_ACCESS_KEY_ID=AKIAIOSFODNN7EXAMPLE",
        )
        .unwrap();
        assert!(delta_from_overlay_upper(&base, &upper).is_err());
    }

    #[test]
    fn initial_sync_rejects_a_symlink_instead_of_following_it() {
        let tmp = tempfile::tempdir().unwrap();
        let repo = tmp.path().join("repo");
        fs::create_dir_all(repo.join("src")).unwrap();
        let target = tmp.path().join("outside-secret.txt");
        fs::write(&target, "GITHUB_TOKEN=ghp_abcdefghijklmnopqrstuvwxyz123456").unwrap();
        if !create_file_symlink(&target, &repo.join("src/link.txt")) {
            return;
        }

        let error = collect_sync_paths(&repo, &[]).unwrap_err();
        assert!(matches!(error, PytxoError::CloudPolicy(_)));
    }

    #[test]
    fn overlay_delta_rejects_a_symlink_instead_of_following_it() {
        let tmp = tempfile::tempdir().unwrap();
        let base = tmp.path().join("base");
        let upper = tmp.path().join("upper");
        fs::create_dir_all(&base).unwrap();
        fs::create_dir_all(upper.join("src")).unwrap();
        let target = tmp.path().join("outside-secret.txt");
        fs::write(&target, "GITHUB_TOKEN=ghp_abcdefghijklmnopqrstuvwxyz123456").unwrap();
        if !create_file_symlink(&target, &upper.join("src/link.txt")) {
            return;
        }

        let error = delta_from_overlay_upper(&base, &upper).unwrap_err();
        assert!(matches!(error, PytxoError::CloudPolicy(_)));
    }

    fn create_file_symlink(target: &Path, link: &Path) -> bool {
        #[cfg(unix)]
        {
            std::os::unix::fs::symlink(target, link).is_ok()
        }
        #[cfg(windows)]
        {
            std::os::windows::fs::symlink_file(target, link).is_ok()
        }
        #[cfg(not(any(unix, windows)))]
        {
            let _ = (target, link);
            false
        }
    }

    #[test]
    fn empty_upper_yields_empty_delta() {
        let tmp = tempfile::tempdir().unwrap();
        let delta = delta_from_overlay_upper(tmp.path(), &tmp.path().join("missing")).unwrap();
        assert!(delta.files.is_empty());
    }
}
