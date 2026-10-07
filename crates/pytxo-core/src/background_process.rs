//! Non-interactive maintenance processes launched by the CLI or Desktop.

use std::ffi::OsStr;
use std::process::Command;

/// Build a captured-output command without changing its arguments or environment.
/// Agent execution and PTYs have separate launch contracts and must not use this.
pub fn background_command(program: impl AsRef<OsStr>) -> Command {
    let command = Command::new(program);
    #[cfg(windows)]
    let command = {
        use std::os::windows::process::CommandExt;
        const CREATE_NO_WINDOW: u32 = 0x0800_0000;
        let mut command = command;
        command.creation_flags(CREATE_NO_WINDOW);
        command
    };
    command
}

#[cfg(all(test, windows))]
mod tests {
    use super::*;
    use std::os::windows::process::CommandExt;

    #[test]
    fn maintenance_child_stays_console_free_from_detached_parent() {
        const ROLE: &str = "PYTXO_MAINTENANCE_CONSOLE_TEST_ROLE";
        const TEST: &str =
            "background_process::tests::maintenance_child_stays_console_free_from_detached_parent";
        #[link(name = "kernel32")]
        extern "system" {
            fn FreeConsole() -> i32;
            fn GetConsoleWindow() -> *mut std::ffi::c_void;
        }

        match std::env::var(ROLE).as_deref() {
            Ok("probe") => {
                assert!(
                    unsafe { GetConsoleWindow() }.is_null(),
                    "maintenance process created a console"
                );
                println!("maintenance-stdout");
                eprintln!("maintenance-stderr");
            }
            Ok("driver") => {
                // Detach only this dedicated process, not the shared test host.
                unsafe { FreeConsole() };
                assert!(unsafe { GetConsoleWindow() }.is_null());
                let output = background_command(std::env::current_exe().unwrap())
                    .args(["--exact", TEST, "--nocapture", "--test-threads=1"])
                    .env(ROLE, "probe")
                    .output()
                    .expect("start maintenance child");
                assert!(
                    output.status.success(),
                    "maintenance child failed: {}",
                    String::from_utf8_lossy(&output.stderr)
                );
                assert!(String::from_utf8_lossy(&output.stdout).contains("maintenance-stdout"));
                assert!(String::from_utf8_lossy(&output.stderr).contains("maintenance-stderr"));
            }
            _ => {
                const DETACHED_PROCESS: u32 = 0x0000_0008;
                let output = Command::new(std::env::current_exe().unwrap())
                    .args(["--exact", TEST, "--nocapture", "--test-threads=1"])
                    .env(ROLE, "driver")
                    .creation_flags(DETACHED_PROCESS)
                    .output()
                    .expect("start detached maintenance test driver");
                assert!(
                    output.status.success(),
                    "detached driver failed: {}",
                    String::from_utf8_lossy(&output.stderr)
                );
            }
        }
    }
}
