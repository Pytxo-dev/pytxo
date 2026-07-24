<script lang="ts">
  import { IconBell, IconChevronDown, IconFolder, IconHistory, IconPlus, IconSettings } from "@tabler/icons-svelte";
  import { ROUTE_LABELS, type AppRoute } from "../../lib/navigation.svelte";
  import type { CatalogEntryStatus } from "../../lib/types";

  let {
    route,
    approvalsCount,
    hypervisorOnline,
    activeDomainLabel = null,
    activeDomainId = null,
    domains = [],
    live = false,
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
    live?: boolean;
    onOpenHistory: () => void;
    onOpenNotifications: () => void;
    onSelectDomain: (domainId: string) => void;
    onAddWorkspace: () => void;
    onOpenWorkspaceSettings: (domainId: string) => void;
  } = $props();

  let open = $state(false);

  const available = $derived(domains.filter((d) => d.is_available && !d.is_temporary));

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
        <!-- svelte-ignore a11y_no_static_element_interactions -->
        <div class="backdrop" onclick={close}></div>
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
    <span class="connection" class:offline={!hypervisorOnline} class:live={live && hypervisorOnline}>
      <i></i> {hypervisorOnline ? (live ? "Live" : "Local hypervisor") : "Hypervisor unreachable"}
    </span>
    <button aria-label="Run history" title="Run history" onclick={onOpenHistory}>
      <IconHistory size={17} />
    </button>
    <button aria-label="Notifications" title="Approvals" onclick={onOpenNotifications}>
      <IconBell size={17} />
      {#if approvalsCount}<i class="badge"></i>{/if}
    </button>
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
  .app-bar::after {
    content: "";
    position: absolute;
    left: 0;
    right: 0;
    bottom: 0;
    height: 1px;
    background: var(--pytxo-hairline);
    pointer-events: none;
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
    color: #5e6571;
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
    color: #6f7784;
    font-size: 10.5px;
  }
  .menu .empty {
    margin: 0;
    padding: 10px;
    color: #6f7784;
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
  .connection {
    display: flex;
    align-items: center;
    gap: 6px;
    margin-right: 8px;
    color: #717885;
    font-size: 10px;
  }
  .connection i {
    width: 6px;
    height: 6px;
    border-radius: 50%;
    background: #3ed7a1;
    box-shadow: 0 0 0 3px rgba(62, 215, 161, 0.08);
  }
  .connection.live i {
    background: var(--pytxo-accent, var(--pytxo-teal));
  }
  .connection.offline {
    color: #d98994;
  }
  .connection.offline i {
    background: #df6576;
    box-shadow: 0 0 0 3px rgba(223, 101, 118, 0.1);
  }
  .app-actions button {
    position: relative;
    width: 30px;
    height: 30px;
    border-radius: 5px;
    border: 0;
    background: none;
    color: #707783;
    cursor: pointer;
    display: flex;
    align-items: center;
    justify-content: center;
  }
  .app-actions button:hover {
    background: #14161c;
    color: #ccd1d8;
  }
  .app-actions button:focus-visible {
    outline: 2px solid var(--pytxo-accent, var(--pytxo-teal));
    outline-offset: 1px;
  }
  .app-actions button .badge {
    position: absolute;
    right: 5px;
    top: 5px;
    width: 5px;
    height: 5px;
    border-radius: 50%;
    background: var(--pytxo-gold, #eeac47);
  }
  .mono {
    font-family: "Geist Mono", ui-monospace, monospace;
  }
</style>
