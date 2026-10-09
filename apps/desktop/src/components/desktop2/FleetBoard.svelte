<script lang="ts">
  import { onDestroy, untrack } from "svelte";
  import { agentState } from "../../lib/epistemic";
  import { eventLines } from "../../lib/terminal-text";
  import { adeDisplayName } from "../../lib/ade-status";
  import { activityLevels, clock, readFleetTail, sparkline, type FleetTail } from "../../lib/fleet-events";
  import type { DesktopBackend } from "../../lib/desktop-backend";
  import type { AgentDto, RunDto, RunReviewDto } from "../../lib/types";
  import AdeIdentity from "./AdeIdentity.svelte";
  import FleetRadar, { type Blip } from "./FleetRadar.svelte";

  let {
    run, review, agents, taskDescriptions = {}, taskClis = {}, backend = null, onInspect,
  }: {
    run: RunDto; review: RunReviewDto | null; agents: AgentDto[];
    taskDescriptions?: Record<string, string>; taskClis?: Record<string, string>; backend?: DesktopBackend | null;
    onInspect: (agent: AgentDto) => void;
  } = $props();

  type Task = RunReviewDto["plan"]["waves"][number][number];
  type Tone = "live" | "done" | "failed" | "queued" | "settled";

  const waves = $derived((review?.plan.waves ?? []).filter((wave) => wave.length));
  const tasks = $derived(waves.flat());
  const agentFor = (task: Task) => {
    const matches = agents.filter((agent) => agent.task_id === task.task_id && agent.run_id === run.id && agent.domain_id === run.domain_id);
    return matches.length === 1 ? matches[0] : undefined;
  };
  /** Recorded launcher first; a queued task shows its reviewed CLI. */
  const cliOf = (task: Task, agent: AgentDto | undefined) =>
    agent?.launcher?.id ?? taskClis[task.task_id] ?? (adeDisplayName(task.agent) ? task.agent : null);
  const vendorOf = (task: Task, agent: AgentDto | undefined) =>
    agent?.launcher?.display_name ?? adeDisplayName(cliOf(task, agent)) ?? "Agent";

  /** An earlier-wave task that claims a path this task also claims: Pytxo ran them in order. */
  function sharedOwner(task: Task): { owner: Task; path: string } | null {
    for (const earlier of waves.slice(0, task.wave).flat()) {
      const path = task.paths.find((mine) => earlier.paths.some((theirs) => mine === theirs || mine.startsWith(`${theirs}/`) || theirs.startsWith(`${mine}/`)));
      if (path) return { owner: earlier, path };
    }
    return null;
  }
  /** One column count for every lane, so panes share a width across steps. */
  const columns = $derived(Math.min(4, Math.max(1, ...waves.map((wave) => wave.length))));
  const orderedShares = $derived(tasks.filter((task) => sharedOwner(task)).length);
  /** Tasks with prepared files; null until a candidate exists. A worker can pass its checks and change nothing. */
  const preparedTasks = $derived(review?.prepared_manifest ? new Set(review.prepared_manifest.files.map((file) => file.task_id)) : null);
  const checkedTasks = $derived(tasks.filter((task) => task.verify.length > 0));
  const passedTasks = $derived(checkedTasks.filter((task) => {
    const agent = agentFor(task);
    return agent?.status === "completed" && agent.exit_code === 0;
  }).length);

  let tails = $state<Record<string, FleetTail>>({});
  let readErrors = $state<Record<string, boolean>>({});
  let now = $state(Date.now());
  let timer: ReturnType<typeof setInterval> | undefined;
  let pollingGeneration: number | null = null;
  let generation = 0;
  let scopeKey = "";
  let disposed = false;
  const currentAgents = $derived(agents.filter((agent) => agent.run_id === run.id && agent.domain_id === run.domain_id));
  const anyLive = $derived(currentAgents.some(isLive));

  function isLive(agent: AgentDto | undefined) {
    return !!agent && ["running", "starting", "pending"].includes(agent.status);
  }

  async function poll() {
    const key = `${run.domain_id}:${run.id}`;
    if (key !== scopeKey) {
      scopeKey = key;
      generation += 1;
      tails = {};
      readErrors = {};
    }
    if (!backend || pollingGeneration === generation || disposed) return;
    const requestGeneration = generation;
    pollingGeneration = requestGeneration;
    const reader = backend;
    const runId = run.id;
    const belongs = () => !disposed && requestGeneration === generation && key === `${run.domain_id}:${run.id}`;
    try {
      await Promise.allSettled(currentAgents.map(async (agent) => {
        const tail = tails[agent.id];
        if (tail?.drained && !isLive(agent)) return;
        try {
          const next = await readFleetTail(tail, !isLive(agent), (cursor, limit) =>
            reader.readAgentEvents(runId, agent.id, agent.domain_id, cursor, limit));
          if (belongs()) {
            tails[agent.id] = next;
            delete readErrors[agent.id];
          }
        } catch {
          if (belongs()) readErrors[agent.id] = true;
        }
      }));
    } finally {
      if (pollingGeneration === requestGeneration) pollingGeneration = null;
      if (belongs()) now = Date.now();
    }
  }

  $effect(() => {
    void `${run.domain_id}:${run.id}:${agents.map((agent) => `${agent.id}:${agent.status}`).join(",")}`;
    untrack(() => void poll());
  });
  $effect(() => {
    timer = setInterval(() => { if (currentAgents.some(agent => isLive(agent) || !tails[agent.id]?.drained)) void poll(); }, 1500);
    return () => clearInterval(timer);
  });
  // Clocks tick each second while anything is live.
  $effect(() => {
    if (!anyLive) return;
    const tick = setInterval(() => { now = Date.now(); }, 1000);
    return () => clearInterval(tick);
  });
  onDestroy(() => { disposed = true; generation += 1; clearInterval(timer); });

  const span = $derived.by(() => {
    const starts = Object.values(tails).map((tail) => tail.first).filter((value): value is number => value !== null);
    const ends = Object.values(tails).map((tail) => tail.last).filter((value): value is number => value !== null);
    const start = starts.length ? Math.min(...starts) : Date.parse(run.started_at);
    const end = Math.max(anyLive ? now : 0, ...ends, start + 1000);
    return { start, end };
  });

  function tone(agent: AgentDto | undefined): Tone {
    if (!agent) return "queued";
    if (isLive(agent)) return "live";
    if (agent.status === "completed" && agent.exit_code === 0) return "done";
    if (["failed", "blocked_by_dependency", "verify_failed"].includes(agent.status)) return "failed";
    return "settled";
  }
  /** A live worker with no output for this long reads as quiet, not stuck: Pytxo cannot tell which. */
  const QUIET_MS = 45_000;
  function status(task: Task, agent: AgentDto | undefined): string {
    const kind = tone(agent);
    const tail = agent ? tails[agent.id] : undefined;
    if (kind === "live") return tail?.last && now - tail.last > QUIET_MS ? `quiet ${clock(now - tail.last)}` : "working";
    if (kind === "done") {
      if (preparedTasks && !preparedTasks.has(task.task_id)) return "no changes";
      return task.verify.length ? "passed" : "completed";
    }
    if (kind === "queued") return sharedOwner(task) ? "next" : "queued";
    return (agent ? agentState(agent).label : "settled").toLowerCase();
  }
  function elapsed(agent: AgentDto | undefined): string {
    const tail = agent ? tails[agent.id] : undefined;
    if (!tail || tail.first === null) return "";
    return clock((isLive(agent) ? now : tail.last ?? tail.first) - tail.first);
  }
  function waitsOn(task: Task): string {
    if (!task.depends_on.length) return "waits for a free worker";
    return task.depends_on.length === 1 ? `after ${task.depends_on[0]}` : `after ${task.depends_on.length} tasks`;
  }

  /** Sparkline cells follow the rendered width of the activity column: 1.6 of the row's 2.6 flexible parts. */
  let rowsWidth = $state(0);
  const buckets = $derived(Math.max(12, Math.min(120, Math.floor(((rowsWidth - 314) * 1.6) / 2.6 / 6.1))));
  const activity = $derived.by(() => {
    const rows = tasks.map((task) => {
      const agent = agentFor(task);
      const tail = agent ? tails[agent.id] : undefined;
      return { task, counts: tail ? activityLevels(tail.times, span.start, span.end, buckets) : new Array<number>(buckets).fill(0), tail, agent };
    });
    const peak = Math.max(1, ...rows.flatMap((row) => row.counts));
    const cell = (at: number) => Math.min(buckets - 1, Math.floor(((at - span.start) / Math.max(1, span.end - span.start)) * buckets));
    return new Map(rows.map(({ task, counts, tail, agent }) => [task.task_id, tail?.first != null
      ? sparkline(counts, peak, cell(tail.first), isLive(agent) ? buckets - 1 : cell(tail.last ?? tail.first))
      : sparkline(counts, peak, null, null)]));
  });

  // Follow work as it starts: a worker that leaves the queue scrolls into view,
  // and when the last one reports the board returns to the monitor. Neither
  // happens if the viewer moved the board themselves in the last few seconds.
  let board: HTMLDivElement | undefined = $state();
  let touched = 0;
  const seenTones = new Map<string, Tone>();
  $effect(() => {
    const wasLive = [...seenTones.values()].includes("live");
    const started = tasks.filter((task) => {
      const next = tone(agentFor(task));
      const before = seenTones.get(task.task_id);
      seenTones.set(task.task_id, next);
      return before === "queued" && next === "live";
    });
    const finished = wasLive && tasks.length > 0 && tasks.every((task) => tone(agentFor(task)) === "done");
    if ((!started.length && !finished) || !board || Date.now() - touched < 8_000) return;
    const behavior = matchMedia("(prefers-reduced-motion: reduce)").matches || document.querySelector("[data-force-reduced-motion]") ? "auto" : "smooth";
    const target = board;
    if (finished) { requestAnimationFrame(() => target.scrollTo({ top: 0, behavior })); return; }
    const pane = target.querySelector<HTMLElement>(`[data-task="${CSS.escape(started[0].task_id)}"]`);
    requestAnimationFrame(() => pane?.scrollIntoView({ block: "nearest", behavior }));
  });

  /** 1–9 jump to that worker's pane: the same numbers as the monitor rows and the scope. */
  function jumpToWorker(event: KeyboardEvent) {
    if (!/^[1-9]$/.test(event.key) || event.ctrlKey || event.metaKey || event.altKey || event.repeat) return;
    if ((event.target as HTMLElement | null)?.closest("input, textarea, select, [contenteditable]") || document.querySelector("[aria-modal='true'], dialog[open]")) return;
    const task = tasks[Number(event.key) - 1];
    const pane = task && board?.querySelector<HTMLElement>(`[data-task="${CSS.escape(task.task_id)}"]`);
    if (!pane) return;
    event.preventDefault();
    touched = Date.now();
    const still = matchMedia("(prefers-reduced-motion: reduce)").matches || !!document.querySelector("[data-force-reduced-motion]");
    pane.scrollIntoView({ block: "nearest", behavior: still ? "auto" : "smooth" });
    pane.querySelector<HTMLButtonElement>("button.head:not(:disabled)")?.focus({ preventScroll: true });
    board?.querySelectorAll(".jumped").forEach((element) => element.classList.remove("jumped"));
    void pane.offsetWidth;
    pane.classList.add("jumped");
  }

  const blips = $derived<Blip[]>(tasks.map((task, index) => ({ id: task.task_id, mark: String(index + 1), tone: tone(agentFor(task)), ring: task.wave })));
  const counts = $derived({
    live: tasks.filter((task) => tone(agentFor(task)) === "live").length,
    done: tasks.filter((task) => tone(agentFor(task)) === "done").length,
    waiting: tasks.filter((task) => tone(agentFor(task)) === "queued").length,
  });
</script>

<svelte:window onkeydown={jumpToWorker} />
<div class="fleet" data-testid="fleet-board" data-live={anyLive || undefined} style={`--cols:${columns}`} bind:this={board} onwheel={() => (touched = Date.now())} ontouchmove={() => (touched = Date.now())} onkeydown={() => (touched = Date.now())} role="presentation">
  <section class="monitor tui-pane" aria-label="Fleet monitor">
    <span class="tui-legend"><b>fleet</b><span>{counts.live} working · {counts.done} done · {counts.waiting} waiting</span></span>
    <span class="tui-legend right"><span class="pulse" class:on={anyLive} aria-hidden="true"></span>{anyLive ? "live" : "settled"} <time>{clock(span.end - span.start)}</time></span>
    <div class="scope"><FleetRadar {blips} rings={waves.length} /></div>
    <div class="rows">
      <ol aria-label="Worker activity from recorded events" bind:clientWidth={rowsWidth}>
        {#each tasks as task, index (task.task_id)}
          {@const agent = agentFor(task)}
          {@const spark = activity.get(task.task_id)}
          <li data-tone={tone(agent)}>
            <button disabled={!agent} onclick={() => agent && onInspect(agent)} aria-keyshortcuts={index < 9 ? String(index + 1) : undefined} aria-label={`${index + 1}. ${vendorOf(task, agent)}, ${taskDescriptions[task.task_id] ?? task.task_id}: ${status(task, agent)}${elapsed(agent) ? `, ${elapsed(agent)}` : ""}. Open output.`}>
              <span class="num">{index + 1}</span>
              <span class="cli">{cliOf(task, agent) ?? "agent"}</span>
              <span class="task" title={taskDescriptions[task.task_id] ?? task.task_id}>{taskDescriptions[task.task_id] ?? task.task_id}</span>
              <span class="spark" aria-hidden="true"><i>{spark?.lead}</i>{spark?.body}<i>{spark?.tail}</i></span>
              <span class="status">{#if tone(agent) === "live"}<span class="tui-spin" aria-hidden="true"><span>|/-\</span></span>{/if}{status(task, agent)}</span>
              <time>{elapsed(agent) || "—"}</time>
            </button>
          </li>
        {/each}
      </ol>
      <p class="facts"><span><b>{currentAgents.length}</b> workers started</span><span><b>{orderedShares}</b> {orderedShares === 1 ? "task" : "tasks"} ordered for shared paths</span><span>{#if checkedTasks.length}<b>{passedTasks}/{checkedTasks.length}</b> task checks passed{:else}No task checks configured{/if}</span></p>
    </div>
  </section>

  <div class="lanes">
    {#each waves as wave, index}
      {@const waiting = wave.every((task) => !agentFor(task))}
      {@const feeding = waiting && index > 0 && waves[index - 1].some((task) => isLive(agentFor(task)))}
      <section class="lane" class:feeding aria-label={`Step ${index + 1}`}>
        <h3><b>step {String(index + 1).padStart(2, "0")}</b><small>{wave.length > 1 ? `${wave.length} in parallel` : index === 0 ? "first" : "after its inputs"}</small></h3>
        <div class="cards">
          {#each wave as task (task.task_id)}
            {@const agent = agentFor(task)}
            {@const share = sharedOwner(task)}
            {@const all = agent && tails[agent.id] ? eventLines(tails[agent.id].events) : []}
            {@const offset = Math.max(0, all.length - 5)}
            {@const cli = cliOf(task, agent)}
            {@const vendor = vendorOf(task, agent)}
            {@const number = tasks.indexOf(task) + 1}
            {@const compact = !agent}
            <article class="worker tui-pane" data-task={task.task_id} class:compact data-tone={tone(agent)} data-unchanged={status(task, agent) === "no changes" || undefined} style={compact ? `grid-column:span ${Math.min(2, columns)}` : undefined}>
              <span class="tui-legend"><span class="logo">{#if cli}<AdeIdentity id={cli} />{/if}</span><span class="vendor">{vendor}</span><span class="num">{number}</span></span>
              <span class="tui-legend right state">{#if tone(agent) === "live"}<span class="tui-spin" aria-hidden="true"><span>|/-\</span></span>{:else if tone(agent) === "done"}<span aria-hidden="true">{status(task, agent) === "no changes" ? "○" : "✓"}</span>{:else if tone(agent) === "failed"}<span aria-hidden="true">✗</span>{/if}{status(task, agent)}{#if elapsed(agent)}<time>{elapsed(agent)}</time>{/if}</span>
              <button class="head" onclick={() => agent && onInspect(agent)} disabled={!agent} aria-label={`${vendor}: ${taskDescriptions[task.task_id] ?? task.task_id}. ${agent ? agentState(agent).label : "Not started"}. Open output.`}>
                <strong title={taskDescriptions[task.task_id]}>{taskDescriptions[task.task_id] ?? task.task_id}</strong>
                {#if compact}<small>{#if share}Shares {share.path} with {vendorOf(share.owner, agentFor(share.owner))}. Starts on its result.{:else}{waitsOn(task)}{/if}</small>{/if}
              </button>
              {#if !compact}
                <div class="term" role="log" aria-label={`Recent output from ${vendor}`}>
                  {#if all.length}
                    {#each all.slice(offset) as line, lineIndex (offset + lineIndex)}<div class="line" class:mark={line.startsWith("✓") || line.startsWith("$ ")} class:failed={line.startsWith("✗")} style={`--i:${lineIndex}`}><span>{line}</span>{#if tone(agent) === "live" && offset + lineIndex === all.length - 1}<i class="cursor" aria-hidden="true"></i>{/if}</div>{/each}
                  {:else}
                    <div class="hint">No output recorded yet.</div>
                  {/if}
                </div>
                {#if agent && readErrors[agent.id]}<p class="output-error" role="status">Output unavailable. Retrying...</p>{/if}
              {/if}
              <footer class="owns">
                {#each task.paths as path}<span title={path} class:shared={share?.path === path}>{path}</span>{/each}
              </footer>
            </article>
          {/each}
        </div>
      </section>
    {/each}
  </div>
</div>

<style>
  .fleet {
    --tui-bg: var(--pytxo-work-canvas);
    --mono: var(--pytxo-font-mono);
    container: fleet / inline-size; position: absolute; inset: 0; display: flex; flex-direction: column; gap: 22px; padding: 22px 24px 24px; overflow: auto; background: var(--tui-bg);
  }
  /* Monitor: scope on the left, one activity row per task on the right. */
  .monitor { flex: none; display: grid; grid-template-columns: 440px minmax(0, 1fr); gap: 12px; padding: 12px 16px 8px 4px; }
  .pulse { width: 7px; height: 7px; border-radius: 50%; background: var(--pytxo-text-muted); }
  .pulse.on { background: var(--pytxo-activity); box-shadow: 0 0 0 0 color-mix(in srgb, var(--pytxo-activity) 60%, transparent); animation: pulse 1.6s ease-out infinite; }
  @keyframes pulse { to { box-shadow: 0 0 0 7px transparent; } }
  .scope { display: grid; place-items: center; }
  .rows { display: flex; min-width: 0; flex-direction: column; justify-content: center; gap: 6px; }
  .rows ol { display: grid; margin: 0; padding: 0; list-style: none; max-height: 196px; overflow: auto; }
  .rows button { display: grid; grid-template-columns: 20px 68px minmax(0, 1fr) minmax(0, 1.6fr) 118px 42px; align-items: center; gap: 10px; width: 100%; height: 24px; padding: 0 8px; border: 0; border-radius: 4px; background: transparent; color: var(--pytxo-text-body); font: 12px var(--mono); text-align: left; }
  .rows button:hover:not(:disabled) { background: var(--pytxo-surface-hover); }
  .rows button:disabled { cursor: default; }
  .rows .num { color: var(--pytxo-text-muted); }
  .rows .cli { overflow: hidden; color: var(--pytxo-text-strong); text-overflow: ellipsis; white-space: nowrap; }
  .rows .task { overflow: hidden; color: var(--pytxo-text-soft); text-overflow: ellipsis; white-space: nowrap; }
  .spark { overflow: hidden; color: var(--pytxo-text-muted); letter-spacing: -.02em; white-space: pre; line-height: 1; }
  .spark i { font-style: normal; opacity: .35; }
  li[data-tone="live"] .spark { color: var(--pytxo-activity); text-shadow: 0 0 7px color-mix(in srgb, var(--pytxo-activity) 45%, transparent); }
  li[data-tone="done"] .spark { color: color-mix(in srgb, var(--state-verified) 75%, var(--pytxo-text-muted)); }
  li[data-tone="failed"] .spark { color: var(--state-refuted); }
  .rows .status { display: flex; align-items: center; gap: 6px; overflow: hidden; color: var(--pytxo-text-muted); white-space: nowrap; }
  li[data-tone="live"] .status { color: var(--pytxo-activity); }
  li[data-tone="done"] .status { color: var(--state-verified); }
  li[data-tone="failed"] .status { color: var(--state-refuted); }
  .rows time { color: var(--pytxo-text-muted); font-variant-numeric: tabular-nums; text-align: right; }
  .facts { display: flex; flex-wrap: wrap; gap: 4px 22px; margin: 2px 0 0; padding: 6px 8px 0; border-top: 1px dashed var(--pytxo-line-soft); color: var(--pytxo-text-muted); font: 11.5px var(--mono); }
  .facts b { margin-right: 5px; color: var(--pytxo-text-strong); font-weight: 600; }


  .lanes { display: grid; flex: none; gap: 26px; }
  .lane { position: relative; display: grid; grid-template-columns: 92px minmax(0, 1fr); gap: 14px; }
  /* Work flows down the gutter; it moves only while an earlier step feeds a waiting one. */
  .lane + .lane::before { content: ""; position: absolute; left: 14px; top: -24px; height: 22px; border-left: 1px dashed var(--pytxo-line); }
  .lane.feeding::before { border-left: 0; width: 1px; background: repeating-linear-gradient(to bottom, var(--pytxo-activity) 0 4px, transparent 4px 8px); background-size: 1px 16px; animation: feed .7s linear infinite; }
  @keyframes feed { to { background-position: 0 16px; } }
  .lane h3 { display: grid; align-content: start; gap: 4px; margin: 4px 0 0; }
  .lane h3 b { color: var(--pytxo-text-strong); font: 600 11.5px var(--mono); letter-spacing: .1em; text-transform: uppercase; }
  .lane h3 small { color: var(--pytxo-text-muted); font: 11px var(--mono); }
  .cards { display: grid; grid-template-columns: repeat(var(--cols), minmax(0, 1fr)); gap: 22px 14px; }

  /* A queued pane opens into a terminal when its worker starts; a finished one settles with one ring. */
  .worker { container: worker / inline-size; display: grid; grid-template-rows: auto 1fr auto; height: 206px; padding-top: 8px; interpolate-size: allow-keywords; transition: height .35s cubic-bezier(.2, 0, 0, 1), border-color .3s, background-color .3s; }
  .worker[data-tone="done"]:not([data-unchanged]) { animation: settle 1s ease-out 1; }
  @keyframes settle { from { box-shadow: 0 0 0 3px color-mix(in srgb, var(--state-verified) 40%, transparent); } }
  .worker.compact { grid-template-rows: auto auto; height: auto; border-style: dashed; }
  .worker .tui-legend .logo { display: grid; place-items: center; width: 16px; height: 16px; }
  .worker .tui-legend .logo :global(.ade-identity) { transform: scale(.66); }
  .worker .tui-legend .vendor { overflow: hidden; color: var(--pytxo-text-strong); text-overflow: ellipsis; }
  .worker .tui-legend .num { color: var(--pytxo-text-muted); }
  .worker .tui-legend .num::before { content: "#"; }
  .worker .state { gap: 6px; color: var(--pytxo-text-muted); }
  .worker[data-tone="live"] { border-color: color-mix(in srgb, var(--pytxo-activity) 55%, var(--pytxo-line)); background: color-mix(in srgb, var(--pytxo-activity) 3%, transparent); }
  .worker[data-tone="live"] .state { color: var(--pytxo-activity); }
  .worker[data-tone="done"] { border-color: color-mix(in srgb, var(--state-verified) 40%, var(--pytxo-line)); }
  .worker[data-tone="done"] .state { color: var(--state-verified); }
  .worker[data-unchanged] .state { color: var(--pytxo-text-muted); }
  .worker[data-tone="failed"] { border-color: color-mix(in srgb, var(--state-refuted) 60%, var(--pytxo-line)); background: color-mix(in srgb, var(--state-refuted) 4%, transparent); }
  .worker[data-tone="failed"] .state { color: var(--state-refuted); }
  .head { display: grid; gap: 4px; padding: 8px 14px 8px; border: 0; border-radius: 0; background: transparent; text-align: left; }
  .head:hover:not(:disabled) strong { text-decoration: underline; text-decoration-color: var(--pytxo-line); text-underline-offset: 3px; }
  .head:disabled { cursor: default; }
  .head:focus-visible { outline: 2px solid var(--pytxo-accent); outline-offset: -2px; border-radius: 4px; }
  /* Task titles are the request's own words: two lines before they clip. */
  .head strong { display: -webkit-box; overflow: hidden; color: var(--pytxo-text-strong); font: 600 13.5px/1.3 var(--pytxo-font-ui); -webkit-line-clamp: 2; line-clamp: 2; -webkit-box-orient: vertical; }
  .compact .head strong { -webkit-line-clamp: 1; line-clamp: 1; }
  .head small { color: var(--pytxo-text-muted); font: 11.5px/1.5 var(--mono); }
  .term { position: relative; display: flex; flex-direction: column; justify-content: flex-end; min-height: 0; overflow: hidden; margin: 0 8px; padding: 6px 8px; border-radius: 4px; background: color-mix(in srgb, var(--pytxo-code-surface) 70%, transparent); color: var(--pytxo-text-body); font: 11.5px/1.65 var(--mono); }
  .term .line { display: flex; flex: none; align-items: center; gap: 2px; animation: type-in .5s steps(24, end) both; animation-delay: calc(var(--i) * 70ms); }
  .term .line span { min-width: 0; overflow: hidden; text-overflow: ellipsis; white-space: nowrap; }
  /* New output types in once; settled lines never move. */
  @keyframes type-in { from { clip-path: inset(0 100% 0 0); } }
  .term .line:last-of-type { color: var(--pytxo-text-strong); }
  /* Newest lines sit at the bottom; older ones fade out at the top instead of being cut mid-line. */
  .term { -webkit-mask-image: linear-gradient(to bottom, transparent 0, #000 20px); mask-image: linear-gradient(to bottom, transparent 0, #000 20px); }
  .term .mark { color: var(--state-verified) !important; }
  .term .failed { color: var(--state-refuted) !important; }
  .term .hint { color: var(--pytxo-text-muted); white-space: normal; }
  .cursor { flex: none; width: 7px; height: 13px; background: var(--pytxo-activity); animation: blink 1.05s steps(1) infinite; }
  @keyframes blink { 50% { opacity: 0; } }
  .output-error { margin: 4px 14px 0; color: var(--pytxo-text-soft); font-size: 12px; }
  .owns { display: flex; gap: 10px; overflow: hidden; padding: 8px 14px 10px; }
  .owns span { flex: none; max-width: 100%; overflow: hidden; text-overflow: ellipsis; white-space: nowrap; color: var(--pytxo-text-soft); font: 11.5px var(--mono); }
  .owns span::before { content: "▸ "; color: var(--pytxo-text-muted); }
  .owns span.shared { color: var(--pytxo-glyph-warm, #f3c64e); }
  .owns span.shared::before { content: "⇄ "; color: inherit; }
  .compact .owns { padding-top: 0; }

  @media (prefers-reduced-motion: reduce) { .pulse.on, .lane.feeding::before, .term .line, .cursor, .worker { animation: none; transition: none; } }
  :global([data-force-reduced-motion]) .pulse.on, :global([data-force-reduced-motion]) .lane.feeding::before, :global([data-force-reduced-motion]) .term .line, :global([data-force-reduced-motion]) .cursor, :global([data-force-reduced-motion]) .worker { animation: none; transition: none; }

  @container fleet (max-width: 1240px) { .monitor { grid-template-columns: 330px minmax(0, 1fr); } .scope :global(.fleet-radar) { width: 330px; height: 153px; } }
  @container fleet (max-width: 1100px) { .cards { grid-template-columns: repeat(min(var(--cols), 2), minmax(0, 1fr)); } }
  @container fleet (max-width: 840px) { .monitor { grid-template-columns: minmax(0, 1fr); padding-left: 16px; } .scope { display: none; } }
  @container fleet (max-width: 640px) {
    .lane { grid-template-columns: minmax(0, 1fr); gap: 10px; } .lane h3 { display: flex; gap: 10px; margin: 0; } .lane + .lane::before { display: none; }
    .cards { grid-template-columns: minmax(0, 1fr); } .worker.compact { grid-column: auto !important; }
    .rows button { grid-template-columns: 18px 60px minmax(0, 1fr) 96px 40px; } .rows .spark { display: none; }
  }
  .worker:global(.jumped) { animation: pane-jump .9s ease-out; }
  @keyframes pane-jump { 0%, 30% { border-color: var(--pytxo-activity); box-shadow: 0 0 0 2px color-mix(in srgb, var(--pytxo-activity) 30%, transparent); } }
  @media (prefers-reduced-motion: reduce) { .worker:global(.jumped) { animation: none; } }
</style>
