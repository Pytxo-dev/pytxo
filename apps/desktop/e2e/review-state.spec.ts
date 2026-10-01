import { expect, test } from "@playwright/test";
import {
  advanceDomainCursor,
  recordedWorkerLabel,
  reviewPresentation,
  type DomainCursorPage,
} from "../src/lib/review-state";
import {
  consumeDomainChanges,
  fingerprintDesktopSnapshot,
  incrementalDomainBatchSize,
  loadConsistentDesktopSnapshot,
  selectIncrementalDomainBatch,
} from "../src/lib/desktop-sync";
import type { DesktopSnapshot } from "../src/lib/desktop-backend";
import type { AgentDto } from "../src/lib/types";

test.describe("Recorded review ownership", () => {
  const run = { id: "run-native", domain_id: "fixture-b" };
  const worker = (task_id: string, index: number): AgentDto => ({
    id: `${run.id}:agent-${index}`, domain_id: run.domain_id, run_id: run.id,
    task_id, wave: index === 2 ? 1 : 0, status: "completed", exit_code: 0, root_id: null,
  });

  test("resolves actors by task identity across reordered waves and records", () => {
    // Plan order is implementation, tests, docs; runtime launches docs before tests.
    const agents = [worker("mission-1", 2), worker("mission-0", 0), worker("mission-2", 1)];
    expect(["mission-0", "mission-2", "mission-1"].map(task => recordedWorkerLabel(agents, run, task)))
      .toEqual(["agent-0", "agent-1", "agent-2"]);
  });

  test("does not borrow ownership from another run or execution domain", () => {
    const wrongRun = { ...worker("mission-2", 8), run_id: "another-run" };
    const wrongDomain = { ...worker("mission-2", 9), domain_id: "another-repo" };
    expect(recordedWorkerLabel([wrongRun, wrongDomain], run, "mission-2")).toBe("Worker not recorded");
    expect(recordedWorkerLabel([wrongRun, worker("mission-2", 1), wrongDomain], run, "mission-2"))
      .toBe("agent-1");
  });

  test("does not guess when a task has missing or ambiguous worker records", () => {
    expect(recordedWorkerLabel([], run, "mission-2")).toBe("Worker not recorded");
    expect(recordedWorkerLabel([worker("mission-2", 1), worker("mission-2", 2)], run, "mission-2"))
      .toBe("Worker not recorded");
    expect(recordedWorkerLabel([{ ...worker("mission-2", 1), id: `${run.id}:` }], run, "mission-2"))
      .toBe("Worker not recorded");
  });

  test("retains recorded identities without a run prefix", () => {
    expect(recordedWorkerLabel([{ ...worker("mission-2", 1), id: "documentation-worker" }], run, "mission-2"))
      .toBe("documentation-worker");
  });
});

test.describe("Run Review state contracts", () => {
  test("maps retryable rollback to recovered with one guarded retry", () => {
    expect(
      reviewPresentation({
        apply_status: "ready",
        recovery_state: "rolled_back",
        last_apply_error: {
          at: "2026-08-01T01:00:00Z",
          code: "apply_failed",
          message: "Apply failed and was rolled back.",
          attempt_id: "attempt-1",
          rollback_confirmed: true,
        },
        candidate_verified: true,
        prepared_file_count: 1,
      }),
    ).toMatchObject({
      state: "recovered",
      primaryAction: "retry",
      primaryLabel: "Retry Apply",
      applyAllowed: true,
      discardAllowed: true,
    });
  });

  test("requires refresh for stale and refuses Apply during recovery", () => {
    expect(
      reviewPresentation({
        apply_status: "stale",
        recovery_state: "source_drift",
        last_apply_error: null,
        candidate_verified: false,
        prepared_file_count: 1,
      }),
    ).toMatchObject({
      state: "stale",
      primaryAction: "refresh",
      applyAllowed: false,
      discardAllowed: true,
    });
    expect(
      reviewPresentation({
        apply_status: "recovery_required",
        recovery_state: "unprovable",
        last_apply_error: null,
        candidate_verified: false,
        prepared_file_count: 1,
      }),
    ).toMatchObject({
      state: "recovery_required",
      primaryAction: "reconcile",
      applyAllowed: false,
      discardAllowed: false,
    });
  });

  test("makes a v1 package upgrade failure visibly refreshable", () => {
    expect(
      reviewPresentation({
        apply_status: "review_failed",
        recovery_state: null,
        routing_revision: null,
        last_apply_error: {
          at: "2026-08-09T00:00:00Z",
          code: "review_package_upgrade_required",
          message: "Prepared review package version 1 requires an explicit refresh for Pytxo v1.1.",
          attempt_id: null,
          rollback_confirmed: false,
        },
        candidate_verified: false,
        prepared_file_count: null,
      }),
    ).toMatchObject({
      state: "review_failed",
      detail: "Prepared review package version 1 requires an explicit refresh for Pytxo v1.1.",
      primaryAction: "refresh",
      primaryLabel: "Retry preparation",
      applyAllowed: false,
      discardAllowed: true,
    });
  });

  test("requires combined-candidate evidence and refuses empty Apply", () => {
    expect(
      reviewPresentation({
        apply_status: "ready",
        recovery_state: null,
        last_apply_error: null,
        candidate_verified: false,
        prepared_file_count: 2,
      }),
    ).toMatchObject({
      state: "verification_required",
      title: "Verify before Apply",
      primaryAction: "refresh",
      primaryLabel: "Verify candidate",
      applyAllowed: false,
    });
    expect(
      reviewPresentation({
        apply_status: "ready",
        recovery_state: null,
        last_apply_error: null,
        candidate_verified: false,
        prepared_file_count: 0,
      }),
    ).toMatchObject({
      state: "nothing_to_apply",
      title: "Nothing to Apply",
      primaryAction: null,
      applyAllowed: false,
    });
  });

  test("advances, resets, and reconnects a domain cursor deterministically", () => {
    const page = (
      next_cursor: number,
      cursor_gap = false,
      has_more = false,
    ): DomainCursorPage => ({
      changes: [],
      next_cursor,
      cursor_gap,
      has_more,
    });

    expect(advanceDomainCursor(12, page(18))).toEqual({
      cursor: 18,
      needsSnapshot: false,
      continueCatchUp: false,
    });
    expect(advanceDomainCursor(18, page(2, true))).toEqual({
      cursor: 2,
      needsSnapshot: true,
      continueCatchUp: false,
    });
    expect(advanceDomainCursor(null, page(44, false, true))).toEqual({
      cursor: 44,
      needsSnapshot: false,
      continueCatchUp: true,
    });
  });

  test("contract-only snapshots have distinct fingerprints", () => {
    const snapshot = {
      domains: [],
      runs: [{
        domain_id: "C:/repo",
        id: "run-1",
        status: "completed",
        repo_root: "C:/repo",
        started_at: "2026-08-01T00:00:00Z",
        estimated_cost_usd: 0,
        permission_profile: "orbit",
        isolation_mode: "copy_on_write",
        isolation_backend: "git_worktree",
        apply_status: "ready",
        applied_at: null,
        prepared_digest: "digest-1",
        prepared_at: "2026-08-01T00:01:00Z",
        last_apply_error: null,
        recovery_state: null,
      }],
      agents: [],
      approvals: [],
      fleets: [],
      diagnostics: [],
      error: null,
    } satisfies DesktopSnapshot;
    const changed = structuredClone(snapshot);
    changed.runs[0].apply_status = "stale";
    changed.runs[0].last_apply_error = {
      at: "2026-08-01T00:02:00Z",
      code: "source_drift",
      message: "changed",
      attempt_id: null,
      rollback_confirmed: false,
    };
    changed.runs[0].recovery_state = "source_drift";

    expect(fingerprintDesktopSnapshot(changed)).not.toBe(
      fingerprintDesktopSnapshot(snapshot),
    );
    const routed = structuredClone(snapshot);
    routed.runs[0].routing_revision = "3";
    expect(fingerprintDesktopSnapshot(routed)).not.toBe(
      fingerprintDesktopSnapshot(snapshot),
    );
  });

  test("snapshot boundary reloads after a mutation races cursor priming", async () => {
    const snapshots = [
      { marker: "before", domains: [{ domain_id: "repo" }] },
      { marker: "after", domains: [{ domain_id: "repo" }] },
    ];
    let loads = 0;
    const cursors = new Map<string, number>();
    const result = await loadConsistentDesktopSnapshot(
      async () => snapshots[Math.min(loads++, snapshots.length - 1)],
      async (_domainId, cursor) => ({
        changes: cursor === 0
          ? [{ sequence: 1, entity_kind: "contract", entity_id: "run-1", changed_at: "now" }]
          : [],
        next_cursor: 1,
        has_more: false,
        cursor_gap: false,
      }),
      cursors,
    );

    expect(result.marker).toBe("after");
    expect(loads).toBeGreaterThanOrEqual(2);
    expect(cursors.get("repo")).toBe(1);
  });

  test("cursor reset and reconnect require a snapshot without swallowing the next delta", async () => {
    const cursors = new Map([["repo", 99]]);
    let nativeBoundary = 2;
    const nativeChanges: DomainCursorPage["changes"] = [];
    const loadNativePage = async (
      _domainId: string,
      cursor: number,
    ): Promise<DomainCursorPage> => {
      if (cursor > nativeBoundary) {
        return {
          changes: [],
          next_cursor: nativeBoundary,
          has_more: false,
          cursor_gap: true,
        };
      }
      const changes = nativeChanges.filter((change) => change.sequence > cursor);
      return {
        changes,
        next_cursor: changes.at(-1)?.sequence ?? cursor,
        has_more: false,
        cursor_gap: false,
      };
    };

    const reset = await consumeDomainChanges(
      ["repo"],
      cursors,
      loadNativePage,
    );
    expect(reset.needsSnapshot).toBe(true);
    expect(cursors.get("repo")).toBe(2);

    const settled = await consumeDomainChanges(["repo"], cursors, loadNativePage);
    expect(settled).toEqual({ changed: false, needsSnapshot: false });
    expect(cursors.get("repo")).toBe(2);

    nativeBoundary = 3;
    nativeChanges.push({
      sequence: 3,
      entity_kind: "contract",
      entity_id: "run-1",
      changed_at: "now",
    });
    const reconnect = await consumeDomainChanges(["repo"], cursors, loadNativePage);
    expect(reconnect.changed).toBe(true);
    expect(cursors.get("repo")).toBe(3);
  });
});


test("domain batches bound IPC calls and retain pagination and reset semantics", async () => {
  const ids = Array.from({ length: 149 }, (_, index) => `domain-${index}`);
  const cursors = new Map(ids.map(id => [id, 10]));
  const batches: number[] = [];
  const followups: string[] = [];
  const result = await consumeDomainChanges(ids, cursors, async (id, cursor) => {
    followups.push(id);
    expect(cursor).toBe(11);
    return { changes: [], next_cursor: 11, has_more: false, cursor_gap: false };
  }, 200, async requests => {
    batches.push(requests.length);
    return requests.map(({ domain_id, cursor }) => ({
      changes: domain_id === "domain-0" ? [{ sequence: 11, entity_kind: "run", entity_id: "run", changed_at: "now" }] : [],
      next_cursor: domain_id === "domain-0" ? 11 : domain_id === "domain-148" ? 0 : cursor,
      has_more: domain_id === "domain-0",
      cursor_gap: domain_id === "domain-148",
    }));
  });
  expect(batches).toEqual([128, 21]);
  expect(followups).toEqual(["domain-0"]);
  expect(cursors.get("domain-148")).toBe(0);
  expect(result).toEqual({ changed: true, needsSnapshot: true });
});

test("incremental polling prioritizes live work and cycles through a large dormant catalog", () => {
  const ids = Array.from({ length: 149 }, (_, index) => `domain-${index}`);
  const urgent = ["domain-148", "domain-77", "missing-domain", "domain-148"];
  const idleBatchSize = incrementalDomainBatchSize(ids.length, 8_000);
  const activeBatchSize = incrementalDomainBatchSize(ids.length, 2_500);
  expect(idleBatchSize).toBe(20);
  expect(activeBatchSize).toBe(7);

  let offset = 0;
  const observed = new Set<string>();
  for (let pass = 0; pass < 8; pass += 1) {
    const selected = selectIncrementalDomainBatch(ids, urgent, offset, idleBatchSize);
    offset = selected.nextDormantOffset;
    expect(selected.domainIds.slice(0, 2)).toEqual(["domain-148", "domain-77"]);
    expect(selected.domainIds.length).toBeLessThanOrEqual(22);
    selected.domainIds.forEach(domainId => observed.add(domainId));
  }
  expect(observed).toEqual(new Set(ids));
});

test("failed or incomplete change batches never advance their cursors", async () => {
  for (const fails of [true, false]) {
    const cursors = new Map([["repo", 42]]);
    await expect(consumeDomainChanges(["repo"], cursors, async () => {
      throw new Error("Unexpected fallback");
    }, 200, async () => {
      if (fails) throw new Error("Store missing");
      return [];
    })).rejects.toThrow(fails ? "Store missing" : "Incomplete domain change batch");
    expect(cursors.get("repo")).toBe(42);
  }
});


test("batched snapshot priming reloads a mutation arriving across chunk boundaries", async () => {
  const domains = Array.from({ length: 149 }, (_, index) => ({ domain_id: `repo-${index}` }));
  const cursors = new Map<string, number>();
  let revision = 1;
  let loads = 0;
  let batchCalls = 0;
  const result = await loadConsistentDesktopSnapshot(
    async () => { loads++; return { domains, revision }; },
    async () => { throw new Error("Unexpected single-domain fallback"); },
    cursors,
    async requests => {
      batchCalls++;
      // The second chunk races the snapshot reload: an already-read domain
      // advances after its first batch page has been returned.
      if (batchCalls === 2) revision = 2;
      return requests.map(({ domain_id, cursor }) => {
        const boundary = domain_id === "repo-0" ? revision : 1;
        return {
          changes: cursor < boundary ? [{ sequence: boundary, entity_kind: "contract", entity_id: "run", changed_at: "now" }] : [],
          next_cursor: boundary, has_more: false, cursor_gap: false,
        };
      });
    },
  );
  expect(result.revision).toBe(2);
  expect(loads).toBe(3);
  expect(cursors.get("repo-0")).toBe(2);
  expect(cursors.size).toBe(149);
  expect(batchCalls).toBe(6);
});
