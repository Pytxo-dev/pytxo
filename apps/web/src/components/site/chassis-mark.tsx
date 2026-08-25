/** Stencil lambda — metal, not the rainbow ribbon mark. */
export function ChassisMark({ className }: { className?: string }) {
  return (
    <svg
      viewBox="0 0 32 32"
      className={className}
      aria-hidden="true"
      focusable="false"
    >
      <path
        fill="currentColor"
        d="M8.4 26.5 15.2 6.8h2.35L10.75 26.5H8.4Zm6.2-10.2 8.85 10.2h-2.55L13.7 17.55l.9-1.25Z"
      />
    </svg>
  );
}
