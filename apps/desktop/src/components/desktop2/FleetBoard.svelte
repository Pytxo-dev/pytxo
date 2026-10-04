<script lang="ts">
  import { onDestroy, untrack } from "svelte";
  import { agentState } from "../../lib/epistemic";
  import { eventLines } from "../../lib/terminal-text";
  import { adeDisplayName } from "../../lib/ade-status";
  import { readFleetTail, type FleetTail } from "../../lib/fleet-events";
  import type { DesktopBackend } from "../../lib/desktop-backend";
  import type { AgentDto, RunDto, RunReviewDto } from "../../lib/types";
  import AdeIdentity from "./AdeIdentity.svelte";

  let {
    run, review, agents, taskDescriptions = {}, taskClis = {}, backend = null, onInspect,
  }: {
    run: RunDto; review: RunReviewDto | null; agents: AgentDto[];
    taskDescriptions?: Record<string, string>; taskClis?: Record<string, string>; backend?: DesktopBackend | null;
    onInspect: (agent: AgentDto) => void;
  } = $props();

  type Task = RunReviewDto["plan"]["waves"][number][number];

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
  const checkedTasks = $derived(waves.flat().filter((task) => task.verify.length > 0));
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
  onDestroy(() => { disposed = true; generation += 1; clearInterval(timer); });

  const span = $derived.by(() => {
    const starts = Object.values(tails).map((tail) => tail.first).filter((value): value is number => value !== null);
    const ends = Object.values(tails).map((tail) => tail.last).filter((value): value is number => value !== null);
    const start = starts.length ? Math.min(...starts) : Date.parse(run.started_at);
    const end = Math.max(currentAgents.some(isLive) ? now : 0, ...ends, start + 1000);
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
      <section class="lane" style={`--columns:${columns};--weight:${columns}`} aria-label={`Step ${index + 1}`}>
        <h3><span>Step {index + 1}</span><small>{wave.length > 1 ? `${wave.length} in parallel` : index === 0 ? "first" : "after its inputs"}</small></h3>
        <div class="cards">
          {#each wave as task (task.task_id)}
            {@const agent = agentFor(task)}
            {@const state = agent ? agentState(agent) : null}
            {@const share = sharedOwner(task)}
            {@const lines = agent && tails[agent.id] ? eventLines(tails[agent.id].events).slice(-7) : []}
            {@const cli = cliOf(task, agent)}
            {@const vendor = vendorOf(task, agent)}
            {@const unchanged = tone(agent) === "done" && !!preparedTasks && !preparedTasks.has(task.task_id)}
            <article class="worker" data-tone={tone(agent)} data-unchanged={unchanged || undefined}>
              <button class="head" onclick={() => agent && onInspect(agent)} disabled={!agent} aria-label={`${vendor}: ${taskDescriptions[task.task_id] ?? task.task_id}. ${state?.label ?? "Not started"}. Open output.`}>
                <span class="logo">{#if cli}<AdeIdentity id={cli} />{/if}</span>
                <span class="who"><strong>{vendor}</strong><small title={taskDescriptions[task.task_id]}>{taskDescriptions[task.task_id] ?? task.task_id}</small></span>
                <span class="state">{#if unchanged}No changes{:else if tone(agent) === "done"}{#if task.verify.length}<span class="wide">✓ Checks passed</span><span class="narrow">✓ Passed</span>{:else}Completed{/if}{:else}{tone(agent) === "live" ? "● Working" : tone(agent) === "queued" ? (share ? "Next" : "Queued") : state?.label ?? "Settled"}{/if}</span>
              </button>
              <div class="term" role="log" aria-label={`Recent output from ${vendor}`}>
                {#if lines.length}
                  {#each lines as line, lineIndex (lineIndex)}<div class:mark={line.startsWith("✓") || line.startsWith("$ ")} class:failed={line.startsWith("✗")}>{line}</div>{/each}
                {:else if share && !agent}
                  <div class="hint">Shares {share.path} with {vendorOf(share.owner, agentFor(share.owner))}.</div>
                  <div class="hint">Starts on its result, so neither overwrites the other.</div>
                {:else}
                  <div class="hint">{agent ? "No output recorded yet." : "Waits for its inputs."}</div>
                {/if}
              </div>
              {#if agent && readErrors[agent.id]}<p class="output-error" role="status">Output unavailable. Retrying...</p>{/if}
              <footer class="owns">
                {#each task.paths as path}<span title={path} class:shared={share?.path === path}>{path}</span>{/each}
              </footer>
            </article>
          {/each}
        </div>
      </section>
    {/each}
  </div>

  <p class="facts"><span><b>{currentAgents.length}</b> workers started</span><span><b>{orderedShares}</b> {orderedShares === 1 ? "task" : "tasks"} ordered for shared paths</span><span>{#if checkedTasks.length}<b>{passedTasks}/{checkedTasks.length}</b> task checks passed{:else}No task checks configured{/if}</span></p>
  <details class="activity" aria-label="Worker activity from recorded events">
    <summary><span>Activity timeline</span><small>Recorded events · {clock(span.end - span.start)}</small></summary>
    {#each waves.flat() as task (task.task_id)}
      {@const agent = agentFor(task)}
      {@const segment = bar(agent)}
      <div class="row"><span>{vendorOf(task, agent)}</span><div class="track">{#if segment}<i data-tone={tone(agent)} style={`left:${segment.left}%;width:${segment.width}%`}></i>{/if}</div></div>
    {/each}
  </details>
</div>

<style>
  .fleet { container: fleet / inline-size; position: absolute; inset: 0; display: flex; flex-direction: column; gap: 18px; padding: 18px 20px; overflow: auto; }
  .lanes { display: flex; flex: none; gap: 20px; }
  .lane { display: flex; flex: var(--weight); flex-direction: column; min-width: 0; }
  .lane h3 { display: flex; justify-content: space-between; align-items: baseline; gap: 12px; margin: 0 0 12px; color: var(--pytxo-text-strong); font: 600 13px var(--pytxo-font-ui); letter-spacing: 0; }
  .lane h3 small { color: var(--pytxo-text-soft); font: 13px var(--pytxo-font-ui); letter-spacing: 0; text-transform: none; }
  .cards { display: grid; grid-template-columns: repeat(var(--columns), minmax(0, 1fr)); grid-auto-rows: auto; align-content: start; align-items: start; gap: 14px; }
  .worker { container: worker / inline-size; display: flex; flex-direction: column; min-height: 0; overflow: hidden; border: 1px solid var(--pytxo-line); border-radius: 8px; background: var(--pytxo-surface-panel); }
  /* An active worker carries an activity rail: it shows activity, never progress. */
  .worker[data-tone="live"] { position: relative; }
  .worker[data-tone="live"]::before { content: ""; position: absolute; inset: 0 0 auto 0; height: 2px; background: var(--pytxo-aperture-horizontal); background-size: 200% 100%; animation: activity-rail 2.4s linear infinite; pointer-events: none; }
  @keyframes activity-rail { to { background-position: -200% 0; } }
  @media (prefers-reduced-motion: reduce) { .worker[data-tone="live"]::before { animation: none; } }
  :global([data-force-reduced-motion]) .worker[data-tone="live"]::before { animation: none; }
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
  .term { display: flex; flex: none; flex-direction: column; justify-content: flex-end; height: 132px; overflow: hidden; padding: 10px 14px; border-top: 1px solid var(--pytxo-line-soft); background: var(--pytxo-code-surface); color: var(--pytxo-text-body); font: 13px/1.5 var(--pytxo-font-mono, "IBM Plex Mono", monospace); }
  .worker[data-tone="queued"] .term { height: auto; }
  .term div { flex: none; overflow: hidden; text-overflow: ellipsis; white-space: nowrap; }
  .term .mark { color: var(--state-verified); }
  .term .failed { color: var(--state-refuted); }
  .term .hint { color: var(--pytxo-text-muted); white-space: normal; }
  .output-error { margin: 0; padding: 8px 14px; color: var(--pytxo-text-soft); font-size: 12px; }
  .owns { display: flex; flex-wrap: wrap; gap: 6px; padding: 8px 14px 11px; border-top: 1px solid var(--pytxo-line-soft); }
  .owns span { max-width: 100%; overflow: hidden; text-overflow: ellipsis; white-space: nowrap; padding: 2px 7px; border: 1px solid var(--pytxo-line); border-radius: 4px; background: var(--pytxo-surface-raised); color: var(--pytxo-text-body); font: 12.5px var(--pytxo-font-mono, "IBM Plex Mono", monospace); }
  .owns span.shared { border-color: color-mix(in srgb, var(--pytxo-glyph-warm, #f3c64e) 55%, var(--pytxo-line)); color: var(--pytxo-glyph-warm, #f3c64e); }
  .state .narrow { display: none; }
  @container worker (max-width: 320px) { .state .wide { display: none; } .state .narrow { display: inline; } }
  .activity { flex: none; padding: 0 0 12px; border-top: 1px solid var(--pytxo-line-soft); }
  .activity summary { display: flex; justify-content: space-between; align-items: center; gap: 16px; min-height: 40px; color: var(--pytxo-text-strong); font-size: 13px; font-weight: 500; cursor: pointer; }
  .activity summary::before { content: "+"; color: var(--pytxo-text-muted); }
  .activity[open] summary::before { content: "−"; }
  .activity summary small { margin-left: auto; color: var(--pytxo-text-muted); font-weight: 400; font-size: 12px; }
  .row { display: grid; grid-template-columns: 130px minmax(0, 1fr); align-items: center; gap: 12px; height: 22px; color: var(--pytxo-text-body); font-size: 13.5px; }
  .track { position: relative; height: 12px; border-radius: 6px; background: color-mix(in srgb, var(--pytxo-surface-raised) 80%, transparent); }
  .track i { position: absolute; top: 0; bottom: 0; border-radius: 6px; background: var(--pytxo-text-muted); }
  .track i[data-tone="live"] { background: linear-gradient(90deg, color-mix(in srgb, var(--pytxo-activity) 35%, transparent), var(--pytxo-activity)); }
  .track i[data-tone="done"] { background: var(--state-verified); }
  .track i[data-tone="failed"] { background: var(--state-refuted); }
  .facts { display: flex; flex-wrap: wrap; gap: 8px 26px; margin: 0; color: var(--pytxo-text-soft); font-size: 13px; }
  .facts b { margin-right: 5px; color: var(--pytxo-text-strong); font-size: 16px; }
  @container fleet (max-width: 1050px) { .lanes { flex-wrap: wrap; }.lane:first-child { flex-basis: 100%; }.lane:not(:first-child) { flex: 1 1 240px; } }
  @container fleet (max-width: 560px) { .lanes { flex-direction: column; }.lane:not(:first-child) { flex: none; }.cards { grid-template-columns: minmax(0, 1fr); }.activity summary { flex-wrap: wrap; gap: 8px; }.row { grid-template-columns: 110px minmax(0, 1fr); } }
</style>
