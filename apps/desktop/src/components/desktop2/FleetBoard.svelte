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
  /** One column count for every lane, so cards share a width across steps. */
  const columns = $derived(Math.min(4, Math.max(1, ...waves.map((wave) => wave.length))));
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

<div class="fleet" data-testid="fleet-board" style={`--cols:${columns}`}>
  <div class="lanes">
    {#each waves as wave, index}
      <section class="lane" aria-label={`Step ${index + 1}`}>
        <h3><span>Step {index + 1}</span><small>{wave.length > 1 ? `${wave.length} in parallel` : index === 0 ? "first" : "after its inputs"}</small></h3>
        <div class="cards">
          {#each wave as task (task.task_id)}
            {@const agent = agentFor(task)}
            {@const state = agent ? agentState(agent) : null}
            {@const share = sharedOwner(task)}
            {@const lines = agent && tails[agent.id] ? eventLines(tails[agent.id].events).slice(-5) : []}
            {@const cli = cliOf(task, agent)}
            {@const vendor = vendorOf(task, agent)}
            {@const unchanged = tone(agent) === "done" && !!preparedTasks && !preparedTasks.has(task.task_id)}
            <article class="worker" data-tone={tone(agent)} data-unchanged={unchanged || undefined}>
              <button class="head" onclick={() => agent && onInspect(agent)} disabled={!agent} aria-label={`${vendor}: ${taskDescriptions[task.task_id] ?? task.task_id}. ${state?.label ?? "Not started"}. Open output.`}>
                <span class="logo">{#if cli}<AdeIdentity id={cli} />{/if}</span>
                <span class="who"><strong title={taskDescriptions[task.task_id]}>{taskDescriptions[task.task_id] ?? task.task_id}</strong><small>{vendor}</small></span>
                <span class="state">{#if unchanged}No changes{:else if tone(agent) === "done"}{task.verify.length ? "✓ Passed" : "Completed"}{:else}{tone(agent) === "live" ? "● Working" : tone(agent) === "queued" ? (share ? "Next" : "Queued") : state?.label ?? "Settled"}{/if}</span>
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
  .fleet { container: fleet / inline-size; position: absolute; inset: 0; display: flex; flex-direction: column; gap: 18px; padding: 20px 24px; overflow: auto; }
  .lanes { display: grid; flex: none; gap: 14px; }
  /* One lane per step: label on the left, equal cards on a shared grid. */
  .lane { position: relative; display: grid; grid-template-columns: 112px minmax(0, 1fr); gap: 16px; }
  .lane + .lane::before { content: ""; position: absolute; left: 12px; top: -14px; height: 14px; border-left: 1px dashed var(--pytxo-line); }
  .lane h3 { display: grid; align-content: start; gap: 3px; margin: 10px 0 0; color: var(--pytxo-text-strong); font: 600 13px var(--pytxo-font-ui); }
  .lane h3 small { color: var(--pytxo-text-muted); font: 11.5px var(--pytxo-font-mono, "IBM Plex Mono", monospace); }
  .cards { display: grid; grid-template-columns: repeat(var(--cols), minmax(0, 1fr)); gap: 12px; }
  .worker { container: worker / inline-size; display: grid; grid-template-rows: auto 1fr auto; height: 214px; overflow: hidden; border: 1px solid var(--pytxo-line); border-radius: 10px; background: var(--pytxo-surface-panel); }
  /* An active worker carries an activity rail: it shows activity, never progress. */
  .worker[data-tone="live"] { position: relative; border-color: color-mix(in srgb, var(--pytxo-activity) 45%, var(--pytxo-line)); }
  .worker[data-tone="live"]::before { content: ""; position: absolute; inset: 0 0 auto 0; height: 2px; background: var(--pytxo-aperture-horizontal); background-size: 200% 100%; animation: activity-rail 2.4s linear infinite; pointer-events: none; }
  @keyframes activity-rail { to { background-position: -200% 0; } }
  @media (prefers-reduced-motion: reduce) { .worker[data-tone="live"]::before { animation: none; } }
  :global([data-force-reduced-motion]) .worker[data-tone="live"]::before { animation: none; }
  .worker[data-tone="done"] { border-color: color-mix(in srgb, var(--state-verified) 35%, var(--pytxo-line)); }
  .worker[data-tone="failed"] { border-color: color-mix(in srgb, var(--state-refuted) 50%, var(--pytxo-line)); }
  .worker[data-tone="queued"] { border-style: dashed; }
  .worker[data-tone="queued"] .term { opacity: .75; }
  .head { display: grid; grid-template-columns: 26px minmax(0, 1fr) auto; gap: 12px; align-items: center; padding: 12px 14px; border: 0; border-radius: 0; background: transparent; text-align: left; }
  .head:hover:not(:disabled) { background: var(--pytxo-surface-hover); }
  .logo { display: grid; place-items: center; width: 26px; height: 26px; border-radius: 7px; background: var(--pytxo-surface-raised); }
  .who { display: grid; gap: 2px; min-width: 0; }
  /* Task titles are the request's own words: two lines before they clip. */
  .who strong { display: -webkit-box; overflow: hidden; color: var(--pytxo-text-strong); font-size: 14px; font-weight: 600; line-height: 1.3; -webkit-line-clamp: 2; -webkit-box-orient: vertical; }
  .who small { overflow: hidden; color: var(--pytxo-text-muted); font: 11.5px var(--pytxo-font-mono, "IBM Plex Mono", monospace); text-overflow: ellipsis; white-space: nowrap; }
  .state { align-self: start; margin-top: 2px; color: var(--pytxo-text-muted); font: 500 11.5px var(--pytxo-font-mono, "IBM Plex Mono", monospace); white-space: nowrap; }
  .worker[data-tone="live"] .state { color: var(--pytxo-activity); }
  .worker[data-tone="done"] .state { color: var(--state-verified); }
  .worker[data-unchanged] .state { color: var(--pytxo-text-muted); }
  .worker[data-tone="failed"] .state { color: var(--state-refuted); }
  .term { display: flex; flex-direction: column; justify-content: flex-end; min-height: 0; overflow: hidden; padding: 8px 14px; border-top: 1px solid var(--pytxo-line-soft); background: var(--pytxo-code-surface); color: var(--pytxo-text-body); font: 12px/1.6 var(--pytxo-font-mono, "IBM Plex Mono", monospace); }
  .term div { flex: none; overflow: hidden; text-overflow: ellipsis; white-space: nowrap; }
  /* Newest lines sit at the bottom; older ones fade out at the top instead of being cut mid-line. */
  .term { -webkit-mask-image: linear-gradient(to bottom, transparent 0, #000 22px); mask-image: linear-gradient(to bottom, transparent 0, #000 22px); }
  .term .mark { color: var(--state-verified); }
  .term .failed { color: var(--state-refuted); }
  .term .hint { color: var(--pytxo-text-muted); white-space: normal; }
  .output-error { margin: 0; padding: 6px 14px; color: var(--pytxo-text-soft); font-size: 12px; }
  .owns { display: flex; gap: 6px; overflow: hidden; padding: 9px 14px; border-top: 1px solid var(--pytxo-line-soft); }
  .owns span { flex: none; max-width: 100%; overflow: hidden; text-overflow: ellipsis; white-space: nowrap; padding: 2px 7px; border-radius: 5px; background: var(--pytxo-surface-raised); color: var(--pytxo-text-body); font: 11.5px var(--pytxo-font-mono, "IBM Plex Mono", monospace); }
  .owns span.shared { box-shadow: inset 0 0 0 1px color-mix(in srgb, var(--pytxo-glyph-warm, #f3c64e) 55%, transparent); color: var(--pytxo-glyph-warm, #f3c64e); }
  .activity { flex: none; padding: 0 0 12px; border-top: 1px solid var(--pytxo-line-soft); }
  .activity summary { display: flex; justify-content: space-between; align-items: center; gap: 16px; min-height: 40px; color: var(--pytxo-text-strong); font-size: 13px; font-weight: 500; cursor: pointer; }
  .activity summary::before { content: "+"; color: var(--pytxo-text-muted); }
  .activity[open] summary::before { content: "\2212"; }
  .activity summary small { margin-left: auto; color: var(--pytxo-text-muted); font-weight: 400; font-size: 12px; }
  .row { display: grid; grid-template-columns: 130px minmax(0, 1fr); align-items: center; gap: 12px; height: 22px; color: var(--pytxo-text-body); font-size: 13.5px; }
  .track { position: relative; height: 12px; border-radius: 6px; background: color-mix(in srgb, var(--pytxo-surface-raised) 80%, transparent); }
  .track i { position: absolute; top: 0; bottom: 0; border-radius: 6px; background: var(--pytxo-text-muted); }
  .track i[data-tone="live"] { background: linear-gradient(90deg, color-mix(in srgb, var(--pytxo-activity) 35%, transparent), var(--pytxo-activity)); }
  .track i[data-tone="done"] { background: var(--state-verified); }
  .track i[data-tone="failed"] { background: var(--state-refuted); }
  .facts { display: flex; flex-wrap: wrap; gap: 8px 26px; margin: 0; padding-left: 128px; color: var(--pytxo-text-soft); font-size: 13px; }
  .facts b { margin-right: 5px; color: var(--pytxo-text-strong); font-size: 15px; }
  @container fleet (max-width: 1100px) { .cards { grid-template-columns: repeat(min(var(--cols), 2), minmax(0, 1fr)); } }
  @container fleet (max-width: 640px) { .lane { grid-template-columns: minmax(0, 1fr); gap: 8px; } .lane h3 { display: flex; gap: 10px; margin: 0; } .lane + .lane::before { display: none; } .cards { grid-template-columns: minmax(0, 1fr); } .facts { padding-left: 0; } .row { grid-template-columns: 110px minmax(0, 1fr); } }
</style>
