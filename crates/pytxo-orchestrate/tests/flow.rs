use chrono::Utc;
use pytxo_orchestrate::{
    dispatch_flow, preview_flow, save_flow_draft, save_reviewed_flow_plan, FlowBlockedReason,
    FlowDraftInput, FlowSource, FlowStatus,
};
use pytxo_store::{Catalog, FlowDraftRecord};
use std::sync::OnceLock;

static TEST_ADE_DIR: OnceLock<tempfile::TempDir> = OnceLock::new();

fn ensure_test_ade() {
    TEST_ADE_DIR.get_or_init(|| {
        let dir = tempfile::tempdir().unwrap();
        #[cfg(windows)]
        std::fs::write(dir.path().join("codex.cmd"), "@exit /b 0\r\n").unwrap();
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;

            let executable = dir.path().join("codex");
            std::fs::write(&executable, "#!/bin/sh\nexit 0\n").unwrap();
            std::fs::set_permissions(&executable, std::fs::Permissions::from_mode(0o755)).unwrap();
        }

        let mut paths = vec![dir.path().to_path_buf()];
        paths.extend(std::env::split_paths(
            &std::env::var_os("PATH").unwrap_or_default(),
        ));
        std::env::set_var("PATH", std::env::join_paths(paths).unwrap());
        dir
    });
}

fn input(repo: &std::path::Path) -> FlowDraftInput {
    ensure_test_ade();
    FlowDraftInput {
        id: "flow-1".into(),
        title: "Ship it".into(),
        mission_text: "Update src/lib.rs".into(),
        source: FlowSource::Text,
        domain_id: Some(repo.to_string_lossy().into_owned()),
        project_id: None,
        ade_id: Some("codex".into()),
    }
}

fn seed_mission_path(repo: &std::path::Path) {
    std::fs::create_dir_all(repo.join("src")).unwrap();
    std::fs::write(repo.join("src/lib.rs"), "pub fn value() {}\n").unwrap();
}

#[test]
fn flow_rejects_empty_mission_and_missing_domain() {
    let dir = tempfile::tempdir().unwrap();
    let catalog = Catalog::open(&dir.path().join("catalog.db")).unwrap();
    let mut empty = input(dir.path());
    empty.mission_text = "  ".into();
    assert!(preview_flow(&catalog, empty).is_err());

    let mut missing = input(dir.path());
    missing.domain_id = None;
    assert!(preview_flow(&catalog, missing).is_err());
}

#[test]
fn saving_draft_intent_cannot_create_execution_state() {
    let dir = tempfile::tempdir().unwrap();
    let catalog = Catalog::open(&dir.path().join("catalog.db")).unwrap();
    let saved = save_flow_draft(&catalog, input(dir.path())).unwrap();
    assert_eq!(saved.status, "draft");
    assert!(saved.plan_json.is_none());
    assert!(saved.dispatched_run_id.is_none());
}

#[test]
fn valid_flow_is_persisted_ready_with_execution_metadata() {
    let dir = tempfile::tempdir().unwrap();
    std::fs::create_dir(dir.path().join("src")).unwrap();
    std::fs::write(dir.path().join("src/lib.rs"), "pub fn value() {}\n").unwrap();
    std::fs::write(
        dir.path().join("pytxo.toml"),
        "permission_profile = \"galaxy\"\nexecution_backend = \"subprocess\"\n",
    )
    .unwrap();
    let catalog = Catalog::open(&dir.path().join("catalog.db")).unwrap();

    let plan = preview_flow(&catalog, input(dir.path())).unwrap();
    assert_eq!(plan.status, FlowStatus::Ready);
    assert_eq!(plan.permission_profile, "galaxy");
    assert_eq!(plan.execution_backend, "subprocess");
    assert!(!plan.tasks.is_empty());
    assert!(!plan.waves.is_empty());
    assert!(plan.ade.available);
    assert!(plan.ade.requested.is_some());
    let persisted = catalog.get_flow_draft("flow-1").unwrap().unwrap();
    assert_eq!(persisted.status, "ready");
    assert!(persisted.plan_json.is_some());
}

#[test]
fn staged_overlapping_claims_warn_and_unavailable_requested_ade_blocks_preview() {
    let dir = tempfile::tempdir().unwrap();
    seed_mission_path(dir.path());
    std::fs::write(
        dir.path().join("pytxo.toml"),
        r#"
[[task]]
id = "one"
agent = "a"
paths = ["src/shared.rs"]

[[task]]
id = "two"
agent = "b"
paths = ["src/shared.rs"]
"#,
    )
    .unwrap();
    let catalog = Catalog::open(&dir.path().join("catalog.db")).unwrap();
    let overlap = preview_flow(&catalog, input(dir.path())).unwrap();
    assert_eq!(overlap.status, FlowStatus::Ready);
    assert!(!overlap
        .blocked_reasons
        .iter()
        .any(|reason| matches!(reason, FlowBlockedReason::OverlappingPathClaims { .. })));
    assert!(overlap
        .warnings
        .iter()
        .any(|warning| warning.code == "path_claim_staged"));

    let other = tempfile::tempdir().unwrap();
    seed_mission_path(other.path());
    let other_catalog = Catalog::open(&other.path().join("catalog.db")).unwrap();
    let mut unavailable = input(other.path());
    unavailable.ade_id = Some("definitely-not-an-installed-ade".into());
    let plan = preview_flow(&other_catalog, unavailable).unwrap();
    assert_eq!(plan.status, FlowStatus::Blocked);
    assert!(!plan.ade.available);
}

#[test]
fn dispatch_requires_a_persisted_ready_preview() {
    let dir = tempfile::tempdir().unwrap();
    let catalog = Catalog::open(&dir.path().join("catalog.db")).unwrap();
    let now = Utc::now().to_rfc3339();
    catalog
        .upsert_flow_draft(&FlowDraftRecord {
            id: "draft-only".into(),
            title: "Draft".into(),
            mission_text: "Do work".into(),
            source: "text".into(),
            domain_id: Some(dir.path().to_string_lossy().into_owned()),
            project_id: None,
            status: "draft".into(),
            plan_json: None,
            dispatched_run_id: None,
            created_at: now.clone(),
            updated_at: now,
        })
        .unwrap();
    let err = dispatch_flow(&catalog, "draft-only").unwrap_err();
    assert!(err.to_string().contains("ready preview"));
}

#[test]
fn a_claimed_flow_cannot_be_dispatched_twice() {
    let dir = tempfile::tempdir().unwrap();
    seed_mission_path(dir.path());
    let catalog = Catalog::open(&dir.path().join("catalog.db")).unwrap();
    preview_flow(&catalog, input(dir.path())).unwrap();
    let expected = catalog
        .get_flow_draft("flow-1")
        .unwrap()
        .unwrap()
        .plan_json
        .unwrap();
    assert!(catalog.claim_flow_dispatch("flow-1", &expected).unwrap());
    assert!(!catalog.claim_flow_dispatch("flow-1", &expected).unwrap());

    let error = dispatch_flow(&catalog, "flow-1").unwrap_err();
    assert!(error.to_string().contains("ready preview"));
}

#[test]
fn task_cannot_escalate_above_domain_permission_profile() {
    let dir = tempfile::tempdir().unwrap();
    std::fs::write(
        dir.path().join("pytxo.toml"),
        r#"
permission_profile = "orbit"

[[agent]]
name = "elevated"
permission_profile = "supernova"

[[task]]
id = "unsafe"
agent = "elevated"
paths = ["src/lib.rs"]
"#,
    )
    .unwrap();
    let catalog = Catalog::open(&dir.path().join("catalog.db")).unwrap();
    let plan = preview_flow(&catalog, input(dir.path())).unwrap();
    assert_eq!(plan.status, FlowStatus::Blocked);
    assert!(plan
        .blocked_reasons
        .iter()
        .any(|reason| matches!(reason, FlowBlockedReason::PermissionViolation { .. })));
}

#[test]
fn reviewed_prompt_edits_replace_the_persisted_dispatch_snapshot() {
    let dir = tempfile::tempdir().unwrap();
    std::fs::create_dir(dir.path().join("src")).unwrap();
    std::fs::write(dir.path().join("src/lib.rs"), "pub fn value() {}\n").unwrap();
    let catalog = Catalog::open(&dir.path().join("catalog.db")).unwrap();
    let mut reviewed = preview_flow(&catalog, input(dir.path())).unwrap();
    reviewed.tasks[0].prompt = "Use the reviewed implementation constraint".into();

    let saved = save_reviewed_flow_plan(&catalog, reviewed).unwrap();
    assert_eq!(
        saved.tasks[0].prompt,
        "Use the reviewed implementation constraint"
    );
    let persisted = catalog.get_flow_draft("flow-1").unwrap().unwrap();
    let persisted_plan: pytxo_orchestrate::FlowPlan =
        serde_json::from_str(persisted.plan_json.as_deref().unwrap()).unwrap();
    assert_eq!(persisted_plan.tasks[0].prompt, saved.tasks[0].prompt);
}

#[test]
fn reviewed_plan_cannot_change_execution_structure() {
    let dir = tempfile::tempdir().unwrap();
    seed_mission_path(dir.path());
    let catalog = Catalog::open(&dir.path().join("catalog.db")).unwrap();
    let mut reviewed = preview_flow(&catalog, input(dir.path())).unwrap();
    reviewed.tasks[0].paths = vec!["outside/**".into()];

    let error = save_reviewed_flow_plan(&catalog, reviewed).unwrap_err();
    assert!(error.to_string().contains("structure changed"));
}
