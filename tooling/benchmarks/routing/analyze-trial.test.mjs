import assert from "node:assert/strict";
import { execFile } from "node:child_process";
import { mkdtemp, readFile, writeFile } from "node:fs/promises";
import { tmpdir } from "node:os";
import path from "node:path";
import test from "node:test";
import { fileURLToPath } from "node:url";
import { promisify } from "node:util";

import { analyzeRoutingTrial, clopperPearsonUpper, manifestDigest } from "./analyze-trial.mjs";

const execFileAsync = promisify(execFile);
const scriptPath = fileURLToPath(new URL("./analyze-trial.mjs", import.meta.url));

function fixture(repositoryCount = 12, casesPerRepository = 1) {
  const protocol = {
    schema_version: 1,
    phase: "fixture",
    campaign_id: "synthetic_only",
    analysis_seed: 26192,
    bootstrap_resamples: 10000,
  };
  const assignments = [];
  for (let repo = 0; repo < repositoryCount; repo++) {
    for (let caseIndex = 0; caseIndex < casesPerRepository; caseIndex++) {
    for (const arm of ["R0", "RJ"]) {
      for (const repetition of [1, 2]) {
        assignments.push({
          assignment_id: `a_${repo}_${caseIndex}_${arm}_${repetition}`,
          case_id: `case_${repo}_${caseIndex}`,
          repository_group_id: `repo_${repo}`,
          snapshot_digest: "a".repeat(64),
          packet_digest: "b".repeat(64),
          family: "mechanical",
          split: "fixture",
          billing_mode: "api",
          billing_cohort_id: "one_tariff",
          arm,
          repetition,
        });
      }
    }
    }
  }
  const digest = manifestDigest(protocol, assignments);
  const observations = assignments.map((assignment) => ({
    assignment_id: assignment.assignment_id,
    manifest_digest: digest,
    status: "complete",
    accepted: true,
    outcome_provenance: "independent_blinded",
    cost_nano_usd: assignment.arm === "R0" ? 100 : 80,
    cost_components_nano_usd: {
      advisor: 0,
      worker: assignment.arm === "R0" ? 100 : 80,
      checks: 0,
      handoff: 0,
      rework: 0,
      differing_infrastructure: 0,
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
  return { protocol, assignments, observations };
}

test("the analysis is reproducible under assignment and observation ordering", () => {
  const bundle = fixture(12, 2);
  for (const row of bundle.observations) {
    const assignment = bundle.assignments.find((candidate) => candidate.assignment_id === row.assignment_id);
    const repository = Number(assignment.repository_group_id.slice(5));
    const caseIndex = Number(assignment.case_id.split("_").at(-1));
    row.end_to_end_ms += repository * 11 + caseIndex * 37;
    row.human_intervention_ms = repository * (assignment.arm === "RJ" ? 7 : 3);
    row.cost_nano_usd += repository + caseIndex * 3;
    row.cost_components_nano_usd.worker += repository + caseIndex * 3;
  }
  const original = analyzeRoutingTrial(bundle);
  const reordered = analyzeRoutingTrial({
    ...bundle,
    assignments: [...bundle.assignments].reverse(),
    observations: [...bundle.observations].reverse(),
  });
  assert.deepEqual(original, reordered);
  assert.equal(original.phase, "fixture");
  assert.equal(original.statistical_gates_passed, false);
  assert.equal(original.production_routing_authorized, false);
  assert.equal(original.reported_exposure.rj_initial_applied_admissions, 0);
  assert.match(original.issues.join(" "), /no Jev-influenced admission/);
  assert.ok(original.point.cost_ratio < 1);
  assert.equal(original.point.repository_weighted_cost_ratio, original.point.cost_ratio);
  assert.equal(original.point.repository_weighted_quality_difference, original.point.quality_difference);
});

test("zero observed harm still has a conservative nonzero quality bound", () => {
  const upper = clopperPearsonUpper(0, 200);
  assert.ok(Math.abs(upper - (1 - 0.05 ** (1 / 200))) < 1e-12);
  const report = analyzeRoutingTrial(fixture());
  assert.equal(report.bounds.quality_method, "conservative_repository_harm_cp");
  assert.ok(report.bounds.quality_lower < 0);
  assert.equal(report.bounds.discordant_repositories, 0);
});

test("report separates bundle-reported applied admissions from planned opportunities and unexposed numerical wins", () => {
  const bundle = fixture(2);
  const rj = bundle.assignments.find((assignment) => assignment.arm === "RJ");
  const observation = bundle.observations.find((row) => row.assignment_id === rj.assignment_id);
  observation.routing_exposure = {
    advisor_may_send: 1, completed_advisor_sends: 1,
    initial_admission: { reason: "advice_everyday", role: "everyday", advice_status: "applied" },
  };
  const report = analyzeRoutingTrial(bundle);
  assert.equal(report.reported_exposure.rj_initial_applied_admissions, 1);
  assert.equal(report.reported_exposure.rj_completed_advisor_sends, 1);
  assert.equal(report.reported_exposure.r0_initial_strong_default_admissions, 4);
  assert.match(report.evidence_gaps.join(" "), /Store trace and review ledger/);
  assert.doesNotMatch(report.issues.join(" "), /no Jev-influenced admission/);
  assert.equal(report.statistical_gates_passed, false);
});

test("ten discordant repository clusters switch to paired bootstrap", () => {
  const bundle = fixture();
  for (const assignment of bundle.assignments) {
    if (Number(assignment.repository_group_id.slice(5)) < 10 && assignment.arm === "R0") {
      bundle.observations.find((row) => row.assignment_id === assignment.assignment_id).accepted = false;
    }
  }
  const report = analyzeRoutingTrial(bundle);
  assert.equal(report.bounds.quality_method, "paired_hierarchical_bootstrap");
  assert.equal(report.bounds.discordant_repositories, 10);
  assert.equal(report.bounds.harmed_repositories, 0);
  assert.ok(report.point.quality_difference > 0);
});

test("unknown or unreconciled spend cannot become zero-cost evidence", () => {
  const bundle = fixture(2);
  bundle.observations[0].cost_nano_usd = null;
  bundle.observations[0].cost_components_nano_usd.worker = null;
  bundle.observations[0].cost_evidence = "unknown";
  bundle.observations[0].usage_reconciled = false;
  const report = analyzeRoutingTrial(bundle);
  assert.equal(report.point.cost_ratio, null);
  assert.equal(report.point.all_assigned_cost_nano_usd, null);
  assert.equal(report.bounds.cost_ratio_upper, null);
  assert.equal(report.conditional_thresholds.cost, false);
  assert.match(report.issues.join(" "), /unknown or unreconciled/);
});

test("zero accepted treatment runs yield infinite cost per acceptance and fail economics", () => {
  const bundle = fixture(2);
  for (const row of bundle.observations) {
    if (row.assignment_id.includes("_RJ_")) row.accepted = false;
  }
  const report = analyzeRoutingTrial(bundle);
  assert.equal(report.point.cost_per_accepted_nano_usd.RJ, "Infinity");
  assert.equal(report.point.cost_ratio, "Infinity");
  assert.equal(report.conditional_thresholds.cost, false);
});

test("subscription dollars are never mixed into API cost ratio", () => {
  const bundle = fixture(2);
  for (const assignment of bundle.assignments) {
    if (assignment.arm === "RJ") assignment.billing_mode = "subscription";
  }
  assert.throws(() => analyzeRoutingTrial(bundle), /case facts drift/);
  for (const assignment of bundle.assignments) assignment.billing_mode = "subscription";
  const digest = manifestDigest(bundle.protocol, bundle.assignments);
  for (const row of bundle.observations) row.manifest_digest = digest;
  const report = analyzeRoutingTrial(bundle);
  assert.equal(report.point.cost_ratio, null);
  assert.equal(report.conditional_thresholds.cost, false);
  assert.match(report.issues.join(" "), /API billing cohort/);
});

test("missing assigned outcomes remain visible and suppress all gates", () => {
  const bundle = fixture(2);
  bundle.observations.pop();
  const report = analyzeRoutingTrial(bundle);
  assert.equal(report.assigned_runs, 8);
  assert.equal(report.observed_runs, 7);
  assert.equal(report.point, null);
  assert.equal(report.statistical_gates_passed, false);
  assert.match(report.issues[0], /1 assigned run/);
});

test("extra, duplicated, or misbound observations fail closed", () => {
  const bundle = fixture(2);
  assert.throws(() => analyzeRoutingTrial({ ...bundle, observations: [...bundle.observations, bundle.observations[0]] }), /duplicate observation/);
  assert.throws(() => analyzeRoutingTrial({ ...bundle, observations: [...bundle.observations, { ...bundle.observations[0], assignment_id: "foreign" }] }), /no frozen assignment/);
  assert.throws(() => analyzeRoutingTrial({ ...bundle, observations: bundle.observations.map((row, index) => index === 0 ? { ...row, manifest_digest: "f".repeat(64) } : row) }), /manifest digest mismatch/);
  assert.throws(() => analyzeRoutingTrial({ ...bundle, observations: bundle.observations.map((row, index) => index === 0 ? { ...row, arm: "RJ" } : row) }), /unexpected field: arm/);
  assert.throws(() => analyzeRoutingTrial({ ...bundle, observations: bundle.observations.map((row, index) => index === 0 ? { ...row, goal: "private" } : row) }), /unexpected field: goal/);
  assert.throws(() => analyzeRoutingTrial({ ...bundle, observations: bundle.observations.map((row, index) => index === 0 ? { ...row, cost_components_nano_usd: { ...row.cost_components_nano_usd, advisor: 1 } } : row) }), /total cost does not match component sum/);
  assert.throws(() => analyzeRoutingTrial({ ...bundle, observations: bundle.observations.map((row, index) => index === 0 ? { ...row, routing_exposure: undefined } : row) }), /routing exposure/);
  assert.throws(() => analyzeRoutingTrial({ ...bundle, observations: bundle.observations.map((row, index) => index === 0 ? { ...row, routing_exposure: { ...row.routing_exposure, completed_advisor_sends: 1 } } : row) }), /completed.*may-send/);
  assert.throws(() => analyzeRoutingTrial({ ...bundle, observations: bundle.observations.map((row, index) => index === 0 ? { ...row, routing_exposure: { ...row.routing_exposure, initial_admission: { reason: "advice_everyday", role: "everyday", advice_status: "applied" } } } : row) }), /rules arm.*applied/);
});

test("authority violations, foreground wait and intervention each block the relevant gates", () => {
  const bundle = fixture(2);
  bundle.observations[0].authority_violation = true;
  bundle.observations[1].foreground_routing_wait_ms = 2001;
  bundle.observations[2].human_intervention_ms = 2_000_000;
  const report = analyzeRoutingTrial(bundle);
  assert.equal(report.conditional_thresholds.intervention, false);
  assert.equal(report.conditional_thresholds.latency, false);
  assert.equal(report.point.max_foreground_routing_wait_ms, 2001);
});

test("confirmation cannot be claimed from a small synthetic corpus", () => {
  const bundle = fixture(2);
  bundle.protocol.phase = "confirmation";
  for (const assignment of bundle.assignments) {
    assignment.split = "holdout";
    assignment.planned_advice_opportunity = true;
  }
  const digest = manifestDigest(bundle.protocol, bundle.assignments);
  for (const row of bundle.observations) row.manifest_digest = digest;
  assert.throws(() => analyzeRoutingTrial(bundle), /400 cases, 200 repositories/);
});

test("a self-labeled full-size synthetic holdout without a freeze is rejected", () => {
  const bundle = fixture(200, 2);
  bundle.protocol.phase = "confirmation";
  const families = ["mechanical", "local_defects", "state_persistence", "concurrency", "adapter_protocol", "cross_component"];
  const limits = [67, 134, 201, 268, 334, 400];
  for (const assignment of bundle.assignments) {
    const [, repository, caseIndex] = assignment.case_id.split("_");
    const ordinal = Number(repository) * 2 + Number(caseIndex);
    assignment.family = families[limits.findIndex((limit) => ordinal < limit)];
    assignment.planned_advice_opportunity = ordinal >= 67 && ordinal < 134;
    assignment.split = "holdout";
  }
  const digest = manifestDigest(bundle.protocol, bundle.assignments);
  for (const row of bundle.observations) row.manifest_digest = digest;
  assert.throws(() => analyzeRoutingTrial(bundle), /frozen confirmation/);
});

test("CLI writes a report once without overwriting a reviewed result", async () => {
  const directory = await mkdtemp(path.join(tmpdir(), "pytxo-routing-analysis-"));
  const input = path.join(directory, "synthetic-bundle.json");
  const output = path.join(directory, "report.json");
  await writeFile(input, JSON.stringify(fixture(2)));
  const first = await execFileAsync(process.execPath, [scriptPath, input, output]);
  assert.match(first.stdout, /statistical gates passed: false/);
  const report = JSON.parse(await readFile(output, "utf8"));
  assert.equal(report.production_routing_authorized, false);
  await assert.rejects(execFileAsync(process.execPath, [scriptPath, input, output]), /EEXIST/);
  assert.deepEqual(JSON.parse(await readFile(output, "utf8")), report);
});
