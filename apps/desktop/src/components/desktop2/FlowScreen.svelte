<script lang="ts">
  import IconAlertTriangle from "@tabler/icons-svelte/icons/alert-triangle";
  import IconArrowRight from "@tabler/icons-svelte/icons/arrow-right";
  import IconFolderPlus from "@tabler/icons-svelte/icons/folder-plus";
  import IconLoader2 from "@tabler/icons-svelte/icons/loader-2";
  import IconMicrophone from "@tabler/icons-svelte/icons/microphone";
  import IconPlayerRecord from "@tabler/icons-svelte/icons/player-record";
  import { onDestroy, onMount, untrack } from "svelte";
  import HostedRoutingReview from "./HostedRoutingReview.svelte";
  import AdeIdentity from "./AdeIdentity.svelte";
  import type { ComposerDraft } from "../../lib/composer-draft";
  import { adeAvailabilityLabel, betaAdesInOrder, DESKTOP_BETA_MAX_WORKERS, isAdeRunnable, isBetaAde } from "../../lib/ade-status";
  import { draftTitle } from "../../lib/draft-title";
  import type { DesktopBackend } from "../../lib/desktop-backend";
  import type { AdeCliStatusDto, FlowDraftRecord, FlowPlan, ProposedHostedAdvisorPacketPreview, ReviewedDemandFacts, RoutedAdvisorConsentStatus, RoutedAdvisorPacketPreview, RunDto, VoiceSessionDto, VoiceState } from "../../lib/types";

  const ADE_CHOICE_KEY = "pytxo-flow-ade-v1";
  const CLAUDE_ROUTE_CHOICE = "claude-proposal-route-experiment";
  const CLAUDE_HOSTED_CHOICE = "claude-hosted-shadow-review-experiment";
  let {
    backend,
    domains,
    runs = [],
    preferredDomainId = null,
    previewState = "draft",
    onAddWorkspace = null,
    onDispatched = null,
    draft = null,
    preferredAdeId = null,
    onDraftChange = () => {},
  }: {
    backend: DesktopBackend;
    domains: { domain_id: string; repo_root: string; project_id?: string | null }[];
    runs?: RunDto[];
    preferredDomainId?: string | null;
    previewState?: "draft" | "recording" | "paused" | "transcribing" | "cancelled" | "failed" | "uncertain" | "planning" | "ready" | "blocked" | "dispatched";
    onAddWorkspace?: (() => void | Promise<void>) | null;
    onDispatched?: ((runId: string) => void) | null;
    draft?: ComposerDraft | null;
    preferredAdeId?: string | null;
    onDraftChange?: (draft: ComposerDraft | null) => void;
  } = $props();
  let selectedDomainId = $state("");
  let selectedAde = $state(untrack(() => preferredAdeId ?? draft?.adeId ?? ""));
  let adeClis = $state<AdeCliStatusDto[]>([]);
  let adeLoading = $state(true);
  let adeError = $state("");
  let experimentalClaudeAvailable = $state(false);
  let experimentalHostedAvailable = $state(false);
  let mission = $state(untrack(() => draft?.mission ?? ""));
  let runChecks = $state(untrack(() => draft?.verificationCommands ?? ""));
  let reviewedTaskKind = $state<ReviewedDemandFacts["task_kind"]>("other");
  let reviewedContextComplete = $state(false);
  let crossComponent = $state<"unknown" | "yes" | "no">("unknown");
  let repeatableSymptom = $state<"unknown" | "yes" | "no">("unknown");
  let specificCauseHypothesis = $state<"unknown" | "yes" | "no">("unknown");
  let factsMissionText = untrack(() => draft?.mission ?? "");
  let missionSource = $state<"text" | "voice">(untrack(() => draft?.source ?? "text"));
  let voiceState = $state<VoiceState>("idle");
  let recording = $derived(voiceState === "recording");
  let voiceSessionId = $state<string | null>(null);
  let voiceDevices = $state<string[]>(["default"]);
  let selectedVoiceDevice = $state("default");
  let voiceSegments = $state<VoiceSessionDto["transcript_segments"]>([]);
  let transcriptionProgress = $state(0);
  let voicePointerStartedAt = 0;
  let voicePointerHadSession = false;
  let suppressVoiceClick = false;
  let voicePointerAction: Promise<void> | null = null;
  let keyboardVoiceHeld = false;
  let voiceDisposed = false;
  let voiceAvailable = $state(false);
  let voiceCapture = $state<"both" | "hold" | "click">("both");
  const voiceHint = $derived(voiceCapture === "hold" ? "Press and hold to record. Hold Space or Enter with the keyboard."
    : voiceCapture === "click" ? "Click to start, then click again to finish recording." : "Click to record or press and hold");
  let plan = $state<FlowPlan | null>(null);
  let planInputKey = $state<string | null>(null);
  let planAttempted = $state(false);
  let planning = $state(false);
  let error = $state("");
  let dispatchedRun = $state("");
  // Teardown can read a reactive value from before the navigation batch.
  // Draft consumption is an instance lifecycle fact, not rendered state.
  let draftConsumed = false;
  let dispatchedStatus = $state("");
  let dispatching = $state(false);
  let reviewedRouteReady = $state(false);
  let routedStopPending = $state(false);
  let routedStopRequested = $state(false);
  let routedStopNotice = $state("");
  let historyStopPending = $state("");
  let historyStopNotice = $state("");
  let history = $state<FlowDraftRecord[]>([]);
  let packetEligible = $state<Record<string, boolean>>({});
  let packetDraftId = $state<string | null>(null);
  let packetPreview = $state<RoutedAdvisorPacketPreview | null>(null);
  let proposedHostedPacket = $state<ProposedHostedAdvisorPacketPreview | null>(null);
  let proposedHostedBody = $state("");
  let proposedHostedError = $state("");
  let packetConsent = $state<RoutedAdvisorConsentStatus | null>(null);
  let packetBody = $state("");
  let packetLoading = $state(false);
  let packetActionPending = $state(false);
  let packetNotice = $state("");
  let packetError = $state("");
  let workspaceGrants = $state<Record<string, RoutedAdvisorConsentStatus>>({});
  let workspaceGrantRevoking = $state<string | null>(null);
  let workspaceGrantError = $state("");
  let workspaceGrantReadError = $state("");
  let workspaceGrantNotice = $state("");
  let workspaceGrantGeneration = 0;
  let historyGeneration = 0;
  let packetOpenGeneration = 0;
  let packetDisposed = false;
  let historySyncedRun = "";
  let missionInput: HTMLTextAreaElement | undefined = $state();
  let compactPane = $state<"request" | "plan">("request");
  let draftNotice = $state("");
  let domainSelectionInitialized = false;
  let adeSelectionInitialized = false;
  /** Extra Beta CLIs that share the plan with the primary agent, in assignment order. */
  let teamAdes = $state<string[]>([]);
  /** Explicit per-task CLI choices; any change requires a fresh plan. */
  let taskAdes = $state<Record<string, string>>({});
  let workers = $state(1);
  let workersTouched = false;

  const currentInputKey = $derived(
    JSON.stringify({
      mission: mission.trim(),
      source: missionSource,
      domain: selectedDomainId,
      ade: selectedAde,
      team: teamAdes,
      taskAdes,
      workers,
      checks: runChecks.trim(),
      routingFacts: selectedAde === CLAUDE_ROUTE_CHOICE || selectedAde === CLAUDE_HOSTED_CHOICE
        ? [reviewedTaskKind, reviewedContextComplete, crossComponent, repeatableSymptom, specificCauseHypothesis] : null,
    }),
  );
  const planMatchesInputs = $derived(!!plan && planInputKey === currentInputKey);
  const routedChoice = $derived(selectedAde === CLAUDE_ROUTE_CHOICE || selectedAde === CLAUDE_HOSTED_CHOICE);
  /** Every CLI in this request, primary first. */
  const team = $derived(routedChoice || !selectedAde ? [] : [selectedAde, ...teamAdes.filter((id) => id !== selectedAde)]);
  const teamCandidates = $derived(betaAdesInOrder(adeClis).filter((cli) => isAdeReady(cli) && cli.id !== selectedAde));
  const adeName = (id: string | null | undefined) => adeClis.find((cli) => cli.id === id)?.display_name ?? id ?? "Not selected";
  function toggleTeamAde(id: string) {
    teamAdes = teamAdes.includes(id) ? teamAdes.filter((other) => other !== id) : [...teamAdes, id];
    taskAdes = {};
    if (!workersTouched) workers = Math.min(DESKTOP_BETA_MAX_WORKERS, 1 + teamAdes.filter((other) => other !== selectedAde).length);
  }
  function stepWorkers(delta: number) {
    workersTouched = true;
    workers = Math.min(DESKTOP_BETA_MAX_WORKERS, Math.max(1, workers + delta));
  }
  const selectedAdeStatus = $derived(adeClis.find((cli) => cli.id === selectedAde) ?? null);
  const selectedAdeReady = $derived(selectedAde === CLAUDE_ROUTE_CHOICE ? experimentalClaudeAvailable : selectedAde === CLAUDE_HOSTED_CHOICE ? experimentalHostedAvailable : !!selectedAdeStatus && isAdeReady(selectedAdeStatus));
  const unavailableAdes = $derived(adeClis.filter((cli) => !isAdeReady(cli)));
  const verificationCommands = $derived(
    verificationCommandsFor(plan),
  );
  const planHasVerification = $derived(verificationCommands.length > 0);
  const activeWorkspaceGrants = $derived(Object.values(workspaceGrants).filter((grant) => grant.enabled));
  const canDispatchPlan = $derived(
    !!plan && plan.status === "ready" && !plan.blocked_reasons.length && planMatchesInputs && planHasVerification && selectedAdeReady && !adeLoading && !dispatchedRun && !dispatching,
  );
  const buildPlanDisabledReason = $derived.by(() => {
    if (planning) return "Plan construction is already in progress.";
    if (!mission.trim()) return "Describe the outcome before building a plan.";
    if (/\[(?:existing file path|expected behavior|test file path)\]/i.test(mission)) return "Replace the bracketed guidance with your repository's paths and expected behavior.";
    if (!selectedDomainId) return "Select a workspace before building a plan.";
    if (adeLoading) return "Checking the selected agent CLI.";
    if (!selectedAdeReady) return "Select an installed, signed-in agent CLI before building a plan.";
    return null;
  });
  const runDisabledReason = $derived.by(() => {
    if (!plan) return "Build and review a plan before starting the run.";
    if (!planMatchesInputs) return "The plan no longer matches the outcome, workspace, or agent CLI. Build it again.";
    if (plan.status === "review_only") return "This hosted Shadow draft is for packet review only; it cannot start a run.";
    if (plan.status !== "ready" || plan.blocked_reasons.length) return plan.blocked_reasons.length
      ? plan.blocked_reasons.map(describeBlocker).join("\n")
      : "The planner blocked this run without a reason. Check the workspace and agent CLI, then build a new plan.";
    if (adeLoading || !selectedAdeReady) return "Recheck the selected agent CLI before starting the run.";
    if (!planHasVerification) return "This plan is unverified because it reports no verification commands. It cannot run.";
    if (dispatchedRun) return `This plan already started run ${dispatchedRun}.`;
    if (dispatching) return "The run is starting.";
    return null;
  });
  const showPlanPanel = $derived(planning || planAttempted || !!plan);
  const dispatchedLabel = $derived(
    dispatchedStatus === "verify_failed"
      ? "Verification failed"
      : dispatchedStatus
        ? `${dispatchedStatus.charAt(0).toUpperCase()}${dispatchedStatus.slice(1)}`
        : "Running",
  );

  $effect(() => {
    const runId = dispatchedRun;
    const observed = runs.find((run) => run.id === runId);
    if (!runId || !observed) return;
    dispatchedStatus = observed.status.toLowerCase();
    if (
      ["cancelled", "completed", "failed", "verify_failed"].includes(dispatchedStatus) &&
      historySyncedRun !== runId
    ) {
      historySyncedRun = runId;
      void loadHistory();
    }
  });

  $effect(() => {
    const inputKey = currentInputKey;
    if (planInputKey && planInputKey !== inputKey) planInputKey = null;
  });

  $effect(() => {
    const currentMission = mission;
    if (factsMissionText === currentMission) return;
    factsMissionText = currentMission;
    reviewedTaskKind = "other";
    reviewedContextComplete = false;
    crossComponent = "unknown";
    repeatableSymptom = "unknown";
    specificCauseHypothesis = "unknown";
  });

  $effect(() => {
    const domainIds = domains.map((domain) => domain.domain_id);
    void loadWorkspaceRoutingGrants(domainIds);
  });

  function isAdeReady(cli: AdeCliStatusDto) {
    return isAdeRunnable(cli);
  }

  function verificationCommandsFor(candidate: FlowPlan | null) {
    return candidate
      ? [...new Set(candidate.tasks.flatMap((task) => task.verify ?? []).filter((command) => command.trim()))]
      : [];
  }

  function reviewedRoutedRunId(candidate: unknown): string | null {
    if (!candidate || typeof candidate !== "object" || !("routing" in candidate)) return null;
    const routing = candidate.routing;
    if (!routing || typeof routing !== "object" || !("authorization" in routing)) return null;
    const authorization = routing.authorization;
    if (!authorization || typeof authorization !== "object" || !("run_id" in authorization)) return null;
    return typeof authorization.run_id === "string" && authorization.run_id.length > 0 ? authorization.run_id : null;
  }

  function savedRoutedRunId(draft: FlowDraftRecord): string | null {
    if (draft.status !== "dispatching" || !draft.plan_json) return null;
    try { return reviewedRoutedRunId(JSON.parse(draft.plan_json)); }
    catch { return null; }
  }

  function hasPersistedRoutedReview(draft: FlowDraftRecord) {
    if (draft.status !== "ready" || draft.dispatched_run_id || !draft.plan_json) return false;
    try {
      const plan: unknown = JSON.parse(draft.plan_json);
      return !!plan && typeof plan === "object" && "routing" in plan && !!plan.routing;
    } catch {
      return false;
    }
  }

  async function loadHistory() {
    const generation = ++historyGeneration;
    const drafts = await backend.flowHistory();
    if (packetDisposed || generation !== historyGeneration) return;
    history = drafts;
    packetEligible = {};
    closePacket();
    for (const draft of drafts.slice(0, 6).filter(hasPersistedRoutedReview)) {
      void backend.previewFlowAdvisorPacket(draft.id).then((preview) => {
        if (packetDisposed || generation !== historyGeneration || preview.domain_id !== draft.domain_id) return;
        packetEligible = { ...packetEligible, [draft.id]: true };
      }).catch(() => {
        // Rules, stale, and incomplete reviews do not get a packet control.
      });
    }
  }

  async function revealPacket(draft: FlowDraftRecord) {
    const generation = ++packetOpenGeneration;
    packetDraftId = draft.id;
    packetPreview = null;
    proposedHostedPacket = null;
    proposedHostedBody = "";
    proposedHostedError = "";
    packetConsent = null;
    packetBody = "";
    packetNotice = "";
    packetError = "";
    packetLoading = true;
    try {
      const preview = await backend.previewFlowAdvisorPacket(draft.id);
      if (packetDisposed || generation !== packetOpenGeneration || packetDraftId !== draft.id) return;
      if (preview.domain_id !== draft.domain_id) throw new Error("Packet workspace changed");
      const body = new TextDecoder("utf-8", { fatal: true }).decode(Uint8Array.from(preview.request_body));
      JSON.parse(body);
      const consent = await backend.flowAdvisorConsent(preview.domain_id);
      if (packetDisposed || generation !== packetOpenGeneration || packetDraftId !== draft.id) return;
      if (consent.domain_id !== preview.domain_id || consent.recipient_identity !== preview.recipient_identity) throw new Error("Routing recipient changed");
      packetPreview = preview;
      packetConsent = consent;
      packetBody = body;
      try {
        const proposed = await backend.previewProposedHostedPacket(draft.id);
        if (packetDisposed || generation !== packetOpenGeneration || packetDraftId !== draft.id) return;
        if (proposed.domain_id !== preview.domain_id || proposed.run_id !== preview.run_id || proposed.task_id !== preview.task_id || proposed.packet_digest !== preview.packet_digest || proposed.source_review_recipient_identity !== preview.recipient_identity) throw new Error("Proposed packet differs from saved review");
        const proposedBody = new TextDecoder("utf-8", { fatal: true }).decode(Uint8Array.from(proposed.packet_body));
        JSON.parse(proposedBody);
        proposedHostedPacket = proposed;
        proposedHostedBody = proposedBody;
      } catch {
        if (!packetDisposed && generation === packetOpenGeneration && packetDraftId === draft.id) proposedHostedError = "Proposed hosted packet is unavailable for this review. Local Shadow access is unchanged.";
      }
    } catch {
      if (packetDisposed || generation !== packetOpenGeneration || packetDraftId !== draft.id) return;
      packetEligible = Object.fromEntries(Object.entries(packetEligible).filter(([id]) => id !== draft.id));
      packetError = "This saved review no longer has a current Shadow packet. Rebuild the review before inspecting it.";
    } finally {
      if (!packetDisposed && generation === packetOpenGeneration) packetLoading = false;
    }
  }

  async function changePacketConsent(draft: FlowDraftRecord, enabled: boolean) {
    const preview = packetPreview;
    const consent = packetConsent;
    if (!preview || !consent || packetDraftId !== draft.id || packetActionPending) return;
    const generation = packetOpenGeneration;
    packetActionPending = true;
    packetNotice = "";
    packetError = "";
    try {
      const updated = enabled
        ? await backend.enableFlowAdvisorConsent(draft.id, preview.domain_id, preview.request_digest, preview.recipient_identity, consent.revision)
        : await backend.revokeFlowAdvisorConsent(preview.domain_id, consent.revision);
      if (packetDisposed || generation !== packetOpenGeneration || packetDraftId !== draft.id) return;
      if (updated.domain_id !== preview.domain_id || updated.recipient_identity !== preview.recipient_identity || updated.revision !== consent.revision + 1 || updated.enabled !== enabled) throw new Error("Routing grant changed");
      packetConsent = updated;
      await loadWorkspaceRoutingGrants(domains.map((domain) => domain.domain_id));
      packetNotice = enabled
        ? "Local Shadow fixture allowed for this workspace. No request was sent."
        : "Local Shadow fixture revoked. This saved review now needs a new consent revision before use.";
    } catch {
      if (packetDisposed || generation !== packetOpenGeneration || packetDraftId !== draft.id) return;
      packetConsent = null;
      packetError = "The reviewed packet or workspace grant changed. Close and inspect it again before changing access.";
    } finally {
      if (!packetDisposed && generation === packetOpenGeneration) packetActionPending = false;
    }
  }

  async function loadWorkspaceRoutingGrants(domainIds: string[]) {
    const generation = ++workspaceGrantGeneration;
    let pinnedIds: string[];
    try {
      pinnedIds = await backend.flowAdvisorConsentDomains();
    } catch {
      if (!packetDisposed && generation === workspaceGrantGeneration) {
        workspaceGrants = {};
        workspaceGrantReadError = "Workspace routing access could not be checked. Experimental routing stays unavailable.";
      }
      return;
    }
    const pinned = new Set(pinnedIds);
    const results = await Promise.all([...new Set([...domainIds, ...pinnedIds])].map(async (domainId) => {
      try { return { grant: await backend.flowAdvisorConsent(domainId), failedPinned: false }; }
      catch { return { grant: null, failedPinned: pinned.has(domainId) }; }
    }));
    if (packetDisposed || generation !== workspaceGrantGeneration) return;
    workspaceGrants = Object.fromEntries(results.map(({ grant }) => grant).filter((grant): grant is RoutedAdvisorConsentStatus => !!grant && grant.enabled).map((grant) => [grant.domain_id, grant]));
    workspaceGrantReadError = results.some((result) => result.failedPinned)
      ? "A saved workspace routing grant could not be checked. Experimental routing stays unavailable until its Store is recovered."
      : "";
  }

  async function revokeWorkspaceGrant(grant: RoutedAdvisorConsentStatus) {
    if (workspaceGrantRevoking) return;
    workspaceGrantRevoking = grant.domain_id;
    workspaceGrantError = "";
    workspaceGrantNotice = "";
    try {
      const updated = await backend.revokeFlowAdvisorConsent(grant.domain_id, grant.revision);
      if (packetDisposed) return;
      if (updated.domain_id !== grant.domain_id || updated.revision !== grant.revision + 1 || updated.enabled) throw new Error("Routing grant changed");
      if (packetPreview?.domain_id === grant.domain_id) packetConsent = updated;
      await loadWorkspaceRoutingGrants(domains.map((domain) => domain.domain_id));
      workspaceGrantNotice = "Workspace routing access revoked.";
    } catch {
      if (!packetDisposed) workspaceGrantError = "Routing access changed. Refresh this workspace before revoking it.";
    } finally {
      if (!packetDisposed) workspaceGrantRevoking = null;
    }
  }

  function closePacket() {
    ++packetOpenGeneration;
    packetDraftId = null;
    packetPreview = null;
    proposedHostedPacket = null;
    proposedHostedBody = "";
    proposedHostedError = "";
    packetConsent = null;
    packetBody = "";
    packetNotice = "";
    packetError = "";
    packetLoading = false;
    packetActionPending = false;
  }

  function adeUnavailableReason(cli: AdeCliStatusDto) {
    return adeAvailabilityLabel(cli);
  }

  async function loadAdeClis() {
    adeLoading = true;
    adeError = "";
    try {
      const [detected, experiment, hosted] = await Promise.all([
        backend.listAdeClis(),
        backend.experimentalClaudeRoutingAvailable().catch(() => false),
        backend.experimentalHostedReviewAvailable().catch(() => false),
      ]);
      adeClis = detected;
      experimentalClaudeAvailable = experiment;
      experimentalHostedAvailable = hosted;
      const saved = typeof localStorage === "undefined" ? null : localStorage.getItem(ADE_CHOICE_KEY);
      const savedReady = saved && saved !== CLAUDE_ROUTE_CHOICE && saved !== CLAUDE_HOSTED_CHOICE ? adeClis.find((cli) => cli.id === saved && isAdeReady(cli))?.id : null;
      const currentReady = selectedAde === CLAUDE_ROUTE_CHOICE && experiment ? CLAUDE_ROUTE_CHOICE : selectedAde === CLAUDE_HOSTED_CHOICE && hosted ? CLAUDE_HOSTED_CHOICE : adeClis.find((cli) => cli.id === selectedAde && isAdeReady(cli))?.id;
      // An explicit agent choice must never silently become a different agent.
      const requested = preferredAdeId ?? draft?.adeId;
      selectedAde = adeSelectionInitialized || requested
        ? currentReady ?? ""
        : savedReady ?? currentReady ?? adeClis.find((cli) => cli.id === "codex" && isAdeReady(cli))?.id ?? "";
      if (requested && !selectedAde) adeError = "The chosen agent is no longer ready. Recheck it or choose another agent explicitly.";
      adeSelectionInitialized = true;
    } catch (cause) {
      adeClis = [];
      experimentalClaudeAvailable = false;
      experimentalHostedAvailable = false;
      selectedAde = "";
      adeError = cause instanceof Error ? cause.message : String(cause);
    } finally {
      adeLoading = false;
    }
  }

  function chooseAde(value: string) {
    selectedAde = value;
    taskAdes = {};
    if (typeof localStorage !== "undefined" && value !== CLAUDE_ROUTE_CHOICE && value !== CLAUDE_HOSTED_CHOICE) localStorage.setItem(ADE_CHOICE_KEY, value);
  }

  $effect(() => {
    const preferred = preferredDomainId;
    const list = domains;
    const selected = untrack(() => selectedDomainId);
    if (!domainSelectionInitialized && list.length) {
      const retained = untrack(() => draft);
      // A mounted composer owns its scope; shell navigation must not retarget it.
      selectedDomainId = retained
        ? list.find((domain) => domain.domain_id === retained.domainId)?.domain_id ?? ""
        : list.find((domain) => domain.domain_id === preferred)?.domain_id ?? list[0]?.domain_id ?? "";
      domainSelectionInitialized = true;
      if (retained && !selectedDomainId) draftNotice = "The draft's workspace is no longer available. Select a workspace explicitly and build a fresh plan.";
    } else if (selected && !list.some((domain) => domain.domain_id === selected)) {
      selectedDomainId = "";
      draftNotice = "The selected workspace is no longer available. Select a workspace explicitly and build a fresh plan.";
    }
  });

  async function buildPlan() {
    if (buildPlanDisabledReason) return;
    const requestedInputKey = currentInputKey;
    planning = true; error = "";
    compactPane = "plan";
    planAttempted = true;
    // A new preview request revokes the previous dispatch authority immediately.
    plan = null;
    planInputKey = null;
    draftConsumed = false;
    dispatchedRun = "";
    dispatchedStatus = "";
    historySyncedRun = "";
    try {
      const input = { id: crypto.randomUUID(), title: draftTitle(mission), mission_text: mission, source: missionSource, domain_id: selectedDomainId || null, project_id: null, ade_id: routedChoice ? null : selectedAde, ade_ids: team.length > 1 ? team : undefined, task_ades: Object.keys(taskAdes).length ? taskAdes : undefined, max_workers: routedChoice ? 1 : workers, verification_commands: runChecks.split(/\r?\n/).map((command) => command.trim()).filter(Boolean) };
      const facts: ReviewedDemandFacts = {
        task_kind: reviewedTaskKind,
        context_complete: reviewedContextComplete,
        cross_component_requirement: crossComponent === "unknown" ? null : crossComponent === "yes",
        repeatable_symptom_supplied: reviewedTaskKind === "diagnosis" && repeatableSymptom !== "unknown" ? repeatableSymptom === "yes" : null,
        specific_cause_hypothesis_supplied: reviewedTaskKind === "diagnosis" && specificCauseHypothesis !== "unknown" ? specificCauseHypothesis === "yes" : null,
      };
      plan = selectedAde === CLAUDE_HOSTED_CHOICE
        ? await backend.previewExperimentalClaudeHostedFlow(input, facts)
        : selectedAde === CLAUDE_ROUTE_CHOICE
        ? await backend.previewExperimentalClaudeFlow(input, facts)
        : await backend.previewFlow(input);
      planInputKey = requestedInputKey;
      await loadHistory();
    } catch (cause) {
      error = cause instanceof Error ? cause.message : String(cause);
    } finally { planning = false; }
  }

  async function dispatch() {
    if (!plan || !canDispatchPlan) return;
    if (!domains.length || !selectedDomainId) {
      error = "Add a workspace first, then pick it below.";
      return;
    }
    dispatching = true;
    reviewedRouteReady = false;
    routedStopRequested = false;
    routedStopNotice = "";
    error = "";
    try {
      const reviewedPlan = await backend.saveReviewedFlow(plan);
      plan = reviewedPlan;
      reviewedRouteReady = !!reviewedRoutedRunId(reviewedPlan);
      if (reviewedPlan.status !== "ready" || verificationCommandsFor(reviewedPlan).length === 0) {
        error = reviewedPlan.status !== "ready"
          ? "The reviewed plan is blocked and was not dispatched."
          : "The reviewed plan reports no verification commands, so it remains unverified and was not dispatched.";
        return;
      }
      dispatchedRun = await backend.dispatchFlow(plan.draft_id);
      dispatchedStatus = "running";
      draftConsumed = true;
      // Consume the draft while this component is still live. Destruction
      // happens during parent navigation and must not own successful cleanup.
      onDraftChange(null);
      onDispatched?.(dispatchedRun);
    } catch (cause) {
      error = routedStopRequested
        ? "Stop was requested. Check the saved request for the final state before trying again."
        : cause instanceof Error ? cause.message : String(cause);
      // Dispatch can fail after its single-use claim. Refresh durable status
      // and require a new preview if the status read is unavailable.
      planInputKey = null;
      try { await loadHistory(); }
      catch { history = []; packetEligible = {}; closePacket(); }
    } finally {
      dispatching = false;
      reviewedRouteReady = false;
    }
  }

  async function stopRoutedDispatch() {
    const runId = reviewedRoutedRunId(plan);
    if (!plan || !dispatching || !reviewedRouteReady || !runId || routedStopPending || routedStopRequested) return;
    routedStopPending = true;
    routedStopNotice = "";
    try {
      await backend.stopRoutedFlow(plan.draft_id, runId);
      routedStopRequested = true;
      routedStopNotice = "Stop requested. Waiting for the run and any probes to settle.";
    } catch (cause) {
      routedStopNotice = `Stop could not be confirmed: ${cause instanceof Error ? cause.message : String(cause)}`;
    } finally {
      routedStopPending = false;
    }
  }

  async function stopSavedRoutedDispatch(draft: FlowDraftRecord) {
    const runId = savedRoutedRunId(draft);
    if (!runId || historyStopPending) return;
    historyStopPending = draft.id;
    historyStopNotice = "";
    try {
      await backend.stopRoutedFlow(draft.id, runId);
      historyStopNotice = "Stop requested. Check the saved request for its final state.";
      await loadHistory();
    } catch (cause) {
      historyStopNotice = `Stop could not be confirmed: ${cause instanceof Error ? cause.message : String(cause)}`;
    } finally {
      historyStopPending = "";
    }
  }

  async function toggleVoice() {
    error = "";
    try {
      if (!voiceSessionId) {
        const session = await backend.startVoice(selectedVoiceDevice, "en");
        if (voiceDisposed) { await backend.cancelVoice(session.session_id); return; }
        voiceSessionId = session.session_id;
        voiceState = session.state;
      } else {
        voiceState = "transcribing";
        const session = await backend.finishVoice(voiceSessionId);
        voiceState = session.state;
        voiceSessionId = null;
        if (session.state === "ready") {
          voiceSegments = session.transcript_segments;
          mission = voiceSegments.map((segment) => segment.text).join(" ");
          missionSource = "voice";
        }
        else if (session.error) error = session.error;
      }
    } catch (cause) { voiceState = "failed"; voiceSessionId = null; error = String(cause); }
  }

  function handleVoicePointerDown(event: PointerEvent) {
    if (voiceCapture === "click") return;
    if (event.button !== 0 || voiceState === "transcribing") return;
    try { (event.currentTarget as HTMLButtonElement | null)?.setPointerCapture(event.pointerId); } catch { /* Synthetic and unsupported pointer sources fall back to the button handler. */ }
    voicePointerStartedAt = performance.now();
    voicePointerHadSession = !!voiceSessionId;
    if (!voicePointerHadSession) voicePointerAction = toggleVoice();
  }

  async function handleVoicePointerCancel() {
    await voicePointerAction;
    if (voiceSessionId) await cancelVoice();
    voicePointerStartedAt = 0;
    voicePointerHadSession = false;
    voicePointerAction = null;
    suppressVoiceClick = true;
  }

  async function handleVoicePointerUp() {
    if (!voicePointerStartedAt) return;
    const held = voiceCapture === "hold" || performance.now() - voicePointerStartedAt >= 350;
    if (held && !voicePointerHadSession) {
      suppressVoiceClick = true;
      await voicePointerAction;
      if (voiceSessionId) await toggleVoice();
    }
  }

  function handleVoiceClick(event: MouseEvent) {
    if (voiceCapture === "click" || event.detail === 0) {
      void toggleVoice();
    } else if (suppressVoiceClick) {
      suppressVoiceClick = false;
    } else if (voicePointerHadSession) {
      void toggleVoice();
    }
    voicePointerStartedAt = 0;
    voicePointerHadSession = false;
    voicePointerAction = null;
  }

  async function pauseOrResumeVoice() {
    if (!voiceSessionId) return;
    const session = voiceState === "paused"
      ? await backend.resumeVoice(voiceSessionId)
      : await backend.pauseVoice(voiceSessionId);
    voiceState = session.state;
  }

  async function cancelVoice() {
    if (!voiceSessionId) return;
    const session = await backend.cancelVoice(voiceSessionId);
    voiceState = session.state;
    voiceSessionId = null;
  }

  function correctSegment(index: number, text: string) {
    voiceSegments = voiceSegments.map((segment, segmentIndex) => segmentIndex === index ? { ...segment, text, uncertain: false } : segment);
    mission = voiceSegments.map((segment) => segment.text).join(" ");
  }

  function editTask(taskId: string, prompt: string) {
    if (plan?.routing) return;
    if (plan) plan = { ...plan, tasks: plan.tasks.map((task) => task.id === taskId ? { ...task, prompt } : task) };
  }

  async function deleteDraft(draftId: string) {
    try {
      await backend.deleteFlowDraft(draftId);
      await loadHistory();
    } catch (cause) {
      draftNotice = cause instanceof Error ? cause.message : String(cause);
    }
  }

  const planSummary = $derived(
    plan
      ? `${plan.tasks.length} task${plan.tasks.length === 1 ? "" : "s"} · ${plan.waves.length} step${plan.waves.length === 1 ? "" : "s"} · ${[...new Set(plan.tasks.map((t) => adeName(t.ade_id ?? plan!.ade.requested)))].join(" + ") || "no agent assigned"}`
      : "",
  );

  function useTemplate(value: string) {
    mission = value;
    missionSource = "text";
    queueMicrotask(() => {
      missionInput?.focus();
      const start = value.indexOf("[");
      if (start >= 0) missionInput?.setSelectionRange(start, value.indexOf("]", start) + 1);
    });
  }

  function handleVoiceKeyDown(event: KeyboardEvent) {
    if (voiceCapture !== "hold" || ![" ", "Enter"].includes(event.key)) return;
    event.preventDefault();
    if (!event.repeat && !voiceSessionId && !voicePointerAction) {
      keyboardVoiceHeld = true;
      voicePointerAction = toggleVoice();
    }
  }
  async function handleVoiceKeyUp(event: KeyboardEvent) {
    if (voiceCapture !== "hold" || ![" ", "Enter"].includes(event.key)) return;
    event.preventDefault();
    if (!keyboardVoiceHeld) return;
    keyboardVoiceHeld = false;
    const action = voicePointerAction;
    await action;
    voicePointerAction = null;
    if (!voiceDisposed && voiceSessionId) await toggleVoice();
  }

  async function cancelKeyboardVoice() {
    if (!keyboardVoiceHeld) return;
    keyboardVoiceHeld = false;
    const action = voicePointerAction;
    await action;
    voicePointerAction = null;
    if (!voiceDisposed && voiceSessionId) await cancelVoice();
  }

  function restoreDraft(draft: FlowDraftRecord) {
    if (planning || dispatching) return;
    mission = draft.mission_text;
    runChecks = "";
    reviewedTaskKind = "other";
    reviewedContextComplete = false;
    crossComponent = "unknown";
    repeatableSymptom = "unknown";
    specificCauseHypothesis = "unknown";
    missionSource = draft.source === "voice" ? "voice" : "text";
    selectedDomainId = !draft.project_id && domains.some((domain) => domain.domain_id === draft.domain_id) ? draft.domain_id! : "";
    // Reuse guidance only. A saved plan is never restored as dispatch authority.
    plan = null;
    planInputKey = null;
    planAttempted = false;
    draftConsumed = false;
    dispatchedRun = "";
    dispatchedStatus = "";
    historySyncedRun = "";
    error = "";
    let originalAde: string | null = null;
    try {
      const saved: unknown = JSON.parse(draft.plan_json ?? "null");
      if (saved && typeof saved === "object" && "ade" in saved) {
        const ade = saved.ade;
        if (ade && typeof ade === "object" && "requested" in ade && typeof ade.requested === "string") originalAde = ade.requested;
      }
    } catch { /* Older drafts may have no readable plan. Ask for a fresh one. */ }
    if (originalAde) {
      adeSelectionInitialized = true;
      const available = adeClis.find((cli) => cli.id === originalAde && isAdeReady(cli));
      if (available) chooseAde(available.id);
      else selectedAde = "";
    }
    draftNotice = [
      selectedDomainId ? "Original workspace selected." : "Original workspace is unavailable in this form. Select a workspace explicitly.",
      originalAde && !selectedAde ? "The original agent CLI is unavailable. Select a ready CLI." : "Confirm the selected agent CLI.",
      "Build a fresh plan before running; previous checks and approvals do not carry over.",
    ].join(" ");
    queueMicrotask(() => missionInput?.focus());
  }

  function describeBlocker(reason: unknown) {
    if (typeof reason === "string") return reason;
    if (!reason || typeof reason !== "object") return "The planner did not provide a blocker description. Rebuild after checking the workspace and agent CLI.";
    const value = reason as Record<string, unknown>;
    for (const key of ["message", "reason", "detail"]) {
      if (typeof value[key] === "string" && value[key]) return value[key] as string;
    }
    if (value.kind === "path_claim_overlap") return "Two tasks claim the same path. Split their ownership or revise the mission, then rebuild.";
    if (value.kind === "ade_unavailable") return "The selected agent CLI is unavailable. Choose an installed, signed-in CLI, then rebuild.";
    const kind = typeof value.kind === "string" ? value.kind.replaceAll("_", " ") : "Planner constraint";
    return `${kind}. Adjust the mission, workspace, or agent CLI, then rebuild.`;
  }

  onMount(() => {
    const capture = localStorage.getItem("pytxo-desktop-voice-capture-v1");
    if (capture === "hold" || capture === "click") voiceCapture = capture;
    let disposed = false;
    let unlisten: (() => void) | null = null;
    void backend.onVoiceProgress((event) => {
      if (voiceSessionId && event.session_id !== voiceSessionId) return;
      if (event.kind === "capture_state") voiceState = event.state;
      if (event.kind === "transcription_progress") transcriptionProgress = event.progress;
      if (event.kind === "partial_transcript") {
        voiceSegments = [...voiceSegments, { text: event.text, confidence: event.confidence, uncertain: event.confidence < 0.7 }];
        mission = voiceSegments.map((segment) => segment.text).join(" ");
        missionSource = "voice";
      }
    }).then((stop) => { if (disposed) stop(); else unlisten = stop; });
    void backend.voiceAvailable().then((available) => (voiceAvailable = available));
    void backend.listVoiceDevices().then((devices) => {
      voiceDevices = devices.length ? devices : ["default"];
      selectedVoiceDevice = voiceDevices[0];
    }).catch(() => (voiceDevices = ["default"]));
    void loadHistory();
    if (previewState !== "draft") mission = "Make the desktop shell production ready";
    else if (draft?.mission && !draftNotice) draftNotice = "Draft restored in this window. Build a fresh plan before running.";
    if (["recording", "paused", "transcribing", "cancelled", "failed"].includes(previewState)) voiceState = previewState as VoiceState;
    if (previewState === "recording" || previewState === "paused") voiceSessionId = "preview-session";
    if (previewState === "cancelled") error = "Voice capture cancelled. No audio was retained.";
    if (previewState === "failed") error = "The input device was lost. Choose another device and try again.";
    if (previewState === "uncertain") {
      voiceState = "ready";
      voiceSegments = [{ text: "Update the desktop approval flow", confidence: 0.52, uncertain: true }];
      mission = voiceSegments[0].text;
      missionSource = "voice";
    }
    void loadAdeClis().then(() => {
      if (!["ready", "blocked", "dispatched"].includes(previewState)) return;
      void buildPlan().then(() => {
        if (plan && previewState === "blocked") plan = { ...plan, status: "blocked", blocked_reasons: [{ kind: "path_claim_overlap" }] };
        if (previewState === "dispatched") dispatchedRun = "run-preview";
      });
    });
    if (previewState === "planning") planning = true;
    return () => {
      disposed = true;
      voiceDisposed = true;
      unlisten?.();
      if (voiceSessionId) void backend.cancelVoice(voiceSessionId);
    };
  });

  onDestroy(() => {
    packetDisposed = true;
    ++historyGeneration;
    ++packetOpenGeneration;
    if (draftConsumed) return;
    onDraftChange(dispatchedRun || !mission.trim() ? null : {
      mission,
      source: missionSource,
      domainId: selectedDomainId,
      adeId: selectedAde,
      verificationCommands: runChecks,
    });
  });
</script>

<svelte:window onblur={() => void cancelKeyboardVoice()} />

<section class="screen flow-screen">
  <header class="screen-heading">
    <div>
      <h1>New work</h1>
      <p class="mission-intro">Tell Pytxo what to build or fix. You’ll see the plan before any agent starts.</p>
    </div>
    {#if domains.length}
      <div class="composer-context">
        <label><span>Project</span><select bind:value={selectedDomainId} aria-label="Project">{#if !selectedDomainId}<option value="" disabled>Select a project</option>{/if}{#each domains as domain}<option value={domain.domain_id}>{domain.repo_root.split(/[\\/]/).pop()}{domain.project_id ? " · primary folder" : ""}</option>{/each}</select></label>
        <label><span>Agent</span><select value={selectedAde} onchange={(event) => chooseAde(event.currentTarget.value)} aria-label="Agent CLI" disabled={adeLoading || (!adeClis.length && !experimentalClaudeAvailable && !experimentalHostedAvailable)}>{#if adeLoading}<option value="">Checking detected CLIs…</option>{:else if !adeClis.length && !experimentalClaudeAvailable && !experimentalHostedAvailable}<option value="">No ready CLI detected</option>{:else}{#if !selectedAde}<option value="" disabled>Select a ready CLI</option>{/if}<optgroup label="Beta agents">{#each betaAdesInOrder(adeClis) as cli (cli.id)}<option value={cli.id} disabled={!isAdeReady(cli)}>{cli.display_name}{isAdeReady(cli) ? "" : ` — ${adeUnavailableReason(cli)}`}</option>{/each}</optgroup><optgroup label="Additional agents">{#each adeClis.filter(cli => !isBetaAde(cli.id)) as cli (cli.id)}<option value={cli.id} disabled={!isAdeReady(cli)}>{cli.display_name}{isAdeReady(cli) ? "" : ` — ${adeUnavailableReason(cli)}`}</option>{/each}</optgroup>{#if experimentalClaudeAvailable || experimentalHostedAvailable}<optgroup label="Experimental">{#if experimentalClaudeAvailable}<option value={CLAUDE_ROUTE_CHOICE}>Claude proposal route · subscription</option>{/if}{#if experimentalHostedAvailable}<option value={CLAUDE_HOSTED_CHOICE}>Hosted Routing packet review · no run</option>{/if}</optgroup>{/if}{/if}</select></label>
      </div>
    {/if}
  </header>

  {#if !domains.length}
    <div class="panel">
      <div class="empty flow-empty">
        <strong>Add a project</strong>
        <p>Select a repository folder, then choose the agent CLI you already use.</p>
        {#if onAddWorkspace}
          <button class="primary" onclick={() => void onAddWorkspace()}>
            <IconFolderPlus size={16} /> Add project
          </button>
        {/if}
      </div>
    </div>
  {:else}
  {#if showPlanPanel}<div class="compact-pane-switch" role="group" aria-label="New work pane"><button class:active={compactPane === "request"} aria-pressed={compactPane === "request"} onclick={() => compactPane = "request"}>Request</button><button class:active={compactPane === "plan"} aria-pressed={compactPane === "plan"} onclick={() => compactPane = "plan"}>Plan</button></div>{/if}
  <div class="flow-layout" class:flow-layout--solo={!showPlanPanel}>
    <article class="panel composer-panel" class:compact-hidden={showPlanPanel && compactPane !== "request"}>
      <div class="panel-head">
        <div>
          <h2>Describe the job</h2>
        </div>
      </div>
      <p id="mission-guidance" class="mission-guidance">Name the file or folder if you know it, the expected behavior, and anything that must stay unchanged.</p>
      <textarea bind:this={missionInput} bind:value={mission} aria-label="What should Pytxo do?" aria-describedby="mission-guidance" placeholder="Fix the parser in src/parser.rs so empty input returns an error. Add a regression test. Keep the public API unchanged."></textarea>
      <div class="composer-command-bar"><span>{mission.trim() ? `${mission.trim().length} characters` : ""}</span><button class="primary" disabled={!!buildPlanDisabledReason} aria-describedby={buildPlanDisabledReason ? "build-plan-disabled-reason" : undefined} onclick={buildPlan}>{planning ? "Building…" : "Build plan"}</button>{#if buildPlanDisabledReason}<small id="build-plan-disabled-reason" class="action-reason">{buildPlanDisabledReason}</small>{/if}</div>
      <details class="checks-editor" open={!!runChecks}>
      <summary>Verification commands <span>{runChecks.trim() ? "Custom checks added" : "Use project checks or add your own"}</span></summary>
      <label class="run-checks">
        <span>Additional verification commands</span>
        <textarea bind:value={runChecks} rows="2" aria-describedby="run-checks-help" placeholder="One command per line, for example: npm test"></textarea>
      </label>
      <p id="run-checks-help" class="mission-guidance">Optional when Pytxo finds checks in your project. These commands are added to every task and checked again on the combined candidate. Changing them requires a new plan; your configuration stays unchanged.</p>
      </details>
      {#if selectedAde === CLAUDE_ROUTE_CHOICE || selectedAde === CLAUDE_HOSTED_CHOICE}
        <details class="checks-editor" open>
          <summary>Experimental routing facts <span>Reviewed labels only; unknown work uses the strong profile</span></summary>
          <label><span>Reviewed task type</span><select bind:value={reviewedTaskKind} aria-label="Reviewed task type">
            <option value="other">Unknown or mixed</option>
            <option value="documentation">Documentation</option>
            <option value="formatting">Formatting</option>
            <option value="rename">Rename</option>
            <option value="local_transformation">Specified local change</option>
            <option value="diagnosis">Diagnosis</option>
            <option value="architecture">Architecture</option>
          </select></label>
          <label><input type="checkbox" bind:checked={reviewedContextComplete} aria-label="Task context is complete" /> Task context is complete</label>
          <label><span>Cross-component requirement</span><select bind:value={crossComponent} aria-label="Cross-component requirement">
            <option value="unknown">Unknown</option>
            <option value="yes">Yes, behavior spans components</option>
            <option value="no">No, one component</option>
          </select></label>
          {#if reviewedTaskKind === "diagnosis"}
            <label><span>Repeatable symptom supplied</span><select bind:value={repeatableSymptom} aria-label="Repeatable symptom supplied">
              <option value="unknown">Unknown</option>
              <option value="yes">Yes, the request states a repeatable symptom</option>
              <option value="no">No, the request does not supply one</option>
            </select></label>
            <label><span>Specific cause hypothesis supplied</span><select bind:value={specificCauseHypothesis} aria-label="Specific cause hypothesis supplied">
              <option value="unknown">Unknown</option>
              <option value="yes">Yes, the request states a specific hypothesis</option>
              <option value="no">No, the request does not supply one</option>
            </select></label>
          {/if}
          <p class="mission-guidance">Choose only what the reviewed request establishes. These coarse labels enter the packet; the task text and file paths stay local. Architecture or cross-component work requires the strong profile.</p>
        </details>
      {/if}
      <div class="flow-templates">
        <span>Start with</span>
        <button type="button" onclick={() => useTemplate("Fix [existing file path] so [expected behavior]. Reproduce the failure, make the smallest repair, and add a regression test in [test file path]. Preserve unrelated changes.")}>Fix a failure</button>
        <button type="button" onclick={() => useTemplate("Implement [expected behavior] in [existing file path]. Keep the public API unchanged and add coverage in [test file path]. Preserve unrelated changes.")}>Build a feature</button>
        <button type="button" onclick={() => useTemplate("Add regression coverage in [test file path] for [expected behavior] in [existing file path]. Keep production behavior unchanged.")}>Add coverage</button>
      </div>
      {#if draftNotice}<p class="draft-notice" role="status">{draftNotice}</p>{/if}
      {#if voiceState === "cancelled" || voiceState === "failed"}
        <p class:error={voiceState === "failed"} class="voice-state-message">
          {voiceState === "cancelled" ? "Voice capture cancelled · no audio retained" : "Voice capture failed · choose another input device"}
        </p>
      {/if}
      {#if voiceSegments.some((segment) => segment.uncertain)}
        <div class="uncertain-segments">
          <span>Review uncertain transcript</span>
          {#each voiceSegments as segment, index}
            {#if segment.uncertain}
              <label>
                <input aria-label={`Correct uncertain segment ${index + 1}`} value={segment.text} oninput={(event) => correctSegment(index, event.currentTarget.value)} />
                <small>{Math.round(segment.confidence * 100)}% confidence</small>
              </label>
            {/if}
          {/each}
        </div>
      {/if}
      <div class="composer-actions">
        {#if voiceAvailable}<details class="voice-disclosure"><summary>Voice input</summary><div class="voice-controls">
          <select class="voice-device" bind:value={selectedVoiceDevice} aria-label="Voice input device" disabled={!!voiceSessionId}>
            {#each voiceDevices as device}<option value={device}>{device}</option>{/each}
          </select>
          <button
            data-testid="voice-capture"
            class:recording
            class="voice-button"
            onclick={handleVoiceClick}
            onpointerdown={handleVoicePointerDown}
            onpointerup={handleVoicePointerUp}
            onpointercancel={handleVoicePointerCancel}
            onkeydown={handleVoiceKeyDown}
            onkeyup={handleVoiceKeyUp}
            onblur={() => void cancelKeyboardVoice()}
            aria-pressed={recording}
            disabled={voiceState === "transcribing"}
            title={voiceHint}
          >
            {#if recording}<IconPlayerRecord size={17} /> Finish recording
            {:else if voiceState === "paused"}<IconMicrophone size={17} /> Finish paused recording
            {:else if voiceState === "transcribing"}<IconLoader2 size={17} class="spin" /> Transcribing…
            {:else}<IconMicrophone size={17} /> {voiceCapture === "hold" ? "Hold to record" : "Start Voice"}{/if}
          </button>
          {#if voiceSessionId && (voiceState === "recording" || voiceState === "paused")}
            <button class="quiet" onclick={pauseOrResumeVoice}>{voiceState === "paused" ? "Resume" : "Pause"}</button>
          {/if}
          {#if voiceSessionId}<button class="quiet" onclick={cancelVoice}>Cancel</button>{/if}
        </div></details>{/if}
      </div>
      <div class="ade-readiness">
        <div class="readiness-summary" aria-live="polite">
        {#if adeError}
          <span class="error">Agent readiness could not be checked: {adeError}</span>
        {:else if selectedAde === CLAUDE_ROUTE_CHOICE && experimentalClaudeAvailable}
          <span>Claude proposal route: configured · account and model readiness pending Run</span>
        {:else if selectedAde === CLAUDE_HOSTED_CHOICE && experimentalHostedAvailable}
          <span>Hosted Routing review: configured · packet inspection only</span>
        {:else if selectedAdeStatus}
          <span class:ready={selectedAdeReady}>{selectedAdeStatus.display_name}: {adeLoading ? "checking…" : adeAvailabilityLabel(selectedAdeStatus)}</span>
        {:else if !adeLoading}
          <span class="error">Install and sign in to an agent CLI before building a plan.</span>
        {/if}
        <button class="quiet" onclick={loadAdeClis} disabled={adeLoading || planning || dispatching}>{adeLoading ? "Checking…" : "Check again"}</button>
        </div>
        {#if !routedChoice && selectedAde && teamCandidates.length}
          <fieldset class="agent-team">
            <legend>Also put to work</legend>
            {#each teamCandidates as cli (cli.id)}
              <label class="team-chip" class:on={teamAdes.includes(cli.id)}><input type="checkbox" checked={teamAdes.includes(cli.id)} onchange={() => toggleTeamAde(cli.id)} /><AdeIdentity id={cli.id} />{cli.display_name}</label>
            {/each}
          </fieldset>
        {/if}
        {#if !routedChoice}
          <div class="workers-row">
            <span id="workers-label">At most at once</span>
            <div class="stepper" role="group" aria-labelledby="workers-label">
              <button aria-label="Fewer workers at once" disabled={workers <= 1} onclick={() => stepWorkers(-1)}>−</button>
              <output aria-live="polite">{workers}</output>
              <button aria-label="More workers at once" disabled={workers >= DESKTOP_BETA_MAX_WORKERS} onclick={() => stepWorkers(1)}>+</button>
            </div>
            <small>Up to {DESKTOP_BETA_MAX_WORKERS}. Tasks that share files never run together.</small>
          </div>
        {/if}
        <p>{selectedAde === CLAUDE_HOSTED_CHOICE ? "Review the redacted packet and optional account grant. This path cannot run an agent or send a Jev request." : selectedAde === CLAUDE_ROUTE_CHOICE ? "A Claude subscription worker proposes a one-file change. Run checks the account and both models before starting; these probes may consume quota." : team.length > 1 ? `${team.length} agents share the tasks, each in its own isolated copy. Installed and signed in means ready to try; a successful run validates execution.` : "Each task runs in its own isolated copy. Installed and signed in means ready to try; a successful run validates execution."}</p>
        {#if selectedAde && !routedChoice && !isBetaAde(selectedAde)}<p>Your agent selection is preserved. Desktop Beta runs Codex, Claude Code, Cursor Agent, OpenCode and Antigravity; choose one of those to start a beta run.</p>{/if}
        {#if unavailableAdes.length}
          <details><summary>{unavailableAdes.length} other CLI{unavailableAdes.length === 1 ? "" : "s"} unavailable</summary><ul>{#each unavailableAdes as cli}<li>{cli.display_name} · {adeUnavailableReason(cli)}</li>{/each}</ul></details>
        {/if}
      </div>
    </article>

    {#if showPlanPanel}
      <article class="panel plan-panel" class:compact-hidden={compactPane !== "plan"} aria-busy={planning}>
        <div class="panel-head">
          <div>
          <h2>{planning ? "Building plan…" : plan ? (!planMatchesInputs ? "Review needs a new plan" : plan.status === "review_only" ? "Review-only routing packet" : plan.status === "ready" ? (planHasVerification ? "Review plan" : "Plan needs verification") : "Plan blocked") : "Plan"}</h2>
          </div>
          {#if plan}
            <span class:ready={planMatchesInputs && plan.status === "ready" && planHasVerification} class="plan-state" data-tone={!planMatchesInputs ? "unknown" : plan.status !== "ready" ? "blocked" : planHasVerification ? "ready" : "unknown"} role="status" aria-live="polite" aria-atomic="true">
              {!planMatchesInputs ? "Stale" : plan.status === "review_only" ? "Review only" : plan.status !== "ready" ? "Blocked" : planHasVerification ? "Ready" : "Unverified"}
            </span>
          {/if}
        </div>
        {#if planning}<div class="planning-strip" aria-hidden="true"><span>&gt; . : + * = x &gt; . : + * = x</span></div>{/if}
        {#if plan}
          <p class="plan-summary">{planSummary}</p>
          <p class="plan-explainer">Maximum concurrent workers: <strong>{plan.max_workers}</strong>. All {plan.tasks.length} approved tasks remain in scope.</p>
          {#if selectedAde === CLAUDE_ROUTE_CHOICE && plan.routing}
            <p class="plan-explainer">{#if plan.routing.authorization.limits.max_attempts === 2}This reviewed Rules route allows up to two Claude subscription worker calls: Haiku first, then one fresh Sonnet attempt only after an intact frozen-check failure. Each call can consume subscription quota; exact cost is unknown.{:else}This reviewed Rules route allows one Claude subscription worker call. Account and model probes can also consume quota; exact cost is unknown.{/if}</p>
          {/if}
          <p class="plan-explainer">This is the work your agents will do. Review the steps before starting.</p>
          {#if !planMatchesInputs}
            <div class="plan-stale" role="status">
              <IconAlertTriangle size={15} />
              <span><strong>Plan is stale</strong><small>The outcome, workspace, agent CLI, or checks changed. Build a matching plan before Run is available.</small></span>
            </div>
          {/if}
          <div class="plan-command-bar">
            <span>{plan.estimated_cost_usd !== null ? `$${plan.estimated_cost_usd.toFixed(2)} estimate` : "Cost not estimated"}</span>
            <button class="primary" disabled={!canDispatchPlan} aria-describedby={runDisabledReason ? "run-disabled-reason" : undefined} onclick={dispatch}>{dispatchedRun ? `${dispatchedLabel} ${dispatchedRun}` : dispatching ? "Starting…" : "Run"} <IconArrowRight size={16} /></button>
            {#if dispatching && reviewedRouteReady && reviewedRoutedRunId(plan)}
              <button class="quiet" disabled={routedStopPending || routedStopRequested} onclick={stopRoutedDispatch}>{routedStopPending ? "Requesting Stop…" : routedStopRequested ? "Stop requested" : "Stop starting run"}</button>
            {/if}
            {#if runDisabledReason}<small id="run-disabled-reason" class="action-reason">{runDisabledReason}</small>{/if}
          </div>
          {#if routedStopNotice}<p class="action-reason" role="status">{routedStopNotice}</p>{/if}
          {#each plan.waves as wave, waveIndex}
            <div class="plan-wave">
              <span>Step {waveIndex + 1}</span>
              {#each wave as taskId}
                {@const task = plan.tasks.find((item) => item.id === taskId)}
                {#if task}
                  <div>
                    <b>{String(plan.tasks.findIndex(item => item.id === taskId) + 1).padStart(2, "0")}</b>
                    <div class="plan-task">
                      <label><span>{task.id} · {task.agent}</span><textarea aria-label={`Task ${task.id} prompt`} rows="2" value={task.prompt || task.id} readonly={!!plan.routing} oninput={(event) => editTask(task.id, event.currentTarget.value)}></textarea></label>
                      {#if !plan.routing}
                        {@const taskAde = task.ade_id ?? plan.ade.requested}
                        <div class="task-agent">
                          {#if taskAde}<AdeIdentity id={taskAde} />{/if}
                          {#if team.length > 1}
                            <select aria-label={`Agent for ${task.id}`} value={taskAdes[task.id] ?? taskAde} onchange={(event) => taskAdes = { ...taskAdes, [task.id]: event.currentTarget.value }}>{#each team as id}<option value={id}>{adeName(id)}</option>{/each}</select>
                          {:else}<span>{adeName(taskAde)}</span>{/if}
                        </div>
                      {/if}
                      <small class="task-paths">{task.paths.join(", ") || "No ownership paths reported"}</small>
                      <small class="task-dependencies">{task.dependencies.length ? `After ${task.dependencies.join(", ")}` : "No task dependencies"}</small>
                    </div>
                  </div>
                {/if}
              {/each}
            </div>
          {/each}
          <section class="plan-contract" aria-label="Plan safety contract">
            <details class="plan-technical"><summary>Permissions and technical details</summary>
            <dl>
              <div><dt>Permission</dt><dd>{plan.permission_profile || "Not reported"}</dd></div>
              <div><dt>Isolation</dt><dd>{plan.isolation_mode || "Not reported"} · {plan.isolation_backend_intent || "backend not reported"}</dd></div>
              <div><dt>Execution</dt><dd>{plan.execution_backend || "Not reported"}</dd></div>
              <div><dt>Agent CLI</dt><dd>{selectedAde === CLAUDE_HOSTED_CHOICE ? "Hosted Routing packet review · no execution" : selectedAde === CLAUDE_ROUTE_CHOICE ? "Claude proposal route · subscription · fresh auth/model checks on Run" : `${plan.ade.requested ?? "Not selected"} · ${plan.ade.available ? "ready" : "unavailable"}${plan.ade.command ? ` · ${plan.ade.command}` : ""}`}</dd></div>
            </dl>
            </details>
            <div class="contract-list">
              <strong>Per-task verification commands</strong>
              {#if verificationCommands.length}
                <ul>{#each verificationCommands as command}<li><code>{command}</code></li>{/each}</ul>
                <p>These checks run in each task's workspace. Passing task checks does not prove the combined candidate passes.</p>
              {:else}<p class="unverified-copy">No verification commands were reported. Enter a command in Additional verification commands, then build the plan again. This plan cannot run without checks.</p>{/if}
            </div>
            <div class="contract-list" data-tone={plan.warnings.length ? "warning" : "quiet"}>
              <strong>Warnings</strong>
              {#if plan.warnings.length}
                <ul>{#each plan.warnings as warning}<li>{warning.message}<details><summary>Technical code</summary><code>{warning.code}</code></details></li>{/each}</ul>
              {:else}<p>No warnings reported.</p>{/if}
            </div>
          </section>
          {#if error}<p class="voice-state-message error" role="alert" aria-live="assertive">{error}</p>{/if}
        {:else if planning}
          <div class="plan-empty" role="status" aria-live="polite" aria-atomic="true"><strong>Building plan</strong><p>Checking paths, permissions, and agent CLI availability.</p></div>
        {:else}
          <div class="plan-empty">
            <strong role={error ? "alert" : undefined} aria-live={error ? "assertive" : undefined}>{error || "No plan yet"}</strong>
            <p>{error ? "The previous plan was cleared. Fix the issue, then build a new matching plan." : "Write an outcome, then Build plan."}</p>
            {#if mission.trim() && selectedAdeReady}
              <button class="primary" onclick={buildPlan} disabled={planning}>Build plan</button>
            {/if}
            {#if error}<button class="primary" disabled aria-describedby="empty-run-disabled-reason">Run <IconArrowRight size={16} /></button><small id="empty-run-disabled-reason" class="action-reason">Build a new matching plan before starting a run.</small>{/if}
          </div>
        {/if}
      </article>
    {/if}
  </div>
  {#if activeWorkspaceGrants.length || workspaceGrantError || workspaceGrantReadError || workspaceGrantNotice}
    <section class="advisor-workspace-grants" aria-label="Experimental routing access">
      {#if activeWorkspaceGrants.length}<strong>Experimental routing access</strong>{/if}
      {#each activeWorkspaceGrants as grant (grant.domain_id)}
        <div>
          <span>{domains.find((domain) => domain.domain_id === grant.domain_id)?.repo_root ?? grant.domain_id} · {grant.current_scope ? "local Shadow fixture" : "older routing scope"}</span>
          <button class="advisor-packet-action" disabled={!!workspaceGrantRevoking} onclick={() => void revokeWorkspaceGrant(grant)}>Revoke routing access</button>
        </div>
      {/each}
      {#if workspaceGrantError}<p role="alert">{workspaceGrantError}</p>{/if}
      {#if workspaceGrantReadError}<p role="alert">{workspaceGrantReadError}</p>{/if}
      {#if workspaceGrantNotice}<p role="status">{workspaceGrantNotice}</p>{/if}
    </section>
  {/if}
  <HostedRoutingReview {backend} {history} {domains} available={experimentalHostedAvailable} />
  {#if history.length}
    <details class="flow-history">
      <summary>Saved requests <span>{history.length}</span></summary>
      {#if historyStopNotice}<p class="action-reason" role="status">{historyStopNotice}</p>{/if}
      {#each history.slice(0, 6) as draft}
        <div>
          <button aria-label={`Use as new request: ${draft.title}`} disabled={planning || dispatching} onclick={() => restoreDraft(draft)}>
            <strong>{draft.title}</strong>
            <small>{draft.domain_id ?? "Workspace not recorded"} · {draft.status} · Use as new request</small>
          </button>
          {#if draft.status === "dispatching" || draft.status === "recovery_required" || (draft.status === "failed" && draft.dispatched_run_id)}
            <small class="action-reason">{draft.status === "dispatching" ? "Startup state unresolved; run may need recovery" : draft.status === "recovery_required" ? "Run needs recovery" : "Reviewed run retained; inspect status"}</small>
            {#if savedRoutedRunId(draft)}
              <button class="advisor-packet-action" disabled={!!historyStopPending} aria-label={`Stop starting ${draft.title}`} onclick={() => void stopSavedRoutedDispatch(draft)}>{historyStopPending === draft.id ? "Requesting Stop…" : "Stop starting run"}</button>
            {/if}
          {:else}
            <button aria-label={`Delete ${draft.title}`} onclick={() => deleteDraft(draft.id)}>×</button>
          {/if}
        </div>
        {#if packetEligible[draft.id] || packetDraftId === draft.id}
          <section class="advisor-packet-entry" aria-label={`Routing packet for ${draft.title}`}>
            {#if packetEligible[draft.id]}
              <button class="advisor-packet-action" onclick={() => packetDraftId === draft.id ? closePacket() : void revealPacket(draft)}>{packetDraftId === draft.id ? "Close packet" : "Inspect routing packet"}</button>
            {/if}
            {#if packetDraftId === draft.id}
              <p class="advisor-packet-note">Local preview; viewing does not send or grant consent. The packet uses coarse reviewed categories, which can still reveal the task type. This experimental grant reaches only the no-network Shadow fixture.</p>
              {#if packetLoading}<p role="status">Checking the saved Shadow review…</p>{/if}
              {#if packetError}<p role="alert">{packetError}</p>{/if}
              {#if packetPreview}
                <p class="advisor-packet-meta">Recipient · {packetPreview.recipient_identity}</p>
                <p class="advisor-packet-meta">Exact request body · SHA-256 {packetPreview.request_digest}</p>
                <pre aria-label="Exact advisory request JSON">{packetBody}</pre>
                <p class="advisor-packet-meta">Packet SHA-256 {packetPreview.packet_digest}</p>
                {#if packetConsent}
                  <p class="advisor-packet-meta">Workspace grant · {packetConsent.current_scope ? "Allowed for local Shadow fixture" : packetConsent.enabled ? "Older or different grant active" : "Off"} · revision {packetConsent.revision}</p>
                  {#if packetConsent.enabled}
                    <button class="advisor-packet-action" disabled={packetActionPending} onclick={() => void changePacketConsent(draft, false)}>{packetConsent.current_scope ? "Revoke local Shadow fixture" : "Revoke existing routing grant"}</button>
                  {:else if !packetConsent.enabled && packetConsent.revision + 1 === packetPreview.reviewed_consent_revision}
                    <button class="advisor-packet-action" disabled={packetActionPending} onclick={() => void changePacketConsent(draft, true)}>Allow local Shadow fixture</button>
                  {:else}
                    <p class="advisor-packet-note">This saved review does not match the next workspace grant. Create a new experimental Shadow review before enabling it.</p>
                  {/if}
                {/if}
                {#if proposedHostedPacket}
                  <section class="advisor-packet-entry" aria-label="Proposed hosted routing disclosure">
                    <p class="advisor-packet-note">Proposed hosted packet only. This saved review covers the local fixture, not hosted routing. Viewing this does not opt in or send data.</p>
                    <p class="advisor-packet-meta">Proposed recipient · {proposedHostedPacket.recipient_identity}</p>
                    <p class="advisor-packet-meta">Packet schema · {proposedHostedPacket.wire_schema_version} · {proposedHostedPacket.decision_kind} · {proposedHostedPacket.question_set_version}</p>
                    <p class="advisor-packet-meta">Hosted scope SHA-256 {proposedHostedPacket.scope_digest}</p>
                    <pre aria-label="Proposed hosted packet JSON">{proposedHostedBody}</pre>
                    <p class="advisor-packet-meta">Proposed packet SHA-256 {proposedHostedPacket.packet_digest}</p>
                    <p class="advisor-packet-note">A future request would add a request ID. The hosted service would build the Jev request separately. Neither is shown here.</p>
                  </section>
                {/if}
                {#if proposedHostedError}<p role="status">{proposedHostedError}</p>{/if}
              {/if}
              {#if packetNotice}<p role="status">{packetNotice}</p>{/if}
            {/if}
          </section>
        {/if}
      {/each}
    </details>
  {/if}
  {/if}
</section>

<style>
  .composer-context{padding:0 0 18px;border-bottom:1px solid var(--pytxo-line)}.composer-context select{min-height:36px;max-width:100%;flex:1;min-width:0;padding:6px 10px;border:1px solid var(--pytxo-line);border-radius:4px;background:var(--pytxo-surface-input);color:var(--pytxo-text-strong);font:inherit;font-size:13px}.composer-context select:focus-visible{outline:2px solid var(--pytxo-accent);outline-offset:2px}.flow-screen .flow-templates button{background:var(--pytxo-surface-raised);color:var(--pytxo-text-soft);border-color:var(--pytxo-line)}

  .flow-screen .flow-layout{grid-template-columns:minmax(0,1.15fr) minmax(280px,.85fr);max-width:1200px;gap:36px;align-items:start}
  .flow-screen .composer-panel{border-radius:0;border:0;background:transparent}.flow-screen .composer-panel>textarea{margin-inline:0;width:100%;min-height:210px;padding:20px;font-size:15px;line-height:1.7;border:1px solid var(--pytxo-line);background:var(--pytxo-surface-input)}
  .flow-screen .composer-panel .panel-head{padding-inline:0}.flow-screen .mission-guidance{margin-inline:0}.checks-editor{margin:20px 0;border-block:1px solid var(--pytxo-line);padding:14px 0}.checks-editor summary,.voice-disclosure summary{cursor:pointer;font-size:12px;color:var(--pytxo-text-soft)}.checks-editor summary span{display:block;margin:6px 0 0 16px;font-size:11px;color:var(--pytxo-text-muted)}.voice-disclosure{order:3;width:100%}.voice-disclosure .voice-controls{padding-top:12px}.checks-editor summary:focus-visible,.voice-disclosure summary:focus-visible{outline:2px solid var(--pytxo-accent);outline-offset:4px}
  .flow-screen .flow-layout--solo{grid-template-columns:minmax(0,1fr);max-width:880px}
  .flow-history{max-width:880px;border:0;border-top:1px solid var(--pytxo-line);background:transparent;border-radius:0}
  .flow-history>summary{padding:18px 0;cursor:pointer;color:var(--pytxo-text-soft);font-size:13px}
  .flow-history>summary span{margin-left:12px;color:var(--pytxo-text-muted);font-variant-numeric:tabular-nums}
  .flow-history>summary:focus-visible{outline:2px solid var(--pytxo-accent);outline-offset:3px}
  .advisor-packet-entry{padding:8px 12px 12px;border-top:1px solid var(--pytxo-line-soft);font-size:11px;color:var(--pytxo-text-soft)}
  .advisor-packet-action{min-height:40px;padding:5px 8px;border:1px solid var(--pytxo-line);border-radius:4px;background:var(--pytxo-surface-panel);color:var(--pytxo-text-strong);cursor:pointer}
  .advisor-packet-action:focus-visible{outline:2px solid var(--pytxo-accent);outline-offset:2px}
  .advisor-packet-note{margin:8px 0;color:var(--pytxo-text-muted)}
  .advisor-packet-meta{margin:8px 0 4px;font:11px "IBM Plex Mono",monospace;overflow-wrap:anywhere}
  .advisor-packet-entry pre{max-height:180px;margin:6px 0;overflow:auto;white-space:pre-wrap;overflow-wrap:anywhere;border:1px solid var(--pytxo-line);border-radius:4px;padding:8px;background:var(--pytxo-surface-input);color:var(--pytxo-text-strong);font:12px/1.5 "IBM Plex Mono",monospace}
  .advisor-workspace-grants{margin:16px 0;padding:12px;border:1px solid var(--pytxo-line);border-radius:4px;color:var(--pytxo-text-soft);font-size:11px}.advisor-workspace-grants>strong{display:block;margin-bottom:8px;color:var(--pytxo-text-strong)}.advisor-workspace-grants>div{display:flex;align-items:center;justify-content:space-between;gap:12px}.advisor-workspace-grants>div>span{overflow-wrap:anywhere}.advisor-workspace-grants p{margin:8px 0 0}
  .flow-screen .plan-panel{border-radius:0;border:0;border-left:1px solid var(--pytxo-line);padding-left:22px;background:transparent}.flow-screen .plan-wave{position:relative;border-left:1px solid var(--pytxo-line);padding-left:16px;margin-left:12px}.flow-screen .plan-wave>span{background:var(--pytxo-surface-shell);padding-block:8px}.flow-screen .plan-task textarea{min-height:80px}.flow-screen .composer-actions{padding-inline:0}.flow-screen .ade-readiness{padding-inline:0}
  @media(max-width:1100px){.flow-screen .flow-layout{grid-template-columns:1fr;max-width:850px}.flow-screen .plan-panel{border-left:0;border-top:1px solid var(--pytxo-line);padding:20px 0 0}}
  .composer-panel .run-checks { display: block; margin: 17px; color: var(--pytxo-text-soft); font-size: 12px; font-weight: 600; }
  .composer-panel .run-checks textarea { width: 100%; min-height: 76px; margin: 8px 0 0; padding: 10px; border: 1px solid var(--pytxo-line); border-radius: 4px; background: var(--pytxo-surface-input); resize: vertical; }
  .composer-panel .run-checks textarea::placeholder { color: var(--pytxo-text-muted); }
  .plan-technical > summary { padding: 12px; color: var(--pytxo-text-soft); font-size: 12px; cursor: pointer; }
  .plan-technical > summary:focus-visible { outline: 2px solid var(--pytxo-accent); outline-offset: -2px; }

  .flow-screen{position:relative;display:flex;width:100%;max-width:none;height:100%;min-height:0;box-sizing:border-box;flex-direction:column;padding:14px 18px 16px;overflow:hidden}
  .flow-screen>.screen-heading{display:flex;min-height:54px;flex:0 0 auto;align-items:center;gap:20px;margin:0 0 10px}.flow-screen>.screen-heading>div:first-child{min-width:180px}.flow-screen>.screen-heading h1{margin:0;font-size:20px}.flow-screen>.screen-heading p.mission-intro{display:block;margin:4px 0 0;color:var(--pytxo-text-muted);font-size:11px!important;white-space:nowrap}
  .composer-context{display:grid;grid-template-columns:minmax(150px,1fr) minmax(150px,1fr);gap:8px;width:min(620px,64vw);margin-left:auto;padding:0;border:0}.composer-context label{display:grid;grid-template-columns:auto minmax(0,1fr);align-items:center;gap:7px}.composer-context label>span{color:var(--pytxo-text-muted);font:10px "IBM Plex Mono",monospace}.composer-context select{width:100%;min-width:0;min-height:34px;padding:5px 8px;font-size:11px}
  .flow-screen .flow-layout{display:grid;grid-template-columns:minmax(0,44fr) minmax(0,56fr);flex:1;min-height:0;max-width:none;gap:12px;align-items:stretch;overflow:hidden}.flow-screen .flow-layout--solo{grid-template-columns:minmax(0,760px);max-width:none}
  .flow-screen .composer-panel,.flow-screen .plan-panel{min-height:0;padding:0 16px 18px;overflow:auto;overscroll-behavior:contain;border:1px solid var(--pytxo-line);border-radius:6px;background:var(--pytxo-surface-panel);scrollbar-gutter:stable}.flow-screen .plan-panel{padding-left:16px}.flow-screen .composer-panel .panel-head,.flow-screen .plan-panel .panel-head{position:sticky;z-index:4;top:0;margin-inline:-16px;padding:12px 16px;background:color-mix(in srgb,var(--pytxo-surface-panel) 96%,transparent);backdrop-filter:blur(10px)}
  .flow-screen .composer-panel>textarea{width:100%;min-height:clamp(150px,29vh,270px);box-sizing:border-box;margin:0;padding:14px;border:1px solid var(--pytxo-line);border-radius:4px;background:var(--pytxo-surface-input);font-size:14px;line-height:1.65;resize:none}
  .composer-command-bar,.plan-command-bar{position:sticky;z-index:5;bottom:0;display:flex;align-items:center;gap:8px;margin:10px -8px 0;padding:8px;border:1px solid var(--pytxo-line);border-radius:5px;background:color-mix(in srgb,var(--pytxo-surface-raised) 96%,transparent);box-shadow:0 -8px 28px color-mix(in srgb,var(--pytxo-surface-shell) 52%,transparent);backdrop-filter:blur(12px)}.composer-command-bar>span,.plan-command-bar>span{margin-right:auto;color:var(--pytxo-text-muted);font:10px "IBM Plex Mono",monospace}.composer-command-bar small,.plan-command-bar small{position:absolute;right:8px;top:calc(100% + 3px);max-width:360px;color:var(--pytxo-text-muted);font-size:10px;text-align:right}.plan-command-bar{top:49px;bottom:auto;margin-bottom:12px}.plan-command-bar .primary{min-width:84px}
  .planning-strip{position:sticky;z-index:6;top:49px;height:3px;margin-inline:-16px;overflow:hidden;background:var(--pytxo-line)}.planning-strip span{position:absolute;top:-7px;left:-20%;width:140%;color:var(--pytxo-activity);font:12px "IBM Plex Mono",monospace;word-spacing:18px;white-space:nowrap;animation:planning-scan 1.2s steps(12,end) infinite}
  .compact-pane-switch{display:none;flex:0 0 auto;margin-bottom:8px;border:1px solid var(--pytxo-line);border-radius:4px;overflow:hidden}.compact-pane-switch button{flex:1;min-height:34px;border:0;background:transparent;color:var(--pytxo-text-muted);font:11px var(--pytxo-font-ui)}.compact-pane-switch button.active{background:var(--pytxo-surface-active);color:var(--pytxo-text-strong)}
  .flow-history{position:absolute;z-index:20;right:18px;bottom:16px;width:min(420px,calc(100% - 36px));max-height:min(430px,70%);margin:0;overflow:auto;border:1px solid var(--pytxo-line);border-radius:5px;background:var(--pytxo-surface-raised);box-shadow:0 16px 44px #0009}.flow-history:not([open]){width:auto;overflow:hidden}.flow-history>summary{min-height:34px;padding:8px 12px}.flow-history[open]>summary{position:sticky;top:0;z-index:1;background:var(--pytxo-surface-raised)}
  @keyframes planning-scan{from{transform:translateX(-7%)}to{transform:translateX(7%)}}
  @media(max-width:1100px){.flow-screen .flow-layout{grid-template-columns:minmax(0,1fr);max-width:none}.flow-screen .plan-panel{padding:0 16px 18px;border:1px solid var(--pytxo-line)}.compact-pane-switch{display:flex}.flow-screen .compact-hidden{display:none}.flow-screen .flow-layout--solo .composer-panel{display:block}}
  @media(max-width:700px){.flow-screen{padding:10px}.flow-screen>.screen-heading{align-items:flex-start}.flow-screen>.screen-heading p.mission-intro{display:none}.composer-context{grid-template-columns:1fr;width:auto}.composer-context label{grid-template-columns:58px minmax(0,1fr)}.flow-history{right:10px;bottom:10px;width:calc(100% - 20px)}.flow-screen .composer-panel,.flow-screen .plan-panel{padding-inline:12px}.flow-screen .composer-panel .panel-head,.flow-screen .plan-panel .panel-head{margin-inline:-12px;padding-inline:12px}}
  @media(prefers-reduced-motion:reduce){.planning-strip span{left:0;animation:none}}
  @media (max-width: 700px) {
    .flow-screen > .screen-heading { flex-direction: column; align-items: stretch; gap: 10px; }
    .composer-context { width: 100%; margin-left: 0; }
  }
  .plan-command-bar { flex-wrap: wrap; }
  .plan-command-bar .action-reason {
    position: static;
    flex-basis: 100%;
    max-width: none;
    max-height: 108px;
    overflow: auto;
    white-space: pre-line;
    overflow-wrap: anywhere;
    color: var(--pytxo-text-strong);
    font: 12px/1.5 var(--pytxo-font-ui);
    text-align: left;
  }
  .agent-team { display: flex; flex-wrap: wrap; gap: 8px; margin: 12px 0 0; padding: 0; border: 0; }
  .agent-team legend { width: 100%; margin-bottom: 8px; color: var(--pytxo-text-muted); font: 500 11px var(--pytxo-font-mono, "IBM Plex Mono", monospace); letter-spacing: .06em; text-transform: uppercase; }
  .team-chip { display: inline-flex; align-items: center; gap: 8px; min-height: 36px; padding: 0 12px 0 10px; border: 1px solid var(--pytxo-line); border-radius: 8px; background: var(--pytxo-surface-panel); color: var(--pytxo-text-body); font-size: 13px; cursor: pointer; transition: border-color 140ms ease, background-color 140ms ease; }
  .team-chip:hover { border-color: var(--pytxo-text-muted); }
  .team-chip.on { border-color: color-mix(in srgb, var(--pytxo-activity) 55%, var(--pytxo-line)); background: color-mix(in srgb, var(--pytxo-activity) 8%, var(--pytxo-surface-panel)); color: var(--pytxo-text-strong); }
  .team-chip input { width: 15px; height: 15px; margin: 0; accent-color: var(--pytxo-activity); }
  .team-chip:has(input:focus-visible) { outline: 2px solid var(--pytxo-accent); outline-offset: 2px; }
  .workers-row { display: flex; flex-wrap: wrap; align-items: center; gap: 10px 14px; margin-top: 12px; font-size: 13px; color: var(--pytxo-text-body); }
  .workers-row small { color: var(--pytxo-text-muted); font-size: 12px; }
  .stepper { display: inline-flex; border: 1px solid var(--pytxo-line); border-radius: 8px; overflow: hidden; }
  .stepper button { width: 34px; min-height: 34px; padding: 0; border: 0; border-radius: 0; background: transparent; color: var(--pytxo-text-body); font-size: 16px; }
  .stepper button:disabled { opacity: .35; }
  .stepper output { display: grid; place-items: center; min-width: 40px; border-inline: 1px solid var(--pytxo-line); color: var(--pytxo-text-strong); font-weight: 600; font-variant-numeric: tabular-nums; }
  .task-agent { display: flex; align-items: center; gap: 8px; margin-top: 6px; color: var(--pytxo-text-strong); font-size: 13px; font-weight: 600; }
  .task-agent select { min-height: 30px; font-size: 13px; }
</style>
