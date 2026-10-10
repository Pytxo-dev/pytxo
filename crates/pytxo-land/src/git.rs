//! Thin `git` runner. Every command in a Pytxo-owned worktree runs with hooks
//! pointed at a directory that does not exist, so a branch cannot execute
//! code during checkout or merge; checks are the only code the gate runs.

use std::path::{Path, PathBuf};
use std::process::Output;

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
        let no_hooks = std::env::temp_dir().join("pytxo-land-no-hooks-dir-does-not-exist");
        let mut config = vec![
            format!("core.hooksPath={}", no_hooks.display()),
            "commit.gpgsign=false".into(),
            "gc.auto=0".into(),
            "advice.detachedHead=false".into(),
            "merge.autoStash=false".into(),
        ];
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

    pub fn output(&self, args: &[&str]) -> Result<Output> {
        let mut cmd = pytxo_core::background_command("git");
        for pair in &self.config {
            cmd.arg("-c").arg(pair);
        }
        cmd.args(args)
            .current_dir(&self.cwd)
            .env("GIT_TERMINAL_PROMPT", "0")
            .env("LC_ALL", "C");
        cmd.output()
            .with_context(|| format!("running git {}", args.join(" ")))
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
