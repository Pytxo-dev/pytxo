import type { Metadata } from "next";
import Link from "next/link";
import { InfoIcon, RocketIcon } from "lucide-react";

import { Badge } from "@/components/ui/badge";
import { Button } from "@/components/ui/button";
import { PlanRoadmapCard } from "@/components/site/plan-roadmap-card";
import { Alert, AlertDescription, AlertTitle } from "@/components/ui/alert";
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
  description:
    "Pytxo is pre-release. OSS core is available now; paid tiers and billing ship at launch.",
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
  },
  {
    name: "Pytxo Pro Cloud",
    status: "At release",
    statusVariant: "secondary" as const,
    highlights: ["Unlimited local agents", "Pytxo Link", "Cloud context caching"],
    detail: "Cloud offload for context and link routing — pricing announced at launch.",
  },
  {
    name: "Pytxo Max Swarm",
    status: "Planned",
    statusVariant: "secondary" as const,
    highlights: [
      "Hosted cloud sandboxes",
      "Heavy parallel pipelines",
      "High-throughput swarms",
    ],
    detail: "For teams running large parallel agent pipelines in isolated sandboxes.",
  },
  {
    name: "Pytxo Ultra",
    status: "Planned",
    statusVariant: "secondary" as const,
    highlights: [
      "Managed metered billing",
      "Pytxo Link proxy",
      "Frontier models + Signal arbitrage",
    ],
    detail: "Managed metering and model routing for production agent workloads.",
  },
] as const;

export default function PlansPage() {
  return (
    <div className="mx-auto max-w-6xl px-4 py-20 sm:px-6 sm:py-28">
      <div className="flex flex-col items-center gap-4 text-center">
        <Badge variant="outline" className="border-white/15 bg-card/30">
          Pre-release
        </Badge>
        <h1 className="max-w-2xl text-4xl font-semibold tracking-tight sm:text-5xl">
          Plans at{" "}
          <span className="chroma-text">launch</span>
        </h1>
        <p className="max-w-xl text-lg text-muted-foreground">
          Paid tiers and billing are not live yet. The OSS core is available now —
          cloud plans ship with the first public release.
        </p>
      </div>

      <Alert className="mt-10 glass-panel border-white/10">
        <InfoIcon />
        <AlertTitle>Billing not live</AlertTitle>
        <AlertDescription>
          Pytxo Core is open source today. Pro, Max, and Ultra tiers — plus enterprise
          options — will be announced when we ship. No charges apply during pre-release.
        </AlertDescription>
      </Alert>

      <Empty className="mt-10 glass-panel chroma-border border-0 p-10">
        <EmptyHeader>
          <EmptyMedia variant="icon" className="chroma-glow size-12 rounded-full bg-card/60">
            <RocketIcon className="size-6 text-primary" />
          </EmptyMedia>
          <EmptyTitle className="text-xl">Early access is open source</EmptyTitle>
          <EmptyDescription className="text-muted-foreground">
            Install the CLI via npm, trust your folder, and orchestrate agents locally.
            Follow release updates on GitHub and npm.
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
        </EmptyContent>
      </Empty>

      <div className="mt-16 flex flex-col gap-3 text-center">
        <h2 className="text-2xl font-semibold tracking-tight">Roadmap</h2>
        <p className="mx-auto max-w-lg text-sm text-muted-foreground">
          Capability preview only — no pricing until release.
        </p>
      </div>

      <div className="mt-8 grid gap-4 sm:grid-cols-2">
        {ROADMAP.map((tier) => (
          <PlanRoadmapCard key={tier.name} {...tier} />
        ))}
      </div>

      <p className="mt-12 text-center text-sm text-muted-foreground">
        Pricing and tier details will be published at launch. See{" "}
        <Link href="/docs/concepts/what-is-pytxo" className="text-primary hover:underline">
          what is Pytxo
        </Link>{" "}
        for the product model today.
      </p>
    </div>
  );
}
