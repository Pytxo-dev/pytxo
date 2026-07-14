export type AppRoute =
  | "operations"
  | "workspaces"
  | "runs"
  | "flow"
  | "approvals"
  | "integrations"
  | "settings"
  | "topology-focus"
  | "run-review";

export const ROUTE_LABELS: Record<AppRoute, string> = {
  operations: "Operations",
  workspaces: "Workspaces",
  runs: "Runs",
  flow: "Flow",
  approvals: "Approvals",
  integrations: "Integrations",
  settings: "Settings",
  "topology-focus": "Topology Focus",
  "run-review": "Run Review",
};

const ROUTE_KEY = "pytxo-desktop-route-v2";
const RECENTS_KEY = "pytxo-desktop-recents-v2";

function isRoute(value: string | null): value is AppRoute {
  return value !== null && value in ROUTE_LABELS;
}

export function routeFromDeepLink(value: string): AppRoute | null {
  try {
    const parsed = new URL(value);
    const target = parsed.hostname || parsed.pathname.replace(/^\//, "");
    return isRoute(target) ? target : null;
  } catch {
    return null;
  }
}

export function initialRoute(): AppRoute {
  const hash = window.location.hash.replace(/^#\/?/, "");
  if (isRoute(hash)) return hash;
  const stored = localStorage.getItem(ROUTE_KEY);
  return isRoute(stored) ? stored : "operations";
}

export function persistRoute(route: AppRoute) {
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
