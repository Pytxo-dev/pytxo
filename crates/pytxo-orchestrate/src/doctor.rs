use std::path::Path;
use std::process::Command;

use pytxo_core::PytxoConfig;
use serde::Serialize;

#[derive(Clone, Debug, Serialize)]
pub struct DoctorCheck {
    pub name: String,
    pub ok: bool,
    pub detail: String,
}

#[derive(Clone, Debug, Serialize)]
pub struct DoctorReport {
    pub checks: Vec<DoctorCheck>,
}

impl DoctorReport {
    pub fn all_ok(&self) -> bool {
        self.checks.iter().all(|c| c.ok)
    }
}

pub fn run_doctor(repo: Option<&Path>) -> anyhow::Result<DoctorReport> {
    let repo_root = super::resolve_repo_root(repo)?;
    let checks = vec![
        check_git_installed(),
        check_inside_git_repo(&repo_root),
        check_head_exists(&repo_root),
        check_worktree_command(&repo_root),
        check_pytxo_dirs_writable(&repo_root),
    ];
    Ok(DoctorReport { checks })
}

fn check_git_installed() -> DoctorCheck {
    match Command::new("git").arg("--version").output() {
        Ok(o) if o.status.success() => DoctorCheck {
            name: "git_installed".into(),
            ok: true,
            detail: String::from_utf8_lossy(&o.stdout).trim().to_string(),
        },
        Ok(o) => DoctorCheck {
            name: "git_installed".into(),
            ok: false,
            detail: format!(
                "git --version failed: {}",
                String::from_utf8_lossy(&o.stderr)
            ),
        },
        Err(e) => DoctorCheck {
            name: "git_installed".into(),
            ok: false,
            detail: format!("git not found: {e}"),
        },
    }
}

fn check_inside_git_repo(repo: &Path) -> DoctorCheck {
    git_ok(
        "git_repo",
        repo,
        &["rev-parse", "--is-inside-work-tree"],
        "not inside a git repository",
    )
}

fn check_head_exists(repo: &Path) -> DoctorCheck {
    match Command::new("git")
        .args(["rev-parse", "HEAD"])
        .current_dir(repo)
        .output()
    {
        Ok(o) if o.status.success() => DoctorCheck {
            name: "git_head".into(),
            ok: true,
            detail: String::from_utf8_lossy(&o.stdout).trim().to_string(),
        },
        Ok(_) => DoctorCheck {
            name: "git_head".into(),
            ok: false,
            detail: "no commits yet — run: git add . && git commit -m \"initial commit\"".into(),
        },
        Err(e) => DoctorCheck {
            name: "git_head".into(),
            ok: false,
            detail: format!("git rev-parse HEAD failed: {e}"),
        },
    }
}

fn check_worktree_command(repo: &Path) -> DoctorCheck {
    git_ok(
        "git_worktree",
        repo,
        &["worktree", "list"],
        "git worktree unavailable",
    )
}

fn check_pytxo_dirs_writable(repo: &Path) -> DoctorCheck {
    let cfg = PytxoConfig::default();
    let wt = repo.join(&cfg.worktree_dir);
    let data = repo.join(&cfg.data_dir);
    match (std::fs::create_dir_all(&wt), std::fs::create_dir_all(&data)) {
        (Ok(()), Ok(())) => DoctorCheck {
            name: "pytxo_dirs".into(),
            ok: true,
            detail: format!("writable: {} and {}", wt.display(), data.display()),
        },
        (Err(e), _) | (_, Err(e)) => DoctorCheck {
            name: "pytxo_dirs".into(),
            ok: false,
            detail: format!("cannot create .pytxo directories: {e}"),
        },
    }
}

fn git_ok(name: &str, repo: &Path, args: &[&str], fail_msg: &str) -> DoctorCheck {
    match Command::new("git").args(args).current_dir(repo).output() {
        Ok(o) if o.status.success() => DoctorCheck {
            name: name.into(),
            ok: true,
            detail: String::from_utf8_lossy(&o.stdout).trim().to_string(),
        },
        Ok(o) => DoctorCheck {
            name: name.into(),
            ok: false,
            detail: format!("{fail_msg}: {}", String::from_utf8_lossy(&o.stderr).trim()),
        },
        Err(e) => DoctorCheck {
            name: name.into(),
            ok: false,
            detail: format!("{fail_msg}: {e}"),
        },
    }
}

pub fn assert_git_ready(repo: &Path) -> anyhow::Result<()> {
    let report = run_doctor(Some(repo))?;
    let failed: Vec<_> = report.checks.iter().filter(|c| !c.ok).collect();
    if failed.is_empty() {
        return Ok(());
    }
    let msg: Vec<String> = failed
        .iter()
        .map(|c| format!("{}: {}", c.name, c.detail))
        .collect();
    anyhow::bail!("git preflight failed:\n{}", msg.join("\n"))
}
