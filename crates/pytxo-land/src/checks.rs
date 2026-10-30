//! Run one check command in a worktree with a timeout, keeping the output tail.

use std::collections::BTreeSet;
use std::ffi::OsString;
use std::fs::{File, OpenOptions};
use std::io::Read;
use std::path::Path;
use std::process::Stdio;
use std::time::{Duration, Instant};

use pytxo_core::ReceiptCheck;
use serde_json::Value;

const TAIL_LINES: usize = 40;
/// Lines kept from above the tail: error lines, each with the line after it.
const FIRST_ERRORS: usize = 10;

/// Runs `command` in `cwd`, its output in a log under `logs`.
pub fn run_check(
    scope: &str,
    command: &str,
    cwd: &Path,
    logs: &Path,
    timeout: Duration,
) -> ReceiptCheck {
    let started = Instant::now();
    let finish = |exit_code: Option<i32>, timed_out: bool, (first_errors, tail)| ReceiptCheck {
        scope: scope.to_string(),
        command: command.to_string(),
        exit_code,
        passed: exit_code == Some(0) && !timed_out,
        duration_ms: started.elapsed().as_millis() as u64,
        timed_out,
        output_tail: tail,
        first_errors,
        rerun: false,
    };

    let log_path = logs.join(format!("pytxo-check-{}.log", uuid::Uuid::new_v4().simple()));
    // Started and recorded under one lock, so an interrupt stops it too.
    let mut active = crate::active();
    let spawned = new_log(&log_path).and_then(|log| {
        let err = log.try_clone()?;
        let mut cmd = shell(command);
        // Nothing to answer a prompt with: git asks for no password, and
        // input reads as empty.
        cmd.current_dir(cwd)
            .env("GIT_TERMINAL_PROMPT", "0")
            .stdin(Stdio::null())
            .stdout(Stdio::from(log))
            .stderr(Stdio::from(err));
        if let Some(path) = python_path(cwd) {
            cmd.env("PYTHONPATH", path);
        }
        cmd.spawn()
    });
    let mut child = match spawned {
        Ok(child) => child,
        Err(error) => {
            let _ = std::fs::remove_file(&log_path);
            let said = vec![format!("could not start check: {error}")];
            return finish(None, false, (Vec::new(), said));
        }
    };
    let pid = child.id();
    active.checks.push(pid);
    active.paths.push(log_path.clone());
    drop(active);

    let (exit_code, timed_out) = loop {
        // Reaped and forgotten under the lock: an interrupt never signals a
        // pid the system may have given another process since.
        let mut active = crate::active();
        let done = match child.try_wait() {
            Ok(Some(status)) => Some((status.code(), false)),
            Ok(None) if started.elapsed() >= timeout => {
                kill_tree(pid);
                let _ = child.kill();
                let _ = child.wait();
                Some((None, true))
            }
            Ok(None) => None,
            Err(_) => Some((None, false)),
        };
        if let Some(done) = done {
            active.checks.retain(|&p| p != pid);
            break done;
        }
        drop(active);
        std::thread::sleep(Duration::from_millis(40));
    };
    let read = read_tail(&log_path);
    let _ = std::fs::remove_file(&log_path);
    crate::active().paths.retain(|p| p != &log_path);
    finish(exit_code, timed_out, read)
}

/// The raw output, credentials and all: on Unix only its owner reads it.
fn new_log(path: &Path) -> std::io::Result<File> {
    let mut options = OpenOptions::new();
    options.write(true).create_new(true);
    #[cfg(unix)]
    std::os::unix::fs::OpenOptionsExt::mode(&mut options, 0o600);
    options.open(path)
}

/// The worktree's own Python code first: with the project installed in
/// editable mode, `import` would otherwise find the user's checkout.
fn python_path(cwd: &Path) -> Option<OsString> {
    let src = cwd.join("src");
    let ours = src.is_dir().then_some(src).into_iter();
    let theirs = std::env::var_os("PYTHONPATH").filter(|p| !p.is_empty());
    let all = ours
        .chain([cwd.to_path_buf()])
        .chain(theirs.iter().flat_map(std::env::split_paths));
    std::env::join_paths(all).ok()
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
    // npm and Composer accept the BOM that Windows PowerShell 5.1 writes.
    let json = |file: &str| {
        read(file)
            .and_then(|t| serde_json::from_str::<Value>(t.trim_start_matches('\u{feff}')).ok())
    };

    if let Some(pkg) = json("package.json") {
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
    if let Some(file) = [
        "pyproject.toml",
        "pytest.ini",
        "setup.py",
        "setup.cfg",
        "tox.ini",
    ]
    .into_iter()
    .find(|f| has(f))
    {
        return found("python -m pytest", file);
    }
    if let Some(file) = ["deno.json", "deno.jsonc"].into_iter().find(|f| has(f)) {
        return found("deno test", file);
    }
    if has("gradlew") {
        let gradle = if cfg!(windows) {
            "gradlew test"
        } else {
            "./gradlew test"
        };
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
    if let Some(composer) = json("composer.json") {
        // A script is a command or a list of them.
        let test = &composer["scripts"]["test"];
        if test.is_string() || test.as_array().is_some_and(|a| !a.is_empty()) {
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
        // A group of its own: `sh -c "npm ci && npm test"` does not exec npm,
        // so stopping only `sh` would leave the test run going. From a
        // terminal that group runs in the background, so a check that opens
        // the terminal itself (an ssh passphrase, sudo) waits until --timeout.
        #[cfg(unix)]
        std::os::unix::process::CommandExt::process_group(&mut cmd, 0);
        cmd
    }
}

/// Stops check `pid` and everything it started.
pub(crate) fn kill_tree(pid: u32) {
    #[cfg(not(unix))]
    let _ = pytxo_core::background_command("taskkill")
        .args(["/T", "/F", "/PID", &pid.to_string()])
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status();
    // It leads its own process group, which `-<pid>` names. Called, not run
    // as `kill`: slim images (`node:22-slim`) have no kill binary.
    // SAFETY: kill(2) only sends a signal.
    #[cfg(unix)]
    unsafe {
        libc::kill(-(pid as libc::pid_t), libc::SIGKILL);
    }
}

/// Whether process `pid` may still run: only a sure answer says no.
pub fn alive(pid: u32) -> bool {
    #[cfg(not(unix))]
    {
        let out = pytxo_core::background_command("tasklist")
            .args(["/FI", &format!("PID eq {pid}"), "/FO", "CSV", "/NH"])
            .output();
        let listed = |o: &std::process::Output| {
            String::from_utf8_lossy(&o.stdout).contains(&format!("\"{pid}\""))
        };
        !out.is_ok_and(|o| o.status.success() && !listed(&o))
    }
    // Signal 0 only asks. EPERM, another user's process, is one too.
    // SAFETY: signal 0 sends nothing.
    #[cfg(unix)]
    {
        let asked = unsafe { libc::kill(pid as libc::pid_t, 0) };
        asked == 0 || std::io::Error::last_os_error().raw_os_error() != Some(libc::ESRCH)
    }
}

/// The output's first error lines above its tail, each with the line after
/// it, then its last lines, as a terminal shows them, credentials masked:
/// receipts, `--json`, hook replies and pull-request comments all show the
/// tail, and the why and the cause read the error lines as well.
fn read_tail(path: &Path) -> (Vec<String>, Vec<String>) {
    let mut bytes = Vec::new();
    if File::open(path)
        .and_then(|mut f| f.read_to_end(&mut bytes))
        .is_err()
    {
        return (Vec::new(), Vec::new());
    }
    let text = mask_private_keys(&String::from_utf8_lossy(&bytes));
    let shown: Vec<String> = text
        .lines()
        .map(as_shown)
        .filter(|l| !l.trim().is_empty())
        .collect();
    let (above, tail) = shown.split_at(shown.len().saturating_sub(TAIL_LINES));
    let kept = |lines: &[String]| -> Vec<String> {
        let cut = |l: &String| redact(l).chars().take(400).collect();
        lines.iter().map(cut).collect()
    };
    let mut first = Vec::new();
    let mut next = 0;
    for (i, line) in above.iter().enumerate() {
        let error = crate::render::rank(line.trim()).is_some_and(|r| r <= 2);
        if i >= next && error && first.len() + 2 <= FIRST_ERRORS {
            next = (i + 2).min(above.len());
            first.extend(kept(&above[i..next]));
        }
    }
    (first, kept(tail))
}

/// `line` as a terminal shows it: the text after its last carriage return,
/// without escape sequences (colors, titles, links) or control characters.
/// In a pull-request comment a carriage return would end a code span.
fn as_shown(line: &str) -> String {
    let line = line.trim_end_matches('\r');
    let line = line.rsplit('\r').next().unwrap_or(line);
    let mut out = String::with_capacity(line.len());
    let mut chars = line.chars();
    while let Some(c) = chars.next() {
        if c != '\u{1b}' {
            if c == '\t' || !c.is_control() {
                out.push(c);
            }
            continue;
        }
        match chars.next() {
            // CSI, `ESC[1;31m`: parameters up to a final byte.
            Some('[') => {
                for c in chars.by_ref() {
                    if ('\u{40}'..='\u{7e}').contains(&c) {
                        break;
                    }
                }
            }
            // OSC, a title or a link: up to BEL or `ESC\`.
            Some(']') => {
                while let Some(c) = chars.next() {
                    match c {
                        '\u{7}' => break,
                        '\u{1b}' => {
                            chars.next();
                            break;
                        }
                        _ => {}
                    }
                }
            }
            // `ESC(B`: intermediates, then a final byte.
            Some(' '..='/') => {
                for c in chars.by_ref() {
                    if !(' '..='/').contains(&c) {
                        break;
                    }
                }
            }
            // `ESC7`, `ESC=` and the like.
            _ => {}
        }
    }
    out
}

const REDACTED: &str = "[REDACTED]";

/// `text` with each PEM private key, its BEGIN line through its END line,
/// as one `[REDACTED]`.
fn mask_private_keys(text: &str) -> String {
    const BEGIN: &str = "-----BEGIN ";
    let mut out = String::with_capacity(text.len());
    let mut rest = text;
    while let Some(at) = rest.find(BEGIN) {
        let header = rest[at..].lines().next().unwrap_or_default();
        let body = at + BEGIN.len();
        out.push_str(&rest[..at]);
        if !header.contains("PRIVATE KEY") {
            out.push_str(BEGIN);
            rest = &rest[body..];
            continue;
        }
        out.push_str(REDACTED);
        // Through the END line's closing dashes, or all the rest.
        let end = rest[body..].find("-----END ").and_then(|e| {
            let label = body + e + "-----END ".len();
            rest[label..]
                .find("-----")
                .map(|d| label + d + "-----".len())
        });
        rest = &rest[end.unwrap_or(rest.len())..];
    }
    out.push_str(rest);
    out
}

/// Whether `key`, in any case and without `-` and `_`, names a credential:
/// `AWS_SECRET_ACCESS_KEY`, `NPM_TOKEN`, `x-api-key`, `privateKey`,
/// `Authorization`... An error's name does not: `JsonWebTokenError: invalid
/// signature`.
fn credential_key(key: &str) -> bool {
    let key = key.to_ascii_lowercase().replace(['-', '_'], "");
    if key.ends_with("error") || key.ends_with("exception") {
        return false;
    }
    [
        "secret",
        "token",
        "password",
        "passwd",
        "apikey",
        "accesskey",
        "privatekey",
        "credential",
        "authorization",
    ]
    .iter()
    .any(|word| key.contains(word))
}

/// Letters, digits, `_` and `-`.
fn key_char(c: char) -> bool {
    c.is_ascii_alphanumeric() || c == '_' || c == '-'
}

fn aws_char(c: char) -> bool {
    c.is_ascii_uppercase() || c.is_ascii_digit()
}

fn npm_char(c: char) -> bool {
    c.is_ascii_alphanumeric()
}

/// A credential's prefix, the least length of the rest and its characters.
type Key = (&'static str, usize, fn(char) -> bool);

/// After pytxo-core's cloud upload patterns: GitHub, OpenAI and Anthropic,
/// Stripe, Slack, Google and AWS keys, and npm tokens.
const KEYS: [Key; 17] = [
    ("ghp_", 20, key_char),
    ("gho_", 20, key_char),
    ("ghu_", 20, key_char),
    ("ghs_", 20, key_char),
    ("ghr_", 20, key_char),
    ("github_pat_", 20, key_char),
    ("sk-", 20, key_char),
    ("sk_live_", 16, key_char),
    ("xoxa-", 16, key_char),
    ("xoxb-", 16, key_char),
    ("xoxp-", 16, key_char),
    ("xoxr-", 16, key_char),
    ("xoxs-", 16, key_char),
    ("AIza", 30, key_char),
    ("AKIA", 16, aws_char),
    ("ASIA", 16, aws_char),
    ("npm_", 36, npm_char),
];

/// `line` with the credentials a failing test may print masked: the `KEYS`,
/// bearer tokens, the password in `scheme://user:password@host`, and the
/// value given a key that names a credential, such as `DB_PASSWORD=...`,
/// `"api_key": "..."` or `password => "..."`. Prefix scans rather than
/// regexes, which would add 1.5 MB to the npm binary.
fn redact(line: &str) -> String {
    // Bytes at the start of `s` that `allowed` accepts.
    let run =
        |s: &str, allowed: &dyn Fn(char) -> bool| s.find(|c: char| !allowed(c)).unwrap_or(s.len());
    let mut out = String::with_capacity(line.len());
    let mut rest = line;
    while let Some(c) = rest.chars().next() {
        if let Some(after) = rest.strip_prefix("://") {
            let authority = &after[..run(after, &|c| !c.is_whitespace() && c != '/')];
            let userinfo = authority.rsplit_once('@').map(|(info, _)| info);
            if let Some((user, password)) = userinfo.and_then(|i| i.split_once(':')) {
                if !password.is_empty() {
                    // The user can be a token too (`https://<token>:x-oauth-basic@`).
                    out.push_str(&format!("://{}:{REDACTED}", redact(user)));
                    rest = &after[user.len() + 1 + password.len()..];
                    continue;
                }
            }
        }
        // Keys start a word, as `\b` would say.
        if !out.ends_with(|p: char| p.is_alphanumeric() || p == '_') {
            if let Some((prefix, least, allowed)) = KEYS.iter().find(|k| rest.starts_with(k.0)) {
                let key = prefix.len() + run(&rest[prefix.len()..], allowed);
                if key - prefix.len() >= *least {
                    out.push_str(REDACTED);
                    rest = &rest[key..];
                    continue;
                }
            }
            if rest
                .get(..6)
                .is_some_and(|w| w.eq_ignore_ascii_case("bearer"))
            {
                let gap = run(&rest[6..], &char::is_whitespace);
                let token = run(&rest[6 + gap..], &|c| {
                    c.is_ascii_alphanumeric() || "._~+/=-".contains(c)
                });
                if gap > 0 && token >= 16 {
                    out.push_str(&rest[..6 + gap]);
                    out.push_str(REDACTED);
                    rest = &rest[6 + gap + token..];
                    continue;
                }
            }
        }
        // A whole key that names a credential, maybe its closing quote, `=`,
        // `:`, `=>` or `:=`, then the value. Not `==`, a comparison, nor
        // `::`, which joins a path such as `auth::token::tests`.
        let key = if out.ends_with(key_char) {
            0
        } else {
            run(rest, &key_char)
        };
        if key > 0 && credential_key(&rest[..key]) {
            let quote = usize::from(rest[key..].starts_with(['"', '\'']));
            let sep = key + quote + run(&rest[key + quote..], &char::is_whitespace);
            let after = &rest[sep..];
            let sep_len = if after.starts_with("=>") || after.starts_with(":=") {
                2
            } else if after.starts_with(['=', ':']) && !after[1..].starts_with(['=', ':']) {
                1
            } else {
                0
            };
            if sep_len > 0 {
                let at = sep + sep_len + run(&rest[sep + sep_len..], &char::is_whitespace);
                let value = &rest[at..];
                let word = |s: &str| run(s, &|c| !c.is_whitespace() && c != ',' && c != ';');
                let first = word(value);
                let len = match value.chars().next() {
                    Some(q @ ('"' | '\'')) => value[1..].find(q).map_or(value.len(), |end| end + 2),
                    // `Bearer <token>`, `Basic <base64>`: the credential follows.
                    _ if ["bearer", "basic"]
                        .iter()
                        .any(|s| value[..first].eq_ignore_ascii_case(s)) =>
                    {
                        let gap = run(&value[first..], &char::is_whitespace);
                        first + gap + word(&value[first + gap..])
                    }
                    _ => first,
                };
                if len > 0 {
                    out.push_str(&rest[..at]);
                    out.push_str(REDACTED);
                    rest = &rest[at + len..];
                    continue;
                }
            }
        }
        out.push(c);
        rest = &rest[c.len_utf8()..];
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn passes_fails_and_times_out() {
        let dir = std::env::temp_dir();
        let ok = run_check(
            "combined",
            "echo hello",
            &dir,
            &dir,
            Duration::from_secs(30),
        );
        assert!(ok.passed, "{ok:?}");
        assert_eq!(ok.output_tail, ["hello"]);

        let fail = run_check("combined", "exit 3", &dir, &dir, Duration::from_secs(30));
        assert!(!fail.passed);
        assert_eq!(fail.exit_code, Some(3));

        #[cfg(windows)]
        let slow = "ping -n 30 127.0.0.1 > nul";
        #[cfg(not(windows))]
        let slow = "sleep 30";
        let timeout = run_check("combined", slow, &dir, &dir, Duration::from_millis(300));
        assert!(timeout.timed_out && !timeout.passed, "{timeout:?}");
    }

    /// A failing test that prints a credential must not publish it: the tail
    /// goes into receipts, hook replies and pull-request comments.
    #[test]
    fn masks_the_credentials_a_test_prints() {
        // Built here, so no scanner takes the source for a leak.
        let gh = format!("ghp_{}", "a1B2".repeat(9));
        let aws = format!("AKIA{}", "Z9".repeat(8));
        let ant = format!("sk-ant-api03-{}", "x_Y-".repeat(6));
        let line = format!(
            "FAIL {gh} key={aws},{ant} db postgres://admin:hunter2@db:5432/prod Authorization: bearer {} √",
            "eyJ.a-b~c".repeat(3)
        );
        assert_eq!(
            redact(&line),
            "FAIL [REDACTED] key=[REDACTED],[REDACTED] db postgres://admin:[REDACTED]@db:5432/prod Authorization: [REDACTED] √"
        );
        let ghs = format!("ghs_{}", "q".repeat(36));
        assert_eq!(
            redact(&format!(
                "https://{ghs}:x-oauth-basic@github.com/o/r and redis://:pw@cache:6379"
            )),
            "https://[REDACTED]:[REDACTED]@github.com/o/r and redis://:[REDACTED]@cache:6379"
        );
        // Whole words only, long enough, and URLs that carry a password.
        for kept in [
            "task-runner-integration-tests-pass",
            "sk-short",
            "ASIA_PACIFIC_SOUTHEAST_REGION",
            &format!("x{gh}"),
            "see https://example.com:8080/a@b:c and http://user@host/",
            "git@github.com:o/r.git, redis://localhost:6379, file:///tmp/x",
            "Bearer short",
        ] {
            assert_eq!(redact(kept), kept);
        }

        // Masked where it is captured.
        let dir = tempfile::tempdir().unwrap();
        std::fs::write(dir.path().join("out.txt"), format!("token {gh}\n")).unwrap();
        #[cfg(windows)]
        let print = "type out.txt";
        #[cfg(not(windows))]
        let print = "cat out.txt";
        let check = run_check(
            "combined",
            print,
            dir.path(),
            dir.path(),
            Duration::from_secs(30),
        );
        assert_eq!(check.output_tail, ["token [REDACTED]"]);
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
        let check = run_check(
            "combined",
            print,
            dir.path(),
            dir.path(),
            Duration::from_secs(30),
        );
        assert_eq!(check.output_tail, ["    √ should be equal"]);
    }

    #[test]
    fn masks_private_keys_and_credential_values() {
        // Built here, so no scanner takes the source for a key.
        let pem = |label: &str| {
            format!("-----BEGIN {label}-----\nb3BlbnNzaC1rZXk\nAAAA\n-----END {label}-----")
        };
        let key = pem("OPENSSH PRIVATE KEY");
        assert_eq!(
            mask_private_keys(&format!("before\n{key}\nafter")),
            "before\n[REDACTED]\nafter"
        );
        // Cut off, it still goes; a certificate is public.
        let cut = &key[..key.find("AAAA").unwrap()];
        assert_eq!(mask_private_keys(&format!("x {cut}")), "x [REDACTED]");
        let cert = pem("CERTIFICATE");
        assert_eq!(mask_private_keys(&cert), cert);

        for (line, masked) in [
            ("DB_PASSWORD=hunter2 next", "DB_PASSWORD=[REDACTED] next"),
            (
                r#"{"api_key": "abc 123", "user": "x"}"#,
                r#"{"api_key": [REDACTED], "user": "x"}"#,
            ),
            ("Secret-Key : s3cr3t;", "Secret-Key : [REDACTED];"),
            ("export AUTH_TOKEN='t0k'", "export AUTH_TOKEN=[REDACTED]"),
            ("password: p@ss, user: me", "password: [REDACTED], user: me"),
            ("mypassword=x", "mypassword=[REDACTED]"),
            ("token := abc", "token := [REDACTED]"),
        ] {
            assert_eq!(redact(line), masked);
        }
        // Keys that only contain the word, Ruby's `=>`, and npm tokens. Built
        // here, so no scanner takes the source for a leak.
        let npm = format!("npm_{}", "abcdefghijklmnopqrstuvwxyz0123456789");
        for (line, masked) in [
            (
                format!(
                    "AWS_SECRET_ACCESS_KEY=wJalrXUtnFEMIK7MDENG{}",
                    "bPxRfiCYEXAMPLEKEY"
                ),
                "AWS_SECRET_ACCESS_KEY=[REDACTED]",
            ),
            (format!("NPM_TOKEN={npm}"), "NPM_TOKEN=[REDACTED]"),
            (
                format!("SECRET_TOKEN={}", "plainsecretvalue"),
                "SECRET_TOKEN=[REDACTED]",
            ),
            (
                format!("password => \"{}\"", "rocket"),
                "password => [REDACTED]",
            ),
            (format!("registry token {npm}"), "registry token [REDACTED]"),
            (
                format!("x-access-token: Bearer {}", "eyJ.a-b~c".repeat(3)),
                "x-access-token: [REDACTED]",
            ),
            (
                format!("X-Auth-Token: Basic {}", "dXNlcjpwYXNzd29yZA=="),
                "X-Auth-Token: [REDACTED]",
            ),
            // camelCase names, and the header itself.
            (
                format!("wallet privateKey: 0x{}", "4c0883a6".repeat(8)),
                "wallet privateKey: [REDACTED]",
            ),
            (
                format!("s3 accessKey={}", "AbCdEfGh12345678wxyz"),
                "s3 accessKey=[REDACTED]",
            ),
            (
                format!("failed Authorization: Basic {}", "dXNlcjpwYXNzd29yZA=="),
                "failed Authorization: [REDACTED]",
            ),
        ] {
            assert_eq!(redact(&line), masked);
        }
        for kept in [
            "passwordless login works",
            "expected password",
            "test result: FAILED. 3 passed; 1 failed; 0 ignored",
            "expected: 3, got: 4 at src/token.rs:12:5",
            "test auth::token::tests::refresh ... FAILED",
            "assertion failed: token == expected",
            "npm_config_user_agent=npm/10.8.2 node/v22.11.0",
            // An error's name, though it holds a credential word.
            "JsonWebTokenError: invalid signature",
            "TokenExpiredError: jwt expired",
            "CredentialsProviderError: Could not load credentials from any providers",
            "org.example.InvalidTokenException: token is invalid",
            "authorization failed: user is not allowed",
            "AuthorizationError: denied",
        ] {
            assert_eq!(redact(kept), kept);
        }

        // Masked where it is captured, across lines.
        let dir = tempfile::tempdir().unwrap();
        std::fs::write(
            dir.path().join("out.txt"),
            format!("before\n{key}\nafter\n"),
        )
        .unwrap();
        #[cfg(windows)]
        let print = "type out.txt";
        #[cfg(not(windows))]
        let print = "cat out.txt";
        let check = run_check(
            "combined",
            print,
            dir.path(),
            dir.path(),
            Duration::from_secs(30),
        );
        assert_eq!(check.output_tail, ["before", "[REDACTED]", "after"]);
    }

    /// A long run's error, far above its last 40 lines, is kept apart with
    /// the line after it, such as rustc's place, and the tail stays the last
    /// 40 lines. A short run's error is in its tail and kept once.
    #[test]
    fn an_error_above_the_tail_is_kept_with_its_place() {
        let dir = tempfile::tempdir().unwrap();
        let d = dir.path();
        let packages: String = (1..=45)
            .map(|n| format!("ok   shop/pkg{n} 0.1s\n"))
            .collect();
        let error =
            "error[E0432]: unresolved import `crate::cart::price`\n --> src/discount.rs:1:5\n  |\n";
        std::fs::write(d.join("long.txt"), format!("{error}{packages}FAIL\n")).unwrap();
        std::fs::write(d.join("short.txt"), format!("{error}FAIL\n")).unwrap();
        #[cfg(windows)]
        let print = |f: &str| format!("type {f}");
        #[cfg(not(windows))]
        let print = |f: &str| format!("cat {f}");
        let run = |f: &str| run_check("combined", &print(f), d, d, Duration::from_secs(30));
        let long = run("long.txt");
        assert_eq!(
            long.first_errors,
            [
                "error[E0432]: unresolved import `crate::cart::price`",
                " --> src/discount.rs:1:5"
            ]
        );
        assert_eq!(long.output_tail.len(), 40);
        assert_eq!(long.output_tail.last().map(String::as_str), Some("FAIL"));
        let short = run("short.txt");
        assert!(short.first_errors.is_empty(), "{short:?}");
        assert_eq!(short.output_tail.len(), 4);
    }

    /// A check never waits on a prompt nobody can answer.
    #[test]
    fn git_asks_a_check_for_no_password() {
        let dir = tempfile::tempdir().unwrap();
        #[cfg(windows)]
        let print = "echo %GIT_TERMINAL_PROMPT%";
        #[cfg(not(windows))]
        let print = "echo \"$GIT_TERMINAL_PROMPT\"";
        let check = run_check(
            "combined",
            print,
            dir.path(),
            dir.path(),
            Duration::from_secs(30),
        );
        assert_eq!(check.output_tail, ["0"]);
    }

    #[test]
    fn shows_lines_as_a_terminal_would() {
        assert_eq!(as_shown("\u{1b}[1;31mFAIL\u{1b}[0m x\t1"), "FAIL x\t1");
        assert_eq!(as_shown("50%\r100%\r"), "100%");
        assert_eq!(
            as_shown("\u{1b}]0;title\u{7}ok \u{1b}]8;;https://e.x\u{1b}\\link\u{1b}]8;;\u{1b}\\"),
            "ok link"
        );
        assert_eq!(as_shown("\u{1b}(Bplain\u{8}\u{7}"), "plain");
    }

    /// Python imports the worktree's code before an installed copy of it.
    #[test]
    fn python_finds_the_worktrees_code_first() {
        let dir = tempfile::tempdir().unwrap();
        let d = dir.path();
        #[cfg(windows)]
        let print = "echo %PYTHONPATH%";
        #[cfg(not(windows))]
        let print = "echo \"$PYTHONPATH\"";
        let shown = || {
            run_check("combined", print, d, d, Duration::from_secs(30))
                .output_tail
                .concat()
        };
        let root = d.display().to_string();
        assert!(shown().starts_with(&root), "{}", shown());
        std::fs::create_dir(d.join("src")).unwrap();
        let sep = if cfg!(windows) { ';' } else { ':' };
        let both = format!("{}{sep}{root}", d.join("src").display());
        assert!(shown().starts_with(&both), "{}", shown());
    }

    /// A timeout stops what the check started, not only its shell.
    #[cfg(unix)]
    #[test]
    fn a_timeout_stops_the_whole_check() {
        let dir = tempfile::tempdir().unwrap();
        let d = dir.path();
        let check = run_check(
            "combined",
            "(sleep 2; touch late) & sleep 30",
            d,
            d,
            Duration::from_millis(300),
        );
        assert!(check.timed_out);
        std::thread::sleep(Duration::from_secs(3));
        assert!(!d.join("late").exists());
    }

    #[cfg(unix)]
    #[test]
    fn logs_are_the_users_alone() {
        use std::os::unix::fs::PermissionsExt;
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("check.log");
        new_log(&path).unwrap();
        let mode = std::fs::metadata(&path).unwrap().permissions().mode();
        assert_eq!(mode & 0o777, 0o600);
        // Never a file that is there already, which anyone could have made.
        assert!(new_log(&path).is_err());
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
            Some(if cfg!(windows) {
                "gradlew test"
            } else {
                "./gradlew test"
            })
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
            cmd(&[(
                "composer.json",
                r#"{"scripts":{"test":["@phpunit"],"phpunit":"phpunit"}}"#
            )])
            .as_deref(),
            Some("composer install && composer test")
        );
        assert_eq!(
            cmd(&[("composer.json", r#"{"scripts":{"test":[]}}"#)]),
            None
        );
        // With the BOM Windows PowerShell 5.1 writes.
        let bom = |json: &str| format!("\u{feff}{json}");
        assert_eq!(
            cmd(&[("package.json", bom(pkg).as_str())]).as_deref(),
            Some("npm install && npm test")
        );
        assert_eq!(
            cmd(&[(
                "composer.json",
                bom(r#"{"scripts":{"test":"phpunit"}}"#).as_str()
            )])
            .as_deref(),
            Some("composer install && composer test")
        );
        assert_eq!(
            cmd(&[("Makefile", "build:\n\tcc main.c\ntest: build\n\t./run\n")]).as_deref(),
            Some("make test")
        );
        assert_eq!(cmd(&[("Makefile", "build:\n\tcc main.c\n")]), None);
    }
}
