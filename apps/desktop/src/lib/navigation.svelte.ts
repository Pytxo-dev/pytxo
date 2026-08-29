/**
 * Three top-level destinations, matching the three things an operator actually
 * does: run work, investigate what already happened, and configure the machine.
 *
 * Workspace is global context (title bar), not a destination. Approvals are an
 * overlay, not a destination — a two-item queue does not earn a screen, and a
 * decision should be reachable from wherever the operator already is.
 */
export type CanonicalRoute = "work" | "history" | "setup";

/**
 * Canonical destinations plus the hash aliases retained for deep links, stored
 * routes written by earlier versions, and existing tests.
 */
export type AppRoute =
  | CanonicalRoute
  | "operations"
  | "workspaces"
  | "missions"
  | "approvals"
  | "agents"
  | "settings"
  | "flow"
  | "runs"
  | "integrations"
  | "topology-focus"
  | "run-review";

/** The loop: watch it run, plan the next one, review what it produced. */
export type WorkPane = "active" | "plan" | "review";

/**
 * Sub-views of the run surface. These are panes within Work and History, not
 * destinations — a run inventory and a run detail are the same subject at two
 * zoom levels.
 */
export type MissionView = "list" | "compose" | "detail";
export type MissionPane = "plan" | "live" | "review";

export type SetupSectionId =
  | "general"
  | "appearance"
  | "keyboard"
  | "providers"
  | "workspaces"
  | "agents"
  | "voice"
  | "privacy"
  | "account";

/** @deprecated Kept as an alias while `SettingsScreen` is the Setup surface. */
export type SettingsSectionId = SetupSectionId;

export type ResolvedRoute = {
  route: CanonicalRoute;
  workPane: WorkPane;
  setupSection: SetupSectionId | null;
  /** Legacy `#/approvals` links open the inbox overlay rather than a screen. */
  openApprovals: boolean;
};

export const ROUTE_LABELS: Record<CanonicalRoute, string> = {
  work: "Work",
  history: "History",
  setup: "Setup",
};

const ROUTE_KEY = "pytxo-desktop-route-v3";
const RECENTS_KEY = "pytxo-desktop-recents-v2";

function target(
  route: CanonicalRoute,
  workPane: WorkPane = "active",
  setupSection: SetupSectionId | null = null,
  openApprovals = false,
): ResolvedRoute {
  return { route, workPane, setupSection, openApprovals };
}

const ALIAS_TO_CANONICAL: Record<AppRoute, ResolvedRoute> = {
  work: target("work", "active"),
  history: target("history"),
  setup: target("setup"),

  // Retired destinations, resolved to their new home.
  operations: target("work", "active"),
  flow: target("work", "plan"),
  "run-review": target("work", "review"),
  "topology-focus": target("work", "review"),
  missions: target("history"),
  runs: target("history"),
  approvals: target("work", "active", null, true),
  workspaces: target("setup", "active", "workspaces"),
  agents: target("setup", "active", "agents"),
  integrations: target("setup", "active", "agents"),
  settings: target("setup"),
};

export function resolveRoute(value: string | null | undefined): ResolvedRoute {
  if (value && value in ALIAS_TO_CANONICAL) return ALIAS_TO_CANONICAL[value as AppRoute];
  return ALIAS_TO_CANONICAL.work;
}

export function isAppRoute(value: string | null): value is AppRoute {
  return value !== null && value in ALIAS_TO_CANONICAL;
}

export function routeFromDeepLink(value: string): AppRoute | null {
  try {
    const parsed = new URL(value);
    const target = parsed.hostname || parsed.pathname.replace(/^\//, "");
    return isAppRoute(target) ? target : null;
  } catch {
    return null;
  }
}

export function initialResolvedRoute(): ResolvedRoute {
  const hash = window.location.hash.replace(/^#\/?/, "");
  if (hash && hash in ALIAS_TO_CANONICAL) return resolveRoute(hash);
  const stored = localStorage.getItem(ROUTE_KEY);
  return resolveRoute(stored);
}

export function persistRoute(route: CanonicalRoute) {
  localStorage.setItem(ROUTE_KEY, route);
  history.replaceState(null, "", `#/${route}`);
}

export type WorkspaceRecent = { id: string; label: string; domainId: string };

const MAX_RECENTS = 8;

/** Record a just-opened workspace as most-recent, deduplicated by domainId. */
export function addWorkspaceRecent(recent: WorkspaceRecent): WorkspaceRecent[] {
  const existing = migrateWorkspaceRecents().filter((r) => r.domainId !== recent.domainId);
  const next = [recent, ...existing].slice(0, MAX_RECENTS);
  localStorage.setItem(RECENTS_KEY, JSON.stringify(next));
  return next;
}

export function migrateWorkspaceRecents(): WorkspaceRecent[] {
  const current = localStorage.getItem(RECENTS_KEY);
  if (current) {
    try {
      return JSON.parse(current) as WorkspaceRecent[];
    } catch {
      return [];
    }
  }
  const legacy = localStorage.getItem("pytxo-deck-tabs-v1");
  if (!legacy) return [];
  try {
    const parsed = JSON.parse(legacy) as {
      tabs?: { id: string; label: string; domainId: string }[];
    };
    const recents = (parsed.tabs ?? []).map(({ id, label, domainId }) => ({ id, label, domainId }));
    localStorage.setItem(RECENTS_KEY, JSON.stringify(recents));
    return recents;
  } catch {
    return [];
  }
}

/** Drop a forgotten domain from the recent list. */
export function removeWorkspaceRecent(domainId: string): WorkspaceRecent[] {
  const next = migrateWorkspaceRecents().filter((r) => r.domainId !== domainId);
  localStorage.setItem(RECENTS_KEY, JSON.stringify(next));
  return next;
}

const SETUP_SECTION_KEY = "pytxo-desktop-settings-section-v1";

const SETUP_SECTIONS: readonly SetupSectionId[] = [
  "general",
  "appearance",
  "keyboard",
  "providers",
  "workspaces",
  "agents",
  "voice",
  "privacy",
  "account",
];

export function isSetupSection(value: string | null): value is SetupSectionId {
  return value !== null && (SETUP_SECTIONS as readonly string[]).includes(value);
}

export function loadSettingsSection(): SetupSectionId {
  const raw = localStorage.getItem(SETUP_SECTION_KEY);
  return isSetupSection(raw) ? raw : "appearance";
}

export function persistSettingsSection(section: SetupSectionId) {
  localStorage.setItem(SETUP_SECTION_KEY, section);
}
