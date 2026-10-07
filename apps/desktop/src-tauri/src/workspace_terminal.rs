//! User-owned interactive shells. These are outside agent permission profiles and
//! the reviewed Apply boundary. A session is bound to its creating main window and
//! canonical workspace; no agent id, PID, command or alternate cwd is accepted.
use crate::ipc_error::{IpcResult, PytxoIpcError};
use base64::Engine;
use portable_pty::{native_pty_system, ChildKiller, CommandBuilder, MasterPty, PtySize};
use serde::Serialize;
use std::{
    collections::{HashMap, VecDeque},
    io::{Read, Write},
    sync::{Arc, Mutex},
};
use tauri::{State, Window};

const REPLAY_BYTES: usize = 1024 * 1024;
const MAX_SESSIONS: usize = 8;
fn err(e: impl ToString) -> PytxoIpcError {
    PytxoIpcError::new("workspace_terminal", e.to_string())
}

#[derive(Default)]
struct Output {
    bytes: VecDeque<u8>,
    start: u64,
    end: u64,
    exit: Option<u32>,
    error: Option<String>,
}
impl Output {
    fn append(&mut self, bytes: &[u8]) {
        self.bytes.extend(bytes);
        self.end += bytes.len() as u64;
        let discard = self.bytes.len().saturating_sub(REPLAY_BYTES);
        self.bytes.drain(..discard);
        self.start += discard as u64;
    }
}
struct Session {
    id: String,
    owner: String,
    domain: String,
    output: Arc<Mutex<Output>>,
    master: Option<Box<dyn MasterPty + Send>>,
    writer: Option<Box<dyn Write + Send>>,
    killer: Box<dyn ChildKiller + Send + Sync>,
    ending: bool,
}
#[derive(Default)]
pub struct WorkspaceTerminals(Mutex<HashMap<String, Session>>);
#[derive(Serialize, Clone)]
pub struct TerminalInfo {
    id: String,
    domain_id: String,
    cwd: String,
    owner: String,
    state: String,
    exit_code: Option<u32>,
}
#[derive(Serialize)]
pub struct TerminalPage {
    session: TerminalInfo,
    start: u64,
    next: u64,
    gap: bool,
    data_base64: String,
    error: Option<String>,
}
impl Session {
    fn info(&self, out: &Output) -> TerminalInfo {
        TerminalInfo {
            id: self.id.clone(),
            domain_id: self.domain.clone(),
            cwd: self.domain.clone(),
            owner: "You".into(),
            state: if out.exit.is_some() {
                "ended"
            } else if self.ending {
                "ending"
            } else {
                "running"
            }
            .into(),
            exit_code: out.exit,
        }
    }
    fn end(&mut self) -> IpcResult<()> {
        let kill_error = if self.output.lock().map_err(err)?.exit.is_none() {
            self.killer.kill().err()
        } else {
            None
        };
        self.ending = true;
        self.writer.take();
        self.master.take();
        // portable-pty 0.8.1's Windows cloned killer reverses the Win32 return
        // test. Neither its Ok nor Err proves completion: require the original
        // child's wait result instead. Never retarget a PID to hide that error.
        let deadline = std::time::Instant::now() + std::time::Duration::from_secs(2);
        while self.output.lock().map_err(err)?.exit.is_none() {
            if std::time::Instant::now() >= deadline {
                return Err(err(format!(
                    "Shell exit is not yet confirmed: {}",
                    kill_error
                        .map(|e| e.to_string())
                        .unwrap_or_else(|| "waiting for child process".into())
                )));
            }
            std::thread::sleep(std::time::Duration::from_millis(10));
        }
        Ok(())
    }
}
impl Drop for Session {
    fn drop(&mut self) {
        let _ = self.end();
    }
}
impl WorkspaceTerminals {
    pub fn live_count(&self) -> usize {
        self.0
            .lock()
            .map(|sessions| {
                sessions
                    .values()
                    .filter(|s| s.output.lock().map(|o| o.exit.is_none()).unwrap_or(true))
                    .count()
            })
            .unwrap_or(1)
    }
    pub fn end_all(&self) -> IpcResult<()> {
        let mut sessions = self.0.lock().map_err(err)?;
        for s in sessions.values_mut() {
            s.end()?;
        }
        Ok(())
    }
    fn create(&self, domain: String, owner: String) -> IpcResult<TerminalInfo> {
        let mut sessions = self.0.lock().map_err(err)?;
        // Keep ended references until app exit; a stale reference never respawns.
        if sessions
            .values()
            .filter(|s| s.output.lock().map(|o| o.exit.is_none()).unwrap_or(true))
            .count()
            >= MAX_SESSIONS
        {
            return Err(err(
                "Eight terminals are live. End an unused session before creating another.",
            ));
        }
        if sessions.len() >= 64 {
            return Err(err("Terminal history reached its 64-session limit. Restart Desktop after ending live sessions to clear it."));
        }
        let pair = native_pty_system()
            .openpty(PtySize {
                rows: 24,
                cols: 80,
                pixel_width: 0,
                pixel_height: 0,
            })
            .map_err(err)?;
        #[cfg(windows)]
        let mut command = {
            let root = std::env::var_os("SystemRoot")
                .ok_or_else(|| err("Windows system directory unavailable"))?;
            let mut c = CommandBuilder::new(
                std::path::PathBuf::from(root)
                    .join("System32/WindowsPowerShell/v1.0/powershell.exe"),
            );
            c.args(["-NoLogo", "-NoProfile"]);
            c
        };
        #[cfg(not(windows))]
        let mut command = CommandBuilder::new("/bin/sh");
        command.cwd(&domain);
        // Do not pass Pytxo's in-memory hydrated service credentials into a shell.
        for key in [
            "PYTXO_ULTRA_SESSION",
            "PYTXO_ACCESS_TOKEN",
            "PYTXO_REFRESH_TOKEN",
            "PYTXO_AUTH_TOKEN",
        ] {
            command.env_remove(key);
        }
        let mut reader = pair.master.try_clone_reader().map_err(err)?;
        let writer = pair.master.take_writer().map_err(err)?;
        let mut child = pair.slave.spawn_command(command).map_err(err)?;
        drop(pair.slave);
        let killer = child.clone_killer();
        let output = Arc::new(Mutex::new(Output::default()));
        let read_output = output.clone();
        std::thread::spawn(move || {
            let mut bytes = [0u8; 8192];
            loop {
                match reader.read(&mut bytes) {
                    Ok(0) => break,
                    Ok(n) => {
                        if let Ok(mut out) = read_output.lock() {
                            out.append(&bytes[..n]);
                        } else {
                            break;
                        }
                    }
                    Err(e) => {
                        if let Ok(mut out) = read_output.lock() {
                            out.error = Some(e.to_string());
                        }
                        break;
                    }
                }
            }
        });
        let exit_output = output.clone();
        std::thread::spawn(move || {
            let result = child.wait();
            if let Ok(mut out) = exit_output.lock() {
                match result {
                    Ok(status) => out.exit = Some(status.exit_code()),
                    Err(e) => {
                        out.error = Some(e.to_string());
                    }
                }
            }
        });
        let id = uuid::Uuid::new_v4().to_string();
        let session = Session {
            id: id.clone(),
            owner,
            domain,
            output,
            master: Some(pair.master),
            writer: Some(writer),
            killer,
            ending: false,
        };
        let info = session.info(&*session.output.lock().map_err(err)?);
        sessions.insert(id, session);
        Ok(info)
    }
    fn with_session<T>(
        &self,
        id: &str,
        domain: &str,
        owner: &str,
        f: impl FnOnce(&mut Session) -> IpcResult<T>,
    ) -> IpcResult<T> {
        let mut sessions = self.0.lock().map_err(err)?;
        let session = sessions.get_mut(id).ok_or_else(|| {
            err("Session unavailable after app exit. Create a new terminal explicitly.")
        })?;
        if session.domain != domain || session.owner != owner {
            return Err(err("Terminal ownership or workspace does not match"));
        }
        f(session)
    }
    fn read(&self, id: &str, domain: &str, owner: &str, after: u64) -> IpcResult<TerminalPage> {
        self.with_session(id, domain, owner, |s| {
            let out = s.output.lock().map_err(err)?;
            if after > out.end {
                return Err(err("Output cursor is ahead of this session"));
            }
            let start = after.max(out.start);
            let bytes: Vec<u8> = out
                .bytes
                .iter()
                .skip((start - out.start) as usize)
                .take(65536)
                .copied()
                .collect();
            Ok(TerminalPage {
                session: s.info(&out),
                start,
                next: start + bytes.len() as u64,
                gap: after < out.start,
                data_base64: base64::engine::general_purpose::STANDARD.encode(bytes),
                error: out.error.clone(),
            })
        })
    }
}

#[tauri::command]
pub async fn workspace_terminal_create(
    window: Window,
    state: State<'_, WorkspaceTerminals>,
    domain_id: String,
) -> IpcResult<TerminalInfo> {
    let _upgrade_guard = pytxo_core::UpgradeGuard::work().map_err(err)?;
    if window.label() != "main" {
        return Err(err("Workspace terminals belong to the main window"));
    }
    let canonical = std::fs::canonicalize(&domain_id).map_err(err)?;
    if !canonical.is_dir() {
        return Err(err("Workspace folder is unavailable"));
    }
    let catalog = pytxo_orchestrate::list_catalog_domains().map_err(err)?;
    if !catalog
        .iter()
        .any(|entry| std::fs::canonicalize(&entry.repo_root).ok().as_ref() == Some(&canonical))
    {
        return Err(err(
            "Select a registered workspace before opening a terminal",
        ));
    }
    state.create(
        canonical.to_string_lossy().into_owned(),
        window.label().into(),
    )
}
#[tauri::command]
pub fn workspace_terminal_list(
    window: Window,
    state: State<'_, WorkspaceTerminals>,
) -> IpcResult<Vec<TerminalInfo>> {
    state
        .0
        .lock()
        .map_err(err)?
        .values()
        .filter(|s| s.owner == window.label())
        .map(|s| Ok(s.info(&*s.output.lock().map_err(err)?)))
        .collect()
}
#[tauri::command]
pub fn workspace_terminal_read(
    window: Window,
    state: State<'_, WorkspaceTerminals>,
    session_id: String,
    domain_id: String,
    after: u64,
) -> IpcResult<TerminalPage> {
    state.read(&session_id, &domain_id, window.label(), after)
}
#[tauri::command]
pub fn workspace_terminal_input(
    window: Window,
    state: State<'_, WorkspaceTerminals>,
    session_id: String,
    domain_id: String,
    data: String,
) -> IpcResult<()> {
    if data.len() > 65536 {
        return Err(err("Paste exceeds 64 KiB; split it into smaller inputs"));
    }
    state.with_session(&session_id, &domain_id, window.label(), |s| {
        if s.ending || s.output.lock().map_err(err)?.exit.is_some() {
            return Err(err("Terminal has ended"));
        }
        let writer = s
            .writer
            .as_mut()
            .ok_or_else(|| err("Terminal input is unavailable"))?;
        writer.write_all(data.as_bytes()).map_err(err)?;
        writer.flush().map_err(err)
    })
}
#[tauri::command]
pub fn workspace_terminal_resize(
    window: Window,
    state: State<'_, WorkspaceTerminals>,
    session_id: String,
    domain_id: String,
    rows: u16,
    cols: u16,
) -> IpcResult<()> {
    if !(2..=300).contains(&rows) || !(10..=500).contains(&cols) {
        return Err(err("Terminal dimensions are outside supported bounds"));
    }
    state.with_session(&session_id, &domain_id, window.label(), |s| {
        s.master
            .as_ref()
            .ok_or_else(|| err("Terminal has ended"))?
            .resize(PtySize {
                rows,
                cols,
                pixel_width: 0,
                pixel_height: 0,
            })
            .map_err(err)
    })
}
#[tauri::command]
pub async fn workspace_terminal_end(
    window: Window,
    state: State<'_, WorkspaceTerminals>,
    session_id: String,
    domain_id: String,
) -> IpcResult<()> {
    state.with_session(&session_id, &domain_id, window.label(), |s| s.end())
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn bounded_replay_reports_loss_without_changing_raw_bytes() {
        let mut out = Output::default();
        out.append(&vec![b'x'; REPLAY_BYTES + 20]);
        out.append(&[0, 255, 27]);
        assert_eq!(out.bytes.len(), REPLAY_BYTES);
        assert_eq!(out.start, 23);
        assert_eq!(out.end, REPLAY_BYTES as u64 + 23);
        assert_eq!(
            out.bytes.iter().rev().take(3).copied().collect::<Vec<_>>(),
            [27, 255, 0]
        );
    }
    #[test]
    fn native_user_shell_replay_resize_and_ownership() {
        let dir = tempfile::tempdir().unwrap();
        let registry = WorkspaceTerminals::default();
        let cwd = dir
            .path()
            .canonicalize()
            .unwrap()
            .to_string_lossy()
            .into_owned();
        let info = registry.create(cwd.clone(), "main".into()).unwrap();
        assert!(registry
            .read(&info.id, "other-workspace", "main", 0)
            .is_err());
        assert!(registry.read(&info.id, &cwd, "other-window", 0).is_err());
        registry
            .with_session(&info.id, &cwd, "main", |s| {
                s.master
                    .as_ref()
                    .unwrap()
                    .resize(PtySize {
                        rows: 31,
                        cols: 97,
                        pixel_width: 0,
                        pixel_height: 0,
                    })
                    .map_err(err)?;
                let size = s.master.as_ref().unwrap().get_size().map_err(err)?;
                assert_eq!((size.rows, size.cols), (31, 97));
                let writer = s.writer.as_mut().unwrap();
                writer
                    .write_all(b"echo pytxo-shell-proof > shell-proof.txt\r\n")
                    .map_err(err)?;
                writer.flush().map_err(err)
            })
            .unwrap();
        let deadline = std::time::Instant::now() + std::time::Duration::from_secs(15);
        loop {
            let page = registry.read(&info.id, &cwd, "main", 0).unwrap();
            let bytes = base64::engine::general_purpose::STANDARD
                .decode(&page.data_base64)
                .unwrap();
            if dir.path().join("shell-proof.txt").exists() && !bytes.is_empty() {
                break;
            }
            assert!(
                std::time::Instant::now() < deadline,
                "shell did not return output"
            );
            std::thread::sleep(std::time::Duration::from_millis(100));
        }
        let a = registry.read(&info.id, &cwd, "main", 0).unwrap();
        let b = registry.read(&info.id, &cwd, "main", 0).unwrap();
        assert!(b.next >= a.next);
        registry
            .with_session(&info.id, &cwd, "main", |s| s.end())
            .unwrap();
        assert!(registry
            .with_session(&info.id, &cwd, "main", |s| {
                assert!(s.writer.is_none());
                Ok(())
            })
            .is_ok());
    }
}
