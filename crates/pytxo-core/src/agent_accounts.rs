//! Named sign-ins per agent CLI ([[2026-10-17-agent-cli-accounts-design]]).
//!
//! An account is that CLI's own config directory under
//! `~/.pytxo/agent-accounts/<cli>/<name>/`, selected for a launch through the
//! one variable the vendor documents for it. Pytxo creates and deletes the
//! folder but never opens anything inside it: the vendor CLI writes and reads
//! its own credentials there or in the OS keychain entry it keys to that folder.

use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::time::{Duration, Instant};

use crate::{all_ade_clis, resolve_ade, AdeCliSpec, PytxoError, Result};

/// The CLI's own home (`~/.claude`, `~/.codex`): no variable set.
pub const DEFAULT_ACCOUNT: &str = "default";
const ACTIVE_FILE: &str = ".active";

/// The variable that moves this CLI's whole sign-in, or `None` when the CLI
/// keeps one sign-in per OS user (Antigravity's credential-manager entry and
/// Gemini CLI's keychain entry are fixed; Cursor Agent's `CURSOR_CONFIG_DIR`
/// moves settings but a fresh one still reports signed in; see the design note).
pub fn account_env(cli_id: &str) -> Option<&'static str> {
    match cli_id {
        "claude" => Some("CLAUDE_CONFIG_DIR"),
        "codex" => Some("CODEX_HOME"),
        _ => None,
    }
}

/// Registry entries whose sign-in can be moved per account.
pub fn account_clis() -> impl Iterator<Item = &'static AdeCliSpec> {
    all_ade_clis()
        .iter()
        .filter(|spec| account_env(spec.id).is_some())
}

pub fn default_root() -> PathBuf {
    std::env::var_os("PYTXO_HOME")
        .or_else(|| std::env::var_os(if cfg!(windows) { "USERPROFILE" } else { "HOME" }))
        .or_else(|| std::env::var_os("HOME"))
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from("."))
        .join(".pytxo")
        .join("agent-accounts")
}

pub fn validate_name(name: &str) -> Result<()> {
    let ok = (1..=32).contains(&name.len())
        && name
            .bytes()
            .all(|b| b.is_ascii_lowercase() || b.is_ascii_digit() || b == b'-')
        && !name.starts_with('-');
    if !ok {
        return Err(PytxoError::Config(format!(
            "account name {name:?} must be 1-32 lowercase letters, digits or dashes"
        )));
    }
    Ok(())
}

fn account_cli(cli: &str) -> Result<&'static AdeCliSpec> {
    let spec =
        resolve_ade(cli).ok_or_else(|| PytxoError::Config(format!("unknown agent CLI {cli:?}")))?;
    if account_env(spec.id).is_none() {
        return Err(PytxoError::Config(format!(
            "{} keeps one sign-in per OS user, so Pytxo cannot hold separate accounts for it",
            spec.display_name
        )));
    }
    Ok(spec)
}

fn named(cli: &str, name: &str) -> Result<&'static AdeCliSpec> {
    let spec = account_cli(cli)?;
    validate_name(name)?;
    if name == DEFAULT_ACCOUNT {
        return Err(PytxoError::Config(format!(
            "{DEFAULT_ACCOUNT:?} is the CLI's own sign-in and cannot be added or removed"
        )));
    }
    Ok(spec)
}

pub fn home(root: &Path, cli_id: &str, name: &str) -> PathBuf {
    root.join(cli_id).join(name)
}

/// Named accounts for a CLI, sorted. `default` is implicit and not listed.
pub fn list(root: &Path, cli: &str) -> Result<Vec<String>> {
    let spec = account_cli(cli)?;
    let Ok(entries) = std::fs::read_dir(root.join(spec.id)) else {
        return Ok(Vec::new());
    };
    let mut names: Vec<String> = entries
        .filter_map(|entry| entry.ok())
        .filter(|entry| entry.path().is_dir())
        .filter_map(|entry| entry.file_name().into_string().ok())
        .filter(|name| name != DEFAULT_ACCOUNT && validate_name(name).is_ok())
        .collect();
    names.sort();
    Ok(names)
}

pub fn add(root: &Path, cli: &str, name: &str) -> Result<PathBuf> {
    let spec = named(cli, name)?;
    let dir = home(root, spec.id, name);
    if dir.exists() {
        return Err(PytxoError::Config(format!(
            "{} account {name:?} already exists",
            spec.display_name
        )));
    }
    std::fs::create_dir_all(&dir)?;
    Ok(dir)
}

/// Deletes the account folder, and with it the vendor's credential file. Run
/// the vendor's sign-out first so the session is revoked and any keychain
/// entry keyed to the folder is removed.
pub fn remove(root: &Path, cli: &str, name: &str) -> Result<()> {
    let spec = named(cli, name)?;
    let dir = home(root, spec.id, name);
    if !dir.is_dir() {
        return Err(PytxoError::Config(format!(
            "{} account {name:?} does not exist",
            spec.display_name
        )));
    }
    if active(root, spec.id) == name {
        set_active(root, spec.id, DEFAULT_ACCOUNT)?;
    }
    std::fs::remove_dir_all(dir)?;
    Ok(())
}

/// The account new work uses for this CLI; `default` when none is chosen or
/// the chosen folder is gone.
pub fn active(root: &Path, cli_id: &str) -> String {
    std::fs::read_to_string(root.join(cli_id).join(ACTIVE_FILE))
        .ok()
        .map(|text| text.trim().to_string())
        .filter(|name| validate_name(name).is_ok() && home(root, cli_id, name).is_dir())
        .unwrap_or_else(|| DEFAULT_ACCOUNT.to_string())
}

pub fn set_active(root: &Path, cli: &str, name: &str) -> Result<()> {
    let spec = account_cli(cli)?;
    validate_name(name)?;
    let file = root.join(spec.id).join(ACTIVE_FILE);
    if name == DEFAULT_ACCOUNT {
        return match std::fs::remove_file(file) {
            Err(error) if error.kind() != std::io::ErrorKind::NotFound => Err(error.into()),
            _ => Ok(()),
        };
    }
    if !home(root, spec.id, name).is_dir() {
        return Err(PytxoError::Config(format!(
            "{} account {name:?} does not exist; add it first",
            spec.display_name
        )));
    }
    std::fs::write(file, name)?;
    Ok(())
}

/// The variable and folder that select a named account, or `None` for
/// `default`.
pub fn env_for(root: &Path, cli_id: &str, name: &str) -> Result<Option<(&'static str, PathBuf)>> {
    if name == DEFAULT_ACCOUNT {
        return Ok(None);
    }
    let spec = named(cli_id, name)?;
    let dir = home(root, spec.id, name);
    if !dir.is_dir() {
        return Err(PytxoError::Config(format!(
            "{} account {name:?} is not set up; run `pytxo accounts add {} {name}`",
            spec.display_name, spec.id
        )));
    }
    let var = account_env(spec.id).expect("named() checked account support");
    Ok(Some((var, dir)))
}

/// The registry CLI a launch command starts, from its first word.
pub fn cli_for_command(cmd: &str) -> Option<&'static AdeCliSpec> {
    let first = cmd.split_whitespace().next()?.trim_matches(['"', '\'']);
    // Both separators on every host: a Windows-style path in pytxo.toml is not
    // split by `Path` on Unix.
    let file = first.rsplit(['/', '\\']).next()?.to_ascii_lowercase();
    if file.is_empty() {
        return None;
    }
    let stem = [".exe", ".cmd", ".bat", ".ps1"]
        .iter()
        .find_map(|ext| file.strip_suffix(ext))
        .unwrap_or(&file);
    all_ade_clis().iter().find(|spec| spec.probe_bin == stem)
}

/// The account one agent launch runs under.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AccountBinding {
    pub cli_id: &'static str,
    pub cli_name: &'static str,
    pub account: String,
    /// Unset for `default`.
    pub env: Option<(&'static str, PathBuf)>,
}

impl AccountBinding {
    pub fn label(&self) -> String {
        format!("{} · {}", self.cli_name, self.account)
    }

    /// One `pytxo:` line for a failed run whose output shows the vendor's
    /// signed-out or usage-limit message. Deliberately does not suggest
    /// another account: rotating accounts around a limit is what the
    /// providers' terms forbid.
    pub fn failure_hint(&self, output: &str) -> Option<String> {
        let tail = output
            .char_indices()
            .rev()
            .nth(8_000)
            .map_or(output, |(at, _)| &output[at..])
            .to_ascii_lowercase();
        let has = |needles: &[&str]| needles.iter().any(|needle| tail.contains(needle));
        if has(&[
            "login expired",
            "please run /login",
            "not logged in",
            "could not be refreshed",
            "401 unauthorized",
            "authentication_error",
            "not authenticated",
            "invalid_grant",
        ]) {
            let fix = if self.account == DEFAULT_ACCOUNT && account_env(self.cli_id).is_none() {
                "sign in again from Setup".to_string()
            } else if self.account == DEFAULT_ACCOUNT {
                format!("run `pytxo accounts login {}` or use Setup", self.cli_id)
            } else {
                format!(
                    "run `pytxo accounts login {} {}` or use Setup",
                    self.cli_id, self.account
                )
            };
            return Some(format!(
                "pytxo: {} looks signed out or its login expired; {fix}.",
                self.label()
            ));
        }
        if has(&[
            "usage limit",
            "hit your limit",
            "usage_limit_reached",
            "limit reached",
            "quota exceeded",
        ]) {
            return Some(format!(
                "pytxo: {} reported a usage limit; it resets on the provider's schedule.",
                self.label()
            ));
        }
        None
    }
}

/// Which account a launch uses: the agent's own `account` from `pytxo.toml`,
/// else the CLI's active account. `None` for commands that are not a registry
/// CLI. Refuses an agent account on a CLI that cannot hold one.
pub fn launch_binding(
    root: &Path,
    launcher: &str,
    agent_account: Option<&str>,
) -> Result<Option<AccountBinding>> {
    let Some(spec) = cli_for_command(launcher) else {
        if let Some(name) = agent_account.filter(|name| *name != DEFAULT_ACCOUNT) {
            return Err(PytxoError::Config(format!(
                "agent account {name:?} is set, but `{launcher}` is not a CLI Pytxo can sign in per account"
            )));
        }
        return Ok(None);
    };
    if account_env(spec.id).is_none() {
        if agent_account.is_some_and(|name| name != DEFAULT_ACCOUNT) {
            // Refuses with the CLI's name and the reason.
            account_cli(spec.id)?;
        }
        return Ok(Some(AccountBinding {
            cli_id: spec.id,
            cli_name: spec.display_name,
            account: DEFAULT_ACCOUNT.into(),
            env: None,
        }));
    }
    let account = match agent_account {
        Some(name) => {
            validate_name(name)?;
            name.to_string()
        }
        None => active(root, spec.id),
    };
    let env = env_for(root, spec.id, &account)?;
    Ok(Some(AccountBinding {
        cli_id: spec.id,
        cli_name: spec.display_name,
        account,
        env,
    }))
}

/// Sign-in state from the vendor's own status command. Expiry of a refresh
/// token is not visible here for every CLI (Codex reports "Logged in" until a
/// request fails), so runs also report it through [`AccountBinding::failure_hint`].
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SessionState {
    SignedIn,
    SignedOut,
    Unknown,
}

impl SessionState {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::SignedIn => "signed_in",
            Self::SignedOut => "signed_out",
            Self::Unknown => "unknown",
        }
    }
}

pub struct ProbeOutput {
    pub success: bool,
    pub output: String,
}

/// Vendor status command (executable, arguments) for CLIs Pytxo can check.
pub fn status_command(cli_id: &str) -> Option<(&'static str, &'static [&'static str])> {
    match cli_id {
        "codex" => Some(("codex", &["login", "status"])),
        "claude" => Some(("claude", &["auth", "status", "--json"])),
        "cursor" => Some(("cursor-agent", &["status"])),
        "opencode" => Some(("opencode", &["auth", "list"])),
        _ => None,
    }
}

/// Vendor sign-in and sign-out commands for CLIs that hold accounts.
pub fn sign_in_command(cli_id: &str) -> Option<(&'static str, &'static [&'static str])> {
    match cli_id {
        "codex" => Some(("codex", &["login"])),
        "claude" => Some(("claude", &["auth", "login"])),
        _ => None,
    }
}

pub fn sign_out_command(cli_id: &str) -> Option<(&'static str, &'static [&'static str])> {
    match cli_id {
        "codex" => Some(("codex", &["logout"])),
        "claude" => Some(("claude", &["auth", "logout"])),
        _ => None,
    }
}

pub fn session_state(cli_id: &str, probe: &ProbeOutput) -> SessionState {
    let lower = probe.output.to_ascii_lowercase();
    let negative = lower.contains("not logged") || lower.contains("not authenticated");
    match cli_id {
        "claude" => {
            let logged_in = serde_json::from_str::<serde_json::Value>(probe.output.trim())
                .ok()
                .and_then(|value| value.get("loggedIn").and_then(serde_json::Value::as_bool))
                .unwrap_or(false);
            if probe.success && logged_in {
                SessionState::SignedIn
            } else {
                SessionState::SignedOut
            }
        }
        "cursor" if lower.contains("unable to fetch") => SessionState::Unknown,
        "codex" | "cursor" if probe.success && !negative && lower.contains("logged in") => {
            SessionState::SignedIn
        }
        _ if !probe.success || negative => SessionState::SignedOut,
        _ => SessionState::Unknown,
    }
}

/// Probe one account. `None` when the CLI has no status command or the probe
/// could not finish in 10 seconds.
pub fn probe_account(root: &Path, cli_id: &str, account: &str) -> Option<SessionState> {
    let (executable, args) = status_command(cli_id)?;
    let mut command = host_command(executable, args);
    if let Ok(Some((var, dir))) = env_for(root, cli_id, account) {
        command.env(var, dir);
    }
    run_probe_command(command).map(|probe| session_state(cli_id, &probe))
}

pub fn host_command(executable: &str, args: &[&str]) -> Command {
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

/// Vendor auth commands run from the user's home, never a repository.
pub fn host_auth_dir() -> PathBuf {
    std::env::var_os(if cfg!(windows) { "USERPROFILE" } else { "HOME" })
        .map(PathBuf::from)
        .filter(|path| path.is_dir())
        .unwrap_or_else(std::env::temp_dir)
}

pub fn run_probe_command(mut command: Command) -> Option<ProbeOutput> {
    // Readiness checks run in the background; only an explicit sign-in opens a terminal.
    #[cfg(windows)]
    {
        use std::os::windows::process::CommandExt;
        const CREATE_NO_WINDOW: u32 = 0x0800_0000;
        command.creation_flags(CREATE_NO_WINDOW);
    }
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
    Some(ProbeOutput {
        success: output.status.success(),
        output: combined,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn names_are_plain_and_default_is_reserved() {
        for good in ["work", "max-2", "a"] {
            validate_name(good).unwrap();
        }
        for bad in ["", "Work", "-x", "a/b", "..", ".active", &"x".repeat(33)] {
            assert!(validate_name(bad).is_err(), "{bad:?}");
        }
        let root = tempfile::tempdir().unwrap();
        assert!(add(root.path(), "claude", DEFAULT_ACCOUNT).is_err());
        assert!(remove(root.path(), "claude", DEFAULT_ACCOUNT).is_err());
    }

    #[test]
    fn add_use_list_remove_round_trip() {
        let root = tempfile::tempdir().unwrap();
        let root = root.path();
        assert!(list(root, "claude").unwrap().is_empty());
        assert_eq!(active(root, "claude"), DEFAULT_ACCOUNT);

        let work = add(root, "claude", "work").unwrap();
        add(root, "claude_code", "max-2").unwrap();
        assert!(add(root, "claude", "work").is_err(), "duplicate refused");
        assert_eq!(list(root, "claude").unwrap(), ["max-2", "work"]);
        assert!(list(root, "codex").unwrap().is_empty(), "per-CLI folders");

        assert!(set_active(root, "claude", "missing").is_err());
        set_active(root, "claude", "work").unwrap();
        assert_eq!(active(root, "claude"), "work");
        assert_eq!(
            env_for(root, "claude", "work").unwrap(),
            Some(("CLAUDE_CONFIG_DIR", work.clone()))
        );

        remove(root, "claude", "work").unwrap();
        assert!(!work.exists());
        assert_eq!(active(root, "claude"), DEFAULT_ACCOUNT, "active resets");
        set_active(root, "claude", DEFAULT_ACCOUNT).unwrap();
    }

    #[test]
    fn single_sign_in_clis_are_refused() {
        let root = tempfile::tempdir().unwrap();
        for cli in ["agy", "gemini", "cursor", "copilot"] {
            assert!(add(root.path(), cli, "work").is_err(), "{cli}");
        }
        assert!(add(root.path(), "nope", "work").is_err());
    }

    #[test]
    fn commands_map_to_registry_clis() {
        assert_eq!(
            cli_for_command("claude -p --permission-mode acceptEdits")
                .unwrap()
                .id,
            "claude"
        );
        assert_eq!(
            cli_for_command("codex exec --sandbox workspace-write")
                .unwrap()
                .id,
            "codex"
        );
        assert_eq!(
            cli_for_command(r#""C:\tools\cursor-agent.cmd" -p"#)
                .unwrap()
                .id,
            "cursor"
        );
        assert_eq!(cli_for_command("/usr/bin/agy -p").unwrap().id, "agy");
        assert!(cli_for_command("powershell -NoProfile").is_none());
        assert!(cli_for_command("").is_none());
    }

    #[test]
    fn launch_binding_sets_one_variable_for_named_accounts() {
        let root = tempfile::tempdir().unwrap();
        let root = root.path();
        let codex = "codex exec --sandbox workspace-write";

        let default = launch_binding(root, codex, None).unwrap().unwrap();
        assert_eq!(default.account, DEFAULT_ACCOUNT);
        assert_eq!(default.env, None, "default leaves the CLI's own home");

        assert!(
            launch_binding(root, codex, Some("work")).is_err(),
            "missing account fails before spawn"
        );
        let dir = add(root, "codex", "work").unwrap();
        let bound = launch_binding(root, codex, Some("work")).unwrap().unwrap();
        assert_eq!(bound.env, Some(("CODEX_HOME", dir.clone())));
        assert_eq!(bound.label(), "OpenAI Codex · work");

        set_active(root, "codex", "work").unwrap();
        let active = launch_binding(root, codex, None).unwrap().unwrap();
        assert_eq!(active.env, Some(("CODEX_HOME", dir)));
        let pinned = launch_binding(root, codex, Some(DEFAULT_ACCOUNT))
            .unwrap()
            .unwrap();
        assert_eq!(pinned.env, None, "an agent can pin the default account");

        assert!(launch_binding(root, "agy -p", Some("work")).is_err());
        assert_eq!(
            launch_binding(root, "agy -p", None).unwrap().unwrap().env,
            None
        );
        assert!(launch_binding(root, "echo hi", Some("work")).is_err());
        assert_eq!(launch_binding(root, "echo hi", None).unwrap(), None);
    }

    #[test]
    fn failure_hints_name_the_account_but_never_suggest_rotation() {
        let binding = AccountBinding {
            cli_id: "codex",
            cli_name: "OpenAI Codex",
            account: "work".into(),
            env: None,
        };
        let expired = binding
            .failure_hint("ERROR: Your access token could not be refreshed (401)")
            .unwrap();
        assert!(expired.contains("OpenAI Codex · work"), "{expired}");
        assert!(
            expired.contains("pytxo accounts login codex work"),
            "{expired}"
        );

        let limit = binding
            .failure_hint("You've hit your usage limit. Try again later.")
            .unwrap();
        assert!(limit.contains("usage limit"), "{limit}");
        assert!(!limit.contains("another"), "{limit}");

        assert_eq!(binding.failure_hint("error[E0308]: mismatched types"), None);
    }

    #[test]
    fn session_states_from_vendor_status_output() {
        let probe = |success, output: &str| ProbeOutput {
            success,
            output: output.into(),
        };
        let state = |cli, success, output| session_state(cli, &probe(success, output));
        assert_eq!(
            state("claude", true, r#"{"loggedIn":true}"#),
            SessionState::SignedIn
        );
        assert_eq!(
            state("claude", true, r#"{"loggedIn":false}"#),
            SessionState::SignedOut
        );
        assert_eq!(
            state("codex", true, "Logged in using ChatGPT"),
            SessionState::SignedIn
        );
        assert_eq!(
            state("codex", false, "Not logged in"),
            SessionState::SignedOut
        );
        assert_eq!(
            state("codex", true, "Not logged in"),
            SessionState::SignedOut
        );
        assert_eq!(
            state("cursor", true, "Logged in (unable to fetch user details)"),
            SessionState::Unknown
        );
        assert_eq!(
            state("cursor", true, "Logged in as a@b.c"),
            SessionState::SignedIn
        );
    }
}
