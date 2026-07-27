<script lang="ts">
  import { onMount } from "svelte";
  import { IconArrowLeft, IconCheck, IconGitBranch, IconLoader2, IconShieldCheck } from "@tabler/icons-svelte";
  import { ipc } from "../../lib/ipc";
  import type { AgentArbitrageDto, AgentDto, RunDto, StructuralGraphDto } from "../../lib/types";

  let {
    mode,
    run,
    domainId,
    onBack,
    onRunCompleted,
  }: {
    mode: "topology-focus" | "run-review";
    run: RunDto | null;
    domainId: string | null;
    onBack: () => void;
    onRunCompleted: () => void;
  } = $props();

  let graph = $state<StructuralGraphDto>({ nodes: [], edges: [] });
  let graphLoading = $state(true);
  let graphError = $state<string | null>(null);
  let editedOnly = $state(false);

  let agents = $state<AgentDto[]>([]);
  let agentsLoading = $state(true);
  let diffText = $state("");
  let diffLoading = $state(false);
  let completing = $state(false);
  let completeError = $state<string | null>(null);

  let arbitrage = $state<AgentArbitrageDto[]>([]);
  let arbitrageLoading = $state(false);

  const visibleNodes = $derived(editedOnly ? graph.nodes.filter((n) => n.edited) : graph.nodes);
  const targetsByNode = $derived.by(() => {
    const labelById = new Map(graph.nodes.map((n) => [n.id, n.label] as const));
    const map = new Map<string, string[]>();
    for (const e of graph.edges) {
      const list = map.get(e.from) ?? [];
      list.push(labelById.get(e.to) ?? e.to);
      map.set(e.from, list);
    }
    return map;
  });
  function targetsOf(nodeId: string) {
    return targetsByNode.get(nodeId) ?? [];
  }

  const allExited = $derived(agents.length > 0 && agents.every((a) => a.exit_code === 0));
  const anyFailed = $derived(agents.some((a) => a.exit_code !== null && a.exit_code !== 0));

  const arbitrageTotals = $derived.by(() => {
    const saved = arbitrage.reduce((n, a) => n + (a.saved_tokens ?? 0), 0);
    const edited = arbitrage.reduce((n, a) => n + (a.edited_paths ?? 0), 0);
    const fallback = arbitrage.reduce((n, a) => n + (a.fallback_paths ?? 0), 0);
    return { saved, edited, fallback, agents: arbitrage.length };
  });

  async function loadTopology() {
    graphLoading = true;
    graphError = null;
    try {
      graph = run
        ? await ipc.structuralGraph(run.id, domainId)
        : domainId
          ? await ipc.workspaceStructuralGraph(domainId)
          : { nodes: [], edges: [] };
    } catch (e) {
      graphError = e instanceof Error ? e.message : String(e);
    } finally {
      graphLoading = false;
    }
  }

  async function loadArbitrage() {
    if (!run) {
      arbitrage = [];
      return;
    }
    arbitrageLoading = true;
    try {
      arbitrage = await ipc.agentArbitrage(run.id, domainId);
    } catch {
      arbitrage = [];
    } finally {
      arbitrageLoading = false;
    }
  }

  async function loadReview() {
    if (!run) {
      agentsLoading = false;
      return;
    }
    agentsLoading = true;
    try {
      agents = await ipc.listAgents(run.id, domainId);
      const withRoot = agents.find((a) => a.root_id) ?? agents[0] ?? null;
      if (withRoot) await loadDiffFor(withRoot.id);
    } finally {
      agentsLoading = false;
    }
  }

  async function loadDiffFor(agentId: string) {
    diffLoading = true;
    try {
      diffText = await ipc.gitDiff(agentId, domainId);
    } catch {
      diffText = "";
    } finally {
      diffLoading = false;
    }
  }

  async function completeRun() {
    if (!run) return;
    if (!allExited) {
      completeError = "Cannot apply: every agent must exit 0 with verifies passing.";
      return;
    }
    completing = true;
    completeError = null;
    try {
      for (const agent of agents) {
        if (agent.exit_code === 0) await ipc.commitWorkspace(run.id, agent.id, domainId);
      }
      onRunCompleted();
    } catch (e) {
      completeError = e instanceof Error ? e.message : String(e);
    } finally {
      completing = false;
    }
  }

  onMount(() => {
    if (mode === "topology-focus") {
      void loadTopology();
      void loadArbitrage();
    } else {
      void loadReview();
      void loadArbitrage();
    }
  });
</script>

<section class="screen focus-screen">
  <header class="screen-heading">
    <div>
      <button class="back" onclick={onBack}><IconArrowLeft size={15} /> Runs</button>
      <p class="eyebrow">{run ? run.id : domainId ? (domainId.split(/[\\/]/).pop() ?? domainId) : "No selection"}</p>
      <h1>{mode === "topology-focus" ? "Topology Focus" : "Run Review"}</h1>
      <p>{mode === "topology-focus" ? "Structural relationships from the workspace graph." : "Validate the result before completion and sandbox flush."}</p>
    </div>
    {#if mode === "run-review" && run}
      <button class="primary" disabled={completing || agentsLoading || !agents.length || !allExited} onclick={completeRun} title={!allExited ? "All agents must exit successfully with verifies passing before apply" : "Apply accepted isolated changes"}>
        {#if completing}<IconLoader2 size={16} class="spin" />{:else}<IconCheck size={16} />{/if} Complete run
      </button>
    {/if}
  </header>

  {#if run}
    <aside class="arbitrage-bar" aria-live="polite">
      <p class="eyebrow">Structure savings</p>
      {#if arbitrageLoading}
        <strong><IconLoader2 size={14} class="spin" /> Loading savings…</strong>
      {:else if arbitrageTotals.agents === 0}
        <strong>No savings samples yet</strong>
        <span>Scaffold savings appear after agents read structured context.</span>
      {:else}
        <strong>{arbitrageTotals.saved.toLocaleString()} tokens saved</strong>
        <span>{arbitrageTotals.agents} agent{arbitrageTotals.agents === 1 ? "" : "s"} · {arbitrageTotals.edited} edited path{arbitrageTotals.edited === 1 ? "" : "s"}{#if arbitrageTotals.fallback} · {arbitrageTotals.fallback} fallback{/if}</span>
      {/if}
    </aside>
  {/if}

  {#if mode === "topology-focus"}
    <article class="panel topology-canvas">
      <div class="topology-tools">
        <span>Structural graph{domainId ? ` · ${domainId.split(/[\\/]/).pop()}` : ""}</span>
        <button class:active={editedOnly} onclick={() => (editedOnly = !editedOnly)}>Edited only</button>
      </div>
      {#if graphLoading}
        <div class="empty"><IconLoader2 size={24} class="spin" /><strong>Loading structural graph…</strong></div>
      {:else if graphError}
        <div class="empty"><strong>Could not load structural graph</strong><span>{graphError}</span></div>
      {:else if !visibleNodes.length}
        <div class="empty">
          <IconGitBranch size={26} />
          <strong>No structural data yet</strong>
          <span>{run ? "This run has not touched any tracked files." : "Open a workspace with at least one run to see its structure."}</span>
        </div>
      {:else}
        <div class="structural-list">
          {#each visibleNodes as node (node.id)}
            {@const targets = targetsOf(node.id)}
            <div class="structural-node" class:edited={node.edited}>
              <IconGitBranch size={15} />
              <strong>{node.label}</strong>
              {#if targets.length}<small>→ {targets.join(", ")}</small>{/if}
            </div>
          {/each}
        </div>
      {/if}
    </article>
  {:else}
    <div class="review-grid">
      <article class="panel">
        <h2>{run ? "Ready to review" : "No run selected"}</h2>
        {#if run}
          <div class="check-row" class:check-row--fail={!agentsLoading && !allExited}>
            {#if agentsLoading}<IconLoader2 size={16} class="spin" />{:else}<IconCheck size={16} />{/if}
            {agentsLoading ? "Checking agents…" : `${agents.filter((a) => a.exit_code === 0).length}/${agents.length} agents exited successfully`}
          </div>
          <div class="check-row"><IconCheck size={16} /> Permission profile: {run.permission_profile ?? "unknown"}</div>
          <div class="check-row"><IconCheck size={16} /> Isolation: {run.isolation_mode} via {run.isolation_backend}</div>
          <div class="check-row" class:check-row--fail={anyFailed}><IconCheck size={16} /> Run status: {run.status}</div>
          <div class="shield-note">
            <IconShieldCheck size={19} />
            <p><strong>Sandbox {run.isolation_mode === "copy_on_write" ? "intact" : "not isolated for this run"}</strong><small>Changes stay isolated until you approve completion.</small></p>
          </div>
          {#if completeError}<p class="voice-state-message error">{completeError}</p>{/if}
        {:else}
          <div class="empty"><strong>Select a run from the Runs table first.</strong></div>
        {/if}
      </article>
      <article class="panel">
        <h2>{diffLoading ? "Loading diff…" : diffText ? "Changes recorded" : "No file changes"}</h2>
        {#if diffLoading}
          <div class="empty"><IconLoader2 size={22} class="spin" /><strong>Loading diff…</strong></div>
        {:else if diffText}
          <pre class="diff-raw">{diffText}</pre>
        {:else}
          <div class="empty">
            <strong>No file changes recorded</strong>
            <span>{run ? "This run's agents have not modified any tracked files." : "Select a run to inspect its diff."}</span>
          </div>
        {/if}
      </article>
    </div>
  {/if}
</section>

<style>
  .arbitrage-bar {
    display: flex;
    flex-wrap: wrap;
    align-items: baseline;
    gap: 8px 16px;
    margin: 0 0 16px;
    padding: 12px 16px;
    border: 1px solid color-mix(in srgb, #38d6c1 28%, transparent);
    border-radius: 10px;
    background: color-mix(in srgb, #101816 80%, transparent);
  }
  .arbitrage-bar .eyebrow {
    margin: 0;
    width: 100%;
  }
  .arbitrage-bar strong {
    display: inline-flex;
    align-items: center;
    gap: 6px;
    font-size: 14px;
    color: #9fe6d6;
  }
  .arbitrage-bar span {
    font-size: 12px;
    color: #8b929c;
  }
  :global(.desktop2 .topology-tools button.active) {
    border-color: #38d6c1;
    color: #9fe6d6;
    background: #101816;
  }
  .diff-raw {
    margin: 0;
    padding: 14px;
    max-height: 460px;
    overflow: auto;
    font: 10px/1.6 "Geist Mono", monospace;
    color: #c3c8cf;
    white-space: pre-wrap;
    word-break: break-word;
  }
  :global(.spin) {
    animation: focus-spin 0.9s linear infinite;
  }
  @keyframes focus-spin {
    to {
      transform: rotate(360deg);
    }
  }
  @media (prefers-reduced-motion: reduce) {
    :global(.spin) {
      animation: none;
    }
  }
</style>
