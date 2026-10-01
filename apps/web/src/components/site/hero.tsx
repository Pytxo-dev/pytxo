import Link from "next/link";
import { Button } from "@/components/ui/button";
import { CANDIDATE_VERSION, PUBLISHED_VERSION } from "@/lib/site";
import { ProductWalkthrough } from "./product-walkthrough";
import { AsciiAperture } from "./ascii-aperture";

export function Hero() {
  return <section className="border-b border-white/10" data-testid="marketing-hero">
    <div className="mx-auto max-w-[92rem] px-4 pb-12 pt-14 sm:px-6 lg:px-10 lg:pt-20">
      <div className="relative grid items-center gap-8 lg:grid-cols-[1.7fr_0.8fr] lg:gap-16">
        <div>
          <h1 className="text-[clamp(2.5rem,4.6vw,4.6rem)] font-medium leading-[1.09] tracking-[-0.03em]">
            Agents do the work.<br /><span className="text-[#aaaab3]">You decide what lands.</span>
          </h1>
          <p className="mt-6 max-w-[60ch] text-base leading-relaxed text-[#c4c4cc]">Give your coding agent a task. Pytxo runs it in an isolated copy, collects the changes, and shows the checks. Review the result before it reaches your repository.</p>
          <p className="mt-4 max-w-[60ch] text-sm leading-relaxed text-[#aaaab3]">Keep your editor and agent account. Local Core needs no Pytxo account.</p>
          <div className="mt-7 flex flex-wrap items-center gap-5">
            <Button size="lg" className="h-11 rounded-[4px] bg-white px-6 text-black hover:bg-white/85" asChild><Link href="/download">Download current v{PUBLISHED_VERSION}</Link></Button>
            <Link href="/docs/getting-started/first-mission" className="aperture-link text-sm text-[#c4c4cc]">First mission →</Link>
          </div>
          <p className="mt-4 text-xs leading-relaxed text-[#aaaab3]">The walkthrough below previews the unpublished v{CANDIDATE_VERSION} Desktop interface.</p>
          <p className="mt-5 font-mono text-xs text-[#aaaab3]">Windows x64 · Codex CLI · Git · Local Core</p>
        </div>
        <AsciiAperture />
      </div>
      <div id="product" className="scroll-mt-20" data-testid="hero-product"><ProductWalkthrough /></div>
    </div>
  </section>;
}
