//! DeepSpace outbound network isolation hooks ([[deepspace-network-v2]], ADR-0022, ADR-0026).
//!
//! # Platform matrix (Phase 71)
//!
//! | Platform | Mechanism | Opt-in |
//! |----------|-----------|--------|
//! | Linux | `unshare -n` netns (default on) | `PYTXO_DEEPSPACE_NETNS=0` disables |
//! | macOS | `sandbox-exec` deny-outbound profile | always on for DeepSpace |
//! | Windows | WFP/netsh loopback egress block | `PYTXO_DEEPSPACE_WFP=1` (elevated) installs rule; else stub |

#[cfg(not(any(target_os = "linux", target_os = "macos", target_os = "windows")))]
use pytxo_core::PytxoError;
use pytxo_core::Result;

/// Linux netns is default-on; set `PYTXO_DEEPSPACE_NETNS=0` to disable for debugging.
#[cfg(target_os = "linux")]
fn linux_netns_enabled() -> bool {
    std::env::var("PYTXO_DEEPSPACE_NETNS").as_deref() != Ok("0")
}

#[cfg(target_os = "windows")]
const WFP_RULE_NAME: &str = "PytxoDeepSpaceEgressBlock";

/// True when `PYTXO_DEEPSPACE_WFP=1` (or `true`) — attempt real netsh rule install/probe.
#[cfg(target_os = "windows")]
fn windows_wfp_opt_in() -> bool {
    matches!(
        std::env::var("PYTXO_DEEPSPACE_WFP").as_deref(),
        Ok("1") | Ok("true") | Ok("TRUE")
    )
}

/// Probe whether the named advfirewall rule already exists.
#[cfg(target_os = "windows")]
fn windows_wfp_rule_present() -> bool {
    let output = windows_background_probe("netsh")
        .args([
            "advfirewall",
            "firewall",
            "show",
            "rule",
            &format!("name={WFP_RULE_NAME}"),
        ])
        .output();
    matches!(output, Ok(o) if o.status.success() && !String::from_utf8_lossy(&o.stdout).contains("No rules match"))
}

#[cfg(target_os = "windows")]
fn windows_background_probe(program: impl AsRef<std::ffi::OsStr>) -> std::process::Command {
    use std::os::windows::process::CommandExt;
    const CREATE_NO_WINDOW: u32 = 0x0800_0000;
    let mut command = std::process::Command::new(program);
    // Receipt queries run before agent startup, including from the windowed
    // Desktop host. Their output is captured; they need no interactive console.
    command.creation_flags(CREATE_NO_WINDOW);
    command
}

/// Attempt to install a program-scoped outbound block via netsh (requires elevation).
///
/// Scoped to the current executable so we never install a machine-wide deny.
/// Loopback remains reachable (Windows does not match 127.0.0.1 on this block path
/// the same way as remote Internet). Returns `true` when the rule is present after.
///
/// Remove with: `netsh advfirewall firewall delete rule name=PytxoDeepSpaceEgressBlock`
#[cfg(target_os = "windows")]
fn windows_ensure_wfp_rule() -> bool {
    if windows_wfp_rule_present() {
        return true;
    }
    let Ok(exe) = std::env::current_exe() else {
        return false;
    };
    let exe_str = exe.to_string_lossy();
    let add = std::process::Command::new("netsh")
        .args([
            "advfirewall",
            "firewall",
            "add",
            "rule",
            &format!("name={WFP_RULE_NAME}"),
            "dir=out",
            "action=block",
            "enable=yes",
            "profile=any",
            &format!("program={exe_str}"),
            "protocol=any",
            "description=Pytxo DeepSpace egress deny (Phase 71); program-scoped; opt-in PYTXO_DEEPSPACE_WFP=1",
        ])
        .output();
    match add {
        Ok(o) if o.status.success() => windows_wfp_rule_present(),
        _ => false, // Non-elevated installs fail; leave stub semantics.
    }
}

/// macOS Seatbelt profile: deny outbound network, allow local filesystem + process defaults.
#[cfg(target_os = "macos")]
fn macos_sandbox_profile() -> &'static str {
    concat!(
        "(version 1)\n",
        "(deny default)\n",
        "(allow process-exec)\n",
        "(allow process-fork)\n",
        "(allow signal)\n",
        "(allow sysctl-read)\n",
        "(allow mach-lookup)\n",
        "(allow file-read*)\n",
        "(allow file-write* (subpath \"/private/tmp\") (subpath \"/tmp\") (subpath \"/var/folders\"))\n",
        "(allow file-write* (subpath \"/Users\"))\n",
        "(deny network-outbound)\n",
        "(deny network-inbound)\n",
        "(allow network-outbound (remote ip \"localhost:*\"))\n",
        "(allow network-inbound (local ip \"localhost:*\"))\n",
    )
}

/// Platform label for doctor / WAL telemetry.
///
/// Values (Phase 71):
/// - `linux-netns-unshare` / `linux-netns-disabled`
/// - `macos-sandbox-exec`
/// - `windows-wfp-rule-present` — netsh rule installed (opt-in elevated)
/// - `windows-wfp-stub` — no rule; policy-only / AppContainer hint path
/// - `windows-appcontainer-attempt` — `PYTXO_NETWORK_ISOLATION=1` without WFP rule
#[allow(clippy::needless_return)] // cfg-specific branches are terminal on different platforms.
pub fn isolation_mechanism() -> &'static str {
    #[cfg(target_os = "linux")]
    {
        if linux_netns_enabled() {
            return "linux-netns-unshare";
        }
        return "linux-netns-disabled";
    }
    #[cfg(target_os = "macos")]
    {
        return "macos-sandbox-exec";
    }
    #[cfg(target_os = "windows")]
    {
        if windows_wfp_opt_in() {
            if windows_ensure_wfp_rule() {
                return "windows-wfp-rule-present";
            }
            return "windows-wfp-stub";
        }
        let isolation = std::env::var("PYTXO_NETWORK_ISOLATION").unwrap_or_default();
        if isolation == "1" || isolation.eq_ignore_ascii_case("true") {
            if windows_wfp_rule_present() {
                return "windows-wfp-rule-present";
            }
            return "windows-appcontainer-attempt";
        }
        if windows_wfp_rule_present() {
            return "windows-wfp-rule-present";
        }
        return "windows-wfp-stub";
    }
    #[cfg(not(any(target_os = "linux", target_os = "macos", target_os = "windows")))]
    {
        "unsupported-platform"
    }
}

/// Wrap a shell one-liner for PTY / portable-pty paths that cannot take `Command` mutations.
#[allow(clippy::needless_return)] // cfg-specific branches are terminal on different platforms.
pub fn wrap_deepspace_shell_cmd(cmd: &str) -> String {
    #[cfg(target_os = "linux")]
    {
        if linux_netns_enabled() {
            return format!("unshare -n sh -c {}", shell_quote(cmd));
        }
        return cmd.to_string();
    }
    #[cfg(target_os = "macos")]
    {
        let profile = macos_sandbox_profile();
        return format!(
            "sandbox-exec -p {} sh -c {}",
            shell_quote(profile),
            shell_quote(cmd)
        );
    }
    #[cfg(target_os = "windows")]
    {
        // Process-level wrap is a no-op; isolation is via WFP rule or stub env markers.
        let _ = windows_wfp_opt_in();
        return cmd.to_string();
    }
    #[cfg(not(any(target_os = "linux", target_os = "macos", target_os = "windows")))]
    {
        cmd.to_string()
    }
}

/// Apply DeepSpace network isolation to a subprocess `Command` before spawn.
///
/// Orbit+ profiles must not call this hook ([[permission-profile-engine]]).
#[allow(clippy::needless_return)] // cfg-specific branches are terminal on different platforms.
pub fn isolate_deepspace_network(cmd: &mut std::process::Command) -> Result<()> {
    #[cfg(target_os = "linux")]
    {
        if !linux_netns_enabled() {
            cmd.env("PYTXO_NETWORK_ISOLATION", "deepspace-v2-disabled");
            cmd.env("PYTXO_DEEPSPACE_NETNS", "0");
            return Ok(());
        }
    }
    #[cfg(any(target_os = "linux", target_os = "macos"))]
    {
        rewrap_command(cmd, |wrapped, program, args| {
            #[cfg(target_os = "linux")]
            {
                wrapped.arg("-n");
                wrapped.arg(program);
                wrapped.args(args);
                wrapped.env("PYTXO_NETWORK_ISOLATION", "deepspace-v2-netns");
                Ok(())
            }
            #[cfg(target_os = "macos")]
            {
                let profile = macos_sandbox_profile();
                wrapped.arg("-p");
                wrapped.arg(profile);
                wrapped.arg(program);
                wrapped.args(args);
                wrapped.env("PYTXO_NETWORK_ISOLATION", "deepspace-v2-sandbox");
                Ok(())
            }
        })?;
        return Ok(());
    }
    #[cfg(target_os = "windows")]
    {
        let mechanism = isolation_mechanism();
        cmd.env(
            "PYTXO_NETWORK_ISOLATION",
            format!("deepspace-v2-{mechanism}"),
        );
        cmd.env("PYTXO_ISOLATION_MECHANISM", mechanism);
        match mechanism {
            "windows-wfp-rule-present" => {
                cmd.env(
                    "PYTXO_NETWORK_ISOLATION_HINT",
                    "netsh advfirewall rule PytxoDeepSpaceEgressBlock is active",
                );
            }
            "windows-appcontainer-attempt" => {
                cmd.env(
                    "PYTXO_NETWORK_ISOLATION_HINT",
                    "AppContainer SID pending; set PYTXO_DEEPSPACE_WFP=1 (elevated) for netsh rule",
                );
            }
            _ => {
                cmd.env(
                    "PYTXO_NETWORK_ISOLATION_HINT",
                    "stub: set PYTXO_DEEPSPACE_WFP=1 as Administrator to install egress block rule",
                );
            }
        }
        return Ok(());
    }
    #[cfg(not(any(target_os = "linux", target_os = "macos", target_os = "windows")))]
    {
        Err(PytxoError::Runner(
            "DeepSpace network isolation unsupported on this platform".into(),
        ))
    }
}

#[cfg(unix)]
/// Shell one-liner that exits 0 when TCP egress to 1.1.1.1:443 is blocked, 1 when it succeeds.
fn tcp_probe_shell() -> &'static str {
    concat!(
        "if command -v python3 >/dev/null 2>&1; then ",
        "python3 -c \"import socket,sys; s=socket.socket(); s.settimeout(2); sys.exit(1 if s.connect_ex(('1.1.1.1', 443)) == 0 else 0)\"; ",
        "elif command -v python >/dev/null 2>&1; then ",
        "python -c \"import socket,sys; s=socket.socket(); s.settimeout(2); sys.exit(1 if s.connect_ex(('1.1.1.1', 443)) == 0 else 0)\"; ",
        "elif nc -z -w 2 1.1.1.1 443 2>/dev/null; then exit 1; else exit 0; fi"
    )
}

#[cfg(unix)]
fn tcp_probe_shell_cmd() -> std::process::Command {
    let mut cmd = std::process::Command::new("sh");
    cmd.arg("-c").arg(tcp_probe_shell());
    cmd
}

#[cfg(windows)]
fn tcp_probe_shell_cmd() -> std::process::Command {
    let mut cmd = std::process::Command::new("powershell");
    cmd.args([
        "-NoProfile",
        "-Command",
        concat!(
            "$t = New-Object Net.Sockets.TcpClient; ",
            "try { $t.Connect('1.1.1.1', 443); if ($t.Connected) { exit 1 } } ",
            "catch { exit 0 } finally { $t.Close() }"
        ),
    ]);
    cmd
}

/// Doctor probe: spawn isolated child that attempts TCP egress; returns `(blocked, detail)`.
pub fn doctor_deepspace_socket_probe() -> (bool, String) {
    let mut cmd = tcp_probe_shell_cmd();
    let apply = isolate_deepspace_network(&mut cmd);
    if let Err(e) = apply {
        return (
            false,
            format!(
                "mechanism={}; socket_probe=error; detail={e}",
                isolation_mechanism()
            ),
        );
    }

    match cmd.output() {
        Ok(output) => {
            let blocked = output.status.success();
            let detail = format!(
                "mechanism={}; socket_probe_blocked={blocked}; exit={}",
                isolation_mechanism(),
                output.status.code().unwrap_or(-1)
            );
            (blocked, detail)
        }
        Err(e) => (
            false,
            format!(
                "mechanism={}; socket_probe=spawn_failed; detail={e}",
                isolation_mechanism()
            ),
        ),
    }
}

#[cfg(any(target_os = "linux", target_os = "macos"))]
fn rewrap_command(
    cmd: &mut std::process::Command,
    configure: impl FnOnce(
        &mut std::process::Command,
        &std::ffi::OsStr,
        &[std::ffi::OsString],
    ) -> Result<()>,
) -> Result<()> {
    let program = cmd.get_program().to_os_string();
    let args: Vec<_> = cmd.get_args().map(|a| a.to_os_string()).collect();
    let envs: Vec<_> = cmd
        .get_envs()
        .map(|(k, v)| (k.to_os_string(), v.map(|x| x.to_os_string())))
        .collect();
    let cwd = cmd.get_current_dir().map(|p| p.to_path_buf());

    #[cfg(target_os = "linux")]
    let wrapper = "unshare";
    #[cfg(target_os = "macos")]
    let wrapper = "sandbox-exec";

    let mut wrapped = std::process::Command::new(wrapper);
    configure(&mut wrapped, program.as_os_str(), &args)?;
    for (key, val) in envs {
        match val {
            Some(v) => {
                wrapped.env(key, v);
            }
            None => {
                wrapped.env_remove(key);
            }
        }
    }
    if let Some(dir) = cwd {
        wrapped.current_dir(dir);
    }
    *cmd = wrapped;
    Ok(())
}

#[allow(dead_code)]
fn shell_quote(s: &str) -> String {
    if s.contains('\'') {
        format!("\"{}\"", s.replace('"', "\\\""))
    } else {
        format!("'{s}'")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[cfg(windows)]
    #[test]
    fn background_probe_stays_console_free_from_detached_desktop() {
        use std::os::windows::process::CommandExt;
        use std::process::Command;

        const ROLE: &str = "PYTXO_WFP_CONSOLE_TEST_ROLE";
        const TEST: &str =
            "network_isolation::tests::background_probe_stays_console_free_from_detached_desktop";
        #[link(name = "kernel32")]
        extern "system" {
            fn FreeConsole() -> i32;
            fn GetConsoleWindow() -> *mut std::ffi::c_void;
        }

        match std::env::var(ROLE).as_deref() {
            Ok("probe") => {
                assert!(
                    unsafe { GetConsoleWindow() }.is_null(),
                    "background probe created a console from a console-free parent"
                );
                println!("background-probe-stdout");
                eprintln!("background-probe-stderr");
            }
            Ok("driver") => {
                // Detach only this dedicated child, never the shared test host.
                unsafe { FreeConsole() };
                assert!(unsafe { GetConsoleWindow() }.is_null());
                let output = windows_background_probe(std::env::current_exe().unwrap())
                    .args(["--exact", TEST, "--nocapture", "--test-threads=1"])
                    .env(ROLE, "probe")
                    .output()
                    .expect("spawn controlled background probe");
                assert!(
                    output.status.success(),
                    "probe failed: {}",
                    String::from_utf8_lossy(&output.stderr)
                );
                assert!(String::from_utf8_lossy(&output.stdout).contains("background-probe-stdout"));
                assert!(String::from_utf8_lossy(&output.stderr).contains("background-probe-stderr"));
            }
            _ => {
                const DETACHED_PROCESS: u32 = 0x0000_0008;
                let output = Command::new(std::env::current_exe().unwrap())
                    .args(["--exact", TEST, "--nocapture", "--test-threads=1"])
                    .env(ROLE, "driver")
                    .creation_flags(DETACHED_PROCESS)
                    .output()
                    .expect("spawn detached test driver");
                assert!(
                    output.status.success(),
                    "detached driver failed: {}",
                    String::from_utf8_lossy(&output.stderr)
                );
            }
        }
    }

    #[test]
    fn wrap_preserves_simple_cmd() {
        let wrapped = wrap_deepspace_shell_cmd("echo ok");
        #[cfg(target_os = "linux")]
        if linux_netns_enabled() {
            assert!(wrapped.contains("unshare"));
        }
        #[cfg(target_os = "macos")]
        {
            assert!(wrapped.contains("sandbox-exec"));
            assert!(wrapped.contains("deny network-outbound"));
            assert!(wrapped.contains("allow process-exec"));
        }
        #[cfg(target_os = "windows")]
        assert_eq!(wrapped, "echo ok");
    }

    #[test]
    fn isolate_sets_marker_env() {
        let mut cmd = std::process::Command::new("echo");
        cmd.arg("hi");
        isolate_deepspace_network(&mut cmd).unwrap();
        #[cfg(target_os = "linux")]
        {
            if linux_netns_enabled() {
                assert_eq!(cmd.get_program(), std::ffi::OsStr::new("unshare"));
            }
        }
        #[cfg(target_os = "windows")]
        {
            assert!(cmd
                .get_envs()
                .any(|(k, _)| k == std::ffi::OsStr::new("PYTXO_NETWORK_ISOLATION")));
            assert!(cmd
                .get_envs()
                .any(|(k, _)| k == std::ffi::OsStr::new("PYTXO_ISOLATION_MECHANISM")));
        }
    }

    #[test]
    fn isolation_mechanism_is_nonempty() {
        let m = isolation_mechanism();
        assert!(!m.is_empty());
        #[cfg(target_os = "windows")]
        assert!(
            m.starts_with("windows-"),
            "expected windows-* mechanism, got {m}"
        );
        #[cfg(target_os = "linux")]
        assert!(m.starts_with("linux-"));
        #[cfg(target_os = "macos")]
        assert_eq!(m, "macos-sandbox-exec");
    }

    #[test]
    fn socket_probe_reports_mechanism() {
        let (blocked, detail) = doctor_deepspace_socket_probe();
        assert!(detail.contains("mechanism="));
        assert!(detail.contains("socket_probe_blocked="));
        #[cfg(target_os = "linux")]
        if linux_netns_enabled() {
            let netns_available = std::process::Command::new("unshare")
                .args(["-n", "true"])
                .status()
                .is_ok_and(|status| status.success());
            assert_eq!(
                blocked, netns_available,
                "socket probe must reflect whether the host can create a network namespace: {detail}"
            );
        }
        let _ = blocked;
    }
}
