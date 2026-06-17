<script lang="ts">
  import { onMount } from "svelte";
  import { check, type Update } from "@tauri-apps/plugin-updater";
  import { relaunch } from "@tauri-apps/plugin-process";

  let update: Update | null = $state(null);
  let checking = $state(true);
  let installing = $state(false);
  let error = $state("");

  onMount(async () => {
    try {
      const found = await check();
      if (found) update = found;
    } catch (e) {
      error = String(e);
    } finally {
      checking = false;
    }
  });

  async function installUpdate() {
    if (!update) return;
    installing = true;
    try {
      await update.downloadAndInstall();
      await relaunch();
    } catch (e) {
      error = String(e);
      installing = false;
    }
  }
</script>

{#if update}
  <div class="update-banner glass-panel chroma-border" role="status">
    <span class="chroma-text">Update available</span>
    <span class="version">v{update.version}</span>
    <button class="primary chroma-glow" disabled={installing} onclick={installUpdate}>
      {installing ? "Installing…" : "Restart to update"}
    </button>
  </div>
{:else if error && !checking}
  <div class="update-banner update-banner--muted" role="status">
    <span class="muted">Updater: {error}</span>
  </div>
{/if}

<style>
  .update-banner {
    display: flex;
    align-items: center;
    gap: 0.75rem;
    margin: 0 0.5rem 0.5rem;
    padding: 0.5rem 0.75rem;
    font-size: 0.85rem;
  }
  .update-banner--muted {
    opacity: 0.6;
    padding: 0.25rem 0.75rem;
    margin: 0 0.5rem;
  }
  .version {
    color: var(--muted-foreground);
    font-variant-numeric: tabular-nums;
  }
  .muted {
    color: var(--muted-foreground);
    font-size: 0.75rem;
  }
</style>
