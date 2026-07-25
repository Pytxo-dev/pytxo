/**
 * Fixture data and backend implementation used ONLY outside a real Tauri
 * runtime (browser dev server, Storybook, Playwright). `createDesktopBackend`
 * in `desktop-backend.ts` gates this behind a strict `__TAURI_INTERNALS__`
 * check, so a packaged Tauri build can never silently fall back to this data
 * — a failed IPC call surfaces as a real error instead.
 */
import type { DesktopBackend, DesktopSnapshot } from "./desktop-backend";
import type { AdeCliStatusDto, FlowDraftInput, FlowDraftRecord, FlowPlan, VoiceProgressEvent, VoiceSessionDto } from "./types";

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
      hitl_pending: 0,
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
      action: "Flush Blast Shield workspace",
      reason: "14 files changed · permission profile Orbit",
      created_at_ms: String(Date.now()),
      domain_id: "pytxo",
    },
  ],
  fleets: [],
  error: null,
};

const previewAdeClis: AdeCliStatusDto[] = [
  { id: "claude", display_name: "Claude Code", default_cmd: "claude", installed: true },
  { id: "cursor", display_name: "Cursor Agent", default_cmd: "cursor agent", installed: true },
  { id: "codex", display_name: "OpenAI Codex", default_cmd: "codex", installed: false },
  { id: "agy", display_name: "Antigravity", default_cmd: "agy", installed: false },
  { id: "opencode", display_name: "OpenCode", default_cmd: "opencode", installed: false },
  { id: "aider", display_name: "Aider", default_cmd: "aider", installed: false },
];

export class PreviewDesktopBackend implements DesktopBackend {
  private readonly voiceDevices = new Map<string, string>();
  private readonly cancelledVoiceSessions = new Set<string>();
  private readonly activeVoiceSessions = new Set<string>();
  async loadSnapshot(_opts: { includeAgents?: boolean } = {}) {
    return structuredClone(previewSnapshot);
  }
  async approve() {}
  async deny() {}
  async forgetDomain() {}
  async listAdeClis() {
    return previewAdeClis;
  }
  async previewFlow(input: FlowDraftInput): Promise<FlowPlan> {
    return { draft_id: input.id, domain_id: input.domain_id ?? "pytxo", project_id: input.project_id, status: "ready", tasks: [
      { id: "map", agent: "architect", prompt: input.mission_text, paths: ["src/**"], dependencies: [], root: null },
      { id: "implement", agent: input.ade_id ?? "cursor", prompt: input.mission_text, paths: ["src/**"], dependencies: ["map"], root: null },
      { id: "verify", agent: "codex", prompt: `Verify: ${input.mission_text}`, paths: ["tests/**"], dependencies: ["implement"], root: null },
    ], waves: [["map"], ["implement", "verify"]], permission_profile: "orbit", isolation_mode: "copy_on_write", isolation_backend_intent: "projfs", execution_backend: "pty", ade: { requested: input.ade_id ?? "cursor", available: true, installed: ["cursor", "codex"], command: "cursor agent" }, warnings: [], blocked_reasons: [], estimated_tokens: 18000, estimated_cost_usd: 0.64, previewed_at: new Date().toISOString() };
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
    await new Promise((resolve) => setTimeout(resolve, 180));
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
  async voiceAvailable() { return true; }
  async flowHistory() { return [] as FlowDraftRecord[]; }
  async deleteFlowDraft() {}
}
