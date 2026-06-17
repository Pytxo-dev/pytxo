<script lang="ts">
  import { onMount, onDestroy } from "svelte";
  import { Terminal } from "@xterm/xterm";
  import { FitAddon } from "@xterm/addon-fit";
  import "@xterm/xterm/css/xterm.css";
  import "./app.css";

  import { ipc } from "./lib/ipc";
  import type { AgentDto, CatalogEntry, DomainDto } from "./lib/types";
  import NebulaShell from "./components/shell/NebulaShell.svelte";
  import Header from "./components/shell/Header.svelte";
  import OnboardingBanner from "./components/shell/OnboardingBanner.svelte";
  import UpdateBanner from "./components/shell/UpdateBanner.svelte";
  import { notifyHitlIfNeeded } from "./lib/hitl-notify";
  import SidebarPanel from "./components/panels/SidebarPanel.svelte";
  import LogPanel from "./components/panels/LogPanel.svelte";
  import DiffPanel from "./components/panels/DiffPanel.svelte";

  let domains: DomainDto[] = $state([]);
  let allDomains: CatalogEntry[] = $state([]);
  let projects = $state<Awaited<ReturnType<typeof ipc.listProjects>>>([]);
  let selectedProjectId = $state<string | null>(null);
  let rootFilter = $state<string | null>(null);
  let selectedDomainId = $state<string | null>(null);
  let runs = $state<Awaited<ReturnType<typeof ipc.listRuns>>>([]);
  let agents: AgentDto[] = $state([]);
  let selectedRunId = $state<string | null>(null);
  let selectedAgentId = $state<string | null>(null);
  let hitl = $state<Awaited<ReturnType<typeof ipc.listHitl>>>([]);
  let arbitrage = $state<Awaited<ReturnType<typeof ipc.agentArbitrage>>>([]);

  let cmd = $state("echo pytxo-wave");
  let dispatchRepo = $state("");
  let dryRunOut = $state("");
  let diffText = $state("");
  let signalRetries: string[] = $state([]);
  let tier = $state("core");
  let maxAgents = $state(3);
  let cliMissing = $state(false);
  let signedIn = $state(false);

  let termEl: HTMLDivElement | undefined = $state();
  let terminal: Terminal | null = null;
  let fitAddon: FitAddon | null = null;
  let pollTimer: ReturnType<typeof setInterval> | null = null;
  let hitlTimer: ReturnType<typeof setInterval> | null = null;
  const MAX_TERMINAL_LINES = 2000;
  let terminalLineCount = 0;

  let catalogForView = $derived(
    selectedProjectId
      ? allDomains.filter((e) => e.project_id === selectedProjectId)
      : allDomains,
  );

  let filteredAgents = $derived(
    rootFilter ? agents.filter((a) => a.root_id === rootFilter) : agents,
  );

  function recordEvent(kind: string, payload: string) {
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
    domains = await ipc.listDomains();
    allDomains = await ipc.listAllDomains();
    projects = await ipc.listProjects();
    if (!selectedDomainId && domains.length > 0) {
      await selectDomain(domains[0].domain_id);
    }
  }

  async function selectCatalogDomain(entry: CatalogEntry) {
    if (!domains.some((d) => d.domain_id === entry.domain_id)) {
      domains = [...domains, { domain_id: entry.domain_id, repo_root: entry.repo_root }];
    }
    await selectDomain(entry.domain_id);
  }

  async function selectDomain(domainId: string) {
    selectedDomainId = domainId;
    await ipc.selectDomain(domainId);
    selectedRunId = null;
    selectedAgentId = null;
    await refreshRuns();
  }

  async function refreshRuns() {
    runs = await ipc.listRuns(20, selectedDomainId);
    if (!selectedRunId && runs.length > 0) {
      selectedRunId = runs[0].id;
      await selectRun(selectedRunId);
    }
  }

  async function selectRun(runId: string) {
    selectedRunId = runId;
    agents = await ipc.listAgents(runId, selectedDomainId);
    if (agents.length > 0) {
      selectedAgentId = agents[0].id;
      await loadTerminalHistory();
    }
    await refreshArbitrage();
  }

  async function selectAgent(agentId: string) {
    selectedAgentId = agentId;
    await loadTerminalHistory();
  }

  async function loadTerminalHistory() {
    if (!selectedAgentId || !terminal) return;
    terminal.clear();
    const events = await ipc.tailEvents(selectedAgentId, 200, selectedDomainId);
    terminalLineCount = 0;
    signalRetries = [];
    for (const ev of events) {
      writelnCapped(`[${ev.kind}] ${ev.payload}`);
      recordEvent(ev.kind, ev.payload);
    }
  }

  async function pollLogs() {
    if (!selectedAgentId || !terminal) return;
    const lines = await ipc.pollLogLines(selectedAgentId, 32, selectedDomainId);
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
    arbitrage = await ipc.agentArbitrage(selectedRunId, selectedDomainId);
  }

  async function refreshHitl() {
    if (!selectedDomainId) {
      hitl = [];
      return;
    }
    hitl = await ipc.listHitl(selectedDomainId);
    const actions = hitl.map((h) => h.action ?? h.id);
    await notifyHitlIfNeeded(hitl.length, actions);
  }

  async function respondHitl(id: string, approve: boolean) {
    await ipc.hitlRespond(id, approve, selectedDomainId);
    await refreshHitl();
  }

  async function doDryRun() {
    dryRunOut = await ipc.dryRun(3, selectedDomainId);
  }

  async function doStart() {
    const repoRoot = dispatchRepo.trim() || undefined;
    const runId = await ipc.dispatchRun(cmd, 3, repoRoot);
    await refreshDomains();
    await refreshRuns();
    selectedRunId = runId;
    await selectRun(runId);
  }

  async function doStop() {
    await ipc.stopRun(false);
    await refreshRuns();
  }

  async function loadDiff() {
    if (!selectedAgentId) return;
    try {
      diffText = await ipc.gitDiff(selectedAgentId, selectedDomainId);
    } catch (e) {
      diffText = String(e);
    }
  }

  async function doCommit() {
    if (!selectedRunId || !selectedAgentId) return;
    const parts = selectedAgentId.split(":");
    const agentOnly = parts.length > 1 ? parts[1] : selectedAgentId;
    await ipc.commitWorkspace(selectedRunId, agentOnly, selectedDomainId);
    await loadDiff();
  }

  async function loadEntitlements() {
    const status = await ipc.entitlementStatus();
    tier = status.tier;
    maxAgents = status.max_agents;
    const auth = await ipc.authStatus();
    signedIn = auth.signed_in;
  }

  onMount(async () => {
    document.documentElement.classList.add("dark");
    document.documentElement.setAttribute("data-chroma-theme", "dark");

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

    cliMissing = !(await ipc.checkPytxoCli());
    await loadEntitlements();
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

<NebulaShell>
  <Header
    bind:cmd
    bind:dispatchRepo
    {tier}
    {maxAgents}
    {signedIn}
    onDryRun={doDryRun}
    onDispatch={doStart}
    onStop={doStop}
    onRefresh={refreshRuns}
    onAuthChange={loadEntitlements}
  />
  <OnboardingBanner visible={cliMissing} />
  <UpdateBanner />

  <div class="layout">
    <SidebarPanel
      {projects}
      {catalogForView}
      {domains}
      {runs}
      {agents}
      {hitl}
      {arbitrage}
      bind:selectedProjectId
      bind:selectedDomainId
      bind:selectedRunId
      bind:selectedAgentId
      bind:rootFilter
      {filteredAgents}
      onSelectCatalogDomain={selectCatalogDomain}
      onSelectDomain={selectDomain}
      onSelectRun={selectRun}
      onSelectAgent={selectAgent}
      onRespondHitl={respondHitl}
    />
    <LogPanel bind:termEl {signalRetries} />
    <DiffPanel {diffText} {dryRunOut} onLoad={loadDiff} onCommit={doCommit} />
  </div>
</NebulaShell>

<style>
  .layout {
    display: flex;
    flex: 1;
    min-height: 0;
    gap: 0;
  }
  .layout :global(.sidebar) {
    width: 280px;
    flex-shrink: 0;
  }
  .layout :global(.log-pane) {
    flex: 1;
    min-width: 0;
  }
</style>
