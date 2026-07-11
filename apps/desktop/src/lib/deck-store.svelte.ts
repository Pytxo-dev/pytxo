import type { AgentDto, ProjectRootDto, RunDto, StructuralGraphDto } from "./types";

export const TABS_STORAGE_KEY = "pytxo-deck-tabs-v1";

export type WorkspaceTab = {
  id: string;
  domainId: string;
  label: string;
  /** Modular project id when this tab is a multi-root Workspace. */
  projectId: string | null;
  roots: ProjectRootDto[];
  runs: RunDto[];
  agents: AgentDto[];
  selectedRunId: string | null;
  selectedAgentId: string | null;
  structural: StructuralGraphDto | null;
  dispatchRepo: string;
};

export function tabLabelFromPath(path: string): string {
  return path.split(/[/\\]/).filter(Boolean).pop() ?? path;
}

export function createWorkspaceTab(domainId: string): WorkspaceTab {
  return {
    id: crypto.randomUUID(),
    domainId,
    label: tabLabelFromPath(domainId),
    projectId: null,
    roots: [],
    runs: [],
    agents: [],
    selectedRunId: null,
    selectedAgentId: null,
    structural: null,
    dispatchRepo: "",
  };
}

type PersistedTabs = {
  tabs: Array<{
    id: string;
    domainId: string;
    label: string;
    projectId?: string | null;
    selectedRunId: string | null;
    selectedAgentId: string | null;
    dispatchRepo: string;
  }>;
  activeTabId: string | null;
};

export function loadPersistedTabs(): PersistedTabs {
  if (typeof localStorage === "undefined") {
    return { tabs: [], activeTabId: null };
  }
  try {
    const raw = localStorage.getItem(TABS_STORAGE_KEY);
    if (!raw) return { tabs: [], activeTabId: null };
    const parsed = JSON.parse(raw) as PersistedTabs;
    if (!parsed || !Array.isArray(parsed.tabs)) {
      return { tabs: [], activeTabId: null };
    }
    return parsed;
  } catch {
    return { tabs: [], activeTabId: null };
  }
}

export function persistTabs(tabs: WorkspaceTab[], activeTabId: string | null) {
  if (typeof localStorage === "undefined") return;
  const payload: PersistedTabs = {
    tabs: tabs.map((t) => ({
      id: t.id,
      domainId: t.domainId,
      label: t.label,
      projectId: t.projectId,
      selectedRunId: t.selectedRunId,
      selectedAgentId: t.selectedAgentId,
      dispatchRepo: t.dispatchRepo,
    })),
    activeTabId,
  };
  localStorage.setItem(TABS_STORAGE_KEY, JSON.stringify(payload));
}

export function hydrateTabsFromStorage(): { tabs: WorkspaceTab[]; activeTabId: string | null } {
  const persisted = loadPersistedTabs();
  const tabs = persisted.tabs.map((t) => ({
    ...createWorkspaceTab(t.domainId),
    id: t.id,
    label: t.label || tabLabelFromPath(t.domainId),
    projectId: t.projectId ?? null,
    selectedRunId: t.selectedRunId,
    selectedAgentId: t.selectedAgentId,
    dispatchRepo: t.dispatchRepo,
  }));
  return { tabs, activeTabId: persisted.activeTabId };
}
