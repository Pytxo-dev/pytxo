use std::process::Command;
use std::thread;
use std::time::{Duration, Instant};

use pytxo_core::{PytxoError, Result};

const EXIT_CONFIRM_TIMEOUT: Duration = Duration::from_secs(10);

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
            return Ok(None);
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
        let token = String::from_utf8_lossy(&output.stdout).trim().to_string();
        return Ok((!token.is_empty()).then(|| format!("unix-lstart:{token}")));
    }

    #[allow(unreachable_code)]
    Ok(None)
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
