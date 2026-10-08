import type { EventDto } from "./types";

export type FleetTail = {
  cursor: number;
  events: EventDto[];
  first: number | null;
  last: number | null;
  drained: boolean;
  /** Every event time read so far (bounded), for the activity sparkline. */
  times: number[];
};

/** A bounded read is only final after observing the end of a settled worker's log. */
export async function readFleetTail(
  previous: FleetTail | undefined,
  settled: boolean,
  read: (cursor: number, limit: number) => Promise<EventDto[]>,
): Promise<FleetTail> {
  let cursor = previous?.cursor ?? 0;
  const collected = [...(previous?.events ?? [])];
  let first = previous?.first ?? null;
  let last = previous?.last ?? null;
  const times = [...(previous?.times ?? [])];
  let drained = false;
  for (let page = 0; page < 10; page += 1) {
    const events = await read(cursor, 200);
    for (const event of events) {
      if (event.id <= cursor) throw new Error("Worker output cursor did not advance");
      cursor = event.id;
      const at = Date.parse(event.ts);
      if (Number.isFinite(at)) { first ??= at; last = at; times.push(at); }
    }
    collected.push(...events);
    if (events.length < 200) { drained = settled; break; }
  }
  // ponytail: 4000 timestamps per worker; older slices undercount on very long logs.
  return { cursor, events: collected.slice(-400), first, last, drained, times: times.slice(-4000) };
}

/** Block characters for the activity sparkline, quietest first. */
export const SPARK = "▁▂▃▄▅▆▇█";

/**
 * Output intensity per time slice across the fleet's span: each event adds one
 * and fades with a few seconds' half-life, so bursts read as hills rather than
 * isolated ticks. Volume of output, never progress.
 */
export function activityLevels(times: number[], start: number, end: number, width: number): number[] {
  const levels = new Array<number>(width).fill(0);
  const total = Math.max(1, end - start);
  for (const at of times) {
    if (at < start || at > end) continue;
    levels[Math.min(width - 1, Math.floor(((at - start) / total) * width))] += 1;
  }
  const decay = Math.exp(-(total / width) / Math.max(2_500, (1.5 * total) / width));
  for (let index = 1; index < width; index += 1) levels[index] += levels[index - 1] * decay;
  return levels;
}

/**
 * One cell per slice: `·` before the worker's first event and after its last
 * (unless it is still live), then a block whose height follows output volume
 * on a linear scale shared by the whole fleet.
 */
export function sparkline(levels: number[], peak: number, from: number | null, to: number | null): { lead: string; body: string; tail: string } {
  if (from === null || to === null || to < from) return { lead: "·".repeat(levels.length), body: "", tail: "" };
  const body = levels.slice(from, to + 1).map((level) =>
    level / Math.max(1, peak) < .06 ? SPARK[0] : SPARK[Math.min(SPARK.length - 1, 1 + Math.floor((level / Math.max(1, peak)) * (SPARK.length - 1.001)))]).join("");
  return { lead: "·".repeat(from), body, tail: "·".repeat(levels.length - to - 1) };
}

export function clock(ms: number): string {
  const seconds = Math.max(0, Math.round(ms / 1000));
  return `${Math.floor(seconds / 60)}:${String(seconds % 60).padStart(2, "0")}`;
}
