<script lang="ts">
  import { onMount } from "svelte";
  import { desktopUpdater } from "$lib/desktop-updater";
  import UpdateControls from "./UpdateControls.svelte";
  onMount(() => { void desktopUpdater.check(); });
</script>

{#if $desktopUpdater.version || $desktopUpdater.error || $desktopUpdater.notice}
  <div class="update-banner"><UpdateControls compact /></div>
{/if}

<style>
  .update-banner {
    padding: 8px 16px;
    border-bottom: 1px solid var(--pytxo-line);
    background: var(--pytxo-surface-panel);
  }
</style>
