<script module lang="ts">
  /** Runs this window has shown on the wall; finished work stays until cleared. */
  let keptRuns = new Set<string>();
</script>

<script lang="ts">
  import { onDestroy, untrack } from "svelte";
  import { getCurrentWindow } from "@tauri-apps/api/window";
  import type { DesktopBackend, DesktopSnapshot } from "../../lib/desktop-backend";
  import { clock } from "../../lib/fleet-events";
  import { fleetLinks, fleetWall, homeTilde, isActiveRun, landedTile, runKey, runVerdict, wallLayout, type WallVerdict } from "../../lib/fleet-wall";
  import type { AgentDto, AgentTailDto, FleetNodeDto, RunDto, RunReviewDto } from "../../lib/types";
  import AdeIdentity from "./AdeIdentity.svelte";
  import ApertureGlyph from "./ApertureGlyph.svelte";

  let { snapshot, backend, onOpenRun, onOpenReview, onOpenApprovals, onClose }: {
    snapshot: DesktopSnapshot;
    backend: DesktopBackend;
    onOpenRun: (runId: string, domainId: string) => void;
    /** Opens a run's Review; the wall itself never applies anything. */
    onOpenReview: (runId: string, domainId: string) => void;
    onOpenApprovals: () => void;
    onClose: () => void;
  } = $props();

  let cleared = $state(0);
  const wall = $derived.by(() => {
    void cleared;
    const model = fleetWall(snapshot, keptRuns);
    keptRuns = model.keys;
    return model;
  });
  const tiles = $derived(wall.groups.flatMap((group) => group.tiles));
  const anyActiveRun = $derived(wall.runs.some(isActiveRun));
  const repoOf = (domainId: string) => wall.groups.find((group) => group.domainId === domainId)?.repo ?? domainId;

  function clearFinished() {
    keptRuns = new Set();
    cleared += 1;
  }

  const tailKey = (domainId: string, runId: string, agentId: string) => `${domainId}\n${runId}\n${agentId}`;
  const keyOf = (agent: AgentDto) => tailKey(agent.domain_id, agent.run_id, agent.id);
  const live = (agent: AgentDto) => ["running", "starting", "pending"].includes(agent.status);

  let tails = $state<Record<string, AgentTailDto>>({});
  let readError = $state<string | null>(null);
  let now = $state(Date.now());
  /** Status a settled worker had at its last good read; its output is final until that changes. */
  const settledReads = new Map<string, string>();
  let polling = false;
  let disposed = false;

  /** One batched read for every tile whose output can still change. */
  async function poll() {
    if (polling || disposed) return;
    const due = new Map(tiles.map((tile) => tile.agent)
      .filter((agent) => live(agent) || settledReads.get(keyOf(agent)) !== agent.status)
      .map((agent) => [keyOf(agent), agent]));
    if (!due.size) return;
    polling = true;
    try {
      const read = await backend.agentOutputTails([...due.values()].map((agent) => ({ domain_id: agent.domain_id, run_id: agent.run_id, agent_id: agent.id })), 2);
      if (disposed) return;
      const next = { ...tails };
      for (const tail of read) {
        const key = tailKey(tail.domain_id, tail.run_id, tail.agent_id);
        const agent = due.get(key);
        if (!agent) continue;
        next[key] = tail;
        if (!tail.error && !live(agent)) settledReads.set(key, agent.status);
      }
      tails = next;
      readError = null;
    } catch (error) {
      if (!disposed) readError = error instanceof Error ? error.message : String(error);
    } finally {
      polling = false;
      if (!disposed) now = Date.now();
    }
  }

  $effect(() => {
    void tiles.map((tile) => `${keyOf(tile.agent)}:${tile.agent.status}`).join(",");
    untrack(() => void poll());
  });
  $effect(() => {
    const timer = setInterval(() => void poll(), 1500);
    return () => clearInterval(timer);
  });
  // Clocks tick each second while any run on the wall is active.
  $effect(() => {
    if (!anyActiveRun) return;
    const tick = setInterval(() => { now = Date.now(); }, 1000);
    return () => clearInterval(tick);
  });
  onDestroy(() => { disposed = true; });

  function elapsed(agent: AgentDto): string {
    const tail = tails[keyOf(agent)];
    const first = Date.parse(tail?.first_at ?? "");
    if (!Number.isFinite(first)) return "";
    const last = Date.parse(tail?.last_at ?? "");
    return clock((live(agent) || !Number.isFinite(last) ? now : last) - first);
  }
  const startedAt = $derived(Math.min(...wall.runs.map((run) => Date.parse(run.started_at)).filter(Number.isFinite)));
  const wallElapsed = $derived.by(() => {
    if (!Number.isFinite(startedAt)) return "—";
    if (anyActiveRun) return clock(now - startedAt);
    const ends = tiles.map((tile) => Date.parse(tails[keyOf(tile.agent)]?.last_at ?? "")).filter(Number.isFinite);
    return clock((ends.length ? Math.max(...ends) : now) - startedAt);
  });

  // A fleet that finished before the first run here started cannot contain any of them.
  let fleetNodes = $state<Record<string, FleetNodeDto[]>>({});
  let fleetQuery = "";
  let fleetRequest = 0;
  $effect(() => {
    const candidates = snapshot.fleets.filter((fleet) => !fleet.finished_at || Date.parse(fleet.finished_at) >= startedAt);
    const query = `${candidates.map((fleet) => `${fleet.id}:${fleet.finished_at}`).join(",")}|${[...wall.keys].join(",")}`;
    if (query === fleetQuery) return;
    fleetQuery = query;
    const request = ++fleetRequest;
    Promise.all(candidates.map(async (fleet) => [fleet.id, await backend.fleetRunNodes(fleet.id)] as const))
      .then((entries) => { if (request === fleetRequest) fleetNodes = Object.fromEntries(entries); })
      .catch(() => {});
  });
  const fleets = $derived(fleetLinks(wall.runs, fleetNodes));

  // Task wording from the reviewed plan that dispatched each run. Presentation only.
  let prompts = $state<Record<string, string>>({});
  let promptQuery = "";
  let promptRequest = 0;
  $effect(() => {
    const keys = wall.keys;
    const query = [...keys].join(",");
    if (query === promptQuery) return;
    promptQuery = query;
    const request = ++promptRequest;
    backend.flowHistory().then((drafts) => {
      if (request !== promptRequest) return;
      const next: Record<string, string> = {};
      for (const draft of drafts) {
        if (!draft.dispatched_run_id || !draft.domain_id) continue;
        const run = runKey({ domain_id: draft.domain_id, id: draft.dispatched_run_id });
        if (!keys.has(run)) continue;
        try {
          const plan = JSON.parse(draft.plan_json ?? "{}");
          for (const task of Array.isArray(plan.tasks) ? plan.tasks : []) {
            if (typeof task.id === "string" && typeof task.prompt === "string" && task.prompt.trim()) next[`${run}\n${task.id}`] = task.prompt.trim();
          }
        } catch { /* A missing or invalid saved plan supplies no task wording. */ }
      }
      prompts = next;
    }).catch(() => {});
  });

  // One review read per run each time its recorded review state changes, never on the
  // output poll. Pending and preparing runs need none: the snapshot already says so.
  const reviewKey = (run: RunDto) => `${run.apply_status}\n${run.prepared_digest}`;
  let reviews = $state<Record<string, { key: string; review: RunReviewDto | null }>>({});
  const reviewReads = new Set<string>();
  $effect(() => {
    for (const run of wall.runs) {
      const id = runKey(run);
      const key = reviewKey(run);
      if (!run.apply_status || ["pending", "preparing"].includes(run.apply_status)) continue;
      if (untrack(() => reviews[id]?.key) === key || reviewReads.has(`${id}\n${key}`)) continue;
      reviewReads.add(`${id}\n${key}`);
      backend.runReview(run.id, run.domain_id).catch(() => null).then((review) => {
        reviewReads.delete(`${id}\n${key}`);
        if (!disposed) reviews = { ...reviews, [id]: { key, review } };
      });
    }
  });

  const READING: WallVerdict = { tone: "unknown", label: "Reading the review…", landed: false };
  const UNREADABLE: WallVerdict = { tone: "unknown", label: "Review could not be read", landed: false };
  /** Each run's last verdict; it stands while a newer read is on its way, unless it claimed verification. */
  const shownVerdicts = new Map<string, WallVerdict>();
  const verdicts = $derived.by(() => new Map(wall.runs.map((run) => {
    const id = runKey(run);
    const read = reviews[id]?.key === reviewKey(run) ? reviews[id] : null;
    const working = tiles.filter((tile) => tile.tone === "active" && runKey(tile.run) === id).length;
    const previous = shownVerdicts.get(id);
    const verdict = runVerdict(run, working, read?.review ?? null) ?? (read ? UNREADABLE : previous && !previous.landed ? previous : READING);
    shownVerdicts.set(id, verdict);
    return [id, verdict] as const;
  })));
  /** Runs whose combined change is verified: their done tiles turn solid. */
  const landed = $derived(new Set([...verdicts].filter(([, verdict]) => verdict.landed).map(([id]) => id)));
  const anyRefuted = $derived(wall.counts.failed > 0 || [...verdicts.values()].some((verdict) => verdict.tone === "refuted"));

  // Presenter mode: the wall alone, full screen. Exit, Esc, leaving full screen or leaving the wall ends it.
  let presenting = $state(false);
  let home = $state<string | null>(null);
  let stopFullscreenWatch: (() => void) | undefined;
  const nativeWindow = () => typeof window !== "undefined" && "__TAURI_INTERNALS__" in window ? getCurrentWindow() : null;
  async function present(on: boolean) {
    if (presenting === on) return;
    presenting = on;
    document.documentElement.toggleAttribute("data-presenting", on);
    stopFullscreenWatch?.();
    stopFullscreenWatch = undefined;
    if (on && home === null) backend.homeDir().then((dir) => { home = dir; }, () => {});
    const win = nativeWindow();
    if (!win) return;
    try {
      await win.setFullscreen(on);
      if (!on) return;
      const stop = await win.onResized(async () => {
        if (presenting && !(await win.isFullscreen())) void present(false);
      });
      if (presenting) stopFullscreenWatch = stop; else stop();
    } catch { /* Without full screen the chrome still hides. */ }
  }
  function presentKey(event: KeyboardEvent) {
    if (!presenting || event.key !== "Escape" || event.defaultPrevented || document.querySelector("dialog[open]")) return;
    event.preventDefault();
    void present(false);
  }
  onDestroy(() => { if (presenting) void present(false); });

  /** The body's padding; the layout fills what is inside it. */
  const BODY_PADDING = 14;
  let bodyWidth = $state(0);
  let bodyHeight = $state(0);
  const layout = $derived(wallLayout(wall.groups.map((group) => group.tiles.length), bodyWidth - 2 * BODY_PADDING, bodyHeight - 2 * BODY_PADDING));
  const MARK: Record<string, string> = { verified: "✓", refuted: "✗", queued: "·" };

  let heading: HTMLElement | undefined = $state();
  export function focusHeading() {
    queueMicrotask(() => heading?.focus());
  }
</script>

<svelte:window onkeydown={presentKey} />

<section class="screen work wall" class:presenting aria-label="All agents" data-testid="fleet-wall">
  <header class="wall-head">
    <ApertureGlyph active={wall.counts.working > 0} tone={anyRefuted ? "refuted" : wall.counts.working ? "active" : wall.runs.length ? "settled" : "unknown"} />
    <h1 bind:this={heading} tabindex="-1">All agents</h1>
    <dl class="stats" aria-label="Counts over the runs listed below">
      <div data-kind="working" class:hot={wall.counts.working > 0}><dt>working</dt><dd>{wall.counts.working}</dd></div>
      <div data-kind="done"><dt>done</dt><dd>{wall.counts.done}</dd></div>
      <div data-kind="failed" class:hot={wall.counts.failed > 0}><dt>failed</dt><dd>{wall.counts.failed}</dd></div>
      {#if wall.counts.queued}<div data-kind="queued"><dt>queued</dt><dd>{wall.counts.queued}</dd></div>{/if}
      {#if wall.counts.other}<div data-kind="other"><dt>stopped or unknown</dt><dd>{wall.counts.other}</dd></div>{/if}
      <div data-kind="ready" class:hot={landed.size > 0} title="Runs whose combined change is verified, of the runs on the wall"><dt>ready</dt><dd>{landed.size}/{wall.runs.length}</dd></div>
      <div data-kind="runs"><dt>{wall.runs.length === 1 ? "run" : "runs"}</dt><dd>{wall.runs.length}</dd></div>
      <div data-kind="repos"><dt>{wall.groups.length === 1 ? "repo" : "repos"}</dt><dd>{wall.groups.length}</dd></div>
      <div data-kind="elapsed"><dt>elapsed</dt><dd><time>{wallElapsed}</time></dd></div>
    </dl>
    <div class="wall-actions">
      {#if presenting && snapshot.approvals.length}
        <button class="approvals" onclick={() => { void present(false); onOpenApprovals(); }}>{snapshot.approvals.length} {snapshot.approvals.length === 1 ? "approval" : "approvals"} waiting</button>
      {/if}
      {#if !presenting && wall.runs.some((run) => !isActiveRun(run))}<button class="clear" onclick={clearFinished}>Clear finished</button>{/if}
      <button class="present" title={presenting ? "Exit presenting (Esc)" : "Only the wall, full screen"} onclick={() => void present(!presenting)}>{presenting ? "Exit" : "Present"}</button>
      {#if !presenting}
        <div class="work-view" role="group" aria-label="Work view">
          <button aria-pressed="false" onclick={onClose}>Run</button>
          <button aria-pressed="true">Wall</button>
        </div>
      {/if}
    </div>
  </header>

  {#if readError}<p class="wall-feedback" role="status">Output could not be read: {readError} Retrying.</p>{/if}

  <div class="wall-body" bind:clientWidth={bodyWidth} bind:clientHeight={bodyHeight} style={`--body-padding:${BODY_PADDING}px`}>
    {#if !wall.groups.length}
      <div class="wall-empty">
        <h2>No agents are working</h2>
        <p>Agents from every workspace's active runs appear here as they start. Finished runs stay until you clear them.</p>
      </div>
    {:else}
      <div class="bands" style={`--blocks:${layout.blocks};--cols:${layout.columns};--tile-w:${layout.tileWidth}px;--tile-h:${layout.tileHeight}px`}>
        {#each wall.groups as group (group.domainId)}
          <section class="group tui-pane" aria-label={`${group.repo}: ${group.tiles.length} ${group.tiles.length === 1 ? "agent" : "agents"}`}>
            <span class="tui-legend"><b>{group.repo}</b><span>{group.tiles.length}</span></span>
            <ol class="tiles">
              {#each group.tiles.map((entry) => landedTile(entry, landed)) as tile (tile.agent.id)}
                {@const tail = tails[keyOf(tile.agent)]}
                {@const prompt = prompts[`${runKey(tile.run)}\n${tile.agent.task_id}`]}
                {@const time = elapsed(tile.agent)}
                <li>
                  <button class="tile" data-tone={tile.tone} title={prompt ?? tile.agent.task_id}
                    aria-label={`${group.repo} ${tile.agent.task_id}${prompt ? `, ${prompt}` : ""}: ${tile.label}${time ? `, ${time}` : ""}. ${tile.agent.launcher?.display_name ?? "Custom command"}. Open run.`}
                    onclick={() => onOpenRun(tile.run.id, tile.run.domain_id)}>
                    <span class="head">
                      <span class="cli">{#if tile.agent.launcher}<AdeIdentity id={tile.agent.launcher.id} />{:else}<i aria-hidden="true">&gt;_</i>{/if}</span>
                      <strong>{tile.agent.task_id}</strong>
                    </span>
                    {#if prompt}<span class="prompt">{prompt}</span>{/if}
                    <span class="out">
                      {#if tail?.error}<span class="hint">Output unavailable</span>
                      {:else if tail?.lines.length}{#each tail.lines as line, index (index)}<span>{presenting ? homeTilde(line, home) : line}</span>{/each}
                      {:else}<span class="hint">{tail ? "No output yet" : "Reading output"}</span>{/if}
                    </span>
                    <span class="foot">
                      <span class="state">{#if tile.tone === "active"}<span class="tui-spin" aria-hidden="true"><span>|/-\</span></span>{:else}<i aria-hidden="true">{MARK[tile.tone] ?? "○"}</i>{/if}{tile.label}</span>
                      <time>{time || "—"}</time>
                    </span>
                  </button>
                </li>
              {/each}
              {#if !group.tiles.length}<li class="pending">No workers recorded yet</li>{/if}
            </ol>
            <span class="verdicts">
              {#each group.runs as run (runKey(run))}
                {@const verdict = verdicts.get(runKey(run)) ?? READING}
                <button class="verdict" data-tone={verdict.tone} title={verdict.label}
                  aria-label={`${group.repo}${group.runs.length > 1 ? ` ${run.id}` : ""}: ${verdict.label}. Open Review.`}
                  onclick={() => { void present(false); onOpenReview(run.id, run.domain_id); }}>
                  {#if verdict.tone === "active"}<span class="tui-spin" aria-hidden="true"><span>|/-\</span></span>{:else}<i aria-hidden="true">{MARK[verdict.tone] ?? "○"}</i>{/if}
                  <span>{verdict.label}</span>
                </button>
              {/each}
            </span>
          </section>
        {/each}
      </div>
    {/if}
  </div>

  <footer class="evidence" aria-label="Runs these counts come from">
    <span>{wall.runs.length ? `Counts from ${wall.runs.length} ${wall.runs.length === 1 ? "run" : "runs"}:` : "No runs on the wall."}</span>
    {#each wall.runs as run (runKey(run))}<code title={run.domain_id}>{repoOf(run.domain_id)} {run.id}</code>{/each}
    {#each fleets as fleetRunId (fleetRunId)}<code>fleet {fleetRunId}</code>{/each}
  </footer>
</section>

<style>
  /* `work` keeps the shell's reading-page padding off; the wall fills the Work pane like the run view. */
  .screen.wall { --tui-bg: var(--pytxo-work-canvas); display: flex; width: 100%; max-width: none; height: 100%; margin: 0; min-height: 0; box-sizing: border-box; flex-direction: column; gap: 9px; padding: 10px 14px 10px; overflow: hidden; }

  .wall-head { display: grid; grid-template-columns: 34px auto minmax(0, 1fr) auto; min-height: 58px; flex: none; align-items: center; gap: 12px 18px; padding: 4px 0 10px; border-bottom: 1px solid var(--pytxo-line); }
  .wall-head :global(.aperture-glyph) { width: 34px; height: 34px; flex-basis: 34px; }
  h1 { margin: 0; font-size: var(--pytxo-title-size); font-weight: 600; letter-spacing: -.02em; white-space: nowrap; outline: 0; }
  h1:focus-visible { outline: 2px solid var(--pytxo-accent); outline-offset: 3px; }
  .stats { display: flex; min-width: 0; align-items: flex-end; gap: 4px 26px; margin: 0; overflow: hidden; }
  .stats div { display: flex; flex: none; flex-direction: column; }
  .stats dd { order: 1; margin: 0; color: var(--pytxo-text-strong); font: 600 26px/1 var(--pytxo-font-ui); font-variant-numeric: tabular-nums; letter-spacing: -.02em; }
  .stats dt { order: 2; margin-top: 5px; color: var(--pytxo-text-muted); font: 500 10.5px var(--pytxo-font-mono); letter-spacing: .08em; text-transform: uppercase; }
  .stats [data-kind="working"].hot dd { color: var(--pytxo-activity); }
  .stats [data-kind="ready"].hot dd { color: var(--state-verified); }
  .stats [data-kind="failed"].hot dd { color: var(--state-refuted); }
  .stats [data-kind="ready"], .stats [data-kind="elapsed"] { padding-left: 26px; border-left: 1px solid var(--pytxo-line-soft); }
  .wall-actions { display: flex; align-items: center; gap: 7px; }
  .clear, .present, .approvals { min-height: 34px; padding: 0 12px; border: 1px solid var(--pytxo-line); border-radius: 4px; background: transparent; color: var(--pytxo-text-soft); font: 500 12px var(--pytxo-font-ui); cursor: pointer; }
  .clear:hover, .present:hover, .approvals:hover { background: var(--pytxo-surface-active); color: var(--pytxo-text-strong); }
  .clear:focus-visible, .present:focus-visible, .approvals:focus-visible { outline: 2px solid var(--pytxo-accent); outline-offset: 2px; }
  /* Same hue as the app bar's inbox: the one thing that blocks the operator. */
  .approvals, .approvals:hover { border-color: color-mix(in oklab, var(--state-attention) 50%, var(--pytxo-line)); color: var(--state-attention); font-weight: 600; }
  .wall-feedback { flex: none; margin: 0; padding: 8px 12px; border: 1px solid color-mix(in srgb, var(--state-refuted) 48%, var(--pytxo-line)); border-radius: 4px; color: var(--state-refuted); font-size: 12px; }

  /* The layout fills this box exactly, so a stable gutter keeps it from flipping when it scrolls. */
  .wall-body { flex: 1; min-height: 0; box-sizing: border-box; padding: var(--body-padding); overflow: auto; scrollbar-gutter: stable; border: 1px solid var(--pytxo-line); border-radius: 7px; background: var(--pytxo-work-canvas); }
  .wall-empty { display: grid; height: 100%; place-content: center; gap: 8px; text-align: center; }
  .wall-empty h2 { margin: 0; color: var(--pytxo-text-strong); font-size: 16px; font-weight: 600; }
  .wall-empty p { max-width: 52ch; margin: 0; color: var(--pytxo-text-soft); font-size: 12.5px; line-height: 1.6; text-wrap: pretty; }

  /* Pane sizes mirror WALL_METRICS in lib/fleet-wall.ts. Bands keep room between a verdict line and the next legend. */
  .bands { display: grid; grid-template-columns: repeat(var(--blocks), max-content); align-items: start; justify-content: center; gap: 24px 12px; }
  .group { padding: 15px 7px 15px; }
  /* Each run's verdict sits on the pane's bottom border, as the legend sits on its top one. */
  .verdicts { position: absolute; bottom: -9px; left: 12px; z-index: 1; display: flex; max-width: calc(100% - 24px); gap: 6px; }
  .verdict { --verdict: var(--pytxo-text-soft); position: relative; display: flex; min-width: 0; height: 18px; flex: 0 1 auto; align-items: center; gap: 6px; padding: 0 7px; border: 0; background: var(--tui-bg); color: var(--verdict); font: 600 11.5px var(--pytxo-font-mono); white-space: nowrap; cursor: pointer; }
  .verdict::after { content: ""; position: absolute; inset: -5px 0; }
  .verdict > span:last-child { overflow: hidden; text-overflow: ellipsis; }
  .verdict i { font-style: normal; }
  .verdict:hover > span:last-child { text-decoration: underline; text-underline-offset: 3px; }
  .verdict:focus-visible { outline: 2px solid var(--pytxo-accent); outline-offset: 2px; }
  .verdict[data-tone="active"] { --verdict: var(--pytxo-activity); }
  .verdict[data-tone="verified"] { --verdict: var(--state-verified); }
  .verdict[data-tone="refuted"] { --verdict: var(--state-refuted); }
  .group .tui-legend { max-width: calc(100% - 24px); overflow: hidden; }
  .group .tui-legend b { overflow: hidden; letter-spacing: .04em; text-overflow: ellipsis; }
  .group .tui-legend span { color: var(--pytxo-text-muted); }
  .tiles { display: grid; grid-template-columns: repeat(var(--cols), var(--tile-w)); grid-auto-rows: var(--tile-h); gap: 8px; margin: 0; padding: 0; list-style: none; }
  .tiles > li { min-width: 0; min-height: 0; }
  .pending { display: grid; place-items: center; border: 1px dashed var(--pytxo-line); border-radius: 5px; color: var(--pytxo-text-muted); font: 11.5px var(--pytxo-font-mono); }

  /* State reads at phone size: a top rule, a tinted face and a coloured label, plus a shape per state. */
  .tile { --accent: var(--pytxo-text-muted); container: tile / size; position: relative; display: flex; width: 100%; height: 100%; box-sizing: border-box; flex-direction: column; gap: 3px; overflow: hidden; padding: 9px 10px 6px; border: 1px solid color-mix(in srgb, var(--accent) 45%, var(--pytxo-line)); border-radius: 5px; background: color-mix(in srgb, var(--accent) 8%, var(--pytxo-work-canvas)); color: var(--pytxo-text-body); text-align: left; cursor: pointer; transition: border-color var(--pytxo-motion-fast) var(--pytxo-motion-ease), background-color 200ms var(--pytxo-motion-ease); }
  .tile::before { content: ""; position: absolute; inset: 0 0 auto; height: 4px; background: var(--accent); }
  .tile[data-tone="active"] { --accent: var(--pytxo-activity); }
  /* Settled states get a stronger face than working ones, so done and failed separate from live at a glance. */
  .tile[data-tone="verified"] { --accent: var(--state-verified); background: color-mix(in srgb, var(--accent) 15%, var(--pytxo-work-canvas)); animation: settle 1s ease-out 1; }
  /* Exit 0 is the agent's claim: hollow until the run's combined change is verified, then it fills. */
  .tile[data-tone="claimed"] { --accent: var(--state-claimed); background: transparent; }
  .tile[data-tone="refuted"] { --accent: var(--state-refuted); background: color-mix(in srgb, var(--accent) 17%, var(--pytxo-work-canvas)); }
  .tile[data-tone="queued"] { border-style: dashed; }
  .tile[data-tone="queued"]::before { background: repeating-linear-gradient(90deg, var(--accent) 0 6px, transparent 6px 10px); }
  .tile[data-tone="unknown"]::before, .tile[data-tone="claimed"]::before { background: repeating-linear-gradient(45deg, var(--accent) 0 2px, transparent 2px 4px); }
  @keyframes settle { from { box-shadow: 0 0 0 3px color-mix(in srgb, var(--state-verified) 40%, transparent); } }
  .tile:hover { border-color: color-mix(in srgb, var(--accent) 75%, var(--pytxo-line)); }
  .tile:focus-visible { outline: 2px solid var(--pytxo-accent); outline-offset: 2px; }
  .head { display: flex; min-width: 0; flex: none; align-items: center; gap: 7px; }
  .cli { display: grid; width: 18px; height: 18px; flex: none; place-items: center; }
  .cli :global(.ade-identity) { transform: scale(.78); }
  .cli i { color: var(--pytxo-text-muted); font: 600 11px var(--pytxo-font-mono); font-style: normal; }
  .head strong { min-width: 0; overflow: hidden; color: var(--pytxo-text-strong); font: 600 15px/1.2 var(--pytxo-font-ui); text-overflow: ellipsis; white-space: nowrap; }
  .prompt { flex: none; overflow: hidden; color: var(--pytxo-text-soft); font: 12px/1.3 var(--pytxo-font-ui); text-overflow: ellipsis; white-space: nowrap; }
  .out { display: flex; min-height: 0; flex: 1; flex-direction: column; justify-content: flex-end; overflow: hidden; color: var(--pytxo-text-muted); font: 11px/1.45 var(--pytxo-font-mono); }
  .out span { overflow: hidden; text-overflow: ellipsis; white-space: nowrap; }
  .out span:last-child:not(.hint) { color: var(--pytxo-text-body); }
  .foot { display: flex; flex: none; align-items: center; justify-content: space-between; gap: 8px; font: 600 12px var(--pytxo-font-mono); }
  .state { display: flex; min-width: 0; align-items: center; gap: 6px; overflow: hidden; color: var(--accent); white-space: nowrap; }
  .tile[data-tone="queued"] .state, .tile[data-tone="unknown"] .state { color: var(--pytxo-text-soft); }
  .state i { font-style: normal; }
  .foot time { flex: none; color: var(--pytxo-text-soft); font-weight: 500; font-variant-numeric: tabular-nums; }
  /* Tall tiles (a 48-agent wall) get larger names and states so a phone-sized replay stays readable.
     Container sizes are the tile's content box: 17px less than the tile. */
  @container tile (min-height: 90px) { .head strong { font-size: 18px; } .foot { font-size: 14px; } }
  @container tile (max-height: 96px) { .prompt { display: none; } }
  @container tile (max-height: 74px) { .out span:not(:last-child) { display: none; } }

  /* Presenting: the wall alone, its counts read from across a room. */
  .presenting .wall-head { grid-template-columns: 46px auto minmax(0, 1fr) auto; min-height: 84px; }
  .presenting .wall-head :global(.aperture-glyph) { width: 46px; height: 46px; flex-basis: 46px; }
  .presenting .stats { gap: 4px 34px; }
  .presenting .stats dd { font-size: 42px; }
  .presenting .stats dt { font-size: 12.5px; }
  .presenting .stats [data-kind="ready"], .presenting .stats [data-kind="elapsed"] { padding-left: 34px; }

  .evidence { display: flex; max-height: 46px; flex: none; flex-wrap: wrap; align-items: baseline; gap: 2px 14px; overflow: auto; color: var(--pytxo-text-muted); font: 11px/1.6 var(--pytxo-font-mono); }
  .evidence code { color: var(--pytxo-text-soft); font: inherit; white-space: nowrap; }

  @media (prefers-reduced-motion: reduce) { .tile { animation: none; transition: none; } }
  :global([data-force-reduced-motion]) .tile { animation: none; transition: none; }
  @container mission (max-width: 1120px) { .stats { gap: 4px 16px; } .stats [data-kind="ready"], .stats [data-kind="elapsed"] { padding-left: 16px; } }
  @container mission (max-width: 880px) { .wall-head { grid-template-columns: 34px minmax(0, 1fr) auto; } .stats { grid-column: 1 / -1; flex-wrap: wrap; } }
</style>
