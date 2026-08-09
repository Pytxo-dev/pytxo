import Link from "next/link";
import { Braces, GitMerge, ShieldCheck, type LucideIcon } from "lucide-react";

import { Badge } from "@/components/ui/badge";
import { cn } from "@/lib/utils";

type Accent = "teal" | "gold" | "violet";

const ACCENT_VAR: Record<Accent, string> = {
  teal: "var(--brand-teal)",
  gold: "var(--brand-gold)",
  violet: "var(--brand-violet)",
};

type Moat = {
  title: string;
  badge: string;
  description: string;
  href: string;
  icon: LucideIcon;
  accent: Accent;
  stat?: { value: string; label: string };
};

const MOATS: Moat[] = [
  {
    title: "Structure before source",
    badge: "Signal Core",
    description:
      "Starts reads with AST skeletons (signatures, types, and imports) so agents pull full source only when the task needs it.",
    href: "/docs/concepts/signal-core",
    icon: Braces,
    accent: "teal",
    stat: {
      value: "82.9%",
      label:
        "measured scaffold-byte reduction across 185 tracked production files; model-token and task impact are not measured",
    },
  },
  {
    title: "Prepared changes, one Apply",
    badge: "Blast Shield",
    description:
      "Orbit and Galaxy keep writes isolated. Within one execution domain and one repository root, Pytxo stores target blobs, validates affected paths, and journals Apply.",
    href: "/docs/concepts/blast-shield",
    icon: ShieldCheck,
    accent: "gold",
  },
  {
    title: "Schedule overlapping paths in order",
    badge: "Race Shield",
    description:
      "Path claims and dependencies become execution waves, so tasks that touch the same area do not run as independent work.",
    href: "/docs/concepts/race-shield",
    icon: GitMerge,
    accent: "violet",
  },
];

const ALSO_INCLUDED = [
  {
    title: "Mission planning with Flow",
    description:
      "Compose a mission, review tasks, paths, permissions, agent assignments, and waves, then dispatch the approved plan.",
    href: "/docs/concepts/desktop#pytxo-flow",
  },
  {
    title: "Flow and Run Review",
    description:
      "Track active work and history in Flow. Run Review shows the package, exact file changes, policy evidence, recovery state, and Apply history.",
    href: "/docs/concepts/desktop",
  },
  {
    title: "Your agents keep their sessions",
    description:
      "Codex, Claude Code, Cursor, Gemini, OpenCode, and generic commands stay vendor-owned while Pytxo coordinates execution.",
    href: "/docs/reference/providers-byok",
  },
  {
    title: "Workspaces and fleet runs",
    description:
      "Coordinate folders for planning and execution, or order separate repositories in a fleet. Each repository root keeps its own Apply boundary.",
    href: "/docs/concepts/modular-projects",
  },
] as const;

function MoatCard({ moat, className }: { moat: Moat; className?: string }) {
  const Icon = moat.icon;
  const accentVar = ACCENT_VAR[moat.accent];

  return (
    <div
      className={cn(
        "relative flex flex-col gap-4 overflow-hidden rounded-[var(--radius-lg)] border border-border p-6 sm:p-7",
        className,
      )}
      style={{ background: `color-mix(in oklab, ${accentVar} 4%, var(--background))` }}
    >
      <div
        className="absolute inset-x-0 top-0 h-px"
        style={{ background: `color-mix(in oklab, ${accentVar} 55%, transparent)` }}
        aria-hidden
      />

      <div className="flex items-start justify-between gap-3">
        <span
          className="flex size-10 shrink-0 items-center justify-center rounded-[var(--radius-md)] border"
          style={{
            borderColor: `color-mix(in oklab, ${accentVar} 35%, transparent)`,
            background: `color-mix(in oklab, ${accentVar} 14%, transparent)`,
            color: accentVar,
          }}
        >
          <Icon className="size-5" />
        </span>
        <Badge variant="secondary" className="shrink-0 bg-white/5">
          {moat.badge}
        </Badge>
      </div>

      <div className="flex flex-col gap-2">
        <h3 className="text-lg font-semibold tracking-tight">{moat.title}</h3>
        <p className="max-w-md text-sm leading-relaxed text-muted-foreground">
          {moat.description}
        </p>
      </div>

      {moat.stat ? (
        <div className="flex items-baseline gap-2 border-t border-border pt-4">
          <span
            className="font-mono text-2xl font-semibold tabular-nums"
            style={{ color: accentVar }}
          >
            {moat.stat.value}
          </span>
          <span className="text-xs text-muted-foreground">{moat.stat.label}</span>
        </div>
      ) : null}

      <Link
        href={moat.href}
        className="mt-auto text-sm font-medium text-primary transition-colors hover:text-foreground"
      >
        Learn more
      </Link>
    </div>
  );
}

export function FeatureGrid() {
  const [signalCore, blastShield, raceShield] = MOATS;

  return (
    <section className="section-pad mx-auto max-w-6xl">
      <div className="flex max-w-2xl flex-col gap-3">
        <h2 className="text-3xl sm:text-4xl">Controls around repository changes</h2>
        <p className="text-muted-foreground">
          Pytxo starts reads from syntax structure, isolates agent writes, assigns path
          ownership, and prepares one package for review.
        </p>
      </div>

      <div className="mt-12 grid gap-4 lg:grid-cols-2">
        <MoatCard moat={signalCore} className="lg:col-span-2" />
        <MoatCard moat={blastShield} />
        <MoatCard moat={raceShield} />
      </div>

      <div className="mt-16 flex max-w-2xl flex-col gap-3">
        <h3 className="text-2xl font-semibold tracking-tight">
          Fits the tools you already use
        </h3>
        <p className="text-muted-foreground">
          Keep your editor, agent accounts, and provider keys. Add the coordination layer
          around them.
        </p>
      </div>
      <div className="mt-8 grid gap-px overflow-hidden rounded-[var(--radius-lg)] border border-border bg-border sm:grid-cols-2">
        {ALSO_INCLUDED.map((item) => (
          <div key={item.title} className="flex h-full flex-col gap-3 bg-background p-6">
            <h4 className="font-semibold tracking-tight">{item.title}</h4>
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
