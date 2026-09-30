<script lang="ts">
  import { withPreviewsHidden } from "../../lib/preview-overlay";
  import IconArrowRight from "@tabler/icons-svelte/icons/arrow-right";
  import { canOpenRunReview } from "../../lib/review-state";
  import IconAlertTriangle from "@tabler/icons-svelte/icons/alert-triangle";
  import IconPlayerStop from "@tabler/icons-svelte/icons/player-stop";
  import IconPlus from "@tabler/icons-svelte/icons/plus";
  import type { DesktopBackend, DesktopSnapshot } from "../../lib/desktop-backend";
  import { runState, isPartiallyApplied } from "../../lib/epistemic";
  import type { RunDto, RunReviewDto } from "../../lib/types";
  import BoundaryPanel from "./BoundaryPanel.svelte";
  import RunLedger from "./RunLedger.svelte";
  import RoutingRunDetails from "./RoutingRunDetails.svelte";
  import StateChip from "./StateChip.svelte";
  import type { DockReference } from "../../lib/dock-layout";
  import { selectMissionRun } from "../../lib/mission-selection";

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
    onInspect?: ((ref: DockReference) => void) | null;
  } = $props();

  let heading: HTMLElement | undefined = $state();
  let stopDialog: HTMLDialogElement | undefined = $state();
  let stopCancel: HTMLButtonElement | undefined = $state();
  let stopTarget = $state<{ run: RunDto; domainId: string; workspace: string } | null>(null);
  let stopError = $state("");
  let stopMessage = $state("");
  let stopping = $state(false);
  let selectedAgentId = $state<string | null>(null);

  let loadedReview = $state<RunReviewDto | null>(null);
  let loadedReviewDomain = $state<string | null>(null);
  let reviewError = $state<string | null>(null);
  let reviewLoading = $state(false);
  let missionTitle = $state<string | null>(null);
  let missionText = $state<string | null>(null);
  const displayMissionTitle = $derived(missionTitle && missionText?.startsWith(missionTitle) && missionText.length > missionTitle.length ? `${missionTitle.trimEnd()}…` : missionTitle);
  $effect(() => {
    const id = focusRun?.id;
    const domain = focusRun?.domain_id;
    missionTitle = null; missionText = null;
    if (!id) return;
    let valid = true;
    backend.flowHistory().then(records => {
      if (!valid) return;
      const draft = records.find(d => d.dispatched_run_id === id && d.domain_id === domain);
      missionTitle = draft?.title ?? null; missionText = draft?.mission_text ?? null;
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
  const agents = $derived(focusRun ? snapshot.agents.filter((agent) => agent.run_id === focusRun.id && agent.domain_id === focusRun.domain_id) : []);
  const focusState = $derived(focusRun ? runState(focusRun) : null);
  const review = $derived(loadedReview?.run_id === focusRun?.id && loadedReviewDomain === focusRun?.domain_id ? loadedReview : null);
  const partialAttempt = $derived(review?.apply_attempts.find(isPartiallyApplied) ?? null);
  const receiptProfile = $derived((review?.enforcement?.run?.effective_profile ?? "").toLowerCase());
  const canReviewFocusRun = $derived(canOpenRunReview(focusRun, review));
  const canStopFocusRun = $derived(!!focusRun && ACTIVE_STATUSES.includes(focusRun.status.toLowerCase()));
  const checks = $derived(review?.prepared_manifest?.candidate_verification?.checks ?? []);
  const workSummary = $derived.by(() => {
    if (!focusRun) return "";
    if (partialAttempt) return "Some project files may have changed. Check recovery before continuing.";
    if (review?.last_apply_error || focusRun.last_apply_error) return "Something needs your attention before these changes can be saved.";
    if (review?.recovery_state || focusRun.recovery_state || ["review_failed", "recovery_required"].includes(review?.apply_status ?? focusRun.apply_status ?? "")) return "The changes need attention. Review the problem before continuing.";
    if (focusRun.applied_at) return "These changes have been saved to your project.";
    if (["starting", "pending", "dispatching"].includes(focusRun.status.toLowerCase())) return "Getting the work ready. Your agents have not reported a result yet.";
    if (focusRun.status === "failed_startup") return "The work could not start. Open details to see what needs fixing.";
    if (canStopFocusRun) return "Your agents are working on this request. You can follow their progress below.";
    if (["failed", "stopped", "cancelled"].includes(focusRun.status.toLowerCase())) return "The work stopped before finishing. Open details to see what happened.";
    if (checks.some(check => !check.passed)) return "Some checks failed. Review the results before deciding what to do next.";
    if (canReviewFocusRun) return review?.prepared_manifest?.version === 3 && review.prepared_manifest.candidate_verification?.version === 1 && checks.length > 0 && checks.every(check => check.passed)
      ? "The changes are ready for your review. The saved checks passed."
      : "The changes are ready for your review. Check the results before saving them.";
    return focusRun.status === "completed" ? "The agents have reported back. Details show what is ready and what still needs checking." : "The current work status needs checking. Open details before continuing.";
  });
  const runApprovals = $derived(
    snapshot.approvals.filter((approval) => !activeDomainId || approval.domain_id === activeDomainId),
  );

  /**
   * The enforcement receipt is the boundary panel's whole substance, so it is
   * fetched per focused run. A stale response from a previous run is discarded
   * rather than rendered against the wrong candidate.
   */
  $effect(() => {
    const runId = focusRun?.id ?? null;
    if (!runId) {
      loadedReview = null;
      reviewError = null;
      return;
    }
    let current = true;
    const domainId = focusRun?.domain_id ?? null;
    reviewLoading = true;
    reviewError = null;
    backend
      .runReview(runId, focusRun?.domain_id ?? null)
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
  <header class="work-heading">
    <div bind:this={heading} tabindex="-1">
      <h1>{focusRun ? (displayMissionTitle ?? "Your coding task") : "Work"}</h1>
      {#if activeDomainLabel}<span class="scope">Project · {activeDomainLabel}</span>{/if}
    </div>
  </header>
  {#if missionText}
    {#key `${focusRun?.domain_id}:${focusRun?.id}`}
      <details class="mission-outcome">
        <summary>Read full request</summary>
        <p>{missionText}</p>
      </details>
    {/key}
  {/if}
  {#if focusRun}<p class="work-summary">{workSummary}</p>{/if}

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
      <button class="new-run" onclick={onNewRun}><IconPlus size={15} />New run</button>
      <ol aria-label="A run in three steps"><li>Describe a change</li><li>Let agents work</li><li>Review the result</li></ol>
    </section>
  {:else}
    <section class="run-bar" aria-label="Focused run">
      {#if focusState}<StateChip tone={focusState.tone} label={focusState.label} />{/if}
      <details class="run-reference"><summary>Other runs &amp; IDs</summary>
        <div class="run-switch" role="tablist" aria-label="Runs in this snapshot">
          {#each runs.slice(0, 8) as run (run.id)}
            {@const chip = runState(run)}
            <button role="tab" aria-selected={run.id === focusRun?.id} data-tone={chip.tone} class:active={run.id === focusRun?.id} title={run.id} onclick={() => onSelectRun(run.id)}><i aria-hidden="true"></i>{displayRunId(run.id)}</button>
          {/each}
        </div>
      </details>
      {#if focusRun}
        <div class="run-actions">
          <div class="review-action">
            <button class="review-run" disabled={!canReviewFocusRun}
              aria-describedby={!canReviewFocusRun ? "review-unavailable-reason" : undefined}
              onclick={() => canReviewFocusRun && onReviewRun(focusRun.id)}>
              Review changes<IconArrowRight size={15} />
            </button>
            {#if !canReviewFocusRun}<small id="review-unavailable-reason">Available when the agents prepare changes.</small>{/if}
          </div>
        <div class="stop-action">
          <button class="stop-run" disabled={!canStopFocusRun} aria-describedby={!canStopFocusRun ? "stop-disabled-reason" : undefined} onclick={() => requestStop(focusRun)}><IconPlayerStop size={14} />Stop</button>
          {#if !canStopFocusRun}<small id="stop-disabled-reason">There is no running work to stop.</small>{/if}
        </div>
        </div>
      {/if}
    </section>

    <div class="work-layout">
      {#if focusRun.routing_revision != null}<RoutingRunDetails run={focusRun} {backend} />{/if}
      <RunLedger
        {agents}
        plan={review?.plan ?? null}
        agentReceipts={review?.enforcement?.agents ?? null}
        {selectedAgentId}
        inlineInspector={!onInspect}
        onSelect={(agentId) => {
          selectedAgentId = agentId;
          const agent = agents.find(a => a.id === agentId);
          if (agent) onInspect?.({ kind: "agent", domainId: agent.domain_id, runId: agent.run_id, agentId, title: agent.task_id });
        }}
      />
      {#if onInspect}
        <section class="mission-boundary" aria-label="Checks and details">
          <span>{focusRun.applied_at ? "Changes saved to your project." : (receiptProfile === "orbit" || receiptProfile === "galaxy") && review?.enforcement?.run?.workspace_isolation.status === "enforced" && review?.enforcement?.run?.apply_boundary.status === "enforced" ? "Changes stay separate until you choose Apply." : receiptProfile === "supernova" ? "Agents can change this project directly." : receiptProfile === "deepspace" ? "This job cannot apply changes to your project." : "Check permissions before trusting changes to your project."}</span>
          <button onclick={() => onInspect?.({ kind: "evidence", domainId: focusRun.domain_id, runId: focusRun.id, title: "Checks & details" })}>View details</button>
        </section>
        {#if reviewError}<p class="work-feedback error" role="alert">Checks could not be loaded: {reviewError}</p>{/if}
        {#if partialAttempt}
          <div class="work-feedback error" role="alert"><strong>Working tree may be partially modified</strong><span>Attempt {partialAttempt.attempt_id} has no confirmed rollback. Reconcile before starting another run.</span><button onclick={() => recover(focusRun.id)}>Reconcile recovery state</button></div>
        {/if}
        {#if runApprovals.length}<button class="work-feedback" onclick={onOpenApprovals}>A decision is waiting · Open approvals</button>{/if}
        {#if review?.last_apply_error || focusRun.last_apply_error || review?.recovery_state || focusRun.recovery_state || ["review_failed", "recovery_required"].includes(review?.apply_status ?? focusRun.apply_status ?? "")}
          <div class="work-feedback error" role="alert">{(review?.apply_status ?? focusRun.apply_status) === "review_failed" ? "Package preparation failed. " : ""}{review?.last_apply_error?.message ?? focusRun.last_apply_error?.message ?? "Review the run’s preparation or recovery state before continuing."}<button onclick={() => onReviewRun(focusRun.id)}>Review recovery</button></div>
        {/if}
      {:else}
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
  .work{display:flex;flex-direction:column;gap:14px}
  .mission-outcome{margin:0;max-width:84ch;white-space:pre-wrap;overflow-wrap:anywhere;color:var(--pytxo-text-soft);font-size:13px;line-height:1.6}
  .mission-outcome summary{cursor:pointer;font-size:12px;color:var(--pytxo-text-muted)}.mission-outcome p{margin:8px 0 0}.mission-outcome summary:focus-visible{outline:2px solid var(--pytxo-accent);outline-offset:3px}
  .work-summary{margin:0;max-width:65ch;color:var(--pytxo-text-soft);font-size:15px;line-height:1.6}
  .run-reference{font-size:12px;color:var(--pytxo-text-muted)}.run-reference summary{cursor:pointer}.run-reference[open]{flex-basis:100%;order:3}.run-reference .run-switch{padding-top:10px}
  .work-heading{display:flex;align-items:baseline;justify-content:space-between;gap:16px}
  .work-heading>div{display:flex;align-items:baseline;gap:10px}
  .work-heading h1{margin:0;min-width:0;overflow-wrap:anywhere;font-size:clamp(24px,2.4vw,34px);font-weight:640;letter-spacing:-.025em}
  .work-heading .scope{color:var(--pytxo-text-muted);font:12px "IBM Plex Mono",monospace}
  .work-heading div:focus-visible{outline:2px solid var(--pytxo-accent);outline-offset:4px}
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

  .work-feedback{padding:9px 12px;border:1px solid var(--pytxo-line-soft);border-left:2px solid var(--state-verified);border-radius:4px;background:var(--pytxo-surface-panel);font-size:12px}
  .work-feedback.error{border-left-color:var(--state-refuted);color:var(--state-refuted)}


  .stop-run{display:flex;min-height:40px;align-items:center;gap:6px;padding:0 12px;border:1px solid var(--pytxo-line);border-radius:var(--pytxo-control-radius);background:transparent;color:var(--pytxo-text-soft);font-size:12px;cursor:pointer;transition:background-color var(--pytxo-motion-fast) var(--pytxo-motion-ease),border-color var(--pytxo-motion-fast) var(--pytxo-motion-ease)}
  .stop-run:hover{border-color:var(--state-refuted);color:var(--state-refuted)}
  .stop-run:disabled{cursor:not-allowed;opacity:.42}
  .stop-action{display:flex;flex-direction:column;align-items:flex-end;gap:4px}
  .stop-action small, .review-action small{max-width:150px;color:var(--pytxo-text-muted);font-size:10px;text-align:right}

  .run-actions{display:flex;grid-column:3;grid-row:1;align-items:flex-start;gap:10px}
  .review-action{display:flex;flex-direction:column;align-items:flex-end;gap:4px}
  .review-run{display:flex;min-height:40px;align-items:center;justify-content:center;gap:7px;padding:0 12px;border:1px solid var(--pytxo-line);border-radius:var(--pytxo-control-radius);background:var(--pytxo-text-strong);color:var(--pytxo-surface-shell);font-size:12px;cursor:pointer;white-space:nowrap}
  .review-run:disabled{opacity:.45;cursor:not-allowed;background:transparent;color:var(--pytxo-text-muted)}
  .review-run:hover:not(:disabled){background:var(--pytxo-text-soft)}
  .review-run:focus-visible{outline:2px solid var(--pytxo-accent);outline-offset:3px}
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

  .mission-boundary{display:flex;flex-wrap:wrap;align-items:center;gap:12px 20px;order:-1;padding:12px 0;border-block:1px solid var(--pytxo-line-soft);font-size:12px;color:var(--pytxo-text-soft)}.mission-boundary button,.work-feedback button{min-height:30px;padding:4px 8px;margin-left:auto;border:1px solid var(--pytxo-line);border-radius:4px;background:transparent;color:var(--pytxo-text-strong);font:inherit;cursor:pointer}
  .work-layout{display:flex;min-height:0;flex-direction:column;gap:14px;align-items:stretch}
  .work-heading>div{min-width:0;display:block}.work-heading .scope{display:block;margin-top:8px}.work-layout :global(.ledger){border-inline:0;border-radius:0}
  @media(max-width:760px){.run-actions{flex-wrap:wrap;justify-content:flex-start}.review-action,.stop-action{align-items:flex-start}.run-actions small{text-align:left;max-width:180px}.empty-work{margin:12px 0;padding:16px}.empty-work h2{font-size:21px}.scope{overflow-wrap:anywhere}}
  .run-bar{display:flex;flex-wrap:wrap;align-items:center;gap:12px;padding:4px 0 16px;border:0;border-radius:0;background:transparent;border-bottom:1px solid var(--pytxo-line-soft)}
  .run-actions{margin-left:auto;align-items:center}.run-reference{margin-left:8px}.work-heading h1{max-width:32ch;line-height:1.2}
  .mission-boundary{order:0;border-top:0;font-size:12px}.work-layout{gap:12px}
  @media(max-width:760px){.run-actions{margin-left:0;width:100%}.work-heading h1{font-size:26px}}
</style>
