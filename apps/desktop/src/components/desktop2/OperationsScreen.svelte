<script lang="ts">
  import { IconAlertTriangle, IconArrowUpRight, IconCircleCheck, IconClockHour4, IconLayersIntersect, IconShieldCheck } from "@tabler/icons-svelte";
  import type { DesktopSnapshot } from "../../lib/desktop-backend";

  let {
    snapshot,
    live = false,
    lastPollAt = null,
    activeDomainLabel = null,
    onRoute,
    onReviewRun,
  }: {
    snapshot: DesktopSnapshot;
    live?: boolean;
    lastPollAt?: number | null;
    activeDomainLabel?: string | null;
    onRoute: (route: "approvals" | "runs" | "flow" | "workspaces") => void;
    onReviewRun: (runId: string) => void;
  } = $props();

  const activeRuns = $derived(snapshot.runs.filter((run) => ["running", "pending", "dispatching"].includes(run.status)).length);
  const healthyAgents = $derived(snapshot.agents.filter((agent) => agent.status !== "failed").length);
  const failedAgents = $derived(snapshot.agents.length - healthyAgents);
  const activeRunList = $derived(
    snapshot.runs
      .filter((run) => ["running", "pending", "dispatching", "active"].includes(run.status.toLowerCase()))
      .slice(0, 12),
  );
  const isolatedRuns = $derived(snapshot.runs.filter((run) => run.isolation_mode && run.isolation_mode !== "none").length);
  const isolationPct = $derived(snapshot.runs.length ? Math.round((isolatedRuns / snapshot.runs.length) * 100) : null);

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
</script>

<section class="screen operations">
  <header class="screen-heading">
    <div>
      <p class="eyebrow">Live supervision{#if activeDomainLabel} · {activeDomainLabel}{/if}</p>
      <h1>Operations</h1>
      <p>One calm view of active work, risk, and system health.</p>
    </div>
    <button class="primary" onclick={() => onRoute("flow")}>New Flow <IconArrowUpRight size={16} /></button>
  </header>

  {#if snapshot.error}
    <div class="panel chroma-edge">
      <div class="empty">
        <IconAlertTriangle size={26} />
        <strong>Hypervisor unreachable</strong>
        <span>{snapshot.error.message}</span>
      </div>
    </div>
  {:else}
    <div class="metrics" aria-label="Operations summary">
      <article><span>Active runs</span><strong>{activeRuns}</strong><small>Across {snapshot.domains.length} execution domain{snapshot.domains.length === 1 ? "" : "s"}</small></article>
      <article>
        <span>Agent health</span><strong>{healthyAgents}/{snapshot.agents.length}</strong>
        {#if snapshot.agents.length === 0}
          <small>No agents dispatched yet</small>
        {:else if failedAgents > 0}
          <small class="risk-text"><IconAlertTriangle size={14} /> {failedAgents} failed</small>
        {:else}
          <small class="good"><IconCircleCheck size={14} /> All responsive</small>
        {/if}
      </article>
      <article><span>Approvals</span><strong>{snapshot.approvals.length}</strong><small>{snapshot.approvals.length ? "Operator review required" : "Inbox clear"}</small></article>
      <article>
        <span>Isolation</span><strong>{isolationPct === null ? "—" : `${isolationPct}%`}</strong>
        {#if isolationPct === null}
          <small>No runs yet</small>
        {:else}
          <small class="good"><IconShieldCheck size={14} /> Blast Shield coverage</small>
        {/if}
      </article>
    </div>

    <div class="workspace-grid">
      <article class="panel run-panel">
        <div class="panel-head"><div><p class="eyebrow">Execution yard</p><h2>Active runs</h2></div><button class="quiet" onclick={() => onRoute("runs")}>View all</button></div>
        <div class="run-list">
          {#if activeRunList.length}
            {#each activeRunList as run (run.id)}
              <button class="run-row" onclick={() => onReviewRun(run.id)}>
                <span class:running={run.status === "running"} class="status-dot"></span>
                <span class="run-copy"><strong>{run.repo_root.split(/[\\/]/).pop()}</strong><small>{run.id} · {run.permission_profile ?? "orbit"} · {run.isolation_mode ?? "none"}</small></span>
                <span class="run-state">{run.status}</span>
                <span class="run-cost">${(run.estimated_cost_usd ?? 0).toFixed(2)}</span>
              </button>
            {/each}
          {:else}
            <div class="empty"><IconClockHour4 size={24} /><strong>No active runs</strong><span>Dispatch a Flow to see it here.</span></div>
          {/if}
        </div>
      </article>

      <article class="panel approval-panel chroma-edge">
        <div class="panel-head"><div><p class="eyebrow">Decision queue</p><h2>Needs attention</h2></div><button class="quiet" onclick={() => onRoute("approvals")}>Open inbox</button></div>
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
          <div class="empty"><IconCircleCheck size={26} /><strong>Nothing needs review</strong><span>New approval requests will appear here.</span></div>
        {/if}
      </article>
    </div>

    {#if snapshot.fleets.length}
      <article class="panel fleet-panel">
        <div class="panel-head"><div><p class="eyebrow">Cross-repo</p><h2>Fleet runs</h2></div><span>{snapshot.fleets.length}</span></div>
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
        <div><p class="eyebrow">Structured events</p><h2>Recent activity</h2></div>
        {#if live}<span class="live" title={lastPollAt ? `Updated ${relativeTime(lastPollAt)}` : "Polling"}>Live</span>{:else if timeline.length}<span class="live muted">Paused</span>{/if}
      </div>
      {#if timeline.length}
        <div class="timeline">
          {#each timeline as entry (entry.key)}
            <div><time>{relativeTime(entry.ts)}</time><span class="event-mark {entry.tone}"></span><p>{entry.label}</p><small>{entry.detail}</small></div>
          {/each}
        </div>
      {:else}
        <div class="empty"><IconClockHour4 size={24} /><strong>No activity yet</strong><span>Runs and approvals will show up here as they happen.</span></div>
      {/if}
    </article>
  {/if}
</section>

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
