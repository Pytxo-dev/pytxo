import Image from "next/image";
import Link from "next/link";
import { Button } from "@/components/ui/button";
import { CANDIDATE_PUBLISHED, CANDIDATE_VERSION, PUBLISHED_VERSION } from "@/lib/site";

// Each mark sits beside its full product name and implies no partnership.
const AGENTS = [
  { name: "Codex", mark: "/brands/openai-on-dark.svg" },
  { name: "Claude Code", mask: "/brands/anthropic.svg" },
  { name: "Cursor", mark: "/brands/cursor-on-dark.svg" },
  { name: "OpenCode", mark: "/brands/opencode.svg" },
  { name: "Antigravity", mark: "/brands/antigravity.png" },
] as const;

export function Hero() {
  return <section className="border-b border-white/10" data-testid="marketing-hero">
    <div className="mx-auto max-w-[92rem] px-4 pb-14 pt-16 sm:px-6 lg:px-10 lg:pt-24">
      <div className="mx-auto max-w-[84rem] text-center">
        <p className="text-sm font-medium tracking-[0.01em] text-[#aaaab3]">The agent hypervisor for your repository</p>
        <h1 className="mt-5 text-[clamp(2.75rem,7.4vw,6.75rem)] font-semibold leading-[1.02] tracking-[-0.045em]">
          Many agents.<br /><span className="chroma-text">One verified change.</span>
        </h1>
        <p className="mx-auto mt-7 max-w-[58ch] text-lg leading-relaxed text-[#c4c4cc] sm:text-xl">Describe a change once. Pytxo splits it across the coding agents you choose, keeps them off each other&apos;s files, runs your checks on the combined result, and applies exactly what you reviewed.</p>
        <div className="mt-9 flex flex-wrap items-center justify-center gap-4">
          <Button size="lg" className="h-12 rounded-[6px] bg-white px-7 text-base text-black hover:bg-white/85" asChild><Link href="/download">Download for Windows</Link></Button>
          <Link href="#how" className="aperture-link text-base text-[#c4c4cc]">See how it works ↓</Link>
        </div>
        {!CANDIDATE_PUBLISHED && <p className="mt-4 text-sm text-[#aaaab3]">Mixed-agent runs arrive in v{CANDIDATE_VERSION}. Today&apos;s download, v{PUBLISHED_VERSION}, runs Codex.</p>}
      </div>

      <ul className="mx-auto mt-10 flex max-w-5xl flex-wrap items-center justify-center gap-x-8 gap-y-3 text-base text-[#d4d4da]" aria-label="Agent CLIs Pytxo runs">
        <li className="text-[#aaaab3]">With the CLIs you already pay for</li>
        {AGENTS.map((agent) => <li key={agent.name} className="flex items-center gap-2">
          {"mark" in agent && <Image src={agent.mark} alt="" width={20} height={20} className="h-5 w-5 object-contain" />}
          {"mask" in agent && <span aria-hidden className="h-5 w-5 bg-current" style={{ mask: `url(${agent.mask}) center / contain no-repeat` }} />}
          {agent.name}
        </li>)}
      </ul>

      <figure className="mx-auto mt-12 max-w-[78rem]" data-testid="hero-fleet">
        <div className="overflow-hidden rounded-xl border border-white/10 shadow-[0_40px_120px_-40px_rgba(62,224,208,0.25)]">
          <Image src="/product/fleet-1600x1000.png" alt="Pytxo Work view of one request split across agents: Codex, Claude Code, Cursor Agent and OpenCode working in parallel, a second Codex task waiting on a file Claude Code owns, and Antigravity queued for the README" width={1600} height={1000} sizes="(min-width: 1280px) 78rem, 100vw" priority className="h-auto w-full" />
        </div>
        <figcaption className="mt-3 text-center text-sm text-[#aaaab3]">One request, five agents, each in its own copy of the project. A task that shares a file waits for the agent that owns it. Preview capture.</figcaption>
      </figure>
    </div>
  </section>;
}
