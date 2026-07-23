<script lang="ts">
  import { onMount } from "svelte";
  import { IconActivity, IconChecks, IconFolders, IconPlayerPlay, IconPlugConnected, IconSettings, IconSparkles } from "@tabler/icons-svelte";
  import { createDesktopBackend, type DesktopSnapshot } from "../../lib/desktop-backend";
  import { addWorkspaceRecent, initialRoute, migrateWorkspaceRecents, persistRoute, routeFromDeepLink, type AppRoute, type WorkspaceRecent } from "../../lib/navigation.svelte";
  import { onPytxoDeepLink } from "../../lib/ipc";
  import Sidebar from "./Sidebar.svelte";
  import AppBar from "./AppBar.svelte";
  import CommandPalette from "./CommandPalette.svelte";
  import OperationsScreen from "./OperationsScreen.svelte";
  import FlowScreen from "./FlowScreen.svelte";
  import CollectionScreen from "./CollectionScreen.svelte";
  import FocusScreen from "./FocusScreen.svelte";
  import "./desktop2-shared.css";

  const SNAPSHOT_POLL_MS = 1000;

  let {
    routeOverride = null,
    previewState = "default",
    collapsedOverride = null,
    tier = "core",
    signedIn = false,
    cliMissing = false,
    subscriptionPortalUrl = null,
    onReplayOnboarding = () => {},
  }: {
    routeOverride?: AppRoute | null;
    previewState?: "default" | "loading" | "empty" | "offline" | "error";
    /** Forces initial sidebar collapse state for Storybook; real usage always reads localStorage. */
    collapsedOverride?: boolean | null;
    tier?: string;
    signedIn?: boolean;
    cliMissing?: boolean;
    subscriptionPortalUrl?: string | null;
    onReplayOnboarding?: () => void;
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
  let loading = $state(true);
  let commandOpen = $state(false);
  let loadMessage = $state("");
  let recents = $state<WorkspaceRecent[]>([]);
  let collapsed = $state(readSidebarCollapsed());
  let focusRunId = $state<string | null>(null);
  let focusDomainId = $state<string | null>(null);
  let activeDomainId = $state<string | null>(null);
  let lastPollAt = $state<number | null>(null);
  let pollLive = $state(false);

  const activeDomain = $derived(
    snapshot.domains.find((d) => d.domain_id === activeDomainId) ?? null,
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

  function navigate(next: AppRoute) {
    route = next;
    persistRoute(next);
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
      const next = await backend.loadSnapshot();
      if (!opts.silent) snapshot = next;
      else snapshot = next;
      lastPollAt = Date.now();
      pollLive = !next.error;
    } catch {
      pollLive = false;
    }
  }

  function selectDomain(domainId: string, opts: { route?: AppRoute } = {}) {
    const domain = snapshot.domains.find((d) => d.domain_id === domainId);
    if (!domain) return;
    activeDomainId = domainId;
    focusDomainId = domainId;
    const label = domain.repo_root.split(/[\\/]/).pop() ?? domain.domain_id;
    recents = addWorkspaceRecent({ id: domain.domain_id, label, domainId: domain.domain_id });
    navigate(opts.route ?? "operations");
  }

  function openRecent(recent: WorkspaceRecent) {
    const domain = snapshot.domains.find((d) => d.domain_id === recent.domainId);
    if (domain) selectDomain(domain.domain_id);
    else navigate("workspaces");
  }

  async function onRunCompleted() {
    await refreshSnapshot();
    navigate("runs");
  }

  async function onWorkspaceOpened(openedPath?: string | null) {
    await refreshSnapshot();
    if (openedPath) {
      const domain = snapshot.domains.find((d) => d.repo_root === openedPath);
      if (domain) {
        selectDomain(domain.domain_id);
        return;
      }
    }
    recents = migrateWorkspaceRecents();
  }

  function onDomainForgotten(domainId: string) {
    snapshot = { ...snapshot, domains: snapshot.domains.filter((d) => d.domain_id !== domainId) };
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
    let pollTimer: ReturnType<typeof setInterval> | null = null;
    route = routeOverride ?? initialRoute();
    const onHashChange = () => {
      if (!routeOverride) route = initialRoute();
    };
    const handleDeepLink = (value: string) => {
      const target = routeFromDeepLink(value);
      if (target) navigate(target);
    };
    const onBrowserDeepLink = (event: Event) => handleDeepLink((event as CustomEvent<string>).detail);
    window.addEventListener("hashchange", onHashChange);
    window.addEventListener("pytxo-deep-link", onBrowserDeepLink);
    window.addEventListener("keydown", onGlobalKeydown);
    recents = migrateWorkspaceRecents();

    const cleanup = () => {
      disposed = true;
      window.removeEventListener("hashchange", onHashChange);
      window.removeEventListener("pytxo-deep-link", onBrowserDeepLink);
      window.removeEventListener("keydown", onGlobalKeydown);
      if (pollTimer) clearInterval(pollTimer);
      pollTimer = null;
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
        if (!activeDomainId && snapshot.domains[0]) activeDomainId = snapshot.domains[0].domain_id;
      } finally {
        loading = false;
      }
      if (!disposed && previewState === "default") {
        pollTimer = setInterval(() => {
          void refreshSnapshot({ silent: true });
        }, SNAPSHOT_POLL_MS);
      }
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
    onNavigate={navigate}
    onToggleCollapse={toggleSidebar}
    onOpenCommand={() => (commandOpen = true)}
    onOpenRecent={openRecent}
    onAccountClick={() => navigate("settings")}
  />

  <main>
    <AppBar
      {route}
      approvalsCount={snapshot.approvals.length}
      hypervisorOnline={!snapshot.error}
      activeDomainLabel={activeDomain ? (activeDomain.repo_root.split(/[\\/]/).pop() ?? activeDomain.domain_id) : null}
      live={pollLive}
      onOpenHistory={() => navigate("runs")}
      onOpenNotifications={() => navigate("approvals")}
    />
    <div class="content">
      {#if loadMessage}<div class:error-banner={previewState === "error"} class="status-banner">{loadMessage}</div>{/if}
      {#if loading}
        <div class="loading-state"><div></div><div></div><div></div></div>
      {:else if route === "operations"}
        <OperationsScreen
          {snapshot}
          live={pollLive}
          lastPollAt={lastPollAt}
          onRoute={navigate}
          onReviewRun={(runId) => onFocusRun(runId, "run-review")}
        />
      {:else if route === "flow"}
        <FlowScreen {backend} domains={snapshot.domains} preferredDomainId={activeDomainId} />
      {:else if route === "topology-focus" || route === "run-review"}
        <FocusScreen mode={route} run={focusedRun} domainId={focusDomainId} onBack={() => navigate("runs")} {onRunCompleted} />
      {:else}
        <CollectionScreen
          {route}
          {snapshot}
          {backend}
          onRoute={navigate}
          {tier}
          {signedIn}
          {cliMissing}
          {subscriptionPortalUrl}
          {onReplayOnboarding}
          onReviewRun={(runId) => onFocusRun(runId, "run-review")}
          {onViewTopology}
          onSelectDomain={selectDomain}
          onWorkspaceOpened={onWorkspaceOpened}
          {onDomainForgotten}
          onApprovalsChanged={refreshSnapshot}
        />
      {/if}
    </div>
  </main>

  <CommandPalette open={commandOpen} items={commandItems} onNavigate={navigate} onClose={() => (commandOpen = false)} />
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
    background: radial-gradient(circle at 80% -10%, rgba(62, 93, 107, 0.08), transparent 30%), #08090c;
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
