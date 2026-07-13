<script lang="ts">
  import { onMount } from "svelte";
  import { check, type Update } from "@tauri-apps/plugin-updater";
  import { relaunch } from "@tauri-apps/plugin-process";
  import { Button } from "$lib/components/ui/button";

  let update: Update | null = $state(null);
  let checking = $state(true);
  let installing = $state(false);
  let error = $state("");

  function isBenignUpdaterError(message: string): boolean {
    const lower = message.toLowerCase();
    return (
      lower.includes("release json") ||
      lower.includes("could not fetch") ||
      lower.includes("network") ||
      lower.includes("404") ||
      lower.includes("not found") ||
      lower.includes("disabled")
    );
  }

  onMount(async () => {
    try {
      const found = await check();
      if (found) update = found;
    } catch (e) {
      const msg = String(e);
      if (!isBenignUpdaterError(msg)) {
        error = msg;
      }
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
  <div class="update-banner" role="status">
    <span>Update available</span>
    <span class="version tabular-nums">v{update.version}</span>
    <Button size="sm" disabled={installing} onclick={installUpdate}>
      {installing ? "Installing…" : "Restart to update"}
    </Button>
  </div>
{:else if error && !checking && !isBenignUpdaterError(error)}
  <div class="update-banner update-banner--muted" role="status">
    <span class="muted">Updater: {error}</span>
  </div>
{/if}

<style>
  .update-banner {
    display: flex;
    align-items: center;
    gap: 0.75rem;
    padding: 0.35rem 0.75rem;
    font-size: 0.8rem;
    border-bottom: 1px solid var(--border);
    background: color-mix(in oklab, var(--primary) 8%, var(--background));
  }
  .update-banner--muted {
    opacity: 0.6;
  }
  .version {
    color: var(--muted-foreground);
  }
  .muted {
    color: var(--muted-foreground);
    font-size: 0.75rem;
  }
</style>
