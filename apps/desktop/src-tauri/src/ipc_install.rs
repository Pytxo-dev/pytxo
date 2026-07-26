//! Install Pytxo CLI from public GitHub releases (Pytxo Desktop setup wizard).
//!
//! Windows: no PowerShell / visible consoles — ureq download + winreg PATH + CREATE_NO_WINDOW.

use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::{LazyLock, Mutex};

use serde::Serialize;

use crate::ipc_error::{map_io_err, IpcResult, PytxoIpcError};

const RELEASES_REPO: &str = "Pytxo-dev/pytxo-releases";
const DESKTOP_VERSION: &str = env!("CARGO_PKG_VERSION");

static INSTALL_STATE: LazyLock<Mutex<InstallState>> = LazyLock::new(|| {
    Mutex::new(InstallState {
        phase: String::from("idle"),
        message: String::new(),
        installed_path: None,
        path_pending: false,
    })
});

struct InstallState {
    phase: String,
    message: String,
    installed_path: Option<PathBuf>,
    path_pending: bool,
}

#[derive(Serialize)]
pub struct InstallCliStatusDto {
    pub phase: String,
    pub message: String,
    pub cli_present: bool,
    /// True when binary exists but may not be on PATH yet (restart recommended).
    pub path_pending: bool,
}

fn set_state(phase: &str, message: impl Into<String>) {
    if let Ok(mut guard) = INSTALL_STATE.lock() {
        guard.phase = phase.into();
        guard.message = message.into();
    }
}

fn set_installed_path(path: PathBuf, path_pending: bool) {
    if let Ok(mut guard) = INSTALL_STATE.lock() {
        guard.installed_path = Some(path);
        guard.path_pending = path_pending;
    }
}

/// Hide console windows when spawning helpers from the Tauri GUI process.
#[cfg(windows)]
fn hide_window(cmd: &mut Command) {
    use std::os::windows::process::CommandExt;
    const CREATE_NO_WINDOW: u32 = 0x0800_0000;
    cmd.creation_flags(CREATE_NO_WINDOW);
}

#[cfg(not(windows))]
fn hide_window(_cmd: &mut Command) {}

/// Default install destination for this OS/arch (may not be on PATH yet).
pub fn default_cli_binary_path() -> Option<PathBuf> {
    install_paths().ok().map(|(_, dest, _)| dest)
}

/// Whether the CLI binary is usable (PATH, sidecar env, or known install dir).
pub fn cli_binary_usable() -> bool {
    if let Ok(sidecar) = std::env::var("PYTXO_SIDECAR") {
        if !sidecar.is_empty() && Path::new(&sidecar).is_file() {
            return true;
        }
    }
    if let Some(dest) = default_cli_binary_path() {
        if dest.is_file() {
            return true;
        }
    }
    binary_on_path("pytxo")
}

/// PATH probe without spawning a visible console.
fn binary_on_path(name: &str) -> bool {
    let path = std::env::var_os("PATH").unwrap_or_default();
    for dir in std::env::split_paths(&path) {
        #[cfg(windows)]
        {
            let exe = dir.join(format!("{name}.exe"));
            if exe.is_file() {
                return true;
            }
            let bare = dir.join(name);
            if bare.is_file() {
                return true;
            }
        }
        #[cfg(not(windows))]
        {
            let candidate = dir.join(name);
            if candidate.is_file() {
                return true;
            }
        }
    }
    false
}

/// PATH probe only (excludes sidecar / install dir).
fn cli_on_path() -> bool {
    binary_on_path("pytxo")
}

#[tauri::command]
pub fn install_pytxo_cli_status() -> InstallCliStatusDto {
    let (phase, message, path_pending_cached, installed) = INSTALL_STATE
        .lock()
        .map(|g| {
            (
                g.phase.to_string(),
                g.message.to_string(),
                g.path_pending,
                g.installed_path.clone(),
            )
        })
        .unwrap_or_else(|_| ("idle".into(), String::new(), false, None));

    // During active install, trust in-memory state — do not re-walk PATH every poll.
    if phase == "downloading" || phase == "path" || phase == "verifying" {
        return InstallCliStatusDto {
            phase,
            message,
            cli_present: false,
            path_pending: false,
        };
    }

    let present = installed
        .as_ref()
        .map(|p| p.is_file())
        .unwrap_or_else(cli_binary_usable);
    let path_pending = if phase == "done" {
        path_pending_cached
    } else {
        present && !cli_on_path()
    };
    InstallCliStatusDto {
        phase,
        message,
        cli_present: present,
        path_pending,
    }
}

#[tauri::command]
pub fn install_pytxo_cli() -> IpcResult<InstallCliStatusDto> {
    set_state("downloading", "Fetching Pytxo CLI from GitHub Releases…");
    match install_pytxo_cli_inner() {
        Ok(dest) => {
            // Session-local so Desktop can invoke CLI before PATH refresh / restart.
            std::env::set_var("PYTXO_SIDECAR", &dest);
            let on_path = cli_on_path() || binary_dir_on_session_path(dest.parent());
            set_installed_path(dest, !on_path);
            if on_path {
                set_state("done", "Pytxo CLI installed and available.");
            } else {
                set_state(
                    "done",
                    "Pytxo CLI installed. Restart Desktop later so PATH picks it up; this session uses the local binary.",
                );
            }
        }
        Err(e) => set_state("error", &e.message),
    }
    Ok(install_pytxo_cli_status())
}

fn binary_dir_on_session_path(dir: Option<&Path>) -> bool {
    let Some(dir) = dir else {
        return false;
    };
    let dir_s = dir.to_string_lossy();
    std::env::var_os("PATH")
        .map(|p| {
            std::env::split_paths(&p).any(|entry| entry == dir || entry.to_string_lossy() == dir_s)
        })
        .unwrap_or(false)
}

fn install_pytxo_cli_inner() -> IpcResult<PathBuf> {
    let (install_dir, dest, asset) = install_paths()?;
    std::fs::create_dir_all(&install_dir).map_err(map_io_err)?;
    let tag = format!("v{DESKTOP_VERSION}");
    let url = format!("https://github.com/{RELEASES_REPO}/releases/download/{tag}/{asset}");
    set_state("downloading", format!("Downloading {asset}…"));
    download_file(&url, &dest)?;
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        std::fs::set_permissions(&dest, std::fs::Permissions::from_mode(0o755))
            .map_err(map_io_err)?;
    }
    if !dest.is_file() {
        return Err(PytxoIpcError::new(
            "install",
            "download finished but CLI binary was not found",
        ));
    }
    set_state("path", "Updating user PATH…");
    append_user_path(&install_dir)?;
    // Prepend for this process so doctor can find siblings if needed.
    if let Ok(mut path) = std::env::var("PATH") {
        let dir = install_dir.to_string_lossy();
        if !path.split(';').any(|p| p == dir) && !path.split(':').any(|p| p == dir) {
            #[cfg(windows)]
            {
                path = format!("{dir};{path}");
            }
            #[cfg(not(windows))]
            {
                path = format!("{dir}:{path}");
            }
            std::env::set_var("PATH", path);
        }
    }
    set_state("verifying", "Verifying CLI…");
    let mut doctor = Command::new(&dest);
    doctor.args(["doctor", "--quick"]);
    hide_window(&mut doctor);
    let doctor = doctor.output();
    if let Ok(out) = doctor {
        if !out.status.success() {
            let stderr = String::from_utf8_lossy(&out.stderr);
            if !stderr.is_empty() {
                set_state("verifying", format!("Installed (doctor notes: {stderr})"));
            }
        }
    }
    Ok(dest)
}

fn install_paths() -> IpcResult<(PathBuf, PathBuf, &'static str)> {
    if cfg!(target_os = "windows") {
        let local = std::env::var("LOCALAPPDATA").map_err(map_io_err)?;
        let dir = PathBuf::from(local).join("Programs").join("pytxo");
        let asset = if cfg!(target_arch = "aarch64") {
            "pytxo-windows-arm64.exe"
        } else {
            "pytxo-windows-x64.exe"
        };
        Ok((dir.clone(), dir.join("pytxo.exe"), asset))
    } else if cfg!(target_os = "macos") {
        let home = std::env::var("HOME").map_err(map_io_err)?;
        let dir = PathBuf::from(home).join(".local").join("bin");
        let asset = if cfg!(target_arch = "aarch64") {
            "pytxo-darwin-arm64"
        } else {
            "pytxo-darwin-x64"
        };
        Ok((dir.clone(), dir.join("pytxo"), asset))
    } else {
        let home = std::env::var("HOME").map_err(map_io_err)?;
        let dir = PathBuf::from(home).join(".local").join("bin");
        let asset = if cfg!(target_arch = "aarch64") {
            "pytxo-linux-arm64"
        } else {
            "pytxo-linux-x64"
        };
        Ok((dir.clone(), dir.join("pytxo"), asset))
    }
}

fn download_file(url: &str, dest: &Path) -> IpcResult<()> {
    let resp = ureq::get(url)
        .set("User-Agent", "pytxo-desktop-installer")
        .call()
        .map_err(|e| PytxoIpcError::new("install", format!("download failed: {e}")))?;
    if !(200..300).contains(&resp.status()) {
        return Err(PytxoIpcError::new(
            "install",
            format!("download HTTP {}", resp.status()),
        ));
    }
    let mut reader = resp.into_reader();
    let mut file = std::fs::File::create(dest).map_err(map_io_err)?;
    std::io::copy(&mut reader, &mut file).map_err(map_io_err)?;
    Ok(())
}

fn append_user_path(install_dir: &Path) -> IpcResult<()> {
    #[cfg(windows)]
    {
        append_user_path_windows(install_dir)
    }
    #[cfg(not(windows))]
    {
        let dir = install_dir.to_string_lossy();
        let home = std::env::var("HOME").map_err(map_io_err)?;
        for rc in [".bashrc", ".zshrc", ".profile"] {
            let path = PathBuf::from(&home).join(rc);
            if path.exists() {
                let export = format!("\nexport PATH=\"{dir}:$PATH\"\n");
                if let Ok(content) = std::fs::read_to_string(&path) {
                    if !content.contains(&*dir) {
                        let _ = std::fs::write(&path, format!("{content}{export}"));
                    }
                }
                break;
            }
        }
        Ok(())
    }
}

#[cfg(windows)]
fn append_user_path_windows(install_dir: &Path) -> IpcResult<()> {
    use winreg::enums::{HKEY_CURRENT_USER, KEY_READ, KEY_WRITE};
    use winreg::RegKey;

    let dir = install_dir
        .canonicalize()
        .unwrap_or_else(|_| install_dir.to_path_buf());
    let dir_s = dir.to_string_lossy().replace('/', "\\");
    let hkcu = RegKey::predef(HKEY_CURRENT_USER);
    let env = hkcu
        .open_subkey_with_flags("Environment", KEY_READ | KEY_WRITE)
        .map_err(|e| PytxoIpcError::new("install", format!("PATH registry: {e}")))?;
    let current: String = env.get_value("Path").unwrap_or_default();
    let already = current
        .split(';')
        .any(|p| p.eq_ignore_ascii_case(&dir_s) || Path::new(p) == dir.as_path());
    if !already {
        let next = if current.is_empty() {
            dir_s.clone()
        } else {
            format!("{current};{dir_s}")
        };
        env.set_value("Path", &next)
            .map_err(|e| PytxoIpcError::new("install", format!("PATH write: {e}")))?;
    }
    Ok(())
}

#[tauri::command]
pub fn pick_workspace_folder() -> Option<String> {
    rfd::FileDialog::new()
        .set_title("Select project workspace")
        .pick_folder()
        .map(|p| p.to_string_lossy().into_owned())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_cli_path_is_some() {
        // May fail only if HOME/LOCALAPPDATA missing in exotic envs.
        let _ = default_cli_binary_path();
    }

    #[test]
    fn binary_on_path_does_not_panic() {
        let _ = binary_on_path("pytxo");
    }
}
