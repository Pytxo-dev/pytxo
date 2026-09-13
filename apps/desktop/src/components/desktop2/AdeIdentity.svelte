<script lang="ts">
  let { id }: { id: string } = $props();
  const sources: Record<string, string> = {
    cursor: "/ade/cursor.ico",
    opencode: "/ade/opencode.png",
    gemini: "/ade/gemini.ico",
  };
  let failed = $state(false);
  const source = $derived(sources[id]);
</script>

{#if source && !failed}
  <span class="ade-identity" aria-hidden="true"><img src={source} alt="" width="20" height="20" onerror={() => { failed = true; }} /></span>
{/if}

<style>
  .ade-identity { display: inline-flex; align-items: center; justify-content: center; width: 28px; height: 28px; flex: none; border-radius: 5px; background: #fafafa; }
  img { display: block; object-fit: contain; }
</style>
