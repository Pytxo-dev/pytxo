<script lang="ts">
  import IconAlertTriangle from "@tabler/icons-svelte/icons/alert-triangle";
  import IconSearch from "@tabler/icons-svelte/icons/search";
  import type { DesktopBackend, DesktopSnapshot } from "../../lib/desktop-backend";
  import { attemptTone, isPartiallyApplied, runState, worstTone, type EpistemicTone } from "../../lib/epistemic";
  import type { RunDto, RunReviewDto } from "../../lib/types";
  import BoundaryPanel from "./BoundaryPanel.svelte";
  import StateChip from "./StateChip.svelte";

  let {
    snapshot,
    backend,
    activeDomainId = null,
    focusRunId = null,
    onOpenRun,
    onSelectRun,
  }: {
    snapshot: DesktopSnapshot;
    backend: DesktopBackend;
    activeDomainId?: string | null;
    focusRunId?: string | null;
    onOpenRun: (runId: string) => void;
    onSelectRun: (runId: string) => void;
  } = $props();

  let query = $state("");
  let onlyUnresolved = $state(false);
  let loadedReview = $state<RunReviewDto | null>(null);
  let reviewError = $state<string | null>(null);
  let reviewLoading = $state(false);
  let rows: HTMLButtonElement[] = $state([]);

  const ordered = $derived(
    [...snapshot.runs].sort((a, b) => (b.started_at ?? "").localeCompare(a.started_at ?? "")),
  );

  const filtered = $derived(
    ordered
      .filter((run) => {
        const needle = query.trim().toLowerCase();
        if (!needle) return true;
        return run.id.toLowerCase().includes(needle) || run.repo_root.toLowerCase().includes(needle);
      })
      .filter((run) => !onlyUnresolved || needsAttention(run)),
  );

  const selected = $derived(filtered.find((run) => run.id === focusRunId) ?? filtered[0] ?? null);
  const review = $derived(loadedReview?.run_id === selected?.id ? loadedReview : null);

  /**
   * A run needs attention when its own record says an apply did not finish
   * cleanly, which is the only signal available without loading each receipt.
   */
  function needsAttention(run: RunDto): boolean {
    if (run.recovery_state && run.recovery_state !== "none") return true;
    if (run.last_apply_error) return true;
    return runState(run).tone === "refuted";
  }

  function applyLabel(run: RunDto): { tone: EpistemicTone; label: string } {
    if (run.applied_at) return { tone: "verified", label: "Applied" };
    if (run.apply_status === "review_failed") return { tone: "refuted", label: "Preparation failed" };
    if (run.last_apply_error) return { tone: "refuted", label: "Apply failed" };
    const status = (run.apply_status ?? "").toLowerCase();
    if (!status || status === "none") return { tone: "unknown", label: "Not attempted" };
    if (status === "ready" || status === "prepared" || status === "waiting") {
      return { tone: "claimed", label: "Prepared, not applied" };
    }
    return { tone: "unknown", label: run.apply_status ?? "Not reported" };
  }

  function workspaceLabel(run: RunDto) {
    return run.repo_root.split(/[\\/]/).pop() ?? run.repo_root;
  }

  function startedAt(run: RunDto) {
    const parsed = new Date(run.started_at);
    return Number.isNaN(parsed.getTime()) ? run.started_at : parsed.toLocaleString();
  }

  function domainIdForRun(run: RunDto) {
    return run.domain_id;
  }

  /** The receipt for the selected run, so History answers "what was enforced". */
  $effect(() => {
    const run = selected;
    if (!run) {
      loadedReview = null;
      reviewError = null;
      return;
    }
    let current = true;
    reviewLoading = true;
    reviewError = null;
    backend
      .runReview(run.id, domainIdForRun(run))
      .then((result) => {
        if (current) loadedReview = result;
      })
      .catch((error: unknown) => {
        if (!current) return;
        loadedReview = null;
        reviewError = error instanceof Error ? error.message : String(error);
      })
      .finally(() => {
        if (current) reviewLoading = false;
      });
    return () => {
      current = false;
    };
  });

  function onKeydown(event: KeyboardEvent, index: number) {
    if (event.defaultPrevented || event.metaKey || event.ctrlKey || event.altKey || event.shiftKey) return;
    const key = event.key.toLowerCase();
    const delta = key === "j" || key === "arrowdown" ? 1 : key === "k" || key === "arrowup" ? -1 : 0;
    if (!delta) return;
    event.preventDefault();
    const next = (index + delta + filtered.length) % filtered.length;
    rows[next]?.focus();
    onSelectRun(filtered[next].id);
  }
</script>

<section class="screen history">
  <header class="history-heading">
    <h1>History</h1>
    <div class="filters">
      <label>
        <IconSearch size={14} />
        <input bind:value={query} placeholder="Search run or workspace" aria-label="Search runs" />
      </label>
      <button class:active={onlyUnresolved} aria-pressed={onlyUnresolved} onclick={() => (onlyUnresolved = !onlyUnresolved)}>
        {onlyUnresolved ? "Unresolved only" : "All runs"}
      </button>
    </div>
  </header>

  {#if snapshot.error}
    <div class="panel offline-panel">
      <div class="empty">
        <IconAlertTriangle size={26} />
        <strong>Local service offline</strong>
        <span>{snapshot.error.message}</span>
      </div>
    </div>
  {:else}
    <div class="history-layout">
      <article class="table" aria-labelledby="history-table-title">
        <div class="table-head">
          <h2 id="history-table-title">Runs</h2>
          <span class="count">{filtered.length} of {ordered.length}</span>
        </div>
        <div class="columns" aria-hidden="true">
          <span>Run</span><span>Workspace</span><span>Outcome</span><span>Apply</span><span>Started</span>
        </div>
        <div class="rows">
          {#each filtered as run, index (run.id)}
            {@const state = runState(run)}
            {@const apply = applyLabel(run)}
            {@const tone = worstTone([state.tone, apply.tone], "verified" as EpistemicTone)}
            <button
              bind:this={rows[index]}
              class="row"
              data-tone={tone}
              class:selected={selected?.id === run.id}
              aria-pressed={selected?.id === run.id}
              onclick={() => onSelectRun(run.id)}
              ondblclick={() => onOpenRun(run.id)}
              onkeydown={(event) => onKeydown(event, index)}
            >
              <span class="edge" aria-hidden="true"></span>
              <span class="run-id" title={run.id}>{run.id}</span>
              <span class="workspace" title={workspaceLabel(run)}>{workspaceLabel(run)}</span>
              <StateChip tone={state.tone} label={state.label} />
              <StateChip tone={apply.tone} label={apply.label} />
              <span class="started" title={startedAt(run)}>{startedAt(run)}</span>
            </button>
          {:else}
            <div class="empty">
              <strong>No runs match</strong>
              <span>{ordered.length ? "Clear the search or the unresolved filter." : "Runs appear here once a workspace has dispatched one."}</span>
            </div>
          {/each}
        </div>
      </article>

      {#if selected}
      <BoundaryPanel
        run={selected}
        {review}
        {reviewError}
        loading={reviewLoading}
        approvals={[]}
        onOpenApprovals={() => {}}
        onReview={onOpenRun}
      />
      {:else}
        <aside class="empty detail-empty" aria-label="Run details">
          <strong>No run selected</strong>
          <span>Choose a matching run to inspect its evidence and review package.</span>
        </aside>
      {/if}
    </div>

    {#if review?.apply_attempts.some((attempt) => isPartiallyApplied(attempt))}
      <p class="unresolved" role="alert">
        <IconAlertTriangle size={14} />
        This run has an attempt whose rollback was never confirmed. Reconcile it before trusting the
        working tree in {selected ? workspaceLabel(selected) : "this workspace"}.
      </p>
    {:else if review?.apply_attempts.length}
      <p class="resolved">
        Last attempt: {review.apply_attempts.at(-1)?.phase} ·
        {attemptTone(review.apply_attempts.at(-1)!.outcome) === "verified" ? "committed" : "not committed"}
      </p>
    {/if}
  {/if}
</section>

<style>
  .screen.history{display:flex;flex:1;min-height:0;width:100%;box-sizing:border-box;flex-direction:column;gap:14px;padding:20px 24px}
  .history-heading,.unresolved,.resolved{flex-shrink:0}
  .history-heading{display:flex;align-items:center;justify-content:space-between;gap:16px;flex-wrap:wrap}
  .history-heading h1{margin:0;font-size:22px;font-weight:640;letter-spacing:-.025em}
  .filters{display:flex;align-items:center;gap:8px}
  .filters label{display:flex;height:30px;align-items:center;gap:7px;padding:0 9px;border:1px solid var(--pytxo-line);border-radius:4px;background:var(--pytxo-surface-input);color:var(--pytxo-text-muted)}
  .filters input{width:190px;border:0;background:transparent;color:var(--pytxo-text-strong);font-family:inherit;font-size:12px;outline:none}
  .filters button{height:30px;padding:0 11px;border:1px solid var(--pytxo-line);border-radius:4px;background:transparent;color:var(--pytxo-text-muted);font-family:inherit;font-size:12px;cursor:pointer}
  .filters button.active{border-color:color-mix(in oklab,var(--state-attention) 50%,var(--pytxo-line));color:var(--state-attention)}

  .history-layout{display:grid;flex:1;min-height:0;grid-template-columns:minmax(0,1.62fr) minmax(300px,1fr);gap:14px;align-items:start}
  .history-layout :global(.boundary){max-height:100%;box-sizing:border-box}
  .history-layout :global(.boundary > *){flex-shrink:0}
  .table{display:flex;min-width:0;min-height:0;max-height:100%;flex-direction:column;overflow:hidden;border:1px solid var(--pytxo-line);border-radius:var(--pytxo-panel-radius,6px);background:var(--pytxo-surface-panel)}
  .table-head,.columns{flex-shrink:0}
  .table-head{display:flex;align-items:center;justify-content:space-between;gap:12px;min-height:40px;padding:0 14px;border-bottom:1px solid var(--pytxo-line-soft)}
  .table-head h2{margin:0;font-size:14px;font-weight:600;letter-spacing:-.02em}
  .table-head .count{color:var(--pytxo-text-muted);font:11px "IBM Plex Mono",monospace;font-variant-numeric:tabular-nums}

  .columns,.row{display:grid;grid-template-columns:minmax(66px,.75fr) minmax(66px,.75fr) minmax(90px,1fr) minmax(105px,1.1fr) minmax(80px,.9fr);align-items:center;gap:8px;padding:0 14px;text-align:left}
  .columns{min-height:28px;border-bottom:1px solid var(--pytxo-line-soft);color:var(--pytxo-text-muted);font:11px "IBM Plex Mono",monospace;text-transform:uppercase;letter-spacing:.04em}
  .rows{display:flex;min-height:0;flex-direction:column;overflow:auto;overscroll-behavior:contain}
  .rows > *{flex-shrink:0}

  .row{position:relative;min-height:44px;padding-block:8px;border:0;border-bottom:1px solid var(--pytxo-line-soft);background:transparent;color:inherit;cursor:pointer;font-family:inherit;transition:background-color var(--pytxo-motion-fast) var(--pytxo-motion-ease)}
  .row :global(.chip strong){white-space:normal;line-height:1.4;overflow:visible;overflow-wrap:anywhere}
  .filters label:focus-within{outline:2px solid var(--pytxo-accent);outline-offset:2px}
  .filters button:focus-visible{outline:2px solid var(--pytxo-accent);outline-offset:2px}
  .row:last-child{border-bottom:0}
  .row:hover{background:color-mix(in oklab,var(--pytxo-surface-raised) 55%,transparent)}
  .row.selected{background:var(--pytxo-surface-active)}
  .row:focus-visible{outline:2px solid var(--pytxo-accent);outline-offset:-2px}
  .edge{position:absolute;inset-block:0;left:0;width:2px;background:var(--tone)}
  .run-id{overflow:hidden;color:var(--pytxo-text-strong);font:11px "IBM Plex Mono",monospace;text-overflow:ellipsis;white-space:nowrap}
  .workspace,.started{overflow:hidden;color:var(--pytxo-text-muted);font-size:11px;text-overflow:ellipsis;white-space:nowrap}
  .started{font-family:"IBM Plex Mono",monospace;font-variant-numeric:tabular-nums}

  .empty{display:flex;min-height:160px;flex-direction:column;align-items:center;justify-content:center;gap:6px;color:var(--pytxo-text-muted);text-align:center}
  .empty strong{color:var(--pytxo-text-soft);font-size:13px}
  .detail-empty{border:1px solid var(--pytxo-line-soft);border-radius:var(--pytxo-panel-radius,6px);padding:24px;background:var(--pytxo-surface-panel)}
  .empty span{max-width:320px;font-size:12px;line-height:1.5}

  .unresolved,.resolved{display:flex;align-items:center;gap:8px;margin:0;padding:10px 12px;border:1px solid var(--pytxo-line-soft);border-radius:4px;font-size:12px;line-height:1.45}
  .unresolved{border-left:2px solid var(--state-refuted);color:var(--state-refuted)}
  .resolved{color:var(--pytxo-text-muted)}
  @container history-viewport (max-width: 940px){
    .history-layout{display:flex;flex-direction:column;overflow:auto;overscroll-behavior:contain}
    .table{width:100%;max-height:none;flex-shrink:0}
    .rows{overflow:visible}
    .history-layout :global(.boundary){width:100%;max-height:none;flex-shrink:0;overflow:visible}
  }
  @container history-viewport (max-width: 560px){
    .screen.history{padding:16px}
    .columns{display:none}
    .row{grid-template-columns:repeat(2,minmax(0,1fr));gap:8px 12px}
    .started{grid-column:1/-1}
    .filters{flex-wrap:wrap}.filters input{width:min(190px,40vw)}
  }
</style>
