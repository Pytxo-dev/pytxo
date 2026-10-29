//! Thin `git` runner. Every command in a Pytxo-owned worktree runs with hooks
//! pointed at a path that holds none, so a branch cannot execute code during
//! checkout or merge; checks are the only code the gate runs.

use std::io::Read as _;
use std::path::{Path, PathBuf};
use std::process::{Child, Command, ExitStatus, Output, Stdio};
use std::time::Duration;

use anyhow::{anyhow, Context, Result};

#[derive(Clone, Debug)]
pub struct Git {
    cwd: PathBuf,
    /// Extra `-c key=value` pairs prepended to every command.
    config: Vec<String>,
}

impl Git {
    /// For the user's own repository: their config and hooks apply.
    pub fn user(cwd: &Path) -> Self {
        Self {
            cwd: cwd.to_path_buf(),
            config: Vec::new(),
        }
    }

    /// For a Pytxo-owned worktree: hooks off, no signing prompts, no auto-gc.
    pub fn owned(cwd: &Path, identity: Option<(&str, &str)>) -> Self {
        // Any local user can create a directory in a Unix temp dir, and plant
        // hooks in it; nothing can sit under /dev/null.
        let no_hooks = if cfg!(unix) {
            PathBuf::from("/dev/null")
        } else {
            std::env::temp_dir().join("pytxo-land-no-hooks-dir-does-not-exist")
        };
        let mut config = vec![
            format!("core.hooksPath={}", no_hooks.display()),
            "commit.gpgsign=false".into(),
            "gc.auto=0".into(),
            "advice.detachedHead=false".into(),
            "merge.autoStash=false".into(),
            // Submodules stay out, even where the user set submodule.recurse:
            // checkout and reset would look for theirs in the worktree.
            "submodule.recurse=false".into(),
        ];
        if cfg!(windows) {
            // Temp worktrees sit deep under %TEMP%; let checkouts pass 260 chars.
            config.push("core.longpaths=true".into());
        }
        if let Some((name, email)) = identity {
            config.push(format!("user.name={name}"));
            config.push(format!("user.email={email}"));
        }
        Self {
            cwd: cwd.to_path_buf(),
            config,
        }
    }

    pub fn cwd(&self) -> &Path {
        &self.cwd
    }

    fn command(&self, args: &[&str]) -> Command {
        let mut cmd = pytxo_core::background_command("git");
        // Paths as they are, not `"caf\303\251.txt"`.
        cmd.arg("-c").arg("core.quotePath=false");
        for pair in &self.config {
            cmd.arg("-c").arg(pair);
        }
        cmd.args(args)
            .current_dir(&self.cwd)
            .env("GIT_TERMINAL_PROMPT", "0")
            .env("LC_ALL", "C");
        cmd
    }

    pub fn output(&self, args: &[&str]) -> Result<Output> {
        self.command(args)
            .output()
            .with_context(|| format!("running git {}", args.join(" ")))
    }

    /// `run` for a long command an interrupt must stop before it clears up,
    /// such as the checkout of a big repository: recorded with the checks,
    /// and on Unix leading a process group, like them. On Windows Ctrl+C does
    /// not reach git, which would go on writing the worktree being removed.
    pub(crate) fn run_stoppable(&self, args: &[&str]) -> Result<()> {
        let what = || format!("git {}", args.join(" "));
        let mut cmd = self.command(args);
        cmd.stdin(Stdio::null())
            .stdout(Stdio::null())
            .stderr(Stdio::piped());
        let mut child = spawn_recorded(&mut cmd, &what)?;
        // Read meanwhile: git stops on a full pipe.
        let mut stderr = child.stderr.take();
        let errors = std::thread::spawn(move || {
            let mut bytes = Vec::new();
            if let Some(pipe) = stderr.as_mut() {
                let _ = pipe.read_to_end(&mut bytes);
            }
            String::from_utf8_lossy(&bytes).into_owned()
        });
        let status = reap_recorded(&mut child, Duration::from_millis(40), &what)?;
        if !status.success() {
            let errors = errors.join().unwrap_or_default();
            return Err(anyhow!("{} failed: {}", what(), errors.trim()));
        }
        Ok(())
    }

    /// `output` for each of `calls`, which an interrupt stops: recorded and
    /// reaped like `run_stoppable`, for reads that can take long, such as
    /// `git grep` over a big tree. Only stdout is kept. Eight run at a time,
    /// each started before the first is read, so a few short calls take
    /// about as long as one, and each is read to its end before its wait,
    /// which is then over at once. The outputs come back in order.
    pub(crate) fn outputs_stoppable(&self, calls: &[Vec<String>]) -> Result<Vec<Output>> {
        let mut outputs = Vec::with_capacity(calls.len());
        // ponytail: eight at a time, each eight waiting for its slowest; a
        // pool if one slow call ever holds up quick ones.
        for batch in calls.chunks(8) {
            let mut started = Vec::new();
            let mut failed = None;
            for args in batch {
                let args: Vec<&str> = args.iter().map(String::as_str).collect();
                let mut cmd = self.command(&args);
                cmd.stdin(Stdio::null())
                    .stdout(Stdio::piped())
                    .stderr(Stdio::null());
                match spawn_recorded(&mut cmd, &|| format!("git {}", args.join(" "))) {
                    Ok(child) => started.push(child),
                    Err(e) => {
                        failed = Some(e);
                        break;
                    }
                }
            }
            // Every call that started is reaped, even after one did not.
            for (mut child, args) in started.into_iter().zip(batch) {
                let what = || format!("git {}", args.join(" "));
                let mut stdout = Vec::new();
                let read = child.stdout.take().map(|mut p| p.read_to_end(&mut stdout));
                if let Some(Err(_)) = read {
                    // Nothing reads the rest, which git would wait to write.
                    let _ = child.kill();
                }
                let status = reap_recorded(&mut child, Duration::from_millis(1), &what);
                match (status, read) {
                    (Err(e), _) => failed = failed.or(Some(e)),
                    (_, Some(Err(e))) => {
                        failed = failed.or(Some(anyhow!(e).context(format!("reading {}", what()))));
                    }
                    (Ok(status), _) => outputs.push(Output {
                        status,
                        stdout,
                        stderr: Vec::new(),
                    }),
                }
            }
            if let Some(e) = failed {
                return Err(e);
            }
        }
        Ok(outputs)
    }

    /// Trimmed stdout; error carries stderr.
    pub fn run(&self, args: &[&str]) -> Result<String> {
        let out = self.output(args)?;
        if !out.status.success() {
            return Err(anyhow!(
                "git {} failed: {}",
                args.join(" "),
                String::from_utf8_lossy(&out.stderr).trim()
            ));
        }
        Ok(String::from_utf8_lossy(&out.stdout).trim().to_string())
    }

    pub fn ok(&self, args: &[&str]) -> bool {
        self.output(args)
            .map(|o| o.status.success())
            .unwrap_or(false)
    }

    pub fn lines(&self, args: &[&str]) -> Result<Vec<String>> {
        Ok(self
            .run(args)?
            .lines()
            .map(str::trim)
            .filter(|l| !l.is_empty())
            .map(str::to_string)
            .collect())
    }

    pub fn commit_of(&self, rev: &str) -> Result<String> {
        self.run(&[
            "rev-parse",
            "--verify",
            "--quiet",
            &format!("{rev}^{{commit}}"),
        ])
        .map_err(|_| anyhow!("not a commit: {rev}"))
    }

    /// File content at `rev:path`, `None` when the path does not exist there.
    pub fn show(&self, rev: &str, path: &str) -> Option<String> {
        let out = self.output(&["show", &format!("{rev}:{path}")]).ok()?;
        out.status
            .success()
            .then(|| String::from_utf8_lossy(&out.stdout).into_owned())
    }
}

/// Starts `cmd` and records it with the checks, under one lock as they are,
/// so an interrupt stops it too. On Unix it leads a process group, like them.
fn spawn_recorded(cmd: &mut Command, what: &dyn Fn() -> String) -> Result<Child> {
    #[cfg(unix)]
    std::os::unix::process::CommandExt::process_group(cmd, 0);
    let mut active = crate::active();
    let child = cmd.spawn().with_context(|| format!("running {}", what()))?;
    active.checks.push(child.id());
    Ok(child)
}

/// Waits for `child`, looking every `pause`, and forgets it under the lock,
/// as checks are reaped: an interrupt never signals a pid the system may
/// have given another process since.
fn reap_recorded(
    child: &mut Child,
    pause: Duration,
    what: &dyn Fn() -> String,
) -> Result<ExitStatus> {
    let pid = child.id();
    loop {
        let mut active = crate::active();
        if let Some(status) = child.try_wait().transpose() {
            active.checks.retain(|&p| p != pid);
            return status.with_context(|| format!("running {}", what()));
        }
        drop(active);
        std::thread::sleep(pause);
    }
}
