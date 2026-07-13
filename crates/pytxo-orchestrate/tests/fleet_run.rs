use std::path::PathBuf;
use std::process::Command;
use std::sync::{Mutex, OnceLock};
use std::time::Duration;

use pytxo_core::{FleetManifest, PermissionProfile};
use pytxo_orchestrate::{
    fleet_run, fleet_status_nodes, open_store_for_domain, trust_repo, FleetRunOptions,
};
use pytxo_store::Catalog;
use tempfile::TempDir;

fn shared_trust_store() -> &'static PathBuf {
    static STORE: OnceLock<PathBuf> = OnceLock::new();
    STORE.get_or_init(|| {
        let dir = TempDir::new().expect("trust tempdir");
        let path = dir.path().join("trusted-domains.json");
        std::mem::forget(dir);
        unsafe { std::env::set_var("PYTXO_TRUST_STORE", &path) };
        path
    })
}
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

fn trust(path: &std::path::Path) {
    static TRUST_LOCK: OnceLock<Mutex<()>> = OnceLock::new();
    let _ = shared_trust_store();
    let _guard = TRUST_LOCK.get_or_init(|| Mutex::new(())).lock().unwrap();
    trust_repo(path, PermissionProfile::Orbit).unwrap();
}
#[tokio::test]
async fn fleet_dag_orders_cross_repo_waves() {
    let home = TempDir::new().unwrap();
    unsafe { std::env::set_var("HOME", home.path()) };
    unsafe { std::env::set_var("USERPROFILE", home.path()) };
    let _ = shared_trust_store();
    let api = TempDir::new().unwrap();
    let web = TempDir::new().unwrap();
    init_git_repo(api.path());
    init_git_repo(web.path());
    trust(api.path());
    trust(web.path());

    let manifest_dir = TempDir::new().unwrap();
    let manifest_path = manifest_dir.path().join("fleet.toml");
    std::fs::write(
        &manifest_path,
        format!(
            r#"
[fleet]
id = "test-fleet"

[[node]]
id = "fix-api"
repo = "{api}"
cmd = "echo api"
agents = 1

[[node]]
id = "deploy-web"
repo = "{web}"
cmd = "echo web"
agents = 1
depends_on = ["fix-api"]
"#,
            api = toml_path(api.path()),
            web = toml_path(web.path()),
        ),
    )
    .unwrap();

    let plan = FleetManifest::load(&manifest_path).unwrap().plan().unwrap();
    assert_eq!(plan.waves.len(), 2);

    let result = fleet_run(FleetRunOptions {
        manifest: Some(manifest_path),
        fleet_id: None,
        dry_run: false,
        wait_timeout: Duration::from_secs(120),
        ..FleetRunOptions::default()
    })
    .await
    .unwrap();

    assert_eq!(result.status, "completed");
    assert_eq!(result.nodes.len(), 2);
    assert_eq!(result.nodes[0].wave, 0);
    assert_eq!(result.nodes[1].wave, 1);

    let nodes = fleet_status_nodes(&result.fleet_run_id).unwrap();
    assert_eq!(nodes.len(), 2);
    assert_eq!(nodes[0].node_id, "fix-api");
    assert_eq!(nodes[0].status, "completed");
    assert_eq!(nodes[1].node_id, "deploy-web");
    assert_eq!(nodes[1].status, "completed");

    let store_a = open_store_for_domain(&api.path().to_string_lossy(), None)
        .unwrap()
        .1;
    let store_b = open_store_for_domain(&web.path().to_string_lossy(), None)
        .unwrap()
        .1;
    assert!(!store_a.list_runs(5).unwrap().is_empty());
    assert!(!store_b.list_runs(5).unwrap().is_empty());

    let cat = Catalog::open_default().unwrap();
    let domains = cat.list_domains().unwrap();
    assert!(domains.len() >= 2);
}
