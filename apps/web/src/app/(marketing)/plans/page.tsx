import type { Metadata } from "next";
import Link from "next/link";

import { Badge } from "@/components/ui/badge";
import { Button } from "@/components/ui/button";
import { PlanRoadmapCard } from "@/components/site/plan-roadmap-card";

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
      "The full local agent hypervisor ships open source. Install the CLI and run agents today.",
    plan: null as null,
    featured: true,
  },
  {
    name: "Pytxo Pro Cloud",
    status: "Subscribe",
    statusVariant: "secondary" as const,
    highlights: ["Pytxo Link entitlements", "Cloud context caching", "`pytxo doctor` connectivity checks"],
    detail: "Cloud offload for context and link routing.",
    plan: "pro" as const,
    featured: false,
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
    featured: false,
  },
  {
    name: "Pytxo Ultra",
    status: "Subscribe",
    statusVariant: "secondary" as const,
    highlights: [
      "Managed metered billing",
      "Pytxo Link proxy",
      "Improved token estimates for Ultra workloads",
    ],
    detail: "Managed metering and model routing for production agent workloads.",
    plan: "ultra" as const,
    featured: false,
  },
] as const;

export default function PlansPage() {
  return (
    <div className="mx-auto max-w-6xl px-4 py-20 sm:px-6 sm:py-28">
      <div className="flex max-w-2xl flex-col gap-4">
        <Badge variant="outline" className="w-fit border-border">
          Live billing
        </Badge>
        <h1 className="text-4xl font-semibold tracking-tight sm:text-5xl">
          Plans and subscriptions
        </h1>
        <p className="text-lg text-muted-foreground">
          Core stays open source. Subscribe to unlock Pytxo Link, cloud caching, and hosted
          sandboxes.
        </p>
        <div className="mt-2 flex flex-wrap gap-3">
          <Button size="lg" asChild>
            <Link href="/download">Download Pytxo</Link>
          </Button>
          <Button size="lg" variant="outline" className="border-border" asChild>
            <Link href="/account">Sign in</Link>
          </Button>
        </div>
      </div>

      <div className="mt-16 border-t border-border pt-10">
        <h2 className="text-2xl font-semibold tracking-tight">Choose a tier</h2>
        <p className="mt-2 max-w-lg text-sm text-muted-foreground">
          Checkout is handled securely via MBCZ. Requires a signed-in pytxo.com account.
        </p>
      </div>

      <div className="mt-8 grid gap-4 sm:grid-cols-2">
        {ROADMAP.map((tier) => (
          <div
            key={tier.name}
            className={`flex flex-col gap-3 ${tier.featured ? "sm:col-span-2" : ""}`}
          >
            <PlanRoadmapCard
              name={tier.name}
              status={tier.status}
              statusVariant={tier.statusVariant}
              highlights={tier.highlights}
              detail={tier.detail}
            />
            {tier.plan ? (
              <Button className="w-full" asChild>
                <Link href={`/api/billing/checkout-redirect?plan=${tier.plan}`}>
                  Subscribe to {tier.name}
                </Link>
              </Button>
            ) : null}
          </div>
        ))}
      </div>

      <p className="mt-12 text-sm text-muted-foreground">
        See{" "}
        <Link href="/docs/concepts/what-is-pytxo" className="text-primary hover:underline">
          what is Pytxo
        </Link>{" "}
        for the product model.
      </p>
    </div>
  );
}
