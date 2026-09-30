import { createHash } from "node:crypto";
import { readFile, writeFile } from "node:fs/promises";
import { pathToFileURL } from "node:url";

import { manifestDigest } from "./analyze-trial.mjs";

const FAMILIES = [
  "mechanical", "local_defects", "state_persistence", "concurrency",
  "adapter_protocol", "cross_component",
];
const FAMILY_COUNTS = [67, 67, 67, 67, 66, 66];
const INPUT_KEYS = new Set(["protocol", "randomization_seed", "pins", "cases"]);
const PROTOCOL_KEYS = new Set(["schema_version", "phase", "campaign_id", "analysis_seed", "bootstrap_resamples"]);
const PIN_KEYS = new Set([
  "profile_pair_digest", "adapter_digest", "r0_policy_digest", "rj_policy_digest", "oracle_manifest_digest",
]);
const CASE_KEYS = new Set([
  "case_id", "repository_group_id", "snapshot_digest", "packet_digest", "task_contract_digest", "oracle_digest",
  "family", "planned_advice_opportunity", "split", "billing_mode", "billing_cohort_id",
]);
const FROZEN_KEYS = new Set([
  "schema_version", "protocol", "randomization_seed", "pins", "case_manifest_digest",
  "assignments", "manifest_digest", "schedule", "freeze_digest",
]);
const ASSIGNMENT_KEYS = new Set([
  "assignment_id", "case_id", "repository_group_id", "snapshot_digest", "packet_digest", "family", "split",
  "planned_advice_opportunity", "billing_mode", "billing_cohort_id", "arm", "repetition",
]);
const SCHEDULE_KEYS = new Set(["case_id", "repository_group_id", "assignment_ids"]);

function requireObject(value, name, keys) {
  if (value === null || typeof value !== "object" || Array.isArray(value)) {
    throw new Error(`${name} must be an object`);
  }
  for (const key of Object.keys(value)) {
    if (!keys.has(key)) throw new Error(`${name} contains unexpected field: ${key}`);
  }
}

function requireToken(value, name) {
  if (typeof value !== "string" || !/^[a-zA-Z0-9_-]{1,96}$/.test(value)) {
    throw new Error(`${name} must be an opaque token`);
  }
}

function requireSha256(value, name) {
  if (typeof value !== "string" || !/^[a-f0-9]{64}$/.test(value)) {
    throw new Error(`${name} must be a lowercase SHA-256 digest`);
  }
}

function requireUint32(value, name) {
  if (!Number.isInteger(value) || value < 1 || value > 0xffffffff) {
    throw new Error(`${name} must be a nonzero uint32`);
  }
}

function stableJson(value) {
  if (Array.isArray(value)) return `[${value.map(stableJson).join(",")}]`;
  if (value !== null && typeof value === "object") {
    return `{${Object.keys(value).sort().map((key) => `${JSON.stringify(key)}:${stableJson(value[key])}`).join(",")}}`;
  }
  return JSON.stringify(value);
}

function sha256(value) {
  return createHash("sha256").update(value).digest("hex");
}

function compareOpaque(a, b) {
  return a < b ? -1 : a > b ? 1 : 0;
}

function shuffled(values, random) {
  const result = [...values];
  for (let index = result.length - 1; index > 0; index--) {
    const other = Math.floor(random() * (index + 1));
    [result[index], result[other]] = [result[other], result[index]];
  }
  return result;
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

function validateInput(input) {
  requireObject(input, "campaign", INPUT_KEYS);
  requireObject(input.protocol, "protocol", PROTOCOL_KEYS);
  const protocol = input.protocol;
  if (protocol.schema_version !== 1 || protocol.phase !== "confirmation" || protocol.bootstrap_resamples !== 10000) {
    throw new Error("confirmation requires schema v1 and 10,000 bootstrap resamples");
  }
  requireToken(protocol.campaign_id, "campaign_id");
  requireUint32(protocol.analysis_seed, "analysis_seed");
  requireUint32(input.randomization_seed, "randomization_seed");
  requireObject(input.pins, "pins", PIN_KEYS);
  for (const key of PIN_KEYS) requireSha256(input.pins[key], key);
  if (!Array.isArray(input.cases) || input.cases.length !== 400) {
    throw new Error("confirmation requires 400 cases");
  }
  const caseIds = new Set();
  const taskDigests = new Set();
  const packetDigests = new Set();
  const opportunityPacketDigests = new Set();
  const snapshotRepositories = new Map();
  const byRepository = new Map();
  const familyCounts = new Map(FAMILIES.map((family) => [family, 0]));
  let billingCohort = null;
  for (const row of input.cases) {
    requireObject(row, "case", CASE_KEYS);
    for (const key of ["case_id", "repository_group_id", "billing_cohort_id"]) requireToken(row[key], key);
    for (const key of ["snapshot_digest", "packet_digest", "task_contract_digest", "oracle_digest"]) requireSha256(row[key], key);
    if (caseIds.has(row.case_id)) throw new Error("duplicate case identity");
    if (taskDigests.has(row.task_contract_digest)) throw new Error("duplicate task contract across cases");
    caseIds.add(row.case_id);
    taskDigests.add(row.task_contract_digest);
    packetDigests.add(row.packet_digest);
    if (typeof row.planned_advice_opportunity !== "boolean") {
      throw new Error("planned_advice_opportunity must be a boolean reviewed before outcomes");
    }
    if (row.planned_advice_opportunity) opportunityPacketDigests.add(row.packet_digest);
    const owner = snapshotRepositories.get(row.snapshot_digest);
    if (owner !== undefined && owner !== row.repository_group_id) {
      throw new Error("snapshot reused across repositories");
    }
    snapshotRepositories.set(row.snapshot_digest, row.repository_group_id);
    if (row.split !== "holdout") throw new Error("confirmation case must be untouched holdout");
    if (!familyCounts.has(row.family)) throw new Error("unknown task family");
    familyCounts.set(row.family, familyCounts.get(row.family) + 1);
    if (row.billing_mode !== "api" || (billingCohort !== null && row.billing_cohort_id !== billingCohort)) {
      throw new Error("confirmation dollar hypothesis requires one API billing cohort");
    }
    billingCohort = row.billing_cohort_id;
    if (!byRepository.has(row.repository_group_id)) byRepository.set(row.repository_group_id, []);
    byRepository.get(row.repository_group_id).push(row);
  }
  if (byRepository.size !== 200 || [...byRepository.values()].some((rows) => rows.length !== 2)) {
    throw new Error("confirmation requires 200 repositories with two cases each");
  }
  if (FAMILIES.some((family, index) => familyCounts.get(family) !== FAMILY_COUNTS[index])) {
    throw new Error("confirmation family balance differs from frozen protocol");
  }
  // A uniform packet makes Jev's response a constant choice, reproducible by R0.
  // Planned opportunity is an independently reviewed pretrial claim, not proof
  // of runtime Core eligibility. Diversity here is necessary, not sufficient.
  if (packetDigests.size < 2) throw new Error("confirmation has no packet diversity");
  if (opportunityPacketDigests.size < 2) {
    throw new Error("confirmation has no opportunity packet diversity");
  }
  return byRepository;
}

function buildSchedule(byRepository, byTuple, seed) {
  const random = randomGenerator(seed);
  const repositories = shuffled([...byRepository.entries()].sort(([a], [b]) => compareOpaque(a, b)), random);
  const schedule = [];
  for (const [repositoryId, repositoryCases] of repositories) {
    const orderedCases = shuffled([...repositoryCases].sort((a, b) => compareOpaque(a.case_id, b.case_id)), random);
    for (const [caseIndex, row] of orderedCases.entries()) {
      const sequence = caseIndex === 0
        ? [["R0", 1], ["RJ", 1], ["RJ", 2], ["R0", 2]]
        : [["RJ", 1], ["R0", 1], ["R0", 2], ["RJ", 2]];
      schedule.push({
        case_id: row.case_id,
        repository_group_id: repositoryId,
        assignment_ids: sequence.map(([arm, repetition]) => byTuple.get(`${row.case_id}/${arm}/${repetition}`)),
      });
    }
  }
  return schedule;
}

/** Verify schema, identities, pairing, order and digests; archival registration is external. */
export function verifyFrozenConfirmation(frozen) {
  requireObject(frozen, "frozen confirmation", FROZEN_KEYS);
  if (frozen.schema_version !== 2) throw new Error("unsupported frozen confirmation schema");
  requireObject(frozen.protocol, "protocol", PROTOCOL_KEYS);
  const protocol = frozen.protocol;
  if (protocol.schema_version !== 1 || protocol.phase !== "confirmation" || protocol.bootstrap_resamples !== 10000) {
    throw new Error("confirmation requires schema v1 and 10,000 bootstrap resamples");
  }
  requireToken(protocol.campaign_id, "campaign_id");
  requireUint32(protocol.analysis_seed, "analysis_seed");
  requireUint32(frozen.randomization_seed, "randomization_seed");
  requireObject(frozen.pins, "pins", PIN_KEYS);
  for (const key of PIN_KEYS) requireSha256(frozen.pins[key], key);
  for (const key of ["case_manifest_digest", "manifest_digest", "freeze_digest"]) requireSha256(frozen[key], key);
  if (!Array.isArray(frozen.assignments) || frozen.assignments.length !== 1600) {
    throw new Error("confirmation requires 1,600 assignments");
  }
  const byRepository = new Map();
  const byCase = new Map();
  const byTuple = new Map();
  const assignmentIds = new Set();
  const snapshotRepositories = new Map();
  const packetDigests = new Set();
  const opportunityPacketDigests = new Set();
  const familyCounts = new Map(FAMILIES.map((family) => [family, 0]));
  let cohort = null;
  for (const assignment of frozen.assignments) {
    requireObject(assignment, "assignment", ASSIGNMENT_KEYS);
    for (const key of ["assignment_id", "case_id", "repository_group_id", "billing_cohort_id"]) {
      requireToken(assignment[key], key);
    }
    requireSha256(assignment.snapshot_digest, "snapshot_digest");
    requireSha256(assignment.packet_digest, "packet_digest");
    packetDigests.add(assignment.packet_digest);
    if (typeof assignment.planned_advice_opportunity !== "boolean") {
      throw new Error("planned_advice_opportunity must be a frozen boolean");
    }
    if (!familyCounts.has(assignment.family) || assignment.split !== "holdout"
      || assignment.billing_mode !== "api" || !["R0", "RJ"].includes(assignment.arm)
      || ![1, 2].includes(assignment.repetition)) {
      throw new Error("invalid confirmation assignment");
    }
    if (cohort !== null && cohort !== assignment.billing_cohort_id) throw new Error("mixed billing cohort");
    cohort = assignment.billing_cohort_id;
    const expectedId = `a_${sha256(stableJson([
      protocol.campaign_id, assignment.case_id, assignment.arm, assignment.repetition,
    ])).slice(0, 32)}`;
    if (assignment.assignment_id !== expectedId || assignmentIds.has(expectedId)) {
      throw new Error("assignment identity mismatch or duplicate");
    }
    assignmentIds.add(expectedId);
    const tuple = `${assignment.case_id}/${assignment.arm}/${assignment.repetition}`;
    if (byTuple.has(tuple)) throw new Error("duplicate assignment tuple");
    byTuple.set(tuple, expectedId);
    const facts = stableJson({
      repository_group_id: assignment.repository_group_id,
      snapshot_digest: assignment.snapshot_digest,
      packet_digest: assignment.packet_digest,
      family: assignment.family,
      planned_advice_opportunity: assignment.planned_advice_opportunity,
      billing_cohort_id: assignment.billing_cohort_id,
    });
    if (byCase.has(assignment.case_id) && byCase.get(assignment.case_id) !== facts) {
      throw new Error("case facts drift between assignments");
    }
    if (!byCase.has(assignment.case_id)) {
      byCase.set(assignment.case_id, facts);
      if (assignment.planned_advice_opportunity) opportunityPacketDigests.add(assignment.packet_digest);
      familyCounts.set(assignment.family, familyCounts.get(assignment.family) + 1);
      if (!byRepository.has(assignment.repository_group_id)) byRepository.set(assignment.repository_group_id, []);
      byRepository.get(assignment.repository_group_id).push({ case_id: assignment.case_id });
    }
    const owner = snapshotRepositories.get(assignment.snapshot_digest);
    if (owner !== undefined && owner !== assignment.repository_group_id) {
      throw new Error("snapshot reused across repositories");
    }
    snapshotRepositories.set(assignment.snapshot_digest, assignment.repository_group_id);
  }
  if (byCase.size !== 400 || byRepository.size !== 200
    || [...byRepository.values()].some((rows) => rows.length !== 2)
    || FAMILIES.some((family, index) => familyCounts.get(family) !== FAMILY_COUNTS[index])) {
    throw new Error("confirmation case or family balance mismatch");
  }
  if (packetDigests.size < 2) throw new Error("confirmation has no packet diversity");
  if (opportunityPacketDigests.size < 2) throw new Error("confirmation has no opportunity packet diversity");
  for (const caseId of byCase.keys()) {
    for (const arm of ["R0", "RJ"]) {
      for (const repetition of [1, 2]) {
        if (!byTuple.has(`${caseId}/${arm}/${repetition}`)) throw new Error("unpaired assignment");
      }
    }
  }
  if (stableJson(frozen.assignments) !== stableJson([...frozen.assignments].sort((a, b) => compareOpaque(a.assignment_id, b.assignment_id)))) {
    throw new Error("assignments are not in frozen order");
  }
  if (frozen.manifest_digest !== manifestDigest(protocol, frozen.assignments)) {
    throw new Error("manifest digest mismatch");
  }
  if (!Array.isArray(frozen.schedule) || frozen.schedule.length !== 400) throw new Error("invalid schedule");
  for (const row of frozen.schedule) {
    requireObject(row, "schedule", SCHEDULE_KEYS);
    if (!Array.isArray(row.assignment_ids) || row.assignment_ids.length !== 4) throw new Error("invalid schedule block");
  }
  if (stableJson(frozen.schedule) !== stableJson(buildSchedule(byRepository, byTuple, frozen.randomization_seed))) {
    throw new Error("schedule differs from frozen randomization");
  }
  const { freeze_digest: digest, ...body } = frozen;
  if (digest !== sha256(stableJson(body))) throw new Error("freeze digest mismatch");
  return true;
}

/** Freeze only assignment identities and order. No goal, oracle bytes, credentials or worker calls. */
export function freezeConfirmationCampaign(input) {
  const byRepository = validateInput(input);
  const cases = [...input.cases].sort((a, b) => compareOpaque(a.case_id, b.case_id));
  const caseManifestDigest = sha256(stableJson(cases));
  const assignments = [];
  const byTuple = new Map();
  for (const row of cases) {
    for (const arm of ["R0", "RJ"]) {
      for (const repetition of [1, 2]) {
        const assignment = {
          assignment_id: `a_${sha256(stableJson([input.protocol.campaign_id, row.case_id, arm, repetition])).slice(0, 32)}`,
          case_id: row.case_id,
          repository_group_id: row.repository_group_id,
          snapshot_digest: row.snapshot_digest,
          packet_digest: row.packet_digest,
          family: row.family,
          planned_advice_opportunity: row.planned_advice_opportunity,
          split: row.split,
          billing_mode: row.billing_mode,
          billing_cohort_id: row.billing_cohort_id,
          arm,
          repetition,
        };
        assignments.push(assignment);
        byTuple.set(`${row.case_id}/${arm}/${repetition}`, assignment.assignment_id);
      }
    }
  }
  assignments.sort((a, b) => compareOpaque(a.assignment_id, b.assignment_id));
  if (new Set(assignments.map((row) => row.assignment_id)).size !== assignments.length) {
    throw new Error("assignment identity collision");
  }
  const schedule = buildSchedule(byRepository, byTuple, input.randomization_seed);
  const frozen = {
    schema_version: 2,
    protocol: { ...input.protocol },
    randomization_seed: input.randomization_seed,
    pins: { ...input.pins },
    case_manifest_digest: caseManifestDigest,
    assignments,
    manifest_digest: manifestDigest(input.protocol, assignments),
    schedule,
  };
  return { ...frozen, freeze_digest: sha256(stableJson(frozen)) };
}

if (process.argv[1] && import.meta.url === pathToFileURL(process.argv[1]).href) {
  try {
    if (process.argv.length === 4 && process.argv[2] === "--verify") {
      verifyFrozenConfirmation(JSON.parse(await readFile(process.argv[3], "utf8")));
    } else if (process.argv.length === 4) {
      const input = JSON.parse(await readFile(process.argv[2], "utf8"));
      const frozen = freezeConfirmationCampaign(input);
      await writeFile(process.argv[3], `${JSON.stringify(frozen, null, 2)}\n`, { flag: "wx" });
    } else {
      throw new Error("usage: node freeze-confirmation.mjs INPUT.json OUTPUT.json | --verify FROZEN.json");
    }
  } catch (error) {
    process.stderr.write(`${error.message}\n`);
    process.exitCode = 1;
  }
}
