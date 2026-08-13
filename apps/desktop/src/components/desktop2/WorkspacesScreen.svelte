<script lang="ts">
  import {
    IconChevronRight,
    IconFolder,
    IconLoader2,
    IconPlus,
    IconSettings,
  } from "@tabler/icons-svelte";
  import type { DesktopBackend, DesktopSnapshot } from "../../lib/desktop-backend";

  let {
    snapshot,
    backend,
    activeDomainId = null,
    onSelectDomain,
    onWorkspaceOpened,
    onDomainForgotten,
    onEditWorkspace,
    onNewMission,
  }: {
    snapshot: DesktopSnapshot;
    backend: DesktopBackend;
    activeDomainId?: string | null;
    onSelectDomain: (domainId: string, opts?: { route?: "operations" }) => void;
    onWorkspaceOpened: (openedPath?: string | null) => void | Promise<void>;
    onDomainForgotten: (domainId: string) => void;
    onEditWorkspace: (domainId: string) => void;
    onNewMission: () => void;
  } = $props();

  let query = $state("");
  let healthFilter = $state<"all" | "active">("all");
  let openedWorkspace = $state<string | null>(null);
  let creatingExample = $state(false);
  let showCleanup = $state(false);
  let forgettingId = $state<string | null>(null);
  let forgetError = $state<string | null>(null);

  const availableDomains = $derived(snapshot.domains.filter((d) => d.is_available && !d.is_temporary));
  const staleDomains = $derived(snapshot.domains.filter((d) => !d.is_available || d.is_temporary));
  const filteredDomains = $derived(
    availableDomains
      .filter((d) => d.repo_root.toLowerCase().includes(query.toLowerCase()))
      .filter((d) => (healthFilter === "active" ? d.active_runs > 0 : true)),
  );

  function profileHint(repoRoot: string): string {
    const run = snapshot.runs.find((r) => r.repo_root === repoRoot);
    return run?.permission_profile ?? "—";
  }

  async function openWorkspace() {
    try {
      openedWorkspace = await backend.openWorkspace();
      if (openedWorkspace) await onWorkspaceOpened(openedWorkspace);
    } catch (e) {
      forgetError = e instanceof Error ? e.message : String(e);
    }
  }

  async function forgetDomain(domainId: string) {
    forgettingId = domainId;
    forgetError = null;
    try {
      await backend.forgetDomain(domainId);
      onDomainForgotten(domainId);
    } catch (e) {
      forgetError = e instanceof Error ? e.message : String(e);
    } finally {
      forgettingId = null;
    }
  }

  async function createGuidedExample() {
    creatingExample = true;
    forgetError = null;
    try {
      openedWorkspace = await backend.createExampleWorkspace();
      await onWorkspaceOpened(openedWorkspace);
      onNewMission();
    } catch (error) {
      forgetError = error instanceof Error ? error.message : String(error);
    } finally {
      creatingExample = false;
    }
  }
</script>

<section class="screen collection-screen">
  <header class="screen-heading">
    <div>
      <h1>Workspaces</h1>
    </div>
    <div class="workspace-heading-actions">
      <button class="quiet" disabled={creatingExample} onclick={() => void createGuidedExample()}>
        {#if creatingExample}<IconLoader2 size={14} class="spin" />{/if}
        {creatingExample ? "Creating example" : "Try guided example"}
      </button>
      <button class="primary" onclick={openWorkspace}><IconPlus size={16} /> Add workspace</button>
    </div>
  </header>

  <div class="toolbar">
    <label><IconFolder size={16} /><input bind:value={query} placeholder="Search workspaces" aria-label="Search workspaces" /></label>
    <button class="quiet" onclick={() => (healthFilter = healthFilter === "all" ? "active" : "all")}>
      {healthFilter === "all" ? "All" : "Running only"}
    </button>
  </div>
  {#if openedWorkspace}<p class="workspace-opened">Added {openedWorkspace}</p>{/if}
  {#if forgetError}<p class="voice-state-message error" role="alert">{forgetError}</p>{/if}

  {#if filteredDomains.length}
    <div class="workspace-table data-table" role="table">
      <div class="table-row table-header" role="row">
        <span role="columnheader">Name</span>
        <span role="columnheader">Path</span>
        <span role="columnheader">Policy</span>
        <span role="columnheader">Running</span>
        <span role="columnheader">Approvals</span>
        <span role="columnheader"> </span>
      </div>
      {#each filteredDomains as domain (domain.domain_id)}
        <div class="table-row" class:active={domain.domain_id === activeDomainId} role="row">
          <span role="cell">
            <strong>{domain.repo_root.split(/[\\/]/).pop()}</strong>
            {#if domain.domain_id === activeDomainId}<small>Current</small>{/if}
          </span>
          <span class="mono" role="cell">{domain.repo_root}</span>
          <span role="cell">{profileHint(domain.repo_root)}</span>
          <span data-col="running" role="cell">{domain.active_runs}</span>
          <span data-col="approvals" role="cell">{domain.hitl_pending}</span>
          <span class="row-actions" role="cell">
            <button class="card-link" onclick={() => onSelectDomain(domain.domain_id, { route: "operations" })}>
              Open <IconChevronRight size={14} />
            </button>
            <button class="card-link" onclick={() => onEditWorkspace(domain.domain_id)}>
              <IconSettings size={14} /> Settings
            </button>
          </span>
        </div>
      {/each}
    </div>
  {:else if snapshot.error}
    <div class="empty"><IconFolder size={26} /><strong>Local service offline</strong><span>{snapshot.error.message}</span></div>
  {:else}
    <div class="empty">
      <IconFolder size={26} />
      <strong>No workspaces {query || healthFilter === "active" ? "match this filter" : "yet"}</strong>
      <span>{query || healthFilter === "active" ? "Try clearing the search or running filter." : "Add a workspace to start a mission."}</span>
    </div>
  {/if}

  {#if staleDomains.length}
    <button class="cleanup-toggle" onclick={() => (showCleanup = !showCleanup)} aria-expanded={showCleanup}>
      {showCleanup ? "Hide" : "Show"} {staleDomains.length} stale or missing workspace{staleDomains.length === 1 ? "" : "s"}
    </button>
    {#if showCleanup}
      <div class="cleanup-list">
        {#each staleDomains as domain}
          <div class="cleanup-row">
            <p><strong>{domain.repo_root.split(/[\\/]/).pop()}</strong><small>{domain.repo_root} · {domain.is_temporary ? "temporary test artifact" : "path not found on disk"}</small></p>
            <button class="quiet" disabled={forgettingId === domain.domain_id || domain.active_runs > 0 || domain.hitl_pending > 0} onclick={() => forgetDomain(domain.domain_id)}>
              {forgettingId === domain.domain_id ? "Removing…" : domain.active_runs > 0 ? "Has active runs" : domain.hitl_pending > 0 ? "Has approvals" : "Forget"}
            </button>
          </div>
        {/each}
      </div>
    {/if}
  {/if}
</section>
