/**
 * Fixture data and backend implementation used ONLY outside a real Tauri
 * runtime (browser dev server, Storybook, Playwright). `createDesktopBackend`
 * in `desktop-backend.ts` gates this behind a strict `__TAURI_INTERNALS__`
 * check, so a packaged Tauri build can never silently fall back to this data
 * — a failed IPC call surfaces as a real error instead.
 */
import type { DesktopBackend, DesktopSnapshot } from "./desktop-backend";
import type { AdeCliStatusDto, FlowDraftInput, FlowDraftRecord, FlowPlan, ProviderStatusDto, VoiceProgressEvent, VoiceSessionDto } from "./types";

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
      status: "running",
      repo_root: "C:/dev/pytxo",
      started_at: new Date().toISOString(),
      estimated_cost_usd: 0.42,
      permission_profile: "orbit",
      isolation_mode: "copy_on_write",
      isolation_backend: "projfs",
    },
    {
      id: "run-71ad",
      status: "completed",
      repo_root: "C:/dev/signal-lab",
      started_at: new Date(Date.now() - 3_600_000).toISOString(),
      estimated_cost_usd: 0.18,
      permission_profile: "orbit",
      isolation_mode: "copy_on_write",
      isolation_backend: "git_worktree",
    },
  ],
  agents: [
    { id: "architect", run_id: "run-8f2c", task_id: "plan", wave: 0, status: "completed", exit_code: 0, root_id: null },
    { id: "desktop", run_id: "run-8f2c", task_id: "ui", wave: 1, status: "running", exit_code: null, root_id: null },
    { id: "verification", run_id: "run-8f2c", task_id: "tests", wave: 2, status: "queued", exit_code: null, root_id: null },
  ],
  approvals: [
    {
      id: "approval-1",
      agent_key: "desktop",
      action: "blast.flush",
      reason: "commit agent workspace to repo root",
      created_at_ms: String(Date.now()),
      domain_id: "pytxo",
    },
    {
      id: "approval-2",
      agent_key: "verification",
      action: "net.egress",
      reason: "network fetch detected in agent command",
      created_at_ms: String(Date.now() - 90_000),
      domain_id: "signal-lab",
    },
  ],
  fleets: [],
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

const previewProviders: ProviderStatusDto[] = [
  { id: "deepseek", name: "DeepSeek", api_key_env: "DEEPSEEK_API_KEY", key_configured: false, openai_compatible: true, builtin: true },
  { id: "openrouter", name: "OpenRouter", api_key_env: "OPENROUTER_API_KEY", key_configured: false, openai_compatible: true, builtin: true },
  { id: "openai", name: "OpenAI", api_key_env: "OPENAI_API_KEY", key_configured: false, openai_compatible: true, builtin: true },
  { id: "anthropic", name: "Anthropic", api_key_env: "ANTHROPIC_API_KEY", key_configured: false, openai_compatible: false, builtin: true },
  { id: "gemini", name: "Google Gemini", api_key_env: "GOOGLE_API_KEY", key_configured: false, openai_compatible: false, builtin: true },
  { id: "mistral", name: "Mistral", api_key_env: "MISTRAL_API_KEY", key_configured: false, openai_compatible: true, builtin: true },
];

export class PreviewDesktopBackend implements DesktopBackend {
  private readonly snapshot = structuredClone(previewSnapshot);
  private readonly voiceDevices = new Map<string, string>();
  private readonly cancelledVoiceSessions = new Set<string>();
  private readonly activeVoiceSessions = new Set<string>();

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
  }
  async deny(requestId: string) {
    this.resolveApproval(requestId);
  }
  async stopRun(runId: string, domainId: string) {
    const domain = this.snapshot.domains.find((item) => item.domain_id === domainId);
    if (!domain) throw new Error(`Execution domain not found: ${domainId}`);
    const active = this.snapshot.runs.find(
      (run) =>
        run.repo_root === domain.repo_root &&
        ["running", "pending", "dispatching", "active"].includes(run.status.toLowerCase()),
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
        ["running", "pending", "dispatching", "active"].includes(run.status.toLowerCase()),
    ).length;
    domain.latest_run_status = active.status;
  }
  async forgetDomain(domainId: string) {
    this.snapshot.domains = this.snapshot.domains.filter((domain) => domain.domain_id !== domainId);
  }
  async listAdeClis() {
    return previewAdeClis;
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
    return { draft_id: input.id, domain_id: input.domain_id ?? "pytxo", project_id: input.project_id, status: "ready", tasks: [
      { id: "map", agent: "architect", prompt: input.mission_text, paths: ["src/**"], dependencies: [], root: null },
      { id: "implement", agent: input.ade_id ?? "cursor", prompt: input.mission_text, paths: ["src/**"], dependencies: ["map"], root: null },
      { id: "verify", agent: "codex", prompt: `Verify: ${input.mission_text}`, paths: ["tests/**"], dependencies: ["implement"], root: null },
    ], waves: [["map"], ["implement", "verify"]], permission_profile: "orbit", isolation_mode: "copy_on_write", isolation_backend_intent: "projfs", execution_backend: "pty", ade: { requested: input.ade_id ?? "cursor", available: true, installed: ["cursor", "codex"], command: "cursor-agent -p --trust" }, warnings: [], blocked_reasons: [], estimated_tokens: 18000, estimated_cost_usd: 0.64, previewed_at: new Date().toISOString() };
  }
  async dispatchFlow() { return "run-preview"; }
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
  async flowHistory() { return [] as FlowDraftRecord[]; }
  async deleteFlowDraft() {}
}
