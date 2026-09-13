<script lang="ts">
  import { untrack } from "svelte";
  import type { DesktopBackend, DesktopSnapshot } from "../../lib/desktop-backend";
  import type { DockView } from "../../lib/dock-layout";
  import type { EventDto, RunReviewDto } from "../../lib/types";
  import AgentInspector from "./AgentInspector.svelte";
  import BoundaryPanel from "./BoundaryPanel.svelte";
  import PreparedDockFile from "./PreparedDockFile.svelte";
  import StateChip from "./StateChip.svelte";
  import AdeIdentity from "./AdeIdentity.svelte";
  import { agentState } from "../../lib/epistemic";
  import { createRecordedOutputReader } from "../../lib/recorded-output";
  let { view, backend, snapshot, visible, onReview, onOpenApprovals }: {
    view: DockView; backend: DesktopBackend; snapshot: DesktopSnapshot; visible: boolean;
    onReview: (runId: string, domainId: string) => void; onOpenApprovals: () => void;
  } = $props();
  let review = $state<RunReviewDto | null>(null);
  let error = $state("");
  let loading = $state(true);
  let events = $state<(EventDto & { readable: string })[]>([]);
  const readOutput = createRecordedOutputReader();
  let rawOutput = $state(false);
  let cursor = $state(0);
  let trimmed = $state(false);
  let mode = $state<"output" | "activity" | "task">("output");
  let following = $state(true);
  let output: HTMLDivElement | undefined = $state();
  let selectedFile = $state<string | null>(null);
  const run = $derived(snapshot.runs.find(r => r.domain_id === view.domainId && r.id === view.runId) ?? null);
  const agent = $derived(snapshot.agents.find(a => a.domain_id === view.domainId && a.run_id === view.runId && a.id === view.agentId) ?? null);
  const task = $derived(review?.plan.waves.flat().find(t => t.task_id === agent?.task_id) ?? null);
  const recordedState = $derived(agent ? agentState(agent) : null);
  const activity = $derived(events.filter(event => event.kind !== "stdout" && event.kind !== "stderr"));
  const workerOutput = $derived(events.filter(event => event.kind === "stdout" || event.kind === "stderr"));
  function onOutputScroll() {
    if (output && output.scrollHeight - output.clientHeight - output.scrollTop > 24) following = false;
  }
  function toggleFollowing() {
    following = !following;
    if (following && output) output.scrollTop = output.scrollHeight;
  }
  // Never share poll_log_lines' backend cursor with another consumer.
  $effect(() => {
    if (!visible) return;
    let disposed = false;
    let timer: ReturnType<typeof setTimeout>;
    let after = untrack(() => cursor);
    async function poll() {
      try {
        if (view.kind === "agent" && view.agentId) {
          const page = await backend.readAgentEvents(view.runId, view.agentId, view.domainId, after, 200);
          if (disposed) return;
          if (page.length) {
            after = page[page.length - 1].id;
            cursor = after;
            const fresh = page.filter(e => !events.some(old => old.id === e.id))
              .map(e => ({ ...e, readable: readOutput(e.kind, e.payload) }));
            const merged = [...events, ...fresh];
            trimmed ||= merged.length > 600;
            events = merged.slice(-600);
          }
        }
        const next = await backend.runReview(view.runId, view.domainId);
        if (disposed) return;
        if (next.run_id !== view.runId) throw new Error("Review response belongs to another run.");
        review = next;
        error = "";
      } catch (cause) {
        if (!disposed) error = cause instanceof Error ? cause.message : String(cause);
      } finally {
        if (!disposed) { loading = false; timer = setTimeout(poll, 2000); }
      }
    }
    void poll();
    return () => { disposed = true; clearTimeout(timer); };
  });
  $effect(() => { if (following && output && events.length) queueMicrotask(() => { if (output) output.scrollTop = output.scrollHeight; }); });
</script>

<section class="inspection" class:agent-output={view.kind === "agent" && mode === "output"} aria-label={`${view.title} inspection`}>
  <header>
    {#if view.kind === "agent" && agent?.launcher}<AdeIdentity id={agent.launcher.id} />{/if}
    <strong>{view.kind === "agent" ? (agent?.task_id ?? view.title) : view.kind === "files" ? "Prepared changes" : view.kind === "diagram" ? "Task dependencies" : "Run evidence"}</strong>
    {#if view.kind === "agent" && agent?.launcher}<small aria-label="Recorded launcher">{agent.launcher.display_name}</small>{/if}
    {#if view.kind === "agent" && recordedState}<StateChip tone={recordedState.tone} label={recordedState.label} />{/if}
    <details class="source"><summary>Source details</summary><div>
      <span>{view.domainId}</span><small>Run {view.runId}</small>
      {#if view.agentId}<small>Agent {view.agentId}</small><small>Recorded agent output contains claims. Independent checks appear in Checks & details.</small>{/if}
      {#if view.kind === "agent"}
        <small>Recorded launcher: {agent?.launcher?.display_name ?? "Not identified"}. This identifies the saved launch command, not a current authenticated session.</small>
        <small>Recorded workspace: {agent?.workspace_path ?? "Not recorded"}</small>
        <small>The saved workspace can be a review source. Its path does not establish the current process folder or that the folder still exists.</small>
        <small>Plain text removes terminal formatting; cursor redraws are not replayed. Stored event text is unchanged.</small>
        <button aria-pressed={rawOutput} onclick={() => rawOutput = !rawOutput}>{rawOutput ? "Show plain text" : "Show raw event text"}</button>
      {/if}
    </div></details>
  </header>
  {#if !run}<p class="notice">Outside the current snapshot. This view retains its original scope.</p>{/if}
  {#if error}<p class="error" role="alert">{error} Displayed records may be outdated.</p>{/if}
  {#if loading}<p role="status">Loading recorded evidence…</p>{/if}
  {#if view.kind === "agent"}
    <div class="view-tools"><button class:active={mode === "output"} aria-pressed={mode === "output"} onclick={() => mode = "output"}>Output</button><button class:active={mode === "activity"} aria-pressed={mode === "activity"} onclick={() => mode = "activity"}>Activity</button><button class:active={mode === "task"} aria-pressed={mode === "task"} onclick={() => mode = "task"}>Task & receipt</button><span>{mode === "output" ? `${rawOutput ? "Raw events" : "Plain text"} · read only · claims` : "Recorded run data · read only"}</span></div>
    {#if mode === "output"}
      {#if trimmed}<p class="notice history-limit">Latest 600 loaded events · earlier output remains on disk.</p>{/if}
      <!-- Scroll regions need a tab stop so keyboard users can read long output. -->
      <!-- svelte-ignore a11y_no_noninteractive_tabindex -->
      <div class="output" bind:this={output} onscroll={onOutputScroll} tabindex="0" role="region" aria-label="Recorded agent output">
        {#each workerOutput as event (event.id)}{#if rawOutput || event.readable.trim()}<div class="event"><small>{event.kind}</small><pre>{rawOutput ? event.payload : event.readable}</pre></div>{/if}{/each}
        {#if !workerOutput.length && !loading && !error}<p>No recorded worker output yet.</p>{/if}
        {#if workerOutput.length && !rawOutput && !workerOutput.some(event => event.readable.trim())}<p>No plain text in these events. Raw event text is available in Source details.</p>{/if}
      </div>
      <div class="view-tools"><span>Headless job · input unavailable</span><button aria-pressed={following} onclick={toggleFollowing}>{following ? "Pause following" : "Follow output"}</button></div>
    {:else if mode === "activity"}
      <div class="activity" aria-label="Recorded agent activity">
        <p class="notice">Runner and tool events for this agent. Worker stdout and stderr stay in Output. These records do not certify the prepared result.</p>
        {#if trimmed}<p class="notice">Latest 600 loaded events only. Earlier activity remains on disk.</p>{/if}
        {#each activity as event (event.id)}
          <article><div><strong>{event.kind}</strong><time datetime={event.ts}>{event.ts}</time></div><pre>{event.readable || "No plain-text detail recorded."}</pre></article>
        {:else}<p class="notice">No runner or tool activity recorded in the loaded events.</p>{/each}
      </div>
    {:else if agent}
      <AgentInspector {agent} {task} receipt={review?.enforcement.agents[agent.id] ?? review?.enforcement.agents[agent.id.replace(`${view.runId}:`, "")] ?? null} />
    {:else}<p>Worker details are not present in this snapshot.</p>{/if}
  {:else if view.kind === "evidence"}
    {#if review?.prepared_manifest?.candidate_verification}
      <section class="checks" aria-label="Combined candidate checks"><h3>Combined candidate checks</h3>
        {#each review.prepared_manifest.candidate_verification.checks as check}<div><strong class:failed={!check.passed}>{check.passed ? "Passed" : "Failed"}</strong><code>{check.command}</code><small>{check.task_id} · {check.effective_profile}</small></div>{/each}
        {#if !review.prepared_manifest.candidate_verification.checks.length}<p>No combined checks recorded.</p>{/if}
      </section>
    {:else}<p class="notice">Combined candidate verification has not been recorded.</p>{/if}
    <BoundaryPanel {run} {review} reviewError={error || null} {loading} approvals={snapshot.approvals.filter(a => a.domain_id === view.domainId)} showReviewAction={false} {onOpenApprovals} onReview={() => onReview(view.runId, view.domainId)} />
  {:else if view.kind === "files"}
    <p class="notice">Frozen prepared files. Inspect the complete contents in Review before Apply.</p>
    {#each review?.prepared_manifest?.files ?? [] as file (file.path)}<button class="file" aria-pressed={selectedFile === file.path} onclick={() => selectedFile = file.path}><span>{file.path}</span><small>{file.kind} · {file.byte_count} bytes</small></button>{/each}
    {#if review?.prepared_manifest && selectedFile}
      {@const file = review.prepared_manifest.files.find(f => f.path === selectedFile)}
      {#if file}{#key `${review.prepared_manifest.package_digest}:${file.path}`}<PreparedDockFile {backend} {file} domainId={view.domainId} runId={view.runId} packageDigest={review.prepared_manifest.package_digest} />{/key}{/if}
    {/if}
    {#if !review?.prepared_manifest}<p>No prepared package available.</p>{/if}
    {#if review?.prepared_manifest}<button class="open-review" onclick={() => onReview(view.runId, view.domainId)}>Open prepared review</button>{/if}
  {:else}
    <p class="notice">Dependencies from the saved plan. This describes scheduling, not verified completion.</p>
    {#each review?.plan.waves ?? [] as wave, index}<section class="wave"><h3>Wave {index + 1}</h3>{#each wave as task}<div><strong>{task.task_id}</strong><small>{task.depends_on.length ? `After ${task.depends_on.join(", ")}` : "No task dependencies"}</small><code>{task.paths.join(", ") || "No paths declared"}</code></div>{/each}</section>{/each}
  {/if}
</section>

<style>
  .inspection{display:flex;min-width:0;flex-direction:column;background:var(--pytxo-surface-panel);font-size:13px}
  .agent-output{flex:1;min-height:0}.agent-output>*{flex-shrink:0}
  header{display:flex;flex-wrap:wrap;align-items:center;gap:8px;padding:8px 12px;border-bottom:1px solid var(--pytxo-line-soft)}
  header strong{flex:1;min-width:0}.source{margin-left:auto}.source[open]{flex-basis:100%;margin-left:0}.source summary{cursor:pointer;font-size:11px;color:var(--pytxo-text-muted)}.source summary:focus-visible{outline:2px solid var(--pytxo-accent);outline-offset:3px}.source div{display:flex;flex-direction:column;gap:6px;padding-block:8px}
  header strong{font-size:14px;overflow-wrap:anywhere}header span,header small{font:11px "IBM Plex Mono",monospace;overflow-wrap:anywhere;color:var(--pytxo-text-soft)}
  p{margin:12px 18px;line-height:1.6}.notice{font-size:12px;color:var(--pytxo-text-muted)}.error,.failed{color:var(--state-refuted)}
  .view-tools{display:flex;flex-wrap:wrap;align-items:center;gap:8px;padding:8px 12px;border-block:1px solid var(--pytxo-line-soft);font-size:11px;color:var(--pytxo-text-muted)}.view-tools span{margin-left:auto}
  button{min-height:30px;padding:5px 10px;border:1px solid var(--pytxo-line);border-radius:4px;background:transparent;color:var(--pytxo-text-soft);cursor:pointer;font:inherit}button:hover,button.active{background:var(--pytxo-surface-active);color:var(--pytxo-text-strong)}button:focus-visible,.output:focus-visible{outline:2px solid var(--pytxo-accent);outline-offset:-2px}
  .output{flex:1 1 80px;min-height:64px;overflow:auto;padding:8px 12px}.history-limit{margin:4px 12px;font-size:11px}.event{display:grid;grid-template-columns:68px minmax(0,1fr);gap:10px;padding:5px 0}.event small{overflow-wrap:anywhere;color:var(--pytxo-text-muted);font-size:10px}.event pre{margin:0;white-space:pre-wrap;overflow-wrap:anywhere;color:var(--pytxo-text-soft);font:12px/1.65 "IBM Plex Mono",monospace}
  .checks,.wave{padding:14px 18px}.checks h3,.wave h3{margin:0 0 12px;font-size:13px}.checks div,.wave div{display:flex;flex-direction:column;gap:6px;padding:10px 0;border-bottom:1px solid var(--pytxo-line-soft)}code{white-space:pre-wrap;overflow-wrap:anywhere;font:12px "IBM Plex Mono",monospace}small{color:var(--pytxo-text-muted)}.file{display:flex;flex-direction:column;gap:6px;align-items:flex-start;margin:4px 12px;text-align:left;overflow-wrap:anywhere}.open-review{margin:12px}
  .activity article{padding:12px;border-bottom:1px solid var(--pytxo-line-soft)}
  .activity article div{display:flex;flex-wrap:wrap;justify-content:space-between;gap:6px}
  .activity strong{font-size:12px;overflow-wrap:anywhere}.activity time{font-size:11px;color:var(--pytxo-text-muted);font-variant-numeric:tabular-nums}
  .activity pre{margin:8px 0 0;white-space:pre-wrap;overflow-wrap:anywhere;font:12px/1.6 "IBM Plex Mono",monospace;color:var(--pytxo-text-soft)}
  .inspection :global(.boundary){border:0;border-radius:0}.inspection :global(.head strong),.inspection :global(dd),.inspection :global(.surface small){white-space:normal;overflow-wrap:anywhere}
</style>
