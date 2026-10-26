import Link from "next/link";

const POINTS = [
  ["Any agent tool", "Branches from Orca, Conductor, Codex, Claude Code, Copilot or your own scripts. Label each with the agent that wrote it."],
  ["The combined change", "Pytxo merges the branches in an isolated worktree and runs your checks on the result, and on each branch alone with --per-input."],
  ["Same function, two agents", "Shared files are mapped to the functions and types each branch changed, so two agents editing one function are flagged even when Git merges cleanly."],
  ["Shadow first", "By default nothing in your repository changes. --apply lands a ready result by fast-forward or compare-and-swap, and refuses if the branch moved."],
  ["A receipt every time", "Each decision is written to .git/pytxo/receipts as an in-toto statement. --markdown prints a pull-request comment."],
] as const;

export function LandSection() {
  return <section id="land" className="scroll-mt-20 border-b border-white/10" aria-labelledby="land-title" data-testid="land-section">
    <div className="mx-auto grid max-w-[92rem] gap-12 px-4 py-[clamp(4.5rem,7vw,7.5rem)] sm:px-6 lg:grid-cols-[0.85fr_1.35fr] lg:px-10">
      <div className="min-w-0">
        <span className="flex items-center gap-3 font-mono text-xs uppercase tracking-[0.14em] text-[#8d8d96]">new in 1.3<i aria-hidden className="h-px w-16 bg-[linear-gradient(90deg,#45dccb,transparent)]" /></span>
        <h2 id="land-title" className="mt-3 max-w-[18ch] text-[clamp(2rem,3.4vw,3.25rem)] leading-[1.06] tracking-[-0.04em]">Land branches from any agent tool.</h2>
        <p className="mt-6 max-w-[44ch] text-base leading-relaxed text-[#b4b4bd]">Agent workspaces hand you one branch per agent. <code className="font-mono text-[0.92em] text-[#e4e4ea]">pytxo land</code> checks them together before any of it reaches your branch.</p>
        <div className="mt-6 flex flex-wrap gap-x-6 gap-y-6 text-sm"><Link href="/docs/getting-started/first-landing" className="aperture-link">First landing in 5 minutes →</Link><Link href="/docs/getting-started/land-branches" className="aperture-link">Read the landing guide →</Link></div>
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
