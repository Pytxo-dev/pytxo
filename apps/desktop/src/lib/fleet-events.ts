import type { EventDto } from "./types";

export type FleetTail = {
  cursor: number;
  events: EventDto[];
  first: number | null;
  last: number | null;
  drained: boolean;
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
  let drained = false;
  for (let page = 0; page < 10; page += 1) {
    const events = await read(cursor, 200);
    for (const event of events) {
      if (event.id <= cursor) throw new Error("Worker output cursor did not advance");
      cursor = event.id;
      const at = Date.parse(event.ts);
      if (Number.isFinite(at)) { first ??= at; last = at; }
    }
    collected.push(...events);
    if (events.length < 200) { drained = settled; break; }
  }
  return { cursor, events: collected.slice(-400), first, last, drained };
}
