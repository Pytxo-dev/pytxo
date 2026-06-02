use std::path::{Path, PathBuf};

/// Strip Windows `\\?\` verbatim prefix so external tools (e.g. git) accept paths.
pub fn strip_extended_path(path: PathBuf) -> PathBuf {
    #[cfg(windows)]
    {
        let s = path.to_string_lossy();
        if let Some(rest) = s.strip_prefix(r"\\?\") {
            return PathBuf::from(rest);
        }
    }
    path
}

pub fn canonical_repo_root(path: &Path) -> std::io::Result<PathBuf> {
    let canonical = std::fs::canonicalize(path)?;
    Ok(strip_extended_path(canonical))
}
