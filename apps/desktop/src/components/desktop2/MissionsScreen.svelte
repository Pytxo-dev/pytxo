<script lang="ts">
  /**
   * The two Work panes that are not the live ledger: plan a run, or read the
   * immutable package a finished run produced. There is no run-inventory branch
   * and no combined plan/live/review tab strip — History owns the inventory and
   * `WorkActive` owns the live view, so each surface answers one question.
   */
  import { IconArrowLeft } from "@tabler/icons-svelte";
  import type { DesktopBackend, DesktopSnapshot } from "../../lib/desktop-backend";
  import type { MissionPane, MissionView } from "../../lib/navigation.svelte";
  import FlowScreen from "./FlowScreen.svelte";
  import RunReviewScreen from "./RunReviewScreen.svelte";

  let {
    view,
    snapshot,
    backend,
    preferredDomainId = null,
    focusRunId = null,
    focusDomainId = null,
    onView,
    onOpenMission,
    onAddWorkspace,
    onRunCompleted,
  }: {
    view: MissionView;
    snapshot: DesktopSnapshot;
    backend: DesktopBackend;
    preferredDomainId?: string | null;
    focusRunId?: string | null;
    focusDomainId?: string | null;
    onView: (view: MissionView) => void;
    onOpenMission: (runId: string, pane?: MissionPane) => void;
    onAddWorkspace: () => void | Promise<void>;
    onRunCompleted: () => void;
  } = $props();

  const focusedRun = $derived(snapshot.runs.find((r) => r.id === focusRunId) ?? snapshot.runs[0] ?? null);
  const detailDomainId = $derived(
    focusDomainId ??
      focusedRun?.domain_id ??
      null,
  );
</script>

{#if view === "compose"}
  <FlowScreen
    {backend}
    domains={snapshot.domains}
    runs={snapshot.runs}
    {preferredDomainId}
    {onAddWorkspace}
    onDispatched={(runId) => onOpenMission(runId, "live")}
  />
{:else if focusedRun}
  <RunReviewScreen
    {backend}
    run={focusedRun}
    domainId={detailDomainId}
    onBack={() => onView("list")}
    onChanged={onRunCompleted}
  />
{:else}
  <section class="screen">
    <header class="screen-heading">
      <div>
        <button class="back" onclick={() => onView("list")}>
          <IconArrowLeft size={15} /> Runs
        </button>
        <h1>Run Review</h1>
      </div>
    </header>
    <article class="panel">
      <div class="empty">
        <strong>No run selected</strong>
        <span>Choose a run in History to read the package it prepared.</span>
      </div>
    </article>
  </section>
{/if}
