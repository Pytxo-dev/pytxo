/** In-memory inspection context only. Never carries approval or candidate authority. */
export type WorkbenchCamera = { x: number; y: number; scale: number; mode?: "fit" | "manual" };
export type WorkbenchSelection = { taskId?: string; path?: string; camera?: WorkbenchCamera };
const selections = new Map<string, WorkbenchSelection>();
const key = (runId: string, domainId: string | null) => JSON.stringify([domainId, runId]);
export function workbenchSelection(runId: string, domainId: string | null): WorkbenchSelection {
  return selections.get(key(runId, domainId)) ?? {};
}
export function rememberWorkbenchSelection(runId: string, domainId: string | null, selection: WorkbenchSelection) {
  const id = key(runId, domainId);
  const previous = selections.get(id) ?? {};
  selections.delete(id);
  selections.set(id, { ...previous, ...selection });
  if (selections.size > 100) selections.delete(selections.keys().next().value!);
}
