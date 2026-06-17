<script lang="ts">
  let {
    termEl = $bindable(undefined as HTMLDivElement | undefined),
    signalRetries = [],
  }: {
    termEl?: HTMLDivElement | undefined;
    signalRetries?: string[];
  } = $props();
</script>

<section class="log-pane glass-panel chroma-edge-top">
  <h2 class="panel-title">Telemetry</h2>
  {#if signalRetries.length > 0}
    <div class="signal-retries chroma-edge-top">
      <span class="signal-retries__label">Signal retries ({signalRetries.length})</span>
      <ul>
        {#each signalRetries.slice(-5) as retry}
          <li>{retry}</li>
        {/each}
      </ul>
    </div>
  {/if}
  <div class="term chroma-border" bind:this={termEl}></div>
</section>

<style>
  .log-pane {
    display: flex;
    flex-direction: column;
    min-height: 0;
    padding: 0.75rem;
    margin: 0 0.5rem 0.5rem 0;
    flex: 1;
  }
  .signal-retries {
    padding: 0.4rem 0.6rem;
    margin-bottom: 0.5rem;
    background: color-mix(in oklab, var(--brand-gold) 8%, transparent);
    font-size: 0.8rem;
    border-radius: 6px;
  }
  .signal-retries__label {
    color: var(--brand-gold);
    font-weight: 600;
  }
  .signal-retries ul {
    margin: 0.25rem 0 0;
    padding-left: 1rem;
  }
  .term {
    flex: 1;
    min-height: 200px;
    background: #010409;
    border-radius: 8px;
    padding: 4px;
  }
</style>
