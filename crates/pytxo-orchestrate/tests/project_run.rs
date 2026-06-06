use std::process::Command;

use pytxo_orchestrate::{open_store_for_domain, project_run, ProjectRunOptions};
use tempfile::TempDir;

fn init_git_repo(path: &std::path::Path) {
    for args in [
        vec!["init"],
        vec!["config", "user.email", "pytxo@test.local"],
        vec!["config", "user.name", "Pytxo Test"],
    ] {
        assert!(Command::new("git")
            .args(&args)
            .current_dir(path)
            .status()
            .unwrap()
            .success());
    }
    std::fs::write(path.join("README.md"), "test\n").unwrap();
    let _ = Command::new("git")
        .args(["add", "."])
        .current_dir(path)
        .status();
    let _ = Command::new("git")
        .args(["commit", "-m", "init"])
        .current_dir(path)
        .status();
}

fn toml_path(p: &std::path::Path) -> String {
    p.to_string_lossy().replace('\\', "/")
}

fn norm_path(p: &str) -> String {
    p.replace('\\', "/").to_lowercase()
}

#[tokio::test]
async fn unified_project_run_includes_writable_roots() {
    let api = TempDir::new().unwrap();
    let web = TempDir::new().unwrap();
    let protos = TempDir::new().unwrap();
    init_git_repo(api.path());
    init_git_repo(web.path());
    init_git_repo(protos.path());

    let manifest_dir = TempDir::new().unwrap();
    let manifest_path = manifest_dir.path().join("project.toml");
    std::fs::write(
        &manifest_path,
        format!(
            r#"
[project]
id = "test-proj"
name = "Test Project"

[[roots]]
path = "{api}"
label = "api"
primary = true

[[roots]]
path = "{web}"
label = "web"

[[roots]]
path = "{protos}"
label = "protos"
read_only = true
"#,
            api = toml_path(api.path()),
            web = toml_path(web.path()),
            protos = toml_path(protos.path()),
        ),
    )
    .unwrap();

    let results = project_run(ProjectRunOptions {
        manifest: Some(manifest_path),
        project_id: None,
        cmd: "echo pytxo".into(),
        agents: 1,
        config: None,
        dry_run: true,
    })
    .await
    .unwrap();

    assert_eq!(results.len(), 1);
    assert_eq!(results[0].project_id, "test-proj");
    assert!(results[0].roots.contains(&"api".to_string()));
    assert!(results[0].roots.contains(&"web".to_string()));
    assert!(!results[0].roots.contains(&"protos".to_string()));
}

#[tokio::test]
async fn live_project_run_tags_wal_with_project_and_root() {
    let api = TempDir::new().unwrap();
    init_git_repo(api.path());

    std::fs::write(
        api.path().join("pytxo.toml"),
        r#"
max_agents = 1
signal_core = false

[[task]]
id = "task-api"
agent = "builder"
paths = ["README.md"]
root = "api"
"#,
    )
    .unwrap();

    let manifest_dir = TempDir::new().unwrap();
    let manifest_path = manifest_dir.path().join("project.toml");
    std::fs::write(
        &manifest_path,
        format!(
            r#"
[project]
id = "live-proj"

[[roots]]
path = "{api}"
label = "api"
primary = true
"#,
            api = toml_path(api.path()),
        ),
    )
    .unwrap();

    let results = project_run(ProjectRunOptions {
        manifest: Some(manifest_path),
        project_id: None,
        cmd: "echo pytxo-live".into(),
        agents: 1,
        config: None,
        dry_run: false,
    })
    .await
    .unwrap();

    assert_eq!(results.len(), 1);
    let run_id = results[0].run_id.clone();

    let canon = std::fs::canonicalize(api.path()).unwrap();
    let domain_id = canon.to_string_lossy().to_string();
    let (_cfg, store) = open_store_for_domain(&domain_id, None).unwrap();

    assert_eq!(
        store.run_project(&run_id).unwrap().as_deref(),
        Some("live-proj")
    );

    let agents = store.list_agents_for_run(&run_id).unwrap();
    assert_eq!(agents.len(), 1);
    assert_eq!(agents[0].root_id.as_deref(), Some("api"));
}

#[tokio::test]
async fn multi_root_agents_share_run_id_and_distinct_root_ids() {
    let api = TempDir::new().unwrap();
    let web = TempDir::new().unwrap();
    init_git_repo(api.path());
    init_git_repo(web.path());
    std::fs::write(api.path().join("README.md"), "api-marker\n").unwrap();
    std::fs::write(web.path().join("README.md"), "web-marker\n").unwrap();
    let _ = Command::new("git")
        .args(["add", "README.md"])
        .current_dir(api.path())
        .status();
    let _ = Command::new("git")
        .args(["commit", "-m", "api marker"])
        .current_dir(api.path())
        .status();
    let _ = Command::new("git")
        .args(["add", "README.md"])
        .current_dir(web.path())
        .status();
    let _ = Command::new("git")
        .args(["commit", "-m", "web marker"])
        .current_dir(web.path())
        .status();

    std::fs::write(
        api.path().join("pytxo.toml"),
        r#"
max_agents = 2
signal_core = false

[[task]]
id = "task-api"
agent = "builder"
paths = ["README.md"]
root = "api"

[[task]]
id = "task-web"
agent = "builder"
paths = ["README.md"]
root = "web"
"#,
    )
    .unwrap();

    let manifest_dir = TempDir::new().unwrap();
    let manifest_path = manifest_dir.path().join("project.toml");
    std::fs::write(
        &manifest_path,
        format!(
            r#"
[project]
id = "multi-root-proj"

[[roots]]
path = "{api}"
label = "api"
primary = true

[[roots]]
path = "{web}"
label = "web"
"#,
            api = toml_path(api.path()),
            web = toml_path(web.path()),
        ),
    )
    .unwrap();

    let results = project_run(ProjectRunOptions {
        manifest: Some(manifest_path.clone()),
        project_id: None,
        cmd: "echo pytxo-multi".into(),
        agents: 2,
        config: None,
        dry_run: false,
    })
    .await
    .unwrap();

    assert_eq!(results.len(), 1);
    let run_id = results[0].run_id.clone();
    assert_eq!(results[0].project_id, "multi-root-proj");

    let canon = std::fs::canonicalize(api.path()).unwrap();
    let domain_id = canon.to_string_lossy().to_string();
    let (_cfg, store) = open_store_for_domain(&domain_id, None).unwrap();

    let agents = store.list_agents_for_run(&run_id).unwrap();
    assert_eq!(agents.len(), 2, "expected two agents in one run");

    let mut root_ids: Vec<String> = agents.iter().filter_map(|a| a.root_id.clone()).collect();
    root_ids.sort();
    assert_eq!(root_ids, vec!["api".to_string(), "web".to_string()]);

    let api_marker = norm_path(
        api.path()
            .file_name()
            .unwrap()
            .to_string_lossy()
            .as_ref(),
    );
    let web_marker = norm_path(
        web.path()
            .file_name()
            .unwrap()
            .to_string_lossy()
            .as_ref(),
    );
    for agent in &agents {
        let wt_norm = norm_path(agent.worktree_path.as_deref().unwrap_or(""));
        assert!(wt_norm.contains("/.pytxo/worktrees/") || wt_norm.contains("\\.pytxo\\worktrees\\"));
        match agent.root_id.as_deref() {
            Some("api") => assert!(wt_norm.contains(&api_marker), "api worktree: {wt_norm}"),
            Some("web") => assert!(wt_norm.contains(&web_marker), "web worktree: {wt_norm}"),
            other => panic!("unexpected root_id: {other:?}"),
        }
    }
}

#[tokio::test]
async fn project_status_lists_deduped_root_ids() {
    use pytxo_orchestrate::project_status;

    let api = TempDir::new().unwrap();
    let web = TempDir::new().unwrap();
    init_git_repo(api.path());
    init_git_repo(web.path());

    std::fs::write(
        api.path().join("pytxo.toml"),
        r#"
max_agents = 2
signal_core = false

[[task]]
id = "task-api"
agent = "builder"
paths = ["README.md"]
root = "api"

[[task]]
id = "task-web"
agent = "builder"
paths = ["README.md"]
root = "web"
"#,
    )
    .unwrap();

    let manifest_dir = TempDir::new().unwrap();
    let manifest_path = manifest_dir.path().join("project.toml");
    std::fs::write(
        &manifest_path,
        format!(
            r#"
[project]
id = "status-proj"

[[roots]]
path = "{api}"
label = "api"
primary = true

[[roots]]
path = "{web}"
label = "web"
"#,
            api = toml_path(api.path()),
            web = toml_path(web.path()),
        ),
    )
    .unwrap();

    project_run(ProjectRunOptions {
        manifest: Some(manifest_path.clone()),
        project_id: None,
        cmd: "echo status".into(),
        agents: 2,
        config: None,
        dry_run: false,
    })
    .await
    .unwrap();

    let rows = project_status(Some(manifest_path), None, 10).unwrap();
    assert!(!rows.is_empty());
    let row = rows
        .iter()
        .find(|r| r.agent_count == 2)
        .expect("multi-root run row");
    assert_eq!(row.agent_count, 2);
    let mut roots = row.root_ids.clone();
    roots.sort();
    assert_eq!(roots, vec!["api".to_string(), "web".to_string()]);
}
