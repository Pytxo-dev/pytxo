<script lang="ts">
  import { onMount } from "svelte";
  import { IconAlertTriangle, IconArrowLeft, IconBinaryTree2, IconCheck, IconFingerprint, IconGitBranch, IconLoader2, IconShieldCheck } from "@tabler/icons-svelte";
  import type { DesktopBackend } from "../../lib/desktop-backend";
  import type { AgentArbitrageDto, AgentDto, RunDto, RunReviewDto, StructuralGraphDto } from "../../lib/types";

  let {
    backend,
    mode,
    run,
    domainId,
    onBack,
    onRunCompleted,
  }: {
    backend: DesktopBackend;
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
  let review = $state<RunReviewDto | null>(null);
  let reviewError = $state<string | null>(null);
  let selectedAgentId = $state<string | null>(null);
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
  const reviewTasks = $derived(review?.plan.waves.flat() ?? []);
  const canApply = $derived(allExited && review?.apply_status === "ready");
  const runReceipt = $derived(review?.enforcement.run ?? null);
  const applyLabel = $derived(
    review?.apply_status === "applied"
      ? "Applied"
      : review?.apply_status === "applying"
        ? "Applying…"
        : "Apply run",
  );

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
        ? await backend.structuralGraph(run.id, domainId)
        : domainId
          ? await backend.workspaceStructuralGraph(domainId)
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
      arbitrage = await backend.agentArbitrage(run.id, domainId);
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
    reviewError = null;
    try {
      const [nextAgents, nextReview] = await Promise.all([
        backend.listAgents(run.id, domainId),
        backend.runReview(run.id, domainId),
      ]);
      agents = nextAgents;
      review = nextReview;
      const withRoot = agents.find((a) => a.root_id) ?? agents[0] ?? null;
      selectedAgentId = withRoot?.id ?? null;
      if (withRoot) await loadDiffFor(withRoot.id);
    } catch (error) {
      reviewError = error instanceof Error ? error.message : String(error);
    } finally {
      agentsLoading = false;
    }
  }

  async function loadDiffFor(agentId: string) {
    selectedAgentId = agentId;
    // Run Review now consumes only the immutable prepared manifest. The
    // legacy Focus surface never shells out to Git in an agent workspace.
    diffLoading = false;
    diffText = "";
  }

  async function completeRun() {
    if (!run) return;
    if (!canApply) {
      completeError = review?.apply_status !== "ready"
        ? `Cannot apply: run contract is ${review?.apply_status ?? "unavailable"}.`
        : "Cannot apply: every agent must exit 0 with verifies passing.";
      return;
    }
    completing = true;
    completeError = null;
    try {
      await backend.applyRunChanges(run.id, domainId);
      onRunCompleted();
    } catch (e) {
      completeError = e instanceof Error ? e.message : String(e);
    } finally {
      completing = false;
    }
  }

  function shortRevision(revision: string | null | undefined): string {
    return revision ? revision.slice(0, 12) : "not recorded";
  }

  function statusTone(status: string): string {
    if (status === "enforced" || status === "ready" || status === "applied") return "positive";
    if (status === "advisory" || status === "pending" || status === "applying") return "caution";
    return "negative";
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
      <p>{mode === "topology-focus" ? "Structural relationships from the workspace graph." : "Review the plan, enforcement receipt, and isolated changes before one atomic Apply."}</p>
    </div>
    {#if mode === "run-review" && run}
      <button class="primary" disabled={completing || agentsLoading || !canApply} onclick={completeRun} title={!allExited ? "All agents must exit successfully with verifies passing before Apply" : review?.apply_status !== "ready" ? `Run contract is ${review?.apply_status ?? "unavailable"}` : "Apply all reviewed run changes atomically"}>
        {#if completing}<IconLoader2 size={16} class="spin" />{:else}<IconCheck size={16} />{/if} {applyLabel}
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
    {#if reviewError}
      <article class="panel review-blocked">
        <IconAlertTriangle size={22} />
        <div><h2>Review contract unavailable</h2><p>{reviewError}</p></div>
      </article>
    {/if}
    <div class="review-grid">
      <article class="panel trust-panel">
        <div class="panel-heading-row">
          <div><p class="eyebrow">Run contract</p><h2>{run ? "Trust boundary" : "No run selected"}</h2></div>
          {#if run}<span class={`status-chip ${statusTone(review?.apply_status ?? "unavailable")}`}>{review?.apply_status ?? "loading"}</span>{/if}
        </div>
        {#if run}
          <div class="contract-facts">
            <div><IconFingerprint size={15} /><span>Base revision</span><strong>{shortRevision(review?.base_revision)}</strong></div>
            <div><IconBinaryTree2 size={15} /><span>Plan</span><strong>{reviewTasks.length} tasks · {review?.plan.waves.length ?? 0} waves</strong></div>
            <div class:any-failed={anyFailed}><IconCheck size={15} /><span>Verification</span><strong>{agents.filter((agent) => agent.exit_code === 0).length}/{agents.length} passed</strong></div>
          </div>
          <div class="check-row" class:check-row--fail={!agentsLoading && !allExited}>
            {#if agentsLoading}<IconLoader2 size={16} class="spin" />{:else}<IconCheck size={16} />{/if}
            {agentsLoading ? "Checking agents…" : `${agents.filter((a) => a.exit_code === 0).length}/${agents.length} agents exited successfully`}
          </div>
          <div class="check-row"><IconCheck size={16} /> Permission profile: {run.permission_profile ?? "unknown"}</div>
          <div class="check-row"><IconCheck size={16} /> Isolation: {run.isolation_mode} via {run.isolation_backend}</div>
          <div class="check-row" class:check-row--fail={anyFailed}><IconCheck size={16} /> Run status: {run.status}</div>
          {#if runReceipt}
            <div class="surface-list">
              <div class="surface-row">
                <span class={`surface-mark ${statusTone(runReceipt.workspace_isolation.status)}`}></span>
                <div><strong>Workspace isolation</strong><small>{runReceipt.workspace_isolation.mechanism}</small></div>
                <span>{runReceipt.workspace_isolation.status}</span>
              </div>
              <div class="surface-row">
                <span class={`surface-mark ${statusTone(runReceipt.network.status)}`}></span>
                <div><strong>Network boundary</strong><small>{runReceipt.network.mechanism}</small></div>
                <span>{runReceipt.network.status}</span>
              </div>
              <div class="surface-row">
                <span class={`surface-mark ${statusTone(runReceipt.apply_boundary.status)}`}></span>
                <div><strong>Apply boundary</strong><small>{runReceipt.apply_boundary.mechanism}</small></div>
                <span>{runReceipt.apply_boundary.status}</span>
              </div>
            </div>
            {#if runReceipt.host_filesystem_boundary.status === "advisory"}
              <div class="boundary-note">
                <IconAlertTriangle size={16} />
                <p><strong>Host filesystem is policy-gated, not syscall-sandboxed.</strong><small>{runReceipt.host_filesystem_boundary.detail}</small></p>
              </div>
            {/if}
          {/if}
          <div class="shield-note">
            <IconShieldCheck size={19} />
            <p><strong>One reviewed run, one Apply</strong><small>Base and agent result digests are revalidated before any primary-repo write.</small></p>
          </div>
          {#if completeError}<p class="voice-state-message error">{completeError}</p>{/if}
        {:else}
          <div class="empty"><strong>Select a run from the Runs table first.</strong></div>
        {/if}
      </article>
      <article class="panel plan-panel">
        <div class="panel-heading-row">
          <div><p class="eyebrow">Reviewed DAG</p><h2>Ownership plan</h2></div>
          <span class="muted-count">{review?.plan.waves.length ?? 0} waves</span>
        </div>
        {#if reviewTasks.length}
          <div class="task-ledger">
            {#each reviewTasks as task (task.task_id)}
              <div class="task-row">
                <span class="wave-index">W{task.wave + 1}</span>
                <div>
                  <strong>{task.task_id}</strong>
                  <small>{task.agent} · {task.paths.join(", ")}</small>
                  {#if task.depends_on.length}<em>after {task.depends_on.join(", ")}</em>{/if}
                </div>
                <span class="verify-count">{task.verify.length ? `${task.verify.length} checks` : "no checks"}</span>
              </div>
            {/each}
          </div>
        {:else}
          <div class="empty compact"><strong>Loading reviewed plan…</strong></div>
        {/if}
      </article>
      <article class="panel diff-panel">
        <h2>{diffLoading ? "Loading diff…" : diffText ? "Changes recorded" : "No file changes"}</h2>
        <div class="agent-switcher" aria-label="Agent workspace">
          {#each agents as agent (agent.id)}
            <button class:active={selectedAgentId === agent.id} onclick={() => void loadDiffFor(agent.id)}>
              {agent.task_id}
              <span class:failed={agent.exit_code !== null && agent.exit_code !== 0}>{agent.exit_code === 0 ? "passed" : agent.status}</span>
            </button>
          {/each}
        </div>
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
  :global(.desktop2 .focus-screen .screen-heading .primary) {
    flex: 0 0 auto;
    min-width: 82px;
    white-space: nowrap;
  }
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
  .review-grid {
    display: grid;
    grid-template-columns: minmax(0, 0.9fr) minmax(0, 1.1fr);
    gap: 12px;
    align-items: start;
  }
  :global(.desktop2) .review-grid {
    grid-template-columns: minmax(0, 0.9fr) minmax(0, 1.1fr);
  }
  :global(.desktop2) .review-grid > .panel {
    padding: 0;
  }
  .review-grid > .panel {
    min-width: 0;
  }
  .diff-panel {
    grid-column: 1 / -1;
  }
  .panel-heading-row {
    display: flex;
    align-items: flex-start;
    justify-content: space-between;
    gap: 16px;
    padding: 16px 16px 12px;
    border-bottom: 1px solid #242a31;
  }
  .panel-heading-row .eyebrow,
  .panel-heading-row h2 {
    margin: 0;
  }
  .panel-heading-row h2 {
    margin-top: 4px;
    font-size: 15px;
  }
  .status-chip,
  .muted-count {
    flex: 0 0 auto;
    padding: 4px 8px;
    border: 1px solid #303740;
    border-radius: 999px;
    font: 600 10px/1 "Geist Mono", monospace;
    letter-spacing: 0.04em;
    text-transform: uppercase;
    color: #9aa3ae;
    background: #12161b;
  }
  .status-chip.positive {
    border-color: color-mix(in srgb, #38d6c1 40%, #303740);
    color: #9fe6d6;
  }
  .status-chip.caution {
    border-color: color-mix(in srgb, #e4b65a 42%, #303740);
    color: #e7c981;
  }
  .status-chip.negative {
    border-color: color-mix(in srgb, #ef6d78 42%, #303740);
    color: #f2a0a7;
  }
  .contract-facts {
    display: grid;
    grid-template-columns: repeat(3, minmax(0, 1fr));
    gap: 1px;
    margin: 12px 16px;
    overflow: hidden;
    border: 1px solid #262d35;
    border-radius: 8px;
    background: #262d35;
  }
  .contract-facts > div {
    display: grid;
    grid-template-columns: auto 1fr;
    gap: 2px 8px;
    align-items: center;
    min-width: 0;
    padding: 10px;
    background: #11151a;
    color: #66717d;
  }
  .contract-facts span {
    font-size: 10px;
    text-transform: uppercase;
    letter-spacing: 0.05em;
  }
  .contract-facts strong {
    grid-column: 1 / -1;
    overflow: hidden;
    font: 600 11px/1.35 "Geist Mono", monospace;
    color: #d3d8de;
    text-overflow: ellipsis;
    white-space: nowrap;
  }
  .contract-facts .any-failed strong {
    color: #f2a0a7;
  }
  .surface-list {
    margin: 0 16px 12px;
    border-top: 1px solid #242a31;
  }
  .surface-row {
    display: grid;
    grid-template-columns: 8px minmax(0, 1fr) auto;
    gap: 10px;
    align-items: center;
    padding: 9px 0;
    border-bottom: 1px solid #20262d;
  }
  .surface-row div {
    display: grid;
    gap: 2px;
    min-width: 0;
  }
  .surface-row strong {
    font-size: 12px;
    color: #d2d7dd;
  }
  .surface-row small {
    overflow: hidden;
    font: 10px/1.3 "Geist Mono", monospace;
    color: #727d89;
    text-overflow: ellipsis;
    white-space: nowrap;
  }
  .surface-row > span:last-child {
    font: 600 9px/1 "Geist Mono", monospace;
    color: #7f8a95;
    text-transform: uppercase;
  }
  .surface-mark {
    width: 7px;
    height: 7px;
    border-radius: 50%;
    background: #ef6d78;
    box-shadow: 0 0 0 3px color-mix(in srgb, #ef6d78 12%, transparent);
  }
  .surface-mark.positive {
    background: #38d6c1;
    box-shadow: 0 0 0 3px color-mix(in srgb, #38d6c1 12%, transparent);
  }
  .surface-mark.caution {
    background: #e4b65a;
    box-shadow: 0 0 0 3px color-mix(in srgb, #e4b65a 12%, transparent);
  }
  .boundary-note,
  .review-blocked {
    display: flex;
    gap: 10px;
    align-items: flex-start;
    color: #e7c981;
  }
  .boundary-note {
    margin: 0 16px 12px;
    padding: 10px 12px;
    border: 1px solid color-mix(in srgb, #e4b65a 25%, #252c34);
    border-radius: 8px;
    background: color-mix(in srgb, #e4b65a 6%, #11151a);
  }
  .boundary-note p,
  .boundary-note strong,
  .boundary-note small,
  .review-blocked h2,
  .review-blocked p {
    display: block;
    margin: 0;
  }
  .boundary-note strong {
    font-size: 11px;
  }
  .boundary-note small {
    margin-top: 3px;
    color: #8d8572;
  }
  .review-blocked {
    margin-bottom: 12px;
    padding: 14px 16px;
  }
  .review-blocked h2 {
    font-size: 14px;
  }
  .review-blocked p {
    margin-top: 3px;
    color: #a68f92;
  }
  .task-ledger {
    display: grid;
  }
  .task-row {
    display: grid;
    grid-template-columns: auto minmax(0, 1fr) auto;
    gap: 10px;
    align-items: start;
    padding: 11px 16px;
    border-bottom: 1px solid #20262d;
  }
  .task-row:last-child {
    border-bottom: 0;
  }
  .wave-index {
    padding: 3px 5px;
    border-radius: 4px;
    font: 600 9px/1 "Geist Mono", monospace;
    color: #80cabf;
    background: #14211f;
  }
  .task-row > div {
    display: grid;
    gap: 3px;
    min-width: 0;
  }
  .task-row strong {
    font-size: 12px;
    color: #d5dae0;
  }
  .task-row small,
  .task-row em,
  .verify-count {
    overflow: hidden;
    font: 10px/1.35 "Geist Mono", monospace;
    color: #78838e;
    text-overflow: ellipsis;
    white-space: nowrap;
  }
  .task-row em {
    color: #9a8fbc;
  }
  .verify-count {
    color: #69737e;
  }
  .agent-switcher {
    display: flex;
    gap: 6px;
    padding: 10px 16px;
    overflow-x: auto;
    border-bottom: 1px solid #242a31;
  }
  .agent-switcher button {
    display: inline-flex;
    gap: 7px;
    align-items: center;
    padding: 6px 8px;
    border: 1px solid #2b323a;
    border-radius: 6px;
    font: 600 10px/1 "Geist Mono", monospace;
    color: #87919c;
    white-space: nowrap;
    background: #11151a;
  }
  .agent-switcher button.active {
    border-color: color-mix(in srgb, #38d6c1 42%, #2b323a);
    color: #d9e7e4;
    background: #14201f;
  }
  .agent-switcher span {
    color: #6fb7ac;
    font-size: 9px;
    text-transform: uppercase;
  }
  .agent-switcher span.failed {
    color: #e1878e;
  }
  .empty.compact {
    min-height: 110px;
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
  @media (max-width: 980px) {
    :global(.desktop2) .review-grid {
      grid-template-columns: 1fr;
    }
    .diff-panel {
      grid-column: auto;
    }
  }
  @media (max-width: 680px) {
    .contract-facts {
      grid-template-columns: 1fr;
    }
    .task-row {
      grid-template-columns: auto minmax(0, 1fr);
    }
    .verify-count {
      display: none;
    }
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
