<script lang="ts">
  import { withPreviewsHidden } from "../../lib/preview-overlay";
  import IconArrowRight from "@tabler/icons-svelte/icons/arrow-right";
  import { canOpenRunReview } from "../../lib/review-state";
  import IconAlertTriangle from "@tabler/icons-svelte/icons/alert-triangle";
  import IconPlayerStop from "@tabler/icons-svelte/icons/player-stop";
  import IconPlus from "@tabler/icons-svelte/icons/plus";
  import type { DesktopBackend, DesktopSnapshot } from "../../lib/desktop-backend";
  import { runState, isPartiallyApplied } from "../../lib/epistemic";
  import type { RoutingDisplaySummary, RunDto, RunReviewDto } from "../../lib/types";
  import BoundaryPanel from "./BoundaryPanel.svelte";
  import ApertureGlyph from "./ApertureGlyph.svelte";
  import { workActivity, hasCurrentApplyIssue } from "../../lib/work-activity";
  import ExecutionMap from "./ExecutionMap.svelte";
  import { exactAttemptAgent } from "../../lib/execution-topology";
  import RoutingRunDetails from "./RoutingRunDetails.svelte";
  import StateChip from "./StateChip.svelte";
  import type { DockPosition, DockReference } from "../../lib/dock-layout";
  import { selectMissionRun } from "../../lib/mission-selection";
  import { readReviewRevision, reviewRevisionKey } from "../../lib/review-detail-cache";

  let {
    snapshot,
    backend,
    activeDomainLabel = null,
    activeDomainId = null,
    focusRunId = null,
    onNewRun,
    onOpenApprovals,
    onReviewRun,
    onStopRun,
    onSelectRun,
    onInspect = null,
    onDismissInspect = () => {},
  }: {
    snapshot: DesktopSnapshot;
    backend: DesktopBackend;
    activeDomainLabel?: string | null;
    activeDomainId?: string | null;
    focusRunId?: string | null;
    onNewRun: () => void;
    onOpenApprovals: () => void;
    onReviewRun: (runId: string) => void;
    onStopRun: (runId: string, domainId: string) => Promise<void>;
    onSelectRun: (runId: string) => void;
    onInspect?: ((ref: DockReference, position?: DockPosition) => void) | null;
    onDismissInspect?: () => void;
  } = $props();

  let heading: HTMLElement | undefined = $state();
  let stopDialog: HTMLDialogElement | undefined = $state();
  let stopCancel: HTMLButtonElement | undefined = $state();
  let stopTarget = $state<{ run: RunDto; domainId: string; workspace: string } | null>(null);
  let stopError = $state("");
  let stopMessage = $state("");
  let stopping = $state(false);
  let selectedAgentId = $state<string | null>(null);
  let routingRetry = $state(0);
  let routingRead = $state<{
    runId: string; domainId: string; revision: string;
    summary: RoutingDisplaySummary | null; error: string | null; loading: boolean;
  } | null>(null);

  let loadedReview = $state<RunReviewDto | null>(null);
  let loadedReviewDomain = $state<string | null>(null);
  let reviewError = $state<string | null>(null);
  let reviewLoading = $state(false);
  let missionTitle = $state<string | null>(null);
  let missionText = $state<string | null>(null);
  let taskDescriptions = $state<Record<string, string>>({});
  const displayMissionTitle = $derived(missionTitle && missionText?.startsWith(missionTitle) && missionText.length > missionTitle.length ? `${missionTitle.trimEnd()}…` : missionTitle);
  $effect(() => {
    const id = focusRun?.id;
    const domain = focusRun?.domain_id;
    missionTitle = null; missionText = null; taskDescriptions = {};
    if (!id) return;
    let valid = true;
    backend.flowHistory().then(records => {
      if (!valid) return;
      const draft = records.find(d => d.dispatched_run_id === id && d.domain_id === domain);
      missionTitle = draft?.title.trim() || draft?.mission_text.trim() || null; missionText = draft?.mission_text ?? null;
      // Saved plan prose is presentation only; scheduling still uses the recorded run plan.
      try {
        const plan = JSON.parse(draft?.plan_json ?? "{}");
        if (Array.isArray(plan.tasks)) {
          const copy: Record<string,string> = {};
          for (const task of plan.tasks) if (typeof task.id === "string" && typeof task.prompt === "string" && task.prompt.trim()) copy[task.id] = task.prompt.trim();
          taskDescriptions = copy;
        }
      } catch { /* A missing or invalid saved plan supplies no descriptive copy. */ }
    }).catch(() => {});
    return () => { valid = false; };
  });

  const ACTIVE_STATUSES = ["starting", "running", "pending", "dispatching", "active"];

  const runs = $derived(
    activeDomainId
      ? snapshot.runs.filter((run) => run.domain_id === activeDomainId)
      : snapshot.runs,
  );
  const focusRun = $derived(selectMissionRun(snapshot.runs, activeDomainId, focusRunId));
  const routingIdentityUnknown = $derived(!!focusRun && snapshot.diagnostics.some((diagnostic) =>
    diagnostic.stage === "routing_revision" && diagnostic.domain_id === focusRun.domain_id && diagnostic.run_id === focusRun.id));
  const currentRoutingRead = $derived(
    focusRun?.routing_revision != null
      && routingRead?.runId === focusRun.id
      && routingRead.domainId === focusRun.domain_id
      && routingRead.revision === focusRun.routing_revision
        ? routingRead : null,
  );
  const agents = $derived(focusRun ? snapshot.agents.filter((agent) => agent.run_id === focusRun.id && agent.domain_id === focusRun.domain_id) : []);
  const focusState = $derived(focusRun ? runState(focusRun) : null);
  const review = $derived(loadedReview?.run_id === focusRun?.id && loadedReviewDomain === focusRun?.domain_id ? loadedReview : null);
  const partialAttempt = $derived(review?.apply_attempts.find(isPartiallyApplied) ?? null);
  const currentApplyIssue = $derived(!!focusRun && (hasCurrentApplyIssue(focusRun) || !!review && hasCurrentApplyIssue(review)));
  const receiptProfile = $derived((review?.enforcement?.run?.effective_profile ?? "").toLowerCase());
  const canReviewFocusRun = $derived(canOpenRunReview(focusRun, review));
  const canStopFocusRun = $derived(!!focusRun && ACTIVE_STATUSES.includes(focusRun.status.toLowerCase()));
  const checks = $derived(review?.prepared_manifest?.candidate_verification?.checks ?? []);
  const workSummary = $derived.by(() => {
    if (!focusRun) return "";
    if (partialAttempt) return "Some project files may have changed. Check recovery before continuing.";
    if (currentApplyIssue) return "The changes need attention. Review the problem before continuing.";
    if (focusRun.applied_at) return "These changes have been saved to your project.";
    if (["starting", "pending", "dispatching"].includes(focusRun.status.toLowerCase())) return "Getting the work ready. Your agents have not reported a result yet.";
    if (focusRun.status === "failed_startup") return "The work could not start. Open details to see what needs fixing.";
    if (runApprovals.length) return "A decision for this run needs your attention.";
    if (routingIdentityUnknown) return "Routing identity could not be checked. Worker selection is unavailable.";
    if (focusRun.routing_revision != null && canStopFocusRun) {
      if (!currentRoutingRead?.summary) return currentRoutingRead?.loading === false
        ? "Routing record unavailable. Retry the record below to see the latest attempt."
        : "Checking the latest routing attempt…";
      const admitted = currentRoutingRead.summary.tasks.flatMap((task) => task.attempts
        .filter((attempt) => task.current_attempt_id === attempt.attempt_id && attempt.state === "admitted")
        .map((attempt) => ({ task, attempt })));
      if (activity.active) return "A recorded worker is running. You can follow its attempt below.";
      if (admitted.length) return exactAttemptAgent(agents, focusRun, admitted[0].task.task_id, admitted[0].attempt.agent_id)
        ? `Attempt ${admitted[0].attempt.ordinal} is admitted. Waiting for its worker state to settle.`
        : `Attempt ${admitted[0].attempt.ordinal} is admitted. Its worker has not been recorded yet.`;
      return "Routing is in progress. Open the attempt record for its latest state.";
    }
    if (canStopFocusRun) return "Your agents are working on this request. You can follow their progress below.";
    if (["failed", "stopped", "cancelled"].includes(focusRun.status.toLowerCase())) return "The work stopped before finishing. Open details to see what happened.";
    if (checks.some(check => !check.passed)) return "Some checks failed. Review the results before deciding what to do next.";
    if (canReviewFocusRun) return review?.prepared_manifest?.version === 3 && review.prepared_manifest.candidate_verification?.version === 1 && checks.length > 0 && checks.every(check => check.passed)
      ? "The changes are ready for your review. The saved checks passed."
      : "The changes are ready for your review. Check the results before saving them.";
    return focusRun.status === "completed" ? "The agents have reported back. Details show what is ready and what still needs checking." : "The current work status needs checking. Open details before continuing.";
  });
  const runApprovals = $derived(
    focusRun
      ? snapshot.approvals.filter((approval) => approval.domain_id === focusRun.domain_id && approval.run_id === focusRun.id)
      : [],
  );

  const activity = $derived(workActivity(focusRun, agents,
    runApprovals.length > 0,
    !!snapshot.error || snapshot.diagnostics.some(d => d.domain_id === focusRun?.domain_id && (!d.run_id || d.run_id === focusRun?.id)), canReviewFocusRun, !!partialAttempt || currentApplyIssue, currentRoutingRead?.summary ?? null));

  /**
   * The enforcement receipt is the boundary panel's whole substance, so it is
   * fetched per focused run. A stale response from a previous run is discarded
  * rather than rendered against the wrong candidate.
  */
  $effect(() => {
    const currentRun = focusRun;
    if (!currentRun) {
      loadedReview = null;
      reviewError = null;
      return;
    }
    const runId = currentRun.id;
    const revisionKey = reviewRevisionKey(currentRun);
    let current = true;
    const domainId = currentRun.domain_id;
    reviewLoading = true;
    reviewError = null;
    readReviewRevision(backend, revisionKey, runId, domainId)
      .then((result) => {
        if (!current) return;
        loadedReview = result;
        loadedReviewDomain = domainId;
      })
      .catch((error: unknown) => {
        if (!current) return;
        loadedReview = null;
        reviewError = error instanceof Error ? error.message : String(error);
      })
      .finally(() => {
        if (current) reviewLoading = false;
      });
    return () => {
      current = false;
    };
  });

  // The Work canvas and the detailed ledger share one revision-matched Store
  // read. An older response must never identify a worker in a newer run.
  $effect(() => {
    void routingRetry;
    const runId = focusRun?.id;
    const domainId = focusRun?.domain_id;
    const revision = focusRun?.routing_revision;
    routingRead = null;
    if (!runId || !domainId || revision == null) return;
    const key = { runId, domainId, revision };
    let current = true;
    routingRead = { ...key, summary: null, error: null, loading: true };
    backend.routingRunSummary(runId, domainId)
      .then((summary) => {
        if (!current) return;
        routingRead = summary?.run_id === runId && summary.domain_id === domainId && summary.routing_revision === revision
          ? { ...key, summary, error: null, loading: false }
          : { ...key, summary: null, error: "Routing record changed or could not be matched to this run. Refreshing the run list may resolve it.", loading: false };
      })
      .catch((cause: unknown) => {
        if (current) routingRead = { ...key, summary: null, error: cause instanceof Error ? cause.message : String(cause), loading: false };
      });
    return () => { current = false; };
  });

  function domainForRun(run: RunDto) {
    return snapshot.domains.find((domain) => domain.domain_id === run.domain_id) ?? null;
  }

  function displayRunId(id: string) {
    return id.length > 20 ? `${id.slice(0, 8)}…${id.slice(-4)}` : id;
  }

  export function focusActiveRun() {
    queueMicrotask(() => heading?.focus());
  }

  function requestStop(run: RunDto) {
    if (!ACTIVE_STATUSES.includes(run.status.toLowerCase())) return;
    const domain = domainForRun(run);
    if (!domain) {
      stopMessage = "";
      stopError = `Cannot stop ${run.id}: its execution domain is not in the current snapshot.`;
      return;
    }
    stopMessage = "";
    stopError = "";
    stopTarget = {
      run,
      domainId: domain.domain_id,
      workspace: domain.repo_root.split(/[\\/]/).pop() ?? domain.domain_id,
    };
    void withPreviewsHidden(() => {
      if (stopDialog && !stopDialog.open) stopDialog.showModal();
      stopCancel?.focus();
    });
  }

  function closeStopDialog() {
    if (stopping) return;
    stopDialog?.close();
    stopTarget = null;
    stopError = "";
  }

  async function confirmStop() {
    if (!stopTarget || stopping) return;
    const target = stopTarget;
    stopping = true;
    stopError = "";
    try {
      await onStopRun(target.run.id, target.domainId);
      stopMessage = `Stop requested for ${target.run.id} in ${target.workspace}.`;
      stopDialog?.close();
      stopTarget = null;
      focusActiveRun();
    } catch (error) {
      stopError = error instanceof Error ? error.message : String(error);
    } finally {
      stopping = false;
    }
  }

  async function recover(runId: string) {
    try {
      await backend.reconcileRunRecovery(runId, activeDomainId);
      loadedReview = await backend.runReview(runId, activeDomainId);
      loadedReviewDomain = activeDomainId;
      stopMessage = `Recovery reconciled for ${runId}.`;
    } catch (error) {
      stopError = error instanceof Error ? error.message : String(error);
    }
  }

  export function requestStopRun(runId: string) {
    const run = runs.find((candidate) => candidate.id === runId);
    if (run) requestStop(run);
  }

  function stopConsequence(run: RunDto) {
    const reportedProfile = (run.permission_profile ?? "").toLowerCase();
    if (receiptProfile && reportedProfile && receiptProfile !== reportedProfile) {
      return `The run reports ${reportedProfile}, but its enforcement receipt reports ${receiptProfile}. Stop will be requested, but repository impact is unknown until that mismatch is reviewed.`;
    }
    const profile = receiptProfile || reportedProfile;
    if (profile === "supernova") return "Agents stop where they are. Supernova writes directly, so repository changes already made remain in place.";
    if (profile === "deepspace") return "Agents stop where they are. DeepSpace is non-flushable, so no prepared package can be applied.";
    if (profile === "orbit" || profile === "galaxy") return `Agents stop where they are. Work already prepared for review is kept outside the repository until explicit Apply.`;
    return "Agents stop where they are. Check the run's effective permission profile and enforcement receipt for repository impact.";
  }
</script>

<section class="screen work" aria-label="Work">
  <header class="command-strip work-heading" role={focusRun ? "region" : undefined} aria-label={focusRun ? "Focused run" : undefined}>
    {#if focusRun}<ApertureGlyph active={activity.active} tone={activity.tone} />{/if}
    <div class="command-copy" bind:this={heading} tabindex="-1">
      <h1>{focusRun ? (displayMissionTitle ?? `Work in ${focusRun.repo_root.split(/[\\/]/).pop() || "selected repository"}`) : "Work"}</h1>
      <div class="heading-meta">{#if activeDomainLabel}<span class="scope">{activeDomainLabel}</span>{/if}{#if focusState}<StateChip tone={focusState.tone} label={focusState.label} />{/if}</div>
      {#if focusRun}<p class="work-summary" role="status">{workSummary}</p>{/if}
      {#if missionText}{#key `${focusRun?.domain_id}:${focusRun?.id}`}<details class="mission-outcome"><summary>Full request</summary><p>{missionText}</p></details>{/key}{/if}
    </div>
    {#if focusRun}
      <div class="command-actions run-bar">
        <details class="run-reference"><summary>Runs</summary><div class="run-switch" role="tablist" aria-label="Runs in this snapshot">{#each runs.slice(0, 8) as run (run.id)}{@const chip = runState(run)}<button role="tab" aria-selected={run.id === focusRun?.id} data-tone={chip.tone} class:active={run.id === focusRun?.id} title={run.id} onclick={(event) => { event.currentTarget.closest("details")?.removeAttribute("open"); onSelectRun(run.id); }}><i aria-hidden="true"></i>{displayRunId(run.id)}</button>{/each}</div></details>
        {#if onInspect}<button class="details-action" onclick={() => onInspect?.({ kind: "evidence", domainId: focusRun.domain_id, runId: focusRun.id, title: "Checks & details" }, "right")}>Details</button>{/if}
        {#if runApprovals.length}<button class="attention-action" onclick={onOpenApprovals}><strong>Decision needed</strong><span>Open approval</span></button>{/if}
        <button class="review-run" class:secondary={runApprovals.length > 0} disabled={!canReviewFocusRun} aria-describedby={!canReviewFocusRun ? "review-unavailable-reason" : undefined} onclick={() => canReviewFocusRun && onReviewRun(focusRun.id)}>Review changes<IconArrowRight size={15} /></button>
        <button class="stop-run" disabled={!canStopFocusRun} aria-describedby={!canStopFocusRun ? "stop-disabled-reason" : undefined} onclick={() => requestStop(focusRun)}><IconPlayerStop size={14} />Stop</button>
        {#if !canReviewFocusRun}<span id="review-unavailable-reason" class="sr-reason">Review is available when the agents prepare changes.</span>{/if}
        {#if !canStopFocusRun}<span id="stop-disabled-reason" class="sr-reason">There is no running work to stop.</span>{/if}
      </div>
    {/if}
  </header>
  {#if stopMessage}
    <div class="work-feedback" role="status">{stopMessage}</div>
  {:else if stopError && !stopTarget}
    <div class="work-feedback error" role="alert">{stopError}</div>
  {/if}

  {#if snapshot.error}
    <div class="panel offline-panel">
      <div class="empty">
        <IconAlertTriangle size={26} />
        <strong>Local service offline</strong>
        <span>{snapshot.error.message}</span>
      </div>
    </div>
  {:else if !focusRun}
    <section class="empty-work" aria-labelledby="empty-work-title">
      <span class="empty-mark" aria-hidden="true"><IconPlus size={22} stroke={1.4} /></span>
      <h2 id="empty-work-title">What would you like to build?</h2>
      <p>Pytxo puts coding agents to work on your project. Describe a change, follow the work, and review the result before saving it.</p>
      <button class="new-run" onclick={onNewRun}><IconPlus size={15} />New work</button>
      <ol aria-label="A run in three steps"><li>Describe a change</li><li>Let agents work</li><li>Review the result</li></ol>
    </section>
  {:else}
    <div class="work-layout">
      {#if reviewError}<p class="work-feedback error" role="alert">Checks could not be loaded: {reviewError}</p>{/if}
      {#if partialAttempt}<div class="work-feedback error" role="alert"><strong>Working tree may be partially modified</strong><span>Attempt {partialAttempt.attempt_id} has no confirmed rollback. Reconcile before starting another run.</span><button onclick={() => recover(focusRun.id)}>Reconcile recovery state</button></div>{/if}
      {#if currentApplyIssue}<div class="work-feedback error" role="alert">{(review?.apply_status ?? focusRun.apply_status) === "review_failed" ? "Package preparation failed. " : ""}{review?.last_apply_error?.message ?? focusRun.last_apply_error?.message ?? "Review the run’s preparation or recovery state before continuing."}<button onclick={() => onReviewRun(focusRun.id)}>Review recovery</button></div>{/if}
      {#if routingIdentityUnknown}<div class="work-feedback error" role="alert">Routing identity could not be checked for this run. Worker selection is unavailable until the run list is repaired.</div>{/if}
      <ExecutionMap {taskDescriptions} run={focusRun} {review} {agents} {selectedAgentId} routingSummary={currentRoutingRead?.summary ?? null} routingLoading={currentRoutingRead?.loading ?? currentRoutingRead == null} {routingIdentityUnknown} onSelect={(agentId) => selectedAgentId = agentId} onInspect={(reference, position) => onInspect?.(reference, position)} {onDismissInspect} />
      {#if focusRun.routing_revision != null}<RoutingRunDetails run={focusRun} {backend} record={{ summary: currentRoutingRead?.summary ?? null, error: currentRoutingRead?.error ?? null, loading: currentRoutingRead?.loading ?? currentRoutingRead == null }} onRetry={() => routingRetry += 1} />{/if}
      {#if !onInspect}
      <BoundaryPanel
        showReviewAction={false}
        run={focusRun}
        {review}
        {reviewError}
        loading={reviewLoading}
        approvals={runApprovals}
        {onOpenApprovals}
        onReview={onReviewRun}
        onRecover={recover}
      />
      {/if}
    </div>
  {/if}
</section>

<dialog bind:this={stopDialog} class="confirm-dialog" onclose={() => (stopTarget = null)}>
  {#if stopTarget}
    <h2>Stop {stopTarget.run.id}?</h2>
    <p>
      <strong>{stopTarget.workspace}</strong>: {stopConsequence(stopTarget.run)}
    </p>
    {#if stopError}<p class="dialog-error" role="alert">{stopError}</p>{/if}
    <div class="dialog-actions">
      <button bind:this={stopCancel} class="quiet" onclick={closeStopDialog} disabled={stopping}>Keep running</button>
      <button class="danger" onclick={confirmStop} disabled={stopping}>{stopping ? "Stopping" : "Stop run"}</button>
    </div>
  {/if}
</dialog>

<style>
  .new-run{display:flex;min-height:var(--pytxo-control-height);align-items:center;gap:7px;padding:0 14px;border:1px solid transparent;border-radius:var(--pytxo-control-radius);background:var(--pytxo-text-strong);color:var(--pytxo-surface-shell);font-size:12px;font-weight:620;cursor:pointer;transition:background-color var(--pytxo-motion-fast) var(--pytxo-motion-ease)}
  .new-run:hover{background:var(--pytxo-text-soft)}
  .new-run:focus-visible{outline:2px solid var(--pytxo-accent);outline-offset:3px}
  .empty-work{display:flex;flex-direction:column;align-items:flex-start;max-width:640px;margin:48px auto;padding:32px;gap:16px}
  .empty-mark{display:grid;place-items:center;width:48px;height:48px;border:1px solid var(--pytxo-line);border-radius:12px;background:var(--pytxo-surface-panel);color:var(--pytxo-text-soft)}
  .empty-work h2{margin:0;font-size:24px;font-weight:600;letter-spacing:-.035em;text-wrap:balance}
  .empty-work p{max-width:52ch;margin:0;color:var(--pytxo-text-soft);font-size:13px;line-height:1.7;text-wrap:pretty}
  .empty-work ol{display:flex;flex-wrap:wrap;gap:10px 24px;margin:12px 0 0;padding:0;list-style:none;counter-reset:step}
  .empty-work li{display:flex;align-items:center;gap:8px;color:var(--pytxo-text-muted);font-size:11px;counter-increment:step}
  .empty-work li::before{content:counter(step);display:grid;place-items:center;width:20px;height:20px;border:1px solid var(--pytxo-line);border-radius:50%;font:10px "IBM Plex Mono",monospace}

  .work-feedback{display:flex;flex-wrap:wrap;align-items:center;gap:8px 12px;padding:9px 12px;border:1px solid var(--pytxo-line-soft);border-radius:4px;background:var(--pytxo-surface-panel);font-size:12px}
  .work-feedback.error{border-color:color-mix(in srgb,var(--state-refuted) 48%,var(--pytxo-line));color:var(--state-refuted)}


  .stop-run{display:flex;min-height:40px;align-items:center;gap:6px;padding:0 12px;border:1px solid var(--pytxo-line);border-radius:var(--pytxo-control-radius);background:transparent;color:var(--pytxo-text-soft);font-size:12px;cursor:pointer;transition:background-color var(--pytxo-motion-fast) var(--pytxo-motion-ease),border-color var(--pytxo-motion-fast) var(--pytxo-motion-ease)}
  .stop-run:hover{border-color:var(--state-refuted);color:var(--state-refuted)}
  .stop-run:disabled{cursor:not-allowed;opacity:.42}
  .review-run{display:flex;min-height:40px;align-items:center;justify-content:center;gap:7px;padding:0 12px;border:1px solid var(--pytxo-line);border-radius:var(--pytxo-control-radius);background:var(--pytxo-text-strong);color:var(--pytxo-surface-shell);font-size:12px;cursor:pointer;white-space:nowrap}
  .review-run:disabled{opacity:.45;cursor:not-allowed;background:transparent;color:var(--pytxo-text-muted)}
  .review-run:hover:not(:disabled){background:var(--pytxo-text-soft)}
  .review-run:focus-visible{outline:2px solid var(--pytxo-accent);outline-offset:3px}
  .review-run.secondary{background:transparent;color:var(--pytxo-text-strong)}
  .review-run.secondary:hover:not(:disabled){background:var(--pytxo-surface-raised)}
  .attention-action{display:grid;min-height:40px;align-content:center;gap:1px;padding:5px 12px;border:1px solid #f5f5f7;border-radius:var(--pytxo-control-radius);background:#f5f5f7;color:#07090c;text-align:left;cursor:pointer}
  .attention-action strong{font-size:12px}.attention-action span{font-size:10px;opacity:.72}.attention-action:focus-visible{outline:2px solid var(--pytxo-accent);outline-offset:3px}
  .run-switch{display:flex;min-width:0;flex-wrap:wrap;gap:6px}
  .run-switch button{display:flex;height:26px;align-items:center;gap:6px;padding:0 9px;border:1px solid var(--pytxo-line-soft);border-radius:4px;background:transparent;color:var(--pytxo-text-muted);font:11px "IBM Plex Mono",monospace;cursor:pointer}
  .run-switch button i{width:6px;height:6px;border:1px solid var(--tone);border-radius:1px;background:var(--tone)}
  .run-switch button.active{border-color:var(--pytxo-line);background:var(--pytxo-surface-active);color:var(--pytxo-text-strong)}

  .confirm-dialog{position:fixed;inset:0;width:min(430px,calc(100vw - 36px));margin:auto;padding:18px;border:1px solid var(--pytxo-line);border-radius:8px;background:var(--pytxo-surface-panel);color:var(--pytxo-text-strong);box-shadow:0 24px 80px color-mix(in oklab,var(--pytxo-surface-shell) 82%,transparent)}
  .confirm-dialog::backdrop{background:color-mix(in oklab,black 62%,transparent)}
  .confirm-dialog h2{margin:0 0 8px;font-size:15px;font-weight:640;letter-spacing:-.02em}
  .confirm-dialog p{margin:0;color:var(--pytxo-text-soft);font-size:12px;line-height:1.55}
  .dialog-error{margin-top:8px!important;color:var(--state-refuted)}
  .dialog-actions{display:flex;justify-content:flex-end;gap:9px;margin-top:16px}
  .dialog-actions button{height:32px;padding:0 12px;border-radius:4px;font-size:12px;font-weight:620;cursor:pointer}
  .dialog-actions .quiet{border:1px solid var(--pytxo-line);background:transparent;color:var(--pytxo-text-soft)}
  .dialog-actions .danger{border:1px solid var(--state-refuted);background:transparent;color:var(--state-refuted)}
  .dialog-actions button:disabled{cursor:not-allowed;opacity:.45}

  /* Work is a cockpit. The shell owns its height; only explicit inspectors scroll. */
  .screen.work {
    display:flex;
    width:100%;
    max-width:none;
    height:100%;
    min-height:0;
    box-sizing:border-box;
    flex-direction:column;
    align-items:stretch;
    gap:9px;
    padding:10px 14px 12px;
    overflow:hidden;
  }
  .command-strip {
    position:relative;
    z-index:2;
    display:grid;
    grid-template-columns:64px minmax(0,1fr) auto;
    min-height:92px;
    flex:0 0 auto;
    align-items:center;
    gap:12px;
    padding:7px 0 9px;
    border-bottom:1px solid var(--pytxo-line);
  }
  .command-copy { min-width:0; outline:0; }
  .command-copy:focus-visible { outline:2px solid var(--pytxo-accent); outline-offset:3px; }
  .command-copy h1 { display:block;max-width:46ch;margin:0;overflow:hidden;font-size:clamp(19px,1.7vw,25px);font-weight:650;line-height:1.2;letter-spacing:-.035em;text-overflow:ellipsis;white-space:nowrap; }
  .heading-meta { display:flex;align-items:center;gap:8px;margin-top:5px; }
  .heading-meta .scope { overflow:hidden;color:var(--pytxo-text-muted);font:10px "IBM Plex Mono",monospace;text-overflow:ellipsis;white-space:nowrap; }
  .work-summary { max-width:72ch;margin:5px 0 0;overflow:hidden;color:var(--pytxo-text-soft);font-size:12px;line-height:1.35;text-overflow:ellipsis;white-space:nowrap; }
  .mission-outcome { position:relative;display:inline-block;margin-top:2px;font-size:10px;line-height:1.4; }
  .mission-outcome summary { min-height:18px;color:var(--pytxo-text-muted);cursor:pointer; }
  .mission-outcome p { position:absolute;z-index:12;top:100%;left:0;width:min(620px,70vw);max-height:240px;margin:4px 0 0;padding:12px;overflow:auto;border:1px solid var(--pytxo-line);border-radius:5px;background:var(--pytxo-surface-raised);box-shadow:0 16px 40px #0008;color:var(--pytxo-text-body);white-space:pre-wrap; }
  .command-actions { display:flex;align-items:center;justify-content:flex-end;gap:7px; }
  .command-actions>button,.run-reference>summary { min-height:34px;padding:0 10px;border:1px solid var(--pytxo-line);border-radius:4px;background:transparent;color:var(--pytxo-text-soft);font:11px var(--pytxo-font-ui);cursor:pointer; }
  .command-actions>button:focus-visible,.run-reference>summary:focus-visible { outline:2px solid var(--pytxo-accent);outline-offset:2px; }
  .details-action:hover,.run-reference>summary:hover { background:var(--pytxo-surface-active);color:var(--pytxo-text-strong); }
  .command-actions .review-run { min-height:36px;border-color:var(--pytxo-text-strong);background:var(--pytxo-text-strong);color:var(--pytxo-surface-shell); }
  .command-actions .review-run.secondary { background:transparent;color:var(--pytxo-text-strong); }
  .command-actions .stop-run { min-height:36px; }
  .command-actions .attention-action { min-height:36px;padding-inline:10px;border-color:#f5f5f7;background:#f5f5f7;color:#07090c; }
  :global(html[data-chroma-theme="light"]) .command-actions .attention-action { border-color:#0f1419;background:#0f1419;color:#f8fafc; }
  .run-reference { position:relative;order:0;margin:0;font-size:11px; }
  .run-reference[open] { width:auto;flex-basis:auto;order:0; }
  .run-reference summary { display:grid;place-items:center;list-style:none; }
  .run-reference .run-switch { position:absolute;z-index:12;top:calc(100% + 6px);right:0;display:grid;width:220px;padding:6px;border:1px solid var(--pytxo-line);border-radius:5px;background:var(--pytxo-surface-raised);box-shadow:0 16px 40px #0008; }
  .run-switch button { width:100%;justify-content:flex-start; }
  .sr-reason { position:absolute;width:1px;height:1px;overflow:hidden;clip:rect(0,0,0,0);white-space:nowrap; }
  .work-layout { display:flex;flex:1;min-height:0;flex-direction:column;gap:8px;overflow:hidden;container:work-space / inline-size; }
  .work-layout>.work-feedback { flex:0 0 auto;order:0;margin:0; }
  .work-layout :global(.execution-map) { flex:1;min-height:0; }
  .work-layout :global(.routing-record) { flex:0 1 auto;min-height:0;max-height:min(44%,360px);overflow:auto;overscroll-behavior:contain; }
  .work-feedback { flex:0 0 auto;margin:0; }
  .work-feedback button { min-height:28px;margin-left:auto;padding:3px 8px;border:1px solid var(--pytxo-line);border-radius:4px;background:transparent;color:inherit;font:inherit;cursor:pointer; }
  .offline-panel,.empty-work { flex:1;min-height:0;overflow:auto; }
  .empty-work { margin:auto;padding:20px; }

  @container mission (max-width:880px) {
    .command-strip { grid-template-columns:52px minmax(0,1fr);min-height:112px;align-content:center; }
    .command-actions { grid-column:1 / -1;justify-content:flex-start;flex-wrap:wrap;padding-bottom:2px; }
    .run-reference .run-switch { right:auto;left:0; }
    .command-copy h1 { font-size:20px; }
    .work-summary { max-width:100%; }
  }
  @container mission (max-width:520px) {
    .screen.work { padding:8px; }
    .command-strip { grid-template-columns:minmax(0,1fr);min-height:126px;gap:5px; }
    .command-strip :global(.aperture-glyph) { display:none; }
    .command-actions { gap:5px; }
    .command-actions>button,.run-reference>summary { padding-inline:8px;white-space:nowrap; }
    .details-action,.run-reference,.attention-action { display:none; }
  }
</style>
