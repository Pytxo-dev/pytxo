<script lang="ts">
  import { Button } from "$lib/components/ui/button";
  import SetupStepFrame from "./SetupStepFrame.svelte";
  let { onContinue }: { onContinue: () => void } = $props();
</script>

<SetupStepFrame>
  <h1 class="title">Put your coding agents on one job.</h1>
  <p class="lead">Describe a change once. Pytxo splits it across the agents you choose, runs your checks on the combined result, and writes only the files you review.</p>
  <ol class="welcome-flow">
    <li><span>01</span><div><strong>Connect your agents</strong><p>Codex, Claude Code, Cursor Agent, OpenCode or Antigravity, signed in with your own accounts.</p></div></li>
    <li><span>02</span><div><strong>Choose your project</strong><p>Start in a project folder or try the local example.</p></div></li>
    <li><span>03</span><div><strong>Review, then Apply</strong><p>See every changed line and its checks. By default, your project changes only when you Apply.</p></div></li>
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
