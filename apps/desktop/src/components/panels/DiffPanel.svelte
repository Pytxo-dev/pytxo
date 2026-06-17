<script lang="ts">
  let {
    diffText = "",
    dryRunOut = "",
    onLoad,
    onCommit,
  }: {
    diffText?: string;
    dryRunOut?: string;
    onLoad: () => void;
    onCommit: () => void;
  } = $props();
</script>

<section class="diff-pane glass-panel chroma-edge-top">
  <h2 class="panel-title">
    Diff
    <span class="diff-actions">
      <button onclick={onLoad}>Load</button>
      <button class="gold chroma-glow" onclick={onCommit}>Approve merge</button>
    </span>
  </h2>
  <pre class="chroma-border">{diffText || dryRunOut || "Dry-run output appears here after Dry run."}</pre>
</section>

<style>
  .diff-pane {
    display: flex;
    flex-direction: column;
    min-height: 0;
    padding: 0.75rem;
    margin: 0 0.5rem 0.5rem 0;
    width: 320px;
    flex-shrink: 0;
  }
  h2 {
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: 0.5rem;
  }
  .diff-actions {
    display: flex;
    gap: 0.35rem;
  }
  .diff-actions button {
    font-size: 0.75rem;
    padding: 0.2rem 0.5rem;
  }
  pre {
    flex: 1;
    overflow: auto;
    font-size: 12px;
    background: color-mix(in oklab, var(--card) 55%, transparent);
    padding: 0.75rem;
    border-radius: 8px;
    margin: 0;
    font-family: ui-monospace, monospace;
  }
</style>
