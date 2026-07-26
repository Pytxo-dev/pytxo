<script lang="ts">
  import { setUiDensity, setUiScale, UI_SCALES, uiPrefs, type UiDensity } from "../../lib/ui-prefs.svelte";
  import { Button } from "$lib/components/ui/button";

  let { onContinue }: { onContinue: () => void } = $props();

  const densities: { value: UiDensity; label: string }[] = [
    { value: "compact", label: "Compact" },
    { value: "comfortable", label: "Comfortable" },
  ];
</script>

<div class="step">
  <h2 class="title">Set your display</h2>
  <p class="lead">
    Defaults to compact density at 100% scale. Drop to 90% if you want more on screen, or change later
    in Settings → Appearance.
  </p>

  <div class="field">
    <span class="field-label">Scale</span>
    <div class="segmented">
      {#each UI_SCALES as opt (opt.value)}
        <button class:active={uiPrefs.scale === opt.value} onclick={() => setUiScale(opt.value)}>{opt.label}</button>
      {/each}
    </div>
  </div>

  <div class="field">
    <span class="field-label">Density</span>
    <div class="segmented">
      {#each densities as opt (opt.value)}
        <button class:active={uiPrefs.density === opt.value} onclick={() => setUiDensity(opt.value)}>{opt.label}</button>
      {/each}
    </div>
  </div>

  <Button class="wide" onclick={onContinue}>Continue</Button>
</div>

<style>
  .step {
    display: flex;
    flex-direction: column;
    align-items: center;
    text-align: center;
    gap: 1.1rem;
    max-width: 420px;
    margin: 0 auto;
    width: 100%;
  }
  .title {
    margin: 0;
    font-size: 1.35rem;
    text-wrap: balance;
  }
  .lead {
    margin: 0;
    color: var(--muted-foreground);
    font-size: 0.9rem;
    text-wrap: pretty;
    line-height: 1.5;
  }
  .field {
    width: 100%;
    display: flex;
    flex-direction: column;
    gap: 0.4rem;
    align-items: flex-start;
  }
  .field-label {
    font-size: 0.7rem;
    font-weight: 600;
    letter-spacing: 0.06em;
    text-transform: uppercase;
    color: var(--muted-foreground);
  }
  .segmented {
    display: flex;
    width: 100%;
    padding: 2px;
    border: 1px solid var(--border);
    border-radius: calc(var(--panel-radius) - 2px);
    background: color-mix(in oklab, var(--card) 60%, transparent);
  }
  .segmented button {
    flex: 1;
    border: 0;
    border-radius: calc(var(--panel-radius) - 4px);
    background: none;
    color: var(--muted-foreground);
    font-size: 0.78rem;
    padding: 0.4rem 0;
    cursor: pointer;
    transition: background-color 150ms ease, color 150ms ease;
  }
  .segmented button:hover {
    color: var(--foreground);
  }
  .segmented button.active {
    background: var(--primary);
    color: var(--primary-foreground);
  }
  :global(.wide) {
    min-width: 220px;
    margin-top: 0.25rem;
  }
</style>
