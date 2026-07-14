import { ipc } from "./ipc";
import {
  createWorkspaceTab,
  tabLabelFromPath,
  type WorkspaceTab,
} from "./deck-store.svelte";
import type { ProjectDto, ProjectRootDto } from "./types";

export type WorkspaceListItem = {
  kind: "project" | "domain";
  id: string;
  label: string;
  path: string;
  projectId: string | null;
};

/** Open a single folder as a simple one-root workspace tab. */
export async function openFolderAsWorkspace(
  path: string,
  tabs: WorkspaceTab[],
): Promise<{ tabs: WorkspaceTab[]; tabId: string }> {
  const domainId = await ipc.ensureWorkspace(path);
  const existing = tabs.find((t) => t.domainId === domainId && !t.projectId);
  if (existing) {
    return { tabs, tabId: existing.id };
  }
  const tab = createWorkspaceTab(domainId);
  return { tabs: [...tabs, tab], tabId: tab.id };
}

/** Open a modular project: primary root becomes the domain; roots load into the tab. */
export async function openProjectAsWorkspace(
  project: ProjectDto,
  tabs: WorkspaceTab[],
): Promise<{ tabs: WorkspaceTab[]; tabId: string }> {
  const existing = tabs.find((t) => t.projectId === project.id);
  if (existing) {
    return { tabs, tabId: existing.id };
  }

  const roots = await ipc.projectRoots(project.id);
  const primary = roots.find((r) => r.primary) ?? roots[0];
  if (!primary) {
    throw new Error(`Project ${project.id} has no path roots`);
  }

  const domainId = await ipc.ensureWorkspace(primary.path);
  const tab: WorkspaceTab = {
    ...createWorkspaceTab(domainId),
    projectId: project.id,
    label: project.id,
    roots,
  };
  return { tabs: [...tabs, tab], tabId: tab.id };
}

export async function loadWorkspaceCatalog(): Promise<WorkspaceListItem[]> {
  const [projects, domains] = await Promise.all([
    ipc.listProjects(),
    ipc.listDomainsStatus().catch(() => []),
  ]);

  const items: WorkspaceListItem[] = [];
  const projectIds = new Set(projects.map((p) => p.id));

  for (const p of projects) {
    items.push({
      kind: "project",
      id: p.id,
      label: p.id,
      path: p.manifest_path,
      projectId: p.id,
    });
  }

  for (const d of domains) {
    if (!d.is_available || d.is_temporary) continue;
    if (d.project_id && projectIds.has(d.project_id)) continue;
    items.push({
      kind: "domain",
      id: d.domain_id,
      label: tabLabelFromPath(d.repo_root || d.domain_id),
      path: d.repo_root || d.domain_id,
      projectId: d.project_id,
    });
  }

  return items;
}

export async function refreshTabRoots(tab: WorkspaceTab): Promise<ProjectRootDto[]> {
  if (!tab.projectId) return tab.roots;
  return ipc.projectRoots(tab.projectId);
}
