import Link from "next/link";

import { InstallSnippet } from "@/components/site/install-snippet";
import { NPM_INSTALL } from "@/lib/site";

// Real terminal output of `pytxo land` (1.3.0 candidate) on the Pybench
// "rename vs new caller" sample repository, recorded 10 October 2026. Only the
// absolute receipt path was shortened.
const COMMAND = 'pytxo land codex:agent/rename claude:agent/caller -c "node test.js" --per-input';
const OUTPUT = `pytxo land  rcpt_20261010T150149Z_1d7ca1a3  shadow
target main @ a6afbff09 · 2 inputs · 2 files changed

inputs
  codex:agent/rename                 1 file   merged
  claude:agent/caller                1 file   merged

checks
  input:agent/rename           pass           0.7s  node test.js
  input:agent/caller           pass           0.4s  node test.js
  combined                     FAIL exit 1    0.7s  node test.js

! Every input passed its checks alone. The combined change failed.

decision  BLOCKED
  - combined check failed: node test.js (exit 1)
receipt   .git/pytxo/receipts/rcpt_20261010T150149Z_1d7ca1a3.json`;

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
        <dl className="mt-8 divide-y divide-white/10 border-y border-white/10">
          {POINTS.map(([term, detail]) => <div key={term} className="py-4">
            <dt className="text-sm text-[#f5f5f7]">{term}</dt>
            <dd className="mt-1.5 text-sm leading-relaxed text-[#9d9da6]">{detail}</dd>
          </div>)}
        </dl>
        <Link href="/docs/getting-started/land-branches" className="aperture-link mt-6 inline-block text-sm">Read the landing guide →</Link>
      </div>
      <div className="min-w-0 self-start">
        <figure className="relative rounded-md border border-white/15 p-2 pt-4" data-testid="land-output">
          <span className="absolute -top-2.5 left-4 bg-background px-2 font-mono text-xs text-[#7d7d87]"><b className="font-semibold uppercase tracking-[0.12em] text-[#f5f5f7]">terminal</b> · pytxo 1.3.0</span>
          <pre className="overflow-x-auto rounded-[4px] bg-[#09090b] p-5 font-mono text-[12.5px] leading-[1.65] text-[#c7c7ce]" tabIndex={0} aria-label="Output of pytxo land">
            <span className="text-[#8d8d96]">$ </span><span className="text-[#f5f5f7]">{COMMAND}</span>{"\n"}{OUTPUT.split("\n").map((line, i) => <span key={i} className={line.startsWith("!") || line.includes("FAIL") || line.startsWith("decision") ? "text-[#ff9b8a]" : undefined}>{line}{"\n"}</span>)}
          </pre>
          <figcaption className="mt-3 px-1 text-sm text-[#8d8d96]">Recorded output on a sample repository from Pybench: one agent renamed a function, another added a caller of the old name. Each branch passed alone; together they fail, so nothing lands.</figcaption>
        </figure>
        <div className="mt-6">
          <InstallSnippet copyLabel="Copy the npm install command">{NPM_INSTALL}</InstallSnippet>
        </div>
      </div>
    </div>
  </section>;
}
