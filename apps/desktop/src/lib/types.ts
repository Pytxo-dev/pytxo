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
  apply_status: string | null;
  applied_at: string | null;
  prepared_digest: string | null;
  prepared_at: string | null;
  last_apply_error: RunApplyError | null;
  recovery_state: string | null;
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

export type RunApplyManifest = {
  transaction_id: string;
  changes: Array<{
    path: string;
    kind: "add" | "modify" | "delete";
    source_agent_id: string;
    source_task_id: string;
    base_digest: string | null;
    result_digest: string | null;
  }>;
};

export type RunApplyError = {
  at: string;
  code: string;
  message: string;
  attempt_id: string | null;
  rollback_confirmed: boolean;
};

export type RunApplyAttempt = {
  attempt_id: string;
  created_at: string | null;
  phase: string;
  outcome: "committed" | "rolled_back" | "recovery_required" | "interrupted";
  error_code: string | null;
  error_message: string | null;
  rollback_confirmed: boolean;
};

export type PreparedRunSummary = {
  added: number;
  modified: number;
  deleted: number;
  bytes: number;
};

export type PreparedRunFile = {
  path: string;
  kind: "add" | "modify" | "delete";
  before_sha256: string | null;
  after_sha256: string | null;
  byte_count: number;
  task_id: string;
  agent_id: string;
  blob_digest: string | null;
  before_mode: number | null;
  after_mode: number | null;
  before_byte_count: number;
  after_byte_count: number;
  before_is_binary: boolean | null;
  after_is_binary: boolean | null;
  before_chunks: PreparedBlobChunk[];
  after_chunks: PreparedBlobChunk[];
};

export type PreparedBlobChunk = {
  offset: number;
  length: number;
  sha256: string;
};

export type PreparedContentChunkDto = {
  run_id: string;
  package_digest: string;
  path: string;
  side: "before" | "after";
  digest: string;
  byte_count: number;
  binary: boolean;
  offset: number;
  length: number;
  next_offset: number;
  complete: boolean;
  data_base64: string;
};

export type PreparedRunManifest = {
  version: number;
  run_id: string;
  base_revision: string;
  prepared_at: string;
  package_digest: string;
  summary: PreparedRunSummary;
  files: PreparedRunFile[];
};

export type DomainChangeDto = {
  sequence: number;
  entity_kind: string;
  entity_id: string;
  changed_at: string;
};

export type DomainChangesPageDto = {
  changes: DomainChangeDto[];
  next_cursor: number;
  has_more: boolean;
  cursor_gap: boolean;
};

export type DesktopChangedEvent = {
  domain_id: string;
  entity_kind: string;
  entity_id: string;
};

export type EnforcementSurface = {
  status: "enforced" | "advisory" | "unavailable" | "bypassed";
  mechanism: string;
  detail: string;
};

export type PermissionEnforcementReceipt = {
  requested_profile: string;
  effective_profile: string;
  execution_domain: string;
  workspace_isolation: EnforcementSurface;
  host_filesystem_boundary: EnforcementSurface;
  network: EnforcementSurface;
  apply_boundary: EnforcementSurface;
};

export type RunReviewDto = {
  run_id: string;
  base_revision: string | null;
  apply_status: string;
  applied_at: string | null;
  plan: {
    waves: Array<Array<{
      task_id: string;
      agent: string;
      paths: string[];
      depends_on: string[];
      wave: number;
      root: string | null;
      verify: string[];
    }>>;
    warnings: string[];
  };
  enforcement: {
    run: PermissionEnforcementReceipt;
    agents: Record<string, PermissionEnforcementReceipt>;
  };
  apply_manifest: RunApplyManifest | { error: string } | null;
  prepared_manifest: PreparedRunManifest | null;
  prepared_digest: string | null;
  prepared_at: string | null;
  last_apply_error: RunApplyError | null;
  recovery_state: string | null;
  apply_attempts: RunApplyAttempt[];
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
  /** `repo_root` still exists on disk. */
  is_available: boolean;
  /** `repo_root` resolves inside the OS temp directory (stale test artifact). */
  is_temporary: boolean;
};

export type AdeCliStatusDto = {
  id: string;
  display_name: string;
  default_cmd: string;
  installed: boolean;
  auth_state: "signed_in" | "signed_out" | "unknown" | "not_applicable" | "not_installed";
  auth_label: string;
  auth_owner: string;
  login_supported: boolean;
  login_label: string | null;
  docs_url: string;
  detail: string;
};

export type ProviderStatusDto = {
  id: string;
  name: string;
  api_key_env: string;
  key_configured: boolean;
  openai_compatible: boolean;
  builtin: boolean;
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

export type FlowStatus = "draft" | "transcribing" | "planning" | "ready" | "blocked" | "dispatching" | "dispatched" | "failed";
export type FlowDraftInput = { id: string; title: string; mission_text: string; source: "text" | "voice"; domain_id: string | null; project_id: string | null; ade_id: string | null };
export type FlowDraftRecord = { id: string; title: string; mission_text: string; source: string; domain_id: string | null; project_id: string | null; status: string; plan_json: string | null; dispatched_run_id: string | null; created_at: string; updated_at: string };
export type FlowPlan = { draft_id: string; domain_id: string; project_id: string | null; status: "ready" | "blocked"; tasks: { id: string; agent: string; prompt: string; paths: string[]; dependencies: string[]; root: string | null; verify?: string[] }[]; waves: string[][]; permission_profile: string; isolation_mode: string; isolation_backend_intent: string; execution_backend: string; ade: { requested: string | null; available: boolean; installed: string[]; command: string | null }; warnings: { code: string; message: string }[]; blocked_reasons: unknown[]; estimated_tokens: number | null; estimated_cost_usd: number | null; previewed_at: string };
export type VoiceState = "idle" | "recording" | "paused" | "transcribing" | "ready" | "cancelled" | "failed";
export type VoiceSessionDto = { session_id: string; device: string; language: string; state: VoiceState; elapsed_ms: number; buffered_samples: number; confidence: number | null; transcript_segments: { text: string; confidence: number; uncertain: boolean }[]; error: string | null };
export type VoiceProgressEvent =
  | { kind: "audio_level"; session_id: string; level: number }
  | { kind: "capture_state"; session_id: string; state: VoiceState }
  | { kind: "transcription_progress"; session_id: string; progress: number }
  | { kind: "partial_transcript"; session_id: string; text: string; confidence: number };
export type VoiceModel = { id: string; url: string; sha256: string; multilingual: boolean };
