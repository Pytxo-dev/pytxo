<script lang="ts">
  import { onMount } from "svelte";
  import { Button } from "$lib/components/ui/button";
  import {
    loadWorkspaceCatalog,
    type WorkspaceListItem,
  } from "../../lib/workspace";

  let {
    onOpenFolder,
    onOpenItem,
  }: {
    onOpenFolder: () => void;
    onOpenItem: (item: WorkspaceListItem) => void;
  } = $props();

  let items = $state<WorkspaceListItem[]>([]);
  let loading = $state(true);
  let error = $state("");

  async function refresh() {
    loading = true;
    error = "";
    try {
      items = await loadWorkspaceCatalog();
    } catch (e) {
      error = e instanceof Error ? e.message : String(e);
      items = [];
    } finally {
      loading = false;
    }
  }

  onMount(() => {
    void refresh();
  });
</script>

<div class="workspace-home">
  <div class="workspace-home__hero">
    <h1 class="workspace-home__title">Workspaces</h1>
    <p class="workspace-home__lead">
      Open a folder or a multi-root project. Pytxo runs agents in the background and shows what
      they change here.
    </p>
    <div class="workspace-home__actions">
      <Button onclick={onOpenFolder}>Open folder</Button>
      {#if items.length > 0}
        <Button variant="ghost" onclick={() => void refresh()}>Refresh</Button>
      {/if}
    </div>
  </div>

  {#if error}
    <p class="workspace-home__error" role="alert">{error}</p>
  {/if}

  <section class="workspace-home__list" aria-label="Recent workspaces">
    {#if loading}
      <p class="workspace-home__empty">Loading…</p>
    {:else if items.length === 0}
      <p class="workspace-home__empty">
        No workspaces yet. Open a folder to start a run.
      </p>
    {:else}
      <h2 class="workspace-home__section">Recent</h2>
      <ul class="workspace-home__items">
        {#each items as item (item.kind + item.id)}
          <li>
            <button type="button" class="workspace-home__item" onclick={() => onOpenItem(item)}>
              <span class="workspace-home__item-label">{item.label}</span>
              <span class="workspace-home__item-meta"
                >{item.kind === "project" ? "multi-root" : "folder"}</span
              >
              <span class="workspace-home__item-path">{item.path}</span>
            </button>
          </li>
        {/each}
      </ul>
    {/if}
  </section>
</div>

<style>
  .workspace-home {
    display: flex;
    flex-direction: column;
    flex: 1;
    min-height: 0;
    padding: 2rem 2.5rem;
    gap: 1.75rem;
    overflow: auto;
    background: var(--background);
  }
  .workspace-home__hero {
    display: flex;
    flex-direction: column;
    gap: 0.75rem;
    max-width: 36rem;
  }
  .workspace-home__title {
    margin: 0;
    font-size: 1.75rem;
    font-weight: 600;
    letter-spacing: -0.02em;
  }
  .workspace-home__lead {
    margin: 0;
    color: var(--muted-foreground);
    font-size: 0.95rem;
    line-height: 1.5;
    text-wrap: pretty;
  }
  .workspace-home__actions {
    display: flex;
    gap: 0.5rem;
    flex-wrap: wrap;
    margin-top: 0.25rem;
  }
  .workspace-home__section {
    margin: 0 0 0.75rem;
    font-size: 0.75rem;
    font-weight: 600;
    letter-spacing: 0.06em;
    text-transform: uppercase;
    color: var(--muted-foreground);
  }
  .workspace-home__empty {
    margin: 0;
    color: var(--muted-foreground);
    font-size: 0.9rem;
  }
  .workspace-home__error {
    margin: 0;
    color: var(--destructive, #f87171);
    font-size: 0.85rem;
  }
  .workspace-home__items {
    list-style: none;
    margin: 0;
    padding: 0;
    display: flex;
    flex-direction: column;
    gap: 0;
    max-width: 40rem;
    border-top: 1px solid var(--border);
  }
  .workspace-home__item {
    display: grid;
    grid-template-columns: 1fr auto;
    grid-template-rows: auto auto;
    gap: 0.15rem 0.75rem;
    width: 100%;
    text-align: left;
    padding: 0.65rem 0;
    border-radius: 0;
    border: none;
    border-bottom: 1px solid var(--border);
    background: transparent;
    color: var(--foreground);
    cursor: pointer;
  }
  .workspace-home__item:hover {
    background: color-mix(in oklab, var(--foreground) 4%, transparent);
  }
  .workspace-home__item-label {
    font-weight: 500;
    font-size: 0.9rem;
  }
  .workspace-home__item-meta {
    font-size: 0.7rem;
    color: var(--muted-foreground);
  }
  .workspace-home__item-path {
    grid-column: 1 / -1;
    font-size: 0.72rem;
    font-family: ui-monospace, monospace;
    color: var(--muted-foreground);
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }
</style>
