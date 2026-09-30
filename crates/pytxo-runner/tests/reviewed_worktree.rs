use std::fs;
use std::io::Write;
#[cfg(windows)]
use std::os::windows::process::CommandExt;
use std::path::Path;
use std::process::{Command, Stdio};

use pytxo_core::canonical_repo_root;
use pytxo_core::routing::{canonical_digest, BaseSnapshot, Digest};
use pytxo_runner::{
    capture_reviewed_inputs, capture_sealed_output, capture_sealed_output_after_dependency,
    create_plain_verification_view, materialize_dependency_output, materialize_reviewed_inputs,
    materialize_sealed_output, prepare_reviewed_worktree, read_reviewed_claim_text,
    require_exact_reviewed_checkout, seal_one_existing_claimed_text_proposal,
    verify_materialized_dependency_output, verify_materialized_reviewed_inputs,
    verify_sealed_output_view,
};

#[test]
fn verification_view_rejects_escaped_or_redirected_ancestors() {
    let data = tempfile::tempdir().unwrap();
    assert!(create_plain_verification_view(data.path(), "../escape", "attempt").is_err());
    let view = create_plain_verification_view(data.path(), "run", "attempt").unwrap();
    assert!(view.is_dir());

    let blocked = tempfile::tempdir().unwrap();
    fs::write(blocked.path().join("routed-verification"), b"file").unwrap();
    assert!(create_plain_verification_view(blocked.path(), "run", "attempt").is_err());

    #[cfg(windows)]
    {
        let redirected = tempfile::tempdir().unwrap();
        let target = tempfile::tempdir().unwrap();
        let link = redirected.path().join("routed-verification");
        if std::os::windows::fs::symlink_dir(target.path(), &link).is_ok() {
            assert!(create_plain_verification_view(redirected.path(), "run", "attempt").is_err());
        }
    }
}

fn git(repo: &Path, args: &[&str]) -> String {
    let mut command = Command::new("git");
    command.args(args).current_dir(repo);
    for (key, _) in std::env::vars_os() {
        if key
            .to_string_lossy()
            .to_ascii_uppercase()
            .starts_with("GIT_")
        {
            command.env_remove(key);
        }
    }
    let output = command.output().unwrap();
    assert!(
        output.status.success(),
        "git {}: {}",
        args.join(" "),
        String::from_utf8_lossy(&output.stderr)
    );
    String::from_utf8(output.stdout).unwrap().trim().to_owned()
}

fn git_with_input(repo: &Path, args: &[&str], input: &[u8]) -> String {
    let mut child = Command::new("git")
        .args(args)
        .current_dir(repo)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .unwrap();
    child.stdin.take().unwrap().write_all(input).unwrap();
    let output = child.wait_with_output().unwrap();
    assert!(
        output.status.success(),
        "git {}: {}",
        args.join(" "),
        String::from_utf8_lossy(&output.stderr)
    );
    String::from_utf8(output.stdout).unwrap().trim().to_owned()
}

fn committed_base(repo: &Path) -> BaseSnapshot {
    let tree = git(repo, &["rev-parse", "--verify", "HEAD^{tree}"]);
    BaseSnapshot {
        repository_identity: canonical_repo_root(repo)
            .unwrap()
            .to_string_lossy()
            .into_owned(),
        git_revision: git(repo, &["rev-parse", "--verify", "HEAD^{commit}"]),
        snapshot_digest: canonical_digest(&(1_u32, "git-head-tree", tree), 1).unwrap(),
    }
}

fn repo() -> (tempfile::TempDir, std::path::PathBuf, BaseSnapshot) {
    let root = tempfile::tempdir().unwrap();
    let repo = root.path().join("source");
    fs::create_dir(&repo).unwrap();
    git(&repo, &["init", "-q"]);
    fs::write(repo.join("result.txt"), "first\n").unwrap();
    git(&repo, &["add", "result.txt"]);
    git(
        &repo,
        &[
            "-c",
            "user.name=Test",
            "-c",
            "user.email=test@pytxo.local",
            "commit",
            "-qm",
            "first",
        ],
    );
    let base = committed_base(&repo);
    (root, repo, base)
}

#[test]
fn exact_reviewed_checkout_rejects_clean_crlf_normalization() {
    let (root, source, source_base) = repo();
    require_exact_reviewed_checkout(&source, &source_base, &[]).unwrap();
    let checkout = root.path().join("normalized");
    git(
        &source,
        &[
            "-c",
            "core.autocrlf=true",
            "clone",
            "--quiet",
            source.to_str().unwrap(),
            checkout.to_str().unwrap(),
        ],
    );
    assert_eq!(fs::read(checkout.join("result.txt")).unwrap(), b"first\r\n");
    assert_eq!(git(&checkout, &["status", "--porcelain"]), "");
    let error =
        require_exact_reviewed_checkout(&checkout, &committed_base(&checkout), &[]).unwrap_err();
    assert!(
        error
            .to_string()
            .contains("checkout bytes differ from Git blobs"),
        "{error}"
    );
}

#[test]
fn exact_reviewed_checkout_rejects_clean_ignored_physical_file() {
    let (_root, repo, base) = repo();
    fs::write(repo.join(".git/info/exclude"), b"scratch.log\n").unwrap();
    fs::write(
        repo.join("scratch.log"),
        b"ignored but included by Review\n",
    )
    .unwrap();
    assert_eq!(git(&repo, &["status", "--porcelain"]), "");
    let error = require_exact_reviewed_checkout(&repo, &base, &[]).unwrap_err();
    assert!(
        error
            .to_string()
            .contains("included physical file outside the Git snapshot"),
        "{error}"
    );
}

#[cfg(windows)]
#[test]
fn exact_reviewed_checkout_rejects_clean_ignored_junction() {
    let (root, repo, base) = repo();
    let outside = root.path().join("outside");
    fs::create_dir(&outside).unwrap();
    fs::write(outside.join("sentinel.txt"), b"outside the repository\n").unwrap();
    fs::write(repo.join(".git/info/exclude"), b"ignored-junction/\n").unwrap();
    let junction = repo.join("ignored-junction");
    let output = Command::new("cmd.exe")
        .args(["/D", "/C"])
        .raw_arg(format!(
            "mklink /J \"{}\" \"{}\"",
            junction.display(),
            outside.display()
        ))
        .output()
        .unwrap();
    assert!(
        output.status.success(),
        "mklink: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert_eq!(git(&repo, &["status", "--porcelain"]), "");
    let result = require_exact_reviewed_checkout(&repo, &base, &[]);
    fs::remove_dir(&junction).unwrap();
    let error = result.unwrap_err();
    assert!(error.to_string().contains("included symlink"), "{error}");
    assert_eq!(
        fs::read(outside.join("sentinel.txt")).unwrap(),
        b"outside the repository\n"
    );
}

#[test]
fn inert_reviewed_worktree_pins_commit_without_checking_out_files() {
    let (root, repo, base) = repo();
    fs::write(repo.join("result.txt"), "second\n").unwrap();
    git(&repo, &["add", "result.txt"]);
    git(
        &repo,
        &[
            "-c",
            "user.name=Test",
            "-c",
            "user.email=test@pytxo.local",
            "commit",
            "-qm",
            "second",
        ],
    );
    assert_ne!(
        git(&repo, &["rev-parse", "HEAD^{commit}"]),
        base.git_revision
    );

    let worktree = root.path().join("worker");
    prepare_reviewed_worktree(&repo, &worktree, &base).unwrap();
    assert_eq!(
        git(&worktree, &["rev-parse", "HEAD^{commit}"]),
        base.git_revision
    );
    assert!(!worktree.join("result.txt").exists());
}

#[test]
fn reviewed_worktree_rejects_wrong_base_before_creating_a_path() {
    let (root, repo, mut base) = repo();
    base.snapshot_digest = Digest::of_bytes(b"invented tree");
    let worktree = root.path().join("worker");
    assert!(prepare_reviewed_worktree(&repo, &worktree, &base).is_err());
    assert!(!worktree.exists());
}

#[test]
fn inert_preparation_does_not_invoke_a_configured_checkout_hook() {
    let (root, repo, base) = repo();
    let hooks = root.path().join("hooks");
    fs::create_dir(&hooks).unwrap();
    let hook = hooks.join("post-checkout");
    fs::write(&hook, "#!/bin/sh\nprintf fired > hook-fired\n").unwrap();
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        fs::set_permissions(&hook, fs::Permissions::from_mode(0o755)).unwrap();
    }
    git(
        &repo,
        &["config", "core.hooksPath", hooks.to_str().unwrap()],
    );

    let calibration = root.path().join("calibration");
    git(
        &repo,
        &[
            "worktree",
            "add",
            "--detach",
            calibration.to_str().unwrap(),
            &base.git_revision,
        ],
    );
    assert!(
        calibration.join("hook-fired").exists(),
        "the disposable checkout hook must be active for this regression"
    );

    let inert = root.path().join("worker");
    prepare_reviewed_worktree(&repo, &inert, &base).unwrap();
    assert!(!inert.join("hook-fired").exists());
    assert!(!inert.join("result.txt").exists());
}

#[test]
fn retained_git_input_manifest_materializes_exact_committed_bytes_without_checkout_filters() {
    let (root, repo, _base) = repo();
    fs::write(repo.join(".gitattributes"), "result.txt filter=fixture\n").unwrap();
    git(
        &repo,
        &[
            "config",
            "filter.fixture.smudge",
            "echo fired > smudge-fired; cat",
        ],
    );
    git(&repo, &["add", ".gitattributes"]);
    git(
        &repo,
        &[
            "-c",
            "user.name=Test",
            "-c",
            "user.email=test@pytxo.local",
            "commit",
            "-qm",
            "attributes",
        ],
    );
    let base = committed_base(&repo);
    let calibration = root.path().join("calibration");
    git(
        &repo,
        &[
            "worktree",
            "add",
            "--detach",
            calibration.to_str().unwrap(),
            &base.git_revision,
        ],
    );
    assert!(
        calibration.join("smudge-fired").exists(),
        "normal checkout must execute the configured fixture filter"
    );
    let manifest = capture_reviewed_inputs(&repo, &base).unwrap();
    assert!(manifest.files.iter().any(|file| file.path == "result.txt"));
    let worker = root.path().join("worker");
    prepare_reviewed_worktree(&repo, &worker, &base).unwrap();
    materialize_reviewed_inputs(&repo, &worker, &manifest).unwrap();
    assert_eq!(fs::read(worker.join("result.txt")).unwrap(), b"first\n");
    assert_eq!(
        fs::read(worker.join(".gitattributes")).unwrap(),
        b"result.txt filter=fixture\n"
    );
    assert!(!worker.join("smudge-fired").exists());
    assert_eq!(manifest.base, base);
}

#[test]
fn materialization_rejects_changed_manifest_before_writing_any_file() {
    let (root, repo, base) = repo();
    let mut manifest = capture_reviewed_inputs(&repo, &base).unwrap();
    manifest.files[0].content_digest = Digest::of_bytes(b"forged");
    let worker = root.path().join("worker");
    prepare_reviewed_worktree(&repo, &worker, &base).unwrap();
    assert!(materialize_reviewed_inputs(&repo, &worker, &manifest).is_err());
    assert!(!worker.join("result.txt").exists());
}

#[test]
fn materialization_refuses_a_preexisting_worktree_file() {
    let (root, repo, base) = repo();
    let manifest = capture_reviewed_inputs(&repo, &base).unwrap();
    let worker = root.path().join("worker");
    prepare_reviewed_worktree(&repo, &worker, &base).unwrap();
    fs::write(worker.join("result.txt"), "foreign bytes").unwrap();
    assert!(materialize_reviewed_inputs(&repo, &worker, &manifest).is_err());
    assert_eq!(
        fs::read(worker.join("result.txt")).unwrap(),
        b"foreign bytes"
    );
}

#[test]
fn physical_input_attestation_rejects_modified_or_extra_files() {
    let (root, repo, base) = repo();
    let manifest = capture_reviewed_inputs(&repo, &base).unwrap();
    let worker = root.path().join("worker");
    prepare_reviewed_worktree(&repo, &worker, &base).unwrap();
    materialize_reviewed_inputs(&repo, &worker, &manifest).unwrap();
    fs::write(worker.join("result.txt"), "changed after materialization").unwrap();
    assert!(verify_materialized_reviewed_inputs(&repo, &worker, &manifest).is_err());
    fs::write(worker.join("result.txt"), "first\n").unwrap();
    fs::write(worker.join("unreviewed.txt"), "extra").unwrap();
    assert!(verify_materialized_reviewed_inputs(&repo, &worker, &manifest).is_err());
}

#[cfg(unix)]
#[test]
fn physical_input_attestation_rejects_mode_drift_without_byte_drift() {
    use std::os::unix::fs::PermissionsExt;

    let (root, repo, base) = repo();
    let manifest = capture_reviewed_inputs(&repo, &base).unwrap();
    let worker = root.path().join("worker");
    prepare_reviewed_worktree(&repo, &worker, &base).unwrap();
    materialize_reviewed_inputs(&repo, &worker, &manifest).unwrap();
    fs::set_permissions(worker.join("result.txt"), fs::Permissions::from_mode(0o600)).unwrap();
    assert!(verify_materialized_reviewed_inputs(&repo, &worker, &manifest).is_err());
}

#[test]
fn sealed_physical_output_materializes_a_fresh_view_and_detects_checker_mutation() {
    let (root, repo, base) = repo();
    let input = capture_reviewed_inputs(&repo, &base).unwrap();
    assert_eq!(
        read_reviewed_claim_text(&repo, &input, "result.txt").unwrap(),
        "first\n"
    );
    assert!(read_reviewed_claim_text(&repo, &input, "../outside.txt").is_err());
    let mut mismatched = input.clone();
    mismatched.files[0].content_digest = Digest::of_bytes(b"different");
    assert!(read_reviewed_claim_text(&repo, &mismatched, "result.txt").is_err());
    let worker = root.path().join("worker");
    prepare_reviewed_worktree(&repo, &worker, &base).unwrap();
    materialize_reviewed_inputs(&repo, &worker, &input).unwrap();
    fs::write(worker.join("result.txt"), b"worker result\n").unwrap();
    fs::create_dir(worker.join("generated")).unwrap();
    fs::write(worker.join("generated").join("proof.txt"), b"proof\n").unwrap();

    let sealed = capture_sealed_output(
        &repo,
        &worker,
        &input,
        &["result.txt".into(), "generated".into()],
    )
    .unwrap();
    assert_eq!(sealed.schema_version, 2);
    let encoded = serde_json::to_vec(&sealed).unwrap();
    let retained = serde_json::from_slice(&encoded).unwrap();
    let view = root.path().join("verification");
    fs::create_dir(&view).unwrap();
    materialize_sealed_output(&view, &retained).unwrap();
    assert_eq!(
        fs::read(view.join("result.txt")).unwrap(),
        b"worker result\n"
    );
    assert_eq!(
        fs::read(view.join("generated").join("proof.txt")).unwrap(),
        b"proof\n"
    );
    fs::create_dir(view.join("checker-extra-empty-dir")).unwrap();
    assert!(verify_sealed_output_view(&view, &retained).is_err());
    fs::remove_dir(view.join("checker-extra-empty-dir")).unwrap();
    verify_sealed_output_view(&view, &retained).unwrap();
    fs::write(view.join("result.txt"), b"checker mutated output\n").unwrap();
    assert!(verify_sealed_output_view(&view, &retained).is_err());
}

#[test]
fn one_file_text_proposal_seals_only_reviewed_bytes_without_worker_write() {
    let (root, repo, base) = repo();
    let input = capture_reviewed_inputs(&repo, &base).unwrap();
    let worker = root.path().join("worker");
    prepare_reviewed_worktree(&repo, &worker, &base).unwrap();
    materialize_reviewed_inputs(&repo, &worker, &input).unwrap();

    let proposed = b"proposed by worker\n";
    let sealed =
        seal_one_existing_claimed_text_proposal(&repo, &worker, &input, "result.txt", proposed)
            .unwrap();
    assert_eq!(fs::read(worker.join("result.txt")).unwrap(), b"first\n");
    assert_eq!(sealed.files.len(), 1);
    assert_eq!(sealed.files[0].content_digest, Digest::of_bytes(proposed));
    let view = root.path().join("proposal-view");
    fs::create_dir(&view).unwrap();
    materialize_sealed_output(&view, &sealed).unwrap();
    assert_eq!(fs::read(view.join("result.txt")).unwrap(), proposed);

    assert!(seal_one_existing_claimed_text_proposal(
        &repo,
        &worker,
        &input,
        "result.txt",
        b"first\n"
    )
    .is_err());
    assert!(
        seal_one_existing_claimed_text_proposal(&repo, &worker, &input, "new.txt", proposed)
            .is_err()
    );
    assert!(seal_one_existing_claimed_text_proposal(
        &repo,
        &worker,
        &input,
        "../outside.txt",
        proposed
    )
    .is_err());
    assert!(
        seal_one_existing_claimed_text_proposal(&repo, &worker, &input, "result.txt", b"\xff")
            .is_err()
    );
    assert!(seal_one_existing_claimed_text_proposal(
        &repo,
        &worker,
        &input,
        "result.txt",
        &vec![b'x'; 16 * 1024 + 1]
    )
    .is_err());

    fs::write(worker.join("result.txt"), b"worker changed\n").unwrap();
    assert!(seal_one_existing_claimed_text_proposal(
        &repo,
        &worker,
        &input,
        "result.txt",
        proposed
    )
    .is_err());
    fs::write(worker.join("result.txt"), b"first\n").unwrap();
    fs::write(worker.join("unclaimed.txt"), b"extra\n").unwrap();
    assert!(seal_one_existing_claimed_text_proposal(
        &repo,
        &worker,
        &input,
        "result.txt",
        proposed
    )
    .is_err());
}

#[test]
fn sealed_output_rejects_changed_bytes_and_unreviewed_view_entries() {
    let (root, repo, base) = repo();
    let input = capture_reviewed_inputs(&repo, &base).unwrap();
    let worker = root.path().join("worker");
    prepare_reviewed_worktree(&repo, &worker, &base).unwrap();
    materialize_reviewed_inputs(&repo, &worker, &input).unwrap();
    let mut sealed = capture_sealed_output(&repo, &worker, &input, &["result.txt".into()]).unwrap();
    sealed.files[0].content_base64.push('A');
    let view = root.path().join("verification");
    fs::create_dir(&view).unwrap();
    assert!(materialize_sealed_output(&view, &sealed).is_err());
    assert!(fs::read_dir(&view).unwrap().next().is_none());

    let mut false_legacy_mode =
        capture_sealed_output(&repo, &worker, &input, &["result.txt".into()]).unwrap();
    false_legacy_mode.schema_version = 1;
    false_legacy_mode.files[0].mode = Some(0o755);
    assert!(materialize_sealed_output(&view, &false_legacy_mode).is_err());
    assert!(fs::read_dir(&view).unwrap().next().is_none());

    let sealed = capture_sealed_output(&repo, &worker, &input, &["result.txt".into()]).unwrap();
    materialize_sealed_output(&view, &sealed).unwrap();
    fs::write(view.join(".git"), "unreviewed Git metadata").unwrap();
    assert!(verify_sealed_output_view(&view, &sealed).is_err());
}

#[test]
fn sealing_rejects_unclaimed_edits_empty_directories_and_oversized_files() {
    let (root, repo, base) = repo();
    let input = capture_reviewed_inputs(&repo, &base).unwrap();
    let worker = root.path().join("worker");
    prepare_reviewed_worktree(&repo, &worker, &base).unwrap();
    materialize_reviewed_inputs(&repo, &worker, &input).unwrap();
    fs::write(worker.join("outside.txt"), b"unclaimed").unwrap();
    assert!(capture_sealed_output(&repo, &worker, &input, &["result.txt".into()]).is_err());
    fs::remove_file(worker.join("outside.txt")).unwrap();
    fs::create_dir(worker.join("empty-dir")).unwrap();
    assert!(capture_sealed_output(&repo, &worker, &input, &["result.txt".into()]).is_err());
    assert!(verify_materialized_reviewed_inputs(&repo, &worker, &input).is_err());
    fs::remove_dir(worker.join("empty-dir")).unwrap();
    fs::write(worker.join("result.txt"), vec![b'x'; 9 * 1024 * 1024 + 1]).unwrap();
    assert!(capture_sealed_output(&repo, &worker, &input, &["result.txt".into()]).is_err());
}

#[test]
fn dependent_worktree_inherits_exact_parent_bytes_and_seals_only_its_claim() {
    let (root, repo, base) = repo();
    let input = capture_reviewed_inputs(&repo, &base).unwrap();
    let parent = root.path().join("parent");
    prepare_reviewed_worktree(&repo, &parent, &base).unwrap();
    materialize_reviewed_inputs(&repo, &parent, &input).unwrap();
    fs::write(parent.join("seed.txt"), b"from parent\n").unwrap();
    let retained_parent =
        capture_sealed_output(&repo, &parent, &input, &["seed.txt".into()]).unwrap();

    let child = root.path().join("child");
    prepare_reviewed_worktree(&repo, &child, &base).unwrap();
    let mut corrupt = retained_parent.clone();
    corrupt.files[0].content_digest = Digest::of_bytes(b"forged parent");
    assert!(materialize_dependency_output(&repo, &child, &input, &corrupt).is_err());
    assert!(!child.join("result.txt").exists());
    materialize_dependency_output(&repo, &child, &input, &retained_parent).unwrap();
    verify_materialized_dependency_output(&repo, &child, &input, &retained_parent).unwrap();
    assert_eq!(fs::read(child.join("seed.txt")).unwrap(), b"from parent\n");
    fs::write(child.join("seed.txt"), b"tampered\n").unwrap();
    assert!(
        verify_materialized_dependency_output(&repo, &child, &input, &retained_parent).is_err()
    );
    assert!(capture_sealed_output_after_dependency(
        &repo,
        &child,
        &input,
        &retained_parent,
        &["child.txt".into()],
    )
    .is_err());
    fs::write(child.join("seed.txt"), b"from parent\n").unwrap();
    fs::write(child.join("child.txt"), b"read parent seed\n").unwrap();
    let output = capture_sealed_output_after_dependency(
        &repo,
        &child,
        &input,
        &retained_parent,
        &["child.txt".into()],
    )
    .unwrap();
    assert!(output.files.iter().any(|file| file.path == "seed.txt"));
    assert!(output.files.iter().any(|file| file.path == "child.txt"));
}

#[cfg(unix)]
#[test]
fn dependent_sealing_rejects_unclaimed_mode_drift_without_byte_drift() {
    use std::os::unix::fs::PermissionsExt;

    let (root, repo, base) = repo();
    let input = capture_reviewed_inputs(&repo, &base).unwrap();
    let parent = root.path().join("parent");
    prepare_reviewed_worktree(&repo, &parent, &base).unwrap();
    materialize_reviewed_inputs(&repo, &parent, &input).unwrap();
    fs::write(parent.join("seed.txt"), b"from parent\n").unwrap();
    fs::set_permissions(parent.join("seed.txt"), fs::Permissions::from_mode(0o644)).unwrap();
    let retained_parent =
        capture_sealed_output(&repo, &parent, &input, &["seed.txt".into()]).unwrap();

    let child = root.path().join("child");
    prepare_reviewed_worktree(&repo, &child, &base).unwrap();
    materialize_dependency_output(&repo, &child, &input, &retained_parent).unwrap();
    fs::set_permissions(child.join("seed.txt"), fs::Permissions::from_mode(0o600)).unwrap();
    fs::write(child.join("child.txt"), b"child\n").unwrap();
    assert!(capture_sealed_output_after_dependency(
        &repo,
        &child,
        &input,
        &retained_parent,
        &["child.txt".into()],
    )
    .is_err());
}

#[test]
fn same_commit_in_a_foreign_worktree_is_not_reviewed_source_ownership() {
    let (root, repo, base) = repo();
    let manifest = capture_reviewed_inputs(&repo, &base).unwrap();
    let clone = root.path().join("clone");
    git(
        root.path(),
        &[
            "clone",
            "-q",
            repo.to_str().unwrap(),
            clone.to_str().unwrap(),
        ],
    );
    let foreign_worker = root.path().join("foreign-worker");
    prepare_reviewed_worktree(&clone, &foreign_worker, &committed_base(&clone)).unwrap();
    assert!(materialize_reviewed_inputs(&repo, &foreign_worker, &manifest).is_err());
}

#[test]
fn launch_attestation_rejects_empty_manifest_and_changed_git_link() {
    let (root, repo, base) = repo();
    let manifest = capture_reviewed_inputs(&repo, &base).unwrap();
    let worker = root.path().join("worker");
    prepare_reviewed_worktree(&repo, &worker, &base).unwrap();
    let mut empty = manifest.clone();
    empty.files.clear();
    assert!(verify_materialized_reviewed_inputs(&repo, &worker, &empty).is_err());
    materialize_reviewed_inputs(&repo, &worker, &manifest).unwrap();
    let clone = root.path().join("clone");
    git(
        root.path(),
        &[
            "clone",
            "-q",
            repo.to_str().unwrap(),
            clone.to_str().unwrap(),
        ],
    );
    let foreign_worker = root.path().join("foreign-worker");
    prepare_reviewed_worktree(&clone, &foreign_worker, &committed_base(&clone)).unwrap();
    fs::write(
        worker.join(".git"),
        fs::read(foreign_worker.join(".git")).unwrap(),
    )
    .unwrap();
    assert!(verify_materialized_reviewed_inputs(&repo, &worker, &manifest).is_err());
}

#[test]
fn reviewed_input_capture_rejects_git_symlinks() {
    let (_root, repo, _base) = repo();
    let target = repo.join("link-target.txt");
    fs::write(&target, "result.txt").unwrap();
    let oid = git(&repo, &["hash-object", "-w", target.to_str().unwrap()]);
    git(
        &repo,
        &[
            "update-index",
            "--add",
            "--cacheinfo",
            &format!("120000,{oid},escape-link"),
        ],
    );
    git(
        &repo,
        &[
            "-c",
            "user.name=Test",
            "-c",
            "user.email=test@pytxo.local",
            "commit",
            "-qm",
            "symlink",
        ],
    );
    let base = committed_base(&repo);
    assert!(capture_reviewed_inputs(&repo, &base).is_err());
}

#[test]
fn reviewed_input_capture_rejects_case_colliding_git_paths() {
    let (_root, repo, _base) = repo();
    let oid = git(&repo, &["rev-parse", "HEAD:result.txt"]);
    git(
        &repo,
        &[
            "update-index",
            "--add",
            "--cacheinfo",
            &format!("100644,{oid},Result.txt"),
        ],
    );
    git(
        &repo,
        &[
            "-c",
            "user.name=Test",
            "-c",
            "user.email=test@pytxo.local",
            "commit",
            "-qm",
            "case collision",
        ],
    );
    let base = committed_base(&repo);
    assert!(capture_reviewed_inputs(&repo, &base).is_err());
}

#[test]
fn oversized_git_tree_output_is_stopped_at_the_stream_limit() {
    let (_root, repo, _base) = repo();
    let oid = git(&repo, &["rev-parse", "HEAD:result.txt"]);
    let mut tree_input = Vec::new();
    for index in 0..10_000 {
        tree_input
            .extend(format!("100644 blob {oid}\tfile-{index:05}-{}\0", "x".repeat(170)).as_bytes());
    }
    let tree = git_with_input(&repo, &["mktree", "-z"], &tree_input);
    let commit = git(
        &repo,
        &[
            "-c",
            "user.name=Test",
            "-c",
            "user.email=test@pytxo.local",
            "commit-tree",
            &tree,
            "-m",
            "large tree",
        ],
    );
    git(&repo, &["update-ref", "HEAD", &commit]);
    let base = committed_base(&repo);
    let error = capture_reviewed_inputs(&repo, &base).unwrap_err();
    assert!(
        error.to_string().contains("output exceeds limit"),
        "{error}"
    );
}
