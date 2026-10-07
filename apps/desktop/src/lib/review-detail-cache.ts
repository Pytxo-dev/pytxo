import type { DesktopBackend } from "./desktop-backend";
import type { RunDto, RunReviewDto } from "./types";

const revisionRequests = new WeakMap<DesktopBackend, Map<string, Promise<RunReviewDto>>>();

export function reviewRevisionKey(run: RunDto): string {
  const revision = JSON.stringify([
    run.prepared_digest,
    run.prepared_at,
    run.apply_status,
    run.applied_at,
    run.recovery_state,
    run.last_apply_error?.attempt_id,
  ]);
  return `${run.domain_id}\u0000${run.id}\u0000${revision}`;
}

/**
 * Share one review read across every inspector for the same backend and
 * candidate revision. Rejected reads are evicted so a later inspection can
 * recover normally.
 */
export function readReviewRevision(
  backend: DesktopBackend,
  key: string,
  runId: string,
  domainId: string,
): Promise<RunReviewDto> {
  let backendRequests = revisionRequests.get(backend);
  if (!backendRequests) {
    backendRequests = new Map();
    revisionRequests.set(backend, backendRequests);
  }
  const existing = backendRequests.get(key);
  if (existing) return existing;

  const request = backend.runReview(runId, domainId).catch((error: unknown) => {
    if (backendRequests?.get(key) === request) backendRequests.delete(key);
    throw error;
  });
  backendRequests.set(key, request);
  return request;
}
