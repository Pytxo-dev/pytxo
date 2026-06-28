<script lang="ts">
  import { onMount, onDestroy } from "svelte";
  import { Terminal } from "@xterm/xterm";
  import { FitAddon } from "@xterm/addon-fit";
  import "@xterm/xterm/css/xterm.css";
  import "./app.css";

  import { ipc, onAuthChanged } from "./lib/ipc";
  import type { AgentDto, DomainDto } from "./lib/types";
  import {
    applyDeckTheme,
    isSetupComplete,
    loadTheme,
    terminalThemeFor,
    type DeckTheme,
  } from "./lib/theme";
  import NebulaShell from "./components/shell/NebulaShell.svelte";
  import SetupWizard from "./components/setup/SetupWizard.svelte";
  import AgentDashboard from "./components/dashboard/AgentDashboard.svelte";

  let showSetup = $state(!isSetupComplete());
  let deckTheme = $state<DeckTheme>("void");

  let domains: DomainDto[] = $state([]);
  let selectedDomainId = $state<string | null>(null);
  let workspacePath = $state("");
  let runs = $state<Awaited<ReturnType<typeof ipc.listRuns>>>([]);
  let agents: AgentDto[] = $state([]);
  let selectedRunId = $state<string | null>(null);
  let selectedAgentId = $state<string | null>(null);
  let hitl = $state<Awaited<ReturnType<typeof ipc.listHitl>>>([]);
  let arbitrage = $state<Awaited<ReturnType<typeof ipc.agentArbitrage>>>([]);
  let fleetRuns = $state<Awaited<ReturnType<typeof ipc.listFleetRuns>>>([]);
  let structural = $state<Awaited<ReturnType<typeof ipc.structuralGraph>> | null>(null);
  let topologyLoading = $state(false);

  let cmd = $state("echo pytxo-wave");
  let dispatchRepo = $state("");
  let dryRunOut = $state("");
  let diffText = $state("");
  let signalRetries: string[] = $state([]);
  let cloudRunBadge = $state<"cloud" | "fallback" | null>(null);
  let networkIsolationBadge = $state<string | null>(null);
  let walletMicrocredits = $state<number | null>(null);
  let permissionCeiling = $state<string | null>(null);
  let subscriptionPortalUrl = $state<string | null>(null);
  let tier = $state("core");
  let maxAgents = $state(3);
  let cliMissing = $state(false);
  let signedIn = $state(false);
  let logExpanded = $state(false);
  let selectedTopologyNode = $state<string | null>(null);

  let termEl: HTMLDivElement | undefined = $state();
  let terminal: Terminal | null = null;
  let fitAddon: FitAddon | null = null;
  let pollTimer: ReturnType<typeof setInterval> | null = null;
  let hitlTimer: ReturnType<typeof setInterval> | null = null;
  let authUnlisten: (() => void) | null = null;
  const MAX_TERMINAL_LINES = 2000;
  let terminalLineCount = 0;

  const selectedRun = $derived(runs.find((r) => r.id === selectedRunId) ?? null);

  const workspaceLabel = $derived(
    workspacePath
      ? workspacePath.split(/[/\\]/).pop() ?? workspacePath
      : selectedDomainId?.split(/[/\\]/).pop() ?? "",
  );

  $effect(() => {
    applyDeckTheme(deckTheme);
    applyTerminalTheme();
  });

  function applyTerminalTheme() {
    if (!terminal) return;
    terminal.options.theme = terminalThemeFor(deckTheme);
  }

  function recordEvent(kind: string, payload: string) {
    if (kind === "signal-retry") {
      signalRetries = [...signalRetries, payload].slice(-20);
    }
    if (kind === "cloud-fallback") cloudRunBadge = "fallback";
    if (kind === "cloud-delta" || kind === "cloud-exec") cloudRunBadge = "cloud";
    if (kind === "network-isolation") {
      networkIsolationBadge = payload.replace(/^deepspace-v2:/, "");
    }
  }

  function writelnCapped(line: string) {
    if (!terminal) return;
    terminal.writeln(line);
    terminalLineCount += 1;
    if (terminalLineCount > MAX_TERMINAL_LINES) {
      terminal.clear();
      terminalLineCount = 0;
    }
  }

  async function registerWorkspace(path: string) {
    workspacePath = path;
    const domainId = path;
    if (!domains.some((d) => d.domain_id === domainId)) {
      domains = [...domains, { domain_id: domainId, repo_root: path }];
    }
    await selectDomain(domainId);
  }

  async function refreshDomains() {
    domains = await ipc.listDomains();
    if (!selectedDomainId && domains.length > 0) {
      await selectDomain(domains[0].domain_id);
    } else if (selectedDomainId) {
      await refreshRuns();
      await refreshTopology();
    }
  }

  async function selectDomain(domainId: string) {
    selectedDomainId = domainId;
    workspacePath = domainId;
    await ipc.selectDomain(domainId);
    selectedRunId = null;
    selectedAgentId = null;
    await refreshRuns();
    await refreshTopology();
    await loadEntitlements();
  }

  async function refreshRuns() {
    runs = await ipc.listRuns(20, selectedDomainId);
    if (!selectedRunId && runs.length > 0) {
      selectedRunId = runs[0].id;
      await selectRun(selectedRunId);
    } else if (!selectedRunId) {
      agents = [];
      await refreshTopology();
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
    await refreshTopology();
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

  async function refreshTopology() {
    if (!selectedDomainId) {
      structural = null;
      return;
    }
    topologyLoading = true;
    try {
      if (selectedRunId) {
        arbitrage = await ipc.agentArbitrage(selectedRunId, selectedDomainId);
        structural = await ipc.structuralGraph(selectedRunId, selectedDomainId);
      } else {
        arbitrage = [];
        structural = await ipc.workspaceStructuralGraph(selectedDomainId);
      }
    } finally {
      topologyLoading = false;
    }
  }

  async function refreshFleetRuns() {
    fleetRuns = await ipc.listFleetRuns(8);
  }

  async function refreshHitl() {
    hitl = await ipc.listHitlAll();
    const actions = hitl.map((h) => h.action ?? h.id);
    const { notifyHitlIfNeeded } = await import("./lib/hitl-notify");
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
    const repoRoot = dispatchRepo.trim() || workspacePath || undefined;
    const runId = await ipc.dispatchRun(cmd, 3, repoRoot);
    await refreshDomains();
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
    walletMicrocredits = status.wallet_balance_microcredits;
    permissionCeiling = status.permission_ceiling;
    subscriptionPortalUrl = status.subscription_portal_url;
    const auth = await ipc.authStatus();
    signedIn = auth.signed_in;
  }

  async function initDashboard() {
    cliMissing = !(await ipc.checkPytxoCli());
    await loadEntitlements();
    await refreshDomains();
    await refreshFleetRuns();
  }

  function finishSetup() {
    showSetup = false;
    initDashboard();
  }

  onMount(async () => {
    deckTheme = loadTheme();
    applyDeckTheme(deckTheme);

    if (termEl) {
      terminal = new Terminal({
        theme: terminalThemeFor(deckTheme),
        fontSize: 13,
        convertEol: true,
      });
      fitAddon = new FitAddon();
      terminal.loadAddon(fitAddon);
      terminal.open(termEl);
      fitAddon.fit();
    }

    authUnlisten = await onAuthChanged(() => {
      loadEntitlements();
    });

    if (!showSetup) {
      await initDashboard();
    }

    pollTimer = setInterval(pollLogs, 16);
    hitlTimer = setInterval(() => {
      refreshHitl();
      if (selectedRunId) refreshTopology();
      refreshFleetRuns();
    }, 1000);
  });

  onDestroy(() => {
    if (pollTimer) clearInterval(pollTimer);
    if (hitlTimer) clearInterval(hitlTimer);
    authUnlisten?.();
    terminal?.dispose();
  });
</script>

<NebulaShell>
  {#if showSetup}
    <SetupWizard onComplete={finishSetup} onWorkspaceSelected={registerWorkspace} />
  {:else}
    <AgentDashboard
      bind:deckTheme
      bind:cmd
      bind:dispatchRepo
      bind:termEl
      bind:selectedDomainId
      bind:selectedRunId
      bind:selectedAgentId
      bind:selectedTopologyNode
      bind:logExpanded
      {domains}
      {runs}
      {agents}
      {hitl}
      {structural}
      {topologyLoading}
      {fleetRuns}
      {workspaceLabel}
      workspacePath={workspacePath}
      {diffText}
      {dryRunOut}
      {signalRetries}
      {tier}
      {maxAgents}
      {signedIn}
      {cliMissing}
      {cloudRunBadge}
      {walletMicrocredits}
      {permissionCeiling}
      {subscriptionPortalUrl}
      {networkIsolationBadge}
      selectedRun={selectedRun}
      onSelectRun={selectRun}
      onSelectAgent={selectAgent}
      onChangeWorkspace={registerWorkspace}
      onDryRun={doDryRun}
      onDispatch={doStart}
      onStop={doStop}
      onRefresh={refreshRuns}
      onAuthChange={loadEntitlements}
      onLoadDiff={loadDiff}
      onCommit={doCommit}
      onRespondHitl={respondHitl}
    />
  {/if}
</NebulaShell>
