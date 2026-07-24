import Link from "next/link";

import { GITHUB_URL, NPM_URL } from "@/lib/site";

const MARKS = [
  { label: "GitHub", href: GITHUB_URL },
  { label: "npm", href: NPM_URL },
  { label: "Claude Code", href: null },
  { label: "Codex", href: null },
  { label: "Cursor MCP", href: "/docs/getting-started/mcp-from-cursor" },
] as const;

export function ProofBand() {
  return (
    <section
      className="relative border-y border-border/80 bg-card/20"
      aria-label="Trust and measured proof"
    >
      <div className="header-chroma-line absolute inset-x-0 top-0" aria-hidden />
      <div className="mx-auto flex max-w-6xl flex-col gap-5 px-4 py-7 sm:flex-row sm:items-center sm:justify-between sm:px-6 sm:py-8">
        <ul className="flex flex-wrap items-center gap-x-5 gap-y-2">
          {MARKS.map((mark) => (
            <li key={mark.label}>
              {mark.href ? (
                <Link
                  href={mark.href}
                  className="text-xs font-medium uppercase tracking-[0.14em] text-muted-foreground transition-colors hover:text-foreground"
                  {...(mark.href.startsWith("http")
                    ? { target: "_blank", rel: "noopener noreferrer" }
                    : {})}
                >
                  {mark.label}
                </Link>
              ) : (
                <span className="text-xs font-medium uppercase tracking-[0.14em] text-muted-foreground/80">
                  {mark.label}
                </span>
              )}
            </li>
          ))}
        </ul>
        <p className="text-sm text-muted-foreground">
          <span
            className="font-mono text-base font-semibold tabular-nums"
            style={{ color: "var(--brand-teal)" }}
          >
            4.8%
          </span>{" "}
          context on a tiny Signal fixture
          <span className="text-muted-foreground/70"> · aspirational ~60% on large files</span>
        </p>
      </div>
    </section>
  );
}
