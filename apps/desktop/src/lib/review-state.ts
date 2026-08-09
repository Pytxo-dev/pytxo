import type { RunApplyError } from "./types";

export type ReviewSurfaceState =
  | "preparing"
  | "ready"
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
        detail: "Affected checkout paths changed after this package was prepared.",
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
