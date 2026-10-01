<script lang="ts">
  import { Button } from "$lib/components/ui/button";
  import SetupStepFrame from "./SetupStepFrame.svelte";
  let { onContinue }: { onContinue: () => void } = $props();
</script>

<SetupStepFrame>
  <h1 class="title">Your coding agents. One clear place to work.</h1>
  <p class="lead">Tell Pytxo what you want to build or fix. It puts your coding agents to work, shows their progress, and helps you review the changes.</p>
  <ol class="welcome-flow">
    <li><span>01</span><div><strong>Connect a coding agent</strong><p>Start with Codex. Other installed tools can be inspected in Setup.</p></div></li>
    <li><span>02</span><div><strong>Choose your project</strong><p>Start in a project folder or try the local example.</p></div></li>
    <li><span>03</span><div><strong>Review every change</strong><p>Inspect the exact changes before you decide what to save.</p></div></li>
  </ol>
  {#snippet actions()}<Button onclick={onContinue}>Get started</Button>{/snippet}
</SetupStepFrame>

<style>
  .title { margin: 0; font-weight: 600; text-wrap: balance; }
  .lead { margin: 0; color: var(--muted-foreground); text-wrap: pretty; }
  .welcome-flow { list-style: none; margin: 8px 0 0; padding: 0; }
  .welcome-flow li { display: flex; gap: 18px; padding: 18px 0; border-top: 1px solid var(--border); }
  .welcome-flow span { color: var(--muted-foreground); font: 12px var(--font-mono, monospace); padding-top: 3px; font-variant-numeric: tabular-nums; }
  .welcome-flow strong { font-size: 14px; font-weight: 600; }
  .welcome-flow p { margin: 5px 0 0; font-size: 13px; line-height: 1.5; color: var(--muted-foreground); }
</style>
