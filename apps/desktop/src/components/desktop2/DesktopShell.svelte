<script lang="ts">
  import { onMount, tick, untrack } from "svelte";
  import IconActivity from "@tabler/icons-svelte/icons/activity";
  import IconChecks from "@tabler/icons-svelte/icons/checks";
  import IconHistory from "@tabler/icons-svelte/icons/history";
  import IconPlayerStop from "@tabler/icons-svelte/icons/player-stop";
  import IconSettings from "@tabler/icons-svelte/icons/settings";
  import IconTarget from "@tabler/icons-svelte/icons/target";
  import { createDesktopBackend, type DesktopSnapshot } from "../../lib/desktop-backend";
  import type { ComposerDraft } from "../../lib/composer-draft";
  import { SETTINGS_SECTIONS } from "../../lib/settings-catalog";
  import {
    consumeDomainChanges,
    fingerprintDesktopSnapshot,
    incrementalDomainBatchSize,
    loadConsistentDesktopSnapshot,
    selectIncrementalDomainBatch,
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
  import MissionDock from "./MissionDock.svelte";
  import { selectMissionRun, restoreSelectedWork, SELECTED_WORK_KEY } from "../../lib/mission-selection";
  import type { DockPosition, DockReference } from "../../lib/dock-layout";
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
  const RECOVERY_AUDIT_MS = 5 * 60_000;
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
  let shellElement: HTMLDivElement | undefined = $state();
  let autoCollapsed = $state(false);
  const sidebarCollapsed = $derived(collapsed || autoCollapsed);
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
  let workScreen = $state<{ focusActiveRun: () => void; requestStopRun: (id: string) => void } | null>(null);
  let composerDraft = $state<ComposerDraft | null>(null);
  const workspaceDrafts = new Map<string, ComposerDraft>();

  function retainComposerDraft(draft: ComposerDraft | null) {
    if (composerDraft) workspaceDrafts.delete(composerDraft.domainId);
    if (draft) workspaceDrafts.set(draft.domainId, draft);
    composerDraft = draft;
    preferredAdeId = null;
  }
  let preferredAdeId = $state<string | null>(null);
  let contentElement = $state<HTMLDivElement | null>(null);
  let missionDock = $state<{ open: (ref: DockReference, position?: DockPosition) => void; leaveFocus: () => void; dismiss: (position?: DockPosition) => void } | null>(null);
  const dockRun = $derived(selectMissionRun(snapshot.runs, activeDomainId, focusRunId));

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
  let catalogFingerprint: string | null = null;
  let dormantDomainPollOffset = 0;
  const hasActiveRuns = $derived(
    snapshot.runs.some((r) =>
      ["starting", "running", "pending", "dispatching", "active"].includes(r.status.toLowerCase()),
    ),
  );
  const commandRuns = $derived(snapshot.runs.filter((r) => r.domain_id === activeDomainId));
  const activeCommandRuns = $derived(commandRuns.filter((r) =>
      ["starting", "running", "pending", "dispatching", "active"].includes(r.status.toLowerCase()),
  ));
  const activeCommandRun = $derived(activeCommandRuns.find((r) => r.id === focusRunId) ?? activeCommandRuns[0] ?? null);
  const latestReviewRun = $derived(
    commandRuns.find(
      (r) =>
        Boolean(r.prepared_digest) ||
        r.apply_status === "ready" ||
        r.apply_status === "waiting" ||
        r.apply_status === "prepared",
    ) ??
      commandRuns.find(
        (r) => !["starting", "running", "pending", "dispatching", "active"].includes(r.status.toLowerCase()),
      ) ??
      null,
  );
  const commandItems = $derived([
    ...SETTINGS_SECTIONS.map((section) => ({
      id: `settings-${section.id}`,
      label: section.label,
      icon: IconSettings,
      group: "Settings" as const,
      aliases: [...section.keywords, "settings", "setup"],
      hint: section.description,
      run: () => navigate("setup", section.id),
    })),
    ...[...primary, ...system].map((item) => ({
      id: item.route,
      route: item.route,
      label: item.label,
      icon: item.icon,
      group: "Navigate" as const,
    })),
    {
      id: "flow",
      label: "New work",
      icon: IconTarget,
      aliases: ["flow", "compose", "mission"],
      group: "Actions" as const,
      run: () => openCompose(),
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
      hint: activeCommandRun?.id ?? "No active run in this workspace",
      run: () => {
        const run = activeCommandRun;
        if (!run) return;
        void requestCommandStop(run.id, run.domain_id);
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
    missionDock?.leaveFocus();
    composerDraft ??= workspaceDrafts.get(activeDomainId ?? "") ?? null;
    const draftDomain = composerDraft?.domainId;
    if (draftDomain && snapshot.domains.some((d) => d.domain_id === draftDomain)) {
      activeDomainId = draftDomain;
    }
    navigate("flow");
  }

  function composeWithAgent(adeId: string) {
    preferredAdeId = adeId;
    openCompose();
  }

  async function requestCommandStop(runId: string, domainId: string) {
    if (domainId !== activeDomainId) return;
    focusRun(runId);
    navigate("work");
    await tick();
    if (domainId === activeDomainId) workScreen?.requestStopRun(runId);
  }

  /**
   * Opening a run keeps the operator inside Work: a live run lands on the
   * ledger, a finished one lands on the review pane. History is for finding a
   * run, not for reading one.
   */
  function openMission(runId: string, pane?: MissionPane, requestedDomainId?: string) {
    focusRunId = runId;
    focusDomainId = requestedDomainId ?? domainIdForRun(runId);
    const found = snapshot.runs.find((r) => r.id === runId && r.domain_id === focusDomainId);
    if (found) activeDomainId = found.domain_id;
    if (found) rememberSelectedWork(found.id, found.domain_id);
    const running = !!found && ["starting", "running", "pending", "dispatching", "active"].includes(found.status.toLowerCase());
    const resolvedPane = pane ?? (running ? "live" : "review");
    route = "work";
    missionView = "detail";
    missionPane = resolvedPane;
    workPane = resolvedPane === "live" ? "active" : "review";
    persistRoute("work");
    void tick().then(() => {
      if (route === "work" && focusRunId === runId) {
        contentElement?.scrollTo({ top: 0, left: 0, behavior: "instant" });
      }
    });
  }

  function focusRun(runId: string) {
    focusRunId = runId;
    focusDomainId = snapshot.runs.find(run => run.id === runId && run.domain_id === activeDomainId)?.domain_id ?? null;
    if (focusDomainId) rememberSelectedWork(runId, focusDomainId);
  }

  function rememberSelectedWork(runId: string | null, domainId: string) {
    try { localStorage.setItem(SELECTED_WORK_KEY, JSON.stringify({ runId, domainId })); }
    catch { /* Storage is optional; navigation still works. */ }
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
        ["starting", "running", "pending", "dispatching", "active"].includes(run.status.toLowerCase()),
      ) ??
      runs[0] ??
      null
    );
  }

  let snapshotRequest = 0;
  async function refreshSnapshot(opts: { silent?: boolean; primeCursors?: boolean } = {}): Promise<boolean> {
    const request = ++snapshotRequest;
    // Cursor progress belongs to the snapshot we publish, never a discarded read.
    const nextCursors = new Map(domainCursors);
    try {
      const opsHeavy = route === "work";
      const documentHidden =
        typeof document !== "undefined" && document.visibilityState === "hidden";
      const idleChrome = !windowFocused || documentHidden || route === "setup";
      // Work still renders worker identity and evidence while unfocused.
      // Throttle background polling, but never publish an incomplete Work snapshot.
      const includeAgents = opsHeavy;
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
            nextCursors,
            backend.domainChangesBatch?.bind(backend),
          );
      // A late lightweight read must not erase worker rows after entering Work.
      if (request !== snapshotRequest || (!includeAgents && route === "work")) return false;
      if (opts.primeCursors !== false) {
        domainCursors.clear();
        for (const [domain, cursor] of nextCursors) domainCursors.set(domain, cursor);
      }
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
      return next.error === null;
    } catch {
      /* keep last snapshot */
      return false;
    }
  }

  function openWorkspaceCompose() {
    // New work belongs to the visible workspace. Only Continue draft may
    // deliberately return to a different workspace's unfinished request.
    composerDraft = workspaceDrafts.get(activeDomainId ?? "") ?? null;
    openCompose();
  }

  let previousSnapshotRoute: CanonicalRoute = "work";
  $effect(() => {
    const nextRoute = route;
    const enteredWork = nextRoute === "work" && previousSnapshotRoute !== "work";
    previousSnapshotRoute = nextRoute;
    // Setup/History snapshots intentionally omit agents. Route changes do not
    // produce backend events, so fetch the full Work snapshot immediately.
    if (enteredWork) untrack(() => { void refreshSnapshot({ silent: true }); });
  });

  async function catchUpDomainChanges() {
    const observedCatalog = await backend.catalogFingerprint().catch(() => null);
    if (observedCatalog !== null && (catalogFingerprint === null || observedCatalog !== catalogFingerprint)) {
      // Capture before loading. A catalog write racing this snapshot changes
      // the next fingerprint, so the following poll refreshes again.
      const refreshed = await refreshSnapshot({ silent: true });
      if (refreshed) catalogFingerprint = observedCatalog;
      nowMs = Date.now();
      return;
    }
    const urgentDomainIds = new Set<string>();
    if (activeDomainId) urgentDomainIds.add(activeDomainId);
    for (const domain of snapshot.domains) {
      if (domain.active_runs > 0 || domain.hitl_pending > 0) urgentDomainIds.add(domain.domain_id);
    }
    for (const run of snapshot.runs) {
      if (["starting", "running", "pending", "dispatching", "active"].includes(run.status.toLowerCase())) {
        urgentDomainIds.add(run.domain_id);
      }
    }
    for (const approval of snapshot.approvals) urgentDomainIds.add(approval.domain_id);
    const interval = deltaIntervalMs();
    const documentHidden =
      typeof document !== "undefined" && document.visibilityState === "hidden";
    const coverageMs = !windowFocused || documentHidden ? RECOVERY_AUDIT_MS : 60_000;
    const selected = selectIncrementalDomainBatch(
      snapshot.domains.map(domain => domain.domain_id),
      urgentDomainIds,
      dormantDomainPollOffset,
      incrementalDomainBatchSize(snapshot.domains.length, interval, coverageMs),
    );
    dormantDomainPollOffset = selected.nextDormantOffset;
    const result = await consumeDomainChanges(
      selected.domainIds,
      domainCursors,
      (domainId, cursor, limit) => backend.domainChanges(domainId, cursor, limit),
      200,
      backend.domainChangesBatch?.bind(backend),
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
    focusRunId = selectMissionRun(snapshot.runs, domainId, null)?.id ?? null;
    rememberSelectedWork(focusRunId, domainId);
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
    if (event.target instanceof Element && event.target.closest("[data-workspace-terminal]")) return;
    if (document.querySelector('dialog[open], .workspace-switcher [role="dialog"]')) return;
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
    // Content width accounts for text zoom; viewport media queries alone do not.
    const shellSize = new ResizeObserver(entries => autoCollapsed = entries[0].contentRect.width < 1280);
    if (shellElement) shellSize.observe(shellElement);
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
      shellSize.disconnect();
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
        const observedCatalog = await backend.catalogFingerprint().catch(() => null);
        const refreshed = await refreshSnapshot();
        if (refreshed && observedCatalog !== null) catalogFingerprint = observedCatalog;
        const openBehavior =
          typeof localStorage !== "undefined" &&
          localStorage.getItem(OPEN_BEHAVIOR_KEY) === "picker"
            ? "picker"
            : "last";
        if (openBehavior === "picker") {
          if (!routeOverride && route === "work") navigate("setup", "workspaces");
        } else if (!activeDomainId) {
          // Restore only a reference validated against the new snapshot. Never
          // restore processes, input permission, a plan, or freshness authority.
          let remembered = null;
          try { remembered = !routeOverride ? restoreSelectedWork(localStorage.getItem(SELECTED_WORK_KEY), snapshot.runs) : null; }
          catch { /* Storage may be unavailable. */ }
          if (remembered) {
            focusRunId = remembered.id;
            focusDomainId = remembered.domain_id;
          }
          const preferred =
            (remembered && snapshot.domains.some(d => d.domain_id === remembered.domain_id) ? remembered.domain_id : null) ??
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
          RECOVERY_AUDIT_MS,
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

<div class="desktop2 deck-scroll" bind:this={shellElement} class:sidebar-collapsed={sidebarCollapsed}>
  <Sidebar
    {route}
    {primary}
    {system}
    {recents}
    collapsed={sidebarCollapsed}
    {autoCollapsed}
    {tier}
    {signedIn}
    onNavigate={(next) => navigate(next)}
    onToggleCollapse={toggleSidebar}
    onOpenCommand={() => (commandOpen = true)}
    onOpenRecent={openRecent}
    onAccountClick={() => navigate("setup", "account")}
    onNewRun={openCompose}
    hasDraft={!!(composerDraft ?? workspaceDrafts.get(activeDomainId ?? ""))?.mission.trim()}
    activeRunsCount={activeCommandRuns.length}
    workspaceLabel={activeDomain?.repo_root.split(/[\\/]/).pop() ?? "Workspace"}
    onOpenWorkspace={() => navigate("setup", "workspaces")}
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
    <MissionDock bind:this={missionDock} {backend} {snapshot} {activeDomainId} run={dockRun}
      surface={route === "work" && workPane === "active" ? "run" : route === "work" && workPane === "review" ? "review" : "other"}
      previewAllowed={route === "work" && !approvalsOpen && !editingWorkspaceId}
      onReview={(runId, domainId) => openMission(runId, "review", domainId)} onOpenApprovals={() => approvalsOpen = true}>
    <div class="content" class:work-content={route === "work"} class:history-content={route === "history"} class:setup-content={route === "setup"} bind:this={contentElement}>
      {#if authErrorMessage}<div class="status-banner error-banner">{authErrorMessage}</div>{/if}
      {#if workspaceError}<div class="status-banner error-banner">{workspaceError}</div>{/if}
      {#if workspaceMessage}<div class="status-banner" role="status"><span>{workspaceMessage}</span><button aria-label="Dismiss workspace message" onclick={() => workspaceMessage = ""}>×</button></div>{/if}
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
        <div class="loading-state" role="status" aria-label="Loading workspace">
          <span>Loading workspace…</span>
          <div class="loading-summary" aria-hidden="true"></div>
          <div class="loading-tasks" aria-hidden="true"></div>
          <div class="loading-boundary" aria-hidden="true"></div>
        </div>
      {:else if route === "work" && workPane === "active"}
        <WorkActive
          bind:this={workScreen}
          {snapshot}
          {backend}
          activeDomainLabel={activeDomain ? (activeDomain.repo_root.split(/[\\/]/).pop() ?? null) : null}
          {activeDomainId}
          {focusRunId}
          onNewRun={openWorkspaceCompose}
          onOpenApprovals={() => (approvalsOpen = true)}
          onReviewRun={(runId) => openMission(runId, "review")}
          onStopRun={stopRunFromOps}
          onSelectRun={focusRun}
          onInspect={(ref, position) => missionDock?.open(ref, position)}
          onDismissInspect={() => missionDock?.dismiss("right")}
        />
      {:else if route === "history"}
        <HistoryScreen
          {snapshot}
          {backend}
          {activeDomainId}
          {focusRunId}
          onOpenRun={(runId, domainId) => openMission(runId, "review", domainId)}
          onSelectRun={(runId, domainId) => { focusRunId = runId; focusDomainId = domainId ?? null; }}
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
          {composerDraft}
          {preferredAdeId}
          onDraftChange={retainComposerDraft}
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
            <AgentsScreen embedded {backend} onUseInMission={composeWithAgent} />
          {/snippet}
        </SettingsScreen>
      {/if}
    </div>
    </MissionDock>
  </main>

  <CommandPalette open={commandOpen} items={commandItems} onNavigate={navigate} onClose={() => (commandOpen = false)} />

  <ApprovalsInbox
    open={approvalsOpen}
    {snapshot}
    {backend}
    onClose={() => (approvalsOpen = false)}
    onReviewRun={(runId) => openMission(runId, "review")}
    onApprovalsChanged={async () => { await refreshSnapshot(); }}
  />

  {#if editingDomain}
    <WorkspaceSettingsPanel
      domain={editingDomain}
      onClose={() => (editingWorkspaceId = null)}
      onForgotten={onDomainForgotten}
      onChanged={async () => { await refreshSnapshot(); }}
    />
  {/if}
</div>

<style>
  .desktop2 {
    display: grid;
    grid-template-columns: 184px minmax(0, 1fr);
    height: 100%;
    background: var(--pytxo-surface-shell);
    color: var(--pytxo-text-strong);
    font-family: "Satoshi", "Sora", "IBM Plex Sans", ui-sans-serif, system-ui, sans-serif;
  }
  .desktop2.sidebar-collapsed {
    grid-template-columns: 58px minmax(0, 1fr);
  }
  main {
    min-width: 0;
    min-height: 0;
    display: flex;
    flex-direction: column;
    background: var(--pytxo-surface-shell);
  }
  .content {
    flex: 1;
    min-height: 0;
    overflow: auto;
    scrollbar-gutter: stable;
    scroll-padding-block: 20px;
  }
  .history-content { display: flex; flex-direction: column; container: history-viewport / inline-size; }
  /* Work owns a viewport. Canvas, plan, details, and output own their own scroll. */
  .content.work-content { display:flex;min-height:0;flex-direction:column;overflow:hidden;scrollbar-gutter:auto; }
  .work-content > .status-banner { flex-shrink:0; }
  /* Setup owns two sibling scroll areas; shell chrome must not scroll with them. */
  .content.setup-content { display: flex; flex-direction: column; overflow: hidden; scrollbar-gutter: auto; }
  .setup-content > .status-banner { flex-shrink: 0; }
  .history-content > .status-banner { flex-shrink: 0; }
  .loading-state {
    display: grid;
    grid-template-columns: minmax(0, 1.6fr) minmax(260px, 1fr);
    gap: 12px;
    padding: 30px 32px;
  }
  .loading-state > span { grid-column: 1 / -1; color: var(--pytxo-text-muted); font-size: 13px; }
  .loading-state div {
    height: 240px;
    border: 1px solid var(--pytxo-line-soft);
    border-radius: var(--pytxo-panel-radius);
    background: var(--pytxo-surface-panel);
    animation: pulse 1.5s ease-in-out infinite alternate;
  }
  .loading-state .loading-summary { grid-column: 1 / -1; height: 80px; }
  @keyframes pulse {
    to {
      opacity: .45;
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

  @media (max-width: 1279px) {
    .desktop2:not(.sidebar-collapsed) {
      grid-template-columns: 60px minmax(0, 1fr);
    }
  }
  @media (max-width: 760px) {
    .desktop2:not(.sidebar-collapsed) {
      grid-template-columns: 60px minmax(0, 1fr);
    }
    .loading-state { grid-template-columns: 1fr; padding: 24px 16px; }
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
