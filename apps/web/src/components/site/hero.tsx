import Image from "next/image";
import Link from "next/link";
import { Button } from "@/components/ui/button";
import { CANDIDATE_VERSION, PUBLISHED_VERSION } from "@/lib/site";
import { ProductWalkthrough } from "./product-walkthrough";

// Marks identify each CLI beside its full name only; Antigravity's mark is not
// cleared for public use, so it is named without one.
const AGENTS = [
  { name: "Codex", mark: "/brands/openai-on-dark.svg" },
  { name: "Claude Code", mask: "/brands/anthropic.svg" },
  { name: "Cursor", mark: "/brands/cursor-on-dark.svg" },
  { name: "OpenCode", mark: "/brands/opencode.svg" },
  { name: "Antigravity" },
] as const;

export function Hero() {
  return <section className="border-b border-white/10" data-testid="marketing-hero">
    <div className="mx-auto max-w-[92rem] px-4 pb-12 pt-16 sm:px-6 lg:px-10 lg:pt-24">
      <div className="mx-auto max-w-[84rem] text-center">
        <h1 className="text-[clamp(2.75rem,7.4vw,6.75rem)] font-semibold leading-[1.02] tracking-[-0.045em]">
          Every coding agent you have.<br /><span className="chroma-text">At once.</span>
        </h1>
        <p className="mx-auto mt-7 max-w-[54ch] text-lg leading-relaxed text-[#c4c4cc] sm:text-xl">Pytxo runs Codex, Claude Code, Cursor, OpenCode and Antigravity on one repository in parallel, keeps them out of each other&apos;s files, and lands only the exact changes you approve.</p>
        <div className="mt-9 flex flex-wrap items-center justify-center gap-4">
          <Button size="lg" className="h-12 rounded-[6px] bg-white px-7 text-base text-black hover:bg-white/85" asChild><Link href="/download">Download for Windows</Link></Button>
          <Link href="/docs/getting-started/first-mission" className="aperture-link text-base text-[#c4c4cc]">Run your first mission →</Link>
        </div>
        <p className="mt-4 text-sm text-[#aaaab3]">Mixed-CLI runs arrive in v{CANDIDATE_VERSION}. The current download, v{PUBLISHED_VERSION}, runs Codex.</p>
      </div>

      <ul className="mx-auto mt-10 flex max-w-5xl flex-wrap items-center justify-center gap-x-8 gap-y-3 text-base text-[#d4d4da]" aria-label="Agent CLIs Pytxo runs">
        <li className="text-[#aaaab3]">Works with the CLIs you already use</li>
        {AGENTS.map((agent) => <li key={agent.name} className="flex items-center gap-2">
          {"mark" in agent && <Image src={agent.mark} alt="" width={20} height={20} className="h-5 w-5" />}
          {"mask" in agent && <span aria-hidden className="h-5 w-5 bg-current" style={{ mask: `url(${agent.mask}) center / contain no-repeat` }} />}
          {agent.name}
        </li>)}
      </ul>

      <figure className="mx-auto mt-12 max-w-[78rem]" data-testid="hero-fleet">
        <div className="overflow-hidden rounded-xl border border-white/10 shadow-[0_40px_120px_-40px_rgba(62,224,208,0.25)]">
          <Image src="/product/fleet-1600x1000.png" alt="Pytxo Work view of a mixed-CLI run: Codex, Claude Code, Cursor Agent and OpenCode working in parallel, a second Codex task waiting on a shared file, and Antigravity queued for the README" width={1600} height={1000} sizes="(min-width: 1280px) 78rem, 100vw" priority className="h-auto w-full" />
        </div>
        <figcaption className="mt-3 text-center text-sm text-[#aaaab3]">Preview capture of the v{CANDIDATE_VERSION} fleet board. Each worker runs in its own isolated copy; a shared file waits for its owner.</figcaption>
      </figure>

      <div id="product" className="scroll-mt-20" data-testid="hero-product"><ProductWalkthrough /></div>
    </div>
  </section>;
}
