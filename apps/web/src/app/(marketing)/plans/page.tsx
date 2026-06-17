import type { Metadata } from "next";
import Link from "next/link";
import { RocketIcon } from "lucide-react";

import { Badge } from "@/components/ui/badge";
import { Button } from "@/components/ui/button";
import { PlanRoadmapCard } from "@/components/site/plan-roadmap-card";
import {
  Empty,
  EmptyContent,
  EmptyDescription,
  EmptyHeader,
  EmptyMedia,
  EmptyTitle,
} from "@/components/ui/empty";
import { GITHUB_URL } from "@/lib/site";

export const metadata: Metadata = {
  title: "Plans",
  description: "Pytxo Core is open source. Subscribe to Pro, Max, or Ultra for cloud features.",
};

const ROADMAP = [
  {
    name: "Pytxo Core",
    status: "Available now",
    statusVariant: "default" as const,
    highlights: [
      "Open-source Rust CLI + desktop shell",
      "Tree-sitter scaffolding",
      "Local agent swarms (BYOK)",
    ],
    detail:
      "The full local hypervisor ships open source. Install from GitHub and run headless agents today.",
    plan: null as null,
  },
  {
    name: "Pytxo Pro Cloud",
    status: "Subscribe",
    statusVariant: "secondary" as const,
    highlights: ["Unlimited local agents", "Pytxo Link", "Cloud context caching"],
    detail: "Cloud offload for context and link routing.",
    plan: "pro" as const,
  },
  {
    name: "Pytxo Max Swarm",
    status: "Subscribe",
    statusVariant: "secondary" as const,
    highlights: [
      "Hosted cloud sandboxes",
      "Heavy parallel pipelines",
      "High-throughput swarms",
    ],
    detail: "For teams running large parallel agent pipelines in isolated sandboxes.",
    plan: "max" as const,
  },
  {
    name: "Pytxo Ultra",
    status: "Subscribe",
    statusVariant: "secondary" as const,
    highlights: [
      "Managed metered billing",
      "Pytxo Link proxy",
      "Frontier models + Signal arbitrage",
    ],
    detail: "Managed metering and model routing for production agent workloads.",
    plan: "ultra" as const,
  },
] as const;

export default function PlansPage() {
  return (
    <div className="mx-auto max-w-6xl px-4 py-20 sm:px-6 sm:py-28">
      <div className="flex flex-col items-center gap-4 text-center">
        <Badge variant="outline" className="border-white/15 bg-card/30">
          Live billing
        </Badge>
        <h1 className="max-w-2xl text-4xl font-semibold tracking-tight sm:text-5xl">
          Plans &{" "}
          <span className="chroma-text">subscriptions</span>
        </h1>
        <p className="max-w-xl text-lg text-muted-foreground">
          Core stays open source. Subscribe to unlock Pytxo Link, cloud caching, and hosted
          sandboxes.
        </p>
      </div>

      <Empty className="mt-10 glass-panel chroma-border border-0 p-10">
        <EmptyHeader>
          <EmptyMedia variant="icon" className="chroma-glow size-12 rounded-full bg-card/60">
            <RocketIcon className="size-6 text-primary" />
          </EmptyMedia>
          <EmptyTitle className="text-xl">Start with Core — upgrade anytime</EmptyTitle>
          <EmptyDescription className="text-muted-foreground">
            Install the CLI via npm, trust your folder, and orchestrate agents locally. Sign in
            to subscribe; entitlements sync to Pytxo Link within a minute.
          </EmptyDescription>
        </EmptyHeader>
        <EmptyContent className="flex flex-col gap-3 sm:flex-row">
          <Button size="lg" className="chroma-glow" asChild>
            <a href={GITHUB_URL} target="_blank" rel="noopener noreferrer">
              Releases on GitHub
            </a>
          </Button>
          <Button size="lg" variant="outline" className="border-white/15 bg-card/30" asChild>
            <Link href="/download">Install the CLI</Link>
          </Button>
          <Button size="lg" variant="outline" className="border-white/15 bg-card/30" asChild>
            <Link href="/account">Sign in</Link>
          </Button>
        </EmptyContent>
      </Empty>

      <div className="mt-16 flex flex-col gap-3 text-center">
        <h2 className="text-2xl font-semibold tracking-tight">Choose a tier</h2>
        <p className="mx-auto max-w-lg text-sm text-muted-foreground">
          Checkout is handled securely via MBCZ. Requires a signed-in pytxo.com account.
        </p>
      </div>

      <div className="mt-8 grid gap-4 sm:grid-cols-2">
        {ROADMAP.map((tier) => (
          <div key={tier.name} className="flex flex-col gap-3">
            <PlanRoadmapCard
              name={tier.name}
              status={tier.status}
              statusVariant={tier.statusVariant}
              highlights={tier.highlights}
              detail={tier.detail}
            />
            {tier.plan ? (
              <Button className="chroma-glow w-full" asChild>
                <Link href={`/api/billing/checkout-redirect?plan=${tier.plan}`}>
                  Subscribe to {tier.name}
                </Link>
              </Button>
            ) : null}
          </div>
        ))}
      </div>

      <p className="mt-12 text-center text-sm text-muted-foreground">
        See{" "}
        <Link href="/docs/concepts/what-is-pytxo" className="text-primary hover:underline">
          what is Pytxo
        </Link>{" "}
        for the product model.
      </p>
    </div>
  );
}
