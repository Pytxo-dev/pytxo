<script lang="ts">
  import type { AgentDto, FleetRunDto, RunDto } from "../../lib/types";
  import { ScrollArea } from "$lib/components/ui/scroll-area";
  import { Badge } from "$lib/components/ui/badge";

  let {
    runs,
    agents,
    fleetRuns = [],
    selectedRunId = $bindable(null as string | null),
    selectedAgentId = $bindable(null as string | null),
    onSelectRun,
    onSelectAgent,
    networkIsolationBadge = null as string | null,
  }: {
    runs: RunDto[];
    agents: AgentDto[];
    fleetRuns?: FleetRunDto[];
    selectedRunId?: string | null;
    selectedAgentId?: string | null;
    onSelectRun: (runId: string) => void;
    onSelectAgent: (agentId: string) => void;
    networkIsolationBadge?: string | null;
  } = $props();

  const selectedRun = $derived(runs.find((r) => r.id === selectedRunId) ?? null);

  function statusVariant(status: string): "default" | "secondary" | "muted" | "outline" {
    const s = status.toLowerCase();
    if (s === "running" || s === "active" || s === "dispatching" || s === "pending") {
      return "default";
    }
    if (s === "failed" || s === "error" || s === "cancelled") return "secondary";
    if (s === "completed" || s === "done" || s === "success") return "muted";
    return "outline";
  }
</script>

<aside class="activity-sidebar">
  <ScrollArea class="activity-sidebar__scroll">
    <div class="activity-sidebar__inner">
      <section>
        <h2 class="section-title">Runs</h2>
        {#if selectedRun}
          <div class="meta">
            <Badge variant="muted">{selectedRun.permission_profile ?? "orbit"}</Badge>
            {#if networkIsolationBadge || selectedRun.permission_profile === "deep_space"}
              <Badge variant="outline">net: {networkIsolationBadge ?? "pending"}</Badge>
            {/if}
          </div>
        {/if}
        <ul class="nav-list">
          {#each runs as run}
            <li>
              <button
                type="button"
                class="nav-item"
                class:nav-item--active={run.id === selectedRunId}
                aria-current={run.id === selectedRunId ? "true" : undefined}
                onclick={() => onSelectRun(run.id)}
              >
                <span class="nav-item__row">
                  <span class="tabular-nums">{run.id.slice(0, 8)}</span>
                  <Badge variant={statusVariant(run.status)}>{run.status}</Badge>
                </span>
                {#if run.estimated_cost_usd != null}
                  <span class="nav-item__cost tabular-nums">${run.estimated_cost_usd.toFixed(4)}</span>
                {/if}
              </button>
            </li>
          {:else}
            <li class="empty">No runs yet — use Run agents in the command bar.</li>
          {/each}
        </ul>
      </section>

      <section>
        <h2 class="section-title">Agents</h2>
        <ul class="nav-list">
          {#each agents as agent}
            <li>
              <button
                type="button"
                class="nav-item"
                class:nav-item--active={agent.id === selectedAgentId}
                aria-current={agent.id === selectedAgentId ? "true" : undefined}
                onclick={() => onSelectAgent(agent.id)}
              >
                <span class="nav-item__row">
                  <span>
                    <span class="tabular-nums">w{agent.wave}</span>
                    {agent.task_id}
                  </span>
                  <Badge variant={statusVariant(agent.status)}>{agent.status}</Badge>
                </span>
              </button>
            </li>
          {:else}
            <li class="empty">Select a run to see agents.</li>
          {/each}
        </ul>
      </section>

      {#if fleetRuns.length > 0}
        <section>
          <h2 class="section-title">Fleet</h2>
          <ul class="nav-list fleet">
            {#each fleetRuns as fr}
              <li class="fleet-row">
                <span class="tabular-nums">{fr.id.slice(0, 8)}…</span>
                <Badge variant={statusVariant(fr.status)}>{fr.status}</Badge>
              </li>
            {/each}
          </ul>
        </section>
      {/if}
    </div>
  </ScrollArea>
</aside>

<style>
  .activity-sidebar {
    width: 240px;
    flex-shrink: 0;
    border-right: 1px solid var(--sidebar-border);
    background: var(--sidebar);
    min-height: 0;
  }
  :global(.activity-sidebar__scroll) {
    height: 100%;
  }
  .activity-sidebar__inner {
    padding: 0.75rem;
  }
  .section-title {
    font-size: var(--text-xs, 0.75rem);
    text-transform: uppercase;
    letter-spacing: 0.06em;
    color: var(--muted-foreground);
    margin: 0 0 0.5rem;
    font-weight: 500;
  }
  .meta {
    display: flex;
    flex-wrap: wrap;
    gap: 0.3rem;
    margin-bottom: 0.45rem;
  }
  .nav-list {
    list-style: none;
    padding: 0;
    margin: 0 0 1rem;
  }
  .nav-item {
    display: flex;
    flex-direction: column;
    align-items: flex-start;
    gap: 0.2rem;
    width: 100%;
    text-align: left;
    padding: 0.45rem 0.55rem;
    margin-bottom: 0.2rem;
    border: none;
    border-radius: calc(var(--radius) - 2px);
    background: transparent;
    color: inherit;
    font-size: var(--text-sm, 0.875rem);
    cursor: pointer;
    box-shadow: none;
    min-height: unset;
  }
  .nav-item:hover {
    background: var(--sidebar-accent);
  }
  .nav-item--active {
    background: color-mix(in oklab, var(--primary) 10%, var(--sidebar-accent));
    box-shadow: inset 2px 0 0 var(--primary);
  }
  .nav-item__row {
    display: flex;
    width: 100%;
    align-items: center;
    justify-content: space-between;
    gap: 0.35rem;
  }
  .nav-item__cost {
    font-size: var(--text-xs, 0.75rem);
    color: var(--muted-foreground);
  }
  .empty {
    font-size: var(--text-xs, 0.75rem);
    color: var(--muted-foreground);
    padding: 0.35rem 0;
    list-style: none;
  }
  .fleet-row {
    display: flex;
    justify-content: space-between;
    align-items: center;
    font-size: var(--text-xs, 0.75rem);
    padding: 0.35rem 0;
    gap: 0.35rem;
  }
</style>
