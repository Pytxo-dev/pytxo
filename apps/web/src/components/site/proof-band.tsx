import Image from "next/image";
import Link from "next/link";

import { GITHUB_URL, NPM_URL } from "@/lib/site";

const MARKS = [
  { label: "GitHub", href: GITHUB_URL, src: "/brands/github.svg", width: 24, height: 24 },
  { label: "npm", href: NPM_URL, src: "/brands/npm.svg", width: 42, height: 18 },
  {
    label: "Claude Code",
    href: null,
    src: "/brands/anthropic.svg",
    width: 30,
    height: 24,
  },
  {
    label: "Cursor MCP",
    href: "/docs/getting-started/mcp-from-cursor",
    src: "/brands/cursor.svg",
    width: 24,
    height: 24,
  },
] as const;

export function ProofBand() {
  return (
    <section
      className="relative border-y border-border bg-card"
      aria-label="Supported tools and measured proof"
    >
      <div className="mx-auto flex max-w-7xl flex-col gap-6 px-4 py-7 lg:flex-row lg:items-center lg:justify-between lg:px-6 lg:py-8">
        <div className="flex flex-col gap-4 sm:flex-row sm:items-center sm:gap-6">
          <p className="text-sm text-muted-foreground">Works with the tools you already use.</p>
          <ul className="flex items-center gap-6" aria-label="Compatible tools">
            {MARKS.map((mark) => (
              <li key={mark.label} className="shrink-0">
                {mark.href ? (
                  <Link
                    href={mark.href}
                    className="block shrink-0 opacity-60 transition-opacity hover:opacity-100"
                    {...(mark.href.startsWith("http")
                      ? { target: "_blank", rel: "noopener noreferrer" }
                      : {})}
                  >
                    <Image
                      src={mark.src}
                      alt={mark.label}
                      width={mark.width}
                      height={mark.height}
                      className="block shrink-0 invert"
                      style={{ width: mark.width, height: mark.height }}
                      unoptimized
                    />
                  </Link>
                ) : (
                  <span className="block shrink-0 opacity-60" title={mark.label}>
                    <Image
                      src={mark.src}
                      alt={mark.label}
                      width={mark.width}
                      height={mark.height}
                      className="block shrink-0 invert"
                      style={{ width: mark.width, height: mark.height }}
                      unoptimized
                    />
                  </span>
                )}
              </li>
            ))}
          </ul>
        </div>
        <Link
          href="/docs/concepts/signal-core"
          className="max-w-xl text-sm leading-relaxed text-muted-foreground transition-colors hover:text-foreground lg:text-right"
        >
          <span
            className="font-mono text-base font-semibold tabular-nums"
            style={{ color: "var(--live)" }}
          >
            82.9%
          </span>{" "}
          measured scaffold-byte reduction across 185 tracked production files. This is
          not a model-token or task-success claim.
        </Link>
      </div>
    </section>
  );
}
