<script lang="ts">
  import { invoke } from "@tauri-apps/api/core";
  import { onMount, onDestroy } from "svelte";
  import { Terminal } from "@xterm/xterm";
  import { FitAddon } from "@xterm/addon-fit";
  import "@xterm/xterm/css/xterm.css";

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
  };

  type EventDto = {
    id: number;
    agent_id: string;
    kind: string;
    payload: string;
    ts: string;
  };

  let runs: RunDto[] = $state([]);
  let agents: AgentDto[] = $state([]);
  let selectedRunId = $state<string | null>(null);
  let selectedAgentId = $state<string | null>(null);
  let cmd = $state("echo pytxo-wave");
  let dryRunOut = $state("");
  let diffText = $state("");
  let termEl: HTMLDivElement | undefined = $state();
  let terminal: Terminal | null = null;
  let fitAddon: FitAddon | null = null;
  let pollTimer: ReturnType<typeof setInterval> | null = null;
  const MAX_TERMINAL_LINES = 2000;
  let terminalLineCount = 0;

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

  async function refreshRuns() {
    runs = await invoke<RunDto[]>("list_runs", { limit: 20 });
    if (!selectedRunId && runs.length > 0) {
      selectedRunId = runs[0].id;
      await selectRun(selectedRunId);
    }
  }

  async function selectRun(runId: string) {
    selectedRunId = runId;
    agents = await invoke<AgentDto[]>("list_agents", { runId });
    if (agents.length > 0) {
      selectedAgentId = agents[0].id;
      await loadTerminalHistory();
    }
  }

  async function loadTerminalHistory() {
    if (!selectedAgentId || !terminal) return;
    terminal.clear();
    const events = await invoke<EventDto[]>("tail_events", {
      agentId: selectedAgentId,
      tail: 200,
    });
    terminalLineCount = 0;
    for (const ev of events) {
      writelnCapped(`[${ev.kind}] ${ev.payload}`);
    }
  }

  async function pollLogs() {
    if (!selectedAgentId || !terminal) return;
    const lines = await invoke<EventDto[]>("poll_log_lines", {
      agentId: selectedAgentId,
      limit: 32,
    });
    for (const ev of lines) {
      writelnCapped(`[${ev.kind}] ${ev.payload}`);
    }
  }

  async function doDryRun() {
    dryRunOut = await invoke<string>("dry_run", { agents: 3 });
  }

  async function doStart() {
    const runId = await invoke<string>("start_run", { cmd, agents: 3 });
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
      });
    } catch (e) {
      diffText = String(e);
    }
  }

  onMount(async () => {
    if (termEl) {
      terminal = new Terminal({
        theme: { background: "#0f1419", foreground: "#e6edf3" },
        fontSize: 13,
        convertEol: true,
      });
      fitAddon = new FitAddon();
      terminal.loadAddon(fitAddon);
      terminal.open(termEl);
      fitAddon.fit();
    }
    await refreshRuns();
    pollTimer = setInterval(pollLogs, 16);
  });

  onDestroy(() => {
    if (pollTimer) clearInterval(pollTimer);
    terminal?.dispose();
  });
</script>

<main>
  <header>
    <h1>Pytxo Reality Deck</h1>
    <div class="actions">
      <input bind:value={cmd} placeholder="command" />
      <button onclick={doDryRun}>Dry run</button>
      <button onclick={doStart}>Start run</button>
      <button onclick={doStop}>Stop</button>
      <button onclick={refreshRuns}>Refresh</button>
    </div>
  </header>

  <div class="layout">
    <aside>
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
      <ul class="agents">
        {#each agents as agent}
          <li>
            <button
              class:selected={agent.id === selectedAgentId}
              onclick={async () => {
                selectedAgentId = agent.id;
                await loadTerminalHistory();
              }}
            >
              w{agent.wave} {agent.task_id} — {agent.status}
            </button>
          </li>
        {/each}
      </ul>
    </aside>

    <section class="terminal-pane">
      <h2>Terminal</h2>
      <div class="term" bind:this={termEl}></div>
    </section>

    <section class="diff-pane">
      <h2>
        Diff
        <button onclick={loadDiff}>Load</button>
      </h2>
      <pre>{diffText || dryRunOut || "Dry-run output appears here after Dry run."}</pre>
    </section>
  </div>
</main>

<style>
  :global(body) {
    margin: 0;
    font-family: system-ui, sans-serif;
    background: #0f1419;
    color: #e6edf3;
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
    border-bottom: 1px solid #30363d;
  }
  .actions {
    display: flex;
    gap: 0.5rem;
  }
  input {
    min-width: 220px;
    padding: 0.35rem 0.5rem;
    border-radius: 6px;
    border: 1px solid #30363d;
    background: #161b22;
    color: inherit;
  }
  button {
    padding: 0.35rem 0.75rem;
    border-radius: 6px;
    border: 1px solid #30363d;
    background: #21262d;
    color: inherit;
    cursor: pointer;
  }
  button.selected {
    border-color: #58a6ff;
  }
  .layout {
    display: grid;
    grid-template-columns: 260px 1fr 320px;
    flex: 1;
    min-height: 0;
  }
  aside {
    border-right: 1px solid #30363d;
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
  }
  pre {
    flex: 1;
    overflow: auto;
    font-size: 12px;
    background: #161b22;
    padding: 0.75rem;
    border-radius: 8px;
    margin: 0;
  }
  h2 {
    font-size: 0.9rem;
    margin: 0 0 0.5rem;
  }
</style>
