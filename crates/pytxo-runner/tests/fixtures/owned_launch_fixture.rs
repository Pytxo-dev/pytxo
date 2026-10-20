//! Payload for the owned-launch integration tests. Cargo builds it, so tests no
//! longer compile a C# fixture into %TEMP% at run time; Bitdefender's behaviour
//! engine treated that self-respawning temp executable as malicious and removed
//! the whole process tree, Cargo included.
use base64::Engine;
use std::fs;
use std::io::Read;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::thread::sleep;
use std::time::Duration;

fn main() {
    let output = PathBuf::from(std::env::var_os("TEST_OUTPUT").expect("TEST_OUTPUT"));
    let args: Vec<String> = std::env::args().skip(1).collect();
    if let [flag] = args.as_slice() {
        if flag == "--child" || flag == "--child-gated" {
            child(&output, flag == "--child-gated");
        }
    }
    match std::env::var("TEST_MODE").unwrap_or_default().as_str() {
        "embedded" => embedded(&output, &args),
        "echo" => {
            let mut lines = args
                .iter()
                .map(|arg| encode(arg.as_bytes()))
                .collect::<Vec<_>>();
            lines.push(encode(&stdin()));
            fs::write(&output, lines.join("\r\n") + "\r\n").unwrap();
            println!("fixture complete");
        }
        mode => {
            fs::write(&output, std::process::id().to_string()).unwrap();
            match mode {
                "child-gated" => {
                    wait_for(&with_suffix(&output, ".go"));
                    respawn("--child-gated", &output);
                }
                "child" => respawn("--child", &output),
                "gated-late" => {
                    fs::write(with_suffix(&output, ".ready"), "ready").unwrap();
                    wait_for(&with_suffix(&output, ".go"));
                    fs::write(
                        with_suffix(&output, ".late"),
                        "unexpected gated payload effect",
                    )
                    .unwrap();
                }
                "delayed" => {
                    sleep(Duration::from_secs(1));
                    fs::write(
                        with_suffix(&output, ".late"),
                        "unexpected delayed payload effect",
                    )
                    .unwrap();
                }
                _ => {}
            }
            if !matches!(mode, "child" | "child-gated") {
                sleep(Duration::from_secs(30));
            }
        }
    }
}

fn child(output: &Path, gated: bool) -> ! {
    fs::write(
        with_suffix(output, ".child"),
        std::process::id().to_string(),
    )
    .unwrap();
    println!("child holds stdout");
    if gated {
        wait_for(&with_suffix(output, ".late-go"));
    } else {
        sleep(Duration::from_secs(1));
    }
    fs::write(
        with_suffix(output, ".late"),
        "unexpected delayed child effect",
    )
    .unwrap();
    sleep(Duration::from_secs(29));
    std::process::exit(0);
}

/// The embedded-host payload: argument mode is `args[0]`, not `TEST_MODE`.
fn embedded(output: &Path, args: &[String]) {
    if args.first().map(String::as_str) == Some("wait") {
        fs::write(with_suffix(output, ".started"), "started").unwrap();
        sleep(Duration::from_secs(30));
        fs::write(with_suffix(output, ".late"), "escaped job").unwrap();
        return;
    }
    // Rebuilt from argv: the test only checks the prompt never reached it.
    let command_line = std::env::args().collect::<Vec<_>>().join(" ");
    fs::write(with_suffix(output, ".cmdline"), command_line).unwrap();
    let argv = args
        .iter()
        .map(|arg| format!("{arg}\r\n"))
        .collect::<String>();
    fs::write(with_suffix(output, ".argv"), argv).unwrap();
    fs::write(with_suffix(output, ".stdin"), stdin()).unwrap();
    println!("fake payload complete");
}

/// Starts this same image again, inheriting stdout like the root payload.
/// The child is never waited on: tests prove the Job still owns it after the
/// root exits.
#[allow(clippy::zombie_processes)]
fn respawn(flag: &str, output: &Path) {
    let mut command = Command::new(std::env::current_exe().unwrap());
    command.arg(flag);
    #[cfg(windows)]
    std::os::windows::process::CommandExt::creation_flags(&mut command, 0x0800_0000);
    command.spawn().unwrap();
    wait_for(&with_suffix(output, ".child"));
}

fn stdin() -> Vec<u8> {
    let mut bytes = Vec::new();
    std::io::stdin().read_to_end(&mut bytes).unwrap();
    bytes
}

fn encode(bytes: &[u8]) -> String {
    base64::engine::general_purpose::STANDARD.encode(bytes)
}

fn with_suffix(path: &Path, suffix: &str) -> PathBuf {
    let mut joined = path.as_os_str().to_owned();
    joined.push(suffix);
    PathBuf::from(joined)
}

fn wait_for(path: &Path) {
    while !path.exists() {
        sleep(Duration::from_millis(10));
    }
}
