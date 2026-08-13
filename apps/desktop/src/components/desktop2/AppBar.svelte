<script lang="ts">
  import { IconChevronDown, IconFolder, IconPlus, IconSettings } from "@tabler/icons-svelte";
  import { ROUTE_LABELS, type CanonicalRoute } from "../../lib/navigation.svelte";
  import type { CatalogEntryStatus } from "../../lib/types";

  let {
    route,
    hypervisorOnline,
    activeDomainLabel = null,
    activeDomainId = null,
    domains = [],
    onSelectDomain,
    onAddWorkspace,
    onOpenWorkspaceSettings,
  }: {
    route: CanonicalRoute;
    hypervisorOnline: boolean;
    activeDomainLabel?: string | null;
    activeDomainId?: string | null;
    domains?: CatalogEntryStatus[];
    onSelectDomain: (domainId: string) => void;
    onAddWorkspace: () => void;
    onOpenWorkspaceSettings: (domainId: string) => void;
  } = $props();

  let open = $state(false);

  const available = $derived(domains.filter((d) => d.is_available && !d.is_temporary));
  const connectionLabel = $derived(!hypervisorOnline ? "Offline" : "Local");

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
    <strong class="page-name">{ROUTE_LABELS[route]}</strong>
  </div>
  <div class="app-actions">
    <span class="connection" class:offline={!hypervisorOnline}>
      <i></i> {connectionLabel}
    </span>
  </div>
</div>

<style>
  .app-bar {
    position: relative;
    height: 44px;
    flex: none;
    display: flex;
    align-items: center;
    justify-content: space-between;
    border-bottom: 1px solid var(--pytxo-line, #1e2026);
    padding: 0 20px;
    gap: 12px;
  }
  .left {
    display: flex;
    align-items: center;
    gap: 14px;
    min-width: 0;
  }
  .page-name {
    font-size: 13px;
    font-weight: 600;
    letter-spacing: -0.02em;
    color: var(--pytxo-text-strong);
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
    background: var(--pytxo-surface-input, #0c0e13);
    color: var(--pytxo-text-body);
    border-radius: var(--pytxo-control-radius, 4px);
    padding: 5px 8px;
    font-size: 13px;
    cursor: pointer;
  }
  .switcher span {
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }
  .switcher:hover {
    border-color: color-mix(in oklab, var(--pytxo-accent, var(--pytxo-teal)) 35%, var(--pytxo-line));
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
    border-radius: var(--pytxo-panel-radius, 6px);
    background: var(--pytxo-surface-panel, #0c0e13);
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
    color: var(--pytxo-text-body);
    padding: 8px 10px;
    border-radius: var(--pytxo-control-radius, 4px);
    cursor: pointer;
    text-align: left;
  }
  .menu > button:hover,
  .menu > button.active {
    background: color-mix(in oklab, var(--pytxo-accent, var(--pytxo-teal)) 12%, var(--pytxo-surface-hover));
  }
  .menu strong {
    font-size: 13px;
    font-weight: 550;
  }
  .menu small {
    color: var(--pytxo-text-muted);
    font-size: 12px;
  }
  .menu .empty {
    margin: 0;
    padding: 10px;
    color: var(--pytxo-text-muted);
    font-size: 13px;
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
    color: var(--pytxo-text-muted);
    padding: 8px 10px;
    border-radius: var(--pytxo-control-radius, 4px);
    cursor: pointer;
    font-size: 13px;
  }
  .menu-actions button:hover {
    background: var(--pytxo-surface-hover);
    color: var(--pytxo-text-strong);
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
    color: var(--pytxo-text-muted);
    font-size: 12px;
  }
  .connection i {
    width: 6px;
    height: 6px;
    border-radius: 50%;
    background: var(--pytxo-success);
  }
  .connection.offline {
    color: var(--pytxo-danger, #d98994);
  }
  .connection.offline i {
    background: var(--pytxo-danger, #df6576);
  }
  .mono {
    font-family: "Geist Mono", ui-monospace, monospace;
  }
</style>
