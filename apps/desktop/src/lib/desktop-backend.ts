import { ipc } from "./ipc";
import type { AgentDto, CatalogEntryStatus, FleetRunDto, HitlDto, RunDto } from "./types";
import type { FlowDraftInput, FlowDraftRecord, FlowPlan, VoiceProgressEvent, VoiceSessionDto } from "./types";

export type DesktopSnapshot = {
  domains: CatalogEntryStatus[];
  runs: RunDto[];
  agents: AgentDto[];
  approvals: HitlDto[];
  fleets: FleetRunDto[];
};

export interface DesktopBackend {
  loadSnapshot(): Promise<DesktopSnapshot>;
  approve(requestId: string, domainId: string | null): Promise<void>;
  deny(requestId: string, domainId: string | null): Promise<void>;
  previewFlow(input: FlowDraftInput): Promise<FlowPlan>;
  saveReviewedFlow(plan: FlowPlan): Promise<FlowPlan>;
  dispatchFlow(draftId: string): Promise<string>;
  startVoice(device: string, language: string): Promise<VoiceSessionDto>;
  pauseVoice(sessionId: string): Promise<VoiceSessionDto>;
  resumeVoice(sessionId: string): Promise<VoiceSessionDto>;
  finishVoice(sessionId: string): Promise<VoiceSessionDto>;
  cancelVoice(sessionId: string): Promise<VoiceSessionDto>;
  listVoiceDevices(): Promise<string[]>;
  onVoiceProgress(callback: (event: VoiceProgressEvent) => void): Promise<() => void>;
  voiceModelStatus(): Promise<string | null>;
  installVoiceModel(): Promise<string>;
  openWorkspace(): Promise<string | null>;
  voiceAvailable(): Promise<boolean>;
  flowHistory(): Promise<FlowDraftRecord[]>;
  deleteFlowDraft(draftId: string): Promise<void>;
}

const previewSnapshot: DesktopSnapshot = {
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
};

class PreviewDesktopBackend implements DesktopBackend {
  private readonly voiceDevices = new Map<string, string>();
  private readonly cancelledVoiceSessions = new Set<string>();
  private readonly activeVoiceSessions = new Set<string>();
  async loadSnapshot() {
    return structuredClone(previewSnapshot);
  }
  async approve() {}
  async deny() {}
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

class TauriDesktopBackend implements DesktopBackend {
  async loadSnapshot(): Promise<DesktopSnapshot> {
    const domains = await ipc.listDomainsStatus();
    const runsByDomain = await Promise.all(
      domains.map(async (domain) => ({ domain, runs: await ipc.listRuns(30, domain.domain_id) })),
    );
    const runs = runsByDomain.flatMap((entry) => entry.runs);
    const agentGroups = await Promise.all(
      runsByDomain.flatMap(({ domain, runs: domainRuns }) =>
        domainRuns
          .filter((run) => ["running", "pending", "dispatching", "active"].includes(run.status.toLowerCase()))
          .map((run) => ipc.listAgents(run.id, domain.domain_id)),
      ),
    );
    const agents = agentGroups.flat();
    const [approvals, fleets] = await Promise.all([ipc.listHitlAll(), ipc.listFleetRuns(20)]);
    return { domains, runs, agents, approvals, fleets };
  }
  async approve(requestId: string, domainId: string | null) {
    await ipc.hitlRespond(requestId, true, domainId);
  }
  async deny(requestId: string, domainId: string | null) {
    await ipc.hitlRespond(requestId, false, domainId);
  }
  async previewFlow(input: FlowDraftInput) { return ipc.flowPreview(input); }
  async saveReviewedFlow(plan: FlowPlan) { return ipc.flowSaveReviewedPlan(plan); }
  async dispatchFlow(draftId: string) { return ipc.flowDispatch(draftId); }
  async startVoice(device: string, language: string) { return ipc.voiceStartSession(device, language); }
  async pauseVoice(sessionId: string) { return ipc.voicePauseSession(sessionId); }
  async resumeVoice(sessionId: string) { return ipc.voiceResumeSession(sessionId); }
  async finishVoice(sessionId: string) { return ipc.voiceFinishSession(sessionId); }
  async cancelVoice(sessionId: string) { return ipc.voiceCancelSession(sessionId); }
  async listVoiceDevices() { return ipc.voiceListDevices(); }
  async onVoiceProgress(callback: (event: VoiceProgressEvent) => void) { return ipc.onVoiceProgress(callback); }
  async voiceModelStatus() { return ipc.voiceModelStatus(); }
  async installVoiceModel() { return ipc.voiceInstallDefaultModel(); }
  async openWorkspace() {
    const selected = await ipc.pickWorkspaceFolder();
    return selected ? ipc.ensureWorkspace(selected) : null;
  }
  async voiceAvailable() { return ipc.voiceLocalAvailable(); }
  async flowHistory() { return ipc.flowHistory(); }
  async deleteFlowDraft(draftId: string) { return ipc.flowDelete(draftId); }
}

export function createDesktopBackend(): DesktopBackend {
  return typeof window !== "undefined" && "__TAURI_INTERNALS__" in window
    ? new TauriDesktopBackend()
    : new PreviewDesktopBackend();
}
