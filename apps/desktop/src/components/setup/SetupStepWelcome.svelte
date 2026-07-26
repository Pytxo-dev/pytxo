<script lang="ts">
  import { Button } from "$lib/components/ui/button";

  let { onContinue }: { onContinue: () => void } = $props();
</script>

<div class="step">
  <img class="logo" src="/logo-mark.png" alt="Pytxo" width="72" height="72" />
  <h1 class="title">Pytxo</h1>
  <p class="lead">
    Run and supervise coding agents locally — structure, spend, and approvals — without a wall of
    terminals.
  </p>
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
  :global(.continue) {
    margin-top: 0.5rem;
    min-width: 220px;
  }
</style>
