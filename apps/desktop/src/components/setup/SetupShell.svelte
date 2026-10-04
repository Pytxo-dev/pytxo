<script lang="ts">
  import type { Snippet } from "svelte";

  let { children }: { children: Snippet } = $props();
</script>

<!-- One fixed frame for every step, so the window never jumps between steps:
     a step rail on the left and the step itself on the right. Small windows
     fold the rail into a row of tabs above the step. -->
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
    padding: 24px;
    min-height: 0;
    overflow: hidden;
    background:
      radial-gradient(1200px 520px at 50% -10%, color-mix(in oklab, var(--foreground) 5%, transparent), transparent 70%),
      var(--background);
  }
  .setup-shell__card {
    width: min(1040px, 100%);
    height: min(640px, 100%);
    min-height: 0;
    display: grid;
    grid-template-columns: 296px minmax(0, 1fr);
    grid-template-rows: minmax(0, 1fr);
    border-radius: var(--radius-xl);
    background: var(--card);
    border: 1px solid var(--border);
    box-shadow: 0 1px 2px rgba(0, 0, 0, 0.2), 0 24px 60px rgba(0, 0, 0, 0.28);
    overflow: hidden;
  }
  @media (max-width: 860px), (max-height: 600px) {
    .setup-shell { padding: 12px; }
    .setup-shell__card { grid-template-columns: minmax(0, 1fr); grid-template-rows: auto minmax(0, 1fr); }
  }
</style>
