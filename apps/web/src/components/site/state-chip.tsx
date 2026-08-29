import { cn } from "@/lib/utils";

/**
 * The four epistemic states are the product's core vocabulary, so the website
 * renders them with the same encoding as Pytxo Desktop: a non-colour treatment
 * first (fill, outline, hatch), colour second, and always a text label.
 */
export type StateTone = "verified" | "claimed" | "unknown" | "refuted" | "attention";

const TREATMENT: Record<StateTone, string> = {
  verified: "bg-[var(--state-verified)] border-[var(--state-verified)]",
  refuted: "bg-[var(--state-refuted)] border-[var(--state-refuted)]",
  attention: "bg-[var(--state-attention)] border-[var(--state-attention)]",
  claimed: "border-[var(--state-claimed)]",
  unknown: "border-[var(--state-unknown)] bg-[repeating-linear-gradient(45deg,var(--state-unknown)_0_1px,transparent_1px_3px)]",
};

const TEXT: Record<StateTone, string> = {
  verified: "text-[var(--state-verified)]",
  refuted: "text-[var(--state-refuted)]",
  attention: "text-[var(--state-attention)]",
  claimed: "text-[var(--state-claimed)]",
  unknown: "text-[var(--state-unknown)]",
};

export function StateChip({
  tone,
  label,
  className,
}: {
  tone: StateTone;
  label: string;
  className?: string;
}) {
  return (
    <span className={cn("inline-flex items-center gap-2 whitespace-nowrap", className)}>
      <span aria-hidden className={cn("size-[9px] shrink-0 rounded-[2px] border", TREATMENT[tone])} />
      <span className={cn("text-[13px] font-medium", TEXT[tone])}>{label}</span>
    </span>
  );
}
