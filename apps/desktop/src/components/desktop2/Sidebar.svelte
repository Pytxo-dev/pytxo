<script lang="ts">
  import { IconChevronsLeft, IconChevronsRight, IconSearch, IconSettings, IconUserCircle } from "@tabler/icons-svelte";
  import type { CanonicalRoute, WorkspaceRecent } from "../../lib/navigation.svelte";

  type NavItem = { route: CanonicalRoute; label: string; icon: typeof IconSearch };

  let {
    route,
    primary,
    system,
    approvalsCount,
    recents,
    collapsed,
    tier,
    signedIn,
    onNavigate,
    onToggleCollapse,
    onOpenCommand,
    onOpenRecent,
    onAccountClick,
  }: {
    route: CanonicalRoute;
    primary: NavItem[];
    system: NavItem[];
    approvalsCount: number;
    recents: WorkspaceRecent[];
    collapsed: boolean;
    tier: string;
    signedIn: boolean;
    onNavigate: (route: CanonicalRoute) => void;
    onToggleCollapse: () => void;
    onOpenCommand: () => void;
    onOpenRecent: (recent: WorkspaceRecent) => void;
    onAccountClick: () => void;
  } = $props();

  const isMac = typeof navigator !== "undefined" && /Mac|iPhone|iPod|iPad/.test(navigator.userAgent);
  const shortcutHint = isMac ? "⌘K" : "Ctrl K";

  function go(next: CanonicalRoute) {
    return (event: MouseEvent) => {
      event.preventDefault();
      onNavigate(next);
    };
  }

  const accountLabel = $derived(signedIn ? `${tier.charAt(0).toUpperCase()}${tier.slice(1)} tier` : "Not signed in");
  const accountSub = $derived(signedIn ? "Account & billing" : "Local Core · optional sign-in");
</script>

<aside class="sidebar" class:collapsed aria-label="Primary sidebar">
  <div class="brand">
    <img class="brand-mark" src="/logo-mark.png" alt="" width="22" height="22" />
    {#if !collapsed}<div class="brand-copy"><strong>Pytxo</strong><span>Desktop</span></div>{/if}
    <button
      class="collapse-btn"
      onclick={onToggleCollapse}
      aria-label={collapsed ? "Expand sidebar" : "Collapse sidebar"}
      title={collapsed ? "Expand sidebar" : "Collapse sidebar"}
    >
      {#if collapsed}<IconChevronsRight size={15} />{:else}<IconChevronsLeft size={15} />{/if}
    </button>
  </div>

  <button
    class="command-trigger"
    onclick={onOpenCommand}
    aria-label="Open command palette"
    title={collapsed ? `Search or command (${shortcutHint})` : undefined}
  >
    <IconSearch size={15} />
    {#if !collapsed}<span>Search or command</span><kbd>{shortcutHint}</kbd>{/if}
  </button>

  <div class="sidebar-scroll">
  <nav aria-label="Primary">
    {#each primary as item (item.route)}
      <a
        href={`#/${item.route}`}
        class:active={route === item.route}
        onclick={go(item.route)}
        title={collapsed ? item.label : undefined}
        aria-current={route === item.route ? "page" : undefined}
      >
        <item.icon size={17} stroke={1.7} />
        {#if !collapsed}<span>{item.label}</span>{#if item.route === "approvals" && approvalsCount}<b>{approvalsCount}</b>{/if}{/if}
      </a>
    {/each}
  </nav>

  {#if recents.length && !collapsed}
    <div class="recents">
      <p>Recent</p>
      {#each recents.slice(0, 4) as recent (recent.id)}
        <button onclick={() => onOpenRecent(recent)} title={recent.label}><i></i><span>{recent.label}</span></button>
      {/each}
    </div>
  {/if}

  {#if system.length}
  <nav aria-label="System" class="system-nav">
    {#each system as item (item.route)}
      <a
        href={`#/${item.route}`}
        class:active={route === item.route}
        onclick={go(item.route)}
        title={collapsed ? item.label : undefined}
        aria-current={route === item.route ? "page" : undefined}
      >
        <item.icon size={17} stroke={1.7} />
        {#if !collapsed}<span>{item.label}</span>{/if}
      </a>
    {/each}
  </nav>
  {/if}
  </div>

  <div class="sidebar-footer">
    <button aria-label="Open account and billing" onclick={onAccountClick} title={collapsed ? `${accountLabel} · ${accountSub}` : undefined}>
      <span class="avatar"><IconUserCircle size={18} /></span>
      {#if !collapsed}
        <span class="account-copy"><strong>{accountLabel}</strong><small>{accountSub}</small></span>
        <IconSettings size={15} />
      {/if}
    </button>
  </div>
</aside>

<style>
  .sidebar {
    display: flex;
    flex-direction: column;
    min-width: 0;
    min-height: 0;
    height: 100%;
    background: var(--pytxo-surface-shell);
    border-right: 1px solid var(--pytxo-line);
    padding: 14px 12px 12px;
    transition: padding 150ms ease;
  }
  .sidebar.collapsed {
    padding: 14px 8px 12px;
    align-items: center;
  }
  .brand {
    position: relative;
    height: 38px;
    display: flex;
    align-items: center;
    gap: 10px;
    padding: 0 6px 12px;
    width: 100%;
  }
  .brand::after {
    content: "";
    position: absolute;
    left: 6px;
    right: 6px;
    bottom: 0;
    height: 1px;
    background: var(--pytxo-hairline);
    opacity: 0.9;
  }
  .brand .collapse-btn {
    margin-left: auto;
  }
  .sidebar.collapsed .brand::after {
    left: 2px;
    right: 2px;
  }
  .sidebar.collapsed .brand {
    flex-direction: column;
    height: auto;
    gap: 8px;
    justify-content: center;
    padding-inline: 0;
  }
  .brand-mark {
    width: 22px;
    height: 22px;
    flex-shrink: 0;
    border-radius: 2px;
  }
  .brand-copy {
    display: flex;
    align-items: baseline;
    gap: 5px;
    min-width: 0;
  }
  .brand strong {
    font-size: 14px;
    letter-spacing: -0.02em;
  }
  .brand span {
    font-size: 11px;
    color: var(--pytxo-text-muted);
  }
  .collapse-btn {
    margin-left: auto;
    display: flex;
    align-items: center;
    justify-content: center;
    width: 24px;
    height: 24px;
    border: 0;
    border-radius: 5px;
    background: none;
    color: #6b7280;
    cursor: pointer;
    transition: background-color 150ms ease, color 150ms ease;
  }
  .sidebar.collapsed .collapse-btn {
    margin-left: 0;
  }
  .collapse-btn:hover {
    background: #14161c;
    color: #d9dde2;
  }
  .collapse-btn:focus-visible {
    outline: 2px solid var(--pytxo-accent, var(--pytxo-teal));
    outline-offset: 1px;
  }
  button {
    font-family: inherit;
  }
  .command-trigger {
    display: flex;
    align-items: center;
    width: 100%;
    gap: 8px;
    height: 34px;
    border: 1px solid #262932;
    border-radius: 6px;
    background: #111319;
    padding: 0 9px;
    color: #89909d;
    font-size: 11px;
    cursor: pointer;
    transition: border-color 150ms ease, color 150ms ease, background-color 150ms ease;
  }
  .sidebar.collapsed .command-trigger {
    justify-content: center;
    padding: 0;
    width: 34px;
  }
  .command-trigger:hover {
    border-color: #35524e;
    color: #d9fff8;
    background: #101816;
  }
  .command-trigger:focus-visible {
    outline: 2px solid var(--pytxo-accent, var(--pytxo-teal));
    outline-offset: 1px;
  }
  .command-trigger span {
    flex: 1;
    text-align: left;
  }
  .command-trigger kbd {
    font: 10px "IBM Plex Mono", monospace;
    color: var(--pytxo-text-muted);
    border: 1px solid #2b2e37;
    border-radius: 4px;
    padding: 2px 4px;
  }
  .sidebar-scroll {
    flex: 1;
    min-height: 0;
    overflow-y: auto;
    overflow-x: hidden;
    scrollbar-width: thin;
    scrollbar-color: var(--pytxo-line) transparent;
  }
  nav,
  .recents {
    margin-top: 20px;
    width: 100%;
  }
  .recents p {
    margin: 0 8px 7px;
    letter-spacing: 0.04em;
    font-size: 12px;
    font-weight: 600;
    color: var(--pytxo-text-muted);
  }
  nav a {
    position: relative;
    display: flex;
    align-items: center;
    height: 34px;
    gap: 10px;
    padding: 0 9px;
    margin: 2px 0;
    border-radius: 6px;
    color: #858c98;
    text-decoration: none;
    font-size: 13px;
    transition: background-color 140ms ease, color 140ms ease;
  }
  .sidebar.collapsed nav a {
    justify-content: center;
    padding: 0;
  }
  nav a:hover {
    background: #14161c;
    color: #d9dde2;
  }
  nav a:focus-visible {
    outline: 2px solid var(--pytxo-accent, var(--pytxo-teal));
    outline-offset: 1px;
  }
  nav a.active {
    background: var(--pytxo-surface-active, #111113);
    color: var(--pytxo-text-strong, #ededef);
  }
  nav a.active:before {
    content: "";
    position: absolute;
    left: 0;
    width: 2px;
    height: 16px;
    border-radius: 1px;
    background-image: linear-gradient(180deg, var(--chroma-spectrum));
    background-size: 100% 200%;
    animation: 14s chroma-shift linear infinite;
  }
  @keyframes chroma-shift {
    0% {
      background-position: 0% 50%;
    }
    100% {
      background-position: 200% 50%;
    }
  }
  @media (prefers-reduced-motion: reduce) {
    nav a.active:before {
      animation: none;
    }
  }
  .sidebar.collapsed nav a.active:before {
    display: none;
  }
  nav a b {
    margin-left: auto;
    display: grid;
    place-items: center;
    min-width: 17px;
    height: 17px;
    border-radius: 9px;
    background: #3b2d17;
    color: #f1bc59;
    font-size: 11px;
  }
  .recents button {
    display: flex;
    align-items: center;
    gap: 9px;
    width: 100%;
    height: 29px;
    padding: 0 9px;
    border: 0;
    background: none;
    color: var(--pytxo-text-muted);
    font-size: 11px;
    cursor: pointer;
    border-radius: 5px;
    transition: color 140ms ease, background-color 140ms ease;
  }
  .recents button:hover {
    color: #d8dce2;
    background: #12141a;
  }
  .recents button:focus-visible {
    outline: 2px solid var(--pytxo-accent, var(--pytxo-teal));
    outline-offset: -1px;
  }
  .recents button span {
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }
  .recents i {
    flex-shrink: 0;
    width: 7px;
    height: 7px;
    border-radius: 2px;
    background: var(--pytxo-text-muted);
  }
  .system-nav {
    margin-top: auto;
  }
  .sidebar-footer {
    border-top: 1px solid var(--pytxo-line);
    margin-top: 14px;
    padding-top: 10px;
    width: 100%;
  }
  .sidebar-footer button {
    width: 100%;
    display: flex;
    align-items: center;
    gap: 9px;
    padding: 5px 4px;
    cursor: pointer;
    border: 0;
    background: none;
    border-radius: 6px;
    color: inherit;
    transition: background-color 140ms ease;
  }
  .sidebar.collapsed .sidebar-footer button {
    justify-content: center;
  }
  .sidebar-footer button:hover {
    background: #14161c;
  }
  .sidebar-footer button:focus-visible {
    outline: 2px solid var(--pytxo-accent, var(--pytxo-teal));
    outline-offset: 1px;
  }
  .avatar {
    display: grid;
    place-items: center;
    width: 28px;
    height: 28px;
    flex-shrink: 0;
    border-radius: 7px;
    background: #242833;
    color: #cdd2da;
  }
  .account-copy {
    display: flex;
    flex-direction: column;
    text-align: left;
    flex: 1;
    min-width: 0;
  }
  .account-copy strong {
    font-size: 10px;
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }
  .account-copy small {
    font-size: 9px;
    color: var(--pytxo-text-muted);
    margin-top: 2px;
  }

  @media (max-width: 760px) {
    .sidebar:not(.collapsed) {
      padding-inline: 8px;
      align-items: center;
    }
    .sidebar:not(.collapsed) .brand-copy,
    .sidebar:not(.collapsed) .collapse-btn,
    .sidebar:not(.collapsed) .command-trigger span,
    .sidebar:not(.collapsed) .command-trigger kbd,
    .sidebar:not(.collapsed) nav a span,
    .sidebar:not(.collapsed) .recents,
    .sidebar:not(.collapsed) .account-copy {
      display: none;
    }
    .sidebar:not(.collapsed) .brand {
      justify-content: center;
      padding-inline: 0;
    }
    .sidebar:not(.collapsed) .command-trigger {
      justify-content: center;
      width: 34px;
      padding: 0;
    }
    .sidebar:not(.collapsed) nav a {
      justify-content: center;
      padding: 0;
    }
  }

  :global(html[data-chroma-theme="light"]) nav a.active {
    background: var(--pytxo-surface-active);
    color: var(--pytxo-text-strong);
  }
  :global(html[data-chroma-theme="light"]) .command-trigger {
    background: var(--pytxo-surface-raised);
    border-color: var(--pytxo-line);
  }
</style>
