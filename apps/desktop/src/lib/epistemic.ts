import type { AgentDto, EnforcementSurface, RunApplyAttempt, RunDto } from "./types";

/**
 * What the system can prove about a claim. Mirrors the four-value vocabulary the
 * Rust layer already models in `EnforcementSurface.status` and
 * `RunApplyAttempt.outcome`, so the UI can render orchestration evidence without
 * widening or narrowing it.
 */
export type EpistemicTone = "verified" | "claimed" | "unknown" | "refuted";

/** In-flight lifecycle positions. No truth claim is available for these yet. */
export type LifecycleTone = "queued" | "active";

export type StateTone = EpistemicTone | LifecycleTone;

export type StateDescriptor = {
  tone: StateTone;
  label: string;
  /** Plain-language evidence statement. Never asserts more than the field supports. */
  detail: string;
};

/**
 * Agent status values written by the orchestrator: `running` on insert
 * (`pytxo-store::insert_agent_with_root`), then `completed`, `verify_failed`, or
 * `failed` on `finish_agent`. Unrecognised values resolve to `unknown` rather
 * than being pattern-matched into a state the orchestrator did not report.
 */
export function agentState(agent: AgentDto): StateDescriptor {
  const exit = agent.exit_code;
  const exitDetail = exit === null ? null : `exited ${exit}`;

  switch (agent.status) {
    case "running":
      return { tone: "active", label: "Running", detail: "Process active, no result reported yet" };
    case "queued":
    case "pending":
      return { tone: "queued", label: "Queued", detail: "Not started yet" };
    case "blocked":
      return { tone: "queued", label: "Blocked", detail: "Waiting for an earlier wave to finish" };
    case "completed":
      return {
        tone: "verified",
        label: "Completed",
        detail: exitDetail ? `Verify passed, ${exitDetail}` : "Reported complete without an exit code",
      };
    case "verify_failed":
      return {
        tone: "refuted",
        label: "Verify failed",
        detail: exitDetail ? `Task ran, verify command failed, ${exitDetail}` : "Task ran, verify command failed",
      };
    case "failed":
      return {
        tone: "refuted",
        label: "Failed",
        detail: exitDetail ?? "Failed without reporting an exit code",
      };
    case "stopped":
    case "cancelled":
      return { tone: "unknown", label: "Stopped", detail: "Stopped before reporting a result" };
    default:
      return {
        tone: "unknown",
        label: agent.status || "Unreported",
        detail: "Orchestrator reported no recognised status",
      };
  }
}

/** `EnforcementSurface.status` → epistemic tone. */
export function surfaceTone(status: EnforcementSurface["status"]): EpistemicTone {
  switch (status) {
    case "enforced":
      return "verified";
    case "advisory":
      return "claimed";
    case "bypassed":
      return "refuted";
    default:
      return "unknown";
  }
}

export const SURFACE_LABELS: Record<EnforcementSurface["status"], string> = {
  enforced: "Enforced",
  advisory: "Advisory only",
  unavailable: "Not reported",
  bypassed: "Bypassed",
};

/** `RunApplyAttempt.outcome` → epistemic tone. */
export function attemptTone(outcome: RunApplyAttempt["outcome"]): EpistemicTone {
  switch (outcome) {
    case "committed":
      return "verified";
    case "rolled_back":
    case "recovery_required":
      return "refuted";
    default:
      return "unknown";
  }
}

export const ATTEMPT_LABELS: Record<RunApplyAttempt["outcome"], string> = {
  committed: "Committed",
  rolled_back: "Rolled back",
  recovery_required: "Recovery required",
  interrupted: "Interrupted",
};

/**
 * An attempt that stopped without a confirmed rollback may have left the working
 * tree partially modified. This is the one condition that must never be
 * presented as a clean failure.
 */
export function isPartiallyApplied(attempt: RunApplyAttempt): boolean {
  return attempt.outcome !== "committed" && !attempt.rollback_confirmed;
}

const TONE_SEVERITY: Record<StateTone, number> = {
  refuted: 4,
  unknown: 3,
  claimed: 2,
  active: 1,
  queued: 1,
  verified: 0,
};

/**
 * Summarising several claims takes the least-proven of them, so a panel headline
 * can never read stronger than its weakest underlying surface.
 */
export function worstTone<T extends StateTone>(tones: readonly T[], fallback: T): T {
  return tones.reduce((worst, tone) => (TONE_SEVERITY[tone] > TONE_SEVERITY[worst] ? tone : worst), fallback);
}

/**
 * Wave completion is the only progress ratio the orchestrator actually supports:
 * agents carry a wave index and a terminal status, so a wave is complete when
 * every agent in it has stopped. Returns `null` when there is nothing to count,
 * so callers render a state instead of a fabricated quantity.
 */
export function waveProgress(agents: readonly AgentDto[]): { completed: number; total: number } | null {
  if (!agents.length) return null;
  const waves = new Map<number, AgentDto[]>();
  for (const agent of agents) {
    const bucket = waves.get(agent.wave);
    if (bucket) bucket.push(agent);
    else waves.set(agent.wave, [agent]);
  }
  let completed = 0;
  for (const bucket of waves.values()) {
    if (bucket.every((agent) => isTerminalAgent(agent))) completed += 1;
  }
  return { completed, total: waves.size };
}

export function isTerminalAgent(agent: AgentDto): boolean {
  const tone = agentState(agent).tone;
  return tone === "verified" || tone === "refuted" || tone === "unknown";
}

/** Run-level status values from `pytxo-store::is_terminal_run_status` plus `stopped`. */
export function runState(run: RunDto): StateDescriptor {
  switch (run.status) {
    case "running":
      return { tone: "active", label: "Running", detail: "Agents are executing" };
    case "completed":
      return { tone: "verified", label: "Completed", detail: "All agents reported a result" };
    case "failed":
      return { tone: "refuted", label: "Failed", detail: "At least one agent or the review step failed" };
    case "cancelled":
    case "stopped":
      return { tone: "unknown", label: "Stopped", detail: "Stopped before every agent reported" };
    default:
      return { tone: "unknown", label: run.status || "Unreported", detail: "No recognised run status" };
  }
}
