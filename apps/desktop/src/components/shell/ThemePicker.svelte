<script lang="ts">
  import type { DeckTheme } from "../../lib/theme";
  import { DECK_THEMES } from "../../lib/theme";

  let {
    theme = $bindable("void" as DeckTheme),
  }: {
    theme?: DeckTheme;
  } = $props();
</script>

<div class="theme-picker" role="group" aria-label="Theme">
  {#each DECK_THEMES as t}
    <button
      type="button"
      class:selected={theme === t.id}
      onclick={() => (theme = t.id)}
      title={t.label}
    >
      {t.label}
    </button>
  {/each}
</div>

<style>
  .theme-picker {
    display: flex;
    gap: 0.25rem;
    padding: 0.2rem;
    border-radius: 10px;
    background: color-mix(in oklab, var(--card) 70%, transparent);
    box-shadow: 0 1px 2px rgba(0, 0, 0, 0.12);
  }
  button {
    min-height: 36px;
    min-width: 36px;
    padding: 0.35rem 0.55rem;
    font-size: 0.72rem;
    border-radius: 8px;
    border: none;
    background: transparent;
    color: var(--muted-foreground);
    transition: transform 0.15s cubic-bezier(0.2, 0, 0, 1),
      opacity 0.15s cubic-bezier(0.2, 0, 0, 1),
      background 0.15s cubic-bezier(0.2, 0, 0, 1);
  }
  button:hover {
    color: var(--foreground);
    background: color-mix(in oklab, var(--foreground) 8%, transparent);
  }
  button:active {
    transform: scale(0.96);
  }
  button.selected {
    color: var(--brand-teal);
    background: color-mix(in oklab, var(--brand-teal) 14%, transparent);
    box-shadow: inset 0 0 0 1px color-mix(in oklab, var(--brand-teal) 35%, transparent);
  }
</style>
