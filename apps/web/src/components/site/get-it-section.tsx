import Link from "next/link";
import { InstallSnippet } from "@/components/site/install-snippet";
import { DEMO_COMMAND, DISCORD_URL } from "@/lib/site";

const LINKS = [
  ["Quickstart", "/docs/getting-started/quickstart", "Run it on your repository, read the verdict, change what it checks."],
  ["Add it to CI", "/docs/getting-started/land-in-ci", "One uses: line on every agent pull request."],
  ["Ask for help", DISCORD_URL, "Share the verdict and what you expected."],
] as const;

export function GetItSection() {
  return <section className="border-t border-white/15" aria-labelledby="get-it-title" data-testid="get-it-section">
    <div className="mx-auto grid max-w-[92rem] gap-10 px-4 py-16 sm:px-6 lg:grid-cols-2 lg:px-10 lg:py-24">
      <div>
        <h2 id="get-it-title" className="text-[clamp(2rem,3.4vw,3.25rem)] leading-[1.06] tracking-[-0.04em]">See it catch a break<br />in under 30 seconds.</h2>
        <p className="mt-6 max-w-[42ch] text-base leading-relaxed text-[#b4b4bd]">Run the demo. Then run <code className="font-mono text-[0.92em]">npx pytxo</code> in a repository where your agents work. No flags, no config, no account.</p>
        <InstallSnippet className="mt-8 max-w-[28rem]" copyLabel="Copy the demo command">{DEMO_COMMAND}</InstallSnippet>
      </div>
      <nav aria-label="Start and support" className="divide-y divide-white/15 border-y border-white/15">
        {LINKS.map(([title, href, detail]) => <Link key={title} href={href} className="block py-5 focus-visible:outline-2 focus-visible:outline-[#a59bff]"><span className="text-base">{title} →</span><span className="mt-2 block text-sm text-[#aaaab3]">{detail}</span></Link>)}
      </nav>
    </div>
  </section>;
}
