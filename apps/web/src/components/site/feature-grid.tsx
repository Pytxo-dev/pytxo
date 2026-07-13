import Link from "next/link";

import { Badge } from "@/components/ui/badge";

const MOATS = [
  {
    title: "Less noise, lower token bills",
    badge: "Signal Core",
    description:
      "Sends agents file skeletons (signatures, types, imports) instead of whole files. Often far less input context.",
    href: "/docs/concepts/signal-core",
  },
  {
    title: "Safe parallel sandboxes",
    badge: "Blast Shield",
    description:
      "Each agent works in its own copy. Overlapping file paths get scheduled separately so nothing gets overwritten silently.",
    href: "/docs/concepts/blast-shield",
  },
  {
    title: "Parallel runs without collisions",
    badge: "Race Shield",
    description:
      "Dependency-aware scheduling so agents that must run in order do, without locking your whole repo.",
    href: "/docs/concepts/race-shield",
  },
  {
    title: "Works with Cursor and your IDE",
    badge: "MCP",
    description:
      "Local tool bridge so you can trigger runs and inspect status from the editor you already use.",
    href: "/docs/getting-started/mcp-from-cursor",
  },
] as const;

const SHIPPED = [
  {
    title: "Mission planning with Flow",
    badge: "Flow",
    description:
      "Describe a mission, review the generated tasks, permissions, ADE assignments, and execution waves, then dispatch the approved plan.",
    href: "/docs/concepts/desktop#pytxo-flow",
  },
  {
    title: "Local-first voice capture",
    badge: "Voice",
    description:
      "Turn speech into an editable Flow mission locally. Audio stays in bounded memory and is never written to Pytxo logs or storage.",
    href: "/docs/concepts/desktop#pytxo-voice",
  },
  {
    title: "Multi-folder Workspaces",
    badge: "Multi-root",
    description:
      "One coordinated run across API, web, and shared folders. Each folder can have its own trust level.",
    href: "/docs/concepts/modular-projects",
  },
  {
    title: "Cross-repo fleet runs",
    badge: "Fleet",
    description:
      "Coordinate agents across separate git repos when steps must finish in order, with status in Pytxo Desktop.",
    href: "/docs/concepts/fleet-runs",
  },
  {
    title: "Approval gates for risky actions",
    badge: "Approvals",
    description:
      "Require a human OK before sensitive spawns or applying batched changes, from CLI, Desktop, or your editor.",
    href: "/docs/concepts/galaxy-approvals",
  },
] as const;

export function FeatureGrid() {
  return (
    <section className="section-pad mx-auto max-w-6xl section-reveal">
      <div className="flex max-w-2xl flex-col gap-3">
        <h2 className="text-3xl sm:text-4xl">Built for parallel agents, without the mess</h2>
        <p className="text-muted-foreground">
          Smarter context, isolated sandboxes, and safe parallel writes on every run. Optional
          Desktop shows what is changing, pending approvals, and fleet status.
        </p>
      </div>

      <div className="mt-12 grid gap-px overflow-hidden rounded-[var(--radius-lg)] border border-border bg-border sm:grid-cols-2">
        {MOATS.map((moat, i) => (
          <div
            key={moat.title}
            className={`flex h-full flex-col gap-3 bg-background p-6 ${
              i === 0 ? "chroma-border" : ""
            }`}
          >
            <div className="flex items-start justify-between gap-3">
              <h3 className="text-lg font-semibold tracking-tight">{moat.title}</h3>
              <Badge variant="secondary" className="shrink-0 bg-white/5">
                {moat.badge}
              </Badge>
            </div>
            <p className="text-sm leading-relaxed text-muted-foreground">{moat.description}</p>
            <Link
              href={moat.href}
              className="mt-auto text-sm font-medium text-primary transition-colors hover:text-foreground"
            >
              Learn more
            </Link>
          </div>
        ))}
      </div>

      <div className="mt-16 flex max-w-2xl flex-col gap-3">
        <h3 className="text-2xl font-semibold tracking-tight">Also shipped</h3>
        <p className="text-muted-foreground">
          Text-first Flow, local-first Voice, multi-folder Workspaces, cross-repo fleets, and
          approval gates.
        </p>
      </div>
      <div className="mt-8 grid gap-6 sm:grid-cols-3">
        {SHIPPED.map((item) => (
          <div key={item.title} className="flex flex-col gap-3 border-t border-border pt-5">
            <div className="flex items-center justify-between gap-2">
              <h4 className="font-semibold tracking-tight">{item.title}</h4>
              <Badge variant="secondary" className="bg-white/5">
                {item.badge}
              </Badge>
            </div>
            <p className="text-sm leading-relaxed text-muted-foreground">{item.description}</p>
            <Link
              href={item.href}
              className="mt-auto text-sm font-medium text-primary transition-colors hover:text-foreground"
            >
              Learn more
            </Link>
          </div>
        ))}
      </div>
    </section>
  );
}
