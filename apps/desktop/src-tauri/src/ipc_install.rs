//! Install Pytxo CLI from public GitHub releases (Reality Deck setup wizard).

use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::{LazyLock, Mutex};

use serde::Serialize;

use crate::ipc_error::{map_io_err, PytxoIpcError, IpcResult};

const RELEASES_REPO: &str = "Pytxo-dev/pytxo-releases";
const DESKTOP_VERSION: &str = env!("CARGO_PKG_VERSION");

static INSTALL_STATE: LazyLock<Mutex<InstallState>> = LazyLock::new(|| {
    Mutex::new(InstallState {
        phase: String::from("idle"),
        message: String::new(),
    })
});

struct InstallState {
    phase: String,
    message: String,
}

#[derive(Serialize)]
pub struct InstallCliStatusDto {
    pub phase: String,
    pub message: String,
    pub cli_present: bool,
}

fn set_state(phase: &str, message: impl Into<String>) {
    if let Ok(mut guard) = INSTALL_STATE.lock() {
        guard.phase = phase.into();
        guard.message = message.into();
    }
}

#[tauri::command]
pub fn install_pytxo_cli_status() -> InstallCliStatusDto {
    let (phase, message) = INSTALL_STATE
        .lock()
        .map(|g| (g.phase.to_string(), g.message.to_string()))
        .unwrap_or_else(|_| ("idle".into(), String::new()));
    InstallCliStatusDto {
        phase,
        message,
        cli_present: super::ipc_meta::check_pytxo_cli(),
    }
}

#[tauri::command]
pub fn install_pytxo_cli() -> IpcResult<InstallCliStatusDto> {
    set_state("downloading", "Fetching Pytxo CLI from GitHub Releases…");
    match install_pytxo_cli_inner() {
        Ok(()) => set_state("done", "Pytxo CLI installed successfully."),
        Err(e) => set_state("error", &e.message),
    }
    Ok(install_pytxo_cli_status())
}

fn install_pytxo_cli_inner() -> IpcResult<()> {
    let (install_dir, dest, asset) = install_paths()?;
    std::fs::create_dir_all(&install_dir).map_err(map_io_err)?;
    let tag = format!("v{DESKTOP_VERSION}");
    let url = format!("https://github.com/{RELEASES_REPO}/releases/download/{tag}/{asset}");
    download_file(&url, &dest)?;
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        std::fs::set_permissions(&dest, std::fs::Permissions::from_mode(0o755))
            .map_err(map_io_err)?;
    }
    append_user_path(&install_dir)?;
    set_state("verifying", "Running pytxo doctor…");
    let doctor = Command::new(&dest).arg("doctor").output();
    if let Ok(out) = doctor {
        if !out.status.success() {
            let stderr = String::from_utf8_lossy(&out.stderr);
            if !stderr.is_empty() {
                set_state("done", &format!("Installed (doctor: {stderr})"));
            }
        }
    }
    Ok(())
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
    if cfg!(target_os = "windows") {
        let script = format!(
            "$ProgressPreference='SilentlyContinue'; Invoke-WebRequest -Uri '{url}' -OutFile '{}' -UseBasicParsing",
            dest.display()
        );
        let out = Command::new("powershell")
            .args(["-NoProfile", "-Command", &script])
            .output()
            .map_err(map_io_err)?;
        if !out.status.success() {
            return Err(PytxoIpcError::new(
                "install",
                format!(
                    "download failed: {}",
                    String::from_utf8_lossy(&out.stderr)
                ),
            ));
        }
        return Ok(());
    }
    let out = Command::new("curl")
        .args(["-fsSL", url, "-o"])
        .arg(dest)
        .output()
        .map_err(map_io_err)?;
    if !out.status.success() {
        return Err(PytxoIpcError::new(
            "install",
            format!("curl failed: {}", String::from_utf8_lossy(&out.stderr)),
        ));
    }
    Ok(())
}

fn append_user_path(install_dir: &Path) -> IpcResult<()> {
    let dir = install_dir.to_string_lossy();
    if cfg!(target_os = "windows") {
        let script = format!(
            "$d='{dir}'; $p=[Environment]::GetEnvironmentVariable('Path','User'); if ($p -notlike \"*$d*\") {{ [Environment]::SetEnvironmentVariable('Path', \"$p;$d\", 'User') }}"
        );
        let _ = Command::new("powershell")
            .args(["-NoProfile", "-Command", &script])
            .output();
        return Ok(());
    }
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

#[tauri::command]
pub fn pick_workspace_folder() -> Option<String> {
    rfd::FileDialog::new()
        .set_title("Select project workspace")
        .pick_folder()
        .map(|p| p.to_string_lossy().into_owned())
}
