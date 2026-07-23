<script lang="ts">
  import { IconArrowRight, IconMicrophone, IconPlayerRecord, IconShieldCheck, IconSparkles } from "@tabler/icons-svelte";
  import { onMount } from "svelte";
  import type { DesktopBackend } from "../../lib/desktop-backend";
  import type { FlowDraftRecord, FlowPlan, VoiceSessionDto, VoiceState } from "../../lib/types";
  let { backend, domains, preferredDomainId = null, previewState = "draft" }: { backend: DesktopBackend; domains: { domain_id: string; repo_root: string }[]; preferredDomainId?: string | null; previewState?: "draft" | "recording" | "paused" | "transcribing" | "cancelled" | "failed" | "uncertain" | "planning" | "ready" | "blocked" | "dispatched" } = $props();
  let selectedDomainId = $state("");
  let selectedAde = $state("cursor");
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
  let planning = $state(false);
  let error = $state("");
  let dispatchedRun = $state("");
  let dispatching = $state(false);
  let history = $state<FlowDraftRecord[]>([]);

  $effect(() => {
    const preferred = preferredDomainId;
    const list = domains;
    if (preferred && list.some((d) => d.domain_id === preferred)) {
      selectedDomainId = preferred;
    } else if (!selectedDomainId || !list.some((d) => d.domain_id === selectedDomainId)) {
      selectedDomainId = list[0]?.domain_id ?? "";
    }
  });

  async function buildPlan() {
    planning = true; error = "";
    try {
      plan = await backend.previewFlow({ id: crypto.randomUUID(), title: mission.slice(0, 72), mission_text: mission, source: missionSource, domain_id: selectedDomainId || null, project_id: null, ade_id: selectedAde });
      history = await backend.flowHistory();
    } catch (cause) { error = String(cause); } finally { planning = false; }
  }

  async function dispatch() {
    if (!plan || plan.status !== "ready" || dispatching) return;
    dispatching = true;
    try {
      plan = await backend.saveReviewedFlow(plan);
      dispatchedRun = await backend.dispatchFlow(plan.draft_id);
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
  }

  function restoreDraft(draft: FlowDraftRecord) {
    mission = draft.mission_text;
    missionSource = draft.source === "voice" ? "voice" : "text";
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
    if (["ready", "blocked", "dispatched"].includes(previewState)) {
      void buildPlan().then(() => {
        if (plan && previewState === "blocked") plan = { ...plan, status: "blocked", blocked_reasons: [{ kind: "path_claim_overlap" }] };
        if (previewState === "dispatched") dispatchedRun = "run-preview";
      });
    } else if (previewState === "planning") planning = true;
    return () => {
      disposed = true;
      unlisten?.();
      if (voiceSessionId) void backend.cancelVoice(voiceSessionId);
    };
  });
</script>

<section class="screen flow-screen">
  <header class="screen-heading"><div><p class="eyebrow">Mission planning</p><h1>Flow</h1><p>Turn an outcome into a safe, reviewable execution plan.</p></div><span class="policy-chip"><IconShieldCheck size={15} /> Preview required</span></header>
  <div class="flow-layout">
    <article class="panel composer-panel">
      <div class="panel-head"><div><p class="eyebrow">Draft mission</p><h2>What should Pytxo accomplish?</h2></div><span class="source">Text + Voice</span></div>
      <textarea bind:value={mission} aria-label="Flow mission" placeholder="Describe the outcome, constraints, and the parts of the system that matter…"></textarea>
      <div class="flow-templates"><span>Templates</span><button onclick={() => useTemplate("Diagnose the failing checks, implement the smallest safe fix, and verify it")}>Fix a failure</button><button onclick={() => useTemplate("Map the affected architecture, implement the feature, and prepare Run Review")}>Build a feature</button><button onclick={() => useTemplate("Review security, permissions, and path claims without changing files")}>Audit safely</button></div>
      {#if voiceState === "cancelled" || voiceState === "failed"}<p class:error={voiceState === "failed"} class="voice-state-message">{voiceState === "cancelled" ? "Voice capture cancelled · no audio retained" : "Voice capture failed · choose another input device"}</p>{/if}
      {#if voiceSegments.some((segment) => segment.uncertain)}<div class="uncertain-segments"><span>Review uncertain transcript</span>{#each voiceSegments as segment, index}{#if segment.uncertain}<label><input aria-label={`Correct uncertain segment ${index + 1}`} value={segment.text} oninput={(event) => correctSegment(index, event.currentTarget.value)} /><small>{Math.round(segment.confidence * 100)}% confidence</small></label>{/if}{/each}</div>{/if}
      <div class="composer-actions">
        <select class="voice-device" bind:value={selectedVoiceDevice} aria-label="Voice input device" disabled={!!voiceSessionId}>{#each voiceDevices as device}<option value={device}>{device}</option>{/each}</select>
        <button data-testid="voice-capture" class:recording class="voice-button" onclick={handleVoiceClick} onpointerdown={handleVoicePointerDown} onpointerup={handleVoicePointerUp} onpointercancel={handleVoicePointerCancel} aria-pressed={recording} disabled={!voiceAvailable || voiceState === "transcribing"} title={voiceAvailable ? "Click to record or press and hold" : "Enable the voice-whisper build feature"}>
          {#if recording}<IconPlayerRecord size={17} /> Finish recording{:else if voiceState === "paused"}<IconMicrophone size={17} /> Finish paused recording{:else if voiceState === "transcribing"}<IconSparkles size={17} /> Transcribing…{:else}<IconMicrophone size={17} /> {voiceAvailable ? "Start Voice" : "Voice unavailable"}{/if}
        </button>
        {#if voiceSessionId && (voiceState === "recording" || voiceState === "paused")}<button class="quiet" onclick={pauseOrResumeVoice}>{voiceState === "paused" ? "Resume" : "Pause"}</button>{/if}{#if voiceSessionId}<button class="quiet" onclick={cancelVoice}>Cancel</button>{/if}
        <div class="domain"><span>Execution domain & ADE</span><select bind:value={selectedDomainId} aria-label="Execution domain">{#each domains as domain}<option value={domain.domain_id}>{domain.repo_root.split(/[\\/]/).pop()}</option>{/each}</select><select bind:value={selectedAde} aria-label="Agent development environment"><option value="cursor">Cursor CLI</option><option value="codex">Codex CLI</option><option value="claude">Claude Code</option><option value="opencode">OpenCode</option><option value="aider">Aider</option></select></div>
        <button class="primary" disabled={!mission.trim() || planning} onclick={buildPlan}>{planning ? "Planning…" : "Build plan"} <IconSparkles size={16} /></button>
      </div>
      {#if recording || voiceState === "paused" || voiceState === "transcribing"}<div class="waveform" aria-label={voiceState === "transcribing" ? "Transcription progress" : "Recording audio level"}>{#each [4,8,13,20,9,25,17,7,15,22,11,5,18,10,4] as height}<i style={`height:${height}px`}></i>{/each}<span>{voiceState === "paused" ? "Paused" : voiceState === "transcribing" ? `${Math.round(transcriptionProgress * 100)}%` : "00:08"}</span></div>{/if}
    </article>

    <article class="panel plan-panel">
      <div class="panel-head"><div><p class="eyebrow">Plan preview</p><h2>{plan ? (plan.status === "ready" ? "Ready for review" : "Plan blocked") : "Waiting for a mission"}</h2></div>{#if plan}<span class="ready">{plan.status === "ready" ? "Validated" : "Blocked"}</span>{/if}</div>
      {#if plan}
        <div class="mission-summary"><strong>{mission}</strong><p>{planSummary}</p></div>
        {#each plan.waves as wave, waveIndex}<div class="plan-wave"><span>Wave {waveIndex + 1}</span>{#each wave as taskId, taskIndex}{@const task = plan.tasks.find((item) => item.id === taskId)}{#if task}<div><b>{String(taskIndex + 1).padStart(2, "0")}</b><p><input aria-label={`Task ${task.id} prompt`} value={task.prompt || task.id} oninput={(event) => editTask(task.id, event.currentTarget.value)} /><small>{task.agent} · {task.paths.join(", ") || "read-only"}</small></p></div>{/if}{/each}</div>{/each}
        <div class="plan-footer"><div><span>Estimate</span><strong>{plan.estimated_cost_usd ? `$${plan.estimated_cost_usd.toFixed(2)}` : "Local"}</strong></div><div><span>Path claims</span><strong>{plan.blocked_reasons.length ? `${plan.blocked_reasons.length} blockers` : "No collisions"}</strong></div><button class="primary" disabled={plan.status !== "ready" || !!dispatchedRun || dispatching} onclick={dispatch}>{dispatchedRun ? `Dispatched ${dispatchedRun}` : dispatching ? "Dispatching…" : "Dispatch Flow"} <IconArrowRight size={16} /></button></div>
      {:else}
        <div class="plan-empty"><div class="orbit"><span></span></div><strong>{error || "A plan will appear here"}</strong><p>Pytxo will enrich paths, validate claims, check permissions and ADE availability before anything can run.</p></div>
      {/if}
    </article>
  </div>
  {#if history.length}<article class="panel flow-history"><div class="panel-head"><div><p class="eyebrow">Saved locally</p><h2>Draft history</h2></div><span>{history.length} drafts</span></div>{#each history.slice(0, 6) as draft}<div><button onclick={() => restoreDraft(draft)}><strong>{draft.title}</strong><small>{draft.source} · {draft.status} · {draft.updated_at}</small></button><button aria-label={`Delete ${draft.title}`} onclick={() => deleteDraft(draft.id)}>×</button></div>{/each}</article>{/if}
</section>
