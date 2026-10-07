import assert from "node:assert/strict";
import { execFile } from "node:child_process";
import { createHash } from "node:crypto";
import { mkdtemp, readFile, realpath, rm, writeFile } from "node:fs/promises";
import { tmpdir } from "node:os";
import path from "node:path";
import test from "node:test";
import { fileURLToPath } from "node:url";
import { promisify } from "node:util";

import { analyzeRoutingTrial } from "./analyze-trial.mjs";
import { freezeConfirmationCampaign } from "./freeze-confirmation.mjs";
import { freezeStudy, assembleStudy, createExecutionOrderLedger, benchmarkDigestAlias } from "./record-trial.mjs";

const execFileAsync = promisify(execFile);
const script = fileURLToPath(new URL("./record-trial.mjs", import.meta.url));
const hash = (value) => createHash("sha256").update(value).digest("hex");
const id = (value) => hash(value);

function registration() {
  const protocol = {
    schema_version: 1, phase: "fixture", campaign_id: "synthetic_recorder",
    analysis_seed: 7, bootstrap_resamples: 10000,
  };
  const assignments = ["R0", "RJ"].map((arm) => ({
    assignment_id: `assignment_${arm}`, case_id: "case_one", repository_group_id: "repo_one",
    snapshot_digest: id("snapshot"), packet_digest: id("packet"), family: "local_defects", split: "fixture",
    billing_mode: "api", billing_cohort_id: "pinned_api_tariff", arm, repetition: 1,
  }));
  const bindings = assignments.map((assignment, index) => ({
    assignment_id: assignment.assignment_id, task_contract_digest: id("contract"),
    scope: { domain_id: "C:\\private workspace\\repo", run_id: `run_${assignment.arm}` },
    task_id: `task_${assignment.arm}`, alias_key_hex: `${index + 1}`.repeat(64),
  }));
  const pins = {
    profile_pair_digest: id(JSON.stringify([
      id("everyday-profile"), id("everyday-binding"), id("strong-profile"), id("strong-binding"),
    ])),
    adapter_digest: id("qualified-adapter"),
    r0_policy_digest: id("frozen-rules-policy"),
    rj_policy_digest: id("frozen-jev-policy"),
  };
  return { protocol, assignments, pins, bindings };
}

function trace(assignment, binding) {
  const attemptDigest = id(`${assignment.arm}/attempt`);
  const observationDigest = id(`${assignment.arm}/observation`);
  const requestDigest = id(`${assignment.arm}/request`);
  const resultDigest = id(`${assignment.arm}/result`);
  const usageReceipt = id(`${assignment.arm}/usage`);
  const withAdvice = assignment.arm === "RJ";
  const decision = {
    selection: { kind: "selected", role: withAdvice ? "everyday" : "strong" },
    reason: withAdvice ? "advice_everyday" : "strong_default",
    advice_status: withAdvice ? "applied" : "not_used",
  };
  return {
    schema_version: 3,
    scope_digest: binding.scope_digest,
    task_digest: binding.task_digest,
    run_pins: structuredClone(binding.run_pins),
    routing_revision: 8,
    mode: withAdvice ? "live" : "rules",
    decision_links_complete: true,
    events: [
      { sequence: 1, kind: {
        kind: "decision_observed", observation_digest: observationDigest, ordinal: 1,
        decision, shadow_choice: null,
        advice_request_digest: withAdvice ? requestDigest : null,
        advice_digest: withAdvice ? resultDigest : null,
      } },
      { sequence: 2, kind: {
        kind: "decision_resolved", observation_digest: observationDigest,
        outcome: { kind: "admitted", attempt_digest: attemptDigest },
      } },
      { sequence: 3, kind: {
        kind: "admitted", attempt_digest: attemptDigest, observation_digest: observationDigest,
      } },
      { sequence: 4, kind: {
        kind: "transitioned", attempt_digest: attemptDigest, state: "passed",
      } },
      { sequence: 5, kind: {
        kind: "usage_settled", attempt_digest: attemptDigest,
        usage: { kind: "known", nano_usd: 100, receipt: usageReceipt },
      } },
    ],
    attempts: [{
      attempt_digest: attemptDigest, ordinal: 1, predecessor_digest: null,
      state: "passed", role: withAdvice ? "everyday" : "strong", billing_mode: "api", handoff_digest: null,
      receipt_aliases: {
        inputs: null, launch_checks: null, process_identity: null, no_worker_created: null,
        quiescence: id(`${assignment.arm}/quiescence`),
        sealed_output: id(`${assignment.arm}/sealed`), checks: id(`${assignment.arm}/checks`),
        reconciliation: null,
      },
      usage: { kind: "known", nano_usd: 100, receipt: usageReceipt }, ownership_released: true,
    }],
    advisor_requests: withAdvice ? [{
      request_digest: requestDigest, ordinal: 1, packet_digest: binding.packet_digest,
      phase: "completed", result_digest: resultDigest,
    }] : [],
  };
}

async function writeJson(directory, name, value) {
  await writeFile(path.join(directory, name), `${JSON.stringify(value, null, 2)}\n`);
  return name;
}

async function writeSource(directory, name, contents) {
  await writeFile(path.join(directory, name), contents);
  return { source: name, source_sha256: hash(contents) };
}

async function evidence(directory, study) {
  const rows = [];
  for (const assignment of study.assignments) {
    const binding = study.bindings.find((row) => row.assignment_id === assignment.assignment_id);
    const prefix = assignment.arm.toLowerCase();
    const base = {
      assignment_id: assignment.assignment_id,
      scope_digest: binding.scope_digest,
      task_digest: binding.task_digest,
    };
    const exported = trace(assignment, binding);
    const oracle = await writeSource(directory, `${prefix}-oracle.txt`, "independent oracle PASS\n");
    const review = await writeSource(directory, `${prefix}-review.txt`, "blinded reviewer ACCEPT\n");
    const provider = await writeSource(directory, `${prefix}-provider.txt`, "provider billed worker=100 advisor=0\n");
    const clock = await writeSource(directory, `${prefix}-clock.txt`, "monotonic 0..1000\n");
    const audit = await writeSource(directory, `${prefix}-audit.txt`, "controller ownership reconciled\n");
    const files = {
      assignment_id: assignment.assignment_id,
      trace: await writeJson(directory, `${prefix}-trace.json`, exported),
      acceptance: await writeJson(directory, `${prefix}-acceptance.json`, {
        ...base, task_contract_digest: binding.task_contract_digest,
        snapshot_digest: assignment.snapshot_digest, accepted: true,
        outcome_provenance: "independent_blinded", reviewer_id: `reviewer_${prefix}`,
        oracle_source: oracle.source, oracle_source_sha256: oracle.source_sha256,
        review_source: review.source, review_source_sha256: review.source_sha256,
      }),
      charges: await writeJson(directory, `${prefix}-charges.json`, {
        ...base, billing_mode: assignment.billing_mode, cost_nano_usd: 100,
        cost_components_nano_usd: {
          advisor: 0, worker: 100, checks: 0, handoff: 0, rework: 0, differing_infrastructure: 0,
        },
        usage_reconciled: true,
        provider_charge_provenance: { kind: "provider_invoice", ...provider },
        advisor_request_digests: exported.advisor_requests.map((row) => row.request_digest),
        attempt_usage_receipts: exported.attempts.map((row) => row.usage.receipt),
      }),
      timing: await writeJson(directory, `${prefix}-timing.json`, {
        ...base, clock_source: "monotonic", end_to_end_ms: 1000,
        foreground_routing_wait_ms: assignment.arm === "RJ" ? 100 : 0,
        human_intervention_ms: 0, ...clock,
      }),
      authority: await writeJson(directory, `${prefix}-authority.json`, {
        ...base, authority_violation: false, ownership_reconciled: true, ...audit,
      }),
    };
    rows.push(files);
  }
  return { study_digest: study.study_digest, rows };
}

test("freeze strips private task/run IDs and keys, then separate receipts assemble a conditional bundle", async () => {
  const input = registration();
  const study = freezeStudy(input);
  const serialized = JSON.stringify(study);
  const rjBinding = study.bindings.find((row) => row.assignment_id === "assignment_RJ");
  assert.equal(rjBinding.packet_digest,
    benchmarkDigestAlias(input.bindings[1].alias_key_hex, input.assignments[1].packet_digest));
  assert.ok(!serialized.includes(input.bindings[0].alias_key_hex));
  assert.ok(!serialized.includes("run_R0"));
  assert.ok(!serialized.includes("task_RJ"));
  assert.ok(!serialized.includes("private workspace"));
  const directory = await mkdtemp(path.join(tmpdir(), "pytxo-routing-record-"));
  const index = await evidence(directory, study);
  const { bundle, review } = await assembleStudy(study, index, directory);
  assert.equal(bundle.observations.length, 2);
  assert.equal(bundle.observations[1].cost_evidence, "provider_billed");
  assert.deepEqual(bundle.observations[1].routing_exposure, {
    advisor_may_send: 1, completed_advisor_sends: 1,
    initial_admission: { reason: "advice_everyday", role: "everyday", advice_status: "applied" },
  });
  assert.equal(review.receipt_hashes.length, 2);
  assert.equal(review.source_authentication, "external_review_required");
  const report = analyzeRoutingTrial(bundle);
  assert.equal(report.assigned_runs, 2);
  assert.equal(report.production_routing_authorized, false);
  assert.equal(report.statistical_gates_passed, false);
});

test("real Store-exported rules trace crosses into the recorder without implying a Jev trial", {
  skip: process.env.PYTXO_ROUTING_EXPORT_TEST_BINARY ? false
    : "set PYTXO_ROUTING_EXPORT_TEST_BINARY to a compiled exporter Rust test binary",
}, async () => {
  // The Rust test creates a genuine Store mission and exports its blocked R0
  // decision through run(). Every external receipt below is synthetic test data.
  const directory = await mkdtemp(path.join(tmpdir(), "pytxo-routing-export-node-"));
  try {
    await execFileAsync(process.env.PYTXO_ROUTING_EXPORT_TEST_BINARY, [
      "--exact", "tests::exports_a_real_store_trace_for_the_frozen_rules_assignment",
    ], {
      cwd: path.resolve(path.dirname(script), "../../.."),
      env: { ...process.env, PYTXO_ROUTING_EXPORT_FIXTURE_DIR: directory },
    });
    const registrationPath = path.join(directory, "registration.json");
    let registrationBytes;
    try {
      registrationBytes = await readFile(registrationPath, "utf8");
    } catch (error) {
      if (error.code !== "ENOENT") throw error;
      throw new Error("exporter test binary did not retain its Store fixture; rebuild the binary from current source with PYTXO_ROUTING_EXPORT_FIXTURE_DIR support", { cause: error });
    }
    const rustRegistration = JSON.parse(registrationBytes);
    const protocol = {
      schema_version: 1, phase: "fixture", campaign_id: "store_export_test_only",
      analysis_seed: 7, bootstrap_resamples: 10000,
    };
    const assignments = rustRegistration.assignments.map((assignment) => ({
      ...assignment, case_id: "case_one", repository_group_id: "local_test_repo",
      family: "mechanical", split: "fixture", billing_mode: "api",
      billing_cohort_id: "synthetic_no_provider", repetition: 1,
    }));
    // Freezing here validates the cross-language alias and pin scheme. This is
    // deliberately not chronological preregistration: the Store fixture exists.
    const study = freezeStudy({ ...rustRegistration, protocol, assignments });
    const realTrace = JSON.parse(await readFile(path.join(directory, "trace.json"), "utf8"));
    assert.equal(realTrace.mode, "rules");
    assert.equal(realTrace.decision_links_complete, true);
    assert.equal(realTrace.events[0]?.kind.kind, "blocked");
    assert.equal(realTrace.attempts.length, 0);
    assert.equal(realTrace.advisor_requests.length, 0);
    const rawDomain = JSON.stringify(rustRegistration.bindings[0].scope.domain_id).slice(1, -1);
    const rawRun = rustRegistration.bindings[0].scope.run_id;
    const rawKey = rustRegistration.bindings[0].alias_key_hex;
    for (const publicArtifact of [study, realTrace]) {
      const serialized = JSON.stringify(publicArtifact);
      assert.ok(!serialized.includes(rawDomain));
      assert.ok(!serialized.includes(JSON.stringify(rawRun)));
      assert.ok(!serialized.includes(rawKey));
    }

    const rows = [];
    for (const assignment of study.assignments) {
      const binding = study.bindings.find((entry) => entry.assignment_id === assignment.assignment_id);
      const prefix = assignment.arm.toLowerCase();
      const selectedTrace = assignment.arm === "R0"
        ? realTrace : trace(assignment, binding); // RJ is synthetic; no Jev call exists.
      const base = {
        assignment_id: assignment.assignment_id,
        scope_digest: binding.scope_digest,
        task_digest: binding.task_digest,
      };
      const oracle = await writeSource(directory, `${prefix}-fixture-oracle.txt`,
        "SYNTHETIC TEST ONLY: no independent oracle ran\n");
      const reviewer = await writeSource(directory, `${prefix}-fixture-review.txt`,
        "SYNTHETIC TEST ONLY: no blinded review ran\n");
      const clock = await writeSource(directory, `${prefix}-fixture-clock.txt`,
        "SYNTHETIC TEST ONLY: no monotonic timing measured\n");
      const audit = await writeSource(directory, `${prefix}-fixture-audit.txt`,
        "SYNTHETIC TEST ONLY: no external authority audit ran\n");
      rows.push({
        assignment_id: assignment.assignment_id,
        trace: assignment.arm === "R0" ? "trace.json"
          : await writeJson(directory, `${prefix}-fixture-trace.json`, selectedTrace),
        acceptance: await writeJson(directory, `${prefix}-fixture-acceptance.json`, {
          ...base, task_contract_digest: binding.task_contract_digest,
          snapshot_digest: assignment.snapshot_digest, accepted: false,
          // This shape exercises the recorder contract; the source makes clear
          // that the fixture is not a genuine independent blinded assessment.
          outcome_provenance: "independent_blinded", reviewer_id: "synthetic_fixture",
          oracle_source: oracle.source, oracle_source_sha256: oracle.source_sha256,
          review_source: reviewer.source, review_source_sha256: reviewer.source_sha256,
        }),
        charges: await writeJson(directory, `${prefix}-fixture-charges.json`, {
          ...base, billing_mode: assignment.billing_mode, cost_nano_usd: null,
          cost_components_nano_usd: {
            advisor: null, worker: null, checks: null, handoff: null,
            rework: null, differing_infrastructure: null,
          },
          usage_reconciled: false,
          provider_charge_provenance: { kind: "unavailable", source: null, source_sha256: null },
          advisor_request_digests: selectedTrace.advisor_requests.map((request) => request.request_digest),
          attempt_usage_receipts: selectedTrace.attempts
            .filter((attempt) => ["known", "estimated"].includes(attempt.usage.kind))
            .map((attempt) => attempt.usage.receipt),
        }),
        timing: await writeJson(directory, `${prefix}-fixture-timing.json`, {
          ...base, clock_source: "monotonic", end_to_end_ms: 1,
          foreground_routing_wait_ms: 0, human_intervention_ms: 0, ...clock,
        }),
        authority: await writeJson(directory, `${prefix}-fixture-authority.json`, {
          ...base, authority_violation: false, ownership_reconciled: true, ...audit,
        }),
      });
    }
    const index = { study_digest: study.study_digest, rows };
    const { bundle, review } = await assembleStudy(study, index, directory);
    assert.equal(bundle.observations.length, 2);
    assert.ok(bundle.observations.every((row) => row.cost_nano_usd === null));
    assert.ok(bundle.observations.every((row) => row.cost_evidence === "unknown"));
    assert.equal(review.receipt_hashes[0].trace_sha256,
      hash(await readFile(path.join(directory, "trace.json"))));
    assert.equal(review.source_authentication, "external_review_required");
    const report = analyzeRoutingTrial(bundle);
    assert.equal(report.production_routing_authorized, false);
    assert.equal(report.statistical_gates_passed, false);

    const changed = structuredClone(realTrace);
    changed.run_pins.snapshot_digest = id("tampered-real-store-snapshot");
    const altered = await writeJson(directory, "tampered-trace.json", changed);
    const alteredIndex = {
      ...index, rows: index.rows.map((row) => row.assignment_id === "frozen_r0"
        ? { ...row, trace: altered } : row),
    };
    await assert.rejects(assembleStudy(study, alteredIndex, directory), /trace snapshot_digest differs/);
  } finally {
    const root = await realpath(directory);
    const temp = await realpath(tmpdir());
    assert.ok(root.startsWith(`${temp}${path.sep}`));
    await rm(root, { recursive: true, force: true });
  }
});

test("real Store advisor journal packet alias matches its frozen assignment", {
  skip: process.env.PYTXO_ROUTING_STORE_TEST_BINARY ? false
    : "set PYTXO_ROUTING_STORE_TEST_BINARY to a compiled routing_store Rust test binary",
}, async () => {
  // The Rust test records synthetic advice through the real Store journal and
  // exports its alias projection. No Jev request or worker execution occurs.
  const directory = await mkdtemp(path.join(tmpdir(), "pytxo-routing-advisor-node-"));
  try {
    await execFileAsync(process.env.PYTXO_ROUTING_STORE_TEST_BINARY, [
      "--exact", "live_jev_applied_observation_requires_journal_and_admission_rechecks_consent",
    ], {
      cwd: path.resolve(path.dirname(script), "../../.."),
      env: { ...process.env, PYTXO_ROUTING_ADVISOR_BRIDGE_DIR: directory },
    });
    const exported = JSON.parse(await readFile(path.join(directory, "advisor-trace.json"), "utf8"));
    const privatePacket = JSON.parse(await readFile(path.join(directory, "private-packet.json"), "utf8"));
    assert.equal(exported.mode, "live");
    assert.equal(exported.advisor_requests.length, 1);
    assert.equal(exported.advisor_requests[0].phase, "completed");
    const input = registration();
    for (const assignment of input.assignments) assignment.packet_digest = privatePacket.packet_digest;
    input.bindings[1].alias_key_hex = privatePacket.alias_key_hex;
    const study = freezeStudy(input);
    const binding = study.bindings.find((row) => row.assignment_id === "assignment_RJ");
    assert.equal(exported.advisor_requests[0].packet_digest, binding.packet_digest);
    assert.notEqual(exported.advisor_requests[0].packet_digest, privatePacket.packet_digest);
    assert.notEqual(benchmarkDigestAlias(privatePacket.alias_key_hex, id("wrong packet")),
      binding.packet_digest);
  } finally {
    const root = await realpath(directory);
    const temp = await realpath(tmpdir());
    assert.ok(root.startsWith(`${temp}${path.sep}`));
    await rm(root, { recursive: true, force: true });
  }
});

test("Store digest alias agrees with an independent SHA-256 hex fixture", () => {
  // Independently calculated with hashlib over prefix, 32 key bytes, LE kind length, kind and raw hex.
  assert.equal(benchmarkDigestAlias("0f".repeat(32), "a".repeat(64)),
    "4816641ef23a6f45a21d92084ea7c8da32f0e01b1193ef77ecfd27b0be78a3db");
});

test("fixture pins are required and freeze derives the assigned arm policy pin", () => {
  const input = registration();
  const study = freezeStudy(input);
  for (const assignment of input.assignments) {
    const privateBinding = input.bindings.find((row) => row.assignment_id === assignment.assignment_id);
    const frozenBinding = study.bindings.find((row) => row.assignment_id === assignment.assignment_id);
    const policy = assignment.arm === "R0" ? input.pins.r0_policy_digest : input.pins.rj_policy_digest;
    assert.equal(frozenBinding.run_pins.task_contract_digest,
      benchmarkDigestAlias(privateBinding.alias_key_hex, privateBinding.task_contract_digest));
    assert.equal(frozenBinding.run_pins.snapshot_digest,
      benchmarkDigestAlias(privateBinding.alias_key_hex, assignment.snapshot_digest));
    assert.equal(frozenBinding.run_pins.profile_pair_digest,
      benchmarkDigestAlias(privateBinding.alias_key_hex, input.pins.profile_pair_digest));
    assert.equal(frozenBinding.run_pins.policy_digest,
      benchmarkDigestAlias(privateBinding.alias_key_hex, policy));
    assert.equal(frozenBinding.run_pins.adapter_digest,
      benchmarkDigestAlias(privateBinding.alias_key_hex, input.pins.adapter_digest));
  }
  const missing = structuredClone(input);
  delete missing.pins;
  assert.throws(() => freezeStudy(missing), /study pins must be an object/);
});

test("v3 trace refuses each missing or changed mission pin", async () => {
  const study = freezeStudy(registration());
  const directory = await mkdtemp(path.join(tmpdir(), "pytxo-routing-pins-"));
  const index = await evidence(directory, study);
  const original = JSON.parse(await readFile(path.join(directory, "rj-trace.json"), "utf8"));
  for (const pin of ["task_contract_digest", "snapshot_digest", "profile_pair_digest", "policy_digest", "adapter_digest"]) {
    const changed = structuredClone(original);
    changed.run_pins[pin] = id(`changed_${pin}`);
    await writeJson(directory, "rj-trace.json", changed);
    await assert.rejects(assembleStudy(study, index, directory), new RegExp(`trace ${pin} differs`));
    const missing = structuredClone(original);
    delete missing.run_pins[pin];
    await writeJson(directory, "rj-trace.json", missing);
    await assert.rejects(assembleStudy(study, index, directory), new RegExp(`trace ${pin}.*SHA-256`));
  }
  const old = structuredClone(original);
  old.schema_version = 2;
  await writeJson(directory, "rj-trace.json", old);
  await assert.rejects(assembleStudy(study, index, directory), /v3 trace identity/);
});

test("assembly rejects missing, duplicate, wrong-arm and wrong-task evidence", async () => {
  const study = freezeStudy(registration());
  const directory = await mkdtemp(path.join(tmpdir(), "pytxo-routing-record-"));
  const index = await evidence(directory, study);
  await assert.rejects(assembleStudy(study, { ...index, rows: index.rows.slice(1) }, directory), /one evidence row/);
  await assert.rejects(assembleStudy(study, { ...index, rows: [index.rows[0], index.rows[0]] }, directory), /duplicate evidence/);
  const rjTrace = path.join(directory, "rj-trace.json");
  const original = JSON.parse(await readFile(rjTrace, "utf8"));
  await writeJson(directory, "rj-trace.json", { ...original, mode: "shadow" });
  await assert.rejects(assembleStudy(study, index, directory), /preassigned arm/);
  await writeJson(directory, "rj-trace.json", { ...original, task_digest: id("wrong-task") });
  await assert.rejects(assembleStudy(study, index, directory), /frozen binding/);
  const wrongPacket = structuredClone(original);
  wrongPacket.advisor_requests[0].packet_digest = id("wrong packet");
  await writeJson(directory, "rj-trace.json", wrongPacket);
  await assert.rejects(assembleStudy(study, index, directory), /advisor packet differs from frozen case/);
});

test("one completed advisor result is bound to one observation at its request ordinal", async () => {
  const study = freezeStudy(registration());
  const directory = await mkdtemp(path.join(tmpdir(), "pytxo-routing-record-"));
  const index = await evidence(directory, study);
  const original = JSON.parse(await readFile(path.join(directory, "rj-trace.json"), "utf8"));
  const wrongOrdinal = structuredClone(original);
  wrongOrdinal.advisor_requests[0].ordinal = 2;
  await writeJson(directory, "rj-trace.json", wrongOrdinal);
  await assert.rejects(assembleStudy(study, index, directory), /request ordinal differs/);
  const reused = structuredClone(original);
  reused.events.push({
    sequence: 6,
    kind: { ...structuredClone(reused.events[0].kind), observation_digest: id("second_observation") },
  });
  reused.events.push({
    sequence: 7,
    kind: { kind: "decision_resolved", observation_digest: id("second_observation"),
      outcome: { kind: "not_admitted", stage: "capacity_unavailable" } },
  });
  await writeJson(directory, "rj-trace.json", reused);
  await assert.rejects(assembleStudy(study, index, directory), /result reused for distinct observations/);
});

test("an applied route requires a completed bound result and a first everyday admission", async () => {
  const study = freezeStudy(registration());
  const directory = await mkdtemp(path.join(tmpdir(), "pytxo-routing-applied-"));
  const index = await evidence(directory, study);
  const original = JSON.parse(await readFile(path.join(directory, "rj-trace.json"), "utf8"));
  const mutations = [
    ["missing result", (trace) => { trace.events[0].kind.advice_digest = null; }, /applied.*completed.*result/],
    ["wrong role", (trace) => { trace.events[0].kind.decision.selection.role = "strong"; }, /applied.*everyday/],
    ["wrong reason", (trace) => { trace.events[0].kind.decision.reason = "strong_default"; }, /applied.*advice_everyday/],
    ["later ordinal", (trace) => { trace.events[0].kind.ordinal = 2; trace.advisor_requests[0].ordinal = 2; }, /applied.*first/],
    ["false not-used", (trace) => { trace.events[0].kind.decision.advice_status = "not_used"; }, /not_used.*result/],
    ["shadow choice in live arm", (trace) => { trace.events[0].kind.shadow_choice = "everyday_fit"; }, /shadow-only advice/],
    ["blocked applied", (trace) => {
      trace.events[0].kind.kind = "blocked";
      trace.events[0].kind.decision.selection = { kind: "blocked", code: "disabled" };
    }, /blocked decision differs/],
    ["unresolved admission", (trace) => {
      trace.events[1].kind.outcome = { kind: "not_admitted", stage: "capacity_unavailable" };
    }, /admitted event lacks matching resolved admission/],
  ];
  for (const [name, mutate, expected] of mutations) {
    const changed = structuredClone(original);
    mutate(changed);
    await writeJson(directory, "rj-trace.json", changed);
    await assert.rejects(assembleStudy(study, index, directory), expected, name);
  }
});

test("assembly rejects missing source, changed source, duplicate sends and non-independent acceptance", async () => {
  const study = freezeStudy(registration());
  const directory = await mkdtemp(path.join(tmpdir(), "pytxo-routing-record-"));
  const index = await evidence(directory, study);
  await writeFile(path.join(directory, "rj-provider.txt"), "provider value changed\n");
  await assert.rejects(assembleStudy(study, index, directory), /provider charge source digest mismatch/);
  await writeFile(path.join(directory, "rj-provider.txt"), "provider billed worker=100 advisor=0\n");
  const charges = JSON.parse(await readFile(path.join(directory, "rj-charges.json"), "utf8"));
  charges.advisor_request_digests = [];
  await writeJson(directory, "rj-charges.json", charges);
  await assert.rejects(assembleStudy(study, index, directory), /advisor charge coverage/);
  charges.advisor_request_digests = [id("RJ/request")];
  await writeJson(directory, "rj-charges.json", charges);
  const acceptance = JSON.parse(await readFile(path.join(directory, "rj-acceptance.json"), "utf8"));
  acceptance.outcome_provenance = "worker_claim";
  await writeJson(directory, "rj-acceptance.json", acceptance);
  await assert.rejects(assembleStudy(study, index, directory), /independently bound/);
  acceptance.outcome_provenance = "independent_blinded";
  acceptance.oracle_source = "absent.txt";
  await writeJson(directory, "rj-acceptance.json", acceptance);
  await assert.rejects(assembleStudy(study, index, directory), /ENOENT/);
});

test("unknown provider charge remains unknown in the analysis input", async () => {
  const study = freezeStudy(registration());
  const directory = await mkdtemp(path.join(tmpdir(), "pytxo-routing-record-"));
  const index = await evidence(directory, study);
  const charges = JSON.parse(await readFile(path.join(directory, "rj-charges.json"), "utf8"));
  charges.cost_nano_usd = null;
  charges.cost_components_nano_usd.worker = null;
  charges.usage_reconciled = false;
  charges.provider_charge_provenance = { kind: "unavailable", source: null, source_sha256: null };
  await writeJson(directory, "rj-charges.json", charges);
  const { bundle, review } = await assembleStudy(study, index, directory);
  const rj = bundle.observations.find((row) => row.assignment_id === "assignment_RJ");
  assert.equal(rj.cost_nano_usd, null);
  assert.equal(rj.cost_evidence, "unknown");
  assert.equal(review.receipt_hashes[1].provider_source_sha256, null);
  const report = analyzeRoutingTrial(bundle);
  assert.equal(report.point.cost_ratio, null);
  assert.equal(report.conditional_thresholds.cost, false);
});

test("CLI freezes and assembles write-once artifacts without touching existing reports", async () => {
  const directory = await mkdtemp(path.join(tmpdir(), "pytxo-routing-cli-"));
  const registrationFile = await writeJson(directory, "registration.json", registration());
  const studyPath = path.join(directory, "study.json");
  await execFileAsync(process.execPath, [script, "--freeze", path.join(directory, registrationFile), studyPath]);
  await assert.rejects(execFileAsync(process.execPath, [script, "--freeze", path.join(directory, registrationFile), studyPath]), /EEXIST/);
  const study = JSON.parse(await readFile(studyPath, "utf8"));
  const indexFile = await writeJson(directory, "index.json", await evidence(directory, study));
  const bundlePath = path.join(directory, "bundle.json");
  const reviewPath = path.join(directory, "review.json");
  await execFileAsync(process.execPath, [script, "--assemble", studyPath, path.join(directory, indexFile), bundlePath, reviewPath]);
  assert.equal(JSON.parse(await readFile(bundlePath, "utf8")).observations.length, 2);
  assert.equal(JSON.parse(await readFile(reviewPath, "utf8")).analysis_bundle_sha256, hash(await readFile(bundlePath)));
  await assert.rejects(execFileAsync(process.execPath, [script, "--assemble", studyPath, path.join(directory, indexFile), bundlePath, reviewPath]), /output already exists/);
});

test("freeze rejects rebinding paired contracts, repeated alias keys and post-freeze edits", async () => {
  const input = registration();
  input.bindings[1].task_contract_digest = id("different-contract");
  assert.throws(() => freezeStudy(input), /task contract differs/);
  input.bindings[1].task_contract_digest = input.bindings[0].task_contract_digest;
  input.bindings[1].alias_key_hex = input.bindings[0].alias_key_hex;
  assert.throws(() => freezeStudy(input), /distinct nonzero random/);
  input.bindings[1].alias_key_hex = id("fresh_key");
  input.bindings[1].scope = structuredClone(input.bindings[0].scope);
  assert.throws(() => freezeStudy(input), /distinct run scopes/);
  const study = freezeStudy(registration());
  const directory = await mkdtemp(path.join(tmpdir(), "pytxo-routing-record-"));
  const index = await evidence(directory, study);
  study.bindings[0].task_digest = id("changed-after-freeze");
  await assert.rejects(assembleStudy(study, index, directory), /study digest mismatch/);
});

function confirmationRegistration() {
  const families = ["mechanical", "local_defects", "state_persistence", "concurrency", "adapter_protocol", "cross_component"];
  const limits = [67, 134, 201, 268, 334, 400];
  const cases = Array.from({ length: 400 }, (_, index) => ({
    case_id: `case_${index}`, repository_group_id: `repo_${Math.floor(index / 2)}`,
    snapshot_digest: id(`snapshot_${index}`), packet_digest: id(`packet_${index % 12}`), task_contract_digest: id(`contract_${index}`),
    oracle_digest: id(`oracle_${index}`), family: families[limits.findIndex((limit) => index < limit)],
    planned_advice_opportunity: index >= 67 && index < 134,
    split: "holdout", billing_mode: "api", billing_cohort_id: "one_tariff",
  }));
  const campaign = {
    protocol: { schema_version: 1, phase: "confirmation", campaign_id: "synthetic_confirmation", analysis_seed: 4, bootstrap_resamples: 10000 },
    randomization_seed: 5,
    pins: {
      profile_pair_digest: id("profiles"), adapter_digest: id("adapter"),
      r0_policy_digest: id("rules"), rj_policy_digest: id("jev"),
      oracle_manifest_digest: id("oracles"),
    },
    cases,
  };
  const frozen = freezeConfirmationCampaign(campaign);
  const caseById = new Map(cases.map((row) => [row.case_id, row]));
  const input = {
    frozen_confirmation: frozen,
    registered_freeze_digest: frozen.freeze_digest,
    private_cases: cases,
    bindings: frozen.assignments.map((assignment, index) => ({
      assignment_id: assignment.assignment_id,
      task_contract_digest: caseById.get(assignment.case_id).task_contract_digest,
      scope: { domain_id: "domain_trial", run_id: `run_${index}` },
      task_id: `task_${index}`, alias_key_hex: id(`synthetic_key_${index}`),
    })),
  };
  return input;
}

function controllerOrderSource(study) {
  const byId = new Map(study.bindings.map((row) => [row.assignment_id, row]));
  return {
    schema_version: 1, study_digest: study.study_digest, controller_id: "synthetic_controller",
    events: study.frozen_confirmation.schedule.flatMap((block) => block.assignment_ids)
      .map((assignmentId, index) => ({
        sequence: index + 1, kind: "assignment_started", assignment_id: assignmentId,
        scope_digest: byId.get(assignmentId).scope_digest,
        task_digest: byId.get(assignmentId).task_digest,
        controller_receipt_digest: id(`controller-start:${index}:${assignmentId}`),
      })),
  };
}

test("confirmation registration replays the private case manifest and freezes all 1,600 task bindings", () => {
  const input = confirmationRegistration();
  const study = freezeStudy(input);
  assert.equal(study.bindings.length, 1600);
  assert.equal(study.registered_freeze_digest, input.frozen_confirmation.freeze_digest);
  assert.ok(!JSON.stringify(study).includes("synthetic_key_0"));
  const assignment = study.assignments[0];
  const privateBinding = input.bindings.find((row) => row.assignment_id === assignment.assignment_id);
  const frozenBinding = study.bindings[0];
  const rawPolicy = assignment.arm === "R0"
    ? input.frozen_confirmation.pins.r0_policy_digest
    : input.frozen_confirmation.pins.rj_policy_digest;
  assert.equal(frozenBinding.run_pins.policy_digest,
    benchmarkDigestAlias(privateBinding.alias_key_hex, rawPolicy));
  const altered = structuredClone(input);
  altered.private_cases[0].task_contract_digest = id("changed-after-case-freeze");
  assert.throws(() => freezeStudy(altered), /private cases differ/);
});

test("confirmation controller ledger binds the exact frozen crossover and writes once", async () => {
  const study = freezeStudy(confirmationRegistration());
  const directory = await mkdtemp(path.join(tmpdir(), "pytxo-routing-order-"));
  const studyFile = await writeJson(directory, "study.json", study);
  const source = controllerOrderSource(study);
  const sourceFile = await writeJson(directory, "controller-source.json", source);
  const sourceBytes = await readFile(path.join(directory, sourceFile));
  const expected = createExecutionOrderLedger(study, sourceBytes);
  assert.equal(expected.assignment_ids.length, 1600);
  assert.equal(expected.source_sha256, hash(sourceBytes));
  const ledgerPath = path.join(directory, "controller-ledger.json");
  await execFileAsync(process.execPath, [
    script, "--record-order", path.join(directory, studyFile), path.join(directory, sourceFile), ledgerPath,
  ]);
  assert.deepEqual(JSON.parse(await readFile(ledgerPath, "utf8")), expected);
  await assert.rejects(execFileAsync(process.execPath, [
    script, "--record-order", path.join(directory, studyFile), path.join(directory, sourceFile), ledgerPath,
  ]), /EEXIST/);
  const rows = study.assignments.map((assignment) => ({
    assignment_id: assignment.assignment_id, trace: "missing-trace.json",
    acceptance: "missing-acceptance.json", charges: "missing-charges.json",
    timing: "missing-timing.json", authority: "missing-authority.json",
  }));
  await assert.rejects(assembleStudy(study, { study_digest: study.study_digest, rows }, directory),
    /confirmation requires a controller execution-order/);
  await assert.rejects(assembleStudy(study, {
    study_digest: study.study_digest, rows,
    execution_order_source: sourceFile, execution_order_ledger: path.basename(ledgerPath),
  }, directory), /ENOENT/); // The order gate passed; the deliberately absent task files are next.
  const reordered = structuredClone(source);
  const byArm = new Map(study.assignments.map((assignment) => [assignment.assignment_id, assignment.arm]));
  reordered.events.sort((a, b) => byArm.get(a.assignment_id).localeCompare(byArm.get(b.assignment_id)));
  reordered.events.forEach((event, index) => { event.sequence = index + 1; });
  assert.throws(() => createExecutionOrderLedger(study, Buffer.from(JSON.stringify(reordered))),
    /differs from frozen crossover schedule/);
  const altered = structuredClone(source);
  altered.controller_id = "another_controller";
  await writeJson(directory, "controller-source.json", altered);
  await assert.rejects(assembleStudy(study, {
    study_digest: study.study_digest, rows,
    execution_order_source: sourceFile, execution_order_ledger: path.basename(ledgerPath),
  }, directory), /execution-order ledger differs from source/);
});
