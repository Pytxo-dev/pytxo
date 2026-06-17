<script lang="ts">
  import type { Snippet } from "svelte";

  let { children }: { children: Snippet } = $props();
</script>

<div class="nebula-shell nebula-bg">
  <div class="nebula-shell__noise nebula-noise" aria-hidden="true"></div>
  <div class="nebula-shell__content">
    {@render children()}
  </div>
</div>

<style>
  .nebula-shell {
    position: relative;
    min-height: 100vh;
    display: flex;
    flex-direction: column;
  }
  .nebula-shell__noise {
    position: fixed;
    inset: 0;
    pointer-events: none;
    z-index: 0;
  }
  .nebula-shell__content {
    position: relative;
    z-index: 1;
    display: flex;
    flex-direction: column;
    flex: 1;
    min-height: 100vh;
  }
</style>
