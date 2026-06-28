<script lang="ts">
  import { markSetupComplete } from "../../lib/theme";

  let { onFinish }: { onFinish: () => void } = $props();

  function enter() {
    markSetupComplete();
    onFinish();
  }
</script>

<div class="step">
  <h2 class="title">You're ready</h2>
  <p class="lead">
    Reality Deck will show workspace topology, active runs, and approvals. Dispatch your first wave
    from the command bar.
  </p>
  <button type="button" class="primary enter" onclick={enter}>Enter Reality Deck</button>
</div>

<style>
  .step {
    display: flex;
    flex-direction: column;
    align-items: center;
    text-align: center;
    gap: 1rem;
    max-width: 400px;
    margin: 0 auto;
  }
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
  .enter {
    min-height: 44px;
    padding: 0.6rem 1.75rem;
    border-radius: 10px;
    margin-top: 0.5rem;
  }
</style>
