use super::*;
use portable_pty::{native_pty_system, Child, CommandBuilder, MasterPty, PtySize};
use std::collections::HashSet;
use std::io::{Read, Write};
use std::os::windows::ffi::OsStrExt;
use std::os::windows::io::FromRawHandle;
use std::os::windows::process::CommandExt;
use std::process::{Command, Stdio};
use std::sync::mpsc::{self, Receiver};
use std::{ptr, thread, time::Instant};
use windows_sys::Win32::Foundation::*;
use windows_sys::Win32::Security::{Authorization::*, *};
use windows_sys::Win32::Storage::FileSystem::*;
use windows_sys::Win32::System::SystemServices::JOB_OBJECT_QUERY;
use windows_sys::Win32::System::{JobObjects::*, Pipes::*, Threading::*};

const PROTOCOL: &str = "pytxo-attempt-host/1";
const LOWERING: &str = "rust-std-command-windows-structured-argv/v1";
const DIRECT_PROTOCOL: &str = "pytxo-direct-suspended/1";
const DIRECT_LOWERING: &str = "windows-createprocess-structured-argv/v1";
const DIRECT_CMD_TAIL_LOWERING: &str = "windows-createprocess-cmd-verbatim-tail/v1";
const FRAME_LIMIT: usize = 128 * 1024;
const MAX_JOB_MEMBER_SLOTS: usize = 65_536;
const JOB_MEMBER_HEADER_WORDS: usize =
    std::mem::offset_of!(JOBOBJECT_BASIC_PROCESS_ID_LIST, ProcessIdList)
        / std::mem::size_of::<usize>();
// JOB_OBJECT_TERMINATE access right (not exported by windows-sys 0.61).
const JOB_OBJECT_TERMINATE_ACCESS: u32 = 0x0008;

#[derive(serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
struct LaunchFrame {
    version: u32,
    pty: bool,
    executable: PinnedFile,
    dependencies: Vec<PinnedFile>,
    arguments: Vec<String>,
    environment: BTreeMap<String, String>,
    working_directory: PathBuf,
    stdin: Vec<u8>,
}
impl LaunchFrame {
    fn encode(spec: &OwnedLaunchSpec) -> Result<Vec<u8>> {
        let bytes = serde_json::to_vec(&Self {
            version: 1,
            pty: spec.transport == OwnedTransport::Pty,
            executable: spec.executable.clone(),
            dependencies: spec.dependencies.clone(),
            arguments: spec.arguments.clone(),
            environment: spec.environment.clone(),
            working_directory: spec.working_directory.clone(),
            stdin: spec.stdin.clone(),
        })
        .map_err(|_| failure("cannot encode private launch frame"))?;
        if bytes.len() > FRAME_LIMIT {
            return Err(failure("private launch frame exceeds limit"));
        }
        let mut frame = (bytes.len() as u32).to_le_bytes().to_vec();
        frame.extend(bytes);
        Ok(frame)
    }
}

fn failure(message: impl Into<String>) -> PytxoError {
    PytxoError::Runner(message.into())
}
fn os_error(action: &str) -> PytxoError {
    failure(format!("{action}: {}", std::io::Error::last_os_error()))
}
fn wide(value: &str) -> Vec<u16> {
    value.encode_utf16().chain(Some(0)).collect()
}

struct Handle(HANDLE);
impl Drop for Handle {
    fn drop(&mut self) {
        unsafe {
            CloseHandle(self.0);
        }
    }
}
impl Handle {
    fn new(raw: HANDLE, action: &str) -> Result<Self> {
        if raw.is_null() || raw == INVALID_HANDLE_VALUE {
            Err(os_error(action))
        } else {
            Ok(Self(raw))
        }
    }
}
struct Job(Handle);
impl Job {
    fn create(name: &str) -> Result<Self> {
        let h = unsafe { CreateJobObjectW(ptr::null(), wide(name).as_ptr()) };
        let existing = unsafe { GetLastError() } == ERROR_ALREADY_EXISTS;
        let handle = Handle::new(h, "create owned job")?;
        if existing {
            return Err(failure("refusing existing owned job"));
        }
        let mut limits = JOBOBJECT_EXTENDED_LIMIT_INFORMATION::default();
        limits.BasicLimitInformation.LimitFlags = JOB_OBJECT_LIMIT_KILL_ON_JOB_CLOSE;
        if unsafe {
            SetInformationJobObject(
                handle.0,
                JobObjectExtendedLimitInformation,
                (&limits as *const JOBOBJECT_EXTENDED_LIMIT_INFORMATION).cast(),
                std::mem::size_of_val(&limits) as u32,
            )
        } == 0
        {
            return Err(os_error("configure owned job"));
        }
        Ok(Self(handle))
    }
    fn assign(&self, process: HANDLE) -> Result<()> {
        if unsafe { AssignProcessToJobObject(self.0 .0, process) } == 0 {
            return Err(os_error("assign bootstrap to job"));
        }
        let mut member = 0;
        if unsafe { IsProcessInJob(process, self.0 .0, &mut member) } == 0 || member == 0 {
            return Err(os_error("verify job membership"));
        }
        Ok(())
    }
    fn active(&self) -> Result<u32> {
        let mut info = JOBOBJECT_BASIC_ACCOUNTING_INFORMATION::default();
        if unsafe {
            QueryInformationJobObject(
                self.0 .0,
                JobObjectBasicAccountingInformation,
                (&mut info as *mut JOBOBJECT_BASIC_ACCOUNTING_INFORMATION).cast(),
                std::mem::size_of_val(&info) as u32,
                ptr::null_mut(),
            )
        } == 0
        {
            return Err(os_error("query owned job"));
        }
        Ok(info.ActiveProcesses)
    }
    fn members(&self) -> Result<Vec<u32>> {
        let mut capacity = 4096;
        loop {
            let mut storage = vec![0usize; capacity + JOB_MEMBER_HEADER_WORDS];
            if unsafe {
                QueryInformationJobObject(
                    self.0 .0,
                    JobObjectBasicProcessIdList,
                    storage.as_mut_ptr().cast(),
                    (storage.len() * std::mem::size_of::<usize>()) as u32,
                    ptr::null_mut(),
                )
            } == 0
            {
                if unsafe { GetLastError() } == ERROR_MORE_DATA && capacity < MAX_JOB_MEMBER_SLOTS {
                    capacity = (capacity * 2).min(MAX_JOB_MEMBER_SLOTS);
                    continue;
                }
                return Err(os_error("query owned job members"));
            }
            let list = unsafe { &*storage.as_ptr().cast::<JOBOBJECT_BASIC_PROCESS_ID_LIST>() };
            let ids = unsafe { std::slice::from_raw_parts(list.ProcessIdList.as_ptr(), capacity) };
            let (pids, needs_larger_buffer) = member_snapshot(
                ids,
                list.NumberOfAssignedProcesses as usize,
                list.NumberOfProcessIdsInList as usize,
            )?;
            if needs_larger_buffer && capacity < MAX_JOB_MEMBER_SLOTS {
                capacity = (list.NumberOfAssignedProcesses as usize)
                    .max(capacity * 2)
                    .min(MAX_JOB_MEMBER_SLOTS);
                continue;
            }
            // A process can enter or leave the Job during this telemetry read.
            // Valid returned IDs are useful; an incomplete list is not a failed
            // containment check. IsProcessInJob and wait_zero enforce ownership.
            return Ok(pids);
        }
    }
    fn terminate(&self) -> Result<()> {
        if unsafe { TerminateJobObject(self.0 .0, 125) } == 0 {
            Err(os_error("terminate owned job"))
        } else {
            Ok(())
        }
    }
    fn wait_zero(&self, timeout: Duration) -> Result<()> {
        let until = Instant::now() + timeout;
        loop {
            if self.active()? == 0 {
                return Ok(());
            }
            if Instant::now() >= until {
                return Err(failure(
                    "owned job did not reach zero before settlement deadline",
                ));
            }
            thread::sleep(Duration::from_millis(10));
        }
    }
}
fn member_snapshot(ids: &[usize], assigned: usize, listed: usize) -> Result<(Vec<u32>, bool)> {
    let capacity = ids.len();
    if listed > capacity {
        return Err(failure("owned membership snapshot exceeds buffer"));
    }
    Ok((
        ids[..listed].iter().map(|pid| *pid as u32).collect(),
        assigned > capacity,
    ))
}
fn record_job_members(receipt: &mut OwnedLaunchReceipt, seen: &mut HashSet<u32>, pids: Vec<u32>) {
    for pid in pids {
        if seen.insert(pid) {
            receipt.observed_job_pids.push(pid);
        }
    }
}
fn process_identity(handle: HANDLE) -> Result<String> {
    let mut creation = FILETIME::default();
    let mut exit = FILETIME::default();
    let mut kernel = FILETIME::default();
    let mut user = FILETIME::default();
    if unsafe { GetProcessTimes(handle, &mut creation, &mut exit, &mut kernel, &mut user) } == 0 {
        return Err(os_error("query bootstrap start identity"));
    }
    Ok(format!(
        "windows-filetime:{}",
        ((creation.dwHighDateTime as u64) << 32) | creation.dwLowDateTime as u64
    ))
}
pub(super) fn observe(process: &OwnedProcess) -> Result<Option<u32>> {
    let Some(expected) = &process.start_identity else {
        return Ok(None);
    };
    let raw = unsafe {
        OpenProcess(
            PROCESS_QUERY_LIMITED_INFORMATION | PROCESS_SYNCHRONIZE,
            0,
            process.pid,
        )
    };
    if raw.is_null() {
        return if unsafe { GetLastError() } == ERROR_INVALID_PARAMETER {
            Ok(None)
        } else {
            Err(os_error("open recovery process"))
        };
    }
    let handle = Handle::new(raw, "recovery process")?;
    if process_identity(handle.0)? != *expected || !alive(handle.0)? {
        return Ok(None);
    }
    let raw = unsafe { OpenJobObjectW(JOB_OBJECT_QUERY, 0, wide(&process.job_name).as_ptr()) };
    if raw.is_null() {
        return if unsafe { GetLastError() } == ERROR_FILE_NOT_FOUND {
            Ok(None)
        } else {
            Err(os_error("open recovery job"))
        };
    }
    let job = Job(Handle::new(raw, "recovery job")?);
    let mut member = 0;
    if unsafe { IsProcessInJob(handle.0, job.0 .0, &mut member) } == 0 {
        return Err(os_error("verify recovery membership"));
    }
    if member == 0 {
        return Ok(None);
    }
    Ok(Some(job.active()?))
}

pub(super) fn terminate_registered(
    process: &OwnedProcess,
    timeout: Duration,
) -> Result<OwnedJobStopResult> {
    // The caller has already checked the canonical UUIDv4 name against the
    // separately retained launch nonce. Missing identity cannot target a Job.
    let Some(expected) = &process.start_identity else {
        return Ok(OwnedJobStopResult::Unknown);
    };
    if process.pid == 0
        || expected
            .strip_prefix("windows-filetime:")
            .and_then(|value| value.parse::<u64>().ok())
            .is_none_or(|value| value == 0)
    {
        return Ok(OwnedJobStopResult::Unknown);
    }
    let raw = unsafe {
        OpenJobObjectW(
            JOB_OBJECT_QUERY | JOB_OBJECT_TERMINATE_ACCESS,
            0,
            wide(&process.job_name).as_ptr(),
        )
    };
    if raw.is_null() {
        return if unsafe { GetLastError() } == ERROR_FILE_NOT_FOUND {
            Ok(OwnedJobStopResult::Unknown)
        } else {
            Err(os_error("open registered owned job for stop"))
        };
    }
    // This handle pins the selected kernel object through termination and the
    // positive zero query, even if another controller drops its handle.
    let job = Job(Handle::new(raw, "registered owned job")?);
    let raw = unsafe {
        OpenProcess(
            PROCESS_QUERY_LIMITED_INFORMATION | PROCESS_SYNCHRONIZE,
            0,
            process.pid,
        )
    };
    let root_live_member = if raw.is_null() {
        if unsafe { GetLastError() } != ERROR_INVALID_PARAMETER {
            return Err(os_error("open registered bootstrap for stop"));
        }
        false
    } else {
        let root = Handle::new(raw, "registered bootstrap")?;
        if process_identity(root.0)? != *expected {
            // A reused PID is not evidence for stopping a named Job.
            return Ok(OwnedJobStopResult::Unknown);
        }
        if !alive(root.0)? {
            false
        } else {
            let mut member = 0;
            if unsafe { IsProcessInJob(root.0, job.0 .0, &mut member) } == 0 {
                return Err(os_error("verify registered Job membership for stop"));
            }
            if member == 0 {
                // An unrelated live process plus a matching name is not enough.
                return Ok(OwnedJobStopResult::Unknown);
            }
            // The retained handle and creation identity prevent PID reuse.
            alive(root.0)?
        }
    };
    if job.active()? == 0 {
        return Ok(OwnedJobStopResult::Unknown);
    }
    job.terminate()?;
    job.wait_zero(timeout)?;
    if root_live_member {
        Ok(OwnedJobStopResult::VerifiedZero)
    } else {
        // Exact-name cleanup of surviving descendants is useful, but a dead
        // root no longer proves that this Job contains the registered process.
        Ok(OwnedJobStopResult::NamedJobTerminatedUnverified)
    }
}
fn alive(handle: HANDLE) -> Result<bool> {
    // A retained handle avoids PID reuse and distinguishes error from absence.
    match unsafe { WaitForSingleObject(handle, 0) } {
        WAIT_TIMEOUT => Ok(true),
        WAIT_OBJECT_0 => Ok(false),
        _ => Err(os_error("query bootstrap liveness")),
    }
}
pub(super) fn system_root() -> Result<PathBuf> {
    let mut buffer = vec![0u16; 32768];
    let n = unsafe {
        windows_sys::Win32::System::SystemInformation::GetWindowsDirectoryW(
            buffer.as_mut_ptr(),
            buffer.len() as u32,
        )
    } as usize;
    if n == 0 || n >= buffer.len() {
        return Err(os_error("resolve Windows directory"));
    }
    Ok(PathBuf::from(
        String::from_utf16(&buffer[..n]).map_err(|_| failure("invalid Windows path"))?,
    ))
}
fn validate(spec: &OwnedLaunchSpec) -> Result<Vec<PinnedFile>> {
    resolve_attempt_host(Some(&spec.bootstrap_host))?;
    if spec
        .bootstrap_host
        .path
        .extension()
        .is_none_or(|x| !x.eq_ignore_ascii_case("exe"))
    {
        return Err(failure("requires a qualified native attempt host"));
    }
    if !spec.working_directory.is_absolute()
        || !spec.working_directory.is_dir()
        || spec
            .executable
            .path
            .extension()
            .is_none_or(|x| !x.eq_ignore_ascii_case("exe"))
    {
        return Err(failure(
            "requires an absolute working directory and resolved native .exe payload",
        ));
    }
    if spec.stdin.len() > 16384
        || spec.output_limit == 0
        || spec.output_limit > 16 * 1024 * 1024
        || spec.barrier_timeout < Duration::from_millis(1)
        || spec.barrier_timeout > Duration::from_secs(30)
        || spec.execution_timeout.is_zero()
        || spec.execution_timeout > Duration::from_secs(86400)
        || spec.settlement_timeout.is_zero()
        || spec.settlement_timeout > Duration::from_secs(60)
    {
        return Err(failure("owned launch bounds are invalid"));
    }
    let mut seen = std::collections::BTreeSet::new();
    for (key, value) in &spec.environment {
        if key.is_empty()
            || key.contains(['=', '\0'])
            || value.contains('\0')
            || key.to_ascii_uppercase().starts_with("PYTXO_")
            || !seen.insert(key.to_ascii_uppercase())
        {
            return Err(failure("invalid or reserved payload environment key"));
        }
    }
    validate_payload_system_root(&spec.environment)?;
    if spec.arguments.iter().any(|arg| arg.contains('\0'))
        || spec
            .arguments
            .iter()
            .map(|a| a.encode_utf16().count() * 2 + 3)
            .sum::<usize>()
            > 24000
    {
        return Err(failure("invalid or oversized structured arguments"));
    }
    LaunchFrame::encode(spec)?;
    let mut chain = vec![];
    for pin in std::iter::once(&spec.bootstrap_host)
        .chain(std::iter::once(&spec.executable))
        .chain(&spec.dependencies)
    {
        let actual = PinnedFile::observe(pin.path.clone())?;
        if &actual != pin {
            return Err(failure(format!(
                "launch identity changed: {}",
                pin.path.display()
            )));
        }
        chain.push(actual);
    }
    Ok(chain)
}
fn environment() -> Result<BTreeMap<String, String>> {
    let root = system_root()?.to_string_lossy().into_owned();
    Ok(BTreeMap::from([
        ("SystemRoot".into(), root.clone()),
        ("WINDIR".into(), root),
    ]))
}

fn validate_payload_system_root(environment: &BTreeMap<String, String>) -> Result<()> {
    let observed = system_root()?.to_string_lossy().into_owned();
    for (key, value) in environment {
        if (key.eq_ignore_ascii_case("SystemRoot") || key.eq_ignore_ascii_case("WINDIR"))
            && !value.eq_ignore_ascii_case(&observed)
        {
            return Err(failure(
                "payload Windows root differs from the operating system",
            ));
        }
    }
    Ok(())
}
type Capture = Receiver<(Vec<u8>, bool, bool)>;
fn decode_capture(bytes: &[u8]) -> (String, bool) {
    match std::str::from_utf8(bytes) {
        Ok(text) => (text.to_owned(), true),
        Err(_) => (String::from_utf8_lossy(bytes).into_owned(), false),
    }
}
fn capture(mut reader: impl Read + Send + 'static, limit: usize) -> Capture {
    let (tx, rx) = mpsc::channel();
    thread::spawn(move || {
        let mut output = Vec::new();
        let mut truncated = false;
        let mut complete = true;
        let mut block = [0u8; 4096];
        loop {
            match reader.read(&mut block) {
                Ok(0) => break,
                Ok(n) => {
                    let take = n.min(limit.saturating_sub(output.len()));
                    output.extend_from_slice(&block[..take]);
                    truncated |= take < n;
                }
                Err(_) => {
                    complete = false;
                    break;
                }
            }
        }
        let _ = tx.send((output, truncated, complete));
    });
    rx
}
struct Running {
    child: Box<dyn Child + Send + Sync>,
    master: Option<Box<dyn MasterPty + Send>>,
    stdout: Capture,
    stderr: Option<Capture>,
}
impl Running {
    fn handle(&self) -> Result<HANDLE> {
        self.child
            .as_raw_handle()
            .map(|h| h as HANDLE)
            .ok_or_else(|| failure("bootstrap process handle unavailable"))
    }
}
fn spawn(
    spec: &OwnedLaunchSpec,
    env: &BTreeMap<String, String>,
    intent: &LaunchIntent,
) -> Result<Running> {
    let args = vec![
        PROTOCOL.to_string(),
        intent.launch_nonce.clone(),
        std::process::id().to_string(),
        process_identity(unsafe { GetCurrentProcess() })?,
        spec.barrier_timeout.as_millis().to_string(),
    ];
    let cwd = system_root()?.join("System32");
    match spec.transport {
        OwnedTransport::Subprocess => {
            let mut child = Command::new(&spec.bootstrap_host.path)
                .args(&args)
                .env_clear()
                .envs(env)
                .current_dir(cwd)
                .creation_flags(CREATE_NO_WINDOW)
                .stdin(Stdio::null())
                .stdout(Stdio::piped())
                .stderr(Stdio::piped())
                .spawn()?;
            let stdout = capture(
                child.stdout.take().expect("piped stdout"),
                spec.output_limit,
            );
            let stderr = Some(capture(
                child.stderr.take().expect("piped stderr"),
                spec.output_limit,
            ));
            Ok(Running {
                child: Box::new(child),
                master: None,
                stdout,
                stderr,
            })
        }
        OwnedTransport::Pty => {
            let pair = native_pty_system()
                .openpty(PtySize {
                    rows: 24,
                    cols: 120,
                    pixel_width: 0,
                    pixel_height: 0,
                })
                .map_err(|e| failure(format!("open owned PTY: {e}")))?;
            // Acquire fallible I/O resources before spawning the inert bootstrap.
            let reader = pair
                .master
                .try_clone_reader()
                .map_err(|e| failure(format!("owned PTY reader: {e}")))?;
            let mut command = CommandBuilder::new(&spec.bootstrap_host.path);
            command.args(&args);
            command.cwd(cwd);
            command.env_clear();
            for (k, v) in env {
                command.env(k, v);
            }
            let child = pair
                .slave
                .spawn_command(command)
                .map_err(|e| failure(format!("spawn owned PTY: {e}")))?;
            drop(pair.slave);
            Ok(Running {
                child,
                master: Some(pair.master),
                stdout: capture(reader, spec.output_limit),
                stderr: None,
            })
        }
    }
}
// Explicit current-user DACL; no inherited default DACL or remote clients.
struct PrivateSecurity(PSECURITY_DESCRIPTOR);
impl Drop for PrivateSecurity {
    fn drop(&mut self) {
        unsafe {
            LocalFree(self.0);
        }
    }
}
impl PrivateSecurity {
    fn new() -> Result<Self> {
        let mut token = ptr::null_mut();
        if unsafe { OpenProcessToken(GetCurrentProcess(), TOKEN_QUERY, &mut token) } == 0 {
            return Err(os_error("open owner token"));
        }
        let token = Handle::new(token, "owner token")?;
        let mut len = 0;
        unsafe {
            GetTokenInformation(token.0, TokenUser, ptr::null_mut(), 0, &mut len);
        }
        if len == 0 || len > 65536 {
            return Err(failure("invalid owner token length"));
        }
        let mut storage = vec![0usize; (len as usize).div_ceil(std::mem::size_of::<usize>())];
        if unsafe {
            GetTokenInformation(
                token.0,
                TokenUser,
                storage.as_mut_ptr().cast(),
                len,
                &mut len,
            )
        } == 0
        {
            return Err(os_error("read owner token"));
        }
        let user = unsafe { &*storage.as_ptr().cast::<TOKEN_USER>() };
        let mut sid = ptr::null_mut();
        if unsafe { ConvertSidToStringSidW(user.User.Sid, &mut sid) } == 0 {
            return Err(os_error("encode owner SID"));
        }
        let mut length = 0;
        unsafe {
            while *sid.add(length) != 0 {
                length += 1;
            }
        }
        let text = unsafe { String::from_utf16_lossy(std::slice::from_raw_parts(sid, length)) };
        unsafe {
            LocalFree(sid.cast());
        }
        let mut descriptor = ptr::null_mut();
        if unsafe {
            ConvertStringSecurityDescriptorToSecurityDescriptorW(
                wide(&format!("D:P(A;;GA;;;{text})")).as_ptr(),
                1,
                &mut descriptor,
                ptr::null_mut(),
            )
        } == 0
        {
            return Err(os_error("create owner DACL"));
        }
        Ok(Self(descriptor))
    }
    fn attributes(&self) -> SECURITY_ATTRIBUTES {
        SECURITY_ATTRIBUTES {
            nLength: std::mem::size_of::<SECURITY_ATTRIBUTES>() as u32,
            lpSecurityDescriptor: self.0,
            bInheritHandle: 0,
        }
    }
}
struct PromptPipe {
    handle: Handle,
    frame: Vec<u8>,
    written: usize,
    connected: bool,
    ack: Vec<u8>,
}
impl PromptPipe {
    fn create(nonce: &str, spec: &OwnedLaunchSpec, security: &PrivateSecurity) -> Result<Self> {
        let attrs = security.attributes();
        let handle = Handle::new(
            unsafe {
                CreateNamedPipeW(
                    wide(&format!("\\\\.\\pipe\\PytxoAttempt-{nonce}")).as_ptr(),
                    PIPE_ACCESS_DUPLEX | FILE_FLAG_FIRST_PIPE_INSTANCE,
                    PIPE_TYPE_BYTE | PIPE_READMODE_BYTE | PIPE_NOWAIT | PIPE_REJECT_REMOTE_CLIENTS,
                    1,
                    FRAME_LIMIT as u32 + 4,
                    32,
                    0,
                    &attrs,
                )
            },
            "create private launch pipe",
        )?;
        Ok(Self {
            handle,
            frame: LaunchFrame::encode(spec)?,
            written: 0,
            connected: false,
            ack: vec![],
        })
    }
    fn poll(&mut self, expected_pid: u32) -> Result<Vec<(u8, u32)>> {
        if !self.connected {
            if unsafe { ConnectNamedPipe(self.handle.0, ptr::null_mut()) } == 0 {
                match unsafe { GetLastError() } {
                    ERROR_PIPE_CONNECTED => {}
                    ERROR_PIPE_LISTENING => return Ok(vec![]),
                    _ => return Err(os_error("connect private launch pipe")),
                }
            }
            let mut actual = 0;
            if unsafe { GetNamedPipeClientProcessId(self.handle.0, &mut actual) } == 0
                || actual != expected_pid
            {
                return Err(failure(
                    "private launch pipe peer does not match retained helper",
                ));
            }
            self.connected = true;
        }
        if self.written < self.frame.len() {
            let remaining = &self.frame[self.written..];
            let mut n = 0;
            if unsafe {
                WriteFile(
                    self.handle.0,
                    remaining.as_ptr(),
                    remaining.len().min(4096) as u32,
                    &mut n,
                    ptr::null_mut(),
                )
            } == 0
            {
                return Err(os_error("write private launch pipe"));
            }
            self.written += n as usize;
            if self.written != self.frame.len() {
                return Ok(vec![]);
            }
        }
        let mut bytes = [0u8; 10];
        let mut n = 0;
        if unsafe {
            ReadFile(
                self.handle.0,
                bytes.as_mut_ptr(),
                bytes.len() as u32,
                &mut n,
                ptr::null_mut(),
            )
        } == 0
        {
            match unsafe { GetLastError() } {
                ERROR_NO_DATA | ERROR_BROKEN_PIPE => return Ok(vec![]),
                _ => return Err(os_error("read helper acknowledgement")),
            }
        }
        self.ack.extend_from_slice(&bytes[..n as usize]);
        if self.ack.len() > 10 {
            return Err(failure("oversized helper acknowledgement"));
        }
        let count = self.ack.len() / 5;
        let messages = self.ack[..count * 5]
            .chunks_exact(5)
            .map(|b| (b[0], u32::from_le_bytes(b[1..5].try_into().unwrap())))
            .collect();
        self.ack.drain(..count * 5);
        Ok(messages)
    }
}

pub(super) fn host_main() -> Result<i32> {
    let args = std::env::args().skip(1).collect::<Vec<_>>();
    if args.len() != 5 || args[0] != PROTOCOL || uuid::Uuid::parse_str(&args[1]).is_err() {
        return Err(failure("invalid attempt host entry"));
    }
    let parent_pid: u32 = args[2].parse().map_err(|_| failure("invalid parent"))?;
    let timeout: u32 = args[4]
        .parse()
        .map_err(|_| failure("invalid barrier timeout"))?;
    if timeout == 0 || timeout > 30000 {
        return Err(failure("invalid barrier timeout"));
    }
    let parent = Handle::new(
        unsafe {
            OpenProcess(
                PROCESS_QUERY_LIMITED_INFORMATION | PROCESS_SYNCHRONIZE,
                0,
                parent_pid,
            )
        },
        "open parent",
    )?;
    if process_identity(parent.0)? != args[3] {
        return Err(failure("stale parent identity"));
    }
    let gate = Handle::new(
        unsafe {
            OpenEventW(
                0x00100000,
                0,
                wide(&format!("Local\\PytxoGate-{}", args[1])).as_ptr(),
            )
        },
        "open release barrier",
    )?;
    let handles = [parent.0, gate.0];
    if unsafe { WaitForMultipleObjects(2, handles.as_ptr(), 0, timeout) } != WAIT_OBJECT_0 + 1 {
        return Err(failure("unreleased or abandoned attempt"));
    }
    if !alive(parent.0)? {
        return Err(failure("parent exited before launch"));
    }
    let raw = unsafe {
        CreateFileW(
            wide(&format!("\\\\.\\pipe\\PytxoAttempt-{}", args[1])).as_ptr(),
            GENERIC_READ | GENERIC_WRITE,
            0,
            ptr::null(),
            OPEN_EXISTING,
            0,
            ptr::null_mut(),
        )
    };
    let pipe = Handle::new(raw, "open private launch pipe")?;
    let mut server = 0;
    if unsafe { GetNamedPipeServerProcessId(pipe.0, &mut server) } == 0 || server != parent_pid {
        return Err(failure("launch pipe server mismatch"));
    }
    let raw = pipe.0;
    std::mem::forget(pipe);
    let mut pipe = unsafe { std::fs::File::from_raw_handle(raw) };
    let mut size = [0u8; 4];
    pipe.read_exact(&mut size)?;
    let size = u32::from_le_bytes(size) as usize;
    if size == 0 || size > FRAME_LIMIT {
        return Err(failure("invalid private frame size"));
    }
    let mut bytes = vec![0u8; size];
    pipe.read_exact(&mut bytes)?;
    let frame: LaunchFrame =
        serde_json::from_slice(&bytes).map_err(|_| failure("invalid private launch frame"))?;
    if frame.version != 1
        || frame.stdin.len() > 16384
        || !frame.working_directory.is_absolute()
        || !alive(parent.0)?
    {
        return Err(failure("invalid or abandoned private frame"));
    }
    for pin in std::iter::once(&frame.executable).chain(&frame.dependencies) {
        if PinnedFile::observe(pin.path.clone())? != *pin {
            return Err(failure("payload chain changed before launch"));
        }
    }
    let mut child = Command::new(&frame.executable.path)
        .args(&frame.arguments)
        .env_clear()
        .envs(&frame.environment)
        .current_dir(&frame.working_directory)
        .creation_flags(if frame.pty { 0 } else { CREATE_NO_WINDOW })
        .stdin(Stdio::piped())
        .stdout(Stdio::inherit())
        .stderr(Stdio::inherit())
        .spawn()?;
    let mut started = vec![1];
    started.extend(child.id().to_le_bytes());
    pipe.write_all(&started)?;
    let mut stdin = child
        .stdin
        .take()
        .ok_or_else(|| failure("payload stdin missing"))?;
    let input = thread::spawn(move || stdin.write_all(&frame.stdin));
    let code = child
        .wait()?
        .code()
        .map(|c| c as u32)
        .ok_or_else(|| failure("payload exit unknown"))?;
    input
        .join()
        .map_err(|_| failure("payload stdin writer failed"))??;
    let mut completed = vec![2];
    completed.extend(code.to_le_bytes());
    pipe.write_all(&completed)?;
    Ok(if code == 0 { 0 } else { 1 })
}
fn append_error(receipt: &mut OwnedLaunchReceipt, error: impl std::fmt::Display) {
    receipt.outcome = OwnedOutcome::RecoveryRequired;
    receipt.error = Some(match receipt.error.take() {
        Some(old) => format!("{old}; {error}"),
        None => error.to_string(),
    });
}

fn validate_direct(spec: &DirectOwnedLaunchSpec) -> Result<Vec<PinnedFile>> {
    if !spec.working_directory.is_absolute()
        || !spec.working_directory.is_dir()
        || spec
            .executable
            .path
            .extension()
            .is_none_or(|value| !value.eq_ignore_ascii_case("exe"))
        || spec.execution_timeout.is_zero()
        || spec.execution_timeout > Duration::from_secs(86400)
        || spec.settlement_timeout.is_zero()
        || spec.settlement_timeout > Duration::from_secs(60)
        || spec.output_limit == 0
        || spec.output_limit > 16 * 1024 * 1024
    {
        return Err(failure("direct owned launch bounds are invalid"));
    }
    let mut keys = std::collections::BTreeSet::new();
    for (key, value) in &spec.environment {
        if key.is_empty()
            || key.contains(['=', '\0'])
            || value.contains('\0')
            || key.to_ascii_uppercase().starts_with("PYTXO_")
            || !keys.insert(key.to_ascii_uppercase())
        {
            return Err(failure("invalid direct payload environment"));
        }
    }
    if spec.arguments.iter().any(|value| value.contains('\0'))
        || spec
            .arguments
            .iter()
            .map(|value| value.encode_utf16().count() * 2 + 3)
            .sum::<usize>()
            > 24000
    {
        return Err(failure("invalid direct structured arguments"));
    }
    if spec.windows_cmd_verbatim_tail {
        let expected_cmd = system_root()?.join("System32").join("cmd.exe");
        let actual_cmd = spec.executable.path.to_string_lossy().replace('/', "\\");
        let expected_cmd = expected_cmd.to_string_lossy().replace('/', "\\");
        let valid_command = matches!(
            spec.arguments.as_slice(),
            [disable_auto_run, execute, command]
                if disable_auto_run == "/D"
                    && execute == "/C"
                    && !command.is_empty()
                    && command == command.trim()
                    && command.len() <= 4096
                    && !command.chars().any(char::is_control)
        );
        if !valid_command || !actual_cmd.eq_ignore_ascii_case(&expected_cmd) {
            return Err(failure(
                "verbatim Windows check requires the pinned cmd.exe /D /C tail",
            ));
        }
    }
    let actual = PinnedFile::observe(spec.executable.path.clone())?;
    if actual != spec.executable {
        return Err(failure("direct payload executable identity changed"));
    }
    Ok(vec![actual])
}

fn quote_direct_argument(value: &str) -> String {
    if !value.is_empty() && !value.contains([' ', '\t', '"']) {
        return value.into();
    }
    let mut quoted = String::from("\"");
    let mut slashes = 0;
    for character in value.chars() {
        match character {
            '\\' => slashes += 1,
            '"' => {
                quoted.extend(std::iter::repeat_n('\\', slashes * 2 + 1));
                quoted.push('"');
                slashes = 0;
            }
            _ => {
                quoted.extend(std::iter::repeat_n('\\', slashes));
                quoted.push(character);
                slashes = 0;
            }
        }
    }
    quoted.extend(std::iter::repeat_n('\\', slashes * 2));
    quoted.push('"');
    quoted
}

struct DirectPipe {
    read: Handle,
    write: Handle,
}
impl DirectPipe {
    fn create() -> Result<Self> {
        let attrs = SECURITY_ATTRIBUTES {
            nLength: std::mem::size_of::<SECURITY_ATTRIBUTES>() as u32,
            lpSecurityDescriptor: ptr::null_mut(),
            bInheritHandle: 1,
        };
        let mut read = ptr::null_mut();
        let mut write = ptr::null_mut();
        if unsafe { CreatePipe(&mut read, &mut write, &attrs, 0) } == 0 {
            return Err(os_error("create direct output pipe"));
        }
        let pipe = Self {
            read: Handle::new(read, "direct pipe reader")?,
            write: Handle::new(write, "direct pipe writer")?,
        };
        if unsafe { SetHandleInformation(pipe.read.0, HANDLE_FLAG_INHERIT, 0) } == 0 {
            return Err(os_error("protect direct pipe reader from inheritance"));
        }
        Ok(pipe)
    }
    fn into_capture(self, limit: usize) -> Capture {
        drop(self.write);
        let raw = self.read.0;
        std::mem::forget(self.read);
        capture(unsafe { std::fs::File::from_raw_handle(raw) }, limit)
    }
}

struct DirectAttributes {
    storage: Vec<usize>,
    initialized: bool,
}
impl DirectAttributes {
    fn new(job_handle: &HANDLE, inherited: &[HANDLE; 3]) -> Result<Self> {
        let mut size = 0;
        unsafe { InitializeProcThreadAttributeList(ptr::null_mut(), 2, 0, &mut size) };
        if size == 0 {
            return Err(os_error("size direct process attributes"));
        }
        let words = size.div_ceil(std::mem::size_of::<usize>());
        let mut result = Self {
            storage: vec![0; words],
            initialized: false,
        };
        if unsafe { InitializeProcThreadAttributeList(result.ptr(), 2, 0, &mut size) } == 0 {
            return Err(os_error("initialize direct process attributes"));
        }
        result.initialized = true;
        if unsafe {
            UpdateProcThreadAttribute(
                result.ptr(),
                0,
                PROC_THREAD_ATTRIBUTE_JOB_LIST as usize,
                (job_handle as *const HANDLE).cast(),
                std::mem::size_of::<HANDLE>(),
                ptr::null_mut(),
                ptr::null(),
            )
        } == 0
            || unsafe {
                UpdateProcThreadAttribute(
                    result.ptr(),
                    0,
                    PROC_THREAD_ATTRIBUTE_HANDLE_LIST as usize,
                    inherited.as_ptr().cast(),
                    std::mem::size_of_val(inherited),
                    ptr::null_mut(),
                    ptr::null(),
                )
            } == 0
        {
            return Err(os_error("bind direct Job and inherited handles"));
        }
        Ok(result)
    }
    fn ptr(&mut self) -> LPPROC_THREAD_ATTRIBUTE_LIST {
        self.storage.as_mut_ptr().cast()
    }
}
impl Drop for DirectAttributes {
    fn drop(&mut self) {
        if self.initialized {
            unsafe { DeleteProcThreadAttributeList(self.ptr()) };
        }
    }
}

struct DirectRunning {
    process: Handle,
    thread: Handle,
    pid: u32,
    stdout: Capture,
    stderr: Capture,
}

fn spawn_direct(spec: &DirectOwnedLaunchSpec, job: &Job) -> Result<DirectRunning> {
    let stdout = DirectPipe::create()?;
    let stderr = DirectPipe::create()?;
    let attrs = SECURITY_ATTRIBUTES {
        nLength: std::mem::size_of::<SECURITY_ATTRIBUTES>() as u32,
        lpSecurityDescriptor: ptr::null_mut(),
        bInheritHandle: 1,
    };
    let null_input = Handle::new(
        unsafe {
            CreateFileW(
                wide("NUL").as_ptr(),
                GENERIC_READ,
                FILE_SHARE_READ | FILE_SHARE_WRITE,
                &attrs,
                OPEN_EXISTING,
                FILE_ATTRIBUTE_NORMAL,
                ptr::null_mut(),
            )
        },
        "open direct null stdin",
    )?;
    let inherited = [null_input.0, stdout.write.0, stderr.write.0];
    let mut attributes = DirectAttributes::new(&job.0 .0, &inherited)?;
    let mut startup = STARTUPINFOEXW {
        StartupInfo: STARTUPINFOW {
            cb: std::mem::size_of::<STARTUPINFOW>() as u32,
            dwFlags: STARTF_USESTDHANDLES,
            hStdInput: null_input.0,
            hStdOutput: stdout.write.0,
            hStdError: stderr.write.0,
            ..Default::default()
        },
        lpAttributeList: attributes.ptr(),
    };
    startup.StartupInfo.cb = std::mem::size_of::<STARTUPINFOEXW>() as u32;
    let mut info = PROCESS_INFORMATION::default();
    let application_argument = spec.executable.path.to_string_lossy().replace('/', "\\");
    let argument_line = if spec.windows_cmd_verbatim_tail {
        // The exact reviewed command is one shell tail. CRT-style escaping
        // changes cmd.exe's quote semantics (for example findstr /C:"a b").
        format!("/D /C {}", spec.arguments[2])
    } else {
        spec.arguments
            .iter()
            .map(|arg| quote_direct_argument(arg))
            .collect::<Vec<_>>()
            .join(" ")
    };
    let mut command = format!(
        "{} {}",
        quote_direct_argument(&application_argument),
        argument_line
    )
    .encode_utf16()
    .chain(Some(0))
    .collect::<Vec<_>>();
    let mut env = environment()?;
    for (key, value) in &spec.environment {
        if env
            .keys()
            .any(|existing| existing.eq_ignore_ascii_case(key))
        {
            return Err(failure(
                "direct payload environment conflicts with system identity",
            ));
        }
        env.insert(key.clone(), value.clone());
    }
    let mut environment_block = env
        .iter()
        .flat_map(|(key, value)| {
            format!("{key}={value}\0")
                .encode_utf16()
                .collect::<Vec<_>>()
        })
        .chain(Some(0))
        .collect::<Vec<_>>();
    let application = spec
        .executable
        .path
        .as_os_str()
        .encode_wide()
        .chain(Some(0))
        .collect::<Vec<_>>();
    let cwd = spec
        .working_directory
        .as_os_str()
        .encode_wide()
        .chain(Some(0))
        .collect::<Vec<_>>();
    if unsafe {
        CreateProcessW(
            application.as_ptr(),
            command.as_mut_ptr(),
            ptr::null(),
            ptr::null(),
            1,
            CREATE_SUSPENDED
                | CREATE_NO_WINDOW
                | CREATE_UNICODE_ENVIRONMENT
                | EXTENDED_STARTUPINFO_PRESENT,
            environment_block.as_mut_ptr().cast(),
            cwd.as_ptr(),
            &startup.StartupInfo,
            &mut info,
        )
    } == 0
    {
        return Err(os_error("create suspended direct payload"));
    }
    let process = Handle::new(info.hProcess, "direct process handle")?;
    let thread = Handle::new(info.hThread, "direct thread handle")?;
    // The parent closes its writers before readers begin. The Job settles all
    // inheriting descendants before a capture is considered complete.
    Ok(DirectRunning {
        process,
        thread,
        pid: info.dwProcessId,
        stdout: stdout.into_capture(spec.output_limit),
        stderr: stderr.into_capture(spec.output_limit),
    })
}

pub(super) fn run_direct(
    spec: &DirectOwnedLaunchSpec,
    callbacks: &mut dyn LaunchCallbacks,
    cancelled: impl Fn() -> Result<bool>,
) -> Result<OwnedLaunchReceipt> {
    let started = Instant::now();
    let chain = validate_direct(spec)?;
    let argument_lowering = if spec.windows_cmd_verbatim_tail {
        DIRECT_CMD_TAIL_LOWERING
    } else {
        DIRECT_LOWERING
    };
    let nonce = uuid::Uuid::new_v4().to_string();
    let intent = LaunchIntent {
        job_name: format!("Local\\PytxoAttempt-{nonce}"),
        launch_nonce: nonce,
    };
    let job = Job::create(&intent.job_name)?;
    let mut receipt = OwnedLaunchReceipt {
        outcome: OwnedOutcome::Failed,
        intent: intent.clone(),
        bootstrap: None,
        process_registered: false,
        barrier_released: false,
        payload_pid_reported: None,
        payload_exit_code: None,
        observed_job_pids: vec![],
        active_processes: None,
        terminated_job: false,
        exit_code: None,
        stdout: String::new(),
        stderr: String::new(),
        stdout_utf8_valid: true,
        output_complete: false,
        output_truncated: false,
        observed_model: None,
        observed_usage: None,
        bundle_sha256: format!(
            "{:x}",
            Sha256::digest(
                serde_json::to_vec(&(&chain, DIRECT_PROTOCOL, argument_lowering, &spec.arguments))
                    .map_err(|_| failure("direct bundle encoding failed"))?
            )
        ),
        chain,
        bootstrap_sha256: spec.executable.sha256.clone(),
        protocol: DIRECT_PROTOCOL.into(),
        argument_lowering: argument_lowering.into(),
        elapsed_ms: 0,
        error: None,
    };
    let mut seen_job_pids = HashSet::new();
    let running;
    let mut assigned = false;
    {
        let mut guard = callbacks.authorize(&intent)?;
        let preflight = (|| -> Result<bool> {
            if guard.cancelled()? || cancelled()? {
                return Ok(false);
            }
            validate_direct(spec)?;
            if guard.cancelled()? || cancelled()? {
                return Ok(false);
            }
            if started.elapsed() >= spec.execution_timeout {
                return Err(failure(
                    "direct owned execution deadline exceeded before spawn",
                ));
            }
            Ok(true)
        })();
        if !matches!(preflight, Ok(true)) {
            match preflight {
                Ok(false) => receipt.outcome = OwnedOutcome::Cancelled,
                Err(error) => receipt.error = Some(error.to_string()),
                _ => unreachable!(),
            }
            receipt.active_processes = Some(0);
            receipt.output_complete = true;
            receipt.elapsed_ms = started.elapsed().as_millis();
            drop(guard);
            if let Err(error) = callbacks.settle(&receipt) {
                append_error(&mut receipt, error);
            }
            return Ok(receipt);
        }
        if let Err(error) = guard.permit_create(&intent) {
            receipt.error = Some(error.to_string());
            receipt.active_processes = Some(0);
            receipt.output_complete = true;
            receipt.elapsed_ms = started.elapsed().as_millis();
            drop(guard);
            if let Err(error) = callbacks.settle(&receipt) {
                append_error(&mut receipt, error);
            }
            return Ok(receipt);
        }
        if started.elapsed() >= spec.execution_timeout {
            receipt.error =
                Some("direct owned execution deadline exceeded after create authorization".into());
            receipt.active_processes = Some(0);
            receipt.output_complete = true;
            receipt.elapsed_ms = started.elapsed().as_millis();
            drop(guard);
            if let Err(error) = callbacks.settle(&receipt) {
                append_error(&mut receipt, error);
            }
            return Ok(receipt);
        }
        running = match spawn_direct(spec, &job) {
            Ok(running) => running,
            Err(error) => {
                append_error(&mut receipt, error);
                receipt.elapsed_ms = started.elapsed().as_millis();
                drop(guard);
                if let Err(error) = callbacks.settle(&receipt) {
                    append_error(&mut receipt, error);
                }
                return Ok(receipt);
            }
        };
        let setup = (|| -> Result<()> {
            let process = OwnedProcess {
                pid: running.pid,
                start_identity: Some(process_identity(running.process.0)?),
                job_name: intent.job_name.clone(),
            };
            receipt.bootstrap = Some(process.clone());
            let mut member = 0;
            if unsafe { IsProcessInJob(running.process.0, job.0 .0, &mut member) } == 0
                || member == 0
            {
                return Err(os_error("verify atomic direct Job membership"));
            }
            assigned = true;
            guard.register(&process)?;
            receipt.process_registered = true;
            if guard.cancelled()? || cancelled()? {
                receipt.outcome = OwnedOutcome::Cancelled;
                return Ok(());
            }
            if started.elapsed() >= spec.execution_timeout {
                return Err(failure(
                    "direct owned execution deadline exceeded before release",
                ));
            }
            let previous = unsafe { ResumeThread(running.thread.0) };
            if previous != 1 {
                return Err(failure(format!(
                    "registered direct payload had unexpected suspend count {previous}"
                )));
            }
            receipt.barrier_released = true;
            receipt.payload_pid_reported = Some(running.pid);
            Ok(())
        })();
        if let Err(error) = setup {
            append_error(&mut receipt, error);
        }
    }
    if receipt.barrier_released && receipt.error.is_none() {
        loop {
            match job.members() {
                Ok(pids) => record_job_members(&mut receipt, &mut seen_job_pids, pids),
                Err(error) => {
                    append_error(&mut receipt, error);
                    break;
                }
            }
            match cancelled() {
                Ok(true) => {
                    receipt.outcome = OwnedOutcome::Cancelled;
                    break;
                }
                Err(error) => {
                    append_error(&mut receipt, error);
                    break;
                }
                _ => {}
            }
            if started.elapsed() >= spec.execution_timeout {
                append_error(&mut receipt, "direct owned execution deadline exceeded");
                break;
            }
            match unsafe { WaitForSingleObject(running.process.0, 0) } {
                WAIT_OBJECT_0 => {
                    let mut code = 0;
                    if unsafe { GetExitCodeProcess(running.process.0, &mut code) } == 0 {
                        append_error(&mut receipt, os_error("query direct payload exit"));
                    } else {
                        receipt.exit_code = Some(code);
                        receipt.payload_exit_code = Some(code);
                    }
                    if started.elapsed() >= spec.execution_timeout {
                        append_error(&mut receipt, "direct owned execution deadline exceeded");
                    }
                    break;
                }
                WAIT_TIMEOUT => {}
                _ => {
                    append_error(&mut receipt, os_error("wait for direct payload"));
                    break;
                }
            }
            if started.elapsed() >= spec.execution_timeout {
                append_error(&mut receipt, "direct owned execution deadline exceeded");
                break;
            }
            thread::sleep(Duration::from_millis(10));
        }
    }
    let settlement = (|| -> Result<()> {
        if !assigned {
            if alive(running.process.0)? && unsafe { TerminateProcess(running.process.0, 125) } == 0
            {
                return Err(os_error("terminate unassigned suspended payload"));
            }
            if unsafe {
                WaitForSingleObject(
                    running.process.0,
                    spec.settlement_timeout.as_millis() as u32,
                )
            } != WAIT_OBJECT_0
            {
                return Err(failure("unassigned direct payload exit remains unknown"));
            }
        }
        if receipt.exit_code.is_some()
            && receipt.error.is_none()
            && receipt.outcome != OwnedOutcome::Cancelled
        {
            // cmd.exe's console infrastructure may leave the Job a few ticks
            // after the command has exited; only observed Job zero is success.
            let until = Instant::now() + Duration::from_millis(300).min(spec.settlement_timeout);
            while job.active()? != 0 && Instant::now() < until {
                thread::sleep(Duration::from_millis(10));
            }
        }
        record_job_members(&mut receipt, &mut seen_job_pids, job.members()?);
        if job.active()? != 0 {
            receipt.terminated_job = true;
            job.terminate()?;
        }
        job.wait_zero(spec.settlement_timeout)?;
        receipt.active_processes = Some(0);
        Ok(())
    })();
    if let Err(error) = settlement {
        append_error(&mut receipt, error);
    }
    let deadline = Instant::now() + spec.settlement_timeout;
    let mut complete = true;
    for (index, (source, target)) in [
        (&running.stdout, &mut receipt.stdout),
        (&running.stderr, &mut receipt.stderr),
    ]
    .into_iter()
    .enumerate()
    {
        match source.recv_timeout(deadline.saturating_duration_since(Instant::now())) {
            Ok((bytes, truncated, eof)) => {
                let (decoded, valid) = decode_capture(&bytes);
                if index == 0 {
                    receipt.stdout_utf8_valid = valid;
                }
                *target = decoded;
                receipt.output_truncated |= truncated;
                complete &= eof;
            }
            Err(_) => complete = false,
        }
    }
    receipt.output_complete = complete;
    if receipt.outcome == OwnedOutcome::Failed
        && receipt.error.is_none()
        && receipt.exit_code == Some(0)
        && receipt.payload_exit_code == Some(0)
        && receipt.process_registered
        && receipt.barrier_released
        && receipt.active_processes == Some(0)
        && !receipt.terminated_job
        && complete
        && !receipt.output_truncated
    {
        receipt.outcome = OwnedOutcome::Succeeded;
    }
    receipt.elapsed_ms = started.elapsed().as_millis();
    if let Err(error) = callbacks.settle(&receipt) {
        append_error(&mut receipt, error);
    }
    Ok(receipt)
}

// Private, explicit OS seam. Production always uses NativeLaunchOps; fixtures can
// inject OS failures without ambient switches or changing process security policy.
trait LaunchOps {
    fn validate(&self, spec: &OwnedLaunchSpec) -> Result<Vec<PinnedFile>> {
        validate(spec)
    }
    fn spawn(
        &self,
        spec: &OwnedLaunchSpec,
        env: &BTreeMap<String, String>,
        intent: &LaunchIntent,
    ) -> Result<Running> {
        spawn(spec, env, intent)
    }
    fn identity(&self, handle: HANDLE) -> Result<String> {
        process_identity(handle)
    }
    fn try_wait(&self, running: &mut Running) -> std::io::Result<Option<portable_pty::ExitStatus>> {
        running.child.try_wait()
    }
    fn release(&self, barrier: HANDLE) -> Result<()> {
        if unsafe { SetEvent(barrier) } == 0 {
            return Err(os_error("release owned barrier"));
        }
        Ok(())
    }
}
struct NativeLaunchOps;
impl LaunchOps for NativeLaunchOps {}

pub(super) fn run(
    spec: &OwnedLaunchSpec,
    callbacks: &mut dyn LaunchCallbacks,
    cancelled: impl Fn() -> Result<bool>,
) -> Result<OwnedLaunchReceipt> {
    run_with_ops(spec, callbacks, cancelled, &NativeLaunchOps)
}

fn run_with_ops(
    spec: &OwnedLaunchSpec,
    callbacks: &mut dyn LaunchCallbacks,
    cancelled: impl Fn() -> Result<bool>,
    ops: &dyn LaunchOps,
) -> Result<OwnedLaunchReceipt> {
    let started = Instant::now();
    let chain = ops.validate(spec)?;
    let nonce = uuid::Uuid::new_v4().to_string();
    let intent = LaunchIntent {
        job_name: format!("Local\\PytxoAttempt-{nonce}"),
        launch_nonce: nonce,
    };
    let env = environment()?;
    let security = PrivateSecurity::new()?;
    let attrs = security.attributes();
    let job = Job::create(&intent.job_name)?;
    let event = unsafe {
        CreateEventW(
            &attrs,
            1,
            0,
            wide(&format!("Local\\PytxoGate-{}", intent.launch_nonce)).as_ptr(),
        )
    };
    let existing = unsafe { GetLastError() } == ERROR_ALREADY_EXISTS;
    let event = Handle::new(event, "create private release barrier")?;
    if existing {
        return Err(failure("refusing existing release barrier"));
    }
    let mut pipe = PromptPipe::create(&intent.launch_nonce, spec, &security)?;
    let mut receipt = OwnedLaunchReceipt {
        outcome: OwnedOutcome::Failed,
        intent: intent.clone(),
        bootstrap: None,
        process_registered: false,
        barrier_released: false,
        payload_pid_reported: None,
        payload_exit_code: None,
        observed_job_pids: vec![],
        active_processes: None,
        terminated_job: false,
        exit_code: None,
        stdout: String::new(),
        stderr: String::new(),
        stdout_utf8_valid: true,
        output_complete: false,
        output_truncated: false,
        observed_model: None,
        observed_usage: None,
        bundle_sha256: format!(
            "{:x}",
            Sha256::digest(
                serde_json::to_vec(&(&chain, PROTOCOL, LOWERING))
                    .map_err(|_| failure("bundle encoding failed"))?
            )
        ),
        chain,
        bootstrap_sha256: spec.bootstrap_host.sha256.clone(),
        protocol: PROTOCOL.into(),
        argument_lowering: LOWERING.into(),
        elapsed_ms: 0,
        error: None,
    };
    let mut seen_job_pids = HashSet::new();
    let mut running;
    let mut assigned = false;
    {
        // Caller-owned durable gate spans launching, creation, registration and release.
        let mut guard = callbacks.authorize(&intent)?;
        let preflight = (|| -> Result<bool> {
            if guard.cancelled()? || cancelled()? {
                return Ok(false);
            }
            // Rehash under the gate; a failed recheck still settles the recorded intent.
            ops.validate(spec)?;
            if guard.cancelled()? || cancelled()? {
                return Ok(false);
            }
            if started.elapsed() >= spec.execution_timeout {
                return Err(failure("owned execution deadline exceeded before spawn"));
            }
            Ok(true)
        })();
        if !matches!(preflight, Ok(true)) {
            match preflight {
                Ok(false) => receipt.outcome = OwnedOutcome::Cancelled,
                Err(e) => receipt.error = Some(e.to_string()),
                _ => unreachable!(),
            }
            receipt.active_processes = Some(0);
            receipt.output_complete = true;
            receipt.elapsed_ms = started.elapsed().as_millis();
            drop(guard);
            if let Err(e) = callbacks.settle(&receipt) {
                append_error(&mut receipt, e);
            }
            return Ok(receipt);
        }
        if let Err(error) = guard.permit_create(&intent) {
            receipt.error = Some(error.to_string());
            receipt.active_processes = Some(0);
            receipt.output_complete = true;
            receipt.elapsed_ms = started.elapsed().as_millis();
            drop(guard);
            if let Err(error) = callbacks.settle(&receipt) {
                append_error(&mut receipt, error);
            }
            return Ok(receipt);
        }
        if started.elapsed() >= spec.execution_timeout {
            receipt.error =
                Some("owned execution deadline exceeded after create authorization".into());
            receipt.active_processes = Some(0);
            receipt.output_complete = true;
            receipt.elapsed_ms = started.elapsed().as_millis();
            drop(guard);
            if let Err(error) = callbacks.settle(&receipt) {
                append_error(&mut receipt, error);
            }
            return Ok(receipt);
        }
        running = match ops.spawn(spec, &env, &intent) {
            Ok(running) => running,
            Err(e) => {
                // Backend errors cannot establish that CreateProcess never ran.
                append_error(&mut receipt, e);
                receipt.elapsed_ms = started.elapsed().as_millis();
                drop(guard);
                if let Err(e) = callbacks.settle(&receipt) {
                    append_error(&mut receipt, e);
                }
                return Ok(receipt);
            }
        };
        let setup = (|| -> Result<()> {
            let pid = running
                .child
                .process_id()
                .ok_or_else(|| failure("missing bootstrap pid"))?;
            receipt.bootstrap = Some(OwnedProcess {
                pid,
                start_identity: None,
                job_name: intent.job_name.clone(),
            });
            let handle = running.handle()?;
            receipt.bootstrap.as_mut().unwrap().start_identity = Some(ops.identity(handle)?);
            let process = receipt.bootstrap.as_ref().unwrap().clone();
            job.assign(handle)?;
            assigned = true;
            guard.register(&process)?;
            receipt.process_registered = true;
            if guard.cancelled()? || cancelled()? {
                receipt.outcome = OwnedOutcome::Cancelled;
                return Ok(());
            }
            if !alive(handle)? {
                return Err(failure("bootstrap exited before release"));
            }
            if started.elapsed() >= spec.execution_timeout {
                return Err(failure("owned execution deadline exceeded before release"));
            }
            ops.release(event.0)?;
            receipt.barrier_released = true;
            Ok(())
        })();
        if let Err(e) = setup {
            append_error(&mut receipt, e);
        }
    }
    if receipt.barrier_released && receipt.error.is_none() {
        loop {
            match job.members() {
                Ok(pids) => record_job_members(&mut receipt, &mut seen_job_pids, pids),
                Err(e) => {
                    append_error(&mut receipt, e);
                    break;
                }
            }
            match cancelled() {
                Ok(true) => {
                    receipt.outcome = OwnedOutcome::Cancelled;
                    break;
                }
                Err(e) => {
                    append_error(&mut receipt, e);
                    break;
                }
                _ => {}
            }
            if started.elapsed() >= spec.execution_timeout {
                receipt.error = Some("owned execution deadline exceeded".into());
                break;
            }
            match pipe.poll(receipt.bootstrap.as_ref().unwrap().pid) {
                Ok(messages) => {
                    for (kind, value) in messages {
                        match kind {
                            1 if receipt.payload_pid_reported.is_none() => {
                                receipt.payload_pid_reported = Some(value)
                            }
                            2 if receipt.payload_pid_reported.is_some()
                                && receipt.payload_exit_code.is_none() =>
                            {
                                receipt.payload_exit_code = Some(value)
                            }
                            _ => {
                                append_error(&mut receipt, "invalid helper acknowledgement order");
                            }
                        }
                    }
                }
                Err(e) => {
                    append_error(&mut receipt, e);
                    break;
                }
            }
            if receipt.error.is_some() {
                break;
            }
            match ops.try_wait(&mut running) {
                Ok(Some(status)) => {
                    receipt.exit_code = Some(status.exit_code());
                    // Drain the completion acknowledgement after root exit; it may share the last tick.
                    match pipe.poll(receipt.bootstrap.as_ref().unwrap().pid) {
                        Ok(messages) => {
                            for (kind, value) in messages {
                                match kind {
                                    1 if receipt.payload_pid_reported.is_none() => {
                                        receipt.payload_pid_reported = Some(value)
                                    }
                                    2 if receipt.payload_pid_reported.is_some()
                                        && receipt.payload_exit_code.is_none() =>
                                    {
                                        receipt.payload_exit_code = Some(value)
                                    }
                                    _ => append_error(
                                        &mut receipt,
                                        "invalid final helper acknowledgement",
                                    ),
                                }
                            }
                        }
                        Err(e) => append_error(&mut receipt, e),
                    }
                    if started.elapsed() >= spec.execution_timeout {
                        append_error(&mut receipt, "owned execution deadline exceeded");
                    }
                    break;
                }
                Err(e) => {
                    append_error(&mut receipt, e);
                    break;
                }
                _ => {}
            }
            if started.elapsed() >= spec.execution_timeout {
                receipt.error = Some("owned execution deadline exceeded".into());
                break;
            }
            thread::sleep(Duration::from_millis(10));
        }
    }
    if receipt.outcome == OwnedOutcome::Failed
        && receipt.barrier_released
        && receipt.exit_code.is_some()
        && receipt.payload_exit_code.is_none()
    {
        // Helper/control failure is not an acknowledged ordinary worker failure.
        // Keep the receipt even when the subsequent owned Job reaches zero.
        append_error(
            &mut receipt,
            "helper exited without a payload completion acknowledgement",
        );
    }
    drop(pipe); // Private input never enters the PTY or appears in argv/environment.
    let settlement = (|| -> Result<()> {
        if !assigned {
            // Assignment failure: terminate only the exact retained bootstrap handle.
            let h = running.handle()?;
            if alive(h)? && unsafe { TerminateProcess(h, 125) } == 0 {
                return Err(os_error("terminate unassigned inert bootstrap"));
            }
            if unsafe { WaitForSingleObject(h, spec.settlement_timeout.as_millis() as u32) }
                != WAIT_OBJECT_0
            {
                return Err(failure("unassigned bootstrap exit remains unknown"));
            }
        }
        if receipt.exit_code.is_some()
            && receipt.error.is_none()
            && receipt.outcome != OwnedOutcome::Cancelled
        {
            // ConPTY can briefly retain console infrastructure after the root exits.
            let until = Instant::now() + Duration::from_millis(300).min(spec.settlement_timeout);
            while job.active()? != 0 && Instant::now() < until {
                thread::sleep(Duration::from_millis(10));
            }
        }
        record_job_members(&mut receipt, &mut seen_job_pids, job.members()?);
        if job.active()? != 0 {
            receipt.terminated_job = true;
            job.terminate()?;
        }
        job.wait_zero(spec.settlement_timeout)?;
        receipt.active_processes = Some(0);
        Ok(())
    })();
    if let Err(e) = settlement {
        append_error(&mut receipt, e);
    }
    // Never use root exit, reader completion or a missing job as a quiescence receipt.
    drop(running.master.take());
    let capture_deadline = Instant::now() + spec.settlement_timeout;
    let mut complete = true;
    for (index, (rx, target)) in [(&running.stdout, &mut receipt.stdout)]
        .into_iter()
        .chain(running.stderr.as_ref().map(|rx| (rx, &mut receipt.stderr)))
        .enumerate()
    {
        match rx.recv_timeout(capture_deadline.saturating_duration_since(Instant::now())) {
            Ok((bytes, truncated, eof)) => {
                let (decoded, valid) = decode_capture(&bytes);
                if index == 0 {
                    receipt.stdout_utf8_valid = valid;
                }
                *target = decoded;
                receipt.output_truncated |= truncated;
                complete &= eof;
            }
            Err(_) => complete = false,
        }
    }
    receipt.output_complete = complete;
    if receipt.outcome == OwnedOutcome::Failed
        && receipt.error.is_none()
        && receipt.exit_code == Some(0)
        && receipt.payload_pid_reported.is_some()
        && receipt.payload_exit_code == Some(0)
        && !receipt.terminated_job
        && receipt.active_processes == Some(0)
        && complete
        && !receipt.output_truncated
    {
        receipt.outcome = OwnedOutcome::Succeeded;
    }
    receipt.elapsed_ms = started.elapsed().as_millis();
    if let Err(e) = callbacks.settle(&receipt) {
        append_error(&mut receipt, e);
    }
    Ok(receipt)
}

#[cfg(test)]
mod fault_tests {
    use super::*;
    use std::cell::Cell;

    #[test]
    fn invalid_capture_is_displayable_but_cannot_be_candidate_text() {
        assert_eq!(decode_capture(b"valid\n"), ("valid\n".into(), true));
        let (display, valid) = decode_capture(b"invalid\xff");
        assert!(!valid);
        assert!(display.contains('\u{fffd}'));
    }

    #[test]
    fn partial_job_member_snapshot_preserves_valid_telemetry() {
        assert_eq!(
            member_snapshot(&[12, 34, 0], 2, 2).unwrap(),
            (vec![12, 34], false)
        );
        // Churn can make the counts disagree even with enough buffer space.
        assert_eq!(
            member_snapshot(&[12, 34, 0], 3, 2).unwrap(),
            (vec![12, 34], false)
        );
        assert_eq!(
            member_snapshot(&[12, 34, 0], 5, 2).unwrap(),
            (vec![12, 34], true)
        );
        assert!(member_snapshot(&[12, 34], 3, 3).is_err());
    }

    #[test]
    fn hosted_payload_cannot_redirect_windows_root() {
        let observed = system_root().unwrap().to_string_lossy().into_owned();
        let mut environment = BTreeMap::from([
            ("SystemRoot".to_owned(), observed.clone()),
            ("WINDIR".to_owned(), observed.clone()),
        ]);
        validate_payload_system_root(&environment).unwrap();
        environment.insert("WINDIR".to_owned(), format!("{observed}\\System32"));
        assert!(validate_payload_system_root(&environment).is_err());
    }

    #[derive(Default)]
    struct RecordingCallbacks {
        settled: usize,
        registered: bool,
        create_permitted: usize,
        permit_delay: Duration,
    }
    struct Guard<'a>(&'a mut RecordingCallbacks);
    impl LaunchGuard for Guard<'_> {
        fn permit_create(&mut self, _: &LaunchIntent) -> Result<()> {
            self.0.create_permitted += 1;
            thread::sleep(self.0.permit_delay);
            Ok(())
        }

        fn register(&mut self, process: &OwnedProcess) -> Result<()> {
            assert!(
                observe(process)?.is_some_and(|active| active > 0),
                "registered live helper must have observed membership"
            );
            self.0.registered = true;
            Ok(())
        }
        fn cancelled(&mut self) -> Result<bool> {
            Ok(false)
        }
    }
    impl LaunchCallbacks for RecordingCallbacks {
        fn authorize(&mut self, _: &LaunchIntent) -> Result<Box<dyn LaunchGuard + '_>> {
            Ok(Box::new(Guard(self)))
        }
        fn settle(&mut self, _: &OwnedLaunchReceipt) -> Result<()> {
            self.settled += 1;
            Ok(())
        }
    }
    enum Fault {
        Spawn,
        Identity,
    }
    impl LaunchOps for Fault {
        fn spawn(
            &self,
            spec: &OwnedLaunchSpec,
            env: &BTreeMap<String, String>,
            intent: &LaunchIntent,
        ) -> Result<Running> {
            if matches!(self, Self::Spawn) {
                Err(failure("injected ambiguous spawn failure"))
            } else {
                spawn(spec, env, intent)
            }
        }
        fn identity(&self, handle: HANDLE) -> Result<String> {
            if matches!(self, Self::Identity) {
                Err(failure("injected start identity query failure"))
            } else {
                process_identity(handle)
            }
        }
    }
    fn cheap_chain(spec: &OwnedLaunchSpec) -> Vec<PinnedFile> {
        std::iter::once(spec.bootstrap_host.clone())
            .chain(std::iter::once(spec.executable.clone()))
            .chain(spec.dependencies.iter().cloned())
            .collect()
    }
    #[derive(Default)]
    struct DelayedValidation {
        validations: Cell<usize>,
        spawned: Cell<bool>,
    }
    impl LaunchOps for DelayedValidation {
        fn validate(&self, spec: &OwnedLaunchSpec) -> Result<Vec<PinnedFile>> {
            self.validations.set(self.validations.get() + 1);
            if self.validations.get() == 2 {
                thread::sleep(Duration::from_millis(40));
            }
            Ok(cheap_chain(spec))
        }
        fn spawn(
            &self,
            _: &OwnedLaunchSpec,
            _: &BTreeMap<String, String>,
            _: &LaunchIntent,
        ) -> Result<Running> {
            self.spawned.set(true);
            Err(failure("spawn should not follow an expired rehash"))
        }
    }
    #[derive(Default)]
    struct NeverSpawn(Cell<bool>);
    impl LaunchOps for NeverSpawn {
        fn validate(&self, spec: &OwnedLaunchSpec) -> Result<Vec<PinnedFile>> {
            Ok(cheap_chain(spec))
        }
        fn spawn(
            &self,
            _: &OwnedLaunchSpec,
            _: &BTreeMap<String, String>,
            _: &LaunchIntent,
        ) -> Result<Running> {
            self.0.set(true);
            Err(failure("spawn should not follow an expired create permit"))
        }
    }
    struct DelayedIdentity;
    impl LaunchOps for DelayedIdentity {
        fn validate(&self, spec: &OwnedLaunchSpec) -> Result<Vec<PinnedFile>> {
            Ok(cheap_chain(spec))
        }
        fn identity(&self, handle: HANDLE) -> Result<String> {
            thread::sleep(Duration::from_millis(1200));
            process_identity(handle)
        }
    }
    #[derive(Default)]
    struct DelayedExit {
        saw_completed_child: Cell<bool>,
    }
    impl LaunchOps for DelayedExit {
        fn validate(&self, spec: &OwnedLaunchSpec) -> Result<Vec<PinnedFile>> {
            Ok(cheap_chain(spec))
        }
        fn try_wait(
            &self,
            running: &mut Running,
        ) -> std::io::Result<Option<portable_pty::ExitStatus>> {
            let result = running.child.try_wait()?;
            if result.is_some() {
                self.saw_completed_child.set(true);
                thread::sleep(Duration::from_millis(5200));
            }
            Ok(result)
        }
    }
    fn spec() -> OwnedLaunchSpec {
        let pin = PinnedFile::observe(PathBuf::from(
            std::env::var_os("PYTXO_TEST_ATTEMPT_HOST")
                .expect("prebuild native companion and set its explicit test pin"),
        ))
        .unwrap();
        // The pinned companion can be an embedded Desktop/CLI host rather
        // than the old standalone helper. Exercise the owned payload edge
        // with a separate, bounded system child in either case.
        let payload = PinnedFile::observe(system_root().unwrap().join("System32/cmd.exe")).unwrap();
        let execution_timeout = std::env::var("PYTXO_TEST_OWNED_EXECUTION_TIMEOUT_SECS")
            .ok()
            .and_then(|value| value.parse::<u64>().ok())
            .filter(|seconds| (1..=120).contains(seconds))
            .map(Duration::from_secs)
            .unwrap_or(Duration::from_secs(8));
        OwnedLaunchSpec {
            bootstrap_host: pin,
            executable: payload,
            dependencies: vec![],
            arguments: vec!["/C".into(), "ping -n 30 127.0.0.1 >NUL".into()],
            environment: BTreeMap::new(),
            working_directory: system_root().unwrap().join("System32"),
            stdin: vec![],
            transport: OwnedTransport::Subprocess,
            barrier_timeout: Duration::from_secs(5),
            execution_timeout,
            settlement_timeout: Duration::from_secs(3),
            output_limit: 4096,
        }
    }
    #[test]
    fn expired_prelaunch_budget_never_calls_spawn() {
        let mut launch = spec();
        launch.execution_timeout = Duration::from_millis(20);
        let mut callbacks = RecordingCallbacks::default();
        let ops = DelayedValidation::default();
        let r = run_with_ops(&launch, &mut callbacks, || Ok(false), &ops).unwrap();
        assert_eq!(r.outcome, OwnedOutcome::Failed);
        assert_eq!(r.active_processes, Some(0));
        assert!(!r.process_registered && !r.barrier_released);
        assert!(r.error.as_deref().unwrap_or_default().contains("deadline"));
        assert!(r.elapsed_ms >= 40);
        assert_eq!(callbacks.settled, 1);
        assert_eq!(callbacks.create_permitted, 0);
        assert_eq!(ops.validations.get(), 2);
        assert!(!ops.spawned.get());
    }
    #[test]
    fn expired_postpermit_budget_never_calls_spawn() {
        let mut launch = spec();
        launch.execution_timeout = Duration::from_millis(50);
        let mut callbacks = RecordingCallbacks {
            permit_delay: Duration::from_millis(100),
            ..Default::default()
        };
        let ops = NeverSpawn::default();
        let receipt = run_with_ops(&launch, &mut callbacks, || Ok(false), &ops).unwrap();
        assert_eq!(callbacks.create_permitted, 1);
        assert_eq!(callbacks.settled, 1);
        assert!(!ops.0.get());
        assert!(!receipt.process_registered && !receipt.barrier_released);
        assert_eq!(receipt.active_processes, Some(0));
        assert!(receipt
            .error
            .as_deref()
            .unwrap_or_default()
            .contains("deadline"));
        assert!(receipt.elapsed_ms >= 100);
    }
    #[test]
    fn expired_pre_release_budget_settles_job_without_releasing_payload() {
        let mut launch = spec();
        launch.execution_timeout = Duration::from_secs(1);
        let mut callbacks = RecordingCallbacks::default();
        let r = run_with_ops(&launch, &mut callbacks, || Ok(false), &DelayedIdentity).unwrap();
        assert!(callbacks.registered);
        assert_eq!(callbacks.settled, 1);
        assert!(r.process_registered);
        assert!(!r.barrier_released);
        assert_eq!(r.payload_pid_reported, None);
        assert_eq!(r.active_processes, Some(0));
        assert!(r.terminated_job);
        assert!(r.error.as_deref().unwrap_or_default().contains("deadline"));
    }
    #[test]
    fn completed_child_observed_after_deadline_cannot_succeed() {
        let mut launch = spec();
        launch.executable =
            PinnedFile::observe(system_root().unwrap().join("System32/cmd.exe")).unwrap();
        launch.arguments = vec!["/C".into(), "exit".into(), "0".into()];
        // Leave room for a loaded Windows host to reach the already-exited
        // child; DelayedExit then crosses the deadline after observing it.
        launch.execution_timeout = Duration::from_secs(5);
        let mut callbacks = RecordingCallbacks::default();
        let ops = DelayedExit::default();
        let r = run_with_ops(&launch, &mut callbacks, || Ok(false), &ops).unwrap();
        assert!(
            ops.saw_completed_child.get(),
            "test must exercise a late observed exit: {r:?}"
        );
        assert!(r.barrier_released);
        assert_ne!(r.outcome, OwnedOutcome::Succeeded);
        assert!(r.error.as_deref().unwrap_or_default().contains("deadline"));
        assert_eq!(r.active_processes, Some(0));
        assert_eq!(callbacks.settled, 1);
        assert_eq!(callbacks.create_permitted, 1);
    }
    #[test]
    fn ambiguous_spawn_error_settles_recovery_without_inventing_no_launch() {
        let mut callbacks = RecordingCallbacks::default();
        let r = run_with_ops(&spec(), &mut callbacks, || Ok(false), &Fault::Spawn).unwrap();
        assert_eq!(r.outcome, OwnedOutcome::RecoveryRequired);
        assert_eq!(r.active_processes, None);
        assert!(!r.process_registered && !r.barrier_released);
        assert!(!r.intent.launch_nonce.is_empty());
        assert_eq!(callbacks.settled, 1);
        assert_eq!(callbacks.create_permitted, 1);
    }
    #[test]
    fn start_query_failure_retains_pid_job_nonce_and_settles_exact_helper() {
        let mut callbacks = RecordingCallbacks::default();
        let r = run_with_ops(&spec(), &mut callbacks, || Ok(false), &Fault::Identity).unwrap();
        let process = r.bootstrap.as_ref().unwrap();
        assert_ne!(process.pid, 0);
        assert_eq!(process.start_identity, None);
        assert_eq!(process.job_name, r.intent.job_name);
        assert!(!r.intent.launch_nonce.is_empty());
        assert_eq!(r.outcome, OwnedOutcome::RecoveryRequired);
        assert_eq!(r.active_processes, Some(0));
        assert!(!r.process_registered && !r.barrier_released && !callbacks.registered);
        assert_eq!(callbacks.settled, 1);
    }
    #[test]
    fn cancellation_query_errors_settle_before_create_after_registration_and_after_release() {
        for failure_call in 1..=4 {
            let mut callbacks = RecordingCallbacks::default();
            let calls = Cell::new(0);
            let r = run_with_ops(
                &spec(),
                &mut callbacks,
                || {
                    calls.set(calls.get() + 1);
                    if calls.get() == failure_call {
                        Err(failure("injected cancellation query failure"))
                    } else {
                        Ok(false)
                    }
                },
                &NativeLaunchOps,
            )
            .unwrap();
            assert_eq!(callbacks.settled, 1);
            assert_eq!(r.active_processes, Some(0));
            assert!(r.error.as_ref().unwrap().contains("cancellation query"));
            assert_eq!(r.bootstrap.is_some(), failure_call > 2);
            assert_eq!(r.barrier_released, failure_call == 4);
            assert_eq!(
                r.outcome,
                if failure_call <= 2 {
                    OwnedOutcome::Failed
                } else {
                    OwnedOutcome::RecoveryRequired
                }
            );
        }
    }
}
