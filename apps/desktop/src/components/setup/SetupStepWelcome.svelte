<script lang="ts">
  import { Button } from "$lib/components/ui/button";
  import ApertureGlyph from "../desktop2/ApertureGlyph.svelte";
  import SetupStepFrame from "./SetupStepFrame.svelte";
  let { onContinue }: { onContinue: () => void } = $props();
</script>

<SetupStepFrame centered>
  <div class="mark"><ApertureGlyph tone="settled" /></div>
  <h1 class="title">Many agents. One reviewed change.</h1>
  <p class="lead">Pytxo runs your coding agents side by side in isolated copies of your project, checks what they did, and changes nothing until you Apply.</p>
  {#snippet actions()}<Button onclick={onContinue}>Get started <kbd aria-hidden="true">↵</kbd></Button>{/snippet}
</SetupStepFrame>

<style>
  .mark { width: 200px; height: 200px; margin-bottom: 6px; }
  .mark :global(.aperture-glyph) { width: 100%; height: 100%; flex-basis: auto; }
  .title { font-size: 40px !important; }
  kbd { margin-left: 8px; font: 500 11px var(--font-mono, monospace); opacity: .55; }
  @media (max-height: 640px) { .mark { width: 120px; height: 120px; } .title { font-size: 28px !important; } }
  @media (max-height: 430px) { .mark { display: none; } }
</style>
