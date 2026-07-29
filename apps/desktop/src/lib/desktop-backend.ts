import { ipc } from "./ipc";
import type { AdeCliStatusDto, AgentDto, CatalogEntryStatus, FleetRunDto, HitlDto, RunDto } from "./types";
import type { FlowDraftInput, FlowDraftRecord, FlowPlan, VoiceProgressEvent, VoiceSessionDto } from "./types";
import { PreviewDesktopBackend } from "./desktop-backend.preview";

export type DesktopSnapshotError = { kind: "hypervisor-unavailable"; message: string };

export type DesktopSnapshot = {
  domains: CatalogEntryStatus[];
  runs: RunDto[];
  agents: AgentDto[];
  approvals: HitlDto[];
  fleets: FleetRunDto[];
  /**
   * Set when the primary domains fetch itself failed (hypervisor unreachable),
   * as opposed to succeeding with a genuinely empty catalog. Screens must
   * render these as distinct states rather than collapsing both into "empty".
   */
  error: DesktopSnapshotError | null;
};

export interface DesktopBackend {
  loadSnapshot(opts?: {
    includeAgents?: boolean;
    runLimit?: number;
    fleetLimit?: number;
  }): Promise<DesktopSnapshot>;
  approve(requestId: string, domainId: string | null): Promise<void>;
  deny(requestId: string, domainId: string | null): Promise<void>;
  stopRun(runId: string, domainId: string): Promise<void>;
  forgetDomain(domainId: string): Promise<void>;
  listAdeClis(): Promise<AdeCliStatusDto[]>;
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

class TauriDesktopBackend implements DesktopBackend {
  async loadSnapshot(
    opts: { includeAgents?: boolean; runLimit?: number; fleetLimit?: number } = {},
  ): Promise<DesktopSnapshot> {
    try {
      const snap = await ipc.loadDesktopSnapshot(
        opts.runLimit ?? 30,
        opts.fleetLimit ?? 20,
        opts.includeAgents !== false,
      );
      return {
        domains: snap.domains,
        runs: snap.runs,
        agents: snap.agents,
        approvals: snap.approvals,
        fleets: snap.fleets,
        error: null,
      };
    } catch (e) {
      return {
        domains: [],
        runs: [],
        agents: [],
        approvals: [],
        fleets: [],
        error: { kind: "hypervisor-unavailable", message: e instanceof Error ? e.message : String(e) },
      };
    }
  }
  async approve(requestId: string, domainId: string | null) {
    await ipc.hitlRespond(requestId, true, domainId);
  }
  async deny(requestId: string, domainId: string | null) {
    await ipc.hitlRespond(requestId, false, domainId);
  }
  async stopRun(runId: string, domainId: string) {
    await ipc.stopRun(false, domainId, runId);
  }
  async forgetDomain(domainId: string) {
    await ipc.forgetDomain(domainId);
  }
  async listAdeClis() {
    return ipc.listAdeClis();
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

/** True only inside a real packaged/dev Tauri webview, never in browser/Storybook/Playwright. */
export function isTauriRuntime(): boolean {
  return typeof window !== "undefined" && "__TAURI_INTERNALS__" in window;
}

export function createDesktopBackend(): DesktopBackend {
  return isTauriRuntime() ? new TauriDesktopBackend() : new PreviewDesktopBackend();
}
