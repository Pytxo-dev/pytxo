<script lang="ts">
  import type { Snippet } from "svelte";
  import { getContext, onMount } from "svelte";
  import { Button } from "$lib/components/ui/button";
  const navigation = getContext<{ canGoBack: boolean; back: () => void } | undefined>("setup-navigation");
  let { children, actions }: { children: Snippet; actions: Snippet } = $props();
  let content: HTMLDivElement;
  onMount(() => {
    const heading = content.querySelector<HTMLElement>("h1, h2");
    heading?.setAttribute("tabindex", "-1");
    heading?.focus({ preventScroll: true });
  });
</script>

<section class="setup-step">
  <div class="setup-step__content" bind:this={content}>{@render children()}</div>
  <footer class="setup-step__actions">
    {#if navigation?.canGoBack}<div class="setup-back"><Button variant="ghost" onclick={navigation.back}>Back</Button></div>{/if}
    <div class="step-actions">{@render actions()}</div>
  </footer>
</section>

<style>
  .setup-step { display: flex; flex-direction: column; width: 100%; height: 100%; min-height: 0; min-width: 0; }
  .setup-step__content { display: flex; flex-direction: column; align-items: stretch; gap: 16px; min-height: 0; overflow: auto; padding: 4px 4px 20px; scrollbar-gutter: stable; }
  .setup-step__actions { margin-top: auto; padding: 16px 4px 0; border-top: 1px solid var(--border); display: flex; flex-wrap: wrap; align-items: center; justify-content: flex-end; gap: 8px; flex: none; }
  .setup-back { margin-right: auto; }
  .step-actions { display: flex; flex-wrap: wrap; justify-content: flex-end; gap: 8px; min-width: 0; }
  .setup-step__content :global(.title) { text-align: left; font-size: 24px; line-height: 1.25; letter-spacing: -0.025em; }
  .setup-step__content :global(.lead) { text-align: left; max-width: 56ch; font-size: 14px; line-height: 1.6; }
  .setup-step__content :global(h1[tabindex="-1"]:focus), .setup-step__content :global(h2[tabindex="-1"]:focus) { outline: none; }
  .setup-step__actions :global(button) { min-height: 40px; }
  .setup-step__content :global(.agents) { flex-shrink: 0; }
  @media (prefers-reduced-motion: reduce) { .setup-step :global(.spinner) { animation: none; } }
</style>
