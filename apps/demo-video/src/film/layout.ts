import {ledger} from "./kit";

// Short names for the six recorded tasks (each request is in fleet-props.json).
export const TASK_TITLE: Record<string, string> = {
  "mission-0": "Search and status filters",
  "mission-1": "Dark theme",
  "mission-2": "Spanish interface",
  "mission-3": "Filter bar",
  "mission-4": "Validation and mount",
  "mission-5": "README",
};

export type Worker = (typeof ledger.workers)[number];
export const workers = ledger.workers as Worker[];
export type Box = {x: number; y: number; w: number; h: number};

// Three columns, one per wave: four tasks together, then one, then one.
const COLUMNS = [{x: 120, w: 760}, {x: 920, w: 420}, {x: 1380, w: 420}];
export const CARD_H = 162;
export const GRID_TOP = 300;
export const cardBox = (worker: Worker): Box => {
  const column = COLUMNS[worker.wave];
  const row = workers.filter((other) => other.wave === worker.wave).indexOf(worker);
  return {x: column.x, y: GRID_TOP + row * (CARD_H + 16), w: column.w, h: CARD_H};
};
export const columnHeader = (wave: number) => ({x: COLUMNS[wave].x, y: GRID_TOP - 52});

// Recorded clock: every worker's start and end, in seconds from the run's start.
const runStart = Math.min(...workers.map((worker) => Date.parse(worker.startedAt)));
export const RUN_SECONDS = ledger.run.durationSeconds;
export const startOf = (worker: Worker) => (Date.parse(worker.startedAt) - runStart) / 1000;
export const endOf = (worker: Worker) => (Date.parse(worker.endedAt) - runStart) / 1000;

/** What a worker printed, without the check lines Pytxo itself adds. */
export const printed = (worker: Worker) => worker.output.filter((line) => !/^(\$ npm test|✓ Task checks passed)/.test(line));
