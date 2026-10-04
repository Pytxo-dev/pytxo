<script lang="ts">
  import type { Snippet } from "svelte";
  import { getContext, onMount } from "svelte";
  import { Button } from "$lib/components/ui/button";
  const navigation = getContext<{ canGoBack: boolean; back: () => void } | undefined>("setup-navigation");
  let { children, actions, centered = false }: { children: Snippet; actions: Snippet; centered?: boolean } = $props();
  let content: HTMLDivElement;
  let footer: HTMLElement;
  // Enter continues when focus is not already on a control.
  function continueOnEnter(event: KeyboardEvent) {
    if (event.key !== "Enter" || event.defaultPrevented || (event.target as HTMLElement).closest("input, textarea, select, button, a, [contenteditable]")) return;
    const primary = footer?.querySelector<HTMLButtonElement>("button:not(:disabled)");
    if (primary) { event.preventDefault(); primary.click(); }
  }
  onMount(() => {
    const heading = content.querySelector<HTMLElement>("h1, h2");
    heading?.setAttribute("tabindex", "-1");
    heading?.focus({ preventScroll: true });
  });
</script>

<svelte:window onkeydown={continueOnEnter} />

<section class="setup-step" class:centered>
  <div class="setup-step__content" bind:this={content}>{@render children()}</div>
  <footer class="setup-step__actions" bind:this={footer}>
    {@render actions()}
    {#if navigation?.canGoBack}<Button variant="ghost" onclick={navigation.back}>Back</Button>{/if}
  </footer>
</section>

<style>
  .setup-step { display: flex; flex-direction: column; gap: 28px; width: min(560px, 100%); max-height: 100%; margin: auto; padding: 24px 0; }
  .setup-step__content { display: grid; align-content: start; gap: 18px; min-width: 0; min-height: 0; overflow: auto; padding: 2px; }
  .setup-step__actions { flex: none; display: flex; flex-wrap: wrap; align-items: center; gap: 10px; }
  .centered .setup-step__content { justify-items: center; text-align: center; }
  .centered .setup-step__actions { justify-content: center; }
  .setup-step__actions :global(button), .setup-step__actions :global(a) { min-height: 40px; }
  .setup-step__actions :global(button) { font-size: 14px; font-weight: 500; }
  .setup-step__actions :global(button:first-child) { padding-inline: 18px; }
  .setup-step__content :global(.kicker) { margin: 0; color: var(--muted-foreground); font: 500 11px var(--font-mono, "IBM Plex Mono", monospace); letter-spacing: .12em; text-transform: uppercase; }
  .setup-step__content :global(.title) { margin: 0; font-size: 32px; font-weight: 600; line-height: 1.12; letter-spacing: -0.025em; text-wrap: balance; }
  .setup-step__content :global(.lead) { max-width: 54ch; margin: 0; color: var(--muted-foreground); font-size: 15px; line-height: 1.6; text-wrap: pretty; }
  .setup-step__content :global(h1[tabindex="-1"]:focus), .setup-step__content :global(h2[tabindex="-1"]:focus) { outline: none; }
  @media (max-height: 640px) { .setup-step { gap: 16px; padding: 12px 0; } .setup-step__content :global(.title) { font-size: 26px; } }
  @media (prefers-reduced-motion: reduce) { .setup-step :global(.spinner) { animation: none; } }
</style>
