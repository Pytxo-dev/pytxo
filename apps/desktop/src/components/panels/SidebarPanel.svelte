<script lang="ts">
  import TopologyPanel from "../topology/TopologyPanel.svelte";
  import ProjectPathPanel from "./ProjectPathPanel.svelte";
  import FleetPanel from "./FleetPanel.svelte";
  import UsagePanel from "./UsagePanel.svelte";
  import type {
    AgentArbitrageDto,
    AgentDto,
    CatalogEntry,
    CatalogEntryStatus,
    DomainDto,
    FleetRunDto,
    HitlDto,
    ProjectDto,
    ProjectRootDto,
    RunDto,
    StructuralGraphDto,
  } from "../../lib/types";

  let {
    projects,
    catalogForView,
    domainStatus = [],
    domains,
    runs,
    agents,
    hitl,
    arbitrage,
    projectRoots = [],
    fleetRuns = [],
    structural = null as StructuralGraphDto | null,
    selectedProjectId = $bindable(null as string | null),
    selectedDomainId = $bindable(null as string | null),
    selectedRunId = $bindable(null as string | null),
    selectedAgentId = $bindable(null as string | null),
    rootFilter = $bindable(null as string | null),
    filteredAgents,
    onSelectCatalogDomain,
    onSelectDomain,
    onSelectRun,
    onSelectAgent,
    onRespondHitl,
    onProjectRootsChange,
    networkIsolationBadge = null as string | null,
    usageTier = "core",
    usageMaxAgents = 3,
    walletMicrocredits = null as number | null,
    cloudEnabled = false,
    permissionCeiling = null as string | null,
    subscriptionPortalUrl = null as string | null,
  }: {
    projects: ProjectDto[];
    catalogForView: CatalogEntry[];
    domainStatus?: CatalogEntryStatus[];
    domains: DomainDto[];
    runs: RunDto[];
    agents: AgentDto[];
    hitl: HitlDto[];
    arbitrage: AgentArbitrageDto[];
    projectRoots?: ProjectRootDto[];
    fleetRuns?: FleetRunDto[];
    structural?: StructuralGraphDto | null;
    selectedProjectId?: string | null;
    selectedDomainId?: string | null;
    selectedRunId?: string | null;
    selectedAgentId?: string | null;
    rootFilter?: string | null;
    filteredAgents: AgentDto[];
    onSelectCatalogDomain: (entry: CatalogEntry) => void;
    onSelectDomain: (domainId: string) => void;
    onSelectRun: (runId: string) => void;
    onSelectAgent: (agentId: string) => void;
    onRespondHitl: (id: string, approve: boolean) => void;
    onProjectRootsChange?: (roots: ProjectRootDto[]) => void;
    networkIsolationBadge?: string | null;
    usageTier?: string;
    usageMaxAgents?: number;
    walletMicrocredits?: number | null;
    cloudEnabled?: boolean;
    permissionCeiling?: string | null;
    subscriptionPortalUrl?: string | null;
  } = $props();

  const uniqueRoots = $derived(
    [...new Set(agents.map((a) => a.root_id).filter(Boolean))] as string[],
  );

  const selectedRun = $derived(runs.find((r) => r.id === selectedRunId) ?? null);
</script>

<aside class="sidebar glass-panel">
  {#if projects.length > 0}
    <h2 class="panel-title">Projects</h2>
    <ul class="list-nav">
      <li>
        <button
          class:selected={selectedProjectId === null}
          onclick={() => {
            selectedProjectId = null;
            rootFilter = null;
          }}
        >
          All
        </button>
      </li>
      {#each projects as p}
        <li>
          <button
            class:selected={selectedProjectId === p.id}
            onclick={() => {
              selectedProjectId = p.id;
              rootFilter = null;
            }}
            title={p.manifest_path}
          >
            {p.id}
          </button>
        </li>
      {/each}
    </ul>
  {/if}

  {#if catalogForView.length > 0}
    <h2 class="panel-title">Domains ({catalogForView.length})</h2>
    <ul class="list-nav">
      {#each catalogForView as entry}
        {@const status = domainStatus.find((d) => d.domain_id === entry.domain_id)}
        <li>
          <button
            class:selected={entry.domain_id === selectedDomainId}
            onclick={() => onSelectCatalogDomain(entry)}
            title={entry.repo_root}
          >
            {entry.repo_root.split(/[/\\]/).pop() ?? entry.domain_id.slice(0, 12)}
            {#if status && status.active_runs > 0}
              <span class="status-badge status-badge--running">{status.active_runs} active</span>
            {:else if status?.hitl_pending}
              <span class="status-badge status-badge--hitl">HITL {status.hitl_pending}</span>
            {:else if status?.latest_run_status}
              <span class="status-badge">{status.latest_run_status}</span>
            {/if}
            {#if entry.project_id}
              <span class="cost-badge">{entry.project_id}</span>
            {/if}
          </button>
        </li>
      {/each}
    </ul>
  {/if}

  <h2 class="panel-title">Active domains</h2>
  <ul class="list-nav">
    {#each domains as d}
      <li>
        <button
          class:selected={d.domain_id === selectedDomainId}
          onclick={() => onSelectDomain(d.domain_id)}
        >
          {d.repo_root.split(/[/\\]/).pop() ?? d.domain_id.slice(0, 12)}
        </button>
      </li>
    {/each}
  </ul>

  <h2 class="panel-title">Runs</h2>
  {#if selectedRun}
    <div class="run-meta">
      <span class="cost-badge">{selectedRun.permission_profile ?? "orbit"}</span>
      <span class="cost-badge">{selectedRun.isolation_backend ?? selectedRun.isolation_mode}</span>
      {#if networkIsolationBadge || selectedRun.permission_profile === "deep_space"}
        <span class="cost-badge cost-badge--net" title="DeepSpace network isolation">
          net: {networkIsolationBadge ?? "pending"}
        </span>
      {/if}
    </div>
  {/if}
  <ul class="list-nav">
    {#each runs as run}
      <li>
        <button
          class:selected={run.id === selectedRunId}
          onclick={() => onSelectRun(run.id)}
        >
          {run.id.slice(0, 8)}… — {run.status}
          {#if run.estimated_cost_usd != null}
            <span class="cost-badge">${run.estimated_cost_usd.toFixed(4)}</span>
          {/if}
        </button>
      </li>
    {/each}
  </ul>

  <h2 class="panel-title">Waves</h2>
  {#if uniqueRoots.length > 0}
    <div class="root-filter">
      <button class:selected={rootFilter === null} onclick={() => (rootFilter = null)}>
        All roots
      </button>
      {#each uniqueRoots as root}
        <button
          class:selected={rootFilter === root}
          onclick={() => (rootFilter = root)}
        >
          {root}
        </button>
      {/each}
    </div>
  {/if}
  <ul class="list-nav agents">
    {#each filteredAgents as agent}
      <li>
        <button
          class:selected={agent.id === selectedAgentId}
          onclick={() => onSelectAgent(agent.id)}
        >
          w{agent.wave} {agent.task_id}
          {#if agent.root_id}
            <span class="cost-badge">@{agent.root_id}</span>
          {/if}
          — {agent.status}
        </button>
      </li>
    {/each}
  </ul>

  {#if hitl.length > 0}
    <h2 class="panel-title panel-title--gold">Approvals ({hitl.length})</h2>
    <ul class="hitl">
      {#each hitl as req}
        <li class="hitl-item chroma-edge-top">
          <div class="hitl-action">{req.action}</div>
          <div class="hitl-agent">{req.agent_key}</div>
          {#if req.domain_id}
            <div class="hitl-domain">{req.domain_id.slice(0, 12)}…</div>
          {/if}
          <div class="hitl-reason">{req.reason}</div>
          <div class="hitl-buttons">
            <button class="gold" onclick={() => onRespondHitl(req.id, true)}>Approve</button>
            <button onclick={() => onRespondHitl(req.id, false)}>Deny</button>
          </div>
        </li>
      {/each}
    </ul>
  {/if}

  <ProjectPathPanel
    projectId={selectedProjectId}
    roots={projectRoots}
    onRootsChange={onProjectRootsChange}
  />
  <FleetPanel runs={fleetRuns} />
  <UsagePanel
    tier={usageTier}
    maxAgents={usageMaxAgents}
    {walletMicrocredits}
    {cloudEnabled}
    {permissionCeiling}
    {subscriptionPortalUrl}
  />
  <TopologyPanel agents={filteredAgents} {arbitrage} structural={structural} />
</aside>

<style>
  .sidebar {
    margin: 0 0 0.5rem 0.5rem;
    padding: 0.75rem;
    overflow: auto;
    min-height: 0;
  }
  .root-filter {
    display: flex;
    flex-wrap: wrap;
    gap: 0.25rem;
    margin-bottom: 0.5rem;
  }
  .root-filter button {
    font-size: 0.75rem;
    padding: 0.15rem 0.4rem;
  }
  ul.hitl {
    list-style: none;
    padding: 0;
    margin: 0 0 1rem;
  }
  .hitl-item {
    border: 1px solid color-mix(in oklab, var(--brand-gold) 50%, transparent);
    border-radius: 6px;
    padding: 0.4rem 0.5rem;
    margin-bottom: 0.4rem;
    background: color-mix(in oklab, var(--brand-gold) 8%, transparent);
  }
  .hitl-action {
    font-weight: 600;
    color: var(--brand-gold);
    font-size: 0.85rem;
  }
  .hitl-agent {
    font-size: 0.7rem;
    opacity: 0.75;
    font-family: ui-monospace, monospace;
    word-break: break-all;
  }
  .hitl-reason {
    font-size: 0.75rem;
    opacity: 0.8;
    margin: 0.2rem 0 0.4rem;
    word-break: break-word;
  }
  .hitl-buttons {
    display: flex;
    gap: 0.4rem;
  }
  .hitl-buttons button {
    flex: 1;
  }
  .status-badge {
    font-size: 0.65rem;
    margin-left: 0.25rem;
    padding: 0.05rem 0.3rem;
    border-radius: 4px;
    opacity: 0.85;
    background: color-mix(in oklab, var(--foreground) 12%, transparent);
  }
  .status-badge--running {
    color: var(--brand-teal);
    background: color-mix(in oklab, var(--brand-teal) 18%, transparent);
  }
  .status-badge--hitl {
    color: var(--brand-gold);
    background: color-mix(in oklab, var(--brand-gold) 18%, transparent);
  }
  .run-meta {
    display: flex;
    gap: 0.35rem;
    margin: 0 0 0.45rem;
    flex-wrap: wrap;
  }
  .cost-badge--net {
    color: var(--brand-teal);
    background: color-mix(in oklab, var(--brand-teal) 12%, transparent);
  }
</style>
