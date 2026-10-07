import Link from "next/link";

import { Button } from "@/components/ui/button";
import { PlanRoadmapCard } from "@/components/site/plan-roadmap-card";
import { SITE_AUTH_ENABLED } from "@/lib/auth-config";
import { pageMetadata } from "@/lib/page-metadata";

export const metadata = pageMetadata(
  "/plans",
  "Plans · Pytxo",
  "Pytxo Core is available now. Cloud and Teams are not a default hosted product.",
);

const ROADMAP = [
  {
    name: "Pytxo Core",
    status: "Available now",
    statusVariant: "default" as const,
    highlights: [
      "Local Rust core; Windows Desktop MSI",
      "CLI on Windows, macOS, and Linux",
      "Your existing agent CLI; no Pytxo account",
    ],
    detail:
      "Run and review local repository work without a Pytxo account. Agent subscriptions or API usage are billed by your chosen provider. Public downloads are available from the releases repository.",
  },
  {
    name: "Cloud / Teams",
    status: "Not a default hosted product",
    statusVariant: "secondary" as const,
    highlights: [
      "Configured deployments only",
      "Link, cloud dispatch, and metering when those services exist",
      "No public subscription checkout",
    ],
    detail:
      "Cloud execution and team entitlements are not a default hosted product. They work only in deployments that already have those services configured. Local Core does not require checkout.",
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
          Core works today on your machine without checkout. Cloud and Teams require
          a deployment with those services configured.
        </p>
        <div className="mt-2 flex flex-wrap gap-3">
          <Button size="lg" asChild>
            <Link href="/download">Download Pytxo</Link>
          </Button>
        </div>
      </div>

      <div data-testid="plans-grid" className="mt-16 grid gap-4 md:grid-cols-2">
        {ROADMAP.map((tier) => (
          <div key={tier.name} data-testid="plan-cell" className="flex flex-col gap-3">
            <PlanRoadmapCard
              name={tier.name}
              status={tier.status}
              statusVariant={tier.statusVariant}
              highlights={tier.highlights}
              detail={tier.detail}
            />
          </div>
        ))}
      </div>

      {SITE_AUTH_ENABLED ? (
        <p className="mt-8 text-sm text-muted-foreground">
          Already have access through a configured Cloud or Teams deployment?{" "}
          <Link href="/account" className="font-medium text-foreground underline underline-offset-4 hover:text-primary">
            View your account entitlements
          </Link>
          .
        </p>
      ) : null}

      <p className="mt-12 text-sm text-muted-foreground">
        See{" "}
        <Link href="/docs/concepts/what-is-pytxo" className="text-primary hover:underline">
          what is Pytxo
        </Link>{" "}
        for the product model and current boundaries.
      </p>
    </div>
  );
}
