import Link from "next/link";
import { Button } from "@/components/ui/button";
import { InstallSnippet } from "@/components/site/install-snippet";
import { NPM_INSTALL, SOURCE_URL } from "@/lib/site";
import capture from "../../../public/evidence/land-blocked-1.3.5-2026-10-25.json";

// Exact output of the published 1.3.5 CLI, recorded on Pybench's
// rename-vs-new-caller repository; the record holds the receipt as well.
const CAPTURE_URL = "/evidence/land-blocked-1.3.5-2026-10-25.json";
const failed = (line: string) => line.startsWith("!") || line.startsWith("decision") || line.includes("FAIL");

export function Hero() {
  return <section className="border-b border-white/10" data-testid="marketing-hero">
    <div className="mx-auto max-w-[92rem] px-4 pb-16 pt-12 sm:px-6 lg:px-10 lg:pb-20 lg:pt-16">
      <p className="text-sm font-medium text-[#aaaab3]">The landing layer for coding agents</p>
      <h1 className="mt-4 max-w-[30ch] text-balance text-[clamp(2.125rem,4.2vw,3.75rem)] font-semibold leading-[1.08] tracking-[-0.035em]">
        <code className="font-mono text-[0.88em] font-medium tracking-[-0.05em] [word-spacing:-0.3em]">pytxo land</code> refuses agent branches that pass alone but fail together.
      </h1>

      <div className="mt-10 grid gap-10 xl:grid-cols-[minmax(0,1fr)_22rem] xl:gap-14">
        <figure className="min-w-0" data-testid="hero-capture">
          <div className="relative rounded-md border border-white/15 p-2 pt-4">
            <span className="absolute -top-2.5 left-4 bg-background px-2 font-mono text-xs text-[#7d7d87]"><b className="font-semibold uppercase tracking-[0.12em] text-[#f5f5f7]">terminal</b> · pytxo {capture.receipt.predicate.pytxo_version}</span>
            <pre className="overflow-x-auto rounded-[4px] bg-[#09090b] p-4 font-mono text-[11px] leading-[1.65] text-[#c7c7ce] sm:p-5 sm:text-[12.5px]" tabIndex={0} aria-label="Recorded output of pytxo land">
              <span className="text-[#8d8d96]">$ </span><span className="text-[#f5f5f7]">{capture.command}</span>{"\n"}{capture.output.trimEnd().split("\n").map((line, i) => <span key={i} className={failed(line) ? "text-[#ff9b8a]" : undefined}>{line}{"\n"}</span>)}
            </pre>
          </div>
          <figcaption className="mt-3 max-w-[80ch] px-1 text-sm leading-relaxed text-[#8d8d96]">
            Recorded on 25 October 2026 with the published 1.3.5 CLI, on a sample repository from Pybench. One branch renames <code className="font-mono text-[0.92em]">price()</code> to <code className="font-mono text-[0.92em]">unitPrice()</code>; the other adds code that calls <code className="font-mono text-[0.92em]">price()</code>. Each passes the test on its own. Together the test fails, and fails again when pytxo reruns it, so nothing lands. <a className="aperture-link text-[#c4c4cc]" href={CAPTURE_URL}>Raw output and receipt</a>
          </figcaption>
        </figure>

        <div className="min-w-0 max-w-[40rem]">
          <p className="text-lg leading-relaxed text-[#c4c4cc]">Give it the branches your agents left, from Claude Code, Codex or any other tool, and the command that tests your project. It merges them in a separate worktree and runs that command on the result. Your branch moves only when the result passes and you add <code className="whitespace-nowrap font-mono text-[0.9em] text-[#e4e4ea]">--apply</code>.</p>
          <InstallSnippet className="mt-8" copyLabel="Copy the npm install command">{NPM_INSTALL}</InstallSnippet>
          <div className="mt-6 flex flex-wrap items-center gap-x-6 gap-y-4">
            <Button size="lg" className="h-11 gap-2 rounded-[6px] bg-white px-5 text-base text-black hover:bg-white/85" asChild>
              <a href={SOURCE_URL}><span aria-hidden className="size-[18px] bg-current [mask:url(/brands/github.svg)_center/contain_no-repeat]" />Source on GitHub</a>
            </Button>
            <Link href="/download" className="aperture-link text-base text-[#c4c4cc]">Download for Windows</Link>
          </div>
          {/* PLACEHOLDER builder line: one short line in Matt's own words on who builds Pytxo and why. Waiting on his wording; do not merge without it. */}
        </div>
      </div>
    </div>
  </section>;
}
