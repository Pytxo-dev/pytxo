import Link from "next/link";

import s from "@/components/site/site.module.css";
import { pageMetadata } from "@/lib/page-metadata";
import { DISCORD_URL, GET_STARTED_HREF, PRO_LAUNCH } from "@/lib/site";
import { cn } from "@/lib/utils";

export const metadata = pageMetadata(
  "/plans",
  "Plans · Pytxo",
  `Pytxo is free on your machine and on public repositories. Pro, for the GitHub Action on private repositories, is not on sale yet: ${PRO_LAUNCH}. Team is planned.`,
);

const ROADMAP = [
  {
    name: "Free",
    status: "Free · available now",
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
    <div className={s.page}>
      <div className={cn(s.wrap, s.inner)}>
        <h1>Plans</h1>
        <p className={s.lede}>
          Free on your machine and on public repositories. Pro covers the GitHub Action on private repositories and is
          not on sale yet: {PRO_LAUNCH}. Team is planned.
        </p>
        <div className={s.act}>
          <Link href={GET_STARTED_HREF} className={s.btn}>Get started</Link>
          <a href={DISCORD_URL} className={s.quiet}>Talk to us about Team</a>
        </div>

        <div data-testid="plans-grid" className={s.tiers}>
          {ROADMAP.map((tier) => (
            <section key={tier.name} data-testid="plan-cell" className={s.tier} aria-labelledby={`plan-${tier.name}`}>
              <p className={s.dim}>{tier.status}</p>
              <h2 id={`plan-${tier.name}`}>{tier.name}</h2>
              <p>{tier.detail}</p>
              <ul>
                {tier.highlights.map((highlight) => <li key={highlight}>{highlight}</li>)}
              </ul>
            </section>
          ))}
        </div>
      </div>
    </div>
  );
}
