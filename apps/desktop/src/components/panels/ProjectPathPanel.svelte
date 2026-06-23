<script lang="ts">
  import { ipc } from "../../lib/ipc";
  import type { ProjectRootDto } from "../../lib/types";

  let {
    projectId,
    roots = [],
    onRootsChange,
  }: {
    projectId: string | null;
    roots: ProjectRootDto[];
    onRootsChange?: (roots: ProjectRootDto[]) => void;
  } = $props();

  let addPath = $state("");
  let addReadOnly = $state(false);
  let busy = $state(false);
  let error = $state("");

  async function refreshRoots() {
    if (!projectId) return;
    const next = await ipc.projectRoots(projectId);
    onRootsChange?.(next);
  }

  async function addRoot() {
    if (!projectId || !addPath.trim()) return;
    busy = true;
    error = "";
    try {
      const next = await ipc.projectAddRoot(projectId, addPath.trim(), addReadOnly);
      onRootsChange?.(next);
      addPath = "";
      addReadOnly = false;
    } catch (e) {
      error = e instanceof Error ? e.message : String(e);
    } finally {
      busy = false;
    }
  }

  async function removeRoot(label: string) {
    if (!projectId) return;
    busy = true;
    error = "";
    try {
      const next = await ipc.projectRemoveRoot(projectId, label);
      onRootsChange?.(next);
    } catch (e) {
      error = e instanceof Error ? e.message : String(e);
    } finally {
      busy = false;
    }
  }
</script>

{#if projectId}
  <h2 class="panel-title">Project paths</h2>
  <p class="project-id">{projectId}</p>
  <ul class="roots">
    {#each roots as root}
      <li class="root-item">
        <div class="root-head">
          <span class="root-label">{root.label}</span>
          {#if root.primary}<span class="badge">primary</span>{/if}
          {#if root.read_only}<span class="badge badge--ro">read-only</span>{/if}
          {#if root.permission_profile}
            <span class="badge badge--profile">{root.permission_profile}</span>
          {/if}
          {#if !root.primary}
            <button class="remove" disabled={busy} onclick={() => removeRoot(root.label)}>
              Remove
            </button>
          {/if}
        </div>
        <div class="root-path">{root.path}</div>
      </li>
    {/each}
  </ul>
  <div class="add-root">
    <input bind:value={addPath} placeholder="Absolute path to add" disabled={busy} />
    <label class="ro-toggle">
      <input type="checkbox" bind:checked={addReadOnly} disabled={busy} />
      read-only
    </label>
    <button class="primary" disabled={busy || !addPath.trim()} onclick={addRoot}>Add root</button>
  </div>
  {#if error}
    <p class="error">{error}</p>
  {/if}
{/if}

<style>
  .project-id {
    font-size: 0.75rem;
    opacity: 0.7;
    margin: 0 0 0.5rem;
  }
  ul.roots {
    list-style: none;
    padding: 0;
    margin: 0;
    display: flex;
    flex-direction: column;
    gap: 0.5rem;
  }
  .root-item {
    padding: 0.4rem 0.5rem;
    border-radius: 6px;
    background: color-mix(in oklab, var(--foreground) 6%, transparent);
  }
  .root-head {
    display: flex;
    flex-wrap: wrap;
    align-items: center;
    gap: 0.35rem;
  }
  .root-label {
    font-weight: 600;
    font-size: 0.85rem;
  }
  .root-path {
    font-size: 0.7rem;
    opacity: 0.75;
    word-break: break-all;
  }
  .badge {
    font-size: 0.6rem;
    padding: 0.05rem 0.3rem;
    border-radius: 4px;
    background: color-mix(in oklab, var(--brand-teal) 20%, transparent);
    color: var(--brand-teal);
  }
  .badge--ro {
    background: color-mix(in oklab, var(--brand-violet) 20%, transparent);
    color: var(--brand-violet);
  }
  .badge--profile {
    background: color-mix(in oklab, var(--brand-gold) 20%, transparent);
    color: var(--brand-gold);
  }
  .add-root {
    display: flex;
    flex-direction: column;
    gap: 0.35rem;
    margin-top: 0.5rem;
  }
  .add-root input[type="text"] {
    width: 100%;
    min-width: 0;
  }
  .ro-toggle {
    font-size: 0.75rem;
    display: flex;
    align-items: center;
    gap: 0.35rem;
  }
  button.remove {
    margin-left: auto;
    font-size: 0.7rem;
    padding: 0.15rem 0.4rem;
  }
  .error {
    font-size: 0.75rem;
    color: var(--brand-gold);
    margin: 0.35rem 0 0;
  }
</style>
