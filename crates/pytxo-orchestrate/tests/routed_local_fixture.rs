#![cfg(windows)]

use std::collections::BTreeSet;
use std::fs;
use std::panic::{catch_unwind, AssertUnwindSafe};
use std::process::Command;
use std::sync::Mutex;
#[cfg(feature = "routed-test-faults")]
use std::sync::{
    atomic::{AtomicUsize, Ordering},
    Arc,
};

static FIXTURE_ENV_LOCK: Mutex<()> = Mutex::new(());

fn alternate_power_shell_child(task_id: &str) -> bool {
    #[cfg(feature = "routed-test-faults")]
    {
        task_id == "result"
            && std::env::var("PYTXO_TEST_ROUTED_POWERSHELL_HANDOFF")
                .ok()
                .as_deref()
                == Some("1")
    }
    #[cfg(not(feature = "routed-test-faults"))]
    {
        let _ = task_id;
        false
    }
}

#[cfg(feature = "routed-test-faults")]
const HOSTED_PAYLOAD_SOURCE: &str = r#"
using System;
using System.IO;
using System.Security.Cryptography;
using System.Text;
using System.Threading;
class PytxoSyntheticPayload {
    static int Main(string[] args) {
        if (args.Length != 1) return 2;
        string input = Console.In.ReadToEnd();
        if (args[0] == "probe") {
            if (input != "synthetic owned-host probe; no provider call") return 3;
            File.WriteAllBytes("probe.txt", Encoding.ASCII.GetBytes("pytxo-probe\r\n"));
            return 0;
        }
        if (args[0] == "wait") {
            if (input != "synthetic owned-host probe; no provider call") return 3;
            Thread.Sleep(30000);
            return 0;
        }
        if (args[0] != "run"
            || !input.Contains("\"goal\":\"pytxo-local-fixture-v1:write-result\"")
            || !input.Contains("\"attempt_id\":\"routed-")) return 4;
        using (var sha = SHA256.Create()) {
            byte[] digest = sha.ComputeHash(Encoding.UTF8.GetBytes(input));
            var hex = new StringBuilder();
            foreach (byte value in digest) hex.Append(value.ToString("x2"));
            Console.WriteLine("synthetic-stdin-sha256:" + hex.ToString());
        }
        File.WriteAllBytes("result.txt", Encoding.ASCII.GetBytes("pytxo-routed\r\n"));
        return 0;
    }
}
"#;

#[cfg(feature = "routed-test-faults")]
fn prepare_hosted_payload(dir: &std::path::Path) {
    let source = dir.join("pytxo-synthetic-payload.cs");
    let executable = dir.join("pytxo-synthetic-payload.exe");
    fs::write(&source, HOSTED_PAYLOAD_SOURCE).unwrap();
    let system_root = std::path::PathBuf::from(std::env::var_os("SystemRoot").unwrap());
    let compiler = system_root.join("Microsoft.NET/Framework64/v4.0.30319/csc.exe");
    let build = Command::new(compiler)
        .args(["/nologo", "/target:exe"])
        .arg(format!("/out:{}", executable.display()))
        .arg(&source)
        .output()
        .unwrap();
    assert!(
        build.status.success(),
        "synthetic payload compile failed: {}",
        String::from_utf8_lossy(&build.stderr)
    );
    unsafe { std::env::set_var("PYTXO_TEST_ROUTED_HOSTED_PAYLOAD", executable) };
}

use pytxo_core::routing::*;
use pytxo_core::{ExecutionBackend, PermissionProfile, PytxoConfig, RunId, TaskId};
use pytxo_orchestrate::flow::{
    dispatch_experimental_routed_flow, freeze_experimental_routed_checks,
    observe_experimental_routed_git_base, preview_experimental_routed_flow,
    reconcile_routed_flow_startup, save_reviewed_flow_plan, FlowDraftInput, FlowPlan, FlowSource,
};
#[cfg(feature = "routed-test-faults")]
use pytxo_orchestrate::stop_exact_routed;
use pytxo_orchestrate::{apply_run_changes, discard_run_review, refresh_run_review, trust_repo};
use pytxo_runner::{registry_path, ProcessEntry, ProcessRegistryFile};
use pytxo_store::capacity::CapacityPoolConfig;
use pytxo_store::routing::{RegisteredProfile, RegisteredTask, RoutingMission};
use pytxo_store::{Catalog, PytxoStore};

fn write_exact_terminal_marker(
    catalog: &Catalog,
    draft_id: &str,
    run_id: &str,
    data_dir: &std::path::Path,
    repo_root: &str,
) {
    let owner = catalog
        .routed_flow_dispatch_owner(draft_id, run_id)
        .unwrap()
        .unwrap();
    fs::write(
        data_dir.join("active_run.json"),
        serde_json::json!({
            "run_id": run_id,
            "repo_root": repo_root,
            "supervisor_pid": owner.controller_pid,
            "supervisor_start_identity": owner.controller_start_identity,
        })
        .to_string(),
    )
    .unwrap();
}

fn fixture_profile(name: &str) -> RegisteredProfile {
    #[cfg(feature = "routed-test-faults")]
    let alternate = name == "strong"
        && std::env::var("PYTXO_TEST_ROUTED_POWERSHELL_HANDOFF")
            .ok()
            .as_deref()
            == Some("1");
    #[cfg(not(feature = "routed-test-faults"))]
    let alternate = false;
    let adapter = if alternate {
        "pytxo-local-fixture-powershell-v1"
    } else {
        "pytxo-local-fixture-v1"
    };
    let profile = ExecutionProfile {
        schema_version: 1,
        canonicalization_version: 1,
        id: ProfileId(name.into()),
        revision: 1,
        harness_id: adapter.into(),
        adapter_contract_version: "1".into(),
        adapter_digest: Digest::of_bytes(adapter.as_bytes()),
        requested_model: ModelIdentity {
            provider: "local-fixture".into(),
            model: name.into(),
            reasoning: None,
            revision: None,
        },
        skill_tool_bundle_digest: Digest::of_bytes(b"tools"),
        backend: ExecutionBackend::Subprocess,
        capabilities: BTreeSet::from(["edit".into()]),
    };
    let binding = ProfileBinding {
        schema_version: 1,
        canonicalization_version: 1,
        id: BindingId(name.into()),
        revision: 1,
        profile_digest: profile.digest().unwrap(),
        credential_reference: None,
        auth_owner: "local-fixture".into(),
        billing_source_id: BillingSourceId("local".into()),
        billing_mode: BillingSourceMode::Local,
        endpoint_identity: adapter.into(),
        trust_class: "local".into(),
        capacity_pool_ids: BTreeSet::from(["host".into()]),
    };
    RegisteredProfile { profile, binding }
}

fn mission(
    plan: &FlowPlan,
    run_id: &RunId,
    plan_digest: &Digest,
) -> anyhow::Result<RoutingMission> {
    #[cfg(feature = "routed-test-faults")]
    let fake_shadow = std::env::var_os("PYTXO_TEST_ROUTED_SHADOW_FAKE_ADVICE").is_some();
    #[cfg(feature = "routed-test-faults")]
    let mock_shadow = std::env::var_os("PYTXO_TEST_ROUTED_SHADOW_MOCK_ADVICE").is_some();
    #[cfg(not(feature = "routed-test-faults"))]
    let fake_shadow = false;
    #[cfg(not(feature = "routed-test-faults"))]
    let mock_shadow = false;
    let profiles = vec![fixture_profile("everyday"), fixture_profile("strong")];
    let targets = profiles
        .iter()
        .map(|profile| RouteTarget {
            profile_id: profile.profile.id.clone(),
            binding_id: profile.binding.id.clone(),
        })
        .collect::<Vec<_>>();
    let mut policy = RoutingPolicy {
        schema_version: 1,
        version: "local-fixture-rules-v1".into(),
        mode: if fake_shadow || mock_shadow {
            RoutingMode::Shadow
        } else {
            RoutingMode::Rules
        },
        everyday: targets[0].clone(),
        strong: targets[1].clone(),
        evaluated_manifest_digest: None,
        everyday_threshold_ppm: 800_000,
        unclear_ceiling_ppm: 100_000,
        advice_model: if mock_shadow || fake_shadow {
            pytxo_planner::advisor::MODEL_ID
        } else {
            "disabled"
        }
        .into(),
        advice_template: if mock_shadow || fake_shadow {
            "pending-reviewed-request".into()
        } else {
            "disabled".into()
        },
        disclosure_scope_digest: if mock_shadow || fake_shadow {
            Some(pytxo_orchestrate::flow::reviewed_routing_advisor_disclosure_scope_digest())
        } else {
            None
        },
        advisor_recipient: None,
    };
    let tasks = plan
        .tasks
        .iter()
        .map(|planned| -> anyhow::Result<RegisteredTask> {
            let recipes = freeze_experimental_routed_checks(&planned.id, &planned.verify)?;
            Ok(RegisteredTask {
                contract: TaskContract {
                    schema_version: 1,
                    canonicalization_version: 1,
                    task_id: TaskId(planned.id.clone()),
                    revision: 1,
                    plan_digest: plan_digest.clone(),
                    base: observe_experimental_routed_git_base(std::path::Path::new(
                        &plan.domain_id,
                    ))?,
                    goal: planned.prompt.clone(),
                    constraints: vec![],
                    claim_roots: planned.paths.clone(),
                    dependencies: planned.dependencies.iter().cloned().map(TaskId).collect(),
                    task_kind: Some(if fake_shadow || mock_shadow {
                        TaskKind::Other
                    } else {
                        TaskKind::LocalTransformation
                    }),
                    task_kind_evidence: Some(Digest::of_bytes(b"local-fixture-task-kind")),
                    required_capabilities: BTreeSet::from(["edit".into()]),
                    checks: recipes
                        .iter()
                        .map(|recipe| recipe.reference())
                        .collect::<pytxo_core::Result<Vec<_>>>()?,
                    required_resources: BTreeSet::from(["host".into()]),
                    skill_tool_bundle_digest: Digest::of_bytes(b"tools"),
                    permission_profile: PermissionProfile::Orbit,
                    required_egress: BTreeSet::new(),
                    required_target: None,
                    strong_only: alternate_power_shell_child(&planned.id),
                    cross_component_requirement: Some(false),
                    context_complete: true,
                    repeatable_symptom_supplied: None,
                    specific_cause_hypothesis_supplied: None,
                },
                attempt_budget_nano_usd: 0,
                check_recipes: recipes,
            })
        })
        .collect::<anyhow::Result<Vec<_>>>()?;
    if mock_shadow || fake_shadow {
        let [registered] = tasks.as_slice() else {
            anyhow::bail!("Shadow fixture requires exactly one reviewed task");
        };
        policy.advice_template =
            pytxo_orchestrate::flow::reviewed_routing_advisor_identity_for_task(
                &registered.contract,
            )?;
    }
    let allowed_task_digests = tasks
        .iter()
        .map(|task| task.contract.digest())
        .collect::<Result<BTreeSet<_>, _>>()?;
    let authorization = MissionAuthorization {
        schema_version: 1,
        domain_id: pytxo_core::DomainId(plan.domain_id.clone()),
        run_id: run_id.clone(),
        plan_id: PlanId("local-fixture-plan".into()),
        plan_digest: plan_digest.clone(),
        revision: 1,
        cancel_epoch: 0,
        allowed_task_digests,
        allowed_profiles: profiles
            .iter()
            .zip(targets)
            .map(|(entry, target)| ApprovedProfile {
                target,
                profile_digest: entry.profile.digest().unwrap(),
                binding_digest: entry.binding.digest().unwrap(),
            })
            .collect(),
        allowed_billing_sources: BTreeSet::from([BillingSourceId("local".into())]),
        allowed_billing_modes: BTreeSet::from([BillingSourceMode::Local]),
        permission_profile: PermissionProfile::Orbit,
        allowed_egress: BTreeSet::new(),
        minimum_model_identity: ModelIdentityLevel::Requested,
        limits: MissionLimits {
            deadline_ms: u64::MAX,
            max_workers: u32::try_from(plan.max_workers)?,
            max_attempts: 2,
            max_spend_nano_usd: None,
            spend_guarantee: SpendGuarantee::RiskBounded,
        },
        policy_digest: policy.digest()?,
        live_advice_authorized: false,
        consent_revision: 1,
    };
    Ok(RoutingMission {
        authorization,
        policy,
        tasks,
        profiles,
    })
}

#[cfg(feature = "routed-test-faults")]
fn hosted_shadow_mission(
    plan: &FlowPlan,
    run_id: &RunId,
    plan_digest: &Digest,
) -> anyhow::Result<RoutingMission> {
    let mut reviewed = mission(plan, run_id, plan_digest)?;
    reviewed.tasks[0].contract.task_kind = Some(TaskKind::Other);
    reviewed.authorization.allowed_task_digests = reviewed
        .tasks
        .iter()
        .map(|task| task.contract.digest())
        .collect::<Result<BTreeSet<_>, _>>()?;
    reviewed.policy.version = "local-fixture-hosted-shadow-v1".into();
    reviewed.policy.mode = RoutingMode::Shadow;
    reviewed.policy.advice_model = pytxo_planner::advisor::MODEL_ID.into();
    reviewed.policy.advisor_recipient = Some(pytxo_planner::advisor::HOSTED_RECIPIENT.into());
    reviewed.policy.disclosure_scope_digest = Some(pytxo_planner::advisor::hosted_scope_digest());
    reviewed.policy.advice_template =
        pytxo_orchestrate::flow::reviewed_hosted_routing_advisor_identity_for_task(
            &reviewed.tasks[0].contract,
        )?;
    reviewed.authorization.limits.max_attempts = 1;
    reviewed.authorization.policy_digest = reviewed.policy.digest()?;
    Ok(reviewed)
}

fn run_reviewed_fixture(expected_state: AttemptState, expected_run_status: &str) {
    run_reviewed_fixture_with_config(expected_state, expected_run_status, false);
}

#[cfg(feature = "routed-test-faults")]
fn stop_during_routed_startup(stage: &str, change_data_dir: bool) {
    let _env_lock = FIXTURE_ENV_LOCK.lock().unwrap();
    let active_worker = stage == "ACTIVE_WORKER_CONFIG_DRIFT";
    let late_terminal_stop = stage == "PYTXO_TEST_ROUTED_PAUSE_AFTER_TERMINAL_BEFORE_ACK";
    let home = tempfile::tempdir().unwrap();
    let pause = tempfile::tempdir().unwrap();
    unsafe {
        std::env::set_var("PYTXO_HOME", home.path().join("home"));
        std::env::set_var("PYTXO_TRUST_STORE", home.path().join("trust.json"));
        std::env::set_var("PYTXO_EXPERIMENTAL_ROUTED_LOCAL_FIXTURE", "1");
        if active_worker {
            std::env::set_var("PYTXO_TEST_ROUTED_SLOW_SIBLINGS", "1");
        } else {
            std::env::set_var(stage, pause.path());
        }
    }
    let repo = tempfile::tempdir().unwrap();
    fs::create_dir_all(repo.path().join("src")).unwrap();
    fs::write(repo.path().join("src/lib.rs"), "pub fn value() {}\n").unwrap();
    fs::write(repo.path().join("result.txt"), "").unwrap();
    fs::write(
        repo.path().join("pytxo.toml"),
        "permission_profile = 'orbit'\nexecution_backend = 'subprocess'\nmax_agents = 1\n[blast]\nprefer_kernel_overlay = false\n[[task]]\nid = 'fixture'\nagent = 'default'\npaths = ['result.txt']\n",
    )
    .unwrap();
    for args in [
        vec!["init", "-q"],
        vec!["add", "."],
        vec![
            "-c",
            "user.name=Test",
            "-c",
            "user.email=test@pytxo.local",
            "commit",
            "-qm",
            "baseline",
        ],
    ] {
        let output = Command::new("git")
            .args(args)
            .current_dir(repo.path())
            .output()
            .unwrap();
        assert!(
            output.status.success(),
            "{}",
            String::from_utf8_lossy(&output.stderr)
        );
    }
    trust_repo(repo.path(), PermissionProfile::Orbit).unwrap();
    let catalog_path = repo.path().join(".git/catalog.db");
    let catalog = Catalog::open(&catalog_path).unwrap();
    catalog
        .configure_capacity_pool(&CapacityPoolConfig {
            resource_id: "host".into(),
            capacity_units: 1,
            expected_revision: None,
            configured_at_ms: 100,
        })
        .unwrap();
    let preview = preview_experimental_routed_flow(
        &catalog,
        FlowDraftInput {
            id: "routed-stop-startup".into(),
            title: "Routed stop".into(),
            mission_text: "pytxo-local-fixture-v1:write-result".into(),
            source: FlowSource::Text,
            domain_id: Some(repo.path().to_string_lossy().into_owned()),
            project_id: None,
            ade_id: None,
            max_workers: Some(1),
            verification_commands: vec!["if exist result.txt (exit /b 0) else (exit /b 1)".into()],
        },
        mission,
    )
    .unwrap();
    let reviewed = save_reviewed_flow_plan(&catalog, preview).unwrap();
    let run_id = reviewed
        .routing
        .as_ref()
        .unwrap()
        .authorization
        .run_id
        .0
        .clone();
    let draft_id = reviewed.draft_id.clone();
    let thread_path = catalog_path.clone();
    let worker = std::thread::spawn(move || {
        let catalog = Catalog::open(&thread_path).unwrap();
        dispatch_experimental_routed_flow(&catalog, &draft_id)
    });
    let deadline = std::time::Instant::now()
        + std::time::Duration::from_secs(if active_worker || late_terminal_stop {
            60
        } else {
            15
        });
    let registered_worker = if active_worker {
        let store = PytxoStore::open(&PytxoConfig::default().db_path_at(repo.path())).unwrap();
        loop {
            let targets = store
                .owned_job_stop_snapshot(
                    &pytxo_core::DomainId(reviewed.domain_id.clone()),
                    Some(&RunId(run_id.clone())),
                )
                .unwrap();
            if let Some(target) = targets.into_iter().find(|target| {
                target.phase == pytxo_store::routing_launch::OwnedJobStopPhase::Registered
            }) {
                break Some((target.pid.unwrap(), target.start_identity.unwrap()));
            }
            assert!(
                std::time::Instant::now() < deadline && !worker.is_finished(),
                "routed worker never reached registered native ownership"
            );
            std::thread::sleep(std::time::Duration::from_millis(10));
        }
    } else {
        while !pause.path().join("entered").exists() {
            assert!(
                std::time::Instant::now() < deadline,
                "dispatch did not reach the held stage"
            );
            std::thread::sleep(std::time::Duration::from_millis(10));
        }
        None
    };
    if change_data_dir {
        let config = fs::read_to_string(repo.path().join("pytxo.toml")).unwrap();
        fs::write(
            repo.path().join("pytxo.toml"),
            format!("data_dir = '.pytxo/changed-data'\n{config}"),
        )
        .unwrap();
    }
    if late_terminal_stop {
        let store = PytxoStore::open(&PytxoConfig::default().db_path_at(repo.path())).unwrap();
        assert_eq!(store.get_run(&run_id).unwrap().unwrap().status, "completed");
    }
    let active_repo = pytxo_orchestrate::request_stop_experimental_routed_flow(
        &catalog,
        &reviewed.draft_id,
        &run_id,
    )
    .unwrap();
    if stage == "PYTXO_TEST_ROUTED_PAUSE_AFTER_RESERVE" || active_worker {
        let active_repo = active_repo.expect("reserved run has an exact active marker");
        let stop_result = tokio::runtime::Runtime::new()
            .unwrap()
            .block_on(stop_exact_routed(active_repo, &run_id));
        if active_worker {
            if let Err(error) = stop_result {
                assert!(
                    format!("{error:#}")
                        .contains("retains process or routed attempt ownership; recovery required"),
                    "unexpected Stop error: {error:#}"
                );
            }
        } else {
            stop_result.unwrap();
        }
    } else {
        assert!(active_repo.is_none());
    }
    if !active_worker {
        fs::write(pause.path().join("release"), b"").unwrap();
    }
    let dispatch_result = worker.join().unwrap();
    if late_terminal_stop {
        assert_eq!(dispatch_result.unwrap(), run_id);
        assert_eq!(
            catalog
                .get_flow_draft(&reviewed.draft_id)
                .unwrap()
                .unwrap()
                .status,
            "dispatched"
        );
        let store = PytxoStore::open(&PytxoConfig::default().db_path_at(repo.path())).unwrap();
        assert_eq!(store.get_run(&run_id).unwrap().unwrap().status, "completed");
        let scope = pytxo_store::routing::RoutingScope {
            domain_id: pytxo_core::DomainId(reviewed.domain_id),
            run_id: RunId(run_id),
        };
        assert!(!store.routing_history(&scope).unwrap().unwrap().cancelled);
        unsafe { std::env::remove_var(stage) };
        return;
    }
    if let Some((pid, start)) = registered_worker {
        let draft = catalog.get_flow_draft(&reviewed.draft_id).unwrap().unwrap();
        let store = PytxoStore::open(&PytxoConfig::default().db_path_at(repo.path())).unwrap();
        assert!(format!("{:#}", dispatch_result.unwrap_err()).contains("was cancelled by Stop"));
        assert_eq!(draft.status, "cancelled");
        assert_eq!(store.get_run(&run_id).unwrap().unwrap().status, "cancelled");
        let scope = pytxo_store::routing::RoutingScope {
            domain_id: pytxo_core::DomainId(reviewed.domain_id.clone()),
            run_id: RunId(run_id.clone()),
        };
        assert!(store.routing_history(&scope).unwrap().unwrap().cancelled);
        assert!(store
            .owned_job_stop_snapshot(&scope.domain_id, Some(&scope.run_id))
            .unwrap()
            .is_empty());
        assert!(!pytxo_runner::process_matches(pid, &start).unwrap());
        assert_eq!(fs::read(repo.path().join("result.txt")).unwrap(), b"");
        unsafe { std::env::remove_var("PYTXO_TEST_ROUTED_SLOW_SIBLINGS") };
        return;
    }
    assert!(dispatch_result.is_err());
    let draft = catalog.get_flow_draft(&reviewed.draft_id).unwrap().unwrap();
    assert_eq!(draft.status, "cancelled");
    let store = PytxoStore::open(&PytxoConfig::default().db_path_at(repo.path())).unwrap();
    let scope = pytxo_store::routing::RoutingScope {
        domain_id: pytxo_core::DomainId(reviewed.domain_id),
        run_id: RunId(run_id.clone()),
    };
    assert!(store.routing_history(&scope).unwrap().is_none());
    assert!(!repo.path().join(".pytxo/data/active_run.json").exists());
    assert_eq!(
        catalog
            .capacity_pool_status("host")
            .unwrap()
            .unwrap()
            .available_units,
        1
    );
    unsafe { std::env::remove_var(stage) };
}

#[cfg(feature = "routed-test-faults")]
#[test]
fn stop_during_preflight_prevents_all_probes_and_attempts() {
    stop_during_routed_startup("PYTXO_TEST_ROUTED_PAUSE_BEFORE_PROBES", false);
}

#[cfg(feature = "routed-test-faults")]
#[test]
fn stop_between_reserve_and_registration_never_registers_a_mission() {
    stop_during_routed_startup("PYTXO_TEST_ROUTED_PAUSE_AFTER_RESERVE", false);
}

#[cfg(feature = "routed-test-faults")]
#[test]
fn stop_uses_original_data_directory_after_config_changes() {
    stop_during_routed_startup("PYTXO_TEST_ROUTED_PAUSE_AFTER_RESERVE", true);
}

#[cfg(feature = "routed-test-faults")]
#[test]
fn stop_terminates_registered_worker_after_config_data_dir_changes() {
    stop_during_routed_startup("ACTIVE_WORKER_CONFIG_DRIFT", true);
}

#[cfg(feature = "routed-test-faults")]
#[test]
fn late_stop_after_terminal_run_does_not_reclassify_completion() {
    stop_during_routed_startup("PYTXO_TEST_ROUTED_PAUSE_AFTER_TERMINAL_BEFORE_ACK", false);
}

#[cfg(feature = "routed-test-faults")]
#[test]
fn stop_during_owned_probe_quiesces_before_any_run_or_attempt() {
    stop_during_routed_startup("PYTXO_TEST_ROUTED_PAUSE_PROBE_REGISTER", false);
}

fn run_reviewed_fixture_with_config(
    expected_state: AttemptState,
    expected_run_status: &str,
    overlay_config: bool,
) {
    let home = tempfile::tempdir().unwrap();
    #[cfg(feature = "routed-test-faults")]
    let hosted = std::env::var("PYTXO_TEST_ROUTED_HOSTED_OWNED")
        .ok()
        .as_deref()
        == Some("1");
    #[cfg(feature = "routed-test-faults")]
    if hosted {
        prepare_hosted_payload(home.path());
    }
    unsafe {
        std::env::set_var("PYTXO_HOME", home.path().join("home"));
        std::env::set_var("PYTXO_TRUST_STORE", home.path().join("trust.json"));
        std::env::set_var("PYTXO_EXPERIMENTAL_ROUTED_LOCAL_FIXTURE", "1");
    }
    let repo = tempfile::tempdir().unwrap();
    fs::create_dir_all(repo.path().join("src")).unwrap();
    fs::write(repo.path().join("src/lib.rs"), "pub fn value() {}\n").unwrap();
    fs::write(repo.path().join("result.txt"), "").unwrap();
    let config_text = if overlay_config {
        "permission_profile = 'orbit'\nisolation = 'overlay'\nexecution_backend = 'subprocess'\nmax_agents = 1\n[blast]\nprefer_kernel_overlay = false\n[[task]]\nid = 'fixture'\nagent = 'default'\npaths = ['result.txt']\n"
    } else {
        "permission_profile = 'orbit'\nexecution_backend = 'subprocess'\nmax_agents = 1\n[blast]\nprefer_kernel_overlay = false\n[[task]]\nid = 'fixture'\nagent = 'default'\npaths = ['result.txt']\n"
    };
    fs::write(repo.path().join("pytxo.toml"), config_text).unwrap();
    for args in [
        vec!["init", "-q"],
        vec!["add", "."],
        vec![
            "-c",
            "user.name=Test",
            "-c",
            "user.email=test@pytxo.local",
            "commit",
            "-qm",
            "baseline",
        ],
    ] {
        let output = Command::new("git")
            .args(args)
            .current_dir(repo.path())
            .output()
            .unwrap();
        assert!(
            output.status.success(),
            "{}",
            String::from_utf8_lossy(&output.stderr)
        );
    }
    trust_repo(repo.path(), PermissionProfile::Orbit).unwrap();
    let catalog = Catalog::open(&repo.path().join(".git/catalog.db")).unwrap();
    catalog
        .configure_capacity_pool(&CapacityPoolConfig {
            resource_id: "host".into(),
            capacity_units: 1,
            expected_revision: None,
            configured_at_ms: 100,
        })
        .unwrap();
    let preview = preview_experimental_routed_flow(
        &catalog,
        FlowDraftInput {
            id: "routed-local-fixture".into(),
            title: "Routed local fixture".into(),
            mission_text: "pytxo-local-fixture-v1:write-result".into(),
            source: FlowSource::Text,
            domain_id: Some(repo.path().to_string_lossy().into_owned()),
            project_id: None,
            ade_id: None,
            max_workers: Some(1),
            verification_commands: vec!["if exist result.txt (exit /b 0) else (exit /b 1)".into()],
        },
        mission,
    )
    .unwrap();
    assert_eq!(
        preview.tasks[0].prompt,
        "pytxo-local-fixture-v1:write-result"
    );
    assert_eq!(preview.tasks[0].paths, ["result.txt"]);
    let reviewed = save_reviewed_flow_plan(&catalog, preview).unwrap();
    #[cfg(feature = "routed-test-faults")]
    let expected_mock_packet = if std::env::var_os("PYTXO_TEST_ROUTED_SHADOW_MOCK_ADVICE").is_some()
    {
        Some(
            pytxo_orchestrate::flow::preview_experimental_routed_advisor_packet(
                &catalog,
                &reviewed.draft_id,
            )
            .unwrap()
            .packet_digest,
        )
    } else {
        None
    };
    #[cfg(feature = "routed-test-faults")]
    if std::env::var_os("PYTXO_TEST_ROUTED_SHADOW_MOCK_ADVICE").is_some() {
        let store = PytxoStore::open(&PytxoConfig::default().db_path_at(repo.path())).unwrap();
        store
            .set_routing_advisor_consent(
                &pytxo_core::DomainId(reviewed.domain_id.clone()),
                0,
                true,
                Some(pytxo_orchestrate::flow::reviewed_routing_advisor_disclosure_scope_digest()),
                0,
            )
            .unwrap();
    }
    let before_status = catalog
        .get_flow_draft(&reviewed.draft_id)
        .unwrap()
        .unwrap()
        .status;
    let review_fault = std::env::var_os("PYTXO_TEST_ROUTED_FAIL_AFTER_REVIEW_BEGIN").is_some()
        || std::env::var_os("PYTXO_TEST_ROUTED_FAIL_BEFORE_REVIEW_VIEW").is_some();
    let catalog_stop_after_checker =
        std::env::var_os("PYTXO_TEST_ROUTED_CATALOG_STOP_AFTER_CHECKER_JOB_ZERO").is_some();
    #[cfg(feature = "routed-test-faults")]
    if std::env::var_os("PYTXO_TEST_ROUTED_CRASH_AFTER_ONE_ADMITTED").is_some() {
        assert!(catch_unwind(AssertUnwindSafe(|| {
            dispatch_experimental_routed_flow(&catalog, &reviewed.draft_id)
        }))
        .is_err());
        let run_id = reviewed
            .routing
            .as_ref()
            .unwrap()
            .authorization
            .run_id
            .0
            .clone();
        let scope = pytxo_store::routing::RoutingScope {
            domain_id: pytxo_core::DomainId(reviewed.domain_id.clone()),
            run_id: RunId(run_id.clone()),
        };
        let store = PytxoStore::open(&PytxoConfig::default().db_path_at(repo.path())).unwrap();
        let before = store.routing_history(&scope).unwrap().unwrap();
        assert_eq!(before.attempts.len(), 1);
        let attempt = &before.attempts[0];
        assert_eq!(attempt.state, AttemptState::Preparing);
        assert!(!attempt.ownership_released);
        assert_eq!(
            store
                .launch_ownership(&attempt.attempt_id)
                .unwrap()
                .unwrap()
                .phase,
            pytxo_store::routing_launch::LaunchOwnershipPhase::Prepared
        );
        catalog
            .mark_routed_flow_recovery_required(&reviewed.draft_id, &run_id)
            .unwrap();
        let owner = catalog
            .routed_flow_dispatch_owner(&reviewed.draft_id, &run_id)
            .unwrap()
            .unwrap();
        let marker_path = repo.path().join(".pytxo/data/active_run.json");
        let mut marker: serde_json::Value =
            serde_json::from_slice(&fs::read(&marker_path).unwrap()).unwrap();
        marker["supervisor_start_identity"] = "simulated-dead-controller".into();
        rusqlite::Connection::open(repo.path().join(".git/catalog.db"))
            .unwrap()
            .execute(
                "UPDATE flow_dispatch_claims SET controller_start_identity=?1 WHERE draft_id=?2 AND run_id=?3 AND controller_pid=?4",
                rusqlite::params!["simulated-dead-controller", reviewed.draft_id, run_id, owner.controller_pid],
            )
            .unwrap();
        assert!(!reconcile_routed_flow_startup(&catalog, &reviewed.draft_id).unwrap());
        fs::write(&marker_path, serde_json::to_vec(&marker).unwrap()).unwrap();
        assert!(reconcile_routed_flow_startup(&catalog, &reviewed.draft_id).unwrap());
        let recovered = store.routing_history(&scope).unwrap().unwrap();
        assert!(recovered.cancelled);
        assert_eq!(recovered.attempts.len(), 1);
        assert_eq!(recovered.attempts[0].state, AttemptState::FailedNoLaunch);
        assert!(recovered.attempts[0].ownership_released);
        assert!(matches!(
            recovered.attempts[0].usage,
            pytxo_store::routing::RoutedUsage::Known { nano_usd: 0, .. }
        ));
        assert_eq!(store.get_run_status(&run_id).unwrap().unwrap().0, "failed");
        assert!(!marker_path.exists());
        assert!(store
            .unresolved_capacity_intent_scopes()
            .unwrap()
            .is_empty());
        let event_count = recovered.events.len();
        let _ = reconcile_routed_flow_startup(&catalog, &reviewed.draft_id).unwrap();
        let replayed = store.routing_history(&scope).unwrap().unwrap();
        assert_eq!(replayed.events.len(), event_count);
        assert_eq!(replayed.attempts[0].state, AttemptState::FailedNoLaunch);
        return;
    }
    let dispatch = dispatch_experimental_routed_flow(&catalog, &reviewed.draft_id);
    #[cfg(feature = "routed-test-faults")]
    if std::env::var_os("PYTXO_TEST_ROUTED_EXPIRE_AFTER_CREATE").is_some() {
        let authorization = &reviewed.routing.as_ref().unwrap().authorization;
        if let Ok(dispatched) = &dispatch {
            assert_eq!(dispatched, &authorization.run_id.0);
        }
        let scope = pytxo_store::routing::RoutingScope {
            domain_id: pytxo_core::DomainId(reviewed.domain_id.clone()),
            run_id: authorization.run_id.clone(),
        };
        let store = PytxoStore::open(&PytxoConfig::default().db_path_at(repo.path())).unwrap();
        let history = store.routing_history(&scope).unwrap().unwrap();
        assert_eq!(history.attempts.len(), 1);
        let attempt = &history.attempts[0];
        let owner = store
            .launch_ownership(&attempt.attempt_id)
            .unwrap()
            .unwrap();
        assert_eq!(
            owner.phase,
            pytxo_store::routing_launch::LaunchOwnershipPhase::CreateMayHaveStarted,
            "dispatch={dispatch:?}; attempt_state={:?}; attempt_failure={:?}",
            attempt.state,
            attempt.failure
        );
        assert!(owner.pid.is_none() && owner.settlement_blob.is_none());
        assert_eq!(attempt.state, AttemptState::Launching);
        assert!(!attempt.ownership_released);
        assert!(attempt.receipts.process_identity.is_none());
        assert_eq!(
            catalog
                .capacity_reservation(&attempt.capacity_reservation)
                .unwrap()
                .unwrap()
                .state,
            pytxo_store::capacity::CapacityReservationState::Bound
        );
        assert!(fs::read(repo.path().join("result.txt")).unwrap().is_empty());
        return;
    }
    if overlay_config {
        assert!(
            dispatch.is_err(),
            "overlay cannot claim a worktree-only routed run"
        );
        let run_id = &reviewed.routing.as_ref().unwrap().authorization.run_id.0;
        let store = PytxoStore::open(&PytxoConfig::default().db_path_at(repo.path())).unwrap();
        assert!(store.get_run(run_id).unwrap().is_none());
        assert!(store
            .routing_history(&pytxo_store::routing::RoutingScope {
                domain_id: pytxo_core::DomainId(reviewed.domain_id.clone()),
                run_id: RunId(run_id.clone()),
            })
            .unwrap()
            .is_none());
        assert!(!repo
            .path()
            .join(&PytxoConfig::default().data_dir)
            .join("active_run.json")
            .exists());
        assert_eq!(
            catalog
                .get_flow_draft(&reviewed.draft_id)
                .unwrap()
                .unwrap()
                .status,
            before_status
        );
        return;
    }
    let run_id = if review_fault || catalog_stop_after_checker {
        assert!(dispatch.is_err());
        eprintln!(
            "routed Review fault dispatch: {:#}",
            dispatch.as_ref().unwrap_err()
        );
        reviewed
            .routing
            .as_ref()
            .unwrap()
            .authorization
            .run_id
            .0
            .clone()
    } else {
        let run_id = dispatch.unwrap();
        assert_eq!(
            catalog
                .get_flow_draft(&reviewed.draft_id)
                .unwrap()
                .unwrap()
                .status,
            "dispatched"
        );
        run_id
    };
    let store = PytxoStore::open(&PytxoConfig::default().db_path_at(repo.path())).unwrap();
    let contract = store.get_run_contract(&run_id).unwrap().unwrap();
    assert_eq!(
        contract.base_revision.as_deref(),
        Some(
            observe_experimental_routed_git_base(repo.path())
                .unwrap()
                .git_revision
                .as_str()
        )
    );
    if review_fault {
        assert_eq!(contract.apply_status, "review_failed");
        assert!(contract.prepared_manifest.is_none());
    } else if expected_state == AttemptState::Passed {
        assert_eq!(contract.apply_status, "ready");
        let manifest = contract.prepared_manifest.as_ref().unwrap();
        assert_eq!(manifest.files.len(), 1);
        assert_eq!(manifest.files[0].path, "result.txt");
        assert!(manifest.candidate_verification.is_some());
    } else {
        assert_eq!(contract.apply_status, "pending");
        assert!(contract.prepared_manifest.is_none());
    }
    let plan_json: serde_json::Value =
        serde_json::from_str(contract.plan_json.as_deref().unwrap()).unwrap();
    assert_eq!(plan_json["waves"][0][0]["task_id"], reviewed.tasks[0].id);
    let enforcement: serde_json::Value =
        serde_json::from_str(contract.enforcement_json.as_deref().unwrap()).unwrap();
    assert_eq!(enforcement["run"]["effective_profile"], "orbit");
    let scope = pytxo_store::routing::RoutingScope {
        domain_id: pytxo_core::DomainId(reviewed.domain_id.clone()),
        run_id: RunId(run_id.clone()),
    };
    let history = store.routing_history(&scope).unwrap().unwrap();
    assert_eq!(history.attempts.len(), 1);
    #[cfg(feature = "routed-test-faults")]
    if hosted {
        let attempt = &history.attempts[0];
        let launch = attempt.selected.observation.launch.as_ref().unwrap();
        assert_eq!(launch.transport, LaunchTransport::HostSubprocess);
        assert_eq!(launch.stdin_delivery, StdinDelivery::PrivateHostPipe);
        assert_eq!(launch.output_protocol, "pytxo-attempt-host/1");
        let prompt = pytxo_orchestrate::routed_prompt::render_private_attempt_prompt(
            pytxo_orchestrate::routed_prompt::RoutedPromptInput {
                task: &history.tasks[0].registration.contract,
                attempt_id: &attempt.attempt_id,
                ordinal: 1,
                dependencies: &[],
                previous: None,
                observed_check: None,
            },
        )
        .unwrap();
        assert_eq!(launch.private_stdin_digest, Some(Digest::of_bytes(&prompt)));
        assert_ne!(
            attempt.selected.observation.executable.digest,
            history.tasks[0].registration.check_recipes[0]
                .executor
                .shell
                .digest,
            "hosted payload must be distinct from the independently pinned checker"
        );
        let owner = store
            .launch_ownership(&attempt.attempt_id)
            .unwrap()
            .unwrap();
        assert_eq!(
            owner.phase,
            pytxo_store::routing_launch::LaunchOwnershipPhase::Settled
        );
        assert!(owner.pid.is_some_and(|pid| pid > 0));
        assert!(owner.start_identity.is_some());
        assert!(owner.settlement_blob.is_some());
        assert!(attempt.receipts.process_identity.is_some());
        assert!(attempt.receipts.quiescence.is_some());
        let checker = store
            .checker_ownership(&attempt.attempt_id, 1)
            .unwrap()
            .unwrap();
        assert_eq!(checker.passed, Some(true));
        assert!(fs::read(repo.path().join("result.txt")).unwrap().is_empty());
    }
    let trace = store
        .routing_benchmark_trace(
            &scope,
            &history.tasks[0].registration.contract.task_id,
            &[7; 32],
        )
        .unwrap()
        .unwrap();
    assert!(trace.decision_links_complete);
    assert_eq!(trace.attempts.len(), 1);
    let stop_event_recorded = history.events.iter().any(|entry| {
        entry.event_id == "controller.stop.cancel.v1"
            && matches!(
                entry.event,
                pytxo_store::routing::RoutingControlEvent::Cancelled { .. }
            )
    });
    let launch_phase = store
        .launch_ownership(&history.attempts[0].attempt_id)
        .unwrap()
        .map(|owner| owner.phase);
    let checker_phase = store
        .checker_ownership(&history.attempts[0].attempt_id, 1)
        .unwrap()
        .map(|owner| owner.phase);
    assert_eq!(
        trace.attempts[0].state,
        expected_state,
        "stop_after_checker_prepared={}, stop_after_checker_job_zero={}, fail_after_checker_prepared={}, mission_cancelled={}, stop_event_recorded={}, launch_phase={launch_phase:?}, checker_phase={checker_phase:?}, receipts={:?}, failure={:?}",
        std::env::var_os("PYTXO_TEST_ROUTED_STOP_AFTER_CHECKER_PREPARED").is_some(),
        std::env::var_os("PYTXO_TEST_ROUTED_STOP_AFTER_CHECKER_JOB_ZERO").is_some(),
        std::env::var_os("PYTXO_TEST_ROUTED_FAIL_AFTER_CHECKER_PREPARED").is_some(),
        history.cancelled,
        stop_event_recorded,
        history.attempts[0].receipts,
        history.attempts[0].failure,
    );
    assert!(trace.events.iter().any(|entry| matches!(
        entry.kind,
        pytxo_store::routing::RoutingBenchmarkEventKind::DecisionObserved { .. }
    )));
    assert!(trace.events.iter().any(|entry| matches!(
        entry.kind,
        pytxo_store::routing::RoutingBenchmarkEventKind::Admitted { .. }
    )));
    #[cfg(feature = "routed-test-faults")]
    if std::env::var_os("PYTXO_TEST_ROUTED_SHADOW_FAKE_ADVICE").is_some() {
        // The reviewed policy is now Jev-specific; this unjournaled synthetic
        // reply must be discarded before it can claim a Shadow observation.
        assert_eq!(history.attempts[0].selected.profile.id.0, "strong");
        assert_eq!(
            history.attempts[0].decision.reason,
            RouteReason::StrongDefault
        );
        assert_eq!(
            history.attempts[0].decision.advice_status,
            AdviceStatus::NotUsed
        );
        assert!(history.events.iter().any(|entry| matches!(
            &entry.event,
            pytxo_store::routing::RoutingControlEvent::DecisionObserved(observed)
                if observed.shadow_choice.is_none()
                    && observed.advice_digest.is_none()
                    && observed.advice_request_id.is_none()
        )));
    }
    #[cfg(feature = "routed-test-faults")]
    if std::env::var_os("PYTXO_TEST_ROUTED_SHADOW_MOCK_ADVICE").is_some() {
        assert_eq!(history.attempts[0].selected.profile.id.0, "strong");
        assert_eq!(
            history.attempts[0].decision.reason,
            RouteReason::StrongDefault
        );
        assert_eq!(
            history.attempts[0].decision.advice_status,
            AdviceStatus::ShadowRecorded
        );
        let observed = history
            .events
            .iter()
            .find_map(|entry| match &entry.event {
                pytxo_store::routing::RoutingControlEvent::DecisionObserved(observed)
                    if observed.shadow_choice == Some(AdviceChoice::EverydayFit) =>
                {
                    Some(observed)
                }
                _ => None,
            })
            .unwrap();
        let request_id = observed.advice_request_id.as_ref().unwrap();
        let journal = store
            .routing_advisor_request(&scope, request_id)
            .unwrap()
            .unwrap();
        assert_eq!(
            journal.phase,
            pytxo_store::routing::AdvisorSendPhase::Completed
        );
        assert_eq!(
            journal.result_digest.as_ref(),
            observed.advice_digest.as_ref()
        );
        assert_eq!(journal.packet_digest, expected_mock_packet.unwrap());
    }
    assert_eq!(history.attempts[0].state, expected_state);
    assert_eq!(
        history.tasks[0].winner.is_some(),
        expected_state == AttemptState::Passed
    );
    assert_eq!(
        store.get_run_status(&run_id).unwrap().unwrap().0,
        expected_run_status
    );
    if !review_fault {
        assert!(!store
            .unreconciled_registered_routing_scopes()
            .unwrap()
            .contains(&scope));
    }
    let reservation = catalog
        .capacity_reservation(&history.attempts[0].capacity_reservation)
        .unwrap()
        .unwrap();
    assert_eq!(
        reservation.state,
        pytxo_store::capacity::CapacityReservationState::Released
    );
    assert_eq!(
        store
            .capacity_intent(&history.attempts[0].capacity_reservation)
            .unwrap()
            .unwrap()
            .phase,
        pytxo_store::routing_capacity_intent::CapacityIntentPhase::Closed
    );
    assert!(matches!(
        &history.attempts[0].usage,
        pytxo_store::routing::RoutedUsage::Known { nano_usd: 0, .. }
    ));
    if expected_state == AttemptState::Passed {
        let checker = store
            .checker_ownership(&history.attempts[0].attempt_id, 1)
            .unwrap()
            .unwrap();
        assert_eq!(
            checker.phase,
            pytxo_store::routing_checker::CheckerOwnershipPhase::Settled
        );
        assert_eq!(checker.passed, Some(true));
    }
    if expected_state == AttemptState::FailedNoLaunch {
        assert_eq!(
            store
                .launch_ownership(&history.attempts[0].attempt_id)
                .unwrap()
                .unwrap()
                .phase,
            pytxo_store::routing_launch::LaunchOwnershipPhase::ClosedNoLaunch
        );
    }
    if std::env::var_os("PYTXO_TEST_ROUTED_STOP_AFTER_CHECKER_JOB_ZERO").is_some()
        || catalog_stop_after_checker
    {
        let checker = store
            .checker_ownership(&history.attempts[0].attempt_id, 1)
            .unwrap()
            .unwrap();
        assert_eq!(
            checker.phase,
            pytxo_store::routing_checker::CheckerOwnershipPhase::Settled
        );
        assert_eq!(checker.passed, Some(true));
    } else if expected_state == AttemptState::Cancelled
        || std::env::var_os("PYTXO_TEST_ROUTED_FAIL_AFTER_CHECKER_PREPARED").is_some()
    {
        let checker = store
            .checker_ownership(&history.attempts[0].attempt_id, 1)
            .unwrap()
            .unwrap();
        assert_eq!(
            checker.phase,
            pytxo_store::routing_checker::CheckerOwnershipPhase::ClosedNoCreate
        );
    }
    if expected_state == AttemptState::Passed && !review_fault {
        let digest = contract.prepared_manifest.unwrap().package_digest;
        assert!(apply_run_changes(
            None,
            Some(repo.path().to_path_buf()),
            &run_id,
            "unreviewed-package",
        )
        .is_err());
        assert!(fs::read(repo.path().join("result.txt")).unwrap().is_empty());
        let alternate_config = home.path().join("alternate-pytxo.toml");
        fs::write(
            &alternate_config,
            config_text.replacen(
                "max_agents = 1\n",
                "max_agents = 1\nworktree_dir = '.pytxo/alternate-worktrees'\n",
                1,
            ),
        )
        .unwrap();
        let applied = apply_run_changes(
            Some(alternate_config),
            Some(repo.path().to_path_buf()),
            &run_id,
            &digest,
        )
        .unwrap();
        assert_eq!(applied.changes.len(), 1);
        assert_eq!(
            fs::read(repo.path().join("result.txt")).unwrap(),
            b"pytxo-routed\r\n"
        );
        assert_eq!(
            store
                .get_run_contract(&run_id)
                .unwrap()
                .unwrap()
                .apply_status,
            "applied"
        );
    }
    if expected_state == AttemptState::Passed && review_fault {
        if std::env::var_os("PYTXO_TEST_ROUTED_DISCARD_BEFORE_REVIEW_VIEW").is_some() {
            let actor = store.list_agents_for_run(&run_id).unwrap().remove(0);
            let original = std::path::PathBuf::from(actor.worktree_path.unwrap());
            assert!(original.exists());
            assert!(store
                .routed_original_worktree(&actor.id, &run_id)
                .unwrap()
                .is_none());
            let replacement =
                std::env::var_os("PYTXO_TEST_ROUTED_REPLACE_ORIGINAL_WORKTREE").is_some();
            let same_base_replacement =
                std::env::var_os("PYTXO_TEST_ROUTED_REPLACE_SAME_BASE").is_some();
            assert!(!same_base_replacement || replacement);
            if replacement {
                let mut operations = vec![vec![
                    "worktree",
                    "remove",
                    "--force",
                    original.to_str().unwrap(),
                ]];
                if !same_base_replacement {
                    operations.push(vec![
                        "-c",
                        "user.name=Test",
                        "-c",
                        "user.email=test@pytxo.local",
                        "commit",
                        "--allow-empty",
                        "-qm",
                        "different-base",
                    ]);
                }
                operations.push(vec![
                    "worktree",
                    "add",
                    "--detach",
                    original.to_str().unwrap(),
                    "HEAD",
                ]);
                for args in operations {
                    let output = Command::new("git")
                        .args(args)
                        .current_dir(repo.path())
                        .output()
                        .unwrap();
                    assert!(
                        output.status.success(),
                        "{}",
                        String::from_utf8_lossy(&output.stderr)
                    );
                }
                fs::write(original.join("sentinel.txt"), b"different-worktree").unwrap();
            }
            let alternate_config = home.path().join("alternate-pytxo.toml");
            fs::write(
                &alternate_config,
                config_text.replacen(
                    "max_agents = 1\n",
                    "max_agents = 1\nworktree_dir = '.pytxo/alternate-worktrees'\n",
                    1,
                ),
            )
            .unwrap();
            discard_run_review(
                Some(alternate_config),
                Some(repo.path().to_path_buf()),
                &run_id,
            )
            .unwrap();
            if replacement {
                assert_eq!(
                    fs::read(original.join("sentinel.txt")).unwrap(),
                    b"different-worktree",
                    "Discard must preserve a replacement worktree at the original path"
                );
            } else {
                assert!(!original.exists(), "Discard must retire the owned worktree");
            }
            assert_eq!(
                store
                    .get_run_contract(&run_id)
                    .unwrap()
                    .unwrap()
                    .apply_status,
                "discarded"
            );
            return;
        }
        assert_eq!(
            catalog
                .get_flow_draft(&reviewed.draft_id)
                .unwrap()
                .unwrap()
                .status,
            "dispatched",
            "a Passed winner with failed Review must have a recoverable Flow identity"
        );
        let actor = store.list_agents_for_run(&run_id).unwrap().remove(0);
        fs::write(
            std::path::Path::new(actor.worktree_path.as_deref().unwrap()).join("result.txt"),
            b"attacker-substitution",
        )
        .unwrap();
        let changed_backend = home.path().join("changed-backend.toml");
        fs::write(
            &changed_backend,
            config_text.replacen(
                "execution_backend = 'subprocess'",
                "execution_backend = 'pty'",
                1,
            ),
        )
        .unwrap();
        assert!(refresh_run_review(
            Some(changed_backend),
            Some(repo.path().to_path_buf()),
            &run_id,
        )
        .unwrap_err()
        .to_string()
        .contains("execution backend changed"));
        assert_eq!(
            store
                .get_run_contract(&run_id)
                .unwrap()
                .unwrap()
                .apply_status,
            "review_failed"
        );
        let refreshed = refresh_run_review(None, Some(repo.path().to_path_buf()), &run_id).unwrap();
        assert_eq!(refreshed.files.len(), 1);
        assert_eq!(refreshed.files[0].path, "result.txt");
        let refreshed_actor = store.get_agent(&actor.id).unwrap().unwrap();
        assert_ne!(refreshed_actor.worktree_path, actor.worktree_path);
        assert_eq!(
            fs::read(
                std::path::Path::new(refreshed_actor.worktree_path.as_deref().unwrap())
                    .join("result.txt")
            )
            .unwrap(),
            b"pytxo-routed\r\n"
        );
        let original = repo
            .path()
            .join(&PytxoConfig::default().worktree_dir)
            .join(&run_id)
            .join(&history.attempts[0].attempt_id.0);
        let recorded = store
            .routed_original_worktree(&actor.id, &run_id)
            .unwrap()
            .unwrap();
        assert_eq!(
            fs::canonicalize(recorded).unwrap(),
            fs::canonicalize(original).unwrap()
        );
        assert_eq!(
            store
                .get_run_contract(&run_id)
                .unwrap()
                .unwrap()
                .apply_status,
            "ready"
        );
        let applied = apply_run_changes(
            None,
            Some(repo.path().to_path_buf()),
            &run_id,
            &refreshed.package_digest,
        )
        .unwrap();
        assert_eq!(applied.changes.len(), 1);
        assert_eq!(
            fs::read(repo.path().join("result.txt")).unwrap(),
            b"pytxo-routed\r\n"
        );
    }
    if expected_state == AttemptState::Passed {
        let original = repo
            .path()
            .join(&PytxoConfig::default().worktree_dir)
            .join(&run_id)
            .join(&history.attempts[0].attempt_id.0);
        assert!(
            !original.exists(),
            "Apply must retire the exact owned worktree"
        );
        let views = repo
            .path()
            .join(&PytxoConfig::default().data_dir)
            .join("routed-verification")
            .join(&run_id);
        assert!(!views.exists(), "Apply must retire private routed views");
    }
    #[cfg(feature = "routed-test-faults")]
    if hosted {
        unsafe { std::env::remove_var("PYTXO_TEST_ROUTED_HOSTED_PAYLOAD") };
    }
}

#[cfg(feature = "routed-test-faults")]
#[test]
fn dead_controller_closes_one_prepared_attempt_without_native_create() {
    let _env_lock = FIXTURE_ENV_LOCK.lock().unwrap();
    unsafe { std::env::set_var("PYTXO_TEST_ROUTED_CRASH_AFTER_ONE_ADMITTED", "1") };
    run_reviewed_fixture(AttemptState::FailedNoLaunch, "failed");
    unsafe { std::env::remove_var("PYTXO_TEST_ROUTED_CRASH_AFTER_ONE_ADMITTED") };
}

#[test]
fn reviewed_local_fixture_executes_and_releases_one_owned_attempt() {
    let _env_lock = FIXTURE_ENV_LOCK.lock().unwrap();
    run_reviewed_fixture(AttemptState::Passed, "completed");
    #[cfg(feature = "routed-test-faults")]
    {
        unsafe { std::env::set_var("PYTXO_TEST_ROUTED_FAIL_AFTER_WORKTREE_PREPARED", "1") };
        run_reviewed_fixture(AttemptState::FailedNoLaunch, "failed");
        unsafe { std::env::remove_var("PYTXO_TEST_ROUTED_FAIL_AFTER_WORKTREE_PREPARED") };
        unsafe { std::env::set_var("PYTXO_TEST_ROUTED_FAIL_AFTER_JOB_ZERO", "1") };
        run_reviewed_fixture(AttemptState::Failed, "failed");
        unsafe { std::env::remove_var("PYTXO_TEST_ROUTED_FAIL_AFTER_JOB_ZERO") };
        unsafe { std::env::set_var("PYTXO_TEST_ROUTED_FAIL_AFTER_CHECKER_PREPARED", "1") };
        run_reviewed_fixture(AttemptState::Failed, "failed");
        unsafe { std::env::remove_var("PYTXO_TEST_ROUTED_FAIL_AFTER_CHECKER_PREPARED") };
        unsafe { std::env::set_var("PYTXO_TEST_ROUTED_STOP_AFTER_CHECKER_PREPARED", "1") };
        run_reviewed_fixture(AttemptState::Cancelled, "cancelled");
        unsafe { std::env::remove_var("PYTXO_TEST_ROUTED_STOP_AFTER_CHECKER_PREPARED") };
        unsafe { std::env::set_var("PYTXO_TEST_ROUTED_STOP_AFTER_CHECKER_JOB_ZERO", "1") };
        run_reviewed_fixture(AttemptState::Cancelled, "cancelled");
        unsafe { std::env::remove_var("PYTXO_TEST_ROUTED_STOP_AFTER_CHECKER_JOB_ZERO") };
        unsafe { std::env::set_var("PYTXO_TEST_ROUTED_FAIL_AFTER_WORKER_REGISTERED", "1") };
        run_reviewed_fixture(AttemptState::Failed, "failed");
        unsafe { std::env::remove_var("PYTXO_TEST_ROUTED_FAIL_AFTER_WORKER_REGISTERED") };
        unsafe { std::env::set_var("PYTXO_TEST_ROUTED_SHADOW_FAKE_ADVICE", "1") };
        run_reviewed_fixture(AttemptState::Passed, "completed");
        unsafe { std::env::remove_var("PYTXO_TEST_ROUTED_SHADOW_FAKE_ADVICE") };
        unsafe { std::env::set_var("PYTXO_TEST_ROUTED_FAIL_FLOW_ACK", "1") };
        run_reviewed_fixture(AttemptState::Passed, "completed");
        unsafe { std::env::set_var("PYTXO_TEST_ROUTED_FAIL_AFTER_JOB_ZERO", "1") };
        run_reviewed_fixture(AttemptState::Failed, "failed");
        unsafe { std::env::remove_var("PYTXO_TEST_ROUTED_FAIL_AFTER_JOB_ZERO") };
        unsafe { std::env::remove_var("PYTXO_TEST_ROUTED_FAIL_FLOW_ACK") };
    }
}

#[cfg(feature = "routed-test-faults")]
#[test]
fn catalog_stop_after_checker_job_zero_blocks_winner_publication() {
    let _env_lock = FIXTURE_ENV_LOCK.lock().unwrap();
    unsafe {
        std::env::set_var("PYTXO_TEST_ROUTED_CATALOG_STOP_AFTER_CHECKER_JOB_ZERO", "1");
    }
    run_reviewed_fixture(AttemptState::Cancelled, "cancelled");
    unsafe {
        std::env::remove_var("PYTXO_TEST_ROUTED_CATALOG_STOP_AFTER_CHECKER_JOB_ZERO");
    }
}

#[cfg(feature = "routed-test-faults")]
#[test]
fn expired_precreate_budget_closes_prepared_owner_and_releases_capacity() {
    let _env_lock = FIXTURE_ENV_LOCK.lock().unwrap();
    unsafe { std::env::set_var("PYTXO_TEST_ROUTED_EXPIRE_BEFORE_CREATE", "1") };
    run_reviewed_fixture(AttemptState::FailedNoLaunch, "failed");
    unsafe { std::env::remove_var("PYTXO_TEST_ROUTED_EXPIRE_BEFORE_CREATE") };
}

#[cfg(feature = "routed-test-faults")]
#[test]
fn expired_postcreate_budget_retains_recovery_ownership_without_launch() {
    let _env_lock = FIXTURE_ENV_LOCK.lock().unwrap();
    unsafe { std::env::set_var("PYTXO_TEST_ROUTED_EXPIRE_AFTER_CREATE", "1") };
    run_reviewed_fixture(AttemptState::Launching, "recovery_required");
    unsafe { std::env::remove_var("PYTXO_TEST_ROUTED_EXPIRE_AFTER_CREATE") };
}

#[cfg(feature = "routed-test-faults")]
#[test]
#[ignore = "requires the separate test-only embedded host in PYTXO_TEST_EMBEDDED_HOST"]
fn synthetic_hosted_owned_attempt_reaches_review_and_apply() {
    let _env_lock = FIXTURE_ENV_LOCK.lock().unwrap();
    let host = std::path::PathBuf::from(
        std::env::var_os("PYTXO_TEST_EMBEDDED_HOST")
            .expect("set PYTXO_TEST_EMBEDDED_HOST to the separate test-only embedded host"),
    );
    assert!(host.is_absolute() && host.is_file());
    assert!(host
        .file_name()
        .and_then(|name| name.to_str())
        .is_some_and(|name| name.eq_ignore_ascii_case("pytxo-embedded-hosted-test.exe")));
    unsafe { std::env::set_var("PYTXO_TEST_ROUTED_HOSTED_OWNED", "1") };
    run_reviewed_fixture(AttemptState::Passed, "completed");
    unsafe { std::env::remove_var("PYTXO_TEST_ROUTED_HOSTED_OWNED") };
}

#[test]
fn failed_everyday_worker_retries_once_on_strong_and_prepares_review() {
    let _env_lock = FIXTURE_ENV_LOCK.lock().unwrap();
    run_repair_fixture(false, false);
}

#[cfg(feature = "routed-test-faults")]
#[test]
fn failed_review_after_strong_repair_refreshes_from_exact_winner() {
    let _env_lock = FIXTURE_ENV_LOCK.lock().unwrap();
    unsafe { std::env::set_var("PYTXO_TEST_ROUTED_FAIL_BEFORE_REVIEW_VIEW", "1") };
    run_repair_fixture(true, false);
}

#[cfg(feature = "routed-test-faults")]
#[test]
fn dead_controller_after_repairable_failure_fences_second_attempt() {
    let _env_lock = FIXTURE_ENV_LOCK.lock().unwrap();
    unsafe { std::env::set_var("PYTXO_TEST_ROUTED_CRASH_AFTER_REPAIRABLE_FAILURE", "1") };
    run_repair_fixture(false, true);
    unsafe { std::env::remove_var("PYTXO_TEST_ROUTED_CRASH_AFTER_REPAIRABLE_FAILURE") };
}

fn run_repair_fixture(review_fault: bool, crash_after_failure: bool) {
    let home = tempfile::tempdir().unwrap();
    unsafe {
        std::env::set_var("PYTXO_HOME", home.path().join("home"));
        std::env::set_var("PYTXO_TRUST_STORE", home.path().join("trust.json"));
        std::env::set_var("PYTXO_EXPERIMENTAL_ROUTED_LOCAL_FIXTURE", "1");
    }
    let repo = tempfile::tempdir().unwrap();
    fs::write(repo.path().join("result.txt"), "").unwrap();
    fs::write(
        repo.path().join("pytxo.toml"),
        "permission_profile = 'orbit'\nexecution_backend = 'subprocess'\nmax_agents = 1\n[blast]\nprefer_kernel_overlay = false\n[[task]]\nid = 'fixture'\nagent = 'default'\npaths = ['result.txt']\n",
    )
    .unwrap();
    for args in [
        vec!["init", "-q"],
        vec!["add", "."],
        vec![
            "-c",
            "user.name=Test",
            "-c",
            "user.email=test@pytxo.local",
            "commit",
            "-qm",
            "baseline",
        ],
    ] {
        let output = Command::new("git")
            .args(args)
            .current_dir(repo.path())
            .output()
            .unwrap();
        assert!(
            output.status.success(),
            "{}",
            String::from_utf8_lossy(&output.stderr)
        );
    }
    trust_repo(repo.path(), PermissionProfile::Orbit).unwrap();
    let catalog = Catalog::open(&repo.path().join(".git/catalog.db")).unwrap();
    catalog
        .configure_capacity_pool(&CapacityPoolConfig {
            resource_id: "host".into(),
            capacity_units: 1,
            expected_revision: None,
            configured_at_ms: 100,
        })
        .unwrap();
    let preview = preview_experimental_routed_flow(
        &catalog,
        FlowDraftInput {
            id: "routed-local-repair-fixture".into(),
            title: "Routed local repair fixture".into(),
            mission_text: "pytxo-local-fixture-v1:write-result-with-repair".into(),
            source: FlowSource::Text,
            domain_id: Some(repo.path().to_string_lossy().into_owned()),
            project_id: None,
            ade_id: None,
            max_workers: Some(1),
            verification_commands: vec!["if exist result.txt (exit /b 0) else (exit /b 1)".into()],
        },
        mission,
    )
    .unwrap();
    let reviewed = save_reviewed_flow_plan(&catalog, preview).unwrap();
    if crash_after_failure {
        assert!(catch_unwind(AssertUnwindSafe(|| {
            dispatch_experimental_routed_flow(&catalog, &reviewed.draft_id)
        }))
        .is_err());
        let run_id = &reviewed.routing.as_ref().unwrap().authorization.run_id.0;
        let store = PytxoStore::open(&PytxoConfig::default().db_path_at(repo.path())).unwrap();
        let scope = pytxo_store::routing::RoutingScope {
            domain_id: pytxo_core::DomainId(reviewed.domain_id.clone()),
            run_id: RunId(run_id.clone()),
        };
        let history = store.routing_history(&scope).unwrap().unwrap();
        assert_eq!(history.attempts.len(), 1);
        assert_eq!(history.attempts[0].state, AttemptState::Failed);
        assert!(history.attempts[0].ownership_released);
        assert_eq!(history.tasks[0].next_ordinal, 2);
        assert_eq!(store.get_run_status(run_id).unwrap().unwrap().0, "starting");
        catalog
            .mark_routed_flow_recovery_required(&reviewed.draft_id, run_id)
            .unwrap();
        let owner = catalog
            .routed_flow_dispatch_owner(&reviewed.draft_id, run_id)
            .unwrap()
            .unwrap();
        rusqlite::Connection::open(repo.path().join(".git/catalog.db"))
            .unwrap()
            .execute(
                "UPDATE flow_dispatch_claims SET controller_start_identity=?1 WHERE draft_id=?2 AND run_id=?3 AND controller_pid=?4",
                rusqlite::params!["simulated-dead-controller", reviewed.draft_id, run_id, owner.controller_pid],
            )
            .unwrap();
        let marker_path = repo
            .path()
            .join(&PytxoConfig::default().data_dir)
            .join("active_run.json");
        let mut marker: serde_json::Value =
            serde_json::from_slice(&fs::read(&marker_path).unwrap()).unwrap();
        marker["supervisor_start_identity"] = "simulated-dead-controller".into();
        fs::write(&marker_path, serde_json::to_vec(&marker).unwrap()).unwrap();
        assert!(reconcile_routed_flow_startup(&catalog, &reviewed.draft_id).unwrap());
        let recovered = store.routing_history(&scope).unwrap().unwrap();
        assert!(recovered.cancelled);
        assert_eq!(recovered.attempts.len(), 1);
        assert_eq!(store.get_run_status(run_id).unwrap().unwrap().0, "failed");
        assert!(!repo
            .path()
            .join(&PytxoConfig::default().data_dir)
            .join("active_run.json")
            .exists());
        return;
    }
    let dispatch = dispatch_experimental_routed_flow(&catalog, &reviewed.draft_id);
    let run_id = if review_fault {
        assert!(dispatch.is_err());
        unsafe { std::env::remove_var("PYTXO_TEST_ROUTED_FAIL_BEFORE_REVIEW_VIEW") };
        reviewed
            .routing
            .as_ref()
            .unwrap()
            .authorization
            .run_id
            .0
            .clone()
    } else {
        dispatch.unwrap()
    };
    let store = PytxoStore::open(&PytxoConfig::default().db_path_at(repo.path())).unwrap();
    let scope = pytxo_store::routing::RoutingScope {
        domain_id: pytxo_core::DomainId(reviewed.domain_id.clone()),
        run_id: RunId(run_id.clone()),
    };
    let history = store.routing_history(&scope).unwrap().unwrap();
    assert_eq!(history.attempts.len(), 2);
    assert_eq!(history.attempts[0].ordinal, 1);
    assert_eq!(history.attempts[0].selected.profile.id.0, "everyday");
    assert_eq!(history.attempts[0].state, AttemptState::Failed);
    let failure = history.attempts[0].failure.as_ref().unwrap();
    assert_eq!(failure.failure_class, AttemptFailureClass::Implementation);
    assert!(failure.actionable_evidence_digest.is_some());
    assert!(history.attempts[0].ownership_released);
    assert_eq!(history.attempts[1].ordinal, 2);
    assert_eq!(history.attempts[1].selected.profile.id.0, "strong");
    assert_eq!(
        history.attempts[1].decision.reason,
        RouteReason::StrongRepair
    );
    assert_eq!(history.attempts[1].state, AttemptState::Passed);
    assert!(history.tasks[0].winner.is_some());
    assert_eq!(
        store.get_run_status(&run_id).unwrap().unwrap().0,
        if review_fault { "failed" } else { "completed" }
    );
    let contract = store.get_run_contract(&run_id).unwrap().unwrap();
    assert_eq!(
        contract.apply_status,
        if review_fault {
            "review_failed"
        } else {
            "ready"
        }
    );
    assert_eq!(contract.prepared_manifest.is_some(), !review_fault);
    assert!(fs::read(repo.path().join("result.txt")).unwrap().is_empty());
    assert!(store.unresolved_routing_attempts().unwrap().is_empty());
    if !review_fault {
        assert!(!store
            .unreconciled_registered_routing_scopes()
            .unwrap()
            .contains(&scope));
    }
    let data_dir = repo.path().join(&PytxoConfig::default().data_dir);
    write_exact_terminal_marker(
        &catalog,
        &reviewed.draft_id,
        &run_id,
        &data_dir,
        &reviewed.domain_id,
    );
    assert_eq!(
        rusqlite::Connection::open(repo.path().join(".git/catalog.db"))
            .unwrap()
            .execute(
                "UPDATE flow_drafts SET status='recovery_required' WHERE id=?1 AND dispatched_run_id=?2",
                rusqlite::params![reviewed.draft_id, run_id],
            )
            .unwrap(),
        1
    );
    assert!(reconcile_routed_flow_startup(&catalog, &reviewed.draft_id).unwrap());
    assert!(!data_dir.join("active_run.json").exists());
    assert_eq!(
        store.get_run_status(&run_id).unwrap().unwrap().0,
        if review_fault { "failed" } else { "completed" }
    );
    let digest = if review_fault {
        refresh_run_review(None, Some(repo.path().to_path_buf()), &run_id)
            .unwrap()
            .package_digest
    } else {
        contract.prepared_manifest.unwrap().package_digest
    };
    apply_run_changes(None, Some(repo.path().to_path_buf()), &run_id, &digest).unwrap();
    assert_eq!(
        fs::read(repo.path().join("result.txt")).unwrap(),
        b"pytxo-routed\r\n"
    );
}

#[test]
fn two_independent_siblings_publish_one_review_and_apply_both_outputs() {
    let _env_lock = FIXTURE_ENV_LOCK.lock().unwrap();
    run_parallel_sibling_fixture(false, false, false, 2);
}

#[test]
fn two_sibling_wave_requires_two_host_slots_before_admission() {
    let _env_lock = FIXTURE_ENV_LOCK.lock().unwrap();
    run_parallel_sibling_fixture(false, false, false, 1);
}

#[cfg(feature = "routed-test-faults")]
#[test]
fn settled_parallel_siblings_recover_review_from_retained_winners() {
    let _env_lock = FIXTURE_ENV_LOCK.lock().unwrap();
    run_parallel_sibling_fixture(true, false, false, 2);
}

#[cfg(feature = "routed-test-faults")]
#[test]
fn dead_controller_closes_two_prepared_sibling_owners() {
    let _env_lock = FIXTURE_ENV_LOCK.lock().unwrap();
    unsafe { std::env::set_var("PYTXO_TEST_ROUTED_CRASH_AFTER_SIBLINGS_ADMITTED", "1") };
    run_parallel_sibling_fixture(true, false, false, 2);
    unsafe { std::env::remove_var("PYTXO_TEST_ROUTED_CRASH_AFTER_SIBLINGS_ADMITTED") };
}

#[cfg(feature = "routed-test-faults")]
#[test]
fn dead_controller_closes_first_prepared_sibling_before_second_admission() {
    let _env_lock = FIXTURE_ENV_LOCK.lock().unwrap();
    unsafe { std::env::set_var("PYTXO_TEST_ROUTED_CRASH_AFTER_FIRST_SIBLING_ADMITTED", "1") };
    run_parallel_sibling_fixture(true, false, false, 2);
    unsafe { std::env::remove_var("PYTXO_TEST_ROUTED_CRASH_AFTER_FIRST_SIBLING_ADMITTED") };
}

#[cfg(feature = "routed-test-faults")]
#[test]
fn dead_controller_closes_prepared_siblings_after_durable_stop() {
    let _env_lock = FIXTURE_ENV_LOCK.lock().unwrap();
    unsafe {
        std::env::set_var("PYTXO_TEST_ROUTED_CRASH_AFTER_SIBLINGS_ADMITTED", "1");
        std::env::set_var("PYTXO_TEST_ROUTED_TERMINAL_STOP_BEFORE_RECOVERY", "1");
    }
    run_parallel_sibling_fixture(true, false, false, 2);
    unsafe {
        std::env::remove_var("PYTXO_TEST_ROUTED_CRASH_AFTER_SIBLINGS_ADMITTED");
        std::env::remove_var("PYTXO_TEST_ROUTED_TERMINAL_STOP_BEFORE_RECOVERY");
    }
}

#[cfg(feature = "routed-test-faults")]
#[test]
fn dead_controller_preserves_passed_sibling_and_closes_prepared_owner() {
    let _env_lock = FIXTURE_ENV_LOCK.lock().unwrap();
    unsafe { std::env::set_var("PYTXO_TEST_ROUTED_CRASH_AFTER_FIRST_SIBLING_PASSED", "1") };
    run_parallel_sibling_fixture(true, false, false, 2);
    unsafe { std::env::remove_var("PYTXO_TEST_ROUTED_CRASH_AFTER_FIRST_SIBLING_PASSED") };
}

#[cfg(feature = "routed-test-faults")]
#[test]
fn dead_controller_preserves_failed_sibling_and_closes_prepared_owner() {
    let _env_lock = FIXTURE_ENV_LOCK.lock().unwrap();
    unsafe {
        std::env::set_var("PYTXO_TEST_ROUTED_CRASH_AFTER_FIRST_SIBLING_FAILED", "1");
        std::env::set_var("PYTXO_TEST_ROUTED_FORCE_PARENT_WORKER_FAILURE", "1");
    }
    run_parallel_sibling_fixture(true, false, false, 2);
    unsafe {
        std::env::remove_var("PYTXO_TEST_ROUTED_CRASH_AFTER_FIRST_SIBLING_FAILED");
        std::env::remove_var("PYTXO_TEST_ROUTED_FORCE_PARENT_WORKER_FAILURE");
    }
}

#[cfg(feature = "routed-test-faults")]
#[test]
fn failed_sibling_fences_the_other_and_releases_both_owners() {
    let _env_lock = FIXTURE_ENV_LOCK.lock().unwrap();
    run_parallel_sibling_fixture(false, true, false, 2);
}

#[cfg(feature = "routed-test-faults")]
#[test]
fn two_native_sibling_workers_overlap_under_owned_host_slots() {
    let _env_lock = FIXTURE_ENV_LOCK.lock().unwrap();
    run_parallel_sibling_fixture(false, false, true, 2);
}

#[cfg(feature = "routed-test-faults")]
#[test]
fn catalog_stop_before_sibling_review_blocks_publication() {
    let _env_lock = FIXTURE_ENV_LOCK.lock().unwrap();
    unsafe {
        std::env::set_var("PYTXO_TEST_ROUTED_CATALOG_STOP_BEFORE_SIBLING_REVIEW", "1");
    }
    run_parallel_sibling_fixture(false, false, false, 2);
    unsafe {
        std::env::remove_var("PYTXO_TEST_ROUTED_CATALOG_STOP_BEFORE_SIBLING_REVIEW");
    }
}

fn run_parallel_sibling_fixture(
    crash_after_pass: bool,
    fail_first: bool,
    observe_overlap: bool,
    capacity_units: u64,
) {
    #[cfg(feature = "routed-test-faults")]
    let prepared_crash =
        std::env::var_os("PYTXO_TEST_ROUTED_CRASH_AFTER_SIBLINGS_ADMITTED").is_some();
    #[cfg(feature = "routed-test-faults")]
    let one_admitted_crash =
        std::env::var_os("PYTXO_TEST_ROUTED_CRASH_AFTER_FIRST_SIBLING_ADMITTED").is_some();
    #[cfg(feature = "routed-test-faults")]
    let first_pass_crash =
        std::env::var_os("PYTXO_TEST_ROUTED_CRASH_AFTER_FIRST_SIBLING_PASSED").is_some();
    #[cfg(feature = "routed-test-faults")]
    let first_failed_crash =
        std::env::var_os("PYTXO_TEST_ROUTED_CRASH_AFTER_FIRST_SIBLING_FAILED").is_some();
    let catalog_stop_before_review =
        std::env::var_os("PYTXO_TEST_ROUTED_CATALOG_STOP_BEFORE_SIBLING_REVIEW").is_some();
    let home = tempfile::tempdir().unwrap();
    unsafe {
        std::env::set_var("PYTXO_HOME", home.path().join("home"));
        std::env::set_var("PYTXO_TRUST_STORE", home.path().join("trust.json"));
        std::env::set_var("PYTXO_EXPERIMENTAL_ROUTED_LOCAL_FIXTURE", "1");
    }
    let repo = tempfile::tempdir().unwrap();
    fs::write(repo.path().join("seed.txt"), "").unwrap();
    fs::write(repo.path().join("result.txt"), "").unwrap();
    fs::write(
        repo.path().join("pytxo.toml"),
        "permission_profile = 'orbit'\nexecution_backend = 'subprocess'\nmax_agents = 2\n[blast]\nprefer_kernel_overlay = false\n[[task]]\nid = 'seed'\nagent = 'default'\npaths = ['seed.txt']\nverify = ['if exist seed.txt (exit /b 0) else (exit /b 1)']\n[[task]]\nid = 'result'\nagent = 'default'\npaths = ['result.txt']\nverify = ['if exist result.txt (exit /b 0) else (exit /b 1)']\n",
    ).unwrap();
    for args in [
        vec!["init", "-q"],
        vec!["add", "."],
        vec![
            "-c",
            "user.name=Test",
            "-c",
            "user.email=test@pytxo.local",
            "commit",
            "-qm",
            "baseline",
        ],
    ] {
        let output = Command::new("git")
            .args(args)
            .current_dir(repo.path())
            .output()
            .unwrap();
        assert!(
            output.status.success(),
            "{}",
            String::from_utf8_lossy(&output.stderr)
        );
    }
    trust_repo(repo.path(), PermissionProfile::Orbit).unwrap();
    let catalog = Catalog::open(&repo.path().join(".git/catalog.db")).unwrap();
    catalog
        .configure_capacity_pool(&CapacityPoolConfig {
            resource_id: "host".into(),
            capacity_units,
            expected_revision: None,
            configured_at_ms: 100,
        })
        .unwrap();
    let preview = preview_experimental_routed_flow(
        &catalog,
        FlowDraftInput {
            id: "routed-siblings-fixture".into(),
            title: "Routed siblings fixture".into(),
            mission_text: "pytxo-local-fixture-v1:write-seed;pytxo-local-fixture-v1:write-result"
                .into(),
            source: FlowSource::Text,
            domain_id: Some(repo.path().to_string_lossy().into_owned()),
            project_id: None,
            ade_id: None,
            max_workers: Some(2),
            verification_commands: vec![],
        },
        mission,
    )
    .unwrap();
    assert_eq!(
        preview.waves,
        [vec!["seed".to_owned(), "result".to_owned()]]
    );
    let reviewed = save_reviewed_flow_plan(&catalog, preview).unwrap();
    #[cfg(feature = "routed-test-faults")]
    if fail_first {
        unsafe { std::env::set_var("PYTXO_TEST_ROUTED_FORCE_PARENT_WORKER_FAILURE", "1") };
    }
    #[cfg(feature = "routed-test-faults")]
    if crash_after_pass {
        unsafe { std::env::set_var("PYTXO_TEST_ROUTED_CRASH_AFTER_FINAL_PASS", "1") };
    }
    let store = PytxoStore::open(&PytxoConfig::default().db_path_at(repo.path())).unwrap();
    let expected_run_id = reviewed
        .routing
        .as_ref()
        .unwrap()
        .authorization
        .run_id
        .0
        .clone();
    let run_id = if observe_overlap {
        #[cfg(feature = "routed-test-faults")]
        unsafe {
            std::env::set_var("PYTXO_TEST_ROUTED_SLOW_SIBLINGS", "1")
        };
        let catalog_path = repo.path().join(".git/catalog.db");
        let draft_id = reviewed.draft_id.clone();
        let dispatch = std::thread::spawn(move || {
            let reopened = Catalog::open_existing_for_capacity_recovery(&catalog_path).unwrap();
            dispatch_experimental_routed_flow(&reopened, &draft_id)
        });
        let scope = pytxo_store::routing::RoutingScope {
            domain_id: pytxo_core::DomainId(reviewed.domain_id.clone()),
            run_id: RunId(expected_run_id.clone()),
        };
        let deadline = std::time::Instant::now() + std::time::Duration::from_secs(90);
        let mut simultaneous_registered = false;
        let mut observed_phases = BTreeSet::new();
        while std::time::Instant::now() < deadline && !dispatch.is_finished() {
            if let Some(history) = store.routing_history(&scope).unwrap() {
                let phases = history
                    .attempts
                    .iter()
                    .map(|attempt| {
                        store
                            .launch_ownership(&attempt.attempt_id)
                            .unwrap()
                            .map(|owner| owner.phase)
                    })
                    .collect::<Vec<_>>();
                observed_phases.insert(format!("{phases:?}"));
                simultaneous_registered = phases.len() == 2
                    && phases.iter().all(|phase| {
                        *phase
                            == Some(pytxo_store::routing_launch::LaunchOwnershipPhase::Registered)
                    });
                if simultaneous_registered {
                    break;
                }
            }
            std::thread::sleep(std::time::Duration::from_millis(20));
        }
        let result = dispatch.join().unwrap();
        #[cfg(feature = "routed-test-faults")]
        unsafe {
            std::env::remove_var("PYTXO_TEST_ROUTED_SLOW_SIBLINGS")
        };
        let run_id = result.unwrap_or_else(|error| {
            let history = store.routing_history(&scope).unwrap();
            let states = history.map(|history| {
                history
                    .attempts
                    .iter()
                    .map(|attempt| {
                        (
                            attempt.task_id.0.clone(),
                            attempt.state,
                            attempt.ownership_released,
                        )
                    })
                    .collect::<Vec<_>>()
            });
            panic!("overlapping sibling dispatch failed: {error:#}; phases={observed_phases:?}; states={states:?}")
        });
        assert!(simultaneous_registered, "two owned native workers were never registered concurrently; phases={observed_phases:?}");
        run_id
    } else if crash_after_pass {
        assert!(catch_unwind(AssertUnwindSafe(|| {
            dispatch_experimental_routed_flow(&catalog, &reviewed.draft_id)
        }))
        .is_err());
        #[cfg(feature = "routed-test-faults")]
        unsafe {
            std::env::remove_var("PYTXO_TEST_ROUTED_CRASH_AFTER_FINAL_PASS")
        };
        reviewed
            .routing
            .as_ref()
            .unwrap()
            .authorization
            .run_id
            .0
            .clone()
    } else {
        let result = dispatch_experimental_routed_flow(&catalog, &reviewed.draft_id);
        #[cfg(feature = "routed-test-faults")]
        if fail_first {
            unsafe { std::env::remove_var("PYTXO_TEST_ROUTED_FORCE_PARENT_WORKER_FAILURE") };
        }
        if fail_first || capacity_units < 2 || catalog_stop_before_review {
            if let Err(error) = &result {
                eprintln!("expected sibling dispatch failure: {error:#}");
            }
            assert!(result.is_err());
            reviewed
                .routing
                .as_ref()
                .unwrap()
                .authorization
                .run_id
                .0
                .clone()
        } else {
            result.unwrap_or_else(|error| panic!("sibling dispatch failed: {error:#}"))
        }
    };
    let scope = pytxo_store::routing::RoutingScope {
        domain_id: pytxo_core::DomainId(reviewed.domain_id.clone()),
        run_id: RunId(run_id.clone()),
    };
    let history = store.routing_history(&scope).unwrap().unwrap();
    if capacity_units < 2 {
        assert!(history.cancelled);
        assert!(history.attempts.is_empty());
        assert_eq!(store.get_run_status(&run_id).unwrap().unwrap().0, "failed");
        assert!(!repo.path().join(".pytxo/data/active_run.json").exists());
        return;
    }
    if fail_first {
        assert!(history.cancelled);
        assert_eq!(history.attempts.len(), 2);
        assert!(history
            .attempts
            .iter()
            .all(|attempt| { attempt.state.is_terminal() && attempt.ownership_released }));
        assert!(history.attempts.iter().any(|attempt| {
            attempt.task_id.0 == "seed" && attempt.state == AttemptState::Failed
        }), "forced sibling failure did not settle on seed: attempts={:?}; cancelled={}; owners={:?}",
            history.attempts.iter().map(|attempt| (
                attempt.task_id.0.clone(),
                attempt.state,
                attempt.ownership_released,
                attempt.failure.clone(),
                attempt.receipts.clone(),
            )).collect::<Vec<_>>(),
            history.cancelled,
            history.attempts.iter().map(|attempt| (
                attempt.task_id.0.clone(),
                store.launch_ownership(&attempt.attempt_id).unwrap().map(|owner| owner.phase),
            )).collect::<Vec<_>>(),
        );
        assert_eq!(store.get_run_status(&run_id).unwrap().unwrap().0, "failed");
        assert!(store
            .unresolved_routing_attempts()
            .unwrap()
            .iter()
            .all(|attempt| attempt.scope != scope));
        assert!(!store
            .unresolved_capacity_intent_scopes()
            .unwrap()
            .contains(&scope));
        assert!(!repo.path().join(".pytxo/data/active_run.json").exists());
        return;
    }
    if catalog_stop_before_review {
        assert!(history.cancelled);
        assert_eq!(history.attempts.len(), 2);
        assert!(history.attempts.iter().all(|attempt| {
            attempt.state == AttemptState::Passed && attempt.ownership_released
        }));
        assert_eq!(
            store.get_run_status(&run_id).unwrap().unwrap().0,
            "cancelled"
        );
        let contract = store.get_run_contract(&run_id).unwrap().unwrap();
        assert!(contract.prepared_manifest.is_none());
        assert_ne!(contract.apply_status, "ready");
        assert!(!repo.path().join(".pytxo/data/active_run.json").exists());
        assert!(catalog
            .unresolved_capacity_reservations()
            .unwrap()
            .iter()
            .all(|reservation| reservation.run_id != run_id));
        assert!(fs::read(repo.path().join("seed.txt")).unwrap().is_empty());
        assert!(fs::read(repo.path().join("result.txt")).unwrap().is_empty());
        return;
    }
    #[cfg(feature = "routed-test-faults")]
    if prepared_crash || first_pass_crash || first_failed_crash || one_admitted_crash {
        let admitted_count = if one_admitted_crash { 1 } else { 2 };
        assert_eq!(history.attempts.len(), admitted_count);
        let prepared = history
            .attempts
            .iter()
            .filter(|attempt| {
                store
                    .launch_ownership(&attempt.attempt_id)
                    .unwrap()
                    .unwrap()
                    .phase
                    == pytxo_store::routing_launch::LaunchOwnershipPhase::Prepared
            })
            .count();
        assert_eq!(prepared, if prepared_crash { 2 } else { 1 });
        if first_pass_crash {
            let first = history
                .attempts
                .iter()
                .find(|attempt| attempt.task_id.0 == "seed")
                .unwrap();
            assert_eq!(first.state, AttemptState::Passed);
            assert!(first.ownership_released);
        }
        if first_failed_crash {
            let first = history
                .attempts
                .iter()
                .find(|attempt| attempt.task_id.0 == "seed")
                .unwrap();
            assert_eq!(first.state, AttemptState::Failed);
            assert!(first.ownership_released);
            assert!(history.cancelled);
        }
        assert_eq!(
            store.get_run_status(&run_id).unwrap().unwrap().0,
            "starting"
        );
        let stopped = std::env::var_os("PYTXO_TEST_ROUTED_TERMINAL_STOP_BEFORE_RECOVERY").is_some();
        if stopped {
            let stopped_at = u64::try_from(
                std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .unwrap()
                    .as_millis(),
            )
            .unwrap();
            store
                .cancel_routing_mission(
                    &scope,
                    "controller.stop.cancel.v1",
                    history.cancel_epoch,
                    stopped_at,
                )
                .unwrap();
            store.finish_run(&run_id, "cancelled").unwrap();
        }
        catalog
            .mark_routed_flow_recovery_required(&reviewed.draft_id, &run_id)
            .unwrap();
        let owner = catalog
            .routed_flow_dispatch_owner(&reviewed.draft_id, &run_id)
            .unwrap()
            .unwrap();
        let marker_path = repo.path().join(".pytxo/data/active_run.json");
        let mut marker: serde_json::Value =
            serde_json::from_slice(&fs::read(&marker_path).unwrap()).unwrap();
        marker["supervisor_start_identity"] = "simulated-dead-controller".into();
        rusqlite::Connection::open(repo.path().join(".git/catalog.db"))
            .unwrap()
            .execute(
                "UPDATE flow_dispatch_claims SET controller_start_identity=?1 WHERE draft_id=?2 AND run_id=?3 AND controller_pid=?4",
                rusqlite::params!["simulated-dead-controller", reviewed.draft_id, run_id, owner.controller_pid],
            )
            .unwrap();
        if prepared_crash && !stopped {
            assert!(!reconcile_routed_flow_startup(&catalog, &reviewed.draft_id).unwrap());
            let guarded = store.routing_history(&scope).unwrap().unwrap();
            assert!(guarded
                .attempts
                .iter()
                .all(|attempt| attempt.state == AttemptState::Preparing
                    && !attempt.ownership_released));
            assert_eq!(
                catalog
                    .unresolved_capacity_reservations()
                    .unwrap()
                    .iter()
                    .filter(|reservation| reservation.run_id == run_id)
                    .count(),
                2
            );
        }
        fs::write(&marker_path, serde_json::to_vec(&marker).unwrap()).unwrap();
        if first_failed_crash {
            // A released native Job is insufficient to retire its sibling's
            // capacity if the failed attempt's usage settlement is missing.
            let failed = history
                .attempts
                .iter()
                .find(|attempt| attempt.task_id.0 == "seed")
                .unwrap();
            let db =
                rusqlite::Connection::open(PytxoConfig::default().db_path_at(repo.path())).unwrap();
            let original: String = db
                .query_row(
                    "SELECT record_json FROM routing_attempts WHERE attempt_id=?1",
                    rusqlite::params![failed.attempt_id.0],
                    |row| row.get(0),
                )
                .unwrap();
            let mut incomplete = failed.clone();
            incomplete.usage = pytxo_store::routing::RoutedUsage::Unreported;
            assert_eq!(
                db.execute(
                    "UPDATE routing_attempts SET record_json=?1 WHERE attempt_id=?2",
                    rusqlite::params![
                        serde_json::to_string(&incomplete).unwrap(),
                        failed.attempt_id.0
                    ],
                )
                .unwrap(),
                1
            );
            assert!(!reconcile_routed_flow_startup(&catalog, &reviewed.draft_id).unwrap());
            assert_eq!(
                store.get_run_status(&run_id).unwrap().unwrap().0,
                "starting"
            );
            assert_eq!(
                db.execute(
                    "UPDATE routing_attempts SET record_json=?1 WHERE attempt_id=?2",
                    rusqlite::params![original, failed.attempt_id.0],
                )
                .unwrap(),
                1
            );
        }
        let recovered = reconcile_routed_flow_startup(&catalog, &reviewed.draft_id).unwrap();
        assert!(
            recovered,
            "prepared recovery did not finish: marker={marker:?}; owner={:?}; run={:?}; attempts={:?}; unresolved_intents={:?}",
            catalog
                .routed_flow_dispatch_owner(&reviewed.draft_id, &run_id)
                .unwrap(),
            store.get_run_status(&run_id).unwrap(),
            store.routing_history(&scope).unwrap().map(|history| {
                history
                    .attempts
                    .into_iter()
                    .map(|attempt| (attempt.task_id, attempt.state, attempt.ownership_released))
                    .collect::<Vec<_>>()
            }),
            store.unresolved_capacity_intent_scopes().unwrap(),
        );
        let recovered = store.routing_history(&scope).unwrap().unwrap();
        assert!(recovered.cancelled);
        assert_eq!(recovered.attempts.len(), admitted_count);
        assert_eq!(
            recovered
                .attempts
                .iter()
                .filter(|attempt| attempt.state == AttemptState::FailedNoLaunch)
                .count(),
            prepared
        );
        assert!(recovered
            .attempts
            .iter()
            .all(|attempt| attempt.state.is_terminal() && attempt.ownership_released));
        assert_eq!(
            store.get_run_status(&run_id).unwrap().unwrap().0,
            if stopped { "cancelled" } else { "failed" }
        );
        assert!(!marker_path.exists());
        assert!(!store
            .unresolved_capacity_intent_scopes()
            .unwrap()
            .contains(&scope));
        assert!(catalog
            .unresolved_capacity_reservations()
            .unwrap()
            .iter()
            .all(|reservation| reservation.run_id != run_id));
        return;
    }
    assert!(
        !history.cancelled,
        "successful sibling fixture was cancelled: attempts={:?}; events={:?}",
        history
            .attempts
            .iter()
            .map(|attempt| (&attempt.task_id, &attempt.state, &attempt.receipts))
            .collect::<Vec<_>>(),
        history
            .events
            .iter()
            .map(|event| &event.event_id)
            .collect::<Vec<_>>()
    );
    assert_eq!(history.attempts.len(), 2);
    assert!(history.attempts.iter().all(|attempt| {
        attempt.state == AttemptState::Passed
            && attempt.ownership_released
            && attempt.dependencies.is_empty()
            && attempt.handoff.is_none()
    }));
    let manifest = if crash_after_pass {
        assert_eq!(
            store.get_run_status(&run_id).unwrap().unwrap().0,
            "starting"
        );
        catalog
            .mark_routed_flow_recovery_required(&reviewed.draft_id, &run_id)
            .unwrap();
        let owner = catalog
            .routed_flow_dispatch_owner(&reviewed.draft_id, &run_id)
            .unwrap()
            .unwrap();
        rusqlite::Connection::open(repo.path().join(".git/catalog.db"))
            .unwrap()
            .execute(
                "UPDATE flow_dispatch_claims SET controller_start_identity=?1 WHERE draft_id=?2 AND run_id=?3 AND controller_pid=?4",
                rusqlite::params!["simulated-dead-controller", reviewed.draft_id, run_id, owner.controller_pid],
            )
            .unwrap();
        let marker_path = repo.path().join(".pytxo/data/active_run.json");
        let mut marker: serde_json::Value =
            serde_json::from_slice(&fs::read(&marker_path).unwrap()).unwrap();
        marker["supervisor_start_identity"] = "simulated-dead-controller".into();
        fs::write(&marker_path, serde_json::to_vec(&marker).unwrap()).unwrap();
        assert!(reconcile_routed_flow_startup(&catalog, &reviewed.draft_id).unwrap());
        assert_eq!(store.get_run_status(&run_id).unwrap().unwrap().0, "failed");
        assert_eq!(
            store
                .get_run_contract(&run_id)
                .unwrap()
                .unwrap()
                .apply_status,
            "review_failed"
        );
        refresh_run_review(None, Some(repo.path().to_path_buf()), &run_id).unwrap()
    } else {
        assert_eq!(
            store.get_run_status(&run_id).unwrap().unwrap().0,
            "completed"
        );
        let contract = store.get_run_contract(&run_id).unwrap().unwrap();
        assert_eq!(contract.apply_status, "ready");
        contract.prepared_manifest.unwrap()
    };
    assert_eq!(manifest.files.len(), 2);
    assert_eq!(
        manifest
            .files
            .iter()
            .map(|file| file.path.as_str())
            .collect::<BTreeSet<_>>(),
        BTreeSet::from(["seed.txt", "result.txt"])
    );
    apply_run_changes(
        None,
        Some(repo.path().to_path_buf()),
        &run_id,
        &manifest.package_digest,
    )
    .unwrap();
    assert_eq!(
        fs::read(repo.path().join("seed.txt")).unwrap(),
        b"pytxo-routed\r\n"
    );
    assert_eq!(
        fs::read(repo.path().join("result.txt")).unwrap(),
        b"pytxo-routed\r\n"
    );
}

#[test]
fn reviewed_two_wave_fixture_inherits_parent_and_applies_both_outputs() {
    let _env_lock = FIXTURE_ENV_LOCK.lock().unwrap();
    run_reviewed_two_wave_fixture(false, false, false, false, false);
}

#[cfg(feature = "routed-test-faults")]
#[test]
fn reviewed_two_wave_fixture_hands_off_across_cmd_and_powershell() {
    let _env_lock = FIXTURE_ENV_LOCK.lock().unwrap();
    unsafe { std::env::set_var("PYTXO_TEST_ROUTED_POWERSHELL_HANDOFF", "1") };
    run_reviewed_two_wave_fixture(false, false, false, false, false);
    unsafe { std::env::remove_var("PYTXO_TEST_ROUTED_POWERSHELL_HANDOFF") };
}

#[cfg(feature = "routed-test-faults")]
#[test]
fn two_wave_manifest_mismatch_fails_publication_and_refreshes_from_winners() {
    let _env_lock = FIXTURE_ENV_LOCK.lock().unwrap();
    unsafe {
        std::env::set_var("PYTXO_TEST_ROUTED_MUTATE_MANIFEST_BEFORE_PUBLICATION", "1");
    }
    run_reviewed_two_wave_fixture(true, false, false, false, false);
}

#[cfg(feature = "routed-test-faults")]
#[test]
fn two_wave_empty_candidate_checks_cannot_publish_review() {
    let _env_lock = FIXTURE_ENV_LOCK.lock().unwrap();
    unsafe {
        std::env::set_var(
            "PYTXO_TEST_ROUTED_EMPTY_CANDIDATE_CHECKS_BEFORE_PUBLICATION",
            "1",
        );
    }
    run_reviewed_two_wave_fixture(true, false, false, false, false);
}

#[cfg(feature = "routed-test-faults")]
#[test]
fn two_wave_candidate_command_must_match_registered_recipe() {
    let _env_lock = FIXTURE_ENV_LOCK.lock().unwrap();
    unsafe {
        std::env::set_var(
            "PYTXO_TEST_ROUTED_WRONG_CANDIDATE_COMMAND_BEFORE_PUBLICATION",
            "1",
        );
    }
    run_reviewed_two_wave_fixture(true, false, false, false, false);
}

#[cfg(feature = "routed-test-faults")]
#[test]
fn two_wave_candidate_mode_must_match_sealed_winner() {
    let _env_lock = FIXTURE_ENV_LOCK.lock().unwrap();
    unsafe {
        std::env::set_var(
            "PYTXO_TEST_ROUTED_WRONG_CANDIDATE_MODE_BEFORE_PUBLICATION",
            "1",
        );
    }
    run_reviewed_two_wave_fixture(true, false, false, false, false);
}

#[cfg(feature = "routed-test-faults")]
#[test]
fn stop_between_two_waves_never_admits_the_child() {
    let _env_lock = FIXTURE_ENV_LOCK.lock().unwrap();
    unsafe {
        std::env::set_var("PYTXO_TEST_ROUTED_STOP_BETWEEN_WAVES", "1");
    }
    run_reviewed_two_wave_fixture(false, true, false, false, false);
    unsafe {
        std::env::remove_var("PYTXO_TEST_ROUTED_STOP_BETWEEN_WAVES");
    }
}

#[cfg(feature = "routed-test-faults")]
#[test]
fn failed_parent_worker_never_admits_the_child() {
    let _env_lock = FIXTURE_ENV_LOCK.lock().unwrap();
    unsafe {
        std::env::set_var("PYTXO_TEST_ROUTED_FORCE_PARENT_WORKER_FAILURE", "1");
    }
    run_reviewed_two_wave_fixture(false, false, true, false, false);
    unsafe {
        std::env::remove_var("PYTXO_TEST_ROUTED_FORCE_PARENT_WORKER_FAILURE");
    }
}

#[cfg(feature = "routed-test-faults")]
#[test]
fn dead_controller_after_parent_pass_fences_child_and_reconciles_registry() {
    let _env_lock = FIXTURE_ENV_LOCK.lock().unwrap();
    unsafe { std::env::set_var("PYTXO_TEST_ROUTED_CRASH_AFTER_PARENT_PASS", "1") };
    run_reviewed_two_wave_fixture(false, false, false, true, false);
    unsafe { std::env::remove_var("PYTXO_TEST_ROUTED_CRASH_AFTER_PARENT_PASS") };
}

#[cfg(feature = "routed-test-faults")]
#[test]
fn dead_controller_after_parent_pass_honors_durable_stop() {
    let _env_lock = FIXTURE_ENV_LOCK.lock().unwrap();
    unsafe {
        std::env::set_var("PYTXO_TEST_ROUTED_CRASH_AFTER_PARENT_PASS", "1");
        std::env::set_var("PYTXO_TEST_ROUTED_STOP_AFTER_PARENT_CRASH", "1");
    }
    run_reviewed_two_wave_fixture(false, false, false, true, false);
    unsafe {
        std::env::remove_var("PYTXO_TEST_ROUTED_CRASH_AFTER_PARENT_PASS");
        std::env::remove_var("PYTXO_TEST_ROUTED_STOP_AFTER_PARENT_CRASH");
    }
}

#[cfg(feature = "routed-test-faults")]
#[test]
fn dead_controller_after_final_pass_recovers_review_from_winners() {
    let _env_lock = FIXTURE_ENV_LOCK.lock().unwrap();
    unsafe { std::env::set_var("PYTXO_TEST_ROUTED_CRASH_AFTER_FINAL_PASS", "1") };
    run_reviewed_two_wave_fixture(false, false, false, true, false);
    unsafe { std::env::remove_var("PYTXO_TEST_ROUTED_CRASH_AFTER_FINAL_PASS") };
}

#[cfg(feature = "routed-test-faults")]
#[test]
fn discarding_recovered_failed_review_releases_routed_domain_claim() {
    let _env_lock = FIXTURE_ENV_LOCK.lock().unwrap();
    unsafe {
        std::env::set_var("PYTXO_TEST_ROUTED_CRASH_AFTER_FINAL_PASS", "1");
        std::env::set_var("PYTXO_TEST_ROUTED_DISCARD_AFTER_FINAL_PASS", "1");
    }
    run_reviewed_two_wave_fixture(false, false, false, true, false);
    unsafe {
        std::env::remove_var("PYTXO_TEST_ROUTED_CRASH_AFTER_FINAL_PASS");
        std::env::remove_var("PYTXO_TEST_ROUTED_DISCARD_AFTER_FINAL_PASS");
    }
}

#[test]
fn completed_two_wave_review_recovers_only_exact_settled_registry_entries() {
    let _env_lock = FIXTURE_ENV_LOCK.lock().unwrap();
    run_reviewed_two_wave_fixture(false, false, false, false, true);
}

fn run_reviewed_two_wave_fixture(
    review_fault: bool,
    stop_between: bool,
    worker_fail: bool,
    crash_between: bool,
    recover_completed: bool,
) {
    let home = tempfile::tempdir().unwrap();
    unsafe {
        std::env::set_var("PYTXO_HOME", home.path().join("home"));
        std::env::set_var("PYTXO_TRUST_STORE", home.path().join("trust.json"));
        std::env::set_var("PYTXO_EXPERIMENTAL_ROUTED_LOCAL_FIXTURE", "1");
    }
    let repo = tempfile::tempdir().unwrap();
    fs::write(repo.path().join("seed.txt"), "").unwrap();
    fs::write(repo.path().join("result.txt"), "").unwrap();
    fs::write(
        repo.path().join("pytxo.toml"),
        "permission_profile = 'orbit'\nexecution_backend = 'subprocess'\nmax_agents = 1\n[blast]\nprefer_kernel_overlay = false\n[[task]]\nid = 'seed'\nagent = 'default'\npaths = ['seed.txt']\nverify = ['if exist seed.txt (exit /b 0) else (exit /b 1)']\n[[task]]\nid = 'result'\nagent = 'default'\npaths = ['result.txt']\ndepends_on = ['seed']\nverify = ['if exist result.txt (exit /b 0) else (exit /b 1)']\n",
    ).unwrap();
    for args in [
        vec!["init", "-q"],
        vec!["add", "."],
        vec![
            "-c",
            "user.name=Test",
            "-c",
            "user.email=test@pytxo.local",
            "commit",
            "-qm",
            "baseline",
        ],
    ] {
        let output = Command::new("git")
            .args(args)
            .current_dir(repo.path())
            .output()
            .unwrap();
        assert!(
            output.status.success(),
            "{}",
            String::from_utf8_lossy(&output.stderr)
        );
    }
    trust_repo(repo.path(), PermissionProfile::Orbit).unwrap();
    let catalog = Catalog::open(&repo.path().join(".git/catalog.db")).unwrap();
    catalog
        .configure_capacity_pool(&CapacityPoolConfig {
            resource_id: "host".into(),
            capacity_units: 1,
            expected_revision: None,
            configured_at_ms: 100,
        })
        .unwrap();
    let preview = preview_experimental_routed_flow(
        &catalog,
        FlowDraftInput {
            id: "routed-two-wave-fixture".into(),
            title: "Routed two-wave fixture".into(),
            mission_text:
                "pytxo-local-fixture-v1:write-seed;pytxo-local-fixture-v1:write-result-from-seed"
                    .into(),
            source: FlowSource::Text,
            domain_id: Some(repo.path().to_string_lossy().into_owned()),
            project_id: None,
            ade_id: None,
            max_workers: Some(1),
            verification_commands: vec![],
        },
        mission,
    )
    .unwrap();
    assert_eq!(
        preview.waves,
        vec![vec!["seed".to_owned()], vec!["result".to_owned()]]
    );
    let reviewed = save_reviewed_flow_plan(&catalog, preview).unwrap();
    if crash_between {
        let crash_after_final =
            std::env::var_os("PYTXO_TEST_ROUTED_CRASH_AFTER_FINAL_PASS").is_some();
        let stop_after_parent =
            std::env::var_os("PYTXO_TEST_ROUTED_STOP_AFTER_PARENT_CRASH").is_some();
        let crash_result = catch_unwind(AssertUnwindSafe(|| {
            dispatch_experimental_routed_flow(&catalog, &reviewed.draft_id)
        }));
        if crash_result.is_ok() {
            let run_id = &reviewed.routing.as_ref().unwrap().authorization.run_id.0;
            let store = PytxoStore::open(&PytxoConfig::default().db_path_at(repo.path())).unwrap();
            let scope = pytxo_store::routing::RoutingScope {
                domain_id: pytxo_core::DomainId(reviewed.domain_id.clone()),
                run_id: RunId(run_id.clone()),
            };
            let history = store.routing_history(&scope).unwrap();
            panic!(
                "expected routed controller crash seam; dispatch={crash_result:?}; run={:?}; attempts={:?}; events={:?}",
                store.get_run_status(run_id).unwrap(),
                history.as_ref().map(|history| history.attempts.iter().map(|attempt| (&attempt.task_id, &attempt.state, &attempt.receipts)).collect::<Vec<_>>()),
                history.as_ref().map(|history| history.events.iter().map(|event| &event.event_id).collect::<Vec<_>>()),
            );
        }
        let run_id = &reviewed.routing.as_ref().unwrap().authorization.run_id.0;
        let store = PytxoStore::open(&PytxoConfig::default().db_path_at(repo.path())).unwrap();
        let scope = pytxo_store::routing::RoutingScope {
            domain_id: pytxo_core::DomainId(reviewed.domain_id.clone()),
            run_id: RunId(run_id.clone()),
        };
        let history = store.routing_history(&scope).unwrap().unwrap();
        assert_eq!(store.get_run_status(run_id).unwrap().unwrap().0, "starting");
        assert_eq!(
            history.attempts.len(),
            if crash_after_final { 2 } else { 1 }
        );
        assert!(history
            .attempts
            .iter()
            .any(|attempt| attempt.task_id.0 == "seed" && attempt.state == AttemptState::Passed));
        let data_dir = repo.path().join(".pytxo/data");
        let registry_file = registry_path(&data_dir);
        let registry = ProcessRegistryFile::load(&registry_file).unwrap();
        assert!(!registry.for_run(run_id).is_empty());
        let mut forged: ProcessEntry = registry.for_run(run_id)[0].clone();
        forged.agent_key = format!("{run_id}:unknown-owner");
        ProcessRegistryFile::update(&registry_file, |registry| {
            registry.push(forged);
            Ok(())
        })
        .unwrap();
        if stop_after_parent {
            assert!(pytxo_orchestrate::request_stop_experimental_routed_flow(
                &catalog,
                &reviewed.draft_id,
                run_id,
            )
            .unwrap()
            .is_some());
        }
        catalog
            .mark_routed_flow_recovery_required(&reviewed.draft_id, run_id)
            .unwrap();
        let owner = catalog
            .routed_flow_dispatch_owner(&reviewed.draft_id, run_id)
            .unwrap()
            .unwrap();
        rusqlite::Connection::open(repo.path().join(".git/catalog.db"))
            .unwrap()
            .execute(
                "UPDATE flow_dispatch_claims SET controller_start_identity=?1 WHERE draft_id=?2 AND run_id=?3 AND controller_pid=?4",
                rusqlite::params!["simulated-dead-controller", reviewed.draft_id, run_id, owner.controller_pid],
            )
            .unwrap();
        let marker_path = data_dir.join("active_run.json");
        let mut marker: serde_json::Value =
            serde_json::from_slice(&fs::read(&marker_path).unwrap()).unwrap();
        marker["supervisor_start_identity"] = "simulated-dead-controller".into();
        fs::write(&marker_path, serde_json::to_vec(&marker).unwrap()).unwrap();
        assert!(!reconcile_routed_flow_startup(&catalog, &reviewed.draft_id).unwrap());
        assert_eq!(store.get_run_status(run_id).unwrap().unwrap().0, "starting");
        ProcessRegistryFile::update(&registry_file, |registry| {
            registry.remove_agent(&format!("{run_id}:unknown-owner"));
            Ok(())
        })
        .unwrap();
        assert!(reconcile_routed_flow_startup(&catalog, &reviewed.draft_id).unwrap());
        let history = store.routing_history(&scope).unwrap().unwrap();
        assert_eq!(history.cancelled, !crash_after_final);
        assert_eq!(
            history.attempts.len(),
            if crash_after_final { 2 } else { 1 }
        );
        assert_eq!(
            store.get_run_status(run_id).unwrap().unwrap().0,
            if stop_after_parent {
                "cancelled"
            } else {
                "failed"
            }
        );
        assert!(ProcessRegistryFile::load(&registry_file)
            .unwrap()
            .for_run(run_id)
            .is_empty());
        assert!(!data_dir.join("active_run.json").exists());
        assert_eq!(
            catalog
                .get_flow_draft(&reviewed.draft_id)
                .unwrap()
                .unwrap()
                .status,
            if stop_after_parent {
                "cancelled"
            } else {
                "dispatched"
            }
        );
        if stop_after_parent {
            assert_eq!(history.attempts.len(), 1);
            assert_eq!(fs::read(repo.path().join("result.txt")).unwrap(), b"");
            return;
        }
        if crash_after_final {
            assert_eq!(
                store
                    .get_run_contract(run_id)
                    .unwrap()
                    .unwrap()
                    .apply_status,
                "review_failed"
            );
            unsafe { std::env::remove_var("PYTXO_TEST_ROUTED_CRASH_AFTER_FINAL_PASS") };
            if std::env::var_os("PYTXO_TEST_ROUTED_DISCARD_AFTER_FINAL_PASS").is_some() {
                discard_run_review(None, Some(repo.path().to_path_buf()), run_id).unwrap();
                assert_eq!(
                    store
                        .get_run_contract(run_id)
                        .unwrap()
                        .unwrap()
                        .apply_status,
                    "discarded"
                );
                assert!(!store
                    .unreconciled_registered_routing_scopes()
                    .unwrap()
                    .contains(&scope));
                write_exact_terminal_marker(
                    &catalog,
                    &reviewed.draft_id,
                    run_id,
                    &data_dir,
                    &reviewed.domain_id,
                );
                assert_eq!(
                    rusqlite::Connection::open(repo.path().join(".git/catalog.db"))
                        .unwrap()
                        .execute(
                            "UPDATE flow_drafts SET status='recovery_required' WHERE id=?1 AND dispatched_run_id=?2",
                            rusqlite::params![reviewed.draft_id, run_id],
                        )
                        .unwrap(),
                    1
                );
                assert!(reconcile_routed_flow_startup(&catalog, &reviewed.draft_id).unwrap());
                assert!(!data_dir.join("active_run.json").exists());
                return;
            }
            let manifest =
                refresh_run_review(None, Some(repo.path().to_path_buf()), run_id).unwrap();
            apply_run_changes(
                None,
                Some(repo.path().to_path_buf()),
                run_id,
                &manifest.package_digest,
            )
            .unwrap();
            assert_eq!(
                fs::read(repo.path().join("seed.txt")).unwrap(),
                b"pytxo-routed\r\n"
            );
            assert_eq!(
                fs::read(repo.path().join("result.txt")).unwrap(),
                b"pytxo-routed\r\n"
            );
        }
        return;
    }
    let dispatch = dispatch_experimental_routed_flow(&catalog, &reviewed.draft_id);
    let run_id = if review_fault || stop_between {
        let error = dispatch.unwrap_err();
        if review_fault {
            let expected =
                if std::env::var_os("PYTXO_TEST_ROUTED_EMPTY_CANDIDATE_CHECKS_BEFORE_PUBLICATION")
                    .is_some()
                {
                    "exact candidate is not verified"
                } else if std::env::var_os(
                    "PYTXO_TEST_ROUTED_WRONG_CANDIDATE_COMMAND_BEFORE_PUBLICATION",
                )
                .is_some()
                {
                    "routed Review candidate checks differ from registered recipes"
                } else if std::env::var_os(
                    "PYTXO_TEST_ROUTED_WRONG_CANDIDATE_MODE_BEFORE_PUBLICATION",
                )
                .is_some()
                {
                    "candidate inventory does not match exact prepared composition"
                } else {
                    "routed Review package digest changed"
                };
            assert!(format!("{error:#}").contains(expected), "{error:#}");
        } else {
            assert!(
                format!("{error:#}").contains("cancelled before admission"),
                "{error:#}"
            );
        }
        reviewed
            .routing
            .as_ref()
            .unwrap()
            .authorization
            .run_id
            .0
            .clone()
    } else {
        dispatch.unwrap_or_else(|error| {
            let stage_run_id = &reviewed.routing.as_ref().unwrap().authorization.run_id.0;
            let store = PytxoStore::open(&PytxoConfig::default().db_path_at(repo.path())).unwrap();
            let history = store
                .routing_history(&pytxo_store::routing::RoutingScope {
                    domain_id: pytxo_core::DomainId(reviewed.domain_id.clone()),
                    run_id: RunId(stage_run_id.clone()),
                })
                .unwrap();
            let summary = history.map(|history| {
                (
                    history.cancelled,
                    history
                        .tasks
                        .iter()
                        .map(|task| {
                            (
                                task.registration.contract.task_id.0.clone(),
                                task.state.clone(),
                            )
                        })
                        .collect::<Vec<_>>(),
                    history
                        .events
                        .iter()
                        .map(|event| event.event_id.clone())
                        .collect::<Vec<_>>(),
                )
            });
            panic!("two-wave dispatch failed: {error:#}; history: {summary:?}");
        })
    };
    let store = PytxoStore::open(&PytxoConfig::default().db_path_at(repo.path())).unwrap();
    let scope = pytxo_store::routing::RoutingScope {
        domain_id: pytxo_core::DomainId(reviewed.domain_id.clone()),
        run_id: RunId(run_id.clone()),
    };
    let history = store.routing_history(&scope).unwrap().unwrap();
    if worker_fail {
        assert!(history.cancelled);
        assert_eq!(history.attempts.len(), 1);
        assert_eq!(history.attempts[0].task_id.0, "seed");
        assert_eq!(history.attempts[0].state, AttemptState::Failed);
        assert_eq!(store.get_run_status(&run_id).unwrap().unwrap().0, "failed");
        assert_eq!(
            store
                .get_run_contract(&run_id)
                .unwrap()
                .unwrap()
                .apply_status,
            "pending"
        );
        assert_eq!(fs::read(repo.path().join("seed.txt")).unwrap(), b"");
        assert_eq!(fs::read(repo.path().join("result.txt")).unwrap(), b"");
        return;
    }
    if stop_between {
        assert!(history.cancelled);
        assert_eq!(history.attempts.len(), 1);
        assert_eq!(history.attempts[0].task_id.0, "seed");
        assert_eq!(history.attempts[0].state, AttemptState::Passed);
        assert_eq!(
            store.get_run_status(&run_id).unwrap().unwrap().0,
            "cancelled"
        );
        assert_eq!(
            store
                .get_run_contract(&run_id)
                .unwrap()
                .unwrap()
                .apply_status,
            "pending"
        );
        assert_eq!(fs::read(repo.path().join("seed.txt")).unwrap(), b"");
        assert_eq!(fs::read(repo.path().join("result.txt")).unwrap(), b"");
        return;
    }
    assert!(
        !history.cancelled,
        "successful two-wave fixture was cancelled: attempts={:?}; events={:?}",
        history
            .attempts
            .iter()
            .map(|attempt| (
                &attempt.task_id,
                &attempt.state,
                &attempt.receipts,
                &attempt.failure,
            ))
            .collect::<Vec<_>>(),
        history
            .events
            .iter()
            .map(|event| &event.event_id)
            .collect::<Vec<_>>()
    );
    assert_eq!(history.attempts.len(), 2);
    #[cfg(feature = "routed-test-faults")]
    if std::env::var("PYTXO_TEST_ROUTED_POWERSHELL_HANDOFF")
        .ok()
        .as_deref()
        == Some("1")
    {
        let seed = history
            .attempts
            .iter()
            .find(|attempt| attempt.task_id.0 == "seed")
            .unwrap();
        let result = history
            .attempts
            .iter()
            .find(|attempt| attempt.task_id.0 == "result")
            .unwrap();
        assert_eq!(seed.selected.profile.harness_id, "pytxo-local-fixture-v1");
        assert_eq!(
            result.selected.profile.harness_id,
            "pytxo-local-fixture-powershell-v1"
        );
        assert_ne!(
            seed.selected.observation.executable.digest,
            result.selected.observation.executable.digest
        );
        assert_ne!(
            seed.selected.observation.executable.path,
            result.selected.observation.executable.path
        );
        let producer = store
            .routed_worktree_instance(&seed.agent_id, &run_id, "seed")
            .unwrap()
            .unwrap();
        assert_eq!(
            fs::read(std::path::Path::new(&producer.path).join("seed.txt")).unwrap(),
            b"untrusted-worker-mutation\r\n"
        );
    }
    assert!(history.attempts.iter().all(|attempt| {
        attempt.state == AttemptState::Passed
            && attempt.ownership_released
            && matches!(
                attempt.usage,
                pytxo_store::routing::RoutedUsage::Known { nano_usd: 0, .. }
            )
    }));
    let inherited = store
        .read_routing_dependency_output(&scope, &TaskId("result".into()), &TaskId("seed".into()))
        .unwrap();
    #[cfg(feature = "routed-test-faults")]
    if std::env::var("PYTXO_TEST_ROUTED_POWERSHELL_HANDOFF")
        .ok()
        .as_deref()
        == Some("1")
    {
        let seed = history
            .attempts
            .iter()
            .find(|attempt| attempt.task_id.0 == "seed")
            .unwrap();
        assert_eq!(
            Some(&inherited.sealed_output.digest),
            seed.receipts.sealed_output.as_ref()
        );
    }
    let child = history
        .attempts
        .iter()
        .find(|attempt| attempt.task_id.0 == "result")
        .unwrap();
    assert_eq!(child.dependencies, [inherited.winner]);
    let handoff_ref = child
        .handoff
        .as_ref()
        .expect("child handoff must be retained");
    let handoff_claim = pytxo_store::routing_private::handoff_manifest_claim(
        &scope,
        &child.task_id,
        &child.attempt_id,
        &child.capacity_reservation,
    );
    let handoff_bytes = store
        .read_private_artifact(&handoff_claim, handoff_ref)
        .unwrap();
    let handoff: pytxo_core::routing::PortableHandoffManifest =
        serde_json::from_slice(&handoff_bytes).unwrap();
    assert_eq!(handoff.dependencies, child.dependencies);
    assert_eq!(handoff.origin.attempt_id, child.attempt_id);
    let contract = store.get_run_contract(&run_id).unwrap().unwrap();
    let manifest = if review_fault {
        assert_eq!(contract.apply_status, "review_failed");
        assert!(contract.prepared_manifest.is_none());
        unsafe {
            std::env::remove_var("PYTXO_TEST_ROUTED_MUTATE_MANIFEST_BEFORE_PUBLICATION");
            std::env::remove_var("PYTXO_TEST_ROUTED_EMPTY_CANDIDATE_CHECKS_BEFORE_PUBLICATION");
            std::env::remove_var("PYTXO_TEST_ROUTED_WRONG_CANDIDATE_COMMAND_BEFORE_PUBLICATION");
            std::env::remove_var("PYTXO_TEST_ROUTED_WRONG_CANDIDATE_MODE_BEFORE_PUBLICATION");
        }
        refresh_run_review(None, Some(repo.path().to_path_buf()), &run_id).unwrap()
    } else {
        assert_eq!(contract.apply_status, "ready");
        contract.prepared_manifest.unwrap()
    };
    assert_eq!(manifest.files.len(), 2);
    assert_eq!(
        manifest
            .files
            .iter()
            .map(|file| file.path.as_str())
            .collect::<BTreeSet<_>>(),
        BTreeSet::from(["seed.txt", "result.txt"])
    );
    assert!(manifest.candidate_verification.is_some());
    if recover_completed {
        let data_dir = repo.path().join(".pytxo/data");
        let registry_file = registry_path(&data_dir);
        let mut registry = ProcessRegistryFile::default();
        for attempt in &history.attempts {
            let worker = store
                .launch_ownership(&attempt.attempt_id)
                .unwrap()
                .unwrap();
            registry.push(ProcessEntry {
                run_id: run_id.clone(),
                repo_root: reviewed.domain_id.clone(),
                agent_key: format!("{run_id}:{}", attempt.attempt_id.0),
                pid: worker.pid.unwrap(),
                start_identity: worker.start_identity,
                worktree_path: String::new(),
                branch: String::new(),
            });
            let task = history
                .tasks
                .iter()
                .find(|task| task.registration.contract.task_id == attempt.task_id)
                .unwrap();
            for index in 0..task.registration.contract.checks.len() {
                let ordinal = u32::try_from(index + 1).unwrap();
                let checker = store
                    .checker_ownership(&attempt.attempt_id, ordinal)
                    .unwrap()
                    .unwrap();
                registry.push(ProcessEntry {
                    run_id: run_id.clone(),
                    repo_root: reviewed.domain_id.clone(),
                    agent_key: format!("{run_id}:{}:checker-{ordinal}", attempt.attempt_id.0),
                    pid: checker.pid.unwrap(),
                    start_identity: checker.start_identity,
                    worktree_path: String::new(),
                    branch: String::new(),
                });
            }
        }
        registry.save(&registry_file).unwrap();
        write_exact_terminal_marker(
            &catalog,
            &reviewed.draft_id,
            &run_id,
            &data_dir,
            &reviewed.domain_id,
        );
        assert_eq!(
            rusqlite::Connection::open(repo.path().join(".git/catalog.db"))
                .unwrap()
                .execute(
                    "UPDATE flow_drafts SET status='recovery_required' WHERE id=?1 AND dispatched_run_id=?2",
                    rusqlite::params![reviewed.draft_id, run_id],
                )
                .unwrap(),
            1
        );
        assert!(reconcile_routed_flow_startup(&catalog, &reviewed.draft_id).unwrap());
        assert!(ProcessRegistryFile::load(&registry_file)
            .unwrap()
            .for_run(&run_id)
            .is_empty());
        assert!(!data_dir.join("active_run.json").exists());
        assert_eq!(
            store.get_run_status(&run_id).unwrap().unwrap().0,
            "completed"
        );
        assert_eq!(
            store
                .get_run_contract(&run_id)
                .unwrap()
                .unwrap()
                .apply_status,
            "ready"
        );
    }
    assert_eq!(fs::read(repo.path().join("seed.txt")).unwrap(), b"");
    assert_eq!(fs::read(repo.path().join("result.txt")).unwrap(), b"");
    apply_run_changes(
        None,
        Some(repo.path().to_path_buf()),
        &run_id,
        &manifest.package_digest,
    )
    .unwrap();
    assert_eq!(
        fs::read(repo.path().join("seed.txt")).unwrap(),
        b"pytxo-routed\r\n"
    );
    assert_eq!(
        fs::read(repo.path().join("result.txt")).unwrap(),
        b"pytxo-routed\r\n"
    );
}

#[cfg(feature = "routed-test-faults")]
#[test]
fn journal_backed_mock_shadow_reaches_review_and_apply() {
    let _env_lock = FIXTURE_ENV_LOCK.lock().unwrap();
    unsafe { std::env::set_var("PYTXO_TEST_ROUTED_SHADOW_MOCK_ADVICE", "1") };
    run_reviewed_fixture(AttemptState::Passed, "completed");
    unsafe { std::env::remove_var("PYTXO_TEST_ROUTED_SHADOW_MOCK_ADVICE") };
}

#[cfg(feature = "routed-test-faults")]
fn run_claimed_hosted_shadow_fixture(crash_stage: Option<&str>, revoke_at_send: bool) {
    use pytxo_orchestrate::{
        dispatch_experimental_hosted_shadow_with_client, HostedClientFuture,
        HostedEvaluationRequest, HostedShadowClient,
    };
    use pytxo_store::{HostedGrantIntent, HostedRemoteGrantReceipt};

    struct FakeHostedClient {
        workspace_id: String,
        sends: Arc<AtomicUsize>,
        revoke_at_send: Option<(std::path::PathBuf, String)>,
    }
    impl HostedShadowClient for FakeHostedClient {
        fn recipient_identity(&self) -> &str {
            pytxo_planner::advisor::HOSTED_RECIPIENT
        }
        fn account_id(&self) -> &str {
            "user_test"
        }
        fn link_origin(&self) -> &str {
            "https://link.pytxo.com"
        }
        fn workspace_id(&self) -> &str {
            &self.workspace_id
        }
        fn grant_revision(&self) -> u64 {
            1
        }
        fn evaluate<'a>(&'a self, request: &'a HostedEvaluationRequest) -> HostedClientFuture<'a> {
            if let Some((catalog_path, domain_id)) = &self.revoke_at_send {
                let revoker = Catalog::open(catalog_path).unwrap();
                let revoked = pytxo_orchestrate::revoke_experimental_hosted_advisor_local_consent(
                    &revoker, domain_id, 1,
                )
                .unwrap();
                assert!(!revoked.enabled);
                std::thread::sleep(std::time::Duration::from_millis(30));
            }
            Box::pin(async move {
                self.sends.fetch_add(1, Ordering::SeqCst);
                Ok(serde_json::to_vec(&serde_json::json!({
                    "schema_version": 1,
                    "evaluation_id": request.request_id,
                    "request_id": request.request_id,
                    "question_set_version": pytxo_planner::advisor::TEMPLATE_VERSION,
                    "model_id": pytxo_planner::advisor::MODEL_ID,
                    "choice": "everyday_fit",
                    "distribution": {"everyday_fit": 0.9, "strong_needed": 0.08, "unclear": 0.02},
                    "usage_status": "known",
                    "usage_receipt_id": "r_0123456789abcdef0123456789abcdef",
                    "input_tokens": 10,
                    "output_tokens": 1,
                    "cost_nano_usd": 420,
                    "packet_digest": request.packet_digest.0,
                }))
                .unwrap())
            })
        }
    }

    let _env_lock = FIXTURE_ENV_LOCK.lock().unwrap();
    let home = tempfile::tempdir().unwrap();
    unsafe {
        std::env::set_var("PYTXO_HOME", home.path().join("home"));
        std::env::set_var("PYTXO_TRUST_STORE", home.path().join("trust.json"));
        std::env::set_var("PYTXO_EXPERIMENTAL_ROUTED_LOCAL_FIXTURE", "1");
        std::env::set_var("PYTXO_TEST_HOSTED_SHADOW_DISPATCH", "1");
    }
    let repo = tempfile::tempdir().unwrap();
    fs::write(repo.path().join("result.txt"), "").unwrap();
    fs::write(
        repo.path().join("pytxo.toml"),
        "permission_profile = 'orbit'\nexecution_backend = 'subprocess'\nmax_agents = 1\n[blast]\nprefer_kernel_overlay = false\n[[task]]\nid = 'fixture'\nagent = 'default'\npaths = ['result.txt']\n",
    )
    .unwrap();
    for args in [
        vec!["init", "-q"],
        vec!["add", "."],
        vec![
            "-c",
            "user.name=Test",
            "-c",
            "user.email=test@pytxo.local",
            "commit",
            "-qm",
            "baseline",
        ],
    ] {
        let output = Command::new("git")
            .args(args)
            .current_dir(repo.path())
            .output()
            .unwrap();
        assert!(
            output.status.success(),
            "{}",
            String::from_utf8_lossy(&output.stderr)
        );
    }
    trust_repo(repo.path(), PermissionProfile::Orbit).unwrap();
    let catalog = Catalog::open(&repo.path().join(".git/catalog.db")).unwrap();
    catalog
        .configure_capacity_pool(&CapacityPoolConfig {
            resource_id: "host".into(),
            capacity_units: 1,
            expected_revision: None,
            configured_at_ms: 100,
        })
        .unwrap();
    let preview = preview_experimental_routed_flow(
        &catalog,
        FlowDraftInput {
            id: "claimed-hosted-shadow".into(),
            title: "Claimed hosted Shadow".into(),
            mission_text: "pytxo-local-fixture-v1:write-result".into(),
            source: FlowSource::Text,
            domain_id: Some(repo.path().to_string_lossy().into_owned()),
            project_id: None,
            ade_id: None,
            max_workers: Some(1),
            verification_commands: vec!["if exist result.txt (exit /b 0) else (exit /b 1)".into()],
        },
        hosted_shadow_mission,
    )
    .unwrap();
    assert_eq!(preview.status, pytxo_orchestrate::FlowStatus::ReviewOnly);
    let reviewed = save_reviewed_flow_plan(&catalog, preview).unwrap();
    assert!(dispatch_experimental_routed_flow(&catalog, &reviewed.draft_id).is_err());
    let early_sends = Arc::new(AtomicUsize::new(0));
    assert!(dispatch_experimental_hosted_shadow_with_client(
        &catalog,
        &reviewed.draft_id,
        Arc::new(FakeHostedClient {
            workspace_id: "unconfirmed-workspace".into(),
            sends: Arc::clone(&early_sends),
            revoke_at_send: None,
        }),
    )
    .is_err());
    assert_eq!(early_sends.load(Ordering::SeqCst), 0);
    assert_eq!(
        catalog
            .get_flow_draft(&reviewed.draft_id)
            .unwrap()
            .unwrap()
            .status,
        "review_only"
    );
    let packet =
        pytxo_orchestrate::preview_reviewed_hosted_advisor_packet(&catalog, &reviewed.draft_id)
            .unwrap();
    pytxo_orchestrate::enable_experimental_hosted_advisor_local_consent(
        &catalog,
        &reviewed.draft_id,
        &packet,
        0,
    )
    .unwrap();
    let pinned = catalog
        .routing_advisor_consent_store(&packet.domain_id)
        .unwrap()
        .unwrap();
    let store_path = PytxoConfig::default().db_path_at(repo.path());
    catalog
        .upsert_domain(
            &packet.domain_id,
            &packet.domain_id,
            &store_path.to_string_lossy(),
            None,
        )
        .unwrap();
    let workspace_id = catalog
        .ensure_hosted_workspace_id(
            &packet.domain_id,
            &pinned.store_db_path,
            &pinned.store_db_file_identity,
        )
        .unwrap();
    let intent = HostedGrantIntent {
        domain_id: packet.domain_id.clone(),
        workspace_id: workspace_id.clone(),
        account_id: "user_test".into(),
        link_origin: "https://link.pytxo.com".into(),
        recipient_identity: packet.recipient_identity.clone(),
        scope_digest: packet.scope_digest.0.clone(),
        store_db_file_identity: packet.store_db_file_identity.clone(),
        consent_revision: 1,
    };
    catalog.begin_hosted_grant(&intent).unwrap();
    catalog
        .confirm_hosted_grant(
            &intent,
            &HostedRemoteGrantReceipt {
                workspace_id: workspace_id.clone(),
                account_id: intent.account_id.clone(),
                link_origin: intent.link_origin.clone(),
                recipient_identity: intent.recipient_identity.clone(),
                scope_digest: intent.scope_digest.clone(),
                revision: 1,
                enabled: true,
            },
        )
        .unwrap();
    let sends = Arc::new(AtomicUsize::new(0));
    let client: Arc<dyn HostedShadowClient> = Arc::new(FakeHostedClient {
        workspace_id,
        sends: Arc::clone(&sends),
        revoke_at_send: revoke_at_send.then(|| {
            (
                repo.path().join(".git/catalog.db"),
                packet.domain_id.clone(),
            )
        }),
    });
    let run_id = if let Some(stage) = crash_stage {
        unsafe { std::env::set_var(stage, "1") };
        assert!(catch_unwind(AssertUnwindSafe(|| {
            dispatch_experimental_hosted_shadow_with_client(&catalog, &reviewed.draft_id, client)
        }))
        .is_err());
        unsafe { std::env::remove_var(stage) };
        reviewed
            .routing
            .as_ref()
            .unwrap()
            .authorization
            .run_id
            .0
            .clone()
    } else {
        dispatch_experimental_hosted_shadow_with_client(&catalog, &reviewed.draft_id, client)
            .unwrap()
    };
    if let Some(stage) = crash_stage {
        use pytxo_store::routing::AdvisorSendPhase;
        let expected_sends = usize::from(stage == "PYTXO_TEST_HOSTED_SHADOW_CRASH_AFTER_COMPLETED");
        let before_request = matches!(
            stage,
            "PYTXO_TEST_HOSTED_SHADOW_CRASH_AFTER_FIRST_QUALIFICATION"
                | "PYTXO_TEST_HOSTED_SHADOW_CRASH_AFTER_SECOND_QUALIFICATION"
        );
        assert_eq!(sends.load(Ordering::SeqCst), expected_sends);
        let store = PytxoStore::open(&store_path).unwrap();
        let scope = pytxo_store::routing::RoutingScope {
            domain_id: pytxo_core::DomainId(reviewed.domain_id.clone()),
            run_id: RunId(run_id.clone()),
        };
        let before = store.routing_history(&scope).unwrap().unwrap();
        assert!(before.attempts.is_empty());
        let requests = store.routing_advisor_requests_for_recovery(&scope).unwrap();
        if before_request {
            assert!(requests.is_empty());
        } else {
            assert_eq!(requests.len(), 1);
            let expected_phase = match stage {
                "PYTXO_TEST_HOSTED_SHADOW_CRASH_AFTER_PREPARE" => AdvisorSendPhase::Prepared,
                "PYTXO_TEST_HOSTED_SHADOW_CRASH_AFTER_MAY_SEND" => {
                    AdvisorSendPhase::SendingMayHaveHappened
                }
                "PYTXO_TEST_HOSTED_SHADOW_CRASH_AFTER_COMPLETED" => AdvisorSendPhase::Completed,
                _ => panic!("unknown hosted Shadow crash stage"),
            };
            assert_eq!(requests[0].phase, expected_phase);
            assert_eq!(
                requests[0].policy_digest, before.mission.authorization.policy_digest,
                "hosted request policy must match its registered mission"
            );
        }
        let owner = catalog
            .routed_flow_dispatch_owner(&reviewed.draft_id, &run_id)
            .unwrap()
            .unwrap();
        let marker_path = repo.path().join(".pytxo/data/active_run.json");
        let mut marker: serde_json::Value =
            serde_json::from_slice(&fs::read(&marker_path).unwrap()).unwrap();
        marker["supervisor_start_identity"] = "simulated-dead-controller".into();
        rusqlite::Connection::open(repo.path().join(".git/catalog.db"))
            .unwrap()
            .execute(
                "UPDATE flow_dispatch_claims SET controller_start_identity=?1 WHERE draft_id=?2 AND run_id=?3 AND controller_pid=?4",
                rusqlite::params!["simulated-dead-controller", reviewed.draft_id, run_id, owner.controller_pid],
            )
            .unwrap();
        fs::write(&marker_path, serde_json::to_vec(&marker).unwrap()).unwrap();
        catalog
            .mark_routed_flow_recovery_required(&reviewed.draft_id, &run_id)
            .unwrap();
        assert!(reconcile_routed_flow_startup(&catalog, &reviewed.draft_id).unwrap());
        let after = store.routing_history(&scope).unwrap().unwrap();
        assert!(after.cancelled && after.attempts.is_empty());
        assert_eq!(
            store.get_run_status(&run_id).unwrap().unwrap().0,
            "failed_startup"
        );
        let requests = store.routing_advisor_requests_for_recovery(&scope).unwrap();
        if before_request {
            assert!(requests.is_empty());
        } else {
            assert_eq!(requests.len(), 1);
            assert_eq!(
                requests[0].phase,
                if stage == "PYTXO_TEST_HOSTED_SHADOW_CRASH_AFTER_COMPLETED" {
                    AdvisorSendPhase::Completed
                } else {
                    AdvisorSendPhase::Uncertain
                }
            );
        }
        assert!(dispatch_experimental_hosted_shadow_with_client(
            &catalog,
            &reviewed.draft_id,
            Arc::new(FakeHostedClient {
                workspace_id: intent.workspace_id,
                sends: Arc::clone(&sends),
                revoke_at_send: None,
            }),
        )
        .is_err());
        assert_eq!(sends.load(Ordering::SeqCst), expected_sends);
        unsafe { std::env::remove_var("PYTXO_TEST_HOSTED_SHADOW_DISPATCH") };
        return;
    }
    assert_eq!(sends.load(Ordering::SeqCst), 1);
    let consent_after_claim = pytxo_orchestrate::read_experimental_hosted_advisor_local_consent(
        &catalog,
        &packet.domain_id,
    )
    .unwrap();
    if revoke_at_send {
        assert!(!consent_after_claim.enabled);
    } else {
        assert!(consent_after_claim.enabled && consent_after_claim.current_scope);
    }
    let store = PytxoStore::open(&store_path).unwrap();
    let history = store
        .routing_history(&pytxo_store::routing::RoutingScope {
            domain_id: pytxo_core::DomainId(reviewed.domain_id.clone()),
            run_id: RunId(run_id.clone()),
        })
        .unwrap()
        .unwrap();
    assert_eq!(
        store.get_run_status(&run_id).unwrap().unwrap().0,
        "completed"
    );
    assert_eq!(history.attempts.len(), 1);
    assert_eq!(history.attempts[0].state, AttemptState::Passed);
    assert_eq!(history.attempts[0].selected.profile.id.0, "strong");
    if revoke_at_send {
        assert_ne!(
            history.attempts[0].decision.advice_status,
            AdviceStatus::ShadowRecorded
        );
    } else {
        assert_eq!(
            history.attempts[0].decision.advice_status,
            AdviceStatus::ShadowRecorded
        );
    }
    let manifest = store
        .get_run_contract(&run_id)
        .unwrap()
        .unwrap()
        .prepared_manifest
        .unwrap();
    assert_eq!(fs::read(repo.path().join("result.txt")).unwrap(), b"");
    apply_run_changes(
        None,
        Some(repo.path().to_path_buf()),
        &run_id,
        &manifest.package_digest,
    )
    .unwrap();
    assert_eq!(
        fs::read(repo.path().join("result.txt")).unwrap(),
        b"pytxo-routed\r\n"
    );
    assert!(dispatch_experimental_hosted_shadow_with_client(
        &catalog,
        &reviewed.draft_id,
        Arc::new(FakeHostedClient {
            workspace_id: intent.workspace_id,
            sends: Arc::clone(&sends),
            revoke_at_send: None,
        }),
    )
    .is_err());
    assert_eq!(sends.load(Ordering::SeqCst), 1);
    unsafe { std::env::remove_var("PYTXO_TEST_HOSTED_SHADOW_DISPATCH") };
}

#[cfg(feature = "routed-test-faults")]
#[test]
fn claimed_hosted_shadow_fake_client_reaches_review_and_apply_without_changing_rules() {
    run_claimed_hosted_shadow_fixture(None, false);
}

#[cfg(feature = "routed-test-faults")]
#[test]
fn claimed_hosted_shadow_revocation_during_response_discards_advice() {
    run_claimed_hosted_shadow_fixture(None, true);
}

#[cfg(feature = "routed-test-faults")]
#[test]
fn crashed_hosted_shadow_may_send_recovers_without_retry_or_attempt() {
    run_claimed_hosted_shadow_fixture(Some("PYTXO_TEST_HOSTED_SHADOW_CRASH_AFTER_MAY_SEND"), false);
}

#[cfg(feature = "routed-test-faults")]
#[test]
fn crashed_hosted_shadow_prepared_request_recovers_without_send() {
    run_claimed_hosted_shadow_fixture(Some("PYTXO_TEST_HOSTED_SHADOW_CRASH_AFTER_PREPARE"), false);
}

#[cfg(feature = "routed-test-faults")]
#[test]
fn crashed_hosted_shadow_completed_request_recovers_without_admission() {
    run_claimed_hosted_shadow_fixture(
        Some("PYTXO_TEST_HOSTED_SHADOW_CRASH_AFTER_COMPLETED"),
        false,
    );
}

#[cfg(feature = "routed-test-faults")]
#[test]
fn crashed_hosted_shadow_after_first_qualification_recovers_without_send() {
    run_claimed_hosted_shadow_fixture(
        Some("PYTXO_TEST_HOSTED_SHADOW_CRASH_AFTER_FIRST_QUALIFICATION"),
        false,
    );
}

#[cfg(feature = "routed-test-faults")]
#[test]
fn crashed_hosted_shadow_after_second_qualification_recovers_without_send() {
    run_claimed_hosted_shadow_fixture(
        Some("PYTXO_TEST_HOSTED_SHADOW_CRASH_AFTER_SECOND_QUALIFICATION"),
        false,
    );
}

#[cfg(feature = "routed-test-faults")]
#[test]
fn failed_routed_review_recovers_only_the_retained_winner() {
    let _env_lock = FIXTURE_ENV_LOCK.lock().unwrap();
    unsafe { std::env::set_var("PYTXO_TEST_ROUTED_FAIL_AFTER_REVIEW_BEGIN", "1") };
    run_reviewed_fixture(AttemptState::Passed, "failed");
    unsafe { std::env::remove_var("PYTXO_TEST_ROUTED_FAIL_AFTER_REVIEW_BEGIN") };
    unsafe { std::env::set_var("PYTXO_TEST_ROUTED_FAIL_BEFORE_REVIEW_VIEW", "1") };
    run_reviewed_fixture(AttemptState::Passed, "failed");
    unsafe { std::env::remove_var("PYTXO_TEST_ROUTED_FAIL_BEFORE_REVIEW_VIEW") };
    unsafe {
        std::env::set_var("PYTXO_TEST_ROUTED_FAIL_BEFORE_REVIEW_VIEW", "1");
        std::env::set_var("PYTXO_TEST_ROUTED_DISCARD_BEFORE_REVIEW_VIEW", "1");
    }
    run_reviewed_fixture(AttemptState::Passed, "failed");
    unsafe {
        std::env::remove_var("PYTXO_TEST_ROUTED_DISCARD_BEFORE_REVIEW_VIEW");
        std::env::remove_var("PYTXO_TEST_ROUTED_FAIL_BEFORE_REVIEW_VIEW");
    }
    unsafe {
        std::env::set_var("PYTXO_TEST_ROUTED_FAIL_BEFORE_REVIEW_VIEW", "1");
        std::env::set_var("PYTXO_TEST_ROUTED_DISCARD_BEFORE_REVIEW_VIEW", "1");
        std::env::set_var("PYTXO_TEST_ROUTED_REPLACE_ORIGINAL_WORKTREE", "1");
    }
    run_reviewed_fixture(AttemptState::Passed, "failed");
    unsafe {
        std::env::remove_var("PYTXO_TEST_ROUTED_REPLACE_ORIGINAL_WORKTREE");
        std::env::remove_var("PYTXO_TEST_ROUTED_DISCARD_BEFORE_REVIEW_VIEW");
        std::env::remove_var("PYTXO_TEST_ROUTED_FAIL_BEFORE_REVIEW_VIEW");
    }
    unsafe {
        std::env::set_var("PYTXO_TEST_ROUTED_FAIL_BEFORE_REVIEW_VIEW", "1");
        std::env::set_var("PYTXO_TEST_ROUTED_DISCARD_BEFORE_REVIEW_VIEW", "1");
        std::env::set_var("PYTXO_TEST_ROUTED_REPLACE_ORIGINAL_WORKTREE", "1");
        std::env::set_var("PYTXO_TEST_ROUTED_REPLACE_SAME_BASE", "1");
    }
    run_reviewed_fixture(AttemptState::Passed, "failed");
    unsafe {
        std::env::remove_var("PYTXO_TEST_ROUTED_REPLACE_SAME_BASE");
        std::env::remove_var("PYTXO_TEST_ROUTED_REPLACE_ORIGINAL_WORKTREE");
        std::env::remove_var("PYTXO_TEST_ROUTED_DISCARD_BEFORE_REVIEW_VIEW");
        std::env::remove_var("PYTXO_TEST_ROUTED_FAIL_BEFORE_REVIEW_VIEW");
    }
}

#[cfg(feature = "routed-test-faults")]
#[test]
fn routed_review_refresh_rejects_changed_execution_backend() {
    let _env_lock = FIXTURE_ENV_LOCK.lock().unwrap();
    unsafe { std::env::set_var("PYTXO_TEST_ROUTED_FAIL_AFTER_REVIEW_BEGIN", "1") };
    run_reviewed_fixture(AttemptState::Passed, "failed");
    unsafe { std::env::remove_var("PYTXO_TEST_ROUTED_FAIL_AFTER_REVIEW_BEGIN") };
}

#[test]
fn overlay_routed_fixture_is_rejected_before_run_claim() {
    let _env_lock = FIXTURE_ENV_LOCK.lock().unwrap();
    run_reviewed_fixture_with_config(AttemptState::Passed, "completed", true);
}
