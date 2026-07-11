<script lang="ts">
  import type { Snippet } from "svelte";

  let { children }: { children: Snippet } = $props();
</script>

<div class="setup-shell">
  <div class="setup-shell__card">
    {@render children()}
  </div>
</div>

<style>
  .setup-shell {
    flex: 1;
    display: flex;
    align-items: center;
    justify-content: center;
    padding: 2rem 1rem;
    min-height: 0;
    background: var(--background);
  }
  .setup-shell__card {
    width: min(520px, 100%);
    padding: 1.5rem;
    border-radius: var(--radius-xl);
    background: var(--card);
    border: 1px solid var(--border);
    box-shadow: 0 1px 3px rgba(0, 0, 0, 0.12);
  }
</style>
