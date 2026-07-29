<script lang="ts">
  import {
    IconAlertTriangle,
    IconArrowUpRight,
    IconCircleCheck,
    IconClockHour4,
    IconLayersIntersect,
    IconLock,
    IconPlayerStop,
    IconShieldLock,
  } from "@tabler/icons-svelte";
  import type { DesktopSnapshot } from "../../lib/desktop-backend";
  import type { RunDto } from "../../lib/types";

  let {
    snapshot,
    live = false,
    lastPollAt = null,
    activeDomainLabel = null,
    onRoute,
    onReviewRun,
    onStopRun,
  }: {
    snapshot: DesktopSnapshot;
    live?: boolean;
    lastPollAt?: number | null;
    activeDomainLabel?: string | null;
    onRoute: (route: "approvals" | "runs" | "flow" | "workspaces") => void;
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

  const activeRuns = $derived(
    snapshot.runs.filter((run) =>
      ["running", "pending", "dispatching", "active"].includes(run.status.toLowerCase()),
    ).length,
  );
  const needsYou = $derived(snapshot.approvals.length);
  const spendUsd = $derived(
    snapshot.runs.reduce((sum, run) => sum + (run.estimated_cost_usd ?? 0), 0),
  );
  const healthyAgents = $derived(snapshot.agents.filter((agent) => agent.status !== "failed").length);
  const failedAgents = $derived(snapshot.agents.length - healthyAgents);
  const activeRunList = $derived(
    snapshot.runs
      .filter((run) => ["running", "pending", "dispatching", "active"].includes(run.status.toLowerCase()))
      .slice(0, 12),
  );
  const isolatedRuns = $derived(snapshot.runs.filter((run) => run.isolation_mode && run.isolation_mode !== "none").length);
  const isolationPct = $derived(snapshot.runs.length ? Math.round((isolatedRuns / snapshot.runs.length) * 100) : null);

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
  const raceVisible = $derived(contendedAgents.length > 0 || snapshot.agents.length > 1);

  type TimelineEntry = { key: string; ts: number; kind: "run" | "approval"; label: string; detail: string; tone: "teal" | "gold" };

  const timeline: TimelineEntry[] = $derived(
    [
      ...snapshot.runs.map((run) => ({
        key: `run-${run.id}`,
        ts: Date.parse(run.started_at) || 0,
        kind: "run" as const,
        label: `${run.repo_root.split(/[\\/]/).pop()} run ${run.status}`,
        detail: `${run.id} · ${run.permission_profile ?? "orbit"}`,
        tone: "teal" as const,
      })),
      ...snapshot.approvals.map((approval) => ({
        key: `approval-${approval.id}`,
        ts: Number(approval.created_at_ms) || 0,
        kind: "approval" as const,
        label: approval.action,
        detail: `${approval.agent_key} · awaiting decision`,
        tone: "gold" as const,
      })),
    ]
      .filter((entry) => entry.ts > 0)
      .sort((a, b) => b.ts - a.ts)
      .slice(0, 6),
  );

  function relativeTime(ts: number): string {
    const diffMs = Date.now() - ts;
    const minutes = Math.round(diffMs / 60000);
    if (minutes < 1) return "Now";
    if (minutes < 60) return `${minutes}m`;
    const hours = Math.round(minutes / 60);
    if (hours < 24) return `${hours}h`;
    return `${Math.round(hours / 24)}d`;
  }

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
      <p>Running work, what needs you, sandboxed share, and spend.</p>
    </div>
    <button class="primary" onclick={() => onRoute("flow")}>New Flow <IconArrowUpRight size={16} /></button>
  </header>

  <div class="ops-shortcuts" aria-label="Ops keyboard shortcuts">
    <span><kbd>Ctrl/⌘ Shift O</kbd> Focus active work</span>
    <span><kbd>Ctrl/⌘ Shift ⌫</kbd> Review stop</span>
  </div>

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
    <div class="ops-truth" aria-label="Ops truth strip">
      <article>
        <button type="button" onclick={() => onRoute("runs")}>
          <span>Running</span>
          <strong>{activeRuns}</strong>
          <small>{snapshot.domains.length} workspace{snapshot.domains.length === 1 ? "" : "s"}</small>
        </button>
      </article>
      <article class:needs-you={needsYou > 0}>
        <button type="button" onclick={() => onRoute("approvals")}>
          <span>Needs you</span>
          <strong>{needsYou}</strong>
          <small>{needsYou ? "Open approvals" : "Inbox clear"}</small>
        </button>
      </article>
      <article>
        <span>Sandboxed</span>
        <strong>{isolationPct === null ? "N/A" : `${isolationPct}%`}</strong>
        <small>{isolationPct === null ? "No runs yet" : "Isolated runs"}</small>
      </article>
      <article>
        <span>Cost</span>
        <strong>${spendUsd.toFixed(2)}</strong>
        <small>Estimated across listed runs</small>
      </article>
    </div>

    <div class="metrics metrics--two" aria-label="Ops detail">
      <article>
        <span>Agent health</span><strong>{healthyAgents}/{snapshot.agents.length}</strong>
        {#if snapshot.agents.length === 0}
          <small>No agents yet</small>
        {:else if failedAgents > 0}
          <small class="risk-text"><IconAlertTriangle size={14} /> {failedAgents} failed</small>
        {:else}
          <small class="good"><IconCircleCheck size={14} /> All responsive</small>
        {/if}
      </article>
      <article>
        <span>Path locks</span><strong>{contendedAgents.length}</strong>
        <small>{contendedAgents.length ? "Path contention" : raceVisible ? "Agents active" : "No locks"}</small>
      </article>
    </div>

    <div class="workspace-grid">
      <article class="panel run-panel">
        <div class="panel-head"><div><h2>Active runs</h2></div><button class="quiet" onclick={() => onRoute("runs")}>View all</button></div>
        <div class="run-list">
          {#if activeRunList.length}
            {#each activeRunList as run, index (run.id)}
              <div class="run-row-group">
                <button
                  class="run-row"
                  bind:this={activeRunButtons[index]}
                  onkeydown={(event) => onRunKeydown(event, run)}
                  onclick={() => onReviewRun(run.id)}
                  title="Open Run Review"
                >
                  <span class:running={run.status === "running"} class="status-dot"></span>
                  <span class="run-copy"><strong>{run.repo_root.split(/[\\/]/).pop()}</strong><small>{run.id} · {run.permission_profile ?? "orbit"} · {run.isolation_mode ?? "none"}</small></span>
                  <span class="run-state">{run.status}</span>
                  <span class="run-cost">${(run.estimated_cost_usd ?? 0).toFixed(2)}</span>
                </button>
                <button
                  class="run-stop"
                  aria-label={`Review stop for run ${run.id}`}
                  title="Review stop"
                  onclick={() => requestStop(run)}
                >
                  <IconPlayerStop size={15} />
                  <span>Stop</span>
                </button>
              </div>
            {/each}
          {:else}
            <div class="empty"><IconClockHour4 size={24} /><strong>No active runs</strong><span>Start a Flow to see it here.</span></div>
          {/if}
        </div>
      </article>

      <article class="panel approval-panel">
        <div class="panel-head"><div><h2>Needs attention</h2></div><button class="quiet" onclick={() => onRoute("approvals")}>Open inbox</button></div>
        {#if snapshot.approvals.length}
          {#each snapshot.approvals as approval}
            <button class="decision-card" onclick={() => onRoute("approvals")}>
              <span class="risk">Review</span>
              <h3>{approval.action}</h3>
              <p>{approval.reason}</p>
              <small>{approval.agent_key} · {approval.domain_id}</small>
            </button>
          {/each}
        {:else}
          <div class="empty"><IconCircleCheck size={26} /><strong>Nothing needs review</strong><span>Approvals appear here when agents wait on you.</span></div>
        {/if}
      </article>
    </div>

    {#if raceVisible}
      <article class="panel race-panel">
        <div class="panel-head">
          <div><h2>Path locks</h2></div>
          <span>{contendedAgents.length || snapshot.agents.length}</span>
        </div>
        {#if contendedAgents.length}
          {#each contendedAgents as agent}
            <div class="race-row">
              <IconLock size={15} />
              <span class="run-copy"><strong>{agent.id}</strong><small>{agent.status} · wave {agent.wave} · {agent.task_id}</small></span>
            </div>
          {/each}
        {:else}
          {#each snapshot.agents.slice(0, 6) as agent}
            <div class="race-row">
              <IconLock size={15} />
              <span class="run-copy"><strong>{agent.id}</strong><small>{agent.status} · wave {agent.wave}</small></span>
            </div>
          {/each}
          <p class="fleet-hint">Locks surface when agents contend for the same paths.</p>
        {/if}
      </article>
    {/if}

    {#if snapshot.fleets.length}
      <article class="panel fleet-panel">
        <div class="panel-head"><div><h2>Fleet runs</h2></div><span>{snapshot.fleets.length}</span></div>
        <div class="fleet-list">
          {#each snapshot.fleets as fleet}
            <div class="fleet-row">
              <IconLayersIntersect size={16} />
              <span class="run-copy"><strong>{fleet.fleet_id}</strong><small class="mono">{fleet.id}</small></span>
              <span class="run-state">{fleet.status}</span>
            </div>
          {/each}
        </div>
        <p class="fleet-hint">Full fleet control remains on the CLI: <code class="mono">pytxo fleet status</code></p>
      </article>
    {/if}

    <article class="panel activity-panel">
      <div class="panel-head">
        <div><h2>Recent activity</h2></div>
        {#if live}<span class="live" title={lastPollAt ? `Updated ${relativeTime(lastPollAt)}` : "Polling"}>Live</span>{:else if timeline.length}<span class="live muted">Paused</span>{/if}
      </div>
      {#if timeline.length}
        <div class="timeline">
          {#each timeline as entry (entry.key)}
            <div><time>{relativeTime(entry.ts)}</time><span class="event-mark {entry.tone}"></span><p>{entry.label}</p><small>{entry.detail}</small></div>
          {/each}
        </div>
      {:else}
        <div class="empty"><IconClockHour4 size={24} /><strong>No activity yet</strong><span>Runs and approvals show up here as they happen.</span></div>
      {/if}
    </article>
  {/if}
</section>

<dialog
  bind:this={stopDialog}
  class="stop-run-dialog"
  aria-labelledby="stop-run-title"
  onclose={() => {
    if (!stopping) {
      stopTarget = null;
      stopError = "";
    }
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
  .risk-text {
    color: var(--pytxo-gold, #d9a44f) !important;
  }
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
  .fleet-panel {
    margin-top: 12px;
  }
  .fleet-list {
    display: flex;
    flex-direction: column;
    gap: 6px;
  }
  .fleet-row {
    display: flex;
    align-items: center;
    gap: 10px;
    padding: 8px 4px;
    color: #8b929c;
  }
  .fleet-hint {
    margin: 10px 0 0;
    font-size: 10px;
    color: #5e6571;
  }
  .live.muted {
    opacity: 0.55;
  }
</style>
