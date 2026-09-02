<script lang="ts">
  import { IconAlertTriangle, IconChevronDown, IconFolder, IconInbox, IconPlus, IconSettings } from "@tabler/icons-svelte";
  import { ROUTE_LABELS, type CanonicalRoute } from "../../lib/navigation.svelte";
  import type { CatalogEntryStatus } from "../../lib/types";

  let {
    route,
    hypervisorOnline,
    activeDomainLabel = null,
    activeDomainId = null,
    domains = [],
    approvalsCount = 0,
    snapshotAge = null,
    cursorGap = false,
    onSelectDomain,
    onAddWorkspace,
    onOpenWorkspaceSettings,
    onOpenApprovals,
    onDismissGap,
  }: {
    route: CanonicalRoute;
    hypervisorOnline: boolean;
    activeDomainLabel?: string | null;
    activeDomainId?: string | null;
    domains?: CatalogEntryStatus[];
    /** Approvals live here, not in the rail: a decision follows the operator. */
    approvalsCount?: number;
    /** How old the rendered snapshot is, so nothing on screen implies "now". */
    snapshotAge?: string | null;
    /** The store dropped changes between polls, so the view may have missed events. */
    cursorGap?: boolean;
    onSelectDomain: (domainId: string) => void;
    onAddWorkspace: () => void;
    onOpenWorkspaceSettings: (domainId: string) => void;
    onOpenApprovals: () => void;
    onDismissGap?: () => void;
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
    {#if cursorGap}
      <button
        class="gap"
        onclick={onDismissGap}
        title="The change log dropped entries between polls. State was reloaded in full, but intermediate events are not recoverable. Click to acknowledge."
      >
        <IconAlertTriangle size={13} /> Missed events, reloaded
      </button>
    {/if}
    {#if snapshotAge}
      <span class="age" role="status" title="Age of the snapshot on screen">{snapshotAge}</span>
    {/if}
    <button class="inbox" class:waiting={approvalsCount > 0} onclick={onOpenApprovals} aria-label={`Approvals inbox, ${approvalsCount} open`}>
      <IconInbox size={14} />
      <span>Approvals</span>
      {#if approvalsCount}<b>{approvalsCount}</b>{/if}
    </button>
    <span class="connection" class:offline={!hypervisorOnline} role="status" aria-label={`Local service ${connectionLabel.toLowerCase()}`}>
      <i></i> <span class="connection-text">{connectionLabel}</span>
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
    gap: 12px;
  }
  .age {
    color: var(--pytxo-text-muted);
    font: 11px "IBM Plex Mono", monospace;
    font-variant-numeric: tabular-nums;
  }
  .gap {
    display: inline-flex;
    align-items: center;
    gap: 5px;
    height: 26px;
    padding: 0 8px;
    border: 1px dashed var(--state-unknown);
    border-radius: var(--pytxo-control-radius, 4px);
    background: transparent;
    color: var(--state-unknown);
    font-family: inherit;
    font-size: 11px;
    cursor: pointer;
  }
  .gap:focus-visible {
    outline: 2px solid var(--pytxo-accent, var(--pytxo-teal));
    outline-offset: 1px;
  }
  .inbox {
    display: inline-flex;
    align-items: center;
    gap: 6px;
    height: 26px;
    padding: 0 8px;
    border: 1px solid var(--pytxo-line);
    border-radius: var(--pytxo-control-radius, 4px);
    background: transparent;
    color: var(--pytxo-text-muted);
    font-family: inherit;
    font-size: 12px;
    cursor: pointer;
  }
  .inbox:hover {
    color: var(--pytxo-text-strong);
  }
  .inbox:focus-visible {
    outline: 2px solid var(--pytxo-accent, var(--pytxo-teal));
    outline-offset: 1px;
  }
  /* The count is the only thing in persistent chrome allowed to use the
     attention hue, because it is the only thing that blocks the operator. */
  .inbox.waiting {
    border-color: color-mix(in oklab, var(--state-attention) 50%, var(--pytxo-line));
    color: var(--state-attention);
  }
  .inbox b {
    display: grid;
    place-items: center;
    min-width: 16px;
    height: 16px;
    border-radius: 8px;
    background: color-mix(in oklab, var(--state-attention) 26%, transparent);
    color: var(--state-attention);
    font-size: 11px;
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
    background: var(--state-verified);
  }
  .connection.offline {
    color: var(--state-refuted);
  }
  .connection.offline i {
    background: var(--state-refuted);
  }
  .mono {
    font-family: "IBM Plex Mono", ui-monospace, monospace;
  }
  @media (max-width: 520px) {
    .app-bar {
      padding: 0 8px;
      gap: 6px;
    }
    .left,
    .app-actions {
      gap: 6px;
    }
    .page-name,
    .inbox span,
    .connection-text {
      display: none;
    }
    .switcher {
      max-width: 104px;
    }
    .inbox {
      min-width: 30px;
      justify-content: center;
      padding: 0 5px;
    }
    .age {
      max-width: 54px;
      overflow: hidden;
      font-size: 9px;
      text-overflow: ellipsis;
      white-space: nowrap;
    }
    .connection {
      min-width: 10px;
      justify-content: center;
    }
    .gap {
      max-width: 92px;
      overflow: hidden;
      text-overflow: ellipsis;
      white-space: nowrap;
    }
  }
</style>
