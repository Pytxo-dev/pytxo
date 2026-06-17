import { invoke } from "@tauri-apps/api/core";
import type {
  AgentArbitrageDto,
  AgentDto,
  CatalogEntry,
  DomainDto,
  EventDto,
  HitlDto,
  ProjectDto,
  PytxoIpcError,
  RunDto,
} from "./types";

export const IPC_VERSION = "0.3.3";

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

export const ipc = {
  listDomains: () => invoke<DomainDto[]>("list_domains_cmd").then(unwrap),
  listAllDomains: () =>
    invoke<CatalogEntry[]>("list_all_domains").then(unwrap).catch(() => [] as CatalogEntry[]),
  listProjects: () =>
    invoke<ProjectDto[]>("list_projects").then(unwrap).catch(() => [] as ProjectDto[]),
  selectDomain: (domainId: string) =>
    invoke<void>("select_domain", { domainId }).then(unwrap),
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
  hitlRespond: (requestId: string, approve: boolean, domainId: string | null) =>
    invoke<boolean>("hitl_respond", { requestId, approve, domainId }).then(unwrap),
  agentArbitrage: (runId: string, domainId: string | null) =>
    invoke<AgentArbitrageDto[]>("agent_arbitrage", { runId, domainId })
      .then(unwrap)
      .catch(() => [] as AgentArbitrageDto[]),
  checkPytxoCli: () => invoke<boolean>("check_pytxo_cli").then(unwrap).catch(() => false),
  entitlementStatus: () =>
    invoke<{ tier: string; max_agents: number }>("entitlement_status")
      .then(unwrap)
      .catch(() => ({ tier: "core", max_agents: 3 })),
  authStatus: () =>
    invoke<{ signed_in: boolean; session_present: boolean }>("auth_status")
      .then(unwrap)
      .catch(() => ({ signed_in: false, session_present: false })),
  authOpenSignIn: () => invoke<void>("auth_open_sign_in").then(unwrap),
  authStoreSession: (token: string) =>
    invoke<void>("auth_store_session", { token }).then(unwrap),
  authClearSession: () => invoke<void>("auth_clear_session").then(unwrap),
};
