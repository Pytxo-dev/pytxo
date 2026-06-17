export type DomainDto = { domain_id: string; repo_root: string };

export type RunDto = {
  id: string;
  status: string;
  repo_root: string;
  started_at: string;
  estimated_cost_usd: number | null;
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

export type HitlDto = {
  id: string;
  agent_key: string;
  action: string;
  reason: string;
  created_at_ms: string;
};

export type AgentArbitrageDto = {
  agent_id: string;
  saved_tokens: number;
  edited_paths: number;
};

export type PytxoIpcError = {
  code: string;
  message: string;
};
