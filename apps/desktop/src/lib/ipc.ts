import { invoke } from "@tauri-apps/api/core";
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
  ProjectDto,
  PytxoIpcError,
  RunDto,
} from "./types";

export const IPC_VERSION = "0.10.0";

export const AUTH_CHANGED_EVENT = "deck-auth-changed";
export const AUTH_ERROR_EVENT = "deck-auth-error";
export const DEEP_LINK_EVENT = "pytxo-deep-link";

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

export function onAuthChanged(callback: () => void) {
  return listen(AUTH_CHANGED_EVENT, callback);
}

export function onAuthError(callback: (message: string) => void) {
  return listen<{ message: string }>(AUTH_ERROR_EVENT, (event) => callback(event.payload.message));
}

export function onPytxoDeepLink(callback: (url: string) => void) {
  return listen<string>(DEEP_LINK_EVENT, (event) => callback(event.payload));
}

export const ipc = {
  listDomains: () => invoke<DomainDto[]>("list_domains_cmd").then(unwrap),
  listAllDomains: () =>
    invoke<CatalogEntry[]>("list_all_domains").then(unwrap).catch(() => [] as CatalogEntry[]),
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
  listAgents: (runId: string, domainId: string | null) =>
    invoke<AgentDto[]>("list_agents", { runId, domainId }).then(unwrap),
  tailEvents: (agentId: string, tail: number, domainId: string | null) =>
    invoke<EventDto[]>("tail_events", { agentId, tail, domainId }).then(unwrap),
  pollLogLines: (agentId: string, limit: number, domainId: string | null) =>
    invoke<EventDto[]>("poll_log_lines", { agentId, limit, domainId }).then(unwrap),
  dryRun: (agents: number, domainId: string | null) =>
    invoke<string>("dry_run", { agents, domainId }).then(unwrap),
  dispatchRun: (cmd: string, agents: number, repoRoot?: string) =>
    invoke<string>("dispatch_run_cmd", { cmd, agents, repoRoot }).then(unwrap),
  stopRun: (all = false) => invoke<void>("stop_run", { all }).then(unwrap),
  gitDiff: (agentId: string, domainId: string | null) =>
    invoke<string>("git_diff", { agentId, domainId }).then(unwrap),
  commitWorkspace: (runId: string, agentId: string, domainId: string | null) =>
    invoke<void>("commit_workspace", { runId, agentId, domainId }).then(unwrap),
  listHitl: (domainId: string | null) =>
    invoke<HitlDto[]>("list_hitl", { domainId }).then(unwrap).catch(() => [] as HitlDto[]),
  listHitlAll: () =>
    invoke<HitlDto[]>("list_hitl_all").then(unwrap).catch(() => [] as HitlDto[]),
  projectRoots: (projectId: string) =>
    invoke<import("./types").ProjectRootDto[]>("project_roots_cmd", { projectId })
      .then(unwrap)
      .catch(() => []),
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
    invoke<AdeCliStatusDto[]>("list_ade_clis").then(unwrap).catch(() => [] as AdeCliStatusDto[]),
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
  flowPreview: (input: import("./types").FlowDraftInput) => invoke<import("./types").FlowPlan>("flow_preview", { input }).then(unwrap),
  flowSaveReviewedPlan: (plan: import("./types").FlowPlan) => invoke<import("./types").FlowPlan>("flow_save_reviewed_plan", { plan }).then(unwrap),
  flowDispatch: (draftId: string) => invoke<string>("flow_dispatch", { draftId }).then(unwrap),
  flowHistory: () => invoke<import("./types").FlowDraftRecord[]>("flow_history").then(unwrap),
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
  openFlowWindow: () => invoke<void>("open_flow_window").then(unwrap),
  listProviders: () =>
    invoke<
      Array<{
        id: string;
        name: string;
        api_key_env: string;
        key_configured: boolean;
        openai_compatible: boolean;
        builtin: boolean;
      }>
    >("list_providers").then(unwrap),
};
