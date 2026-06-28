<script lang="ts">
  let { onContinue }: { onContinue: () => void } = $props();
</script>

<div class="step">
  <img class="logo" src="/logo.png" alt="Pytxo" width="96" height="96" />
  <h1 class="title">Welcome to Reality Deck</h1>
  <p class="lead">
    Your agent hypervisor console — structural telemetry, fleet runs, and Galaxy approvals in one
    place.
  </p>
  <ul class="features">
    <li>3D import topology from Signal Core</li>
    <li>Dispatch and monitor agent waves</li>
    <li>Approve risky actions before they land</li>
  </ul>
  <button type="button" class="primary continue" onclick={onContinue}>Get started</button>
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
    border-radius: 20px;
    outline: 1px solid rgba(255, 255, 255, 0.1);
    box-shadow: 0 8px 32px rgba(0, 0, 0, 0.35);
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
  .continue {
    margin-top: 0.5rem;
    min-height: 44px;
    padding: 0.6rem 1.75rem;
    font-size: 0.95rem;
    border-radius: 10px;
  }
</style>
