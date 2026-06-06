<script lang="ts">
  import { invoke } from "@tauri-apps/api/core";
  import { onMount, onDestroy } from "svelte";
  import { Terminal } from "@xterm/xterm";
  import { FitAddon } from "@xterm/addon-fit";
  import "@xterm/xterm/css/xterm.css";
  import TopologyPanel from "./TopologyPanel.svelte";

  type DomainDto = { domain_id: string; repo_root: string };

  type RunDto = {
    id: string;
    status: string;
    repo_root: string;
    started_at: string;
    estimated_cost_usd: number | null;
  };

  type AgentDto = {
    id: string;
    run_id: string;
    task_id: string;
    wave: number;
    status: string;
    exit_code: number | null;
    root_id: string | null;
  };

  type ProjectDto = { id: string; manifest_path: string };

  type EventDto = {
    id: number;
    agent_id: string;
    kind: string;
    payload: string;
    ts: string;
  };

  type CatalogEntry = {
    domain_id: string;
    repo_root: string;
    db_path: string;
    project_id: string | null;
    status: string;
    updated_at: string;
  };

  let domains: DomainDto[] = $state([]);
  let allDomains: CatalogEntry[] = $state([]);
  let projects: ProjectDto[] = $state([]);
  let selectedProjectId = $state<string | null>(null);
  let rootFilter = $state<string | null>(null);
  let selectedDomainId = $state<string | null>(null);
  let runs: RunDto[] = $state([]);
  let agents: AgentDto[] = $state([]);
  let selectedRunId = $state<string | null>(null);
  let selectedAgentId = $state<string | null>(null);
  type HitlDto = {
    id: string;
    agent_key: string;
    action: string;
    reason: string;
    created_at_ms: string;
  };

  let cmd = $state("echo pytxo-wave");
  let dispatchRepo = $state("");
  let dryRunOut = $state("");
  let hitl: HitlDto[] = $state([]);

  type AgentArbitrageDto = {
    agent_id: string;
    saved_tokens: number;
    edited_paths: number;
  };
  let arbitrage: AgentArbitrageDto[] = $state([]);
  let signalRetries: string[] = $state([]);
  let diffText = $state("");
  let termEl: HTMLDivElement | undefined = $state();
  let terminal: Terminal | null = null;
  let fitAddon: FitAddon | null = null;
  let pollTimer: ReturnType<typeof setInterval> | null = null;
  let hitlTimer: ReturnType<typeof setInterval> | null = null;
  const MAX_TERMINAL_LINES = 2000;
  let terminalLineCount = 0;

  function domainArgs() {
    return { domainId: selectedDomainId };
  }

  let catalogForView = $derived(
    selectedProjectId
      ? allDomains.filter((e) => e.project_id === selectedProjectId)
      : allDomains
  );

  let filteredAgents = $derived(
    rootFilter ? agents.filter((a) => a.root_id === rootFilter) : agents
  );

  function recordEvent(kind: string, payload: string) {
    // Surface closed-loop fidelity escalations ([[closed-loop-fidelity]]).
    if (kind === "signal-retry") {
      signalRetries = [...signalRetries, payload].slice(-20);
    }
  }

  function writelnCapped(line: string) {
    if (!terminal) return;
    terminal.writeln(line);
    terminalLineCount += 1;
    if (terminalLineCount > MAX_TERMINAL_LINES) {
      const trim = terminalLineCount - MAX_TERMINAL_LINES;
      terminal.writeln(`… trimmed ${trim} older lines …`);
      terminal.clear();
      terminalLineCount = 1;
    }
  }

  async function refreshDomains() {
    domains = await invoke<DomainDto[]>("list_domains_cmd");
    try {
      allDomains = await invoke<CatalogEntry[]>("list_all_domains");
    } catch {
      allDomains = [];
    }
    try {
      projects = await invoke<ProjectDto[]>("list_projects");
    } catch {
      projects = [];
    }
    if (!selectedDomainId && domains.length > 0) {
      await selectDomain(domains[0].domain_id);
    }
  }

  async function selectCatalogDomain(entry: CatalogEntry) {
    if (!domains.some((d) => d.domain_id === entry.domain_id)) {
      domains = [
        ...domains,
        { domain_id: entry.domain_id, repo_root: entry.repo_root },
      ];
    }
    await selectDomain(entry.domain_id);
  }

  async function selectDomain(domainId: string) {
    selectedDomainId = domainId;
    await invoke("select_domain", { domainId });
    selectedRunId = null;
    selectedAgentId = null;
    await refreshRuns();
  }

  async function refreshRuns() {
    runs = await invoke<RunDto[]>("list_runs", { limit: 20, ...domainArgs() });
    if (!selectedRunId && runs.length > 0) {
      selectedRunId = runs[0].id;
      await selectRun(selectedRunId);
    }
  }

  async function selectRun(runId: string) {
    selectedRunId = runId;
    agents = await invoke<AgentDto[]>("list_agents", { runId, ...domainArgs() });
    if (agents.length > 0) {
      selectedAgentId = agents[0].id;
      await loadTerminalHistory();
    }
    await refreshArbitrage();
  }

  async function loadTerminalHistory() {
    if (!selectedAgentId || !terminal) return;
    terminal.clear();
    const events = await invoke<EventDto[]>("tail_events", {
      agentId: selectedAgentId,
      tail: 200,
      ...domainArgs(),
    });
    terminalLineCount = 0;
    signalRetries = [];
    for (const ev of events) {
      writelnCapped(`[${ev.kind}] ${ev.payload}`);
      recordEvent(ev.kind, ev.payload);
    }
  }

  async function pollLogs() {
    if (!selectedAgentId || !terminal) return;
    const lines = await invoke<EventDto[]>("poll_log_lines", {
      agentId: selectedAgentId,
      limit: 32,
      ...domainArgs(),
    });
    for (const ev of lines) {
      writelnCapped(`[${ev.kind}] ${ev.payload}`);
      recordEvent(ev.kind, ev.payload);
    }
  }

  async function refreshArbitrage() {
    if (!selectedRunId) {
      arbitrage = [];
      return;
    }
    try {
      arbitrage = await invoke<AgentArbitrageDto[]>("agent_arbitrage", {
        runId: selectedRunId,
        ...domainArgs(),
      });
    } catch {
      arbitrage = [];
    }
  }

  async function refreshHitl() {
    if (!selectedDomainId) {
      hitl = [];
      return;
    }
    try {
      hitl = await invoke<HitlDto[]>("list_hitl", { ...domainArgs() });
    } catch {
      hitl = [];
    }
  }

  async function respondHitl(id: string, approve: boolean) {
    await invoke("hitl_respond", { requestId: id, approve, ...domainArgs() });
    await refreshHitl();
  }

  async function doDryRun() {
    dryRunOut = await invoke<string>("dry_run", { agents: 3, ...domainArgs() });
  }

  async function doStart() {
    const repoRoot = dispatchRepo.trim() || selectedDomainId;
    const runId = await invoke<string>("dispatch_run_cmd", {
      cmd,
      agents: 3,
      repoRoot,
    });
    await refreshDomains();
    await refreshRuns();
    selectedRunId = runId;
    await selectRun(runId);
  }

  async function doStop() {
    await invoke("stop_run", { all: false });
    await refreshRuns();
  }

  async function loadDiff() {
    if (!selectedAgentId) return;
    try {
      diffText = await invoke<string>("git_diff", {
        agentId: selectedAgentId,
        ...domainArgs(),
      });
    } catch (e) {
      diffText = String(e);
    }
  }

  async function doCommit() {
    if (!selectedRunId || !selectedAgentId) return;
    const parts = selectedAgentId.split(":");
    const agentOnly = parts.length > 1 ? parts[1] : selectedAgentId;
    await invoke("commit_workspace", {
      runId: selectedRunId,
      agentId: agentOnly,
      ...domainArgs(),
    });
    await loadDiff();
  }

  onMount(async () => {
    if (termEl) {
      terminal = new Terminal({
        theme: { background: "#020205", foreground: "#e8eaed" },
        fontSize: 13,
        convertEol: true,
      });
      fitAddon = new FitAddon();
      terminal.loadAddon(fitAddon);
      terminal.open(termEl);
      fitAddon.fit();
    }
    await refreshDomains();
    pollTimer = setInterval(pollLogs, 16);
    hitlTimer = setInterval(() => {
      refreshHitl();
      refreshArbitrage();
    }, 1000);
  });

  onDestroy(() => {
    if (pollTimer) clearInterval(pollTimer);
    if (hitlTimer) clearInterval(hitlTimer);
    terminal?.dispose();
  });
</script>

<main>
  <header>
    <h1>Pytxo Reality Deck</h1>
    <div class="actions">
      <input
        bind:value={dispatchRepo}
        placeholder="repo path (blank = selected domain)"
        title="Absolute path to a project folder; blank dispatches to the selected domain"
      />
      <input bind:value={cmd} placeholder="command" />
      <button onclick={doDryRun}>Dry run</button>
      <button class="primary" onclick={doStart}>Dispatch</button>
      <button onclick={doStop}>Stop</button>
      <button onclick={refreshRuns}>Refresh</button>
    </div>
  </header>

  <div class="layout">
    <aside>
      {#if projects.length > 0}
        <h2>Projects</h2>
        <ul>
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
        <h2>Domains ({catalogForView.length})</h2>
        <ul>
          {#each catalogForView as entry}
            <li>
              <button
                class:selected={entry.domain_id === selectedDomainId}
                onclick={() => selectCatalogDomain(entry)}
                title={entry.repo_root}
              >
                {entry.repo_root.split(/[/\\]/).pop() ?? entry.domain_id.slice(0, 12)}
                {#if entry.project_id}
                  <span class="cost">{entry.project_id}</span>
                {/if}
              </button>
            </li>
          {/each}
        </ul>
      {/if}

      <h2>Active domains</h2>
      <ul>
        {#each domains as d}
          <li>
            <button
              class:selected={d.domain_id === selectedDomainId}
              onclick={() => selectDomain(d.domain_id)}
            >
              {d.repo_root.split(/[/\\]/).pop() ?? d.domain_id.slice(0, 12)}
            </button>
          </li>
        {/each}
      </ul>

      <h2>Runs</h2>
      <ul>
        {#each runs as run}
          <li>
            <button
              class:selected={run.id === selectedRunId}
              onclick={() => selectRun(run.id)}
            >
              {run.id.slice(0, 8)}… — {run.status}
              {#if run.estimated_cost_usd != null}
                <span class="cost">${run.estimated_cost_usd.toFixed(4)}</span>
              {/if}
            </button>
          </li>
        {/each}
      </ul>

      <h2>Waves</h2>
      {#if agents.some((a) => a.root_id)}
        <div class="root-filter">
          <button
            class:selected={rootFilter === null}
            onclick={() => (rootFilter = null)}
          >
            All roots
          </button>
          {#each [...new Set(agents.map((a) => a.root_id).filter(Boolean))] as root}
            <button
              class:selected={rootFilter === root}
              onclick={() => (rootFilter = root)}
            >
              {root}
            </button>
          {/each}
        </div>
      {/if}
      <ul class="agents">
        {#each filteredAgents as agent}
          <li>
            <button
              class:selected={agent.id === selectedAgentId}
              onclick={async () => {
                selectedAgentId = agent.id;
                await loadTerminalHistory();
              }}
            >
              w{agent.wave} {agent.task_id}
              {#if agent.root_id}
                <span class="cost">@{agent.root_id}</span>
              {/if}
              — {agent.status}
            </button>
          </li>
        {/each}
      </ul>

      {#if hitl.length > 0}
        <h2 class="hitl-title">Approvals ({hitl.length})</h2>
        <ul class="hitl">
          {#each hitl as req}
            <li class="hitl-item">
              <div class="hitl-action">{req.action}</div>
              <div class="hitl-reason">{req.reason}</div>
              <div class="hitl-buttons">
                <button class="gold" onclick={() => respondHitl(req.id, true)}>
                  Approve
                </button>
                <button onclick={() => respondHitl(req.id, false)}>Deny</button>
              </div>
            </li>
          {/each}
        </ul>
      {/if}

      <TopologyPanel agents={filteredAgents} {arbitrage} />
    </aside>

    <section class="terminal-pane">
      <h2>Telemetry</h2>
      {#if signalRetries.length > 0}
        <div class="signal-retries">
          <span class="signal-retries__label">Signal retries ({signalRetries.length})</span>
          <ul>
            {#each signalRetries.slice(-5) as retry}
              <li>{retry}</li>
            {/each}
          </ul>
        </div>
      {/if}
      <div class="term" bind:this={termEl}></div>
    </section>

    <section class="diff-pane">
      <h2>
        Diff
        <button onclick={loadDiff}>Load</button>
        <button class="gold" onclick={doCommit}>Approve merge</button>
      </h2>
      <pre>{diffText || dryRunOut || "Dry-run output appears here after Dry run."}</pre>
    </section>
  </div>
</main>

<style>
  :global(:root) {
    --void: #020205;
    --void-elevated: #0a0c12;
    --text: #e8eaed;
    --border: #1e2433;
    --accent-teal: #2dd4bf;
    --accent-violet: #a78bfa;
    --accent-gold: #fbbf24;
  }
  :global(body) {
    margin: 0;
    font-family: system-ui, sans-serif;
    background: var(--void);
    color: var(--text);
  }
  .signal-retries {
    border: 1px solid var(--accent-gold);
    border-radius: 6px;
    padding: 0.4rem 0.6rem;
    margin-bottom: 0.5rem;
    background: rgba(251, 191, 36, 0.08);
    font-size: 0.8rem;
  }
  .signal-retries__label {
    color: var(--accent-gold);
    font-weight: 600;
  }
  .signal-retries ul {
    margin: 0.25rem 0 0;
    padding-left: 1rem;
    color: var(--text);
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
  main {
    display: flex;
    flex-direction: column;
    height: 100vh;
  }
  header {
    display: flex;
    align-items: center;
    justify-content: space-between;
    padding: 0.75rem 1rem;
    border-bottom: 1px solid var(--border);
  }
  h1 {
    color: var(--accent-teal);
    font-size: 1.1rem;
    margin: 0;
  }
  .actions {
    display: flex;
    gap: 0.5rem;
  }
  input {
    min-width: 220px;
    padding: 0.35rem 0.5rem;
    border-radius: 6px;
    border: 1px solid var(--border);
    background: var(--void-elevated);
    color: inherit;
  }
  button {
    padding: 0.35rem 0.75rem;
    border-radius: 6px;
    border: 1px solid var(--border);
    background: var(--void-elevated);
    color: inherit;
    cursor: pointer;
  }
  button.primary {
    border-color: var(--accent-teal);
    color: var(--accent-teal);
  }
  button.gold {
    border-color: var(--accent-gold);
    color: var(--accent-gold);
  }
  button.selected {
    border-color: var(--accent-violet);
  }
  .layout {
    display: grid;
    grid-template-columns: 280px 1fr 320px;
    flex: 1;
    min-height: 0;
  }
  aside {
    border-right: 1px solid var(--border);
    padding: 0.75rem;
    overflow: auto;
  }
  aside ul {
    list-style: none;
    padding: 0;
    margin: 0 0 1rem;
  }
  aside li button {
    width: 100%;
    text-align: left;
    margin-bottom: 0.25rem;
  }
  .cost {
    display: block;
    font-size: 0.75rem;
    opacity: 0.7;
    color: var(--accent-gold);
    font-variant-numeric: tabular-nums;
  }
  .terminal-pane,
  .diff-pane {
    display: flex;
    flex-direction: column;
    min-height: 0;
    padding: 0.75rem;
  }
  .term {
    flex: 1;
    min-height: 200px;
    background: #010409;
    border-radius: 8px;
    padding: 4px;
    border: 1px solid var(--border);
  }
  pre {
    flex: 1;
    overflow: auto;
    font-size: 12px;
    background: var(--void-elevated);
    padding: 0.75rem;
    border-radius: 8px;
    margin: 0;
  }
  h2 {
    font-size: 0.9rem;
    margin: 0 0 0.5rem;
    color: var(--accent-violet);
  }
  .hitl-title {
    color: var(--accent-gold);
  }
  ul.hitl {
    list-style: none;
    padding: 0;
    margin: 0 0 1rem;
  }
  .hitl-item {
    border: 1px solid var(--accent-gold);
    border-radius: 6px;
    padding: 0.4rem 0.5rem;
    margin-bottom: 0.4rem;
  }
  .hitl-action {
    font-weight: 600;
    color: var(--accent-gold);
    font-size: 0.85rem;
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
</style>
