import { invoke as tauriInvoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import type {
  AdeCliStatusDto,
  AgentArbitrageDto,
  AgentDto,
  CatalogEntry,
  CatalogEntryStatus,
  DomainDto,
  EventDto,
  HitlDto,
  ProviderStatusDto,
  ProjectDto,
  PytxoIpcError,
  RunDto,
  DesktopChangedEvent,
} from "./types";

export const IPC_VERSION = "1.1.0";

export const AUTH_CHANGED_EVENT = "deck-auth-changed";
export const AUTH_ERROR_EVENT = "deck-auth-error";
export const DEEP_LINK_EVENT = "pytxo-deep-link";
export const DOMAIN_CHANGED_EVENT = "pytxo://domain-changed";

export function normalizeIpcError(cause: unknown): Error {
  if (cause instanceof Error) return cause;
  if (cause && typeof cause === "object") {
    const payload = cause as Record<string, unknown>;
    if (typeof payload.message === "string") {
      const code = typeof payload.code === "string" ? `[${payload.code}] ` : "";
      return new Error(`${code}${payload.message}`);
    }
    if (typeof payload.error === "string") return new Error(payload.error);
    try {
      return new Error(JSON.stringify(payload));
    } catch {
      return new Error("Pytxo Desktop received an unreadable backend error.");
    }
  }
  return new Error(String(cause));
}

function invoke<T>(command: string, args?: Record<string, unknown>): Promise<T> {
  return tauriInvoke<T>(command, args).catch((cause: unknown) => {
    throw normalizeIpcError(cause);
  });
}

function unwrap<T>(result: T | PytxoIpcError): T {
  if (
    result &&
    typeof result === "object" &&
    "code" in result &&
    "message" in result &&
    !("domain_id" in result)
  ) {
    const err = result as PytxoIpcError;
    throw new Error(`[${err.code}] ${err.message}`);
  }
  return result as T;
}

export async function ipcVersion(): Promise<string> {
  return invoke<string>("ipc_version");
}

function inTauriRuntime(): boolean {
  return typeof window !== "undefined" && "__TAURI_INTERNALS__" in window;
}

export function onAuthChanged(callback: () => void) {
  if (!inTauriRuntime()) return Promise.resolve(() => {});
  return listen(AUTH_CHANGED_EVENT, callback);
}

export function onAuthError(callback: (message: string) => void) {
  if (!inTauriRuntime()) return Promise.resolve(() => {});
  return listen<{ message: string }>(AUTH_ERROR_EVENT, (event) => callback(event.payload.message));
}

export function onPytxoDeepLink(callback: (url: string) => void) {
  if (!inTauriRuntime()) return Promise.resolve(() => {});
  return listen<string>(DEEP_LINK_EVENT, (event) => callback(event.payload));
}

export function onDomainChanged(callback: (event: DesktopChangedEvent) => void) {
  if (!inTauriRuntime()) return Promise.resolve(() => {});
  return listen<DesktopChangedEvent>(DOMAIN_CHANGED_EVENT, (event) => callback(event.payload));
}

export const ipc = {
  listDomains: () => invoke<DomainDto[]>("list_domains_cmd").then(unwrap),
  listAllDomains: () =>
    invoke<CatalogEntry[]>("list_all_domains").then(unwrap).catch(() => [] as CatalogEntry[]),
  catalogFingerprint: () => invoke<string>("catalog_fingerprint").then(unwrap),
  /**
   * Deliberately non-catching: this is the primary "is the hypervisor
   * reachable" signal for the Desktop 2 shell. Swallowing failures here would
   * make a genuine backend outage indistinguishable from an honest empty
   * catalog. Callers that want a resilient fallback should catch explicitly.
   */
  listDomainsStatus: () => invoke<CatalogEntryStatus[]>("list_domains_status").then(unwrap),
  loadDesktopSnapshot: (runLimit = 30, fleetLimit = 20, includeAgents = true) =>
    invoke<{
      domains: CatalogEntryStatus[];
      runs: RunDto[];
      agents: AgentDto[];
      approvals: HitlDto[];
      fleets: import("./types").FleetRunDto[];
      diagnostics: import("./desktop-backend").DesktopSnapshotDiagnostic[];
    }>("load_desktop_snapshot", { runLimit, fleetLimit, includeAgents }).then(unwrap),
  forgetDomain: (domainId: string) =>
    invoke<void>("forget_domain", { domainId }).then(unwrap),
  listProjects: () =>
    invoke<ProjectDto[]>("list_projects").then(unwrap).catch(() => [] as ProjectDto[]),
  selectDomain: (domainId: string) =>
    invoke<void>("select_domain", { domainId }).then(unwrap),
  listTrustedDomains: () =>
    invoke<{ domain_id: string; permission_profile: string; trusted_at: string; label: string | null }[]>(
      "list_trusted_domains",
    )
      .then(unwrap)
      .catch(() => []),
  getDomainPermission: (repoRoot: string) =>
    invoke<string | null>("get_domain_permission", { repoRoot }).then(unwrap).catch(() => null),
  setDomainPermission: (repoRoot: string, profile: string) =>
    invoke<string>("set_domain_permission", { repoRoot, profile }).then(unwrap),
  domainIsTrusted: (repoRoot: string) =>
    invoke<boolean>("domain_is_trusted", { repoRoot }).then(unwrap).catch(() => false),
  ensureWorkspace: (domainId: string) =>
    invoke<string>("ensure_workspace", { domainId }).then(unwrap),
  listRuns: (limit: number, domainId: string | null) =>
    invoke<RunDto[]>("list_runs", { limit, domainId }).then(unwrap),
  runReview: (runId: string, domainId: string | null) =>
    invoke<import("./types").RunReviewDto>("run_review", { runId, domainId }).then(unwrap),
  routingRunSummary: (runId: string, domainId: string) =>
    invoke<import("./types").RoutingDisplaySummary | null>("routing_run_summary", { runId, domainId }).then(unwrap),
  runReviewContent: (
    runId: string,
    path: string,
    side: "before" | "after",
    offset: number,
    limit: number,
    domainId: string | null,
  ) => invoke<import("./types").PreparedContentChunkDto>("run_review_content", {
    runId,
    path,
    side,
    offset,
    limit,
    domainId,
  }).then(unwrap),
  refreshRunReview: (runId: string, domainId: string | null) =>
    invoke<import("./types").PreparedRunManifest>("refresh_run_review", { runId, domainId }).then(unwrap),
  discardRunReview: (runId: string, domainId: string | null) =>
    invoke<void>("discard_run_review", { runId, domainId }).then(unwrap),
  reconcileRunRecovery: (runId: string, domainId: string | null) =>
    invoke<{ outcome: string; attempt_id: string | null }>("reconcile_run_recovery", { runId, domainId }).then(unwrap),
  domainChangesBatch: (requests: Array<{ domain_id: string; cursor: number }>, limit = 200) =>
    invoke<import("./types").DomainChangesPageDto[]>("domain_changes_batch", { requests, limit }).then(unwrap),
  domainChanges: (domainId: string, cursor: number, limit = 200) =>
    invoke<import("./types").DomainChangesPageDto>("domain_changes", { domainId, cursor, limit }).then(unwrap),
  listAgents: (runId: string, domainId: string | null) =>
    invoke<AgentDto[]>("list_agents", { runId, domainId }).then(unwrap),
  tailEvents: (agentId: string, tail: number, domainId: string | null) =>
    invoke<EventDto[]>("tail_events", { agentId, tail, domainId }).then(unwrap),
  readAgentEvents: (runId: string, agentId: string, domainId: string, after: number, limit: number) =>
    invoke<EventDto[]>("read_agent_events", { runId, agentId, domainId, after, limit }).then(unwrap),
  pollLogLines: (agentId: string, limit: number, domainId: string | null) =>
    invoke<EventDto[]>("poll_log_lines", { agentId, limit, domainId }).then(unwrap),
  dryRun: (agents: number, domainId: string | null) =>
    invoke<string>("dry_run", { agents, domainId }).then(unwrap),
  dispatchRun: (cmd: string, agents: number, repoRoot?: string) =>
    invoke<string>("dispatch_run_cmd", { cmd, agents, repoRoot }).then(unwrap),
  stopRun: (all = false, domainId: string | null = null, runId: string | null = null) =>
    invoke<void>("stop_run", { all, domainId, runId }).then(unwrap),
  gitDiff: (agentId: string, domainId: string | null) =>
    invoke<string>("git_diff", { agentId, domainId }).then(unwrap),
  applyRunChanges: (runId: string, domainId: string | null, expectedPackageDigest: string) =>
    invoke<import("./types").RunApplyManifest>("apply_run_changes", { runId, domainId, expectedPackageDigest }).then(unwrap),
  listHitl: (domainId: string | null) =>
    invoke<HitlDto[]>("list_hitl", { domainId }).then(unwrap).catch(() => [] as HitlDto[]),
  listHitlAll: () =>
    invoke<HitlDto[]>("list_hitl_all").then(unwrap).catch(() => [] as HitlDto[]),
  projectRoots: (projectId: string) =>
    invoke<import("./types").ProjectRootDto[]>("project_roots_cmd", { projectId })
      .then(unwrap),
  projectCreate: (domainId: string, path: string) =>
    invoke<{ id: string; manifest_path: string }>("project_create_cmd", { domainId, path }).then(unwrap),
  projectAddRoot: (projectId: string, path: string, readOnly: boolean) =>
    invoke<import("./types").ProjectRootDto[]>("project_add_root_cmd", {
      projectId,
      path,
      readOnly,
    }).then(unwrap),
  projectRemoveRoot: (projectId: string, label: string) =>
    invoke<import("./types").ProjectRootDto[]>("project_remove_root_cmd", {
      projectId,
      label,
    }).then(unwrap),
  listFleetRuns: (limit = 10) =>
    invoke<import("./types").FleetRunDto[]>("list_fleet_runs", { limit })
      .then(unwrap)
      .catch(() => []),
  fleetRunStatus: (fleetRunId: string) =>
    invoke<import("./types").FleetRunStatusDto>("fleet_run_status_cmd", { fleetRunId })
      .then(unwrap)
      .then((dto) => dto.nodes)
      .catch(() => [] as import("./types").FleetNodeDto[]),
  structuralGraph: (runId: string, domainId: string | null) =>
    invoke<import("./types").StructuralGraphDto>("structural_graph", { runId, domainId })
      .then(unwrap)
      .catch(() => ({ nodes: [], edges: [] })),
  workspaceStructuralGraph: (domainId: string | null) =>
    invoke<import("./types").StructuralGraphDto>("workspace_structural_graph", { domainId })
      .then(unwrap)
      .catch(() => ({ nodes: [], edges: [] })),
  hitlRespond: (requestId: string, approve: boolean, domainId: string | null) =>
    invoke<boolean>("hitl_respond", { requestId, approve, domainId }).then(unwrap),
  agentArbitrage: (runId: string, domainId: string | null) =>
    invoke<AgentArbitrageDto[]>("agent_arbitrage", { runId, domainId })
      .then(unwrap)
      .catch(() => [] as AgentArbitrageDto[]),
  checkPytxoCli: () => invoke<boolean>("check_pytxo_cli").then(unwrap).catch(() => false),
  listAdeClis: () =>
    invoke<AdeCliStatusDto[]>("list_ade_clis").then(unwrap),
  startAdeLogin: (id: string) =>
    invoke<{ id: string; message: string }>("start_ade_login", { id }).then(unwrap),
  installPytxoCli: () =>
    invoke<{
      phase: string;
      message: string;
      cli_present: boolean;
      path_pending: boolean;
    }>("install_pytxo_cli").then(unwrap),
  installPytxoCliStatus: () =>
    invoke<{
      phase: string;
      message: string;
      cli_present: boolean;
      path_pending: boolean;
    }>("install_pytxo_cli_status").then(unwrap),
  pickWorkspaceFolder: () => invoke<string | null>("pick_workspace_folder"),
  createExampleWorkspace: () => invoke<string>("create_example_workspace").then(unwrap),
  entitlementStatus: (domainId?: string | null) =>
    invoke<{
      tier: string;
      max_agents: number;
      cloud_enabled: boolean;
      wallet_balance_microcredits: number | null;
      permission_ceiling: string | null;
      subscription_portal_url: string | null;
    }>("entitlement_status", { domainId: domainId ?? null })
      .then(unwrap)
      .catch(() => ({
        tier: "core",
        max_agents: 3,
        cloud_enabled: false,
        wallet_balance_microcredits: null,
        permission_ceiling: null,
        subscription_portal_url: null,
      })),
  authStatus: () =>
    invoke<{ signed_in: boolean; session_present: boolean }>("auth_status")
      .then(unwrap)
      .catch(() => ({ signed_in: false, session_present: false })),
  authOpenSignIn: () => invoke<void>("auth_open_sign_in").then(unwrap),
  authClearSession: () => invoke<void>("auth_clear_session").then(unwrap),
  routingAccountStatus: () => invoke<{ bridge_available: boolean; credential_present: boolean; account_id: string | null; expires_at: string | null; remote_status: "absent" | "verified" | "unverified" | "revoked"; recovery_only: boolean }>("routing_account_status").then(unwrap),
  routingAccountConnect: () => invoke<void>("routing_account_connect").then(unwrap),
  routingAccountDisconnect: () => invoke<{ locally_cleared: boolean; remote_revoked: boolean }>("routing_account_disconnect").then(unwrap),
  routingAccountReconnectForRevocation: () => invoke<void>("routing_account_reconnect_for_revocation").then(unwrap),
  flowPreview: (input: import("./types").FlowDraftInput) => invoke<import("./types").FlowPlan>("flow_preview", { input }).then(unwrap),
  flowExperimentalClaudeAvailable: () => invoke<boolean>("flow_experimental_claude_available").then(unwrap),
  flowExperimentalHostedReviewAvailable: () => invoke<boolean>("flow_experimental_hosted_review_available").then(unwrap),
  flowPreviewExperimentalClaude: (input: import("./types").FlowDraftInput, facts: import("./types").ReviewedDemandFacts) => invoke<import("./types").FlowPlan>("flow_preview_experimental_claude", { input, facts }).then(unwrap),
  flowPreviewExperimentalClaudeHosted: (input: import("./types").FlowDraftInput, facts: import("./types").ReviewedDemandFacts) => invoke<import("./types").FlowPlan>("flow_preview_experimental_claude_hosted", { input, facts }).then(unwrap),
  flowSaveReviewedPlan: (plan: import("./types").FlowPlan) => invoke<import("./types").FlowPlan>("flow_save_reviewed_plan", { plan }).then(unwrap),
  flowDispatch: (draftId: string) => invoke<string>("flow_dispatch", { draftId }).then(unwrap),
  flowStopRouted: (draftId: string, runId: string) => invoke<void>("flow_stop_routed", { draftId, runId }).then(unwrap),
  flowHistory: () => invoke<import("./types").FlowDraftRecord[]>("flow_history").then(unwrap),
  flowAdvisorPacketPreview: (draftId: string) => invoke<import("./types").RoutedAdvisorPacketPreview>("flow_advisor_packet_preview", { draftId }).then(unwrap),
  flowProposedHostedPacketPreview: (draftId: string) => invoke<import("./types").ProposedHostedAdvisorPacketPreview>("flow_proposed_hosted_packet_preview", { draftId }).then(unwrap),
  flowReviewedHostedPacketPreview: (draftId: string) => invoke<import("./types").ReviewedHostedAdvisorPacketPreview>("flow_reviewed_hosted_packet_preview", { draftId }).then(unwrap),
  flowHostedConsentStatus: (domainId: string) => invoke<import("./types").RoutedAdvisorConsentStatus>("flow_hosted_consent_status", { domainId }).then(unwrap),
  flowHostedConsentEnable: (draftId: string, domainId: string, requestDigest: string, scopeDigest: string, expectedRevision: number) => invoke<import("./types").RoutedAdvisorConsentStatus>("flow_hosted_consent_enable", { draftId, domainId, requestDigest, scopeDigest, expectedRevision }).then(unwrap),
  flowHostedConsentRevoke: (domainId: string, expectedRevision: number) => invoke<import("./types").RoutedAdvisorConsentStatus>("flow_hosted_consent_revoke", { domainId, expectedRevision }).then(unwrap),
  routingHostedGrantStatus: (domainId: string) => invoke<import("./types").RoutingHostedGrantStatus | null>("routing_hosted_grant_status", { domainId }).then(unwrap),
  routingHostedGrants: () => invoke<import("./types").RoutingHostedGrantStatus[]>("routing_hosted_grants").then(unwrap),
  routingHostedGrantEnable: (domainId: string) => invoke<import("./types").RoutingHostedGrantStatus>("routing_hosted_grant_enable", { domainId }).then(unwrap),
  routingHostedGrantRevoke: (domainId: string, expectedRevision: number) => invoke<import("./types").RoutingHostedGrantStatus>("routing_hosted_grant_revoke", { domainId, expectedRevision }).then(unwrap),
  flowAdvisorConsent: (domainId: string) => invoke<import("./types").RoutedAdvisorConsentStatus>("flow_advisor_consent", { domainId }).then(unwrap),
  flowAdvisorConsentDomains: () => invoke<string[]>("flow_advisor_consent_domains").then(unwrap),
  flowAdvisorConsentEnable: (draftId: string, domainId: string, requestDigest: string, recipientIdentity: string, expectedRevision: number) => invoke<import("./types").RoutedAdvisorConsentStatus>("flow_advisor_consent_enable", { draftId, domainId, requestDigest, recipientIdentity, expectedRevision }).then(unwrap),
  flowAdvisorConsentRevoke: (domainId: string, expectedRevision: number) => invoke<import("./types").RoutedAdvisorConsentStatus>("flow_advisor_consent_revoke", { domainId, expectedRevision }).then(unwrap),
  flowDelete: (draftId: string) => invoke<void>("flow_delete", { draftId }).then(unwrap),
  voiceListDevices: () => invoke<string[]>("voice_list_devices").then(unwrap),
  voiceDefaultModel: () => invoke<import("./types").VoiceModel>("voice_default_model").then(unwrap),
  voiceLocalAvailable: () => invoke<boolean>("voice_local_available").then(unwrap),
  voiceModelStatus: () => invoke<string | null>("voice_model_status").then(unwrap),
  voiceInstallDefaultModel: () => invoke<string>("voice_install_default_model").then(unwrap),
  voiceStartSession: (device: string, language: string) => invoke<import("./types").VoiceSessionDto>("voice_start_session", { device, language }).then(unwrap),
  voicePauseSession: (sessionId: string) => invoke<import("./types").VoiceSessionDto>("voice_pause_session", { sessionId }).then(unwrap),
  voiceResumeSession: (sessionId: string) => invoke<import("./types").VoiceSessionDto>("voice_resume_session", { sessionId }).then(unwrap),
  voiceFinishSession: (sessionId: string) => invoke<import("./types").VoiceSessionDto>("voice_finish_session", { sessionId }).then(unwrap),
  voiceCancelSession: (sessionId: string) => invoke<import("./types").VoiceSessionDto>("voice_cancel_session", { sessionId }).then(unwrap),
  onVoiceProgress: (callback: (event: import("./types").VoiceProgressEvent) => void) =>
    listen<import("./types").VoiceProgressEvent>("pytxo://voice/progress", (event) => callback(event.payload)),
  getCloseToTray: () =>
    invoke<boolean>("get_close_to_tray").catch(() => true),
  setCloseToTray: (enabled: boolean) =>
    invoke<void>("set_close_to_tray", { enabled }),
  setTrayNeedsYou: (count: number) =>
    invoke<void>("set_tray_needs_you", { count }).catch(() => undefined),
  openFlowWindow: () => invoke<void>("open_flow_window").then(unwrap),
  listProviders: () => invoke<ProviderStatusDto[]>("list_providers").then(unwrap),
  onDomainChanged,
};
