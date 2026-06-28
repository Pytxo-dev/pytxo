<script lang="ts">
  import { ipc } from "../../lib/ipc";
  import type { AgentDto, DomainDto, FleetRunDto, RunDto } from "../../lib/types";

  let {
    workspaceLabel = "",
    workspacePath = "",
    domains,
    runs,
    agents,
    fleetRuns = [],
    selectedDomainId = $bindable(null as string | null),
    selectedRunId = $bindable(null as string | null),
    selectedAgentId = $bindable(null as string | null),
    onSelectRun,
    onSelectAgent,
    onChangeWorkspace,
    networkIsolationBadge = null as string | null,
  }: {
    workspaceLabel?: string;
    workspacePath?: string;
    domains: DomainDto[];
    runs: RunDto[];
    agents: AgentDto[];
    fleetRuns?: FleetRunDto[];
    selectedDomainId?: string | null;
    selectedRunId?: string | null;
    selectedAgentId?: string | null;
    onSelectRun: (runId: string) => void;
    onSelectAgent: (agentId: string) => void;
    onChangeWorkspace: (path: string) => void;
    networkIsolationBadge?: string | null;
  } = $props();

  const selectedRun = $derived(runs.find((r) => r.id === selectedRunId) ?? null);

  async function pickWorkspace() {
    const path = await ipc.pickWorkspaceFolder();
    if (path) {
      onChangeWorkspace(path);
    }
  }
</script>

<aside class="nav glass-panel">
  <div class="workspace">
    <span class="workspace__label">Workspace</span>
    <span class="workspace__path" title={workspacePath || workspaceLabel}>
      {workspaceLabel || "No folder selected"}
    </span>
    <button type="button" class="workspace__change" onclick={pickWorkspace}>Change</button>
  </div>

  <h2 class="panel-title">Runs</h2>
  {#if selectedRun}
    <div class="run-meta">
      <span class="pill tabular-nums">{selectedRun.permission_profile ?? "orbit"}</span>
      {#if networkIsolationBadge || selectedRun.permission_profile === "deep_space"}
        <span class="pill pill--net tabular-nums">net: {networkIsolationBadge ?? "pending"}</span>
      {/if}
    </div>
  {/if}
  <ul class="list-nav">
    {#each runs as run}
      <li>
        <button
          type="button"
          class:selected={run.id === selectedRunId}
          onclick={() => onSelectRun(run.id)}
        >
          <span class="tabular-nums">{run.id.slice(0, 8)}</span>
          <span class="status">{run.status}</span>
          {#if run.estimated_cost_usd != null}
            <span class="cost tabular-nums">${run.estimated_cost_usd.toFixed(4)}</span>
          {/if}
        </button>
      </li>
    {:else}
      <li class="empty">No runs yet — dispatch from the command bar.</li>
    {/each}
  </ul>

  <h2 class="panel-title">Agents</h2>
  <ul class="list-nav">
    {#each agents as agent}
      <li>
        <button
          type="button"
          class:selected={agent.id === selectedAgentId}
          onclick={() => onSelectAgent(agent.id)}
        >
          <span class="tabular-nums">w{agent.wave}</span> {agent.task_id}
          <span class="status">{agent.status}</span>
        </button>
      </li>
    {:else}
      <li class="empty">Select a run to see agents.</li>
    {/each}
  </ul>

  {#if fleetRuns.length > 0}
    <h2 class="panel-title">Fleet</h2>
    <ul class="list-nav fleet">
      {#each fleetRuns as fr}
        <li>
          <span class="tabular-nums">{fr.id.slice(0, 8)}…</span>
          <span class="status">{fr.status}</span>
        </li>
      {/each}
    </ul>
  {/if}
</aside>

<style>
  .nav {
    width: 240px;
    flex-shrink: 0;
    margin: 0.5rem 0 0.5rem 0.5rem;
    padding: 0.85rem;
    border-radius: 16px;
    overflow: auto;
    min-height: 0;
    box-shadow: 0 4px 24px rgba(0, 0, 0, 0.2);
  }
  .workspace {
    margin-bottom: 1rem;
    padding: 0.65rem 0.75rem;
    border-radius: 12px;
    background: color-mix(in oklab, var(--card) 75%, transparent);
    box-shadow: inset 0 0 0 1px color-mix(in oklab, var(--foreground) 6%, transparent);
  }
  .workspace__label {
    display: block;
    font-size: 0.65rem;
    text-transform: uppercase;
    letter-spacing: 0.06em;
    color: var(--muted-foreground);
    margin-bottom: 0.25rem;
  }
  .workspace__path {
    display: block;
    font-size: 0.78rem;
    font-family: ui-monospace, monospace;
    word-break: break-all;
    margin-bottom: 0.5rem;
  }
  .workspace__change {
    font-size: 0.75rem;
    min-height: 36px;
    padding: 0.25rem 0.5rem;
    border-radius: 8px;
  }
  .run-meta {
    display: flex;
    flex-wrap: wrap;
    gap: 0.35rem;
    margin-bottom: 0.45rem;
  }
  .pill {
    font-size: 0.65rem;
    padding: 0.15rem 0.4rem;
    border-radius: 6px;
    background: color-mix(in oklab, var(--brand-violet) 12%, transparent);
    color: var(--brand-violet);
  }
  .pill--net {
    color: var(--brand-teal);
    background: color-mix(in oklab, var(--brand-teal) 12%, transparent);
  }
  .list-nav button {
    display: flex;
    flex-direction: column;
    align-items: flex-start;
    gap: 0.1rem;
    min-height: 40px;
    border-radius: 10px;
    transition: transform 0.15s cubic-bezier(0.2, 0, 0, 1);
  }
  .list-nav button:active {
    transform: scale(0.96);
  }
  .status {
    font-size: 0.72rem;
    opacity: 0.75;
  }
  .cost {
    font-size: 0.72rem;
    color: var(--brand-gold);
  }
  .empty {
    font-size: 0.78rem;
    color: var(--muted-foreground);
    padding: 0.35rem 0;
    list-style: none;
  }
  .fleet li {
    font-size: 0.78rem;
    padding: 0.35rem 0;
    display: flex;
    justify-content: space-between;
  }
</style>
