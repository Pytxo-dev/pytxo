use std::path::{Path, PathBuf};

use pytxo_runner::{
    apply_attempt_ids, apply_prepared_review, apply_prepared_review_under_lease,
    apply_prepared_review_with_fault, load_review_manifest, load_review_package,
    prepare_review_package, prepare_run_change_set, read_review_content_chunk,
    read_review_content_chunk_with_fault, read_review_content_chunk_with_metrics,
    reconcile_apply_journals, AgentWorkspaceInput, ApplyFaultPoint, ExecutionDomainMutationLease,
    RecoveryOutcome, ReviewContentSide, ReviewReadFaultPoint, ReviewReadMetrics, RunChangeKind,
};
use sha2::{Digest, Sha256};

fn write(path: &Path, contents: &str) {
    std::fs::create_dir_all(path.parent().expect("fixture parent")).expect("create fixture parent");
    std::fs::write(path, contents).expect("write fixture");
}

#[test]
fn interrupted_apply_recovery_restores_files_and_removes_created_directories_idempotently() {
    let temp = tempfile::tempdir().expect("tempdir");
    let repo = temp.path().join("repo");
    let data_dir = temp.path().join("domain-data");
    write(&repo.join("existing.txt"), "before\n");
    let workspace = temp.path().join("workspace");
    write(&workspace.join("existing.txt"), "after\n");
    write(&workspace.join("new/deep/added.txt"), "added\n");
    let manifest = prepare_review_package(
        &repo,
        &data_dir,
        "run-recover",
        "base",
        &[AgentWorkspaceInput {
            agent_id: "agent-0".into(),
            task_id: "implement".into(),
            workspace_path: workspace,
            claims: vec!["existing.txt".into(), "new".into()],
            depends_on: vec![],
        }],
        &[],
    )
    .unwrap();

    apply_prepared_review_with_fault(
        &repo,
        &data_dir,
        &manifest,
        Some(ApplyFaultPoint::InterruptAfterRename(2)),
    )
    .expect_err("simulated interruption must leave a recoverable journal");
    assert_eq!(
        std::fs::read_to_string(repo.join("existing.txt")).unwrap(),
        "after\n"
    );
    assert!(repo.join("new/deep/added.txt").is_file());

    assert!(matches!(
        reconcile_apply_journals(&repo, &data_dir, "run-recover").unwrap(),
        RecoveryOutcome::RolledBack { .. }
    ));
    assert_eq!(
        std::fs::read_to_string(repo.join("existing.txt")).unwrap(),
        "before\n"
    );
    assert!(!repo.join("new").exists());
    assert_eq!(
        reconcile_apply_journals(&repo, &data_dir, "run-recover").unwrap(),
        RecoveryOutcome::NothingToDo
    );
}

#[test]
fn journal_prepared_recovery_preserves_human_created_planned_directory() {
    let temp = tempfile::tempdir().expect("tempdir");
    let repo = temp.path().join("repo");
    let data_dir = temp.path().join("data");
    std::fs::create_dir_all(&repo).unwrap();
    let workspace = temp.path().join("workspace");
    write(&workspace.join("planned/added.txt"), "agent bytes\n");
    let manifest = prepare_review_package(
        &repo,
        &data_dir,
        "run-prepared-directory",
        "base",
        &[AgentWorkspaceInput {
            agent_id: "agent".into(),
            task_id: "task".into(),
            workspace_path: workspace,
            claims: vec!["planned/added.txt".into()],
            depends_on: vec![],
        }],
        &[],
    )
    .unwrap();

    apply_prepared_review_with_fault(
        &repo,
        &data_dir,
        &manifest,
        Some(ApplyFaultPoint::InterruptAfterJournalPrepared),
    )
    .expect_err("interrupt before repository mutation");
    std::fs::create_dir_all(repo.join("planned")).unwrap();

    assert!(matches!(
        reconcile_apply_journals(&repo, &data_dir, "run-prepared-directory").unwrap(),
        RecoveryOutcome::RolledBack { .. }
    ));
    assert!(
        repo.join("planned").is_dir(),
        "recovery must not delete a directory Apply never created"
    );
}

#[test]
fn directory_intent_is_durable_before_creation_and_preserves_post_crash_human_directory() {
    let temp = tempfile::tempdir().expect("tempdir");
    let repo = temp.path().join("repo");
    let data_dir = temp.path().join("data");
    std::fs::create_dir_all(&repo).unwrap();
    let workspace = temp.path().join("workspace");
    write(&workspace.join("planned/deep/added.txt"), "agent bytes\n");
    let run_id = "run-directory-intent";
    let manifest = prepare_review_package(
        &repo,
        &data_dir,
        run_id,
        "base",
        &[AgentWorkspaceInput {
            agent_id: "agent".into(),
            task_id: "task".into(),
            workspace_path: workspace,
            claims: vec!["planned/deep/added.txt".into()],
            depends_on: vec![],
        }],
        &[],
    )
    .unwrap();

    apply_prepared_review_with_fault(
        &repo,
        &data_dir,
        &manifest,
        Some(ApplyFaultPoint::InterruptAfterDirectoryIntent(1)),
    )
    .expect_err("interrupt after the directory intent is durable");

    assert!(
        !repo.join("planned").exists(),
        "the ownership intent must be journaled before repository directory creation"
    );
    let attempt_id = apply_attempt_ids(&data_dir, run_id)
        .unwrap()
        .into_iter()
        .next()
        .expect("attempt id");
    let journal: serde_json::Value = serde_json::from_slice(
        &std::fs::read(
            data_dir
                .join("apply")
                .join(run_id)
                .join(attempt_id)
                .join("journal.json"),
        )
        .unwrap(),
    )
    .unwrap();
    assert_eq!(journal["created_directories"][0]["path"], "planned");
    assert_eq!(journal["created_directories"][0]["stage"], "intent");

    std::fs::create_dir_all(repo.join("planned")).unwrap();
    assert!(matches!(
        reconcile_apply_journals(&repo, &data_dir, run_id).unwrap(),
        RecoveryOutcome::RolledBack { .. }
    ));
    assert!(
        repo.join("planned").is_dir(),
        "recovery must preserve a directory whose durable intent never reached created"
    );
}

#[test]
fn recovery_refuses_to_overwrite_post_crash_human_drift() {
    let temp = tempfile::tempdir().expect("tempdir");
    let repo = temp.path().join("repo");
    let data_dir = temp.path().join("data");
    write(&repo.join("owned.txt"), "before\n");
    let workspace = temp.path().join("workspace");
    write(&workspace.join("owned.txt"), "after\n");
    let manifest = prepare_review_package(
        &repo,
        &data_dir,
        "run-human-drift",
        "base",
        &[AgentWorkspaceInput {
            agent_id: "agent".into(),
            task_id: "task".into(),
            workspace_path: workspace,
            claims: vec!["owned.txt".into()],
            depends_on: vec![],
        }],
        &[],
    )
    .unwrap();
    apply_prepared_review_with_fault(
        &repo,
        &data_dir,
        &manifest,
        Some(ApplyFaultPoint::InterruptAfterRename(1)),
    )
    .expect_err("interrupt");
    write(&repo.join("owned.txt"), "human-after-crash\n");

    assert!(matches!(
        reconcile_apply_journals(&repo, &data_dir, "run-human-drift").unwrap(),
        RecoveryOutcome::RecoveryRequired {
            attempt_id: Some(_)
        }
    ));
    assert_eq!(
        std::fs::read_to_string(repo.join("owned.txt")).unwrap(),
        "human-after-crash\n"
    );
}

#[test]
fn recovery_handles_each_temp_file_interruption_boundary() {
    let cases = [
        (
            ApplyFaultPoint::InterruptAfterTempWrite(1),
            "temp_written",
            Some("before\n"),
            true,
        ),
        (
            ApplyFaultPoint::InterruptAfterDestinationRemoval(1),
            "destination_removed",
            None,
            true,
        ),
        (
            ApplyFaultPoint::InterruptAfterRename(1),
            "renamed",
            Some("after\n"),
            false,
        ),
    ];
    for (case, (fault, expected_stage, expected_target, expected_temp)) in
        cases.into_iter().enumerate()
    {
        let temp = tempfile::tempdir().expect("tempdir");
        let repo = temp.path().join("repo");
        let data_dir = temp.path().join("data");
        write(&repo.join("owned.txt"), "before\n");
        let workspace = temp.path().join("workspace");
        write(&workspace.join("owned.txt"), "after\n");
        let run_id = format!("run-boundary-{case}");
        let manifest = prepare_review_package(
            &repo,
            &data_dir,
            &run_id,
            "base",
            &[AgentWorkspaceInput {
                agent_id: "agent".into(),
                task_id: "task".into(),
                workspace_path: workspace,
                claims: vec!["owned.txt".into()],
                depends_on: vec![],
            }],
            &[],
        )
        .unwrap();
        apply_prepared_review_with_fault(&repo, &data_dir, &manifest, Some(fault))
            .expect_err("interrupt");

        let attempt_root = std::fs::read_dir(data_dir.join("apply").join(&run_id))
            .unwrap()
            .next()
            .unwrap()
            .unwrap()
            .path();
        let journal: serde_json::Value =
            serde_json::from_slice(&std::fs::read(attempt_root.join("journal.json")).unwrap())
                .unwrap();
        assert_eq!(journal["in_flight_stage"], expected_stage);
        let temp_path = journal["operations"][0]["temporary_path"].as_str().unwrap();
        assert_eq!(repo.join(temp_path).exists(), expected_temp);
        assert_eq!(
            std::fs::read_to_string(repo.join("owned.txt"))
                .ok()
                .as_deref(),
            expected_target
        );

        assert!(matches!(
            reconcile_apply_journals(&repo, &data_dir, &run_id).unwrap(),
            RecoveryOutcome::RolledBack { .. }
        ));
        assert_eq!(
            std::fs::read_to_string(repo.join("owned.txt")).unwrap(),
            "before\n"
        );
        assert!(!repo.join(temp_path).exists());
    }
}

#[cfg(unix)]
fn replace_directory_with_link(directory: &Path, outside: &Path) {
    std::fs::remove_dir(directory).unwrap();
    std::os::unix::fs::symlink(outside, directory).unwrap();
}

#[cfg(windows)]
fn replace_directory_with_link(directory: &Path, outside: &Path) {
    std::fs::remove_dir(directory).unwrap();
    let output = std::process::Command::new("cmd")
        .arg("/C")
        .arg("mklink")
        .arg("/J")
        .arg(directory)
        .arg(outside)
        .output()
        .expect("create junction");
    assert!(output.status.success(), "{output:?}");
}

#[test]
fn recovery_revalidates_ancestors_before_rollback() {
    let temp = tempfile::tempdir().expect("tempdir");
    let repo = temp.path().join("repo");
    let data_dir = temp.path().join("data");
    let outside = temp.path().join("outside");
    write(&repo.join("owned/file.txt"), "before\n");
    write(&outside.join("file.txt"), "outside\n");
    let workspace = temp.path().join("workspace");
    write(&workspace.join("owned/file.txt"), "after\n");
    let manifest = prepare_review_package(
        &repo,
        &data_dir,
        "run-ancestor",
        "base",
        &[AgentWorkspaceInput {
            agent_id: "agent".into(),
            task_id: "task".into(),
            workspace_path: workspace,
            claims: vec!["owned/file.txt".into()],
            depends_on: vec![],
        }],
        &[],
    )
    .unwrap();
    apply_prepared_review_with_fault(
        &repo,
        &data_dir,
        &manifest,
        Some(ApplyFaultPoint::InterruptAfterRename(1)),
    )
    .expect_err("interrupt");
    std::fs::remove_file(repo.join("owned/file.txt")).unwrap();
    replace_directory_with_link(&repo.join("owned"), &outside);

    assert!(matches!(
        reconcile_apply_journals(&repo, &data_dir, "run-ancestor").unwrap(),
        RecoveryOutcome::RecoveryRequired { .. }
    ));
    assert_eq!(
        std::fs::read_to_string(outside.join("file.txt")).unwrap(),
        "outside\n"
    );
}

#[cfg(windows)]
#[test]
fn windows_protected_paths_are_case_insensitive() {
    use sha2::{Digest, Sha256};

    let temp = tempfile::tempdir().expect("tempdir");
    let repo = temp.path().join("repo");
    let data_dir = temp.path().join("data");
    let workspace = temp.path().join("workspace");
    write(&repo.join("safe.txt"), "before\n");
    write(&workspace.join("safe.txt"), "after\n");
    for claim in [".GIT/config", ".PYTXO/review.json"] {
        let error = prepare_run_change_set(
            &repo,
            &[AgentWorkspaceInput {
                agent_id: "agent".into(),
                task_id: "task".into(),
                workspace_path: workspace.clone(),
                claims: vec![claim.into()],
                depends_on: vec![],
            }],
            &[],
        )
        .expect_err("mixed-case protected claim must be rejected");
        assert!(error.to_string().contains("unsafe run change-set claim"));
    }
    for protected in [".GIT/config", ".PYTXO/review.json"] {
        let malicious_workspace = temp
            .path()
            .join(format!("workspace-{}", protected.replace(['/', '.'], "-")));
        write(&malicious_workspace.join("safe.txt"), "after\n");
        write(
            &malicious_workspace.join(protected),
            "must-not-be-inventoried\n",
        );
        let error = prepare_run_change_set(
            &repo,
            &[AgentWorkspaceInput {
                agent_id: "agent".into(),
                task_id: "task".into(),
                workspace_path: malicious_workspace,
                claims: vec!["safe.txt".into()],
                depends_on: vec![],
            }],
            &[],
        )
        .expect_err("mixed-case protected inventory entry must be rejected");
        assert!(error.to_string().contains("protected run change path"));
    }

    let mut manifest = prepare_review_package(
        &repo,
        &data_dir,
        "run-protected-load",
        "base",
        &[AgentWorkspaceInput {
            agent_id: "agent".into(),
            task_id: "task".into(),
            workspace_path: workspace.clone(),
            claims: vec!["safe.txt".into()],
            depends_on: vec![],
        }],
        &[],
    )
    .unwrap();
    manifest.files[0].path = ".GIT/config".into();
    manifest.package_digest.clear();
    manifest.package_digest = format!(
        "{:x}",
        Sha256::digest(serde_json::to_vec(&manifest).unwrap())
    );
    std::fs::write(
        data_dir.join("reviews/run-protected-load/manifest.json"),
        serde_json::to_vec_pretty(&manifest).unwrap(),
    )
    .unwrap();
    assert!(load_review_package(&data_dir, "run-protected-load")
        .unwrap_err()
        .to_string()
        .contains("protected run change path"));

    let manifest = prepare_review_package(
        &repo,
        &data_dir,
        "run-protected-recovery",
        "base",
        &[AgentWorkspaceInput {
            agent_id: "agent".into(),
            task_id: "task".into(),
            workspace_path: workspace,
            claims: vec!["safe.txt".into()],
            depends_on: vec![],
        }],
        &[],
    )
    .unwrap();
    apply_prepared_review_with_fault(
        &repo,
        &data_dir,
        &manifest,
        Some(ApplyFaultPoint::InterruptAfterRename(1)),
    )
    .expect_err("interrupt");
    let attempt_root = std::fs::read_dir(data_dir.join("apply/run-protected-recovery"))
        .unwrap()
        .next()
        .unwrap()
        .unwrap()
        .path();
    let journal_path = attempt_root.join("journal.json");
    let mut journal: serde_json::Value =
        serde_json::from_slice(&std::fs::read(&journal_path).unwrap()).unwrap();
    journal["operations"][0]["path"] = ".PYTXO/recovery-target".into();
    std::fs::write(journal_path, serde_json::to_vec_pretty(&journal).unwrap()).unwrap();
    assert!(matches!(
        reconcile_apply_journals(&repo, &data_dir, "run-protected-recovery").unwrap(),
        RecoveryOutcome::RecoveryRequired { .. }
    ));
}

#[test]
fn mutation_boundary_drift_is_preserved_and_apply_is_rejected() {
    let temp = tempfile::tempdir().expect("tempdir");
    let repo = temp.path().join("repo");
    let data_dir = temp.path().join("domain-data");
    write(&repo.join("owned.txt"), "reviewed-base\n");
    let workspace = temp.path().join("workspace-boundary");
    write(&workspace.join("owned.txt"), "agent-result\n");
    let manifest = prepare_review_package(
        &repo,
        &data_dir,
        "run-boundary",
        "base",
        &[AgentWorkspaceInput {
            agent_id: "agent-0".into(),
            task_id: "implement".into(),
            workspace_path: workspace,
            claims: vec!["owned.txt".into()],
            depends_on: vec![],
        }],
        &[],
    )
    .unwrap();

    let error = apply_prepared_review_with_fault(
        &repo,
        &data_dir,
        &manifest,
        Some(ApplyFaultPoint::DriftBeforeMutation(1)),
    )
    .expect_err("last-moment source drift must reject Apply");

    assert!(error.to_string().contains("immediately before mutation"));
    assert_eq!(
        std::fs::read_to_string(repo.join("owned.txt")).unwrap(),
        "injected source drift\n"
    );
}

#[test]
fn immutable_review_package_round_trips_binary_add_modify_delete_after_workspace_removal() {
    let temp = tempfile::tempdir().expect("tempdir");
    let repo = temp.path().join("repo");
    let data_dir = temp.path().join("domain-data");
    write(&repo.join("src/modified.txt"), "before\n");
    write(&repo.join("src/deleted.txt"), "delete\n");
    let agent_workspace = temp.path().join("copy-overlay-workspace");
    write(&agent_workspace.join("src/modified.txt"), "after\n");
    std::fs::create_dir_all(agent_workspace.join("src")).unwrap();
    std::fs::write(agent_workspace.join("src/binary.bin"), [0, 255, 1, 128]).unwrap();

    let manifest = prepare_review_package(
        &repo,
        &data_dir,
        "run-immutable",
        "base-revision",
        &[AgentWorkspaceInput {
            agent_id: "agent-0".into(),
            task_id: "implement".into(),
            workspace_path: agent_workspace.clone(),
            claims: vec!["src".into()],
            depends_on: vec![],
        }],
        &[],
    )
    .expect("prepare durable review");
    assert_eq!(manifest.summary.added, 1);
    assert_eq!(manifest.summary.modified, 1);
    assert_eq!(manifest.summary.deleted, 1);
    assert!(data_dir
        .join("reviews/run-immutable/manifest.json")
        .is_file());

    std::fs::remove_dir_all(agent_workspace).unwrap();
    let loaded = load_review_package(&data_dir, "run-immutable").expect("load immutable package");
    assert_eq!(loaded, manifest);
    apply_prepared_review(&repo, &data_dir, &loaded).expect("apply immutable package");

    assert_eq!(
        std::fs::read(repo.join("src/binary.bin")).unwrap(),
        [0, 255, 1, 128]
    );
    assert_eq!(
        std::fs::read_to_string(repo.join("src/modified.txt")).unwrap(),
        "after\n"
    );
    assert!(!repo.join("src/deleted.txt").exists());
}

#[test]
fn immutable_review_package_keeps_exact_prepared_before_and_after_blobs() {
    let temp = tempfile::tempdir().expect("tempdir");
    let repo = temp.path().join("repo");
    let data_dir = temp.path().join("domain-data");
    let deleted = b"exact deleted bytes\n";
    let before = b"exact old bytes\n";
    let after = b"exact new bytes\n";
    std::fs::create_dir_all(repo.join("src")).unwrap();
    std::fs::write(repo.join("src/deleted.txt"), deleted).unwrap();
    std::fs::write(repo.join("src/modified.txt"), before).unwrap();
    let workspace = temp.path().join("workspace");
    std::fs::create_dir_all(workspace.join("src")).unwrap();
    std::fs::write(workspace.join("src/modified.txt"), after).unwrap();

    prepare_review_package(
        &repo,
        &data_dir,
        "run-exact-review",
        "base",
        &[AgentWorkspaceInput {
            agent_id: "agent-0".into(),
            task_id: "implement".into(),
            workspace_path: workspace,
            claims: vec!["src".into()],
            depends_on: vec![],
        }],
        &[],
    )
    .unwrap();

    let blobs = data_dir.join("reviews/run-exact-review/blobs");
    for expected in [deleted.as_slice(), before.as_slice(), after.as_slice()] {
        let digest = format!("{:x}", Sha256::digest(expected));
        assert_eq!(
            std::fs::read(blobs.join(digest)).expect("every review side is immutable"),
            expected
        );
    }
}

#[test]
fn exact_review_chunks_are_bounded_and_ignore_unrelated_corrupt_blobs() {
    let temp = tempfile::tempdir().expect("tempdir");
    let repo = temp.path().join("repo");
    let data_dir = temp.path().join("domain-data");
    let workspace = temp.path().join("workspace");
    std::fs::create_dir_all(&repo).unwrap();
    let selected = (0..(64 * 1024 + 913))
        .map(|index| (index % 251) as u8)
        .collect::<Vec<_>>();
    std::fs::create_dir_all(workspace.join("assets")).unwrap();
    std::fs::write(workspace.join("assets/selected.bin"), &selected).unwrap();
    std::fs::write(workspace.join("assets/unrelated.bin"), b"unrelated").unwrap();
    std::fs::write(workspace.join("assets/unreadable.bin"), b"missing").unwrap();
    let prepared = prepare_review_package(
        &repo,
        &data_dir,
        "run-bounded-review",
        "base",
        &[AgentWorkspaceInput {
            agent_id: "agent-0".into(),
            task_id: "review".into(),
            workspace_path: workspace,
            claims: vec!["assets".into()],
            depends_on: vec![],
        }],
        &[],
    )
    .unwrap();
    let unrelated = prepared
        .files
        .iter()
        .find(|file| file.path == "assets/unrelated.bin")
        .and_then(|file| file.after_sha256.as_ref())
        .unwrap();
    std::fs::write(
        data_dir
            .join("reviews/run-bounded-review/blobs")
            .join(unrelated),
        b"corrupt unrelated blob",
    )
    .unwrap();
    let unreadable = prepared
        .files
        .iter()
        .find(|file| file.path == "assets/unreadable.bin")
        .and_then(|file| file.after_sha256.as_ref())
        .unwrap();
    std::fs::remove_file(
        data_dir
            .join("reviews/run-bounded-review/blobs")
            .join(unreadable),
    )
    .unwrap();

    let manifest = load_review_manifest(&data_dir, "run-bounded-review")
        .expect("manifest validation must not touch unrelated blobs");
    let first = read_review_content_chunk(
        &data_dir,
        &manifest,
        "assets/selected.bin",
        ReviewContentSide::After,
        0,
        64 * 1024,
    )
    .expect("selected blob is independently validated");
    let second = read_review_content_chunk(
        &data_dir,
        &manifest,
        "assets/selected.bin",
        ReviewContentSide::After,
        first.next_offset,
        64 * 1024,
    )
    .expect("later chunk is independently bounded");

    assert_eq!(first.bytes, selected[..64 * 1024]);
    assert_eq!(first.length, 64 * 1024);
    assert_eq!(second.bytes, selected[64 * 1024..]);
    assert_eq!(second.length, 913);
    assert!(!first.complete && second.complete);
}

#[test]
fn full_exact_review_expansion_reads_each_multi_megabyte_byte_once() {
    let temp = tempfile::tempdir().expect("tempdir");
    let repo = temp.path().join("repo");
    let data_dir = temp.path().join("domain-data");
    let workspace = temp.path().join("workspace");
    std::fs::create_dir_all(&repo).unwrap();
    std::fs::create_dir_all(&workspace).unwrap();
    let expected = (0..(3 * 1024 * 1024 + 271))
        .map(|index| (index % 251) as u8)
        .collect::<Vec<_>>();
    std::fs::write(workspace.join("large.bin"), &expected).unwrap();
    let manifest = prepare_review_package(
        &repo,
        &data_dir,
        "run-linear-review",
        "base",
        &[AgentWorkspaceInput {
            agent_id: "agent".into(),
            task_id: "linear".into(),
            workspace_path: workspace,
            claims: vec!["large.bin".into()],
            depends_on: vec![],
        }],
        &[],
    )
    .unwrap();
    let file = &manifest.files[0];
    assert_eq!(file.after_chunks.len(), 49);

    let mut metrics = ReviewReadMetrics::default();
    let mut returned = Vec::with_capacity(expected.len());
    let mut offset = 0;
    loop {
        let chunk = read_review_content_chunk_with_metrics(
            &data_dir,
            &manifest,
            "large.bin",
            ReviewContentSide::After,
            offset,
            64 * 1024,
            &mut metrics,
        )
        .unwrap();
        returned.extend_from_slice(&chunk.bytes);
        offset = chunk.next_offset;
        if chunk.complete {
            break;
        }
    }

    assert_eq!(returned, expected);
    assert_eq!(metrics.bytes_read, expected.len() as u64);
    assert_eq!(metrics.read_operations, 49);
}

#[test]
fn exact_review_chunk_fails_closed_on_mid_stream_mutation_and_never_reopens_after_hashing() {
    let fixture = |run_id: &str| {
        let temp = tempfile::tempdir().expect("tempdir");
        let repo = temp.path().join("repo");
        let data_dir = temp.path().join("domain-data");
        let workspace = temp.path().join("workspace");
        let expected = vec![b'a'; 128 * 1024 + 37];
        std::fs::create_dir_all(&repo).unwrap();
        std::fs::create_dir_all(&workspace).unwrap();
        std::fs::write(workspace.join("large.bin"), &expected).unwrap();
        let manifest = prepare_review_package(
            &repo,
            &data_dir,
            run_id,
            "base",
            &[AgentWorkspaceInput {
                agent_id: "agent-0".into(),
                task_id: "review".into(),
                workspace_path: workspace,
                claims: vec!["large.bin".into()],
                depends_on: vec![],
            }],
            &[],
        )
        .unwrap();
        (temp, data_dir, manifest, expected)
    };

    let (_during_temp, during_data, during_manifest, _) = fixture("run-read-during");
    let during = read_review_content_chunk_with_fault(
        &during_data,
        &during_manifest,
        "large.bin",
        ReviewContentSide::After,
        0,
        64 * 1024,
        Some(ReviewReadFaultPoint::CorruptAfterOpen),
    )
    .expect_err("mutation of the opened blob must fail its same-pass digest");
    assert!(during.to_string().contains("digest mismatch"));

    let (_after_temp, after_data, after_manifest, expected) = fixture("run-read-after");
    let after = read_review_content_chunk_with_fault(
        &after_data,
        &after_manifest,
        "large.bin",
        ReviewContentSide::After,
        0,
        64 * 1024,
        Some(ReviewReadFaultPoint::CorruptAfterStream),
    )
    .expect("post-stream path mutation cannot change bytes from the validated open handle");
    assert_eq!(after.bytes, expected[..64 * 1024]);
    assert_eq!(after.length, 64 * 1024);
}

#[test]
fn execution_domain_mutation_lease_is_exclusive_and_recovers_after_holder_exit() {
    let temp = tempfile::tempdir().expect("tempdir");
    let data_dir = temp.path().join("domain-data");

    let first = ExecutionDomainMutationLease::try_acquire(&data_dir).expect("first lease");
    let conflict = ExecutionDomainMutationLease::try_acquire(&data_dir)
        .expect_err("a second Apply or reconciliation must fail closed");
    assert!(conflict.to_string().contains("busy"));

    drop(first);
    ExecutionDomainMutationLease::try_acquire(&data_dir)
        .expect("OS lock release after holder exit prevents an orphaned lease");
}

#[test]
fn execution_domain_mutation_lease_recovers_after_real_child_process_termination() {
    let temp = tempfile::tempdir().expect("tempdir");
    let data_dir = temp.path().join("domain-data");
    let ready = temp.path().join("child-ready");
    let mut child = std::process::Command::new(std::env::current_exe().unwrap())
        .args(["--exact", "lease_child_process_holder", "--nocapture"])
        .env("PYTXO_LEASE_CHILD_DATA_DIR", &data_dir)
        .env("PYTXO_LEASE_CHILD_READY", &ready)
        .spawn()
        .expect("spawn real lease holder process");
    for _ in 0..250 {
        if ready.exists() {
            break;
        }
        std::thread::sleep(std::time::Duration::from_millis(20));
    }
    assert!(ready.exists(), "child never reported acquired lease");
    let busy = ExecutionDomainMutationLease::try_acquire(&data_dir)
        .expect_err("the child process owns the domain lease");
    assert!(busy.to_string().contains("busy"));

    child.kill().expect("terminate child lease holder");
    child.wait().expect("reap child lease holder");
    ExecutionDomainMutationLease::try_acquire(&data_dir)
        .expect("the OS releases a terminated process's lease");
}

#[test]
fn lease_child_process_holder() {
    let Some(data_dir) = std::env::var_os("PYTXO_LEASE_CHILD_DATA_DIR") else {
        return;
    };
    let ready = PathBuf::from(std::env::var_os("PYTXO_LEASE_CHILD_READY").unwrap());
    let _lease = ExecutionDomainMutationLease::try_acquire(Path::new(&data_dir)).unwrap();
    std::fs::write(ready, b"ready").unwrap();
    loop {
        std::thread::park_timeout(std::time::Duration::from_secs(1));
    }
}

#[test]
fn interrupted_journal_recovers_after_real_child_process_termination() {
    let temp = tempfile::tempdir().unwrap();
    let repo = temp.path().join("repo");
    let data_dir = temp.path().join("data");
    let workspace = temp.path().join("workspace");
    let ready = temp.path().join("ready");
    write(&repo.join("existing.txt"), "before\n");
    write(&workspace.join("existing.txt"), "after\n");
    write(&workspace.join("new/deep/added.txt"), "added\n");
    prepare_review_package(
        &repo,
        &data_dir,
        "run-process-recovery",
        "base",
        &[AgentWorkspaceInput {
            agent_id: "worker".into(),
            task_id: "task".into(),
            workspace_path: workspace,
            claims: vec!["existing.txt".into(), "new".into()],
            depends_on: vec![],
        }],
        &[],
    )
    .unwrap();

    // Reap the exact test-owned child even if a pre-termination assertion fails.
    struct ChildGuard(std::process::Child);
    impl Drop for ChildGuard {
        fn drop(&mut self) {
            let _ = self.0.kill();
            let _ = self.0.wait();
        }
    }
    let mut child = ChildGuard(
        std::process::Command::new(std::env::current_exe().unwrap())
            .args(["--exact", "interrupted_journal_child_holder", "--nocapture"])
            .env("PYTXO_RECOVERY_CHILD_ROOT", temp.path())
            .spawn()
            .unwrap(),
    );
    let deadline = std::time::Instant::now() + std::time::Duration::from_secs(10);
    while !ready.exists() && std::time::Instant::now() < deadline {
        assert!(
            child.0.try_wait().unwrap().is_none(),
            "child exited before journal checkpoint"
        );
        std::thread::sleep(std::time::Duration::from_millis(20));
    }
    assert!(ready.exists(), "child did not reach journal checkpoint");
    assert_eq!(
        std::fs::read(repo.join("existing.txt")).unwrap(),
        b"after\n"
    );
    assert_eq!(
        std::fs::read(repo.join("new/deep/added.txt")).unwrap(),
        b"added\n"
    );
    assert!(
        reconcile_apply_journals(&repo, &data_dir, "run-process-recovery")
            .expect_err("recovery must not race the live lease holder")
            .to_string()
            .contains("busy")
    );

    child.0.kill().unwrap();
    assert!(!child.0.wait().unwrap().success());
    assert!(matches!(
        reconcile_apply_journals(&repo, &data_dir, "run-process-recovery").unwrap(),
        RecoveryOutcome::RolledBack { .. }
    ));
    assert_eq!(
        std::fs::read(repo.join("existing.txt")).unwrap(),
        b"before\n"
    );
    assert!(!repo.join("new").exists());
    assert_eq!(
        reconcile_apply_journals(&repo, &data_dir, "run-process-recovery").unwrap(),
        RecoveryOutcome::NothingToDo
    );
}

#[test]
fn interrupted_journal_child_holder() {
    let Some(root) = std::env::var_os("PYTXO_RECOVERY_CHILD_ROOT") else {
        return;
    };
    let root = PathBuf::from(root);
    let repo = root.join("repo");
    let data_dir = root.join("data");
    let manifest = load_review_manifest(&data_dir, "run-process-recovery").unwrap();
    let lease = ExecutionDomainMutationLease::try_acquire(&data_dir).unwrap();
    // The existing fault checkpoint leaves durable mutation evidence intact.
    // Retain the lease until the parent terminates this process without unwinding.
    pytxo_runner::apply_prepared_review_with_fault_under_lease(
        &repo,
        &data_dir,
        &manifest,
        &lease,
        Some(ApplyFaultPoint::InterruptAfterRename(2)),
    )
    .expect_err("checkpoint must interrupt before commit");
    std::fs::write(root.join("ready"), b"journal-ready").unwrap();
    loop {
        std::thread::park_timeout(std::time::Duration::from_secs(1));
    }
}

#[test]
fn execution_domain_lease_serializes_two_reviewed_runs_touching_the_same_path() {
    let temp = tempfile::tempdir().expect("tempdir");
    let repo = temp.path().join("repo");
    let data_dir = temp.path().join("domain-data");
    write(&repo.join("owned.txt"), "base\n");
    let workspace_a = temp.path().join("workspace-a");
    let workspace_b = temp.path().join("workspace-b");
    write(&workspace_a.join("owned.txt"), "first\n");
    write(&workspace_b.join("owned.txt"), "second\n");
    let input = |agent_id: &str, workspace_path: PathBuf| AgentWorkspaceInput {
        agent_id: agent_id.into(),
        task_id: "task".into(),
        workspace_path,
        claims: vec!["owned.txt".into()],
        depends_on: vec![],
    };
    let first = prepare_review_package(
        &repo,
        &data_dir,
        "run-first",
        "base",
        &[input("agent-first", workspace_a)],
        &[],
    )
    .unwrap();
    let second = prepare_review_package(
        &repo,
        &data_dir,
        "run-second",
        "base",
        &[input("agent-second", workspace_b)],
        &[],
    )
    .unwrap();

    let lease = ExecutionDomainMutationLease::try_acquire(&data_dir).unwrap();
    let conflict = apply_prepared_review(&repo, &data_dir, &second)
        .expect_err("the second reviewed run cannot mutate while the domain is leased");
    assert!(conflict.to_string().contains("busy"));
    apply_prepared_review_under_lease(&repo, &data_dir, &first, &lease).unwrap();
    drop(lease);

    let stale = apply_prepared_review(&repo, &data_dir, &second)
        .expect_err("the losing reviewed run must not overwrite the first result");
    assert!(stale.to_string().contains("changed since review"));
    assert_eq!(
        std::fs::read_to_string(repo.join("owned.txt")).unwrap(),
        "first\n"
    );
}

#[test]
fn reconciliation_fails_closed_while_apply_owns_the_execution_domain() {
    let temp = tempfile::tempdir().expect("tempdir");
    let repo = temp.path().join("repo");
    let data_dir = temp.path().join("domain-data");
    std::fs::create_dir_all(&repo).unwrap();

    let lease = ExecutionDomainMutationLease::try_acquire(&data_dir).unwrap();
    let error = reconcile_apply_journals(&repo, &data_dir, "run-reconcile")
        .expect_err("reconciliation cannot overlap an Apply mutation lease");
    assert!(error.to_string().contains("busy"));
    drop(lease);
    assert_eq!(
        reconcile_apply_journals(&repo, &data_dir, "run-reconcile").unwrap(),
        RecoveryOutcome::NothingToDo
    );
}

#[test]
fn blob_changed_after_package_validation_is_never_applied() {
    let temp = tempfile::tempdir().expect("tempdir");
    let repo = temp.path().join("repo");
    let data_dir = temp.path().join("domain-data");
    write(&repo.join("owned.txt"), "before\n");
    let workspace = temp.path().join("workspace");
    write(&workspace.join("owned.txt"), "after\n");
    let manifest = prepare_review_package(
        &repo,
        &data_dir,
        "run-blob-race",
        "base",
        &[AgentWorkspaceInput {
            agent_id: "agent-0".into(),
            task_id: "implement".into(),
            workspace_path: workspace,
            claims: vec!["owned.txt".into()],
            depends_on: vec![],
        }],
        &[],
    )
    .unwrap();

    let error = apply_prepared_review_with_fault(
        &repo,
        &data_dir,
        &manifest,
        Some(ApplyFaultPoint::CorruptBlobBeforeMutation(1)),
    )
    .expect_err("a post-validation blob change must fail closed");

    assert!(error.to_string().contains("prepared blob digest mismatch"));
    assert_eq!(
        std::fs::read_to_string(repo.join("owned.txt")).unwrap(),
        "before\n"
    );
}

#[cfg(unix)]
#[test]
fn unix_mode_only_change_is_prepared_applied_and_restored_on_rollback() {
    use std::os::unix::fs::PermissionsExt;

    let temp = tempfile::tempdir().expect("tempdir");
    let repo = temp.path().join("repo");
    let data_dir = temp.path().join("domain-data");
    write(&repo.join("script.sh"), "#!/bin/sh\nexit 0\n");
    std::fs::set_permissions(
        repo.join("script.sh"),
        std::fs::Permissions::from_mode(0o644),
    )
    .unwrap();
    let workspace = temp.path().join("workspace");
    write(&workspace.join("script.sh"), "#!/bin/sh\nexit 0\n");
    std::fs::set_permissions(
        workspace.join("script.sh"),
        std::fs::Permissions::from_mode(0o755),
    )
    .unwrap();
    let manifest = prepare_review_package(
        &repo,
        &data_dir,
        "run-mode-only",
        "base",
        &[AgentWorkspaceInput {
            agent_id: "agent-0".into(),
            task_id: "mode".into(),
            workspace_path: workspace,
            claims: vec!["script.sh".into()],
            depends_on: vec![],
        }],
        &[],
    )
    .unwrap();

    assert_eq!(
        manifest.files.len(),
        1,
        "mode-only changes must not disappear"
    );
    assert_eq!(manifest.files[0].before_mode, Some(0o644));
    assert_eq!(manifest.files[0].after_mode, Some(0o755));
    apply_prepared_review(&repo, &data_dir, &manifest).unwrap();
    assert_eq!(
        std::fs::metadata(repo.join("script.sh"))
            .unwrap()
            .permissions()
            .mode()
            & 0o7777,
        0o755
    );

    let workspace = temp.path().join("workspace-rollback");
    write(&workspace.join("script.sh"), "#!/bin/sh\nexit 1\n");
    std::fs::set_permissions(
        workspace.join("script.sh"),
        std::fs::Permissions::from_mode(0o700),
    )
    .unwrap();
    let rollback_manifest = prepare_review_package(
        &repo,
        &data_dir,
        "run-mode-rollback",
        "base",
        &[AgentWorkspaceInput {
            agent_id: "agent-0".into(),
            task_id: "mode".into(),
            workspace_path: workspace,
            claims: vec!["script.sh".into()],
            depends_on: vec![],
        }],
        &[],
    )
    .unwrap();
    apply_prepared_review_with_fault(
        &repo,
        &data_dir,
        &rollback_manifest,
        Some(ApplyFaultPoint::FailAfterMutation(1)),
    )
    .expect_err("fault forces rollback");
    assert_eq!(
        std::fs::metadata(repo.join("script.sh"))
            .unwrap()
            .permissions()
            .mode()
            & 0o7777,
        0o755
    );
}

#[cfg(unix)]
fn unix_mode(path: &Path) -> u32 {
    use std::os::unix::fs::PermissionsExt;
    std::fs::metadata(path).unwrap().permissions().mode() & 0o7777
}

#[cfg(unix)]
#[test]
fn unix_executable_addition_preserves_target_mode() {
    use std::os::unix::fs::PermissionsExt;

    let temp = tempfile::tempdir().unwrap();
    let repo = temp.path().join("repo");
    let data_dir = temp.path().join("data");
    let workspace = temp.path().join("workspace");
    std::fs::create_dir_all(&repo).unwrap();
    write(&workspace.join("added.sh"), "#!/bin/sh\nexit 0\n");
    std::fs::set_permissions(
        workspace.join("added.sh"),
        std::fs::Permissions::from_mode(0o755),
    )
    .unwrap();
    let manifest = prepare_review_package(
        &repo,
        &data_dir,
        "run-executable-add",
        "base",
        &[AgentWorkspaceInput {
            agent_id: "agent".into(),
            task_id: "mode".into(),
            workspace_path: workspace,
            claims: vec!["added.sh".into()],
            depends_on: vec![],
        }],
        &[],
    )
    .unwrap();
    assert_eq!(manifest.files[0].before_mode, None);
    assert_eq!(manifest.files[0].after_mode, Some(0o755));
    apply_prepared_review(&repo, &data_dir, &manifest).unwrap();
    assert_eq!(unix_mode(&repo.join("added.sh")), 0o755);
}

#[cfg(unix)]
#[test]
fn unix_executable_modification_preserves_content_and_target_mode() {
    use std::os::unix::fs::PermissionsExt;

    let temp = tempfile::tempdir().unwrap();
    let repo = temp.path().join("repo");
    let data_dir = temp.path().join("data");
    let workspace = temp.path().join("workspace");
    write(&repo.join("script.sh"), "#!/bin/sh\nexit 0\n");
    std::fs::set_permissions(
        repo.join("script.sh"),
        std::fs::Permissions::from_mode(0o755),
    )
    .unwrap();
    write(&workspace.join("script.sh"), "#!/bin/sh\necho changed\n");
    std::fs::set_permissions(
        workspace.join("script.sh"),
        std::fs::Permissions::from_mode(0o700),
    )
    .unwrap();
    let manifest = prepare_review_package(
        &repo,
        &data_dir,
        "run-executable-modify",
        "base",
        &[AgentWorkspaceInput {
            agent_id: "agent".into(),
            task_id: "mode".into(),
            workspace_path: workspace,
            claims: vec!["script.sh".into()],
            depends_on: vec![],
        }],
        &[],
    )
    .unwrap();
    assert_eq!(manifest.files[0].before_mode, Some(0o755));
    assert_eq!(manifest.files[0].after_mode, Some(0o700));
    apply_prepared_review(&repo, &data_dir, &manifest).unwrap();
    assert_eq!(
        std::fs::read_to_string(repo.join("script.sh")).unwrap(),
        "#!/bin/sh\necho changed\n"
    );
    assert_eq!(unix_mode(&repo.join("script.sh")), 0o700);
}

#[cfg(unix)]
#[test]
fn unix_executable_deletion_is_restored_with_mode_on_failure_rollback() {
    use std::os::unix::fs::PermissionsExt;

    let temp = tempfile::tempdir().unwrap();
    let repo = temp.path().join("repo");
    let data_dir = temp.path().join("data");
    let workspace = temp.path().join("workspace");
    write(&repo.join("delete.sh"), "#!/bin/sh\nexit 0\n");
    std::fs::set_permissions(
        repo.join("delete.sh"),
        std::fs::Permissions::from_mode(0o755),
    )
    .unwrap();
    std::fs::create_dir_all(&workspace).unwrap();
    let manifest = prepare_review_package(
        &repo,
        &data_dir,
        "run-executable-delete-rollback",
        "base",
        &[AgentWorkspaceInput {
            agent_id: "agent".into(),
            task_id: "mode".into(),
            workspace_path: workspace,
            claims: vec!["delete.sh".into()],
            depends_on: vec![],
        }],
        &[],
    )
    .unwrap();
    apply_prepared_review_with_fault(
        &repo,
        &data_dir,
        &manifest,
        Some(ApplyFaultPoint::FailAfterMutation(1)),
    )
    .expect_err("injected failure rolls deletion back");
    assert_eq!(
        std::fs::read_to_string(repo.join("delete.sh")).unwrap(),
        "#!/bin/sh\nexit 0\n"
    );
    assert_eq!(unix_mode(&repo.join("delete.sh")), 0o755);
}

#[cfg(unix)]
#[test]
fn unix_mode_and_content_are_restored_on_injected_failure_rollback() {
    use std::os::unix::fs::PermissionsExt;

    let temp = tempfile::tempdir().unwrap();
    let repo = temp.path().join("repo");
    let data_dir = temp.path().join("data");
    let workspace = temp.path().join("workspace");
    write(&repo.join("script.sh"), "before\n");
    std::fs::set_permissions(
        repo.join("script.sh"),
        std::fs::Permissions::from_mode(0o644),
    )
    .unwrap();
    write(&workspace.join("script.sh"), "after\n");
    std::fs::set_permissions(
        workspace.join("script.sh"),
        std::fs::Permissions::from_mode(0o755),
    )
    .unwrap();
    let manifest = prepare_review_package(
        &repo,
        &data_dir,
        "run-mode-failure-rollback",
        "base",
        &[AgentWorkspaceInput {
            agent_id: "agent".into(),
            task_id: "mode".into(),
            workspace_path: workspace,
            claims: vec!["script.sh".into()],
            depends_on: vec![],
        }],
        &[],
    )
    .unwrap();
    apply_prepared_review_with_fault(
        &repo,
        &data_dir,
        &manifest,
        Some(ApplyFaultPoint::FailAfterMutation(1)),
    )
    .expect_err("injected failure rolls mode and bytes back");
    assert_eq!(
        std::fs::read_to_string(repo.join("script.sh")).unwrap(),
        "before\n"
    );
    assert_eq!(unix_mode(&repo.join("script.sh")), 0o644);
}

#[cfg(unix)]
#[test]
fn unix_interrupted_journal_recovery_restores_mode_and_content() {
    use std::os::unix::fs::PermissionsExt;

    let temp = tempfile::tempdir().unwrap();
    let repo = temp.path().join("repo");
    let data_dir = temp.path().join("data");
    let workspace = temp.path().join("workspace");
    write(&repo.join("script.sh"), "before\n");
    std::fs::set_permissions(
        repo.join("script.sh"),
        std::fs::Permissions::from_mode(0o644),
    )
    .unwrap();
    write(&workspace.join("script.sh"), "after\n");
    std::fs::set_permissions(
        workspace.join("script.sh"),
        std::fs::Permissions::from_mode(0o755),
    )
    .unwrap();
    let run_id = "run-mode-interrupted-recovery";
    let manifest = prepare_review_package(
        &repo,
        &data_dir,
        run_id,
        "base",
        &[AgentWorkspaceInput {
            agent_id: "agent".into(),
            task_id: "mode".into(),
            workspace_path: workspace,
            claims: vec!["script.sh".into()],
            depends_on: vec![],
        }],
        &[],
    )
    .unwrap();
    apply_prepared_review_with_fault(
        &repo,
        &data_dir,
        &manifest,
        Some(ApplyFaultPoint::InterruptAfterRename(1)),
    )
    .expect_err("interruption leaves a recoverable journal");
    assert_eq!(unix_mode(&repo.join("script.sh")), 0o755);
    assert!(matches!(
        reconcile_apply_journals(&repo, &data_dir, run_id).unwrap(),
        RecoveryOutcome::RolledBack { .. }
    ));
    assert_eq!(
        std::fs::read_to_string(repo.join("script.sh")).unwrap(),
        "before\n"
    );
    assert_eq!(unix_mode(&repo.join("script.sh")), 0o644);
}

fn workspace(repo: &Path, name: &str) -> PathBuf {
    let path = repo
        .parent()
        .expect("repo parent")
        .join(format!("workspace-{name}"));
    std::fs::create_dir_all(path.join("src")).expect("create workspace");
    write(&path.join("src/modified.txt"), "before\n");
    write(&path.join("src/deleted.txt"), "delete me\n");
    write(&path.join("outside.txt"), "unchanged\n");
    path
}

#[test]
fn prepares_claimed_add_modify_and_delete_changes() {
    let temp = tempfile::tempdir().expect("tempdir");
    let repo = temp.path().join("repo");
    write(&repo.join("src/modified.txt"), "before\n");
    write(&repo.join("src/deleted.txt"), "delete me\n");
    write(&repo.join("outside.txt"), "unchanged\n");

    let agent_workspace = workspace(&repo, "agent-0");
    write(&agent_workspace.join("src/modified.txt"), "after\n");
    write(&agent_workspace.join("src/added.txt"), "new\n");
    std::fs::remove_file(agent_workspace.join("src/deleted.txt")).expect("delete fixture");

    let change_set = prepare_run_change_set(
        &repo,
        &[AgentWorkspaceInput {
            agent_id: "agent-0".into(),
            task_id: "implement".into(),
            workspace_path: agent_workspace,
            claims: vec!["src".into()],
            depends_on: vec![],
        }],
        &[],
    )
    .expect("prepare change set");

    let summary: Vec<_> = change_set
        .changes
        .iter()
        .map(|change| (change.path.as_str(), change.kind))
        .collect();
    assert_eq!(
        summary,
        vec![
            ("src/added.txt", RunChangeKind::Add),
            ("src/deleted.txt", RunChangeKind::Delete),
            ("src/modified.txt", RunChangeKind::Modify),
        ]
    );
}

#[test]
fn rejects_agent_edits_outside_declared_claims() {
    let temp = tempfile::tempdir().expect("tempdir");
    let repo = temp.path().join("repo");
    write(&repo.join("src/owned.txt"), "before\n");
    write(&repo.join("outside.txt"), "before\n");

    let agent_workspace = temp.path().join("workspace-agent-0");
    write(&agent_workspace.join("src/owned.txt"), "after\n");
    write(&agent_workspace.join("outside.txt"), "not owned\n");

    let error = prepare_run_change_set(
        &repo,
        &[AgentWorkspaceInput {
            agent_id: "agent-0".into(),
            task_id: "implement".into(),
            workspace_path: agent_workspace,
            claims: vec!["src".into()],
            depends_on: vec![],
        }],
        &[],
    )
    .expect_err("out-of-claim edit must block the run change set");

    assert!(
        error.to_string().contains("outside declared claims")
            && error.to_string().contains("outside.txt"),
        "unexpected error: {error}"
    );
}

#[test]
fn rejects_conflicting_changes_from_independent_tasks() {
    let temp = tempfile::tempdir().expect("tempdir");
    let repo = temp.path().join("repo");
    write(&repo.join("src/shared.txt"), "before\n");

    let first = temp.path().join("workspace-first");
    write(&first.join("src/shared.txt"), "first\n");
    let second = temp.path().join("workspace-second");
    write(&second.join("src/shared.txt"), "second\n");

    let error = prepare_run_change_set(
        &repo,
        &[
            AgentWorkspaceInput {
                agent_id: "agent-0".into(),
                task_id: "first".into(),
                workspace_path: first,
                claims: vec!["src/shared.txt".into()],
                depends_on: vec![],
            },
            AgentWorkspaceInput {
                agent_id: "agent-1".into(),
                task_id: "second".into(),
                workspace_path: second,
                claims: vec!["src/shared.txt".into()],
                depends_on: vec![],
            },
        ],
        &[],
    )
    .expect_err("independent tasks must not resolve a collision by iteration order");

    assert!(
        error.to_string().contains("run change conflict")
            && error.to_string().contains("src/shared.txt")
            && error.to_string().contains("first")
            && error.to_string().contains("second"),
        "unexpected error: {error}"
    );
}

#[test]
fn rejects_identical_changes_with_divergent_independent_ownership() {
    let temp = tempfile::tempdir().expect("tempdir");
    let repo = temp.path().join("repo");
    write(&repo.join("src/shared.txt"), "before\n");
    let first = temp.path().join("workspace-first-identical");
    let second = temp.path().join("workspace-second-identical");
    write(&first.join("src/shared.txt"), "same-result\n");
    write(&second.join("src/shared.txt"), "same-result\n");

    let error = prepare_run_change_set(
        &repo,
        &[
            AgentWorkspaceInput {
                agent_id: "agent-0".into(),
                task_id: "first".into(),
                workspace_path: first,
                claims: vec!["src/shared.txt".into()],
                depends_on: vec![],
            },
            AgentWorkspaceInput {
                agent_id: "agent-1".into(),
                task_id: "second".into(),
                workspace_path: second,
                claims: vec!["src/shared.txt".into()],
                depends_on: vec![],
            },
        ],
        &[],
    )
    .expect_err("identical bytes must not erase an independent ownership conflict");

    assert!(error.to_string().contains("run change conflict"));
}

#[test]
fn dependency_order_allows_downstream_task_to_supersede_upstream_change() {
    let temp = tempfile::tempdir().expect("tempdir");
    let repo = temp.path().join("repo");
    write(&repo.join("src/shared.txt"), "before\n");

    let upstream = temp.path().join("workspace-upstream");
    write(&upstream.join("src/shared.txt"), "upstream\n");
    let downstream = temp.path().join("workspace-downstream");
    write(&downstream.join("src/shared.txt"), "downstream\n");

    let change_set = prepare_run_change_set(
        &repo,
        &[
            AgentWorkspaceInput {
                agent_id: "agent-0".into(),
                task_id: "upstream".into(),
                workspace_path: upstream,
                claims: vec!["src/shared.txt".into()],
                depends_on: vec![],
            },
            AgentWorkspaceInput {
                agent_id: "agent-1".into(),
                task_id: "downstream".into(),
                workspace_path: downstream,
                claims: vec!["src/shared.txt".into()],
                depends_on: vec!["upstream".into()],
            },
        ],
        &[],
    )
    .expect("dependency order should resolve the shared path");

    assert_eq!(change_set.changes.len(), 1);
    assert_eq!(change_set.changes[0].source_task_id, "downstream");
    assert_eq!(
        std::fs::read_to_string(
            change_set.changes[0]
                .source_path
                .as_ref()
                .expect("downstream source")
        )
        .expect("read downstream source"),
        "downstream\n"
    );
}

#[test]
fn dependency_outputs_outside_downstream_claims_are_not_misattributed() {
    let temp = tempfile::tempdir().expect("tempdir");
    let repo = temp.path().join("repo");
    write(&repo.join("src/upstream.txt"), "before-upstream\n");
    write(&repo.join("src/downstream.txt"), "before-downstream\n");

    let upstream = temp.path().join("workspace-upstream");
    write(&upstream.join("src/upstream.txt"), "after-upstream\n");
    write(&upstream.join("src/downstream.txt"), "before-downstream\n");

    let downstream = temp.path().join("workspace-downstream");
    write(&downstream.join("src/upstream.txt"), "after-upstream\n");
    write(&downstream.join("src/downstream.txt"), "after-downstream\n");

    let change_set = prepare_run_change_set(
        &repo,
        &[
            AgentWorkspaceInput {
                agent_id: "agent-0".into(),
                task_id: "upstream".into(),
                workspace_path: upstream,
                claims: vec!["src/upstream.txt".into()],
                depends_on: vec![],
            },
            AgentWorkspaceInput {
                agent_id: "agent-1".into(),
                task_id: "downstream".into(),
                workspace_path: downstream,
                claims: vec!["src/downstream.txt".into()],
                depends_on: vec!["upstream".into()],
            },
        ],
        &[],
    )
    .expect("unchanged inherited output should remain attributed to its owner");

    assert_eq!(change_set.changes.len(), 2);
    assert_eq!(change_set.changes[0].source_task_id, "downstream");
    assert_eq!(change_set.changes[1].source_task_id, "upstream");
}

#[test]
fn downstream_cannot_modify_inherited_output_without_claiming_it() {
    let temp = tempfile::tempdir().expect("tempdir");
    let repo = temp.path().join("repo");
    write(&repo.join("src/upstream.txt"), "before-upstream\n");
    write(&repo.join("src/downstream.txt"), "before-downstream\n");

    let upstream = temp.path().join("workspace-upstream");
    write(&upstream.join("src/upstream.txt"), "after-upstream\n");
    write(&upstream.join("src/downstream.txt"), "before-downstream\n");

    let downstream = temp.path().join("workspace-downstream");
    write(
        &downstream.join("src/upstream.txt"),
        "unclaimed-downstream-rewrite\n",
    );
    write(&downstream.join("src/downstream.txt"), "after-downstream\n");

    let error = prepare_run_change_set(
        &repo,
        &[
            AgentWorkspaceInput {
                agent_id: "agent-0".into(),
                task_id: "upstream".into(),
                workspace_path: upstream,
                claims: vec!["src/upstream.txt".into()],
                depends_on: vec![],
            },
            AgentWorkspaceInput {
                agent_id: "agent-1".into(),
                task_id: "downstream".into(),
                workspace_path: downstream,
                claims: vec!["src/downstream.txt".into()],
                depends_on: vec!["upstream".into()],
            },
        ],
        &[],
    )
    .expect_err("downstream must not rewrite inherited paths it does not own");

    assert!(
        error.to_string().contains("outside declared claims")
            && error.to_string().contains("src/upstream.txt"),
        "unexpected error: {error}"
    );
}

#[test]
fn rejects_repository_root_as_a_change_claim() {
    let temp = tempfile::tempdir().expect("tempdir");
    let repo = temp.path().join("repo");
    write(&repo.join("src/lib.rs"), "before\n");
    let workspace = temp.path().join("workspace-agent");
    write(&workspace.join("src/lib.rs"), "after\n");

    let error = prepare_run_change_set(
        &repo,
        &[AgentWorkspaceInput {
            agent_id: "agent-0".into(),
            task_id: "unscoped".into(),
            workspace_path: workspace,
            claims: vec![".".into()],
            depends_on: vec![],
        }],
        &[],
    )
    .expect_err("repository-root ownership must require a narrower plan");

    assert!(error.to_string().contains("unsafe run change-set claim"));
}

#[test]
fn apply_rolls_back_all_paths_when_a_later_copy_fails() {
    let temp = tempfile::tempdir().expect("tempdir");
    let repo = temp.path().join("repo");
    let data_dir = temp.path().join("data");
    write(&repo.join("a.txt"), "a-before\n");
    let agent_workspace = temp.path().join("workspace");
    write(&agent_workspace.join("a.txt"), "a-after\n");
    write(&agent_workspace.join("b.txt"), "b-added\n");
    let manifest = prepare_review_package(
        &repo,
        &data_dir,
        "run-rollback",
        "base",
        &[AgentWorkspaceInput {
            agent_id: "agent-0".into(),
            task_id: "implement".into(),
            workspace_path: agent_workspace,
            claims: vec!["a.txt".into(), "b.txt".into()],
            depends_on: vec![],
        }],
        &[],
    )
    .expect("prepare package");
    apply_prepared_review_with_fault(
        &repo,
        &data_dir,
        &manifest,
        Some(ApplyFaultPoint::FailAfterMutation(1)),
    )
    .expect_err("injected failure must roll back");
    assert_eq!(
        std::fs::read_to_string(repo.join("a.txt")).expect("a restored"),
        "a-before\n"
    );
    assert!(!repo.join("b.txt").exists());
}

#[test]
fn apply_rejects_a_repo_path_changed_after_review() {
    let temp = tempfile::tempdir().expect("tempdir");
    let repo = temp.path().join("repo");
    let data_dir = temp.path().join("data");
    write(&repo.join("src/owned.txt"), "reviewed-base\n");

    let agent_workspace = temp.path().join("workspace-agent-0");
    write(&agent_workspace.join("src/owned.txt"), "agent-result\n");
    let manifest = prepare_review_package(
        &repo,
        &data_dir,
        "run-repo-drift",
        "base",
        &[AgentWorkspaceInput {
            agent_id: "agent-0".into(),
            task_id: "implement".into(),
            workspace_path: agent_workspace,
            claims: vec!["src/owned.txt".into()],
            depends_on: vec![],
        }],
        &[],
    )
    .expect("prepare package");

    write(&repo.join("src/owned.txt"), "newer-human-edit\n");
    let error = apply_prepared_review(&repo, &data_dir, &manifest)
        .expect_err("reviewed base drift must invalidate apply");

    assert!(
        error.to_string().contains("changed since review"),
        "unexpected error: {error}"
    );
    assert_eq!(
        std::fs::read_to_string(repo.join("src/owned.txt")).expect("read primary repo"),
        "newer-human-edit\n"
    );
}

#[test]
fn apply_rejects_an_agent_result_changed_after_review() {
    let temp = tempfile::tempdir().expect("tempdir");
    let repo = temp.path().join("repo");
    write(&repo.join("src/owned.txt"), "base\n");

    let agent_workspace = temp.path().join("workspace-agent-0");
    write(
        &agent_workspace.join("src/owned.txt"),
        "reviewed-agent-result\n",
    );
    let error = prepare_review_package(
        &repo,
        &temp.path().join("data"),
        "run-agent-drift",
        "base",
        &[AgentWorkspaceInput {
            agent_id: "agent-0".into(),
            task_id: "implement".into(),
            workspace_path: agent_workspace.clone(),
            claims: vec!["src/owned.txt".into()],
            depends_on: vec![],
        }],
        &[],
    )
    .expect("durable package snapshots the reviewed result");
    write(
        &agent_workspace.join("src/owned.txt"),
        "unreviewed-agent-result\n",
    );
    apply_prepared_review(&repo, &temp.path().join("data"), &error)
        .expect("Apply consumes the immutable package, not the live workspace");
    assert_eq!(
        std::fs::read_to_string(repo.join("src/owned.txt")).expect("read primary repo"),
        "reviewed-agent-result\n"
    );
}

#[test]
fn apply_returns_a_path_level_manifest_for_add_modify_and_delete() {
    let temp = tempfile::tempdir().expect("tempdir");
    let repo = temp.path().join("repo");
    let data_dir = temp.path().join("data");
    write(&repo.join("src/modified.txt"), "before\n");
    write(&repo.join("src/deleted.txt"), "delete\n");

    let agent_workspace = temp.path().join("workspace-agent-0");
    write(&agent_workspace.join("src/modified.txt"), "after\n");
    write(&agent_workspace.join("src/added.txt"), "added\n");
    let package = prepare_review_package(
        &repo,
        &data_dir,
        "run-manifest",
        "base",
        &[AgentWorkspaceInput {
            agent_id: "agent-0".into(),
            task_id: "implement".into(),
            workspace_path: agent_workspace,
            claims: vec!["src".into()],
            depends_on: vec![],
        }],
        &[],
    )
    .expect("prepare package");

    let manifest = apply_prepared_review(&repo, &data_dir, &package).expect("apply package");

    assert_eq!(manifest.changes.len(), 3);
    assert_eq!(manifest.changes[0].path, "src/added.txt");
    assert_eq!(manifest.changes[1].path, "src/deleted.txt");
    assert_eq!(manifest.changes[2].path, "src/modified.txt");
    assert!(manifest
        .changes
        .iter()
        .all(|change| change.source_task_id == "implement"));
    assert_eq!(
        std::fs::read_to_string(repo.join("src/modified.txt")).expect("modified"),
        "after\n"
    );
    assert_eq!(
        std::fs::read_to_string(repo.join("src/added.txt")).expect("added"),
        "added\n"
    );
    assert!(!repo.join("src/deleted.txt").exists());
}

#[test]
fn configured_glob_claims_match_nested_changed_files() {
    let temp = tempfile::tempdir().expect("tempdir");
    let repo = temp.path().join("repo");
    write(&repo.join("src/nested/lib.rs"), "before\n");

    let agent_workspace = temp.path().join("workspace-agent-0");
    write(&agent_workspace.join("src/nested/lib.rs"), "after\n");
    let change_set = prepare_run_change_set(
        &repo,
        &[AgentWorkspaceInput {
            agent_id: "agent-0".into(),
            task_id: "implement".into(),
            workspace_path: agent_workspace,
            claims: vec!["src/**/*.rs".into()],
            depends_on: vec![],
        }],
        &[],
    )
    .expect("glob claim should own the nested Rust file");

    assert_eq!(change_set.changes.len(), 1);
    assert_eq!(change_set.changes[0].path, "src/nested/lib.rs");
}

#[cfg(unix)]
#[test]
fn rejects_symlinks_in_reviewed_workspaces() {
    use std::os::unix::fs::symlink;

    let temp = tempfile::tempdir().expect("tempdir");
    let repo = temp.path().join("repo");
    write(&repo.join("src/owned.txt"), "before\n");
    let workspace = temp.path().join("workspace-symlink");
    write(&workspace.join("src/owned.txt"), "after\n");
    symlink("owned.txt", workspace.join("src/link.txt")).unwrap();

    let error = prepare_run_change_set(
        &repo,
        &[AgentWorkspaceInput {
            agent_id: "agent-0".into(),
            task_id: "implement".into(),
            workspace_path: workspace,
            claims: vec!["src".into()],
            depends_on: vec![],
        }],
        &[],
    )
    .expect_err("symlinks must not enter a review package");
    assert!(error.to_string().contains("symlink"));
}

#[cfg(unix)]
#[test]
fn rejects_special_filesystem_nodes() {
    let temp = tempfile::tempdir().expect("tempdir");
    let repo = temp.path().join("repo");
    write(&repo.join("src/owned.txt"), "before\n");
    let workspace = temp.path().join("workspace-fifo");
    write(&workspace.join("src/owned.txt"), "after\n");
    let fifo = workspace.join("src/pipe");
    assert!(std::process::Command::new("mkfifo")
        .arg(&fifo)
        .status()
        .unwrap()
        .success());

    let error = prepare_run_change_set(
        &repo,
        &[AgentWorkspaceInput {
            agent_id: "agent-0".into(),
            task_id: "implement".into(),
            workspace_path: workspace,
            claims: vec!["src".into()],
            depends_on: vec![],
        }],
        &[],
    )
    .expect_err("special nodes must not enter a review package");
    assert!(error.to_string().contains("special filesystem node"));
}
