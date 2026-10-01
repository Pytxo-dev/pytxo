import type { AgentDto, RunApplyError, RunDto, RunReviewDto } from "./types";

/** Planned agent labels are not runtime identities: dispatch may reorder tasks. */
/**
 * The CLI whose recorded worker prepared a task's files, when the run put more
 * than one CLI to work. Single-CLI runs and unknown launchers name nobody.
 */
export function recordedCliFor(
  agents: AgentDto[],
  run: Pick<RunDto, "id" | "domain_id">,
  taskId: string,
): string | null {
  const recorded = agents.filter((agent) => agent.domain_id === run.domain_id && agent.run_id === run.id);
  if (new Set(recorded.map((agent) => agent.launcher?.id).filter(Boolean)).size < 2) return null;
  const owners = recorded.filter((agent) => agent.task_id === taskId);
  return owners.length === 1 ? owners[0].launcher?.display_name ?? null : null;
}

export function recordedWorkerLabel(
  agents: AgentDto[],
  run: Pick<RunDto, "id" | "domain_id">,
  taskId: string,
): string {
  const recorded = agents.filter((agent) =>
    agent.domain_id === run.domain_id && agent.run_id === run.id && agent.task_id === taskId,
  );
  if (recorded.length !== 1 || !recorded[0].id) return "Worker not recorded";
  const { id } = recorded[0];
  const prefix = `${run.id}:`;
  return (id.startsWith(prefix) ? id.slice(prefix.length) : id) || "Worker not recorded";
}

export type ReviewSurfaceState =
  | "preparing"
  | "ready"
  | "verification_required"
  | "nothing_to_apply"
  | "stale"
  | "retryable_failure"
  | "applying"
  | "recovered"
  | "applied"
  | "review_failed"
  | "recovery_required"
  | "discarded"
  | "non_flushable"
  | "not_applicable"
  | "unsupported"
  | "unavailable";

export type ReviewPrimaryAction =
  | "apply"
  | "retry"
  | "refresh"
  | "reconcile"
  | null;

export type ReviewContractState = {
  apply_status: string;
  recovery_state: string | null;
  last_apply_error: RunApplyError | null;
  candidate_verified: boolean;
  prepared_file_count: number | null;
};

export type ReviewPresentation = {
  state: ReviewSurfaceState;
  title: string;
  detail: string;
  primaryAction: ReviewPrimaryAction;
  primaryLabel: string | null;
  applyAllowed: boolean;
  discardAllowed: boolean;
  busy: boolean;
};

export function reviewPresentation(contract: ReviewContractState): ReviewPresentation {
  const retryable =
    contract.apply_status === "ready" &&
    contract.last_apply_error?.rollback_confirmed === true;
  if (retryable) {
    return {
      state: contract.recovery_state === "rolled_back" ? "recovered" : "retryable_failure",
      title:
        contract.recovery_state === "rolled_back"
          ? "Recovered and ready to retry"
          : "Apply failed safely",
      detail:
        contract.last_apply_error?.message ??
        "The previous Apply did not complete. Repository changes were rolled back.",
      primaryAction: "retry",
      primaryLabel: "Retry Apply",
      applyAllowed: true,
      discardAllowed: true,
      busy: false,
    };
  }

  switch (contract.apply_status) {
    case "preparing":
      return {
        state: "preparing",
        title: "Preparing immutable review",
        detail: "Pytxo is packaging exact changes and ownership evidence.",
        primaryAction: null,
        primaryLabel: null,
        applyAllowed: false,
        discardAllowed: false,
        busy: true,
      };
    case "ready":
      if (contract.prepared_file_count === 0) {
        return {
          state: "nothing_to_apply",
          title: "Nothing to Apply",
          detail: "The prepared package contains no file changes.",
          primaryAction: null,
          primaryLabel: null,
          applyAllowed: false,
          discardAllowed: true,
          busy: false,
        };
      }
      if (!contract.candidate_verified) {
        return {
          state: "verification_required",
          title: "Verify before Apply",
          detail: "Run the approved checks against the exact combined candidate before Apply.",
          primaryAction: "refresh",
          primaryLabel: "Verify candidate",
          applyAllowed: false,
          discardAllowed: true,
          busy: false,
        };
      }
      return {
        state: "ready",
        title: "Ready to Apply",
        detail: "The prepared package matches the reviewed base revision.",
        primaryAction: "apply",
        primaryLabel: "Apply reviewed changes",
        applyAllowed: true,
        discardAllowed: true,
        busy: false,
      };
    case "stale":
      return {
        state: "stale",
        title: "Review is stale",
        // Freshness covers the whole project, not only the reviewed paths: any
        // added or edited file can change what the recorded checks proved.
        detail: "Project files changed after this package was prepared. Refresh to recheck it against the current files.",
        primaryAction: "refresh",
        primaryLabel: "Refresh review",
        applyAllowed: false,
        discardAllowed: true,
        busy: false,
      };
    case "applying":
      return {
        state: "applying",
        title: "Applying reviewed changes",
        detail: "One atomic Apply is in progress. A second Apply is blocked.",
        primaryAction: null,
        primaryLabel: null,
        applyAllowed: false,
        discardAllowed: false,
        busy: true,
      };
    case "applied":
      return {
        state: "applied",
        title: "Applied successfully",
        detail: "The reviewed package was applied to the primary checkout.",
        primaryAction: null,
        primaryLabel: null,
        applyAllowed: false,
        discardAllowed: false,
        busy: false,
      };
    case "review_failed":
      return {
        state: "review_failed",
        title: "Review preparation failed",
        detail:
          contract.last_apply_error?.message ??
          "The immutable package could not be prepared or validated.",
        primaryAction: "refresh",
        primaryLabel: "Retry preparation",
        applyAllowed: false,
        discardAllowed: true,
        busy: false,
      };
    case "recovery_required":
      return {
        state: "recovery_required",
        title: "Recovery required",
        detail:
          contract.last_apply_error?.message ??
          "Apply is blocked until recovery is reconciled.",
        primaryAction: "reconcile",
        primaryLabel: "Reconcile recovery",
        applyAllowed: false,
        discardAllowed: false,
        busy: false,
      };
    case "discarded":
      return {
        state: "discarded",
        title: "Review discarded",
        detail: "The staged package and retained agent workspaces were removed.",
        primaryAction: null,
        primaryLabel: null,
        applyAllowed: false,
        discardAllowed: false,
        busy: false,
      };
    case "non_flushable":
      return {
        state: "non_flushable",
        title: "Apply unavailable in DeepSpace",
        detail: "This permission profile never flushes isolated changes to the host.",
        primaryAction: null,
        primaryLabel: null,
        applyAllowed: false,
        discardAllowed: false,
        busy: false,
      };
    case "not_applicable":
      return {
        state: "not_applicable",
        title: "Apply not applicable",
        detail: "This run wrote directly to the host under its effective profile.",
        primaryAction: null,
        primaryLabel: null,
        applyAllowed: false,
        discardAllowed: false,
        busy: false,
      };
    case "unsupported":
      return {
        state: "unsupported",
        title: "Review unsupported",
        detail: "This run does not have a single-root immutable review package.",
        primaryAction: null,
        primaryLabel: null,
        applyAllowed: false,
        discardAllowed: false,
        busy: false,
      };
    default:
      return {
        state: "unavailable",
        title: "Review unavailable",
        detail: "No prepared review contract is available for this run.",
        primaryAction: null,
        primaryLabel: null,
        applyAllowed: false,
        discardAllowed: false,
        busy: false,
      };
  }
}

export type DomainCursorPage = {
  changes: Array<{
    sequence: number;
    entity_kind: string;
    entity_id: string;
    changed_at: string;
  }>;
  next_cursor: number;
  has_more: boolean;
  cursor_gap: boolean;
};

export function advanceDomainCursor(
  _current: number | null,
  page: DomainCursorPage,
): {
  cursor: number;
  needsSnapshot: boolean;
  continueCatchUp: boolean;
} {
  return {
    cursor: page.next_cursor,
    needsSnapshot: page.cursor_gap,
    continueCatchUp: page.has_more,
  };
}

/** Presentation availability only; opening a review never authorizes Apply. */
export function canOpenRunReview(run: RunDto | null, providedReview: RunReviewDto | null = null): boolean {
  if (!run) return false;
  const review = providedReview?.run_id === run.id ? providedReview : null;
  return !!run.prepared_digest || !!review?.prepared_digest || !!review?.prepared_manifest ||
    ["ready", "stale", "applied", "applying", "recovery_required", "discarded"].includes(
      (review?.apply_status ?? run.apply_status ?? "").toLowerCase(),
    );
}
