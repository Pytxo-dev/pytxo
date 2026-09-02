import type { DesktopSnapshot } from "./desktop-backend";
import { advanceDomainCursor, type DomainCursorPage } from "./review-state";

export function fingerprintDesktopSnapshot(snapshot: DesktopSnapshot): string {
  const domains = snapshot.domains.map((domain) => domain.domain_id).join(",");
  const runs = snapshot.runs
    .map((run) => [
      run.domain_id,
      run.id,
      run.status,
      run.estimated_cost_usd ?? 0,
      run.apply_status ?? "",
      run.applied_at ?? "",
      run.prepared_digest ?? "",
      run.prepared_at ?? "",
      run.last_apply_error?.at ?? "",
      run.last_apply_error?.code ?? "",
      run.last_apply_error?.attempt_id ?? "",
      run.last_apply_error?.rollback_confirmed ? "rolled-back" : "",
      run.recovery_state ?? "",
    ].join(":"))
    .join(",");
  const agents = snapshot.agents.map((agent) => `${agent.domain_id}:${agent.id}:${agent.status}`).join(",");
  const approvals = snapshot.approvals.map((approval) => approval.id).join(",");
  const fleets = snapshot.fleets.map((fleet) => `${fleet.id}:${fleet.status}`).join(",");
  const diagnostics = snapshot.diagnostics
    .map((diagnostic) => `${diagnostic.domain_id}:${diagnostic.stage}:${diagnostic.run_id ?? ""}:${diagnostic.message}`)
    .join(",");
  return `${domains}|${runs}|${agents}|${approvals}|${fleets}|${diagnostics}|${snapshot.error?.message ?? ""}`;
}

export async function consumeDomainChanges(
  domainIds: string[],
  cursors: Map<string, number>,
  loadPage: (
    domainId: string,
    cursor: number,
    limit: number,
  ) => Promise<DomainCursorPage>,
  limit = 200,
): Promise<{ changed: boolean; needsSnapshot: boolean }> {
  let changed = false;
  let needsSnapshot = false;
  for (const domainId of domainIds) {
    let cursor = cursors.get(domainId) ?? 0;
    let continueCatchUp = true;
    while (continueCatchUp) {
      const page = await loadPage(domainId, cursor, limit);
      changed ||= page.changes.length > 0;
      const advanced = advanceDomainCursor(cursor, page);
      cursors.set(domainId, advanced.cursor);
      needsSnapshot ||= advanced.needsSnapshot;
      if (advanced.needsSnapshot) break;
      if (advanced.continueCatchUp && advanced.cursor === cursor) {
        throw new Error(`domain change cursor did not advance for ${domainId}`);
      }
      cursor = advanced.cursor;
      continueCatchUp = advanced.continueCatchUp;
    }
  }
  return { changed, needsSnapshot };
}

/**
 * Establishes a cursor boundary before accepting the returned snapshot.
 *
 * A newly discovered domain is first advanced to a durable boundary, then the
 * snapshot is reloaded. Changes that race that reload are consumed only when a
 * subsequent snapshot reload will include them, so no unseen mutation is
 * silently marked as observed.
 */
export async function loadConsistentDesktopSnapshot<
  T extends { domains: Array<{ domain_id: string }> },
>(
  loadSnapshot: () => Promise<T>,
  loadPage: (
    domainId: string,
    cursor: number,
    limit: number,
  ) => Promise<DomainCursorPage>,
  cursors: Map<string, number>,
): Promise<T> {
  let snapshot = await loadSnapshot();
  const domainIds = snapshot.domains.map((domain) => domain.domain_id);
  const newlyDiscovered = domainIds.filter((domainId) => !cursors.has(domainId));
  if (newlyDiscovered.length > 0) {
    await consumeDomainChanges(newlyDiscovered, cursors, loadPage, 1000);
    snapshot = await loadSnapshot();
  }

  for (let pass = 0; pass < 4; pass += 1) {
    const currentIds = snapshot.domains.map((domain) => domain.domain_id);
    for (const domainId of [...cursors.keys()]) {
      if (!currentIds.includes(domainId)) cursors.delete(domainId);
    }
    const result = await consumeDomainChanges(currentIds, cursors, loadPage);
    if (!result.changed && !result.needsSnapshot) return snapshot;
    snapshot = await loadSnapshot();
  }
  return snapshot;
}
