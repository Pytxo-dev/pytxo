import { agentState, type StateTone } from "./epistemic";
import { reviewContract, reviewPresentation, type ReviewContractState } from "./review-state";
import type { AgentDto, CatalogEntryStatus, FleetNodeDto, RunDto, RunReviewDto } from "./types";

/** Statuses of a run whose workers may still change; the same set Work can stop. */
const ACTIVE_RUN_STATUSES = ["starting", "running", "pending", "dispatching", "active"];

export const isActiveRun = (run: Pick<RunDto, "status">) => ACTIVE_RUN_STATUSES.includes(run.status.toLowerCase());

/** Run ids are unique only within a workspace. */
export const runKey = (run: Pick<RunDto, "domain_id" | "id">) => `${run.domain_id}\n${run.id}`;

export const repoName = (root: string, fallback: string) => root.split(/[\\/]/).filter(Boolean).pop() ?? fallback;

export type WallCounts = { working: number; done: number; failed: number; queued: number; other: number; agents: number };
export type WallTile = { agent: AgentDto; run: RunDto; tone: StateTone; label: string };
export type WallGroup = { domainId: string; repo: string; runs: RunDto[]; tiles: WallTile[]; counts: WallCounts };
export type WallModel = { groups: WallGroup[]; runs: RunDto[]; counts: WallCounts; keys: Set<string> };

const emptyCounts = (): WallCounts => ({ working: 0, done: 0, failed: 0, queued: 0, other: 0, agents: 0 });
const natural = new Intl.Collator(undefined, { numeric: true });

/**
 * Tile wording for an agent's recorded status; its colour is the same epistemic
 * tone, except that exit 0 is only the agent's claim: a done tile stays claimed
 * until its run's combined change is verified (`landedTile`).
 */
export function tileState(agent: AgentDto): { tone: StateTone; label: string } {
  const state = agentState(agent);
  if (state.tone === "active") return { tone: state.tone, label: "working" };
  if (state.tone === "verified") return { tone: "claimed", label: "done" };
  if (agent.status === "completed" && agent.exit_code !== null) return { tone: state.tone, label: `exit ${agent.exit_code}` };
  return { tone: state.tone, label: state.label.toLowerCase() };
}

function tally(counts: WallCounts, tone: StateTone) {
  counts.agents += 1;
  if (tone === "active") counts.working += 1;
  else if (tone === "claimed" || tone === "verified") counts.done += 1;
  else if (tone === "refuted") counts.failed += 1;
  else if (tone === "queued") counts.queued += 1;
  else counts.other += 1;
}

/**
 * Every recorded agent of every active run in every workspace, one group per
 * workspace. `kept` holds runs an earlier read showed, so finished work stays
 * until cleared. Counts are tallies of exactly these rows.
 */
export function fleetWall(
  snapshot: { domains: CatalogEntryStatus[]; runs: RunDto[]; agents: AgentDto[] },
  kept: ReadonlySet<string> = new Set(),
): WallModel {
  const started = (run: RunDto) => Date.parse(run.started_at) || 0;
  // Earliest first, so groups and tiles keep their places while the wall is read.
  const runs = snapshot.runs.filter((run) => isActiveRun(run) || kept.has(runKey(run)))
    .sort((a, b) => started(a) - started(b) || natural.compare(a.domain_id, b.domain_id) || natural.compare(a.id, b.id));
  const groups = new Map<string, WallGroup>();
  const counts = emptyCounts();
  for (const run of runs) {
    let group = groups.get(run.domain_id);
    if (!group) {
      const root = snapshot.domains.find((domain) => domain.domain_id === run.domain_id)?.repo_root ?? run.repo_root;
      group = { domainId: run.domain_id, repo: repoName(root, run.domain_id), runs: [], tiles: [], counts: emptyCounts() };
      groups.set(run.domain_id, group);
    }
    group.runs.push(run);
    const agents = snapshot.agents.filter((agent) => agent.domain_id === run.domain_id && agent.run_id === run.id)
      .sort((a, b) => a.wave - b.wave || natural.compare(a.task_id, b.task_id) || natural.compare(a.id, b.id));
    for (const agent of agents) {
      const state = tileState(agent);
      group.tiles.push({ agent, run, ...state });
      tally(group.counts, state.tone);
      tally(counts, state.tone);
    }
  }
  return { groups: [...groups.values()], runs, counts, keys: new Set(runs.map(runKey)) };
}

/** A done tile turns solid only once its run's combined change is verified. */
export function landedTile(tile: WallTile, landed: ReadonlySet<string>): WallTile {
  return tile.tone === "claimed" && landed.has(runKey(tile.run)) ? { ...tile, tone: "verified" } : tile;
}

/** One run's line under its pane. `landed`: the combined change is verified (ready or applied). */
export type WallVerdict = { tone: StateTone; label: string; landed: boolean };

/** The command named by a `candidate_check_failed` message: "<task>: <command> failed on the combined change". */
const failedCommand = (message: string) => /^.+?: (.+) failed on the combined change$/.exec(message)?.[1] ?? message;

/**
 * A run's verdict, from the same presentation Review derives. `review` is a read
 * of the run's current review state; until it arrives the snapshot's copy of
 * the contract row stands in, and a ready run, whose verification only the
 * prepared package shows, has no verdict yet (null).
 */
export function runVerdict(run: RunDto, working: number, review: RunReviewDto | null): WallVerdict | null {
  const contract: ReviewContractState = review ? reviewContract(run, review) : {
    apply_status: run.apply_status ?? "unavailable",
    recovery_state: run.recovery_state,
    last_apply_error: run.last_apply_error,
    candidate_verified: false,
    prepared_file_count: null,
  };
  const line = (tone: StateTone, label: string, landed = false): WallVerdict => ({ tone, label, landed });
  if (isActiveRun(run) && ["pending", "unavailable"].includes(contract.apply_status)) return line("active", `${working} working`);
  if (contract.apply_status === "preparing") return line("active", "Checking the combined change…");
  const presentation = reviewPresentation(contract);
  const failure = contract.last_apply_error;
  switch (presentation.state) {
    case "verification_required": return review ? line("unknown", presentation.title) : null;
    case "ready": return line("verified", "Combined change verified · Ready to Apply", true);
    case "applied": return line("verified", presentation.title, true);
    case "applying": return line("active", presentation.title);
    case "review_failed":
      return line("refuted", failure?.code === "candidate_check_failed" ? `Combined check failed: ${failedCommand(failure.message)}` : presentation.title);
    case "recovery_required":
    case "retryable_failure": return line("refuted", presentation.title);
    // Stale, empty, discarded and the rest prove nothing about the combination.
    default: return line("unknown", presentation.title);
  }
}

/** `line` with the user's home folder shown as `~`: either slash, any case, whole folder names only. */
export function homeTilde(line: string, home: string | null): string {
  const parts = home?.split(/[\\/]+/).filter(Boolean);
  if (!parts?.length) return line;
  const folder = parts.map((part) => part.replace(/[.*+?^${}()|[\]\\]/g, "\\$&")).join("[\\\\/]+");
  return line.replace(new RegExp(`(?<![\\w.-])${/^[\\/]/.test(home!) ? "[\\\\/]" : ""}${folder}(?![\\w.-])`, "gi"), "~");
}

/** Fleet runs whose recorded nodes dispatched one of `runs` (catalog `fleet_nodes`). */
export function fleetLinks(runs: RunDto[], nodes: Record<string, FleetNodeDto[]>): string[] {
  const keys = new Set(runs.map(runKey));
  return Object.keys(nodes).filter((fleetRunId) => nodes[fleetRunId].some((node) =>
    node.domain_run_id !== null && keys.has(runKey({ domain_id: node.domain_id, id: node.domain_run_id }))));
}

export type WallMetrics = {
  /** Between tiles, between panes in a band, between bands. */
  gap: number; blockGap: number; bandGap: number;
  /** Pane padding: left plus right, and top plus bottom. */
  paneX: number; paneY: number;
  minWidth: number; minHeight: number; maxWidth: number; maxHeight: number;
  /** Tile height once the wall scrolls anyway: room for two output lines. */
  scrollHeight: number;
};
export const WALL_METRICS: WallMetrics = { gap: 8, blockGap: 12, bandGap: 24, paneX: 16, paneY: 32, minWidth: 120, minHeight: 56, maxWidth: 300, maxHeight: 160, scrollHeight: 92 };
export type WallLayout = { blocks: number; columns: number; tileWidth: number; tileHeight: number; fits: boolean };

/**
 * Fills `width` × `height` with every group: each workspace is a pane of
 * `columns` tiles per row, `blocks` panes per band. Picks the largest tile at
 * about 1.6:1; ties go to fewer empty slots. When nothing fits above the
 * minimum size, picks the fewest rows at a readable height and the area scrolls.
 */
export function wallLayout(sizes: number[], width: number, height: number, m: WallMetrics = WALL_METRICS): WallLayout {
  const counts = sizes.length ? sizes.map((size) => Math.max(1, size)) : [1];
  let best: (WallLayout & { score: number; empty: number }) | null = null;
  for (let blocks = 1; blocks <= counts.length; blocks += 1) {
    const paneWidth = (width - (blocks - 1) * m.blockGap) / blocks;
    for (let columns = 1; columns <= Math.max(...counts); columns += 1) {
      const tileWidth = (paneWidth - m.paneX - (columns - 1) * m.gap) / columns;
      // Never trade width for rows: only the one-pane, one-column layout may be narrower than a tile.
      if (tileWidth < m.minWidth && (columns > 1 || blocks > 1)) break;
      let rows = 0;
      let bands = 0;
      let empty = 0;
      for (let start = 0; start < counts.length; start += blocks) {
        const band = counts.slice(start, start + blocks);
        const bandRows = Math.max(...band.map((size) => Math.ceil(size / columns)));
        rows += bandRows;
        bands += 1;
        empty += band.reduce((sum, size) => sum + bandRows * columns - size, 0);
      }
      const tileHeight = (height - (bands - 1) * m.bandGap - bands * m.paneY - (rows - bands) * m.gap) / rows;
      const fits = tileHeight >= m.minHeight && tileWidth >= m.minWidth;
      const score = fits ? Math.min(Math.min(tileWidth, m.maxWidth), Math.min(tileHeight, m.maxHeight) * 1.6) : -rows;
      if (!best || score > best.score + .5 || (score > best.score - .5 && empty < best.empty)) {
        best = { blocks, columns, tileWidth, tileHeight, fits, score, empty };
      }
    }
  }
  const chosen = best!;
  return {
    blocks: chosen.blocks,
    columns: chosen.columns,
    tileWidth: Math.floor(Math.min(m.maxWidth, Math.max(m.minWidth, chosen.tileWidth))),
    tileHeight: Math.floor(chosen.fits ? Math.min(m.maxHeight, chosen.tileHeight) : m.scrollHeight),
    fits: chosen.fits,
  };
}
