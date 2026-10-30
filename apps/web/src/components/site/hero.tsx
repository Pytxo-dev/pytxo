import Link from "next/link";
import { Button } from "@/components/ui/button";
import { InstallSnippet } from "@/components/site/install-snippet";
import { DEMO_COMMAND, SOURCE_URL } from "@/lib/site";
import capture from "../../../public/evidence/demo-blocked-2.0.0-2026-10-30.json";

// Exact output of `npx pytxo demo` from a local build of the 2.0.0 candidate,
// on Pybench's rename-vs-new-caller repository; the record says how it ran.
const CAPTURE_URL = "/evidence/demo-blocked-2.0.0-2026-10-30.json";
const failed = (line: string) => line.startsWith("BLOCKED") || line.includes("FAIL");

export function Hero() {
  return <section className="border-b border-white/10" data-testid="marketing-hero">
    <div className="mx-auto max-w-[92rem] px-4 pb-16 pt-12 sm:px-6 lg:px-10 lg:pb-20 lg:pt-16">
      <p className="text-sm font-medium text-[#aaaab3]">For developers who run more than one coding agent</p>
      <h1 className="mt-4 max-w-[30ch] text-balance text-[clamp(2.125rem,4.2vw,3.75rem)] font-semibold leading-[1.08] tracking-[-0.035em]">
        Each agent&apos;s branch passes. Do they pass together?
      </h1>

      <div className="mt-10 grid gap-10 xl:grid-cols-[minmax(0,1fr)_22rem] xl:gap-14">
        <figure className="min-w-0" data-testid="hero-capture">
          <div className="relative rounded-md border border-white/15 p-2 pt-4">
            <span className="absolute -top-2.5 left-4 bg-background px-2 font-mono text-xs text-[#7d7d87]"><b className="font-semibold uppercase tracking-[0.12em] text-[#f5f5f7]">terminal</b> · pytxo {capture.version}</span>
            <pre className="overflow-x-auto rounded-[4px] bg-[#09090b] p-4 font-mono text-[11px] leading-[1.65] text-[#c7c7ce] sm:p-5 sm:text-[12.5px]" tabIndex={0} aria-label="Recorded output of npx pytxo demo">
              <span className="text-[#8d8d96]">$ </span><span className="text-[#f5f5f7]">{capture.command}</span>{"\n"}{capture.output.trimEnd().split("\n").map((line, i) => <span key={i} className={failed(line) ? "text-[#ff9b8a]" : undefined}>{line}{"\n"}</span>)}
            </pre>
          </div>
          <figcaption className="mt-3 max-w-[80ch] px-1 text-sm leading-relaxed text-[#8d8d96]">
            Recorded on 30 October 2026 with a local build of the 2.0.0 candidate. The demo builds a sample repository from Pybench: one branch renames <code className="font-mono text-[0.92em]">price()</code> to <code className="font-mono text-[0.92em]">unitPrice()</code>, the other adds code that calls <code className="font-mono text-[0.92em]">price()</code>. Each passes <code className="font-mono text-[0.92em]">npm test</code> on its own. Together the test fails, so the check blocks and says which branch has to change. <a className="aperture-link text-[#c4c4cc]" href={CAPTURE_URL}>Raw output</a>
          </figcaption>
        </figure>

        <div className="min-w-0 max-w-[40rem]">
          <p className="text-lg leading-relaxed text-[#c4c4cc]">Pytxo merges your Claude Code, Codex and Copilot branches in a throwaway worktree, runs your tests on the combined result, and blocks the merge if it breaks. Free, open source, runs on your machine or in CI.</p>
          <InstallSnippet className="mt-8" copyLabel="Copy the demo command">{DEMO_COMMAND}</InstallSnippet>
          <div className="mt-6 flex flex-wrap items-center gap-x-6 gap-y-4">
            <Button size="lg" className="h-11 gap-2 rounded-[6px] bg-white px-5 text-base text-black hover:bg-white/85" asChild>
              <a href={SOURCE_URL}><span aria-hidden className="size-[18px] bg-current [mask:url(/brands/github.svg)_center/contain_no-repeat]" />Source on GitHub</a>
            </Button>
            <Link href="/#ci" className="aperture-link text-base text-[#c4c4cc]">Add it to CI</Link>
          </div>
          <p className="mt-8 max-w-[46ch] text-sm leading-relaxed text-[#8d8d96]">
            I&apos;m Matt (<a className="aperture-link text-[#c4c4cc]" href="https://github.com/mattbaconz">mattbaconz</a> on GitHub), and I build Pytxo on my own. I gave Claude Code and Codex two tasks on <Link className="aperture-link text-[#c4c4cc]" href="/docs/getting-started/first-landing#3-if-the-decision-is-blocked">lukeed/mri</Link> that collide on one helper; both branches passed alone and failed together.
          </p>
        </div>
      </div>
    </div>
  </section>;
}
