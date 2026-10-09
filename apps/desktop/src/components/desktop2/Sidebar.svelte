<script lang="ts">
  import IconChevronsLeft from "@tabler/icons-svelte/icons/chevrons-left";
  import IconChevronsRight from "@tabler/icons-svelte/icons/chevrons-right";
  import IconPlus from "@tabler/icons-svelte/icons/plus";
  import IconSearch from "@tabler/icons-svelte/icons/search";
  import IconUserCircle from "@tabler/icons-svelte/icons/user-circle";
  import type { CanonicalRoute, WorkspaceRecent } from "../../lib/navigation.svelte";
  import { MOD_KEY } from "../../lib/platform";
  import AdeIdentity from "./AdeIdentity.svelte";

  export type SidebarAgent = { id: string; label: string; vendor: string; cli: string | null; tone: "live" | "done" | "failed" | "queued" | "settled" };
  export type SidebarJob = { runId: string; title: string; live: boolean; agents: SidebarAgent[] };
  export type SidebarRecent = { runId: string; title: string; meta: string };

  type NavItem = { route: CanonicalRoute; label: string; icon: typeof IconSearch };

  let {
    route,
    primary,
    system,
    recents,
    collapsed,
    autoCollapsed = false,
    tier,
    signedIn,
    onNavigate,
    onToggleCollapse,
    onOpenCommand,
    onOpenRecent,
    onAccountClick,
    onNewRun = null,
    hasDraft = false,
    activeRunsCount = 0,
    job = null,
    recentJobs = [],
    onOpenRun = () => {},
  }: {
    route: CanonicalRoute;
    primary: NavItem[];
    system: NavItem[];
    recents: WorkspaceRecent[];
    collapsed: boolean;
    autoCollapsed?: boolean;
    tier: string;
    signedIn: boolean;
    onNavigate: (route: CanonicalRoute) => void;
    onToggleCollapse: () => void;
    onOpenCommand: () => void;
    onOpenRecent: (recent: WorkspaceRecent) => void;
    onAccountClick: () => void;
    onNewRun?: (() => void) | null;
    hasDraft?: boolean;
    activeRunsCount?: number;
    job?: SidebarJob | null;
    recentJobs?: SidebarRecent[];
    onOpenRun?: (runId: string) => void;
  } = $props();
  const finished = $derived(job ? job.agents.filter((agent) => agent.tone === "done").length : 0);

  const shortcutHint = MOD_KEY === "⌘" ? "⌘K" : "Ctrl K";

  function go(next: CanonicalRoute) {
    return (event: MouseEvent) => {
      event.preventDefault();
      onNavigate(next);
    };
  }

  const accountLabel = $derived(signedIn ? `${tier.charAt(0).toUpperCase()}${tier.slice(1)} tier` : "Local Core");
  const accountSub = $derived(signedIn ? "Account & billing" : "Account settings");
</script>

<aside class="sidebar" class:collapsed aria-label="Primary sidebar">
  <div class="brand">
    <button
      class="collapse-btn"
      disabled={autoCollapsed}
      onclick={onToggleCollapse}
      aria-label={autoCollapsed ? "Sidebar compact at this width" : collapsed ? "Expand sidebar" : "Collapse sidebar"}
      title={autoCollapsed ? "Widen the window or reduce text zoom to expand the sidebar" : collapsed ? "Expand sidebar" : "Collapse sidebar"}
    >
      {#if collapsed}<IconChevronsRight size={15} />{:else}<IconChevronsLeft size={15} />{/if}
    </button>
  </div>

  {#if onNewRun}
    <button class="compose-trigger" onclick={onNewRun} aria-label={hasDraft ? "Continue draft" : "New work from sidebar"} title={hasDraft ? "Continue your draft in this window" : "Create a new run"}>
      <IconPlus size={16} /><span>{hasDraft ? "Continue draft" : "New work"}</span>
      {#if hasDraft}<i aria-hidden="true"></i>{/if}
    </button>
  {/if}

  <button
    class="command-trigger"
    onclick={onOpenCommand}
    aria-label="Open command palette"
    title={collapsed ? `Search or command (${shortcutHint})` : undefined}
  >
    <IconSearch size={15} />
    {#if !collapsed}<span>Search</span><kbd>{shortcutHint}</kbd>{/if}
  </button>

  <div class="sidebar-scroll">
  <nav aria-label="Primary">
    {#each primary as item (item.route)}
      <a
        href={`#/${item.route}`}
        class:active={route === item.route}
        onclick={go(item.route)}
        aria-label={item.label}
        title={item.label}
        aria-current={route === item.route ? "page" : undefined}
      >
        <item.icon size={17} stroke={1.7} />
        {#if !collapsed}<span>{item.label}</span>{/if}
        {#if item.route === "work" && activeRunsCount && !collapsed}<b class="nav-count" aria-hidden="true" title={`${activeRunsCount} active runs`}>{activeRunsCount}</b>{/if}
      </a>
    {/each}
  </nav>

  {#if job && !collapsed}
    <section class="now" aria-label={job.live ? "Running now" : "Latest work"}>
      <p class="section-label">{job.live ? "Now" : "Latest"}</p>
      <button class="job" onclick={() => onOpenRun(job.runId)} title={job.title}>
        <span class="job-title"><i class="dot" data-tone={job.live ? "live" : "settled"}></i><span>{job.title}</span>{#if job.agents.length}<em>{finished}/{job.agents.length}</em>{/if}</span>
        {#if job.agents.length}<span class="job-meter" aria-hidden="true"><b>{"█".repeat(Math.round((finished / job.agents.length) * 24))}</b>{"░".repeat(24 - Math.round((finished / job.agents.length) * 24))}</span>{/if}
      </button>
      {#if job.agents.length}
        <ul class="job-agents" aria-label="Agents on this job">
          {#each job.agents as agent (agent.id)}
            <li><button onclick={() => onOpenRun(job.runId)} title={`${agent.label} · ${agent.vendor}`}><span class="agent-logo">{#if agent.cli}<AdeIdentity id={agent.cli} />{/if}</span><span class="agent-label">{agent.label}</span>{#if agent.tone === "live"}<span class="tui-spin agent-spin" role="img" aria-label="working"><span aria-hidden="true">.:+*=x</span></span>{:else}<i class="dot" data-tone={agent.tone} aria-label={agent.tone}></i>{/if}</button></li>
          {/each}
        </ul>
      {/if}
    </section>
  {/if}

  {#if recentJobs.length && !collapsed}
    <section class="recent-jobs" aria-label="Recent work">
      <p class="section-label">Recent</p>
      {#each recentJobs.slice(0, 4) as recent (recent.runId)}
        <button onclick={() => onOpenRun(recent.runId)} title={recent.title}><span>{recent.title}</span><small>{recent.meta}</small></button>
      {/each}
    </section>
  {/if}

  {#if recents.length && !collapsed}
    <div class="recents">
      <p>Recent workspaces</p>
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
        aria-label={item.label}
        title={item.label}
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
    padding: 12px 10px 10px;
  }
  .sidebar.collapsed {
    padding: 14px 8px 12px;
    align-items: center;
  }
  .brand {
    position: relative;
    height: 52px;
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
  .compose-trigger {
    display: flex; align-items: center; gap: 9px; flex: none;
    width: 100%; min-height: 40px; margin: 0 0 8px; padding: 0 10px;
    border: 1px solid var(--pytxo-line); border-radius: var(--pytxo-control-radius);
    color: var(--pytxo-text-strong); background: var(--pytxo-surface-raised);
    font: inherit; font-size: 13px; font-weight: 550; cursor: pointer;
    transition: background-color 140ms ease, border-color 140ms ease;
  }
  .compose-trigger:hover { background: var(--pytxo-surface-hover); border-color: var(--pytxo-text-muted); }
  .compose-trigger:focus-visible { outline: 2px solid var(--pytxo-accent); outline-offset: 1px; }
  .compose-trigger i { width: 5px; height: 5px; border-radius: 50%; background: var(--pytxo-accent); margin-left: auto; }
  .sidebar.collapsed .compose-trigger { justify-content: center; padding: 0; width: 40px; }
  .sidebar.collapsed .compose-trigger span, .sidebar.collapsed .compose-trigger i { display: none; }
  .nav-count { margin-left: auto; font-size: 11px; font-weight: 500; color: var(--pytxo-text-muted); font-variant-numeric: tabular-nums; }
  .collapse-btn {
    margin-left: auto;
    display: flex;
    align-items: center;
    justify-content: center;
    width: 32px;
    height: 40px;
    flex: none;
    border: 0;
    border-radius: 5px;
    background: none;
    color: var(--pytxo-text-muted);
    cursor: pointer;
    transition: background-color 150ms ease, color 150ms ease;
  }
  .sidebar.collapsed .collapse-btn {
    margin-left: 0;
  }
  .collapse-btn:hover {
    background: var(--pytxo-surface-hover);
    color: var(--pytxo-text-strong);
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
    height: 40px;
    flex: none;
    border: 1px solid var(--pytxo-line);
    border-radius: 6px;
    background: var(--pytxo-surface-input);
    padding: 0 9px;
    color: var(--pytxo-text-soft);
    font-size: 12px;
    cursor: pointer;
    transition: border-color 150ms ease, color 150ms ease, background-color 150ms ease;
  }
  .sidebar.collapsed .command-trigger {
    justify-content: center;
    padding: 0;
    width: 40px;
  }
  .command-trigger:hover {
    border-color: var(--pytxo-text-muted);
    color: var(--pytxo-text-strong);
    background: var(--pytxo-surface-hover);
  }
  .command-trigger:focus-visible {
    outline: 2px solid var(--pytxo-accent, var(--pytxo-teal));
    outline-offset: 1px;
  }
  .command-trigger span {
    flex: 1;
    min-width: 0;
    text-align: left;
    white-space: nowrap;
  }
  .command-trigger kbd {
    font: 11px "IBM Plex Mono", monospace;
    flex: none;
    white-space: nowrap;
    color: var(--pytxo-text-muted);
    border: 1px solid var(--pytxo-line);
    border-radius: 4px;
    padding: 2px 4px;
    background: var(--pytxo-surface-panel);
    box-shadow: none;
  }
  .sidebar-scroll {
    flex: 1;
    min-height: 0;
    overflow-y: auto;
    overflow-x: hidden;
    display: flex;
    flex-direction: column;
    width: 100%;
  }
  .section-label { margin: 14px 8px 6px; color: var(--pytxo-text-muted); font: 500 10px var(--pytxo-font-mono, "IBM Plex Mono", monospace); letter-spacing: .12em; text-transform: uppercase; }
  .now, .recent-jobs { display: grid; }
  .job { display: grid; gap: 8px; width: 100%; padding: 9px 10px; border: 1px solid var(--pytxo-line); border-radius: 8px; background: var(--pytxo-surface-raised); color: var(--pytxo-text-strong); text-align: left; cursor: pointer; }
  .job:hover { border-color: var(--pytxo-text-muted); }
  .job-title { display: flex; align-items: center; gap: 8px; min-width: 0; font-size: 12.5px; font-weight: 600; }
  .job-title > span { min-width: 0; overflow: hidden; text-overflow: ellipsis; white-space: nowrap; }
  .job-title em { margin-left: auto; color: var(--pytxo-activity); font: 500 11px var(--pytxo-font-mono, "IBM Plex Mono", monospace); font-style: normal; }
  .job-agents { display: grid; gap: 1px; margin: 4px 0 0; padding: 0; list-style: none; }
  .job-agents button { display: grid; grid-template-columns: 18px minmax(0, 1fr) auto; align-items: center; gap: 9px; width: 100%; min-height: 30px; padding: 0 8px 0 10px; border: 0; border-radius: 6px; background: transparent; color: var(--pytxo-text-soft); font-size: 12.5px; text-align: left; cursor: pointer; }
  .job-agents button:hover { background: var(--pytxo-surface-hover); color: var(--pytxo-text-strong); }
  .agent-logo { display: grid; place-items: center; width: 18px; height: 18px; }
  .agent-logo :global(svg), .agent-logo :global(img) { width: 14px; height: 14px; }
  .agent-label { overflow: hidden; text-overflow: ellipsis; white-space: nowrap; }
  .dot { flex: none; width: 7px; height: 7px; border-radius: 50%; background: var(--pytxo-line); }
  .agent-spin { color: var(--pytxo-activity); font-size: 12px; }
  .job-meter { display: block; margin-top: 6px; overflow: hidden; color: color-mix(in srgb, var(--pytxo-text-muted) 55%, transparent); font: 8px/1 var(--pytxo-font-mono); white-space: nowrap; }
  .job-meter b { color: var(--state-verified); font-weight: 400; }
  .dot[data-tone="live"] { background: var(--pytxo-activity); box-shadow: 0 0 0 3px color-mix(in srgb, var(--pytxo-activity) 18%, transparent); }
  .dot[data-tone="done"] { background: var(--state-verified); }
  .dot[data-tone="failed"] { background: var(--state-refuted); }
  .recent-jobs button { display: flex; align-items: baseline; justify-content: space-between; gap: 10px; min-height: 30px; padding: 0 8px; border: 0; border-radius: 6px; background: transparent; color: var(--pytxo-text-soft); font-size: 12.5px; text-align: left; cursor: pointer; }
  .recent-jobs button:hover { background: var(--pytxo-surface-hover); color: var(--pytxo-text-strong); }
  .recent-jobs span { min-width: 0; overflow: hidden; text-overflow: ellipsis; white-space: nowrap; }
  .recent-jobs small { flex: none; color: var(--pytxo-text-muted); font: 11px var(--pytxo-font-mono, "IBM Plex Mono", monospace); }
  .job:focus-visible, .job-agents button:focus-visible, .recent-jobs button:focus-visible { outline: 2px solid var(--pytxo-accent); outline-offset: 1px; }
  nav,
  .recents {
    margin-top: 16px;
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
    height: 40px;
    gap: 10px;
    padding: 0 9px;
    margin: 2px 0;
    border-radius: var(--pytxo-control-radius);
    color: var(--pytxo-text-soft);
    text-decoration: none;
    font-size: 13px;
    transition: background-color 140ms ease, color 140ms ease;
  }
  .sidebar.collapsed nav a {
    justify-content: center;
    padding: 0;
  }
  nav a:hover {
    background: var(--pytxo-surface-hover);
    color: var(--pytxo-text-strong);
  }
  nav a:focus-visible {
    outline: 2px solid var(--pytxo-accent, var(--pytxo-teal));
    outline-offset: 1px;
  }
  nav a.active {
    background: var(--pytxo-surface-active, #151518);
    color: var(--pytxo-text-strong, #ededef);
  }
  /* Selection is a static marker. Persistent chrome carries no ornament, so the
     only moving thing on screen is something the system is actually doing. */
  nav a.active:before {
    content: "";
    position: absolute;
    left: 0;
    width: 2px;
    height: 16px;
    border-radius: 1px;
    background: var(--pytxo-text-strong);
  }
  .sidebar.collapsed nav a.active:before {
    display: none;
  }
  .recents button {
    display: flex;
    align-items: center;
    gap: 9px;
    width: 100%;
    height: 40px;
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
    color: var(--pytxo-text-strong);
    background: var(--pytxo-surface-hover);
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
    padding-top: 20px;
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
    min-height: 44px;
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
    background: var(--pytxo-surface-hover);
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
    background: var(--pytxo-surface-raised);
    color: var(--pytxo-text-soft);
  }
  .account-copy {
    display: flex;
    flex-direction: column;
    text-align: left;
    flex: 1;
    min-width: 0;
  }
  .account-copy strong {
    font-size: 11px;
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }
  .account-copy small {
    font-size: 11px;
    color: var(--pytxo-text-muted);
    margin-top: 2px;
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }

  @media (max-width: 1279px) {
    .sidebar:not(.collapsed) {
      padding-inline: 8px;
      align-items: center;
    }
    .sidebar:not(.collapsed) .compose-trigger span,
    .sidebar:not(.collapsed) .compose-trigger i,
    .sidebar:not(.collapsed) .nav-count,
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
      width: 40px;
      padding: 0;
    }
    .sidebar:not(.collapsed) .compose-trigger { justify-content: center; width: 40px; padding: 0; }
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
