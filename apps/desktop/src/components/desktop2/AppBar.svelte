<script lang="ts">
  import { IconChevronDown, IconFolder, IconPlus, IconSettings } from "@tabler/icons-svelte";
  import { ROUTE_LABELS, type AppRoute } from "../../lib/navigation.svelte";
  import type { CatalogEntryStatus } from "../../lib/types";

  let {
    route,
    approvalsCount,
    hypervisorOnline,
    activeDomainLabel = null,
    activeDomainId = null,
    domains = [],
    runningCount = 0,
    spendUsd = 0,
    onOpenHistory,
    onOpenNotifications,
    onSelectDomain,
    onAddWorkspace,
    onOpenWorkspaceSettings,
  }: {
    route: AppRoute;
    approvalsCount: number;
    hypervisorOnline: boolean;
    activeDomainLabel?: string | null;
    activeDomainId?: string | null;
    domains?: CatalogEntryStatus[];
    runningCount?: number;
    spendUsd?: number;
    onOpenHistory: () => void;
    onOpenNotifications: () => void;
    onSelectDomain: (domainId: string) => void;
    onAddWorkspace: () => void;
    onOpenWorkspaceSettings: (domainId: string) => void;
  } = $props();

  let open = $state(false);

  const available = $derived(domains.filter((d) => d.is_available && !d.is_temporary));
  const runningLabel = $derived(`${runningCount} running`);
  const approvalsLabel = $derived(`${approvalsCount} ${approvalsCount === 1 ? "approval" : "approvals"}`);

  function close() {
    open = false;
  }

  function pick(domainId: string) {
    onSelectDomain(domainId);
    close();
  }
</script>

<div class="app-bar">
  <div class="left">
    <div class="breadcrumbs">
      <span>Pytxo</span><i>/</i>
      <span class="workspace-crumb">{activeDomainLabel ?? "No workspace"}</span>
      <i>/</i>
      <strong>{ROUTE_LABELS[route]}</strong>
    </div>
    <div class="workspace-switcher">
      <button
        class="switcher"
        aria-haspopup="listbox"
        aria-expanded={open}
        onclick={() => (open = !open)}
        title="Switch workspace"
      >
        <IconFolder size={14} />
        <span>{activeDomainLabel ?? "No workspace"}</span>
        <IconChevronDown size={14} />
      </button>
      {#if open}
        <button
          type="button"
          class="backdrop"
          aria-label="Close workspace switcher"
          onclick={close}
        ></button>
        <div class="menu" role="listbox" aria-label="Workspaces">
          {#each available as domain (domain.domain_id)}
            <button
              role="option"
              aria-selected={domain.domain_id === activeDomainId}
              class:active={domain.domain_id === activeDomainId}
              onclick={() => pick(domain.domain_id)}
            >
              <strong>{domain.repo_root.split(/[\\/]/).pop()}</strong>
              <small class="mono">{domain.repo_root}</small>
            </button>
          {/each}
          {#if !available.length}
            <p class="empty">No workspaces yet</p>
          {/if}
          <div class="menu-actions">
            <button onclick={() => { close(); onAddWorkspace(); }}><IconPlus size={14} /> Add workspace</button>
            {#if activeDomainId}
              <button onclick={() => { close(); onOpenWorkspaceSettings(activeDomainId); }}>
                <IconSettings size={14} /> Workspace settings
              </button>
            {/if}
          </div>
        </div>
      {/if}
    </div>
  </div>
  <div class="app-actions">
    <div class="ops-summary" role="group" aria-label="Operations summary">
      <button type="button" onclick={onOpenHistory} aria-label={runningLabel} title="Open active runs">
        <span>Running</span><strong class="tabular">{runningCount}</strong>
      </button>
      <button type="button" class:needs={approvalsCount > 0} onclick={onOpenNotifications} aria-label={approvalsLabel} title="Open approvals">
        <span>Approvals</span><strong class="tabular">{approvalsCount}</strong>
      </button>
      <span class="cost" title="Estimated spend"><span>Cost</span><strong class="tabular">${spendUsd.toFixed(2)}</strong></span>
    </div>
    {#if !hypervisorOnline}<span class="service-state" role="status">Offline</span>{/if}
  </div>
</div>

<style>
  .app-bar {
    position: relative;
    height: 47px;
    flex: none;
    display: flex;
    align-items: center;
    justify-content: space-between;
    border-bottom: 1px solid var(--pytxo-line, #1e2026);
    padding: 0 22px;
    gap: 12px;
  }
  .left {
    display: flex;
    align-items: center;
    gap: 14px;
    min-width: 0;
  }
  .breadcrumbs {
    display: flex;
    gap: 8px;
    align-items: center;
    font-size: 11px;
    color: #7d8591;
    min-width: 0;
  }
  .breadcrumbs i {
    font-style: normal;
    color: #343842;
  }
  .breadcrumbs strong {
    color: #b4bac3;
    font-weight: 550;
  }
  .workspace-crumb {
    color: #8b929d;
    max-width: 140px;
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }
  .workspace-switcher {
    position: relative;
  }
  .switcher {
    display: inline-flex;
    align-items: center;
    gap: 6px;
    max-width: 220px;
    border: 1px solid var(--pytxo-line, #1e2026);
    background: #0c0e13;
    color: #c5cad1;
    border-radius: 6px;
    padding: 5px 8px;
    font-size: 11.5px;
    cursor: pointer;
  }
  .switcher span {
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }
  .switcher:hover {
    border-color: color-mix(in oklab, var(--pytxo-accent, var(--pytxo-teal)) 35%, #1e2026);
  }
  .backdrop {
    position: fixed;
    inset: 0;
    z-index: 15;
    padding: 0;
    border: 0;
    background: transparent;
  }
  .menu {
    position: absolute;
    top: calc(100% + 6px);
    left: 0;
    z-index: 16;
    width: min(340px, 70vw);
    border: 1px solid var(--pytxo-line, #1e2026);
    border-radius: 10px;
    background: #0c0e13;
    box-shadow: 0 18px 40px -20px rgba(0, 0, 0, 0.7);
    padding: 6px;
    display: flex;
    flex-direction: column;
    gap: 2px;
  }
  .menu > button {
    display: flex;
    flex-direction: column;
    align-items: flex-start;
    gap: 2px;
    border: 0;
    background: transparent;
    color: #c5cad1;
    padding: 8px 10px;
    border-radius: 7px;
    cursor: pointer;
    text-align: left;
  }
  .menu > button:hover,
  .menu > button.active {
    background: color-mix(in oklab, var(--pytxo-accent, var(--pytxo-teal)) 12%, #12141a);
  }
  .menu strong {
    font-size: 12.5px;
    font-weight: 550;
  }
  .menu small {
    color: #7d8591;
    font-size: 10.5px;
  }
  .menu .empty {
    margin: 0;
    padding: 10px;
    color: #7d8591;
    font-size: 12px;
  }
  .menu-actions {
    border-top: 1px solid var(--pytxo-line, #1e2026);
    margin-top: 4px;
    padding-top: 4px;
    display: flex;
    flex-direction: column;
    gap: 2px;
  }
  .menu-actions button {
    display: flex;
    align-items: center;
    gap: 8px;
    border: 0;
    background: transparent;
    color: #8b929c;
    padding: 8px 10px;
    border-radius: 7px;
    cursor: pointer;
    font-size: 12px;
  }
  .menu-actions button:hover {
    background: #12141a;
    color: #d7dbe1;
  }
  .app-actions {
    display: flex;
    align-items: center;
    gap: 6px;
  }
  .ops-summary {
    display: flex;
    align-items: center;
    gap: 14px;
  }
  .ops-summary button,
  .ops-summary .cost {
    display: flex;
    align-items: baseline;
    gap: 5px;
    padding: 5px 0;
    border: 0;
    background: transparent;
    color: inherit;
    cursor: pointer;
    font: inherit;
  }
  .ops-summary .cost {
    cursor: default;
  }
  .ops-summary button:hover span,
  .ops-summary button:hover strong {
    color: var(--pytxo-text-strong, #e1e5ea);
  }
  .ops-summary button.needs strong {
    color: var(--pytxo-gold, #eeac47);
  }
  .ops-summary span {
    color: var(--pytxo-text-muted, #5d6470);
    font-size: 10px;
    font-weight: 520;
  }
  .ops-summary strong {
    color: var(--pytxo-text-body, #d7dbe0);
    font-size: 11px;
    font-weight: 620;
  }
  .tabular {
    font-variant-numeric: tabular-nums;
    font-family: "Geist Mono", ui-monospace, monospace;
  }
  .service-state {
    color: var(--pytxo-danger, #d98994);
    font-size: 11px;
  }
  .ops-summary button:focus-visible {
    outline: 2px solid var(--pytxo-accent, var(--pytxo-teal));
    outline-offset: 1px;
  }
  @media (max-width: 1050px) {
    .ops-summary {
      gap: 9px;
    }
    .ops-summary button,
    .ops-summary .cost {
      gap: 3px;
    }
  }
  .mono {
    font-family: "Geist Mono", ui-monospace, monospace;
  }
</style>
