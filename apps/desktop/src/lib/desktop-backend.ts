import { ipc } from "./ipc";
import type { AdeCliStatusDto, AgentArbitrageDto, AgentDto, CatalogEntryStatus, DesktopChangedEvent, DomainChangesPageDto, FleetRunDto, HitlDto, PreparedContentChunkDto, PreparedRunManifest, ProviderStatusDto, RoutingDisplaySummary, RunApplyManifest, RunDto, RunReviewDto, StructuralGraphDto } from "./types";
import type { FlowDraftInput, FlowDraftRecord, FlowPlan, SplitDraft, ProposedHostedAdvisorPacketPreview, ReviewedDemandFacts, ReviewedHostedAdvisorPacketPreview, RoutingHostedGrantStatus, RoutedAdvisorConsentStatus, RoutedAdvisorPacketPreview, VoiceProgressEvent, VoiceSessionDto } from "./types";
import { PreviewDesktopBackend } from "./desktop-backend.preview";

export type DesktopSnapshotError = { kind: "hypervisor-unavailable"; message: string };

export type DesktopSnapshotDiagnostic = {
  domain_id: string;
  stage: "config" | "store_path" | "store" | "runs" | "run_contract" | "routing_revision" | "agents";
  run_id: string | null;
  message: string;
};

export type DesktopSnapshot = {
  domains: CatalogEntryStatus[];
  runs: RunDto[];
  agents: AgentDto[];
  approvals: HitlDto[];
  fleets: FleetRunDto[];
  /** Per-domain omissions. A non-empty list means this snapshot is partial. */
  diagnostics: DesktopSnapshotDiagnostic[];
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
  startAdeLogin(id: string): Promise<{ id: string; message: string }>;
  listProviders(): Promise<ProviderStatusDto[]>;
  previewFlow(input: FlowDraftInput): Promise<FlowPlan>;
  splitRequest(domainId: string, request: string, adeId: string): Promise<SplitDraft>;
  cancelSplit(): Promise<void>;
  experimentalClaudeRoutingAvailable(): Promise<boolean>;
  experimentalHostedReviewAvailable(): Promise<boolean>;
  previewExperimentalClaudeFlow(input: FlowDraftInput, facts: ReviewedDemandFacts): Promise<FlowPlan>;
  previewExperimentalClaudeHostedFlow(input: FlowDraftInput, facts: ReviewedDemandFacts): Promise<FlowPlan>;
  saveReviewedFlow(plan: FlowPlan): Promise<FlowPlan>;
  dispatchFlow(draftId: string): Promise<string>;
  stopRoutedFlow(draftId: string, runId: string): Promise<void>;
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
  createExampleWorkspace(): Promise<string>;
  voiceAvailable(): Promise<boolean>;
  flowHistory(): Promise<FlowDraftRecord[]>;
  previewFlowAdvisorPacket(draftId: string): Promise<RoutedAdvisorPacketPreview>;
  previewProposedHostedPacket(draftId: string): Promise<ProposedHostedAdvisorPacketPreview>;
  previewReviewedHostedPacket(draftId: string): Promise<ReviewedHostedAdvisorPacketPreview>;
  hostedConsentStatus(domainId: string): Promise<RoutedAdvisorConsentStatus>;
  enableHostedConsent(draftId: string, domainId: string, requestDigest: string, scopeDigest: string, expectedRevision: number): Promise<RoutedAdvisorConsentStatus>;
  revokeHostedConsent(domainId: string, expectedRevision: number): Promise<RoutedAdvisorConsentStatus>;
  hostedGrantStatus(domainId: string): Promise<RoutingHostedGrantStatus | null>;
  hostedGrants(): Promise<RoutingHostedGrantStatus[]>;
  enableHostedGrant(domainId: string): Promise<RoutingHostedGrantStatus>;
  revokeHostedGrant(domainId: string, expectedRevision: number): Promise<RoutingHostedGrantStatus>;
  flowAdvisorConsent(domainId: string): Promise<RoutedAdvisorConsentStatus>;
  flowAdvisorConsentDomains(): Promise<string[]>;
  enableFlowAdvisorConsent(draftId: string, domainId: string, requestDigest: string, recipientIdentity: string, expectedRevision: number): Promise<RoutedAdvisorConsentStatus>;
  revokeFlowAdvisorConsent(domainId: string, expectedRevision: number): Promise<RoutedAdvisorConsentStatus>;
  deleteFlowDraft(draftId: string): Promise<void>;
  listAgents(runId: string, domainId: string | null): Promise<AgentDto[]>;
  readAgentEvents(runId: string, agentId: string, domainId: string, after: number, limit: number): Promise<import("./types").EventDto[]>;
  /** Last error the worker printed, as one advisory sentence; null when none. */
  agentFailureHint(runId: string, agentId: string, domainId: string): Promise<string | null>;
  runReview(runId: string, domainId: string | null): Promise<RunReviewDto>;
  routingRunSummary(runId: string, domainId: string): Promise<RoutingDisplaySummary | null>;
  runReviewContent(runId: string, path: string, side: "before" | "after", offset: number, limit: number, domainId: string | null): Promise<PreparedContentChunkDto>;
  refreshRunReview(runId: string, domainId: string | null): Promise<PreparedRunManifest>;
  discardRunReview(runId: string, domainId: string | null): Promise<void>;
  reconcileRunRecovery(runId: string, domainId: string | null): Promise<{ outcome: string; attempt_id: string | null }>;
  domainChanges(domainId: string, cursor: number, limit?: number): Promise<DomainChangesPageDto>;
  domainChangesBatch?: import("./desktop-sync").DomainChangesBatchLoader;
  catalogFingerprint(): Promise<string>;
  onDomainChanged(callback: (event: DesktopChangedEvent) => void): Promise<() => void>;
  structuralGraph(runId: string, domainId: string | null): Promise<StructuralGraphDto>;
  workspaceStructuralGraph(domainId: string): Promise<StructuralGraphDto>;
  agentArbitrage(runId: string, domainId: string | null): Promise<AgentArbitrageDto[]>;
  applyRunChanges(runId: string, domainId: string | null, expectedPackageDigest: string): Promise<RunApplyManifest>;
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
        diagnostics: snap.diagnostics,
        error: null,
      };
    } catch (e) {
      return {
        domains: [],
        runs: [],
        agents: [],
        approvals: [],
        fleets: [],
        diagnostics: [],
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
  async startAdeLogin(id: string) {
    return ipc.startAdeLogin(id);
  }
  async listProviders() {
    return ipc.listProviders();
  }
  async previewFlow(input: FlowDraftInput) { return ipc.flowPreview(input); }
  async splitRequest(domainId: string, request: string, adeId: string) { return ipc.flowSplitRequest(domainId, request, adeId); }
  async cancelSplit() { return ipc.flowSplitCancel(); }
  async experimentalClaudeRoutingAvailable() { return ipc.flowExperimentalClaudeAvailable(); }
  async experimentalHostedReviewAvailable() { return ipc.flowExperimentalHostedReviewAvailable(); }
  async previewExperimentalClaudeFlow(input: FlowDraftInput, facts: ReviewedDemandFacts) { return ipc.flowPreviewExperimentalClaude(input, facts); }
  async previewExperimentalClaudeHostedFlow(input: FlowDraftInput, facts: ReviewedDemandFacts) { return ipc.flowPreviewExperimentalClaudeHosted(input, facts); }
  async saveReviewedFlow(plan: FlowPlan) { return ipc.flowSaveReviewedPlan(plan); }
  async dispatchFlow(draftId: string) { return ipc.flowDispatch(draftId); }
  async stopRoutedFlow(draftId: string, runId: string) { return ipc.flowStopRouted(draftId, runId); }
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
  async createExampleWorkspace() {
    const created = await ipc.createExampleWorkspace();
    return ipc.ensureWorkspace(created);
  }
  async voiceAvailable() { return ipc.voiceLocalAvailable(); }
  async flowHistory() { return ipc.flowHistory(); }
  async previewFlowAdvisorPacket(draftId: string) { return ipc.flowAdvisorPacketPreview(draftId); }
  async previewProposedHostedPacket(draftId: string) { return ipc.flowProposedHostedPacketPreview(draftId); }
  async previewReviewedHostedPacket(draftId: string) { return ipc.flowReviewedHostedPacketPreview(draftId); }
  async hostedConsentStatus(domainId: string) { return ipc.flowHostedConsentStatus(domainId); }
  async enableHostedConsent(draftId: string, domainId: string, requestDigest: string, scopeDigest: string, expectedRevision: number) { return ipc.flowHostedConsentEnable(draftId, domainId, requestDigest, scopeDigest, expectedRevision); }
  async revokeHostedConsent(domainId: string, expectedRevision: number) { return ipc.flowHostedConsentRevoke(domainId, expectedRevision); }
  async hostedGrantStatus(domainId: string) { return ipc.routingHostedGrantStatus(domainId); }
  async hostedGrants() { return ipc.routingHostedGrants(); }
  async enableHostedGrant(domainId: string) { return ipc.routingHostedGrantEnable(domainId); }
  async revokeHostedGrant(domainId: string, expectedRevision: number) { return ipc.routingHostedGrantRevoke(domainId, expectedRevision); }
  async flowAdvisorConsent(domainId: string) { return ipc.flowAdvisorConsent(domainId); }
  async flowAdvisorConsentDomains() { return ipc.flowAdvisorConsentDomains(); }
  async enableFlowAdvisorConsent(draftId: string, domainId: string, requestDigest: string, recipientIdentity: string, expectedRevision: number) { return ipc.flowAdvisorConsentEnable(draftId, domainId, requestDigest, recipientIdentity, expectedRevision); }
  async revokeFlowAdvisorConsent(domainId: string, expectedRevision: number) { return ipc.flowAdvisorConsentRevoke(domainId, expectedRevision); }
  async deleteFlowDraft(draftId: string) { return ipc.flowDelete(draftId); }
  async listAgents(runId: string, domainId: string | null) { return ipc.listAgents(runId, domainId); }
  async readAgentEvents(runId: string, agentId: string, domainId: string, after: number, limit: number) { return ipc.readAgentEvents(runId, agentId, domainId, after, limit); }
  async agentFailureHint(runId: string, agentId: string, domainId: string) { return ipc.agentFailureHint(runId, agentId, domainId); }
  async runReview(runId: string, domainId: string | null) { return ipc.runReview(runId, domainId); }
  async routingRunSummary(runId: string, domainId: string) { return ipc.routingRunSummary(runId, domainId); }
  async runReviewContent(runId: string, path: string, side: "before" | "after", offset: number, limit: number, domainId: string | null) { return ipc.runReviewContent(runId, path, side, offset, limit, domainId); }
  async refreshRunReview(runId: string, domainId: string | null) { return ipc.refreshRunReview(runId, domainId); }
  async discardRunReview(runId: string, domainId: string | null) { return ipc.discardRunReview(runId, domainId); }
  async reconcileRunRecovery(runId: string, domainId: string | null) { return ipc.reconcileRunRecovery(runId, domainId); }
  async domainChanges(domainId: string, cursor: number, limit = 200) { return ipc.domainChanges(domainId, cursor, limit); }
  async domainChangesBatch(requests: Array<{ domain_id: string; cursor: number }>, limit: number) { return ipc.domainChangesBatch(requests, limit); }
  async catalogFingerprint() { return ipc.catalogFingerprint(); }
  async onDomainChanged(callback: (event: DesktopChangedEvent) => void) { return ipc.onDomainChanged(callback); }
  async structuralGraph(runId: string, domainId: string | null) { return ipc.structuralGraph(runId, domainId); }
  async workspaceStructuralGraph(domainId: string) { return ipc.workspaceStructuralGraph(domainId); }
  async agentArbitrage(runId: string, domainId: string | null) { return ipc.agentArbitrage(runId, domainId); }
  async applyRunChanges(runId: string, domainId: string | null, expectedPackageDigest: string) { return ipc.applyRunChanges(runId, domainId, expectedPackageDigest); }
}

/** True only inside a real packaged/dev Tauri webview, never in browser/Storybook/Playwright. */
export function isTauriRuntime(): boolean {
  return typeof window !== "undefined" && "__TAURI_INTERNALS__" in window;
}

export function createDesktopBackend(): DesktopBackend {
  return isTauriRuntime() ? new TauriDesktopBackend() : new PreviewDesktopBackend();
}
