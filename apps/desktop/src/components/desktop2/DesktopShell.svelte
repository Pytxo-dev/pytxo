<script lang="ts">
  import { onMount, tick } from "svelte";
  import {
    IconActivity,
    IconChecks,
    IconHistory,
    IconPlayerStop,
    IconSettings,
    IconTarget,
  } from "@tabler/icons-svelte";
  import { createDesktopBackend, type DesktopSnapshot } from "../../lib/desktop-backend";
  import {
    consumeDomainChanges,
    fingerprintDesktopSnapshot,
    loadConsistentDesktopSnapshot,
  } from "../../lib/desktop-sync";
  import {
    addWorkspaceRecent,
    initialResolvedRoute,
    migrateWorkspaceRecents,
    persistRoute,
    removeWorkspaceRecent,
    resolveRoute,
    routeFromDeepLink,
    type AppRoute,
    type CanonicalRoute,
    type MissionPane,
    type MissionView,
    type SetupSectionId,
    type WorkPane,
    type WorkspaceRecent,
  } from "../../lib/navigation.svelte";
  import { ipc, onPytxoDeepLink } from "../../lib/ipc";
  import Sidebar from "./Sidebar.svelte";
  import AppBar from "./AppBar.svelte";
  import CommandPalette from "./CommandPalette.svelte";
  import WorkActive from "./WorkActive.svelte";
  import HistoryScreen from "./HistoryScreen.svelte";
  import MissionsScreen from "./MissionsScreen.svelte";
  import WorkspacesScreen from "./WorkspacesScreen.svelte";
  import ApprovalsInbox from "./ApprovalsInbox.svelte";
  import AgentsScreen from "./AgentsScreen.svelte";
  import SettingsScreen from "./SettingsScreen.svelte";
  import WorkspaceSettingsPanel from "./WorkspaceSettingsPanel.svelte";
  import UpdateBanner from "../shell/UpdateBanner.svelte";
  import "./desktop2-shared.css";

  const DOMAIN_DELTA_ACTIVE_MS = 2500;
  const DOMAIN_DELTA_IDLE_MS = 8000;
  const INTEGRITY_REFRESH_MS = 60_000;
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
  let route = $state<CanonicalRoute>("work");
  let workPane = $state<WorkPane>("active");
  let missionView = $state<MissionView>("list");
  let missionPane = $state<MissionPane>("live");
  let snapshot = $state<DesktopSnapshot>({ domains: [], runs: [], agents: [], approvals: [], fleets: [], diagnostics: [], error: null });
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
  let setupSection = $state<SetupSectionId | null>(null);
  let editingWorkspaceId = $state<string | null>(null);
  let voiceModelPath = $state<string | null>(null);
  let voiceInstalling = $state(false);
  let windowFocused = $state(true);
  let approvalsOpen = $state(false);
  let snapshotLoadedAt = $state<number | null>(null);
  let cursorGap = $state(false);
  let nowMs = $state(Date.now());
  let workScreen = $state<{ focusActiveRun: () => void } | null>(null);

  const activeDomain = $derived(
    snapshot.domains.find((d) => d.domain_id === activeDomainId) ?? null,
  );
  const editingDomain = $derived(
    snapshot.domains.find((d) => d.domain_id === editingWorkspaceId) ?? null,
  );
  const snapshotAge = $derived.by(() => {
    if (snapshotLoadedAt === null) return null;
    const seconds = Math.max(0, Math.round((nowMs - snapshotLoadedAt) / 1000));
    if (seconds < 5) return "Updated just now";
    if (seconds < 60) return `Updated ${seconds}s ago`;
    return `Updated ${Math.round(seconds / 60)}m ago`;
  });

  /**
   * Three destinations, because there are three modes of attention: run work,
   * investigate finished work, configure the machine. Workspace is context in
   * the title bar and approvals are an overlay, so neither takes a rail slot.
   */
  const primary = [
    { route: "work" as const, label: "Work", icon: IconActivity },
    { route: "history" as const, label: "History", icon: IconHistory },
  ];
  const system = [{ route: "setup" as const, label: "Setup", icon: IconSettings }];
  const domainCursors = new Map<string, number>();
  const hasActiveRuns = $derived(
    snapshot.runs.some((r) =>
      ["running", "pending", "dispatching", "active"].includes(r.status.toLowerCase()),
    ),
  );
  const activeCommandRun = $derived(
    snapshot.runs.find((r) =>
      ["running", "pending", "dispatching", "active"].includes(r.status.toLowerCase()),
    ) ?? null,
  );
  const latestReviewRun = $derived(
    snapshot.runs.find(
      (r) =>
        Boolean(r.prepared_digest) ||
        r.apply_status === "ready" ||
        r.apply_status === "waiting" ||
        r.apply_status === "prepared",
    ) ??
      snapshot.runs.find(
        (r) => !["running", "pending", "dispatching", "active"].includes(r.status.toLowerCase()),
      ) ??
      null,
  );
  const commandItems = $derived([
    ...[...primary, ...system].map((item) => ({
      id: item.route,
      route: item.route,
      label: item.label,
      icon: item.icon,
      group: "Navigate" as const,
    })),
    {
      id: "flow",
      route: "flow" as const,
      label: "New run",
      icon: IconTarget,
      aliases: ["flow", "compose", "mission"],
      group: "Navigate" as const,
    },
    {
      id: "approvals",
      label: "Open approvals inbox",
      icon: IconChecks,
      aliases: ["approve", "inbox", "decide"],
      group: "Actions" as const,
      hint: snapshot.approvals.length ? `${snapshot.approvals.length} open` : undefined,
      run: () => {
        approvalsOpen = true;
      },
    },
    {
      id: "stop-run",
      label: "Stop active run",
      icon: IconPlayerStop,
      aliases: ["stop", "kill"],
      group: "Actions" as const,
      disabled: !activeCommandRun,
      hint: activeCommandRun?.id,
      run: () => {
        const run = activeCommandRun;
        if (!run) return;
        const domainId = domainIdForRun(run.id);
        if (!domainId) return;
        void stopRunFromOps(run.id, domainId);
      },
    },
    {
      id: "open-review",
      label: "Open latest review",
      icon: IconChecks,
      aliases: ["review", "apply"],
      group: "Actions" as const,
      disabled: !latestReviewRun,
      hint: latestReviewRun?.id,
      run: () => {
        if (latestReviewRun) openMission(latestReviewRun.id, "review");
      },
    },
  ]);

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

  function applyResolved(next: ReturnType<typeof resolveRoute>, persist: boolean) {
    route = next.route;
    workPane = next.workPane;
    if (next.setupSection) setupSection = next.setupSection;
    // A route change is a deliberate move to a different surface, so the
    // overlay follows the destination rather than lingering over it.
    approvalsOpen = next.openApprovals;
    missionView = next.route === "history" ? "list" : next.workPane === "plan" ? "compose" : "detail";
    if (next.workPane === "review") missionPane = "review";
    else if (next.workPane === "plan") missionPane = "plan";
    if (persist) persistRoute(next.route);
  }

  function navigate(next: AppRoute, section?: SetupSectionId) {
    applyResolved(resolveRoute(next), true);
    if (route === "setup" && section) setupSection = section;
  }

  function openCompose() {
    navigate("flow");
  }

  /**
   * Opening a run keeps the operator inside Work: a live run lands on the
   * ledger, a finished one lands on the review pane. History is for finding a
   * run, not for reading one.
   */
  function openMission(runId: string, pane?: MissionPane) {
    focusRunId = runId;
    focusDomainId = domainIdForRun(runId);
    const found = snapshot.runs.find((r) => r.id === runId);
    const running = !!found && ["running", "pending", "dispatching", "active"].includes(found.status.toLowerCase());
    const resolvedPane = pane ?? (running ? "live" : "review");
    route = "work";
    missionView = "detail";
    missionPane = resolvedPane;
    workPane = resolvedPane === "live" ? "active" : "review";
    persistRoute("work");
  }

  function focusRun(runId: string) {
    focusRunId = runId;
    focusDomainId = domainIdForRun(runId);
  }

  function toggleSidebar() {
    collapsed = !collapsed;
    if (typeof localStorage !== "undefined") localStorage.setItem(SIDEBAR_KEY, String(collapsed));
  }

  function domainIdForRun(runId: string): string | null {
    const run = snapshot.runs.find((r) => r.id === runId);
    return run?.domain_id ?? null;
  }

  function pickDefaultDetailRun(runs: DesktopSnapshot["runs"]) {
    if (missionPane === "review") {
      return (
        runs.find((run) => (run.apply_status ?? "").toLowerCase() === "ready") ??
        runs.find((run) =>
          ["completed", "failed", "verify_failed", "cancelled"].includes(run.status.toLowerCase()),
        ) ??
        runs[0] ??
        null
      );
    }
    return (
      runs.find((run) =>
        ["running", "pending", "dispatching", "active"].includes(run.status.toLowerCase()),
      ) ??
      runs[0] ??
      null
    );
  }

  async function refreshSnapshot(opts: { silent?: boolean; primeCursors?: boolean } = {}) {
    try {
      const opsHeavy = route === "work";
      const documentHidden =
        typeof document !== "undefined" && document.visibilityState === "hidden";
      const idleChrome = !windowFocused || documentHidden || route === "setup";
      const includeAgents = opsHeavy && !idleChrome;
      const load = () => backend.loadSnapshot({
        includeAgents,
        runLimit: idleChrome && !opsHeavy ? 12 : 30,
        fleetLimit: idleChrome && !opsHeavy ? 8 : 20,
      });
      const next = opts.primeCursors === false
        ? await load()
        : await loadConsistentDesktopSnapshot(
            load,
            (domainId, cursor, limit) => backend.domainChanges(domainId, cursor, limit),
            domainCursors,
          );
      const nextFp = fingerprintDesktopSnapshot(next);
      if (nextFp !== snapshotFingerprint) {
        snapshot = next;
        snapshotFingerprint = nextFp;
        void ipc.setTrayNeedsYou(next.approvals.length);
      }
      // Every screen renders a snapshot, never "now". The title bar states how
      // old the rendered state is so nothing implies live truth.
      snapshotLoadedAt = Date.now();
      nowMs = snapshotLoadedAt;
      if (missionView === "detail" && !focusRunId) {
        const pick = pickDefaultDetailRun(next.runs);
        if (pick) {
          focusRunId = pick.id;
          focusDomainId = pick.domain_id;
        }
      }
      if (next.error && !opts.silent) {
        loadMessage = next.error.message;
      } else if (!next.error && loadMessage && previewState === "default") {
        loadMessage = "";
      }
    } catch {
      /* keep last snapshot */
    }
  }

  async function catchUpDomainChanges() {
    const result = await consumeDomainChanges(
      snapshot.domains.map((domain) => domain.domain_id),
      domainCursors,
      (domainId, cursor, limit) => backend.domainChanges(domainId, cursor, limit),
    );
    // A cursor gap means the change log dropped entries between polls, so this
    // view may have missed events. The operator is told rather than left to
    // assume continuity.
    if (result.needsSnapshot) cursorGap = true;
    if (result.changed || result.needsSnapshot) {
      await refreshSnapshot({ silent: true, primeCursors: false });
    }
    nowMs = Date.now();
  }

  function deltaIntervalMs(): number {
    if (!windowFocused || (typeof document !== "undefined" && document.visibilityState === "hidden")) {
      return DOMAIN_DELTA_IDLE_MS;
    }
    return hasActiveRuns ? DOMAIN_DELTA_ACTIVE_MS : DOMAIN_DELTA_IDLE_MS;
  }

  async function selectDomain(domainId: string, opts: { route?: CanonicalRoute } = {}) {
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
    await refreshSnapshot({ silent: true });
    navigate(opts.route ?? "work");
  }

  function openRecent(recent: WorkspaceRecent) {
    const domain = snapshot.domains.find((d) => d.domain_id === recent.domainId);
    if (domain) void selectDomain(domain.domain_id);
    else navigate("setup", "workspaces");
  }

  async function onRunCompleted() {
    await refreshSnapshot();
    route = "work";
    workPane = "review";
    missionView = "detail";
    missionPane = "review";
  }

  async function stopRunFromOps(runId: string, domainId: string) {
    await backend.stopRun(runId, domainId);
    await refreshSnapshot();
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
    try {
      await trustOpenedWorkspace(openedPath);
      await ipc.selectDomain(openedPath);
      activeDomainId = openedPath;
      const label = openedPath.split(/[\\/]/).pop() ?? openedPath;
      recents = addWorkspaceRecent({ id: openedPath, label, domainId: openedPath });
      navigate("work");
    } catch (e) {
      workspaceError = e instanceof Error ? e.message : String(e);
      recents = migrateWorkspaceRecents();
      navigate("setup", "workspaces");
    }
  }

  function onDomainForgotten(domainId: string) {
    snapshot = { ...snapshot, domains: snapshot.domains.filter((d) => d.domain_id !== domainId) };
    snapshotFingerprint = fingerprintDesktopSnapshot(snapshot);
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
      navigate("setup", "workspaces");
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

  function isEditableTarget(target: EventTarget | null): boolean {
    if (!(target instanceof HTMLElement)) return false;
    return (
      target.isContentEditable ||
      target.tagName === "INPUT" ||
      target.tagName === "TEXTAREA" ||
      target.tagName === "SELECT"
    );
  }

  async function focusWork() {
    navigate("work");
    await tick();
    workScreen?.focusActiveRun();
  }

  function onGlobalKeydown(event: KeyboardEvent) {
    const modifier = event.metaKey || event.ctrlKey;
    const plain =
      !event.defaultPrevented && !event.repeat && !modifier && !event.altKey && !event.shiftKey && !isEditableTarget(event.target);
    if (
      !event.defaultPrevented &&
      !event.repeat &&
      modifier &&
      event.shiftKey &&
      !event.altKey &&
      event.key.toLowerCase() === "o" &&
      !isEditableTarget(event.target)
    ) {
      event.preventDefault();
      void focusWork();
      return;
    }
    // `A` opens the decision queue from anywhere. Approvals follow the operator
    // rather than living at a fixed address they have to walk to.
    if (plain && event.key.toLowerCase() === "a" && !approvalsOpen) {
      event.preventDefault();
      approvalsOpen = true;
      return;
    }
    if (modifier && event.key.toLowerCase() === "k") {
      event.preventDefault();
      commandOpen = true;
    }
  }

  onMount(() => {
    let disposed = false;
    let deepLinkUnlisten: (() => void) | null = null;
    let domainChangedUnlisten: (() => void) | null = null;
    let deltaTimer: ReturnType<typeof setTimeout> | null = null;
    let integrityTimer: ReturnType<typeof setInterval> | null = null;
    const ageTimer = setInterval(() => (nowMs = Date.now()), 5000);
    applyResolved(routeOverride ? resolveRoute(routeOverride) : initialResolvedRoute(), false);
    const onHashChange = () => {
      if (!routeOverride) applyResolved(initialResolvedRoute(), false);
    };
    const handleDeepLink = (value: string) => {
      const target = routeFromDeepLink(value);
      if (target) navigate(target);
    };
    const onBrowserDeepLink = (event: Event) => handleDeepLink((event as CustomEvent<string>).detail);
    const onFocus = () => {
      const resumed = !windowFocused;
      windowFocused = true;
      if (resumed) void refreshSnapshot({ silent: true });
    };
    const onBlur = () => {
      windowFocused = false;
    };
    const onVisibilityChange = () => {
      if (document.visibilityState === "visible") void refreshSnapshot({ silent: true });
    };
    window.addEventListener("hashchange", onHashChange);
    window.addEventListener("pytxo-deep-link", onBrowserDeepLink);
    window.addEventListener("keydown", onGlobalKeydown);
    window.addEventListener("focus", onFocus);
    window.addEventListener("blur", onBlur);
    document.addEventListener("visibilitychange", onVisibilityChange);
    recents = migrateWorkspaceRecents();

    const clearDelta = () => {
      if (deltaTimer) clearTimeout(deltaTimer);
      deltaTimer = null;
    };

    const scheduleDelta = () => {
      clearDelta();
      if (disposed || previewState !== "default") return;
      deltaTimer = setTimeout(() => {
        void catchUpDomainChanges().finally(() => {
          if (!disposed) scheduleDelta();
        });
      }, deltaIntervalMs());
    };

    const cleanup = () => {
      disposed = true;
      window.removeEventListener("hashchange", onHashChange);
      window.removeEventListener("pytxo-deep-link", onBrowserDeepLink);
      window.removeEventListener("keydown", onGlobalKeydown);
      window.removeEventListener("focus", onFocus);
      window.removeEventListener("blur", onBlur);
      document.removeEventListener("visibilitychange", onVisibilityChange);
      clearDelta();
      clearInterval(ageTimer);
      if (integrityTimer) clearInterval(integrityTimer);
      integrityTimer = null;
    };

    if (previewState === "loading") return cleanup;
    if (previewState === "empty" || previewState === "offline") {
      loading = false;
      if (previewState === "offline") loadMessage = "Offline · showing locally available workspace state";
      return cleanup;
    }
    if (previewState === "error") {
      loading = false;
      loadMessage = "Pytxo could not reach the local service. Retry when it is available.";
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
          if (!routeOverride && route === "work") navigate("setup", "workspaces");
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
      if (!disposed && previewState === "default") {
        scheduleDelta();
        integrityTimer = setInterval(
          () => void refreshSnapshot({ silent: true }),
          INTEGRITY_REFRESH_MS,
        );
      }
      try {
        const [deepLinkStop, domainStop] = await Promise.all([
          onPytxoDeepLink(handleDeepLink),
          backend.onDomainChanged(() => {
            void refreshSnapshot({ silent: true, primeCursors: false });
          }),
        ]);
        if (disposed) {
          deepLinkStop();
          domainStop();
        } else {
          deepLinkUnlisten = deepLinkStop;
          domainChangedUnlisten = domainStop;
        }
      } catch {
        /* Browser and Storybook use the preview backend without Tauri events. */
      }
    })();
    return () => {
      cleanup();
      deepLinkUnlisten?.();
      domainChangedUnlisten?.();
    };
  });
</script>

<div class="desktop2" class:sidebar-collapsed={collapsed}>
  <Sidebar
    {route}
    {primary}
    {system}
    {recents}
    {collapsed}
    {tier}
    {signedIn}
    onNavigate={(next) => navigate(next)}
    onToggleCollapse={toggleSidebar}
    onOpenCommand={() => (commandOpen = true)}
    onOpenRecent={openRecent}
    onAccountClick={() => navigate("setup", "account")}
  />

  <main>
    <UpdateBanner />
    <AppBar
      {route}
      hypervisorOnline={!snapshot.error}
      activeDomainLabel={activeDomain ? (activeDomain.repo_root.split(/[\\/]/).pop() ?? activeDomain.domain_id) : null}
      {activeDomainId}
      domains={snapshot.domains}
      approvalsCount={snapshot.approvals.length}
      {snapshotAge}
      {cursorGap}
      onSelectDomain={(id) => void selectDomain(id)}
      onAddWorkspace={() => void addWorkspace()}
      onOpenWorkspaceSettings={(id) => (editingWorkspaceId = id)}
      onOpenApprovals={() => (approvalsOpen = true)}
      onDismissGap={() => (cursorGap = false)}
    />
    <div class="content">
      {#if authErrorMessage}<div class="status-banner error-banner">{authErrorMessage}</div>{/if}
      {#if workspaceError}<div class="status-banner error-banner">{workspaceError}</div>{/if}
      {#if workspaceMessage}<div class="status-banner">{workspaceMessage}</div>{/if}
      {#if loadMessage}<div class:error-banner={previewState === "error" || !!snapshot.error} class="status-banner">{loadMessage}</div>{/if}
      {#if snapshot.diagnostics.length}
        <div class="status-banner error-banner" role="status">
          Snapshot is partial: {snapshot.diagnostics.length} workspace read {snapshot.diagnostics.length === 1 ? "failed" : "failures"}. Review the missing evidence before approving or applying.
          <details>
            <summary>Show partial-snapshot details</summary>
            <ul>
              {#each snapshot.diagnostics as diagnostic}
                <li><strong>{diagnostic.domain_id}</strong> · {diagnostic.stage}{diagnostic.run_id ? ` · ${diagnostic.run_id}` : ""}: {diagnostic.message}</li>
              {/each}
            </ul>
          </details>
        </div>
      {/if}
      {#if loading}
        <div class="loading-state"><div></div><div></div><div></div></div>
      {:else if route === "work" && workPane === "active"}
        <WorkActive
          bind:this={workScreen}
          {snapshot}
          {backend}
          activeDomainLabel={activeDomain ? (activeDomain.repo_root.split(/[\\/]/).pop() ?? null) : null}
          {activeDomainId}
          {focusRunId}
          onNewRun={openCompose}
          onOpenApprovals={() => (approvalsOpen = true)}
          onReviewRun={(runId) => openMission(runId, "review")}
          onStopRun={stopRunFromOps}
          onSelectRun={focusRun}
        />
      {:else if route === "history"}
        <HistoryScreen
          {snapshot}
          {backend}
          {activeDomainId}
          {focusRunId}
          onOpenRun={(runId) => openMission(runId, "review")}
          onSelectRun={focusRun}
        />
      {:else if route === "work"}
        <MissionsScreen
          view={missionView}
          {snapshot}
          {backend}
          preferredDomainId={activeDomainId}
          {focusRunId}
          {focusDomainId}
          onView={(next) => {
            missionView = next;
            if (next === "list") {
              route = "history";
              persistRoute("history");
            } else {
              route = "work";
              workPane = next === "compose" ? "plan" : workPane === "plan" ? "review" : workPane;
              persistRoute("work");
            }
          }}
          onOpenMission={openMission}
          onAddWorkspace={addWorkspace}
          {onRunCompleted}
        />
      {:else}
        <SettingsScreen
          initialSection={setupSection}
          {backend}
          {activeDomain}
          {tier}
          {signedIn}
          {cliMissing}
          {subscriptionPortalUrl}
          {voiceModelPath}
          {voiceInstalling}
          {onReplayOnboarding}
          {onAuthChange}
          onInstallVoiceModel={installVoiceModel}
          onEditWorkspace={(id) => (editingWorkspaceId = id)}
        >
          {#snippet workspacesCatalog()}
            <WorkspacesScreen
              embedded
              {snapshot}
              {backend}
              {activeDomainId}
              onSelectDomain={selectDomain}
              onWorkspaceOpened={onWorkspaceOpened}
              {onDomainForgotten}
              onEditWorkspace={(id) => (editingWorkspaceId = id)}
              onNewMission={openCompose}
            />
          {/snippet}
          {#snippet agentsCatalog()}
            <AgentsScreen embedded {backend} onUseInMission={openCompose} />
          {/snippet}
        </SettingsScreen>
      {/if}
    </div>
  </main>

  <CommandPalette open={commandOpen} items={commandItems} onNavigate={navigate} onClose={() => (commandOpen = false)} />

  <ApprovalsInbox
    open={approvalsOpen}
    {snapshot}
    {backend}
    onClose={() => (approvalsOpen = false)}
    onReviewRun={(runId) => openMission(runId, "review")}
    onApprovalsChanged={refreshSnapshot}
  />

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
    grid-template-columns: 208px minmax(0, 1fr);
    height: 100%;
    background: var(--pytxo-surface-shell);
    color: var(--pytxo-text-strong);
    font-family: "Satoshi", "Sora", "IBM Plex Sans", ui-sans-serif, system-ui, sans-serif;
    transition: grid-template-columns 140ms ease;
  }
  .desktop2.sidebar-collapsed {
    grid-template-columns: 58px minmax(0, 1fr);
  }
  main {
    min-width: 0;
    display: flex;
    flex-direction: column;
    background: var(--pytxo-surface-shell);
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
    border-radius: 6px;
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
    font-size: 13px;
  }
  .status-banner.error-banner {
    border-color: #452a30;
    background: #1a1114;
    color: #d98a96;
  }

  @media (max-width: 1050px) {
    .desktop2:not(.sidebar-collapsed) {
      grid-template-columns: 188px minmax(0, 1fr);
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
</style>
