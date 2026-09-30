//! Local benchmark evidence export. This target is absent from normal builds.
//! It reads an existing domain Store and a private, frozen assignment binding;
//! it never launches workers, contacts an advisor, or changes routing policy.

use std::collections::{BTreeMap, BTreeSet};
use std::fs::{self, OpenOptions};
use std::io::Write;
use std::path::{Path, PathBuf};

use anyhow::{bail, Context, Result};
use clap::Parser;
use pytxo_core::routing::{canonical_digest, Digest, RoutingMode};
use pytxo_core::{DomainId, PytxoConfig, TaskId};
use pytxo_store::routing::{RoutingBenchmarkRunPins, RoutingBenchmarkTrace, RoutingScope};
use pytxo_store::PytxoStore;
use serde::Deserialize;

#[derive(Parser)]
#[command(about = "Experimental, local-only Store trace export for a frozen routing assignment")]
struct Args {
    /// Repository root for the execution domain; must already exist.
    #[arg(long)]
    repo: PathBuf,
    /// Private REGISTRATION.json used by record-trial.mjs --freeze. Never publish it.
    #[arg(long)]
    registration: PathBuf,
    #[arg(long)]
    assignment_id: String,
    /// New trace file. Existing files are never overwritten.
    #[arg(long)]
    output: PathBuf,
}

#[derive(Deserialize)]
struct PrivateRegistration {
    assignments: Option<Vec<PrivateAssignment>>,
    frozen_confirmation: Option<PrivateFrozenConfirmation>,
    registered_freeze_digest: Option<String>,
    pins: Option<PrivatePins>,
    bindings: Vec<PrivateBinding>,
}

#[derive(Deserialize)]
struct PrivateFrozenConfirmation {
    assignments: Vec<PrivateAssignment>,
    pins: PrivatePins,
    freeze_digest: String,
}

#[derive(Deserialize)]
struct PrivateAssignment {
    assignment_id: String,
    snapshot_digest: Digest,
    arm: String,
}

#[derive(Deserialize)]
struct PrivatePins {
    profile_pair_digest: Digest,
    adapter_digest: Digest,
    r0_policy_digest: Digest,
    rj_policy_digest: Digest,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct PrivateBinding {
    assignment_id: String,
    task_contract_digest: Digest,
    scope: RoutingScope,
    task_id: TaskId,
    alias_key_hex: String,
}

struct SelectedAssignment {
    scope: RoutingScope,
    task_id: TaskId,
    key: [u8; 32],
    expected_mode: RoutingMode,
    expected_pins: RoutingBenchmarkRunPins,
}

fn require_digest(value: &Digest, label: &str) -> Result<()> {
    if !value.is_valid() {
        bail!("{label} must be a lowercase SHA-256 digest");
    }
    Ok(())
}

fn decode_key(hex: &str) -> Result<[u8; 32]> {
    if hex.len() != 64
        || !hex
            .bytes()
            .all(|b| b.is_ascii_digit() || (b'a'..=b'f').contains(&b))
    {
        bail!("assignment alias key must be 32 lowercase hex bytes");
    }
    let mut bytes = [0; 32];
    for (index, byte) in bytes.iter_mut().enumerate() {
        *byte = u8::from_str_radix(&hex[index * 2..index * 2 + 2], 16)?;
    }
    if bytes == [0; 32] {
        bail!("assignment alias key must be nonzero");
    }
    Ok(bytes)
}

// Same assignment-keyed digest alias as PytxoStore::routing_benchmark_trace.
fn alias_digest(key: &[u8; 32], raw: &Digest) -> Digest {
    let mut preimage = b"pytxo-benchmark-alias-v1\0".to_vec();
    preimage.extend_from_slice(key);
    preimage.extend_from_slice(&6u32.to_le_bytes());
    preimage.extend_from_slice(b"digest");
    preimage.extend_from_slice(raw.0.as_bytes());
    Digest::of_bytes(&preimage)
}

fn select_assignment(
    registration: PrivateRegistration,
    assignment_id: &str,
    repo: &Path,
) -> Result<SelectedAssignment> {
    let (assignments, pins) = match (
        registration.assignments,
        registration.frozen_confirmation,
        registration.registered_freeze_digest,
        registration.pins,
    ) {
        (Some(assignments), None, None, Some(pins)) => (assignments, pins),
        (None, Some(frozen), Some(registered), None)
            if Digest(registered.clone()).is_valid() && registered == frozen.freeze_digest =>
        {
            (frozen.assignments, frozen.pins)
        }
        _ => bail!("registration must contain exactly one fixture or registered confirmation assignment source"),
    };
    if assignments.is_empty() || assignments.len() != registration.bindings.len() {
        bail!("registration needs one private binding per assignment");
    }
    for (label, digest) in [
        ("profile pair", &pins.profile_pair_digest),
        ("adapter", &pins.adapter_digest),
        ("R0 policy", &pins.r0_policy_digest),
        ("RJ policy", &pins.rj_policy_digest),
    ] {
        require_digest(digest, label)?;
    }

    let mut by_id = BTreeMap::new();
    for assignment in &assignments {
        require_digest(&assignment.snapshot_digest, "snapshot")?;
        if !matches!(assignment.arm.as_str(), "R0" | "RJ")
            || assignment.assignment_id.is_empty()
            || by_id
                .insert(&assignment.assignment_id, assignment)
                .is_some()
        {
            bail!("registration has duplicate or invalid assignments");
        }
    }

    let mut used_keys = BTreeSet::new();
    let mut used_scopes = BTreeSet::new();
    let mut used_bindings = BTreeSet::new();
    let mut selected = None;
    for binding in registration.bindings {
        if !by_id.contains_key(&binding.assignment_id)
            || !used_bindings.insert(binding.assignment_id.clone())
            || binding.scope.domain_id.0.is_empty()
            || binding.scope.run_id.0.is_empty()
            || binding.task_id.0.is_empty()
            || !used_scopes.insert((
                binding.scope.domain_id.0.clone(),
                binding.scope.run_id.0.clone(),
            ))
        {
            bail!("registration has a missing, duplicate, or invalid private binding");
        }
        require_digest(&binding.task_contract_digest, "task contract")?;
        let key = decode_key(&binding.alias_key_hex)?;
        if !used_keys.insert(binding.alias_key_hex) {
            bail!("registration reuses an assignment alias key");
        }
        if binding.assignment_id == assignment_id {
            let assignment = by_id[&binding.assignment_id];
            let expected_mode = if assignment.arm == "R0" {
                RoutingMode::Rules
            } else {
                RoutingMode::Live
            };
            let policy = if assignment.arm == "R0" {
                &pins.r0_policy_digest
            } else {
                &pins.rj_policy_digest
            };
            selected = Some(SelectedAssignment {
                scope: binding.scope,
                task_id: binding.task_id,
                key,
                expected_mode,
                expected_pins: RoutingBenchmarkRunPins {
                    task_contract_digest: alias_digest(&key, &binding.task_contract_digest),
                    snapshot_digest: alias_digest(&key, &assignment.snapshot_digest),
                    profile_pair_digest: alias_digest(&key, &pins.profile_pair_digest),
                    policy_digest: alias_digest(&key, policy),
                    adapter_digest: alias_digest(&key, &pins.adapter_digest),
                },
            });
        }
    }
    let selection = selected.context("assignment ID has no frozen private binding")?;
    if selection.scope.domain_id != DomainId::from_repo_root(repo)? {
        bail!("assignment domain differs from the canonical repository root");
    }
    Ok(selection)
}

fn validate_trace(trace: &RoutingBenchmarkTrace, assignment: &SelectedAssignment) -> Result<()> {
    let scope = canonical_digest(&assignment.scope, 1)?;
    let task = canonical_digest(&assignment.task_id, 1)?;
    if trace.schema_version != 3
        || trace.scope_digest != alias_digest(&assignment.key, &scope)
        || trace.task_digest != alias_digest(&assignment.key, &task)
        || trace.mode != assignment.expected_mode
        || trace.run_pins != assignment.expected_pins
    {
        bail!("Store trace differs from frozen assignment scope, arm, or mission pins");
    }
    Ok(())
}

fn write_once(path: &Path, trace: &RoutingBenchmarkTrace) -> Result<()> {
    let mut bytes = serde_json::to_vec_pretty(trace)?;
    bytes.push(b'\n');
    let mut output = OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(path)
        .with_context(|| format!("create new trace at {}", path.display()))?;
    output.write_all(&bytes)?;
    output.sync_all()?;
    Ok(())
}

fn run(args: Args) -> Result<()> {
    let repo = pytxo_core::canonical_repo_root(&args.repo)
        .with_context(|| format!("canonicalize repository {}", args.repo.display()))?;
    let registration: PrivateRegistration = serde_json::from_slice(
        &fs::read(&args.registration).context("read private benchmark registration")?,
    )
    .context("parse private benchmark registration")?;
    let assignment = select_assignment(registration, &args.assignment_id, &repo)?;

    let config_path = repo.join("pytxo.toml");
    let config = if config_path.is_file() {
        PytxoConfig::load(&config_path)?
    } else {
        PytxoConfig::default()
    };
    let db_path = fs::canonicalize(config.db_path_at(&repo))
        .context("benchmark requires an existing Store for this repository")?;
    let store = PytxoStore::open_existing_read_only(&db_path)?;
    let trace = store
        .routing_benchmark_trace(&assignment.scope, &assignment.task_id, &assignment.key)?
        .context("frozen assignment has no matching reviewed Store mission")?;
    validate_trace(&trace, &assignment)?;
    write_once(&args.output, &trace)
}

fn main() {
    if let Err(error) = run(Args::parse()) {
        eprintln!("trace export failed: {error:#}");
        std::process::exit(1);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use pytxo_core::routing::RouteSelection;
    use pytxo_store::routing::{RecordRoutingBlock, RoutingFacts, RoutingMission};

    fn fixture_registration(repo: &Path) -> serde_json::Value {
        let domain = DomainId::from_repo_root(repo).unwrap().0;
        serde_json::json!({
            "assignments": [
                {"assignment_id": "rules_run", "snapshot_digest": "a".repeat(64), "arm": "R0"},
                {"assignment_id": "jev_run", "snapshot_digest": "a".repeat(64), "arm": "RJ"}
            ],
            "pins": {
                "profile_pair_digest": "b".repeat(64),
                "adapter_digest": "c".repeat(64),
                "r0_policy_digest": "d".repeat(64),
                "rj_policy_digest": "e".repeat(64)
            },
            "bindings": [
                {"assignment_id": "rules_run", "task_contract_digest": "f".repeat(64),
                 "scope": {"domain_id": domain, "run_id": "run_r0"}, "task_id": "task",
                 "alias_key_hex": "01".repeat(32)},
                {"assignment_id": "jev_run", "task_contract_digest": "f".repeat(64),
                 "scope": {"domain_id": domain, "run_id": "run_rj"}, "task_id": "task",
                 "alias_key_hex": "02".repeat(32)}
            ]
        })
    }

    #[test]
    fn decode_key_rejects_zero_and_noncanonical_hex() {
        assert!(decode_key(&"0".repeat(64)).is_err());
        assert!(decode_key(&"A".repeat(64)).is_err());
        assert_eq!(decode_key(&"01".repeat(32)).unwrap(), [1; 32]);
    }

    #[test]
    fn trace_alias_matches_recorder_protocol() {
        let digest = Digest("a".repeat(64));
        assert_eq!(
            alias_digest(&[7; 32], &digest).0,
            "389dad73dfb50adc96540fb3805de1d63051c4897e7326db1acaf7ce6db8dc23"
        );
    }

    #[test]
    fn selected_assignment_checks_arm_identity_and_all_frozen_pins() {
        let repo = std::env::current_dir().unwrap();
        let selection = select_assignment(
            serde_json::from_value(fixture_registration(&repo)).unwrap(),
            "jev_run",
            &repo,
        )
        .unwrap();
        assert_eq!(selection.expected_mode, RoutingMode::Live);
        assert_eq!(selection.scope.run_id.0, "run_rj");
        assert_eq!(
            selection.expected_pins.policy_digest,
            alias_digest(&[2; 32], &Digest("e".repeat(64)))
        );
        let mut trace = RoutingBenchmarkTrace {
            schema_version: 3,
            scope_digest: alias_digest(
                &selection.key,
                &canonical_digest(&selection.scope, 1).unwrap(),
            ),
            task_digest: alias_digest(
                &selection.key,
                &canonical_digest(&selection.task_id, 1).unwrap(),
            ),
            run_pins: selection.expected_pins.clone(),
            routing_revision: 1,
            mode: RoutingMode::Live,
            decision_links_complete: false,
            events: Vec::new(),
            attempts: Vec::new(),
            advisor_requests: Vec::new(),
        };
        validate_trace(&trace, &selection).unwrap();
        trace.run_pins.snapshot_digest = Digest("0".repeat(64));
        assert!(validate_trace(&trace, &selection).is_err());
        trace.run_pins = selection.expected_pins.clone();
        trace.mode = RoutingMode::Shadow;
        assert!(validate_trace(&trace, &selection).is_err());
    }

    #[test]
    fn registration_rejects_reused_key_and_wrong_domain() {
        let repo = std::env::current_dir().unwrap();
        let mut registration = fixture_registration(&repo);
        registration["bindings"][1]["alias_key_hex"] = serde_json::json!("01".repeat(32));
        assert!(select_assignment(
            serde_json::from_value(registration.clone()).unwrap(),
            "jev_run",
            &repo,
        )
        .is_err());
        registration["bindings"][1]["alias_key_hex"] = serde_json::json!("02".repeat(32));
        registration["bindings"][1]["scope"]["domain_id"] = serde_json::json!("other_repo");
        assert!(select_assignment(
            serde_json::from_value(registration).unwrap(),
            "jev_run",
            &repo,
        )
        .is_err());
    }

    #[test]
    fn trace_output_is_write_once() {
        let repo = std::env::current_dir().unwrap();
        let selection = select_assignment(
            serde_json::from_value(fixture_registration(&repo)).unwrap(),
            "rules_run",
            &repo,
        )
        .unwrap();
        let trace = RoutingBenchmarkTrace {
            schema_version: 3,
            scope_digest: alias_digest(
                &selection.key,
                &canonical_digest(&selection.scope, 1).unwrap(),
            ),
            task_digest: alias_digest(
                &selection.key,
                &canonical_digest(&selection.task_id, 1).unwrap(),
            ),
            run_pins: selection.expected_pins,
            routing_revision: 1,
            mode: RoutingMode::Rules,
            decision_links_complete: false,
            events: Vec::new(),
            attempts: Vec::new(),
            advisor_requests: Vec::new(),
        };
        let directory = std::env::temp_dir().join(format!(
            "pytxo-routing-trace-export-test-{}",
            uuid::Uuid::new_v4()
        ));
        fs::create_dir(&directory).unwrap();
        let path = directory.join("trace.json");
        write_once(&path, &trace).unwrap();
        let first = fs::read(&path).unwrap();
        assert!(write_once(&path, &trace).is_err());
        assert_eq!(fs::read(&path).unwrap(), first);
        fs::remove_file(path).unwrap();
        fs::remove_dir(directory).unwrap();
    }

    #[test]
    fn exports_a_real_store_trace_for_the_frozen_rules_assignment() {
        // A cross-language Node test can retain this otherwise disposable fixture.
        // Only an explicit, empty child of the OS temp directory may be reused.
        let retained_fixture =
            std::env::var_os("PYTXO_ROUTING_EXPORT_FIXTURE_DIR").map(PathBuf::from);
        let directory = if let Some(requested) = &retained_fixture {
            let directory = fs::canonicalize(requested).unwrap();
            let temp = fs::canonicalize(std::env::temp_dir()).unwrap();
            assert!(directory.starts_with(&temp) && directory != temp);
            assert_eq!(fs::read_dir(&directory).unwrap().count(), 0);
            directory
        } else {
            let directory = std::env::temp_dir()
                .join(format!("pytxo-routing-export-e2e-{}", uuid::Uuid::new_v4()));
            fs::create_dir(&directory).unwrap();
            directory
        };
        let repo = pytxo_core::canonical_repo_root(&directory).unwrap();
        let db_path = repo.join(".pytxo").join("data").join("pytxo.db");
        let db = PytxoStore::open(&db_path).unwrap();
        let mut mission: RoutingMission = serde_json::from_str(include_str!(
            "../../../pytxo-store/tests/fixtures/routing_pre_check_recipes_v8_registration.json"
        ))
        .unwrap();
        mission.authorization.domain_id = DomainId::from_repo_root(&repo).unwrap();
        let scope = RoutingScope {
            domain_id: mission.authorization.domain_id.clone(),
            run_id: mission.authorization.run_id.clone(),
        };
        let task = &mission.tasks[0].contract;
        db.insert_run(&scope.run_id.0, repo.to_str().unwrap())
            .unwrap();
        db.register_routing_mission(&mission).unwrap();
        let facts = RoutingFacts {
            now_ms: 150,
            observed_at_ms: 100,
            expires_at_ms: 200,
            base: task.base.clone(),
            plan_digest: task.plan_digest.clone(),
            permission_profile: task.permission_profile,
            observations: Vec::new(),
            manual_target: None,
            packet_digest: None,
            advice_request_id: None,
        };
        let decision = db
            .preview_routing_decision(&scope, &task.task_id, &facts, None)
            .unwrap();
        assert!(matches!(decision.selection, RouteSelection::Blocked(_)));
        db.record_routing_block(&RecordRoutingBlock {
            scope: scope.clone(),
            event_id: "no-qualified-profile".into(),
            task_id: task.task_id.clone(),
            facts,
            decision,
            advice_json: None,
            observation_event_id: None,
        })
        .unwrap();
        drop(db);

        let everyday = mission
            .profiles
            .iter()
            .find(|profile| {
                profile.profile.id == mission.policy.everyday.profile_id
                    && profile.binding.id == mission.policy.everyday.binding_id
            })
            .unwrap();
        let strong = mission
            .profiles
            .iter()
            .find(|profile| {
                profile.profile.id == mission.policy.strong.profile_id
                    && profile.binding.id == mission.policy.strong.binding_id
            })
            .unwrap();
        let pair = canonical_digest(
            &[
                everyday.profile.digest().unwrap().0,
                everyday.binding.digest().unwrap().0,
                strong.profile.digest().unwrap().0,
                strong.binding.digest().unwrap().0,
            ],
            1,
        )
        .unwrap();
        let registration = serde_json::json!({
            "assignments": [
                {"assignment_id": "frozen_r0", "snapshot_digest": task.base.snapshot_digest,
                 "packet_digest": Digest::of_bytes(b"fixture-reviewed-packet"), "arm": "R0"},
                {"assignment_id": "frozen_rj", "snapshot_digest": task.base.snapshot_digest,
                 "packet_digest": Digest::of_bytes(b"fixture-reviewed-packet"), "arm": "RJ"}
            ],
            "pins": {
                "profile_pair_digest": pair,
                "adapter_digest": everyday.profile.adapter_digest,
                "r0_policy_digest": mission.policy.digest().unwrap(),
                "rj_policy_digest": Digest::of_bytes(b"separate-frozen-rj-policy")
            },
            "bindings": [
                {"assignment_id": "frozen_r0", "task_contract_digest": task.digest().unwrap(),
                 "scope": scope, "task_id": task.task_id, "alias_key_hex": "07".repeat(32)},
                {"assignment_id": "frozen_rj", "task_contract_digest": task.digest().unwrap(),
                 "scope": {"domain_id": mission.authorization.domain_id, "run_id": "unused-rj-run"},
                 "task_id": task.task_id, "alias_key_hex": "08".repeat(32)}
            ]
        });
        let registration_path = repo.join("registration.json");
        let trace_path = repo.join("trace.json");
        fs::write(
            &registration_path,
            serde_json::to_vec(&registration).unwrap(),
        )
        .unwrap();
        let args = Args {
            repo: repo.clone(),
            registration: registration_path,
            assignment_id: "frozen_r0".into(),
            output: trace_path.clone(),
        };
        run(args).unwrap();
        let export: serde_json::Value =
            serde_json::from_slice(&fs::read(&trace_path).unwrap()).unwrap();
        assert_eq!(export["schema_version"], 3);
        assert_eq!(export["mode"], "rules");
        assert_eq!(export["decision_links_complete"], true);
        assert_eq!(export["events"][0]["kind"]["kind"], "blocked");
        assert!(export["attempts"].as_array().unwrap().is_empty());
        assert!(export["advisor_requests"].as_array().unwrap().is_empty());
        assert!(!serde_json::to_string(&export).unwrap().contains(&task.goal));
        let expected = select_assignment(
            serde_json::from_value(registration).unwrap(),
            "frozen_r0",
            &repo,
        )
        .unwrap();
        assert_eq!(
            export["run_pins"],
            serde_json::to_value(expected.expected_pins).unwrap()
        );
        assert!(repo.starts_with(pytxo_core::canonical_repo_root(&std::env::temp_dir()).unwrap()));
        if retained_fixture.is_none() {
            fs::remove_dir_all(repo).unwrap();
        }
    }
}
