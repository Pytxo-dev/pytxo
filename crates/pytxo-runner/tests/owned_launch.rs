#![cfg(windows)]

use pytxo_core::Result;
use pytxo_runner::owned_launch::*;
use std::collections::BTreeMap;
use std::fs;
use std::os::windows::process::CommandExt;
use std::path::PathBuf;
use std::process::Command;
use std::sync::{
    atomic::{AtomicBool, Ordering},
    Arc,
};
use std::time::{Duration, Instant};

struct Callbacks {
    output: PathBuf,
    cancel: Arc<AtomicBool>,
    fail_register: bool,
    fail_settle: bool,
    cancel_at_register: bool,
    delay: Duration,
    registered: bool,
    tamper_on_authorize: Option<PathBuf>,
    tamper_after_register: Option<PathBuf>,
    settlements: usize,
}
struct Guard<'a>(&'a mut Callbacks);
impl LaunchGuard for Guard<'_> {
    fn permit_create(&mut self, _: &LaunchIntent) -> Result<()> {
        Ok(())
    }

    fn register(&mut self, process: &OwnedProcess) -> Result<()> {
        assert!(
            !self.0.output.exists(),
            "payload ran before registration/release"
        );
        assert!(process
            .start_identity
            .as_ref()
            .is_some_and(|x| x.starts_with("windows-filetime:")));
        std::thread::sleep(self.0.delay);
        assert!(
            !self.0.output.exists(),
            "payload ran while registration was delayed"
        );
        self.0.registered = true;
        if let Some(path) = &self.0.tamper_after_register {
            fs::write(path, b"changed after registration")?;
        }
        if self.0.cancel_at_register {
            self.0.cancel.store(true, Ordering::SeqCst);
        }
        if self.0.fail_register {
            return Err(pytxo_core::PytxoError::Runner(
                "injected registration failure".into(),
            ));
        }
        Ok(())
    }
    fn cancelled(&mut self) -> Result<bool> {
        Ok(self.0.cancel.load(Ordering::SeqCst))
    }
}
impl LaunchCallbacks for Callbacks {
    fn authorize(&mut self, intent: &LaunchIntent) -> Result<Box<dyn LaunchGuard + '_>> {
        assert!(intent.job_name.starts_with("Local\\PytxoAttempt-"));
        if let Some(path) = &self.tamper_on_authorize {
            fs::write(path, b"changed under gate")?;
        }
        Ok(Box::new(Guard(self)))
    }
    fn settle(&mut self, receipt: &OwnedLaunchReceipt) -> Result<()> {
        self.settlements += 1;
        assert_eq!(receipt.active_processes, Some(0));
        if self.fail_settle {
            return Err(pytxo_core::PytxoError::Runner(
                "injected settlement failure".into(),
            ));
        }
        Ok(())
    }
}

fn fixture(
    transport: OwnedTransport,
    mode: &str,
) -> (tempfile::TempDir, OwnedLaunchSpec, Callbacks) {
    let dir = tempfile::tempdir().unwrap();
    let source = dir.path().join("fixture.cs");
    let exe = dir.path().join("fixture with spaces.exe");
    fs::write(&source, include_str!("fixtures/owned_launch.cs")).unwrap();
    let root = PathBuf::from(std::env::var_os("SystemRoot").unwrap());
    let compiler = root.join("Microsoft.NET/Framework64/v4.0.30319/csc.exe");
    let result = Command::new(compiler)
        .creation_flags(0x08000000)
        .args(["/nologo", "/target:exe"])
        .arg(format!("/out:{}", exe.display()))
        .arg(&source)
        .output()
        .unwrap();
    assert!(
        result.status.success(),
        "{}",
        String::from_utf8_lossy(&result.stdout)
    );
    let output = dir.path().join("result.txt");
    let mut environment = BTreeMap::new();
    environment.insert("SystemRoot".into(), root.to_string_lossy().into_owned());
    environment.insert("TEST_MODE".into(), mode.into());
    environment.insert("TEST_OUTPUT".into(), output.to_string_lossy().into_owned());
    let helper_pin = PinnedFile::observe(PathBuf::from(
        std::env::var_os("PYTXO_TEST_ATTEMPT_HOST")
            .expect("build the CLI companion first and set its absolute test pin"),
    ))
    .unwrap();
    if let Ok(expected) = std::env::var("PYTXO_TEST_ATTEMPT_HOST_SHA256") {
        assert_eq!(
            helper_pin.sha256,
            expected.to_ascii_lowercase(),
            "CI helper changed after its build pin was recorded"
        );
    }
    // A large embedded CLI/Desktop host can take longer than the small
    // companion to pass repeated pin checks. This test-only override does not
    // change the production total execution budget.
    let execution_timeout = std::env::var("PYTXO_TEST_OWNED_EXECUTION_TIMEOUT_SECS")
        .ok()
        .and_then(|value| value.parse::<u64>().ok())
        .filter(|seconds| (1..=120).contains(seconds))
        .map(Duration::from_secs)
        .unwrap_or(Duration::from_secs(8));
    let spec = OwnedLaunchSpec {
        bootstrap_host: helper_pin,
        executable: PinnedFile::observe(exe).unwrap(),
        dependencies: vec![],
        arguments: vec![
            "".into(),
            "two words".into(),
            "a\"b".into(),
            "trailing\\".into(),
            "\\\"quoted\\\"".into(),
            "&|<>^%$();`".into(),
            "line\nbreak".into(),
            "ไทย".into(),
        ],
        environment,
        working_directory: dir.path().to_path_buf(),
        stdin: b"private prompt\n'\"; exit 9; & injected".to_vec(),
        transport,
        barrier_timeout: Duration::from_secs(5),
        execution_timeout,
        settlement_timeout: Duration::from_secs(3),
        output_limit: 4096,
    };
    let callbacks = Callbacks {
        output,
        cancel: Arc::new(AtomicBool::new(false)),
        fail_register: false,
        fail_settle: false,
        cancel_at_register: false,
        delay: Duration::ZERO,
        registered: false,
        tamper_on_authorize: None,
        tamper_after_register: None,
        settlements: 0,
    };
    (dir, spec, callbacks)
}

#[test]
fn exact_argv_private_stdin_and_delayed_registration_both_transports() {
    for transport in [OwnedTransport::Subprocess, OwnedTransport::Pty] {
        let (_dir, spec, mut callbacks) = fixture(transport, "echo");
        callbacks.delay = Duration::from_millis(500);
        let cancel = callbacks.cancel.clone();
        let started = Instant::now();
        let receipt =
            run_owned_launch(&spec, &mut callbacks, || Ok(cancel.load(Ordering::SeqCst))).unwrap();
        assert_eq!(receipt.outcome, OwnedOutcome::Succeeded, "{receipt:?}");
        assert_eq!(receipt.active_processes, Some(0));
        assert!(receipt.barrier_released && receipt.payload_pid_reported.is_some());
        assert_eq!(receipt.payload_exit_code, Some(0));
        assert!(receipt.stdout.contains("fixture complete"), "{receipt:?}");
        assert_eq!(receipt.protocol, "pytxo-attempt-host/1");
        let lines = fs::read_to_string(&callbacks.output).unwrap();
        let mut expected = spec
            .arguments
            .iter()
            .map(|s| base64_for_test(s.as_bytes()))
            .collect::<Vec<_>>();
        expected.push(base64_for_test(&spec.stdin));
        assert_eq!(lines.lines().collect::<Vec<_>>(), expected);
        assert!(!receipt.stdout.contains("private prompt"));
        assert_eq!(receipt.observed_model, None);
        assert_eq!(receipt.observed_usage, None);
        println!(
            "{transport:?}: elapsed_ms={} receipt={receipt:?}",
            started.elapsed().as_millis()
        );
    }
}

fn base64_for_test(bytes: &[u8]) -> String {
    const B: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";
    let mut s = String::new();
    for c in bytes.chunks(3) {
        s.push(B[(c[0] >> 2) as usize] as char);
        s.push(B[(((c[0] & 3) << 4) | (c.get(1).copied().unwrap_or(0) >> 4)) as usize] as char);
        s.push(if c.len() > 1 {
            B[(((c[1] & 15) << 2) | (c.get(2).copied().unwrap_or(0) >> 6)) as usize] as char
        } else {
            '='
        });
        s.push(if c.len() > 2 {
            B[(c[2] & 63) as usize] as char
        } else {
            '='
        });
    }
    s
}

#[test]
fn registration_failure_and_cancel_before_release_never_run_payload() {
    for transport in [OwnedTransport::Subprocess, OwnedTransport::Pty] {
        for failure in [false, true] {
            let (_dir, spec, mut callbacks) = fixture(transport, "echo");
            callbacks.fail_register = failure;
            callbacks.cancel_at_register = !failure;
            let cancel = callbacks.cancel.clone();
            let r = run_owned_launch(&spec, &mut callbacks, || Ok(cancel.load(Ordering::SeqCst)))
                .unwrap();
            assert!(!callbacks.output.exists());
            assert!(!r.barrier_released);
            assert!(r.bootstrap.is_some());
            assert_eq!(r.active_processes, Some(0));
            assert_eq!(
                r.outcome,
                if failure {
                    OwnedOutcome::RecoveryRequired
                } else {
                    OwnedOutcome::Cancelled
                }
            );
        }
    }
}

#[test]
fn root_exit_does_not_hide_child_or_inherited_output_handles() {
    for transport in [OwnedTransport::Subprocess, OwnedTransport::Pty] {
        let (_dir, spec, mut callbacks) = fixture(transport, "child");
        let r = run_owned_launch(&spec, &mut callbacks, || Ok(false)).unwrap();
        assert!(r.terminated_job, "{r:?}");
        assert_eq!(r.active_processes, Some(0));
        assert_eq!(
            r.outcome,
            OwnedOutcome::Failed,
            "descendant cleanup must not manufacture success"
        );
        assert!(callbacks.output.exists());
    }
}

#[test]
fn cancellation_after_release_and_settlement_failure_preserve_evidence() {
    for transport in [OwnedTransport::Subprocess, OwnedTransport::Pty] {
        let (_dir, spec, mut callbacks) = fixture(transport, "wait");
        let marker = callbacks.output.clone();
        let r = run_owned_launch(&spec, &mut callbacks, || Ok(marker.exists())).unwrap();
        assert_eq!(r.outcome, OwnedOutcome::Cancelled, "{r:?}");
        assert!(r.barrier_released && r.terminated_job);
        assert_eq!(r.active_processes, Some(0));
        let (_dir, spec, mut callbacks) = fixture(transport, "echo");
        callbacks.fail_settle = true;
        let r = run_owned_launch(&spec, &mut callbacks, || Ok(false)).unwrap();
        assert_eq!(r.outcome, OwnedOutcome::RecoveryRequired);
        assert_eq!(r.active_processes, Some(0));
        assert!(r.error.unwrap().contains("settlement"));
    }
}

#[test]
fn expired_barrier_does_not_launch_on_late_release() {
    let (_dir, mut spec, mut callbacks) = fixture(OwnedTransport::Subprocess, "echo");
    spec.barrier_timeout = Duration::from_millis(100);
    callbacks.delay = Duration::from_secs(2);
    let r = run_owned_launch(&spec, &mut callbacks, || Ok(false)).unwrap();
    assert!(!callbacks.output.exists());
    assert_eq!(r.active_processes, Some(0));
    assert_ne!(r.outcome, OwnedOutcome::Succeeded);
}

#[test]
fn changed_pinned_file_is_rejected_before_authorization() {
    let (_dir, spec, mut callbacks) = fixture(OwnedTransport::Subprocess, "echo");
    fs::write(&spec.executable.path, b"changed").unwrap();
    assert!(run_owned_launch(&spec, &mut callbacks, || Ok(false)).is_err());
    assert!(!callbacks.registered);
    assert!(!callbacks.output.exists());
}

#[test]
fn parallel_jobs_are_isolated_and_have_distinct_recovery_names() {
    let a = std::thread::spawn(|| {
        let (_dir, spec, mut callbacks) = fixture(OwnedTransport::Pty, "wait");
        let marker = callbacks.output.clone();
        run_owned_launch(&spec, &mut callbacks, || Ok(marker.exists())).unwrap()
    });
    let b = std::thread::spawn(|| {
        let (_dir, spec, mut callbacks) = fixture(OwnedTransport::Subprocess, "echo");
        let result = run_owned_launch(&spec, &mut callbacks, || Ok(false)).unwrap();
        assert!(callbacks.output.exists());
        result
    });
    let (a, b) = (a.join().unwrap(), b.join().unwrap());
    assert_eq!(a.outcome, OwnedOutcome::Cancelled);
    assert_eq!(b.outcome, OwnedOutcome::Succeeded);
    assert_ne!(a.bootstrap.unwrap().job_name, b.bootstrap.unwrap().job_name);
    assert_eq!((a.active_processes, b.active_processes), (Some(0), Some(0)));
}

#[test]
fn child_membership_is_observed_and_termination_stops_delayed_effects() {
    for transport in [OwnedTransport::Subprocess, OwnedTransport::Pty] {
        let (_dir, spec, mut callbacks) = fixture(transport, "child-gated");
        fs::write(callbacks.output.with_extension("txt.go"), b"go").unwrap();
        let r = run_owned_launch(&spec, &mut callbacks, || Ok(false)).unwrap();
        let child: u32 = fs::read_to_string(callbacks.output.with_extension("txt.child"))
            .unwrap()
            .parse()
            .unwrap();
        assert!(
            r.observed_job_pids.contains(&child),
            "child absent from observed Job membership: {r:?}"
        );
        assert!(r.terminated_job);
        assert_eq!(r.active_processes, Some(0));
        fs::write(callbacks.output.with_extension("txt.late-go"), b"go").unwrap();
        std::thread::sleep(Duration::from_millis(1500));
        assert!(!callbacks.output.with_extension("txt.late").exists());
    }
}

#[test]
fn missing_or_stale_recovery_identity_is_unknown() {
    let current = OwnedProcess {
        pid: std::process::id(),
        start_identity: Some("windows-filetime:0".into()),
        job_name: format!("Local\\PytxoAttempt-{}", uuid::Uuid::new_v4()),
    };
    assert_eq!(observe_owned_job(&current).unwrap(), None);
    let unknown = OwnedProcess {
        start_identity: None,
        ..current
    };
    assert_eq!(observe_owned_job(&unknown).unwrap(), None);
}

// The outer process owns the test controller's handle; no arbitrary PID killing.
#[test]
fn controller_crash_before_release_and_after_payload_start_is_bounded() {
    for transport in ["subprocess", "pty"] {
        for stage in ["unassigned", "registered", "released", "settlement"] {
            let dir = tempfile::tempdir().unwrap();
            let evidence = dir.path().join("controller");
            let mut controller = Command::new(std::env::current_exe().unwrap())
                .args(["--exact", "crash_controller", "--ignored", "--nocapture"])
                .env("PYTXO_CRASH_EVIDENCE", &evidence)
                .env("PYTXO_CRASH_STAGE", stage)
                .env("PYTXO_CRASH_TRANSPORT", transport)
                .creation_flags(0x08000000)
                .stdout(std::process::Stdio::null())
                .stderr(std::process::Stdio::null())
                .spawn()
                .unwrap();
            // This child compiles a disposable fixture and hashes the pinned
            // embedded Desktop host before registering. Those setup steps can
            // exceed 20 seconds when the full Windows workspace suite is busy.
            let deadline = Instant::now() + Duration::from_secs(60);
            let marker = evidence.with_extension(match stage {
                "unassigned" | "registered" => "registered",
                "settlement" => "settlement",
                _ => "payload",
            });
            while !marker.exists() && Instant::now() < deadline {
                if controller.try_wait().unwrap().is_some() {
                    break;
                }
                std::thread::sleep(Duration::from_millis(20));
            }
            assert!(
                marker.exists(),
                "controller did not reach {stage} via {transport}"
            );
            let process: Vec<String> = fs::read_to_string(evidence.with_extension("registered"))
                .unwrap()
                .lines()
                .map(str::to_string)
                .collect();
            let process = OwnedProcess {
                pid: process[0].parse().unwrap(),
                start_identity: Some(process[1].clone()),
                job_name: process[2].clone(),
            };
            use windows_sys::Win32::{
                Foundation::{CloseHandle, WAIT_OBJECT_0},
                System::Threading::{
                    OpenProcess, WaitForSingleObject, PROCESS_QUERY_LIMITED_INFORMATION,
                    PROCESS_SYNCHRONIZE,
                },
            };
            let retained = unsafe {
                OpenProcess(
                    PROCESS_SYNCHRONIZE | PROCESS_QUERY_LIMITED_INFORMATION,
                    0,
                    process.pid,
                )
            };
            if stage != "settlement" {
                assert!(
                    !retained.is_null(),
                    "helper should still be live before controller crash"
                );
            }
            if !retained.is_null() {
                assert_eq!(
                    native_identity(retained),
                    process.start_identity.as_ref().unwrap().as_str()
                );
            }
            controller.kill().unwrap();
            controller.wait().unwrap();
            if !retained.is_null() {
                assert_eq!(
                    unsafe { WaitForSingleObject(retained, 6000) },
                    WAIT_OBJECT_0,
                    "retained helper survived controller death"
                );
                unsafe {
                    CloseHandle(retained);
                }
            }
            // Keep the observer alive beyond the fixture's delayed effect; no vanished-runtime false green.
            std::thread::sleep(Duration::from_millis(1500));
            assert!(!evidence.with_extension("payload.late").exists());
            if stage == "registered" || stage == "unassigned" {
                assert!(!evidence.with_extension("payload").exists());
            }
            assert_eq!(
                observe_owned_job(&process).unwrap(),
                None,
                "missing Job is unknown, not zero"
            );
        }
    }
}

#[test]
#[ignore = "disposable controller invoked by the crash test"]
fn crash_controller() {
    let evidence = PathBuf::from(std::env::var_os("PYTXO_CRASH_EVIDENCE").unwrap());
    let stage = std::env::var("PYTXO_CRASH_STAGE").unwrap();
    let transport = if std::env::var("PYTXO_CRASH_TRANSPORT").unwrap() == "pty" {
        OwnedTransport::Pty
    } else {
        OwnedTransport::Subprocess
    };
    if stage == "unassigned" {
        unassigned_crash_controller(&evidence, transport);
        return;
    }
    let (_dir, mut spec, _) = fixture(
        transport,
        if stage == "settlement" {
            "echo"
        } else {
            "delayed"
        },
    );
    spec.environment.insert(
        "TEST_OUTPUT".into(),
        evidence.with_extension("payload").to_string_lossy().into(),
    );
    struct Crash {
        evidence: PathBuf,
        stage: String,
    }
    impl LaunchGuard for Crash {
        fn permit_create(&mut self, _: &LaunchIntent) -> Result<()> {
            Ok(())
        }

        fn register(&mut self, p: &OwnedProcess) -> Result<()> {
            fs::write(
                self.evidence.with_extension("registered"),
                format!(
                    "{}\n{}\n{}",
                    p.pid,
                    p.start_identity.as_deref().unwrap(),
                    p.job_name
                ),
            )?;
            if self.stage == "registered" {
                std::thread::sleep(Duration::from_secs(30));
            }
            Ok(())
        }
        fn cancelled(&mut self) -> Result<bool> {
            Ok(false)
        }
    }
    impl LaunchCallbacks for Crash {
        fn authorize(&mut self, intent: &LaunchIntent) -> Result<Box<dyn LaunchGuard + '_>> {
            fs::write(self.evidence.with_extension("launching"), &intent.job_name)?;
            Ok(Box::new(Crash {
                evidence: self.evidence.clone(),
                stage: self.stage.clone(),
            }))
        }
        fn settle(&mut self, receipt: &OwnedLaunchReceipt) -> Result<()> {
            if self.stage == "settlement" {
                assert_eq!(receipt.active_processes, Some(0));
                assert_eq!(receipt.payload_exit_code, Some(0));
                fs::write(
                    self.evidence.with_extension("settlement"),
                    "callback entered, receipt not durable",
                )?;
                std::thread::sleep(Duration::from_secs(30));
            }
            panic!("controller should crash before durable settlement")
        }
    }
    let mut callbacks = Crash { evidence, stage };
    let _ = run_owned_launch(&spec, &mut callbacks, || Ok(false));
}

#[test]
fn mutation_after_authorization_settles_without_creating_a_worker() {
    let (_dir, spec, mut callbacks) = fixture(OwnedTransport::Subprocess, "echo");
    callbacks.tamper_on_authorize = Some(spec.executable.path.clone());
    let r = run_owned_launch(&spec, &mut callbacks, || Ok(false))
        .expect("authorized failures retain a receipt");
    assert_eq!(callbacks.settlements, 1);
    assert_eq!(r.active_processes, Some(0));
    assert!(!r.process_registered && r.bootstrap.is_none() && !r.barrier_released);
    assert_ne!(r.outcome, OwnedOutcome::Succeeded);
}

fn native_identity(handle: windows_sys::Win32::Foundation::HANDLE) -> String {
    use windows_sys::Win32::{Foundation::FILETIME, System::Threading::GetProcessTimes};
    let mut c = FILETIME::default();
    let mut e = FILETIME::default();
    let mut k = FILETIME::default();
    let mut u = FILETIME::default();
    assert_ne!(
        unsafe { GetProcessTimes(handle, &mut c, &mut e, &mut k, &mut u) },
        0
    );
    format!(
        "windows-filetime:{}",
        ((c.dwHighDateTime as u64) << 32) | c.dwLowDateTime as u64
    )
}

#[test]
fn unassigned_helper_expires_without_release_on_both_transports() {
    use portable_pty::{native_pty_system, Child, CommandBuilder, PtySize};
    use windows_sys::Win32::{
        Foundation::CloseHandle,
        System::Threading::{CreateEventW, GetCurrentProcess},
    };
    for transport in [OwnedTransport::Subprocess, OwnedTransport::Pty] {
        let nonce = uuid::Uuid::new_v4().to_string();
        let name: Vec<u16> = format!("Local\\PytxoGate-{nonce}")
            .encode_utf16()
            .chain(Some(0))
            .collect();
        let event = unsafe { CreateEventW(std::ptr::null(), 1, 0, name.as_ptr()) };
        assert!(!event.is_null());
        let host = PathBuf::from(std::env::var_os("PYTXO_TEST_ATTEMPT_HOST").unwrap());
        let args = vec![
            "pytxo-attempt-host/1".into(),
            nonce,
            std::process::id().to_string(),
            native_identity(unsafe { GetCurrentProcess() }),
            "200".into(),
        ];
        let started = Instant::now();
        let mut master = None;
        let mut child: Box<dyn Child + Send + Sync> = match transport {
            OwnedTransport::Subprocess => Box::new(
                Command::new(host)
                    .args(args)
                    .creation_flags(0x08000000)
                    .spawn()
                    .unwrap(),
            ),
            OwnedTransport::Pty => {
                let pair = native_pty_system().openpty(PtySize::default()).unwrap();
                let mut command = CommandBuilder::new(host);
                command.args(args);
                let child = pair.slave.spawn_command(command).unwrap();
                master = Some(pair.master);
                child
            }
        };
        let exit = loop {
            if let Some(status) = child.try_wait().unwrap() {
                break status;
            }
            if started.elapsed() > Duration::from_secs(4) {
                child.kill().unwrap();
                panic!("unassigned helper did not expire");
            }
            std::thread::sleep(Duration::from_millis(10));
        };
        assert_eq!(exit.exit_code(), 125);
        assert!(started.elapsed() >= Duration::from_millis(190));
        drop(master);
        unsafe {
            CloseHandle(event);
        }
    }
}

#[test]
fn oversized_private_frame_is_rejected_before_authorization() {
    let (_dir, mut spec, mut callbacks) = fixture(OwnedTransport::Subprocess, "echo");
    spec.environment
        .insert("PRIVATE_OVERSIZE".into(), "x".repeat(128 * 1024));
    assert!(run_owned_launch(&spec, &mut callbacks, || Ok(false)).is_err());
    assert!(!callbacks.registered && !callbacks.output.exists());
}

#[test]
fn wrong_pipe_peer_receives_no_private_launch_data() {
    use std::io::Read;
    use std::os::windows::io::FromRawHandle;
    use windows_sys::Win32::{
        Foundation::{GENERIC_READ, GENERIC_WRITE, INVALID_HANDLE_VALUE},
        Storage::FileSystem::{CreateFileW, OPEN_EXISTING},
    };
    for transport in [OwnedTransport::Subprocess, OwnedTransport::Pty] {
        let (_dir, spec, _) = fixture(transport, "echo");
        struct WrongPeer {
            connection: Option<fs::File>,
            settled: bool,
        }
        impl LaunchGuard for WrongPeer {
            fn permit_create(&mut self, _: &LaunchIntent) -> Result<()> {
                Ok(())
            }

            fn register(&mut self, _: &OwnedProcess) -> Result<()> {
                Ok(())
            }
            fn cancelled(&mut self) -> Result<bool> {
                Ok(false)
            }
        }
        impl LaunchCallbacks for WrongPeer {
            fn authorize(&mut self, intent: &LaunchIntent) -> Result<Box<dyn LaunchGuard + '_>> {
                let name: Vec<u16> = format!("\\\\.\\pipe\\PytxoAttempt-{}", intent.launch_nonce)
                    .encode_utf16()
                    .chain(Some(0))
                    .collect();
                let h = unsafe {
                    CreateFileW(
                        name.as_ptr(),
                        GENERIC_READ | GENERIC_WRITE,
                        0,
                        std::ptr::null(),
                        OPEN_EXISTING,
                        0,
                        std::ptr::null_mut(),
                    )
                };
                assert_ne!(h, INVALID_HANDLE_VALUE);
                self.connection = Some(unsafe { fs::File::from_raw_handle(h) });
                Ok(Box::new(WrongPeer {
                    connection: None,
                    settled: false,
                }))
            }
            fn settle(&mut self, r: &OwnedLaunchReceipt) -> Result<()> {
                self.settled = true;
                assert_eq!(r.active_processes, Some(0));
                Ok(())
            }
        }
        let mut callbacks = WrongPeer {
            connection: None,
            settled: false,
        };
        let receipt = run_owned_launch(&spec, &mut callbacks, || Ok(false)).unwrap();
        assert_eq!(receipt.outcome, OwnedOutcome::RecoveryRequired);
        assert!(receipt.error.as_ref().unwrap().contains("peer"));
        assert!(callbacks.settled);
        let mut leaked = vec![];
        let _ = callbacks
            .connection
            .as_mut()
            .unwrap()
            .read_to_end(&mut leaked);
        assert!(leaked.is_empty());
    }
}

fn unassigned_crash_controller(evidence: &std::path::Path, transport: OwnedTransport) {
    use portable_pty::{native_pty_system, Child, CommandBuilder, PtySize};
    use windows_sys::Win32::System::Threading::{CreateEventW, GetCurrentProcess};
    let nonce = uuid::Uuid::new_v4().to_string();
    let gate: Vec<u16> = format!("Local\\PytxoGate-{nonce}")
        .encode_utf16()
        .chain(Some(0))
        .collect();
    let event = unsafe { CreateEventW(std::ptr::null(), 1, 0, gate.as_ptr()) };
    assert!(!event.is_null());
    let host = PathBuf::from(std::env::var_os("PYTXO_TEST_ATTEMPT_HOST").unwrap());
    let args = vec![
        "pytxo-attempt-host/1".into(),
        nonce.clone(),
        std::process::id().to_string(),
        native_identity(unsafe { GetCurrentProcess() }),
        "5000".into(),
    ];
    let mut master = None;
    let child: Box<dyn Child + Send + Sync> = match transport {
        OwnedTransport::Subprocess => Box::new(
            Command::new(host)
                .args(args)
                .creation_flags(0x08000000)
                .spawn()
                .unwrap(),
        ),
        OwnedTransport::Pty => {
            let pair = native_pty_system().openpty(PtySize::default()).unwrap();
            let mut command = CommandBuilder::new(host);
            command.args(args);
            let child = pair.slave.spawn_command(command).unwrap();
            master = Some(pair.master);
            child
        }
    };
    // Intentionally no Job assignment and no release, reproducing the creation window.
    fs::write(
        evidence.with_extension("registered"),
        format!(
            "{}\n{}\nLocal\\PytxoAttempt-{}",
            child.process_id().unwrap(),
            native_identity(child.as_raw_handle().unwrap().cast()),
            nonce
        ),
    )
    .unwrap();
    std::thread::sleep(Duration::from_secs(30));
    drop(master);
    drop(child);
    panic!("outer test must terminate this exact controller handle");
}

#[test]
fn recovery_checks_missing_nonmember_and_inaccessible_jobs_with_a_valid_live_identity() {
    use windows_sys::Win32::{
        Foundation::{CloseHandle, LocalFree},
        Security::{
            Authorization::ConvertStringSecurityDescriptorToSecurityDescriptorW,
            SECURITY_ATTRIBUTES,
        },
        System::{JobObjects::CreateJobObjectW, Threading::GetCurrentProcess},
    };
    struct TestJob(windows_sys::Win32::Foundation::HANDLE);
    impl Drop for TestJob {
        fn drop(&mut self) {
            unsafe {
                CloseHandle(self.0);
            }
        }
    }
    let process = OwnedProcess {
        pid: std::process::id(),
        start_identity: Some(native_identity(unsafe { GetCurrentProcess() })),
        job_name: format!("Local\\PytxoAttempt-{}", uuid::Uuid::new_v4()),
    };
    assert_eq!(
        observe_owned_job(&process).unwrap(),
        None,
        "live identity reaches missing Job lookup"
    );
    let name: Vec<u16> = process.job_name.encode_utf16().chain(Some(0)).collect();
    let empty = TestJob(unsafe { CreateJobObjectW(std::ptr::null(), name.as_ptr()) });
    assert!(!empty.0.is_null());
    assert_eq!(
        observe_owned_job(&process).unwrap(),
        None,
        "empty unrelated Job is not this process's quiescence"
    );
    drop(empty);

    // Only a new disposable kernel object's DACL is restricted. No host/user
    // settings, existing object permissions or system security policy are changed.
    let deny: Vec<u16> = "D:P(D;;GA;;;WD)".encode_utf16().chain(Some(0)).collect();
    let mut descriptor = std::ptr::null_mut();
    assert_ne!(
        unsafe {
            ConvertStringSecurityDescriptorToSecurityDescriptorW(
                deny.as_ptr(),
                1,
                &mut descriptor,
                std::ptr::null_mut(),
            )
        },
        0
    );
    let attrs = SECURITY_ATTRIBUTES {
        nLength: std::mem::size_of::<SECURITY_ATTRIBUTES>() as u32,
        lpSecurityDescriptor: descriptor,
        bInheritHandle: 0,
    };
    let denied = TestJob(unsafe { CreateJobObjectW(&attrs, name.as_ptr()) });
    unsafe {
        LocalFree(descriptor);
    }
    assert!(
        !denied.0.is_null(),
        "could not create the isolated access-denied fixture"
    );
    assert!(
        observe_owned_job(&process).is_err(),
        "access denied cannot become zero or success"
    );
}

#[test]
fn incomplete_helper_completion_retains_recovery_receipt() {
    for transport in [OwnedTransport::Subprocess, OwnedTransport::Pty] {
        let (_dir, spec, mut callbacks) = fixture(transport, "echo");
        callbacks.tamper_after_register = Some(spec.executable.path.clone());
        let r = run_owned_launch(&spec, &mut callbacks, || Ok(false)).unwrap();
        assert_eq!(r.outcome, OwnedOutcome::RecoveryRequired, "{r:?}");
        assert!(r.process_registered && r.barrier_released);
        assert!(r.payload_pid_reported.is_none() && r.payload_exit_code.is_none());
        assert_eq!(r.active_processes, Some(0));
        assert_eq!(callbacks.settlements, 1);
        assert!(!callbacks.output.exists());
    }
}

struct RecoveryTestJob(windows_sys::Win32::Foundation::HANDLE);
impl RecoveryTestJob {
    fn new(name: &str) -> Self {
        use windows_sys::Win32::System::JobObjects::*;
        let wide: Vec<u16> = name.encode_utf16().chain(Some(0)).collect();
        let handle = unsafe { CreateJobObjectW(std::ptr::null(), wide.as_ptr()) };
        assert!(!handle.is_null(), "create isolated recovery Job");
        let mut limits = JOBOBJECT_EXTENDED_LIMIT_INFORMATION::default();
        limits.BasicLimitInformation.LimitFlags = JOB_OBJECT_LIMIT_KILL_ON_JOB_CLOSE;
        assert_ne!(
            unsafe {
                SetInformationJobObject(
                    handle,
                    JobObjectExtendedLimitInformation,
                    (&limits as *const JOBOBJECT_EXTENDED_LIMIT_INFORMATION).cast(),
                    std::mem::size_of_val(&limits) as u32,
                )
            },
            0
        );
        Self(handle)
    }
    fn assign(&self, child: &std::process::Child) {
        use std::os::windows::io::AsRawHandle;
        use windows_sys::Win32::System::JobObjects::AssignProcessToJobObject;
        assert_ne!(
            unsafe { AssignProcessToJobObject(self.0, child.as_raw_handle().cast()) },
            0,
            "assign isolated fixture to Job"
        );
    }
    fn active(&self) -> u32 {
        use windows_sys::Win32::System::JobObjects::*;
        let mut info = JOBOBJECT_BASIC_ACCOUNTING_INFORMATION::default();
        assert_ne!(
            unsafe {
                QueryInformationJobObject(
                    self.0,
                    JobObjectBasicAccountingInformation,
                    (&mut info as *mut JOBOBJECT_BASIC_ACCOUNTING_INFORMATION).cast(),
                    std::mem::size_of_val(&info) as u32,
                    std::ptr::null_mut(),
                )
            },
            0
        );
        info.ActiveProcesses
    }
}
impl Drop for RecoveryTestJob {
    fn drop(&mut self) {
        use windows_sys::Win32::{Foundation::CloseHandle, System::JobObjects::TerminateJobObject};
        unsafe {
            TerminateJobObject(self.0, 125);
            CloseHandle(self.0);
        }
    }
}

fn recovery_fixture(
    mode: &str,
) -> (
    tempfile::TempDir,
    std::process::Child,
    RecoveryTestJob,
    OwnedProcess,
    String,
    PathBuf,
) {
    use std::os::windows::io::AsRawHandle;
    let (dir, spec, callbacks) = fixture(OwnedTransport::Subprocess, mode);
    let nonce = uuid::Uuid::new_v4().to_string();
    let name = format!("Local\\PytxoAttempt-{nonce}");
    let job = RecoveryTestJob::new(&name);
    let child = Command::new(&spec.executable.path)
        .envs(&spec.environment)
        .creation_flags(0x08000000)
        .spawn()
        .unwrap();
    let process = OwnedProcess {
        pid: child.id(),
        start_identity: Some(native_identity(child.as_raw_handle().cast())),
        job_name: name,
    };
    job.assign(&child);
    (dir, child, job, process, nonce, callbacks.output)
}

fn stop_in_second_process(
    process: &OwnedProcess,
    nonce: &str,
    evidence: &std::path::Path,
) -> String {
    fs::write(
        evidence.with_extension("owner"),
        format!(
            "{}\n{}\n{}\n{}\n",
            process.pid,
            process.start_identity.as_deref().unwrap(),
            process.job_name,
            nonce,
        ),
    )
    .unwrap();
    let output = Command::new(std::env::current_exe().unwrap())
        .args([
            "--exact",
            "stop_owned_job_controller",
            "--ignored",
            "--nocapture",
        ])
        .env("PYTXO_STOP_EVIDENCE", evidence)
        .creation_flags(0x08000000)
        .output()
        .unwrap();
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stdout)
    );
    fs::read_to_string(evidence.with_extension("stop")).unwrap()
}

#[test]
#[ignore = "disposable second-process native Job controller"]
fn stop_owned_job_controller() {
    let evidence = PathBuf::from(std::env::var_os("PYTXO_STOP_EVIDENCE").unwrap());
    let lines = fs::read_to_string(evidence.with_extension("owner")).unwrap();
    let mut lines = lines.lines();
    let process = OwnedProcess {
        pid: lines.next().unwrap().parse().unwrap(),
        start_identity: Some(lines.next().unwrap().to_string()),
        job_name: lines.next().unwrap().to_string(),
    };
    let nonce = lines.next().unwrap();
    let result = terminate_owned_job(&process, nonce, Duration::from_secs(3)).unwrap();
    fs::write(evidence.with_extension("stop"), format!("{result:?}")).unwrap();
}

#[test]
fn exact_live_job_stop_from_second_process_yields_positive_zero_once() {
    let (_dir, mut child, job, process, nonce, output) = recovery_fixture("gated-late");
    let deadline = Instant::now() + Duration::from_secs(5);
    while !output.with_extension("txt.ready").exists() && Instant::now() < deadline {
        std::thread::sleep(Duration::from_millis(10));
    }
    assert!(output.with_extension("txt.ready").exists());
    assert!(!output.with_extension("txt.late").exists());
    let first = stop_in_second_process(&process, &nonce, &output);
    assert_eq!(first, "VerifiedZero");
    assert_eq!(job.active(), 0);
    child.wait().unwrap();
    assert_eq!(stop_in_second_process(&process, &nonce, &output), "Unknown");
    fs::write(output.with_extension("txt.go"), "go").unwrap();
    std::thread::sleep(Duration::from_millis(1200));
    assert!(!output.with_extension("txt.late").exists());
}

#[test]
fn exited_root_with_live_descendant_is_stopped_but_not_release_proof() {
    let (_dir, mut child, job, process, nonce, output) = recovery_fixture("child-gated");
    fs::write(output.with_extension("txt.go"), "go").unwrap();
    let deadline = Instant::now() + Duration::from_secs(5);
    while !output.with_extension("txt.child").exists() && Instant::now() < deadline {
        std::thread::sleep(Duration::from_millis(10));
    }
    assert!(output.with_extension("txt.child").exists());
    child.wait().unwrap();
    assert!(
        job.active() > 0,
        "fixture descendant must survive root exit"
    );
    assert_eq!(
        stop_in_second_process(&process, &nonce, &output),
        "NamedJobTerminatedUnverified"
    );
    assert_eq!(job.active(), 0);
    assert_eq!(stop_in_second_process(&process, &nonce, &output), "Unknown");
    fs::write(output.with_extension("txt.late-go"), "go").unwrap();
    std::thread::sleep(Duration::from_millis(1200));
    assert!(!output.with_extension("txt.late").exists());
}

#[test]
fn wrong_nonce_stale_identity_and_missing_job_never_stop_or_prove_zero() {
    let (_dir, mut child, job, process, nonce, _output) = recovery_fixture("delayed");
    assert!(terminate_owned_job(
        &process,
        &uuid::Uuid::new_v4().to_string(),
        Duration::from_secs(1)
    )
    .is_err());
    assert!(child.try_wait().unwrap().is_none());
    let stale = OwnedProcess {
        start_identity: Some("windows-filetime:0".into()),
        ..process.clone()
    };
    assert_eq!(
        terminate_owned_job(&stale, &nonce, Duration::from_secs(1)).unwrap(),
        OwnedJobStopResult::Unknown
    );
    assert!(job.active() > 0);
    let missing_nonce = uuid::Uuid::new_v4().to_string();
    let missing = OwnedProcess {
        job_name: format!("Local\\PytxoAttempt-{missing_nonce}"),
        ..process
    };
    assert_eq!(
        terminate_owned_job(&missing, &missing_nonce, Duration::from_secs(1)).unwrap(),
        OwnedJobStopResult::Unknown
    );
    assert!(job.active() > 0);
}
