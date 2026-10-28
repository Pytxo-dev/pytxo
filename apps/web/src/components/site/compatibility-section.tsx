import Link from "next/link";

import { SOURCE_URL } from "@/lib/site";

const FACTS = [
  ["Platforms", "macOS, Linux and Windows. npx pytxo downloads the native binary and checks it against the release's SHA-256 list."],
  ["Agents", "Any that leaves its work on a branch: Claude Code, Codex, Copilot, Cursor, Devin, Jules or your own scripts."],
  ["Project", "A Git repository with tests, and the tools those tests need."],
  ["CI", "GitHub Actions, with one uses: line."],
  ["Account", "None. Pytxo calls no model and sends nothing anywhere."],
  ["Source", "Open source under the MIT license, on GitHub."],
] as const;

export function CompatibilitySection() {
  return <section className="mx-auto grid max-w-[92rem] gap-10 px-4 py-16 sm:px-6 lg:grid-cols-[0.8fr_1.4fr] lg:px-10 lg:py-24" aria-labelledby="compatibility-title" data-testid="compatibility-section">
    <div>
      <h2 id="compatibility-title" className="max-w-[16ch] text-[clamp(2rem,3.4vw,3.25rem)] leading-[1.06] tracking-[-0.04em]">Runs on your machine, with your tests.</h2>
      <p className="mt-6 max-w-[42ch] text-base leading-relaxed text-[#b4b4bd]">Pytxo runs your own test command on a merge it makes in a throwaway worktree. It does not route your code through a Pytxo server.</p>
      <Link className="aperture-link mt-6 inline-block text-sm" href="/docs/getting-started/install">Install options →</Link>
    </div>
    <dl className="divide-y divide-white/15 border-y border-white/15">
      {FACTS.map(([term, value]) => <div key={term} className="grid grid-cols-[6.5rem_1fr] gap-5 py-4 text-sm sm:text-base">
        <dt className="text-[#aaaab3]">{term}</dt><dd className="leading-relaxed">{term === "Source" ? <>Open source under the MIT license, <a className="aperture-link" href={SOURCE_URL}>on GitHub</a>.</> : value}</dd>
      </div>)}
    </dl>
  </section>;
}
