//! Inert Codex subscription launch template for the reviewed routed path.
//!
//! This only lowers an already reviewed profile and binding into an exact
//! private-stdin owned-launch shape. It does not probe an account, qualify an
//! adapter, create a Ready observation, admit an attempt, or start a process.

use std::collections::{BTreeMap, BTreeSet};
use std::path::Path;
use std::time::Duration;

use anyhow::{bail, Context};
use pytxo_core::routing::{
    canonical_digest, BillingSourceId, BillingSourceMode, BindingId, Digest, ExecutableIdentity,
    ExecutionProfile, LaunchContract, LaunchTransport, MeteringSupport, ModelIdentity,
    ModelIdentityLevel, ProfileBinding, RouteTarget, StdinDelivery,
};
use pytxo_core::{ExecutionBackend, PermissionProfile};
use pytxo_runner::owned_launch::{OwnedLaunchSpec, OwnedTransport, PinnedFile};

pub const CODEX_ADAPTER_ID: &str = "pytxo-codex-embedded-host/1";
pub const CODEX_SUBSCRIPTION_ENDPOINT: &str = "codex-cli:chatgpt-subscription";

/// Reviewed account selection, not evidence that the current login uses it.
/// The account-specific home is the sole credential source exposed to Codex.
pub struct CodexAccountSource<'a> {
    pub binding_id: &'a BindingId,
    pub billing_source_id: &'a BillingSourceId,
    pub auth_owner: &'a str,
    pub endpoint_identity: &'a str,
    pub capacity_pool_ids: &'a BTreeSet<String>,
    pub codex_home: &'a Path,
}

/// All bounds and selection identities come from Core's reviewed admission.
/// A caller must match a separately demonstrated stable adapter qualification,
/// then bind this exact attempt's launch fingerprint before using the owned
/// launcher. The private prompt stays out of Pytxo argv and
/// routing history; Codex is requested to keep its own session ephemeral.
pub struct CodexTemplateInput<'a> {
    pub profile: &'a ExecutionProfile,
    pub binding: &'a ProfileBinding,
    pub selected_target: &'a RouteTarget,
    pub account: CodexAccountSource<'a>,
    pub codex: &'a PinnedFile,
    pub codex_version: &'a str,
    pub embedded_host: &'a PinnedFile,
    pub embedded_host_version: &'a str,
    pub worktree: &'a Path,
    pub system_root: &'a Path,
    pub private_prompt: &'a [u8],
    pub permission_profile: PermissionProfile,
    pub barrier_timeout: Duration,
    pub execution_timeout: Duration,
    pub settlement_timeout: Duration,
    pub output_limit: usize,
}

/// A draft only. A matching observation requires separate live auth, model,
/// tool, cancellation, Job-zero, and billing-source qualification.
pub struct CodexLaunchTemplate {
    pub spec: OwnedLaunchSpec,
    pub launch: LaunchContract,
    pub executable: ExecutableIdentity,
    pub profile_digest: Digest,
    pub binding_digest: Digest,
    pub requested_model: ModelIdentity,
    pub model_identity_level: ModelIdentityLevel,
    pub metering_support: MeteringSupport,
}

pub fn build_codex_launch_template(
    input: CodexTemplateInput<'_>,
) -> anyhow::Result<CodexLaunchTemplate> {
    let profile = input.profile;
    let binding = input.binding;
    if profile.schema_version != 1
        || profile.canonicalization_version != 1
        || binding.schema_version != 1
        || binding.canonicalization_version != 1
        || profile.revision == 0
        || binding.revision == 0
        || profile.harness_id != "codex"
        || profile.adapter_contract_version != "1"
        || profile.adapter_digest != Digest::of_bytes(CODEX_ADAPTER_ID.as_bytes())
        || !profile.skill_tool_bundle_digest.is_valid()
        || profile.backend != ExecutionBackend::Pty
        || input.permission_profile != PermissionProfile::Orbit
    {
        bail!("unsupported Codex profile, backend, or permission scope");
    }
    let profile_digest = profile.digest()?;
    let binding_digest = binding.digest()?;
    if input.selected_target.profile_id != profile.id
        || input.selected_target.binding_id != binding.id
        || binding.profile_digest != profile_digest
        || binding.billing_mode != BillingSourceMode::Subscription
        || binding.credential_reference.is_some()
        || binding.auth_owner != "Codex"
        || binding.endpoint_identity != CODEX_SUBSCRIPTION_ENDPOINT
        || binding.trust_class != "vendor"
        || binding.billing_source_id.0.is_empty()
        || binding.capacity_pool_ids.is_empty()
        || binding.capacity_pool_ids.iter().any(String::is_empty)
    {
        bail!("Codex selection is not the reviewed subscription binding");
    }
    if input.account.binding_id != &binding.id
        || input.account.billing_source_id != &binding.billing_source_id
        || input.account.auth_owner != binding.auth_owner
        || input.account.endpoint_identity != binding.endpoint_identity
        || input.account.capacity_pool_ids != &binding.capacity_pool_ids
    {
        bail!("Codex account or capacity source differs from the reviewed binding");
    }
    if !input.account.codex_home.is_absolute()
        || !input.account.codex_home.is_dir()
        || !input.worktree.is_absolute()
        || !input.system_root.is_absolute()
        || !input.system_root.is_dir()
    {
        bail!("Codex account and Windows root must exist; attempt worktree must be absolute");
    }
    let canonical_home = std::fs::canonicalize(input.account.codex_home)?;
    // Admission allocates the exact attempt ID before the owned worktree may
    // be created. Resolve its existing ancestor for an early overlap check;
    // the native launch gate must still canonicalize the completed worktree.
    let canonical_worktree = canonical_existing_or_future_directory(input.worktree)?;
    if crate::routed_worker::windows_path_starts_with(&canonical_home, &canonical_worktree)?
        || crate::routed_worker::windows_path_starts_with(&canonical_worktree, &canonical_home)?
    {
        bail!("Codex account home cannot overlap the writable attempt worktree");
    }
    let arguments = codex_exec_arguments(&profile.requested_model)?;
    if input.private_prompt.is_empty()
        || input.private_prompt.len() > 16_384
        || std::str::from_utf8(input.private_prompt).is_err()
        || input.private_prompt.contains(&0)
    {
        bail!("Codex private prompt is invalid or exceeds the owned-host limit");
    }
    if input.barrier_timeout.is_zero()
        || input.barrier_timeout > Duration::from_secs(30)
        || input.execution_timeout.is_zero()
        || input.execution_timeout > Duration::from_secs(86_400)
        || input.settlement_timeout.is_zero()
        || input.settlement_timeout > Duration::from_secs(60)
        || input.output_limit == 0
        || input.output_limit > 16 * 1024 * 1024
    {
        bail!("Codex owned-launch bounds are invalid");
    }
    validate_pin(input.codex, input.codex_version)?;
    validate_pin(input.embedded_host, input.embedded_host_version)?;
    if input.codex.path == input.embedded_host.path {
        bail!("Codex payload and embedded attempt host cannot share one identity");
    }
    let home = input
        .account
        .codex_home
        .to_str()
        .context("non-Unicode Codex home")?;
    let root = input
        .system_root
        .to_str()
        .context("non-Unicode Windows root")?;
    if home.contains('\0') || root.contains('\0') {
        bail!("Codex environment path contains NUL");
    }
    let environment = BTreeMap::from([
        ("CODEX_HOME".to_owned(), home.to_owned()),
        ("SystemRoot".to_owned(), root.to_owned()),
        ("WINDIR".to_owned(), root.to_owned()),
    ]);
    let executable = identity(input.codex, input.codex_version);
    let host = identity(input.embedded_host, input.embedded_host_version);
    let spec = OwnedLaunchSpec {
        bootstrap_host: input.embedded_host.clone(),
        executable: input.codex.clone(),
        dependencies: vec![],
        arguments,
        environment,
        working_directory: input.worktree.to_path_buf(),
        stdin: input.private_prompt.to_vec(),
        transport: OwnedTransport::Pty,
        barrier_timeout: input.barrier_timeout,
        execution_timeout: input.execution_timeout,
        settlement_timeout: input.settlement_timeout,
        output_limit: input.output_limit,
    };
    let launch = LaunchContract {
        schema_version: 1,
        transport: LaunchTransport::HostPty,
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
        bail!("Codex launch template does not match the hosted contract");
    }
    Ok(CodexLaunchTemplate {
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

fn canonical_existing_or_future_directory(path: &Path) -> anyhow::Result<std::path::PathBuf> {
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

/// The only Codex command shape allowed for this adapter version. The native
/// gate regenerates it from the reviewed model rather than trusting a matching
/// observation and argv digest supplied by a future qualification controller.
pub(crate) fn codex_exec_arguments(model: &ModelIdentity) -> anyhow::Result<Vec<String>> {
    if model.provider != "openai" || !valid_token(&model.model) || model.revision.is_some() {
        bail!("Codex model request is unsupported or cannot be pinned");
    }
    if model
        .reasoning
        .as_deref()
        .is_some_and(|value| !matches!(value, "low" | "medium" | "high" | "xhigh" | "max"))
    {
        bail!("Codex reasoning request is unsupported");
    }
    let mut arguments = vec![
        "exec".to_owned(),
        "--ignore-user-config".to_owned(),
        "--ephemeral".to_owned(),
        "--sandbox".to_owned(),
        "workspace-write".to_owned(),
        "--model".to_owned(),
        model.model.clone(),
    ];
    if let Some(reasoning) = &model.reasoning {
        arguments.extend([
            "--config".to_owned(),
            format!("model_reasoning_effort=\"{reasoning}\""),
        ]);
    }
    arguments.push("-".to_owned());
    Ok(arguments)
}

fn valid_token(value: &str) -> bool {
    !value.is_empty()
        && !value.starts_with('-')
        && value
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'-' | b'_' | b'.'))
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
        bail!("Codex executable or embedded-host pin is invalid");
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
    use super::*;
    use pytxo_core::routing::{BindingId, ModelIdentity, ProfileId};
    use std::path::PathBuf;

    struct Fixture {
        root: tempfile::TempDir,
        profile: ExecutionProfile,
        binding: ProfileBinding,
        target: RouteTarget,
        codex: PinnedFile,
        host: PinnedFile,
        home: PathBuf,
        worktree: PathBuf,
        system_root: PathBuf,
    }

    impl Fixture {
        fn new() -> Self {
            let root = tempfile::tempdir().unwrap();
            for name in ["home", "worktree", "windows"] {
                std::fs::create_dir(root.path().join(name)).unwrap();
            }
            let codex_path = root.path().join("codex.exe");
            let host_path = root.path().join("pytxo-embedded.exe");
            std::fs::write(&codex_path, b"codex test pin").unwrap();
            std::fs::write(&host_path, b"embedded host test pin").unwrap();
            let codex = PinnedFile::observe(codex_path).unwrap();
            let host = PinnedFile::observe(host_path).unwrap();
            let home = root.path().join("home");
            let worktree = root.path().join("worktree");
            let system_root = root.path().join("windows");
            let profile = ExecutionProfile {
                schema_version: 1,
                canonicalization_version: 1,
                id: ProfileId("everyday".into()),
                revision: 1,
                harness_id: "codex".into(),
                adapter_contract_version: "1".into(),
                adapter_digest: Digest::of_bytes(CODEX_ADAPTER_ID.as_bytes()),
                requested_model: ModelIdentity {
                    provider: "openai".into(),
                    model: "gpt-6-astra".into(),
                    reasoning: Some("high".into()),
                    revision: None,
                },
                skill_tool_bundle_digest: Digest::of_bytes(b"approved tools"),
                backend: ExecutionBackend::Pty,
                capabilities: BTreeSet::from(["edit".into()]),
            };
            let binding = ProfileBinding {
                schema_version: 1,
                canonicalization_version: 1,
                id: BindingId("personal-codex".into()),
                revision: 1,
                profile_digest: profile.digest().unwrap(),
                credential_reference: None,
                auth_owner: "Codex".into(),
                billing_source_id: BillingSourceId("personal-subscription".into()),
                billing_mode: BillingSourceMode::Subscription,
                endpoint_identity: CODEX_SUBSCRIPTION_ENDPOINT.into(),
                trust_class: "vendor".into(),
                capacity_pool_ids: BTreeSet::from([
                    "account-personal".into(),
                    "host-worker".into(),
                ]),
            };
            let target = RouteTarget {
                profile_id: profile.id.clone(),
                binding_id: binding.id.clone(),
            };
            Self {
                root,
                profile,
                binding,
                target,
                codex,
                host,
                home,
                worktree,
                system_root,
            }
        }

        fn input<'a>(&'a self, prompt: &'a [u8]) -> CodexTemplateInput<'a> {
            CodexTemplateInput {
                profile: &self.profile,
                binding: &self.binding,
                selected_target: &self.target,
                account: CodexAccountSource {
                    binding_id: &self.binding.id,
                    billing_source_id: &self.binding.billing_source_id,
                    auth_owner: &self.binding.auth_owner,
                    endpoint_identity: &self.binding.endpoint_identity,
                    capacity_pool_ids: &self.binding.capacity_pool_ids,
                    codex_home: &self.home,
                },
                codex: &self.codex,
                codex_version: "codex-test-v1",
                embedded_host: &self.host,
                embedded_host_version: "pytxo-test-v1",
                worktree: &self.worktree,
                system_root: &self.system_root,
                private_prompt: prompt,
                permission_profile: PermissionProfile::Orbit,
                barrier_timeout: Duration::from_secs(10),
                execution_timeout: Duration::from_secs(120),
                settlement_timeout: Duration::from_secs(5),
                output_limit: 4096,
            }
        }
    }

    #[test]
    fn exact_subscription_template_is_private_and_only_requested_evidence() {
        let fixture = Fixture::new();
        let prompt = "private request and é".as_bytes();
        let template = build_codex_launch_template(fixture.input(prompt)).unwrap();
        assert_eq!(
            template.spec.arguments,
            [
                "exec",
                "--ignore-user-config",
                "--ephemeral",
                "--sandbox",
                "workspace-write",
                "--model",
                "gpt-6-astra",
                "--config",
                "model_reasoning_effort=\"high\"",
                "-",
            ]
        );
        assert_eq!(template.spec.stdin, prompt);
        assert_eq!(template.spec.environment.len(), 3);
        assert_eq!(
            template.spec.environment.get("CODEX_HOME"),
            Some(
                &fixture
                    .root
                    .path()
                    .join("home")
                    .to_string_lossy()
                    .into_owned()
            )
        );
        assert!(!template.spec.environment.contains_key("OPENAI_API_KEY"));
        assert_eq!(template.launch.transport, LaunchTransport::HostPty);
        assert_eq!(
            template.launch.arguments_digest,
            canonical_digest(&template.spec.arguments, 1).unwrap()
        );
        assert_eq!(
            template.launch.environment_policy_digest,
            canonical_digest(&template.spec.environment, 1).unwrap()
        );
        assert_eq!(
            template.launch.private_stdin_digest,
            Some(Digest::of_bytes(prompt))
        );
        assert!(template.launch.valid_for(ExecutionBackend::Pty));
        assert_eq!(template.profile_digest, fixture.profile.digest().unwrap());
        assert_eq!(template.binding_digest, fixture.binding.digest().unwrap());
        assert_eq!(template.requested_model, fixture.profile.requested_model);
        assert_eq!(template.model_identity_level, ModelIdentityLevel::Requested);
        assert_eq!(template.metering_support, MeteringSupport::Unknown);
        assert!(!format!("{:?}", template.spec).contains("private request"));
        assert!(!serde_json::to_string(&template.launch)
            .unwrap()
            .contains("private request"));
    }

    #[test]
    fn private_prompt_changes_exact_launch_identity_without_entering_argv() {
        let fixture = Fixture::new();
        let first = build_codex_launch_template(fixture.input(b"first")).unwrap();
        let second = build_codex_launch_template(fixture.input(b"second")).unwrap();
        assert_eq!(first.spec.arguments, second.spec.arguments);
        assert_ne!(
            first.launch.private_stdin_digest,
            second.launch.private_stdin_digest
        );
        assert_ne!(first.launch, second.launch);
    }

    #[test]
    fn subscription_home_cannot_overlap_writable_worktree() {
        let mut fixture = Fixture::new();
        fixture.home = fixture.worktree.clone();
        assert!(build_codex_launch_template(fixture.input(b"task")).is_err());

        let mut fixture = Fixture::new();
        fixture.home = fixture.worktree.join("account");
        std::fs::create_dir(&fixture.home).unwrap();
        assert!(build_codex_launch_template(fixture.input(b"task")).is_err());

        let mut fixture = Fixture::new();
        fixture.worktree = fixture.home.join("repository");
        std::fs::create_dir(&fixture.worktree).unwrap();
        assert!(build_codex_launch_template(fixture.input(b"task")).is_err());

        #[cfg(windows)]
        {
            let mut fixture = Fixture::new();
            let canonical = std::fs::canonicalize(&fixture.worktree).unwrap();
            let name = canonical.file_name().unwrap().to_string_lossy();
            let alternate_name: String = name
                .chars()
                .map(|character| {
                    if character.is_ascii_lowercase() {
                        character.to_ascii_uppercase()
                    } else {
                        character.to_ascii_lowercase()
                    }
                })
                .collect();
            assert_ne!(name, alternate_name);
            fixture.home = canonical.parent().unwrap().join(alternate_name);
            assert!(build_codex_launch_template(fixture.input(b"task")).is_err());
        }
    }

    #[test]
    fn future_attempt_worktree_can_be_bound_without_unowned_creation() {
        let mut fixture = Fixture::new();
        fixture.worktree = fixture.root.path().join("future-run").join("attempt-one");
        assert!(!fixture.worktree.exists());
        let template = build_codex_launch_template(fixture.input(b"reviewed task")).unwrap();
        assert_eq!(template.spec.working_directory, fixture.worktree);
        assert!(!template.spec.working_directory.exists());

        fixture.worktree = fixture.home.join("future-run").join("attempt-two");
        assert!(build_codex_launch_template(fixture.input(b"reviewed task")).is_err());

        let file = fixture.root.path().join("not-a-directory");
        std::fs::write(&file, b"file").unwrap();
        fixture.worktree = file.join("attempt-three");
        assert!(build_codex_launch_template(fixture.input(b"reviewed task")).is_err());
    }

    #[test]
    fn billing_modes_and_credential_fallbacks_fail_closed() {
        for mode in [
            BillingSourceMode::Api,
            BillingSourceMode::Managed,
            BillingSourceMode::Local,
        ] {
            let mut fixture = Fixture::new();
            fixture.binding.billing_mode = mode;
            assert!(build_codex_launch_template(fixture.input(b"task")).is_err());
        }
        let mut fixture = Fixture::new();
        fixture.binding.credential_reference = Some("keychain:api".into());
        assert!(build_codex_launch_template(fixture.input(b"task")).is_err());
    }

    #[test]
    fn selected_account_capacity_and_route_must_match_exactly() {
        let fixture = Fixture::new();
        let other_billing = BillingSourceId("other".into());
        let mut input = fixture.input(b"task");
        input.account.billing_source_id = &other_billing;
        assert!(build_codex_launch_template(input).is_err());

        let other_pools = BTreeSet::from(["unreserved".into()]);
        let mut input = fixture.input(b"task");
        input.account.capacity_pool_ids = &other_pools;
        assert!(build_codex_launch_template(input).is_err());

        let other_owner = "another account";
        let mut input = fixture.input(b"task");
        input.account.auth_owner = other_owner;
        assert!(build_codex_launch_template(input).is_err());

        let other_target = RouteTarget {
            profile_id: ProfileId("fallback".into()),
            binding_id: fixture.binding.id.clone(),
        };
        let mut input = fixture.input(b"task");
        input.selected_target = &other_target;
        assert!(build_codex_launch_template(input).is_err());
    }

    #[test]
    fn unsupported_harness_backend_permission_model_and_prompt_fail_closed() {
        let mut fixture = Fixture::new();
        fixture.profile.harness_id = "claude".into();
        assert!(build_codex_launch_template(fixture.input(b"task")).is_err());

        let mut fixture = Fixture::new();
        fixture.profile.backend = ExecutionBackend::Subprocess;
        assert!(build_codex_launch_template(fixture.input(b"task")).is_err());

        let fixture = Fixture::new();
        let mut input = fixture.input(b"task");
        input.permission_profile = PermissionProfile::Supernova;
        assert!(build_codex_launch_template(input).is_err());

        let mut fixture = Fixture::new();
        fixture.profile.requested_model.provider = "other".into();
        assert!(build_codex_launch_template(fixture.input(b"task")).is_err());

        let fixture = Fixture::new();
        assert!(build_codex_launch_template(fixture.input(b"")).is_err());
        assert!(build_codex_launch_template(fixture.input(&vec![b'x'; 16_385])).is_err());
    }

    #[test]
    fn invalid_pins_paths_and_limits_cannot_form_a_launch() {
        let mut fixture = Fixture::new();
        fixture.codex.sha256 = "not-a-digest".into();
        assert!(build_codex_launch_template(fixture.input(b"task")).is_err());

        let fixture = Fixture::new();
        let mut input = fixture.input(b"task");
        input.account.codex_home = Path::new("relative-home");
        assert!(build_codex_launch_template(input).is_err());

        let fixture = Fixture::new();
        let mut input = fixture.input(b"task");
        input.barrier_timeout = Duration::from_secs(31);
        assert!(build_codex_launch_template(input).is_err());

        let fixture = Fixture::new();
        let mut input = fixture.input(b"task");
        input.output_limit = 0;
        assert!(build_codex_launch_template(input).is_err());
    }

    /// Explicit, subscription-account qualification. It is intentionally not
    /// part of normal tests or a Ready observation: the operator must inspect
    /// the exact executable, home, model, host and resulting receipt first.
    #[cfg(windows)]
    #[test]
    #[ignore = "requires explicit native Codex account, model and embedded host"]
    fn prompt_bearing_codex_exec_edits_only_its_owned_disposable_worktree() {
        use pytxo_core::Result;
        use pytxo_runner::owned_launch::{
            run_owned_launch, LaunchCallbacks, LaunchGuard, LaunchIntent, OwnedLaunchReceipt,
            OwnedOutcome, OwnedProcess,
        };
        use std::process::Command;

        const BEFORE: &[u8] = b"before\n";
        const AFTER: &[u8] = b"qualified-pytxo-owned-run\n";
        const PROMPT: &[u8] = b"Edit only task.txt in the current repository. Replace its entire contents with the single line qualified-pytxo-owned-run followed by a newline. Do not create or edit any other file. Do not run tests. Stop after editing.\n";

        struct Callbacks {
            task_file: PathBuf,
            registered: bool,
            settled: usize,
        }
        struct Guard<'a>(&'a mut Callbacks);
        impl LaunchGuard for Guard<'_> {
            fn permit_create(&mut self, _: &LaunchIntent) -> Result<()> {
                Ok(())
            }

            fn register(&mut self, process: &OwnedProcess) -> Result<()> {
                assert_eq!(std::fs::read(&self.0.task_file).unwrap(), BEFORE);
                assert!(process.pid > 0);
                assert!(process
                    .start_identity
                    .as_deref()
                    .is_some_and(|value| value.starts_with("windows-filetime:")));
                self.0.registered = true;
                Ok(())
            }
            fn cancelled(&mut self) -> Result<bool> {
                Ok(false)
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

        fn git(dir: &Path, args: &[&str]) {
            let output = Command::new("git")
                .current_dir(dir)
                .args(args)
                .output()
                .unwrap();
            assert!(
                output.status.success(),
                "disposable Git setup failed: {args:?}"
            );
        }

        let mut fixture = Fixture::new();
        fixture.codex = PinnedFile::observe(PathBuf::from(
            std::env::var_os("PYTXO_TEST_CODEX_EXE")
                .expect("set PYTXO_TEST_CODEX_EXE to the reviewed native CLI"),
        ))
        .unwrap();
        fixture.host = PinnedFile::observe(PathBuf::from(
            std::env::var_os("PYTXO_TEST_EMBEDDED_HOST")
                .expect("set PYTXO_TEST_EMBEDDED_HOST to the separate current-source test host"),
        ))
        .unwrap();
        fixture.home = PathBuf::from(
            std::env::var_os("PYTXO_TEST_CODEX_HOME")
                .expect("set PYTXO_TEST_CODEX_HOME to the reviewed subscription account home"),
        );
        fixture.system_root = PathBuf::from(std::env::var_os("SystemRoot").unwrap());
        fixture.profile.requested_model.model = std::env::var("PYTXO_TEST_CODEX_MODEL")
            .expect("set PYTXO_TEST_CODEX_MODEL to the reviewed account model");
        fixture.profile.requested_model.reasoning = Some("low".into());
        fixture.binding.profile_digest = fixture.profile.digest().unwrap();

        let source = fixture.root.path().join("source");
        std::fs::create_dir(&source).unwrap();
        git(&source, &["init", "--quiet"]);
        std::fs::write(source.join(".gitattributes"), b"task.txt text eol=lf\n").unwrap();
        std::fs::write(source.join("task.txt"), BEFORE).unwrap();
        git(&source, &["add", ".gitattributes", "task.txt"]);
        git(
            &source,
            &[
                "-c",
                "user.name=Pytxo Qualification",
                "-c",
                "user.email=pytxo-qualification@example.invalid",
                "commit",
                "--quiet",
                "-m",
                "baseline",
            ],
        );
        fixture.worktree = fixture.root.path().join("attempt-worktree");
        git(
            &source,
            &[
                "worktree",
                "add",
                "--quiet",
                "--detach",
                fixture.worktree.to_str().unwrap(),
                "HEAD",
            ],
        );
        let task_file = fixture.worktree.join("task.txt");
        assert_eq!(std::fs::read(&task_file).unwrap(), BEFORE);

        let template = build_codex_launch_template(fixture.input(PROMPT)).unwrap();
        assert_eq!(template.spec.stdin, PROMPT);
        assert_eq!(template.spec.environment.len(), 3);
        assert_eq!(
            template.spec.arguments,
            codex_exec_arguments(&fixture.profile.requested_model).unwrap()
        );
        let mut callbacks = Callbacks {
            task_file: task_file.clone(),
            registered: false,
            settled: 0,
        };
        let receipt = run_owned_launch(&template.spec, &mut callbacks, || Ok(false)).unwrap();
        let output = format!("{}\n{}", receipt.stdout, receipt.stderr).to_ascii_lowercase();
        let failure_markers = [
            "sandbox",
            "denied",
            "permission",
            "unsupported",
            "model",
            "auth",
            "login",
            "rate limit",
            "network",
            "400",
            "401",
            "403",
            "429",
            "error",
            "failed",
        ]
        .into_iter()
        .filter(|marker| output.contains(marker))
        .collect::<Vec<_>>();
        assert_eq!(
            receipt.outcome,
            OwnedOutcome::Succeeded,
            "{receipt:?}; diagnostic_markers={failure_markers:?}; stdout_bytes={} stderr_bytes={}",
            receipt.stdout.len(),
            receipt.stderr.len(),
        );
        assert!(receipt.process_registered && receipt.barrier_released);
        assert!(receipt.payload_pid_reported.is_some());
        assert_eq!(receipt.payload_exit_code, Some(0));
        assert_eq!(receipt.active_processes, Some(0));
        assert!(!receipt.terminated_job);
        assert!(receipt.output_complete && !receipt.output_truncated);
        assert!(callbacks.registered && callbacks.settled == 1);
        assert_eq!(
            std::fs::read(&task_file).unwrap(),
            AFTER,
            "Codex settled without the requested edit: stdout_bytes={} stderr_bytes={} sandbox={} denied={} permission={} error={} failed={} blocked={} patch={} tool={} read_only={} unable={}",
            receipt.stdout.len(),
            receipt.stderr.len(),
            output.contains("sandbox"),
            output.contains("denied"),
            output.contains("permission"),
            output.contains("error"),
            output.contains("failed"),
            output.contains("blocked"),
            output.contains("patch"),
            output.contains("tool"),
            output.contains("read-only"),
            output.contains("unable"),
        );
        let status = Command::new("git")
            .current_dir(&fixture.worktree)
            .args(["status", "--porcelain", "--untracked-files=all"])
            .output()
            .unwrap();
        assert!(status.status.success());
        assert_eq!(String::from_utf8_lossy(&status.stdout).trim(), "M task.txt");
    }
}
