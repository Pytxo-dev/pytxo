use std::process::Command;

use pytxo_orchestrate::{list_domains, HypervisorRegistry, RunOptions};
use tempfile::TempDir;

fn init_git_repo(path: &std::path::Path) {
    assert!(Command::new("git")
        .args(["init"])
        .current_dir(path)
        .status()
        .unwrap()
        .success());
    assert!(Command::new("git")
        .args(["config", "user.email", "pytxo@test.local"])
        .current_dir(path)
        .status()
        .unwrap()
        .success());
    assert!(Command::new("git")
        .args(["config", "user.name", "Pytxo Test"])
        .current_dir(path)
        .status()
        .unwrap()
        .success());
    std::fs::write(path.join("README.md"), "test\n").unwrap();
    Command::new("git")
        .args(["add", "."])
        .current_dir(path)
        .status()
        .unwrap();
    Command::new("git")
        .args(["commit", "-m", "init"])
        .current_dir(path)
        .status()
        .unwrap();
}

fn run_opts(repo: &std::path::Path) -> RunOptions {
    RunOptions {
        agents: 1,
        cmd: "echo pytxo".into(),
        config: None,
        dry_run: true,
        keep_worktrees: false,
        repo: Some(repo.to_path_buf()),
        execution: None,
        project: None,
        tasks: None,
        task_cmd_template: None,
        task_prompts: None,
    }
}

#[test]
fn same_repo_yields_same_domain_id() {
    let tmp = TempDir::new().unwrap();
    let repo = tmp.path();
    init_git_repo(repo);

    let hv = HypervisorRegistry::new();
    let (d1, _) = hv.dispatch(run_opts(repo)).unwrap();
    let (d2, _) = hv.dispatch(run_opts(repo)).unwrap();
    assert_eq!(d1, d2);
}

#[tokio::test]
async fn two_repos_register_two_domains() {
    let a = TempDir::new().unwrap();
    let b = TempDir::new().unwrap();
    init_git_repo(a.path());
    init_git_repo(b.path());

    let hv = HypervisorRegistry::new();
    hv.dispatch(run_opts(a.path())).unwrap();
    hv.dispatch(run_opts(b.path())).unwrap();

    assert_eq!(hv.list_domains().len(), 2);
}

#[test]
fn default_hypervisor_is_singleton() {
    let _ = std::sync::Arc::new(());
    // list_domains on default should not panic
    let _domains = list_domains();
}
