import assert from "node:assert/strict";
import { createHash } from "node:crypto";
import { execFile } from "node:child_process";
import { mkdtemp, readFile, writeFile } from "node:fs/promises";
import { tmpdir } from "node:os";
import path from "node:path";
import test from "node:test";
import { fileURLToPath } from "node:url";
import { promisify } from "node:util";

import { analyzeRoutingTrial, manifestDigest } from "./analyze-trial.mjs";
import { freezeConfirmationCampaign, verifyFrozenConfirmation } from "./freeze-confirmation.mjs";

const execFileAsync = promisify(execFile);
const scriptPath = fileURLToPath(new URL("./freeze-confirmation.mjs", import.meta.url));
const families = [
  "mechanical", "local_defects", "state_persistence", "concurrency",
  "adapter_protocol", "cross_component",
];
const limits = [67, 134, 201, 268, 334, 400];
const sha = (value) => createHash("sha256").update(value).digest("hex");

function fixture() {
  const cases = Array.from({ length: 400 }, (_, index) => ({
    case_id: `case_${String(index).padStart(3, "0")}`,
    repository_group_id: `repo_${String(Math.floor(index / 2)).padStart(3, "0")}`,
    snapshot_digest: sha(`snapshot:${index}`),
    packet_digest: sha(`packet:${index % 12}`),
    task_contract_digest: sha(`task:${index}`),
    oracle_digest: sha(`oracle:${index}`),
    family: families[limits.findIndex((limit) => index < limit)],
    planned_advice_opportunity: index >= 67 && index < 134,
    split: "holdout",
    billing_mode: "api",
    billing_cohort_id: "one_pinned_tariff",
  }));
  return {
    protocol: {
      schema_version: 1,
      phase: "confirmation",
      campaign_id: "confirmation_2026",
      analysis_seed: 16531,
      bootstrap_resamples: 10000,
    },
    randomization_seed: 28491,
    pins: {
      profile_pair_digest: sha("profiles"),
      adapter_digest: sha("adapter"),
      r0_policy_digest: sha("rules"),
      rj_policy_digest: sha("jev-policy"),
      oracle_manifest_digest: sha("oracles"),
    },
    cases,
  };
}

test("frozen confirmation is reproducible, paired, and balanced by repository", () => {
  const input = fixture();
  const frozen = freezeConfirmationCampaign(input);
  assert.deepEqual(frozen, freezeConfirmationCampaign({ ...input, cases: [...input.cases].reverse() }));
  assert.equal(frozen.assignments.length, 1600);
  assert.equal(frozen.schema_version, 2);
  assert.equal(frozen.schedule.length, 400);
  assert.equal(frozen.manifest_digest, manifestDigest(input.protocol, frozen.assignments));
  assert.match(frozen.freeze_digest, /^[a-f0-9]{64}$/);
  assert.equal(verifyFrozenConfirmation(frozen), true);
  assert.equal(new Set(frozen.schedule.flatMap((block) => block.assignment_ids)).size, 1600);
  assert.deepEqual(Object.keys(frozen).sort(), [
    "assignments", "case_manifest_digest", "freeze_digest", "manifest_digest", "pins", "protocol", "randomization_seed", "schedule", "schema_version",
  ].sort());
  assert.ok(!JSON.stringify(frozen).includes("oracle_digest"));
  for (const repository of new Set(input.cases.map((row) => row.repository_group_id))) {
    const blocks = frozen.schedule.filter((block) => block.repository_group_id === repository);
    assert.equal(blocks.length, 2);
    const firstArms = blocks.map((block) => frozen.assignments.find((row) => row.assignment_id === block.assignment_ids[0]).arm);
    assert.deepEqual(new Set(firstArms), new Set(["R0", "RJ"]));
    for (const block of blocks) {
      const sequence = block.assignment_ids.map((id) => {
        const row = frozen.assignments.find((assignment) => assignment.assignment_id === id);
        return `${row.arm}-${row.repetition}`;
      });
      assert.ok([
        "R0-1,RJ-1,RJ-2,R0-2", "RJ-1,R0-1,R0-2,RJ-2",
      ].includes(sequence.join(",")));
    }
  }
});

test("case or policy identity drift changes the frozen digest", () => {
  const input = fixture();
  const original = freezeConfirmationCampaign(input);
  const changedCase = structuredClone(input);
  changedCase.cases[0].oracle_digest = sha("different oracle");
  assert.notEqual(freezeConfirmationCampaign(changedCase).freeze_digest, original.freeze_digest);
  const changedPacket = structuredClone(input);
  changedPacket.cases[0].packet_digest = sha("different reviewed packet");
  assert.notEqual(freezeConfirmationCampaign(changedPacket).freeze_digest, original.freeze_digest);
  const changedOpportunity = structuredClone(input);
  changedOpportunity.cases[67].planned_advice_opportunity = false;
  assert.notEqual(freezeConfirmationCampaign(changedOpportunity).freeze_digest, original.freeze_digest);
  const changedPolicy = structuredClone(input);
  changedPolicy.pins.rj_policy_digest = sha("different policy");
  assert.notEqual(freezeConfirmationCampaign(changedPolicy).freeze_digest, original.freeze_digest);
  const changedOrder = structuredClone(input);
  changedOrder.randomization_seed++;
  assert.notEqual(freezeConfirmationCampaign(changedOrder).freeze_digest, original.freeze_digest);
});

test("incomplete, contaminated, or incomparable campaigns fail before freezing", () => {
  const input = fixture();
  assert.throws(() => freezeConfirmationCampaign({ ...input, cases: input.cases.slice(1) }), /400 cases/);
  const duplicate = structuredClone(input);
  duplicate.cases[1].case_id = duplicate.cases[0].case_id;
  assert.throws(() => freezeConfirmationCampaign(duplicate), /duplicate case/);
  const mixedBilling = structuredClone(input);
  mixedBilling.cases[1].billing_mode = "subscription";
  assert.throws(() => freezeConfirmationCampaign(mixedBilling), /API billing cohort/);
  const leakedPrompt = structuredClone(input);
  leakedPrompt.cases[0].goal = "private task text";
  assert.throws(() => freezeConfirmationCampaign(leakedPrompt), /unexpected field: goal/);
  const leakedSecret = structuredClone(input);
  leakedSecret.pins.api_key = "secret";
  assert.throws(() => freezeConfirmationCampaign(leakedSecret), /unexpected field: api_key/);
  const familyDrift = structuredClone(input);
  familyDrift.cases[0].family = "cross_component";
  assert.throws(() => freezeConfirmationCampaign(familyDrift), /family balance/);
  const duplicateTask = structuredClone(input);
  duplicateTask.cases[1].task_contract_digest = duplicateTask.cases[0].task_contract_digest;
  assert.throws(() => freezeConfirmationCampaign(duplicateTask), /duplicate task contract/);
  const identicalPackets = structuredClone(input);
  for (const row of identicalPackets.cases) row.packet_digest = sha("identical packet");
  assert.throws(() => freezeConfirmationCampaign(identicalPackets), /packet diversity/);
  const missingOpportunity = structuredClone(input);
  delete missingOpportunity.cases[67].planned_advice_opportunity;
  assert.throws(() => freezeConfirmationCampaign(missingOpportunity), /planned_advice_opportunity/);
  const invalidOpportunity = structuredClone(input);
  invalidOpportunity.cases[67].planned_advice_opportunity = "yes";
  assert.throws(() => freezeConfirmationCampaign(invalidOpportunity), /planned_advice_opportunity/);
  const uniformOpportunity = structuredClone(input);
  for (const row of uniformOpportunity.cases) {
    if (row.planned_advice_opportunity) row.packet_digest = sha("same opportunity packet");
  }
  assert.throws(() => freezeConfirmationCampaign(uniformOpportunity), /opportunity packet diversity/);
  const reusedRepository = structuredClone(input);
  reusedRepository.cases[2].snapshot_digest = reusedRepository.cases[0].snapshot_digest;
  assert.throws(() => freezeConfirmationCampaign(reusedRepository), /snapshot reused across repositories/);
});

test("verification detects changed assignment or randomized order", () => {
  const frozen = freezeConfirmationCampaign(fixture());
  const changedAssignment = structuredClone(frozen);
  changedAssignment.assignments[0].arm = changedAssignment.assignments[0].arm === "R0" ? "RJ" : "R0";
  assert.throws(() => verifyFrozenConfirmation(changedAssignment), /assignment identity|manifest digest|unpaired/);
  const changedOrder = structuredClone(frozen);
  [changedOrder.schedule[0], changedOrder.schedule[1]] = [changedOrder.schedule[1], changedOrder.schedule[0]];
  assert.throws(() => verifyFrozenConfirmation(changedOrder), /schedule|freeze digest/);
  const leakedField = structuredClone(frozen);
  leakedField.goal = "private task";
  assert.throws(() => verifyFrozenConfirmation(leakedField), /unexpected field: goal/);
  const changedPin = structuredClone(frozen);
  changedPin.pins.adapter_digest = sha("different adapter");
  assert.throws(() => verifyFrozenConfirmation(changedPin), /freeze digest/);
});

test("confirmation analysis is bound to the registered freeze and every observation", () => {
  const frozen = freezeConfirmationCampaign(fixture());
  const observation = {
    assignment_id: frozen.assignments[0].assignment_id,
    manifest_digest: frozen.manifest_digest,
    freeze_digest: frozen.freeze_digest,
    status: "incomplete",
  };
  const bundle = {
    protocol: frozen.protocol,
    assignments: frozen.assignments,
    observations: [observation],
    frozen_confirmation: frozen,
    registered_freeze_digest: frozen.freeze_digest,
  };
  const report = analyzeRoutingTrial(bundle);
  assert.equal(report.registered_freeze_digest, frozen.freeze_digest);
  assert.equal(report.freeze_bound, true);
  assert.equal(report.production_routing_authorized, false);
  assert.throws(() => analyzeRoutingTrial({ ...bundle, frozen_confirmation: undefined }), /frozen confirmation/);
  assert.throws(() => analyzeRoutingTrial({ ...bundle, registered_freeze_digest: "f".repeat(64) }), /registered freeze digest/);
  assert.throws(() => analyzeRoutingTrial({
    ...bundle, observations: [{ ...observation, freeze_digest: "f".repeat(64) }],
  }), /observation freeze digest/);
  const changedPin = structuredClone(frozen);
  changedPin.pins.adapter_digest = sha("changed adapter");
  assert.throws(() => analyzeRoutingTrial({ ...bundle, frozen_confirmation: changedPin }), /freeze digest/);
  const changedOpportunity = structuredClone(frozen);
  changedOpportunity.assignments.find((row) => row.planned_advice_opportunity).planned_advice_opportunity = false;
  assert.throws(() => analyzeRoutingTrial({ ...bundle, frozen_confirmation: changedOpportunity }), /case facts drift|freeze digest/);
});

test("synthetic complete observations can meet numeric gates but cannot authorize routing", () => {
  const frozen = freezeConfirmationCampaign(fixture());
  const observations = frozen.assignments.map((assignment) => ({
    assignment_id: assignment.assignment_id,
    manifest_digest: frozen.manifest_digest,
    freeze_digest: frozen.freeze_digest,
    status: "complete",
    accepted: true,
    outcome_provenance: "independent_blinded",
    cost_nano_usd: assignment.arm === "R0" ? 100 : 80,
    cost_components_nano_usd: {
      advisor: 0, worker: assignment.arm === "R0" ? 100 : 80, checks: 0,
      handoff: 0, rework: 0, differing_infrastructure: 0,
    },
    cost_evidence: "provider_billed",
    usage_reconciled: true,
    end_to_end_ms: 1000,
    foreground_routing_wait_ms: assignment.arm === "R0" ? 0 : 100,
    human_intervention_ms: 0,
    authority_violation: false,
    routing_exposure: {
      advisor_may_send: 0, completed_advisor_sends: 0,
      initial_admission: {
        reason: "strong_default", role: "strong", advice_status: "not_used",
      },
    },
  }));
  const report = analyzeRoutingTrial({
    protocol: frozen.protocol,
    assignments: frozen.assignments,
    observations,
    frozen_confirmation: frozen,
    registered_freeze_digest: frozen.freeze_digest,
  });
  assert.deepEqual(report.conditional_thresholds, {
    quality: true, cost: true, latency: true, intervention: true,
  });
  assert.equal(report.statistical_gates_passed, false);
  assert.equal(report.production_routing_authorized, false);
  assert.deepEqual(report.planned_opportunity, {
    cases: 67, repositories: 34, packet_variants: 12,
  });
  assert.equal(report.reported_exposure.rj_initial_applied_admissions, 0);
  assert.match(report.issues.join(" "), /no Jev-influenced admission/);
});

test("CLI writes once and never replaces a frozen schedule", async () => {
  const dir = await mkdtemp(path.join(tmpdir(), "pytxo-freeze-campaign-"));
  const source = path.join(dir, "input.json");
  const destination = path.join(dir, "frozen.json");
  await writeFile(source, JSON.stringify(fixture()));
  await execFileAsync(process.execPath, [scriptPath, source, destination]);
  const first = await readFile(destination, "utf8");
  assert.deepEqual(JSON.parse(first), freezeConfirmationCampaign(fixture()));
  await execFileAsync(process.execPath, [scriptPath, "--verify", destination]);
  await assert.rejects(execFileAsync(process.execPath, [scriptPath, source, destination]));
  assert.equal(await readFile(destination, "utf8"), first);
});
