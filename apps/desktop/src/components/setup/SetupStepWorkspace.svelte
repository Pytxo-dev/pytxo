<script lang="ts">
  import { onMount } from "svelte";
  import { ipc } from "../../lib/ipc";
  import {
    loadWorkspaceCatalog,
    type WorkspaceListItem,
  } from "../../lib/workspace";
  import { Button } from "$lib/components/ui/button";

  let {
    onContinue,
    onSkip,
    onWorkspaceSelected,
    error = "",
  }: {
    onContinue: () => void;
    onSkip: () => void;
    onWorkspaceSelected: (path: string) => Promise<void> | void;
    error?: string;
  } = $props();

  let selected = $state<string | null>(null);
  let busy = $state(false);
  let localError = $state("");
  let recent = $state<WorkspaceListItem[]>([]);

  const displayError = $derived(localError || error);

  async function pickFolder() {
    busy = true;
    localError = "";
    try {
      const path = await ipc.pickWorkspaceFolder();
      if (path) {
        await selectPath(path);
      }
    } catch (e) {
      localError = e instanceof Error ? e.message : String(e);
    } finally {
      busy = false;
    }
  }

  async function selectPath(path: string) {
    busy = true;
    localError = "";
    try {
      await onWorkspaceSelected(path);
      selected = path;
    } catch (e) {
      localError = e instanceof Error ? e.message : String(e);
      selected = null;
    } finally {
      busy = false;
    }
  }

  onMount(() => {
    void loadWorkspaceCatalog()
      .then((items) => {
        recent = items.slice(0, 5);
      })
      .catch(() => {
        recent = [];
      });
  });
</script>

<div class="step">
  <h2 class="title">Open a Workspace</h2>
  <p class="lead">
    Pick a project folder to start. You can open more folders or multi-root Workspaces later from
    the home screen.
  </p>

  {#if displayError}
    <p class="error" role="alert">{displayError}</p>
  {/if}

  {#if selected}
    <code class="path">{selected}</code>
    <Button class="wide" onclick={onContinue}>Continue</Button>
  {:else}
    <Button class="wide" disabled={busy} onclick={pickFolder}>
      {busy ? "Opening…" : "Select folder"}
    </Button>
  {/if}

  {#if recent.length > 0 && !selected}
    <div class="recent">
      <p class="recent__label">Recent</p>
      <ul class="recent__list">
        {#each recent as item (item.kind + item.id)}
          <li>
            <button
              type="button"
              class="recent__item"
              disabled={busy}
              onclick={() => void selectPath(item.path)}
            >
              <span class="recent__name">{item.label}</span>
              <span class="recent__path">{item.path}</span>
            </button>
          </li>
        {/each}
      </ul>
    </div>
  {/if}

  <Button variant="ghost" class="wide" onclick={onSkip}>Skip for now</Button>
</div>

<style>
  .step {
    display: flex;
    flex-direction: column;
    align-items: center;
    text-align: center;
    gap: 1rem;
    max-width: 480px;
    margin: 0 auto;
    width: 100%;
  }
  .title {
    margin: 0;
    font-size: 1.35rem;
    text-wrap: balance;
  }
  .lead {
    margin: 0;
    color: var(--muted-foreground);
    font-size: 0.9rem;
    text-wrap: pretty;
    line-height: 1.5;
  }
  .error {
    margin: 0;
    width: 100%;
    font-size: 0.85rem;
    color: var(--destructive, #f87171);
    text-align: left;
  }
  .path {
    display: block;
    width: 100%;
    padding: 0.65rem 0.75rem;
    border-radius: var(--panel-radius);
    background: color-mix(in oklab, var(--card) 80%, transparent);
    font-size: 0.75rem;
    word-break: break-all;
    text-align: left;
    border: 1px solid var(--border);
  }
  .recent {
    width: 100%;
    text-align: left;
  }
  .recent__label {
    margin: 0 0 0.35rem;
    font-size: 0.7rem;
    font-weight: 600;
    letter-spacing: 0.06em;
    text-transform: uppercase;
    color: var(--muted-foreground);
  }
  .recent__list {
    list-style: none;
    margin: 0;
    padding: 0;
    display: flex;
    flex-direction: column;
    gap: 0;
    border-top: 1px solid var(--border);
  }
  .recent__item {
    display: flex;
    flex-direction: column;
    gap: 0.1rem;
    width: 100%;
    text-align: left;
    padding: 0.55rem 0;
    border: none;
    border-bottom: 1px solid var(--border);
    background: transparent;
    color: var(--foreground);
    cursor: pointer;
  }
  .recent__item:hover:not(:disabled) {
    background: color-mix(in oklab, var(--foreground) 4%, transparent);
  }
  .recent__name {
    font-size: 0.85rem;
    font-weight: 500;
  }
  .recent__path {
    font-size: 0.7rem;
    font-family: ui-monospace, monospace;
    color: var(--muted-foreground);
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }
  :global(.wide) {
    min-width: 220px;
  }
</style>
