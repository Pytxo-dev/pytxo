<script lang="ts">
  import { withPreviewsHidden } from "../../lib/preview-overlay";
  import IconAlertTriangle from "@tabler/icons-svelte/icons/alert-triangle";
  import IconCheck from "@tabler/icons-svelte/icons/check";
  import IconChevronDown from "@tabler/icons-svelte/icons/chevron-down";
  import IconFolder from "@tabler/icons-svelte/icons/folder";
  import IconInbox from "@tabler/icons-svelte/icons/inbox";
  import IconPlus from "@tabler/icons-svelte/icons/plus";
  import IconSearch from "@tabler/icons-svelte/icons/search";
  import IconSettings from "@tabler/icons-svelte/icons/settings";
  import { tick } from "svelte";
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
  let query = $state("");
  let trigger: HTMLButtonElement | undefined = $state();
  let searchInput: HTMLInputElement | undefined = $state();
  let menuEl: HTMLDivElement | undefined = $state();
  let wrapper: HTMLDivElement | undefined = $state();
  const menuId = $props.id();

  const available = $derived(domains.filter((d) => d.is_available && !d.is_temporary));
  const filtered = $derived(available.filter(domain => domain.repo_root.toLowerCase().includes(query.trim().toLowerCase())));
  const connectionLabel = $derived(!hypervisorOnline ? "Offline" : "Local");

  function close(restoreFocus = true) {
    open = false;
    if (restoreFocus) trigger?.focus();
  }

  async function toggle() {
    if (open) { close(); return; }
    query = "";
    await withPreviewsHidden(() => { open = true; });
    await tick();
    searchInput?.focus();
  }

  function onMenuKeydown(event: KeyboardEvent) {
    if (event.key === "Escape") { event.preventDefault(); event.stopPropagation(); close(); return; }
    if (event.key !== "ArrowDown" && event.key !== "ArrowUp") return;
    const choices = [...(menuEl?.querySelectorAll<HTMLButtonElement>('[role="option"]') ?? [])];
    if (!choices.length) return;
    event.preventDefault();
    const index = choices.indexOf(document.activeElement as HTMLButtonElement);
    const next = index < 0 ? (event.key === "ArrowDown" ? 0 : choices.length - 1)
      : (index + (event.key === "ArrowDown" ? 1 : -1) + choices.length) % choices.length;
    choices[next]?.focus();
  }

  function pick(domainId: string) {
    onSelectDomain(domainId);
    close();
  }
</script>

<div class="app-bar">
  <div class="left">
    <div class="workspace-switcher" bind:this={wrapper} onfocusout={(event) => { if (open && event.relatedTarget && !wrapper?.contains(event.relatedTarget as Node)) close(false); }}>
      <button
        bind:this={trigger}
        class="switcher"
        aria-haspopup="dialog"
        aria-controls={menuId}
        aria-expanded={open}
        onclick={() => void toggle()}
        onkeydown={(event) => { if (event.key === "ArrowDown" && !open) { event.preventDefault(); void toggle(); } }}
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
          tabindex="-1"
          onclick={() => close()}
        ></button>
        <div class="menu" bind:this={menuEl} id={menuId} role="dialog" tabindex="-1" aria-label="Switch workspace" onkeydown={onMenuKeydown}>
          <label class="workspace-search"><IconSearch size={15} /><input bind:this={searchInput} bind:value={query} aria-label="Find workspace" placeholder="Find a workspace…" /></label>
          <div class="workspace-list" role="listbox" aria-label="Workspaces">
          {#each filtered as domain (domain.domain_id)}
            <button
              role="option"
              aria-selected={domain.domain_id === activeDomainId}
              class:active={domain.domain_id === activeDomainId}
              onclick={() => pick(domain.domain_id)}
            >
              <strong><IconFolder size={15} />{domain.repo_root.split(/[\\/]/).pop()}{#if domain.domain_id === activeDomainId}<IconCheck size={14} />{/if}</strong>
              <small class="mono">{domain.repo_root}</small>
            </button>
          {/each}
          {#if !filtered.length}
            <p class="empty" role="status">{available.length ? "No matching workspaces. Try a folder name or path." : "No workspaces yet. Add a folder to begin."}</p>
          {/if}
          </div>
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
    height: 52px;
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
    color: var(--pytxo-text-soft);
  }
  .workspace-switcher {
    position: relative;
    min-width: 0;
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
    min-height: var(--pytxo-control-height);
    padding: 0 10px;
    font-size: 13px;
    cursor: pointer;
    transition: background-color var(--pytxo-motion-fast) var(--pytxo-motion-ease), border-color var(--pytxo-motion-fast) var(--pytxo-motion-ease);
  }
  .switcher span {
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }
  .switcher:hover {
    border-color: color-mix(in oklab, var(--pytxo-accent, var(--pytxo-teal)) 35%, var(--pytxo-line));
    background: var(--pytxo-surface-hover);
  }
  .switcher:focus-visible { outline: 2px solid var(--pytxo-accent); outline-offset: 2px; }
  .switcher :global(svg) { flex: none; }
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
    max-height: min(520px, calc(100dvh - 120px));
    overflow: hidden;
    border: 1px solid var(--pytxo-line, #1e2026);
    border-radius: 10px;
    background: var(--pytxo-surface-panel, #0c0e13);
    box-shadow: 0 18px 40px -20px rgba(0, 0, 0, 0.7);
    padding: 6px;
    display: flex;
    flex-direction: column;
    gap: 4px;
  }
  .workspace-search { flex: none; display: flex; align-items: center; gap: 8px; padding: 6px 8px 10px; border-bottom: 1px solid var(--pytxo-line); color: var(--pytxo-text-muted); }
  .workspace-search input { min-width: 0; width: 100%; min-height: 32px; border: 0; background: transparent; color: var(--pytxo-text-strong); font: inherit; font-size: 13px; }
  .workspace-search input:focus-visible { outline: 2px solid var(--pytxo-accent); outline-offset: 2px; border-radius: 3px; }
  .workspace-list { min-height: 0; overflow-y: auto; max-height: 280px; overscroll-behavior: contain; scroll-padding: 4px; }
  .workspace-list > button {
    display: flex;
    flex-direction: column;
    align-items: flex-start;
    gap: 2px;
    border: 0;
    background: transparent;
    color: var(--pytxo-text-body);
    padding: 8px 10px;
    min-height: 44px;
    border-radius: var(--pytxo-control-radius, 4px);
    cursor: pointer;
    text-align: left;
    width: 100%;
  }
  .workspace-list > button:hover,
  .workspace-list > button.active {
    background: color-mix(in oklab, var(--pytxo-accent, var(--pytxo-teal)) 12%, var(--pytxo-surface-hover));
  }
  .menu strong {
    display: flex; align-items: center; gap: 8px; width: 100%;
    font-size: 13px;
    font-weight: 550;
  }
  .menu small {
    color: var(--pytxo-text-muted);
    font-size: 12px;
    max-width: 100%;
    overflow-wrap: anywhere;
  }
  .menu .empty {
    margin: 0;
    padding: 10px;
    color: var(--pytxo-text-muted);
    font-size: 13px;
  }
  .menu-actions {
    flex: none;
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
    min-height: 40px;
  }
  .menu-actions button:hover {
    background: var(--pytxo-surface-hover);
    color: var(--pytxo-text-strong);
  }
  .app-actions {
    display: flex;
    align-items: center;
    gap: 12px;
    flex: none;
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
    min-height: 40px;
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
    min-height: 40px;
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
    background: var(--pytxo-surface-hover);
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
  .menu strong :global(svg:last-child:not(:first-child)) { margin-left: auto; }
  .menu button:focus-visible { outline: 2px solid var(--pytxo-accent); outline-offset: -2px; }
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
