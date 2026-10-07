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
      run.routing_revision ?? "",
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

export type DomainChangesBatchLoader = (
  requests: Array<{ domain_id: string; cursor: number }>,
  limit: number,
) => Promise<DomainCursorPage[]>;

export function incrementalDomainBatchSize(
  domainCount: number,
  intervalMs: number,
  coverageMs = 60_000,
): number {
  if (domainCount <= 0) return 0;
  return Math.max(1, Math.ceil(domainCount * intervalMs / coverageMs));
}

/**
 * Keep active and selected work current while spreading dormant store checks
 * across the requested coverage window. Every dormant domain is eventually
 * visited, but a large completed catalog no longer lands on one idle frame.
 */
export function selectIncrementalDomainBatch(
  domainIds: string[],
  urgentDomainIds: Iterable<string>,
  dormantOffset: number,
  dormantBatchSize: number,
): { domainIds: string[]; nextDormantOffset: number } {
  const known = new Set(domainIds);
  const urgent = [...new Set(urgentDomainIds)].filter(domainId => known.has(domainId));
  const urgentSet = new Set(urgent);
  const dormant = domainIds.filter(domainId => !urgentSet.has(domainId));
  if (dormant.length === 0 || dormantBatchSize <= 0) {
    return { domainIds: urgent, nextDormantOffset: 0 };
  }
  const start = ((dormantOffset % dormant.length) + dormant.length) % dormant.length;
  const count = Math.min(dormantBatchSize, dormant.length);
  const selected = Array.from(
    { length: count },
    (_, index) => dormant[(start + index) % dormant.length],
  );
  return {
    domainIds: [...urgent, ...selected],
    nextDormantOffset: (start + count) % dormant.length,
  };
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
  loadBatch?: DomainChangesBatchLoader,
): Promise<{ changed: boolean; needsSnapshot: boolean }> {
  let changed = false;
  let needsSnapshot = false;
  // Bound each native payload. Follow-up pages keep the existing cursor loop.
  for (let offset = 0; offset < domainIds.length; offset += 128) {
    const chunk = domainIds.slice(offset, offset + 128);
    const firstPages = loadBatch ? await loadBatch(
      chunk.map(domain_id => ({ domain_id, cursor: cursors.get(domain_id) ?? 0 })), limit,
    ) : null;
    if (firstPages && firstPages.length !== chunk.length) {
      throw new Error("Incomplete domain change batch");
    }
    for (const [index, domainId] of chunk.entries()) {
      let firstPage = firstPages?.[index];
      let cursor = cursors.get(domainId) ?? 0;
      let continueCatchUp = true;
      while (continueCatchUp) {
        const page = firstPage ?? await loadPage(domainId, cursor, limit);
        firstPage = undefined;
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
  loadBatch?: DomainChangesBatchLoader,
): Promise<T> {
  let snapshot = await loadSnapshot();
  const domainIds = snapshot.domains.map((domain) => domain.domain_id);
  const newlyDiscovered = domainIds.filter((domainId) => !cursors.has(domainId));
  if (newlyDiscovered.length > 0) {
    await consumeDomainChanges(newlyDiscovered, cursors, loadPage, 1000, loadBatch);
    snapshot = await loadSnapshot();
  }

  for (let pass = 0; pass < 4; pass += 1) {
    const currentIds = snapshot.domains.map((domain) => domain.domain_id);
    for (const domainId of [...cursors.keys()]) {
      if (!currentIds.includes(domainId)) cursors.delete(domainId);
    }
    const result = await consumeDomainChanges(currentIds, cursors, loadPage, 200, loadBatch);
    if (!result.changed && !result.needsSnapshot) return snapshot;
    snapshot = await loadSnapshot();
  }
  return snapshot;
}
