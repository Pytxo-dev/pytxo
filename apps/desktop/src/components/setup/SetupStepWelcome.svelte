<script lang="ts">
  import { Button } from "$lib/components/ui/button";

  let { onContinue }: { onContinue: () => void } = $props();
</script>

<div class="step">
  <img class="logo" src="/logo.png" alt="Pytxo" width="72" height="72" />
  <h1 class="title">Welcome to Pytxo Desktop</h1>
  <p class="lead">
    See what your coding agents are doing — structure, runs, and approvals — without a wall of
    terminals.
  </p>
  <ul class="features">
    <li>Live map of how agent edits touch your code</li>
    <li>Run and monitor agent waves across workspaces</li>
    <li>Approve risky actions before they land</li>
  </ul>
  <Button class="continue" onclick={onContinue}>Get started</Button>
</div>

<style>
  .step {
    display: flex;
    flex-direction: column;
    align-items: center;
    text-align: center;
    gap: 1rem;
    max-width: 420px;
    margin: 0 auto;
  }
  .logo {
    border-radius: 12px;
    border: 1px solid var(--border);
  }
  .title {
    margin: 0;
    font-size: 1.75rem;
    font-weight: 600;
    text-wrap: balance;
  }
  .lead {
    margin: 0;
    color: var(--muted-foreground);
    font-size: 0.95rem;
    text-wrap: pretty;
    line-height: 1.5;
  }
  .features {
    text-align: left;
    margin: 0;
    padding-left: 1.25rem;
    color: var(--muted-foreground);
    font-size: 0.88rem;
    line-height: 1.6;
  }
  :global(.continue) {
    margin-top: 0.5rem;
    min-width: 220px;
  }
</style>
