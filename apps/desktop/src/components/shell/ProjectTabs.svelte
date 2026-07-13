<script lang="ts">
  import type { WorkspaceTab } from "../../lib/deck-store.svelte";
  import { Button } from "$lib/components/ui/button";
  import { Badge } from "$lib/components/ui/badge";

  let {
    tabs,
    activeTabId = null as string | null,
    homeActive = false,
    onAddTab,
    onCloseTab,
    onSwitchTab,
    onGoHome,
  }: {
    tabs: WorkspaceTab[];
    activeTabId?: string | null;
    homeActive?: boolean;
    onAddTab: () => void;
    onCloseTab: (tabId: string) => void;
    onSwitchTab: (tabId: string) => void;
    onGoHome?: () => void;
  } = $props();
</script>

<nav class="project-tabs" aria-label="Open workspaces">
  {#if onGoHome}
    <button
      type="button"
      class="project-tabs__home"
      class:project-tabs__home--active={homeActive}
      onclick={onGoHome}
      aria-label="Workspaces home"
      aria-current={homeActive ? "page" : undefined}
    >
      Workspaces
    </button>
  {/if}
  <div class="project-tabs__list deck-scroll">
    {#each tabs as tab (tab.id)}
      <div
        class="project-tabs__tab"
        class:project-tabs__tab--active={tab.id === activeTabId && !homeActive}
      >
        <button
          type="button"
          class="project-tabs__select"
          onclick={() => onSwitchTab(tab.id)}
          title={tab.projectId ? `Workspace · ${tab.roots.length || "?"} folders` : tab.domainId}
          aria-current={tab.id === activeTabId && !homeActive ? "true" : undefined}
        >
          <span class="project-tabs__label">{tab.label}</span>
          {#if tab.projectId}
            <Badge variant="muted" class="project-tabs__multi">multi-root</Badge>
          {/if}
        </button>
        <button
          type="button"
          class="project-tabs__close"
          aria-label="Close {tab.label}"
          onclick={() => onCloseTab(tab.id)}
        >
          ×
        </button>
      </div>
    {/each}
  </div>
  <Button
    variant="ghost"
    size="sm"
    class="project-tabs__add"
    onclick={onAddTab}
    aria-label="Open workspace"
    title="Open workspace"
  >
    +
  </Button>
</nav>

<style>
  .project-tabs {
    display: flex;
    align-items: stretch;
    gap: 0;
    border-bottom: 1px solid var(--sidebar-border);
    background: var(--sidebar);
    min-height: 36px;
    flex-shrink: 0;
  }
  .project-tabs__home {
    border: none;
    border-right: 1px solid var(--sidebar-border);
    border-radius: 0;
    background: transparent;
    padding: 0 0.85rem;
    font-size: var(--text-xs, 0.75rem);
    font-weight: 600;
    letter-spacing: 0.02em;
    color: var(--muted-foreground);
    cursor: pointer;
    position: relative;
  }
  .project-tabs__home:hover {
    color: var(--foreground);
    background: color-mix(in oklab, var(--foreground) 5%, transparent);
  }
  .project-tabs__home--active {
    color: var(--foreground);
    background: var(--background);
  }
  .project-tabs__home--active::after {
    content: "";
    position: absolute;
    left: 0;
    right: 0;
    bottom: 0;
    height: 2px;
    background: var(--primary);
  }
  .project-tabs__list {
    display: flex;
    align-items: stretch;
    overflow-x: auto;
    flex: 1;
    min-width: 0;
  }
  .project-tabs__tab {
    display: flex;
    align-items: stretch;
    border-right: 1px solid var(--sidebar-border);
    position: relative;
    background: transparent;
  }
  .project-tabs__tab--active {
    background: var(--background);
  }
  .project-tabs__tab--active::after {
    content: "";
    position: absolute;
    left: 0;
    right: 0;
    bottom: 0;
    height: 2px;
    background: var(--primary);
  }
  .project-tabs__select {
    display: flex;
    align-items: center;
    gap: 0.35rem;
    padding: 0 0.35rem 0 0.85rem;
    border: none;
    background: transparent;
    color: var(--muted-foreground);
    font-size: var(--text-xs, 0.75rem);
    min-height: 36px;
    cursor: pointer;
  }
  .project-tabs__tab--active .project-tabs__select {
    color: var(--foreground);
  }
  .project-tabs__select:hover {
    color: var(--foreground);
  }
  .project-tabs__label {
    max-width: 140px;
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }
  :global(.project-tabs__multi) {
    font-size: 0.6rem !important;
    padding: 0 0.3rem !important;
    height: 1.1rem;
  }
  .project-tabs__close {
    border: none;
    background: transparent;
    color: var(--muted-foreground);
    opacity: 0.5;
    font-size: 1rem;
    line-height: 1;
    padding: 0 0.5rem;
    cursor: pointer;
  }
  .project-tabs__close:hover {
    opacity: 1;
    color: var(--foreground);
  }
  :global(.project-tabs__add) {
    border-radius: 0 !important;
    min-width: 36px;
    border-left: 1px solid var(--sidebar-border);
  }
</style>
