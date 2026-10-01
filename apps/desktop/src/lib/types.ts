export type DomainDto = { domain_id: string; repo_root: string };

export type RunDto = {
  id: string;
  /** Owning execution domain; run-scoped actions must always use this value. */
  domain_id: string;
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
  /** Durable routed journal revision; null for legacy runs. */
  routing_revision: string | null;
};

export type RoutingDisplayRole = "everyday" | "strong";
export type RoutingDisplayTaskState = "waiting_dependencies" | "ready" | "waiting_input" | "active" | "succeeded" | "failed" | "blocked_dependency" | "cancelled" | "recovery_required";
export type RoutingDisplayAttemptState = "admitted" | "preparing" | "launching" | "running" | "sealing" | "verifying" | "passed" | "failed" | "failed_no_launch" | "cancelled" | "recovery_required";
export type RoutingDisplayReason = "blocked" | "manual" | "required" | "mechanical_everyday" | "strong_default" | "advice_everyday" | "strong_repair";
export type RoutingDisplayBlocker = "disabled" | "cancelled" | "scope_drift" | "failed_prerequisite" | "ownership_unresolved" | "budget_exhausted" | "preferred_unavailable" | "invalid_policy" | "stale_catalog" | "invalid_contract" | "capacity_unavailable" | "conflicting_requirement" | "repair_not_allowed";
export type RoutingDisplaySelection =
  | { kind: "selected"; role: RoutingDisplayRole }
  | { kind: "wait_for_capacity"; role: RoutingDisplayRole }
  | { kind: "blocked"; code: RoutingDisplayBlocker };
export type RoutingDisplayDecision = {
  selection: RoutingDisplaySelection;
  reason: RoutingDisplayReason;
  advice_status: "not_used" | "invalid_or_stale" | "shadow_recorded" | "applied" | "rules_fallback";
};
export type RoutingDisplayAttempt = {
  attempt_id: string;
  agent_id: string;
  ordinal: number;
  predecessor_id: string | null;
  state: RoutingDisplayAttemptState;
  role: RoutingDisplayRole;
  decision: RoutingDisplayDecision;
  profile_id: string;
  harness_id: string;
  billing_mode: "subscription" | "api" | "local" | "managed";
  handoff_referenced: boolean;
  sealed_output_recorded: boolean;
  checks_receipt_recorded: boolean;
  usage_status: "unreported" | "known" | "estimated" | "unknown";
  ownership_released: boolean;
  admitted_at_ms: string;
  updated_at_ms: string;
};
export type RoutingDisplayTask = {
  task_id: string;
  state: RoutingDisplayTaskState;
  dependency_task_ids: string[];
  current_attempt_id: string | null;
  winning_attempt_id: string | null;
  last_decision: RoutingDisplayDecision | null;
  pre_admission: {
    ordinal: number;
    decision: RoutingDisplayDecision;
    outcome:
      | { kind: "pending" }
      | { kind: "not_admitted"; stage: "capacity_unavailable" | "admission_rejected" | "cancelled" | "superseded" | "unknown" };
  } | null;
  attempts: RoutingDisplayAttempt[];
};
export type RoutingDisplaySummary = {
  domain_id: string;
  run_id: string;
  routing_revision: string;
  mode: "disabled" | "rules" | "shadow" | "live";
  cancelled: boolean;
  tasks: RoutingDisplayTask[];
};

export type AgentDto = {
  id: string;
  /** Owning execution domain, retained even when multiple stores share run IDs. */
  domain_id: string;
  run_id: string;
  task_id: string;
  wave: number;
  status: string;
  exit_code: number | null;
  root_id: string | null;
  /** Exact saved registry command match; never inferred from a task alias. */
  launcher?: { id: string; display_name: string } | null;
  /** Persisted agent worktree/review source; not a live cwd or existence check. */
  workspace_path?: string | null;
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
  candidate_verification?: {
    version: number;
    verified_at: string;
    exclusions: string[];
    base_inventory: { path: string; sha256: string; mode: number | null }[];
    candidate_inventory: { path: string; sha256: string; mode: number | null }[];
    checks: { task_id: string; command: string; effective_profile: string; passed: boolean; enforcement: unknown }[];
  } | null;
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
  auth_state: "signed_in" | "signed_out" | "unknown" | "vendor_managed" | "detected_only" | "not_applicable" | "not_installed";
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
  domain_id: string;
  run_id?: string | null;
  agent_id?: string | null;
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
export type FlowDraftInput = { id: string; title: string; mission_text: string; source: "text" | "voice"; domain_id: string | null; project_id: string | null; ade_id: string | null; ade_ids?: string[]; task_ades?: Record<string, string>; max_workers?: number; verification_commands?: string[] };
export type ReviewedDemandFacts = { task_kind: "documentation" | "formatting" | "rename" | "local_transformation" | "diagnosis" | "architecture" | "other"; context_complete: boolean; cross_component_requirement: boolean | null; repeatable_symptom_supplied: boolean | null; specific_cause_hypothesis_supplied: boolean | null };
export type FlowDraftRecord = { id: string; title: string; mission_text: string; source: string; domain_id: string | null; project_id: string | null; status: string; plan_json: string | null; dispatched_run_id: string | null; created_at: string; updated_at: string };
export type RoutedAdvisorPacketPreview = { domain_id: string; run_id: string; task_id: string; reviewed_consent_revision: number; recipient_identity: string; packet_digest: string; request_digest: string; request_body: number[] };
export type ProposedHostedAdvisorPacketPreview = { domain_id: string; run_id: string; task_id: string; source_review_recipient_identity: string; recipient_identity: string; scope_digest: string; packet_digest: string; wire_schema_version: number; decision_kind: string; question_set_version: string; packet_body: number[] };
export type ReviewedHostedAdvisorPacketPreview = { domain_id: string; store_db_file_identity: string; run_id: string; task_id: string; reviewed_consent_revision: number; recipient_identity: string; scope_digest: string; packet_digest: string; request_digest: string; wire_schema_version: number; decision_kind: string; question_set_version: string; packet_body: number[]; recordable_shadow_context: boolean };
export type RoutingHostedGrantStatus = { domain_id: string; workspace_id: string; account_id: string; link_origin: string; recipient_identity: string; scope_digest: string; state: "grant_pending" | "enabled" | "revoke_pending" | "revoked"; remote_revision: number | null };
export type RoutedFlowReviewSummary = { authorization: { run_id: string; limits: { max_attempts: number } }; mission_digest: string };
export type RoutedAdvisorConsentStatus = { domain_id: string; revision: number; enabled: boolean; current_scope: boolean; recipient_identity: string; updated_at_ms: number };
export type FlowPlan = { draft_id: string; domain_id: string; project_id: string | null; status: "ready" | "review_only" | "blocked"; tasks: { id: string; agent: string; prompt: string; paths: string[]; dependencies: string[]; root: string | null; verify?: string[]; ade_id?: string | null }[]; waves: string[][]; max_workers: number; permission_profile: string; isolation_mode: string; isolation_backend_intent: string; execution_backend: string; ade: { requested: string | null; available: boolean; installed: string[]; command: string | null }; warnings: { code: string; message: string }[]; blocked_reasons: unknown[]; estimated_tokens: number | null; estimated_cost_usd: number | null; previewed_at: string; routing?: RoutedFlowReviewSummary | null };
export type VoiceState = "idle" | "recording" | "paused" | "transcribing" | "ready" | "cancelled" | "failed";
export type VoiceSessionDto = { session_id: string; device: string; language: string; state: VoiceState; elapsed_ms: number; buffered_samples: number; confidence: number | null; transcript_segments: { text: string; confidence: number; uncertain: boolean }[]; error: string | null };
export type VoiceProgressEvent =
  | { kind: "audio_level"; session_id: string; level: number }
  | { kind: "capture_state"; session_id: string; state: VoiceState }
  | { kind: "transcription_progress"; session_id: string; progress: number }
  | { kind: "partial_transcript"; session_id: string; text: string; confidence: number };
export type VoiceModel = { id: string; url: string; sha256: string; multilingual: boolean };
