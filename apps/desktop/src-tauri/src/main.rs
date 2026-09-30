#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

#[cfg(windows)]
fn is_attempt_host_request(first_arg: Option<&std::ffi::OsStr>) -> bool {
    first_arg == Some(std::ffi::OsStr::new("pytxo-attempt-host/1"))
}

fn main() {
    #[cfg(windows)]
    if is_attempt_host_request(std::env::args_os().nth(1).as_deref()) {
        // This branch must precede Tauri, single-instance, and UI startup.
        std::process::exit(pytxo_runner::owned_launch::attempt_host_main());
    }
    pytxo_desktop_lib::run();
}

#[cfg(all(test, windows))]
mod tests {
    use super::*;
    use std::ffi::OsStr;

    #[test]
    fn attempt_host_marker_is_exact_and_deep_links_keep_the_normal_entry() {
        assert!(is_attempt_host_request(Some(OsStr::new(
            "pytxo-attempt-host/1"
        ))));
        for arg in [
            "pytxo://auth?code=test",
            "--help",
            "pytxo-attempt-host",
            "PYTXO-ATTEMPT-HOST/1",
        ] {
            assert!(!is_attempt_host_request(Some(OsStr::new(arg))));
        }
        assert!(!is_attempt_host_request(None));
    }
}
