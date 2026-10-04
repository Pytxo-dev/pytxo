<script lang="ts">
  import type { Snippet } from "svelte";

  let { children }: { children: Snippet } = $props();
</script>

<!-- Full-bleed onboarding surface: a quiet dot field behind one centered column. -->
<div class="setup-shell">{@render children()}</div>

<style>
  .setup-shell { position: relative; flex: 1; display: grid; grid-template-rows: auto minmax(0, 1fr) auto; min-height: 0; overflow: hidden; background: var(--pytxo-surface-shell, var(--background)); }
  .setup-shell::before { content: ""; position: absolute; inset: 0; pointer-events: none; background-image: radial-gradient(circle, color-mix(in oklab, var(--foreground) 9%, transparent) 1px, transparent 1.4px); background-size: 24px 24px; mask-image: radial-gradient(ellipse 70% 60% at 50% 45%, #000 30%, transparent 80%); }
</style>
