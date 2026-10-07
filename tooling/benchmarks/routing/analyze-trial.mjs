import { createHash } from "node:crypto";
import { readFile, writeFile } from "node:fs/promises";
import { pathToFileURL } from "node:url";

import { verifyFrozenConfirmation } from "./freeze-confirmation.mjs";

const ARMS = ["R0", "RJ"];
const FAMILIES = [
  "mechanical", "local_defects", "state_persistence", "concurrency",
  "adapter_protocol", "cross_component",
];
const EVIDENCED_COST = new Set(["provider_billed", "provider_usage_plus_pinned_rate"]);
const COST_COMPONENTS = ["advisor", "worker", "checks", "handoff", "rework", "differing_infrastructure"];
const ANALYSIS_VERSION = "routing-paired-v2";
const ALPHA = 0.05;
const PROTOCOL_KEYS = new Set(["schema_version", "phase", "campaign_id", "analysis_seed", "bootstrap_resamples"]);
const ASSIGNMENT_KEYS = new Set([
  "assignment_id", "case_id", "repository_group_id", "snapshot_digest", "packet_digest", "family", "split",
  "planned_advice_opportunity", "billing_mode", "billing_cohort_id", "arm", "repetition",
]);
const OBSERVATION_KEYS = new Set([
  "assignment_id", "manifest_digest", "freeze_digest", "status", "accepted", "outcome_provenance",
  "cost_nano_usd", "cost_components_nano_usd", "cost_evidence", "usage_reconciled", "end_to_end_ms",
  "foreground_routing_wait_ms", "human_intervention_ms", "authority_violation",
  "routing_exposure",
]);
const EXPOSED_REASONS = new Set([
  "manual", "required", "mechanical_everyday", "strong_default", "advice_everyday", "strong_repair",
]);
const EXPOSED_STATUSES = new Set(["not_used", "invalid_or_stale", "applied", "rules_fallback"]);

function requireObject(value, name) {
  if (value === null || typeof value !== "object" || Array.isArray(value)) {
    throw new Error(`${name} must be an object`);
  }
}

function requireKnownKeys(value, keys, name) {
  for (const key of Object.keys(value)) {
    if (!keys.has(key)) throw new Error(`${name} contains unexpected field: ${key}`);
  }
}

function requireToken(value, name) {
  if (typeof value !== "string" || !/^[a-zA-Z0-9_-]{1,96}$/.test(value)) {
    throw new Error(`${name} must be an opaque token`);
  }
}

function requireInteger(value, name, min = 0) {
  if (!Number.isSafeInteger(value) || value < min) {
    throw new Error(`${name} must be a safe integer >= ${min}`);
  }
}

function stableJson(value) {
  if (Array.isArray(value)) return `[${value.map(stableJson).join(",")}]`;
  if (value !== null && typeof value === "object") {
    return `{${Object.keys(value).sort().map((key) => `${JSON.stringify(key)}:${stableJson(value[key])}`).join(",")}}`;
  }
  return JSON.stringify(value);
}

function compareOpaque(a, b) {
  return a < b ? -1 : a > b ? 1 : 0;
}

export function manifestDigest(protocol, assignments) {
  return createHash("sha256")
    .update(stableJson({ protocol, assignments: [...assignments].sort((a, b) => compareOpaque(a.assignment_id, b.assignment_id)) }))
    .digest("hex");
}

function validateProtocol(protocol) {
  requireObject(protocol, "protocol");
  requireKnownKeys(protocol, PROTOCOL_KEYS, "protocol");
  if (protocol.schema_version !== 1 || !["fixture", "confirmation"].includes(protocol.phase)) {
    throw new Error("unsupported protocol schema or phase");
  }
  requireToken(protocol.campaign_id, "campaign_id");
  requireInteger(protocol.analysis_seed, "analysis_seed", 1);
  if (protocol.analysis_seed > 0xffffffff) throw new Error("analysis_seed must be uint32");
  if (protocol.bootstrap_resamples !== 10000) {
    throw new Error("routing-paired-v2 requires 10,000 pre-registered bootstrap resamples");
  }
}

function validateAssignments(protocol, assignments) {
  if (!Array.isArray(assignments) || assignments.length === 0) throw new Error("assignments must be nonempty");
  const byId = new Map();
  const byTuple = new Set();
  const cases = new Map();
  const repositories = new Map();
  const cohorts = new Set();
  for (const assignment of assignments) {
    requireObject(assignment, "assignment");
    requireKnownKeys(assignment, ASSIGNMENT_KEYS, "assignment");
    for (const key of ["assignment_id", "case_id", "repository_group_id", "billing_cohort_id"]) {
      requireToken(assignment[key], key);
    }
    if (!/^[a-f0-9]{64}$/.test(assignment.snapshot_digest)) throw new Error("snapshot_digest must be SHA-256");
    if (!/^[a-f0-9]{64}$/.test(assignment.packet_digest)) throw new Error("packet_digest must be SHA-256");
    if (!FAMILIES.includes(assignment.family) || !ARMS.includes(assignment.arm)) {
      throw new Error("unknown family or arm");
    }
    if (assignment.split !== (protocol.phase === "confirmation" ? "holdout" : "fixture")) {
      throw new Error("assignment split differs from protocol phase");
    }
    if (protocol.phase === "confirmation" && typeof assignment.planned_advice_opportunity !== "boolean") {
      throw new Error("confirmation assignment needs planned_advice_opportunity");
    }
    if (protocol.phase === "fixture" && assignment.planned_advice_opportunity !== undefined) {
      throw new Error("fixture cannot claim a frozen advice opportunity");
    }
    if (assignment.billing_mode !== "api" && assignment.billing_mode !== "subscription" && assignment.billing_mode !== "local") {
      throw new Error("unknown billing mode");
    }
    requireInteger(assignment.repetition, "repetition", 1);
    if (byId.has(assignment.assignment_id)) throw new Error("duplicate assignment_id");
    const tuple = `${assignment.case_id}/${assignment.arm}/${assignment.repetition}`;
    if (byTuple.has(tuple)) throw new Error("duplicate case/arm/repetition assignment");
    byTuple.add(tuple);
    byId.set(assignment.assignment_id, assignment);
    const caseFacts = stableJson({
      repository_group_id: assignment.repository_group_id,
      snapshot_digest: assignment.snapshot_digest,
      packet_digest: assignment.packet_digest,
      family: assignment.family,
      ...(protocol.phase === "confirmation" && {
        planned_advice_opportunity: assignment.planned_advice_opportunity,
      }),
      billing_cohort_id: assignment.billing_cohort_id,
      billing_mode: assignment.billing_mode,
    });
    if (cases.has(assignment.case_id) && cases.get(assignment.case_id) !== caseFacts) {
      throw new Error("case facts drift between arms or repetitions");
    }
    cases.set(assignment.case_id, caseFacts);
    repositories.set(assignment.repository_group_id, (repositories.get(assignment.repository_group_id) ?? new Set()).add(assignment.case_id));
    cohorts.add(`${assignment.billing_mode}/${assignment.billing_cohort_id}`);
  }
  const repetitions = protocol.phase === "confirmation" ? 2 : Math.max(...assignments.map((a) => a.repetition));
  for (const caseId of cases.keys()) {
    for (const arm of ARMS) {
      for (let repetition = 1; repetition <= repetitions; repetition++) {
        if (!byTuple.has(`${caseId}/${arm}/${repetition}`)) throw new Error("unpaired or missing arm/repetition assignment");
      }
    }
  }
  if (protocol.phase === "confirmation") {
    if (cases.size !== 400 || repositories.size !== 200 || assignments.length !== 1600) {
      throw new Error("confirmation requires 400 cases, 200 repositories and 1,600 assigned runs");
    }
    const familyCounts = Object.fromEntries(FAMILIES.map((family) => [family, 0]));
    for (const caseFacts of cases.values()) familyCounts[JSON.parse(caseFacts).family]++;
    if (FAMILIES.some((family, index) => familyCounts[family] !== (index < 4 ? 67 : 66))) {
      throw new Error("confirmation family balance differs from frozen protocol");
    }
    if ([...repositories.values()].some((caseIds) => caseIds.size !== 2)) {
      throw new Error("confirmation requires two tasks per repository");
    }
  }
  return { byId, cases, repositories, cohorts, repetitions };
}

function validateObservations(observations, assignments, digest, freezeDigest = null) {
  if (!Array.isArray(observations)) throw new Error("observations must be an array");
  const byId = new Map();
  for (const row of observations) {
    requireObject(row, "observation");
    requireKnownKeys(row, OBSERVATION_KEYS, "observation");
    requireToken(row.assignment_id, "observation assignment_id");
    if (!assignments.byId.has(row.assignment_id)) throw new Error("extra observation has no frozen assignment");
    if (byId.has(row.assignment_id)) throw new Error("duplicate observation for assignment");
    if (row.manifest_digest !== digest) throw new Error("observation manifest digest mismatch");
    if (freezeDigest !== null && row.freeze_digest !== freezeDigest) {
      throw new Error("observation freeze digest mismatch");
    }
    if (freezeDigest === null && row.freeze_digest !== undefined) {
      throw new Error("fixture observation cannot claim a confirmation freeze");
    }
    if (!["complete", "missing", "incomplete"].includes(row.status)) throw new Error("unknown observation status");
    if (row.status === "complete") {
      if (typeof row.accepted !== "boolean" || row.outcome_provenance !== "independent_blinded") {
        throw new Error("complete outcome requires independent blinded acceptance");
      }
      requireObject(row.cost_components_nano_usd, "cost_components_nano_usd");
      requireKnownKeys(row.cost_components_nano_usd, new Set(COST_COMPONENTS), "cost_components_nano_usd");
      const componentCosts = COST_COMPONENTS.map((component) => {
        if (!Object.hasOwn(row.cost_components_nano_usd, component)) {
          throw new Error(`missing cost component: ${component}`);
        }
        const amount = row.cost_components_nano_usd[component];
        if (amount !== null) requireInteger(amount, `cost component ${component}`);
        return amount;
      });
      if (componentCosts.some((amount) => amount === null)) {
        if (row.cost_nano_usd !== null) throw new Error("unknown component requires unknown total cost");
      } else {
        requireInteger(row.cost_nano_usd, "cost_nano_usd");
        if (safeTotal(componentCosts) !== row.cost_nano_usd) {
          throw new Error("total cost does not match component sum");
        }
      }
      if (![...EVIDENCED_COST, "adapter_reported", "estimated", "unknown"].includes(row.cost_evidence)) {
        throw new Error("unknown cost evidence");
      }
      for (const key of ["end_to_end_ms", "foreground_routing_wait_ms", "human_intervention_ms"]) {
        if (row[key] !== null) requireInteger(row[key], key, key === "end_to_end_ms" ? 1 : 0);
      }
      if (typeof row.authority_violation !== "boolean" || typeof row.usage_reconciled !== "boolean") {
        throw new Error("complete observation requires explicit authority and usage states");
      }
      requireObject(row.routing_exposure, "routing exposure");
      requireKnownKeys(row.routing_exposure,
        new Set(["advisor_may_send", "completed_advisor_sends", "initial_admission"]), "routing exposure");
      requireInteger(row.routing_exposure.advisor_may_send, "routing exposure advisor_may_send");
      requireInteger(row.routing_exposure.completed_advisor_sends, "routing exposure completed_advisor_sends");
      if (row.routing_exposure.completed_advisor_sends > row.routing_exposure.advisor_may_send) {
        throw new Error("completed advisor sends exceed may-send journal rows");
      }
      const admission = row.routing_exposure.initial_admission;
      if (admission !== null) {
        requireObject(admission, "routing exposure initial admission");
        requireKnownKeys(admission, new Set(["reason", "role", "advice_status"]), "routing exposure initial admission");
        if (!EXPOSED_REASONS.has(admission.reason) || !["everyday", "strong"].includes(admission.role)
          || !EXPOSED_STATUSES.has(admission.advice_status)) {
          throw new Error("routing exposure initial admission has invalid decision fields");
        }
        if (assignments.byId.get(row.assignment_id).arm === "R0"
          && admission.advice_status !== "not_used") {
          throw new Error("rules arm cannot claim applied or other advisor status");
        }
        if (admission.advice_status === "applied"
          && (admission.reason !== "advice_everyday" || admission.role !== "everyday"
            || row.routing_exposure.completed_advisor_sends === 0)) {
          throw new Error("applied initial admission requires completed everyday advice");
        }
        if (admission.reason === "advice_everyday" && admission.advice_status !== "applied") {
          throw new Error("advice_everyday reason requires applied advice");
        }
        if (admission.advice_status === "rules_fallback" && row.routing_exposure.completed_advisor_sends === 0) {
          throw new Error("rules fallback requires a completed advisor send");
        }
      }
      if (assignments.byId.get(row.assignment_id).arm === "R0"
        && (row.routing_exposure.advisor_may_send !== 0 || row.routing_exposure.completed_advisor_sends !== 0)) {
        throw new Error("rules arm cannot contain advisor send exposure");
      }
    }
    byId.set(row.assignment_id, row);
  }
  return byId;
}

function randomGenerator(seed) {
  let state = seed >>> 0;
  return () => {
    state ^= state << 13;
    state ^= state >>> 17;
    state ^= state << 5;
    return (state >>> 0) / 0x100000000;
  };
}

function percentile(values, p) {
  const sorted = [...values].sort((a, b) => a - b);
  return sorted[Math.ceil(p * sorted.length) - 1];
}

function binomialCdf(x, n, p) {
  if (p === 0) return 1;
  if (p === 1) return x === n ? 1 : 0;
  let term = (1 - p) ** n;
  let sum = term;
  for (let k = 1; k <= x; k++) {
    term *= ((n - k + 1) / k) * (p / (1 - p));
    sum += term;
  }
  return sum;
}

export function clopperPearsonUpper(x, n) {
  requireInteger(x, "harm count");
  requireInteger(n, "repository count", 1);
  if (x > n) throw new Error("harm count exceeds repository count");
  if (x === n) return 1;
  let low = 0;
  let high = 1;
  for (let iteration = 0; iteration < 70; iteration++) {
    const mid = (low + high) / 2;
    if (binomialCdf(x, n, mid) > ALPHA) low = mid;
    else high = mid;
  }
  return (low + high) / 2;
}

function safeTotal(values) {
  let total = 0;
  for (const value of values) {
    total += value;
    if (!Number.isSafeInteger(total)) throw new Error("total cost exceeds safe integer precision");
  }
  return total;
}

function metrics(tasks, includeCost) {
  const armRuns = { R0: [], RJ: [] };
  for (const task of tasks) {
    for (const arm of ARMS) armRuns[arm].push(...task[arm]);
  }
  const accepted = Object.fromEntries(ARMS.map((arm) => [arm, armRuns[arm].filter((row) => row.accepted).length]));
  const qualityDifference = accepted.RJ / armRuns.RJ.length - accepted.R0 / armRuns.R0.length;
  const costTotals = includeCost
    ? Object.fromEntries(ARMS.map((arm) => [arm, safeTotal(armRuns[arm].map((row) => row.cost_nano_usd))]))
    : null;
  const costRatio = !includeCost ? null
    : accepted.RJ === 0 || accepted.R0 === 0 || costTotals.R0 === 0
      ? Infinity
      : (costTotals.RJ / accepted.RJ) / (costTotals.R0 / accepted.R0);
  const latencyRatio = percentile(armRuns.RJ.map((row) => row.end_to_end_ms), 0.95)
    / percentile(armRuns.R0.map((row) => row.end_to_end_ms), 0.95);
  const interventionDifferenceMs = safeTotal(armRuns.RJ.map((row) => row.human_intervention_ms)) / armRuns.RJ.length
    - safeTotal(armRuns.R0.map((row) => row.human_intervention_ms)) / armRuns.R0.length;
  return { qualityDifference, costRatio, latencyRatio, interventionDifferenceMs, accepted, costTotals };
}

function publicNumber(value) {
  return Number.isFinite(value) ? value : "Infinity";
}

export function analyzeRoutingTrial(bundle) {
  requireObject(bundle, "analysis bundle");
  requireKnownKeys(bundle, new Set([
    "protocol", "assignments", "observations", "frozen_confirmation", "registered_freeze_digest",
  ]), "analysis bundle");
  const { protocol, assignments, observations } = bundle;
  validateProtocol(protocol);
  const manifest = validateAssignments(protocol, assignments);
  const digest = manifestDigest(protocol, assignments);
  let freezeDigest = null;
  if (protocol.phase === "confirmation") {
    if (bundle.frozen_confirmation === undefined) throw new Error("confirmation requires frozen confirmation");
    verifyFrozenConfirmation(bundle.frozen_confirmation);
    freezeDigest = bundle.frozen_confirmation.freeze_digest;
    if (bundle.registered_freeze_digest !== freezeDigest) throw new Error("registered freeze digest mismatch");
    if (bundle.frozen_confirmation.manifest_digest !== digest
      || stableJson(bundle.frozen_confirmation.protocol) !== stableJson(protocol)
      || stableJson(bundle.frozen_confirmation.assignments) !== stableJson(assignments)) {
      throw new Error("analysis assignments differ from frozen confirmation");
    }
  } else if (bundle.frozen_confirmation !== undefined || bundle.registered_freeze_digest !== undefined) {
    throw new Error("fixture cannot claim a confirmation freeze");
  }
  const observed = validateObservations(observations, manifest, digest, freezeDigest);
  const missing = assignments.filter((a) => !observed.has(a.assignment_id) || observed.get(a.assignment_id).status !== "complete").length;
  const rows = assignments.map((a) => ({ ...a, ...observed.get(a.assignment_id) }));
  const allComplete = missing === 0;
  const apiCohort = manifest.cohorts.size === 1 && assignments.every((a) => a.billing_mode === "api");
  const costComplete = allComplete && apiCohort && rows.every((row) =>
    row.usage_reconciled && EVIDENCED_COST.has(row.cost_evidence)
    && row.cost_nano_usd !== null
    && COST_COMPONENTS.every((component) => row.cost_components_nano_usd[component] !== null));
  const timingComplete = allComplete && rows.every((row) =>
    row.end_to_end_ms !== null && row.foreground_routing_wait_ms !== null && row.human_intervention_ms !== null);
  const authorityComplete = allComplete && rows.every((row) => typeof row.authority_violation === "boolean");
  const plannedOpportunityCases = protocol.phase === "confirmation"
    ? [...manifest.cases.entries()]
      .map(([caseId, facts]) => ({ caseId, ...JSON.parse(facts) }))
      .filter((row) => row.planned_advice_opportunity)
    : [];
  const exposure = allComplete ? {
    rj_assigned_runs: rows.filter((row) => row.arm === "RJ").length,
    rj_may_send_requests: safeTotal(rows.filter((row) => row.arm === "RJ")
      .map((row) => row.routing_exposure.advisor_may_send)),
    rj_completed_advisor_sends: safeTotal(rows.filter((row) => row.arm === "RJ")
      .map((row) => row.routing_exposure.completed_advisor_sends)),
    rj_initial_applied_admissions: rows.filter((row) => row.arm === "RJ"
      && row.routing_exposure.initial_admission?.advice_status === "applied").length,
    rj_initial_fallback_admissions: rows.filter((row) => row.arm === "RJ"
      && row.routing_exposure.initial_admission?.advice_status === "rules_fallback").length,
    r0_initial_strong_default_admissions: rows.filter((row) => row.arm === "R0"
      && row.routing_exposure.initial_admission?.reason === "strong_default").length,
    planned_opportunity_r0_strong_default_admissions: rows.filter((row) => row.arm === "R0"
      && row.planned_advice_opportunity
      && row.routing_exposure.initial_admission?.reason === "strong_default").length,
    planned_opportunity_rj_applied_admissions: rows.filter((row) => row.arm === "RJ"
      && row.planned_advice_opportunity
      && row.routing_exposure.initial_admission?.advice_status === "applied").length,
    rj_applied_outside_planned_opportunity: rows.filter((row) => row.arm === "RJ"
      && protocol.phase === "confirmation" && !row.planned_advice_opportunity
      && row.routing_exposure.initial_admission?.advice_status === "applied").length,
  } : null;
  const issues = [];
  if (!allComplete) issues.push(`${missing} assigned run(s) missing or incomplete`);
  if (!apiCohort) issues.push("dollar cost requires one comparable API billing cohort");
  if (!costComplete && apiCohort) issues.push("cost or provider evidence is unknown or unreconciled");
  if (!timingComplete && allComplete) issues.push("monotonic latency, routing wait or intervention time is unknown");
  if (!authorityComplete && allComplete) issues.push("authority result is unknown");
  if (exposure?.rj_initial_applied_admissions === 0) {
    issues.push("bundle reports no Jev-influenced admission; numerical differences cannot establish Jev routing benefit");
  }
  if (exposure?.rj_applied_outside_planned_opportunity > 0) {
    issues.push("applied Jev route outside frozen planned-opportunity stratum needs independent review");
  }
  const report = {
    schema_version: 1,
    analysis_version: ANALYSIS_VERSION,
    phase: protocol.phase,
    manifest_digest: digest,
    registered_freeze_digest: freezeDigest,
    freeze_bound: freezeDigest !== null,
    assigned_runs: assignments.length,
    observed_runs: observed.size,
    cases: manifest.cases.size,
    repositories: manifest.repositories.size,
    planned_opportunity: protocol.phase === "confirmation" ? {
      cases: plannedOpportunityCases.length,
      repositories: new Set(plannedOpportunityCases.map((row) => row.repository_group_id)).size,
      packet_variants: new Set(plannedOpportunityCases.map((row) => row.packet_digest)).size,
    } : null,
    reported_exposure: exposure,
    issues,
    evidence_gaps: [
      "frozen manifest and trial permit not independently authenticated",
      "component charges and unknown sends not reconciled to durable source receipts",
      "routing exposure is a bundle claim until its private Store trace and review ledger are authenticated",
      "separate authority stress cases and secret/unaccounted-launch results not validated",
    ],
    point: null,
    bounds: null,
    conditional_thresholds: { quality: false, cost: false, latency: false, intervention: false },
    statistical_gates_passed: false,
    production_routing_authorized: false,
  };
  if (!allComplete || !timingComplete) return report;

  const byCase = new Map();
  for (const row of rows) {
    if (!byCase.has(row.case_id)) {
      byCase.set(row.case_id, { caseId: row.case_id, repository: row.repository_group_id, R0: [], RJ: [] });
    }
    byCase.get(row.case_id)[row.arm].push(row);
  }
  const repositories = new Map();
  for (const task of byCase.values()) {
    task.R0.sort((a, b) => a.repetition - b.repetition);
    task.RJ.sort((a, b) => a.repetition - b.repetition);
    if (!repositories.has(task.repository)) repositories.set(task.repository, []);
    repositories.get(task.repository).push(task);
  }
  const orderedRepos = [...repositories.entries()].sort(([a], [b]) => compareOpaque(a, b));
  for (const [, group] of orderedRepos) group.sort((a, b) => compareOpaque(a.caseId, b.caseId));
  const tasks = orderedRepos.flatMap(([, group]) => group);
  const point = metrics(tasks, costComplete);
  const repoMetrics = orderedRepos.map(([, group]) => ({
    runsPerArm: group.reduce((sum, task) => sum + task.R0.length, 0),
    metrics: metrics(group, costComplete),
  }));
  const repoWeightedAcceptedRate = (arm) => repoMetrics.reduce((sum, repo) =>
    sum + repo.metrics.accepted[arm] / repo.runsPerArm, 0) / repoMetrics.length;
  const repoWeightedCostPerRun = (arm) => repoMetrics.reduce((sum, repo) =>
    sum + repo.metrics.costTotals[arm] / repo.runsPerArm, 0) / repoMetrics.length;
  const repoWeightedCostRatio = !costComplete ? null
    : repoWeightedAcceptedRate("R0") === 0 || repoWeightedAcceptedRate("RJ") === 0
      || repoWeightedCostPerRun("R0") === 0 ? Infinity
      : (repoWeightedCostPerRun("RJ") / repoWeightedAcceptedRate("RJ"))
        / (repoWeightedCostPerRun("R0") / repoWeightedAcceptedRate("R0"));
  report.point = {
    quality_difference: point.qualityDifference,
    cost_ratio: costComplete ? publicNumber(point.costRatio) : null,
    accepted_runs: point.accepted,
    all_assigned_cost_nano_usd: costComplete ? point.costTotals : null,
    cost_per_accepted_nano_usd: costComplete ? Object.fromEntries(ARMS.map((arm) => [arm,
      publicNumber(point.accepted[arm] === 0 ? Infinity : point.costTotals[arm] / point.accepted[arm]),
    ])) : null,
    p95_latency_ratio: point.latencyRatio,
    mean_intervention_difference_ms: point.interventionDifferenceMs,
    max_foreground_routing_wait_ms: Math.max(...rows.map((row) => row.foreground_routing_wait_ms)),
    repository_weighted_quality_difference: repoMetrics.reduce((sum, repo) =>
      sum + repo.metrics.qualityDifference, 0) / repoMetrics.length,
    repository_weighted_cost_ratio: costComplete ? publicNumber(repoWeightedCostRatio) : null,
  };
  report.per_family = Object.fromEntries(FAMILIES.map((family) => {
    const familyTasks = tasks.filter((task) => task.R0[0].family === family);
    if (familyTasks.length === 0) return [family, { cases: 0, accepted_runs: { R0: 0, RJ: 0 }, quality_difference: null, cost_ratio: null }];
    const familyMetrics = metrics(familyTasks, costComplete);
    return [family, {
      cases: familyTasks.length,
      accepted_runs: familyMetrics.accepted,
      quality_difference: familyMetrics.qualityDifference,
      cost_ratio: costComplete ? publicNumber(familyMetrics.costRatio) : null,
    }];
  }));
  let discordantRepos = 0;
  let harmedRepos = 0;
  for (const [, group] of orderedRepos) {
    let discordant = false;
    let harmed = false;
    for (const task of group) {
      for (let i = 0; i < task.R0.length; i++) {
        const r0 = task.R0[i].accepted;
        const rj = task.RJ[i].accepted;
        if (r0 !== rj) discordant = true;
        if (r0 && !rj) harmed = true;
      }
    }
    if (discordant) discordantRepos++;
    if (harmed) harmedRepos++;
  }
  const random = randomGenerator(protocol.analysis_seed);
  const bootstrap = { quality: [], cost: [], latency: [], intervention: [] };
  for (let sample = 0; sample < protocol.bootstrap_resamples; sample++) {
    const selected = [];
    for (let repoIndex = 0; repoIndex < orderedRepos.length; repoIndex++) {
      const group = orderedRepos[Math.floor(random() * orderedRepos.length)][1];
      for (let taskIndex = 0; taskIndex < group.length; taskIndex++) {
        selected.push(group[Math.floor(random() * group.length)]);
      }
    }
    const sampleMetrics = metrics(selected, costComplete);
    bootstrap.quality.push(sampleMetrics.qualityDifference);
    if (costComplete) bootstrap.cost.push(sampleMetrics.costRatio);
    bootstrap.latency.push(sampleMetrics.latencyRatio);
    bootstrap.intervention.push(sampleMetrics.interventionDifferenceMs);
  }
  const qualityLower = discordantRepos < 10
    ? -clopperPearsonUpper(harmedRepos, orderedRepos.length)
    : percentile(bootstrap.quality, ALPHA);
  const costUpper = costComplete ? percentile(bootstrap.cost, 1 - ALPHA) : null;
  const latencyUpper = percentile(bootstrap.latency, 1 - ALPHA);
  const interventionUpper = percentile(bootstrap.intervention, 1 - ALPHA);
  report.bounds = {
    quality_lower: qualityLower,
    quality_method: discordantRepos < 10 ? "conservative_repository_harm_cp" : "paired_hierarchical_bootstrap",
    discordant_repositories: discordantRepos,
    harmed_repositories: harmedRepos,
    cost_ratio_upper: costUpper === null ? null : publicNumber(costUpper),
    p95_latency_ratio_upper: latencyUpper,
    mean_intervention_difference_ms_upper: interventionUpper,
    one_sided_alpha: ALPHA,
    bootstrap_resamples: protocol.bootstrap_resamples,
  };
  report.conditional_thresholds.quality = qualityLower > -0.02;
  report.conditional_thresholds.cost = costComplete && Number.isFinite(point.costRatio)
    && Number.isFinite(costUpper) && costUpper < 0.85;
  report.conditional_thresholds.latency = latencyUpper <= 1.10
    && report.point.max_foreground_routing_wait_ms <= 2000;
  report.conditional_thresholds.intervention = authorityComplete
    && rows.every((row) => !row.authority_violation) && interventionUpper <= 60000;
  // Source receipts, the frozen trial registration and separate safety cases are
  // not yet verifiable here. Numerical thresholds never authorize a rollout.
  return report;
}

if (process.argv[1] && import.meta.url === pathToFileURL(process.argv[1]).href) {
  if (process.argv.length !== 4) {
    process.stderr.write("usage: node analyze-trial.mjs BUNDLE.json REPORT.json\n");
    process.exitCode = 2;
  } else {
    try {
      const bundle = JSON.parse(await readFile(process.argv[2], "utf8"));
      const report = analyzeRoutingTrial(bundle);
      await writeFile(process.argv[3], `${JSON.stringify(report, null, 2)}\n`, { flag: "wx" });
      process.stdout.write(`Wrote ${report.analysis_version} report; statistical gates passed: ${report.statistical_gates_passed}\n`);
    } catch (error) {
      process.stderr.write(`${error.message}\n`);
      process.exitCode = 1;
    }
  }
}
