<script lang="ts">
  import { untrack } from "svelte";
  import type { DesktopBackend, DesktopSnapshot } from "../../lib/desktop-backend";
  import type { DockReference, DockView } from "../../lib/dock-layout";
  import type { EventDto, RunReviewDto } from "../../lib/types";
  import BoundaryPanel from "./BoundaryPanel.svelte";
  import PreparedDockFile from "./PreparedDockFile.svelte";
  import StateChip from "./StateChip.svelte";
  import AdeIdentity from "./AdeIdentity.svelte";
  import { agentState } from "../../lib/epistemic";
  import { createRecordedOutputChunkReader, groupRecordedOutput, type RecordedOutputChunk } from "../../lib/recorded-output";
  import { readReviewRevision, reviewRevisionKey } from "../../lib/review-detail-cache";

  let { view, backend, snapshot, visible, onReview, onOpenApprovals, onOpenOutput = () => {}, contextual = false, taskDescription }: {
    taskDescription?: string; contextual?: boolean; view: DockView; backend: DesktopBackend; snapshot: DesktopSnapshot; visible: boolean;
    onReview: (runId: string, domainId: string) => void; onOpenApprovals: () => void; onOpenOutput?: (ref: DockReference) => void;
  } = $props();

  let review = $state<RunReviewDto | null>(null);
  let error = $state("");
  let loading = $state(true);
  let events = $state<(EventDto & RecordedOutputChunk)[]>([]);
  let rawOutput = $state(false);
  let cursor = $state(0);
  let trimmed = $state(false);
  let mode = $state<"output" | "activity">("output");
  let following = $state(true);
  let renderCount = $state(120);
  let output: HTMLDivElement | undefined = $state();
  let selectedFile = $state<string | null>(null);
  let followFrame = 0;
  let eventIds = new Set<number>();
  let loadedReviewKey = "";
  const readOutput = createRecordedOutputChunkReader();

  const run = $derived(snapshot.runs.find(candidate => candidate.domain_id === view.domainId && candidate.id === view.runId) ?? null);
  const agent = $derived(snapshot.agents.find(candidate => candidate.domain_id === view.domainId && candidate.run_id === view.runId && candidate.id === view.agentId) ?? null);
  const task = $derived(review?.plan.waves.flat().find(candidate => candidate.task_id === agent?.task_id) ?? null);
  const recordedState = $derived(agent ? agentState(agent) : null);
  const receipt = $derived(agent ? review?.enforcement.agents[agent.id] ?? review?.enforcement.agents[agent.id.replace(`${view.runId}:`, "")] ?? null : null);
  const preparedFiles = $derived(review?.prepared_manifest?.files.filter(file => file.task_id === agent?.task_id) ?? []);
  const revisionKey = $derived(run ? reviewRevisionKey(run) : `${view.domainId}\u0000${view.runId}\u0000missing`);
  const visibleEvents = $derived(events.slice(-renderCount));
  const activity = $derived(visibleEvents.filter(event => event.kind !== "stdout" && event.kind !== "stderr"));
  const workerOutput = $derived(visibleEvents.filter(event => event.kind === "stdout" || event.kind === "stderr"));
  const outputRows = $derived(groupRecordedOutput(workerOutput));
  const hasEarlier = $derived(events.length > renderCount || trimmed);

  function onOutputScroll() {
    if (output && output.scrollHeight - output.clientHeight - output.scrollTop > 24) following = false;
  }
  function scheduleFollow() {
    if (!following || !output || followFrame) return;
    followFrame = requestAnimationFrame(() => { followFrame = 0; if (following && output) output.scrollTop = output.scrollHeight; });
  }
  function toggleFollowing() { following = !following; if (following) scheduleFollow(); }
  function loadEarlier() { renderCount = Math.min(600, renderCount + 120); }

  // Review detail is revision keyed. Snapshot refreshes with the same candidate
  // do not create another request, and raw output never asks for review data.
  $effect(() => {
    const reviewKey = revisionKey;
    const kind = view.kind;
    if (!visible || kind === "agent-output" || kind === "terminal" || kind === "preview") return;
    if (loadedReviewKey === reviewKey) return;
    let disposed = false;
    loading = true;
    readReviewRevision(backend, reviewKey, view.runId, view.domainId).then(next => {
      if (disposed) return;
      if (next.run_id !== view.runId) throw new Error("Review response belongs to another run.");
      review = next;
      loadedReviewKey = reviewKey;
      error = "";
    }).catch((cause: unknown) => {
      if (!disposed) error = cause instanceof Error ? cause.message : String(cause);
    }).finally(() => { if (!disposed) loading = false; });
    return () => { disposed = true; };
  });

  // Every output view owns its cursor. Polling and DOM rows exist only while
  // the bottom output dock is actually visible.
  $effect(() => {
    if (!visible || view.kind !== "agent-output" || !view.agentId) return;
    let disposed = false;
    let timer: ReturnType<typeof setTimeout>;
    let after = untrack(() => cursor);
    loading = true;
    async function poll() {
      try {
        const page = await backend.readAgentEvents(view.runId, view.agentId!, view.domainId, after, 200);
        if (disposed) return;
        if (page.length) {
          after = page[page.length - 1].id;
          cursor = after;
          const fresh = page.filter(event => !eventIds.has(event.id)).map(event => ({ ...event, ...readOutput(event.kind, event.payload) }));
          for (const event of fresh) eventIds.add(event.id);
          const merged = [...events, ...fresh];
          if (merged.length > 600) {
            trimmed = true;
            events = merged.slice(-600);
            eventIds = new Set(events.map(event => event.id));
          } else events = merged;
          scheduleFollow();
        }
        error = "";
      } catch (cause) {
        if (!disposed) error = cause instanceof Error ? cause.message : String(cause);
      } finally {
        if (!disposed) { loading = false; timer = setTimeout(poll, 2000); }
      }
    }
    void poll();
    return () => { disposed = true; clearTimeout(timer); cancelAnimationFrame(followFrame); followFrame = 0; };
  });
</script>

<section class="inspection" class:contextual class:agent-output={view.kind === "agent-output"} aria-label={`${view.title} inspection`}>
  <header class="inspection-head">
    {#if (view.kind === "agent" || view.kind === "agent-output") && agent?.launcher}<AdeIdentity id={agent.launcher.id} />{/if}
    <div>
      <strong class="inspection-title">{view.kind === "agent" ? (taskDescription ?? view.title ?? agent?.task_id) : view.kind === "agent-output" ? (agent?.task_id ?? view.title) : view.kind === "files" ? "Prepared changes" : view.kind === "diagram" ? "Task dependencies" : "Run evidence"}</strong>
      {#if (view.kind === "agent" || view.kind === "agent-output") && agent?.launcher}<small aria-label="Recorded launcher">{agent.launcher.display_name}</small>{/if}
    </div>
    {#if (view.kind === "agent" || view.kind === "agent-output") && recordedState}<StateChip tone={recordedState.tone} label={recordedState.label} />{/if}
    <details class="source"><summary>Source</summary><div><span>{view.domainId}</span><small>Run {view.runId}</small>{#if view.agentId}<small>Agent {view.agentId}</small>{/if}{#if agent}<small>Recorded workspace: {agent.workspace_path ?? "Not recorded"}</small>{/if}</div></details>
  </header>

  {#if !run}<p class="notice">Outside the current snapshot. This view retains its original scope.</p>{/if}
  {#if error}<p class="error" role="alert">{error} Displayed records may be outdated.</p>{/if}

  {#if view.kind === "agent"}
    <div class="summary-body">
      {#if loading}<p role="status">Loading recorded worker summary…</p>{/if}
      <details class="worker-request"><summary>Full worker request</summary><p>{taskDescription ?? view.title ?? agent?.task_id}</p></details>
      <section><h3>Status</h3><p>{recordedState?.detail ?? "Worker state is not present in this snapshot."}</p></section>
      <section><h3>Recorded scope</h3><p class="notice">Planned paths are claims from the saved plan.</p>{#each task?.paths ?? [] as path}<code>{path}</code>{:else}<p>No planned paths recorded.</p>{/each}</section>
      <section><h3>Prepared files</h3>{#each preparedFiles as file}<code>{file.path}</code>{:else}<p>No prepared contribution is recorded for this worker.</p>{/each}</section>
      <section><h3>Evidence</h3><dl><div><dt>Workspace isolation</dt><dd>{receipt?.workspace_isolation.status ?? "unknown"}</dd></div><div><dt>Apply boundary</dt><dd>{receipt?.apply_boundary.status ?? "unknown"}</dd></div><div><dt>Combined checks</dt><dd>{review?.prepared_manifest?.candidate_verification?.checks.length ? `${review.prepared_manifest.candidate_verification.checks.filter(check => check.passed).length}/${review.prepared_manifest.candidate_verification.checks.length} passed` : "not recorded"}</dd></div></dl></section>
    </div>
    <footer class="summary-actions"><button class="primary-action" onclick={() => onOpenOutput({ kind: "agent-output", domainId: view.domainId, runId: view.runId, agentId: view.agentId, title: `Output · ${agent?.task_id ?? view.title}` })}>Open output</button><button onclick={() => onReview(view.runId, view.domainId)} disabled={!review?.prepared_manifest}>Open Review</button></footer>
  {:else if view.kind === "agent-output"}
    <div class="output-tools"><div role="group" aria-label="Output view"><button class:active={mode === "output"} aria-pressed={mode === "output"} onclick={() => mode = "output"}>Output</button><button class:active={mode === "activity"} aria-pressed={mode === "activity"} onclick={() => mode = "activity"}>Events</button></div><span>{rawOutput ? "Raw events" : "Plain text"} · read only · claims</span><button aria-pressed={rawOutput} onclick={() => rawOutput = !rawOutput}>{rawOutput ? "Plain text" : "Raw text"}</button></div>
    {#if hasEarlier}<button class="load-earlier" onclick={loadEarlier} disabled={renderCount >= 600 && trimmed}>Load 120 earlier records</button>{/if}
    {#if mode === "output"}
      <!-- svelte-ignore a11y_no_noninteractive_tabindex -->
      <div class="output" bind:this={output} onscroll={onOutputScroll} tabindex="0" role="region" aria-label="Recorded agent output">
        {#if rawOutput}
          {#each workerOutput as event (event.id)}<div class="event"><small>{event.kind}</small><pre>{event.payload}</pre></div>{/each}
        {:else}
          {#each outputRows as row (row.id)}{#if row.readable.trim()}<div class="event"><small>{row.kind}</small><pre>{row.readable}</pre></div>{/if}{/each}
        {/if}
        {#if !workerOutput.length && !loading && !error}<p>No recorded worker output yet.</p>{/if}
      </div>
      <footer class="follow-tools"><span>Latest {Math.min(events.length, renderCount)} of {events.length}{trimmed ? "+" : ""} loaded records</span><button aria-pressed={following} onclick={toggleFollowing}>{following ? "Pause following" : "Follow output"}</button></footer>
    {:else}
      <div class="activity" aria-label="Recorded agent activity"><p class="notice">Runner and tool events. These records do not certify the prepared result.</p>{#each activity as event (event.id)}<article><div><strong>{event.kind}</strong><time datetime={event.ts}>{event.ts}</time></div><pre>{event.readable || "No plain-text detail recorded."}</pre></article>{:else}<p class="notice">No runner or tool activity in the loaded records.</p>{/each}</div>
    {/if}
  {:else if view.kind === "evidence"}
    {#if loading}<p role="status">Loading recorded evidence…</p>{/if}
    {#if review?.prepared_manifest?.candidate_verification}<section class="checks" aria-label="Combined candidate checks"><h3>Combined candidate checks</h3>{#each review.prepared_manifest.candidate_verification.checks as check}<div><strong class:failed={!check.passed}>{check.passed ? "Passed" : "Failed"}</strong><code>{check.command}</code><small>{check.task_id} · {check.effective_profile}</small></div>{/each}{#if !review.prepared_manifest.candidate_verification.checks.length}<p>No combined checks recorded.</p>{/if}</section>{:else if !loading}<p class="notice">Combined candidate verification has not been recorded.</p>{/if}
    <BoundaryPanel {run} {review} reviewError={error || null} {loading} approvals={snapshot.approvals.filter(approval => approval.domain_id === view.domainId)} showReviewAction={false} {onOpenApprovals} onReview={() => onReview(view.runId, view.domainId)} />
  {:else if view.kind === "files"}
    {#if loading}<p role="status">Loading prepared files…</p>{/if}<p class="notice">Frozen prepared files. Inspect the complete contents in Review before Apply.</p>{#each review?.prepared_manifest?.files ?? [] as file (file.path)}<button class="file" aria-pressed={selectedFile === file.path} onclick={() => selectedFile = file.path}><span>{file.path}</span><small>{file.kind} · {file.byte_count} bytes</small></button>{/each}{#if review?.prepared_manifest && selectedFile}{@const file = review.prepared_manifest.files.find(candidate => candidate.path === selectedFile)}{#if file}{#key `${review.prepared_manifest.package_digest}:${file.path}`}<PreparedDockFile {backend} {file} domainId={view.domainId} runId={view.runId} packageDigest={review.prepared_manifest.package_digest} />{/key}{/if}{/if}{#if !review?.prepared_manifest && !loading}<p>No prepared package available.</p>{/if}{#if review?.prepared_manifest}<button class="open-review" onclick={() => onReview(view.runId, view.domainId)}>Open prepared review</button>{/if}
  {:else}
    {#if loading}<p role="status">Loading task dependencies…</p>{/if}<p class="notice">Dependencies from the saved plan. This describes scheduling, not verified completion.</p>{#each review?.plan.waves ?? [] as wave, index}<section class="wave"><h3>Wave {index + 1}</h3>{#each wave as waveTask}<div><strong>{waveTask.task_id}</strong><small>{waveTask.depends_on.length ? `After ${waveTask.depends_on.join(", ")}` : "No task dependencies"}</small><code>{waveTask.paths.join(", ") || "No paths declared"}</code></div>{/each}</section>{/each}
  {/if}
</section>

<style>
  .inspection-title{display:-webkit-box;overflow:hidden;-webkit-box-orient:vertical;-webkit-line-clamp:2;line-clamp:2}
  .worker-request{padding:10px 14px;border-bottom:1px solid var(--pytxo-line-soft)}
  .worker-request summary{cursor:pointer;color:var(--pytxo-text-soft);font-size:12px}
  .worker-request p{white-space:pre-wrap;overflow-wrap:anywhere}
  .inspection{display:flex;min-width:0;min-height:0;flex:1;flex-direction:column;background:var(--pytxo-surface-panel);font-size:13px}.inspection-head{display:flex;flex:0 0 auto;flex-wrap:wrap;align-items:center;gap:9px;padding:11px 13px;border-bottom:1px solid var(--pytxo-line-soft)}.inspection-head>div{display:grid;min-width:0;flex:1;gap:3px}.inspection-head strong{overflow-wrap:anywhere;font-size:14px}.inspection-head small,.inspection-head span{overflow-wrap:anywhere;color:var(--pytxo-text-muted);font:10px "IBM Plex Mono",monospace}.source{position:relative;margin-left:auto}.source[open]{flex-basis:100%;margin-left:0}.source summary{cursor:pointer;color:var(--pytxo-text-muted);font-size:11px}.source div{display:flex;flex-direction:column;gap:5px;padding-block:8px}
  p{margin:10px 14px;line-height:1.55}.notice{color:var(--pytxo-text-muted);font-size:11px}.error,.failed{color:var(--state-refuted)}button{min-height:30px;padding:5px 9px;border:1px solid var(--pytxo-line);border-radius:4px;background:transparent;color:var(--pytxo-text-soft);font:inherit;cursor:pointer}button:hover,button.active{background:var(--pytxo-surface-active);color:var(--pytxo-text-strong)}button:disabled{cursor:not-allowed;opacity:.45}button:focus-visible,.output:focus-visible{outline:2px solid var(--pytxo-accent);outline-offset:-2px}
  .summary-body{min-height:0;flex:1;overflow:auto}.summary-body section{padding:13px 14px;border-bottom:1px solid var(--pytxo-line-soft)}.summary-body h3{margin:0 0 8px;font-size:11px;text-transform:uppercase;letter-spacing:.06em}.summary-body p{margin:6px 0;color:var(--pytxo-text-soft);font-size:12px}.summary-body code{display:block;margin:7px 0;overflow-wrap:anywhere;font:11px "IBM Plex Mono",monospace}.summary-body dl{display:grid;gap:8px;margin:0}.summary-body dl div{display:flex;justify-content:space-between;gap:10px}.summary-body dt{color:var(--pytxo-text-muted);font-size:11px}.summary-body dd{margin:0;color:var(--pytxo-text-soft);font-size:11px}.summary-actions{display:flex;flex:0 0 auto;gap:8px;padding:10px 12px;border-top:1px solid var(--pytxo-line)}.summary-actions button{flex:1}.summary-actions .primary-action{border-color:var(--pytxo-text-strong);background:var(--pytxo-text-strong);color:var(--pytxo-surface-shell);font-weight:650}
  .agent-output{overflow:hidden}.output-tools,.follow-tools{display:flex;flex:0 0 auto;align-items:center;gap:8px;padding:7px 10px;border-bottom:1px solid var(--pytxo-line-soft);color:var(--pytxo-text-muted);font-size:10px}.output-tools>div{display:flex;gap:3px}.output-tools>span{margin-left:auto}.output{min-height:64px;flex:1;overflow:auto;padding:8px 12px;background:var(--pytxo-code-surface)}.event{display:grid;grid-template-columns:58px minmax(0,1fr);gap:8px;padding:4px 0}.event small{color:var(--pytxo-text-muted);font-size:9px}.event pre{margin:0;white-space:pre-wrap;overflow-wrap:anywhere;color:var(--pytxo-text-body);font:11px/1.6 "IBM Plex Mono",monospace}.follow-tools{justify-content:space-between;border-top:1px solid var(--pytxo-line-soft);border-bottom:0}.load-earlier{align-self:center;margin:7px}.activity{min-height:0;overflow:auto}.activity article{padding:10px 12px;border-bottom:1px solid var(--pytxo-line-soft)}.activity article div{display:flex;justify-content:space-between;gap:8px}.activity strong{font-size:11px}.activity time{color:var(--pytxo-text-muted);font-size:10px}.activity pre{margin:7px 0 0;white-space:pre-wrap;overflow-wrap:anywhere;font:11px/1.55 "IBM Plex Mono",monospace}
  .checks,.wave{padding:13px 15px}.checks h3,.wave h3{margin:0 0 10px;font-size:12px}.checks div,.wave div{display:flex;flex-direction:column;gap:5px;padding:9px 0;border-bottom:1px solid var(--pytxo-line-soft)}code{white-space:pre-wrap;overflow-wrap:anywhere;font:11px "IBM Plex Mono",monospace}small{color:var(--pytxo-text-muted)}.file{display:flex;max-width:calc(100% - 24px);flex-direction:column;align-items:flex-start;gap:5px;margin:4px 12px;text-align:left;overflow-wrap:anywhere}.open-review{margin:12px}.inspection :global(.boundary){border:0;border-radius:0}.inspection :global(.head strong),.inspection :global(dd),.inspection :global(.surface small){white-space:normal;overflow-wrap:anywhere}
  .contextual{font-family:var(--pytxo-font-ui);background:transparent}
</style>
