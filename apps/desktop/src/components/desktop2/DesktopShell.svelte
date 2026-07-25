<script lang="ts">
  import { onMount } from "svelte";
  import { IconActivity, IconChecks, IconFolders, IconPlayerPlay, IconPlugConnected, IconSettings, IconSparkles } from "@tabler/icons-svelte";
  import { createDesktopBackend, type DesktopSnapshot } from "../../lib/desktop-backend";
  import {
    addWorkspaceRecent,
    initialRoute,
    migrateWorkspaceRecents,
    persistRoute,
    removeWorkspaceRecent,
    routeFromDeepLink,
    type AppRoute,
    type SettingsSectionId,
    type WorkspaceRecent,
  } from "../../lib/navigation.svelte";
  import { ipc, onPytxoDeepLink } from "../../lib/ipc";
  import Sidebar from "./Sidebar.svelte";
  import AppBar from "./AppBar.svelte";
  import CommandPalette from "./CommandPalette.svelte";
  import OperationsScreen from "./OperationsScreen.svelte";
  import FlowScreen from "./FlowScreen.svelte";
  import CollectionScreen from "./CollectionScreen.svelte";
  import FocusScreen from "./FocusScreen.svelte";
  import SettingsScreen from "./SettingsScreen.svelte";
  import WorkspaceSettingsPanel from "./WorkspaceSettingsPanel.svelte";
  import UpdateBanner from "../shell/UpdateBanner.svelte";
  import "./desktop2-shared.css";

  const SNAPSHOT_POLL_ACTIVE_MS = 2500;
  const SNAPSHOT_POLL_IDLE_MS = 5000;
  const SNAPSHOT_POLL_HIDDEN_MS = 15000;
  const OPEN_BEHAVIOR_KEY = "pytxo-workspace-open-behavior-v1";
  const DEFAULT_PROFILE_KEY = "pytxo-default-permission-profile-v1";

  let {
    routeOverride = null,
    previewState = "default",
    collapsedOverride = null,
    tier = "core",
    signedIn = false,
    cliMissing = false,
    subscriptionPortalUrl = null,
    onReplayOnboarding = () => {},
    onAuthChange = () => {},
    authErrorMessage = null,
  }: {
    routeOverride?: AppRoute | null;
    previewState?: "default" | "loading" | "empty" | "offline" | "error";
    collapsedOverride?: boolean | null;
    tier?: string;
    signedIn?: boolean;
    cliMissing?: boolean;
    subscriptionPortalUrl?: string | null;
    onReplayOnboarding?: () => void;
    onAuthChange?: () => void;
    authErrorMessage?: string | null;
  } = $props();

  const SIDEBAR_KEY = "pytxo-desktop-sidebar-collapsed-v1";
  function readSidebarCollapsed(): boolean {
    if (collapsedOverride !== null) return collapsedOverride;
    if (typeof localStorage === "undefined") return false;
    return localStorage.getItem(SIDEBAR_KEY) === "true";
  }

  const backend = createDesktopBackend();
  let route = $state<AppRoute>("operations");
  let snapshot = $state<DesktopSnapshot>({ domains: [], runs: [], agents: [], approvals: [], fleets: [], error: null });
  let snapshotFingerprint = $state("");
  let loading = $state(true);
  let commandOpen = $state(false);
  let loadMessage = $state("");
  let workspaceMessage = $state("");
  let workspaceError = $state("");
  let recents = $state<WorkspaceRecent[]>([]);
  let collapsed = $state(readSidebarCollapsed());
  let focusRunId = $state<string | null>(null);
  let focusDomainId = $state<string | null>(null);
  let activeDomainId = $state<string | null>(null);
  let lastPollAt = $state<number | null>(null);
  let pollLive = $state(false);
  let settingsSection = $state<SettingsSectionId | null>(null);
  let editingWorkspaceId = $state<string | null>(null);
  let voiceModelPath = $state<string | null>(null);
  let voiceInstalling = $state(false);
  let windowFocused = $state(true);

  const activeDomain = $derived(
    snapshot.domains.find((d) => d.domain_id === activeDomainId) ?? null,
  );
  const editingDomain = $derived(
    snapshot.domains.find((d) => d.domain_id === editingWorkspaceId) ?? null,
  );

  const primary = [
    { route: "operations" as const, label: "Operations", icon: IconActivity },
    { route: "workspaces" as const, label: "Workspaces", icon: IconFolders },
    { route: "runs" as const, label: "Runs", icon: IconPlayerPlay },
    { route: "flow" as const, label: "Flow", icon: IconSparkles },
    { route: "approvals" as const, label: "Approvals", icon: IconChecks },
  ];
  const system = [
    { route: "integrations" as const, label: "Integrations", icon: IconPlugConnected },
    { route: "settings" as const, label: "Settings", icon: IconSettings },
  ];
  const commandItems = [...primary, ...system];

  const focusedRun = $derived(snapshot.runs.find((r) => r.id === focusRunId) ?? null);
  const hasActiveRuns = $derived(
    snapshot.runs.some((r) =>
      ["running", "pending", "dispatching", "active"].includes(r.status.toLowerCase()),
    ),
  );

  function fingerprintSnapshot(snap: DesktopSnapshot): string {
    const domainIds = snap.domains.map((d) => d.domain_id).join(",");
    const runSig = snap.runs.map((r) => `${r.id}:${r.status}`).join(",");
    const agentSig = snap.agents.map((a) => a.id).join(",");
    const approvalSig = snap.approvals.map((a) => a.id).join(",");
    const fleetSig = snap.fleets.map((f) => `${f.id}:${f.status}`).join(",");
    const err = snap.error?.message ?? "";
    return `${domainIds}|${runSig}|${agentSig}|${approvalSig}|${fleetSig}|${err}`;
  }

  function defaultPermissionProfile(): string {
    if (typeof localStorage === "undefined") return "orbit";
    const raw = localStorage.getItem(DEFAULT_PROFILE_KEY);
    if (raw === "deep_space" || raw === "orbit" || raw === "galaxy" || raw === "supernova") return raw;
    return "orbit";
  }

  function profileLabel(profile: string): string {
    switch (profile) {
      case "deep_space":
        return "DeepSpace";
      case "galaxy":
        return "Galaxy";
      case "supernova":
        return "Supernova";
      default:
        return "Orbit";
    }
  }

  async function trustOpenedWorkspace(canonicalPath: string) {
    const profile = defaultPermissionProfile();
    try {
      const trusted = await ipc.domainIsTrusted(canonicalPath);
      if (!trusted) {
        await ipc.setDomainPermission(canonicalPath, profile);
        workspaceMessage = `Trusted as ${profileLabel(profile)}.`;
      }
    } catch (e) {
      workspaceError = e instanceof Error ? e.message : String(e);
    }
  }

  function navigate(next: AppRoute, section?: SettingsSectionId) {
    route = next;
    persistRoute(next);
    if (next === "settings" && section) settingsSection = section;
    else if (next !== "settings") settingsSection = null;
  }

  function toggleSidebar() {
    collapsed = !collapsed;
    if (typeof localStorage !== "undefined") localStorage.setItem(SIDEBAR_KEY, String(collapsed));
  }

  function domainIdForRun(runId: string): string | null {
    const run = snapshot.runs.find((r) => r.id === runId);
    if (!run) return null;
    return snapshot.domains.find((d) => d.repo_root === run.repo_root)?.domain_id ?? null;
  }

  function onFocusRun(runId: string, mode: "topology-focus" | "run-review") {
    focusRunId = runId;
    focusDomainId = domainIdForRun(runId);
    navigate(mode);
  }

  function onViewTopology(domainId: string) {
    focusRunId = null;
    focusDomainId = domainId;
    navigate("topology-focus");
  }

  async function refreshSnapshot(opts: { silent?: boolean } = {}) {
    try {
      const includeAgents = route === "operations" || route === "topology-focus" || route === "run-review";
      const next = await backend.loadSnapshot({ includeAgents });
      const nextFp = fingerprintSnapshot(next);
      if (nextFp !== snapshotFingerprint) {
        snapshot = next;
        snapshotFingerprint = nextFp;
        lastPollAt = Date.now();
      }
      pollLive = !next.error;
      if (next.error && !opts.silent) {
        loadMessage = next.error.message;
      } else if (!next.error && loadMessage && previewState === "default") {
        loadMessage = "";
      }
    } catch {
      pollLive = false;
    }
  }

  function pollIntervalMs(): number {
    if (typeof document !== "undefined" && document.visibilityState === "hidden") {
      return SNAPSHOT_POLL_HIDDEN_MS;
    }
    if (!windowFocused) return SNAPSHOT_POLL_HIDDEN_MS;
    if (route === "operations" && hasActiveRuns) return SNAPSHOT_POLL_ACTIVE_MS;
    if (route === "settings" || route === "integrations") return SNAPSHOT_POLL_IDLE_MS;
    return SNAPSHOT_POLL_IDLE_MS;
  }

  async function selectDomain(domainId: string, opts: { route?: AppRoute } = {}) {
    const domain = snapshot.domains.find((d) => d.domain_id === domainId);
    if (!domain) return;
    activeDomainId = domainId;
    focusDomainId = domainId;
    const label = domain.repo_root.split(/[\\/]/).pop() ?? domain.domain_id;
    recents = addWorkspaceRecent({ id: domain.domain_id, label, domainId: domain.domain_id });
    try {
      await ipc.selectDomain(domain.domain_id);
    } catch {
      /* Preview / Storybook backends may lack select_domain. */
    }
    navigate(opts.route ?? "operations");
  }

  function openRecent(recent: WorkspaceRecent) {
    const domain = snapshot.domains.find((d) => d.domain_id === recent.domainId);
    if (domain) void selectDomain(domain.domain_id);
    else navigate("workspaces");
  }

  async function onRunCompleted() {
    await refreshSnapshot();
    navigate("runs");
  }

  async function onWorkspaceOpened(openedPath?: string | null) {
    workspaceError = "";
    await refreshSnapshot();
    if (!openedPath) {
      recents = migrateWorkspaceRecents();
      return;
    }
    const normalized = openedPath.replace(/\\/g, "/").toLowerCase();
    const domain =
      snapshot.domains.find((d) => d.domain_id === openedPath) ??
      snapshot.domains.find((d) => d.repo_root === openedPath) ??
      snapshot.domains.find((d) => d.repo_root.replace(/\\/g, "/").toLowerCase() === normalized);
    if (domain) {
      await trustOpenedWorkspace(domain.repo_root);
      await selectDomain(domain.domain_id);
      return;
    }
    // Catalog row may lag; still trust and select by canonical path.
    try {
      await trustOpenedWorkspace(openedPath);
      await ipc.selectDomain(openedPath);
      activeDomainId = openedPath;
      const label = openedPath.split(/[\\/]/).pop() ?? openedPath;
      recents = addWorkspaceRecent({ id: openedPath, label, domainId: openedPath });
      navigate("operations");
    } catch (e) {
      workspaceError = e instanceof Error ? e.message : String(e);
      recents = migrateWorkspaceRecents();
      navigate("workspaces");
    }
  }

  function onDomainForgotten(domainId: string) {
    snapshot = { ...snapshot, domains: snapshot.domains.filter((d) => d.domain_id !== domainId) };
    snapshotFingerprint = fingerprintSnapshot(snapshot);
    recents = removeWorkspaceRecent(domainId);
    if (activeDomainId === domainId) activeDomainId = snapshot.domains[0]?.domain_id ?? null;
    if (editingWorkspaceId === domainId) editingWorkspaceId = null;
  }

  async function addWorkspace() {
    workspaceError = "";
    workspaceMessage = "";
    try {
      const opened = await backend.openWorkspace();
      if (opened) await onWorkspaceOpened(opened);
    } catch (e) {
      workspaceError = e instanceof Error ? e.message : String(e);
      navigate("workspaces");
    }
  }

  async function installVoiceModel() {
    voiceInstalling = true;
    try {
      voiceModelPath = await backend.installVoiceModel();
    } finally {
      voiceInstalling = false;
    }
  }

  function onGlobalKeydown(event: KeyboardEvent) {
    if ((event.metaKey || event.ctrlKey) && event.key.toLowerCase() === "k") {
      event.preventDefault();
      commandOpen = true;
    }
  }

  onMount(() => {
    let disposed = false;
    let deepLinkUnlisten: (() => void) | null = null;
    let pollTimer: ReturnType<typeof setTimeout> | null = null;
    route = routeOverride ?? initialRoute();
    const onHashChange = () => {
      if (!routeOverride) route = initialRoute();
    };
    const handleDeepLink = (value: string) => {
      const target = routeFromDeepLink(value);
      if (target) navigate(target);
    };
    const onBrowserDeepLink = (event: Event) => handleDeepLink((event as CustomEvent<string>).detail);
    const onFocus = () => {
      windowFocused = true;
    };
    const onBlur = () => {
      windowFocused = false;
    };
    window.addEventListener("hashchange", onHashChange);
    window.addEventListener("pytxo-deep-link", onBrowserDeepLink);
    window.addEventListener("keydown", onGlobalKeydown);
    window.addEventListener("focus", onFocus);
    window.addEventListener("blur", onBlur);
    recents = migrateWorkspaceRecents();

    const clearPoll = () => {
      if (pollTimer) clearTimeout(pollTimer);
      pollTimer = null;
    };

    const schedulePoll = () => {
      clearPoll();
      if (disposed || previewState !== "default") return;
      pollTimer = setTimeout(() => {
        void refreshSnapshot({ silent: true }).finally(() => {
          if (!disposed) schedulePoll();
        });
      }, pollIntervalMs());
    };

    const cleanup = () => {
      disposed = true;
      window.removeEventListener("hashchange", onHashChange);
      window.removeEventListener("pytxo-deep-link", onBrowserDeepLink);
      window.removeEventListener("keydown", onGlobalKeydown);
      window.removeEventListener("focus", onFocus);
      window.removeEventListener("blur", onBlur);
      clearPoll();
    };

    if (previewState === "loading") return cleanup;
    if (previewState === "empty" || previewState === "offline") {
      loading = false;
      if (previewState === "offline") loadMessage = "Offline · showing locally available workspace state";
      return cleanup;
    }
    if (previewState === "error") {
      loading = false;
      loadMessage = "Pytxo could not reach the local hypervisor. Retry when the service is available.";
      return cleanup;
    }
    void (async () => {
      try {
        await refreshSnapshot();
        const openBehavior =
          typeof localStorage !== "undefined" &&
          localStorage.getItem(OPEN_BEHAVIOR_KEY) === "picker"
            ? "picker"
            : "last";
        if (openBehavior === "picker") {
          if (!routeOverride && route === "operations") navigate("workspaces");
        } else if (!activeDomainId) {
          const preferred =
            recents.find((r) => snapshot.domains.some((d) => d.domain_id === r.domainId))
              ?.domainId ?? snapshot.domains[0]?.domain_id;
          if (preferred) {
            activeDomainId = preferred;
            try {
              await ipc.selectDomain(preferred);
            } catch {
              /* preview */
            }
          }
        }
        try {
          voiceModelPath = await backend.voiceModelStatus();
        } catch {
          /* optional */
        }
      } finally {
        loading = false;
      }
      if (!disposed && previewState === "default") schedulePoll();
      try {
        const unlisten = await onPytxoDeepLink(handleDeepLink);
        if (disposed) unlisten();
        else deepLinkUnlisten = unlisten;
      } catch {
        /* Browser and Storybook use the preview backend without Tauri events. */
      }
    })();
    return () => {
      cleanup();
      deepLinkUnlisten?.();
    };
  });
</script>

<div class="desktop2" class:sidebar-collapsed={collapsed}>
  <Sidebar
    {route}
    {primary}
    {system}
    approvalsCount={snapshot.approvals.length}
    {recents}
    {collapsed}
    {tier}
    {signedIn}
    onNavigate={(next) => navigate(next)}
    onToggleCollapse={toggleSidebar}
    onOpenCommand={() => (commandOpen = true)}
    onOpenRecent={openRecent}
    onAccountClick={() => navigate("settings", "account")}
  />

  <main>
    <UpdateBanner />
    <AppBar
      {route}
      approvalsCount={snapshot.approvals.length}
      hypervisorOnline={!snapshot.error}
      activeDomainLabel={activeDomain ? (activeDomain.repo_root.split(/[\\/]/).pop() ?? activeDomain.domain_id) : null}
      {activeDomainId}
      domains={snapshot.domains}
      live={pollLive}
      onOpenHistory={() => navigate("runs")}
      onOpenNotifications={() => navigate("approvals")}
      onSelectDomain={(id) => void selectDomain(id)}
      onAddWorkspace={() => void addWorkspace()}
      onOpenWorkspaceSettings={(id) => (editingWorkspaceId = id)}
    />
    <div class="content">
      {#if authErrorMessage}<div class="status-banner error-banner">{authErrorMessage}</div>{/if}
      {#if workspaceError}<div class="status-banner error-banner">{workspaceError}</div>{/if}
      {#if workspaceMessage}<div class="status-banner">{workspaceMessage}</div>{/if}
      {#if loadMessage}<div class:error-banner={previewState === "error" || !!snapshot.error} class="status-banner">{loadMessage}</div>{/if}
      {#if loading}
        <div class="loading-state"><div></div><div></div><div></div></div>
      {:else if route === "operations"}
        <OperationsScreen
          {snapshot}
          live={pollLive}
          lastPollAt={lastPollAt}
          activeDomainLabel={activeDomain ? (activeDomain.repo_root.split(/[\\/]/).pop() ?? null) : null}
          onRoute={navigate}
          onReviewRun={(runId) => onFocusRun(runId, "run-review")}
        />
      {:else if route === "flow"}
        <FlowScreen {backend} domains={snapshot.domains} preferredDomainId={activeDomainId} />
      {:else if route === "topology-focus" || route === "run-review"}
        <FocusScreen mode={route} run={focusedRun} domainId={focusDomainId} onBack={() => navigate("runs")} {onRunCompleted} />
      {:else if route === "settings"}
        <SettingsScreen
          initialSection={settingsSection}
          {activeDomain}
          {tier}
          {signedIn}
          {cliMissing}
          {subscriptionPortalUrl}
          {voiceModelPath}
          {voiceInstalling}
          onBack={() => navigate("operations")}
          {onReplayOnboarding}
          {onAuthChange}
          onInstallVoiceModel={installVoiceModel}
          onEditWorkspace={(id) => (editingWorkspaceId = id)}
          onOpenWorkspaces={() => navigate("workspaces")}
        />
      {:else}
        <CollectionScreen
          {route}
          {snapshot}
          {backend}
          onRoute={navigate}
          {activeDomainId}
          onReviewRun={(runId) => onFocusRun(runId, "run-review")}
          {onViewTopology}
          onSelectDomain={selectDomain}
          onWorkspaceOpened={onWorkspaceOpened}
          {onDomainForgotten}
          onApprovalsChanged={refreshSnapshot}
          onEditWorkspace={(id) => (editingWorkspaceId = id)}
        />
      {/if}
    </div>
  </main>

  <CommandPalette open={commandOpen} items={commandItems} onNavigate={navigate} onClose={() => (commandOpen = false)} />

  {#if editingDomain}
    <WorkspaceSettingsPanel
      domain={editingDomain}
      onClose={() => (editingWorkspaceId = null)}
      onForgotten={onDomainForgotten}
      onChanged={refreshSnapshot}
    />
  {/if}
</div>

<style>
  .desktop2 {
    display: grid;
    grid-template-columns: 224px minmax(0, 1fr);
    height: 100%;
    background: #07080b;
    color: #f1f3f5;
    font-family: "Geist", Inter, ui-sans-serif, system-ui, sans-serif;
    transition: grid-template-columns 150ms ease;
  }
  .desktop2.sidebar-collapsed {
    grid-template-columns: 56px minmax(0, 1fr);
  }
  main {
    min-width: 0;
    display: flex;
    flex-direction: column;
    background: radial-gradient(circle at 80% -10%, color-mix(in oklab, var(--pytxo-accent, #3ed7a1) 8%, transparent), transparent 30%), #08090c;
  }
  .content {
    flex: 1;
    min-height: 0;
    overflow: auto;
  }
  .loading-state {
    display: grid;
    grid-template-columns: repeat(3, 1fr);
    gap: 12px;
    padding: 80px 32px;
  }
  .loading-state div {
    height: 130px;
    border-radius: 8px;
    background: linear-gradient(90deg, #0f1116, #161920, #0f1116);
    background-size: 200%;
    animation: pulse 1.5s infinite;
  }
  @keyframes pulse {
    to {
      background-position: -200% 0;
    }
  }
  .status-banner {
    margin: 12px 32px 0;
    padding: 9px 11px;
    border: 1px solid #343027;
    border-radius: 6px;
    background: #15130e;
    color: #bda26d;
    font-size: 10px;
  }
  .status-banner.error-banner {
    border-color: #452a30;
    background: #1a1114;
    color: #d98a96;
  }

  @media (max-width: 1050px) {
    .desktop2:not(.sidebar-collapsed) {
      grid-template-columns: 190px minmax(0, 1fr);
    }
  }
  @media (max-width: 760px) {
    .desktop2:not(.sidebar-collapsed) {
      grid-template-columns: 60px minmax(0, 1fr);
    }
  }
  @media (prefers-reduced-motion: reduce) {
    .desktop2 {
      transition: none;
    }
    .loading-state div {
      animation: none;
    }
  }

  :global(html[data-chroma-theme="light"]) .desktop2 {
    background: var(--pytxo-obsidian);
    color: #17202b;
  }
</style>
