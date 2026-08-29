import Link from "next/link";

import { InstallSnippet } from "@/components/site/install-snippet";
import { StateChip, type StateTone } from "@/components/site/state-chip";
import { Button } from "@/components/ui/button";
import { NPM_INSTALL } from "@/lib/site";

/**
 * Availability is stated the same way the product states enforcement: shipped,
 * partial, or not yet, never rounded up to a promise.
 */
const AVAILABILITY = [
  { label: "CLI, all platforms", tone: "verified" as StateTone, status: "Available" },
  { label: "Desktop, Windows", tone: "verified" as StateTone, status: "Available" },
  { label: "Desktop, macOS and Linux", tone: "unknown" as StateTone, status: "Not built yet" },
  { label: "Managed cloud execution", tone: "claimed" as StateTone, status: "Configured deployments only" },
] as const;

export function GetItSection() {
  return (
    <section className="border-t border-[var(--aperture-line)]" aria-labelledby="get-it-title" data-testid="get-it-section">
      <div className="section-pad mx-auto max-w-[92rem] lg:px-10">
        <div className="grid gap-14 lg:grid-cols-[1fr_0.85fr] lg:gap-20">
          <div>
            <h2
              id="get-it-title"
              className="max-w-[20ch] text-[clamp(2.25rem,4vw,4rem)] leading-[1.02] tracking-[-0.05em]"
            >
              Start with the CLI. Add Desktop when you want the boundary on screen.
            </h2>
            <p className="mt-8 max-w-[38rem] text-base leading-relaxed text-[#a9a9b2] sm:text-lg">
              The CLI does the orchestration: planning, waves, isolation, and Apply.
              Pytxo Desktop is an optional control surface over the same local core, so
              installing it never changes what a run is allowed to do.
            </p>
            <InstallSnippet className="mt-9 max-w-md">{NPM_INSTALL}</InstallSnippet>
            <div className="mt-9 flex flex-col items-start gap-4 sm:flex-row sm:items-center sm:gap-6">
              <Button size="lg" className="h-11 w-full rounded-[4px] bg-white px-6 text-black hover:bg-white/85 sm:w-auto" asChild>
                <Link href="/download">Download Pytxo</Link>
              </Button>
              <Link
                href="/docs/getting-started/first-three-agent-run"
                className="aperture-link text-sm text-[#a9a9b2] transition-colors hover:text-white"
              >
                Your first three-agent run
              </Link>
            </div>
          </div>

          <div className="aperture-panel h-fit p-7 sm:p-9">
            <h3 className="font-mono text-[12px] uppercase tracking-[0.05em] text-[#7d7d87]">
              What ships today
            </h3>
            <ul className="mt-6 divide-y divide-[var(--aperture-line)]">
              {AVAILABILITY.map((row) => (
                <li key={row.label} className="flex flex-wrap items-baseline justify-between gap-x-6 gap-y-1 py-4 first:pt-0 last:pb-0">
                  <span className="text-[15px] text-[#f5f5f7]">{row.label}</span>
                  <StateChip tone={row.tone} label={row.status} />
                </li>
              ))}
            </ul>
            <p className="mt-7 border-t border-[var(--aperture-line)] pt-6 text-sm leading-relaxed text-[#8d8d96]">
              Longer term, Pytxo aims to be the commit layer for autonomous work: typed
              effect contracts at the boundary to systems beyond a repository. That is a
              stated direction, not a shipped feature.
            </p>
            <Link
              href="/plans"
              className="aperture-link mt-6 inline-block text-sm text-[#a9a9b2] transition-colors hover:text-white"
            >
              Plans and availability
            </Link>
          </div>
        </div>
      </div>
    </section>
  );
}
