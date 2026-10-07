import { expect, test } from "@playwright/test";
import { readFleetTail } from "../src/lib/fleet-events";
import type { EventDto } from "../src/lib/types";

const events = (count: number) => Array.from({ length: count }, (_, i) => ({
  id: i + 1, agent_id: "worker", ts: new Date(i * 1000).toISOString(), kind: "stdout", payload: `line ${i + 1}`,
} as EventDto));

test("a settled worker drains its final output after a live read", async () => {
  let log = events(3);
  const read = async (cursor: number, limit: number) => log.filter(e => e.id > cursor).slice(0, limit);
  const live = await readFleetTail(undefined, false, read);
  expect(live.drained).toBe(false);
  log = [...log, { ...events(4)[3], kind: "verify-failed", payload: "Final check failed" }];
  const done = await readFleetTail(live, true, read);
  expect(done.events.at(-1)?.payload).toBe("Final check failed");
  expect(done.drained).toBe(true);
});

test("a long terminal log drains over bounded batches without losing its last event", async () => {
  const log = events(2401);
  const read = async (cursor: number, limit: number) => log.filter(e => e.id > cursor).slice(0, limit);
  const first = await readFleetTail(undefined, true, read);
  expect(first.cursor).toBe(2000);
  expect(first.drained).toBe(false);
  const last = await readFleetTail(first, true, read);
  expect(last.cursor).toBe(2401);
  expect(last.drained).toBe(true);
  expect(last.events).toHaveLength(400);
  expect(new Set(last.events.map(e => e.id)).size).toBe(400);
});

test("a failed page leaves the cached tail unchanged for an exact retry", async () => {
  const log = events(402);
  const before = await readFleetTail(undefined, false, async () => log.slice(0, 1));
  const saved = structuredClone(before);
  let calls = 0;
  await expect(readFleetTail(before, true, async (cursor, limit) => {
    if (++calls === 2) throw new Error("Disconnected");
    return log.filter(e => e.id > cursor).slice(0, limit);
  })).rejects.toThrow("Disconnected");
  expect(before).toEqual(saved);
  const recovered = await readFleetTail(before, true, async (cursor, limit) => log.filter(e => e.id > cursor).slice(0, limit));
  expect(recovered.cursor).toBe(402);
  expect(new Set(recovered.events.map(e => e.id)).size).toBe(400);
});
