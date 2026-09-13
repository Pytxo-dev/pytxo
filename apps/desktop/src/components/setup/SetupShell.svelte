<script lang="ts">
  import type { Snippet } from "svelte";

  let { children, height = 640 }: { children: Snippet; height?: number } = $props();
</script>

<div class="setup-shell">
  <div class="setup-shell__card" style:--setup-height={`${height}px`}>
    {@render children()}
  </div>
</div>

<style>
  .setup-shell {
    flex: 1;
    display: flex;
    align-items: center;
    justify-content: center;
    padding: 24px;
    min-height: 0;
    overflow: hidden;
    background: var(--background);
  }
  .setup-shell__card {
    width: min(780px, 100%);
    height: min(var(--setup-height), 100%);
    min-height: 0;
    display: grid;
    grid-template-columns: minmax(0, 1fr);
    grid-template-rows: auto minmax(0, 1fr);
    border-radius: var(--radius-xl);
    background: var(--card);
    border: 1px solid var(--border);
    box-shadow: 0 1px 3px rgba(0, 0, 0, 0.12);
    overflow: hidden;
  }
  @media (max-width: 700px) {
    .setup-shell { padding: 12px; }
    .setup-shell__card { grid-template-columns: minmax(0, 1fr); grid-template-rows: auto minmax(0, 1fr); }
  }
</style>
