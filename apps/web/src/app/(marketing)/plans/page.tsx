import Link from "next/link";

import { Button } from "@/components/ui/button";
import { PlanRoadmapCard } from "@/components/site/plan-roadmap-card";
import { SITE_AUTH_ENABLED } from "@/lib/auth-config";
import { FREE_RECEIPT_LIMIT, PRO_CHECKOUT_OPEN, PRO_INTERVAL, PRO_PRICE } from "@/lib/billing/pro";
import { pageMetadata } from "@/lib/page-metadata";
import { DISCORD_URL } from "@/lib/site";

export const metadata = pageMetadata(
  "/plans",
  "Plans · Pytxo",
  `Pytxo is free on your machine. A free account adds sharing perks; Pytxo Pro adds private receipts for ${PRO_PRICE}/${PRO_INTERVAL}. Team is in design with partners.`,
);

const ROADMAP = [
  {
    name: "Pytxo",
    status: "Free · available now",
    statusVariant: "default" as const,
    highlights: [
      "Desktop for Windows; CLI for Windows, macOS and Linux",
      "Fleet runs, Review and Apply, pytxo land, receipts, Pybench",
      "Your existing agent CLIs; no Pytxo account",
      "Open source under the MIT license",
    ],
    detail:
      "Everything that runs on your machine is free and stays free. Agent subscriptions or API usage are billed by your chosen provider.",
  },
  {
    name: "Free account",
    status: "Free · new in 1.3",
    statusVariant: "default" as const,
    highlights: [
      "Share a receipt as a public page",
      "Keep shared receipts across machines",
      "Publish Pybench results to the board",
      `Keep up to ${FREE_RECEIPT_LIMIT} shared receipts`,
      "Early access to Team",
    ],
    detail:
      "Optional. Sign in from the CLI (pytxo login) or Desktop Settings when you want hosted perks. Removing your account never removes a local feature.",
  },
  {
    name: "Pro",
    status: `${PRO_PRICE}/${PRO_INTERVAL} · ${PRO_CHECKOUT_OPEN ? "new in 1.3" : "checkout opens soon"}`,
    statusVariant: "default" as const,
    highlights: [
      "Everything in the free account",
      "Private receipts only you can open",
      "Unlimited shared receipts",
      "Private Pybench results, off the public board",
    ],
    detail:
      "Billed monthly through Dodo Payments, our merchant of record. Cancel any time from your account; Pro stays on until the end of the month you paid for.",
  },
  {
    name: "Team",
    status: "Planned · design partners",
    statusVariant: "secondary" as const,
    highlights: [
      "The landing gate on pull requests across a team",
      "Shared receipt history, retention and exports",
      "Policy for what agents may land",
    ],
    detail:
      "Not available to buy. We are building it with a few teams that land agent-written code every week. Tell us about your setup on Discord.",
  },
] as const;

export default function PlansPage() {
  return (
    <div className="mx-auto max-w-6xl px-4 py-20 sm:px-6 sm:py-28">
      <div className="flex max-w-3xl flex-col gap-4">
        <h1 className="text-4xl font-semibold tracking-tight sm:text-5xl">
          Plans
        </h1>
        <p className="max-w-2xl text-lg text-muted-foreground">
          Free on your machine, free to sign in. Pro adds private receipts and unlimited
          history for {PRO_PRICE}/{PRO_INTERVAL}. Team is in design with partners.
        </p>
        <div className="mt-2 flex flex-wrap gap-3">
          <Button size="lg" asChild>
            <Link href="/download">Download Pytxo</Link>
          </Button>
          {SITE_AUTH_ENABLED && PRO_CHECKOUT_OPEN ? (
            <Button size="lg" variant="outline" asChild>
              <a href="/api/billing/checkout-redirect?plan=pro">Get Pro</a>
            </Button>
          ) : null}
          <Button size="lg" variant="outline" asChild>
            <a href={DISCORD_URL}>Talk to us about Team</a>
          </Button>
        </div>
      </div>

      <div data-testid="plans-grid" className="mt-16 grid gap-4 md:grid-cols-2 xl:grid-cols-4">
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
          Signed in already?{" "}
          <Link href="/account" className="font-medium text-foreground underline underline-offset-4 hover:text-primary">
            See your account and shared receipts
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
