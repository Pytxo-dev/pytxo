use serde::Serialize;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::time::{Duration, Instant};

use pytxo_core::DomainId;
use pytxo_orchestrate::{effective_entitlements, fetch_link_wallet_balance};
use tauri::State;

use crate::ipc::{load_cfg_for_domain, open_store_for_domain, resolve_domain, AppState};
use crate::ipc_error::{map_config_err, map_io_err, IpcResult, PytxoIpcError};

#[derive(Serialize)]
pub struct EntitlementStatusDto {
    pub tier: String,
    pub max_agents: usize,
    pub cloud_enabled: bool,
    pub wallet_balance_microcredits: Option<i64>,
    pub permission_ceiling: Option<String>,
    pub subscription_portal_url: Option<String>,
}

#[derive(Serialize)]
pub struct AdeCliStatusDto {
    pub id: String,
    pub display_name: String,
    pub default_cmd: String,
    pub installed: bool,
    pub auth_state: String,
    pub auth_label: String,
    pub auth_owner: String,
    pub login_supported: bool,
    pub login_label: Option<String>,
    pub docs_url: String,
    pub detail: String,
}

#[derive(Serialize)]
pub struct AdeLoginLaunchDto {
    pub id: String,
    pub message: String,
}

#[tauri::command]
pub fn ipc_version() -> String {
    env!("CARGO_PKG_VERSION").to_string()
}

#[tauri::command]
pub fn check_pytxo_cli() -> bool {
    crate::ipc_install::cli_binary_usable()
}

/// Real on-disk detection for every registered ADE CLI (Integrations screen).
#[tauri::command]
pub async fn list_ade_clis() -> IpcResult<Vec<AdeCliStatusDto>> {
    tokio::task::spawn_blocking(list_ade_clis_sync)
        .await
        .map_err(|error| PytxoIpcError::new("integrations", format!("auth probe failed: {error}")))
}

fn list_ade_clis_sync() -> Vec<AdeCliStatusDto> {
    let handles = pytxo_core::all_ade_clis()
        .iter()
        .copied()
        .map(|spec| std::thread::spawn(move || ade_status(spec)))
        .collect::<Vec<_>>();
    handles
        .into_iter()
        .filter_map(|handle| handle.join().ok())
        .collect()
}

fn ade_status(spec: pytxo_core::AdeCliSpec) -> AdeCliStatusDto {
    let installed = pytxo_core::ade_on_path(&spec);
    let meta = auth_meta(spec.id, installed);
    AdeCliStatusDto {
        id: spec.id.to_string(),
        display_name: spec.display_name.to_string(),
        default_cmd: spec.default_cmd.to_string(),
        installed,
        auth_state: meta.state.into(),
        auth_label: meta.label,
        auth_owner: meta.owner.into(),
        login_supported: meta.login_label.is_some(),
        login_label: meta.login_label.map(str::to_string),
        docs_url: meta.docs_url.into(),
        detail: meta.detail.into(),
    }
}

struct AuthMeta {
    state: &'static str,
    label: String,
    owner: &'static str,
    login_label: Option<&'static str>,
    docs_url: &'static str,
    detail: &'static str,
}

fn auth_meta(id: &str, installed: bool) -> AuthMeta {
    let mut meta = match id {
        "codex" => AuthMeta {
            state: "unknown",
            label: "Checking Codex session".into(),
            owner: "Codex",
            login_label: Some("Connect with ChatGPT"),
            docs_url: "https://developers.openai.com/codex/auth",
            detail: "Codex owns the browser session, token storage, and refresh.",
        },
        "claude" => AuthMeta {
            state: "unknown",
            label: "Checking Claude Code session".into(),
            owner: "Claude Code",
            login_label: Some("Open Claude Code sign-in"),
            docs_url: "https://code.claude.com/docs/en/authentication",
            detail: "Pytxo opens Claude Code's official sign-in and never receives its token.",
        },
        "cursor" => AuthMeta {
            state: "unknown",
            label: "Checking Cursor session".into(),
            owner: "Cursor Agent",
            login_label: Some("Open Cursor sign-in"),
            docs_url: "https://docs.cursor.com/en/cli/reference/authentication",
            detail: "Cursor Agent keeps its account credential outside Pytxo.",
        },
        "opencode" => AuthMeta {
            state: "unknown",
            label: "Checking OpenCode providers".into(),
            owner: "OpenCode",
            login_label: Some("Connect an OpenCode provider"),
            docs_url: "https://opencode.ai/docs/providers/",
            detail: "Provider-specific credentials remain owned by OpenCode.",
        },
        "gemini" => AuthMeta {
            state: "unknown",
            label: "Check authentication in Gemini CLI".into(),
            owner: "Gemini CLI",
            login_label: Some("Open Gemini authentication"),
            docs_url: "https://geminicli.com/docs/get-started/authentication/",
            detail: "Gemini CLI owns Google OAuth; Pytxo does not reuse its cached token.",
        },
        "copilot" => AuthMeta {
            state: "unknown",
            label: "Check authentication in Copilot CLI".into(),
            owner: "Copilot CLI",
            login_label: Some("Open GitHub sign-in"),
            docs_url: "https://docs.github.com/en/copilot/how-tos/copilot-cli/set-up-copilot-cli/authenticate-copilot-cli",
            detail: "Copilot CLI owns the GitHub device flow and stores its token in the OS keychain.",
        },
        "aider" => AuthMeta {
            state: "not_applicable",
            label: "Uses the selected API provider".into(),
            owner: "Pytxo run policy",
            login_label: None,
            docs_url: "https://aider.chat/docs/config/api-keys.html",
            detail: "Choose one explicit BYOK credential for the run; unrelated keys stay hidden.",
        },
        _ => AuthMeta {
            state: "unknown",
            label: "Authentication managed by this CLI".into(),
            owner: "Vendor CLI",
            login_label: None,
            docs_url: "https://pytxo.com/docs/reference/providers-byok",
            detail: "Pytxo detects the executable without reading vendor credential stores.",
        },
    };

    if !installed {
        meta.state = "not_installed";
        meta.label = "Not installed".into();
        return meta;
    }

    let probe = match id {
        "codex" => Some(run_auth_probe("codex", &["login", "status"])),
        "claude" => Some(run_auth_probe("claude", &["auth", "status", "--json"])),
        "cursor" => Some(run_auth_probe("cursor-agent", &["status"])),
        "opencode" => Some(run_auth_probe("opencode", &["auth", "list"])),
        _ => None,
    };
    if let Some(probe) = probe {
        if let Some(probe) = probe {
            apply_probe_result(id, &probe, &mut meta);
        } else {
            meta.state = "unknown";
            meta.label = "Session status unavailable — recheck".into();
        }
    }
    meta
}

struct ProbeResult {
    success: bool,
    output: String,
}

fn run_auth_probe(executable: &str, args: &[&str]) -> Option<ProbeResult> {
    let mut command = host_command(executable, args);
    let mut child = command
        .current_dir(host_auth_dir())
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .ok()?;
    let started = Instant::now();
    loop {
        match child.try_wait() {
            Ok(Some(_)) => break,
            Ok(None) if started.elapsed() < Duration::from_secs(10) => {
                std::thread::sleep(Duration::from_millis(50));
            }
            Ok(None) => {
                let _ = child.kill();
                let _ = child.wait();
                return None;
            }
            Err(_) => return None,
        }
    }
    let output = child.wait_with_output().ok()?;
    let mut combined = String::from_utf8_lossy(&output.stdout).into_owned();
    combined.push('\n');
    combined.push_str(&String::from_utf8_lossy(&output.stderr));
    Some(ProbeResult {
        success: output.status.success(),
        output: combined,
    })
}

fn apply_probe_result(id: &str, probe: &ProbeResult, meta: &mut AuthMeta) {
    let lower = probe.output.to_ascii_lowercase();
    match id {
        "codex" if probe.success && lower.contains("logged in") => {
            meta.state = "signed_in";
            meta.label = if lower.contains("chatgpt") {
                "ChatGPT connected".into()
            } else {
                "Codex connected".into()
            };
        }
        "claude" => {
            let logged_in = serde_json::from_str::<serde_json::Value>(probe.output.trim())
                .ok()
                .and_then(|value| value.get("loggedIn").and_then(serde_json::Value::as_bool))
                .unwrap_or(false);
            if probe.success && logged_in {
                meta.state = "signed_in";
                meta.label = "Claude account connected".into();
            } else {
                meta.state = "signed_out";
                meta.label = "Claude Code is not signed in".into();
            }
        }
        "cursor" if probe.success && lower.contains("logged in") => {
            meta.state = "signed_in";
            meta.label = "Cursor account connected".into();
        }
        "opencode" if probe.success && !lower.contains("0 credentials") => {
            meta.state = "signed_in";
            meta.label = "OpenCode provider connected".into();
        }
        "opencode" if probe.success => {
            meta.state = "signed_out";
            meta.label = "No OpenCode provider connected".into();
        }
        _ if !probe.success
            || lower.contains("not logged")
            || lower.contains("not authenticated") =>
        {
            meta.state = "signed_out";
            meta.label = "Not connected".into();
        }
        _ => {}
    }
}

fn host_command(executable: &str, args: &[&str]) -> Command {
    if cfg!(windows) {
        let mut command = Command::new("cmd");
        let command_line = std::iter::once(executable)
            .chain(args.iter().copied())
            .collect::<Vec<_>>()
            .join(" ");
        command.args(["/D", "/S", "/C", &command_line]);
        command
    } else {
        let mut command = Command::new(executable);
        command.args(args);
        command
    }
}

fn host_auth_dir() -> PathBuf {
    std::env::var_os(if cfg!(windows) { "USERPROFILE" } else { "HOME" })
        .map(PathBuf::from)
        .filter(|path| path.is_dir())
        .unwrap_or_else(std::env::temp_dir)
}

#[tauri::command]
pub fn start_ade_login(id: String) -> IpcResult<AdeLoginLaunchDto> {
    let spec = pytxo_core::resolve_ade(&id)
        .ok_or_else(|| PytxoIpcError::new("integrations", "Unknown agent CLI."))?;
    if !pytxo_core::ade_on_path(spec) {
        return Err(PytxoIpcError::new(
            "integrations",
            format!("{} is not installed or is not on PATH.", spec.display_name),
        ));
    }
    let (executable, args, message) = match spec.id {
        "codex" => (
            "codex",
            vec!["login"],
            "Codex sign-in opened. Finish the official ChatGPT flow, then recheck.",
        ),
        "claude" => (
            "claude",
            vec!["auth", "login"],
            "Claude Code sign-in opened. Finish in the vendor terminal, then recheck.",
        ),
        "cursor" => (
            "cursor-agent",
            vec!["login"],
            "Cursor Agent sign-in opened. Finish in your browser, then recheck.",
        ),
        "opencode" => (
            "opencode",
            vec!["auth", "login"],
            "OpenCode provider setup opened. Choose a provider there, then recheck.",
        ),
        "gemini" => (
            "gemini",
            Vec::new(),
            "Gemini CLI opened. Run /auth in that terminal, then recheck.",
        ),
        "copilot" => (
            "copilot",
            vec!["login"],
            "GitHub Copilot sign-in opened. Finish the device flow, then recheck.",
        ),
        _ => {
            return Err(PytxoIpcError::new(
                "integrations",
                "This CLI does not expose a supported vendor-owned sign-in action.",
            ))
        }
    };
    launch_vendor_terminal(executable, &args)?;
    Ok(AdeLoginLaunchDto {
        id: id.to_ascii_lowercase(),
        message: message.into(),
    })
}

fn launch_vendor_terminal(executable: &str, args: &[&str]) -> IpcResult<()> {
    let command_line = std::iter::once(executable)
        .chain(args.iter().copied())
        .collect::<Vec<_>>()
        .join(" ");

    #[cfg(windows)]
    {
        use std::os::windows::process::CommandExt;
        const CREATE_NEW_CONSOLE: u32 = 0x0000_0010;
        Command::new("cmd")
            .args(["/D", "/K", &command_line])
            .current_dir(host_auth_dir())
            .creation_flags(CREATE_NEW_CONSOLE)
            .spawn()
            .map_err(map_io_err)?;
        Ok(())
    }

    #[cfg(target_os = "macos")]
    {
        let script = format!(
            "tell application \"Terminal\" to do script \"{}\"",
            command_line.replace('"', "\\\"")
        );
        Command::new("osascript")
            .args(["-e", &script])
            .current_dir(host_auth_dir())
            .spawn()
            .map_err(map_io_err)?;
        Ok(())
    }

    #[cfg(all(unix, not(target_os = "macos")))]
    {
        let terminal = ["x-terminal-emulator", "gnome-terminal", "konsole"]
            .into_iter()
            .find(|candidate| {
                std::env::var_os("PATH").is_some_and(|path| {
                    std::env::split_paths(&path)
                        .any(|directory| directory.join(candidate).is_file())
                })
            })
            .ok_or_else(|| {
                PytxoIpcError::new(
                    "integrations",
                    "No supported terminal application was found.",
                )
            })?;
        let shell_command = format!("{command_line}; exec sh");
        let mut command = Command::new(terminal);
        if terminal == "gnome-terminal" {
            command.args(["--", "sh", "-lc", &shell_command]);
        } else {
            command.args(["-e", "sh", "-lc", &shell_command]);
        }
        command
            .current_dir(host_auth_dir())
            .spawn()
            .map_err(map_io_err)?;
        Ok(())
    }
}

#[tauri::command]
pub fn entitlement_status(
    state: State<'_, AppState>,
    domain_id: Option<String>,
) -> IpcResult<EntitlementStatusDto> {
    let domain = resolve_domain(&state, domain_id)?;
    let cfg = load_cfg_for_domain(&domain, &state)?;
    let ent = effective_entitlements(&cfg).map_err(map_config_err)?;
    let mut wallet_balance = open_store_for_domain(&cfg, &domain).ok().and_then(|store| {
        DomainId::from_repo_root(Path::new(&domain))
            .ok()
            .and_then(|d| store.wallet_balance_microcredits(&d).ok())
    });
    if ent.tier == "ultra" && cfg.billing.link_reconcile_enabled() {
        if let Ok(remote) = fetch_link_wallet_balance(cfg.billing.link_base_url()) {
            wallet_balance = Some(remote);
        }
    }
    let portal = if ent.tier != "core" {
        Some("https://pytxo.com/account#subscription".into())
    } else {
        Some("https://pytxo.com/plans".into())
    };
    Ok(EntitlementStatusDto {
        tier: ent.tier,
        max_agents: ent.max_agents,
        cloud_enabled: ent.cloud_enabled,
        wallet_balance_microcredits: wallet_balance,
        permission_ceiling: ent.permission_ceiling.map(|p| p.as_str().to_string()),
        subscription_portal_url: portal,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ipc_version_matches_crate() {
        assert_eq!(ipc_version(), env!("CARGO_PKG_VERSION"));
    }

    #[test]
    fn auth_probe_redacts_account_identity() {
        let mut meta = auth_meta("cursor", false);
        apply_probe_result(
            "cursor",
            &ProbeResult {
                success: true,
                output: "Logged in as developer@example.com".into(),
            },
            &mut meta,
        );
        assert_eq!(meta.state, "signed_in");
        assert_eq!(meta.label, "Cursor account connected");
        assert!(!meta.label.contains('@'));
    }

    #[test]
    fn claude_probe_returns_boolean_readiness_only() {
        let mut meta = auth_meta("claude", false);
        apply_probe_result(
            "claude",
            &ProbeResult {
                success: true,
                output: r#"{"loggedIn":true,"email":"private@example.com"}"#.into(),
            },
            &mut meta,
        );
        assert_eq!(meta.state, "signed_in");
        assert_eq!(meta.label, "Claude account connected");
        assert!(!meta.label.contains("private"));
    }

    #[test]
    fn opencode_zero_credentials_is_signed_out() {
        let mut meta = auth_meta("opencode", false);
        apply_probe_result(
            "opencode",
            &ProbeResult {
                success: true,
                output: "0 credentials".into(),
            },
            &mut meta,
        );
        assert_eq!(meta.state, "signed_out");
    }
}
