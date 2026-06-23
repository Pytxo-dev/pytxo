//! DeepSpace outbound network isolation hooks ([[deepspace-network-v2]], ADR-0022).

use pytxo_core::Result;
#[cfg(not(any(target_os = "linux", target_os = "macos", target_os = "windows")))]
use pytxo_core::PytxoError;

/// Platform label for doctor / WAL telemetry.
pub fn isolation_mechanism() -> &'static str {
    #[cfg(all(target_os = "linux", feature = "deepspace-netns"))]
    {
        if std::env::var("PYTXO_DEEPSPACE_NETNS").as_deref() == Ok("1") {
            return "linux-netns-unshare";
        }
        return "linux-netns-stub";
    }
    #[cfg(target_os = "macos")]
    {
        return "macos-sandbox-exec";
    }
    #[cfg(target_os = "windows")]
    {
        return "windows-wfp-stub";
    }
    #[cfg(not(any(target_os = "linux", target_os = "macos", target_os = "windows")))]
    {
        "unsupported-platform"
    }
}

/// Wrap a shell one-liner for PTY / portable-pty paths that cannot take `Command` mutations.
pub fn wrap_deepspace_shell_cmd(cmd: &str) -> String {
    #[cfg(all(target_os = "linux", feature = "deepspace-netns"))]
    {
        if std::env::var("PYTXO_DEEPSPACE_NETNS").as_deref() == Ok("1") {
            return format!("unshare -n sh -c {}", shell_quote(cmd));
        }
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
        // Stub: env marker only; true WFP loopback-only is future work.
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
    #[cfg(all(target_os = "linux", feature = "deepspace-netns"))]
    {
        if std::env::var("PYTXO_DEEPSPACE_NETNS").as_deref() == Ok("1") {
            let program = cmd.get_program().to_os_string();
            let args: Vec<_> = cmd.get_args().map(|a| a.to_os_string()).collect();
            cmd.program("unshare");
            cmd.arg("-n");
            cmd.arg(program);
            for arg in args {
                cmd.arg(arg);
            }
            cmd.env("PYTXO_NETWORK_ISOLATION", "deepspace-v2-netns");
            return Ok(());
        }
        cmd.env("PYTXO_NETWORK_ISOLATION", "deepspace-v2-stub");
        cmd.env("PYTXO_DEEPSPACE_NETNS", "0");
        return Ok(());
    }
    #[cfg(all(target_os = "linux", not(feature = "deepspace-netns")))]
    {
        cmd.env("PYTXO_NETWORK_ISOLATION", "deepspace-v2-stub");
        cmd.env(
            "PYTXO_DEEPSPACE_NETNS",
            "enable deepspace-netns feature and PYTXO_DEEPSPACE_NETNS=1",
        );
        return Ok(());
    }
    #[cfg(target_os = "macos")]
    {
        let program = cmd.get_program().to_os_string();
        let args: Vec<_> = cmd.get_args().map(|a| a.to_os_string()).collect();
        let profile = "(version 1)\n(deny network-outbound)\n(allow default)\n";
        cmd.program("sandbox-exec");
        cmd.arg("-p");
        cmd.arg(profile);
        cmd.arg(program);
        for arg in args {
            cmd.arg(arg);
        }
        cmd.env("PYTXO_NETWORK_ISOLATION", "deepspace-v2-sandbox");
        return Ok(());
    }
    #[cfg(target_os = "windows")]
    {
        // WAL marker until WFP / AppContainer loopback-only lands.
        cmd.env("PYTXO_NETWORK_ISOLATION", "deepspace-v2-stub");
        return Ok(());
    }
    #[cfg(not(any(target_os = "linux", target_os = "macos", target_os = "windows")))]
    {
        Err(PytxoError::Runner(
            "DeepSpace network isolation unsupported on this platform".into(),
        ))
    }
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
        #[cfg(target_os = "windows")]
        assert_eq!(
            cmd.get_envs()
                .find(|(k, _)| k == &std::ffi::OsStr::new("PYTXO_NETWORK_ISOLATION"))
                .and_then(|(_, v)| v)
                .map(|v| v.to_string_lossy().into_owned()),
            Some("deepspace-v2-stub".into())
        );
    }
}
