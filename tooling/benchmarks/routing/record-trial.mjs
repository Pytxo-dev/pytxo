import { createHash } from "node:crypto";
import { lstat, readFile, realpath, writeFile } from "node:fs/promises";
import path from "node:path";
import { pathToFileURL } from "node:url";

import { analyzeRoutingTrial, manifestDigest } from "./analyze-trial.mjs";
import { freezeConfirmationCampaign, verifyFrozenConfirmation } from "./freeze-confirmation.mjs";

const COST_COMPONENTS = ["advisor", "worker", "checks", "handoff", "rework", "differing_infrastructure"];
const SHA256 = /^[a-f0-9]{64}$/;
const TOKEN = /^[a-zA-Z0-9_-]{1,96}$/;
const TERMINAL = new Set(["passed", "failed", "failed_no_launch", "cancelled"]);
const PHASES = new Set(["prepared", "sending_may_have_happened", "completed", "uncertain", "late_receipt"]);
const ADVICE_STATUSES = new Set(["not_used", "invalid_or_stale", "shadow_recorded", "applied", "rules_fallback"]);
const ROUTE_REASONS = new Set([
  "blocked", "manual", "required", "mechanical_everyday", "strong_default",
  "advice_everyday", "strong_repair",
]);
const PROVENANCE = new Map([
  ["provider_invoice", "provider_billed"],
  ["provider_usage_pinned_rate", "provider_usage_plus_pinned_rate"],
  ["adapter_report", "adapter_reported"],
  ["estimate", "estimated"],
  ["unavailable", "unknown"],
]);
const PIN_KEYS = ["profile_pair_digest", "adapter_digest", "r0_policy_digest", "rj_policy_digest"];
const RUN_PIN_KEYS = [
  "task_contract_digest", "snapshot_digest", "profile_pair_digest", "policy_digest", "adapter_digest",
];

function object(value, name, keys) {
  if (value === null || typeof value !== "object" || Array.isArray(value)) throw new Error(`${name} must be an object`);
  for (const key of Object.keys(value)) if (!keys.includes(key)) throw new Error(`${name} contains unexpected field: ${key}`);
}

function token(value, name) {
  if (typeof value !== "string" || !TOKEN.test(value)) throw new Error(`${name} must be an opaque token`);
}

function privateIdentity(value, name) {
  if (typeof value !== "string" || value.length === 0 || value.length > 4096
    || value.trim() !== value || value.includes("\0")) {
    throw new Error(`${name} must be a nonempty private identity`);
  }
}

function digest(value, name) {
  if (typeof value !== "string" || !SHA256.test(value)) throw new Error(`${name} must be a lowercase SHA-256 digest`);
}

function integer(value, name, minimum = 0) {
  if (!Number.isSafeInteger(value) || value < minimum) throw new Error(`${name} must be a safe integer >= ${minimum}`);
}

function stableJson(value) {
  if (Array.isArray(value)) return `[${value.map(stableJson).join(",")}]`;
  if (value !== null && typeof value === "object") {
    return `{${Object.keys(value).sort().map((key) => `${JSON.stringify(key)}:${stableJson(value[key])}`).join(",")}}`;
  }
  return JSON.stringify(value);
}

function sha256(bytes) {
  return createHash("sha256").update(bytes).digest("hex");
}

// Matches pytxo-store benchmark_alias(key, "digest", raw 64-hex digest).
export function benchmarkDigestAlias(keyHex, rawDigest) {
  if (typeof keyHex !== "string" || !SHA256.test(keyHex)) throw new Error("alias key must be 32-byte hex");
  digest(rawDigest, "raw benchmark digest");
  const kind = Buffer.from("digest");
  const length = Buffer.alloc(4);
  length.writeUInt32LE(kind.length);
  return sha256(Buffer.concat([
    Buffer.from("pytxo-benchmark-alias-v1\0"), Buffer.from(keyHex, "hex"), length,
    kind, Buffer.from(rawDigest),
  ]));
}

// Matches pytxo-core canonical_digest(v1), then the Store alias above.
function traceAlias(keyHex, value) {
  return benchmarkDigestAlias(keyHex, sha256(stableJson(value)));
}

function validatePins(pins) {
  object(pins, "study pins", PIN_KEYS);
  for (const key of PIN_KEYS) digest(pins[key], key);
}

function equal(a, b) {
  return stableJson(a) === stableJson(b);
}

function validateStudy(study) {
  object(study, "study", [
    "schema_version", "protocol", "assignments", "manifest_digest", "frozen_confirmation",
    "registered_freeze_digest", "pins", "bindings", "study_digest",
  ]);
  if (study.schema_version !== 1) throw new Error("unsupported study schema");
  digest(study.study_digest, "study_digest");
  const { study_digest: recordedDigest, ...body } = study;
  if (recordedDigest !== sha256(stableJson(body))) throw new Error("study digest mismatch");
  if (study.manifest_digest !== manifestDigest(study.protocol, study.assignments)) {
    throw new Error("study assignment manifest digest mismatch");
  }
  const scaffold = {
    protocol: study.protocol, assignments: study.assignments, observations: [],
  };
  if (study.protocol.phase === "confirmation") {
    if (study.pins !== undefined) throw new Error("confirmation pins must come from frozen confirmation");
    verifyFrozenConfirmation(study.frozen_confirmation);
    if (study.registered_freeze_digest !== study.frozen_confirmation.freeze_digest) {
      throw new Error("registered freeze digest mismatch");
    }
    scaffold.frozen_confirmation = study.frozen_confirmation;
    scaffold.registered_freeze_digest = study.registered_freeze_digest;
  } else if (study.frozen_confirmation !== undefined || study.registered_freeze_digest !== undefined) {
    throw new Error("fixture study cannot claim a confirmation freeze");
  } else {
    validatePins(study.pins);
  }
  analyzeRoutingTrial(scaffold); // Existing analyzer validates pairings and cohort facts.
  if (!Array.isArray(study.bindings) || study.bindings.length !== study.assignments.length) {
    throw new Error("one frozen binding is required per assignment");
  }
  const assignments = new Map(study.assignments.map((row) => [row.assignment_id, row]));
  const bound = new Set();
  const aliases = new Set();
  const caseContracts = new Map();
  for (const binding of study.bindings) {
    object(binding, "binding", ["assignment_id", "task_contract_digest", "scope_digest", "task_digest", "packet_digest", "run_pins"]);
    token(binding.assignment_id, "binding assignment_id");
    for (const key of ["task_contract_digest", "scope_digest", "task_digest", "packet_digest"]) digest(binding[key], key);
    object(binding.run_pins, "binding run_pins", RUN_PIN_KEYS);
    for (const key of RUN_PIN_KEYS) digest(binding.run_pins[key], `binding ${key}`);
    if (!assignments.has(binding.assignment_id) || bound.has(binding.assignment_id)) {
      throw new Error("missing, extra or duplicate frozen binding");
    }
    bound.add(binding.assignment_id);
    const caseId = assignments.get(binding.assignment_id).case_id;
    if (caseContracts.has(caseId) && caseContracts.get(caseId) !== binding.task_contract_digest) {
      throw new Error("task contract differs between paired arms");
    }
    caseContracts.set(caseId, binding.task_contract_digest);
    const identity = `${binding.scope_digest}/${binding.task_digest}`;
    if (aliases.has(identity)) throw new Error("trace aliases reused between assignments");
    aliases.add(identity);
  }
  if (!equal(study.bindings, [...study.bindings].sort((a, b) => a.assignment_id.localeCompare(b.assignment_id)))) {
    throw new Error("bindings are not in frozen order");
  }
  return new Map(study.bindings.map((row) => [row.assignment_id, row]));
}

/** Hash a controller's ordered assignment-start receipts against the frozen crossover. */
export function createExecutionOrderLedger(study, sourceBytes) {
  const bindings = validateStudy(study);
  if (study.protocol.phase !== "confirmation") throw new Error("execution-order ledger applies to confirmation only");
  const source = JSON.parse(Buffer.from(sourceBytes).toString("utf8"));
  object(source, "controller execution-order source", ["schema_version", "study_digest", "controller_id", "events"]);
  if (source.schema_version !== 1 || source.study_digest !== study.study_digest) {
    throw new Error("controller execution-order source differs from frozen study");
  }
  token(source.controller_id, "controller_id");
  const schedule = study.frozen_confirmation.schedule.flatMap((block) => block.assignment_ids);
  if (!Array.isArray(source.events) || source.events.length !== schedule.length) {
    throw new Error("controller execution-order source must cover every frozen assignment");
  }
  const receiptDigests = new Set();
  for (const [index, event] of source.events.entries()) {
    object(event, "controller start event", [
      "sequence", "kind", "assignment_id", "scope_digest", "task_digest", "controller_receipt_digest",
    ]);
    integer(event.sequence, "controller start sequence", 1);
    digest(event.controller_receipt_digest, "controller_receipt_digest");
    const binding = bindings.get(event.assignment_id);
    if (event.sequence !== index + 1 || event.kind !== "assignment_started"
      || event.assignment_id !== schedule[index]
      || event.scope_digest !== binding?.scope_digest || event.task_digest !== binding?.task_digest) {
      throw new Error("controller execution order differs from frozen crossover schedule or binding");
    }
    if (receiptDigests.has(event.controller_receipt_digest)) {
      throw new Error("controller start receipt reused across assignments");
    }
    receiptDigests.add(event.controller_receipt_digest);
  }
  const body = {
    schema_version: 1,
    study_digest: study.study_digest,
    freeze_digest: study.registered_freeze_digest,
    controller_id: source.controller_id,
    source_sha256: sha256(sourceBytes),
    assignment_ids: schedule,
  };
  return { ...body, ledger_digest: sha256(stableJson(body)) };
}

/** Freeze private scope/task/key identity before reading any result or receipt. */
export function freezeStudy(input) {
  object(input, "registration", [
    "protocol", "assignments", "frozen_confirmation", "registered_freeze_digest", "private_cases", "pins", "bindings",
  ]);
  let protocol;
  let assignments;
  let frozen;
  let pins;
  let caseContracts = null;
  if (input.frozen_confirmation !== undefined) {
    frozen = input.frozen_confirmation;
    verifyFrozenConfirmation(frozen);
    if (input.protocol !== undefined || input.assignments !== undefined) {
      throw new Error("confirmation assignments must come from the frozen confirmation");
    }
    if (input.pins !== undefined) throw new Error("confirmation pins must come from frozen confirmation");
    if (input.registered_freeze_digest !== frozen.freeze_digest) throw new Error("registered freeze digest mismatch");
    if (!Array.isArray(input.private_cases)) throw new Error("confirmation requires its private case manifest");
    const replay = freezeConfirmationCampaign({
      protocol: frozen.protocol, randomization_seed: frozen.randomization_seed,
      pins: frozen.pins, cases: input.private_cases,
    });
    if (!equal(replay, frozen)) throw new Error("private cases differ from frozen confirmation");
    caseContracts = new Map(input.private_cases.map((row) => [row.case_id, row.task_contract_digest]));
    protocol = frozen.protocol;
    assignments = frozen.assignments;
    pins = frozen.pins;
  } else {
    if (input.private_cases !== undefined || input.registered_freeze_digest !== undefined) {
      throw new Error("fixture cannot claim a confirmation freeze");
    }
    protocol = input.protocol;
    assignments = input.assignments;
    pins = input.pins;
    validatePins(pins);
    if (protocol?.phase !== "fixture") throw new Error("fixture registration requires fixture protocol");
  }
  analyzeRoutingTrial({
    protocol, assignments, observations: [],
    ...(frozen && { frozen_confirmation: frozen, registered_freeze_digest: frozen.freeze_digest }),
  });
  if (!Array.isArray(input.bindings) || input.bindings.length !== assignments.length) {
    throw new Error("one private binding is required per assignment");
  }
  const byId = new Map(assignments.map((row) => [row.assignment_id, row]));
  const usedIds = new Set();
  const usedKeys = new Set();
  const usedScopes = new Set();
  const bindings = input.bindings.map((row) => {
    object(row, "private binding", ["assignment_id", "task_contract_digest", "scope", "task_id", "alias_key_hex"]);
    token(row.assignment_id, "assignment_id");
    if (!byId.has(row.assignment_id) || usedIds.has(row.assignment_id)) {
      throw new Error("missing, extra or duplicate private binding");
    }
    usedIds.add(row.assignment_id);
    digest(row.task_contract_digest, "task_contract_digest");
    if (caseContracts && row.task_contract_digest !== caseContracts.get(byId.get(row.assignment_id).case_id)) {
      throw new Error("binding task contract differs from frozen private case");
    }
    object(row.scope, "scope", ["domain_id", "run_id"]);
    privateIdentity(row.scope.domain_id, "domain_id");
    token(row.scope.run_id, "run_id");
    token(row.task_id, "task_id");
    const scopeIdentity = stableJson(row.scope);
    if (usedScopes.has(scopeIdentity)) throw new Error("benchmark assignments must use distinct run scopes");
    usedScopes.add(scopeIdentity);
    if (typeof row.alias_key_hex !== "string" || !SHA256.test(row.alias_key_hex)
      || row.alias_key_hex === "0".repeat(64) || usedKeys.has(row.alias_key_hex)) {
      throw new Error("each assignment requires a distinct nonzero random 32-byte alias key");
    }
    usedKeys.add(row.alias_key_hex);
    const assignment = byId.get(row.assignment_id);
    return {
      assignment_id: row.assignment_id,
      task_contract_digest: row.task_contract_digest,
      scope_digest: traceAlias(row.alias_key_hex, row.scope),
      task_digest: traceAlias(row.alias_key_hex, row.task_id),
      packet_digest: benchmarkDigestAlias(row.alias_key_hex, assignment.packet_digest),
      run_pins: {
        task_contract_digest: benchmarkDigestAlias(row.alias_key_hex, row.task_contract_digest),
        snapshot_digest: benchmarkDigestAlias(row.alias_key_hex, assignment.snapshot_digest),
        profile_pair_digest: benchmarkDigestAlias(row.alias_key_hex, pins.profile_pair_digest),
        policy_digest: benchmarkDigestAlias(row.alias_key_hex,
          assignment.arm === "R0" ? pins.r0_policy_digest : pins.rj_policy_digest),
        adapter_digest: benchmarkDigestAlias(row.alias_key_hex, pins.adapter_digest),
      },
    };
  }).sort((a, b) => a.assignment_id.localeCompare(b.assignment_id));
  const body = {
    schema_version: 1, protocol, assignments,
    manifest_digest: manifestDigest(protocol, assignments),
    ...(frozen && { frozen_confirmation: frozen, registered_freeze_digest: frozen.freeze_digest }),
    ...(!frozen && { pins }),
    bindings,
  };
  const study = { ...body, study_digest: sha256(stableJson(body)) };
  validateStudy(study);
  return study;
}

function traceFacts(trace, assignment, binding) {
  object(trace, "trace", [
    "schema_version", "scope_digest", "task_digest", "routing_revision", "mode",
    "run_pins", "decision_links_complete", "events", "attempts", "advisor_requests",
  ]);
  if (trace.schema_version !== 3 || trace.scope_digest !== binding.scope_digest
    || trace.task_digest !== binding.task_digest || trace.decision_links_complete !== true) {
    throw new Error("v3 trace identity or decision links differ from frozen binding");
  }
  object(trace.run_pins, "trace run_pins", RUN_PIN_KEYS);
  for (const key of RUN_PIN_KEYS) {
    digest(trace.run_pins[key], `trace ${key}`);
    if (trace.run_pins[key] !== binding.run_pins[key]) {
      throw new Error(`v3 trace ${key} differs from frozen mission pin`);
    }
  }
  if (trace.mode !== (assignment.arm === "R0" ? "rules" : "live")) {
    throw new Error("trace routing mode differs from preassigned arm");
  }
  integer(trace.routing_revision, "routing_revision");
  if (!Array.isArray(trace.attempts) || trace.attempts.length > 2
    || !Array.isArray(trace.events) || !Array.isArray(trace.advisor_requests)) {
    throw new Error("trace attempts, events and advisor requests must be bounded arrays");
  }
  const attempts = new Map();
  const usageReceipts = [];
  for (const attempt of trace.attempts) {
    object(attempt, "trace attempt", [
      "attempt_digest", "ordinal", "predecessor_digest", "state", "role", "billing_mode",
      "handoff_digest", "receipt_aliases", "usage", "ownership_released",
    ]);
    digest(attempt.attempt_digest, "attempt_digest");
    integer(attempt.ordinal, "attempt ordinal", 1);
    if (attempt.ordinal > 2 || attempts.has(attempt.attempt_digest)
      || [...attempts.values()].some((row) => row.ordinal === attempt.ordinal)) {
      throw new Error("duplicate or out-of-range attempt");
    }
    if (!TERMINAL.has(attempt.state) || attempt.ownership_released !== true
      || !["everyday", "strong"].includes(attempt.role) || attempt.billing_mode !== assignment.billing_mode) {
      throw new Error("attempt is unsettled or differs from assigned billing mode");
    }
    if (attempt.predecessor_digest !== null) digest(attempt.predecessor_digest, "predecessor_digest");
    if (attempt.handoff_digest !== null) digest(attempt.handoff_digest, "handoff_digest");
    object(attempt.receipt_aliases, "receipt_aliases", [
      "inputs", "launch_checks", "process_identity", "no_worker_created", "quiescence",
      "sealed_output", "checks", "reconciliation",
    ]);
    for (const value of Object.values(attempt.receipt_aliases)) if (value !== null) digest(value, "receipt alias");
    object(attempt.usage, "attempt usage", ["kind", "nano_usd", "receipt"]);
    if (!["known", "estimated", "unknown", "unreported"].includes(attempt.usage.kind)) {
      throw new Error("unknown attempt usage kind");
    }
    if (["known", "estimated"].includes(attempt.usage.kind)) {
      integer(attempt.usage.nano_usd, "attempt usage nano_usd");
      digest(attempt.usage.receipt, "attempt usage receipt");
      usageReceipts.push(attempt.usage.receipt);
    } else if (Object.keys(attempt.usage).length !== 1) {
      throw new Error("unknown attempt usage cannot claim an amount or receipt");
    }
    attempts.set(attempt.attempt_digest, attempt);
  }
  const ordinals = [...attempts.values()].map((row) => row.ordinal).sort();
  if (ordinals.some((value, index) => value !== index + 1)) throw new Error("attempt ordinals have a gap");
  for (const attempt of attempts.values()) {
    if (attempt.ordinal === 1 && attempt.predecessor_digest !== null) throw new Error("first attempt has predecessor");
    if (attempt.ordinal === 2 && !attempts.has(attempt.predecessor_digest)) {
      throw new Error("second attempt has no recorded predecessor");
    }
  }
  const requests = new Map();
  for (const request of trace.advisor_requests) {
    object(request, "advisor request", ["request_digest", "ordinal", "packet_digest", "phase", "result_digest"]);
    digest(request.request_digest, "advisor request_digest");
    digest(request.packet_digest, "advisor packet_digest");
    if (request.packet_digest !== binding.packet_digest) {
      throw new Error("advisor packet differs from frozen case");
    }
    integer(request.ordinal, "advisor ordinal", 1);
    if (!PHASES.has(request.phase) || requests.has(request.request_digest)) throw new Error("invalid or duplicate advisor request");
    if (request.result_digest !== null) digest(request.result_digest, "advisor result_digest");
    if (request.phase === "completed" && request.result_digest === null) throw new Error("completed advisor send lacks result");
    requests.set(request.request_digest, request);
  }
  if (assignment.arm === "R0" && requests.size !== 0) throw new Error("rules arm contains advisor sends");
  const observed = new Map();
  const consumedCompletedResults = new Set();
  const resolved = new Set();
  const admitted = new Set();
  const terminalTransitions = new Set();
  const settledUsage = new Set();
  let lastSequence = 0;
  let decisions = 0;
  for (const event of trace.events) {
    object(event, "trace event", ["sequence", "kind"]);
    integer(event.sequence, "event sequence", 1);
    if (event.sequence <= lastSequence) throw new Error("trace event sequence is not increasing");
    lastSequence = event.sequence;
    object(event.kind, "trace event kind", [
      "kind", "observation_digest", "ordinal", "decision", "shadow_choice", "advice_request_digest",
      "advice_digest", "outcome", "attempt_digest", "state", "usage",
    ]);
    const kind = event.kind;
    switch (kind.kind) {
      case "decision_observed": {
        digest(kind.observation_digest, "observation_digest");
        integer(kind.ordinal, "decision ordinal", 1);
        if (observed.has(kind.observation_digest)) throw new Error("duplicate decision observation");
        if (kind.advice_request_digest !== null) {
          digest(kind.advice_request_digest, "advice_request_digest");
          if (!requests.has(kind.advice_request_digest)) throw new Error("decision advice lacks journal request");
          if (requests.get(kind.advice_request_digest).ordinal !== kind.ordinal) {
            throw new Error("advisor request ordinal differs from decision ordinal");
          }
          if (kind.advice_digest !== null && requests.get(kind.advice_request_digest).result_digest !== kind.advice_digest) {
            throw new Error("decision advice differs from journal result");
          }
          if (kind.advice_digest !== null && requests.get(kind.advice_request_digest).phase !== "completed") {
            throw new Error("decision advice lacks completed send");
          }
          if (kind.advice_digest !== null) {
            if (consumedCompletedResults.has(kind.advice_request_digest)) {
              throw new Error("completed advisor result reused for distinct observations");
            }
            consumedCompletedResults.add(kind.advice_request_digest);
          }
        } else if (kind.advice_digest !== null) throw new Error("advice result lacks request");
        if (assignment.arm === "R0" && (kind.advice_request_digest !== null || kind.advice_digest !== null)) {
          throw new Error("rules arm has advisor decision");
        }
        object(kind.decision, "decision", ["selection", "reason", "advice_status"]);
        object(kind.decision.selection, "selection", ["kind", "role", "code"]);
        if (!ROUTE_REASONS.has(kind.decision.reason) || !ADVICE_STATUSES.has(kind.decision.advice_status)) {
          throw new Error("decision has unknown reason or advice status");
        }
        if (assignment.arm === "R0" && kind.decision.advice_status !== "not_used") {
          throw new Error("rules arm claims advisor influence");
        }
        if (assignment.arm === "RJ" && (kind.decision.advice_status === "shadow_recorded"
          || kind.shadow_choice !== null)) {
          throw new Error("treatment decision contains shadow-only advice");
        }
        if (kind.decision.advice_status === "not_used" && kind.advice_digest !== null) {
          throw new Error("not_used decision cannot carry an advisor result");
        }
        if (["applied", "rules_fallback"].includes(kind.decision.advice_status)
          && (kind.advice_request_digest === null || kind.advice_digest === null)) {
          throw new Error(`${kind.decision.advice_status} decision requires a completed bound result`);
        }
        if (kind.decision.advice_status === "applied"
          && (kind.ordinal !== 1 || kind.decision.selection.kind !== "selected"
            || kind.decision.selection.role !== "everyday" || kind.decision.reason !== "advice_everyday")) {
          throw new Error("applied decision requires first everyday selection with advice_everyday reason");
        }
        if (kind.decision.reason === "advice_everyday" && kind.decision.advice_status !== "applied") {
          throw new Error("advice_everyday reason requires applied advice");
        }
        if (kind.decision.selection.kind === "selected") {
          if (!["everyday", "strong"].includes(kind.decision.selection.role)) throw new Error("invalid selected role");
        }
        observed.set(kind.observation_digest, kind);
        decisions++;
        break;
      }
      case "decision_resolved": {
        digest(kind.observation_digest, "observation_digest");
        if (!observed.has(kind.observation_digest) || resolved.has(kind.observation_digest)) {
          throw new Error("decision resolution lacks unique observation");
        }
        object(kind.outcome, "decision outcome", ["kind", "attempt_digest", "stage"]);
        if (kind.outcome.kind === "admitted") digest(kind.outcome.attempt_digest, "resolved attempt_digest");
        else if (kind.outcome.kind !== "not_admitted") throw new Error("invalid decision resolution");
        resolved.add(kind.observation_digest);
        break;
      }
      case "blocked":
        object(kind.decision, "blocked decision", ["selection", "reason", "advice_status"]);
        object(kind.decision.selection, "blocked selection", ["kind", "role", "code"]);
        if (kind.decision.selection.kind !== "blocked"
          || kind.decision.advice_status !== "not_used") {
          throw new Error("blocked decision differs from assigned arm");
        }
        decisions++;
        break;
      case "admitted":
        digest(kind.attempt_digest, "admitted attempt_digest");
        if (!attempts.has(kind.attempt_digest) || admitted.has(kind.attempt_digest)) throw new Error("duplicate or unrecorded admission");
        if (kind.observation_digest === null || !observed.has(kind.observation_digest)) {
          throw new Error("admission lacks decision observation");
        }
        admitted.add(kind.attempt_digest);
        break;
      case "transitioned":
        digest(kind.attempt_digest, "event attempt_digest");
        if (!attempts.has(kind.attempt_digest)) throw new Error("event refers to unrecorded attempt");
        if (kind.state === attempts.get(kind.attempt_digest).state) terminalTransitions.add(kind.attempt_digest);
        break;
      case "usage_settled":
        digest(kind.attempt_digest, "event attempt_digest");
        if (!attempts.has(kind.attempt_digest) || settledUsage.has(kind.attempt_digest)
          || !equal(kind.usage, attempts.get(kind.attempt_digest).usage)) {
          throw new Error("usage settlement differs from final attempt usage");
        }
        settledUsage.add(kind.attempt_digest);
        break;
      case "cancelled":
        break;
      default:
        throw new Error(`unknown trace event kind: ${kind.kind}`);
    }
  }
  if (decisions === 0 || admitted.size !== attempts.size || resolved.size !== observed.size
    || terminalTransitions.size !== attempts.size) {
    throw new Error("trace has missing decision, resolution or admission evidence");
  }
  for (const [id, observation] of observed) {
    const outcome = trace.events.find((event) => event.kind.kind === "decision_resolved" && event.kind.observation_digest === id).kind.outcome;
    if (outcome.kind === "admitted") {
      if (!admitted.has(outcome.attempt_digest)
        || !trace.events.some((event) => event.kind.kind === "admitted"
          && event.kind.attempt_digest === outcome.attempt_digest && event.kind.observation_digest === id)) {
        throw new Error("resolved admission differs from attempt link");
      }
      const attempt = attempts.get(outcome.attempt_digest);
      if (observation.decision.selection.kind !== "selected"
        || observation.decision.selection.role !== attempt.role || observation.ordinal !== attempt.ordinal) {
        throw new Error("selected role differs from admitted attempt");
      }
    }
  }
  for (const event of trace.events.filter((entry) => entry.kind.kind === "admitted")) {
    const resolution = trace.events.find((entry) => entry.kind.kind === "decision_resolved"
      && entry.kind.observation_digest === event.kind.observation_digest);
    if (resolution?.kind.outcome.kind !== "admitted"
      || resolution.kind.outcome.attempt_digest !== event.kind.attempt_digest) {
      throw new Error("admitted event lacks matching resolved admission");
    }
  }
  const initialAttempt = [...attempts.values()].find((attempt) => attempt.ordinal === 1);
  const initialAdmissionEvent = initialAttempt && trace.events.find((event) =>
    event.kind.kind === "admitted" && event.kind.attempt_digest === initialAttempt.attempt_digest);
  const initialDecision = initialAdmissionEvent && observed.get(initialAdmissionEvent.kind.observation_digest)?.decision;
  const initialAdmission = initialDecision ? {
    reason: initialDecision.reason,
    role: initialAttempt.role,
    advice_status: initialDecision.advice_status,
  } : null;
  return {
    requests: [...requests.keys()].sort(), usageReceipts: usageReceipts.sort(), attempts: [...attempts.values()],
    routingExposure: {
      advisor_may_send: [...requests.values()].filter((request) => request.phase !== "prepared").length,
      completed_advisor_sends: [...requests.values()].filter((request) => request.phase === "completed").length,
      initial_admission: initialAdmission,
    },
  };
}

function receiptIdentity(receipt, assignment, binding, name, keys) {
  object(receipt, name, ["assignment_id", "scope_digest", "task_digest", ...keys]);
  if (receipt.assignment_id !== assignment.assignment_id || receipt.scope_digest !== binding.scope_digest
    || receipt.task_digest !== binding.task_digest) {
    throw new Error(`${name} identity differs from frozen assignment`);
  }
}

function sameDigests(values, expected, name) {
  if (!Array.isArray(values) || values.some((value) => !SHA256.test(value))
    || new Set(values).size !== values.length || !equal([...values].sort(), expected)) {
    throw new Error(`${name} does not cover exactly the trace records`);
  }
}

async function artifact(root, reference) {
  if (typeof reference !== "string" || reference.length === 0 || path.isAbsolute(reference)) {
    throw new Error("evidence paths must be relative to the private evidence directory");
  }
  const absolute = await realpath(path.resolve(root, reference));
  const relative = path.relative(root, absolute);
  if (relative === ".." || relative.startsWith(`..${path.sep}`) || path.isAbsolute(relative)) {
    throw new Error("evidence path escapes the private evidence directory");
  }
  const bytes = await readFile(absolute);
  return { bytes, digest: sha256(bytes) };
}

async function jsonArtifact(root, reference) {
  const file = await artifact(root, reference);
  return { value: JSON.parse(file.bytes.toString("utf8")), digest: file.digest };
}

async function externalSource(root, reference, expectedDigest, name) {
  digest(expectedDigest, `${name} source digest`);
  const source = await artifact(root, reference);
  if (source.digest !== expectedDigest) throw new Error(`${name} source digest mismatch`);
  return source.digest;
}

/** Read private, separate source receipts and project only analyzer-safe fields. */
export async function assembleStudy(study, index, evidenceDirectory) {
  const bindings = validateStudy(study);
  object(index, "evidence index", ["study_digest", "rows", "execution_order_source", "execution_order_ledger"]);
  if (index.study_digest !== study.study_digest) throw new Error("evidence index study digest mismatch");
  if (!Array.isArray(index.rows) || index.rows.length !== study.assignments.length) {
    throw new Error("one evidence row is required for every frozen assignment");
  }
  const root = await realpath(evidenceDirectory);
  let executionOrder = null;
  if (study.protocol.phase === "confirmation") {
    if (index.execution_order_source === undefined || index.execution_order_ledger === undefined) {
      throw new Error("confirmation requires a controller execution-order source and write-once ledger");
    }
    const [source, ledger] = await Promise.all([
      artifact(root, index.execution_order_source), jsonArtifact(root, index.execution_order_ledger),
    ]);
    const expected = createExecutionOrderLedger(study, source.bytes);
    if (!equal(ledger.value, expected)) throw new Error("execution-order ledger differs from source or frozen schedule");
    executionOrder = { source_sha256: source.digest, ledger_sha256: ledger.digest, ledger_digest: expected.ledger_digest };
  } else if (index.execution_order_source !== undefined || index.execution_order_ledger !== undefined) {
    throw new Error("fixture cannot claim a confirmation execution-order ledger");
  }
  const rows = new Map();
  for (const row of index.rows) {
    object(row, "evidence row", ["assignment_id", "trace", "acceptance", "charges", "timing", "authority"]);
    token(row.assignment_id, "evidence assignment_id");
    if (rows.has(row.assignment_id)) throw new Error("duplicate evidence assignment");
    rows.set(row.assignment_id, row);
  }
  const observations = [];
  const reviewRows = [];
  for (const assignment of study.assignments) {
    const row = rows.get(assignment.assignment_id);
    if (!row) throw new Error("missing evidence for frozen assignment");
    const binding = bindings.get(assignment.assignment_id);
    const [traceFile, acceptanceFile, chargesFile, timingFile, authorityFile] = await Promise.all([
      jsonArtifact(root, row.trace), jsonArtifact(root, row.acceptance), jsonArtifact(root, row.charges),
      jsonArtifact(root, row.timing), jsonArtifact(root, row.authority),
    ]);
    const trace = traceFile.value;
    const traceEvidence = traceFacts(trace, assignment, binding);
    const acceptance = acceptanceFile.value;
    receiptIdentity(acceptance, assignment, binding, "acceptance receipt", [
      "task_contract_digest", "snapshot_digest", "accepted", "outcome_provenance", "reviewer_id",
      "oracle_source", "oracle_source_sha256", "review_source", "review_source_sha256",
    ]);
    if (acceptance.task_contract_digest !== binding.task_contract_digest
      || acceptance.snapshot_digest !== assignment.snapshot_digest
      || typeof acceptance.accepted !== "boolean" || acceptance.outcome_provenance !== "independent_blinded") {
      throw new Error("acceptance is not independently bound to the frozen task and snapshot");
    }
    token(acceptance.reviewer_id, "reviewer_id");
    if (row.acceptance === acceptance.oracle_source || row.acceptance === acceptance.review_source
      || acceptance.oracle_source === acceptance.review_source) {
      throw new Error("acceptance sources must be separate artifacts");
    }
    const oracleHash = await externalSource(root, acceptance.oracle_source, acceptance.oracle_source_sha256, "oracle");
    const reviewHash = await externalSource(root, acceptance.review_source, acceptance.review_source_sha256, "review");
    if (acceptance.accepted && !traceEvidence.attempts.some((attempt) => attempt.state === "passed"
      && attempt.receipt_aliases.sealed_output !== null && attempt.receipt_aliases.checks !== null)) {
      throw new Error("accepted task has no passed attempt with sealed output and checks");
    }
    const charges = chargesFile.value;
    receiptIdentity(charges, assignment, binding, "charge receipt", [
      "billing_mode", "cost_nano_usd", "cost_components_nano_usd", "usage_reconciled",
      "provider_charge_provenance", "advisor_request_digests", "attempt_usage_receipts",
    ]);
    if (charges.billing_mode !== assignment.billing_mode || typeof charges.usage_reconciled !== "boolean") {
      throw new Error("charge receipt billing mode or reconciliation is invalid");
    }
    object(charges.cost_components_nano_usd, "charge components", COST_COMPONENTS);
    const amounts = COST_COMPONENTS.map((key) => {
      if (!Object.hasOwn(charges.cost_components_nano_usd, key)) throw new Error(`missing charge component: ${key}`);
      const amount = charges.cost_components_nano_usd[key];
      if (amount !== null) integer(amount, `charge component ${key}`);
      return amount;
    });
    if (amounts.includes(null)) {
      if (charges.cost_nano_usd !== null) throw new Error("unknown charge component requires unknown total");
      if (charges.usage_reconciled) throw new Error("unknown total cannot claim reconciled usage");
    } else {
      integer(charges.cost_nano_usd, "cost_nano_usd");
      if (amounts.reduce((a, b) => a + b, 0) !== charges.cost_nano_usd
        || !Number.isSafeInteger(charges.cost_nano_usd)) throw new Error("charge component sum differs from total");
    }
    sameDigests(charges.advisor_request_digests, traceEvidence.requests, "advisor charge coverage");
    sameDigests(charges.attempt_usage_receipts, traceEvidence.usageReceipts, "attempt charge coverage");
    object(charges.provider_charge_provenance, "provider charge provenance", ["kind", "source", "source_sha256"]);
    const provenance = charges.provider_charge_provenance;
    if (!PROVENANCE.has(provenance.kind)) throw new Error("unknown provider charge provenance");
    let chargeSourceHash = null;
    if (provenance.kind === "unavailable") {
      if (provenance.source !== null || provenance.source_sha256 !== null || charges.cost_nano_usd !== null
        || charges.usage_reconciled !== false) throw new Error("unavailable provider charge cannot become known cost");
    } else {
      if (row.charges === provenance.source) throw new Error("provider source must be separate from normalized charges");
      chargeSourceHash = await externalSource(root, provenance.source, provenance.source_sha256, "provider charge");
    }
    if (charges.usage_reconciled && (traceEvidence.attempts.some((attempt) => attempt.usage.kind !== "known")
      || trace.advisor_requests.some((request) => request.phase !== "completed"))
      && provenance.kind !== "provider_invoice") {
      throw new Error("unresolved attempt or advisor send requires provider invoice to claim reconciled usage");
    }
    const timing = timingFile.value;
    receiptIdentity(timing, assignment, binding, "timing receipt", [
      "clock_source", "end_to_end_ms", "foreground_routing_wait_ms", "human_intervention_ms",
      "source", "source_sha256",
    ]);
    if (timing.clock_source !== "monotonic") throw new Error("timing must use a monotonic clock");
    integer(timing.end_to_end_ms, "end_to_end_ms", 1);
    integer(timing.foreground_routing_wait_ms, "foreground_routing_wait_ms");
    integer(timing.human_intervention_ms, "human_intervention_ms");
    if (timing.foreground_routing_wait_ms > timing.end_to_end_ms) {
      throw new Error("foreground routing wait exceeds end-to-end time");
    }
    const timingSourceHash = await externalSource(root, timing.source, timing.source_sha256, "timing");
    const authority = authorityFile.value;
    receiptIdentity(authority, assignment, binding, "authority receipt", [
      "source", "source_sha256", "authority_violation", "ownership_reconciled",
    ]);
    if (typeof authority.authority_violation !== "boolean" || authority.ownership_reconciled !== true) {
      throw new Error("authority receipt does not reconcile ownership");
    }
    const authoritySourceHash = await externalSource(root, authority.source, authority.source_sha256, "authority");
    observations.push({
      assignment_id: assignment.assignment_id,
      manifest_digest: study.manifest_digest,
      ...(study.protocol.phase === "confirmation" && { freeze_digest: study.registered_freeze_digest }),
      status: "complete",
      accepted: acceptance.accepted,
      outcome_provenance: acceptance.outcome_provenance,
      cost_nano_usd: charges.cost_nano_usd,
      cost_components_nano_usd: charges.cost_components_nano_usd,
      cost_evidence: PROVENANCE.get(provenance.kind),
      usage_reconciled: charges.usage_reconciled,
      end_to_end_ms: timing.end_to_end_ms,
      foreground_routing_wait_ms: timing.foreground_routing_wait_ms,
      human_intervention_ms: timing.human_intervention_ms,
      authority_violation: authority.authority_violation,
      routing_exposure: traceEvidence.routingExposure,
    });
    reviewRows.push({
      assignment_id: assignment.assignment_id,
      trace_sha256: traceFile.digest, acceptance_sha256: acceptanceFile.digest,
      charges_sha256: chargesFile.digest, timing_sha256: timingFile.digest,
      authority_sha256: authorityFile.digest, oracle_source_sha256: oracleHash,
      review_source_sha256: reviewHash, provider_source_sha256: chargeSourceHash,
      timing_source_sha256: timingSourceHash, authority_source_sha256: authoritySourceHash,
    });
  }
  const bundle = {
    protocol: study.protocol, assignments: study.assignments, observations,
    ...(study.protocol.phase === "confirmation" && {
      frozen_confirmation: study.frozen_confirmation,
      registered_freeze_digest: study.registered_freeze_digest,
    }),
  };
  const review = {
    schema_version: 1, study_digest: study.study_digest,
    analysis_bundle_sha256: sha256(`${JSON.stringify(bundle, null, 2)}\n`),
    receipt_hashes: reviewRows,
    ...(executionOrder && { execution_order: executionOrder }),
    source_authentication: "external_review_required",
  };
  return { bundle, review };
}

if (process.argv[1] && import.meta.url === pathToFileURL(process.argv[1]).href) {
  try {
    if (process.argv[2] === "--freeze" && process.argv.length === 5) {
      const input = JSON.parse(await readFile(process.argv[3], "utf8"));
      const study = freezeStudy(input);
      await writeFile(process.argv[4], `${JSON.stringify(study, null, 2)}\n`, { flag: "wx" });
    } else if (process.argv[2] === "--record-order" && process.argv.length === 6) {
      const study = JSON.parse(await readFile(process.argv[3], "utf8"));
      const source = await readFile(process.argv[4]);
      const ledger = createExecutionOrderLedger(study, source);
      await writeFile(process.argv[5], `${JSON.stringify(ledger, null, 2)}\n`, { flag: "wx" });
    } else if (process.argv[2] === "--assemble" && process.argv.length === 7) {
      const study = JSON.parse(await readFile(process.argv[3], "utf8"));
      const index = JSON.parse(await readFile(process.argv[4], "utf8"));
      if (path.resolve(process.argv[5]) === path.resolve(process.argv[6])) {
        throw new Error("analysis bundle and review ledger paths must differ");
      }
      const { bundle, review } = await assembleStudy(study, index, path.dirname(path.resolve(process.argv[4])));
      for (const output of [process.argv[5], process.argv[6]]) {
        try {
          await lstat(output);
          throw new Error(`output already exists: ${output}`);
        } catch (error) {
          if (error.code !== "ENOENT") throw error;
        }
      }
      await writeFile(process.argv[6], `${JSON.stringify(review, null, 2)}\n`, { flag: "wx" });
      await writeFile(process.argv[5], `${JSON.stringify(bundle, null, 2)}\n`, { flag: "wx" });
    } else {
      throw new Error("usage: node record-trial.mjs --freeze REGISTRATION.json STUDY.json | --record-order STUDY.json CONTROLLER-SOURCE.json ORDER-LEDGER.json | --assemble STUDY.json EVIDENCE-INDEX.json BUNDLE.json REVIEW.json");
    }
  } catch (error) {
    process.stderr.write(`${error.message}\n`);
    process.exitCode = 1;
  }
}
