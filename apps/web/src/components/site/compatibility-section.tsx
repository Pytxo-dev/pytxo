import Link from "next/link";

const FACTS = [
  ["Desktop", "Windows x64. macOS and Linux have the Pytxo CLI today; Desktop for them is not built yet."],
  ["Agents", "Codex, Claude Code, Cursor Agent, OpenCode and Antigravity. One is enough to start."],
  ["Accounts", "Each agent signs in with its own account. Local work needs no Pytxo account."],
  ["Keys", "Agents run with a clean environment: your shell's API keys are not passed to them."],
  ["Project", "A Git repository with the tools its build and tests need."],
  ["Cost", "No Pytxo charge for local work. Your agent subscriptions bill as usual."],
] as const;

export function CompatibilitySection() {
  return <section className="mx-auto grid max-w-[92rem] gap-10 px-4 py-16 sm:px-6 lg:grid-cols-[0.8fr_1.4fr] lg:px-10 lg:py-24" aria-labelledby="compatibility-title" data-testid="compatibility-section">
    <div>
      <h2 id="compatibility-title" className="max-w-[16ch] text-[clamp(2rem,3.4vw,3.25rem)] leading-[1.06] tracking-[-0.04em]">Runs on your machine, with your accounts.</h2>
      <p className="mt-6 max-w-[42ch] text-base leading-relaxed text-[#b4b4bd]">Pytxo coordinates the agents you already use. It does not route your code through a Pytxo server.</p>
      <Link className="aperture-link mt-6 inline-block text-sm" href="/docs/getting-started/desktop-setup">Check setup requirements →</Link>
    </div>
    <dl className="divide-y divide-white/15 border-y border-white/15">
      {FACTS.map(([term, value]) => <div key={term} className="grid grid-cols-[6.5rem_1fr] gap-5 py-4 text-sm sm:text-base">
        <dt className="text-[#aaaab3]">{term}</dt><dd className="leading-relaxed">{value}</dd>
      </div>)}
    </dl>
  </section>;
}
