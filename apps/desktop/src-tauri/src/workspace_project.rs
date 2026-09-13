//! Desktop folder grouping. This writes project metadata, never trust grants.
use std::path::{Path, PathBuf};

use pytxo_core::{ProjectManifest, ProjectMeta, ProjectRoot};

use crate::ipc_error::{map_io_err, IpcResult, PytxoIpcError};

pub fn checked_folder(path: &Path, existing: &[PathBuf]) -> IpcResult<PathBuf> {
    let path = pytxo_core::canonical_repo_root(path).map_err(map_io_err)?;
    if !path.is_dir() {
        return Err(PytxoIpcError::new(
            "project_folder",
            "Choose a folder, not a file.",
        ));
    }
    for root in existing {
        let root = pytxo_core::canonical_repo_root(root).map_err(map_io_err)?;
        if root == path || root.starts_with(&path) || path.starts_with(&root) {
            return Err(PytxoIpcError::new(
                "project_folder",
                "This folder overlaps an existing root. Choose a separate folder.",
            ));
        }
        if root.file_name().map(|s| s.to_string_lossy().to_lowercase())
            == path.file_name().map(|s| s.to_string_lossy().to_lowercase())
        {
            return Err(PytxoIpcError::new(
                "project_folder",
                "A folder with this name is already attached. Use the project CLI to assign distinct root labels.",
            ));
        }
    }
    Ok(path)
}

pub fn new_manifest(id: String, primary: &Path, additional: &Path) -> IpcResult<ProjectManifest> {
    let primary = checked_folder(primary, &[])?;
    let additional = checked_folder(additional, std::slice::from_ref(&primary))?;
    Ok(ProjectManifest {
        project: ProjectMeta {
            id,
            name: primary
                .file_name()
                .map(|s| s.to_string_lossy().into_owned()),
        },
        roots: vec![primary, additional]
            .into_iter()
            .enumerate()
            .map(|(i, path)| ProjectRoot {
                path,
                label: None,
                primary: i == 0,
                read_only: false,
                permission_profile: None,
            })
            .collect(),
    })
}

pub fn check_label(path: &Path, labels: &[String]) -> IpcResult<()> {
    let root = ProjectRoot {
        path: path.to_path_buf(),
        label: None,
        primary: false,
        read_only: false,
        permission_profile: None,
    };
    let label = root.effective_label();
    if labels
        .iter()
        .any(|existing| existing.eq_ignore_ascii_case(&label))
    {
        return Err(PytxoIpcError::new("project_label", format!("The root label '{label}' is already used. Choose another folder or assign a distinct label with the project CLI.")));
    }
    Ok(())
}

/// create_new prevents an existing manifest from being overwritten, even on collision.
pub fn write_new(path: &Path, manifest: &ProjectManifest) -> IpcResult<()> {
    use std::io::Write;
    let bytes = toml::to_string_pretty(manifest).map_err(map_io_err)?;
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent).map_err(map_io_err)?;
    }
    let mut file = std::fs::OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(path)
        .map_err(map_io_err)?;
    file.write_all(bytes.as_bytes()).map_err(map_io_err)?;
    file.sync_all().map_err(map_io_err)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn adding_a_folder_cannot_shadow_a_custom_root_label() {
        assert!(check_label(Path::new("shared"), &["shared".into()]).is_err());
        assert!(check_label(Path::new("Shared"), &["shared".into()]).is_err());
        assert!(check_label(Path::new("client"), &["shared".into()]).is_ok());
    }

    #[test]
    fn grouping_preserves_files_and_does_not_grant_permissions() {
        let temp = tempfile::tempdir().unwrap();
        let a = temp.path().join("app");
        let b = temp.path().join("shared");
        std::fs::create_dir(&a).unwrap();
        std::fs::create_dir(&b).unwrap();
        std::fs::write(a.join("user.txt"), "keep me").unwrap();
        let manifest = new_manifest("test".into(), &a, &b).unwrap();
        let path = temp.path().join("project.toml");
        write_new(&path, &manifest).unwrap();
        let loaded = ProjectManifest::load(&path).unwrap();
        assert_eq!(loaded.roots.len(), 2);
        assert!(loaded.roots[0].primary);
        assert!(loaded
            .roots
            .iter()
            .all(|root| root.permission_profile.is_none()));
        assert_eq!(
            std::fs::read_to_string(a.join("user.txt")).unwrap(),
            "keep me"
        );
        let before = std::fs::read(&path).unwrap();
        assert!(write_new(&path, &manifest).is_err());
        assert_eq!(std::fs::read(&path).unwrap(), before);
    }

    #[test]
    fn duplicate_nested_missing_and_file_roots_are_rejected() {
        let temp = tempfile::tempdir().unwrap();
        let a = temp.path().join("app");
        let nested = a.join("nested");
        std::fs::create_dir_all(&nested).unwrap();
        let file = temp.path().join("file");
        std::fs::write(&file, "data").unwrap();
        for invalid in [
            &a,
            &nested,
            temp.path(),
            &file,
            &temp.path().join("missing"),
        ] {
            assert!(
                new_manifest("test".into(), &a, invalid).is_err(),
                "{invalid:?}"
            );
        }
    }
}
