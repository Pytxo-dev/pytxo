import type { AgentDto, RoutingDisplaySummary, RunDto } from "./types";
import { exactAttemptAgent } from "./execution-topology";

type ApplyEvidence = Pick<RunDto, "apply_status" | "applied_at" | "recovery_state" | "last_apply_error">;

/** Older stores retained rollback metadata after a successful retry. Only a
 * confirmed rollback older than recorded Apply is historical, never an unknown
 * or newer failure. Attempt journals remain separate evidence. */
export function hasCurrentApplyIssue(state: ApplyEvidence): boolean {
  const recovery = state.recovery_state;
  const error = state.last_apply_error;
  const applied = state.apply_status === "applied" && Number.isFinite(Date.parse(state.applied_at ?? ""));
  const resolvedError = !!error && applied && error.rollback_confirmed
    && Date.parse(error.at) < Date.parse(state.applied_at!);
  const resolvedRecovery = applied && recovery === "rolled_back" && resolvedError;
  return !!(error && !resolvedError)
    || !!(recovery && recovery !== "none" && !resolvedRecovery)
    || ["review_failed", "recovery_required"].includes(state.apply_status ?? "");
}

/** Presentation only. No timers or inferred verifier activity confer authority. */
export function workActivity(run: RunDto | null, agents: AgentDto[], waiting: boolean, unavailable: boolean, reviewable: boolean, recoveryIssue = false, routingSummary: RoutingDisplaySummary | null = null) {
  if (!run || unavailable) return { active: false, tone: "unknown", label: "Activity unknown" } as const;
  if (recoveryIssue || hasCurrentApplyIssue(run)) return { active: false, tone: "refuted", label: "Needs attention" } as const;
  if (["stopped", "cancelled"].includes(run.status)) return { active: false, tone: "unknown", label: "Stopped" } as const;
  if (["failed", "failed_startup"].includes(run.status)) return { active: false, tone: "refuted", label: "Needs attention" } as const;
  if (waiting) return { active: false, tone: "settled", label: "Decision waiting" } as const;
  if (run.applied_at) return { active: false, tone: "settled", label: "Apply recorded" } as const;
  const matchingRouting = run.routing_revision != null
    && routingSummary?.run_id === run.id && routingSummary.domain_id === run.domain_id
    && routingSummary.routing_revision === run.routing_revision ? routingSummary : null;
  const runningAgents = agents.filter(a => a.run_id === run.id && a.domain_id === run.domain_id && a.status === "running");
  const activeWorker = run.routing_revision == null
    ? runningAgents.length > 0
    : !!matchingRouting && matchingRouting.tasks.some(task => {
      const attempt = task.attempts.find(candidate => candidate.attempt_id === task.current_attempt_id);
      return task.state === "active" && attempt?.state === "running"
        && exactAttemptAgent(agents, run, task.task_id, attempt.agent_id)?.status === "running";
    });
  if (run.status === "running" && activeWorker) return { active: true, tone: "active", label: "Worker active" } as const;
  if (run.status === "completed") return { active: false, tone: "settled", label: reviewable ? "Ready for review" : "Workers reported back" } as const;
  return { active: false, tone: "unknown", label: run.status === "starting" ? "Preparing work" : "Activity unknown" } as const;
}
