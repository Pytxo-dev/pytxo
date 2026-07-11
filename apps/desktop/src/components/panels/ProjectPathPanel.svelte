<script lang="ts">
  import { ipc } from "../../lib/ipc";
  import type { ProjectRootDto } from "../../lib/types";
  import { Button } from "$lib/components/ui/button";
  import { Input } from "$lib/components/ui/input";
  import { Badge } from "$lib/components/ui/badge";

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
  <h2 class="panel-title">Workspace folders</h2>
  <p class="project-id">{projectId}</p>
  <ul class="roots">
    {#each roots as root}
      <li class="root-item">
        <div class="root-head">
          <span class="root-label">{root.label}</span>
          {#if root.primary}<Badge variant="default">primary</Badge>{/if}
          {#if root.read_only}<Badge variant="secondary">read-only</Badge>{/if}
          {#if root.permission_profile}
            <Badge variant="muted">{root.permission_profile}</Badge>
          {/if}
          {#if !root.primary}
            <Button
              variant="ghost"
              size="sm"
              class="remove"
              disabled={busy}
              onclick={() => removeRoot(root.label)}
            >
              Remove
            </Button>
          {/if}
        </div>
        <div class="root-path">{root.path}</div>
      </li>
    {/each}
  </ul>
  <div class="add-root">
    <Input bind:value={addPath} placeholder="Absolute path to add" disabled={busy} />
    <label class="ro-toggle">
      <input type="checkbox" bind:checked={addReadOnly} disabled={busy} />
      read-only
    </label>
    <Button size="sm" disabled={busy || !addPath.trim()} onclick={addRoot}>Add folder</Button>
  </div>
  {#if error}
    <p class="error" role="alert">{error}</p>
  {/if}
{/if}

<style>
  .panel-title {
    margin: 0 0 0.25rem;
    font-size: var(--text-sm, 0.875rem);
    font-weight: 600;
  }
  .project-id {
    font-size: var(--text-xs, 0.75rem);
    color: var(--muted-foreground);
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
    padding: 0.45rem 0.55rem;
    border-radius: var(--panel-radius);
    border: 1px solid var(--border);
    background: color-mix(in oklab, var(--foreground) 3%, transparent);
  }
  .root-head {
    display: flex;
    flex-wrap: wrap;
    align-items: center;
    gap: 0.35rem;
  }
  .root-label {
    font-weight: 600;
    font-size: var(--text-sm, 0.875rem);
  }
  .root-path {
    font-size: var(--text-xs, 0.75rem);
    color: var(--muted-foreground);
    word-break: break-all;
    margin-top: 0.2rem;
  }
  .add-root {
    display: flex;
    flex-direction: column;
    gap: 0.4rem;
    margin-top: 0.65rem;
  }
  .ro-toggle {
    font-size: var(--text-xs, 0.75rem);
    display: flex;
    align-items: center;
    gap: 0.35rem;
    color: var(--muted-foreground);
  }
  :global(.remove) {
    margin-left: auto;
  }
  .error {
    font-size: var(--text-xs, 0.75rem);
    color: var(--destructive, #f87171);
    margin: 0.35rem 0 0;
  }
</style>
