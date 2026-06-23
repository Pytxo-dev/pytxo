<script lang="ts">
  import { onMount, onDestroy } from "svelte";
  import { Terminal } from "@xterm/xterm";
  import { FitAddon } from "@xterm/addon-fit";
  import "@xterm/xterm/css/xterm.css";
  import "./app.css";

  import { ipc } from "./lib/ipc";
  import type { AgentDto, CatalogEntry, CatalogEntryStatus, DomainDto } from "./lib/types";
  import NebulaShell from "./components/shell/NebulaShell.svelte";
  import Header from "./components/shell/Header.svelte";
  import OnboardingBanner from "./components/shell/OnboardingBanner.svelte";
  import UpdateBanner from "./components/shell/UpdateBanner.svelte";
  import { notifyHitlIfNeeded } from "./lib/hitl-notify";
  import SidebarPanel from "./components/panels/SidebarPanel.svelte";
  import LogPanel from "./components/panels/LogPanel.svelte";
  import DiffPanel from "./components/panels/DiffPanel.svelte";
  import TopologyScene3D from "./components/topology/TopologyScene3D.svelte";

  const THEME_STORAGE_KEY = "pytxo-deck-theme";

  let domains: DomainDto[] = $state([]);
  let allDomains: CatalogEntry[] = $state([]);
  let domainStatus: CatalogEntryStatus[] = $state([]);
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
  let projectRoots = $state<Awaited<ReturnType<typeof ipc.projectRoots>>>([]);
  let fleetRuns = $state<Awaited<ReturnType<typeof ipc.listFleetRuns>>>([]);
  let structural = $state<Awaited<ReturnType<typeof ipc.structuralGraph>> | null>(null);

  let cmd = $state("echo pytxo-wave");
  let dispatchRepo = $state("");
  let dryRunOut = $state("");
  let diffText = $state("");
  let signalRetries: string[] = $state([]);
  let cloudRunBadge = $state<"cloud" | "fallback" | null>(null);
  let networkIsolationBadge = $state<string | null>(null);
  let orgPolicyLabel = $state<string | null>(null);
  let walletMicrocredits = $state<number | null>(null);
  let cloudEnabled = $state(false);
  let permissionCeiling = $state<string | null>(null);
  let subscriptionPortalUrl = $state<string | null>(null);
  let tier = $state("core");
  let maxAgents = $state(3);
  let cliMissing = $state(false);
  let signedIn = $state(false);
  let lightTheme = $state(false);
  let logExpanded = $state(true);
  let selectedTopologyNode = $state<string | null>(null);

  const TERM_THEME_DARK = { background: "#020205", foreground: "#e8eaed" };
  const TERM_THEME_LIGHT = { background: "#f4f6f8", foreground: "#0f1419" };

  function applyTheme() {
    if (lightTheme) {
      document.documentElement.classList.remove("dark");
      document.documentElement.setAttribute("data-chroma-theme", "light");
      document.documentElement.style.colorScheme = "light";
      localStorage.setItem(THEME_STORAGE_KEY, "light");
    } else {
      document.documentElement.classList.add("dark");
      document.documentElement.setAttribute("data-chroma-theme", "dark");
      document.documentElement.style.colorScheme = "dark";
      localStorage.setItem(THEME_STORAGE_KEY, "dark");
    }
    applyTerminalTheme();
  }

  function applyTerminalTheme() {
    if (!terminal) return;
    terminal.options.theme = lightTheme ? TERM_THEME_LIGHT : TERM_THEME_DARK;
  }

  function toggleTheme() {
    lightTheme = !lightTheme;
    applyTheme();
  }

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

  const selectedRun = $derived(runs.find((r) => r.id === selectedRunId) ?? null);

  function recordEvent(kind: string, payload: string) {
    if (kind === "signal-retry") {
      signalRetries = [...signalRetries, payload].slice(-20);
    }
    if (kind === "cloud-fallback") {
      cloudRunBadge = "fallback";
    }
    if (kind === "cloud-delta" || kind === "cloud-exec") {
      cloudRunBadge = "cloud";
    }
    if (kind === "network-isolation") {
      networkIsolationBadge = payload.replace(/^deepspace-v2:/, "");
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
    domainStatus = await ipc.listDomainsStatus();
    allDomains = domainStatus.length > 0 ? domainStatus : await ipc.listAllDomains();
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
    cloudRunBadge = null;
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
      structural = null;
      return;
    }
    arbitrage = await ipc.agentArbitrage(selectedRunId, selectedDomainId);
    structural = await ipc.structuralGraph(selectedRunId, selectedDomainId);
  }

  async function refreshProjectRoots() {
    if (!selectedProjectId) {
      projectRoots = [];
      return;
    }
    projectRoots = await ipc.projectRoots(selectedProjectId);
  }

  async function refreshFleetRuns() {
    fleetRuns = await ipc.listFleetRuns(8);
  }

  async function refreshHitl() {
    hitl = await ipc.listHitlAll();
    const actions = hitl.map((h) => h.action ?? h.id);
    await notifyHitlIfNeeded(hitl.length, actions);
  }

  async function respondHitl(id: string, approve: boolean) {
    const row = hitl.find((h) => h.id === id);
    await ipc.hitlRespond(id, approve, row?.domain_id ?? selectedDomainId);
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
    const status = await ipc.entitlementStatus(selectedDomainId);
    tier = status.tier;
    maxAgents = status.max_agents;
    cloudEnabled = status.cloud_enabled;
    walletMicrocredits = status.wallet_balance_microcredits;
    permissionCeiling = status.permission_ceiling;
    subscriptionPortalUrl = status.subscription_portal_url;
    if (status.permission_ceiling) {
      orgPolicyLabel = `org ceiling: ${status.permission_ceiling}`;
    } else {
      orgPolicyLabel = null;
    }
    const auth = await ipc.authStatus();
    signedIn = auth.signed_in;
  }

  onMount(async () => {
    lightTheme = localStorage.getItem(THEME_STORAGE_KEY) === "light";
    applyTheme();

    if (termEl) {
      terminal = new Terminal({
        theme: lightTheme ? TERM_THEME_LIGHT : TERM_THEME_DARK,
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
    await refreshFleetRuns();
    pollTimer = setInterval(pollLogs, 16);
    hitlTimer = setInterval(() => {
      refreshHitl();
      refreshArbitrage();
      refreshProjectRoots();
      refreshFleetRuns();
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
    {cloudRunBadge}
    {walletMicrocredits}
    {permissionCeiling}
    {subscriptionPortalUrl}
    onDryRun={doDryRun}
    onDispatch={doStart}
    onStop={doStop}
    onRefresh={refreshRuns}
    onAuthChange={loadEntitlements}
  />
  <OnboardingBanner visible={cliMissing} />
  <UpdateBanner />
  <div class="theme-bar">
    <button type="button" onclick={toggleTheme}>
      {lightTheme ? "Dark deck" : "Light deck"}
    </button>
  </div>

  <div class="layout">
    <SidebarPanel
      {projects}
      {catalogForView}
      {domainStatus}
      {domains}
      {runs}
      {agents}
      {hitl}
      {arbitrage}
      {projectRoots}
      {fleetRuns}
      {structural}
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
      onProjectRootsChange={(next) => (projectRoots = next)}
      {networkIsolationBadge}
      usageTier={tier}
      usageMaxAgents={maxAgents}
      {walletMicrocredits}
      {cloudEnabled}
      {permissionCeiling}
      {subscriptionPortalUrl}
    />
    <div class="center-column">
      <TopologyScene3D
        {structural}
        {lightTheme}
        bind:selectedNodeId={selectedTopologyNode}
      />
      <div class="log-toolbar">
        <button type="button" onclick={() => (logExpanded = !logExpanded)}>
          {logExpanded ? "Hide telemetry" : "Show telemetry"}
        </button>
      </div>
      {#if logExpanded}
        <LogPanel bind:termEl {signalRetries} {lightTheme} />
      {/if}
    </div>
    <DiffPanel
      {diffText}
      {dryRunOut}
      isolationBackend={selectedRun?.isolation_backend ?? ""}
      onLoad={loadDiff}
      onCommit={doCommit}
    />
  </div>
</NebulaShell>

<style>
  .theme-bar {
    display: flex;
    justify-content: flex-end;
    padding: 0.25rem 0.75rem 0;
  }
  .layout {
    display: flex;
    flex: 1;
    min-height: 0;
    gap: 0;
  }
  .center-column {
    display: flex;
    flex-direction: column;
    flex: 1;
    min-width: 0;
    min-height: 0;
  }
  .log-toolbar {
    display: flex;
    justify-content: flex-end;
    padding: 0.25rem 0.75rem 0;
  }
  .layout :global(.sidebar) {
    width: 280px;
    flex-shrink: 0;
  }
  .layout :global(.topology-3d) {
    flex: 1;
    min-height: 200px;
  }
  .layout :global(.log-pane) {
    flex: 0 0 220px;
    min-width: 0;
  }
</style>
