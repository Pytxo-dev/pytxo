<script lang="ts">
  import { onMount } from "svelte";
  import { IconCheck, IconChevronRight, IconCloud, IconFolder, IconGitBranch, IconLoader2, IconPlugConnected, IconPlus, IconSearch, IconSettings, IconShieldLock, IconTerminal2 } from "@tabler/icons-svelte";
  import type { DesktopBackend, DesktopSnapshot } from "../../lib/desktop-backend";
  import type { AppRoute } from "../../lib/navigation.svelte";
  import type { AdeCliStatusDto, HitlDto } from "../../lib/types";

  let {
    route,
    snapshot,
    backend,
    onRoute,
    activeDomainId = null,
    onReviewRun,
    onViewTopology,
    onSelectDomain,
    onWorkspaceOpened,
    onDomainForgotten,
    onApprovalsChanged,
    onEditWorkspace,
  }: {
    route: AppRoute;
    snapshot: DesktopSnapshot;
    backend: DesktopBackend;
    onRoute: (route: AppRoute) => void;
    activeDomainId?: string | null;
    onReviewRun: (runId: string) => void;
    onViewTopology: (domainId: string) => void;
    onSelectDomain: (domainId: string, opts?: { route?: AppRoute }) => void;
    onWorkspaceOpened: (openedPath?: string | null) => void | Promise<void>;
    onDomainForgotten: (domainId: string) => void;
    onApprovalsChanged: () => void | Promise<void>;
    onEditWorkspace: (domainId: string) => void;
  } = $props();

  let query = $state("");
  let healthFilter = $state<"all" | "active">("all");
  let selectedApprovalId = $state<string | null>(null);
  let openedWorkspace = $state<string | null>(null);
  let adeClis = $state<AdeCliStatusDto[]>([]);
  let adeLoading = $state(true);
  let showCleanup = $state(false);
  let forgettingId = $state<string | null>(null);
  let forgetError = $state<string | null>(null);
  let deciding = $state(false);
  let decisionMessage = $state<string | null>(null);

  const availableDomains = $derived(snapshot.domains.filter((d) => d.is_available && !d.is_temporary));
  const staleDomains = $derived(snapshot.domains.filter((d) => !d.is_available || d.is_temporary));
  const filteredDomains = $derived(
    availableDomains
      .filter((d) => d.repo_root.toLowerCase().includes(query.toLowerCase()))
      .filter((d) => (healthFilter === "active" ? d.active_runs > 0 : true)),
  );
  const openApprovals = $derived(snapshot.approvals);
  const selectedApproval: HitlDto | null = $derived(
    openApprovals.find((a) => a.id === selectedApprovalId) ?? openApprovals[0] ?? null,
  );

  function profileHint(domainId: string, repoRoot: string): string {
    const run = snapshot.runs.find((r) => r.repo_root === repoRoot);
    return run?.permission_profile ?? "—";
  }

  async function resolveApproval(approve: boolean) {
    if (!selectedApproval || deciding) return;
    deciding = true;
    decisionMessage = null;
    try {
      if (approve) await backend.approve(selectedApproval.id, selectedApproval.domain_id ?? null);
      else await backend.deny(selectedApproval.id, selectedApproval.domain_id ?? null);
      decisionMessage = approve
        ? "Approved. Sandbox flush proceeds under the workspace permission profile."
        : "Denied. Isolated changes were not flushed.";
      selectedApprovalId = null;
      await onApprovalsChanged();
    } finally {
      deciding = false;
    }
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

  $effect(() => {
    if (route === "approvals" && openApprovals.length && !openApprovals.some((a) => a.id === selectedApprovalId)) {
      selectedApprovalId = openApprovals[0]?.id ?? null;
    }
  });

  onMount(async () => {
    try {
      adeClis = await backend.listAdeClis();
    } finally {
      adeLoading = false;
    }
  });
</script>

<section class="screen collection-screen">
  <header class="screen-heading">
    <div>
      <h1>{route === "run-review" ? "Run Review" : route[0].toUpperCase() + route.slice(1)}</h1>
      <p>
        {route === "workspaces"
          ? "Workspaces, roots, and health in one catalog."
          : route === "runs"
            ? "Every orchestration run, from dispatch through recovery."
            : route === "approvals"
              ? "Human decisions with the context needed to act confidently."
              : "Agent CLIs and how to connect the MCP hub."}
      </p>
    </div>
    {#if route === "workspaces"}<button class="primary" onclick={openWorkspace}><IconPlus size={16} /> Add workspace</button>{/if}
  </header>

  {#if route === "workspaces"}
    <div class="toolbar">
      <label><IconSearch size={16} /><input bind:value={query} placeholder="Search workspaces" aria-label="Search workspaces" /></label>
      <button class="quiet" onclick={() => (healthFilter = healthFilter === "all" ? "active" : "all")}>{healthFilter === "all" ? "Health: all" : "Health: active only"}</button>
    </div>
    {#if openedWorkspace}<p class="workspace-opened">Added {openedWorkspace}</p>{/if}
    {#if forgetError}<p class="voice-state-message error" role="alert">{forgetError}</p>{/if}
    {#if filteredDomains.length}
      <div class="catalog-grid">
        {#each filteredDomains as domain}
          <article class="workspace-card" class:active={domain.domain_id === activeDomainId}>
            <div class="workspace-icon"><IconFolder size={19} /></div>
            <div class="workspace-title">
              <h2>{domain.repo_root.split(/[\\/]/).pop()}</h2>
              <span class="healthy"><i></i>{domain.status}{#if domain.domain_id === activeDomainId} · active{/if}</span>
            </div>
            <p>{domain.repo_root}</p>
            <div class="workspace-stats"><span><small>Active runs</small><strong>{domain.active_runs}</strong></span><span><small>Approvals</small><strong>{domain.hitl_pending}</strong></span><span><small>Policy</small><strong>{profileHint(domain.domain_id, domain.repo_root)}</strong></span></div>
            <div class="card-actions">
              <button class="card-link" onclick={() => onSelectDomain(domain.domain_id, { route: "operations" })}>Open <IconChevronRight size={15} /></button>
              <button class="card-link" onclick={() => onViewTopology(domain.domain_id)}>Structure <IconGitBranch size={15} /></button>
              <button class="card-link" onclick={() => onEditWorkspace(domain.domain_id)}><IconSettings size={14} /> Settings</button>
            </div>
          </article>
        {/each}
      </div>
    {:else if snapshot.error}
      <div class="empty"><IconFolder size={26} /><strong>Local service offline</strong><span>{snapshot.error.message}</span></div>
    {:else}
      <div class="empty"><IconFolder size={26} /><strong>No workspaces {query || healthFilter === "active" ? "match this filter" : "yet"}</strong><span>{query || healthFilter === "active" ? "Try clearing the search or health filter." : "Add a workspace to start dispatching runs."}</span></div>
    {/if}
    {#if staleDomains.length}
      <button class="cleanup-toggle" onclick={() => (showCleanup = !showCleanup)} aria-expanded={showCleanup}>
        <IconChevronRight size={12} />
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
        {#if forgetError}<p class="voice-state-message error">{forgetError}</p>{/if}
      {/if}
    {/if}
  {:else if route === "runs"}
    <article class="panel table-panel">
      <div class="panel-head"><h2>Run history</h2><span>{snapshot.runs.length} run{snapshot.runs.length === 1 ? "" : "s"}</span></div>
      <div class="data-table" role="table">
        <div class="table-row table-header" role="row"><span role="columnheader">Run</span><span role="columnheader">Workspace</span><span role="columnheader">Status</span><span role="columnheader">Policy</span><span role="columnheader">Estimate</span></div>
        {#if snapshot.runs.length}
          {#each snapshot.runs as run}
            <div class="table-row" role="row" tabindex="0" onclick={() => onReviewRun(run.id)} onkeydown={(e) => { if (e.key === "Enter" || e.key === " ") { e.preventDefault(); onReviewRun(run.id); } }}><span class="mono" role="cell">{run.id}</span><span role="cell">{run.repo_root.split(/[\\/]/).pop()}</span><span role="cell"><i class:active={run.status === "running"}></i>{run.status}</span><span role="cell">{run.permission_profile}</span><span role="cell">${(run.estimated_cost_usd ?? 0).toFixed(2)} <IconChevronRight size={15} /></span></div>
          {/each}
        {:else}
          <div class="empty"><strong>No runs yet</strong><span>Dispatch a Flow to populate run history.</span></div>
        {/if}
      </div>
    </article>
  {:else if route === "approvals"}
    <div class="approval-layout">
      <article class="panel inbox">
        <div class="panel-head"><h2>Inbox</h2><span>{openApprovals.length} open</span></div>
        {#if openApprovals.length}
          {#each openApprovals as approval}
            <button class="inbox-row" class:active={selectedApproval?.id === approval.id} onclick={() => (selectedApprovalId = approval.id)}>
              <span class="risk">Review</span>
              <strong>{approval.action}</strong>
              <small>{approval.agent_key} · {new Date(Number(approval.created_at_ms)).toLocaleString()}</small>
              <IconChevronRight size={16} />
            </button>
          {/each}
        {:else}
          <div class="empty"><strong>Inbox clear</strong><span>New approval requests appear here when agents wait on you.</span></div>
        {/if}
      </article>
      <article class="panel decision-detail">
        <h2>{selectedApproval?.action ?? "Inbox clear"}</h2>
        <p>{selectedApproval?.reason ?? "All decisions have been resolved."}</p>
        {#if selectedApproval}
          <div class="diff-summary">
            <span><IconShieldLock size={17} /> Sandbox</span>
            <strong>Approve to flush · Deny to discard</strong>
            <small>Pytxo applies this workspace permission profile when you flush. Open Run Review for file-level evidence.</small>
          </div>
          {#if decisionMessage}<p class="voice-state-message">{decisionMessage}</p>{/if}
          <div class="decision-actions">
            <button class="deny" disabled={deciding} onclick={() => resolveApproval(false)}>Deny</button>
            <button class="primary" disabled={deciding} onclick={() => resolveApproval(true)}><IconCheck size={16} /> Approve & flush</button>
          </div>
        {/if}
      </article>
    </div>
  {:else if route === "integrations"}
    <div class="integration-sections">
      <article class="panel">
        <div class="panel-head"><div><h2>Agent CLIs</h2></div></div>
        {#if adeLoading}
          <div class="empty"><IconLoader2 size={22} class="spin" /><strong>Checking installed CLIs…</strong></div>
        {:else}
          <div class="integration-grid">
            {#each adeClis as cli}
              <div class="integration-card"><div class="provider-icon"><IconTerminal2 size={19} /></div><p><strong>{cli.display_name}</strong><small class="mono">{cli.default_cmd}</small></p><span class:healthy={cli.installed}>{cli.installed ? "Installed" : "Not installed"}</span></div>
            {/each}
          </div>
        {/if}
      </article>
      <article class="panel integration-truth">
        <div class="integration-card wide">
          <div class="provider-icon"><IconPlugConnected size={20} /></div>
          <div>
            <p><strong>MCP hub</strong><small>Configure in your IDE via the <code class="mono">pytxo-mcp</code> stdio server — not an in-app connection panel.</small></p>
            <p class="setup-hint mono">npx pytxo-mcp · Cursor MCP settings · docs: MCP from Cursor</p>
          </div>
        </div>
        <div class="integration-card wide">
          <div class="provider-icon"><IconCloud size={20} /></div>
          <div>
            <p><strong>Cloud sandboxes</strong><small>Local execution only until a cloud dispatcher is configured. No fake cloud toggle here.</small></p>
          </div>
        </div>
      </article>
    </div>
  {/if}
</section>
