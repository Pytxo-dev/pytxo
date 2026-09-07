use std::path::{Path, PathBuf};

/// Normalize supported Windows verbatim disk/UNC paths for external tools.
pub fn strip_extended_path(path: PathBuf) -> PathBuf {
    #[cfg(windows)]
    {
        use std::ffi::OsString;
        use std::path::{Component, Prefix};

        let mut components = path.components();
        if let Some(Component::Prefix(prefix)) = components.next() {
            let mut normal = match prefix.kind() {
                Prefix::VerbatimDisk(letter) => OsString::from(format!("{}:", letter as char)),
                Prefix::VerbatimUNC(server, share) => {
                    let mut unc = OsString::from(r"\\");
                    unc.push(server);
                    unc.push(r"\");
                    unc.push(share);
                    unc
                }
                // Other namespaces have no equivalent ordinary path.
                _ => return path,
            };
            normal.push(components.as_path().as_os_str());
            return PathBuf::from(normal);
        }
    }
    path
}

pub fn canonical_repo_root(path: &Path) -> std::io::Result<PathBuf> {
    let canonical = std::fs::canonicalize(path)?;
    Ok(strip_extended_path(canonical))
}

#[cfg(all(test, windows))]
mod tests {
    use super::*;

    #[test]
    fn extended_path_keeps_unc_storage_absolute() {
        let normal = strip_extended_path(PathBuf::from(r"\\?\UNC\server\share\.pytxo\data"));
        assert_eq!(normal, PathBuf::from(r"\\server\share\.pytxo\data"));
        assert!(normal.is_absolute());
    }

    #[test]
    fn extended_path_normalizes_drive_paths() {
        let normal = strip_extended_path(PathBuf::from(r"\\?\C:\pytxo\src"));
        assert_eq!(normal, PathBuf::from(r"C:\pytxo\src"));
        assert!(normal.is_absolute());
    }

    #[test]
    fn extended_path_preserves_multilingual_unc_components() {
        assert_eq!(
            strip_extended_path(PathBuf::from(r"\\?\UNC\sérver\共有\โครงการ\file.md")),
            PathBuf::from(r"\\sérver\共有\โครงการ\file.md")
        );
    }

    #[test]
    fn extended_path_preserves_normal_and_unsupported_namespaces() {
        for path in [
            r"C:\pytxo\src",
            r"\\server\share\data",
            r"relative\data",
            r"\\.\COM1",
            r"\\?\Volume{00000000-0000-0000-0000-000000000000}\data",
        ] {
            assert_eq!(
                strip_extended_path(PathBuf::from(path)),
                PathBuf::from(path)
            );
        }
    }

    #[test]
    fn extended_path_preserves_non_unicode_windows_components() {
        use std::ffi::OsString;
        use std::os::windows::ffi::{OsStrExt, OsStringExt};

        let mut original = r"\\?\C:\pytxo\".encode_utf16().collect::<Vec<_>>();
        original.push(0xd800);
        original.extend(".txt".encode_utf16());
        let path = PathBuf::from(OsString::from_wide(&original));
        let normal = strip_extended_path(path);
        assert_eq!(
            normal.as_os_str().encode_wide().collect::<Vec<_>>(),
            original[4..]
        );
    }
}
