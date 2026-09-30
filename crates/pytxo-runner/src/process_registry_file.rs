use std::fs::{self, OpenOptions};
use std::io::Write;
use std::path::{Path, PathBuf};

use fs2::FileExt;
use pytxo_core::{PytxoError, Result};
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize, Default)]
pub struct ProcessRegistryFile {
    pub entries: Vec<ProcessEntry>,
    /// Durable cancellation prevents later wave/retry/verifier spawns after Stop.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub cancelled_runs: Vec<String>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ProcessEntry {
    pub run_id: String,
    pub repo_root: String,
    pub agent_key: String,
    pub pid: u32,
    /// OS process creation identity used to reject stale/reused PIDs.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub start_identity: Option<String>,
    pub worktree_path: String,
    pub branch: String,
}

impl ProcessRegistryFile {
    pub fn load(path: &Path) -> Result<Self> {
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent).map_err(PytxoError::Io)?;
        }
        let lock_path = sibling_path(path, ".lock");
        let lock = OpenOptions::new()
            .create(true)
            .truncate(false)
            .read(true)
            .write(true)
            .open(&lock_path)
            .map_err(|error| registry_io("open read lock", &lock_path, error))?;
        FileExt::lock_shared(&lock)
            .map_err(|error| registry_io("acquire read lock", &lock_path, error))?;
        read_registry(path)
    }

    pub fn save(&self, path: &Path) -> Result<()> {
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent).map_err(PytxoError::Io)?;
        }
        let lock_path = sibling_path(path, ".lock");
        let lock = OpenOptions::new()
            .create(true)
            .truncate(false)
            .read(true)
            .write(true)
            .open(&lock_path)
            .map_err(|error| registry_io("open write lock", &lock_path, error))?;
        lock.lock_exclusive()
            .map_err(|error| registry_io("acquire write lock", &lock_path, error))?;
        self.save_unlocked(path)
    }

    pub fn update<T>(
        path: &Path,
        update: impl FnOnce(&mut ProcessRegistryFile) -> Result<T>,
    ) -> Result<T> {
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent).map_err(PytxoError::Io)?;
        }
        let lock_path = sibling_path(path, ".lock");
        let lock = OpenOptions::new()
            .create(true)
            .truncate(false)
            .read(true)
            .write(true)
            .open(&lock_path)
            .map_err(|error| registry_io("open update lock", &lock_path, error))?;
        lock.lock_exclusive()
            .map_err(|error| registry_io("acquire update lock", &lock_path, error))?;

        let mut registry = read_registry(path)?;
        let output = update(&mut registry)?;
        registry.save_unlocked(path)?;
        Ok(output)
    }

    fn save_unlocked(&self, path: &Path) -> Result<()> {
        let raw = serde_json::to_string_pretty(self)
            .map_err(|e| PytxoError::Runner(format!("serialize registry: {e}")))?;
        let backup_path = sibling_path(path, ".bak");
        if read_registry_file(path).is_ok_and(|registry| registry.is_some()) {
            fs::copy(path, &backup_path)
                .map_err(|error| registry_io("write backup", &backup_path, error))?;
        }

        let temp_path = sibling_path(path, &format!(".{}.tmp", uuid::Uuid::new_v4()));
        let write_result = (|| -> Result<()> {
            let mut temp = OpenOptions::new()
                .create_new(true)
                .write(true)
                .open(&temp_path)
                .map_err(|error| registry_io("create replacement", &temp_path, error))?;
            temp.write_all(raw.as_bytes())
                .map_err(|error| registry_io("write replacement", &temp_path, error))?;
            temp.sync_all()
                .map_err(|error| registry_io("sync replacement", &temp_path, error))?;
            drop(temp);

            // rename replaces an existing destination on Windows too. Keep the
            // previous registry in place until publication, rather than creating
            // a delete/rename gap that can lose the latest cancellation state.
            fs::rename(&temp_path, path)
                .map_err(|error| registry_io("publish replacement", path, error))
        })();
        if write_result.is_err() {
            let _ = fs::remove_file(temp_path);
        }
        write_result
    }

    pub fn push(&mut self, entry: ProcessEntry) {
        self.entries.retain(|e| e.agent_key != entry.agent_key);
        self.entries.push(entry);
    }

    pub fn for_run(&self, run_id: &str) -> Vec<&ProcessEntry> {
        self.entries.iter().filter(|e| e.run_id == run_id).collect()
    }

    pub fn all_pids(&self) -> Vec<u32> {
        self.entries.iter().map(|e| e.pid).collect()
    }

    pub fn remove_run(&mut self, run_id: &str) {
        self.entries.retain(|e| e.run_id != run_id);
    }

    pub fn remove_agent(&mut self, agent_key: &str) {
        self.entries.retain(|entry| entry.agent_key != agent_key);
    }

    pub fn clear(&mut self) {
        self.entries.clear();
    }
}

fn read_registry(path: &Path) -> Result<ProcessRegistryFile> {
    match read_registry_file(path) {
        Ok(Some(registry)) => Ok(registry),
        Ok(None) => read_registry_file(&sibling_path(path, ".bak"))
            .map(|registry| registry.unwrap_or_default()),
        Err(primary_error) => match read_registry_file(&sibling_path(path, ".bak")) {
            Ok(Some(registry)) => Ok(registry),
            _ => Err(primary_error),
        },
    }
}

fn read_registry_file(path: &Path) -> Result<Option<ProcessRegistryFile>> {
    if !path.exists() {
        return Ok(None);
    }
    let raw =
        fs::read_to_string(path).map_err(|error| registry_io("read registry", path, error))?;
    serde_json::from_str(&raw)
        .map(Some)
        .map_err(|error| PytxoError::Runner(format!("process registry: {error}")))
}

fn sibling_path(path: &Path, suffix: &str) -> PathBuf {
    let name = path
        .file_name()
        .and_then(|name| name.to_str())
        .unwrap_or("process_registry.json");
    path.with_file_name(format!("{name}{suffix}"))
}

fn registry_io(operation: &str, path: &Path, error: std::io::Error) -> PytxoError {
    PytxoError::Runner(format!(
        "process registry {operation} at {}: {error}",
        path.display()
    ))
}

pub fn registry_path(data_dir: &Path) -> std::path::PathBuf {
    data_dir.join("process_registry.json")
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::{Arc, Barrier};

    #[test]
    fn concurrent_updates_keep_every_agent() {
        let temp = tempfile::tempdir().unwrap();
        let path = temp.path().join("process_registry.json");
        let barrier = Arc::new(Barrier::new(8));
        let threads = (0..8)
            .map(|index| {
                let path = path.clone();
                let barrier = barrier.clone();
                std::thread::spawn(move || {
                    barrier.wait();
                    ProcessRegistryFile::update(&path, |registry| {
                        registry.push(ProcessEntry {
                            run_id: "run".into(),
                            repo_root: "repo".into(),
                            agent_key: format!("run:agent-{index}"),
                            pid: index + 1,
                            start_identity: None,
                            worktree_path: format!("worktree-{index}"),
                            branch: String::new(),
                        });
                        Ok(())
                    })
                    .unwrap();
                })
            })
            .collect::<Vec<_>>();
        for thread in threads {
            thread.join().unwrap();
        }

        let registry = ProcessRegistryFile::load(&path).unwrap();
        assert_eq!(registry.entries.len(), 8);
    }

    #[test]
    fn polling_readers_do_not_break_verifier_registration() {
        let temp = tempfile::tempdir().unwrap();
        let path = temp.path().join("process_registry.json");
        ProcessRegistryFile::default().save(&path).unwrap();
        std::thread::scope(|scope| {
            for _ in 0..4 {
                let path = &path;
                scope.spawn(move || {
                    for _ in 0..100 {
                        ProcessRegistryFile::load(path).expect("poll registry");
                    }
                });
            }
            for index in 0..100 {
                ProcessRegistryFile::update(&path, |registry| {
                    registry.push(ProcessEntry {
                        run_id: "run".into(),
                        repo_root: "repo".into(),
                        agent_key: "run:verifier".into(),
                        pid: index + 1,
                        start_identity: Some(format!("identity-{index}")),
                        worktree_path: "isolated".into(),
                        branch: String::new(),
                    });
                    Ok(())
                })
                .expect("register verifier while polling");
                ProcessRegistryFile::update(&path, |registry| {
                    registry.remove_agent("run:verifier");
                    Ok(())
                })
                .expect("remove verifier while polling");
            }
        });
        assert!(ProcessRegistryFile::load(&path).unwrap().entries.is_empty());
    }

    #[cfg(windows)]
    #[test]
    fn replacement_with_open_reader_preserves_latest_cancellation() {
        use std::os::windows::fs::OpenOptionsExt;

        let temp = tempfile::tempdir().unwrap();
        let path = temp.path().join("process_registry.json");
        ProcessRegistryFile::default().save(&path).unwrap();
        // A reader permits replacement but keeps the previous file object alive.
        let held = OpenOptions::new()
            .read(true)
            .share_mode(1 | 2 | 4)
            .open(&path)
            .unwrap();
        ProcessRegistryFile::update(&path, |registry| {
            registry.cancelled_runs.push("stopped-run".into());
            Ok(())
        })
        .expect("publish cancellation while a compatible reader remains open");
        assert_eq!(
            ProcessRegistryFile::load(&path).unwrap().cancelled_runs,
            ["stopped-run"]
        );
        drop(held);
    }

    #[cfg(windows)]
    #[test]
    fn sharing_failure_identifies_operation_and_preserves_registry() {
        use std::os::windows::fs::OpenOptionsExt;

        let temp = tempfile::tempdir().unwrap();
        let path = temp.path().join("process_registry.json");
        ProcessRegistryFile::default().save(&path).unwrap();
        let before = fs::read(&path).unwrap();
        let held = OpenOptions::new()
            .read(true)
            // Allow readers, but prevent replacement until this handle closes.
            .share_mode(1)
            .open(&path)
            .unwrap();
        let error = ProcessRegistryFile::default().save(&path).unwrap_err();
        let message = error.to_string();
        assert!(message.contains("publish replacement"), "{message}");
        // Windows replacement can report ACCESS_DENIED rather than the
        // SHARING_VIOLATION returned by the former delete-first operation.
        assert!(
            message.contains("os error 5") || message.contains("os error 32"),
            "{message}"
        );
        drop(held);
        assert_eq!(fs::read(&path).unwrap(), before);
        assert!(ProcessRegistryFile::load(&path).unwrap().entries.is_empty());
    }

    #[test]
    fn load_recovers_last_valid_backup() {
        let temp = tempfile::tempdir().unwrap();
        let path = temp.path().join("process_registry.json");
        let mut registry = ProcessRegistryFile::default();
        registry.push(ProcessEntry {
            run_id: "run".into(),
            repo_root: "repo".into(),
            agent_key: "run:agent-0".into(),
            pid: 1,
            start_identity: None,
            worktree_path: "worktree".into(),
            branch: String::new(),
        });
        registry.save(&path).unwrap();
        registry.save(&path).unwrap();
        fs::write(&path, "{broken").unwrap();

        let recovered = ProcessRegistryFile::load(&path).unwrap();
        assert_eq!(recovered.entries.len(), 1);
    }
}
