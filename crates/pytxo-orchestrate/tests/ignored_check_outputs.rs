//! A check that writes files Git ignores, like `python -m unittest` writing
//! `__pycache__`, must not read as an agent edit or as a candidate change.
use pytxo_core::{ExecutionBackend, PermissionProfile, Task, TaskId};
use pytxo_orchestrate::{apply_run_changes, run, trust_repo, RunOptions};
use pytxo_runner::load_review_package;
use pytxo_store::PytxoStore;
use std::{fs, process::Command};

#[tokio::test]
async fn ignored_check_outputs_and_operator_files_are_not_agent_edits() {
    let temp = tempfile::tempdir().unwrap();
    std::env::set_var("PYTXO_HOME", temp.path().join("home"));
    std::env::set_var("PYTXO_TRUST_STORE", temp.path().join("trust.json"));
    let repo = temp.path().join("repo");
    fs::create_dir_all(&repo).unwrap();
    fs::write(repo.join("a.txt"), "0\n").unwrap();
    fs::write(repo.join(".gitignore"), ".pytxo/\ncache/\n.env\n").unwrap();
    // The check passes and leaves an ignored cache in its working directory,
    // both in the agent worktree and in the combined candidate.
    let verify = if cfg!(windows) {
        fs::write(
            repo.join("verify.ps1"),
            "New-Item -ItemType Directory -Force cache | Out-Null\nSet-Content cache/out.txt checked\n",
        )
        .unwrap();
        "powershell -NoProfile -NonInteractive -File verify.ps1"
    } else {
        fs::write(
            repo.join("verify.sh"),
            "mkdir -p cache && echo checked > cache/out.txt\n",
        )
        .unwrap();
        "sh verify.sh"
    };
    for args in [
        vec!["init", "-q"],
        vec!["add", "."],
        vec![
            "-c",
            "user.name=Test",
            "-c",
            "user.email=test@pytxo.local",
            "-c",
            "commit.gpgsign=false",
            "commit",
            "-qm",
            "baseline",
        ],
    ] {
        assert!(Command::new("git")
            .args(args)
            .current_dir(&repo)
            .status()
            .unwrap()
            .success());
    }
    // An ignored operator file that `git worktree add` never copies to the agent.
    fs::write(repo.join(".env"), "TOKEN=operator\n").unwrap();
    trust_repo(&repo, PermissionProfile::Orbit).unwrap();

    let id = run(RunOptions {
        agents: 1,
        cmd: "echo fixture".into(),
        config: None,
        dry_run: false,
        keep_worktrees: true,
        repo: Some(repo.clone()),
        execution: Some(ExecutionBackend::Subprocess),
        project: None,
        tasks: Some(vec![Task {
            id: TaskId("a".into()),
            agent: "a".into(),
            paths: vec!["a.txt".into()],
            depends_on: vec![],
            root: None,
            signal_fidelity: None,
            verify: vec![verify.into()],
        }]),
        task_cmd_template: Some("echo 1 > {task_id}.txt".into()),
        task_prompts: None,
    })
    .await
    .unwrap();

    let data = repo.join(".pytxo/data");
    let prepared = load_review_package(&data, &id.0).unwrap();
    let paths: Vec<_> = prepared
        .files
        .iter()
        .map(|file| file.path.as_str())
        .collect();
    assert_eq!(paths, ["a.txt"]);
    assert_eq!(
        prepared
            .candidate_verification
            .as_ref()
            .unwrap()
            .checks
            .len(),
        1
    );
    let digest = PytxoStore::open(&data.join("pytxo.db"))
        .unwrap()
        .get_run_contract(&id.0)
        .unwrap()
        .and_then(|contract| contract.prepared_digest)
        .unwrap();
    apply_run_changes(None, Some(repo.clone()), &id.0, &digest).unwrap();
    assert_eq!(fs::read_to_string(repo.join("a.txt")).unwrap().trim(), "1");
    assert_eq!(
        fs::read_to_string(repo.join(".env")).unwrap(),
        "TOKEN=operator\n"
    );
    assert!(!repo.join("cache").exists());
}
