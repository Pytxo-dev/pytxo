/**
 * Fixture data and backend implementation used ONLY outside a real Tauri
 * runtime (browser dev server, Storybook, Playwright). `createDesktopBackend`
 * in `desktop-backend.ts` gates this behind a strict `__TAURI_INTERNALS__`
 * check, so a packaged Tauri build can never silently fall back to this data
 * — a failed IPC call surfaces as a real error instead.
 */
import type { DesktopBackend, DesktopSnapshot } from "./desktop-backend";
import type { AdeCliStatusDto, DesktopChangedEvent, DomainChangeDto, FlowDraftInput, FlowDraftRecord, FlowPlan, PermissionEnforcementReceipt, PreparedContentChunkDto, PreparedRunManifest, ProviderStatusDto, RunApplyError, RunApplyManifest, RunReviewDto, VoiceProgressEvent, VoiceSessionDto } from "./types";

const previewSignalMark = Uint8Array.from({ length: 300 }, (_, index) => index % 251);
previewSignalMark.set([0x00, 0xff, 0x50, 0x4e, 0x47], 0);
previewSignalMark.set([0xde, 0xad, 0xbe, 0xef], previewSignalMark.length - 4);

const previewReviewContent: Record<string, { before?: string | Uint8Array; after?: string | Uint8Array }> = {
  "crates/pytxo-signal/src/lib.rs": {
    before: "pub fn skeleton(source: &str) -> String {\n    source.to_owned()\n}\n",
    after: "pub fn skeleton(source: &str) -> String {\n    parse(source).structural_skeleton()\n}\n",
  },
  "crates/pytxo-signal/tests/skeleton.rs": {
    after: "#[test]\nfn keeps_public_shape() {\n    assert!(skeleton(\"pub fn run() {}\").contains(\"run\"));\n}\n",
  },
  "crates/pytxo-signal/src/legacy.rs": {
    before: "pub fn legacy_raw_context(source: &str) -> String { source.to_owned() }\n",
  },
  "assets/signal-mark.bin": {
    after: previewSignalMark,
  },
};

function previewBytes(value: string | Uint8Array): Uint8Array {
  return typeof value === "string" ? new TextEncoder().encode(value) : value;
}

function bytesToBase64(bytes: Uint8Array): string {
  let binary = "";
  for (const byte of bytes) binary += String.fromCharCode(byte);
  return btoa(binary);
}

export const previewSnapshot: DesktopSnapshot = {
  domains: [
    {
      domain_id: "pytxo",
      repo_root: "C:/dev/pytxo",
      db_path: "~/.pytxo/domains/pytxo.db",
      project_id: null,
      status: "active",
      updated_at: new Date().toISOString(),
      active_runs: 2,
      latest_run_status: "running",
      latest_started_at: new Date().toISOString(),
      hitl_pending: 1,
      is_available: true,
      is_temporary: false,
    },
    {
      domain_id: "signal-lab",
      repo_root: "C:/dev/signal-lab",
      db_path: "~/.pytxo/domains/signal-lab.db",
      project_id: null,
      status: "healthy",
      updated_at: new Date().toISOString(),
      active_runs: 0,
      latest_run_status: "completed",
      latest_started_at: new Date().toISOString(),
      hitl_pending: 1,
      is_available: true,
      is_temporary: false,
    },
  ],
  runs: [
    {
      id: "run-8f2c",
      domain_id: "pytxo",
      status: "running",
      repo_root: "C:/dev/pytxo",
      started_at: new Date().toISOString(),
      estimated_cost_usd: 0.42,
      permission_profile: "orbit",
      isolation_mode: "copy_on_write",
      isolation_backend: "projfs",
      apply_status: "pending",
      applied_at: null,
      prepared_digest: null,
      prepared_at: null,
      last_apply_error: null,
      recovery_state: null,
    },
    {
      id: "run-71ad",
      domain_id: "signal-lab",
      status: "completed",
      repo_root: "C:/dev/signal-lab",
      started_at: new Date(Date.now() - 3_600_000).toISOString(),
      estimated_cost_usd: 0.18,
      permission_profile: "orbit",
      isolation_mode: "copy_on_write",
      isolation_backend: "git_worktree",
      apply_status: "ready",
      applied_at: null,
      prepared_digest: "pkg-71ad-immutable",
      prepared_at: "2026-08-01T02:15:00Z",
      last_apply_error: null,
      recovery_state: null,
    },
  ],
  agents: [
    { id: "architect", domain_id: "pytxo", run_id: "run-8f2c", task_id: "plan", wave: 0, status: "completed", exit_code: 0, root_id: null },
    { id: "desktop", domain_id: "pytxo", run_id: "run-8f2c", task_id: "ui", wave: 1, status: "running", exit_code: null, root_id: null },
    { id: "verification", domain_id: "pytxo", run_id: "run-8f2c", task_id: "tests", wave: 2, status: "queued", exit_code: null, root_id: null },
  ],
  approvals: [
    {
      id: "approval-1",
      agent_key: "run-8f2c:desktop",
      run_id: "run-8f2c",
      agent_id: "desktop",
      action: "blast.flush",
      reason: "commit agent workspace to repo root",
      created_at_ms: String(Date.now()),
      domain_id: "pytxo",
    },
    {
      id: "approval-2",
      agent_key: "run-71ad:agent-1",
      run_id: "run-71ad",
      agent_id: "agent-1",
      action: "net.egress",
      reason: "network fetch detected in agent command",
      created_at_ms: String(Date.now() - 90_000),
      domain_id: "signal-lab",
    },
  ],
  fleets: [],
  diagnostics: [],
  error: null,
};

const previewAdeClis: AdeCliStatusDto[] = [
  { id: "codex", display_name: "OpenAI Codex", default_cmd: "codex exec --sandbox workspace-write", installed: true, auth_state: "signed_in", auth_label: "ChatGPT connected", auth_owner: "Codex", login_supported: true, login_label: "Connect with ChatGPT", docs_url: "https://developers.openai.com/codex/auth", detail: "Codex owns the browser session, token storage, and refresh." },
  { id: "claude", display_name: "Claude Code", default_cmd: "claude -p", installed: true, auth_state: "signed_in", auth_label: "Claude account connected", auth_owner: "Claude Code", login_supported: true, login_label: "Open Claude Code sign-in", docs_url: "https://code.claude.com/docs/en/authentication", detail: "Pytxo opens Claude Code's official sign-in and never receives its token." },
  { id: "cursor", display_name: "Cursor Agent", default_cmd: "cursor-agent -p --trust", installed: true, auth_state: "signed_in", auth_label: "Cursor account connected", auth_owner: "Cursor Agent", login_supported: true, login_label: "Open Cursor sign-in", docs_url: "https://docs.cursor.com/en/cli/reference/authentication", detail: "Cursor Agent keeps its account credential outside Pytxo." },
  { id: "opencode", display_name: "OpenCode", default_cmd: "opencode run", installed: true, auth_state: "signed_out", auth_label: "No OpenCode provider connected", auth_owner: "OpenCode", login_supported: true, login_label: "Connect an OpenCode provider", docs_url: "https://opencode.ai/docs/providers/", detail: "Provider-specific credentials remain owned by OpenCode." },
  { id: "gemini", display_name: "Gemini CLI", default_cmd: "gemini --skip-trust -p", installed: true, auth_state: "unknown", auth_label: "Check authentication in Gemini CLI", auth_owner: "Gemini CLI", login_supported: true, login_label: "Open Gemini authentication", docs_url: "https://geminicli.com/docs/get-started/authentication/", detail: "Gemini CLI owns Google OAuth; Pytxo does not reuse its cached token." },
  { id: "copilot", display_name: "GitHub Copilot CLI", default_cmd: "copilot -p", installed: false, auth_state: "not_installed", auth_label: "Not installed", auth_owner: "Copilot CLI", login_supported: true, login_label: "Open GitHub sign-in", docs_url: "https://docs.github.com/en/copilot/how-tos/copilot-cli/set-up-copilot-cli/authenticate-copilot-cli", detail: "Copilot CLI owns the GitHub device flow and stores its token in the OS keychain." },
  { id: "agy", display_name: "Antigravity", default_cmd: "agy", installed: false, auth_state: "not_installed", auth_label: "Not installed", auth_owner: "Vendor CLI", login_supported: false, login_label: null, docs_url: "https://pytxo.com/docs/reference/providers-byok", detail: "Pytxo detects the executable without reading vendor credential stores." },
  { id: "aider", display_name: "Aider", default_cmd: "aider --message", installed: false, auth_state: "not_installed", auth_label: "Not installed", auth_owner: "Pytxo run policy", login_supported: false, login_label: null, docs_url: "https://aider.chat/docs/config/api-keys.html", detail: "Choose one explicit BYOK credential for the run; unrelated keys stay hidden." },
];

function previewAdeState(): AdeCliStatusDto[] {
  const codexOnly = typeof localStorage !== "undefined" && localStorage.getItem("pytxo-preview-ade-state-v1") === "codex-only";
  if (!codexOnly) return structuredClone(previewAdeClis);
  return previewAdeClis.map((cli) => cli.id === "codex" ? { ...cli } : {
    ...cli,
    installed: false,
    auth_state: "not_installed" as const,
    auth_label: "Not installed",
  });
}

const previewProviders: ProviderStatusDto[] = [
  { id: "deepseek", name: "DeepSeek", api_key_env: "DEEPSEEK_API_KEY", key_configured: false, openai_compatible: true, builtin: true },
  { id: "openrouter", name: "OpenRouter", api_key_env: "OPENROUTER_API_KEY", key_configured: false, openai_compatible: true, builtin: true },
  { id: "openai", name: "OpenAI", api_key_env: "OPENAI_API_KEY", key_configured: false, openai_compatible: true, builtin: true },
  { id: "anthropic", name: "Anthropic", api_key_env: "ANTHROPIC_API_KEY", key_configured: false, openai_compatible: false, builtin: true },
  { id: "gemini", name: "Google Gemini", api_key_env: "GOOGLE_API_KEY", key_configured: false, openai_compatible: false, builtin: true },
  { id: "mistral", name: "Mistral", api_key_env: "MISTRAL_API_KEY", key_configured: false, openai_compatible: true, builtin: true },
];

const previewReceipt: PermissionEnforcementReceipt = {
  requested_profile: "orbit",
  effective_profile: "orbit",
  execution_domain: "C:/dev/signal-lab",
  workspace_isolation: {
    status: "enforced",
    mechanism: "git-worktree",
    detail: "Agent mutations remain outside the primary repository until run-level Apply.",
  },
  host_filesystem_boundary: {
    status: "advisory",
    mechanism: "child-cwd-and-policy-gates",
    detail: "Workspace cwd and Pytxo-mediated path gates are active; syscall sandboxing is not.",
  },
  network: {
    status: "advisory",
    mechanism: "spawn-command-policy",
    detail: "Known network commands are denied; arbitrary child socket syscalls are not intercepted.",
  },
  apply_boundary: {
    status: "enforced",
    mechanism: "reviewed-run-atomic-apply",
    detail: "All claimed run changes are validated and in-process write failures roll back as one guarded operation.",
  },
};

/**
 * Preview-only fixture that exercises every `EnforcementSurface.status` and
 * `RunApplyAttempt.outcome` value in one screen. Real runs rarely produce all
 * eight at once, but each has to render a distinct, colour-independent
 * affordance, so the state matrix is asserted against this fixture rather than
 * against whatever a live run happens to report.
 */
const STATE_MATRIX_KEY = "pytxo-preview-state-matrix-v1";

const previewMatrixReceipt: PermissionEnforcementReceipt = {
  requested_profile: "galaxy",
  effective_profile: "orbit",
  execution_domain: "C:/dev/signal-lab",
  workspace_isolation: {
    status: "enforced",
    mechanism: "git-worktree",
    detail: "Agent mutations remain outside the primary repository until run-level Apply.",
  },
  host_filesystem_boundary: {
    status: "advisory",
    mechanism: "child-cwd-and-policy-gates",
    detail: "Workspace cwd and Pytxo-mediated path gates are active; syscall sandboxing is not.",
  },
  network: {
    status: "unavailable",
    mechanism: "not-reported-on-this-platform",
    detail: "No network enforcement evidence was recorded for this run.",
  },
  apply_boundary: {
    status: "bypassed",
    mechanism: "operator-override",
    detail: "The reviewed-apply boundary was explicitly disabled for this run.",
  },
};

function stateMatrixRequested(): boolean {
  return typeof localStorage !== "undefined" && localStorage.getItem(STATE_MATRIX_KEY) === "1";
}

const previewCompletedAgents = [
  { id: "run-71ad:agent-0", domain_id: "signal-lab", run_id: "run-71ad", task_id: "signal-core", wave: 0, status: "completed", exit_code: 0, root_id: null },
  { id: "run-71ad:agent-1", domain_id: "signal-lab", run_id: "run-71ad", task_id: "contract-tests", wave: 1, status: "completed", exit_code: 0, root_id: null },
];

export class PreviewDesktopBackend implements DesktopBackend {
  private readonly snapshot = structuredClone(previewSnapshot);
  private readonly voiceDevices = new Map<string, string>();
  private readonly cancelledVoiceSessions = new Set<string>();
  private readonly activeVoiceSessions = new Set<string>();
  private readonly domainListeners = new Set<(event: DesktopChangedEvent) => void>();
  private readonly changes: DomainChangeDto[] = [];
  private readonly recoveredRuns = new Set<string>();
  private sequence = 0;

  constructor() {
    if (typeof localStorage !== "undefined" && localStorage.getItem("pytxo-preview-run-state-v1") === "review_failed") {
      Object.assign(this.snapshot.runs[0], {
        status: "failed", apply_status: "review_failed", prepared_digest: null, prepared_at: null,
        last_apply_error: { at: "2026-09-07T01:15:56Z", code: "review_preparation_failed", message: "runner: agent agent-0 edited test/risk-policy.test.mjs outside declared claims", attempt_id: null, rollback_confirmed: false },
      });
      this.snapshot.agents = this.snapshot.agents.map((agent) => ({ ...agent, status: "completed", exit_code: 0 }));
    }
    if (typeof localStorage !== "undefined" && localStorage.getItem("pytxo-preview-run-state-v1") === "starting") {
      this.snapshot.runs[0].status = "starting";
    }
    const nativeAgentFixture = typeof localStorage === "undefined" ? null : localStorage.getItem("pytxo-preview-native-agent-ids-v1");
    if (nativeAgentFixture === "switch") {
      this.snapshot.runs.push({ ...this.snapshot.runs[0], id: "run-other", status: "completed" });
      this.snapshot.agents.push(...this.snapshot.agents.map((agent) => ({ ...agent, run_id: "run-other" })));
    }
    if (nativeAgentFixture === "1" || nativeAgentFixture === "switch") {
      this.snapshot.agents = this.snapshot.agents.map((agent) => ({ ...agent, id: `${agent.run_id}:${agent.id}` }));
    }
    const requested =
      typeof localStorage === "undefined"
        ? null
        : localStorage.getItem("pytxo-preview-review-state-v1");
    const reviewRun = this.snapshot.runs.find((run) => run.id === "run-71ad");
    if (reviewRun && requested) {
      reviewRun.apply_status =
        requested === "recovered" ? "ready" : requested;
    }
  }

  private emitChange(domainId: string, entityKind: string, entityId: string) {
    const event = { domain_id: domainId, entity_kind: entityKind, entity_id: entityId };
    this.sequence += 1;
    this.changes.push({
      sequence: this.sequence,
      entity_kind: entityKind,
      entity_id: entityId,
      changed_at: new Date().toISOString(),
    });
    for (const callback of this.domainListeners) callback(event);
  }

  private resolveApproval(requestId: string) {
    const resolved = this.snapshot.approvals.find((approval) => approval.id === requestId);
    this.snapshot.approvals = this.snapshot.approvals.filter((approval) => approval.id !== requestId);
    if (!resolved?.domain_id) return;
    const domain = this.snapshot.domains.find((item) => item.domain_id === resolved.domain_id);
    if (domain) {
      domain.hitl_pending = this.snapshot.approvals.filter(
        (approval) => approval.domain_id === domain.domain_id,
      ).length;
    }
  }

  async loadSnapshot(_opts: { includeAgents?: boolean } = {}) {
    return structuredClone(this.snapshot);
  }
  async approve(requestId: string) {
    this.resolveApproval(requestId);
    this.emitChange("pytxo", "approval", requestId);
  }
  async deny(requestId: string) {
    this.resolveApproval(requestId);
    this.emitChange("pytxo", "approval", requestId);
  }
  async stopRun(runId: string, domainId: string) {
    const domain = this.snapshot.domains.find((item) => item.domain_id === domainId);
    if (!domain) throw new Error(`Execution domain not found: ${domainId}`);
    const active = this.snapshot.runs.find(
      (run) =>
        run.repo_root === domain.repo_root &&
        ["starting", "running", "pending", "dispatching", "active"].includes(run.status.toLowerCase()),
    );
    if (!active || active.id !== runId) {
      throw new Error(
        `Refusing to stop ${runId}: ${active ? `${active.id} is now active` : "no run is active"} in ${domainId}`,
      );
    }
    active.status = "cancelled";
    this.snapshot.agents = this.snapshot.agents.filter((agent) => agent.run_id !== runId);
    domain.active_runs = this.snapshot.runs.filter(
      (run) =>
        run.repo_root === domain.repo_root &&
        ["starting", "running", "pending", "dispatching", "active"].includes(run.status.toLowerCase()),
    ).length;
    domain.latest_run_status = active.status;
    this.emitChange(domainId, "run", runId);
  }
  async forgetDomain(domainId: string) {
    this.snapshot.domains = this.snapshot.domains.filter((domain) => domain.domain_id !== domainId);
  }
  async listAdeClis() {
    return previewAdeState();
  }
  async startAdeLogin(id: string) {
    const target = previewAdeClis.find((item) => item.id === id);
    if (!target?.installed) throw new Error(`${target?.display_name ?? id} is not installed or is not on PATH.`);
    return { id, message: `${target.display_name} sign-in opened. Finish the vendor flow, then recheck.` };
  }
  async listProviders() {
    return structuredClone(previewProviders);
  }
  async previewFlow(input: FlowDraftInput): Promise<FlowPlan> {
    if (typeof localStorage !== "undefined" && localStorage.getItem("pytxo-preview-flow-error-v1") === "1") {
      throw new Error("Preview failed while checking the selected workspace. Fix the workspace issue, then build a new plan.");
    }
    const detected = previewAdeState();
    const requested = detected.find((cli) => cli.id === input.ade_id) ?? null;
    const available = !!requested && requested.installed && (requested.auth_state === "signed_in" || requested.auth_state === "not_applicable");
    const omitVerification = typeof localStorage !== "undefined" && localStorage.getItem("pytxo-preview-flow-verification-v1") === "none";
    const tasks = [
      { id: "desktop-flow", agent: input.ade_id ?? "codex", prompt: `Implement the Desktop slice of: ${input.mission_text}`, paths: ["apps/desktop/src/components/desktop2/FlowScreen.svelte"], dependencies: [], root: null, verify: ["npm run check"] },
      { id: "orchestration-flow", agent: "codex", prompt: `Implement the orchestration slice of: ${input.mission_text}`, paths: ["crates/pytxo-orchestrate/src/flow.rs"], dependencies: [], root: null, verify: ["cargo test -p pytxo-orchestrate"] },
      { id: "contract-tests", agent: "codex", prompt: `Verify the reviewed Flow contract for: ${input.mission_text}`, paths: ["crates/pytxo-orchestrate/tests/flow_mission.rs"], dependencies: ["desktop-flow", "orchestration-flow"], root: null, verify: ["cargo test -p pytxo-orchestrate"] },
    ];
    return { draft_id: input.id, domain_id: input.domain_id ?? "pytxo", project_id: input.project_id, status: "ready", tasks: tasks.map((task) => omitVerification ? { ...task, verify: [] } : task), waves: [["desktop-flow", "orchestration-flow"], ["contract-tests"]], permission_profile: "orbit", isolation_mode: "copy_on_write", isolation_backend_intent: "projfs", execution_backend: "pty", ade: { requested: input.ade_id ?? null, available, installed: detected.filter((cli) => cli.installed).map((cli) => cli.id), command: requested?.default_cmd ?? null }, warnings: [{ code: "preview_fixture", message: "Browser preview uses a contract-valid Pytxo fixture; native preview reads the selected repository." }], blocked_reasons: [], estimated_tokens: 18000, estimated_cost_usd: 0.64, previewed_at: new Date().toISOString() };
  }
  async dispatchFlow() {
    const domain = this.snapshot.domains[0];
    if (!this.snapshot.runs.some((run) => run.id === "run-preview")) {
      this.snapshot.runs.unshift({
        id: "run-preview",
        domain_id: domain?.domain_id ?? "pytxo",
        status: "running",
        repo_root: domain?.repo_root ?? "C:/dev/pytxo",
        started_at: new Date().toISOString(),
        estimated_cost_usd: 0,
        permission_profile: "orbit",
        isolation_mode: "copy_on_write",
        isolation_backend: "projfs",
        apply_status: "pending",
        applied_at: null,
        prepared_digest: null,
        prepared_at: null,
        last_apply_error: null,
        recovery_state: null,
      });
    }
    this.emitChange(domain?.domain_id ?? "pytxo", "run", "run-preview");
    setTimeout(() => {
      const run = this.snapshot.runs.find((item) => item.id === "run-preview");
      if (run) run.status = "completed";
      this.emitChange(domain?.domain_id ?? "pytxo", "run", "run-preview");
    }, 350);
    return "run-preview";
  }
  async saveReviewedFlow(plan: FlowPlan) { return plan; }
  async startVoice(device: string, language: string): Promise<VoiceSessionDto> {
    if (device === "Disconnected microphone") throw new Error("The input device was lost");
    if (this.activeVoiceSessions.size) throw new Error("another Voice session is already active");
    const session_id = crypto.randomUUID();
    this.voiceDevices.set(session_id, device);
    this.activeVoiceSessions.add(session_id);
    return { session_id, device, language, state: "recording", elapsed_ms: 0, buffered_samples: 0, confidence: null, transcript_segments: [], error: null };
  }
  async pauseVoice(sessionId: string): Promise<VoiceSessionDto> { return { session_id: sessionId, device: "default", language: "en", state: "paused", elapsed_ms: 1200, buffered_samples: 16000, confidence: null, transcript_segments: [], error: null }; }
  async resumeVoice(sessionId: string): Promise<VoiceSessionDto> { return { session_id: sessionId, device: "default", language: "en", state: "recording", elapsed_ms: 1200, buffered_samples: 16000, confidence: null, transcript_segments: [], error: null }; }
  async finishVoice(sessionId: string): Promise<VoiceSessionDto> {
    // Keep the browser-preview cancellation affordance operable long enough
    // for a human or E2E pointer action; native transcription owns its timing.
    await new Promise((resolve) => setTimeout(resolve, 650));
    const device = this.voiceDevices.get(sessionId) ?? "default";
    this.activeVoiceSessions.delete(sessionId);
    if (this.cancelledVoiceSessions.has(sessionId)) return { session_id: sessionId, device, language: "en", state: "cancelled", elapsed_ms: 0, buffered_samples: 0, confidence: null, transcript_segments: [], error: null };
    const uncertain = device === "Studio microphone";
    return { session_id: sessionId, device, language: "en", state: "ready", elapsed_ms: 3200, buffered_samples: 0, confidence: uncertain ? 0.52 : 0.94, transcript_segments: [{ text: uncertain ? "Update the desktop approval flour" : "Create a production-ready Flow mission", confidence: uncertain ? 0.52 : 0.94, uncertain }], error: null };
  }
  async cancelVoice(sessionId: string): Promise<VoiceSessionDto> { this.cancelledVoiceSessions.add(sessionId); this.activeVoiceSessions.delete(sessionId); return { session_id: sessionId, device: this.voiceDevices.get(sessionId) ?? "default", language: "en", state: "cancelled", elapsed_ms: 0, buffered_samples: 0, confidence: null, transcript_segments: [], error: null }; }
  async listVoiceDevices() { return ["default", "Studio microphone", "Disconnected microphone"]; }
  async onVoiceProgress() { return () => {}; }
  async voiceModelStatus() { return null; }
  async installVoiceModel() { return "~/.pytxo/models/voice/base.en.bin"; }
  async openWorkspace() { return "C:/dev/new-workspace"; }
  async createExampleWorkspace() {
    if (typeof localStorage !== "undefined" && localStorage.getItem("pytxo-preview-example-error-v1") === "missing-git") {
      throw new Error("Git was not found. Install Git, restart Pytxo Desktop, then try again. You can skip this step for now.");
    }
    const path = "C:/Users/demo/Documents/Pytxo Examples/approval-risk-demo";
    if (!this.snapshot.domains.some((domain) => domain.repo_root === path)) {
      this.snapshot.domains.push({
        domain_id: "approval-risk-demo",
        repo_root: path,
        db_path: "~/.pytxo/domains/approval-risk-demo.db",
        project_id: null,
        status: "healthy",
        updated_at: new Date().toISOString(),
        active_runs: 0,
        latest_run_status: null,
        latest_started_at: null,
        hitl_pending: 0,
        is_available: true,
        is_temporary: false,
      });
    }
    return path;
  }
  async voiceAvailable() { return true; }
  async flowHistory(): Promise<FlowDraftRecord[]> {
    const fixture = typeof localStorage === "undefined" ? null : localStorage.getItem("pytxo-preview-flow-history-v1");
    if (!fixture) return [];
    const domainId = fixture === "long-path" ? `C:/workspaces/${"a-very-long-project-folder/".repeat(8)}repository` : fixture === "missing" ? "unavailable-workspace" : "signal-lab";
    return [{ id: "draft-reuse", title: "Fix the parser regression", mission_text: "Fix src/parser.rs and add a regression test", source: "text", domain_id: domainId, project_id: null, status: "completed", plan_json: JSON.stringify({ ade: { requested: fixture === "unavailable-cli" ? "claude" : "codex" } }), dispatched_run_id: "run-71ad", created_at: "2026-09-06T12:00:00Z", updated_at: "2026-09-06T12:00:00Z" }];
  }
  async deleteFlowDraft() {}
  async listAgents(runId: string) {
    const verification = localStorage.getItem("pytxo-preview-agent-verification-v1");
    if (runId === "run-71ad" && verification === "missing") {
      return [];
    }
    const agents = structuredClone(
      [...this.snapshot.agents, ...previewCompletedAgents].filter(
        (agent) => agent.run_id === runId,
      ),
    );
    if (runId === "run-71ad" && verification === "failed" && agents[0]) {
      agents[0].exit_code = 1;
      agents[0].status = "failed";
    }
    return agents;
  }
  async runReview(runId: string): Promise<RunReviewDto> {
    const delayedOtherRun = runId === "run-other";
    if (delayedOtherRun) await new Promise((resolve) => setTimeout(resolve, 1500));
    const run = this.snapshot.runs.find((item) => item.id === runId);
    if (!run) throw new Error(`Run not found: ${runId}`);
    const isSignalRun = runId === "run-71ad";
    const plan = isSignalRun
      ? {
          waves: [
            [{
              task_id: "signal-core",
              agent: "codex",
              paths: ["crates/pytxo-signal/src/lib.rs"],
              depends_on: [],
              wave: 0,
              root: null,
              verify: ["cargo test -p pytxo-signal"],
            }],
            [{
              task_id: "contract-tests",
              agent: "codex",
              paths: ["crates/pytxo-signal/tests/skeleton.rs"],
              depends_on: ["signal-core"],
              wave: 1,
              root: null,
              verify: ["cargo test -p pytxo-signal"],
            }],
          ],
          warnings: [],
        }
      : {
          waves: [
            [{
              task_id: "plan",
              agent: "architect",
              paths: ["docs/02-areas/orchestration/signal-core.md"],
              depends_on: [],
              wave: 0,
              root: null,
              verify: [],
            }],
            [{
              task_id: "ui",
              agent: "desktop",
              paths: ["apps/desktop/src/components/desktop2/FocusScreen.svelte"],
              depends_on: ["plan"],
              wave: 1,
              root: null,
              verify: ["npm run check"],
            }],
            [{
              task_id: "tests",
              agent: "verification",
              paths: ["crates/pytxo-orchestrate/tests/run_apply.rs"],
              depends_on: ["ui"],
              wave: 2,
              root: null,
              verify: ["cargo test -p pytxo-orchestrate"],
            }],
          ],
          warnings: ["Browser preview reflects a live run fixture; Apply remains disabled until every agent exits cleanly."],
        };
    const preparedManifest: PreparedRunManifest = {
      version: 2,
      run_id: runId,
      base_revision: isSignalRun ? "71ad8f2c4d90b6c6" : "8f2cc9814fc10e31",
      prepared_at: "2026-08-01T02:30:00Z",
      package_digest: isSignalRun ? "pkg-71ad-immutable" : "pkg-8f2c-immutable",
      summary: { added: 2, modified: 1, deleted: 1, bytes: 1847 },
      files: [
        {
          path: "crates/pytxo-signal/src/lib.rs",
          kind: "modify",
          before_sha256: "2e0f6b77",
          after_sha256: "e51c34aa",
          byte_count: 1120,
          task_id: "signal-core",
          agent_id: "run-71ad:agent-0",
          blob_digest: "e51c34aa",
          before_mode: null,
          after_mode: null,
          before_byte_count: previewBytes(previewReviewContent["crates/pytxo-signal/src/lib.rs"].before!).length,
          after_byte_count: previewBytes(previewReviewContent["crates/pytxo-signal/src/lib.rs"].after!).length,
          before_is_binary: false,
          after_is_binary: false,
          before_chunks: [{ offset: 0, length: previewBytes(previewReviewContent["crates/pytxo-signal/src/lib.rs"].before!).length, sha256: "preview-lib-before-chunk" }],
          after_chunks: [{ offset: 0, length: previewBytes(previewReviewContent["crates/pytxo-signal/src/lib.rs"].after!).length, sha256: "preview-lib-after-chunk" }],
        },
        {
          path: "crates/pytxo-signal/tests/skeleton.rs",
          kind: "add",
          before_sha256: null,
          after_sha256: "1a9d80c3",
          byte_count: 612,
          task_id: "contract-tests",
          agent_id: "run-71ad:agent-1",
          blob_digest: "1a9d80c3",
          before_mode: null,
          after_mode: null,
          before_byte_count: 0,
          after_byte_count: previewBytes(previewReviewContent["crates/pytxo-signal/tests/skeleton.rs"].after!).length,
          before_is_binary: null,
          after_is_binary: false,
          before_chunks: [],
          after_chunks: [{ offset: 0, length: previewBytes(previewReviewContent["crates/pytxo-signal/tests/skeleton.rs"].after!).length, sha256: "preview-test-after-chunk" }],
        },
        {
          path: "crates/pytxo-signal/src/legacy.rs",
          kind: "delete",
          before_sha256: "f1e2d3c4",
          after_sha256: null,
          byte_count: 110,
          task_id: "signal-core",
          agent_id: "run-71ad:agent-0",
          blob_digest: null,
          before_mode: null,
          after_mode: null,
          before_byte_count: previewBytes(previewReviewContent["crates/pytxo-signal/src/legacy.rs"].before!).length,
          after_byte_count: 0,
          before_is_binary: false,
          after_is_binary: null,
          before_chunks: [{ offset: 0, length: previewBytes(previewReviewContent["crates/pytxo-signal/src/legacy.rs"].before!).length, sha256: "preview-legacy-before-chunk" }],
          after_chunks: [],
        },
        {
          path: "assets/signal-mark.bin",
          kind: "add",
          before_sha256: null,
          after_sha256: "00ff504e47",
          byte_count: previewSignalMark.length,
          task_id: "contract-tests",
          agent_id: "run-71ad:agent-1",
          blob_digest: "00ff504e47",
          before_mode: null,
          after_mode: null,
          before_byte_count: 0,
          after_byte_count: previewSignalMark.length,
          before_is_binary: null,
          after_is_binary: true,
          before_chunks: [],
          after_chunks: [{ offset: 0, length: previewSignalMark.length, sha256: "preview-signal-mark-chunk" }],
        },
      ],
    };
    const appliedManifest: RunApplyManifest | null = run.apply_status === "applied"
      ? {
          transaction_id: "apply-71ad-20260731",
          changes: [
            {
              path: "crates/pytxo-signal/src/lib.rs",
              kind: "modify",
              source_agent_id: "run-71ad:agent-0",
              source_task_id: "signal-core",
              base_digest: "2e0f6b77",
              result_digest: "e51c34aa",
            },
            {
              path: "crates/pytxo-signal/tests/skeleton.rs",
              kind: "add",
              source_agent_id: "run-71ad:agent-1",
              source_task_id: "contract-tests",
              base_digest: null,
              result_digest: "1a9d80c3",
            },
            {
              path: "crates/pytxo-signal/src/legacy.rs",
              kind: "delete",
              source_agent_id: "run-71ad:agent-0",
              source_task_id: "signal-core",
              base_digest: "f1e2d3c4",
              result_digest: null,
            },
          ],
        }
      : null;
    const requested =
      typeof localStorage === "undefined"
        ? null
        : localStorage.getItem("pytxo-preview-review-state-v1");
    const candidateCheck = typeof localStorage === "undefined" ? null : localStorage.getItem("pytxo-preview-candidate-check-v1");
    if (candidateCheck) {
      preparedManifest.version = 3;
      preparedManifest.candidate_verification = {
        version: 1,
        verified_at: "2026-09-06T02:30:00Z",
        base_inventory: [],
        candidate_inventory: [],
        exclusions: [".git", "node_modules"],
        checks: candidateCheck === "empty" ? [] : [{
          task_id: "ui",
          command: "npm run check",
          effective_profile: "orbit",
          passed: candidateCheck === "passed",
          enforcement: {},
        }],
      };
    }
    const recovered = requested === "recovered" || this.recoveredRuns.has(runId);
    const lastError: RunApplyError | null =
      recovered
        ? {
            at: "2026-08-01T02:20:00Z",
            code: "apply_failed",
            message: "The previous Apply failed and was rolled back.",
            attempt_id: "attempt-preview-1",
            rollback_confirmed: true,
          }
        : run.apply_status === "stale"
          ? {
              at: "2026-08-01T02:25:00Z",
              code: "source_drift",
              message: "An affected checkout path changed after review.",
              attempt_id: null,
              rollback_confirmed: false,
            }
          : run.apply_status === "recovery_required"
            ? {
                at: "2026-08-01T02:26:00Z",
                code: "apply_recovery_required",
                message: "Apply is blocked until recovery is reconciled.",
                attempt_id: "attempt-preview-2",
                rollback_confirmed: false,
              }
            : null;
    const applyAttempts = [
      ...(run.apply_status === "applied"
        ? [{
            attempt_id: "apply-71ad-20260731",
            created_at: run.applied_at,
            phase: "committed",
            outcome: "committed" as const,
            error_code: null,
            error_message: null,
            rollback_confirmed: false,
          }]
        : []),
      ...(recovered
        ? [
            {
              attempt_id: "attempt-preview-1",
              created_at: "2026-08-01T02:20:00Z",
              phase: "rolled_back",
              outcome: "rolled_back" as const,
              error_code: "apply_failed",
              error_message: "The previous Apply failed and was rolled back.",
              rollback_confirmed: true,
            },
            {
              attempt_id: "attempt-preview-0",
              created_at: "2026-08-01T02:10:00Z",
              phase: "rolled_back",
              outcome: "rolled_back" as const,
              error_code: "apply_failed",
              error_message: "A prior filesystem write failed and was rolled back.",
              rollback_confirmed: true,
            },
          ]
        : []),
      ...(stateMatrixRequested()
        ? [
            {
              attempt_id: "attempt-matrix-committed",
              created_at: "2026-08-01T03:00:00Z",
              phase: "committed",
              outcome: "committed" as const,
              error_code: null,
              error_message: null,
              rollback_confirmed: false,
            },
            {
              attempt_id: "attempt-matrix-rolled-back",
              created_at: "2026-08-01T03:05:00Z",
              phase: "rolled_back",
              outcome: "rolled_back" as const,
              error_code: "apply_failed",
              error_message: "A write failed and the attempt was rolled back.",
              rollback_confirmed: true,
            },
            {
              attempt_id: "attempt-matrix-recovery",
              created_at: "2026-08-01T03:10:00Z",
              phase: "recovery_required",
              outcome: "recovery_required" as const,
              error_code: "apply_recovery_required",
              error_message: "Apply is blocked until recovery is reconciled.",
              rollback_confirmed: false,
            },
            {
              attempt_id: "attempt-matrix-interrupted",
              created_at: "2026-08-01T03:15:00Z",
              phase: "interrupted",
              outcome: "interrupted" as const,
              error_code: null,
              error_message: "The process stopped before reporting an outcome.",
              rollback_confirmed: false,
            },
          ]
        : []),
    ];
    const receipt = stateMatrixRequested() || delayedOtherRun ? previewMatrixReceipt : previewReceipt;
    return {
      run_id: runId,
      base_revision: isSignalRun ? "71ad8f2c4d90b6c6" : "8f2cc9814fc10e31",
      apply_status: run.apply_status ?? "pending",
      applied_at: run.applied_at,
      plan,
      enforcement: {
        run: structuredClone(receipt),
        agents: Object.fromEntries(
          (await this.listAgents(runId))
            .map((agent) => [agent.id.startsWith(`${runId}:`) ? agent.id.slice(runId.length + 1) : agent.id, structuredClone(receipt)]),
        ),
      },
      apply_manifest: appliedManifest,
      prepared_manifest: run.apply_status === "review_failed" ? null : preparedManifest,
      prepared_digest: run.apply_status === "review_failed" ? null : preparedManifest.package_digest,
      prepared_at: run.apply_status === "review_failed" ? null : preparedManifest.prepared_at,
      last_apply_error: run.last_apply_error ?? lastError,
      recovery_state: recovered
        ? "rolled_back"
        : run.apply_status === "stale"
          ? "source_drift"
          : run.apply_status === "recovery_required"
            ? "unprovable"
            : null,
      apply_attempts: applyAttempts,
    };
  }
  async runReviewContent(
    runId: string,
    path: string,
    side: "before" | "after",
    offset: number,
    limit: number,
  ): Promise<PreparedContentChunkDto> {
    const value = previewReviewContent[path]?.[side];
    if (value == null) throw new Error(`${side} content does not exist for ${path}`);
    const review = await this.runReview(runId);
    const file = review.prepared_manifest?.files.find((candidate) => candidate.path === path);
    const digest = side === "before" ? file?.before_sha256 : file?.after_sha256;
    if (!review.prepared_manifest || !digest) throw new Error(`${side} content identity is unavailable for ${path}`);
    const requestKey = "pytxo-preview-review-content-requests-v1";
    const requests = JSON.parse(localStorage.getItem(requestKey) ?? "[]") as Array<{ path: string; side: "before" | "after" }>;
    requests.push({ path, side });
    localStorage.setItem(requestKey, JSON.stringify(requests));
    const activeKey = "pytxo-preview-review-content-active-v1";
    const maxActiveKey = "pytxo-preview-review-content-max-active-v1";
    const active = Number(localStorage.getItem(activeKey) ?? "0") + 1;
    localStorage.setItem(activeKey, String(active));
    localStorage.setItem(
      maxActiveKey,
      String(Math.max(active, Number(localStorage.getItem(maxActiveKey) ?? "0"))),
    );
    try {
      const delay = Number(localStorage.getItem("pytxo-preview-review-content-delay-v1") ?? "0");
      if (delay > 0) await new Promise((resolve) => setTimeout(resolve, delay));
      const bytes = previewBytes(value);
      const chunk = bytes.slice(offset, Math.min(bytes.length, offset + Math.min(limit, 64 * 1024)));
      const nextOffset = offset + chunk.length;
      return {
        run_id: runId,
        package_digest: review.prepared_manifest.package_digest,
        path,
        side,
        digest,
        byte_count: bytes.length,
        binary: value instanceof Uint8Array,
        offset,
        length: chunk.length,
        next_offset: nextOffset,
        complete: nextOffset === bytes.length,
        data_base64: bytesToBase64(chunk),
      };
    } finally {
      const remaining = Math.max(0, Number(localStorage.getItem(activeKey) ?? "1") - 1);
      localStorage.setItem(activeKey, String(remaining));
    }
  }
  async refreshRunReview(runId: string) {
    const run = this.snapshot.runs.find((item) => item.id === runId);
    if (!run) throw new Error(`Run not found: ${runId}`);
    run.apply_status = "ready";
    this.emitChange("signal-lab", "contract", runId);
    return (await this.runReview(runId)).prepared_manifest!;
  }
  async discardRunReview(runId: string) {
    const run = this.snapshot.runs.find((item) => item.id === runId);
    if (!run) throw new Error(`Run not found: ${runId}`);
    run.apply_status = "discarded";
    this.emitChange("signal-lab", "contract", runId);
  }
  async reconcileRunRecovery(runId: string) {
    const run = this.snapshot.runs.find((item) => item.id === runId);
    if (!run) throw new Error(`Run not found: ${runId}`);
    run.apply_status = "ready";
    this.recoveredRuns.add(runId);
    this.emitChange("signal-lab", "contract", runId);
    return { outcome: "rolled_back", attempt_id: "attempt-preview-2" };
  }
  async domainChanges(_domainId: string, cursor: number, limit = 200) {
    if (cursor > this.sequence) {
      return {
        changes: [],
        next_cursor: this.sequence,
        has_more: false,
        cursor_gap: true,
      };
    }
    const page = this.changes.filter((change) => change.sequence > cursor).slice(0, limit);
    return {
      changes: structuredClone(page),
      next_cursor: page.at(-1)?.sequence ?? cursor,
      has_more: this.changes.some((change) => change.sequence > (page.at(-1)?.sequence ?? cursor)),
      cursor_gap: false,
    };
  }
  async onDomainChanged(callback: (event: DesktopChangedEvent) => void) {
    this.domainListeners.add(callback);
    return () => this.domainListeners.delete(callback);
  }
  async structuralGraph() {
    return {
      version: 3,
      nodes: [
        { id: "signal-core", label: "SignalCore", edited: true, root_id: null },
        { id: "parser", label: "Parser", edited: true, root_id: null },
        { id: "skeleton", label: "Skeleton", edited: false, root_id: null },
        { id: "tests", label: "contract tests", edited: true, root_id: null },
      ],
      edges: [
        { from: "signal-core", to: "parser" },
        { from: "parser", to: "skeleton" },
        { from: "tests", to: "signal-core" },
      ],
    };
  }
  async workspaceStructuralGraph() {
    return this.structuralGraph();
  }
  async agentArbitrage(runId: string) {
    return (await this.listAgents(runId))
      .map((agent, index) => ({
        agent_id: agent.id,
        saved_tokens: 3200 - (index * 450),
        edited_paths: index + 1,
        fallback_paths: 0,
      }));
  }
  async applyRunChanges(runId: string): Promise<RunApplyManifest> {
    const run = this.snapshot.runs.find((item) => item.id === runId);
    if (!run) throw new Error(`Run not found: ${runId}`);
    const review = await this.runReview(runId);
    const manifest = review.apply_manifest && !("error" in review.apply_manifest)
      ? review.apply_manifest
      : {
          transaction_id: `apply-${runId}`,
          changes: [],
        };
    run.apply_status = "applying";
    const count = Number(localStorage.getItem("pytxo-preview-apply-count-v1") ?? "0") + 1;
    localStorage.setItem("pytxo-preview-apply-count-v1", String(count));
    this.emitChange("signal-lab", "contract", runId);
    await new Promise((resolve) => setTimeout(resolve, 120));
    if (localStorage.getItem("pytxo-preview-apply-outcome-v1") === "stale") {
      run.apply_status = "stale";
      run.last_apply_error = {
        at: new Date().toISOString(),
        code: "source_drift",
        message: "An affected checkout path changed after review.",
        attempt_id: null,
        rollback_confirmed: false,
      };
      run.recovery_state = "source_drift";
      this.emitChange("signal-lab", "contract", runId);
      throw new Error("An affected checkout path changed after review.");
    }
    run.apply_status = "applied";
    run.applied_at = new Date().toISOString();
    run.last_apply_error = null;
    run.recovery_state = null;
    this.emitChange("signal-lab", "contract", runId);
    return manifest;
  }
}
