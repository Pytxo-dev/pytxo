import Link from "next/link";
import { ACTION_USES } from "@/lib/site";

const WORKFLOW = `name: Pytxo
on: pull_request
permissions:
  contents: read
  pull-requests: write
jobs:
  check:
    runs-on: ubuntu-latest
    steps:
      - uses: ${ACTION_USES}`;

const POINTS = [
  ["Agent pull requests only", "Found by branch prefix or by bot author, such as Copilot, Claude, Cursor, Devin or Jules. No labels to add."],
  ["One comment", "Posted on the pull request and updated in place. A blocked combination fails the job. Without comment permission the verdict goes to the job summary."],
  ["Safe by default", "pull_request only, never pull_request_target. A fork's pull request never joins another pull request's run: it is checked in its own run, with GitHub's read-only token and no secrets."],
  ["When your agents finish", "The Claude Code plugin and the Codex hook run the same check on your machine when an agent stops."],
] as const;

export function CiSection() {
  return <section id="ci" className="scroll-mt-20 border-b border-white/10" aria-labelledby="ci-title" data-testid="ci-section">
    <div className="mx-auto grid max-w-[92rem] gap-12 px-4 py-[clamp(4.5rem,7vw,7.5rem)] sm:px-6 lg:grid-cols-[0.85fr_1.35fr] lg:px-10">
      <div className="min-w-0">
        <span className="flex items-center gap-3 font-mono text-xs uppercase tracking-[0.14em] text-[#8d8d96]">github actions<i aria-hidden className="h-px w-16 bg-[linear-gradient(90deg,#45dccb,transparent)]" /></span>
        <h2 id="ci-title" className="mt-3 max-w-[18ch] text-[clamp(2rem,3.4vw,3.25rem)] leading-[1.06] tracking-[-0.04em]">One line in CI.</h2>
        <p className="mt-6 max-w-[44ch] text-base leading-relaxed text-[#b4b4bd]">On every agent pull request, the Action checks it together with the other open agent pull requests into the same base branch, and tells you on the pull request.</p>
        <pre className="mt-8 overflow-x-auto rounded-[4px] border border-white/10 bg-[#09090b] p-4 font-mono text-[12px] leading-[1.65] text-[#c7c7ce]" tabIndex={0} aria-label="GitHub Actions workflow"><span className="text-[#8d8d96]"># .github/workflows/pytxo.yml</span>{"\n"}{WORKFLOW}</pre>
        <div className="mt-6 flex flex-wrap gap-x-6 gap-y-6 text-sm"><Link href="/docs/getting-started/land-in-ci" className="aperture-link">Set it up →</Link><Link href="/docs/getting-started/agent-hooks" className="aperture-link">Claude Code and Codex →</Link></div>
      </div>
      <dl className="min-w-0 self-start divide-y divide-white/10 border-y border-white/10">
        {POINTS.map(([term, detail]) => <div key={term} className="py-4">
          <dt className="text-sm text-[#f5f5f7]">{term}</dt>
          <dd className="mt-1.5 text-sm leading-relaxed text-[#9d9da6]">{detail}</dd>
        </div>)}
      </dl>
    </div>
  </section>;
}
