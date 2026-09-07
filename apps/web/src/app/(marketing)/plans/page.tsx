import Link from "next/link";

import { Button } from "@/components/ui/button";
import { PlanRoadmapCard } from "@/components/site/plan-roadmap-card";
import { SITE_AUTH_ENABLED } from "@/lib/auth-config";
import { pageMetadata } from "@/lib/page-metadata";

export const metadata = pageMetadata(
  "/plans",
  "Plans · Pytxo",
  "Pytxo Core is available now. Paid entitlements are capability-gated while Link, cloud dispatch, and managed metering mature.",
);

const ROADMAP = [
  {
    name: "Pytxo Core",
    status: "Available now",
    statusVariant: "default" as const,
    highlights: [
      "Local Rust core + native Desktop",
      "Tree-sitter scaffolding",
      "One or more existing agent CLIs",
    ],
    detail:
      "Run and review local repository work without a Pytxo account. Agent subscriptions or API usage are billed by your chosen provider. Public downloads are available from the releases repository.",
    plan: null as null,
    ctaLabel: null as null,
  },
  {
    name: "Pytxo Pro Cloud",
    status: "Capability-gated",
    statusVariant: "secondary" as const,
    highlights: [
      "Higher local agent limits",
      "Link entitlements when configured",
      "Cloud context cache roadmap",
    ],
    detail:
      "Checkout provisions the Pro entitlement. Link routing and cloud caching still require their services to be configured.",
    plan: "pro" as const,
    ctaLabel: "Subscribe to Pro",
  },
  {
    name: "Pytxo Max Swarm",
    status: "Configured deployments",
    statusVariant: "secondary" as const,
    highlights: [
      "Higher local swarm limits",
      "Cloud dispatch when configured",
      "Sandbox service when configured",
    ],
    detail:
      "Cloud execution is not a default hosted service yet. It works only in deployments with a configured cloud dispatcher.",
    plan: "max" as const,
    ctaLabel: "Subscribe to Max",
  },
  {
    name: "Pytxo Ultra",
    status: "Local ledger available",
    statusVariant: "secondary" as const,
    highlights: [
      "Local wallet and usage ledger",
      "Managed transport when configured",
      "Link reconciliation target",
    ],
    detail:
      "Ultra mode includes local metering hooks. Managed inference and Link reconciliation are not end-to-end on the default path.",
    plan: "ultra" as const,
    ctaLabel: "Subscribe to Ultra",
  },
] as const;

export default function PlansPage() {
  return (
    <div className="mx-auto max-w-6xl px-4 py-20 sm:px-6 sm:py-28">
      <div className="flex max-w-3xl flex-col gap-4">
        <h1 className="text-4xl font-semibold tracking-tight sm:text-5xl">
          Plans and availability
        </h1>
        <p className="max-w-2xl text-lg text-muted-foreground">
          Core works today. Paid entitlements are available, while cloud dispatch and managed
          metering remain capability-gated.
        </p>
        <div className="mt-2 flex flex-wrap gap-3">
          <Button size="lg" asChild>
            <Link href="/download">Download Pytxo</Link>
          </Button>
          {SITE_AUTH_ENABLED ? (
            <Button size="lg" variant="outline" className="border-border" asChild>
              <Link href="/account">Sign in</Link>
            </Button>
          ) : null}
        </div>
      </div>

      <div className="mt-16 border-t border-border pt-10">
        <h2 className="text-2xl font-semibold tracking-tight">Availability by tier</h2>
        <p className="mt-2 max-w-2xl text-sm text-muted-foreground">
          Checkout provisions a Pytxo entitlement through MBCZ. Runtime availability still
          depends on the configured Link, cloud, and proxy services.
        </p>
      </div>

      <div data-testid="plans-grid" className="mt-8 grid gap-4 md:grid-cols-2">
        {ROADMAP.map((tier) => (
          <div key={tier.name} data-testid="plan-cell" className="flex flex-col gap-3">
            <PlanRoadmapCard
              name={tier.name}
              status={tier.status}
              statusVariant={tier.statusVariant}
              highlights={tier.highlights}
              detail={tier.detail}
            />
            {tier.plan && SITE_AUTH_ENABLED ? (
              <Button className="w-full" asChild>
                <Link href={`/api/billing/checkout-redirect?plan=${tier.plan}`}>
                  {tier.ctaLabel}
                </Link>
              </Button>
            ) : tier.plan ? (
              <Button className="w-full" disabled>
                Account checkout not configured
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
