use chrono::Utc;
use pytxo_orchestrate::{
    dispatch_flow, preview_flow, FlowBlockedReason, FlowDraftInput, FlowSource, FlowStatus,
};
use pytxo_store::{Catalog, FlowDraftRecord};

fn input(repo: &std::path::Path) -> FlowDraftInput {
    FlowDraftInput {
        id: "flow-1".into(),
        title: "Ship it".into(),
        mission_text: "Update src/lib.rs".into(),
        source: FlowSource::Text,
        domain_id: Some(repo.to_string_lossy().into_owned()),
        project_id: None,
        ade_id: None,
    }
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
fn overlapping_claims_and_unavailable_requested_ade_block_preview() {
    let dir = tempfile::tempdir().unwrap();
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
    assert_eq!(overlap.status, FlowStatus::Blocked);
    assert!(overlap
        .blocked_reasons
        .iter()
        .any(|reason| matches!(reason, FlowBlockedReason::OverlappingPathClaims { .. })));

    let other = tempfile::tempdir().unwrap();
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
