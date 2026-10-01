use std::process::Command;
use std::sync::OnceLock;

use pytxo_core::{ExecutionBackend, PermissionProfile, Task, TaskId};
use pytxo_orchestrate::{trust_repo, HypervisorRegistry, RunOptions};
use pytxo_store::PytxoStore;
use tempfile::TempDir;

fn isolate_process_state() {
    static STATE: OnceLock<()> = OnceLock::new();
    STATE.get_or_init(|| {
        let home = TempDir::new().expect("pytxo home tempdir");
        let trust = TempDir::new().expect("trust tempdir");
        let home_path = home.path().to_path_buf();
        let trust_path = trust.path().join("trusted-domains.json");
        std::mem::forget(home);
        std::mem::forget(trust);
        // SAFETY: this integration-test process owns these variables.
        unsafe {
            std::env::set_var("PYTXO_HOME", home_path);
            std::env::set_var("PYTXO_TRUST_STORE", trust_path);
        }
    });
}

fn init_git_repo(path: &std::path::Path) {
    std::fs::write(
        path.join("pytxo.toml"),
        r#"
max_agents = 2
fail_fast = false
dag_explicit_deps = true
execution_backend = "subprocess"
"#,
    )
    .unwrap();
    for args in [
        vec!["init"],
        vec!["config", "user.email", "pytxo@test.local"],
        vec!["config", "user.name", "Pytxo Test"],
        vec!["add", "."],
        vec!["commit", "-m", "init"],
    ] {
        assert!(Command::new("git")
            .args(args)
            .current_dir(path)
            .status()
            .unwrap()
            .success());
    }
}

fn task(id: &str, depends_on: &[&str]) -> Task {
    Task {
        id: TaskId(id.into()),
        agent: "default".into(),
        paths: vec![format!("{id}.txt")],
        depends_on: depends_on.iter().map(|id| (*id).to_string()).collect(),
        root: None,
        signal_fidelity: None,
        verify: Vec::new(),
    }
}

fn partial_then_fail_command() -> String {
    if cfg!(windows) {
        "if \"{task_id}\"==\"upstream\" (echo partial>upstream.txt & exit /b 7) else (echo {task_id}>{task_id}.txt)".into()
    } else {
        "if [ \"{task_id}\" = \"upstream\" ]; then printf partial > upstream.txt; exit 7; else printf '%s' '{task_id}' > {task_id}.txt; fi".into()
    }
}

#[tokio::test]
async fn fail_fast_false_settles_failed_and_returns_error_after_independent_work() {
    isolate_process_state();
    let repo = TempDir::new().unwrap();
    init_git_repo(repo.path());
    trust_repo(repo.path(), PermissionProfile::Orbit).unwrap();

    let error = HypervisorRegistry::new()
        .run_blocking(RunOptions {
            agents: 2,
            cmd: String::new(),
            config: Some(repo.path().join("pytxo.toml")),
            dry_run: false,
            keep_worktrees: true,
            repo: Some(repo.path().to_path_buf()),
            execution: Some(ExecutionBackend::Subprocess),
            project: None,
            tasks: Some(vec![
                task("upstream", &[]),
                task("dependent", &["upstream"]),
                task("independent", &[]),
            ]),
            task_cmd_template: Some(partial_then_fail_command().into()),
            task_prompts: None,
        })
        .await
        .expect_err("a durably failed run must return failure to its caller");
    assert!(error
        .to_string()
        .contains("failed after durable settlement"));

    let store = PytxoStore::open(&repo.path().join(".pytxo/data/pytxo.db")).unwrap();
    let runs = store.list_runs(1).unwrap();
    assert_eq!(runs.len(), 1);
    assert_eq!(runs[0].status, "failed");
    assert!(runs[0].finished_at.is_some());

    let agents = store.list_agents_for_run(&runs[0].id).unwrap();
    let upstream = agents
        .iter()
        .find(|agent| agent.task_id == "upstream")
        .unwrap();
    let dependent = agents
        .iter()
        .find(|agent| agent.task_id == "dependent")
        .unwrap();
    let independent = agents
        .iter()
        .find(|agent| agent.task_id == "independent")
        .unwrap();

    assert_eq!(upstream.status, "failed");
    assert_eq!(dependent.status, "blocked_by_dependency");
    assert_eq!(dependent.exit_code, None);
    assert_eq!(dependent.worktree_path, None, "dependent must never start");
    assert_eq!(independent.status, "completed");
    assert_eq!(independent.exit_code, Some(0));
    assert!(
        std::path::Path::new(independent.worktree_path.as_ref().unwrap())
            .join("independent.txt")
            .exists()
    );
}
