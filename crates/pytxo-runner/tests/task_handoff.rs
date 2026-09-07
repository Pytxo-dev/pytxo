use std::collections::HashMap;
use std::process::Command;

use pytxo_core::{
    BillingMode, ExecutionBackend, ExecutionPlan, FidelityTier, IsolationMode, PermissionProfile,
    RunId, ScheduledTask, TaskId,
};
use pytxo_runner::{execute_plan, ProcessRegistry, RunContext, SwarmRegistry};

async fn delivered_prompt(backend: ExecutionBackend, original: Option<&str>) -> (String, String) {
    let fixture = tempfile::tempdir().unwrap();
    let repo = fixture.path();
    for args in [
        vec!["init"],
        vec!["config", "user.email", "pytxo@test.local"],
        vec!["config", "user.name", "Pytxo Test"],
    ] {
        assert!(Command::new("git")
            .args(args)
            .current_dir(repo)
            .output()
            .unwrap()
            .status
            .success());
    }
    std::fs::write(repo.join("README.md"), "fixture\n").unwrap();
    let capture = if cfg!(windows) {
        std::fs::write(
            repo.join("capture.ps1"),
            "[IO.File]::WriteAllText((Join-Path (Get-Location).Path 'prompt.txt'), $env:PYTXO_TASK_PROMPT)\n[IO.File]::WriteAllText((Join-Path (Get-Location).Path 'present.txt'), [string](Test-Path Env:PYTXO_TASK_PROMPT))\n",
        )
        .unwrap();
        "powershell -NoProfile -NonInteractive -File capture.ps1"
    } else {
        std::fs::write(
            repo.join("capture.sh"),
            "printf '%s' \"${PYTXO_TASK_PROMPT-}\" > prompt.txt\nif [ \"${PYTXO_TASK_PROMPT+x}\" = x ]; then printf True > present.txt; else printf False > present.txt; fi\n",
        )
        .unwrap();
        "sh capture.sh"
    };
    for args in [vec!["add", "."], vec!["commit", "-m", "fixture"]] {
        assert!(Command::new("git")
            .args(args)
            .current_dir(repo)
            .output()
            .unwrap()
            .status
            .success());
    }

    let task = |id: &str, depends_on: Vec<String>, wave| ScheduledTask {
        task_id: TaskId(id.into()),
        agent: "fixture".into(),
        paths: vec!["prompt.txt".into(), "present.txt".into()],
        depends_on,
        wave,
        root: None,
        signal_fidelity: None,
        verify: vec!["echo verification-ok".into()],
    };
    let plan = ExecutionPlan {
        waves: vec![
            vec![task("prepare-input", vec![], 0)],
            vec![task("handoff", vec!["prepare-input".into()], 1)],
        ],
        conflicts: vec![],
        max_agents: 1,
        warnings: vec![],
    };
    let (domain_id, model_router, managed_transport, token_estimator) =
        RunContext::default_metering(repo);
    let ctx = RunContext {
        run_id: RunId::new(),
        repo_root: repo.to_path_buf(),
        worktree_base: repo.join(".pytxo/worktrees"),
        data_dir: repo.join(".pytxo/data"),
        cmd: capture.into(),
        task_cmd_template: None,
        task_prompts: original
            .map(|prompt| HashMap::from([("handoff".into(), prompt.into())]))
            .unwrap_or_default(),
        keep_worktrees: true,
        on_event: None,
        signal_core: false,
        signal_fidelity: FidelityTier::Low,
        isolation_mode: IsolationMode::Worktree,
        permission_profile: PermissionProfile::Orbit,
        agent_profiles: HashMap::new(),
        route_agents: Vec::new(),
        billing_mode: BillingMode::Byok,
        domain_id,
        model_router,
        managed_transport,
        usage_meter: None,
        token_estimator,
        execution_backend: backend,
        pty_rows: 24,
        pty_cols: 120,
        hitl: None,
        hitl_manual_flush: false,
        agent_paths: HashMap::new(),
        agent_fidelity: HashMap::new(),
        roots: HashMap::new(),
        readonly_context_roots: Vec::new(),
        subprocess_stdin: false,
        cloud_dispatcher: None,
        context_cache: None,
        cloud_cache_enabled: false,
        cloud_fallback_local: false,
        mcp_hub: None,
        mcp_hub_enabled: false,
        mcp_allowlist: Vec::new(),
        sparse_exclude: Vec::new(),
    };
    let results = execute_plan(
        &ctx,
        &plan,
        &ProcessRegistry::default(),
        &SwarmRegistry::new(),
    )
    .await
    .unwrap();
    assert!(
        results.iter().all(|result| result.outcome.is_success()),
        "{results:?}"
    );
    let workspace = results[1].worktree_path.as_ref().unwrap();
    assert!(!workspace.join("escaped.txt").exists());
    (
        std::fs::read_to_string(workspace.join("prompt.txt")).unwrap(),
        std::fs::read_to_string(workspace.join("present.txt")).unwrap(),
    )
}

fn contract_list(prompt: &str, label: &str) -> Vec<String> {
    #[cfg(windows)]
    {
        let label = format!("{label}: [");
        let value = prompt
            .split_once(&label)
            .unwrap_or_else(|| panic!("missing {label:?} in delivered prompt {prompt:?}"))
            .1
            .split_once(']')
            .unwrap()
            .0;
        if value.is_empty() {
            return Vec::new();
        }
        value
            .split("; ")
            .map(|entry| serde_json::from_str(&format!("\"{entry}\"")).unwrap())
            .collect()
    }
    #[cfg(not(windows))]
    {
        let label = format!("{label} (JSON): ");
        let value = prompt
            .lines()
            .find_map(|line| line.strip_prefix(&label))
            .unwrap_or_else(|| panic!("missing {label:?} in delivered prompt {prompt:?}"));
        serde_json::from_str(value).unwrap()
    }
}

#[tokio::test]
async fn task_handoff_delivers_reviewed_scope_identically_to_both_backends() {
    let original = "Update \"risk\" behavior.\nKeep this literal: $(Set-Content -LiteralPath escaped.txt -Value unsafe) & `quoted`";
    let mut delivered = Vec::new();
    for backend in [ExecutionBackend::Pty, ExecutionBackend::Subprocess] {
        let (prompt, present) = delivered_prompt(backend, Some(original)).await;
        assert_eq!(present, "True");
        assert!(prompt.starts_with(original), "original task text changed");
        assert_eq!(
            contract_list(&prompt, "Owned paths"),
            vec!["prompt.txt", "present.txt"]
        );
        assert_eq!(
            contract_list(&prompt, "Dependency task IDs"),
            vec!["prepare-input"]
        );
        assert_eq!(
            contract_list(&prompt, "Recorded verification commands"),
            vec!["echo verification-ok"]
        );
        assert!(prompt.contains("Edit only the owned paths"));
        assert!(prompt.contains("generic habits or skills"));
        assert!(prompt.contains("report the required path"));
        assert!(prompt.contains("guidance"));
        delivered.push(prompt);
    }
    assert_eq!(delivered[0], delivered[1]);
}

#[tokio::test]
async fn task_handoff_keeps_empty_prompt_empty() {
    for backend in [ExecutionBackend::Pty, ExecutionBackend::Subprocess] {
        assert_eq!(delivered_prompt(backend, Some("")).await.0, "");
    }
}

#[tokio::test]
async fn task_handoff_keeps_no_prompt_absent() {
    for backend in [ExecutionBackend::Pty, ExecutionBackend::Subprocess] {
        assert_eq!(
            delivered_prompt(backend, None).await,
            ("".into(), "False".into())
        );
    }
}
