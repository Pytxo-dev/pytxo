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

/// Return an available OS creation token for `pid`.
/// The token is persisted with the PID so a crashed supervisor cannot later
/// signal an unrelated process that reused the same numeric PID.
/// Unix retains the token until reaping, including for fast-exiting commands.
/// Use `process_matches` when deciding whether that identity is still live.
pub fn process_start_identity(pid: u32) -> Result<Option<String>> {
    Ok(query_process_identity(pid)?.map(|identity| identity.start_token))
}

pub(crate) fn live_process_start_identity(pid: u32) -> Result<Option<String>> {
    Ok(query_process_identity(pid)?
        .filter(|identity| !identity.terminated)
        .map(|identity| identity.start_token))
}

struct ProcessIdentity {
    start_token: String,
    terminated: bool,
}

fn query_process_identity(pid: u32) -> Result<Option<ProcessIdentity>> {
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
        return Ok(Some(ProcessIdentity {
            start_token: format!("windows-filetime:{token}"),
            terminated: false,
        }));
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
        return Ok(Some(ProcessIdentity {
            start_token: format!("linux-start-ticks:{token}"),
            terminated: linux_leader_is_terminated(after_name),
        }));
    }

    #[cfg(all(unix, not(target_os = "linux")))]
    {
        let output = Command::new("ps")
            .args(["-o", "stat=", "-o", "lstart=", "-p", &pid.to_string()])
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
        let raw = String::from_utf8_lossy(&output.stdout);
        if raw.trim().is_empty() {
            if unix_pid_is_confirmed_missing(pid)? {
                return Ok(None);
            }
            return Err(PytxoError::Runner(format!(
                "query process identity returned no start time for pid {pid}"
            )));
        }
        return unix_ps_identity(&raw, pid).map(Some);
    }

    #[allow(unreachable_code)]
    Ok(None)
}

#[cfg(any(all(unix, not(target_os = "linux")), test))]
fn unix_state_is_terminated(state: &str) -> bool {
    // A zombie cannot execute, even while its PID remains until the parent
    // reaps it. Sleeping, suspended and uninterruptible tasks are still live.
    state.starts_with('Z')
}

#[cfg(any(all(unix, not(target_os = "linux")), test))]
fn unix_ps_identity(raw: &str, pid: u32) -> Result<ProcessIdentity> {
    if raw.trim().lines().count() != 1 {
        return Err(PytxoError::Runner(format!(
            "invalid ps identity record for pid {pid}"
        )));
    }
    let mut fields = raw.trim().splitn(2, char::is_whitespace);
    let state = fields.next().unwrap_or_default();
    if !matches!(
        state.as_bytes().first(),
        Some(b'I' | b'R' | b'S' | b'T' | b'U' | b'D' | b'W' | b'Z' | b'H' | b'?')
    ) {
        return Err(PytxoError::Runner(format!(
            "unknown ps process state {state:?} for pid {pid}"
        )));
    }
    let token = fields.next().unwrap_or_default().trim();
    if token.is_empty() || token == "-" {
        return Err(PytxoError::Runner(format!(
            "missing ps start time for pid {pid}"
        )));
    }
    // Darwin can report H (halted) or ? (task inspection unavailable) during
    // exit. Neither proves death; retain the separate BSD creation timestamp.
    Ok(ProcessIdentity {
        start_token: format!("unix-lstart:{token}"),
        terminated: unix_state_is_terminated(state),
    })
}

#[cfg(any(target_os = "linux", test))]
fn linux_leader_is_terminated(stat_after_name: &str) -> bool {
    let mut fields = stat_after_name.split_whitespace();
    if fields.next() != Some("Z") {
        return false;
    }
    // A zombie thread-group leader may still have live sibling threads.
    // Field 20 (num_threads) must prove this is the only remaining thread.
    fields.nth(16) == Some("1")
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
    Ok(live_process_start_identity(pid)?.as_deref() == Some(expected_start_identity))
}

/// Terminate the process tree identified by both PID and creation token, then
/// wait until the original process identity is no longer live.
pub fn kill_process_tree(pid: u32, expected_start_identity: &str) -> Result<()> {
    let Some(actual_start_identity) = live_process_start_identity(pid)? else {
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
    match live_process_start_identity(pid)? {
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
mod process_state_tests {
    #[cfg(unix)]
    struct ChildGuard(std::process::Child);

    #[cfg(unix)]
    impl Drop for ChildGuard {
        fn drop(&mut self) {
            let _ = self.0.kill();
            let _ = self.0.wait();
        }
    }

    #[test]
    fn ps_state_parser_separates_creation_identity_from_liveness() {
        let token = "Sat Oct  3 07:00:00 2026";
        for state in ["S", "R+", "T", "I", "U", "H", "?", "?E", "Z", "Z+"] {
            let identity = super::unix_ps_identity(&format!("  {state}  {token}\n"), 123).unwrap();
            assert_eq!(identity.start_token, format!("unix-lstart:{token}"));
            assert_eq!(identity.terminated, state.starts_with('Z'));
        }
        for raw in ["", "Q timestamp", "S", "S time\nS other", "Z -", "? -"] {
            assert!(super::unix_ps_identity(raw, 123).is_err());
        }
    }

    #[test]
    fn linux_zombie_leader_with_other_threads_remains_live() {
        let stat = |state: &str, threads: &str| {
            let mut fields = vec!["0"; 20];
            fields[0] = state;
            fields[17] = threads;
            fields.join(" ")
        };
        assert!(super::linux_leader_is_terminated(&stat("Z", "1")));
        for (state, threads) in [("Z", "2"), ("Z", "0"), ("Z", "?"), ("T", "1"), ("S", "1")] {
            assert!(!super::linux_leader_is_terminated(&stat(state, threads)));
        }
        assert!(!super::linux_leader_is_terminated("Z"));
    }

    #[test]
    fn only_zombie_state_is_confirmed_terminated() {
        for state in ["Z", "Z+", "Zs"] {
            assert!(super::unix_state_is_terminated(state));
        }
        for state in ["R", "S", "D", "T", "t", "I", "W", "U", "H", "", "?", "?E"] {
            assert!(!super::unix_state_is_terminated(state));
        }
    }

    #[cfg(unix)]
    #[test]
    fn exited_unreaped_child_keeps_creation_token_but_is_not_live() {
        let mut child = ChildGuard(
            std::process::Command::new("sh")
                .args(["-c", "exit 0"])
                .spawn()
                .unwrap(),
        );
        let pid = child.0.id();
        let identity = super::process_start_identity(pid)
            .unwrap()
            .expect("unreaped child retains its creation token");
        let deadline = std::time::Instant::now() + std::time::Duration::from_secs(5);
        let observed = loop {
            let observed = super::process_matches(pid, &identity);
            if !matches!(observed, Ok(true)) || std::time::Instant::now() >= deadline {
                break observed;
            }
            std::thread::sleep(std::time::Duration::from_millis(10));
        };
        assert_eq!(super::process_start_identity(pid).unwrap(), Some(identity));
        assert!(
            !observed.unwrap(),
            "exited child must not be live before reaping"
        );
        let status = child.0.wait().unwrap();
        assert!(status.success());
        assert_eq!(super::process_start_identity(pid).unwrap(), None);
    }

    #[cfg(unix)]
    #[test]
    fn stopped_child_keeps_its_live_identity() {
        let child = ChildGuard(
            std::process::Command::new("sleep")
                .arg("30")
                .spawn()
                .unwrap(),
        );
        let pid = child.0.id();
        let identity = super::process_start_identity(pid)
            .unwrap()
            .expect("live child");
        assert!(std::process::Command::new("kill")
            .args(["-STOP", &pid.to_string()])
            .status()
            .unwrap()
            .success());
        let deadline = std::time::Instant::now() + std::time::Duration::from_secs(5);
        loop {
            let status = std::process::Command::new("ps")
                .args(["-o", "stat=", "-p", &pid.to_string()])
                .output()
                .unwrap();
            assert!(status.status.success());
            if String::from_utf8_lossy(&status.stdout)
                .trim()
                .starts_with('T')
            {
                break;
            }
            assert!(
                std::time::Instant::now() < deadline,
                "child did not suspend"
            );
            std::thread::sleep(std::time::Duration::from_millis(10));
        }
        assert_eq!(
            super::process_start_identity(pid).unwrap(),
            Some(identity.clone())
        );
        assert!(super::process_matches(pid, &identity).unwrap());
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
