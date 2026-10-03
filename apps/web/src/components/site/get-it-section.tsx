import Link from "next/link";
import { Button } from "@/components/ui/button";
import { CANDIDATE_PUBLISHED, DISCORD_URL, PUBLISHED_VERSION } from "@/lib/site";

const LINKS = [
  ["First mission", "/docs/getting-started/first-mission", "Setup, plan, run, Review and Apply, step by step."],
  ["Troubleshooting", "/docs/troubleshooting", "Agent sign-in, failed checks and recovery."],
  ["Ask for help", DISCORD_URL, "Share the problem and a redacted run report."],
] as const;

export function GetItSection() {
  return <section className="border-t border-white/15" aria-labelledby="get-it-title" data-testid="get-it-section">
    <div className="mx-auto grid max-w-[92rem] gap-10 px-4 py-16 sm:px-6 lg:grid-cols-2 lg:px-10 lg:py-24">
      <div>
        <h2 id="get-it-title" className="text-[clamp(2rem,3.4vw,3.25rem)] leading-[1.06] tracking-[-0.04em]">Start with one agent<br />and a real fix.</h2>
        <p className="mt-6 max-w-[42ch] text-base leading-relaxed text-[#b4b4bd]">Pick a small bug with a test you can run. Once you trust the loop, add agents and give them a bigger job.</p>
        <Button asChild className="mt-8 h-11 rounded-[4px] bg-white px-6 text-black hover:bg-white/85"><Link href="/download">{CANDIDATE_PUBLISHED ? "Download for Windows" : `Download current v${PUBLISHED_VERSION}`}</Link></Button>
      </div>
      <nav aria-label="Start and support" className="divide-y divide-white/15 border-y border-white/15">
        {LINKS.map(([title, href, detail]) => <Link key={title} href={href} className="block py-5 focus-visible:outline-2 focus-visible:outline-[#a59bff]"><span className="text-base">{title} →</span><span className="mt-2 block text-sm text-[#aaaab3]">{detail}</span></Link>)}
      </nav>
    </div>
  </section>;
}
