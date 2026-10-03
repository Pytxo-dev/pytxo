<script lang="ts">
  import { onDestroy } from "svelte";
  import { agentState } from "../../lib/epistemic";
  import { eventLines } from "../../lib/terminal-text";
  import { adeDisplayName } from "../../lib/ade-status";
  import type { DesktopBackend } from "../../lib/desktop-backend";
  import type { AgentDto, EventDto, RunDto, RunReviewDto } from "../../lib/types";
  import AdeIdentity from "./AdeIdentity.svelte";

  let {
    run, review, agents, taskDescriptions = {}, taskClis = {}, backend = null, onInspect,
  }: {
    run: RunDto; review: RunReviewDto | null; agents: AgentDto[];
    taskDescriptions?: Record<string, string>; taskClis?: Record<string, string>; backend?: DesktopBackend | null;
    onInspect: (agent: AgentDto) => void;
  } = $props();

  type Task = RunReviewDto["plan"]["waves"][number][number];
  type Tail = { cursor: number; events: EventDto[]; first: number | null; last: number | null };

  const waves = $derived((review?.plan.waves ?? []).filter((wave) => wave.length));
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
  const orderedShares = $derived(waves.flat().filter((task) => sharedOwner(task)).length);
  /** Tasks with prepared files; null until a candidate exists. A worker can pass its checks and change nothing. */
  const preparedTasks = $derived(review?.prepared_manifest ? new Set(review.prepared_manifest.files.map((file) => file.task_id)) : null);

  let tails = $state<Record<string, Tail>>({});
  let now = $state(Date.now());
  let timer: ReturnType<typeof setInterval> | undefined;
  let polling = false;

  function isLive(agent: AgentDto | undefined) {
    return !!agent && ["running", "starting", "pending"].includes(agent.status);
  }

  async function poll() {
    if (!backend || polling) return;
    polling = true;
    try {
      const current = agents.filter((agent) => agent.run_id === run.id && agent.domain_id === run.domain_id);
      await Promise.all(current.map(async (agent) => {
        const tail = tails[agent.id];
        // Settled workers are read once; live workers keep streaming from their cursor.
        if (tail && !isLive(agent)) return;
        let cursor = tail?.cursor ?? 0;
        const collected = tail?.events ?? [];
        let first = tail?.first ?? null;
        let last = tail?.last ?? null;
        for (let page = 0; page < 10; page += 1) {
          const events = await backend!.readAgentEvents(run.id, agent.id, agent.domain_id, cursor, 200);
          if (!events.length) break;
          for (const event of events) {
            const at = Date.parse(event.ts);
            if (Number.isFinite(at)) { first ??= at; last = at; }
          }
          collected.push(...events);
          cursor = events[events.length - 1].id;
          if (events.length < 200) break;
        }
        // Keep the recent window only; the full record stays in the worker output dock.
        tails[agent.id] = { cursor, events: collected.slice(-400), first, last };
      }));
    } catch {
      // A failed read keeps the last recorded lines; the dock shows the full error path.
    } finally {
      polling = false;
      now = Date.now();
    }
  }

  $effect(() => {
    void `${run.domain_id}:${run.id}:${agents.map((agent) => `${agent.id}:${agent.status}`).join(",")}`;
    void poll();
  });
  $effect(() => {
    timer = setInterval(() => { if (agents.some(isLive)) void poll(); else now = Date.now(); }, 1500);
    return () => clearInterval(timer);
  });
  onDestroy(() => clearInterval(timer));

  const span = $derived.by(() => {
    const starts = Object.values(tails).map((tail) => tail.first).filter((value): value is number => value !== null);
    const ends = Object.values(tails).map((tail) => tail.last).filter((value): value is number => value !== null);
    const start = starts.length ? Math.min(...starts) : Date.parse(run.started_at);
    const end = Math.max(agents.some(isLive) ? now : 0, ...ends, start + 1000);
    return { start, end };
  });
  function bar(agent: AgentDto | undefined) {
    const tail = agent ? tails[agent.id] : undefined;
    if (!tail || tail.first === null) return null;
    const total = span.end - span.start;
    const end = isLive(agent) ? span.end : tail.last ?? tail.first;
    return { left: ((tail.first - span.start) / total) * 100, width: Math.max(1.5, ((end - tail.first) / total) * 100) };
  }
  function clock(ms: number) {
    const seconds = Math.max(0, Math.round(ms / 1000));
    return `${Math.floor(seconds / 60)}:${String(seconds % 60).padStart(2, "0")}`;
  }
  function tone(agent: AgentDto | undefined) {
    if (!agent) return "queued";
    if (isLive(agent)) return "live";
    if (agent.status === "completed" && agent.exit_code === 0) return "done";
    if (["failed", "blocked_by_dependency", "verify_failed"].includes(agent.status)) return "failed";
    return "settled";
  }
</script>

<div class="fleet" data-testid="fleet-board">
  <div class="lanes">
    {#each waves as wave, index}
      {@const columns = wave.length >= 3 ? 2 : 1}
      <section class="lane" style={`--columns:${columns};flex:${columns}`} aria-label={`Step ${index + 1}`}>
        <h3><span>Step {index + 1}</span><small>{wave.length > 1 ? `${wave.length} in parallel` : index === 0 ? "first" : "after its inputs"}</small></h3>
        <div class="cards">
          {#each wave as task (task.task_id)}
            {@const agent = agentFor(task)}
            {@const state = agent ? agentState(agent) : null}
            {@const share = sharedOwner(task)}
            {@const lines = agent && tails[agent.id] ? eventLines(tails[agent.id].events).slice(-14) : []}
            {@const cli = cliOf(task, agent)}
            {@const vendor = vendorOf(task, agent)}
            {@const unchanged = tone(agent) === "done" && !!preparedTasks && !preparedTasks.has(task.task_id)}
            <article class="worker" data-tone={tone(agent)} data-unchanged={unchanged || undefined}>
              <button class="head" onclick={() => agent && onInspect(agent)} disabled={!agent} aria-label={`${vendor}: ${taskDescriptions[task.task_id] ?? task.task_id}. ${state?.label ?? "Not started"}. Open output.`}>
                <span class="logo">{#if cli}<AdeIdentity id={cli} />{/if}</span>
                <span class="who"><strong>{vendor}</strong><small title={taskDescriptions[task.task_id]}>{taskDescriptions[task.task_id] ?? task.task_id}</small></span>
                <span class="state">{#if unchanged}No changes{:else if tone(agent) === "done"}<span class="wide">✓ Checks passed</span><span class="narrow">✓ Passed</span>{:else}{tone(agent) === "live" ? "● Working" : tone(agent) === "queued" ? (share ? "Next" : "Queued") : state?.label ?? "Settled"}{/if}</span>
              </button>
              <div class="term" role="log" aria-label={`Recent output from ${vendor}`}>
                {#if lines.length}
                  {#each lines as line, lineIndex (lineIndex)}<div class:mark={line.startsWith("✓") || line.startsWith("✗") || line.startsWith("$ ")}>{line}</div>{/each}
                {:else if share && !agent}
                  <div class="hint">Shares {share.path} with {vendorOf(share.owner, agentFor(share.owner))}.</div>
                  <div class="hint">Starts on its result, so neither overwrites the other.</div>
                {:else}
                  <div class="hint">{agent ? "No output recorded yet." : "Waits for its inputs."}</div>
                {/if}
              </div>
              <footer class="owns">
                {#each task.paths as path}<span class:shared={share?.path === path}>{path}</span>{/each}
              </footer>
            </article>
          {/each}
        </div>
      </section>
    {/each}
  </div>

  <section class="activity" aria-label="Worker activity from recorded events">
    <h4><span>Activity</span><small>from recorded events · {clock(span.end - span.start)}</small></h4>
    {#each waves.flat() as task (task.task_id)}
      {@const agent = agentFor(task)}
      {@const segment = bar(agent)}
      <div class="row"><span>{vendorOf(task, agent)}</span><div class="track">{#if segment}<i data-tone={tone(agent)} style={`left:${segment.left}%;width:${segment.width}%`}></i>{/if}</div></div>
    {/each}
    <p class="facts"><span><b>{agents.filter((agent) => agent.run_id === run.id).length}</b> isolated workspaces</span><span><b>{orderedShares}</b> shared {orderedShares === 1 ? "path" : "paths"} ordered</span><span><b>{agents.filter((agent) => agent.status === "completed" && agent.exit_code === 0).length}/{waves.flat().length}</b> task checks passed</span></p>
  </section>
</div>

<style>
  .fleet { position: absolute; inset: 0; display: flex; flex-direction: column; gap: 14px; padding: 16px 18px; overflow: auto; }
  .lanes { display: flex; flex: 1 0 auto; gap: 18px; }
  .lane { display: flex; flex-direction: column; min-width: 0; }
  .lane h3 { display: flex; justify-content: space-between; align-items: baseline; margin: 0 2px 10px; color: var(--pytxo-text-muted); font: 500 12px var(--pytxo-font-mono, "IBM Plex Mono", monospace); letter-spacing: .06em; text-transform: uppercase; }
  .lane h3 small { color: var(--pytxo-text-soft); font: 13px var(--pytxo-font-ui); letter-spacing: 0; text-transform: none; }
  .cards { display: grid; grid-template-columns: repeat(var(--columns), minmax(0, 1fr)); grid-auto-rows: minmax(184px, 1fr); gap: 14px; flex: 1 0 auto; }
  .worker { container: worker / inline-size; display: flex; flex-direction: column; min-height: 0; overflow: hidden; border: 1px solid var(--pytxo-line); border-radius: 10px; background: var(--pytxo-surface-panel); }
  .worker[data-tone="live"] { border-color: color-mix(in srgb, var(--pytxo-activity) 55%, var(--pytxo-line)); box-shadow: 0 0 0 1px color-mix(in srgb, var(--pytxo-activity) 18%, transparent); }
  .worker[data-tone="done"] { border-color: color-mix(in srgb, var(--state-verified) 40%, var(--pytxo-line)); }
  .worker[data-tone="failed"] { border-color: color-mix(in srgb, var(--state-refuted) 50%, var(--pytxo-line)); }
  /* A waiting worker has nothing to show yet: keep it compact at the top of its lane. */
  .worker[data-tone="queued"] { align-self: start; border-style: dashed; opacity: .78; }
  .worker[data-tone="queued"] .term { flex: none; min-height: 0; justify-content: flex-start; }
  .head { display: grid; grid-template-columns: 30px minmax(0, 1fr) auto; gap: 2px 12px; align-items: center; padding: 12px 14px 10px; border: 0; border-radius: 0; background: transparent; text-align: left; justify-content: stretch; }
  .head:hover:not(:disabled) { background: var(--pytxo-surface-hover); }
  .logo { grid-row: 1 / span 2; display: grid; place-items: center; width: 30px; height: 30px; border-radius: 8px; background: var(--pytxo-surface-raised); }
  /* The vendor shares its row with the state; the task line spans both. */
  .who { display: contents; }
  .who strong { grid-column: 2; grid-row: 1; overflow: hidden; color: var(--pytxo-text-strong); font-size: 16px; font-weight: 600; text-overflow: ellipsis; white-space: nowrap; }
  .who small { grid-column: 2 / -1; grid-row: 2; overflow: hidden; color: var(--pytxo-text-soft); font-size: 14px; text-overflow: ellipsis; white-space: nowrap; }
  .state { grid-column: 3; grid-row: 1; font-size: 13px; font-weight: 600; color: var(--pytxo-text-muted); white-space: nowrap; }
  .worker[data-tone="live"] .state { color: var(--pytxo-activity); }
  .worker[data-tone="done"] .state { color: var(--state-verified); }
  .worker[data-unchanged] .state { color: var(--pytxo-text-muted); }
  .worker[data-tone="failed"] .state { color: var(--state-refuted); }
  .term { display: flex; flex: 1; flex-direction: column; justify-content: flex-end; min-height: 90px; overflow: hidden; padding: 10px 14px; border-top: 1px solid var(--pytxo-line-soft); background: color-mix(in srgb, var(--pytxo-surface-shell) 70%, black); color: var(--pytxo-text-body); font: 13px/1.5 var(--pytxo-font-mono, "IBM Plex Mono", monospace); }
  .term div { flex: none; overflow: hidden; text-overflow: ellipsis; white-space: nowrap; }
  .term .mark { color: var(--state-verified); }
  .term .hint { color: var(--pytxo-text-muted); white-space: normal; }
  .owns { display: flex; flex-wrap: wrap; gap: 6px; padding: 8px 14px 11px; border-top: 1px solid var(--pytxo-line-soft); }
  .owns span { padding: 2px 7px; border: 1px solid var(--pytxo-line); border-radius: 5px; background: var(--pytxo-surface-raised); color: var(--pytxo-text-body); font: 12.5px var(--pytxo-font-mono, "IBM Plex Mono", monospace); }
  .owns span.shared { border-color: color-mix(in srgb, var(--pytxo-glyph-warm, #f3c64e) 55%, var(--pytxo-line)); color: var(--pytxo-glyph-warm, #f3c64e); }
  .state .narrow { display: none; }
  @container worker (max-width: 320px) { .state .wide { display: none; } .state .narrow { display: inline; } }
  .activity { flex: none; padding: 12px 16px 10px; border: 1px solid var(--pytxo-line); border-radius: 10px; background: var(--pytxo-surface-panel); }
  .activity h4 { display: flex; justify-content: space-between; margin: 0 0 10px; color: var(--pytxo-text-strong); font-size: 15px; font-weight: 600; }
  .activity h4 small { color: var(--pytxo-text-muted); font-weight: 400; font-size: 13px; }
  .row { display: grid; grid-template-columns: 130px minmax(0, 1fr); align-items: center; gap: 12px; height: 22px; color: var(--pytxo-text-body); font-size: 13.5px; }
  .track { position: relative; height: 12px; border-radius: 6px; background: color-mix(in srgb, var(--pytxo-surface-raised) 80%, transparent); }
  .track i { position: absolute; top: 0; bottom: 0; border-radius: 6px; background: var(--pytxo-text-muted); }
  .track i[data-tone="live"] { background: linear-gradient(90deg, color-mix(in srgb, var(--pytxo-activity) 35%, transparent), var(--pytxo-activity)); }
  .track i[data-tone="done"] { background: var(--state-verified); }
  .track i[data-tone="failed"] { background: var(--state-refuted); }
  .facts { display: flex; flex-wrap: wrap; gap: 8px 26px; margin: 10px 0 0; color: var(--pytxo-text-soft); font-size: 14px; }
  .facts b { margin-right: 5px; color: var(--pytxo-text-strong); font-size: 16px; }
  @media (max-width: 900px) { .lanes { flex-direction: column; } .cards { grid-auto-rows: minmax(200px, auto); } }
</style>
