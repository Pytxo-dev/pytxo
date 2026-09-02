use std::process::Command;
use std::sync::OnceLock;
use std::time::{Duration, Instant};

use pytxo_core::{
    DomainId, ExecutionBackend, ExecutionPlan, IsolationMode, PermissionProfile, RaceShield,
    ScheduledTask, Task, TaskId,
};
use pytxo_orchestrate::{
    default_hypervisor, enqueue_agent_stdin, load_config_for_repo, mcp_proxy_call, trust_repo,
    HypervisorRegistry, RunOptions,
};
use pytxo_runner::{permission_enforcement_receipt, spawn_test_mcp_child};
use pytxo_store::PytxoStore;
use serde_json::json;
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

fn init_git_repo(path: &std::path::Path, config: &str) {
    std::fs::write(path.join("pytxo.toml"), config).unwrap();
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

fn one_task(agent: &str) -> Task {
    Task {
        id: TaskId("bounded-task".into()),
        agent: agent.into(),
        paths: vec!["bounded.txt".into()],
        depends_on: Vec::new(),
        root: None,
        signal_fidelity: None,
        verify: Vec::new(),
    }
}

#[test]
fn folder_trust_is_a_ceiling_and_does_not_elevate_repository_defaults() {
    isolate_process_state();
    let repo = TempDir::new().unwrap();
    std::fs::write(
        repo.path().join("pytxo.toml"),
        "permission_profile = \"orbit\"\n",
    )
    .unwrap();
    trust_repo(repo.path(), PermissionProfile::Galaxy).unwrap();

    let cfg = load_config_for_repo(None, repo.path()).unwrap();
    assert_eq!(cfg.permission_profile, PermissionProfile::Orbit);
    assert_eq!(cfg.permission_ceiling, Some(PermissionProfile::Galaxy));
}

#[tokio::test]
async fn repository_supernova_agent_cannot_escape_orbit_folder_ceiling() {
    isolate_process_state();
    let repo = TempDir::new().unwrap();
    init_git_repo(
        repo.path(),
        r#"
max_agents = 1
permission_profile = "orbit"
execution_backend = "subprocess"

[[agent]]
name = "elevated"
permission_profile = "supernova"
"#,
    );
    trust_repo(repo.path(), PermissionProfile::Orbit).unwrap();

    HypervisorRegistry::new()
        .run_blocking(RunOptions {
            agents: 1,
            cmd: "echo isolated".into(),
            config: Some(repo.path().join("pytxo.toml")),
            dry_run: false,
            keep_worktrees: true,
            repo: Some(repo.path().to_path_buf()),
            execution: Some(ExecutionBackend::Subprocess),
            project: None,
            tasks: Some(vec![one_task("elevated")]),
            task_cmd_template: None,
            task_prompts: None,
        })
        .await
        .unwrap();

    let store = PytxoStore::open(&repo.path().join(".pytxo/data/pytxo.db")).unwrap();
    let run = store.list_runs(1).unwrap().pop().unwrap();
    let agent = store.list_agents_for_run(&run.id).unwrap().pop().unwrap();
    let workspace = std::fs::canonicalize(agent.worktree_path.unwrap()).unwrap();
    let repo_root = std::fs::canonicalize(repo.path()).unwrap();
    assert_ne!(
        workspace, repo_root,
        "Orbit must retain workspace isolation"
    );

    let contract = store.get_run_contract(&run.id).unwrap().unwrap();
    let enforcement: serde_json::Value =
        serde_json::from_str(contract.enforcement_json.as_deref().unwrap()).unwrap();
    assert_eq!(
        enforcement["agents"]["elevated"]["requested_profile"],
        "supernova"
    );
    assert_eq!(
        enforcement["agents"]["elevated"]["effective_profile"],
        "orbit"
    );
}

fn seed_live_galaxy_agent(
    repo: &std::path::Path,
    run_id: &str,
) -> std::sync::Arc<pytxo_orchestrate::DomainState> {
    std::fs::write(repo.join("pytxo.toml"), "permission_profile = \"orbit\"\n").unwrap();
    let cfg = load_config_for_repo(None, repo).unwrap();
    let domain = default_hypervisor().ensure_domain(repo, &cfg).unwrap();
    let agent_key = format!("{run_id}:agent-0");
    domain
        .swarm
        .try_claim_paths(&agent_key, &[format!("{run_id}.txt")])
        .unwrap();

    let plan = ExecutionPlan {
        waves: vec![vec![ScheduledTask {
            task_id: TaskId("gated-task".into()),
            agent: "galaxy-agent".into(),
            paths: vec![format!("{run_id}.txt")],
            depends_on: Vec::new(),
            wave: 0,
            root: None,
            signal_fidelity: None,
            verify: Vec::new(),
        }]],
        conflicts: Vec::new(),
        max_agents: 1,
        warnings: Vec::new(),
    };
    let run_receipt = permission_enforcement_receipt(
        PermissionProfile::Orbit,
        PermissionProfile::Orbit,
        &DomainId::from_repo_root(repo).unwrap(),
        IsolationMode::Worktree,
        &[],
    )
    .unwrap();
    let agent_receipt = permission_enforcement_receipt(
        PermissionProfile::Galaxy,
        PermissionProfile::Galaxy,
        &DomainId::from_repo_root(repo).unwrap(),
        IsolationMode::Worktree,
        &[],
    )
    .unwrap();
    let store = PytxoStore::open(&cfg.db_path_at(repo)).unwrap();
    store
        .insert_run_with_profile(
            run_id,
            &repo.to_string_lossy(),
            Some(PermissionProfile::Orbit.as_str()),
        )
        .unwrap();
    store
        .insert_agent(
            &agent_key,
            run_id,
            "gated-task",
            0,
            Some(&repo.to_string_lossy()),
            "fixture",
        )
        .unwrap();
    store
        .save_run_contract(
            run_id,
            "fixture-base",
            &serde_json::to_string(&plan).unwrap(),
            &json!({
                "run": run_receipt,
                "agents": {
                    "galaxy-agent": agent_receipt.clone(),
                    "agent-0": agent_receipt
                }
            })
            .to_string(),
        )
        .unwrap();
    domain
}

fn deny_next_hitl(
    domain: &std::sync::Arc<pytxo_orchestrate::DomainState>,
) -> std::thread::JoinHandle<bool> {
    let hitl = domain.hitl.clone();
    std::thread::spawn(move || {
        let deadline = Instant::now() + Duration::from_secs(2);
        while Instant::now() < deadline {
            if let Some(request) = hitl.pending().into_iter().next() {
                hitl.resolve(&request.id, false);
                return true;
            }
            std::thread::sleep(Duration::from_millis(10));
        }
        false
    })
}

#[test]
fn stdin_gate_uses_live_agents_effective_profile_from_run_envelope() {
    isolate_process_state();
    let repo = TempDir::new().unwrap();
    let run_id = "stdin-effective-profile";
    let domain = seed_live_galaxy_agent(repo.path(), run_id);
    let reviewer = deny_next_hitl(&domain);

    let error = enqueue_agent_stdin(
        Some(repo.path().to_path_buf()),
        &format!("{run_id}:agent-0"),
        b"rm -rf build\n",
    )
    .expect_err("Galaxy stdin must require HITL even when the run default is Orbit");

    assert!(reviewer.join().unwrap(), "stdin action never reached HITL");
    assert!(
        error.to_string().contains("denied"),
        "unexpected error: {error}"
    );
    assert!(domain
        .swarm
        .drain_stdin(&format!("{run_id}:agent-0"))
        .is_empty());
}

#[test]
fn mcp_gate_uses_live_agents_effective_profile_from_run_envelope() {
    isolate_process_state();
    let repo = TempDir::new().unwrap();
    let run_id = "mcp-effective-profile";
    let domain = seed_live_galaxy_agent(repo.path(), run_id);
    let agent_key = format!("{run_id}:agent-0");
    let (session, child) = spawn_test_mcp_child().unwrap();
    domain.mcp_hub.register(&agent_key, session);
    let reviewer = deny_next_hitl(&domain);

    let result = mcp_proxy_call(
        Some(repo.path().to_path_buf()),
        &agent_key,
        "tools/call",
        json!({ "name": "echo_fixture" }),
    );

    domain.mcp_hub.deregister(&agent_key);
    child.join().ok();
    assert!(reviewer.join().unwrap(), "MCP action never reached HITL");
    let error = result.expect_err("Galaxy MCP calls must require HITL when run default is Orbit");
    assert!(
        error.to_string().contains("denied"),
        "unexpected error: {error}"
    );
}
