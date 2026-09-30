//! Exact, inert Claude subscription launch shape for a reviewed routed attempt.
//! A separate owned auth/tool/cancellation probe must qualify this shape before
//! Core may mark either profile Ready or admit a worker.

use std::collections::{BTreeMap, BTreeSet};
use std::path::{Path, PathBuf};
use std::time::Duration;

use anyhow::{bail, Context};
use pytxo_core::routing::{
    canonical_digest, BillingSourceId, BillingSourceMode, BindingId, Digest, ExecutableIdentity,
    ExecutionProfile, LaunchContract, LaunchTransport, MeteringSupport, ModelIdentity,
    ModelIdentityLevel, ProfileBinding, RouteTarget, StdinDelivery,
};
use pytxo_core::{ExecutionBackend, PermissionProfile};
use pytxo_runner::owned_launch::{
    run_owned_launch, LaunchCallbacks, OwnedLaunchReceipt, OwnedLaunchSpec, OwnedOutcome,
    OwnedTransport, PinnedFile,
};

pub const CLAUDE_ADAPTER_ID: &str = "pytxo-claude-embedded-host/2";
pub const CLAUDE_SUBSCRIPTION_ENDPOINT: &str = "claude-cli:claude-ai-subscription";
pub const CLAUDE_TOOL_BUNDLE_ID: &str = "claude-read-claimed-file-edit-no-mcp/2";
pub const CLAUDE_PROPOSAL_ADAPTER_ID: &str = "pytxo-claude-one-file-proposal/3";
pub const CLAUDE_PROPOSAL_TOOL_BUNDLE_ID: &str = "claude-no-tools-one-file-json/3";
const CLAUDE_ONE_FILE_JSON_SCHEMA: &str = r#"{"type":"object","properties":{"content":{"type":"string"}},"required":["content"],"additionalProperties":false}"#;

/// Opaque reviewed source for one selected subscription home. This binds the
/// filesystem account selection, not a provider-side account identity; auth
/// remains a fresh observation at qualification.
pub fn claude_subscription_account_source_id(
    account_home: &Path,
) -> anyhow::Result<BillingSourceId> {
    reject_reparse_components(account_home)?;
    let canonical = std::fs::canonicalize(account_home)?;
    if !canonical.is_dir() {
        bail!("selected Claude subscription home is not a directory");
    }
    let path = canonical
        .to_str()
        .context("selected Claude subscription home is not Unicode")?;
    Ok(BillingSourceId(format!(
        "claude-home-sha256:{}",
        Digest::of_bytes(path.to_lowercase().as_bytes()).0
    )))
}

/// The selected local account source. This is a reviewed assertion, not an
/// auth-status or billing receipt.
pub struct ClaudeAccountSource<'a> {
    pub binding_id: &'a BindingId,
    pub billing_source_id: &'a BillingSourceId,
    pub auth_owner: &'a str,
    pub endpoint_identity: &'a str,
    pub capacity_pool_ids: &'a BTreeSet<String>,
    pub account_home: &'a Path,
}

pub struct ClaudeTemplateInput<'a> {
    pub profile: &'a ExecutionProfile,
    pub binding: &'a ProfileBinding,
    pub selected_target: &'a RouteTarget,
    pub account: ClaudeAccountSource<'a>,
    pub claude: &'a PinnedFile,
    pub claude_version: &'a str,
    pub embedded_host: &'a PinnedFile,
    pub embedded_host_version: &'a str,
    pub worktree: &'a Path,
    pub claim_roots: &'a [String],
    pub system_root: &'a Path,
    pub private_prompt: &'a [u8],
    pub permission_profile: PermissionProfile,
    pub barrier_timeout: Duration,
    pub execution_timeout: Duration,
    pub settlement_timeout: Duration,
    pub output_limit: usize,
}

#[derive(Clone)]
pub struct ClaudeLaunchTemplate {
    pub spec: OwnedLaunchSpec,
    pub launch: LaunchContract,
    pub executable: ExecutableIdentity,
    pub profile_digest: Digest,
    pub binding_digest: Digest,
    pub requested_model: ModelIdentity,
    pub model_identity_level: ModelIdentityLevel,
    pub metering_support: MeteringSupport,
}

/// A point-in-time CLI account observation. This does not prove the account
/// will still use subscription billing for a later worker process.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ClaudeSubscriptionAuth {
    pub subscription_type: String,
    pub account_policy_digest: Digest,
    pub executable_digest: Digest,
}

pub struct ClaudeAuthProbeInput<'a> {
    pub claude: &'a PinnedFile,
    pub embedded_host: &'a PinnedFile,
    pub account_home: &'a Path,
    pub system_root: &'a Path,
    pub probe_directory: &'a Path,
}

/// Read-only, no-inference account-check spec using the same owned host and
/// filtered account environment as a routed edit. Call
/// `observe_owned_claude_subscription_auth` to execute and inspect it in one
/// trusted operation; a separately supplied receipt cannot prove its origin.
pub fn claude_subscription_auth_probe(
    input: ClaudeAuthProbeInput<'_>,
) -> anyhow::Result<OwnedLaunchSpec> {
    validate_pin(input.claude, "claude-auth-probe-v1")?;
    validate_pin(input.embedded_host, "pytxo-auth-probe-v1")?;
    if input.claude.path == input.embedded_host.path
        || input
            .claude
            .path
            .file_name()
            .is_none_or(|name| name != "claude.exe")
        || !input.probe_directory.is_absolute()
        || !input.probe_directory.is_dir()
        || !input.account_home.is_absolute()
        || !input.account_home.is_dir()
        || !input.system_root.is_absolute()
        || !input.system_root.is_dir()
    {
        bail!("Claude auth probe paths or executable are invalid");
    }
    let account = std::fs::canonicalize(input.account_home)?;
    let probe = std::fs::canonicalize(input.probe_directory)?;
    reject_reparse_components(input.account_home)?;
    reject_reparse_components(input.probe_directory)?;
    if crate::routed_worker::windows_path_starts_with(&account, &probe)?
        || crate::routed_worker::windows_path_starts_with(&probe, &account)?
    {
        bail!("Claude auth probe cannot run inside the selected account home");
    }
    #[cfg(windows)]
    if !crate::routed_worker::same_windows_ordinal_case(
        input.system_root.as_os_str(),
        std::ffi::OsStr::new(&crate::routed_worker::observed_windows_root()?),
    )? {
        bail!("Claude auth probe Windows root differs from the operating system");
    }
    Ok(OwnedLaunchSpec {
        bootstrap_host: input.embedded_host.clone(),
        executable: input.claude.clone(),
        dependencies: vec![],
        arguments: vec![
            "--restricted".into(),
            "--strict-mcp-config".into(),
            "auth".into(),
            "status".into(),
        ],
        environment: claude_subscription_environment(input.account_home, input.system_root)?,
        working_directory: input.probe_directory.to_path_buf(),
        stdin: vec![],
        transport: OwnedTransport::Subprocess,
        barrier_timeout: Duration::from_secs(10),
        execution_timeout: Duration::from_secs(120),
        settlement_timeout: Duration::from_secs(10),
        output_limit: 8_192,
    })
}

/// Keep the spec and receipt in the same trusted call. An owned receipt does
/// not itself attest its original argv, environment, or cwd, so a caller must
/// never pair a receipt with a separately constructed account spec.
pub fn observe_owned_claude_subscription_auth(
    input: ClaudeAuthProbeInput<'_>,
    callbacks: &mut dyn LaunchCallbacks,
    cancelled: impl Fn() -> pytxo_core::Result<bool>,
) -> anyhow::Result<ClaudeSubscriptionAuth> {
    let spec = claude_subscription_auth_probe(input)?;
    let receipt = run_owned_launch(&spec, callbacks, cancelled)?;
    if receipt.outcome == OwnedOutcome::Cancelled
        && receipt.active_processes == Some(0)
        && receipt.error.is_none()
    {
        return Err(crate::routed_fixture::RoutedPreflightStopped.into());
    }
    inspect_owned_claude_subscription_auth(&spec, &receipt)
}

fn inspect_owned_claude_subscription_auth(
    spec: &OwnedLaunchSpec,
    receipt: &OwnedLaunchReceipt,
) -> anyhow::Result<ClaudeSubscriptionAuth> {
    let account = spec
        .environment
        .get("HOME")
        .context("Claude auth probe has no account home")?;
    let root = spec
        .environment
        .get("SystemRoot")
        .context("Claude auth probe has no Windows root")?;
    if spec.arguments != ["--restricted", "--strict-mcp-config", "auth", "status"]
        || spec.environment.len() != 6
        || spec.environment != claude_subscription_environment(Path::new(account), Path::new(root))?
        || !spec.stdin.is_empty()
        || spec.transport != OwnedTransport::Subprocess
        || spec.output_limit != 8_192
        || !spec.dependencies.is_empty()
        || !spec.working_directory.is_absolute()
        || !spec.working_directory.is_dir()
        || !Path::new(account).is_absolute()
        || !Path::new(account).is_dir()
        || reject_reparse_components(Path::new(account)).is_err()
        || reject_reparse_components(&spec.working_directory).is_err()
        || crate::routed_worker::windows_path_starts_with(
            &std::fs::canonicalize(account)?,
            &std::fs::canonicalize(&spec.working_directory)?,
        )?
        || crate::routed_worker::windows_path_starts_with(
            &std::fs::canonicalize(&spec.working_directory)?,
            &std::fs::canonicalize(account)?,
        )?
        || PinnedFile::observe(spec.bootstrap_host.path.clone())? != spec.bootstrap_host
        || PinnedFile::observe(spec.executable.path.clone())? != spec.executable
        || receipt.outcome != OwnedOutcome::Succeeded
        || receipt.protocol != "pytxo-attempt-host/1"
        || receipt.argument_lowering != "rust-std-command-windows-structured-argv/v1"
        || receipt.chain != [spec.bootstrap_host.clone(), spec.executable.clone()]
        || receipt.bootstrap_sha256 != spec.bootstrap_host.sha256
        || !receipt.process_registered
        || !receipt.barrier_released
        || receipt
            .bootstrap
            .as_ref()
            .is_none_or(|process| process.pid == 0)
        || receipt.active_processes != Some(0)
        || receipt.exit_code != Some(0)
        || receipt.payload_exit_code != Some(0)
        || !receipt.output_complete
        || receipt.output_truncated
        || !receipt.stdout_utf8_valid
        || receipt.error.is_some()
        || receipt.stdout.len() > 8_192
    {
        #[cfg(feature = "routed-test-faults")]
        eprintln!(
            "Claude auth owned receipt: outcome={:?} protocol_ok={} lowering_ok={} chain_ok={} registered={} released={} bootstrap_present={} active={:?} exit={:?} payload_exit={:?} complete={} truncated={} utf8={} error_present={} stdout_len={}",
            receipt.outcome,
            receipt.protocol == "pytxo-attempt-host/1",
            receipt.argument_lowering == "rust-std-command-windows-structured-argv/v1",
            receipt.chain == [spec.bootstrap_host.clone(), spec.executable.clone()],
            receipt.process_registered,
            receipt.barrier_released,
            receipt.bootstrap.is_some(),
            receipt.active_processes,
            receipt.exit_code,
            receipt.payload_exit_code,
            receipt.output_complete,
            receipt.output_truncated,
            receipt.stdout_utf8_valid,
            receipt.error.is_some(),
            receipt.stdout.len(),
        );
        bail!("Claude subscription auth observation is not a complete owned success");
    }
    let status: serde_json::Value = serde_json::from_str(receipt.stdout.trim())
        .context("Claude auth status is not valid JSON")?;
    let subscription_type = status["subscriptionType"]
        .as_str()
        .filter(|value| !value.is_empty() && !value.chars().any(char::is_control))
        .context("Claude auth status has no subscription type")?;
    if status["loggedIn"] != true || status["authMethod"] != "claude.ai" {
        bail!("Claude account is not observed in subscription mode");
    }
    Ok(ClaudeSubscriptionAuth {
        subscription_type: subscription_type.to_owned(),
        account_policy_digest: canonical_digest(&spec.environment, 1)?,
        executable_digest: Digest(spec.executable.sha256.clone()),
    })
}

pub fn build_claude_launch_template(
    input: ClaudeTemplateInput<'_>,
) -> anyhow::Result<ClaudeLaunchTemplate> {
    build_claude_template(
        input,
        CLAUDE_ADAPTER_ID,
        CLAUDE_TOOL_BUNDLE_ID,
        claude_edit_arguments,
    )
}

/// Write-free v3 candidate transport. The route must still qualify this exact
/// command and validate its bounded output after owned Job-zero settlement.
pub fn build_claude_proposal_template(
    input: ClaudeTemplateInput<'_>,
) -> anyhow::Result<ClaudeLaunchTemplate> {
    build_claude_template(
        input,
        CLAUDE_PROPOSAL_ADAPTER_ID,
        CLAUDE_PROPOSAL_TOOL_BUNDLE_ID,
        claude_proposal_arguments,
    )
}

fn build_claude_template(
    input: ClaudeTemplateInput<'_>,
    adapter_id: &str,
    tool_bundle_id: &str,
    argument_builder: fn(&ModelIdentity, &[String]) -> anyhow::Result<Vec<String>>,
) -> anyhow::Result<ClaudeLaunchTemplate> {
    let profile = input.profile;
    let binding = input.binding;
    if profile.schema_version != 1
        || profile.canonicalization_version != 1
        || profile.revision == 0
        || profile.harness_id != "claude"
        || profile.adapter_contract_version != "1"
        || profile.adapter_digest != Digest::of_bytes(adapter_id.as_bytes())
        || profile.skill_tool_bundle_digest != Digest::of_bytes(tool_bundle_id.as_bytes())
        || profile.backend != ExecutionBackend::Subprocess
        || profile.capabilities != BTreeSet::from(["read".into(), "edit".into()])
        || input.permission_profile != PermissionProfile::Orbit
    {
        bail!("unsupported Claude profile, tool bundle, backend, or permission scope");
    }
    let profile_digest = profile.digest()?;
    let binding_digest = binding.digest()?;
    if binding.schema_version != 1
        || binding.canonicalization_version != 1
        || binding.revision == 0
        || input.selected_target.profile_id != profile.id
        || input.selected_target.binding_id != binding.id
        || binding.profile_digest != profile_digest
        || binding.billing_mode != BillingSourceMode::Subscription
        || binding.credential_reference.is_some()
        || binding.auth_owner != "Claude"
        || binding.endpoint_identity != CLAUDE_SUBSCRIPTION_ENDPOINT
        || binding.trust_class != "vendor"
        || binding.billing_source_id.0.is_empty()
        || binding.capacity_pool_ids.len() != 1
        || binding.capacity_pool_ids.iter().any(String::is_empty)
    {
        bail!("Claude selection is not the reviewed subscription binding");
    }
    if input.account.binding_id != &binding.id
        || input.account.billing_source_id != &binding.billing_source_id
        || input.account.auth_owner != binding.auth_owner
        || input.account.endpoint_identity != binding.endpoint_identity
        || input.account.capacity_pool_ids != &binding.capacity_pool_ids
    {
        bail!("Claude account or capacity source differs from the reviewed binding");
    }
    if adapter_id == CLAUDE_PROPOSAL_ADAPTER_ID
        && binding.billing_source_id
            != claude_subscription_account_source_id(input.account.account_home)?
    {
        bail!("Claude proposal subscription home differs from the reviewed billing source");
    }
    if !input.account.account_home.is_absolute()
        || !input.account.account_home.is_dir()
        || !input.worktree.is_absolute()
        || !input.system_root.is_absolute()
        || !input.system_root.is_dir()
    {
        bail!("Claude account and Windows root must exist; attempt worktree must be absolute");
    }
    let canonical_home = std::fs::canonicalize(input.account.account_home)?;
    let canonical_worktree = canonical_existing_or_future_directory(input.worktree)?;
    reject_reparse_components(input.account.account_home)?;
    reject_reparse_components(input.worktree)?;
    if crate::routed_worker::windows_path_starts_with(&canonical_home, &canonical_worktree)?
        || crate::routed_worker::windows_path_starts_with(&canonical_worktree, &canonical_home)?
    {
        bail!("Claude account home cannot overlap the writable attempt worktree");
    }
    if input.private_prompt.is_empty()
        || input.private_prompt.len() > 16_384
        || std::str::from_utf8(input.private_prompt).is_err()
        || input.private_prompt.contains(&0)
    {
        bail!("Claude private prompt is invalid or exceeds the owned-host limit");
    }
    if input.barrier_timeout.is_zero()
        || input.barrier_timeout > Duration::from_secs(30)
        || input.execution_timeout.is_zero()
        || input.execution_timeout > Duration::from_secs(86_400)
        || input.settlement_timeout.is_zero()
        || input.settlement_timeout > Duration::from_secs(60)
        || input.output_limit == 0
        || input.output_limit > 16 * 1024 * 1024
        || (adapter_id == CLAUDE_PROPOSAL_ADAPTER_ID && input.output_limit > 64 * 1024)
    {
        bail!("Claude owned-launch bounds are invalid");
    }
    validate_pin(input.claude, input.claude_version)?;
    validate_pin(input.embedded_host, input.embedded_host_version)?;
    if input.claude.path == input.embedded_host.path
        || input
            .claude
            .path
            .file_name()
            .is_none_or(|name| name != "claude.exe")
    {
        bail!("Claude payload must be a separately pinned claude.exe");
    }
    #[cfg(windows)]
    if !crate::routed_worker::same_windows_ordinal_case(
        input.system_root.as_os_str(),
        std::ffi::OsStr::new(&crate::routed_worker::observed_windows_root()?),
    )? {
        bail!("Claude Windows root differs from the operating system");
    }
    let environment =
        claude_subscription_environment(input.account.account_home, input.system_root)?;
    let arguments = argument_builder(&profile.requested_model, input.claim_roots)?;
    let executable = identity(input.claude, input.claude_version);
    let host = identity(input.embedded_host, input.embedded_host_version);
    let spec = OwnedLaunchSpec {
        bootstrap_host: input.embedded_host.clone(),
        executable: input.claude.clone(),
        dependencies: vec![],
        arguments,
        environment,
        working_directory: input.worktree.to_path_buf(),
        stdin: input.private_prompt.to_vec(),
        transport: OwnedTransport::Subprocess,
        barrier_timeout: input.barrier_timeout,
        execution_timeout: input.execution_timeout,
        settlement_timeout: input.settlement_timeout,
        output_limit: input.output_limit,
    };
    let launch = LaunchContract {
        schema_version: 1,
        transport: LaunchTransport::HostSubprocess,
        host: Some(host),
        dependencies: vec![],
        arguments_digest: canonical_digest(&spec.arguments, 1)?,
        environment_policy_digest: canonical_digest(&spec.environment, 1)?,
        working_directory_policy: "reviewed_attempt_worktree_v1".into(),
        stdin_delivery: StdinDelivery::PrivateHostPipe,
        private_stdin_digest: Some(Digest::of_bytes(&spec.stdin)),
        output_protocol: "pytxo-attempt-host/1".into(),
        argument_lowering: "rust-std-command-windows-structured-argv/v1".into(),
        barrier_timeout_ms: Some(u64::try_from(spec.barrier_timeout.as_millis())?),
        execution_timeout_ms: u64::try_from(spec.execution_timeout.as_millis())?,
        settlement_timeout_ms: u64::try_from(spec.settlement_timeout.as_millis())?,
        output_limit_bytes: u64::try_from(spec.output_limit)?,
    };
    if !launch.valid_for(profile.backend) {
        bail!("Claude launch template does not match the hosted contract");
    }
    Ok(ClaudeLaunchTemplate {
        spec,
        launch,
        executable,
        profile_digest,
        binding_digest,
        requested_model: profile.requested_model.clone(),
        model_identity_level: ModelIdentityLevel::Requested,
        metering_support: MeteringSupport::Unknown,
    })
}

fn claude_subscription_environment(
    account_home: &Path,
    system_root: &Path,
) -> anyhow::Result<BTreeMap<String, String>> {
    let account_home = account_home.to_str().context("non-Unicode Claude home")?;
    let system_root = system_root.to_str().context("non-Unicode Windows root")?;
    if account_home.contains('\0') || system_root.contains('\0') {
        bail!("Claude environment path contains NUL");
    }
    Ok(BTreeMap::from([
        ("HOME".to_owned(), account_home.to_owned()),
        ("USERPROFILE".to_owned(), account_home.to_owned()),
        (
            "APPDATA".to_owned(),
            Path::new(account_home)
                .join("AppData/Roaming")
                .to_string_lossy()
                .into_owned(),
        ),
        (
            "LOCALAPPDATA".to_owned(),
            Path::new(account_home)
                .join("AppData/Local")
                .to_string_lossy()
                .into_owned(),
        ),
        ("SystemRoot".to_owned(), system_root.to_owned()),
        ("WINDIR".to_owned(), system_root.to_owned()),
    ]))
}

/// Restricted, path-scoped command. Native qualification of this exact shape
/// is required before a subscription profile can become Ready.
pub(crate) fn claude_edit_arguments(
    model: &ModelIdentity,
    claim_roots: &[String],
) -> anyhow::Result<Vec<String>> {
    if model.provider != "anthropic"
        || !matches!(model.model.as_str(), "haiku" | "sonnet")
        || model.reasoning.is_some()
        || model.revision.is_some()
    {
        bail!("Claude model request is outside the qualified aliases");
    }
    let [claim_root] = claim_roots else {
        bail!("Claude v1 edit requires exactly one reviewed claimed file");
    };
    if claim_root.is_empty()
        || claim_root.len() > 240
        || !claim_root.is_ascii()
        || claim_root.starts_with('/')
        || claim_root.contains('\\')
        || claim_root.split('/').any(|component| {
            component.is_empty()
                || component == "."
                || component == ".."
                || component.eq_ignore_ascii_case(".git")
                || !component
                    .bytes()
                    .all(|byte| byte.is_ascii_alphanumeric() || b"._-".contains(&byte))
        })
    {
        bail!("Claude claimed file is outside the narrow v1 edit path syntax");
    }
    Ok(vec![
        "-p".into(),
        "--model".into(),
        model.model.clone(),
        "--restricted".into(),
        "--tools".into(),
        "Read,Edit".into(),
        "--permission-mode".into(),
        "dontAsk".into(),
        "--permission-prompts".into(),
        "none".into(),
        "--allowedTools".into(),
        "Read(./**)".into(),
        format!("Edit(./{claim_root})"),
        "--strict-mcp-config".into(),
        "--disallowedTools".into(),
        "mcp__*".into(),
        "--disable-slash-commands".into(),
        "--no-session-persistence".into(),
        "--max-turns".into(),
        "4".into(),
        "--output-format".into(),
        "json".into(),
    ])
}

/// The only v3 built-in tool set is empty. The fixed schema returns one text
/// value; the controller maps it to the single reviewed path only after Job
/// zero and independently verifies the resulting private snapshot.
pub(crate) fn claude_proposal_arguments(
    model: &ModelIdentity,
    claim_roots: &[String],
) -> anyhow::Result<Vec<String>> {
    // Reuse the narrow alias and plain single-file path validation. The v2
    // Edit argv is never returned or launched by this function.
    claude_edit_arguments(model, claim_roots)?;
    Ok(vec![
        "-p".into(),
        "--model".into(),
        model.model.clone(),
        "--restricted".into(),
        "--safe-mode".into(),
        "--tools".into(),
        "".into(),
        "--permission-mode".into(),
        "dontAsk".into(),
        "--permission-prompts".into(),
        "none".into(),
        "--strict-mcp-config".into(),
        "--disallowedTools".into(),
        "mcp__*".into(),
        "--disable-slash-commands".into(),
        "--no-session-persistence".into(),
        "--max-turns".into(),
        "1".into(),
        "--json-schema".into(),
        CLAUDE_ONE_FILE_JSON_SCHEMA.into(),
        "--output-format".into(),
        "json".into(),
    ])
}

/// Parse only a complete, quiescent v3 owned-host result. This yields text,
/// never a filesystem path or Apply authority. The caller must seal it through
/// Core's single-claim snapshot contract and run the frozen checker.
pub fn parse_owned_claude_one_file_proposal(
    template: &ClaudeLaunchTemplate,
    receipt: &OwnedLaunchReceipt,
) -> anyhow::Result<Vec<u8>> {
    let spec = &template.spec;
    if spec.arguments
        != claude_proposal_arguments(&template.requested_model, &["claim.txt".into()])?
        || template.launch.arguments_digest != canonical_digest(&spec.arguments, 1)?
        || template.launch.output_protocol != "pytxo-attempt-host/1"
        || receipt.outcome != OwnedOutcome::Succeeded
        || receipt.protocol != "pytxo-attempt-host/1"
        || receipt.argument_lowering != "rust-std-command-windows-structured-argv/v1"
        || receipt.chain != [spec.bootstrap_host.clone(), spec.executable.clone()]
        || receipt.bootstrap_sha256 != spec.bootstrap_host.sha256
        || !receipt.process_registered
        || !receipt.barrier_released
        || receipt
            .bootstrap
            .as_ref()
            .is_none_or(|process| process.pid == 0)
        || receipt.active_processes != Some(0)
        || receipt.terminated_job
        || receipt.exit_code != Some(0)
        || receipt.payload_exit_code != Some(0)
        || receipt.payload_pid_reported.is_none_or(|pid| pid == 0)
        || !receipt.output_complete
        || receipt.output_truncated
        || !receipt.stdout_utf8_valid
        || receipt.error.is_some()
        || receipt.stdout.len() > spec.output_limit
    {
        bail!("Claude proposal is not a complete owned write-free result");
    }
    let response: serde_json::Value = serde_json::from_str(&receipt.stdout)
        .context("Claude proposal envelope is invalid JSON")?;
    let structured = response["structured_output"]
        .as_object()
        .context("Claude proposal has no structured output")?;
    let content = structured
        .get("content")
        .and_then(serde_json::Value::as_str)
        .context("Claude proposal has no text content")?;
    if response["is_error"] != false
        || response["permission_denials"]
            .as_array()
            .is_none_or(|denials| !denials.is_empty())
        || structured.len() != 1
        || content.len() > 16 * 1024
        || content.contains('\0')
    {
        bail!("Claude proposal is an error, exceeds the text bound, or requested tools");
    }
    Ok(content.as_bytes().to_vec())
}

fn canonical_existing_or_future_directory(path: &Path) -> anyhow::Result<PathBuf> {
    if path.is_dir() {
        return Ok(std::fs::canonicalize(path)?);
    }
    if path.exists() {
        bail!("attempt worktree path exists but is not a directory");
    }
    let parent = path
        .parent()
        .filter(|parent| *parent != path)
        .context("attempt worktree has no existing ancestor")?;
    let leaf = path
        .file_name()
        .context("attempt worktree has no plain path component")?;
    Ok(canonical_existing_or_future_directory(parent)?.join(leaf))
}

/// Reject junctions and symlinks in every existing ancestor. A subscription
/// account or attempt root must not be silently retargeted between readiness
/// and owned launch. The native gate repeats this check immediately before
/// process creation; the launch remains experimental because this is not an
/// atomic filesystem capability.
pub(crate) fn reject_reparse_components(path: &Path) -> anyhow::Result<()> {
    if !path.is_absolute()
        || path.components().any(|component| {
            matches!(
                component,
                std::path::Component::CurDir | std::path::Component::ParentDir
            )
        })
    {
        bail!("Claude account or attempt path must have plain absolute components");
    }
    #[cfg(windows)]
    {
        use std::os::windows::fs::MetadataExt;
        const FILE_ATTRIBUTE_REPARSE_POINT: u32 = 0x400;
        for component in path.ancestors() {
            match std::fs::symlink_metadata(component) {
                Ok(metadata) if metadata.file_attributes() & FILE_ATTRIBUTE_REPARSE_POINT != 0 => {
                    bail!("Claude account or attempt path contains a reparse point");
                }
                Ok(_) => {}
                Err(error) if error.kind() == std::io::ErrorKind::NotFound => {}
                Err(error) => return Err(error.into()),
            }
        }
    }
    #[cfg(not(windows))]
    let _ = path;
    Ok(())
}

/// The native gate checks the materialized attempt tree as well as its path.
/// A tracked link inside an otherwise plain worktree must not redirect an
/// allowed relative Read/Edit to a file outside that worktree.
pub(crate) fn reject_reparse_descendants(root: &Path) -> anyhow::Result<()> {
    reject_reparse_components(root)?;
    if !root.is_dir() {
        bail!("Claude attempt worktree does not exist");
    }
    let mut directories = vec![root.to_path_buf()];
    let mut visited = 0_usize;
    while let Some(directory) = directories.pop() {
        for entry in std::fs::read_dir(directory)? {
            visited += 1;
            if visited > 100_000 {
                bail!("Claude attempt worktree exceeds the link-scan bound");
            }
            let entry = entry?;
            let metadata = std::fs::symlink_metadata(entry.path())?;
            #[cfg(windows)]
            {
                use std::os::windows::fs::MetadataExt;
                if metadata.file_attributes() & 0x400 != 0 {
                    bail!("Claude attempt worktree contains a reparse point");
                }
            }
            #[cfg(not(windows))]
            if metadata.file_type().is_symlink() {
                bail!("Claude attempt worktree contains a symbolic link");
            }
            if metadata.is_dir() {
                directories.push(entry.path());
            }
        }
    }
    Ok(())
}

fn validate_pin(pin: &PinnedFile, version: &str) -> anyhow::Result<()> {
    if !pin.path.is_absolute()
        || pin
            .path
            .extension()
            .is_none_or(|extension| !extension.eq_ignore_ascii_case("exe"))
        || pin.path.to_str().is_none()
        || !Digest(pin.sha256.clone()).is_valid()
        || version.is_empty()
        || version.chars().any(char::is_control)
    {
        bail!("Claude executable or embedded-host pin is invalid");
    }
    Ok(())
}

fn identity(pin: &PinnedFile, version: &str) -> ExecutableIdentity {
    ExecutableIdentity {
        path: pin.path.to_string_lossy().into_owned(),
        version: version.to_owned(),
        digest: Digest(pin.sha256.clone()),
    }
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeSet;
    use std::path::PathBuf;
    use std::time::Duration;

    use pytxo_core::routing::{
        BillingSourceId, BillingSourceMode, BindingId, Digest, ExecutionProfile, ModelIdentity,
        ProfileBinding, ProfileId, RouteTarget,
    };
    use pytxo_core::{ExecutionBackend, PermissionProfile};
    use pytxo_runner::owned_launch::{
        LaunchIntent, OwnedLaunchReceipt, OwnedOutcome, OwnedProcess, PinnedFile,
    };

    use super::*;

    struct Fixture {
        root: tempfile::TempDir,
        profile: ExecutionProfile,
        binding: ProfileBinding,
        target: RouteTarget,
        cli: PinnedFile,
        host: PinnedFile,
        account_home: PathBuf,
        worktree: PathBuf,
        claim_roots: Vec<String>,
        system_root: PathBuf,
    }

    impl Fixture {
        fn new() -> Self {
            let root = tempfile::tempdir().unwrap();
            for name in ["account", "worktree", "windows"] {
                std::fs::create_dir(root.path().join(name)).unwrap();
            }
            let cli_path = root.path().join("claude.exe");
            let host_path = root.path().join("pytxo-embedded.exe");
            std::fs::write(&cli_path, b"claude pin").unwrap();
            std::fs::write(&host_path, b"host pin").unwrap();
            let profile = ExecutionProfile {
                schema_version: 1,
                canonicalization_version: 1,
                id: ProfileId("everyday".into()),
                revision: 1,
                harness_id: "claude".into(),
                adapter_contract_version: "1".into(),
                adapter_digest: Digest::of_bytes(CLAUDE_ADAPTER_ID.as_bytes()),
                requested_model: ModelIdentity {
                    provider: "anthropic".into(),
                    model: "haiku".into(),
                    reasoning: None,
                    revision: None,
                },
                skill_tool_bundle_digest: Digest::of_bytes(CLAUDE_TOOL_BUNDLE_ID.as_bytes()),
                backend: ExecutionBackend::Subprocess,
                capabilities: BTreeSet::from(["read".into(), "edit".into()]),
            };
            let binding = ProfileBinding {
                schema_version: 1,
                canonicalization_version: 1,
                id: BindingId("claude-subscription".into()),
                revision: 1,
                profile_digest: profile.digest().unwrap(),
                credential_reference: None,
                auth_owner: "Claude".into(),
                billing_source_id: claude_subscription_account_source_id(
                    &root.path().join("account"),
                )
                .unwrap(),
                billing_mode: BillingSourceMode::Subscription,
                endpoint_identity: CLAUDE_SUBSCRIPTION_ENDPOINT.into(),
                trust_class: "vendor".into(),
                capacity_pool_ids: BTreeSet::from(["account-claude".into()]),
            };
            let target = RouteTarget {
                profile_id: profile.id.clone(),
                binding_id: binding.id.clone(),
            };
            Self {
                cli: PinnedFile::observe(cli_path).unwrap(),
                host: PinnedFile::observe(host_path).unwrap(),
                account_home: root.path().join("account"),
                worktree: root.path().join("worktree"),
                claim_roots: vec!["result.txt".into()],
                system_root: if cfg!(windows) {
                    PathBuf::from(crate::routed_worker::observed_windows_root().unwrap())
                } else {
                    root.path().join("windows")
                },
                root,
                profile,
                binding,
                target,
            }
        }

        fn input<'a>(&'a self, prompt: &'a [u8]) -> ClaudeTemplateInput<'a> {
            ClaudeTemplateInput {
                profile: &self.profile,
                binding: &self.binding,
                selected_target: &self.target,
                account: ClaudeAccountSource {
                    binding_id: &self.binding.id,
                    billing_source_id: &self.binding.billing_source_id,
                    auth_owner: &self.binding.auth_owner,
                    endpoint_identity: &self.binding.endpoint_identity,
                    capacity_pool_ids: &self.binding.capacity_pool_ids,
                    account_home: &self.account_home,
                },
                claude: &self.cli,
                claude_version: "claude-test-v1",
                embedded_host: &self.host,
                embedded_host_version: "pytxo-test-v1",
                worktree: &self.worktree,
                claim_roots: &self.claim_roots,
                system_root: &self.system_root,
                private_prompt: prompt,
                permission_profile: PermissionProfile::Orbit,
                barrier_timeout: Duration::from_secs(10),
                execution_timeout: Duration::from_secs(180),
                settlement_timeout: Duration::from_secs(10),
                output_limit: 32 * 1024,
            }
        }
    }

    #[test]
    fn subscription_launch_is_exact_and_keeps_prompt_private() {
        let fixture = Fixture::new();
        let prompt = b"edit the reviewed file only";
        let template = build_claude_launch_template(fixture.input(prompt)).unwrap();
        assert_eq!(
            template.spec.arguments,
            claude_edit_arguments(&fixture.profile.requested_model, &["result.txt".into()])
                .unwrap()
        );
        assert!(template.spec.arguments.contains(&"--restricted".to_owned()));
        assert!(template.spec.arguments.contains(&"dontAsk".to_owned()));
        let deny_mcp = template
            .spec
            .arguments
            .iter()
            .position(|argument| argument == "--disallowedTools")
            .unwrap();
        assert_eq!(template.spec.arguments.get(deny_mcp + 1).unwrap(), "mcp__*");
        assert!(template
            .spec
            .arguments
            .contains(&"Edit(./result.txt)".to_owned()));
        assert!(!template
            .spec
            .arguments
            .iter()
            .any(|argument| argument == "Edit"));
        assert_eq!(template.spec.stdin, prompt);
        assert_eq!(template.spec.environment.len(), 6);
        assert!(!template.spec.environment.contains_key("ANTHROPIC_API_KEY"));
        assert_eq!(
            template.launch.private_stdin_digest,
            Some(Digest::of_bytes(prompt))
        );
        assert!(template.launch.valid_for(ExecutionBackend::Subprocess));
        assert_eq!(
            template.model_identity_level,
            pytxo_core::routing::ModelIdentityLevel::Requested
        );
        assert!(!format!("{:?}", template.spec).contains("edit the reviewed"));
    }

    #[test]
    fn billing_account_model_and_scope_mismatches_fail_closed() {
        for mode in [
            BillingSourceMode::Api,
            BillingSourceMode::Managed,
            BillingSourceMode::Local,
        ] {
            let mut fixture = Fixture::new();
            fixture.binding.billing_mode = mode;
            assert!(build_claude_launch_template(fixture.input(b"task")).is_err());
        }
        let mut fixture = Fixture::new();
        fixture.binding.credential_reference = Some("api-secret".into());
        assert!(build_claude_launch_template(fixture.input(b"task")).is_err());
        let mut fixture = Fixture::new();
        fixture.profile.requested_model.model = "unexpected".into();
        fixture.binding.profile_digest = fixture.profile.digest().unwrap();
        assert!(build_claude_launch_template(fixture.input(b"task")).is_err());
        let mut fixture = Fixture::new();
        fixture.account_home = fixture.worktree.clone();
        assert!(build_claude_launch_template(fixture.input(b"task")).is_err());
        let fixture = Fixture::new();
        let mut input = fixture.input(b"task");
        input.permission_profile = PermissionProfile::Supernova;
        assert!(build_claude_launch_template(input).is_err());
        let fixture = Fixture::new();
        assert!(build_claude_launch_template(fixture.input(b"")).is_err());
        let fixture = Fixture::new();
        let mut input = fixture.input(b"task");
        let forbidden = [".git/config".into()];
        input.claim_roots = &forbidden;
        assert!(build_claude_launch_template(input).is_err());
        let fixture = Fixture::new();
        let mut input = fixture.input(b"task");
        let multiple = ["result.txt".into(), "other.txt".into()];
        input.claim_roots = &multiple;
        assert!(build_claude_launch_template(input).is_err());
        let mut fixture = Fixture::new();
        let renamed = fixture.root.path().join("not-claude.exe");
        std::fs::write(&renamed, b"renamed CLI pin").unwrap();
        fixture.cli = PinnedFile::observe(renamed).unwrap();
        assert!(build_claude_launch_template(fixture.input(b"task")).is_err());
        #[cfg(windows)]
        {
            let mut fixture = Fixture::new();
            fixture.system_root = fixture.root.path().join("windows");
            assert!(build_claude_launch_template(fixture.input(b"task")).is_err());
        }
    }

    #[test]
    fn future_worktree_is_bound_without_creating_it() {
        let mut fixture = Fixture::new();
        fixture.worktree = fixture.root.path().join("future").join("attempt");
        let template = build_claude_launch_template(fixture.input(b"task")).unwrap();
        assert_eq!(template.spec.working_directory, fixture.worktree);
        assert!(!fixture.worktree.exists());
        fixture.worktree = fixture.account_home.join("attempt");
        assert!(build_claude_launch_template(fixture.input(b"task")).is_err());
    }

    #[test]
    fn account_and_worktree_paths_reject_parent_traversal() {
        let mut fixture = Fixture::new();
        fixture.account_home = fixture.root.path().join("account/../account");
        assert!(build_claude_launch_template(fixture.input(b"task")).is_err());
        let mut fixture = Fixture::new();
        fixture.worktree = fixture.root.path().join("worktree/../worktree");
        assert!(build_claude_launch_template(fixture.input(b"task")).is_err());
    }

    #[cfg(windows)]
    #[test]
    fn account_and_worktree_paths_reject_reparse_components() {
        use std::os::windows::fs::symlink_dir;
        let mut fixture = Fixture::new();
        let account_link = fixture.root.path().join("account-link");
        if let Err(error) = symlink_dir(&fixture.account_home, &account_link) {
            if error.kind() == std::io::ErrorKind::PermissionDenied {
                return;
            }
            panic!("cannot create the disposable account symlink: {error}");
        }
        fixture.account_home = account_link;
        assert!(build_claude_launch_template(fixture.input(b"task")).is_err());
        let mut fixture = Fixture::new();
        let worktree_link = fixture.root.path().join("worktree-link");
        symlink_dir(&fixture.worktree, &worktree_link)
            .expect("cannot create the disposable worktree symlink");
        fixture.worktree = worktree_link;
        assert!(build_claude_launch_template(fixture.input(b"task")).is_err());
        let fixture = Fixture::new();
        let outside = fixture.root.path().join("outside.txt");
        std::fs::write(&outside, b"outside").unwrap();
        std::os::windows::fs::symlink_file(&outside, fixture.worktree.join("linked.txt"))
            .expect("cannot create disposable linked file");
        assert!(reject_reparse_descendants(&fixture.worktree).is_err());
    }

    fn auth_receipt(spec: &OwnedLaunchSpec, stdout: &str) -> OwnedLaunchReceipt {
        OwnedLaunchReceipt {
            outcome: OwnedOutcome::Succeeded,
            intent: LaunchIntent {
                job_name: "Local\\PytxoAttempt-test".into(),
                launch_nonce: "test".into(),
            },
            bootstrap: Some(OwnedProcess {
                pid: 1,
                start_identity: Some("windows-filetime:1".into()),
                job_name: "Local\\PytxoAttempt-test".into(),
            }),
            process_registered: true,
            barrier_released: true,
            payload_pid_reported: Some(1),
            payload_exit_code: Some(0),
            observed_job_pids: vec![],
            active_processes: Some(0),
            terminated_job: false,
            exit_code: Some(0),
            stdout: stdout.into(),
            stderr: String::new(),
            stdout_utf8_valid: true,
            output_complete: true,
            output_truncated: false,
            observed_model: None,
            observed_usage: None,
            chain: vec![spec.bootstrap_host.clone(), spec.executable.clone()],
            bootstrap_sha256: spec.bootstrap_host.sha256.clone(),
            protocol: "pytxo-attempt-host/1".into(),
            bundle_sha256: "0".repeat(64),
            argument_lowering: "rust-std-command-windows-structured-argv/v1".into(),
            elapsed_ms: 1,
            error: None,
        }
    }

    #[test]
    fn write_free_proposal_requires_new_identity_and_complete_owned_text() {
        let mut fixture = Fixture::new();
        assert!(build_claude_proposal_template(fixture.input(b"task")).is_err());
        fixture.profile.adapter_digest = Digest::of_bytes(CLAUDE_PROPOSAL_ADAPTER_ID.as_bytes());
        fixture.profile.skill_tool_bundle_digest =
            Digest::of_bytes(CLAUDE_PROPOSAL_TOOL_BUNDLE_ID.as_bytes());
        fixture.binding.profile_digest = fixture.profile.digest().unwrap();
        let template = build_claude_proposal_template(fixture.input(b"task")).unwrap();
        assert!(build_claude_launch_template(fixture.input(b"task")).is_err());
        assert_eq!(
            template.spec.arguments,
            claude_proposal_arguments(&fixture.profile.requested_model, &fixture.claim_roots)
                .unwrap()
        );
        assert!(template
            .spec
            .arguments
            .windows(2)
            .any(|pair| pair == ["--tools", ""]));
        assert!(!template
            .spec
            .arguments
            .iter()
            .any(|arg| arg.starts_with("Edit(")));
        let valid = r#"{"is_error":false,"permission_denials":[],"structured_output":{"content":"changed\n"}}"#;
        let receipt = auth_receipt(&template.spec, valid);
        assert_eq!(
            parse_owned_claude_one_file_proposal(&template, &receipt).unwrap(),
            b"changed\n"
        );
        for invalid in [
            r#"{"is_error":true,"structured_output":{"content":"changed"}}"#,
            r#"{"is_error":false,"structured_output":{"content":"changed","path":"outside"}}"#,
            r#"{"is_error":false,"structured_output":{"content":8}}"#,
            r#"{"is_error":false,"permission_denials":[{"tool_name":"Edit"}],"structured_output":{"content":"changed"}}"#,
            r#"{"is_error":false,"structured_output":{"content":"nul\u0000byte"}}"#,
            r#"{"is_error":false,"structured_output":{"content":"changed"}}"#,
            "{} trailing",
        ] {
            assert!(parse_owned_claude_one_file_proposal(
                &template,
                &auth_receipt(&template.spec, invalid)
            )
            .is_err());
        }
        let valid_json_file = r#"{"is_error":false,"permission_denials":[],"structured_output":{"content":"{\"claim_root\":\"result.txt\",\"replacement_text\":\"valid file contents\"}"}}"#;
        assert!(parse_owned_claude_one_file_proposal(
            &template,
            &auth_receipt(&template.spec, valid_json_file)
        )
        .is_ok());
        let mut incomplete = receipt.clone();
        incomplete.output_complete = false;
        assert!(parse_owned_claude_one_file_proposal(&template, &incomplete).is_err());
        let mut lossy = receipt.clone();
        lossy.stdout_utf8_valid = false;
        assert!(parse_owned_claude_one_file_proposal(&template, &lossy).is_err());
        let mut truncated = receipt.clone();
        truncated.output_truncated = true;
        assert!(parse_owned_claude_one_file_proposal(&template, &truncated).is_err());
        let mut running = receipt.clone();
        running.active_processes = Some(1);
        assert!(parse_owned_claude_one_file_proposal(&template, &running).is_err());
        let mut cancelled = receipt.clone();
        cancelled.outcome = OwnedOutcome::Cancelled;
        assert!(parse_owned_claude_one_file_proposal(&template, &cancelled).is_err());
        fixture.binding.billing_source_id = BillingSourceId("a-different-account-home".into());
        assert!(build_claude_proposal_template(fixture.input(b"task")).is_err());
    }

    #[test]
    fn only_complete_owned_subscription_status_is_accepted() {
        let fixture = Fixture::new();
        let spec = claude_subscription_auth_probe(ClaudeAuthProbeInput {
            claude: &fixture.cli,
            embedded_host: &fixture.host,
            account_home: &fixture.account_home,
            system_root: &fixture.system_root,
            probe_directory: &fixture.worktree,
        })
        .unwrap();
        let valid = r#"{"loggedIn":true,"authMethod":"claude.ai","subscriptionType":"pro"}"#;
        let observed =
            inspect_owned_claude_subscription_auth(&spec, &auth_receipt(&spec, valid)).unwrap();
        assert_eq!(observed.subscription_type, "pro");
        assert_eq!(
            observed.executable_digest,
            Digest(spec.executable.sha256.clone())
        );
        assert_eq!(
            observed.account_policy_digest,
            canonical_digest(&spec.environment, 1).unwrap()
        );
        for invalid in [
            r#"{"loggedIn":true,"authMethod":"api_key","subscriptionType":"pro"}"#,
            r#"{"loggedIn":false,"authMethod":"claude.ai","subscriptionType":"pro"}"#,
            r#"{"loggedIn":true,"authMethod":"claude.ai"}"#,
            "not-json",
        ] {
            assert!(
                inspect_owned_claude_subscription_auth(&spec, &auth_receipt(&spec, invalid))
                    .is_err()
            );
        }
        let mut incomplete = auth_receipt(&spec, valid);
        incomplete.active_processes = Some(1);
        assert!(inspect_owned_claude_subscription_auth(&spec, &incomplete).is_err());
        let mut truncated = auth_receipt(&spec, valid);
        truncated.output_truncated = true;
        assert!(inspect_owned_claude_subscription_auth(&spec, &truncated).is_err());
        let mut other_account = spec.clone();
        other_account
            .environment
            .insert("HOME".into(), "other".into());
        assert!(inspect_owned_claude_subscription_auth(
            &other_account,
            &auth_receipt(&spec, valid)
        )
        .is_err());
        let mut other_chain = auth_receipt(&spec, valid);
        other_chain.chain.clear();
        assert!(inspect_owned_claude_subscription_auth(&spec, &other_chain).is_err());
    }

    #[test]
    fn auth_probe_uses_same_filtered_account_and_owned_host() {
        let fixture = Fixture::new();
        let probe = claude_subscription_auth_probe(ClaudeAuthProbeInput {
            claude: &fixture.cli,
            embedded_host: &fixture.host,
            account_home: &fixture.account_home,
            system_root: &fixture.system_root,
            probe_directory: &fixture.worktree,
        })
        .unwrap();
        let edit = build_claude_launch_template(fixture.input(b"task")).unwrap();
        assert_eq!(probe.environment, edit.spec.environment);
        assert_eq!(
            probe.arguments,
            ["--restricted", "--strict-mcp-config", "auth", "status"]
        );
        assert!(probe.stdin.is_empty());
        assert_eq!(probe.output_limit, 8_192);
        assert!(claude_subscription_auth_probe(ClaudeAuthProbeInput {
            claude: &fixture.cli,
            embedded_host: &fixture.host,
            account_home: &fixture.account_home,
            system_root: &fixture.system_root,
            probe_directory: &fixture.account_home,
        })
        .is_err());
    }

    /// Explicit local qualification of the production builder, selected
    /// subscription account, and MSI-embedded host. Routine CI never calls a
    /// provider or treats this result as durable readiness.
    #[cfg(windows)]
    #[test]
    #[ignore = "requires selected Claude subscription account, embedded host, and live provider access"]
    fn native_subscription_builder_edits_only_disposable_worktree() {
        native_subscription_builder_probe(NativeProbeKind::Claimed);
    }

    /// A stopped subscription worker must close its native Job and leave the
    /// claimed file untouched. This is a live, explicitly selected-account
    /// qualification probe, not a routine CI test or a durable Ready receipt.
    #[cfg(windows)]
    #[test]
    #[ignore = "requires selected Claude subscription account, embedded host, and live provider access"]
    fn native_subscription_builder_cancellation_settles_job() {
        use std::sync::{Arc, Mutex};
        use std::time::{Duration, Instant};

        use pytxo_core::Result;
        use pytxo_runner::owned_launch::{
            run_owned_launch, LaunchCallbacks, LaunchGuard, LaunchIntent, OwnedProcess,
        };

        struct Callbacks {
            registered_at: Arc<Mutex<Option<Instant>>>,
            settled: bool,
        }
        struct Guard<'a>(&'a mut Callbacks);
        impl LaunchGuard for Guard<'_> {
            fn permit_create(&mut self, _: &LaunchIntent) -> Result<()> {
                Ok(())
            }
            fn register(&mut self, process: &OwnedProcess) -> Result<()> {
                assert!(process.pid > 0);
                *self.0.registered_at.lock().unwrap() = Some(Instant::now());
                Ok(())
            }
            fn cancelled(&mut self) -> Result<bool> {
                Ok(false)
            }
        }
        impl LaunchCallbacks for Callbacks {
            fn authorize(&mut self, _: &LaunchIntent) -> Result<Box<dyn LaunchGuard + '_>> {
                Ok(Box::new(Guard(self)))
            }
            fn settle(&mut self, receipt: &OwnedLaunchReceipt) -> Result<()> {
                assert_eq!(receipt.active_processes, Some(0));
                self.settled = true;
                Ok(())
            }
        }

        let mut fixture = Fixture::new();
        fixture.cli = PinnedFile::observe(PathBuf::from(
            std::env::var_os("PYTXO_TEST_CLAUDE_EXE").expect("selected Claude CLI"),
        ))
        .unwrap();
        fixture.host = PinnedFile::observe(PathBuf::from(
            std::env::var_os("PYTXO_TEST_EMBEDDED_HOST").expect("MSI-extracted embedded host"),
        ))
        .unwrap();
        fixture.account_home = PathBuf::from(
            std::env::var_os("PYTXO_TEST_CLAUDE_HOME").expect("selected subscription home"),
        );
        let model = std::env::var("PYTXO_TEST_CLAUDE_MODEL").expect("haiku or sonnet");
        assert!(matches!(model.as_str(), "haiku" | "sonnet"));
        fixture.profile.requested_model.model = model;
        fixture.binding.profile_digest = fixture.profile.digest().unwrap();
        assert!(std::process::Command::new("git")
            .args(["init", "--quiet"])
            .current_dir(&fixture.worktree)
            .status()
            .unwrap()
            .success());
        std::fs::write(fixture.worktree.join("result.txt"), b"old\n").unwrap();
        let template = build_claude_launch_template(fixture.input(
            b"Read result.txt. Before editing, think through a detailed 20-step plan. Then use Edit to replace old with changed. Do not use shell commands.",
        ))
        .unwrap();
        let registered_at = Arc::new(Mutex::new(None::<Instant>));
        let mut callbacks = Callbacks {
            registered_at: Arc::clone(&registered_at),
            settled: false,
        };
        let receipt = run_owned_launch(&template.spec, &mut callbacks, || {
            Ok(registered_at
                .lock()
                .unwrap()
                .is_some_and(|at| at.elapsed() >= Duration::from_millis(100)))
        })
        .unwrap();
        assert_eq!(receipt.outcome, OwnedOutcome::Cancelled);
        assert!(receipt.process_registered && receipt.barrier_released);
        assert_eq!(receipt.active_processes, Some(0));
        assert_eq!(
            std::fs::read(fixture.worktree.join("result.txt")).unwrap(),
            b"old\n"
        );
        assert!(callbacks.settled);
    }

    /// The link points only to a disposable sibling file. This must record an
    /// actual denied Edit; merely leaving the file untouched is insufficient.
    #[cfg(windows)]
    #[test]
    #[ignore = "beta gate: selected Claude has not reported a denied outside Edit under the exact launch"]
    fn native_subscription_builder_denies_linked_outside_edit() {
        native_subscription_builder_probe(NativeProbeKind::LinkedOutside);
    }

    #[cfg(windows)]
    #[test]
    #[ignore = "beta gate: requires a denied outside Edit from the selected Claude subscription CLI"]
    fn native_subscription_builder_denies_direct_outside_edit() {
        native_subscription_builder_probe(NativeProbeKind::DirectOutside);
    }

    /// Read is allowed in the worktree, but Edit must still be denied for a
    /// sibling file that is absent from the reviewed claim roots.
    #[cfg(windows)]
    #[test]
    #[ignore = "requires selected Claude subscription account, embedded host, and live provider access"]
    fn native_subscription_builder_denies_unclaimed_sibling_edit() {
        native_subscription_builder_probe(NativeProbeKind::UnclaimedSibling);
    }

    /// Explore a write-free worker shape: Claude proposes exact content and
    /// Core would own the eventual claimed-file write. This is a native
    /// feasibility probe, not an admitted routed adapter or beta qualification.
    #[cfg(windows)]
    #[test]
    #[ignore = "requires selected Claude subscription account, embedded host, and live provider access"]
    fn native_subscription_toolless_one_file_proposal_probe() {
        use pytxo_core::Result;
        use pytxo_runner::owned_launch::{
            run_owned_launch, LaunchCallbacks, LaunchGuard, LaunchIntent, OwnedProcess,
        };

        struct Callbacks {
            registered: bool,
            settled: bool,
        }
        struct Guard<'a>(&'a mut Callbacks);
        impl LaunchGuard for Guard<'_> {
            fn permit_create(&mut self, _: &LaunchIntent) -> Result<()> {
                Ok(())
            }
            fn register(&mut self, process: &OwnedProcess) -> Result<()> {
                assert!(process.pid > 0);
                self.0.registered = true;
                Ok(())
            }
            fn cancelled(&mut self) -> Result<bool> {
                Ok(false)
            }
        }
        impl LaunchCallbacks for Callbacks {
            fn authorize(&mut self, _: &LaunchIntent) -> Result<Box<dyn LaunchGuard + '_>> {
                Ok(Box::new(Guard(self)))
            }
            fn settle(&mut self, receipt: &OwnedLaunchReceipt) -> Result<()> {
                assert_eq!(receipt.active_processes, Some(0));
                self.settled = true;
                Ok(())
            }
        }

        let mut fixture = Fixture::new();
        fixture.cli = PinnedFile::observe(PathBuf::from(
            std::env::var_os("PYTXO_TEST_CLAUDE_EXE").expect("selected Claude CLI"),
        ))
        .unwrap();
        fixture.host = PinnedFile::observe(PathBuf::from(
            std::env::var_os("PYTXO_TEST_EMBEDDED_HOST").expect("embedded host"),
        ))
        .unwrap();
        fixture.account_home = PathBuf::from(
            std::env::var_os("PYTXO_TEST_CLAUDE_HOME").expect("selected subscription home"),
        );
        let model = std::env::var("PYTXO_TEST_CLAUDE_MODEL").expect("haiku or sonnet");
        assert!(matches!(model.as_str(), "haiku" | "sonnet"));
        fixture.profile.requested_model.model = model;
        fixture.profile.adapter_digest = Digest::of_bytes(CLAUDE_PROPOSAL_ADAPTER_ID.as_bytes());
        fixture.profile.skill_tool_bundle_digest =
            Digest::of_bytes(CLAUDE_PROPOSAL_TOOL_BUNDLE_ID.as_bytes());
        fixture.binding.profile_digest = fixture.profile.digest().unwrap();
        std::fs::write(fixture.worktree.join("result.txt"), b"old\n").unwrap();
        std::fs::write(fixture.worktree.join("sibling.txt"), b"untouched\n").unwrap();
        let prompt = crate::routed_prompt::render_claude_proposal_payload(
            b"Replace the old line with changed and preserve its trailing newline.".to_vec(),
            "result.txt",
            "old\n",
        )
        .unwrap();
        let template = build_claude_proposal_template(fixture.input(&prompt)).unwrap();
        let spec = template.spec.clone();
        assert_eq!(
            spec.arguments,
            claude_proposal_arguments(&fixture.profile.requested_model, &fixture.claim_roots)
                .unwrap()
        );
        let mut callbacks = Callbacks {
            registered: false,
            settled: false,
        };
        let receipt = run_owned_launch(&spec, &mut callbacks, || Ok(false)).unwrap();
        assert_eq!(receipt.outcome, OwnedOutcome::Succeeded);
        assert!(receipt.output_complete && !receipt.output_truncated);
        assert!(callbacks.registered && callbacks.settled);
        assert_eq!(
            parse_owned_claude_one_file_proposal(&template, &receipt).unwrap(),
            b"changed\n"
        );
        assert_eq!(
            std::fs::read(fixture.worktree.join("result.txt")).unwrap(),
            b"old\n"
        );
        assert_eq!(
            std::fs::read(fixture.worktree.join("sibling.txt")).unwrap(),
            b"untouched\n"
        );
    }

    #[cfg(windows)]
    #[derive(Clone, Copy, PartialEq, Eq)]
    enum NativeProbeKind {
        Claimed,
        LinkedOutside,
        DirectOutside,
        UnclaimedSibling,
    }

    #[cfg(windows)]
    fn native_subscription_builder_probe(kind: NativeProbeKind) {
        use pytxo_core::Result;
        use pytxo_runner::owned_launch::{
            run_owned_launch, LaunchCallbacks, LaunchGuard, LaunchIntent, OwnedProcess,
        };

        struct Callbacks {
            registered: bool,
            settled: bool,
        }
        struct Guard<'a>(&'a mut Callbacks);
        impl LaunchGuard for Guard<'_> {
            fn permit_create(&mut self, _: &LaunchIntent) -> Result<()> {
                Ok(())
            }
            fn register(&mut self, process: &OwnedProcess) -> Result<()> {
                assert!(process.pid > 0);
                self.0.registered = true;
                Ok(())
            }
            fn cancelled(&mut self) -> Result<bool> {
                Ok(false)
            }
        }
        impl LaunchCallbacks for Callbacks {
            fn authorize(&mut self, _: &LaunchIntent) -> Result<Box<dyn LaunchGuard + '_>> {
                Ok(Box::new(Guard(self)))
            }
            fn settle(&mut self, receipt: &OwnedLaunchReceipt) -> Result<()> {
                assert_eq!(receipt.active_processes, Some(0));
                self.settled = true;
                Ok(())
            }
        }

        let mut fixture = Fixture::new();
        fixture.cli = PinnedFile::observe(PathBuf::from(
            std::env::var_os("PYTXO_TEST_CLAUDE_EXE").expect("selected Claude CLI"),
        ))
        .unwrap();
        fixture.host = PinnedFile::observe(PathBuf::from(
            std::env::var_os("PYTXO_TEST_EMBEDDED_HOST").expect("MSI-extracted embedded host"),
        ))
        .unwrap();
        fixture.account_home = PathBuf::from(
            std::env::var_os("PYTXO_TEST_CLAUDE_HOME").expect("selected subscription home"),
        );
        let model = std::env::var("PYTXO_TEST_CLAUDE_MODEL").expect("haiku or sonnet");
        assert!(matches!(model.as_str(), "haiku" | "sonnet"));
        fixture.profile.requested_model.model = model;
        fixture.binding.profile_digest = fixture.profile.digest().unwrap();
        assert!(std::process::Command::new("git")
            .args(["init", "--quiet"])
            .current_dir(&fixture.worktree)
            .status()
            .unwrap()
            .success());
        std::fs::write(fixture.worktree.join("result.txt"), b"old\n").unwrap();
        std::fs::write(fixture.worktree.join("starter.txt"), b"unchanged\n").unwrap();
        let outside = fixture.root.path().join("outside.txt");
        let prompt = match kind {
            NativeProbeKind::Claimed => b"Use the Edit tool to replace the sole line in result.txt with exactly pytxo-production-builder. Keep one trailing newline. Do not edit starter.txt or any other file. Do not use shell commands.".to_vec(),
            NativeProbeKind::LinkedOutside => {
                fixture.claim_roots = vec!["linked.txt".into()];
                std::fs::write(&outside, b"outside-before\n").unwrap();
                std::os::windows::fs::symlink_file(&outside, fixture.worktree.join("linked.txt"))
                    .expect("cannot make disposable out-of-worktree link");
                b"Use the Edit tool directly on linked.txt. Replace its sole line outside-before with outside-after, keeping one trailing newline. Do not Read first. Do not use Bash or any other tool. Do not edit result.txt or starter.txt.".to_vec()
            }
            NativeProbeKind::DirectOutside => {
                std::fs::write(&outside, b"outside-before\n").unwrap();
                b"Use the Edit tool directly on ../outside.txt. Replace its sole line outside-before with outside-after, keeping one trailing newline. This path is a sibling of the current worktree. Do not Read first. Do not use Bash or any other tool. Do not edit result.txt or starter.txt.".to_vec()
            }
            NativeProbeKind::UnclaimedSibling => {
                std::fs::write(fixture.worktree.join("unclaimed.txt"), b"sibling-before\n")
                    .unwrap();
                b"Read unclaimed.txt, then use the Edit tool to replace sibling-before with sibling-after in unclaimed.txt. Keep one trailing newline. Do not edit result.txt or starter.txt. Do not use shell commands.".to_vec()
            }
        };
        let template = build_claude_launch_template(fixture.input(&prompt)).unwrap();
        assert_eq!(
            template.spec.arguments,
            claude_edit_arguments(&fixture.profile.requested_model, &fixture.claim_roots).unwrap()
        );
        if kind == NativeProbeKind::Claimed {
            let auth_spec = claude_subscription_auth_probe(ClaudeAuthProbeInput {
                claude: &fixture.cli,
                embedded_host: &fixture.host,
                account_home: &fixture.account_home,
                system_root: &fixture.system_root,
                probe_directory: &fixture.worktree,
            })
            .unwrap();
            assert_eq!(auth_spec.environment, template.spec.environment);
            let mut callbacks = Callbacks {
                registered: false,
                settled: false,
            };
            let auth = observe_owned_claude_subscription_auth(
                ClaudeAuthProbeInput {
                    claude: &fixture.cli,
                    embedded_host: &fixture.host,
                    account_home: &fixture.account_home,
                    system_root: &fixture.system_root,
                    probe_directory: &fixture.worktree,
                },
                &mut callbacks,
                || Ok(false),
            )
            .unwrap();
            assert!(!auth.subscription_type.is_empty());
            assert_eq!(
                auth.account_policy_digest,
                template.launch.environment_policy_digest
            );
            assert!(callbacks.registered && callbacks.settled);
        }
        let mut callbacks = Callbacks {
            registered: false,
            settled: false,
        };
        let receipt = run_owned_launch(&template.spec, &mut callbacks, || Ok(false)).unwrap();
        if kind == NativeProbeKind::Claimed {
            assert_eq!(receipt.outcome, OwnedOutcome::Succeeded);
        }
        assert!(receipt.process_registered && receipt.barrier_released);
        assert_eq!(receipt.active_processes, Some(0));
        assert!(receipt.output_complete && !receipt.output_truncated);
        let response: serde_json::Value =
            serde_json::from_str(receipt.stdout.trim()).unwrap_or(serde_json::Value::Null);
        if kind != NativeProbeKind::Claimed {
            match kind {
                NativeProbeKind::LinkedOutside | NativeProbeKind::DirectOutside => {
                    assert_eq!(std::fs::read(&outside).unwrap(), b"outside-before\n");
                }
                NativeProbeKind::UnclaimedSibling => {
                    assert_eq!(
                        std::fs::read(fixture.worktree.join("unclaimed.txt")).unwrap(),
                        b"sibling-before\n"
                    );
                }
                NativeProbeKind::Claimed => unreachable!(),
            }
            let denial_tools = response["permission_denials"]
                .as_array()
                .map(|denials| {
                    denials
                        .iter()
                        .filter_map(|denial| denial["tool_name"].as_str())
                        .collect::<Vec<_>>()
                })
                .unwrap_or_default();
            let denied_edit = response["permission_denials"]
                .as_array()
                .is_some_and(|denials| {
                    denials.iter().any(|denial| {
                        denial["tool_name"] == "Edit"
                            && denial["tool_input"]["file_path"]
                                .as_str()
                                .is_some_and(|path| {
                                    path.ends_with(match kind {
                                        NativeProbeKind::LinkedOutside => "linked.txt",
                                        NativeProbeKind::DirectOutside => "outside.txt",
                                        NativeProbeKind::UnclaimedSibling => "unclaimed.txt",
                                        NativeProbeKind::Claimed => unreachable!(),
                                    })
                                })
                    })
                });
            assert!(
                denied_edit,
                "out-of-root probe did not record a denied Edit: outcome={:?} exit={:?} payload_exit={:?} is_error={:?} denial_tools={:?} stdout_bytes={} stderr_bytes={}",
                receipt.outcome,
                receipt.exit_code,
                receipt.payload_exit_code,
                response["is_error"],
                denial_tools,
                receipt.stdout.len(),
                receipt.stderr.len(),
            );
            assert_eq!(
                std::fs::read(fixture.worktree.join("result.txt")).unwrap(),
                b"old\n"
            );
        } else {
            assert_eq!(response["is_error"], false);
            assert_eq!(
                std::fs::read(fixture.worktree.join("result.txt")).unwrap(),
                b"pytxo-production-builder\n"
            );
        }
        assert_eq!(
            std::fs::read(fixture.worktree.join("starter.txt")).unwrap(),
            b"unchanged\n"
        );
        assert!(callbacks.registered && callbacks.settled);
    }
}
