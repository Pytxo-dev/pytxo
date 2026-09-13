<script lang="ts">
  /**
   * The two Work panes that are not the live ledger: plan a run, or read the
   * immutable package a finished run produced. There is no run-inventory branch
   * and no combined plan/live/review tab strip — History owns the inventory and
   * `WorkActive` owns the live view, so each surface answers one question.
   */
  import IconArrowLeft from "@tabler/icons-svelte/icons/arrow-left";
  import type { DesktopBackend, DesktopSnapshot } from "../../lib/desktop-backend";
  import type { MissionPane, MissionView } from "../../lib/navigation.svelte";
  import FlowScreen from "./FlowScreen.svelte";
  import type { ComposerDraft } from "../../lib/composer-draft";
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
    composerDraft = null,
    preferredAdeId = null,
    onDraftChange = () => {},
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
    composerDraft?: ComposerDraft | null;
    preferredAdeId?: string | null;
    onDraftChange?: (draft: ComposerDraft | null) => void;
  } = $props();

  const focusedRun = $derived(focusRunId
    ? snapshot.runs.find((r) => r.id === focusRunId && (!focusDomainId || r.domain_id === focusDomainId)) ?? null
    : snapshot.runs.find(r => !preferredDomainId || r.domain_id === preferredDomainId) ?? null);
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
    {preferredAdeId}
    draft={composerDraft}
    {onDraftChange}
    {onAddWorkspace}
    onDispatched={(runId) => onOpenMission(runId, "live")}
  />
{:else if focusedRun}
  {#key JSON.stringify([detailDomainId, focusedRun.id])}
    <RunReviewScreen
      {backend}
      run={focusedRun}
      domainId={detailDomainId}
      onBack={() => onView("list")}
      onChanged={onRunCompleted}
    />
  {/key}
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
