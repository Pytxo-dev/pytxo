<script lang="ts">
  import { IconAlertTriangle, IconArrowRight, IconFolderPlus, IconLoader2, IconMicrophone, IconPlayerRecord } from "@tabler/icons-svelte";
  import { onMount, untrack } from "svelte";
  import type { DesktopBackend } from "../../lib/desktop-backend";
  import type { AdeCliStatusDto, FlowDraftRecord, FlowPlan, RunDto, VoiceSessionDto, VoiceState } from "../../lib/types";

  const ADE_CHOICE_KEY = "pytxo-flow-ade-v1";
  let {
    backend,
    domains,
    runs = [],
    preferredDomainId = null,
    previewState = "draft",
    onAddWorkspace = null,
    onDispatched = null,
  }: {
    backend: DesktopBackend;
    domains: { domain_id: string; repo_root: string }[];
    runs?: RunDto[];
    preferredDomainId?: string | null;
    previewState?: "draft" | "recording" | "paused" | "transcribing" | "cancelled" | "failed" | "uncertain" | "planning" | "ready" | "blocked" | "dispatched";
    onAddWorkspace?: (() => void | Promise<void>) | null;
    onDispatched?: ((runId: string) => void) | null;
  } = $props();
  let selectedDomainId = $state("");
  let selectedAde = $state("");
  let adeClis = $state<AdeCliStatusDto[]>([]);
  let adeLoading = $state(true);
  let adeError = $state("");
  let mission = $state("");
  let missionSource = $state<"text" | "voice">("text");
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
  let voiceAvailable = $state(true);
  let plan = $state<FlowPlan | null>(null);
  let planInputKey = $state<string | null>(null);
  let planAttempted = $state(false);
  let planning = $state(false);
  let error = $state("");
  let dispatchedRun = $state("");
  let dispatchedStatus = $state("");
  let dispatching = $state(false);
  let history = $state<FlowDraftRecord[]>([]);
  let historySyncedRun = "";
  let missionInput: HTMLTextAreaElement | undefined = $state();
  let draftNotice = $state("");
  let domainSelectionInitialized = false;
  let lastPreferredDomainId: string | null = null;
  let adeSelectionInitialized = false;

  const currentInputKey = $derived(
    JSON.stringify({
      mission: mission.trim(),
      source: missionSource,
      domain: selectedDomainId,
      ade: selectedAde,
    }),
  );
  const planMatchesInputs = $derived(!!plan && planInputKey === currentInputKey);
  const selectedAdeStatus = $derived(adeClis.find((cli) => cli.id === selectedAde) ?? null);
  const selectedAdeReady = $derived(!!selectedAdeStatus && isAdeReady(selectedAdeStatus));
  const unavailableAdes = $derived(adeClis.filter((cli) => !isAdeReady(cli)));
  const verificationCommands = $derived(
    verificationCommandsFor(plan),
  );
  const planHasVerification = $derived(verificationCommands.length > 0);
  const canDispatchPlan = $derived(
    !!plan && plan.status === "ready" && planMatchesInputs && planHasVerification && selectedAdeReady && !adeLoading && !dispatchedRun && !dispatching,
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
    if (plan.status !== "ready") return "Resolve the reported plan blockers before starting the run.";
    if (!planMatchesInputs) return "The plan no longer matches the outcome, workspace, or agent CLI. Build it again.";
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
      void backend.flowHistory().then((drafts) => (history = drafts));
    }
  });

  $effect(() => {
    const inputKey = currentInputKey;
    if (planInputKey && planInputKey !== inputKey) planInputKey = null;
  });

  function isAdeReady(cli: AdeCliStatusDto) {
    return cli.installed && (cli.auth_state === "signed_in" || cli.auth_state === "not_applicable");
  }

  function verificationCommandsFor(candidate: FlowPlan | null) {
    return candidate
      ? [...new Set(candidate.tasks.flatMap((task) => task.verify ?? []).filter((command) => command.trim()))]
      : [];
  }

  function adeUnavailableReason(cli: AdeCliStatusDto) {
    if (!cli.installed) return "not installed";
    if (cli.auth_state === "signed_out") return "sign-in required";
    if (cli.auth_state === "unknown") return "sign-in could not be verified";
    if (cli.auth_state === "not_installed") return "not installed";
    return "not ready";
  }

  async function loadAdeClis() {
    adeLoading = true;
    adeError = "";
    try {
      adeClis = await backend.listAdeClis();
      const saved = typeof localStorage === "undefined" ? null : localStorage.getItem(ADE_CHOICE_KEY);
      const savedReady = saved ? adeClis.find((cli) => cli.id === saved && isAdeReady(cli)) : null;
      const currentReady = adeClis.find((cli) => cli.id === selectedAde && isAdeReady(cli));
      selectedAde = adeSelectionInitialized ? currentReady?.id ?? "" : savedReady?.id ?? currentReady?.id ?? adeClis.find(isAdeReady)?.id ?? "";
      adeSelectionInitialized = true;
    } catch (cause) {
      adeClis = [];
      selectedAde = "";
      adeError = cause instanceof Error ? cause.message : String(cause);
    } finally {
      adeLoading = false;
    }
  }

  function chooseAde(value: string) {
    selectedAde = value;
    if (typeof localStorage !== "undefined") localStorage.setItem(ADE_CHOICE_KEY, value);
  }

  $effect(() => {
    const preferred = preferredDomainId;
    const list = domains;
    const selected = untrack(() => selectedDomainId);
    if (!domainSelectionInitialized || preferred !== lastPreferredDomainId) {
      selectedDomainId = list.find((domain) => domain.domain_id === preferred)?.domain_id ?? list[0]?.domain_id ?? "";
      domainSelectionInitialized = true;
      lastPreferredDomainId = preferred;
    } else if (selected && !list.some((domain) => domain.domain_id === selected)) {
      selectedDomainId = "";
      draftNotice = "The selected workspace is no longer available. Select a workspace explicitly and build a fresh plan.";
    }
  });

  async function buildPlan() {
    if (buildPlanDisabledReason) return;
    const requestedInputKey = currentInputKey;
    planning = true; error = "";
    planAttempted = true;
    // A new preview request revokes the previous dispatch authority immediately.
    plan = null;
    planInputKey = null;
    dispatchedRun = "";
    dispatchedStatus = "";
    historySyncedRun = "";
    try {
      plan = await backend.previewFlow({ id: crypto.randomUUID(), title: mission.slice(0, 72), mission_text: mission, source: missionSource, domain_id: selectedDomainId || null, project_id: null, ade_id: selectedAde });
      planInputKey = requestedInputKey;
      history = await backend.flowHistory();
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
    error = "";
    try {
      const reviewedPlan = await backend.saveReviewedFlow(plan);
      plan = reviewedPlan;
      if (reviewedPlan.status !== "ready" || verificationCommandsFor(reviewedPlan).length === 0) {
        error = reviewedPlan.status !== "ready"
          ? "The reviewed plan is blocked and was not dispatched."
          : "The reviewed plan reports no verification commands, so it remains unverified and was not dispatched.";
        return;
      }
      dispatchedRun = await backend.dispatchFlow(plan.draft_id);
      dispatchedStatus = "running";
      onDispatched?.(dispatchedRun);
    } catch (cause) {
      error = cause instanceof Error ? cause.message : String(cause);
    } finally {
      dispatching = false;
    }
  }

  async function toggleVoice() {
    error = "";
    try {
      if (!voiceSessionId) {
        const session = await backend.startVoice(selectedVoiceDevice, "en");
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
    const held = performance.now() - voicePointerStartedAt >= 350;
    if (held && !voicePointerHadSession) {
      suppressVoiceClick = true;
      await voicePointerAction;
      if (voiceSessionId) await toggleVoice();
    }
  }

  function handleVoiceClick(event: MouseEvent) {
    if (event.detail === 0) {
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
    if (plan) plan = { ...plan, tasks: plan.tasks.map((task) => task.id === taskId ? { ...task, prompt } : task) };
  }

  async function deleteDraft(draftId: string) {
    await backend.deleteFlowDraft(draftId);
    history = history.filter((draft) => draft.id !== draftId);
  }

  const planSummary = $derived(
    plan
      ? `${plan.tasks.length} task${plan.tasks.length === 1 ? "" : "s"} · ${plan.waves.length} wave${plan.waves.length === 1 ? "" : "s"} · ${[...new Set(plan.tasks.map((t) => t.agent))].join(" + ") || "no agent assigned"}`
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

  function restoreDraft(draft: FlowDraftRecord) {
    if (planning || dispatching) return;
    mission = draft.mission_text;
    missionSource = draft.source === "voice" ? "voice" : "text";
    selectedDomainId = !draft.project_id && domains.some((domain) => domain.domain_id === draft.domain_id) ? draft.domain_id! : "";
    // Reuse guidance only. A saved plan is never restored as dispatch authority.
    plan = null;
    planInputKey = null;
    planAttempted = false;
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
    void backend.flowHistory().then((drafts) => (history = drafts));
    mission = previewState === "draft" ? "" : "Make the desktop shell production ready";
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
      unlisten?.();
      if (voiceSessionId) void backend.cancelVoice(voiceSessionId);
    };
  });
</script>

<section class="screen flow-screen">
  <header class="screen-heading">
    <div>
      <h1>New run</h1>
      <p class="mission-intro">Give your agent a bounded job. Review the plan before it starts.</p>
    </div>
  </header>

  {#if !domains.length}
    <div class="panel">
      <div class="empty flow-empty">
        <strong>Add a workspace</strong>
        <p>Select a repository folder, then choose the agent CLI you already use.</p>
        {#if onAddWorkspace}
          <button class="primary" onclick={() => void onAddWorkspace()}>
            <IconFolderPlus size={16} /> Add workspace
          </button>
        {/if}
      </div>
    </div>
  {:else}
  <div class="flow-layout" class:flow-layout--solo={!showPlanPanel}>
    <article class="panel composer-panel">
      <div class="panel-head">
        <div>
          <h2>Describe the job</h2>
        </div>
      </div>
      <p id="mission-guidance" class="mission-guidance">Name existing files or folders, the behavior you want, and what must stay unchanged. The planner uses those paths to define scope.</p>
      <textarea bind:this={missionInput} bind:value={mission} aria-label="Mission outcome" aria-describedby="mission-guidance" placeholder="Fix the parser in src/parser.rs so empty input returns an error. Add a regression test. Keep the public API unchanged."></textarea>
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
        <div class="voice-controls">
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
            aria-pressed={recording}
            disabled={!voiceAvailable || voiceState === "transcribing"}
            aria-describedby={!voiceAvailable ? "voice-disabled-reason" : undefined}
            title={voiceAvailable ? "Click to record or press and hold" : "Enable the voice-whisper build feature"}
          >
            {#if recording}<IconPlayerRecord size={17} /> Finish recording
            {:else if voiceState === "paused"}<IconMicrophone size={17} /> Finish paused recording
            {:else if voiceState === "transcribing"}<IconLoader2 size={17} class="spin" /> Transcribing…
            {:else}<IconMicrophone size={17} /> {voiceAvailable ? "Start Voice" : "Voice unavailable"}{/if}
          </button>
          {#if voiceSessionId && (voiceState === "recording" || voiceState === "paused")}
            <button class="quiet" onclick={pauseOrResumeVoice}>{voiceState === "paused" ? "Resume" : "Pause"}</button>
          {/if}
          {#if voiceSessionId}<button class="quiet" onclick={cancelVoice}>Cancel</button>{/if}
          {#if !voiceAvailable}<small id="voice-disabled-reason" class="action-reason">Voice capture is unavailable until the local voice feature is enabled.</small>{/if}
        </div>
        <div class="dispatch-controls">
          <div class="domain">
            <span>Workspace &amp; agent CLI</span>
            <select bind:value={selectedDomainId} aria-label="Workspace">
              {#if !selectedDomainId}<option value="" disabled>Select a workspace</option>{/if}
              {#each domains as domain}
                <option value={domain.domain_id}>{domain.repo_root.split(/[\\/]/).pop()}</option>
              {/each}
            </select>
            <select value={selectedAde} onchange={(event) => chooseAde(event.currentTarget.value)} aria-label="Agent CLI" disabled={adeLoading || !adeClis.length}>
              {#if adeLoading}
                <option value="">Checking detected CLIs…</option>
              {:else if !adeClis.length}
                <option value="">No ready CLI detected</option>
              {:else}
                {#if !selectedAde}<option value="" disabled>Select a ready CLI</option>{/if}
                {#each adeClis as cli (cli.id)}
                  <option value={cli.id} disabled={!isAdeReady(cli)}>
                    {cli.display_name}{isAdeReady(cli) ? "" : ` — ${adeUnavailableReason(cli)}`}
                  </option>
                {/each}
              {/if}
            </select>
          </div>
          <button class="primary" disabled={!!buildPlanDisabledReason} aria-describedby={buildPlanDisabledReason ? "build-plan-disabled-reason" : undefined} onclick={buildPlan}>
            {planning ? "Building…" : "Build plan"}
          </button>
        </div>
        {#if buildPlanDisabledReason}<p id="build-plan-disabled-reason" class="action-reason">{buildPlanDisabledReason}</p>{/if}
      </div>
      <div class="ade-readiness">
        <div class="readiness-summary" aria-live="polite">
        {#if adeError}
          <span class="error">Agent readiness could not be checked: {adeError}</span>
        {:else if selectedAdeStatus}
          <span class:ready={selectedAdeReady}>{selectedAdeStatus.display_name}: {adeLoading ? "checking…" : selectedAdeReady ? "ready" : adeUnavailableReason(selectedAdeStatus)}</span>
        {:else if !adeLoading}
          <span class="error">Install and sign in to an agent CLI before building a plan.</span>
        {/if}
        <button class="quiet" onclick={loadAdeClis} disabled={adeLoading || planning || dispatching}>{adeLoading ? "Checking…" : "Check again"}</button>
        </div>
        <p>One ready CLI is enough. More instances are used only when the reviewed plan calls for them.</p>
        {#if unavailableAdes.length}
          <details><summary>{unavailableAdes.length} other CLI{unavailableAdes.length === 1 ? "" : "s"} unavailable</summary><ul>{#each unavailableAdes as cli}<li>{cli.display_name} · {adeUnavailableReason(cli)}</li>{/each}</ul></details>
        {/if}
      </div>
    </article>

    {#if showPlanPanel}
      <article class="panel plan-panel" aria-busy={planning}>
        <div class="panel-head">
          <div>
            <h2>{planning ? "Building plan…" : plan ? (plan.status === "ready" ? (planHasVerification ? "Review plan" : "Plan needs verification") : "Plan blocked") : "Plan"}</h2>
          </div>
          {#if plan}
            <span class:ready={plan.status === "ready" && planHasVerification} class="plan-state" data-tone={plan.status !== "ready" ? "blocked" : planHasVerification ? "ready" : "unknown"} role="status" aria-live="polite" aria-atomic="true">
              {plan.status !== "ready" ? "Blocked" : planHasVerification ? "Ready" : "Unverified"}
            </span>
          {/if}
        </div>
        {#if plan}
          <p class="plan-summary">{planSummary}</p>
          <p class="plan-explainer">Stages run in order. Path ownership and dependencies below define each task; the configured concurrency limits how many can run at once.</p>
          {#if !planMatchesInputs}
            <div class="plan-stale" role="status">
              <IconAlertTriangle size={15} />
              <span><strong>Plan is stale</strong><small>The outcome, workspace, or agent CLI changed. Build a matching plan before Run is available.</small></span>
            </div>
          {/if}
          {#each plan.waves as wave, waveIndex}
            <div class="plan-wave">
              <span>Stage {waveIndex + 1}</span>
              {#each wave as taskId, taskIndex}
                {@const task = plan.tasks.find((item) => item.id === taskId)}
                {#if task}
                  <div>
                    <b>{String(taskIndex + 1).padStart(2, "0")}</b>
                    <div class="plan-task">
                      <label><span>{task.id} · {task.agent}</span><textarea aria-label={`Task ${task.id} prompt`} rows="2" value={task.prompt || task.id} oninput={(event) => editTask(task.id, event.currentTarget.value)}></textarea></label>
                      <small class="task-paths">{task.paths.join(", ") || "No ownership paths reported"}</small>
                      <small class="task-dependencies">{task.dependencies.length ? `After ${task.dependencies.join(", ")}` : "No task dependencies"}</small>
                    </div>
                  </div>
                {/if}
              {/each}
            </div>
          {/each}
          <section class="plan-contract" aria-label="Plan safety contract">
            <h3>Safety contract</h3>
            <dl>
              <div><dt>Permission</dt><dd>{plan.permission_profile || "Not reported"}</dd></div>
              <div><dt>Isolation</dt><dd>{plan.isolation_mode || "Not reported"} · {plan.isolation_backend_intent || "backend not reported"}</dd></div>
              <div><dt>Execution</dt><dd>{plan.execution_backend || "Not reported"}</dd></div>
              <div><dt>Agent CLI</dt><dd>{plan.ade.requested ?? "Not selected"} · {plan.ade.available ? "ready" : "unavailable"}{plan.ade.command ? ` · ${plan.ade.command}` : ""}</dd></div>
            </dl>
            <div class="contract-list">
              <strong>Per-task verification commands</strong>
              {#if verificationCommands.length}
                <ul>{#each verificationCommands as command}<li><code>{command}</code></li>{/each}</ul>
                <p>These checks run in each task's workspace. Passing task checks does not prove the combined candidate passes.</p>
              {:else}<p class="unverified-copy">No verification commands were reported. Add a verify command to each task in pytxo.toml, then build the plan again. This plan cannot run without checks.</p>{/if}
            </div>
            <div class="contract-list" data-tone={plan.warnings.length ? "warning" : "quiet"}>
              <strong>Warnings</strong>
              {#if plan.warnings.length}
                <ul>{#each plan.warnings as warning}<li><span>{warning.code}</span>{warning.message}</li>{/each}</ul>
              {:else}<p>No warnings reported.</p>{/if}
            </div>
            <div class="contract-list" data-tone={plan.blocked_reasons.length ? "blocked" : "quiet"}>
              <strong>Blockers</strong>
              {#if plan.blocked_reasons.length}
                <ul>{#each plan.blocked_reasons as reason}<li>{describeBlocker(reason)}</li>{/each}</ul>
              {:else}<p>No blockers reported.</p>{/if}
            </div>
          </section>
          <div class="plan-footer">
            <div><span>Estimate</span><strong>{plan.estimated_cost_usd !== null ? `$${plan.estimated_cost_usd.toFixed(2)}` : "Not estimated"}</strong></div>
            <div><span>Path locks</span><strong>{plan.blocked_reasons.length ? `${plan.blocked_reasons.length} collisions` : "Clear"}</strong></div>
            <button class="quiet" disabled={planning} onclick={buildPlan}>Build plan</button>
            <div class="run-action">
              <button class="primary" disabled={!canDispatchPlan} aria-describedby={runDisabledReason ? "run-disabled-reason" : undefined} onclick={dispatch}>
                {dispatchedRun ? `${dispatchedLabel} ${dispatchedRun}` : dispatching ? "Starting…" : "Run"} <IconArrowRight size={16} />
              </button>
              {#if runDisabledReason}<small id="run-disabled-reason" class="action-reason">{runDisabledReason}</small>{/if}
            </div>
          </div>
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
  {#if history.length}
    <article class="panel flow-history">
      <div class="panel-head">
        <div><h2>Drafts</h2></div>
        <span>{history.length}</span>
      </div>
      {#each history.slice(0, 6) as draft}
        <div>
          <button aria-label={`Use as new mission: ${draft.title}`} disabled={planning || dispatching} onclick={() => restoreDraft(draft)}>
            <strong>{draft.title}</strong>
            <small>{draft.domain_id ?? "Workspace not recorded"} · {draft.status} · Use as new mission</small>
          </button>
          <button aria-label={`Delete ${draft.title}`} onclick={() => deleteDraft(draft.id)}>×</button>
        </div>
      {/each}
    </article>
  {/if}
  {/if}
</section>
