<script lang="ts">
  let {
    termEl = $bindable(undefined as HTMLDivElement | undefined),
    signalRetries = [],
    lightTheme = false,
  }: {
    termEl?: HTMLDivElement | undefined;
    signalRetries?: string[];
    lightTheme?: boolean;
  } = $props();
</script>

<section class="log-pane">
  <h2 class="panel-title">Telemetry</h2>
  {#if signalRetries.length > 0}
    <div class="signal-retries">
      <span class="signal-retries__label">Context retries ({signalRetries.length})</span>
      <ul>
        {#each signalRetries.slice(-5) as retry}
          <li>{retry}</li>
        {/each}
      </ul>
    </div>
  {/if}
  <div class="term" class:term--light={lightTheme} bind:this={termEl}></div>
</section>

<style>
  .log-pane {
    display: flex;
    flex-direction: column;
    min-height: 0;
    padding: 0.5rem 0.75rem;
    flex: 1;
    border-top: 1px solid var(--border);
    background: var(--card);
  }
  .signal-retries {
    padding: 0.4rem 0.6rem;
    margin-bottom: 0.5rem;
    background: color-mix(in oklab, var(--foreground) 4%, transparent);
    border: 1px solid var(--border);
    font-size: var(--text-xs, 0.75rem);
    border-radius: var(--panel-radius);
  }
  .signal-retries__label {
    color: var(--muted-foreground);
    font-weight: 600;
  }
  .signal-retries ul {
    margin: 0.25rem 0 0;
    padding-left: 1rem;
    color: var(--muted-foreground);
  }
  .term {
    flex: 1;
    min-height: 200px;
    background: var(--background);
    border-radius: var(--panel-radius);
    padding: 4px;
    border: 1px solid var(--border);
  }
  .term.term--light {
    background: color-mix(in oklab, var(--background) 92%, white);
  }
</style>
