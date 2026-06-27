//! DeepSpace outbound network isolation hooks ([[deepspace-network-v2]], ADR-0022, ADR-0026).

use pytxo_core::Result;
#[cfg(not(any(target_os = "linux", target_os = "macos", target_os = "windows")))]
use pytxo_core::PytxoError;

/// Linux netns is default-on; set `PYTXO_DEEPSPACE_NETNS=0` to disable for debugging.
#[cfg(target_os = "linux")]
fn linux_netns_enabled() -> bool {
    std::env::var("PYTXO_DEEPSPACE_NETNS").as_deref() != Ok("0")
}

/// Platform label for doctor / WAL telemetry.
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
        let isolation = std::env::var("PYTXO_NETWORK_ISOLATION").unwrap_or_default();
        if isolation == "1" || isolation.eq_ignore_ascii_case("true") {
            return "windows-appcontainer-attempt";
        }
        return "windows-wfp-stub";
    }
    #[cfg(not(any(target_os = "linux", target_os = "macos", target_os = "windows")))]
    {
        "unsupported-platform"
    }
}

/// Wrap a shell one-liner for PTY / portable-pty paths that cannot take `Command` mutations.
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
        let profile = "(version 1)\n(deny network-outbound)\n(allow default)\n";
        return format!(
            "sandbox-exec -p {} sh -c {}",
            shell_quote(profile),
            shell_quote(cmd)
        );
    }
    #[cfg(target_os = "windows")]
    {
        // WFP loopback-only is production path; PYTXO_NETWORK_ISOLATION=1 opts into AppContainer attempt.
        let _ = std::env::var("PYTXO_NETWORK_ISOLATION").ok();
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
                let profile = "(version 1)\n(deny network-outbound)\n(allow default)\n";
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
        // Production: WFP loopback-only filter driver. Opt-in AppContainer attempt via env.
        let isolation = std::env::var("PYTXO_NETWORK_ISOLATION").unwrap_or_default();
        if isolation == "1" || isolation.eq_ignore_ascii_case("true") {
            cmd.env("PYTXO_NETWORK_ISOLATION", "deepspace-v2-appcontainer-attempt");
            cmd.env(
                "PYTXO_NETWORK_ISOLATION_HINT",
                "AppContainer SID restriction pending; use WFP for production egress deny",
            );
        } else {
            cmd.env("PYTXO_NETWORK_ISOLATION", "deepspace-v2-wfp-stub");
            cmd.env(
                "PYTXO_NETWORK_ISOLATION_HINT",
                "set PYTXO_NETWORK_ISOLATION=1 to opt into AppContainer attempt",
            );
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
        "python3 -c \"import socket; s=socket.socket(); s.settimeout(2); s.connect(('1.1.1.1', 443))\" 2>/dev/null || ",
        "python -c \"import socket; s=socket.socket(); s.settimeout(2); s.connect(('1.1.1.1', 443))\" 2>/dev/null || ",
        "(nc -z -w 2 1.1.1.1 443 2>/dev/null && exit 1) || exit 0"
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
            let blocked = !output.status.success();
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

    #[test]
    fn wrap_preserves_simple_cmd() {
        let wrapped = wrap_deepspace_shell_cmd("echo ok");
        #[cfg(target_os = "linux")]
        if linux_netns_enabled() {
            assert!(wrapped.contains("unshare"));
        }
        #[cfg(target_os = "macos")]
        assert!(wrapped.contains("sandbox-exec"));
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
        assert!(cmd
            .get_envs()
            .any(|(k, _)| k == std::ffi::OsStr::new("PYTXO_NETWORK_ISOLATION")));
    }

    #[test]
    fn socket_probe_reports_mechanism() {
        let (blocked, detail) = doctor_deepspace_socket_probe();
        assert!(detail.contains("mechanism="));
        assert!(detail.contains("socket_probe_blocked="));
        #[cfg(target_os = "linux")]
        if linux_netns_enabled() {
            assert!(blocked, "netns should block egress: {detail}");
        }
        let _ = blocked;
    }
}
