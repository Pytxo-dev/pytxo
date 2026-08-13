<script lang="ts">
  import { onMount } from "svelte";
  import {
    IconArrowLeft,
    IconCheck,
    IconGitBranch,
    IconLoader2,
    IconPlayerStop,
    IconShieldCheck,
    IconShieldLock,
  } from "@tabler/icons-svelte";
  import { ipc } from "../../lib/ipc";
  import type { DesktopBackend } from "../../lib/desktop-backend";
  import type { MissionPane } from "../../lib/navigation.svelte";
  import type { AgentArbitrageDto, AgentDto, RunDto, RunReviewDto, StructuralGraphDto } from "../../lib/types";

  let {
    pane,
    run,
    domainId,
    backend,
    onPane,
    onBack,
    onRunCompleted,
    onStopRun = null,
  }: {
    pane: MissionPane;
    run: RunDto | null;
    domainId: string | null;
    backend: DesktopBackend;
    onPane: (pane: MissionPane) => void;
    onBack: () => void;
    onRunCompleted: () => void;
    onStopRun?: ((runId: string, domainId: string) => Promise<void>) | null;
  } = $props();

  let graph = $state<StructuralGraphDto>({ nodes: [], edges: [] });
  let graphLoading = $state(true);
  let graphError = $state<string | null>(null);
  let agents = $state<AgentDto[]>([]);
  let agentsLoading = $state(true);
  let diffText = $state("");
  let diffLoading = $state(false);
  let completing = $state(false);
  let completeError = $state<string | null>(null);
  let review = $state<RunReviewDto | null>(null);
  let arbitrage = $state<AgentArbitrageDto[]>([]);
  let stopDialog: HTMLDialogElement | undefined = $state();
  let stopError = $state("");
  let stopping = $state(false);

  const workspaceLabel = $derived(run?.repo_root.split(/[\\/]/).pop() ?? domainId?.split(/[\\/]/).pop() ?? "Mission");
  const running = $derived(
    !!run && ["running", "pending", "dispatching", "active"].includes(run.status.toLowerCase()),
  );
  const contended = $derived(
    agents.filter((agent) => {
      const status = agent.status.toLowerCase();
      return status.includes("block") || status.includes("wait") || status.includes("hold") || status.includes("claim") || status === "queued";
    }),
  );
  const allExited = $derived(agents.length > 0 && agents.every((a) => a.exit_code === 0));
  const anyFailed = $derived(agents.some((a) => a.exit_code !== null && a.exit_code !== 0));
  const applyStatus = $derived(review?.apply_status ?? run?.apply_status ?? "unavailable");
  const canApply = $derived(allExited && applyStatus === "ready");
  const filesTouched = $derived(graph.nodes.filter((n) => n.edited).length ? graph.nodes.filter((n) => n.edited) : graph.nodes);
  const savedTokens = $derived(arbitrage.reduce((n, a) => n + (a.saved_tokens ?? 0), 0));

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

  async function loadReview() {
    if (!run) {
      agentsLoading = false;
      return;
    }
    agentsLoading = true;
    try {
      agents = await ipc.listAgents(run.id, domainId);
      const withRoot = agents.find((a) => a.root_id) ?? agents[0] ?? null;
      if (withRoot) {
        diffLoading = true;
        try {
          diffText = await ipc.gitDiff(withRoot.id, domainId);
        } catch {
          diffText = "";
        } finally {
          diffLoading = false;
        }
      }
    } finally {
      agentsLoading = false;
    }
  }

  async function loadArbitrage() {
    if (!run) {
      arbitrage = [];
      return;
    }
    try {
      arbitrage = await ipc.agentArbitrage(run.id, domainId);
    } catch {
      arbitrage = [];
    }
  }

  async function loadApplyContract() {
    if (!run) {
      review = null;
      return;
    }
    try {
      review = await backend.runReview(run.id, domainId);
    } catch {
      review = null;
    }
  }

  async function applyRun() {
    if (!run) return;
    if (!canApply) {
      completeError = !allExited
        ? "Cannot apply: every agent must exit 0 with verifies passing."
        : `Cannot apply: run contract is ${applyStatus}.`;
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

  async function discardRun() {
    if (!run) return;
    completing = true;
    completeError = null;
    try {
      await backend.discardRunReview(run.id, domainId);
      onBack();
    } catch (e) {
      completeError = e instanceof Error ? e.message : String(e);
    } finally {
      completing = false;
    }
  }

  function requestStop() {
    if (!run || !domainId || !onStopRun) return;
    stopError = "";
    queueMicrotask(() => {
      if (stopDialog && !stopDialog.open) stopDialog.showModal();
    });
  }

  async function confirmStop() {
    if (!run || !domainId || !onStopRun || stopping) return;
    stopping = true;
    try {
      await onStopRun(run.id, domainId);
      stopDialog?.close();
    } catch (e) {
      stopError = e instanceof Error ? e.message : String(e);
    } finally {
      stopping = false;
    }
  }

  onMount(() => {
    void loadTopology();
    void loadReview();
    void loadArbitrage();
    void loadApplyContract();
  });
</script>

<section class="screen focus-screen">
  <header class="screen-heading">
    <div>
      <button class="back" onclick={onBack}><IconArrowLeft size={15} /> Missions</button>
      <h1>Mission</h1>
    </div>
    <div class="heading-actions">
      {#if running && onStopRun && domainId}
        <button class="quiet" onclick={requestStop}><IconPlayerStop size={15} /> Stop</button>
      {/if}
      {#if pane === "review" && run}
        <button class="quiet" disabled={completing} onclick={() => void discardRun()}>Discard</button>
        <button class="primary" disabled={completing || agentsLoading || !canApply} onclick={() => void applyRun()} title={!allExited ? "All agents must exit successfully with verifies passing before Apply" : applyStatus !== "ready" ? `Run contract is ${applyStatus}` : "Apply all reviewed run changes atomically"}>
          {#if completing}<IconLoader2 size={16} class="spin" />{:else}<IconCheck size={16} />{/if} Apply
        </button>
      {/if}
    </div>
  </header>

  {#if run}
    <p class="mission-meta">
      <span>{workspaceLabel}</span>
      <span class="mono">{run.id}</span>
      <span>{run.permission_profile ?? "orbit"}</span>
      <span>{run.isolation_mode} via {run.isolation_backend}</span>
      <span>${(run.estimated_cost_usd ?? 0).toFixed(2)}</span>
      <span>{run.status}</span>
    </p>
  {/if}

  <div class="mission-tabs" role="tablist" aria-label="Mission panes">
    <button class:active={pane === "plan"} onclick={() => onPane("plan")}>Plan</button>
    <button class:active={pane === "live"} onclick={() => onPane("live")}>Live</button>
    <button class:active={pane === "review"} onclick={() => onPane("review")}>Review</button>
  </div>

  {#if !run}
    <div class="empty">
      <strong>No mission selected</strong>
      <span>Open a mission from the list, or start a new one.</span>
    </div>
  {:else if pane === "plan"}
    <article class="panel">
      <div class="panel-head"><div><h2>Tasks and claims</h2></div></div>
      {#if agentsLoading}
        <div class="empty"><IconLoader2 size={22} class="spin" /><strong>Loading tasks…</strong></div>
      {:else if !agents.length}
        <div class="empty"><strong>No tasks yet</strong><span>The planner has not assigned agents for this run.</span></div>
      {:else}
        {#each agents as agent (agent.id)}
          <div class="run-row">
            <span class="run-copy">
              <strong>{agent.task_id || agent.id}</strong>
              <small>Stage {agent.wave} · {agent.status}</small>
            </span>
            <span class="mono">{agent.id}</span>
          </div>
        {/each}
      {/if}
    </article>
  {:else if pane === "live"}
    <article class="panel">
      <div class="panel-head"><div><h2>Agents</h2></div></div>
      {#if agentsLoading}
        <div class="empty"><IconLoader2 size={22} class="spin" /><strong>Loading agents…</strong></div>
      {:else if !agents.length}
        <div class="empty"><strong>No agents yet</strong><span>Live status appears after dispatch.</span></div>
      {:else}
        {#each agents as agent (agent.id)}
          <div class="run-row">
            <span class:running={agent.status === "running"} class="status-dot"></span>
            <span class="run-copy">
              <strong>{agent.id}</strong>
              <small>{agent.status} · stage {agent.wave}{#if agent.exit_code !== null} · exit {agent.exit_code}{/if}</small>
            </span>
          </div>
        {/each}
      {/if}
      {#if contended.length}
        <div class="panel-head"><div><h2>Path locks</h2></div><span>{contended.length}</span></div>
        {#each contended as agent (agent.id)}
          <div class="race-row">
            <IconShieldLock size={15} />
            <span class="run-copy"><strong>{agent.id}</strong><small>{agent.status} · {agent.task_id}</small></span>
          </div>
        {/each}
      {/if}
      <details class="collapsed-log">
        <summary>Log</summary>
        <p>Agent output stays in the vendor CLI session. Pytxo does not mirror a terminal wall here.</p>
      </details>
    </article>
  {:else}
    <div class="review-grid">
      <article class="panel">
        <h2>Checks</h2>
        <div class="check-row" class:check-row--fail={!agentsLoading && !allExited}>
          {#if agentsLoading}<IconLoader2 size={16} class="spin" />{:else}<IconCheck size={16} />{/if}
          {agentsLoading ? "Checking agents…" : `${agents.filter((a) => a.exit_code === 0).length}/${agents.length} agents exited successfully`}
        </div>
        <div class="check-row"><IconCheck size={16} /> Permission profile: {run.permission_profile ?? "unknown"}</div>
        <div class="check-row"><IconCheck size={16} /> Isolation: {run.isolation_mode} via {run.isolation_backend}</div>
        <div class="check-row" class:check-row--fail={anyFailed}><IconCheck size={16} /> Run status: {run.status}</div>
        <div class="check-row">Cost ${(run.estimated_cost_usd ?? 0).toFixed(2)}</div>
        <div class="shield-note">
          <IconShieldCheck size={19} />
          <p><strong>Sandbox {run.isolation_mode === "copy_on_write" ? "intact" : "not isolated for this run"}</strong><small>Changes stay isolated until you apply.</small></p>
        </div>
        {#if savedTokens > 0}
          <p class="honesty">Signal Core saved {savedTokens.toLocaleString()} tokens on this run.</p>
        {/if}
        {#if completeError}<p class="voice-state-message error">{completeError}</p>{/if}
      </article>
      <article class="panel">
        <h2>Files touched</h2>
        {#if graphLoading}
          <div class="empty"><IconLoader2 size={22} class="spin" /><strong>Loading files…</strong></div>
        {:else if graphError}
          <div class="empty"><strong>Could not load files</strong><span>{graphError}</span></div>
        {:else if !filesTouched.length}
          <div class="empty">
            <IconGitBranch size={26} />
            <strong>No files touched</strong>
            <span>This run has not modified any tracked files.</span>
          </div>
        {:else}
          <div class="structural-list">
            {#each filesTouched as node (node.id)}
              <div class="structural-node" class:edited={node.edited}>
                <IconGitBranch size={15} />
                <strong>{node.label}</strong>
              </div>
            {/each}
          </div>
        {/if}
        {#if diffLoading}
          <div class="empty"><IconLoader2 size={22} class="spin" /><strong>Loading diff…</strong></div>
        {:else if diffText}
          <pre class="diff-raw">{diffText}</pre>
        {/if}
      </article>
    </div>
  {/if}
</section>

{#if run && domainId}
  <dialog
    bind:this={stopDialog}
    class="stop-run-dialog"
    aria-labelledby="mission-stop-title"
    oncancel={(event) => {
      event.preventDefault();
      if (!stopping) stopDialog?.close();
    }}
  >
    <section>
      <div class="stop-run-heading">
        <span><IconPlayerStop size={18} /></span>
        <div>
          <small>Process control</small>
          <h2 id="mission-stop-title">Stop active run?</h2>
        </div>
      </div>
      <p>This will terminate the agent processes for this exact run.</p>
      <dl>
        <div><dt>Workspace</dt><dd>{workspaceLabel}</dd></div>
        <div><dt>Run</dt><dd class="mono">{run.id}</dd></div>
        <div><dt>Execution domain</dt><dd class="mono">{domainId}</dd></div>
      </dl>
      <div class="stop-run-consequence">
        <IconShieldLock size={18} />
        <p><strong>Sandbox preserved</strong><small>No changes are flushed and the sandbox is not deleted.</small></p>
      </div>
      {#if stopError}<div class="stop-run-error" role="alert">{stopError}</div>{/if}
      <footer>
        <button class="quiet" disabled={stopping} onclick={() => stopDialog?.close()}>Keep running</button>
        <button class="stop-confirm" disabled={stopping} onclick={confirmStop}>
          <IconPlayerStop size={15} /> {stopping ? "Stopping…" : "Stop run"}
        </button>
      </footer>
    </section>
  </dialog>
{/if}

<style>
  .heading-actions {
    display: flex;
    gap: 8px;
    align-items: center;
  }
  .collapsed-log {
    margin-top: 16px;
    color: var(--pytxo-text-muted);
    font-size: 13px;
  }
  .collapsed-log summary {
    cursor: pointer;
  }
  .diff-raw {
    margin: 12px 0 0;
    padding: 14px;
    max-height: 320px;
    overflow: auto;
    font: 12px/1.6 "Geist Mono", monospace;
    color: var(--pytxo-text-body);
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
