<script lang="ts">
  import { ipc } from "../../lib/ipc";

  let {
    onContinue,
    onSkip,
    onWorkspaceSelected,
  }: {
    onContinue: () => void;
    onSkip: () => void;
    onWorkspaceSelected: (path: string) => void;
  } = $props();

  let selected = $state<string | null>(null);
  let busy = $state(false);

  async function pickFolder() {
    busy = true;
    try {
      const path = await ipc.pickWorkspaceFolder();
      if (path) {
        selected = path;
        onWorkspaceSelected(path);
      }
    } finally {
      busy = false;
    }
  }
</script>

<div class="step">
  <h2 class="title">Choose workspace</h2>
  <p class="lead">
    Pick a project folder to scan for structural topology and agent runs. You can change this
    later from the dashboard.
  </p>

  {#if selected}
    <code class="path">{selected}</code>
    <button type="button" class="primary" onclick={onContinue}>Continue</button>
  {:else}
    <button type="button" class="primary" disabled={busy} onclick={pickFolder}>
      {busy ? "Opening…" : "Select folder"}
    </button>
  {/if}
  <button type="button" class="ghost" onclick={onSkip}>Skip for now</button>
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
  .path {
    display: block;
    width: 100%;
    padding: 0.65rem 0.75rem;
    border-radius: 10px;
    background: color-mix(in oklab, var(--card) 80%, transparent);
    font-size: 0.75rem;
    word-break: break-all;
    text-align: left;
    box-shadow: inset 0 0 0 1px color-mix(in oklab, var(--foreground) 8%, transparent);
  }
  button {
    min-height: 44px;
    min-width: 220px;
    border-radius: 10px;
  }
  .ghost {
    background: transparent;
    border: none;
    color: var(--muted-foreground);
  }
</style>
