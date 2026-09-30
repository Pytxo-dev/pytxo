#![cfg(windows)]

use pytxo_core::{PytxoError, Result};
use pytxo_runner::owned_launch::{
    run_direct_owned_launch, DirectOwnedLaunchSpec, LaunchCallbacks, LaunchGuard, LaunchIntent,
    OwnedLaunchReceipt, OwnedLaunchSpec, OwnedOutcome, OwnedProcess, OwnedTransport, PinnedFile,
};
use std::cell::Cell;
use std::collections::BTreeMap;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::time::{Duration, Instant};

struct Callbacks {
    output: PathBuf,
    registered: bool,
    settled: usize,
    fail_registration: bool,
    cancel_at_registration: bool,
    cancelled: Arc<AtomicBool>,
    permit_delay: Duration,
    registration_delay: Duration,
}
struct Guard<'a>(&'a mut Callbacks);
impl LaunchGuard for Guard<'_> {
    fn permit_create(&mut self, _: &LaunchIntent) -> Result<()> {
        std::thread::sleep(self.0.permit_delay);
        Ok(())
    }

    fn register(&mut self, process: &OwnedProcess) -> Result<()> {
        assert!(!self.0.output.exists(), "payload ran before registration");
        assert!(process.pid > 0);
        assert!(process
            .start_identity
            .as_deref()
            .is_some_and(|id| id.starts_with("windows-filetime:")));
        std::thread::sleep(self.0.registration_delay);
        assert!(!self.0.output.exists(), "payload ran during registration");
        self.0.registered = true;
        if self.0.cancel_at_registration {
            self.0.cancelled.store(true, Ordering::SeqCst);
        }
        if self.0.fail_registration {
            return Err(PytxoError::Runner("injected register failure".into()));
        }
        Ok(())
    }
    fn cancelled(&mut self) -> Result<bool> {
        Ok(self.0.cancelled.load(Ordering::SeqCst))
    }
}
impl LaunchCallbacks for Callbacks {
    fn authorize(&mut self, intent: &LaunchIntent) -> Result<Box<dyn LaunchGuard + '_>> {
        assert!(intent.job_name.starts_with("Local\\PytxoAttempt-"));
        Ok(Box::new(Guard(self)))
    }
    fn settle(&mut self, receipt: &OwnedLaunchReceipt) -> Result<()> {
        assert_eq!(receipt.active_processes, Some(0), "{receipt:?}");
        self.settled += 1;
        Ok(())
    }
}

#[test]
fn owned_launch_debug_redacts_private_process_inputs() {
    let temp = tempfile::tempdir().unwrap();
    let (mut direct, _) = fixture(temp.path(), "exit 0");
    direct.arguments.push("private-argument-sentinel".into());
    direct
        .environment
        .insert("TOKEN".into(), "private-environment-sentinel".into());
    let owned = OwnedLaunchSpec {
        bootstrap_host: direct.executable.clone(),
        executable: direct.executable.clone(),
        dependencies: vec![],
        arguments: direct.arguments.clone(),
        environment: direct.environment.clone(),
        working_directory: direct.working_directory.clone(),
        stdin: b"private-stdin-sentinel".to_vec(),
        transport: OwnedTransport::Subprocess,
        barrier_timeout: Duration::from_secs(1),
        execution_timeout: Duration::from_secs(1),
        settlement_timeout: Duration::from_secs(1),
        output_limit: 4096,
    };
    for debug in [format!("{owned:?}"), format!("{direct:?}")] {
        assert!(!debug.contains("private-argument-sentinel"));
        assert!(!debug.contains("private-environment-sentinel"));
        assert!(!debug.contains("private-stdin-sentinel"));
    }
}

fn fixture(dir: &Path, command: &str) -> (DirectOwnedLaunchSpec, Callbacks) {
    let root = PathBuf::from(std::env::var_os("SystemRoot").unwrap());
    let output = dir.join("result.txt");
    (
        DirectOwnedLaunchSpec {
            executable: PinnedFile::observe(root.join("System32/cmd.exe")).unwrap(),
            arguments: [
                vec!["/D".into(), "/Q".into(), "/C".into()],
                command.split_whitespace().map(str::to_owned).collect(),
            ]
            .concat(),
            windows_cmd_verbatim_tail: false,
            environment: BTreeMap::new(),
            working_directory: dir.into(),
            execution_timeout: Duration::from_secs(5),
            settlement_timeout: Duration::from_secs(3),
            output_limit: 4096,
        },
        Callbacks {
            output,
            registered: false,
            settled: 0,
            fail_registration: false,
            cancel_at_registration: false,
            cancelled: Arc::new(AtomicBool::new(false)),
            permit_delay: Duration::ZERO,
            registration_delay: Duration::from_millis(50),
        },
    )
}

#[test]
fn direct_payload_is_registered_before_write_and_settled_after_job_zero() {
    let dir = tempfile::tempdir().unwrap();
    let (spec, mut callbacks) = fixture(dir.path(), "echo pytxo-direct>result.txt");
    let receipt = run_direct_owned_launch(&spec, &mut callbacks, || Ok(false)).unwrap();
    assert_eq!(
        receipt.outcome,
        OwnedOutcome::Succeeded,
        "{receipt:?} stdout={:?} stderr={:?}",
        receipt.stdout,
        receipt.stderr
    );
    assert_eq!(receipt.protocol, "pytxo-direct-suspended/1");
    assert!(callbacks.registered);
    assert_eq!(callbacks.settled, 1);
    assert_eq!(receipt.active_processes, Some(0));
    assert_eq!(receipt.payload_exit_code, Some(0));
    assert_eq!(
        std::fs::read(callbacks.output).unwrap(),
        b"pytxo-direct\r\n"
    );
    let mut sensitive_receipt = receipt.clone();
    sensitive_receipt.stdout = "private-stdout-sentinel".into();
    sensitive_receipt.stderr = "private-stderr-sentinel".into();
    sensitive_receipt.error = Some("private-error-sentinel".into());
    sensitive_receipt.chain = vec![PinnedFile {
        path: PathBuf::from("C:/private-chain-sentinel.exe"),
        sha256: "private-chain-hash-sentinel".into(),
    }];
    let debug = format!("{sensitive_receipt:?}");
    for private in [
        "private-stdout-sentinel",
        "private-stderr-sentinel",
        "private-error-sentinel",
        "private-chain-sentinel",
    ] {
        assert!(!debug.contains(private));
    }
}

#[test]
fn reviewed_cmd_tail_preserves_quotes_and_records_distinct_lowering() {
    let dir = tempfile::tempdir().unwrap();
    std::fs::write(
        dir.path().join("checked.txt"),
        b"Pytxo native routing verified\r\n",
    )
    .unwrap();
    let (mut spec, mut callbacks) = fixture(dir.path(), "exit 0");
    // The reviewed recipe stores the ordinary Windows spelling, while the
    // fixture helper above uses PathBuf::join with a forward slash.
    let cmd = PathBuf::from(std::env::var_os("SystemRoot").unwrap())
        .join("System32")
        .join("cmd.exe");
    spec.executable = PinnedFile::observe(cmd).unwrap();
    let findstr = PathBuf::from(std::env::var_os("SystemRoot").unwrap())
        .join("System32/findstr.exe")
        .to_string_lossy()
        .into_owned();
    spec.arguments = vec![
        "/D".into(),
        "/C".into(),
        format!(
            "{findstr} /C:\"Pytxo native routing verified\" checked.txt >NUL && echo passed>result.txt"
        ),
    ];
    spec.windows_cmd_verbatim_tail = true;
    let receipt = run_direct_owned_launch(&spec, &mut callbacks, || Ok(false)).unwrap();
    assert_eq!(
        receipt.outcome,
        OwnedOutcome::Succeeded,
        "{receipt:?} stdout={:?} stderr={:?}",
        receipt.stdout,
        receipt.stderr
    );
    assert_eq!(receipt.payload_exit_code, Some(0));
    assert_eq!(
        receipt.argument_lowering,
        "windows-createprocess-cmd-verbatim-tail/v1"
    );
    assert_eq!(std::fs::read(callbacks.output).unwrap(), b"passed\r\n");

    let other = tempfile::tempdir().unwrap();
    let (mut structured, mut callbacks) = fixture(other.path(), "exit 0");
    structured.arguments = vec!["/D".into(), "/C".into(), "exit 0".into()];
    let structured_receipt =
        run_direct_owned_launch(&structured, &mut callbacks, || Ok(false)).unwrap();
    structured.windows_cmd_verbatim_tail = true;
    let verbatim_receipt =
        run_direct_owned_launch(&structured, &mut callbacks, || Ok(false)).unwrap();
    assert_eq!(structured_receipt.outcome, OwnedOutcome::Succeeded);
    assert_eq!(verbatim_receipt.outcome, OwnedOutcome::Succeeded);
    assert_ne!(
        structured_receipt.bundle_sha256,
        verbatim_receipt.bundle_sha256
    );
}

#[test]
fn verbatim_tail_rejects_unreviewed_shape() {
    let dir = tempfile::tempdir().unwrap();
    let (mut spec, mut callbacks) = fixture(dir.path(), "exit 0");
    spec.windows_cmd_verbatim_tail = true;
    spec.arguments = vec!["/D".into(), "/C".into(), "echo ok".into(), "extra".into()];
    assert!(run_direct_owned_launch(&spec, &mut callbacks, || Ok(false)).is_err());
    spec.arguments = vec!["/D".into(), "/C".into(), "echo ok\r\nwhoami".into()];
    assert!(run_direct_owned_launch(&spec, &mut callbacks, || Ok(false)).is_err());
    spec.arguments = vec!["/D".into(), "/C".into(), "echo ok".into()];
    let root = PathBuf::from(std::env::var_os("SystemRoot").unwrap());
    spec.executable = PinnedFile::observe(
        root.join("System32")
            .join("..")
            .join("System32")
            .join("cmd.exe"),
    )
    .unwrap();
    assert!(run_direct_owned_launch(&spec, &mut callbacks, || Ok(false)).is_err());
    assert!(!callbacks.registered);
    assert_eq!(callbacks.settled, 0);
}

#[test]
fn failed_registration_never_resumes_payload() {
    let dir = tempfile::tempdir().unwrap();
    let (spec, mut callbacks) = fixture(dir.path(), "echo must-not-run>result.txt");
    callbacks.fail_registration = true;
    let receipt = run_direct_owned_launch(&spec, &mut callbacks, || Ok(false)).unwrap();
    assert_eq!(
        receipt.outcome,
        OwnedOutcome::RecoveryRequired,
        "{receipt:?}"
    );
    assert!(!receipt.barrier_released);
    assert_eq!(receipt.active_processes, Some(0));
    assert!(!callbacks.output.exists());
    assert_eq!(callbacks.settled, 1);
}

#[test]
fn deadline_during_create_permission_never_spawns_direct_payload() {
    let dir = tempfile::tempdir().unwrap();
    let (mut spec, mut callbacks) = fixture(dir.path(), "echo must-not-run>result.txt");
    spec.execution_timeout = Duration::from_millis(500);
    callbacks.permit_delay = Duration::from_millis(800);
    let receipt = run_direct_owned_launch(&spec, &mut callbacks, || Ok(false)).unwrap();
    assert!(!callbacks.registered);
    assert!(!receipt.barrier_released);
    assert_eq!(receipt.active_processes, Some(0));
    assert!(receipt
        .error
        .as_deref()
        .unwrap_or_default()
        .contains("deadline"));
    assert!(receipt.elapsed_ms >= 800);
    assert!(!callbacks.output.exists());
}

#[test]
fn deadline_during_registration_never_resumes_direct_payload() {
    let dir = tempfile::tempdir().unwrap();
    let (mut spec, mut callbacks) = fixture(dir.path(), "echo must-not-run>result.txt");
    spec.execution_timeout = Duration::from_millis(500);
    callbacks.registration_delay = Duration::from_millis(800);
    let receipt = run_direct_owned_launch(&spec, &mut callbacks, || Ok(false)).unwrap();
    assert!(callbacks.registered);
    assert!(!receipt.barrier_released);
    assert_eq!(receipt.active_processes, Some(0));
    assert!(receipt
        .error
        .as_deref()
        .unwrap_or_default()
        .contains("deadline"));
    assert!(receipt.elapsed_ms >= 800);
    assert!(!callbacks.output.exists());
}

#[test]
fn completed_direct_payload_observed_after_deadline_cannot_succeed() {
    let dir = tempfile::tempdir().unwrap();
    let (mut spec, mut callbacks) = fixture(dir.path(), "exit 0");
    spec.execution_timeout = Duration::from_millis(500);
    let calls = Cell::new(0);
    let receipt = run_direct_owned_launch(&spec, &mut callbacks, || {
        calls.set(calls.get() + 1);
        if calls.get() == 4 {
            std::thread::sleep(Duration::from_millis(800));
        }
        Ok(false)
    })
    .unwrap();
    assert!(callbacks.registered && receipt.barrier_released);
    assert_ne!(receipt.outcome, OwnedOutcome::Succeeded);
    assert!(receipt
        .error
        .as_deref()
        .unwrap_or_default()
        .contains("deadline"));
    assert_eq!(receipt.active_processes, Some(0));
}

#[test]
fn cancellation_before_and_after_resume_reaches_job_zero() {
    let dir = tempfile::tempdir().unwrap();
    let (spec, mut callbacks) = fixture(dir.path(), "echo must-not-run>result.txt");
    callbacks.cancel_at_registration = true;
    let token = callbacks.cancelled.clone();
    let receipt =
        run_direct_owned_launch(&spec, &mut callbacks, || Ok(token.load(Ordering::SeqCst)))
            .unwrap();
    assert_eq!(receipt.outcome, OwnedOutcome::Cancelled, "{receipt:?}");
    assert!(!receipt.barrier_released);
    assert_eq!(receipt.active_processes, Some(0));
    assert!(!callbacks.output.exists());

    let second = tempfile::tempdir().unwrap();
    let (spec, mut callbacks) = fixture(second.path(), "for /L %i in (1,1,2147483647) do @rem");
    let started = Instant::now();
    let receipt = run_direct_owned_launch(&spec, &mut callbacks, || {
        Ok(started.elapsed() > Duration::from_millis(150))
    })
    .unwrap();
    assert_eq!(receipt.outcome, OwnedOutcome::Cancelled, "{receipt:?}");
    assert!(receipt.barrier_released);
    assert_eq!(receipt.active_processes, Some(0));
    assert!(receipt.terminated_job);
}
