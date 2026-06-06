"use client";

import Link from "next/link";

import { Badge } from "@/components/ui/badge";
import {
  Card,
  CardContent,
  CardDescription,
  CardHeader,
  CardTitle,
} from "@/components/ui/card";
import {
  HoverCard,
  HoverCardContent,
  HoverCardTrigger,
} from "@/components/ui/hover-card";

const MOATS = [
  {
    title: "Signal Core",
    badge: "Compression",
    description:
      "Tree-sitter–aware context scaffolding and checkpoint discipline so agents spend tokens on work, not noise.",
    preview: "Scaffold signatures, types, and imports instead of dumping whole files — targeting up to ~60% lower input tokens.",
    href: "/docs/concepts/signal-core",
  },
  {
    title: "Blast Shield",
    badge: "Isolation",
    description:
      "Worktree and overlay isolation so parallel agents do not stomp the same files without explicit merge paths.",
    preview: "Each agent runs in its own git worktree. Overlapping paths trigger separate scheduling waves automatically.",
    href: "/docs/concepts/blast-shield",
  },
  {
    title: "Race Shield",
    badge: "Scheduling",
    description:
      "DAG waves and dependency-aware scheduling instead of naive global locks — throughput without chaos.",
    preview: "Lock-free swarm registry plus stdin buffering keeps monorepo swarms from colliding on writes.",
    href: "/docs/concepts/race-shield",
  },
  {
    title: "MCP hub",
    badge: "Integration",
    description:
      "Local-first MCP routing so your IDE or Cursor session drives orchestration without replacing your editor.",
    preview: "Trigger runs, inspect status, and tail structured logs from Cursor via pytxo-mcp.",
    href: "/docs/getting-started/mcp-from-cursor",
  },
] as const;

export function FeatureGrid() {
  return (
    <section className="mx-auto max-w-6xl px-4 py-20 sm:px-6 sm:py-24">
      <div className="flex flex-col gap-4 text-center">
        <h2 className="text-3xl font-semibold tracking-tight sm:text-4xl">
          Three moats, <span className="chroma-text">one plane</span>
        </h2>
        <p className="mx-auto max-w-2xl text-muted-foreground">
          Future orchestration routes through these layers — not around them. Optional
          Reality Deck surfaces structure; the hypervisor stays headless.
        </p>
      </div>
      <div className="mt-12 grid gap-5 sm:grid-cols-2">
        {MOATS.map((moat) => (
          <HoverCard key={moat.title} openDelay={120} closeDelay={80}>
            <HoverCardTrigger asChild>
              <Card className="glass-panel chroma-edge-top group h-full cursor-default border-white/8 transition duration-300 hover:chroma-glow">
                <CardHeader>
                  <div className="flex items-center justify-between gap-2">
                    <CardTitle>{moat.title}</CardTitle>
                    <Badge variant="secondary" className="bg-white/5">
                      {moat.badge}
                    </Badge>
                  </div>
                  <CardDescription className="text-left leading-relaxed">
                    {moat.description}
                  </CardDescription>
                </CardHeader>
                <CardContent>
                  <Link
                    href={moat.href}
                    className="text-sm font-medium text-primary transition-colors hover:text-foreground"
                  >
                    Learn more →
                  </Link>
                </CardContent>
              </Card>
            </HoverCardTrigger>
            <HoverCardContent className="glass-panel w-80 border-white/10">
              <p className="text-sm leading-relaxed text-muted-foreground">{moat.preview}</p>
            </HoverCardContent>
          </HoverCard>
        ))}
      </div>
    </section>
  );
}
