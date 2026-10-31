/**
 * The Pytxo mark: two strokes, and the dot where they meet in the signal gradient. `id` keeps each copy's
 * gradient and knockout apart; `--mk` recolors the strokes.
 */
export function Mark({ id, className }: { id: string; className?: string }) {
  return (
    <svg viewBox="0 0 512 512" className={className} aria-hidden="true" focusable="false">
      <defs>
        <linearGradient id={`${id}-signal`}>
          <stop offset="0" stopColor="#f04da3" />
          <stop offset="1" stopColor="#ff6a4a" />
        </linearGradient>
        <mask id={`${id}-knock`} maskUnits="userSpaceOnUse" x="0" y="0" width="512" height="512">
          <rect width="512" height="512" fill="#fff" />
          <circle cx="278.5" cy="300" r="93.5" />
        </mask>
      </defs>
      <path
        style={{ fill: "var(--mk, #eceae4)" }}
        mask={`url(#${id}-knock)`}
        d="M86 36H207L427 476H306ZM101.4 476H222.4L310.4 300H189.4Z"
      />
      <circle cx="278.5" cy="300" r="66" fill={`url(#${id}-signal)`} />
    </svg>
  );
}
