<script lang="ts">
  import {
    IconActivity,
    IconCheck,
    IconClockHour4,
    IconHistory,
    IconPlayerPlay,
    IconPlus,
  } from "@tabler/icons-svelte";
  import type { DesktopBackend, DesktopSnapshot } from "../../lib/desktop-backend";
  import type { RunDto } from "../../lib/types";
  import FlowScreen from "./FlowScreen.svelte";
  import RunReviewScreen from "./RunReviewScreen.svelte";

  let {
    backend,
    snapshot,
    preferredDomainId = null,
    initialSurface = "compose",
    initialRunId = null,
    onAddWorkspace = null,
    onSnapshotChanged,
  }: {
    backend: DesktopBackend;
    snapshot: DesktopSnapshot;
    preferredDomainId?: string | null;
    initialSurface?: "compose" | "active" | "history" | "review";
    initialRunId?: string | null;
    onAddWorkspace?: (() => void | Promise<void>) | null;
    onSnapshotChanged: () => void | Promise<void>;
  } = $props();

  let surface = $state<"compose" | "active" | "history" | "review">("compose");
  let selectedRunId = $state<string | null>(null);
  let appliedInitialSurface = $state<"compose" | "active" | "history" | "review">("compose");

  const activeRuns = $derived(
    snapshot.runs.filter((run) =>
      ["running", "pending", "dispatching", "active"].includes(run.status.toLowerCase()),
    ),
  );
  const completedRuns = $derived(
    snapshot.runs.filter((run) =>
      ["completed", "failed", "verify_failed", "cancelled"].includes(run.status.toLowerCase()),
    ),
  );
  const selectedRun = $derived(
    snapshot.runs.find((run) => run.id === selectedRunId) ??
      (surface === "review" ? completedRuns[0] ?? null : null),
  );

  $effect(() => {
    if (initialSurface !== appliedInitialSurface) {
      surface = initialSurface;
      appliedInitialSurface = initialSurface;
    }
    if (initialRunId && initialRunId !== selectedRunId) selectedRunId = initialRunId;
  });

  $effect(() => {
    if (initialSurface === "review" && !selectedRunId && completedRuns[0]) {
      selectedRunId = completedRuns[0].id;
    }
  });

  function domainIdFor(run: RunDto) {
    return snapshot.domains.find((domain) => domain.repo_root === run.repo_root)?.domain_id ?? null;
  }

  function openReview(runId: string) {
    selectedRunId = runId;
    surface = "review";
  }

  type MissionTab = "compose" | "active" | "history";
  const missionTabs: MissionTab[] = ["compose", "active", "history"];

  function activateTab(tab: MissionTab) {
    surface = tab;
  }

  function handleTabKeydown(event: KeyboardEvent, current: MissionTab) {
    const currentIndex = missionTabs.indexOf(current);
    let nextIndex: number | null = null;
    if (event.key === "ArrowRight") nextIndex = (currentIndex + 1) % missionTabs.length;
    if (event.key === "ArrowLeft") nextIndex = (currentIndex - 1 + missionTabs.length) % missionTabs.length;
    if (event.key === "Home") nextIndex = 0;
    if (event.key === "End") nextIndex = missionTabs.length - 1;
    if (nextIndex === null) return;
    event.preventDefault();
    const next = missionTabs[nextIndex];
    activateTab(next);
    document.getElementById(`flow-tab-${next}`)?.focus();
  }

  function relativeTime(value: string) {
    const elapsed = Math.max(0, Date.now() - new Date(value).getTime());
    if (elapsed < 60_000) return "just now";
    if (elapsed < 3_600_000) return `${Math.floor(elapsed / 60_000)}m ago`;
    if (elapsed < 86_400_000) return `${Math.floor(elapsed / 3_600_000)}h ago`;
    return new Date(value).toLocaleDateString();
  }
</script>

{#if surface === "review" && selectedRun}
  <RunReviewScreen
    {backend}
    run={selectedRun}
    domainId={domainIdFor(selectedRun)}
    onBack={() => (surface = "history")}
    onChanged={onSnapshotChanged}
  />
{:else}
  <section class="screen mission-flow">
    <header class="screen-heading mission-heading">
      <div>
        <p class="eyebrow">Mission control</p>
        <h1>Flow</h1>
        <p>Compose, run, and review one mission from a single operational surface.</p>
      </div>
      <div class="mission-tabs" aria-label="Flow mission views" role="tablist">
        <button id="flow-tab-compose" role="tab" aria-selected={surface === "compose"} aria-controls="flow-panel-compose" tabindex={surface === "compose" ? 0 : -1} class:active={surface === "compose"} onclick={() => activateTab("compose")} onkeydown={(event) => handleTabKeydown(event, "compose")}>
          <IconPlus size={14} /> Compose
        </button>
        <button id="flow-tab-active" role="tab" aria-selected={surface === "active"} aria-controls="flow-panel-active" tabindex={surface === "active" ? 0 : -1} class:active={surface === "active"} onclick={() => activateTab("active")} onkeydown={(event) => handleTabKeydown(event, "active")}>
          <IconActivity size={14} /> Active <span>{activeRuns.length}</span>
        </button>
        <button id="flow-tab-history" role="tab" aria-selected={surface === "history"} aria-controls="flow-panel-history" tabindex={surface === "history" ? 0 : -1} class:active={surface === "history"} onclick={() => activateTab("history")} onkeydown={(event) => handleTabKeydown(event, "history")}>
          <IconHistory size={14} /> History <span>{completedRuns.length}</span>
        </button>
      </div>
    </header>

    {#if surface === "compose"}
      <div id="flow-panel-compose" role="tabpanel" aria-labelledby="flow-tab-compose">
        <FlowScreen
          {backend}
          domains={snapshot.domains}
          runs={snapshot.runs}
          {preferredDomainId}
          {onAddWorkspace}
          embedded
        />
      </div>
    {:else if surface === "active"}
      <div id="flow-panel-active" role="tabpanel" aria-labelledby="flow-tab-active" class="panel missions-panel">
        <div class="panel-head">
          <div><p class="eyebrow">In progress</p><h2>Active missions</h2></div>
          <span>{activeRuns.length}</span>
        </div>
        {#if activeRuns.length}
          <div class="mission-list">
            {#each activeRuns as run (run.id)}
              <div class="mission-row">
                <span class="run-state active"><IconPlayerPlay size={14} /></span>
                <div>
                  <strong>{run.id}</strong>
                  <small>{run.repo_root.split(/[\\/]/).pop()} · {run.permission_profile ?? "unknown"} · {relativeTime(run.started_at)}</small>
                </div>
                <span>{run.status}</span>
              </div>
            {/each}
          </div>
        {:else}
          <div class="empty mission-empty">
            <IconClockHour4 size={24} />
            <strong>No active missions</strong>
            <span>Compose a mission when you are ready to dispatch.</span>
            <button class="primary" onclick={() => (surface = "compose")}>Compose mission</button>
          </div>
        {/if}
      </div>
    {:else}
      <div id="flow-panel-history" role="tabpanel" aria-labelledby="flow-tab-history" class="panel missions-panel">
        <div class="panel-head">
          <div><p class="eyebrow">Completed runs</p><h2>Mission history</h2></div>
          <span>{completedRuns.length}</span>
        </div>
        {#if completedRuns.length}
          <div class="mission-list">
            {#each completedRuns as run (run.id)}
              <div class="mission-row completed">
                <span class="run-state"><IconCheck size={14} /></span>
                <div>
                  <strong>{run.id}</strong>
                  <small>{run.repo_root.split(/[\\/]/).pop()} · {run.permission_profile ?? "unknown"} · {relativeTime(run.started_at)}</small>
                </div>
                <span class={`contract ${run.apply_status ?? "unavailable"}`}>{run.apply_status ?? "no review"}</span>
                <button
                  class="review-button"
                  aria-label={`Review ${run.id}`}
                  onclick={() => openReview(run.id)}
                >
                  Review
                </button>
              </div>
            {/each}
          </div>
        {:else}
          <div class="empty mission-empty">
            <IconHistory size={24} />
            <strong>No completed missions</strong>
            <span>Completed and stopped runs appear here with their review state.</span>
          </div>
        {/if}
      </div>
    {/if}
  </section>
{/if}

<style>
  .mission-heading { align-items: flex-end; }
  .mission-heading .eyebrow, .mission-heading h1, .mission-heading p { margin-top: 0; }
  .mission-tabs {
    display: flex;
    gap: 3px;
    padding: 3px;
    border: 1px solid #292f36;
    border-radius: 7px;
    background: #0f1317;
  }
  .mission-tabs button {
    display: inline-flex;
    align-items: center;
    gap: 5px;
    min-height: 29px;
    padding: 6px 9px;
    border: 0;
    color: #7e8993;
    background: transparent;
  }
  .mission-tabs button.active { color: #dbe3e5; background: #1a2224; box-shadow: inset 0 0 0 1px #2d3c3b; }
  .mission-tabs span { min-width: 15px; padding: 2px 4px; border-radius: 8px; color: #7bbdb3; background: #12201e; font: 600 8px/1 "Geist Mono", monospace; }
  .missions-panel { padding: 0; overflow: hidden; }
  .missions-panel .panel-head { padding: 14px 16px; }
  .missions-panel .panel-head h2, .missions-panel .panel-head .eyebrow { margin: 0; }
  .missions-panel .panel-head h2 { margin-top: 4px; font-size: 15px; }
  .mission-list { display: grid; }
  .mission-row {
    display: grid;
    grid-template-columns: auto minmax(0, 1fr) auto;
    gap: 11px;
    align-items: center;
    min-height: 56px;
    padding: 9px 14px;
    border-top: 1px solid #20262d;
  }
  .mission-row.completed { grid-template-columns: auto minmax(0, 1fr) auto auto; }
  .run-state { display: grid; place-items: center; width: 27px; height: 27px; border-radius: 7px; color: #78cabb; background: #13211f; }
  .run-state.active { color: #e0b768; background: #211b10; }
  .mission-row > div { display: grid; gap: 3px; min-width: 0; }
  .mission-row strong { font: 600 11px/1.3 "Geist Mono", monospace; color: #d5dbe0; }
  .mission-row small { overflow: hidden; color: #79848e; font-size: 10px; text-overflow: ellipsis; white-space: nowrap; }
  .mission-row > span { color: #87919b; font: 600 9px/1 "Geist Mono", monospace; text-transform: uppercase; }
  .contract { padding: 4px 6px; border: 1px solid #303841; border-radius: 999px; }
  .contract.ready, .contract.applied { color: #80cbbf; border-color: #2d4a45; }
  .contract.stale, .contract.recovery_required, .contract.review_failed { color: #d9b36b; border-color: #4d4029; }
  .review-button { min-width: 68px; color: #b9d9d4; border-color: #30433f; background: #121b1a; }
  .mission-empty { min-height: 240px; }
  @media (max-width: 760px) {
    .mission-heading { align-items: flex-start; }
    .mission-tabs { width: 100%; overflow-x: auto; }
    .mission-tabs button { flex: 1 0 auto; justify-content: center; }
    .mission-row.completed { grid-template-columns: auto minmax(0, 1fr) auto; }
    .mission-row .contract { grid-column: 2; justify-self: start; }
    .review-button { grid-column: 3; grid-row: 1 / span 2; }
  }
</style>
