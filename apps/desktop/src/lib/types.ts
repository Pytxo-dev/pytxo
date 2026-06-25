export type DomainDto = { domain_id: string; repo_root: string };

export type RunDto = {
  id: string;
  status: string;
  repo_root: string;
  started_at: string;
  estimated_cost_usd: number | null;
  permission_profile: string | null;
  isolation_mode: string;
  isolation_backend: string;
};

export type AgentDto = {
  id: string;
  run_id: string;
  task_id: string;
  wave: number;
  status: string;
  exit_code: number | null;
  root_id: string | null;
};

export type ProjectDto = { id: string; manifest_path: string };

export type EventDto = {
  id: number;
  agent_id: string;
  kind: string;
  payload: string;
  ts: string;
};

export type CatalogEntry = {
  domain_id: string;
  repo_root: string;
  db_path: string;
  project_id: string | null;
  status: string;
  updated_at: string;
};

export type CatalogEntryStatus = CatalogEntry & {
  active_runs: number;
  latest_run_status: string | null;
  latest_started_at: string | null;
  hitl_pending: number;
};

export type HitlDto = {
  id: string;
  agent_key: string;
  action: string;
  reason: string;
  created_at_ms: string;
  domain_id?: string;
};

export type ProjectRootDto = {
  label: string;
  path: string;
  read_only: boolean;
  primary: boolean;
  permission_profile: string | null;
};

export type FleetRunDto = {
  id: string;
  fleet_id: string;
  started_at: string;
  finished_at: string | null;
  status: string;
};

export type FleetNodeDto = {
  node_id: string;
  domain_id: string;
  domain_run_id: string | null;
  wave: number;
  status: string;
};

export type FleetRunStatusDto = {
  id: string;
  fleet_id: string;
  started_at: string;
  finished_at: string | null;
  status: string;
  nodes: FleetNodeDto[];
};

/** Schema version for structural graph IPC (3 = symbol-node styling). */
export const STRUCTURAL_GRAPH_VERSION = 3;

export type StructuralGraphDto = {
  version?: number;
  nodes: { id: string; label: string; edited: boolean; root_id: string | null }[];
  edges: { from: string; to: string }[];
};

export type AgentArbitrageDto = {
  agent_id: string;
  saved_tokens: number;
  edited_paths: number;
  fallback_paths: number;
};

export type PytxoIpcError = {
  code: string;
  message: string;
};
