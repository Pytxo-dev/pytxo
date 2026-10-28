//! Run one check command in a worktree with a timeout, keeping the output tail.

use std::collections::BTreeSet;
use std::fs::File;
use std::io::Read;
use std::path::Path;
use std::process::Stdio;
use std::time::{Duration, Instant};

use pytxo_core::ReceiptCheck;
use serde_json::Value;

const TAIL_LINES: usize = 40;

pub fn run_check(scope: &str, command: &str, cwd: &Path, timeout: Duration) -> ReceiptCheck {
    let started = Instant::now();
    let finish = |exit_code: Option<i32>, timed_out: bool, tail: Vec<String>| ReceiptCheck {
        scope: scope.to_string(),
        command: command.to_string(),
        exit_code,
        passed: exit_code == Some(0) && !timed_out,
        duration_ms: started.elapsed().as_millis() as u64,
        timed_out,
        output_tail: tail,
        rerun: false,
    };

    let log_path =
        std::env::temp_dir().join(format!("pytxo-check-{}.log", uuid::Uuid::new_v4().simple()));
    let spawned = File::create(&log_path).and_then(|log| {
        let err = log.try_clone()?;
        shell(command)
            .current_dir(cwd)
            .stdin(Stdio::null())
            .stdout(Stdio::from(log))
            .stderr(Stdio::from(err))
            .spawn()
    });
    let mut child = match spawned {
        Ok(child) => child,
        Err(error) => {
            let _ = std::fs::remove_file(&log_path);
            return finish(None, false, vec![format!("could not start check: {error}")]);
        }
    };

    let (exit_code, timed_out) = loop {
        match child.try_wait() {
            Ok(Some(status)) => break (status.code(), false),
            Ok(None) if started.elapsed() >= timeout => {
                kill_tree(&mut child);
                break (None, true);
            }
            Ok(None) => std::thread::sleep(Duration::from_millis(40)),
            Err(_) => break (None, false),
        }
    };
    let tail = read_tail(&log_path);
    let _ = std::fs::remove_file(&log_path);
    finish(exit_code, timed_out, tail)
}

/// A check command for the project in the working tree at `root`.
pub fn suggest_check(root: &Path) -> Option<String> {
    let names = std::fs::read_dir(root)
        .map(|dir| {
            dir.filter_map(|e| e.ok()?.file_name().into_string().ok())
                .collect()
        })
        .unwrap_or_default();
    detect_check(&names, |file| std::fs::read_to_string(root.join(file)).ok()).map(|(c, _)| c)
}

/// The test command for a project, decided by the files at its root, and the
/// file that decided it. `names` lists the root entries and `read` returns a
/// root file's text. Checks run in a fresh worktree, so commands that need
/// installed dependencies install them first.
pub fn detect_check(
    names: &BTreeSet<String>,
    read: impl Fn(&str) -> Option<String>,
) -> Option<(String, String)> {
    let has = |file: &str| names.contains(file);
    let found = |command: &str, file: &str| Some((command.to_string(), file.to_string()));

    if let Some(pkg) = read("package.json").and_then(|t| serde_json::from_str::<Value>(&t).ok()) {
        let test = pkg["scripts"]["test"].as_str();
        if test.is_some_and(|t| !t.contains("no test specified")) {
            let (install, run) = if has("pnpm-lock.yaml") {
                ("pnpm install --frozen-lockfile", "pnpm test")
            } else if has("yarn.lock") {
                ("yarn install", "yarn test")
            } else if has("bun.lock") || has("bun.lockb") {
                ("bun install", "bun run test")
            } else if has("package-lock.json") || has("npm-shrinkwrap.json") {
                ("npm ci", "npm test")
            } else {
                ("npm install", "npm test")
            };
            // Nothing to install means nothing to wait for.
            let needs_install = pkg.get("workspaces").is_some()
                || ["dependencies", "devDependencies", "optionalDependencies"]
                    .iter()
                    .any(|k| pkg[k].as_object().is_some_and(|deps| !deps.is_empty()));
            let command = if needs_install {
                format!("{install} && {run}")
            } else {
                run.to_string()
            };
            return Some((command, "package.json".into()));
        }
    }
    if has("Cargo.toml") {
        return found("cargo test", "Cargo.toml");
    }
    if has("go.mod") {
        return found("go test ./...", "go.mod");
    }
    if has("uv.lock") {
        return found("uv run pytest", "uv.lock");
    }
    if has("poetry.lock") {
        return found("poetry install && poetry run pytest", "poetry.lock");
    }
    if let Some(file) = ["pyproject.toml", "pytest.ini", "setup.py", "setup.cfg", "tox.ini"]
        .into_iter()
        .find(|f| has(f))
    {
        return found("python -m pytest", file);
    }
    if let Some(file) = ["deno.json", "deno.jsonc"].into_iter().find(|f| has(f)) {
        return found("deno test", file);
    }
    if has("gradlew") {
        let gradle = if cfg!(windows) { "gradlew test" } else { "./gradlew test" };
        return found(gradle, "gradlew");
    }
    if has("pom.xml") {
        return found("mvn -q test", "pom.xml");
    }
    if let Some(file) = names.iter().find(|n| {
        [".sln", ".csproj", ".fsproj"]
            .iter()
            .any(|ext| n.ends_with(ext))
    }) {
        return found("dotnet test", file);
    }
    if has("mix.exs") {
        return found("mix test", "mix.exs");
    }
    if has("Gemfile") && has(".rspec") {
        return found("bundle exec rspec", ".rspec");
    }
    if has("Gemfile") && has("Rakefile") {
        return found("bundle exec rake test", "Rakefile");
    }
    if let Some(composer) = read("composer.json").and_then(|t| serde_json::from_str::<Value>(&t).ok()) {
        if composer["scripts"]["test"].is_string() {
            return found("composer install && composer test", "composer.json");
        }
    }
    for file in ["Makefile", "makefile", "GNUmakefile"] {
        if read(file).is_some_and(|m| m.lines().any(|l| l.starts_with("test:"))) {
            return found("make test", file);
        }
    }
    None
}

/// Whether `command` installs dependencies before testing.
pub fn installs(command: &str) -> bool {
    let padded = format!("{command} ");
    [
        "npm install",
        "npm i ",
        "npm ci",
        "yarn install",
        "bun install",
        "poetry install",
        "uv run",
        "uv sync",
        "composer install",
    ]
    .iter()
    .any(|step| padded.contains(step))
}

fn shell(command: &str) -> std::process::Command {
    #[cfg(windows)]
    {
        use std::os::windows::process::CommandExt;
        let mut cmd = pytxo_core::background_command("cmd");
        cmd.arg("/d")
            .arg("/s")
            .arg("/c")
            .raw_arg(format!("\"{command}\""));
        cmd
    }
    #[cfg(not(windows))]
    {
        let mut cmd = pytxo_core::background_command("sh");
        cmd.arg("-c").arg(command);
        cmd
    }
}

fn kill_tree(child: &mut std::process::Child) {
    #[cfg(windows)]
    {
        let _ = pytxo_core::background_command("taskkill")
            .args(["/T", "/F", "/PID", &child.id().to_string()])
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .status();
    }
    let _ = child.kill();
    let _ = child.wait();
}

fn read_tail(path: &Path) -> Vec<String> {
    let mut bytes = Vec::new();
    if File::open(path)
        .and_then(|mut f| f.read_to_end(&mut bytes))
        .is_err()
    {
        return Vec::new();
    }
    let text = String::from_utf8_lossy(&bytes);
    let lines: Vec<&str> = text.lines().filter(|l| !l.trim().is_empty()).collect();
    lines[lines.len().saturating_sub(TAIL_LINES)..]
        .iter()
        .map(|l| l.chars().take(400).collect())
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn passes_fails_and_times_out() {
        let dir = std::env::temp_dir();
        let ok = run_check("combined", "echo hello", &dir, Duration::from_secs(30));
        assert!(ok.passed, "{ok:?}");
        assert_eq!(ok.output_tail, ["hello"]);

        let fail = run_check("combined", "exit 3", &dir, Duration::from_secs(30));
        assert!(!fail.passed);
        assert_eq!(fail.exit_code, Some(3));

        #[cfg(windows)]
        let slow = "ping -n 30 127.0.0.1 > nul";
        #[cfg(not(windows))]
        let slow = "sleep 30";
        let timeout = run_check("combined", slow, &dir, Duration::from_millis(300));
        assert!(timeout.timed_out && !timeout.passed, "{timeout:?}");
    }

    /// Test runners on Windows print `√` as UTF-8; the receipt must keep it,
    /// not the cp1252 reading `âˆš`.
    #[test]
    fn keeps_utf8_check_output() {
        let dir = tempfile::tempdir().unwrap();
        std::fs::write(dir.path().join("out.txt"), "    √ should be equal\n").unwrap();
        #[cfg(windows)]
        let print = "type out.txt";
        #[cfg(not(windows))]
        let print = "cat out.txt";
        let check = run_check("combined", print, dir.path(), Duration::from_secs(30));
        assert_eq!(check.output_tail, ["    √ should be equal"]);
    }

    #[test]
    fn suggests_a_check_from_the_project_files() {
        let dir = tempfile::tempdir().unwrap();
        let d = dir.path();
        let suggest = || suggest_check(d);
        assert_eq!(suggest(), None);

        std::fs::write(d.join("pyproject.toml"), "").unwrap();
        assert_eq!(suggest().as_deref(), Some("python -m pytest"));
        std::fs::write(d.join("Cargo.toml"), "").unwrap();
        assert_eq!(suggest().as_deref(), Some("cargo test"));

        // npm's placeholder test script is not a test suite.
        let placeholder = r#"{"scripts":{"test":"echo \"Error: no test specified\" && exit 1"}}"#;
        std::fs::write(d.join("package.json"), placeholder).unwrap();
        assert_eq!(suggest().as_deref(), Some("cargo test"));

        // A test script with nothing to install runs as it is.
        std::fs::write(
            d.join("package.json"),
            r#"{"scripts":{"test":"node run-tests.js"}}"#,
        )
        .unwrap();
        assert_eq!(suggest().as_deref(), Some("npm test"));
        std::fs::write(
            d.join("package.json"),
            r#"{"scripts":{"test":"tape test/*.js"},"devDependencies":{"tape":"^5"}}"#,
        )
        .unwrap();
        assert_eq!(suggest().as_deref(), Some("npm install && npm test"));
        std::fs::write(d.join("package-lock.json"), "{}").unwrap();
        assert_eq!(suggest().as_deref(), Some("npm ci && npm test"));
        std::fs::write(d.join("pnpm-lock.yaml"), "").unwrap();
        assert_eq!(
            suggest().as_deref(),
            Some("pnpm install --frozen-lockfile && pnpm test")
        );

        assert!(installs("npm ci && npm test"));
        assert!(installs("pnpm install --frozen-lockfile && pnpm test"));
        assert!(installs("pnpm i && pnpm test"));
        assert!(!installs("npm test"));
        assert!(!installs("echo tape is not installed && exit 1"));
    }

    #[test]
    fn detects_the_test_command_for_each_ecosystem() {
        let detect = |files: &[(&str, &str)]| {
            let names: BTreeSet<String> = files.iter().map(|(n, _)| n.to_string()).collect();
            detect_check(&names, |f| {
                files
                    .iter()
                    .find(|(n, _)| *n == f)
                    .map(|(_, body)| body.to_string())
            })
        };
        let cmd = |files: &[(&str, &str)]| detect(files).map(|(c, _)| c);
        assert_eq!(detect(&[]), None);
        assert_eq!(detect(&[("README.md", "")]), None);

        let pkg = r#"{"scripts":{"test":"vitest run"},"devDependencies":{"vitest":"^3"}}"#;
        assert_eq!(
            detect(&[("package.json", pkg)]),
            Some(("npm install && npm test".into(), "package.json".into()))
        );
        assert_eq!(
            cmd(&[("package.json", pkg), ("yarn.lock", "")]).as_deref(),
            Some("yarn install && yarn test")
        );
        assert_eq!(
            cmd(&[("package.json", pkg), ("bun.lock", "")]).as_deref(),
            Some("bun install && bun run test")
        );
        // A workspace root installs even with no dependencies of its own.
        let ws = r#"{"workspaces":["packages/*"],"scripts":{"test":"turbo test"}}"#;
        assert_eq!(
            cmd(&[("package.json", ws), ("package-lock.json", "")]).as_deref(),
            Some("npm ci && npm test")
        );
        // A package.json without tests falls through to the next ecosystem.
        assert_eq!(
            cmd(&[("package.json", "{}"), ("go.mod", "")]).as_deref(),
            Some("go test ./...")
        );

        assert_eq!(cmd(&[("Cargo.toml", "")]).as_deref(), Some("cargo test"));
        assert_eq!(
            detect(&[("pyproject.toml", ""), ("uv.lock", "")]),
            Some(("uv run pytest".into(), "uv.lock".into()))
        );
        assert_eq!(
            cmd(&[("pyproject.toml", ""), ("poetry.lock", "")]).as_deref(),
            Some("poetry install && poetry run pytest")
        );
        assert_eq!(
            detect(&[("setup.cfg", "")]),
            Some(("python -m pytest".into(), "setup.cfg".into()))
        );
        assert_eq!(cmd(&[("deno.json", "{}")]).as_deref(), Some("deno test"));
        assert_eq!(
            cmd(&[("gradlew", "")]).as_deref(),
            Some(if cfg!(windows) { "gradlew test" } else { "./gradlew test" })
        );
        assert_eq!(cmd(&[("pom.xml", "")]).as_deref(), Some("mvn -q test"));
        assert_eq!(
            detect(&[("App.sln", "")]),
            Some(("dotnet test".into(), "App.sln".into()))
        );
        assert_eq!(cmd(&[("mix.exs", "")]).as_deref(), Some("mix test"));
        assert_eq!(
            cmd(&[("Gemfile", ""), (".rspec", "")]).as_deref(),
            Some("bundle exec rspec")
        );
        assert_eq!(
            cmd(&[("Gemfile", ""), ("Rakefile", "")]).as_deref(),
            Some("bundle exec rake test")
        );
        assert_eq!(
            cmd(&[("composer.json", r#"{"scripts":{"test":"phpunit"}}"#)]).as_deref(),
            Some("composer install && composer test")
        );
        assert_eq!(
            cmd(&[("Makefile", "build:\n\tcc main.c\ntest: build\n\t./run\n")]).as_deref(),
            Some("make test")
        );
        assert_eq!(cmd(&[("Makefile", "build:\n\tcc main.c\n")]), None);
    }
}
