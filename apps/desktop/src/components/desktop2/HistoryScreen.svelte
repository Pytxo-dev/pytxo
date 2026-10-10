<script lang="ts">
  import { hasCurrentApplyIssue } from "../../lib/work-activity";
  import { tick } from "svelte";
  import IconAlertTriangle from "@tabler/icons-svelte/icons/alert-triangle";
  import IconSearch from "@tabler/icons-svelte/icons/search";
  import type { DesktopBackend, DesktopSnapshot } from "../../lib/desktop-backend";
  import { isPartiallyApplied, runState, worstTone, type EpistemicTone } from "../../lib/epistemic";
  import type { FlowDraftRecord, RunDto, RunReviewDto } from "../../lib/types";
  import BoundaryPanel from "./BoundaryPanel.svelte";
  import RoutingRunDetails from "./RoutingRunDetails.svelte";

  import { canOpenRunReview } from "../../lib/review-state";
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
    onOpenRun: (runId: string, domainId?: string) => void;
    onSelectRun: (runId: string, domainId?: string) => void;
  } = $props();

  let query = $state("");
  let onlyUnresolved = $state(false);
  let loadedReview = $state<RunReviewDto | null>(null);
  let loadedReviewDomain = $state<string | null>(null);
  let reviewError = $state<string | null>(null);
  let reviewLoading = $state(false);
  let rows: HTMLButtonElement[] = $state([]);
  let drafts = $state<FlowDraftRecord[]>([]);

  const ordered = $derived(
    [...snapshot.runs].sort((a, b) => (b.started_at ?? "").localeCompare(a.started_at ?? "")),
  );

  const filtered = $derived(
    ordered
      .filter((run) => {
        const needle = query.trim().toLowerCase();
        if (!needle) return true;
        const draft = draftFor(run);
        return run.id.toLowerCase().includes(needle) ||
          run.repo_root.toLowerCase().includes(needle) ||
          draft?.title.toLowerCase().includes(needle) ||
          draft?.mission_text.toLowerCase().includes(needle);
      })
      .filter((run) => !onlyUnresolved || needsAttention(run)),
  );

  let selectionKey = $state<string | null>(null);
  let detailOpen = $state(false);
  let technicalOpen = $state(false);
  let refresh = $state(0);
  let actionLoading = $state(false);
  let detailHeading = $state<HTMLHeadingElement>();
  let technical = $state<HTMLDetailsElement>();
  let filesSection = $state<HTMLElement>();
  let candidateFiles = $state<HTMLDetailsElement>();
  let checksSection = $state<HTMLElement>();
  let attemptsSection = $state<HTMLElement>();
  const keyFor = (run: RunDto) => JSON.stringify([run.domain_id, run.id]);
  const selected = $derived(filtered.find(run => keyFor(run) === selectionKey) ?? filtered.find(run => run.id === focusRunId && (!activeDomainId || run.domain_id === activeDomainId)) ?? filtered[0] ?? null);
  const selectedRoutingIdentityUnknown = $derived(!!selected && snapshot.diagnostics.some((diagnostic) =>
    diagnostic.stage === "routing_revision" && diagnostic.domain_id === selected.domain_id && diagnostic.run_id === selected.id));
  const review = $derived(loadedReview?.run_id === selected?.id && loadedReviewDomain === selected?.domain_id ? loadedReview : null);

  /**
   * A run needs attention when its own record says an apply did not finish
   * cleanly, which is the only signal available without loading each receipt.
   */
  function needsAttention(run: RunDto): boolean {
    if (hasCurrentApplyIssue(run)) return true;
    return runState(run).tone === "refuted";
  }

  function applyLabel(run: RunDto, record: RunReviewDto | null = null): { tone: EpistemicTone; label: string } {
    const evidence = record ?? run;
    if (evidence.apply_status === "applied" && evidence.applied_at && !hasCurrentApplyIssue(evidence) && !record?.apply_attempts.some(isPartiallyApplied)) return { tone: "verified", label: "Apply recorded" };
    const recovery = record ? record.recovery_state : run.recovery_state;
    const error = record ? record.last_apply_error : run.last_apply_error;
    if (recovery === "rolled_back" && error?.rollback_confirmed && !record?.apply_attempts.some(isPartiallyApplied)) return { tone: "claimed", label: "Rollback confirmed" };
    if (recovery && recovery !== "none" || record?.apply_attempts.some(isPartiallyApplied)) return { tone: "refuted", label: "Outcome needs reconciliation" };
    const status = record ? record.apply_status : run.apply_status;
    if (status === "recovery_required") return { tone: "refuted", label: "Outcome needs reconciliation" };
    if (status === "applying") return { tone: "unknown", label: "Apply outcome unconfirmed" };
    if (!hasCurrentApplyIssue(evidence) && (status === "applied" || evidence.applied_at)) return { tone: "verified", label: "Apply recorded" };
    if (status === "review_failed") return { tone: "refuted", label: "Preparation failed" };
    if (record ? record.last_apply_error : run.last_apply_error) return { tone: "refuted", label: "Apply error recorded" };
    return { tone: "unknown", label: "No confirmed Apply" };
  }
  const outcome = $derived(selected ? applyLabel(selected, review) : null);
  const boundary = $derived(boundarySnapshot(outcome?.label));
  const actionLabel = $derived(outcome?.label === "Outcome needs reconciliation" ? "Inspect recovery" : outcome?.label === "Apply recorded" ? "View applied result" : review?.prepared_manifest ? "Review prepared changes" : "View run details");
  async function choose(run: RunDto, enter = true) {
    selectionKey = keyFor(run); detailOpen = enter; technicalOpen = false;
    onSelectRun(run.id, run.domain_id);
    if (enter) { await tick(); detailHeading?.focus({ preventScroll: true }); }
  }
  async function backToRuns() {
    detailOpen = false; await tick();
    const index = filtered.findIndex(run => selected && keyFor(run) === keyFor(selected));
    rows[index]?.focus();
  }
  function inspectPart(part: "candidate" | "files" | "checks" | "attempts" | "technical") {
    if (part === "technical") technicalOpen = true;
    if (part === "candidate" && candidateFiles) candidateFiles.open = true;
    void tick().then(() => { const el = part === "candidate" ? (candidateFiles ?? filesSection) : part === "files" ? filesSection : part === "checks" ? checksSection : part === "attempts" ? attemptsSection : technical; el?.focus(); el?.scrollIntoView({ block: "nearest" }); });
  }
  async function openSelected() {
    const run = selected;
    if (!run || actionLoading) return;
    if (actionLabel === "View run details") { inspectPart("technical"); return; }
    actionLoading = true;
    try {
      const fresh = await backend.runReview(run.id, run.domain_id);
      if (!selected || keyFor(selected) !== keyFor(run)) return;
      if (fresh.run_id !== run.id || fresh.prepared_manifest && fresh.prepared_manifest.run_id !== run.id) throw new Error("Evidence scope mismatch");
      loadedReview = fresh; loadedReviewDomain = run.domain_id;
      if (canOpenRunReview(run, fresh)) onOpenRun(run.id, run.domain_id);
      else inspectPart("technical");
    } catch (cause) { if (selected && keyFor(selected) === keyFor(run)) reviewError = String(cause); }
    finally { actionLoading = false; }
  }

  function workspaceLabel(run: RunDto) {
    return run.repo_root.split(/[\\/]/).pop() ?? run.repo_root;
  }

  function draftFor(run: RunDto) {
    return drafts.find((draft) => draft.dispatched_run_id === run.id && draft.domain_id === run.domain_id) ?? null;
  }

  function requestLabel(run: RunDto) {
    return draftFor(run)?.title.trim() || `Work in ${workspaceLabel(run)}`;
  }

  function startedAt(run: RunDto) {
    const parsed = new Date(run.started_at);
    return Number.isNaN(parsed.getTime()) ? run.started_at : parsed.toLocaleString();
  }

  function domainIdForRun(run: RunDto) {
    return run.domain_id;
  }

  function boundarySnapshot(label: string | undefined) {
    if (label === "Apply recorded") return { title: "Applied to the project", detail: "Apply recorded" };
    if (label === "Outcome needs reconciliation") return { title: "Project state", detail: "Needs recovery" };
    if (label === "Apply outcome unconfirmed") return { title: "Project state", detail: "Apply not confirmed" };
    if (label === "Rollback confirmed") return { title: "Project state", detail: "Rollback recorded" };
    if (label === "Preparation failed" || label === "Apply error recorded") return { title: "Project state", detail: "Failure recorded" };
    return { title: "Project", detail: "Not applied" };
  }

  function displayAttempt(value: string) {
    const words = value.replaceAll("_", " ");
    return words.charAt(0).toUpperCase() + words.slice(1);
  }

  $effect(() => {
    let current = true;
    backend.flowHistory().then((records) => {
      if (current) drafts = records;
    }).catch(() => {
      if (current) drafts = [];
    });
    return () => {
      current = false;
    };
  });

  /** The receipt for the selected run, so History answers "what was enforced". */
  $effect(() => {
    void refresh;
    const run = selected;
    loadedReview = null; loadedReviewDomain = null;
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
        if (current) {
          if (result.run_id !== run.id || result.prepared_manifest && result.prepared_manifest.run_id !== run.id) throw new Error("Evidence scope mismatch");
          loadedReview = result; loadedReviewDomain = run.domain_id;
        }
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
    void choose(filtered[next], false);
  }
</script>

<section class="screen history">
  <header class="history-heading"><div><h1>History</h1><p>Every run, across your projects</p></div>
    <div class="filters"><label><IconSearch size={14}/><input bind:value={query} placeholder="Search request or project" aria-label="Search work history" /></label><button class:active={onlyUnresolved} aria-pressed={onlyUnresolved} onclick={() => onlyUnresolved = !onlyUnresolved}>{onlyUnresolved ? "Unresolved only" : "All runs"}</button></div>
  </header>
  {#if snapshot.error}<p role="alert">Local service offline · {snapshot.error.message}</p>{:else}
  <div class="history-layout" class:detail-open={detailOpen && !!selected}>
    <nav class="run-navigator table" aria-label="Recorded runs">
      <header><h2>Runs</h2><span>{filtered.length} of {ordered.length}</span></header>
      <div class="rows">
        {#each filtered as run, index (keyFor(run))}
          {@const state = runState(run)}{@const apply = applyLabel(run)}
          <button bind:this={rows[index]} class="row" data-run-id={run.id} data-domain-id={run.domain_id} data-tone={worstTone([state.tone, apply.tone], "unknown")} class:selected={selected && keyFor(selected) === keyFor(run)} aria-pressed={!!selected && keyFor(selected) === keyFor(run)} onclick={() => choose(run)} onkeydown={event => onKeydown(event,index)}>
            <strong>{requestLabel(run)}</strong><span>{workspaceLabel(run)}</span>
            <span class="row-states"><span>{state.label}</span><span>{apply.label}</span></span>
            <time datetime={run.started_at}>{startedAt(run)}</time>
          </button>
        {:else}<div class="empty"><strong>{ordered.length ? "No runs match" : "No recorded runs"}</strong><p>{ordered.length ? "Clear the search or unresolved filter." : "Dispatched work will appear here."}</p></div>{/each}
      </div>
    </nav>
    {#if selected}
      <article class="outcome-inspector" aria-label="Selected run workspace">
        <button class="back-runs" onclick={backToRuns}>Back to runs</button>
        <header class="outcome-heading"><div><h2 bind:this={detailHeading} tabindex="-1">{requestLabel(selected)}</h2><p class="destination">Project · {selected.repo_root}</p><div class="outcome-label"><span><small>Run</small><strong>{runState(selected).label}</strong></span><span><small>Apply</small><StateChip tone={outcome?.tone ?? "unknown"} label={outcome?.label ?? "Outcome unavailable"}/></span></div></div><button class="outcome-action" disabled={reviewLoading || actionLoading || (!!reviewError && actionLabel !== "View run details")} onclick={openSelected}>{actionLoading ? "Refreshing record…" : actionLabel}</button></header>
        {#if draftFor(selected)?.mission_text}<details class="request-text"><summary>Full recorded request</summary><p>{draftFor(selected)?.mission_text}</p></details>{/if}
        {#if reviewLoading}<p role="status">Loading this run’s evidence…</p>{/if}
        {#if reviewError}<p role="alert">Evidence unavailable: {reviewError} <button onclick={() => refresh++}>Reload evidence</button></p>{/if}
        {#if needsAttention(selected) || outcome?.tone === "refuted"}<p class="recovery-notice" role="alert">{review?.last_apply_error?.message ?? selected.last_apply_error?.message ?? "Check the project state before doing anything else."}</p>{/if}
        <section class="recorded-trace" aria-label="Recorded run snapshots">
          <header><h3>What happened</h3><span>From saved records, not a full timeline</span></header>
          <div class="trace-lane"><span class="lane-label">Run</span><button onclick={() => inspectPart("technical")}><strong>{runState(selected).label}</strong><small>Started {startedAt(selected)}</small></button></div>
          <div class="trace-lane integration"><span class="lane-label">Changes</span><button onclick={() => inspectPart("candidate")}><strong>{review?.prepared_manifest ? `${review.prepared_manifest.files.length} prepared file${review.prepared_manifest.files.length === 1 ? "" : "s"}` : reviewLoading ? "Loading changes" : "No changes recorded"}</strong><small>Prepared, not applied by themselves</small></button><button onclick={() => inspectPart("checks")}><strong>{review?.prepared_manifest?.candidate_verification?.checks.length ? (review.prepared_manifest.candidate_verification.checks.every(c => c.passed) ? "Checks passed" : "A check failed") : "Checks not recorded"}</strong><small>On the combined changes</small></button></div>{#if review?.prepared_manifest && (review?.applied_at || review?.apply_attempts.length)}<p class="identity-gap">This record does not link these prepared changes to the Apply below.</p>{/if}<div class="trace-lane integration-record"><span class="lane-label">Apply</span><span class="aperture" aria-hidden="true"></span><button class:confirmed={outcome?.label === "Apply recorded"} onclick={() => inspectPart("attempts")}><strong>{boundary.title}</strong><small>{boundary.detail}</small></button></div>
          <details class="record-limits"><summary>About this record</summary><p>Only saved snapshots and Apply attempts are shown, not every step of the run. {outcome?.label === "Apply recorded" ? "The Apply record shows what was written then, not what is on disk now." : "Having no recorded Apply does not prove the project is unchanged."} Old check results cannot be used to Apply now.</p></details>
        </section>
        {#if selectedRoutingIdentityUnknown}<p class="recovery-notice" role="alert">Routing identity could not be checked for this run. Worker attempt identity is unavailable until the run list is repaired.</p>{/if}
        {#if selected.routing_revision != null}<RoutingRunDetails run={selected} {backend} />{/if}
        <div class="record-details">
          <section bind:this={filesSection} tabindex="-1" class="files-section"><header><h3>{outcome?.label === "Apply recorded" && review?.apply_manifest && "changes" in review.apply_manifest ? "Recorded applied files" : "Prepared files"}</h3>{#if outcome?.label === "Apply recorded" && review?.apply_manifest && "changes" in review.apply_manifest}<span>{review.apply_manifest.changes.length}</span>{:else if review?.prepared_manifest}<span>{review.prepared_manifest.files.length}</span>{/if}</header>
            {#if outcome?.label === "Apply recorded" && review?.apply_manifest && "changes" in review.apply_manifest}
              {#each review.apply_manifest.changes as file}<div class="file"><code>{file.path}</code><span>{file.kind}</span></div>{/each}
              {#if review.prepared_manifest}<details class="candidate-inventory" bind:this={candidateFiles} tabindex="-1"><summary>All prepared files · {review.prepared_manifest.files.length}</summary>{#each review.prepared_manifest.files as file}<div class="file"><code>{file.path}</code><span>{file.kind}</span></div>{/each}</details>{/if}
            {:else if review?.prepared_manifest}
              {#each review.prepared_manifest.files as file}<div class="file"><code>{file.path}</code><span>{file.kind}</span></div>{:else}<p>No files were changed.</p>{/each}
            {:else}<p class="subtle">No file inventory available.</p>{/if}
          </section>
          <section bind:this={checksSection} tabindex="-1"><h3>Recorded checks</h3>
            {#if review?.prepared_manifest?.candidate_verification}
              {#each review.prepared_manifest.candidate_verification.checks as check}<div class="check"><strong>{check.passed ? "Passed" : "Failed"}</strong><code>{check.command}</code><span>{check.task_id}</span></div>{:else}<p>No required checks recorded.</p>{/each}
            {:else}<p class="subtle">Checks not recorded.</p>{/if}
          </section>
        </div>
        {#if review?.applied_at || review?.apply_attempts.length}
          <section bind:this={attemptsSection} tabindex="-1" class="attempts"><h3>Apply attempts</h3>
            {#if review?.applied_at}<p>Apply recorded at {review.applied_at}.</p>{/if}
            {#each review?.apply_attempts ?? [] as attempt}<div class="attempt"><strong>{displayAttempt(attempt.outcome)}</strong><time>{attempt.created_at ?? "Time not recorded"}</time>{#if attempt.error_message}<p>{attempt.error_message}</p>{/if}</div>{/each}
          </section>
        {/if}
        <details bind:this={technical} bind:open={technicalOpen} tabindex="-1" class="technical"><summary>Technical details</summary><div class="technical-ids"><span>Run <code>{selected.id}</code></span><span>Domain <code>{selected.domain_id}</code></span>{#if review?.prepared_manifest}<span>Prepared change <code>{review.prepared_manifest.package_digest}</code></span><span>Prepared <code>{review.prepared_manifest.prepared_at}</code></span>{/if}{#if review?.apply_manifest && "changes" in review.apply_manifest}<span>Apply transaction <code>{review.apply_manifest.transaction_id}</code></span>{/if}{#if review?.prepared_manifest?.candidate_verification}<span>Verified <code>{review.prepared_manifest.candidate_verification.verified_at}</code></span>{/if}{#each review?.apply_attempts ?? [] as attempt}<span>Attempt <code>{attempt.attempt_id}</code> · phase <code>{attempt.phase}</code></span>{/each}</div><p class="subtle">Attempts are listed as recorded. They are matched to a change set only where the record links them.</p><BoundaryPanel showReviewAction={false} run={selected} {review} {reviewError} loading={reviewLoading} approvals={[]} onOpenApprovals={() => {}} onReview={() => onOpenRun(selected.id,selected.domain_id)}/></details>
      </article>
    {:else}<div class="empty detail-empty"><strong>No run selected</strong><p>Choose a matching run to inspect its recorded outcome.</p></div>{/if}
  </div>
  {#if review?.apply_attempts.some(isPartiallyApplied)}<p class="unresolved" role="alert"><IconAlertTriangle size={14}/>An attempt has no confirmed rollback. Inspect recovery before trusting the working tree.</p>{/if}
  {/if}
</section>
<style>
  .history { display:flex;flex:1;min-height:0;flex-direction:column;gap:16px;font-family:var(--pytxo-font-ui); }
  .history-heading { display:flex;justify-content:space-between;align-items:center;gap:16px;flex-wrap:wrap; }
  h1 { margin:0;font-size:var(--pytxo-title-size);font-weight:600;letter-spacing:-.02em; } h2 { margin:0;font-size:22px;line-height:1.4;overflow-wrap:anywhere; } h3 { margin:0 0 12px;font-size:15px; }
  .history-heading p { margin:6px 0 0;color:var(--pytxo-text-muted);font-size:12px; }
  button,input { font:inherit; } button { cursor:pointer; } button:disabled { opacity:.55;cursor:default; }
  .filters { display:flex;gap:8px;flex-wrap:wrap; }.filters label { display:flex;align-items:center;gap:8px;padding:0 10px;border:1px solid var(--pytxo-line);border-radius:4px;background:var(--pytxo-surface-input); }
  input { width:200px;min-height:38px;border:0;background:transparent;color:var(--pytxo-text-strong);font-size:13px; }
  .filters button,.outcome-action,.back-runs,[role="alert"] button { padding:8px 12px;border:1px solid var(--pytxo-line);border-radius:4px;background:var(--pytxo-surface-raised);color:var(--pytxo-text-strong);font-size:13px;min-height:38px; }
  .filters .active { color:var(--state-attention); }
  button:focus-visible,input:focus-visible,summary:focus-visible,[tabindex="-1"]:focus-visible { outline:2px solid var(--pytxo-accent);outline-offset:2px; }
  .history-layout { display:grid;grid-template-columns:300px minmax(0,1fr);gap:18px;flex:1;min-height:0; }
  .run-navigator { display:flex;flex-direction:column;min-height:0;min-width:0;border:1px solid var(--pytxo-line);background:var(--pytxo-surface-panel);align-self:start;max-height:100%; }
  .run-navigator>header { display:flex;justify-content:space-between;padding:14px;border-bottom:1px solid var(--pytxo-line); }.run-navigator h2 { font-size:14px; }.run-navigator header span { font-size:12px;color:var(--pytxo-text-muted); }
  .rows { overflow:auto;min-height:0; }.row { display:grid;width:100%;gap:8px;padding:16px;text-align:left;background:transparent;color:var(--pytxo-text-soft);border:0;border-bottom:1px solid var(--pytxo-line-soft); }
  .row strong { color:var(--pytxo-text-strong);font-size:14px;line-height:1.5;display:-webkit-box;-webkit-line-clamp:2;line-clamp:2;-webkit-box-orient:vertical;overflow:hidden;overflow-wrap:anywhere; }.row>span,.row time { font-size:12px;overflow-wrap:anywhere; }.row-states { display:flex;flex-wrap:wrap;gap:5px 12px; }.row time { color:var(--pytxo-text-muted);font-variant-numeric:tabular-nums; }.row:hover { background:var(--pytxo-surface-raised); }.row.selected { background:var(--pytxo-surface-active);box-shadow:inset 2px 0 var(--pytxo-accent); }
  .outcome-inspector { min-width:0;min-height:0;overflow:auto;padding:24px;border:1px solid var(--pytxo-line);background:var(--pytxo-work-canvas); }
  .outcome-heading { display:flex;align-items:flex-start;justify-content:space-between;gap:20px; }.outcome-heading>div { min-width:0; }.outcome-action { flex-shrink:0; }.destination { margin:5px 0 14px;overflow-wrap:anywhere;font-size:13px;color:var(--pytxo-text-soft); }.outcome-label { display:flex;gap:18px;align-items:flex-start;flex-wrap:wrap;font-size:13px; }.outcome-label>span { display:grid;gap:4px; }.outcome-label small { color:var(--pytxo-text-muted);font-size:10px;font-weight:600;letter-spacing:.06em;text-transform:uppercase; }.outcome-label strong { font-size:13px; }
  .request-text { margin-top:18px; }.request-text summary { cursor:pointer;font-size:13px; }.request-text p { white-space:pre-wrap;max-width:75ch;line-height:1.7; }
  .subtle { color:var(--pytxo-text-muted);font-size:12px;line-height:1.65;overflow-wrap:anywhere; }.recorded-trace { margin-top:24px;border-block:1px solid var(--pytxo-line);padding-block:18px; }.recorded-trace header { display:flex;justify-content:space-between;gap:16px;flex-wrap:wrap;align-items:baseline; }.recorded-trace header>span { font-size:11px;color:var(--pytxo-text-muted); }
  .trace-lane { display:flex;align-items:stretch;gap:14px;margin-block:10px; }.lane-label { width:82px;flex-shrink:0;padding-top:11px;font-size:11px;font-weight:600;letter-spacing:.04em;text-transform:uppercase;color:var(--pytxo-text-muted); }.trace-lane button { position:relative;display:grid;gap:5px;flex:1;min-width:0;padding:10px 8px;text-align:left;border:0;border-top:1px solid var(--pytxo-line);border-radius:0;background:transparent;color:var(--pytxo-text-strong); }.trace-lane button::before { content:"";position:absolute;top:-4px;left:8px;width:7px;height:7px;border:1px solid var(--pytxo-text-muted);border-radius:50%;background:var(--pytxo-work-canvas); }.trace-lane button:hover { background:color-mix(in srgb,var(--pytxo-surface-raised) 55%,transparent); }.trace-lane button strong { font-size:13px;font-weight:600; }.trace-lane small { font-size:11px;color:var(--pytxo-text-muted);line-height:1.5; }.aperture { width:2px;min-height:56px;background:var(--pytxo-aperture);flex-shrink:0; }.trace-lane .confirmed { border-top-color:var(--state-verified); }.trace-lane .confirmed::before { border-color:var(--state-verified);background:var(--state-verified); }
  .record-limits { margin:12px 0 0;color:var(--pytxo-text-muted);font-size:12px; }.record-limits summary { cursor:pointer;width:max-content;max-width:100%; }.record-limits p { max-width:78ch;margin:8px 0 0;line-height:1.65; }
  .record-details { display:grid;grid-template-columns:1fr 1fr;gap:28px;padding-block:24px; }.record-details>section { min-width:0; }.record-details section>header { display:flex;justify-content:space-between;gap:12px;align-items:baseline; }.record-details section>header span { color:var(--pytxo-text-muted);font-variant-numeric:tabular-nums;font-size:12px; }.file,.check { display:flex;gap:8px;justify-content:space-between;flex-wrap:wrap;padding:10px 0;border-bottom:1px solid var(--pytxo-line-soft);font-size:12px; }.check { flex-direction:column; }.file code { flex:1; }.file span,.check span { color:var(--pytxo-text-muted); }
  code { font:11px/1.6 "IBM Plex Mono",monospace;overflow-wrap:anywhere; }.attempts { padding-block:18px;border-top:1px solid var(--pytxo-line);font-size:13px; }.attempt { display:flex;flex-wrap:wrap;gap:8px 14px;padding:10px 0;border-bottom:1px solid var(--pytxo-line-soft); }.attempt p { flex-basis:100%;margin:0;color:var(--state-refuted); }.attempt time { color:var(--pytxo-text-muted);font-variant-numeric:tabular-nums; }.technical { border-top:1px solid var(--pytxo-line);padding-top:18px;font-size:13px; }.technical summary { cursor:pointer; }.technical-ids { display:grid;grid-template-columns:repeat(2,minmax(0,1fr));gap:7px 18px;margin:14px 0;color:var(--pytxo-text-soft);font-size:12px; }.technical :global(.boundary) { margin-top:16px; }
  .recovery-notice,.unresolved,[role="alert"] { color:var(--state-refuted);font-size:13px;line-height:1.6; }.unresolved { display:flex;gap:8px;margin:0; }.empty { padding:28px;color:var(--pytxo-text-muted);font-size:13px;line-height:1.7; }.back-runs { display:none; }
  @container history-viewport (max-width:1000px) { .history-layout { grid-template-columns:250px minmax(0,1fr); }.outcome-inspector { padding:18px; }.outcome-heading { flex-direction:column; }.trace-lane { flex-wrap:wrap; }.lane-label { width:100%; }.record-details { grid-template-columns:1fr; } }
  @container history-viewport (max-width:900px) { .history-layout { display:block;overflow:auto; }.outcome-inspector { display:none;overflow:visible; }.detail-open .run-navigator { display:none; }.detail-open .outcome-inspector { display:block; }.back-runs { display:inline-block;margin-bottom:18px; }.run-navigator { max-height:none; }.rows { overflow:visible; }.trace-lane button { flex-basis:140px; }.filters input { width:min(200px,50vw); } }
  .history { width:100%; max-width:none; margin-inline:0; box-sizing:border-box; }
  .history-layout { grid-template-columns:280px minmax(0,1fr); }
  .recorded-trace { margin-top:16px; padding-block:12px; }
  .trace-lane:first-of-type { align-items:center; margin-block:8px; }
  .trace-lane:first-of-type button { display:flex; gap:12px; align-items:baseline; padding-block:4px; border:0; }
  .trace-lane:first-of-type button::before { display:none; }
  .trace-lane:first-of-type .lane-label { padding-top:0; }
  .identity-gap { margin:8px 0 8px 96px; max-width:65ch; font-size:12px; line-height:1.5; color:var(--pytxo-text-soft); }
  .integration-record { padding-top:4px; }.integration-record .aperture { min-height:38px; }
  .record-details { padding-block:18px; }
  @container history-viewport (max-width:1000px) { .history-layout { grid-template-columns:250px minmax(0,1fr); }.identity-gap { margin-left:0; } }

  .candidate-inventory { margin-top:12px; font-size:12px; }.candidate-inventory summary { cursor:pointer; color:var(--pytxo-text-soft); }

  /* History is a settled record browser, not a continuation of the live run. */

  .filters label,.filters button { border-radius:6px; }
  .history-layout { gap:12px; }
  .run-navigator {
    overflow:hidden;
    border-color:color-mix(in srgb,var(--pytxo-line) 88%,var(--pytxo-text-muted));
    border-radius:0;
    background:color-mix(in srgb,var(--pytxo-surface-panel) 94%,var(--pytxo-work-canvas));
  }
  .run-navigator>header { padding:12px 14px;background:color-mix(in srgb,var(--pytxo-surface-panel) 76%,var(--pytxo-work-canvas)); }
  .row { gap:7px;padding:14px;transition:background-color var(--pytxo-motion-fast) var(--pytxo-motion-ease); }
  .row:hover { background:color-mix(in srgb,var(--pytxo-surface-raised) 82%,var(--pytxo-work-canvas)); }
  .row.selected {
    background:var(--pytxo-surface-active);
    box-shadow:inset 1px 0 var(--pytxo-text-strong);
  }
  .row.selected strong { color:var(--pytxo-text-strong); }
  .outcome-inspector {
    border-color:color-mix(in srgb,var(--pytxo-line) 84%,var(--pytxo-text-muted));
    border-radius:0;
    background:var(--pytxo-work-canvas);
    box-shadow:none;
  }
  .outcome-heading { padding-bottom:18px;border-bottom:1px solid var(--pytxo-line-soft); }
  .outcome-heading h2 { max-width:34ch;font-size:var(--pytxo-title-size);font-weight:600;line-height:1.28;letter-spacing:-.02em;text-wrap:pretty; }
  .outcome-action { border-radius:6px;background:var(--pytxo-text-strong);color:var(--pytxo-surface-shell);font-weight:600; }
  .outcome-action:hover:not(:disabled) { background:var(--pytxo-text-soft); }
  .recorded-trace {
    margin-top:18px;
    padding:16px 0;
    border:0;
    border-radius:0;
    background:transparent;
  }
  .recorded-trace header { padding-bottom:4px; }
  .trace-lane { gap:10px;margin-block:9px; }
  .trace-lane button { padding:10px;border-radius:4px;background:color-mix(in srgb,var(--pytxo-work-node) 70%,transparent); }
  .trace-lane button:hover { background:var(--pytxo-surface-active); }
  .trace-lane:first-of-type button { padding:8px 10px;background:transparent; }
  .trace-lane button::before { left:10px;background:var(--pytxo-surface-panel); }
  .identity-gap { margin-block:6px;padding:7px 10px;border:1px solid var(--pytxo-line-soft);border-radius:4px;background:color-mix(in srgb,var(--pytxo-surface-panel) 56%,transparent); }
  .integration-record .aperture { width:3px;border-radius:2px; }
  .record-details { gap:36px; }
  .record-details h3,.attempts h3 { letter-spacing:-.015em; }
  .file,.check { padding-block:11px; }
  .technical { border-color:var(--pytxo-line-soft); }

  @container history-viewport (max-width:1000px) {
    .outcome-heading h2 { font-size:21px; }
    .recorded-trace { padding:14px; }
  }

  /* Terminal ledger: one glyph per run for its worst recorded state, mono
     metadata, and the trace drawn as a rail. */
  .run-navigator, .outcome-inspector { border-radius:6px; }
  .run-navigator>header h2 { font:600 11.5px var(--pytxo-font-mono); letter-spacing:.08em; text-transform:uppercase; color:var(--pytxo-text-strong); }
  .run-navigator>header span, .row time, .row-states { font-family:var(--pytxo-font-mono); font-size:11px; }
  .row { position:relative; padding-left:36px; }
  .row::before { position:absolute; left:13px; top:15px; content:"○"; color:var(--state-unknown); font:600 12px var(--pytxo-font-mono); }
  .row[data-tone="verified"]::before { content:"✓"; color:var(--state-verified); }
  .row[data-tone="refuted"]::before { content:"✗"; color:var(--state-refuted); }
  .row[data-tone="claimed"]::before { content:"◐"; color:var(--state-claimed, var(--state-attention)); }
  .row[data-tone="active"]::before { content:"▸"; color:var(--live); }
  .row-states>span+span::before { content:"· "; color:var(--pytxo-text-muted); }
  .outcome-label small, .lane-label { font-family:var(--pytxo-font-mono); font-weight:500; letter-spacing:.06em; }
  .recorded-trace header h3 { font:600 11.5px var(--pytxo-font-mono); letter-spacing:.08em; text-transform:uppercase; }
  .trace-lane button { border:1px solid var(--pytxo-line-soft); border-radius:4px; background:transparent; }
  .trace-lane button::before { top:-5px; border-radius:0; transform:rotate(45deg); }
  .trace-lane:first-of-type button { border:0; }
  .trace-lane button small { font-family:var(--pytxo-font-mono); }
  .file code, .check code { color:var(--pytxo-text-body); }
  .file span { font:11px var(--pytxo-font-mono); }
</style>
