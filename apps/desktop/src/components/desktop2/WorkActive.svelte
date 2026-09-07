<script lang="ts">
  import { IconAlertTriangle, IconPlayerStop, IconPlus } from "@tabler/icons-svelte";
  import type { DesktopBackend, DesktopSnapshot } from "../../lib/desktop-backend";
  import { runState, waveProgress } from "../../lib/epistemic";
  import type { RunDto, RunReviewDto } from "../../lib/types";
  import BoundaryPanel from "./BoundaryPanel.svelte";
  import RunLedger from "./RunLedger.svelte";
  import StateChip from "./StateChip.svelte";

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
  } = $props();

  let heading: HTMLElement | undefined = $state();
  let runButtons: HTMLButtonElement[] = $state([]);
  let stopDialog: HTMLDialogElement | undefined = $state();
  let stopCancel: HTMLButtonElement | undefined = $state();
  let stopTarget = $state<{ run: RunDto; domainId: string; workspace: string } | null>(null);
  let stopError = $state("");
  let stopMessage = $state("");
  let stopping = $state(false);
  let selectedAgentId = $state<string | null>(null);

  let loadedReview = $state<RunReviewDto | null>(null);
  let reviewError = $state<string | null>(null);
  let reviewLoading = $state(false);

  const ACTIVE_STATUSES = ["starting", "running", "pending", "dispatching", "active"];

  const runs = $derived(
    activeDomainId
      ? snapshot.runs.filter((run) => run.domain_id === activeDomainId)
      : snapshot.runs,
  );
  const focusRun = $derived(
    runs.find((run) => run.id === focusRunId) ??
      runs.find((run) => ACTIVE_STATUSES.includes(run.status.toLowerCase())) ??
      runs[0] ??
      null,
  );
  const agents = $derived(focusRun ? snapshot.agents.filter((agent) => agent.run_id === focusRun.id) : []);
  const progress = $derived(waveProgress(agents));
  const focusState = $derived(focusRun ? runState(focusRun) : null);
  const review = $derived(loadedReview?.run_id === focusRun?.id ? loadedReview : null);
  const receiptProfile = $derived((review?.enforcement?.run?.effective_profile ?? "").toLowerCase());
  const canStopFocusRun = $derived(!!focusRun && ACTIVE_STATUSES.includes(focusRun.status.toLowerCase()));
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
    reviewLoading = true;
    reviewError = null;
    backend
      .runReview(runId, focusRun?.domain_id ?? null)
      .then((result) => {
        if (!current) return;
        loadedReview = result;
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

  export function focusActiveRun() {
    queueMicrotask(() => (runButtons[0] ?? heading)?.focus());
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
    queueMicrotask(() => {
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
      stopMessage = `Recovery reconciled for ${runId}.`;
    } catch (error) {
      stopError = error instanceof Error ? error.message : String(error);
    }
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

<section class="screen work">
  <header class="work-heading">
    <div bind:this={heading} tabindex="-1">
      <h1>Work</h1>
      {#if activeDomainLabel}<span class="scope">{activeDomainLabel}</span>{/if}
    </div>
    <button class="new-run" onclick={onNewRun}><IconPlus size={15} />New run</button>
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
  {:else}
    <section class="run-bar" aria-label="Focused run">
      {#if runs.length > 1}
        <div class="run-switch" role="tablist" aria-label="Runs in this snapshot">
          {#each runs.slice(0, 8) as run, index (run.id)}
            {@const chip = runState(run)}
            <button
              bind:this={runButtons[index]}
              role="tab"
              aria-selected={run.id === focusRun?.id}
              data-tone={chip.tone}
              class:active={run.id === focusRun?.id}
              onclick={() => onSelectRun(run.id)}
            >
              <i aria-hidden="true"></i>{run.id}
            </button>
          {/each}
        </div>
      {:else}
        <strong class="run-id">{focusRun?.id ?? "No run in this snapshot"}</strong>
      {/if}
      {#if focusState}
        <StateChip tone={focusState.tone} label={focusState.label} detail={focusState.detail} />
      {/if}
      <dl>
        <div>
          <dt>Waves</dt>
          <dd>{progress ? `${progress.settled} of ${progress.total} settled · ${progress.successful} completed` : "Not reported"}</dd>
        </div>
        <div>
          <dt>Isolation</dt>
          <dd>{focusRun ? `${focusRun.isolation_mode} · ${focusRun.isolation_backend}` : "Not reported"}</dd>
        </div>
      </dl>
      {#if focusRun}
        <div class="stop-action">
          <button class="stop-run" disabled={!canStopFocusRun} aria-describedby={!canStopFocusRun ? "stop-disabled-reason" : undefined} onclick={() => requestStop(focusRun)}><IconPlayerStop size={14} />Stop</button>
          {#if !canStopFocusRun}<small id="stop-disabled-reason">Only an active run can be stopped.</small>{/if}
        </div>
      {/if}
    </section>

    <div class="work-layout">
      <RunLedger
        {agents}
        plan={review?.plan ?? null}
        agentReceipts={review?.enforcement?.agents ?? null}
        {selectedAgentId}
        onSelect={(agentId) => (selectedAgentId = selectedAgentId === agentId ? null : agentId)}
      />
      <BoundaryPanel
        run={focusRun}
        {review}
        {reviewError}
        loading={reviewLoading}
        approvals={runApprovals}
        {onOpenApprovals}
        onReview={onReviewRun}
        onRecover={recover}
      />
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
  .work-heading{display:flex;align-items:baseline;justify-content:space-between;gap:16px}
  .work-heading>div{display:flex;align-items:baseline;gap:10px}
  .work-heading h1{margin:0;font-size:22px;font-weight:640;letter-spacing:-.025em}
  .work-heading .scope{color:var(--pytxo-text-muted);font:12px "IBM Plex Mono",monospace}
  .work-heading div:focus-visible{outline:2px solid var(--pytxo-accent);outline-offset:4px}
  .new-run{display:flex;height:32px;align-items:center;gap:6px;padding:0 12px;border:1px solid transparent;border-radius:4px;background:var(--pytxo-text-strong);color:var(--pytxo-surface-shell);font-size:12px;font-weight:620;cursor:pointer}
  .new-run:hover{background:var(--pytxo-text-soft)}

  .work-feedback{padding:9px 12px;border:1px solid var(--pytxo-line-soft);border-left:2px solid var(--state-verified);border-radius:4px;background:var(--pytxo-surface-panel);font-size:12px}
  .work-feedback.error{border-left-color:var(--state-refuted);color:var(--state-refuted)}

  .run-bar{display:flex;align-items:center;gap:18px;padding:9px 12px;border:1px solid var(--pytxo-line);border-radius:var(--pytxo-panel-radius,6px);background:var(--pytxo-surface-panel)}
  .run-id{overflow:hidden;font:13px/1.2 "IBM Plex Mono",monospace;text-overflow:ellipsis;white-space:nowrap}
  .run-bar dl{display:flex;gap:22px;margin:0 0 0 auto}
  .run-bar dt{color:var(--pytxo-text-muted);font-size:11px}
  .run-bar dd{margin:2px 0 0;color:var(--pytxo-text-soft);font-size:12px}
  .stop-run{display:flex;height:28px;align-items:center;gap:6px;padding:0 10px;border:1px solid var(--pytxo-line);border-radius:4px;background:transparent;color:var(--pytxo-text-soft);font-size:11px;cursor:pointer}
  .stop-run:hover{border-color:var(--state-refuted);color:var(--state-refuted)}
  .stop-run:disabled{cursor:not-allowed;opacity:.42}
  .stop-action{display:flex;flex-direction:column;align-items:flex-end;gap:3px}
  .stop-action small{max-width:150px;color:var(--pytxo-text-muted);font-size:10px;text-align:right}

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

  .work-layout{display:grid;min-height:0;grid-template-columns:minmax(0,1.62fr) minmax(330px,1fr);gap:14px;align-items:start}
  @media(max-width:1180px){.work-layout{grid-template-columns:minmax(0,1fr)}.run-bar{flex-wrap:wrap;gap:14px}.run-bar dl{margin-left:0}}
</style>
