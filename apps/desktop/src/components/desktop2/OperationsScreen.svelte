<script lang="ts">
  import {
    IconAlertTriangle,
    IconClockHour4,
    IconLock,
    IconPlayerStop,
    IconShieldLock,
  } from "@tabler/icons-svelte";
  import { approvalPresentation } from "../../lib/approval-presentation";
  import type { DesktopSnapshot } from "../../lib/desktop-backend";
  import type { RunDto } from "../../lib/types";

  let {
    snapshot,
    activeDomainLabel = null,
    onNewMission,
    onOpenApprovals,
    onReviewRun,
    onStopRun,
  }: {
    snapshot: DesktopSnapshot;
    activeDomainLabel?: string | null;
    onNewMission: () => void;
    onOpenApprovals: () => void;
    onReviewRun: (runId: string) => void;
    onStopRun: (runId: string, domainId: string) => Promise<void>;
  } = $props();

  type StopTarget = {
    run: RunDto;
    domainId: string;
    workspace: string;
  };

  let operationsHeading: HTMLElement | undefined = $state();
  let activeRunButtons: HTMLButtonElement[] = $state([]);
  let stopDialog: HTMLDialogElement | undefined = $state();
  let stopCancel: HTMLButtonElement | undefined = $state();
  let stopTarget = $state<StopTarget | null>(null);
  let stopError = $state("");
  let stopMessage = $state("");
  let stopping = $state(false);

  const spendUsd = $derived(
    snapshot.runs.reduce((sum, run) => sum + (run.estimated_cost_usd ?? 0), 0),
  );
  const activeRunList = $derived(
    snapshot.runs
      .filter((run) => ["running", "pending", "dispatching", "active"].includes(run.status.toLowerCase()))
      .slice(0, 12),
  );
  const raceWaves = $derived(
    [...new Set(snapshot.agents.map((agent) => agent.wave))].sort((a, b) => a - b),
  );
  const contendedAgents = $derived(
    snapshot.agents.filter((agent) => {
      const status = agent.status.toLowerCase();
      return (
        status.includes("block") ||
        status.includes("wait") ||
        status.includes("hold") ||
        status.includes("claim") ||
        status === "queued"
      );
    }),
  );

  function domainForRun(run: RunDto) {
    return snapshot.domains.find((domain) => domain.repo_root === run.repo_root) ?? null;
  }

  function isEditableTarget(target: EventTarget | null): boolean {
    if (!(target instanceof HTMLElement)) return false;
    return (
      target.isContentEditable ||
      target.tagName === "INPUT" ||
      target.tagName === "TEXTAREA" ||
      target.tagName === "SELECT"
    );
  }

  export function focusActiveRun() {
    queueMicrotask(() => (activeRunButtons[0] ?? operationsHeading)?.focus());
  }

  function requestStop(run: RunDto) {
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

  function onRunKeydown(event: KeyboardEvent, run: RunDto) {
    if (
      event.defaultPrevented ||
      event.repeat ||
      isEditableTarget(event.target) ||
      !(event.metaKey || event.ctrlKey) ||
      !event.shiftKey ||
      event.altKey ||
      event.key !== "Backspace"
    ) {
      return;
    }
    event.preventDefault();
    requestStop(run);
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
</script>

<section class="screen operations">
  <header class="screen-heading">
    <div bind:this={operationsHeading} tabindex="-1">
      <h1>Ops{#if activeDomainLabel} <span class="ops-domain">{activeDomainLabel}</span>{/if}</h1>
    </div>
    <button class="primary" onclick={onNewMission}>New mission</button>
  </header>

  {#if stopMessage}
    <div class="ops-feedback" role="status">{stopMessage}</div>
  {:else if stopError && !stopTarget}
    <div class="ops-feedback error" role="alert">{stopError}</div>
  {/if}

  {#if snapshot.error}
    <div class="panel">
      <div class="empty">
        <IconAlertTriangle size={26} />
        <strong>Local service offline</strong>
        <span>{snapshot.error.message}</span>
      </div>
    </div>
  {:else}
    <p class="ops-spend">Estimated spend ${spendUsd.toFixed(2)}</p>

    {#if snapshot.approvals.length}
      <article class="panel approval-panel">
        <div class="panel-head">
          <div><h2>Needs you</h2></div>
          <button class="quiet" onclick={onOpenApprovals}>Open inbox</button>
        </div>
        {#each snapshot.approvals as approval}
          {@const presentation = approvalPresentation(approval)}
          <button class="decision-card" onclick={onOpenApprovals}>
            <span class="risk">{presentation.category}</span>
            <h3>{presentation.title}</h3>
            <p>{approval.reason}</p>
            <small>{approval.agent_key} · {approval.domain_id}</small>
          </button>
        {/each}
      </article>
    {/if}

    <article class="panel run-panel">
      <div class="panel-head"><div><h2>Running</h2></div></div>
      <div class="run-list">
        {#if activeRunList.length}
          {#each activeRunList as run, index (run.id)}
            <div class="run-row-group">
              <button
                class="run-row"
                bind:this={activeRunButtons[index]}
                onkeydown={(event) => onRunKeydown(event, run)}
                onclick={() => onReviewRun(run.id)}
                title="Open mission"
              >
                <span class:running={run.status === "running"} class="status-dot"></span>
                <span class="run-copy"><strong>{run.repo_root.split(/[\\/]/).pop()}</strong><small>{run.id} · {run.permission_profile ?? "orbit"} · {run.isolation_mode ?? "none"}</small></span>
                <span class="run-state">{run.status}</span>
                <span class="run-cost">${(run.estimated_cost_usd ?? 0).toFixed(2)}</span>
              </button>
              <button
                class="run-stop"
                aria-label={`Review stop for run ${run.id}`}
                title="Review stop (Ctrl/Cmd+Shift+Backspace)"
                onclick={() => requestStop(run)}
              >
                <IconPlayerStop size={15} />
                <span>Stop</span>
              </button>
            </div>
          {/each}
        {:else}
          <div class="empty">
            <IconClockHour4 size={24} />
            <strong>Nothing running. Start a mission.</strong>
            <span>New missions appear here while agents are isolated.</span>
          </div>
        {/if}
      </div>
    </article>

    <article class="panel race-panel">
      <div class="panel-head">
        <div><h2>Path locks</h2></div>
        <span>{contendedAgents.length}</span>
      </div>
      {#if raceWaves.length}
        <p class="race-waves">Waves {raceWaves.map((wave) => wave + 1).join(" · ")}</p>
      {/if}
      {#if contendedAgents.length}
        {#each contendedAgents as agent}
          <div class="race-row">
            <IconLock size={15} />
            <span class="run-copy"><strong>{agent.id}</strong><small>{agent.status} · wave {agent.wave} · {agent.task_id}</small></span>
          </div>
        {/each}
      {:else}
        <div class="empty compact">
          <IconLock size={18} />
          <strong>No path locks</strong>
          <span>Claims and blocked tasks appear here when overlapping paths wait.</span>
        </div>
      {/if}
    </article>
  {/if}
</section>

<dialog
  bind:this={stopDialog}
  class="stop-run-dialog"
  aria-labelledby="stop-run-title"
  onclose={() => {
    if (!stopping && !stopDialog?.open) {
      stopTarget = null;
      stopError = "";
    }
  }}
  oncancel={(event) => {
    event.preventDefault();
    closeStopDialog();
  }}
  onclick={(event) => {
    if (event.target === stopDialog) closeStopDialog();
  }}
>
  {#if stopTarget}
    <section>
      <div class="stop-run-heading">
        <span><IconPlayerStop size={18} /></span>
        <div>
          <small>Process control</small>
          <h2 id="stop-run-title">Stop active run?</h2>
        </div>
      </div>
      <p>This will terminate the agent processes for this exact run.</p>
      <dl>
        <div><dt>Workspace</dt><dd>{stopTarget.workspace}</dd></div>
        <div><dt>Run</dt><dd class="mono">{stopTarget.run.id}</dd></div>
        <div><dt>Execution domain</dt><dd class="mono">{stopTarget.domainId}</dd></div>
      </dl>
      <div class="stop-run-consequence">
        <IconShieldLock size={18} />
        <p><strong>Sandbox preserved</strong><small>No changes are flushed and the sandbox is not deleted.</small></p>
      </div>
      {#if stopError}<div class="stop-run-error" role="alert">{stopError}</div>{/if}
      <footer>
        <button bind:this={stopCancel} class="quiet" disabled={stopping} onclick={closeStopDialog}>Keep running</button>
        <button class="stop-confirm" disabled={stopping} onclick={confirmStop}>
          <IconPlayerStop size={15} /> {stopping ? "Stopping…" : "Stop run"}
        </button>
      </footer>
    </section>
  {/if}
</dialog>

<style>
  .decision-card {
    display: block;
    width: 100%;
    text-align: left;
    border: 0;
    background: transparent;
    color: inherit;
    cursor: pointer;
    font: inherit;
  }
  .run-panel {
    margin-top: 12px;
  }
</style>
