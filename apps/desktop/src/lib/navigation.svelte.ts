export type CanonicalRoute =
  | "operations"
  | "workspaces"
  | "missions"
  | "approvals"
  | "agents"
  | "settings";

/** Canonical destinations plus hash aliases kept for deep links and tests. */
export type AppRoute =
  | CanonicalRoute
  | "flow"
  | "runs"
  | "integrations"
  | "topology-focus"
  | "run-review";

export type MissionView = "list" | "compose" | "detail";
export type MissionPane = "plan" | "live" | "review";

export type ResolvedRoute = {
  route: CanonicalRoute;
  missionView: MissionView;
  missionPane: MissionPane;
};

export const ROUTE_LABELS: Record<CanonicalRoute, string> = {
  operations: "Ops",
  workspaces: "Workspaces",
  missions: "Missions",
  approvals: "Approvals",
  agents: "Agents",
  settings: "Settings",
};

const ROUTE_KEY = "pytxo-desktop-route-v2";
const RECENTS_KEY = "pytxo-desktop-recents-v2";

const ALIAS_TO_CANONICAL: Record<string, ResolvedRoute> = {
  operations: { route: "operations", missionView: "list", missionPane: "live" },
  workspaces: { route: "workspaces", missionView: "list", missionPane: "live" },
  missions: { route: "missions", missionView: "list", missionPane: "live" },
  approvals: { route: "approvals", missionView: "list", missionPane: "live" },
  agents: { route: "agents", missionView: "list", missionPane: "live" },
  settings: { route: "settings", missionView: "list", missionPane: "live" },
  flow: { route: "missions", missionView: "compose", missionPane: "plan" },
  runs: { route: "missions", missionView: "list", missionPane: "live" },
  integrations: { route: "agents", missionView: "list", missionPane: "live" },
  "topology-focus": { route: "missions", missionView: "detail", missionPane: "plan" },
  "run-review": { route: "missions", missionView: "detail", missionPane: "review" },
};

export function resolveRoute(value: string | null | undefined): ResolvedRoute {
  if (value && value in ALIAS_TO_CANONICAL) return ALIAS_TO_CANONICAL[value];
  return ALIAS_TO_CANONICAL.operations;
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

export type SettingsSectionId =
  | "general"
  | "appearance"
  | "keyboard"
  | "providers"
  | "workspaces"
  | "agents"
  | "voice"
  | "privacy"
  | "account";

const SETTINGS_SECTION_KEY = "pytxo-desktop-settings-section-v1";

export function loadSettingsSection(): SettingsSectionId {
  const raw = localStorage.getItem(SETTINGS_SECTION_KEY);
  if (
    raw === "general" ||
    raw === "appearance" ||
    raw === "keyboard" ||
    raw === "providers" ||
    raw === "workspaces" ||
    raw === "agents" ||
    raw === "voice" ||
    raw === "privacy" ||
    raw === "account"
  ) {
    return raw;
  }
  return "appearance";
}

export function persistSettingsSection(section: SettingsSectionId) {
  localStorage.setItem(SETTINGS_SECTION_KEY, section);
}
