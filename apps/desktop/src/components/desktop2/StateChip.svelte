<script lang="ts">
  import type { StateTone } from "../../lib/epistemic";

  /** `attention` is a separate channel from the four epistemic values: it means a
   * decision is required, not that a claim is weak. */
  type Tone = StateTone | "attention";

  let {
    tone,
    label,
    detail = null,
    layout = "inline",
  }: { tone: Tone; label: string; detail?: string | null; layout?: "inline" | "stacked" } = $props();

  /**
   * State must be legible without colour, so every tone also differs in fill
   * treatment. Hatch is reserved for `unknown` and is legal only inside a marker,
   * a surface row, or a progress track — never as a background or divider.
   */
  const TREATMENT: Record<Tone, "fill" | "outline" | "dashed" | "hatch"> = {
    verified: "fill",
    refuted: "fill",
    attention: "fill",
    active: "fill",
    claimed: "outline",
    queued: "dashed",
    unknown: "hatch",
  };

  const treatment = $derived(TREATMENT[tone]);
</script>

<span class="chip" class:stacked={layout === "stacked"} data-tone={tone}>
  <span class="mark" data-treatment={treatment} aria-hidden="true"></span>
  <span class="copy">
    <strong>{label}</strong>
    {#if detail}<small>{detail}</small>{/if}
  </span>
</span>

<style>
  .chip {
    display: inline-flex;
    min-width: 0;
    align-items: center;
    gap: 7px;
  }

  .mark {
    width: 9px;
    height: 9px;
    flex: none;
    border: 1px solid var(--tone);
    border-radius: 2px;
  }

  .mark[data-treatment="fill"] {
    background: var(--tone);
  }

  .mark[data-treatment="outline"] {
    background: transparent;
  }

  .mark[data-treatment="dashed"] {
    border-style: dashed;
    background: transparent;
  }

  .mark[data-treatment="hatch"] {
    background: repeating-linear-gradient(45deg, var(--tone) 0 1px, transparent 1px 3px);
  }

  .copy {
    display: flex;
    min-width: 0;
    align-items: baseline;
    gap: 7px;
  }

  .chip.stacked .copy {
    flex-direction: column;
    align-items: flex-start;
    gap: 2px;
  }
  .chip.stacked strong,
  .chip.stacked small {
    white-space: normal;
    overflow: visible;
    overflow-wrap: anywhere;
    text-overflow: clip;
  }

  strong {
    overflow: hidden;
    color: var(--pytxo-text-strong);
    font-size: 12px;
    font-weight: 600;
    text-overflow: ellipsis;
    white-space: nowrap;
  }

  small {
    overflow: hidden;
    color: var(--pytxo-text-muted);
    font-size: 11px;
    line-height: 1.35;
    text-overflow: ellipsis;
    white-space: nowrap;
  }
</style>
