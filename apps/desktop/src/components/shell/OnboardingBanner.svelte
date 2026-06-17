<script lang="ts">
  let { visible = false }: { visible?: boolean } = $props();
</script>

{#if visible}
  <div class="onboarding glass-panel chroma-border" role="alert">
    <p>
      <strong>Pytxo CLI not found on PATH.</strong>
      Install via <code>npm i -g pytxo</code> or download from
      <a href="https://pytxo.com/download" target="_blank" rel="noopener noreferrer">pytxo.com/download</a>,
      then run <code>pytxo doctor</code>.
    </p>
  </div>
{/if}

<style>
  .onboarding {
    margin: 0 0.5rem 0.5rem;
    padding: 0.75rem 1rem;
    font-size: 0.85rem;
  }
  code {
    background: rgba(0, 0, 0, 0.3);
    padding: 0.1rem 0.35rem;
    border-radius: 4px;
  }
  a {
    color: var(--brand-teal);
  }
</style>
