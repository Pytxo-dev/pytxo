<script lang="ts">
  import SetupStepFrame from "./SetupStepFrame.svelte";
  import { markSetupComplete } from "../../lib/theme";
  import { displayPath } from "../../lib/path-display";
  import { Button } from "$lib/components/ui/button";

  let {
    onDisplay,
    onFinish,
    workspacePath = null as string | null,
  }: {
    onDisplay?: () => void;
    onFinish: () => void;
    workspacePath?: string | null;
  } = $props();

  function enter() {
    markSetupComplete();
    onFinish();
  }
</script>

<SetupStepFrame>
  <h2 class="title">Desktop setup complete</h2>
  <p class="lead">Describe what you want to build or fix. Pytxo will help you plan the work, follow your agents, and review their changes.</p>
  <p class="lead">Choose a project and a ready coding agent to start. You can finish those steps in Setup.</p>
  {#if workspacePath}
    <code class="path" title={workspacePath}>{displayPath(workspacePath)}</code>
  {/if}
  {#if onDisplay}<Button variant="outline" onclick={onDisplay}>Adjust display</Button>{/if}
  {#snippet actions()}<Button onclick={enter}>Enter Pytxo Desktop</Button>{/snippet}
</SetupStepFrame>

<style>
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
    border-radius: var(--panel-radius);
    background: color-mix(in oklab, var(--card) 80%, transparent);
    font-size: 0.75rem;
    word-break: break-all;
    text-align: left;
    border: 1px solid var(--border);
  }
</style>
