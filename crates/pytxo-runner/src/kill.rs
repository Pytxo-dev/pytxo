use std::path::Path;
use std::process::Command;
use std::thread;
use std::time::{Duration, Instant};

use pytxo_core::{PytxoError, Result};

const EXIT_CONFIRM_TIMEOUT: Duration = Duration::from_secs(10);

/// Physical identity of an existing Store file. A canonical path alone can
/// name a replacement database after a controller crash.
pub fn file_identity(path: &Path) -> Result<String> {
    #[cfg(windows)]
    {
        use std::fs::File;
        let file = File::open(path)?;
        windows_file_identity(&file)
    }

    #[cfg(unix)]
    {
        use std::os::unix::fs::MetadataExt;
        let metadata = std::fs::metadata(path)?;
        if metadata.ino() == 0 {
            return Err(PytxoError::Runner(
                "Store file identity is unavailable".into(),
            ));
        }
        Ok(format!(
            "unix-dev-inode:{}:{}",
            metadata.dev(),
            metadata.ino()
        ))
    }

    #[cfg(not(any(windows, unix)))]
    {
        let _ = path;
        Err(PytxoError::Runner(
            "Store file identity is unsupported on this platform".into(),
        ))
    }
}

#[cfg(windows)]
fn windows_file_identity(file: &std::fs::File) -> Result<String> {
    use std::os::windows::io::AsRawHandle;
    use windows_sys::Win32::Storage::FileSystem::{
        GetFileInformationByHandle, BY_HANDLE_FILE_INFORMATION,
    };

    let mut info = BY_HANDLE_FILE_INFORMATION::default();
    if unsafe { GetFileInformationByHandle(file.as_raw_handle(), &mut info) } == 0 {
        return Err(PytxoError::Runner(format!(
            "query Store file identity failed: {}",
            std::io::Error::last_os_error()
        )));
    }
    let index = (u64::from(info.nFileIndexHigh) << 32) | u64::from(info.nFileIndexLow);
    if index == 0 {
        return Err(PytxoError::Runner(
            "Store file identity is unavailable".into(),
        ));
    }
    Ok(format!(
        "windows-file-id:{}:{index}",
        info.dwVolumeSerialNumber
    ))
}

/// Pin the existing Store file against pathname replacement while a recovery
/// decision opens SQLite and checks ownership. Without this exclusion a
/// pathname can briefly point at a second database during the SQLite open.
/// On platforms without a proven replacement exclusion this fails closed.
pub struct FileIdentityGuard {
    #[cfg(windows)]
    _file: std::fs::File,
    identity: String,
}

impl FileIdentityGuard {
    pub fn acquire(path: &Path) -> Result<Self> {
        #[cfg(windows)]
        {
            use std::fs::OpenOptions;
            use std::os::windows::fs::OpenOptionsExt;
            use windows_sys::Win32::Storage::FileSystem::{FILE_SHARE_READ, FILE_SHARE_WRITE};

            // Excluding FILE_SHARE_DELETE prevents rename/unlink while SQLite
            // opens the path. Reads and writes remain available to SQLite.
            let file = OpenOptions::new()
                .read(true)
                .share_mode(FILE_SHARE_READ | FILE_SHARE_WRITE)
                .open(path)?;
            let identity = windows_file_identity(&file)?;
            Ok(Self {
                _file: file,
                identity,
            })
        }

        #[cfg(not(windows))]
        {
            let _ = path;
            Err(PytxoError::Runner(
                "automatic Store recovery cannot exclude pathname replacement on this platform"
                    .into(),
            ))
        }
    }

    pub fn identity(&self) -> &str {
        &self.identity
    }
}

/// Return an OS creation token for `pid`, or `None` when that process is absent.
/// The token is persisted with the PID so a crashed supervisor cannot later
/// signal an unrelated process that reused the same numeric PID.
pub fn process_start_identity(pid: u32) -> Result<Option<String>> {
    if pid == 0 {
        return Ok(None);
    }

    #[cfg(windows)]
    {
        use windows_sys::Win32::Foundation::{CloseHandle, FILETIME, STILL_ACTIVE};
        use windows_sys::Win32::System::Threading::{
            GetExitCodeProcess, GetProcessTimes, OpenProcess, PROCESS_QUERY_LIMITED_INFORMATION,
        };

        let handle = unsafe { OpenProcess(PROCESS_QUERY_LIMITED_INFORMATION, 0, pid) };
        if handle.is_null() {
            // OpenProcess also fails for access denied. Treat only Windows'
            // invalid-PID result as confirmed absence; uncertainty must not
            // authorize recovery or process-tree replacement.
            let error = std::io::Error::last_os_error();
            if windows_pid_is_confirmed_missing(&error) {
                return Ok(None);
            }
            return Err(PytxoError::Runner(format!(
                "query process identity failed for pid {pid}: {error}"
            )));
        }
        let mut exit_code = 0_u32;
        let exit_queried = unsafe { GetExitCodeProcess(handle, &mut exit_code) };
        if exit_queried == 0 {
            unsafe {
                CloseHandle(handle);
            }
            return Err(PytxoError::Runner(format!(
                "query process state failed for pid {pid}"
            )));
        }
        if exit_code != STILL_ACTIVE as u32 {
            unsafe {
                CloseHandle(handle);
            }
            return Ok(None);
        }
        let mut creation = FILETIME::default();
        let mut exit = FILETIME::default();
        let mut kernel = FILETIME::default();
        let mut user = FILETIME::default();
        let queried =
            unsafe { GetProcessTimes(handle, &mut creation, &mut exit, &mut kernel, &mut user) };
        unsafe {
            CloseHandle(handle);
        }
        if queried == 0 {
            return Err(PytxoError::Runner(format!(
                "query process identity failed for pid {pid}"
            )));
        }
        let token = ((creation.dwHighDateTime as u64) << 32) | creation.dwLowDateTime as u64;
        return Ok(Some(format!("windows-filetime:{token}")));
    }

    #[cfg(target_os = "linux")]
    {
        let path = format!("/proc/{pid}/stat");
        let raw = match std::fs::read_to_string(path) {
            Ok(raw) => raw,
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok(None),
            Err(error) => return Err(PytxoError::Io(error)),
        };
        let after_name = raw
            .rsplit_once(')')
            .map(|(_, rest)| rest.trim())
            .ok_or_else(|| {
                PytxoError::Runner(format!("invalid /proc stat record for pid {pid}"))
            })?;
        // `after_name` begins at field 3; process start time is field 22.
        let token = after_name.split_whitespace().nth(19).ok_or_else(|| {
            PytxoError::Runner(format!("missing process start time for pid {pid}"))
        })?;
        return Ok(Some(format!("linux-start-ticks:{token}")));
    }

    #[cfg(all(unix, not(target_os = "linux")))]
    {
        let output = Command::new("ps")
            .args(["-o", "lstart=", "-p", &pid.to_string()])
            .output()
            .map_err(|error| PytxoError::Runner(format!("query process identity: {error}")))?;
        if !output.status.success() {
            if unix_pid_is_confirmed_missing(pid)? {
                return Ok(None);
            }
            return Err(PytxoError::Runner(format!(
                "query process identity failed for pid {pid}: ps exited {}",
                output.status
            )));
        }
        let token = String::from_utf8_lossy(&output.stdout).trim().to_string();
        if token.is_empty() {
            if unix_pid_is_confirmed_missing(pid)? {
                return Ok(None);
            }
            return Err(PytxoError::Runner(format!(
                "query process identity returned no start time for pid {pid}"
            )));
        }
        return Ok(Some(format!("unix-lstart:{token}")));
    }

    #[allow(unreachable_code)]
    Ok(None)
}

#[cfg(all(unix, not(target_os = "linux")))]
fn unix_pid_is_confirmed_missing(pid: u32) -> Result<bool> {
    let Ok(native_pid) = libc::pid_t::try_from(pid) else {
        return Ok(true);
    };
    if native_pid <= 0 {
        return Ok(true);
    }
    // Signal zero only probes existence. Never cast a large PID into a negative
    // process-group selector, or infer absence from ps failure/access denial.
    if unsafe { libc::kill(native_pid, 0) } == 0 {
        return Ok(false);
    }
    let error = std::io::Error::last_os_error();
    if unix_pid_error_is_confirmed_missing(&error) {
        return Ok(true);
    }
    Err(PytxoError::Runner(format!(
        "query process existence failed for pid {pid}: {error}"
    )))
}

#[cfg(all(unix, not(target_os = "linux")))]
fn unix_pid_error_is_confirmed_missing(error: &std::io::Error) -> bool {
    error.raw_os_error() == Some(libc::ESRCH)
}

#[cfg(windows)]
fn windows_pid_is_confirmed_missing(error: &std::io::Error) -> bool {
    // ERROR_INVALID_PARAMETER: OpenProcess reports an absent PID. Access
    // denied (5) and other failures are uncertainty, not evidence of death.
    error.raw_os_error() == Some(87)
}

pub fn process_matches(pid: u32, expected_start_identity: &str) -> Result<bool> {
    Ok(process_start_identity(pid)?.as_deref() == Some(expected_start_identity))
}

/// Terminate the process tree identified by both PID and creation token, then
/// wait until the original process identity is no longer live.
pub fn kill_process_tree(pid: u32, expected_start_identity: &str) -> Result<()> {
    let Some(actual_start_identity) = process_start_identity(pid)? else {
        // Stop is idempotent: an already-exited process is a confirmed safe outcome.
        return Ok(());
    };
    if actual_start_identity != expected_start_identity {
        return Err(PytxoError::Runner(format!(
            "refusing to stop pid {pid}: the PID now belongs to a different process"
        )));
    }

    #[cfg(windows)]
    let status = Command::new("taskkill")
        .args(["/PID", &pid.to_string(), "/T", "/F"])
        .status()
        .map_err(|error| PytxoError::Runner(format!("taskkill tree: {error}")))?;

    #[cfg(unix)]
    let status = kill_unix_tree(pid, "-TERM")?;

    if !status.success() && process_matches(pid, expected_start_identity)? {
        return Err(PytxoError::Runner(format!(
            "process-tree termination failed for pid {pid}"
        )));
    }

    let deadline = Instant::now() + EXIT_CONFIRM_TIMEOUT;
    while process_matches(pid, expected_start_identity)? {
        if Instant::now() >= deadline {
            #[cfg(unix)]
            {
                let _ = kill_unix_tree(pid, "-KILL");
            }
            if process_matches(pid, expected_start_identity)? {
                return Err(PytxoError::Runner(format!(
                    "process-tree termination was not confirmed for pid {pid}"
                )));
            }
            break;
        }
        thread::sleep(Duration::from_millis(25));
    }
    Ok(())
}

#[cfg(unix)]
fn kill_unix_tree(pid: u32, signal: &str) -> Result<std::process::ExitStatus> {
    let descendants = unix_descendants(pid)?;
    for descendant in descendants.into_iter().rev() {
        let _ = Command::new("kill")
            .args([signal, &descendant.to_string()])
            .status();
    }
    Command::new("kill")
        .args([signal, &pid.to_string()])
        .status()
        .map_err(|error| PytxoError::Runner(format!("kill process tree: {error}")))
}

#[cfg(unix)]
fn unix_descendants(root_pid: u32) -> Result<Vec<u32>> {
    let output = Command::new("ps")
        .args(["-e", "-o", "pid=", "-o", "ppid="])
        .output()
        .map_err(|error| PytxoError::Runner(format!("list process tree: {error}")))?;
    if !output.status.success() {
        return Err(PytxoError::Runner("list process tree failed".into()));
    }
    let pairs = String::from_utf8_lossy(&output.stdout)
        .lines()
        .filter_map(|line| {
            let mut fields = line.split_whitespace();
            Some((fields.next()?.parse().ok()?, fields.next()?.parse().ok()?))
        })
        .collect::<Vec<(u32, u32)>>();
    let mut descendants = Vec::new();
    let mut parents = vec![root_pid];
    while let Some(parent) = parents.pop() {
        for &(pid, ppid) in &pairs {
            if ppid == parent && !descendants.contains(&pid) {
                descendants.push(pid);
                parents.push(pid);
            }
        }
    }
    Ok(descendants)
}

pub fn kill_pid(pid: u32) -> Result<()> {
    match process_start_identity(pid)? {
        Some(identity) => kill_process_tree(pid, &identity),
        None => Ok(()),
    }
}

#[cfg(all(test, windows))]
mod windows_identity_tests {
    use super::windows_pid_is_confirmed_missing;

    #[test]
    fn access_denied_is_not_absence() {
        assert!(windows_pid_is_confirmed_missing(
            &std::io::Error::from_raw_os_error(87)
        ));
        assert!(!windows_pid_is_confirmed_missing(
            &std::io::Error::from_raw_os_error(5)
        ));
    }
}

#[cfg(all(test, unix, not(target_os = "linux")))]
mod unix_identity_tests {
    use super::{process_start_identity, unix_pid_error_is_confirmed_missing};

    #[test]
    fn access_denied_is_not_absence() {
        assert!(unix_pid_error_is_confirmed_missing(
            &std::io::Error::from_raw_os_error(libc::ESRCH)
        ));
        for code in [libc::EPERM, libc::EACCES, libc::EINVAL] {
            assert!(!unix_pid_error_is_confirmed_missing(
                &std::io::Error::from_raw_os_error(code)
            ));
        }
    }

    #[test]
    fn current_process_has_identity_and_out_of_range_pid_is_absent() {
        assert!(process_start_identity(std::process::id())
            .unwrap()
            .is_some());
        assert_eq!(process_start_identity(u32::MAX).unwrap(), None);
    }

    #[test]
    fn reaped_process_is_confirmed_absent() {
        let mut child = std::process::Command::new("sh")
            .args(["-c", "exit 0"])
            .spawn()
            .unwrap();
        let pid = child.id();
        assert!(child.wait().unwrap().success());
        assert_eq!(process_start_identity(pid).unwrap(), None);
    }
}

#[cfg(test)]
mod file_identity_tests {
    use super::{file_identity, FileIdentityGuard};

    #[test]
    fn rename_preserves_identity_and_replacement_changes_it() {
        let dir = tempfile::tempdir().unwrap();
        let original = dir.path().join("store.db");
        let moved = dir.path().join("old-store.db");
        std::fs::write(&original, b"original").unwrap();
        let first = file_identity(&original).unwrap();
        std::fs::rename(&original, &moved).unwrap();
        assert_eq!(file_identity(&moved).unwrap(), first);
        std::fs::write(&original, b"replacement").unwrap();
        assert_ne!(file_identity(&original).unwrap(), first);
    }

    #[cfg(windows)]
    #[test]
    fn recovery_guard_pins_the_file_while_sqlite_opens_its_path() {
        let dir = tempfile::tempdir().unwrap();
        let original = dir.path().join("store.db");
        let moved = dir.path().join("moved.db");
        std::fs::write(&original, b"original").unwrap();
        let guard = FileIdentityGuard::acquire(&original).unwrap();
        assert_eq!(guard.identity(), file_identity(&original).unwrap());
        assert!(std::fs::rename(&original, &moved).is_err());
        drop(guard);
        std::fs::rename(&original, &moved).unwrap();
    }

    #[cfg(not(windows))]
    #[test]
    fn recovery_guard_fails_closed_without_replacement_exclusion() {
        let dir = tempfile::tempdir().unwrap();
        let original = dir.path().join("store.db");
        std::fs::write(&original, b"original").unwrap();
        assert!(FileIdentityGuard::acquire(&original).is_err());
    }
}
