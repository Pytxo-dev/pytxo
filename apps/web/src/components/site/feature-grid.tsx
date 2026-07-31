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
    title: "Less noise, lower token bills",
    badge: "Signal Core",
    description:
      "Sends agents file skeletons (signatures, types, imports) instead of whole files, so every read starts smaller.",
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
    title: "Safe parallel sandboxes",
    badge: "Blast Shield",
    description:
      "Each agent works in its own copy. Nothing touches your real files until you approve the merge.",
    href: "/docs/concepts/blast-shield",
    icon: ShieldCheck,
    accent: "gold",
  },
  {
    title: "Parallel runs without collisions",
    badge: "Race Shield",
    description:
      "Dependency-aware scheduling so agents that must run in order do, without locking your whole repo.",
    href: "/docs/concepts/race-shield",
    icon: GitMerge,
    accent: "violet",
  },
];

const ALSO_INCLUDED = [
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
    title: "Works with Cursor and your IDE",
    badge: "MCP",
    description:
      "Local tool bridge so you can trigger runs and inspect status from the editor you already use.",
    href: "/docs/getting-started/mcp-from-cursor",
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
        <h2 className="text-3xl sm:text-4xl">Built-in protections for real repositories</h2>
        <p className="text-muted-foreground">
          Every run starts with smaller context, isolated changes, and conflict-aware
          scheduling. Signal, Blast, and Race are the internal names.
        </p>
      </div>

      <div className="mt-12 grid gap-4 lg:grid-cols-2">
        <MoatCard moat={signalCore} className="lg:col-span-2" />
        <MoatCard moat={blastShield} />
        <MoatCard moat={raceShield} />
      </div>

      <div className="mt-16 flex max-w-2xl flex-col gap-3">
        <h3 className="text-2xl font-semibold tracking-tight">Also included</h3>
        <p className="text-muted-foreground">
          Text-first Flow, local-first Voice, editor integration, Workspaces, fleets, and
          approval gates.
        </p>
      </div>
      <div className="mt-8 grid gap-px overflow-hidden rounded-[var(--radius-lg)] border border-border bg-border sm:grid-cols-2">
        {ALSO_INCLUDED.map((item) => (
          <div key={item.title} className="flex h-full flex-col gap-3 bg-background p-6">
            <div className="flex items-start justify-between gap-2">
              <h4 className="font-semibold tracking-tight">{item.title}</h4>
              <Badge variant="secondary" className="shrink-0 bg-white/5">
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
