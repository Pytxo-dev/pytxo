<script lang="ts">
  import SetupStepFrame from "./SetupStepFrame.svelte";
  import { onMount } from "svelte";
  import { createDesktopBackend } from "../../lib/desktop-backend";
  import {
    loadWorkspaceCatalog,
    type WorkspaceListItem,
  } from "../../lib/workspace";
  import { displayPath } from "../../lib/path-display";
  import { Button } from "$lib/components/ui/button";

  let {
    onContinue,
    onSkip,
    onWorkspaceSelected,
    selected = $bindable<string | null>(null),
    error = "",
  }: {
    onContinue: () => void;
    onSkip: () => void;
    onWorkspaceSelected: (path: string) => Promise<void> | void;
    selected?: string | null;
    error?: string;
  } = $props();

  let selectedKind = $state<"workspace" | "example" | null>(null);
  let busy = $state(false);
  let creatingExample = $state(false);
  let localError = $state("");
  let recent = $state<WorkspaceListItem[]>([]);

  const displayError = $derived(localError || error);
  const backend = createDesktopBackend();

  async function pickFolder() {
    busy = true;
    localError = "";
    try {
      const path = await backend.openWorkspace();
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
      selectedKind = "workspace";
    } catch (e) {
      localError = e instanceof Error ? e.message : String(e);
      selected = null;
      selectedKind = null;
    } finally {
      busy = false;
    }
  }

  async function createExample() {
    busy = true;
    creatingExample = true;
    localError = "";
    try {
      const path = await backend.createExampleWorkspace();
      await onWorkspaceSelected(path);
      selected = path;
      selectedKind = "example";
    } catch (e) {
      localError = e instanceof Error ? e.message : String(e);
      selected = null;
      selectedKind = null;
    } finally {
      creatingExample = false;
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

<SetupStepFrame>
  <p class="kicker">Project</p>
  <h2 class="title">Where should they work?</h2>
  <p class="lead">Agents work in isolated copies. Your folder changes only when you Apply.</p>

  {#if displayError}
    <p class="error" role="alert">{displayError}</p>
  {/if}

  {#if selected}
    <div class="chosen">
      <span class="glyph ok" aria-hidden="true">✓</span>
      <span><strong>{selectedKind === "example" ? "Guided example ready" : selected.split(/[\\/]/).filter(Boolean).pop()}</strong><code title={selected}>{displayPath(selected)}</code></span>
      <button type="button" class="change" disabled={busy} onclick={() => { selected = null; selectedKind = null; }}>Change</button>
    </div>
    <small class="selected-note">
      {selectedKind === "example"
        ? "A local Git example. Its baseline tests need no API key."
        : "Pytxo uses this folder's existing Git state and configuration."}
    </small>

  {:else}
    <button type="button" class="drop" aria-label={busy && !creatingExample ? "Opening…" : "Select folder"} aria-describedby="drop-hint" disabled={busy} onclick={pickFolder}>
      <strong>{busy && !creatingExample ? "Opening…" : "Select folder"}</strong>
      <span id="drop-hint">Git repository recommended</span>
    </button>
    <button type="button" class="example" aria-label={creatingExample ? "Creating example…" : "Try the guided example"} aria-describedby="example-hint" disabled={busy} onclick={() => void createExample()}>
      <span class="glyph" aria-hidden="true">◇</span>
      <span><strong>{creatingExample ? "Creating example…" : "Try the guided example"}</strong><small id="example-hint">A small app with tests. Requires Git and Node.js; no API key.</small></span>
    </button>
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
              <span class="recent__path" title={item.path}>{displayPath(item.path)}</span>
            </button>
          </li>
        {/each}
      </ul>
    </div>
  {/if}

  {#snippet actions()}
    {#if selected}<Button disabled={busy} onclick={onContinue}>Enter Pytxo Desktop <kbd aria-hidden="true">↵</kbd></Button>{/if}
    {#if !selected}<Button variant="outline" disabled={busy} onclick={onSkip}>Skip for now</Button>{/if}
  {/snippet}
</SetupStepFrame>

<style>
  .error {
    margin: 0;
    width: 100%;
    font-size: 0.85rem;
    color: var(--destructive, #f87171);
    text-align: left;
  }
  .recent {
    width: 100%;
    text-align: left;
    min-height: 72px;
    display: flex;
    flex-direction: column;
    flex: 1 1 auto;
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
    overflow: auto;
    min-height: 0;
    max-height: 250px;
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
  .drop { display: grid; place-content: center; gap: 6px; height: 132px; border: 1px dashed color-mix(in oklab, var(--foreground) 22%, transparent); border-radius: 10px; background: transparent; color: var(--foreground); cursor: pointer; transition: border-color 160ms ease, background-color 160ms ease; }
  .drop:hover:not(:disabled) { border-color: color-mix(in oklab, var(--foreground) 45%, transparent); background: color-mix(in oklab, var(--foreground) 3%, transparent); }
  .drop strong { font-size: 14px; font-weight: 500; }
  .drop span, .example small { color: var(--muted-foreground); font: 12px var(--font-mono, "IBM Plex Mono", monospace); }
  .example { display: flex; align-items: center; gap: 14px; min-height: 60px; padding: 8px 16px; border: 1px solid var(--border); border-radius: 10px; background: color-mix(in oklab, var(--card) 70%, transparent); color: var(--foreground); text-align: left; cursor: pointer; }
  .example:hover:not(:disabled) { border-color: color-mix(in oklab, var(--foreground) 30%, transparent); }
  .example > span:last-child { display: grid; gap: 3px; }
  .example strong { font-size: 14px; font-weight: 500; }
  .glyph { display: grid; place-items: center; width: 26px; height: 26px; border-radius: 6px; background: color-mix(in oklab, var(--foreground) 7%, transparent); color: var(--pytxo-activity, currentColor); }
  .drop:focus-visible, .example:focus-visible { outline: 2px solid var(--ring); outline-offset: 2px; }
  .chosen { display: flex; align-items: center; gap: 14px; min-height: 64px; padding: 10px 16px; border: 1px solid color-mix(in oklab, var(--state-verified) 40%, var(--border)); border-radius: 10px; background: color-mix(in oklab, var(--state-verified) 5%, transparent); }
  .chosen > span:nth-child(2) { flex: 1; display: grid; gap: 3px; min-width: 0; }
  .chosen strong { font-size: 14px; font-weight: 500; }
  .chosen code { overflow: hidden; color: var(--muted-foreground); font-size: 12px; text-overflow: ellipsis; white-space: nowrap; }
  .glyph.ok { color: var(--state-verified); }
  .change { min-height: 32px; padding: 0 10px; border: 0; border-radius: 6px; background: transparent; color: var(--muted-foreground); font-size: 13px; cursor: pointer; }
  .change:hover:not(:disabled) { color: var(--foreground); background: color-mix(in oklab, var(--foreground) 6%, transparent); }
  kbd { margin-left: 8px; font: 500 11px var(--font-mono, monospace); opacity: .55; }
  .selected-note {
    color: var(--muted-foreground);
    font-size: 0.72rem;
  }
</style>
