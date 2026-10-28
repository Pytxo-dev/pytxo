import Link from "next/link";

import { Button } from "@/components/ui/button";
import { PlanRoadmapCard } from "@/components/site/plan-roadmap-card";
import { pageMetadata } from "@/lib/page-metadata";
import { DISCORD_URL, GET_STARTED_HREF } from "@/lib/site";

// Display copy only. Pro is not on sale: the Action's licensing check ships
// switched off, and this page links to no checkout.
const PRO_LAUNCH = "$29 once for the first 500 buyers, then $5/month or $48/year";

export const metadata = pageMetadata(
  "/plans",
  "Plans · Pytxo",
  `Pytxo is free on your machine and on public repositories. Pro, for the GitHub Action on private repositories, is not on sale yet: ${PRO_LAUNCH}. Team is planned.`,
);

const ROADMAP = [
  {
    name: "Free",
    status: "Free · available now",
    statusVariant: "default" as const,
    highlights: [
      "npx pytxo on macOS, Linux and Windows",
      "The Claude Code plugin and the Codex hook",
      "The GitHub Action on public repositories",
      "Open source under the MIT license",
    ],
    detail:
      "Everything that runs on your machine is free and stays free. No account, and Pytxo sends nothing anywhere.",
  },
  {
    name: "Pro",
    status: "$29 once · not on sale yet",
    statusVariant: "default" as const,
    highlights: [
      "The GitHub Action on private repositories",
      "A 14-day trial in each private repository",
      "$29 once for the first 500 buyers",
      "Then $5/month or $48/year",
    ],
    detail:
      "A license key, kept as a repository secret and passed to the Action's license input. Until Pro opens, the Action runs free on private repositories too. If development stops, Pro unlocks for everyone.",
  },
  {
    name: "Team",
    status: "Planned",
    statusVariant: "secondary" as const,
    highlights: [
      "The check across a team's repositories",
      "Shared history of verdicts and receipts",
      "Policy for which agents' branches may merge",
    ],
    detail:
      "Not available to buy. Tell us about your setup on Discord.",
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
          Free on your machine and on public repositories. Pro covers the GitHub Action on
          private repositories and is not on sale yet: {PRO_LAUNCH}. Team is planned.
        </p>
        <div className="mt-2 flex flex-wrap gap-3">
          <Button size="lg" className="touch-target relative" asChild>
            <Link href={GET_STARTED_HREF}>Get started</Link>
          </Button>
          <Button size="lg" variant="outline" className="touch-target relative" asChild>
            <a href={DISCORD_URL}>Talk to us about Team</a>
          </Button>
        </div>
      </div>

      <div data-testid="plans-grid" className="mt-16 grid gap-4 md:grid-cols-3">
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
    </div>
  );
}
