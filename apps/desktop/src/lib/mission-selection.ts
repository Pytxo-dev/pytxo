import type { RunDto } from "./types";
export const SELECTED_WORK_KEY = "pytxo-selected-work-v1";
/** A saved view reference is not execution or review authority. */
export function restoreSelectedWork(raw: string | null, runs: RunDto[]): RunDto | null {
  try {
    const saved = JSON.parse(raw ?? "null");
    if (!saved || typeof saved.runId !== "string" || typeof saved.domainId !== "string") return null;
    return runs.find(run => run.id === saved.runId && run.domain_id === saved.domainId) ?? null;
  } catch { return null; }
}
/** The central mission and contextual view actions must resolve the same run. */
export function selectMissionRun(all: RunDto[], domainId: string | null, focusRunId: string | null): RunDto | null {
  const runs = domainId ? all.filter(r => r.domain_id === domainId) : all;
  return runs.find(r => r.id === focusRunId) ?? runs.find(r => ["starting", "running", "pending", "dispatching", "active"].includes(r.status.toLowerCase())) ?? runs[0] ?? null;
}
