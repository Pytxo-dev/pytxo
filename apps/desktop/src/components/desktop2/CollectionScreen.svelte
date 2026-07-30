<script lang="ts">
  import { onMount } from "svelte";
  import {
    IconAlertCircle,
    IconCheck,
    IconChevronRight,
    IconCircleCheck,
    IconCloud,
    IconExternalLink,
    IconFolder,
    IconGitBranch,
    IconLoader2,
    IconLogin2,
    IconPlugConnected,
    IconPlus,
    IconRefresh,
    IconSearch,
    IconSettings,
    IconShieldLock,
    IconSparkles,
    IconTerminal2,
  } from "@tabler/icons-svelte";
  import { approvalPresentation } from "../../lib/approval-presentation";
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
  let creatingExample = $state(false);
  let adeClis = $state<AdeCliStatusDto[]>([]);
  let adeLoading = $state(true);
  let adeMessage = $state<string | null>(null);
  let adeMessageTone = $state<"success" | "error" | null>(null);
  let loginOpeningId = $state<string | null>(null);
  let showCleanup = $state(false);
  let forgettingId = $state<string | null>(null);
  let forgetError = $state<string | null>(null);
  let deciding = $state(false);
  let decisionMessage = $state<string | null>(null);
  let decisionMessageTone = $state<"success" | "error" | null>(null);

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
  const selectedPresentation = $derived(
    selectedApproval ? approvalPresentation(selectedApproval) : null,
  );
  const selectedIndex = $derived(
    selectedApproval ? openApprovals.findIndex((approval) => approval.id === selectedApproval.id) : -1,
  );
  const selectedDomain = $derived(
    selectedApproval?.domain_id
      ? snapshot.domains.find((domain) => domain.domain_id === selectedApproval.domain_id) ?? null
      : null,
  );
  const selectedRun = $derived(
    selectedDomain
      ? snapshot.runs.find((run) => run.repo_root === selectedDomain.repo_root) ?? null
      : null,
  );
  const sortedAdeClis = $derived(
    [...adeClis].sort((a, b) => {
      const priority = ["codex", "claude", "cursor", "opencode", "gemini", "copilot", "aider", "agy"];
      return priority.indexOf(a.id) - priority.indexOf(b.id);
    }),
  );
  const connectedAdeCount = $derived(
    adeClis.filter((cli) => cli.installed && cli.auth_state === "signed_in").length,
  );

  function workspaceLabel(): string {
    return selectedDomain?.repo_root.split(/[\\/]/).pop() ?? selectedApproval?.domain_id ?? "Unknown";
  }

  function requestedAt(approval: HitlDto): string {
    const timestamp = Number(approval.created_at_ms);
    return Number.isFinite(timestamp) ? new Date(timestamp).toLocaleString() : "Unknown";
  }

  function isEditableTarget(target: EventTarget | null): boolean {
    if (!(target instanceof HTMLElement)) return false;
    return target.isContentEditable || ["INPUT", "TEXTAREA", "SELECT"].includes(target.tagName);
  }

  function selectRelativeApproval(offset: number) {
    if (!openApprovals.length || deciding) return;
    const current = selectedIndex >= 0 ? selectedIndex : 0;
    const next = (current + offset + openApprovals.length) % openApprovals.length;
    selectedApprovalId = openApprovals[next]?.id ?? null;
  }

  function onApprovalKeydown(event: KeyboardEvent) {
    if (route !== "approvals" || !selectedApproval || event.defaultPrevented || event.repeat || isEditableTarget(event.target)) return;
    const modifier = event.ctrlKey || event.metaKey;
    if (!modifier && !event.altKey && !event.shiftKey && event.key.toLowerCase() === "j") {
      event.preventDefault();
      selectRelativeApproval(1);
      return;
    }
    if (!modifier && !event.altKey && !event.shiftKey && event.key.toLowerCase() === "k") {
      event.preventDefault();
      selectRelativeApproval(-1);
      return;
    }
    if (modifier && !event.altKey && !event.shiftKey && event.key === "Enter") {
      event.preventDefault();
      void resolveApproval(true);
      return;
    }
    if (modifier && !event.altKey && !event.shiftKey && event.key === "Backspace") {
      event.preventDefault();
      void resolveApproval(false);
    }
  }

  function profileHint(domainId: string, repoRoot: string): string {
    const run = snapshot.runs.find((r) => r.repo_root === repoRoot);
    return run?.permission_profile ?? "—";
  }

  async function resolveApproval(approve: boolean) {
    if (!selectedApproval || deciding) return;
    const resolved = selectedApproval;
    const presentation = approvalPresentation(resolved);
    deciding = true;
    decisionMessage = null;
    decisionMessageTone = null;
    try {
      if (approve) await backend.approve(resolved.id, resolved.domain_id ?? null);
      else await backend.deny(resolved.id, resolved.domain_id ?? null);
      decisionMessage = `${approve ? "Approved" : "Denied"} ${presentation.title}. ${approve ? presentation.approvedMessage : presentation.deniedMessage}`;
      decisionMessageTone = "success";
      selectedApprovalId = openApprovals.find((approval) => approval.id !== resolved.id)?.id ?? null;
      await onApprovalsChanged();
    } catch (error) {
      decisionMessage = `Could not ${approve ? "approve" : "deny"} ${presentation.title}. ${error instanceof Error ? error.message : String(error)}`;
      decisionMessageTone = "error";
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

  async function createGuidedExample() {
    creatingExample = true;
    forgetError = null;
    try {
      openedWorkspace = await backend.createExampleWorkspace();
      await onWorkspaceOpened(openedWorkspace);
      onRoute("flow");
    } catch (error) {
      forgetError = error instanceof Error ? error.message : String(error);
    } finally {
      creatingExample = false;
    }
  }

  async function refreshAdeClis() {
    adeLoading = true;
    adeMessage = null;
    adeMessageTone = null;
    try {
      adeClis = await backend.listAdeClis();
    } catch (error) {
      adeClis = [];
      adeMessage = error instanceof Error ? error.message : String(error);
      adeMessageTone = "error";
    } finally {
      adeLoading = false;
    }
  }

  async function openAdeLogin(cli: AdeCliStatusDto) {
    if (!cli.installed || !cli.login_supported || loginOpeningId) return;
    loginOpeningId = cli.id;
    adeMessage = null;
    adeMessageTone = null;
    try {
      const launched = await backend.startAdeLogin(cli.id);
      adeMessage = launched.message;
      adeMessageTone = "success";
    } catch (error) {
      adeMessage = error instanceof Error ? error.message : String(error);
      adeMessageTone = "error";
    } finally {
      loginOpeningId = null;
    }
  }

  $effect(() => {
    if (route === "approvals" && openApprovals.length && !openApprovals.some((a) => a.id === selectedApprovalId)) {
      selectedApprovalId = openApprovals[0]?.id ?? null;
    }
  });

  onMount(() => {
    window.addEventListener("keydown", onApprovalKeydown);
    void refreshAdeClis();
    return () => window.removeEventListener("keydown", onApprovalKeydown);
  });
</script>

<section class="screen collection-screen">
  <header class="screen-heading" class:approval-heading={route === "approvals"}>
    <div>
      <h1>{route === "run-review" ? "Run Review" : route[0].toUpperCase() + route.slice(1)}</h1>
      <p>
        {route === "workspaces"
          ? "Workspaces, roots, and health in one catalog."
          : route === "runs"
            ? "Every orchestration run, from dispatch through recovery."
            : route === "approvals"
              ? "Human decisions with the context needed to act confidently."
              : "Installed agent CLIs, vendor-owned sessions, and the MCP hub."}
      </p>
    </div>
    {#if route === "workspaces"}
      <div class="workspace-heading-actions">
        <button class="quiet" disabled={creatingExample} onclick={() => void createGuidedExample()}>
          {#if creatingExample}<IconLoader2 size={14} class="spin" />{:else}<IconSparkles size={14} />{/if}
          {creatingExample ? "Creating example" : "Try guided example"}
        </button>
        <button class="primary" onclick={openWorkspace}><IconPlus size={16} /> Add workspace</button>
      </div>
    {/if}
    {#if route === "integrations"}
      <button class="quiet integration-refresh" onclick={() => void refreshAdeClis()} disabled={adeLoading}>
        <IconRefresh size={14} class={adeLoading ? "spin" : undefined} />
        {adeLoading ? "Checking" : "Recheck all"}
      </button>
    {/if}
    {#if route === "approvals" && openApprovals.length}
      <div class="approval-shortcuts" aria-label="Approval keyboard shortcuts">
        <span><kbd>J</kbd><kbd>K</kbd> Select</span>
        <span><kbd>Ctrl/⌘</kbd><kbd>Enter</kbd> Approve</span>
        <span><kbd>Ctrl/⌘</kbd><kbd>Backspace</kbd> Deny</span>
      </div>
    {/if}
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
    {#if decisionMessage}
      <p
        class="voice-state-message decision-message"
        class:error={decisionMessageTone === "error"}
        role={decisionMessageTone === "error" ? "alert" : "status"}
      >{decisionMessage}</p>
    {/if}
    <div class="approval-layout">
      <article class="panel inbox">
        <div class="panel-head"><h2>Inbox</h2><span>{openApprovals.length} open</span></div>
        {#if openApprovals.length}
          {#each openApprovals as approval}
            {@const presentation = approvalPresentation(approval)}
            <button
              class="inbox-row"
              class:active={selectedApproval?.id === approval.id}
              aria-pressed={selectedApproval?.id === approval.id}
              onclick={() => (selectedApprovalId = approval.id)}
            >
              <span class="risk">{presentation.category}</span>
              <strong>{presentation.title}</strong>
              <small>{approval.agent_key} · {requestedAt(approval)}</small>
              <IconChevronRight size={16} />
            </button>
          {/each}
        {:else}
          <div class="empty"><strong>Inbox clear</strong><span>New approval requests appear here when agents wait on you.</span></div>
        {/if}
      </article>
      <article class="panel decision-detail">
        {#if selectedApproval && selectedPresentation}
          <div class="decision-title">
            <div>
              <span>{selectedPresentation.category}</span>
              <h2>{selectedPresentation.title}</h2>
            </div>
            <span class="selection-position" aria-live="polite">{selectedIndex + 1} of {openApprovals.length}</span>
          </div>
          <p class="decision-reason">{selectedApproval.reason}</p>
          <dl class="decision-meta">
            <div><dt>Requested by</dt><dd class="mono">{selectedApproval.agent_key}</dd></div>
            <div><dt>Workspace</dt><dd>{workspaceLabel()}</dd></div>
            <div><dt>Requested</dt><dd>{requestedAt(selectedApproval)}</dd></div>
            <div><dt>Action</dt><dd class="mono">{selectedApproval.action}</dd></div>
          </dl>
          <div class="diff-summary">
            <span><IconShieldLock size={17} /> Permission gate</span>
            <strong>{selectedPresentation.approveLabel} · {selectedPresentation.denyLabel}</strong>
            <small>{selectedPresentation.consequence}</small>
          </div>
          <div class="decision-evidence">
            <div>
              <strong>Latest run evidence</strong>
              {#if selectedRun}
                <small><span class="mono">{selectedRun.id}</span> · {selectedRun.status} · {selectedRun.permission_profile ?? "unknown"} profile</small>
                <small>This request does not include a run ID. Showing the latest run for {workspaceLabel()}.</small>
              {:else}
                <small>No run is available for this workspace. Decide from the exact action and reason above.</small>
              {/if}
            </div>
            {#if selectedRun}
              <button class="quiet" onclick={() => onReviewRun(selectedRun.id)}>Review latest run <IconChevronRight size={14} /></button>
            {/if}
          </div>
          <div class="decision-actions">
            <button class="deny" disabled={deciding} onclick={() => resolveApproval(false)}>{selectedPresentation.denyLabel}<kbd aria-hidden="true">Ctrl/⌘ ⌫</kbd></button>
            <button class="primary" disabled={deciding} onclick={() => resolveApproval(true)}><IconCheck size={16} /> {selectedPresentation.approveLabel}<kbd aria-hidden="true">Ctrl/⌘ ↵</kbd></button>
          </div>
        {:else}
          <h2>Inbox clear</h2>
          <p>All decisions have been resolved.</p>
        {/if}
      </article>
    </div>
  {:else if route === "integrations"}
    <div class="integration-sections">
      <article class="panel integration-agent-panel">
        <div class="panel-head">
          <div>
            <h2>Agent sessions</h2>
            <p>{connectedAdeCount} connected · credentials stay with each vendor CLI</p>
          </div>
          <span class="integration-safety"><IconShieldLock size={13} /> Pytxo never reads token stores</span>
        </div>
        <p class="integration-intro">
          Pytxo launches official sign-in commands from your home directory and reads only
          non-billable, redacted status. ChatGPT connects through Codex; Claude, Cursor, Gemini,
          Copilot, and OpenCode keep their own sessions.
        </p>
        {#if adeMessage}
          <p class="integration-message" class:error={adeMessageTone === "error"} role={adeMessageTone === "error" ? "alert" : "status"}>
            {#if adeMessageTone === "error"}<IconAlertCircle size={14} />{:else}<IconCircleCheck size={14} />{/if}
            {adeMessage}
          </p>
        {/if}
        {#if adeLoading}
          <div class="empty"><IconLoader2 size={22} class="spin" /><strong>Checking installed CLIs and sessions…</strong><span>No model request is sent.</span></div>
        {:else}
          <div class="integration-grid">
            {#each sortedAdeClis as cli (cli.id)}
              <article class="integration-card integration-card--session" class:connected={cli.auth_state === "signed_in"}>
                <div class="integration-card__top">
                  <div class="provider-icon"><IconTerminal2 size={19} /></div>
                  <div class="integration-card__title">
                    <strong>{cli.display_name}</strong>
                    <small class="mono">{cli.default_cmd}</small>
                  </div>
                  <span class="integration-state" class:healthy={cli.installed}>{cli.installed ? "Installed" : "Not installed"}</span>
                </div>
                <div class="integration-auth-state" data-state={cli.auth_state}>
                  {#if cli.auth_state === "signed_in"}<IconCircleCheck size={14} />
                  {:else if cli.auth_state === "signed_out"}<IconAlertCircle size={14} />
                  {:else}<IconTerminal2 size={14} />{/if}
                  <span>
                    <strong>{cli.auth_label}</strong>
                    <small>Owned by {cli.auth_owner}</small>
                  </span>
                </div>
                <p class="integration-detail">{cli.detail}</p>
                <div class="integration-actions">
                  <a href={cli.docs_url} target="_blank" rel="noopener noreferrer">
                    {cli.installed ? "Docs" : "Install guide"} <IconExternalLink size={12} />
                  </a>
                  {#if cli.installed && cli.auth_state === "signed_in"}
                    <button class="quiet" onclick={() => onRoute("flow")}>Use in Flow</button>
                  {:else if cli.installed && cli.login_supported && cli.login_label}
                    <button class="quiet" disabled={loginOpeningId !== null} onclick={() => void openAdeLogin(cli)}>
                      {#if loginOpeningId === cli.id}<IconLoader2 size={12} class="spin" />{:else}<IconLogin2 size={12} />{/if}
                      {cli.login_label}
                    </button>
                  {/if}
                </div>
              </article>
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
