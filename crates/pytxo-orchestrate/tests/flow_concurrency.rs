//! A one-worker mission must preserve its complete reviewed scope and waves.
use pytxo_core::{PermissionProfile, PytxoConfig};
use pytxo_orchestrate::{
    dispatch_flow, preview_flow, save_reviewed_flow_plan, trust_repo, FlowDraftInput, FlowSource,
};
use pytxo_store::{Catalog, PytxoStore};
use std::{fs, process::Command, time::Duration};

#[tokio::test]
async fn dispatch_preserves_reviewed_single_worker_waves() {
    let temp = tempfile::tempdir().unwrap();
    let repo = temp.path().join("repo");
    let bin = temp.path().join("bin");
    fs::create_dir_all(&repo).unwrap();
    fs::create_dir_all(&bin).unwrap();
    std::env::set_var("PYTXO_HOME", temp.path().join("home"));
    std::env::set_var("PYTXO_TRUST_STORE", temp.path().join("trust.json"));
    // This executable belongs only to this integration-test process.
    #[cfg(windows)]
    fs::write(
        bin.join("codex.cmd"),
        "@echo fixture task\r\n@exit /b 0\r\n",
    )
    .unwrap();
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        fs::write(bin.join("codex"), "#!/bin/sh\necho fixture task\n").unwrap();
        fs::set_permissions(bin.join("codex"), fs::Permissions::from_mode(0o755)).unwrap();
    }
    let mut paths = vec![bin];
    paths.extend(std::env::split_paths(
        &std::env::var_os("PATH").unwrap_or_default(),
    ));
    std::env::set_var("PATH", std::env::join_paths(paths).unwrap());
    for name in ["one.txt", "two.txt", "three.txt"] {
        fs::write(repo.join(name), "baseline\n").unwrap();
    }
    fs::write(
        repo.join("pytxo.toml"),
        "max_agents = 1\npermission_profile = \"orbit\"\nexecution_backend = \"subprocess\"\n",
    )
    .unwrap();
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
    trust_repo(&repo, PermissionProfile::Orbit).unwrap();
    let catalog = Catalog::open(&temp.path().join("catalog.db")).unwrap();
    let plan = preview_flow(
        &catalog,
        FlowDraftInput {
            id: "single-worker".into(),
            title: "Three tasks".into(),
            mission_text: "update one.txt; update two.txt; update three.txt".into(),
            source: FlowSource::Text,
            domain_id: Some(repo.to_string_lossy().into_owned()),
            project_id: None,
            ade_id: Some("codex".into()),
        },
    )
    .unwrap();
    assert_eq!(plan.tasks.len(), 3);
    assert_eq!(plan.waves.len(), 3);
    save_reviewed_flow_plan(&catalog, plan).unwrap();
    let run_id = dispatch_flow(&catalog, "single-worker").unwrap();
    let store = PytxoStore::open(&PytxoConfig::default().db_path_at(&repo)).unwrap();
    let deadline = tokio::time::Instant::now() + Duration::from_secs(30);
    loop {
        let row = store.get_run(&run_id).unwrap().unwrap();
        if !matches!(row.status.as_str(), "starting" | "running") {
            assert_eq!(row.status, "completed");
            break;
        }
        assert!(
            tokio::time::Instant::now() < deadline,
            "mission did not settle"
        );
        tokio::time::sleep(Duration::from_millis(30)).await;
    }
    let agents = store.list_agents_for_run(&run_id).unwrap();
    assert_eq!(agents.len(), 3);
    let mut waves: Vec<_> = agents.iter().map(|agent| agent.wave).collect();
    waves.sort();
    assert_eq!(waves, vec![0, 1, 2]);
}
