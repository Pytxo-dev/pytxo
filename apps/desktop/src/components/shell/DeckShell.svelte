<script lang="ts">
  import type { Snippet } from "svelte";
  import TitleBar from "./TitleBar.svelte";

  let { children }: { children: Snippet } = $props();
</script>

<div class="deck-shell">
  <TitleBar />
  <div class="deck-shell__body">
    {@render children()}
  </div>
</div>

<style>
  .deck-shell {
    display: flex;
    flex-direction: column;
    min-height: 100vh;
    background: var(--background);
    color: var(--foreground);
  }
  .deck-shell__body {
    display: flex;
    flex-direction: column;
    flex: 1;
    min-height: 0;
    overflow: hidden;
  }
</style>
