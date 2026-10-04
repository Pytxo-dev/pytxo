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
  {#if workspacePath}
    <code class="path" title={workspacePath}>{displayPath(workspacePath)}</code>
  {:else}
    <p class="lead">Choose a project and a ready coding agent to start. You can finish those steps in Setup.</p>
  {/if}
  <ol class="next">
    <li><b>1</b>Describe a change in Work, in your own words.</li>
    <li><b>2</b>Review the plan, then run it. Agents work in their own copies.</li>
    <li><b>3</b>Read the changes and checks, then Apply. Nothing is written before that.</li>
  </ol>
  {#if onDisplay}<span class="display"><Button variant="outline" onclick={onDisplay}>Adjust display</Button></span>{/if}
  {#snippet actions()}<Button onclick={enter}>Enter Pytxo Desktop</Button>{/snippet}
</SetupStepFrame>

<style>
  .title {
    margin: 0;
    font-weight: 650;
    text-wrap: balance;
  }
  .lead {
    margin: 0;
    color: var(--muted-foreground);
    font-size: 0.9rem;
    text-wrap: pretty;
    line-height: 1.5;
  }
  .next { display: grid; gap: 0; margin: 4px 0 0; padding: 0; list-style: none; border-top: 1px solid var(--border); }
  .next li { display: flex; align-items: baseline; gap: 14px; padding: 12px 0; border-bottom: 1px solid var(--border); font-size: 13px; line-height: 1.5; }
  .next b { font: 12px var(--font-mono, monospace); color: var(--muted-foreground); font-weight: 400; }
  .display { align-self: flex-start; }
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
