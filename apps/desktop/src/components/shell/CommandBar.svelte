<script lang="ts">
  import { Input } from "$lib/components/ui/input";
  import { Button } from "$lib/components/ui/button";

  let {
    cmd = $bindable(""),
    dispatchRepo = $bindable(""),
    running = false,
    onDispatchRepoChange,
    onDryRun,
    onDispatch,
    onStop,
  }: {
    cmd?: string;
    dispatchRepo?: string;
    running?: boolean;
    onDispatchRepoChange?: (value: string) => void;
    onDryRun: () => void;
    onDispatch: () => void;
    onStop: () => void;
  } = $props();

  let showEmptyHint = $state(false);

  function handleRepoInput(e: Event & { currentTarget: HTMLInputElement }) {
    dispatchRepo = e.currentTarget.value;
    onDispatchRepoChange?.(e.currentTarget.value);
  }

  function handleCmdKeydown(e: KeyboardEvent) {
    if (e.key !== "Enter") return;
    e.preventDefault();
    if (e.shiftKey) {
      if (!running) onDryRun();
      return;
    }
    if (!cmd.trim()) {
      showEmptyHint = true;
      return;
    }
    showEmptyHint = false;
    if (!running) onDispatch();
  }

  function handleDispatch() {
    if (!cmd.trim()) {
      showEmptyHint = true;
      return;
    }
    showEmptyHint = false;
    onDispatch();
  }
</script>

<div class="command-bar" title="Enter to run · Shift+Enter to preview">
  <div class="command-bar__fields">
    <Input
      value={dispatchRepo}
      oninput={handleRepoInput}
      placeholder="Folder path (blank = workspace primary)"
      title="Absolute path to a project folder"
      aria-label="Repository path"
      class="command-bar__repo"
      disabled={running}
    />
    <Input
      bind:value={cmd}
      placeholder="Agent command (e.g. claude …)"
      aria-label="Run command"
      class="command-bar__cmd"
      onkeydown={handleCmdKeydown}
      disabled={running}
    />
  </div>
  <div class="command-bar__actions">
    <Button variant="outline" size="sm" onclick={onDryRun} disabled={running}>Preview</Button>
    <Button size="sm" onclick={handleDispatch} disabled={running}>Run agents</Button>
    <Button variant="secondary" size="sm" onclick={onStop} disabled={!running}>Stop</Button>
  </div>
  {#if showEmptyHint}
    <p class="command-bar__hint" role="status">Enter an agent command, then press Enter to run.</p>
  {/if}
</div>

<style>
  .command-bar {
    display: flex;
    align-items: center;
    gap: 0.5rem;
    flex-wrap: wrap;
    padding: 0.5rem 0.75rem;
    border-top: 1px solid var(--border);
    background: var(--card);
    flex-shrink: 0;
  }
  .command-bar__fields {
    display: flex;
    flex: 1 1 auto;
    gap: 0.5rem;
    min-width: 0;
    flex-wrap: nowrap;
  }
  :global(.command-bar__repo) {
    flex: 0 1 11rem;
    min-width: 7rem;
    font-family: ui-monospace, monospace;
    font-size: var(--text-sm, 0.875rem);
  }
  :global(.command-bar__cmd) {
    flex: 1 1 auto;
    min-width: 10rem;
    font-family: ui-monospace, monospace;
    font-size: var(--text-sm, 0.875rem);
  }
  .command-bar__actions {
    display: flex;
    gap: 0.35rem;
    flex-shrink: 0;
  }
  .command-bar__hint {
    flex-basis: 100%;
    margin: 0;
    font-size: var(--text-xs, 0.75rem);
    color: var(--foreground);
  }
</style>
