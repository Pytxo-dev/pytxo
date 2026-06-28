<script lang="ts">
  let {
    cmd = $bindable(""),
    dispatchRepo = $bindable(""),
    onDryRun,
    onDispatch,
    onStop,
  }: {
    cmd?: string;
    dispatchRepo?: string;
    onDryRun: () => void;
    onDispatch: () => void;
    onStop: () => void;
  } = $props();
</script>

<div class="command-bar glass-panel">
  <input
    bind:value={dispatchRepo}
    placeholder="Repo path (blank = workspace)"
    title="Absolute path to a project folder"
    aria-label="Repository path"
  />
  <input bind:value={cmd} placeholder="Command" aria-label="Dispatch command" class="cmd" />
  <div class="actions">
    <button type="button" onclick={onDryRun}>Dry run</button>
    <button type="button" class="primary chroma-glow" onclick={onDispatch}>Dispatch</button>
    <button type="button" onclick={onStop}>Stop</button>
  </div>
</div>

<style>
  .command-bar {
    display: flex;
    align-items: center;
    gap: 0.5rem;
    flex-wrap: wrap;
    margin: 0 0.5rem 0.5rem;
    padding: 0.65rem 0.85rem;
    border-radius: 14px;
    box-shadow: 0 4px 20px rgba(0, 0, 0, 0.18);
  }
  input {
    flex: 1;
    min-width: 140px;
    min-height: 40px;
    border-radius: 10px;
  }
  .cmd {
    flex: 1.5;
    min-width: 180px;
  }
  .actions {
    display: flex;
    gap: 0.4rem;
  }
  .actions button {
    min-height: 40px;
    min-width: 40px;
    border-radius: 10px;
    transition: transform 0.15s cubic-bezier(0.2, 0, 0, 1);
  }
  .actions button:active {
    transform: scale(0.96);
  }
</style>
