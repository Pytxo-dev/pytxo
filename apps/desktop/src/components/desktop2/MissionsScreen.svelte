<script lang="ts">
  import { IconChevronRight } from "@tabler/icons-svelte";
  import type { DesktopBackend, DesktopSnapshot } from "../../lib/desktop-backend";
  import type { MissionPane, MissionView } from "../../lib/navigation.svelte";
  import FlowScreen from "./FlowScreen.svelte";
  import MissionDetail from "./MissionDetail.svelte";

  let {
    view,
    pane,
    snapshot,
    backend,
    preferredDomainId = null,
    focusRunId = null,
    focusDomainId = null,
    onView,
    onPane,
    onOpenMission,
    onAddWorkspace,
    onRunCompleted,
    onStopRun,
  }: {
    view: MissionView;
    pane: MissionPane;
    snapshot: DesktopSnapshot;
    backend: DesktopBackend;
    preferredDomainId?: string | null;
    focusRunId?: string | null;
    focusDomainId?: string | null;
    onView: (view: MissionView) => void;
    onPane: (pane: MissionPane) => void;
    onOpenMission: (runId: string, pane?: MissionPane) => void;
    onAddWorkspace: () => void | Promise<void>;
    onRunCompleted: () => void;
    onStopRun: (runId: string, domainId: string) => Promise<void>;
  } = $props();

  const focusedRun = $derived(snapshot.runs.find((r) => r.id === focusRunId) ?? snapshot.runs[0] ?? null);
  const detailDomainId = $derived(
    focusDomainId ??
      snapshot.domains.find((d) => d.repo_root === focusedRun?.repo_root)?.domain_id ??
      null,
  );
</script>

{#if view === "compose"}
  <FlowScreen
    {backend}
    domains={snapshot.domains}
    {preferredDomainId}
    {onAddWorkspace}
    onDispatched={(runId) => onOpenMission(runId, "live")}
  />
{:else if view === "detail"}
  <MissionDetail
    {pane}
    run={focusedRun}
    domainId={detailDomainId}
    {onPane}
    onBack={() => onView("list")}
    {onRunCompleted}
    {onStopRun}
  />
{:else}
  <section class="screen collection-screen">
    <header class="screen-heading">
      <div>
        <h1>Missions</h1>
      </div>
      <button class="primary" onclick={() => onView("compose")}>New mission</button>
    </header>
    <article class="panel table-panel">
      <div class="panel-head"><h2>History</h2><span>{snapshot.runs.length}</span></div>
      <div class="data-table" role="table">
        <div class="table-row table-header" role="row">
          <span role="columnheader">Run</span>
          <span role="columnheader">Workspace</span>
          <span role="columnheader">Status</span>
          <span role="columnheader">Policy</span>
          <span role="columnheader">Estimate</span>
        </div>
        {#if snapshot.runs.length}
          {#each snapshot.runs as run (run.id)}
            <div
              class="table-row"
              role="row"
              tabindex="0"
              onclick={() => onOpenMission(run.id)}
              onkeydown={(e) => {
                if (e.key === "Enter" || e.key === " ") {
                  e.preventDefault();
                  onOpenMission(run.id);
                }
              }}
            >
              <span class="mono" role="cell">{run.id}</span>
              <span role="cell">{run.repo_root.split(/[\\/]/).pop()}</span>
              <span role="cell"><i class:active={run.status === "running"}></i>{run.status}</span>
              <span role="cell">{run.permission_profile}</span>
              <span role="cell">${(run.estimated_cost_usd ?? 0).toFixed(2)} <IconChevronRight size={15} /></span>
            </div>
          {/each}
        {:else}
          <div class="empty"><strong>No missions yet</strong><span>Write an outcome to build a plan.</span></div>
        {/if}
      </div>
    </article>
  </section>
{/if}
