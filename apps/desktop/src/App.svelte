<script lang="ts">
  import { onMount, onDestroy } from "svelte";
  import "./app.css";

  import { ipc, onAuthChanged, onAuthError } from "./lib/ipc";
  import {
    applyDeckTheme,
    initThemeChrome,
    isSetupComplete,
    loadTheme,
    resetOnboarding,
    terminalThemeFor,
    type DeckTheme,
  } from "./lib/theme";
  import {
    hydrateTabsFromStorage,
    persistTabs,
    tabLabelFromPath,
    type WorkspaceTab,
  } from "./lib/deck-store.svelte";
  import {
    loadWorkspaceCatalog,
    openFolderAsWorkspace,
    openProjectAsWorkspace,
    type WorkspaceListItem,
  } from "./lib/workspace";
  import { addWorkspaceRecent } from "./lib/navigation.svelte";
  import DeckShell from "./components/shell/DeckShell.svelte";
  import ProjectTabs from "./components/shell/ProjectTabs.svelte";
  import SetupWizard from "./components/setup/SetupWizard.svelte";
  import WorkspaceHome from "./components/workspace/WorkspaceHome.svelte";
  import DesktopShell from "./components/desktop2/DesktopShell.svelte";
  import FlowStandalone from "./components/desktop2/FlowStandalone.svelte";
  import { isTauriRuntime } from "./lib/desktop-backend";

  const isFlowStandalone =
    typeof window !== "undefined" &&
    (window.location.hash.includes("flow-standalone") ||
      window.location.hash === "#/flow-standalone");

  let showSetup = $state(!isSetupComplete());
  let deckTheme = $state<DeckTheme>("void");
  let showHome = $state(false);

  let tabs = $state<WorkspaceTab[]>([]);
  let activeTabId = $state<string | null>(null);

  let hitl = $state<Awaited<ReturnType<typeof ipc.listHitl>>>([]);
  let fleetRuns = $state<Awaited<ReturnType<typeof ipc.listFleetRuns>>>([]);
  let topologyLoading = $state(false);

  let cmd = $state("");
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
  let authErrorMessage = $state<string | null>(null);
  let logExpanded = $state(false);
  let selectedTopologyNode = $state<string | null>(null);
  let showWorkspaceSettings = $state(false);
  const useLegacyShell = localStorage.getItem("desktop_shell_v1") === "true";

  let termEl: HTMLDivElement | undefined = $state();
  let terminal: import("@xterm/xterm").Terminal | null = null;
  let fitAddon: import("@xterm/addon-fit").FitAddon | null = null;
  let TerminalCtor: typeof import("@xterm/xterm").Terminal | null = null;
  let FitAddonCtor: typeof import("@xterm/addon-fit").FitAddon | null = null;
  let pollTimer: ReturnType<typeof setInterval> | null = null;
  let hitlTimer: ReturnType<typeof setInterval> | null = null;
  let structuralTimer: ReturnType<typeof setInterval> | null = null;
  let pollLogsInflight = false;
  let authUnlisten: (() => void) | null = null;
  let authErrorUnlisten: (() => void) | null = null;
  const MAX_TERMINAL_LINES = 2000;
  let terminalLineCount = 0;

  const activeTab = $derived(tabs.find((t) => t.id === activeTabId) ?? null);
  const onHome = $derived(!showSetup && (showHome || !activeTabId || tabs.length === 0));
  const runIsActive = $derived(
    !!activeTab?.selectedRunId &&
      ["running", "pending", "dispatching", "active"].includes(
        (activeTab.runs.find((r) => r.id === activeTab.selectedRunId)?.status ?? "").toLowerCase(),
      ),
  );

  $effect(() => {
    applyDeckTheme(deckTheme);
    applyTerminalTheme();
  });

  $effect(() => {
    if (useLegacyShell) persistTabs(tabs, activeTabId);
  });

  /** Terminal mounts with DeckWorkspace; recreate when the xterm host appears. */
  $effect(() => {
    if (!useLegacyShell) return;
    if (!termEl) {
      if (terminal) {
        terminal.dispose();
        terminal = null;
        fitAddon = null;
        terminalLineCount = 0;
      }
      return;
    }
    if (terminal) return;
    let cancelled = false;
    void (async () => {
      if (!TerminalCtor || !FitAddonCtor) {
        const [xterm, fit] = await Promise.all([
          import("@xterm/xterm"),
          import("@xterm/addon-fit"),
        ]);
        // @ts-expect-error Vite resolves CSS side-effect imports at build time.
        await import("@xterm/xterm/css/xterm.css");
        TerminalCtor = xterm.Terminal;
        FitAddonCtor = fit.FitAddon;
      }
      if (cancelled || !termEl || terminal || !TerminalCtor || !FitAddonCtor) return;
      terminal = new TerminalCtor({
        theme: terminalThemeFor(deckTheme),
        fontSize: 13,
        convertEol: true,
      });
      fitAddon = new FitAddonCtor();
      terminal.loadAddon(fitAddon);
      terminal.open(termEl);
      fitAddon.fit();
    })();
    return () => {
      cancelled = true;
    };
  });

  async function switchTab(tabId: string) {
    showHome = false;
    activeTabId = tabId;
    await activateTab(tabId);
  }

  function applyTerminalTheme() {
    if (!terminal) return;
    terminal.options.theme = terminalThemeFor(deckTheme);
  }

  function updateTab(tabId: string, patch: Partial<WorkspaceTab>) {
    tabs = tabs.map((t) => (t.id === tabId ? { ...t, ...patch } : t));
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

  async function addWorkspaceTab(path: string) {
    const result = await openFolderAsWorkspace(path, tabs);
    tabs = result.tabs;
    await switchTab(result.tabId);
  }

  /**
   * Onboarding's workspace step is shell-agnostic, but "open a workspace"
   * means different things per shell: the legacy shell tracks open tabs,
   * while Desktop 2 has no tab concept and instead just needs the domain
   * registered and recorded as a sidebar recent.
   */
  async function onSetupWorkspaceSelected(path: string) {
    if (useLegacyShell) {
      await addWorkspaceTab(path);
      return;
    }
    if (!isTauriRuntime()) {
      addWorkspaceRecent({
        id: path,
        label: path.split(/[\\/]/).pop() ?? path,
        domainId: path,
      });
      return;
    }
    const domainId = await ipc.ensureWorkspace(path);
    const profile =
      (typeof localStorage !== "undefined" && localStorage.getItem("pytxo-default-permission-profile-v1")) ||
      "orbit";
    try {
      const trusted = await ipc.domainIsTrusted(domainId);
      if (!trusted) await ipc.setDomainPermission(domainId, profile);
    } catch {
      /* Trust is best-effort during onboarding; Workspaces can retry. */
    }
    addWorkspaceRecent({ id: domainId, label: domainId.split(/[\\/]/).pop() ?? domainId, domainId });
  }

  function replayOnboarding() {
    resetOnboarding();
    showSetup = true;
  }

  async function openCatalogItem(item: WorkspaceListItem) {
    if (item.kind === "project") {
      const result = await openProjectAsWorkspace(
        { id: item.id, manifest_path: item.path },
        tabs,
      );
      tabs = result.tabs;
      await switchTab(result.tabId);
      return;
    }
    await addWorkspaceTab(item.path);
  }

  async function activateTab(tabId: string) {
    const tab = tabs.find((t) => t.id === tabId);
    if (!tab) return;
    await ipc.selectDomain(tab.domainId);
    if (tab.projectId) {
      try {
        const roots = await ipc.projectRoots(tab.projectId);
        updateTab(tabId, { roots });
      } catch {
        /* project may be gone */
      }
    }
    await refreshTabData(tabId);
    await loadEntitlements();
    if (tab.selectedAgentId) {
      await loadTerminalHistory(tabId);
    }
  }

  async function refreshTabData(tabId: string, opts: { includeStructural?: boolean } = {}) {
    const includeStructural = opts.includeStructural !== false;
    const tab = tabs.find((t) => t.id === tabId);
    if (!tab) return;
    const runs = await ipc.listRuns(20, tab.domainId);
    let agents = tab.agents;
    let selectedRunId = tab.selectedRunId;
    let selectedAgentId = tab.selectedAgentId;

    if (!selectedRunId && runs.length > 0) {
      selectedRunId = runs[0].id;
    }
    if (selectedRunId) {
      agents = await ipc.listAgents(selectedRunId, tab.domainId);
      if (!selectedAgentId && agents.length > 0) {
        selectedAgentId = agents[0].id;
      }
    } else {
      agents = [];
      selectedAgentId = null;
    }

    if (!includeStructural) {
      updateTab(tabId, { runs, agents, selectedRunId, selectedAgentId });
      return;
    }

    let structural = null;
    topologyLoading = tabId === activeTabId;
    try {
      if (selectedRunId) {
        structural = await ipc.structuralGraph(selectedRunId, tab.domainId);
      } else {
        structural = await ipc.workspaceStructuralGraph(tab.domainId);
      }
    } finally {
      if (tabId === activeTabId) topologyLoading = false;
    }

    updateTab(tabId, { runs, agents, selectedRunId, selectedAgentId, structural });
  }

  async function selectRun(tabId: string, runId: string) {
    const tab = tabs.find((t) => t.id === tabId);
    if (!tab) return;
    cloudRunBadge = null;
    const agents = await ipc.listAgents(runId, tab.domainId);
    const selectedAgentId = agents.length > 0 ? agents[0].id : null;
    updateTab(tabId, { selectedRunId: runId, selectedAgentId, agents });
    if (tabId === activeTabId && selectedAgentId) {
      await loadTerminalHistory(tabId);
    }
    if (tabId === activeTabId) {
      await refreshTabData(tabId);
    }
  }

  async function selectAgent(tabId: string, agentId: string) {
    updateTab(tabId, { selectedAgentId: agentId });
    if (tabId === activeTabId) {
      await loadTerminalHistory(tabId);
    }
  }

  async function loadTerminalHistory(tabId: string) {
    const tab = tabs.find((t) => t.id === tabId);
    if (!tab?.selectedAgentId || !terminal) return;
    terminal.clear();
    const events = await ipc.tailEvents(tab.selectedAgentId, 200, tab.domainId);
    terminalLineCount = 0;
    signalRetries = [];
    for (const ev of events) {
      writelnCapped(`[${ev.kind}] ${ev.payload}`);
      recordEvent(ev.kind, ev.payload);
    }
  }

  async function pollLogs() {
    if (pollLogsInflight || !activeTab?.selectedAgentId || !terminal) return;
    pollLogsInflight = true;
    try {
      const lines = await ipc.pollLogLines(activeTab.selectedAgentId, 32, activeTab.domainId);
      for (const ev of lines) {
        writelnCapped(`[${ev.kind}] ${ev.payload}`);
        recordEvent(ev.kind, ev.payload);
      }
    } finally {
      pollLogsInflight = false;
    }
  }

  async function refreshActiveTab() {
    if (activeTabId) await refreshTabData(activeTabId);
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
    await ipc.hitlRespond(id, approve, row?.domain_id ?? activeTab?.domainId ?? null);
    await refreshHitl();
  }

  async function doDryRun() {
    if (!activeTab) return;
    dryRunOut = await ipc.dryRun(3, activeTab.domainId ?? null);
  }

  async function doStart() {
    if (!activeTab) return;
    const command = cmd.trim();
    if (!command) return;
    const repoRoot = activeTab.dispatchRepo.trim() || activeTab.domainId || undefined;
    const runId = await ipc.dispatchRun(command, 3, repoRoot);
    await refreshTabData(activeTab.id);
    await selectRun(activeTab.id, runId);
  }

  async function doStop() {
    await ipc.stopRun(false);
    if (activeTabId) await refreshTabData(activeTabId);
  }

  async function loadDiff() {
    if (!activeTab?.selectedAgentId) return;
    try {
      diffText = await ipc.gitDiff(activeTab.selectedAgentId, activeTab.domainId);
    } catch (e) {
      diffText = String(e);
    }
  }

  async function doCommit() {
    if (!activeTab?.selectedRunId || !activeTab.selectedAgentId) return;
    const parts = activeTab.selectedAgentId.split(":");
    const agentOnly = parts.length > 1 ? parts[1] : activeTab.selectedAgentId;
    await ipc.commitWorkspace(activeTab.selectedRunId, agentOnly, activeTab.domainId);
    await loadDiff();
  }

  async function loadEntitlements() {
    try {
      const auth = await ipc.authStatus();
      signedIn = auth.signed_in;
    } catch {
      signedIn = false;
    }
    try {
      const status = await ipc.entitlementStatus(activeTab?.domainId ?? null);
      tier = status.tier;
      maxAgents = status.max_agents;
      walletMicrocredits = status.wallet_balance_microcredits;
      permissionCeiling = status.permission_ceiling;
      subscriptionPortalUrl = status.subscription_portal_url;
    } catch {
      /* Link/config errors must not block signed-in UI from updating. */
    }
  }

  async function initDashboard() {
    cliMissing = !(await ipc.checkPytxoCli());
    const hydrated = hydrateTabsFromStorage();
    if (hydrated.tabs.length > 0) {
      tabs = hydrated.tabs;
      const targetId =
        hydrated.activeTabId && hydrated.tabs.some((t) => t.id === hydrated.activeTabId)
          ? hydrated.activeTabId
          : hydrated.tabs[0].id;
      for (const tab of tabs) {
        try {
          const canonical = await ipc.ensureWorkspace(tab.domainId);
          let roots = tab.roots;
          if (tab.projectId) {
            try {
              roots = await ipc.projectRoots(tab.projectId);
            } catch {
              /* ignore */
            }
          }
          updateTab(tab.id, {
            domainId: canonical,
            label: tab.label || tabLabelFromPath(canonical),
            roots,
          });
        } catch {
          /* stale tab path */
        }
      }
      await switchTab(targetId);
    } else {
      showHome = true;
      // Warm catalog so Home is ready
      void loadWorkspaceCatalog();
    }
    await refreshFleetRuns();
  }

  async function openFolderTab() {
    const path = await ipc.pickWorkspaceFolder();
    if (path) await addWorkspaceTab(path);
  }

  function goHome() {
    showHome = true;
    activeTabId = null;
  }

  function closeTab(tabId: string) {
    const idx = tabs.findIndex((t) => t.id === tabId);
    if (idx < 0) return;
    const next = tabs.filter((t) => t.id !== tabId);
    tabs = next;
    if (activeTabId === tabId) {
      const fallback = next[Math.min(idx, next.length - 1)];
      if (fallback) {
        void switchTab(fallback.id);
      } else {
        activeTabId = null;
        showHome = true;
      }
    }
  }

  function onRootsChange(roots: import("./lib/types").ProjectRootDto[]) {
    if (!activeTabId) return;
    updateTab(activeTabId, { roots });
  }

  function finishSetup(openGuidedFlow = false) {
    if (openGuidedFlow && !useLegacyShell) {
      // DesktopShell consumes this on first mount and opens the dedicated
      // planner. A skipped setup still lands on the operational home.
      localStorage.setItem("pytxo-desktop-route-v2", "flow");
    }
    showSetup = false;
    if (useLegacyShell) initDashboard();
  }

  onMount(async () => {
    deckTheme = loadTheme();
    initThemeChrome();
    applyDeckTheme(deckTheme);

    authUnlisten = await onAuthChanged(() => {
      authErrorMessage = null;
      loadEntitlements();
    });
    authErrorUnlisten = await onAuthError((message) => {
      authErrorMessage = message;
    });
    // Desktop 2 shows real account/entitlement state in its sidebar and
    // Settings → Account & billing even though it has no active workspace tab.
    void loadEntitlements();

    if (!useLegacyShell) return;

    if (!showSetup) {
      await initDashboard();
    }

    pollTimer = setInterval(pollLogs, 250);
    hitlTimer = setInterval(() => {
      refreshHitl();
      if (activeTabId && !showHome) void refreshTabData(activeTabId, { includeStructural: false });
      refreshFleetRuns();
    }, 1000);
    structuralTimer = setInterval(() => {
      if (activeTabId && !showHome) void refreshTabData(activeTabId, { includeStructural: true });
    }, 8000);
  });

  onDestroy(() => {
    if (pollTimer) clearInterval(pollTimer);
    if (hitlTimer) clearInterval(hitlTimer);
    if (structuralTimer) clearInterval(structuralTimer);
    authUnlisten?.();
    authErrorUnlisten?.();
    terminal?.dispose();
  });
</script>

{#if isFlowStandalone}
  <FlowStandalone />
{:else}
<DeckShell>
  {#if showSetup}
    <SetupWizard onComplete={finishSetup} onWorkspaceSelected={onSetupWorkspaceSelected} />
  {:else if useLegacyShell}
    <ProjectTabs
      {tabs}
      {activeTabId}
      homeActive={onHome}
      onAddTab={goHome}
      onCloseTab={closeTab}
      onSwitchTab={switchTab}
      onGoHome={goHome}
    />
    {#if onHome}
      <WorkspaceHome onOpenFolder={openFolderTab} onOpenItem={openCatalogItem} />
    {:else}
      {#await import("./components/dashboard/DeckWorkspace.svelte") then { default: DeckWorkspace }}
      <DeckWorkspace
        bind:deckTheme
        bind:cmd
        bind:tabs
        activeTabId={activeTabId}
        bind:termEl
        bind:selectedTopologyNode
        bind:logExpanded
        bind:showWorkspaceSettings
        running={runIsActive}
        {hitl}
        {topologyLoading}
        {fleetRuns}
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
        onSelectRun={selectRun}
        onSelectAgent={selectAgent}
        onDryRun={doDryRun}
        onDispatch={doStart}
        onStop={doStop}
        onRefresh={refreshActiveTab}
        onAuthChange={loadEntitlements}
        onLoadDiff={loadDiff}
        onCommit={doCommit}
        onRespondHitl={respondHitl}
        onRootsChange={onRootsChange}
        onGoHome={goHome}
      />
      {/await}
    {/if}
  {:else}
    <DesktopShell {tier} {signedIn} {cliMissing} {subscriptionPortalUrl} authErrorMessage={authErrorMessage} onReplayOnboarding={replayOnboarding} onAuthChange={loadEntitlements} />
  {/if}
</DeckShell>
{/if}
