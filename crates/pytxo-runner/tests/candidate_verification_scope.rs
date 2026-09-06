//! Negative controls for verification scope: immutable review packaging proves
//! bytes, not that the composition has passed a behavioral check.
use std::path::Path;
use std::process::Command;

use pytxo_runner::{apply_prepared_review, prepare_review_package, AgentWorkspaceInput};

fn write_state(root: &Path, a: &str, b: &str) {
    std::fs::create_dir_all(root).unwrap();
    std::fs::write(root.join("a.txt"), a).unwrap();
    std::fs::write(root.join("b.txt"), b).unwrap();
}

fn check(root: &Path) -> bool {
    // Cross-file invariant: at most one feature can hold the shared resource.
    // Each task preserves the invariant alone, but their union violates it.
    #[cfg(windows)]
    let output = Command::new("powershell")
        .args(["-NoProfile", "-NonInteractive", "-Command",
            "if ((Get-Content -Raw a.txt).Trim() -eq '1' -and (Get-Content -Raw b.txt).Trim() -eq '1') { exit 1 }; exit 0"])
        .current_dir(root)
        .output()
        .unwrap();
    #[cfg(not(windows))]
    let output = Command::new("sh")
        .args([
            "-c",
            "! { [ \"$(cat a.txt)\" = 1 ] && [ \"$(cat b.txt)\" = 1 ]; }",
        ])
        .current_dir(root)
        .output()
        .unwrap();
    output.status.success()
}

fn input(root: &Path, task: &str, claims: &[&str]) -> AgentWorkspaceInput {
    AgentWorkspaceInput {
        agent_id: format!("agent-{task}"),
        task_id: task.into(),
        workspace_path: root.into(),
        claims: claims.iter().map(|path| (*path).into()).collect(),
        depends_on: vec![],
    }
}

#[test]
fn individually_passing_workspaces_can_produce_a_failing_exact_candidate() {
    let temp = tempfile::tempdir().unwrap();
    let repo = temp.path().join("repo");
    let a = temp.path().join("task-a");
    let b = temp.path().join("task-b");
    let data = temp.path().join("data");
    write_state(&repo, "0", "0");
    write_state(&a, "1", "0");
    write_state(&b, "0", "1");

    assert!(check(&repo));
    assert!(check(&a), "task A check passes in its own workspace");
    assert!(check(&b), "task B check passes in its own workspace");
    let manifest = prepare_review_package(
        &repo,
        &data,
        "run-combined-audit",
        "base",
        &[input(&a, "a", &["a.txt"]), input(&b, "b", &["b.txt"])],
        &[],
    )
    .unwrap();
    assert_eq!(manifest.files.len(), 2);
    apply_prepared_review(&repo, &data, &manifest).unwrap();
    assert!(
        !check(&repo),
        "exact prepared bytes violate the invariant despite both task checks passing"
    );
}

#[test]
fn packaging_after_a_check_does_not_bind_the_checked_workspace_bytes() {
    let temp = tempfile::tempdir().unwrap();
    let repo = temp.path().join("repo");
    let workspace = temp.path().join("task");
    let data = temp.path().join("data");
    write_state(&repo, "0", "0");
    write_state(&workspace, "1", "0");
    assert!(check(&workspace));
    let checked_before_refresh = prepare_review_package(
        &repo,
        &data,
        "run-mutated-audit",
        "base",
        &[input(&workspace, "task", &["a.txt", "b.txt"])],
        &[],
    )
    .unwrap();
    // A later successful verifier command can write source after the earlier
    // check. Preparation sees the final bytes and has no earlier check digest.
    std::fs::write(workspace.join("b.txt"), "1").unwrap();
    let manifest = prepare_review_package(
        &repo,
        &data,
        "run-mutated-audit",
        "base",
        &[input(&workspace, "task", &["a.txt", "b.txt"])],
        &[],
    )
    .unwrap();
    assert_ne!(
        checked_before_refresh.package_digest, manifest.package_digest,
        "refresh changes package identity without rerunning the earlier check"
    );
    apply_prepared_review(&repo, &data, &manifest).unwrap();
    assert!(
        !check(&repo),
        "a prior successful check cannot attest later bytes"
    );
}
use pytxo_core::CandidateCheckEvidence;
use pytxo_runner::{load_review_manifest, require_candidate_verification, CandidateVerification};

fn receipt(passed: bool) -> CandidateCheckEvidence {
    CandidateCheckEvidence {
        task_id: "task".into(),
        command: "check shared resource".into(),
        effective_profile: "orbit".into(),
        passed,
        enforcement: serde_json::json!({"actor": "verification", "effective_profile": "orbit"}),
    }
}

#[test]
fn combined_candidate_failure_cannot_be_applied() {
    let temp = tempfile::tempdir().unwrap();
    let repo = temp.path().join("repo");
    let a = temp.path().join("task-a");
    let b = temp.path().join("task-b");
    let data = temp.path().join("data");
    write_state(&repo, "0", "0");
    write_state(&a, "1", "0");
    write_state(&b, "0", "1");
    let manifest = prepare_review_package(
        &repo,
        &data,
        "combined",
        "base",
        &[input(&a, "a", &["a.txt"]), input(&b, "b", &["b.txt"])],
        &[],
    )
    .unwrap();
    let candidate = CandidateVerification::prepare(&repo, &data, &manifest, &[]).unwrap();
    let passed = check(candidate.workspace_root());
    candidate.check_unchanged().unwrap();
    assert!(!passed);
    assert!(candidate.finish(vec![receipt(passed)]).is_err());
    let failed = load_review_manifest(&data, "combined").unwrap();
    assert_eq!(failed.version, 3);
    assert!(require_candidate_verification(&failed).is_err());
    assert!(apply_prepared_review(&repo, &data, &failed).is_err());
    assert_eq!(std::fs::read_to_string(repo.join("a.txt")).unwrap(), "0");
}

#[test]
fn verified_candidate_binds_unchanged_inputs_and_applies_frozen_bytes() {
    let temp = tempfile::tempdir().unwrap();
    let repo = temp.path().join("repo");
    let task = temp.path().join("task");
    let data = temp.path().join("data");
    write_state(&repo, "0", "0");
    write_state(&task, "1", "0");
    let manifest = prepare_review_package(
        &repo,
        &data,
        "verified",
        "base",
        &[input(&task, "task", &["a.txt"])],
        &[],
    )
    .unwrap();
    let candidate = CandidateVerification::prepare(&repo, &data, &manifest, &[]).unwrap();
    assert!(check(candidate.workspace_root()));
    candidate.check_unchanged().unwrap();
    let verified = candidate.finish(vec![receipt(true)]).unwrap();
    assert_eq!(verified.version, 3);
    assert_eq!(
        require_candidate_verification(&verified)
            .unwrap()
            .base_inventory
            .len(),
        2
    );
    std::fs::write(repo.join("b.txt"), "drift").unwrap();
    assert!(apply_prepared_review(&repo, &data, &verified)
        .unwrap_err()
        .to_string()
        .contains("inventory drifted"));
    std::fs::write(repo.join("b.txt"), "0").unwrap();
    std::fs::write(task.join("a.txt"), "later unchecked worker bytes").unwrap();
    apply_prepared_review(&repo, &data, &verified).unwrap();
    assert_eq!(std::fs::read_to_string(repo.join("a.txt")).unwrap(), "1");
}

#[test]
fn verifier_mutations_are_rejected_between_commands() {
    let temp = tempfile::tempdir().unwrap();
    let repo = temp.path().join("repo");
    let task = temp.path().join("task");
    let data = temp.path().join("data");
    write_state(&repo, "0", "0");
    write_state(&task, "1", "0");
    let manifest = prepare_review_package(
        &repo,
        &data,
        "mutation",
        "base",
        &[input(&task, "task", &["a.txt"])],
        &[],
    )
    .unwrap();
    let candidate = CandidateVerification::prepare(&repo, &data, &manifest, &[]).unwrap();
    assert!(check(candidate.workspace_root()));
    std::fs::write(candidate.workspace_root().join("b.txt"), "1").unwrap();
    assert!(candidate.check_unchanged().is_err());
    assert!(candidate.finish(vec![receipt(true)]).is_err());
}

#[test]
fn primary_drift_during_verification_and_empty_recipes_are_rejected() {
    let temp = tempfile::tempdir().unwrap();
    let repo = temp.path().join("repo");
    let task = temp.path().join("task");
    let data = temp.path().join("data");
    write_state(&repo, "0", "0");
    write_state(&task, "1", "0");
    let manifest = prepare_review_package(
        &repo,
        &data,
        "primary-drift",
        "base",
        &[input(&task, "task", &["a.txt"])],
        &[],
    )
    .unwrap();
    let candidate = CandidateVerification::prepare(&repo, &data, &manifest, &[]).unwrap();
    std::fs::write(repo.join("new-source.txt"), "external edit").unwrap();
    assert!(candidate
        .finish(vec![receipt(true)])
        .unwrap_err()
        .to_string()
        .contains("repository inputs changed"));
    std::fs::remove_file(repo.join("new-source.txt")).unwrap();
    let current = load_review_manifest(&data, "primary-drift").unwrap();
    let candidate = CandidateVerification::prepare(&repo, &data, &current, &[]).unwrap();
    assert!(candidate.finish(vec![]).is_err());
}

#[test]
fn explicit_output_exclusions_do_not_allow_changed_inputs_to_escape_attestation() {
    let temp = tempfile::tempdir().unwrap();
    let repo = temp.path().join("repo");
    let task = temp.path().join("task");
    let data = temp.path().join("data");
    write_state(&repo, "0", "0");
    write_state(&task, "1", "0");
    let manifest = prepare_review_package(
        &repo,
        &data,
        "excluded",
        "base",
        &[input(&task, "task", &["a.txt"])],
        &[],
    )
    .unwrap();
    let candidate =
        CandidateVerification::prepare(&repo, &data, &manifest, &["build".into()]).unwrap();
    std::fs::create_dir(candidate.workspace_root().join("build")).unwrap();
    std::fs::write(candidate.workspace_root().join("build/output"), "generated").unwrap();
    candidate.check_unchanged().unwrap();
    let verified = candidate.finish(vec![receipt(true)]).unwrap();
    assert!(verified
        .candidate_verification
        .as_ref()
        .unwrap()
        .exclusions
        .contains(&"build".into()));
    let candidate =
        CandidateVerification::prepare(&repo, &data, &verified, &["a.txt".into()]).unwrap();
    assert!(candidate.finish(vec![receipt(true)]).is_err());
}

#[test]
fn candidate_git_commands_must_not_discover_primary_repository() {
    let temp = tempfile::tempdir().unwrap();
    let repo = temp.path().join("repo");
    let task = temp.path().join("task");
    let data = repo.join(".pytxo/data");
    write_state(&repo, "0", "0");
    write_state(&task, "1", "0");
    assert!(Command::new("git")
        .args(["init", "-q"])
        .current_dir(&repo)
        .status()
        .unwrap()
        .success());
    let manifest = prepare_review_package(
        &repo,
        &data,
        "git-boundary",
        "base",
        &[input(&task, "task", &["a.txt"])],
        &[],
    )
    .unwrap();
    let candidate = CandidateVerification::prepare(&repo, &data, &manifest, &[]).unwrap();
    let result = Command::new("git")
        .args(["rev-parse", "--show-toplevel"])
        .current_dir(candidate.workspace_root())
        .output()
        .unwrap();
    if result.status.success() {
        let discovered = String::from_utf8(result.stdout).unwrap();
        assert_ne!(
            std::fs::canonicalize(discovered.trim()).unwrap(),
            std::fs::canonicalize(&repo).unwrap(),
            "candidate Git verification escaped upward to the primary repository"
        );
    }
}
