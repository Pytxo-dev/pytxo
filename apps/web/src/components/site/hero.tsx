import Image from "next/image";
import Link from "next/link";

import { Button } from "@/components/ui/button";
import { InstallSnippet } from "@/components/site/install-snippet";
import { NPM_INSTALL } from "@/lib/site";

export function Hero() {
  return (
    <section
      className="relative overflow-hidden border-b border-border"
      data-testid="marketing-hero"
    >
      <div
        className="pointer-events-none absolute inset-x-0 top-0 h-[34rem] opacity-60"
        style={{
          background:
            "radial-gradient(circle at 72% 16%, color-mix(in oklab, var(--brand-teal) 16%, transparent), transparent 50%)",
        }}
        aria-hidden
      />
      <div className="relative mx-auto grid max-w-7xl gap-9 px-4 pb-14 pt-10 sm:px-6 sm:pb-16 sm:pt-14 lg:grid-cols-[minmax(0,0.86fr)_minmax(0,1.14fr)] lg:items-center lg:gap-14 lg:pb-20 lg:pt-16">
        <div className="flex max-w-2xl flex-col gap-5 text-left">
          <h1 className="max-w-2xl text-[2.5rem] leading-[0.98] tracking-[-0.055em] sm:text-[3.4rem] lg:text-[2.8rem] 2xl:text-[3.2rem]">
            Coordinate coding agents. <span className="chroma-text">Review one result.</span>
          </h1>
          <p className="max-w-[38rem] text-base leading-relaxed text-muted-foreground sm:text-lg">
            Assign paths and dependencies, run agents in isolated workspaces, then
            approve the exact package prepared for one repository root.
          </p>
          <div className="grid grid-cols-2 gap-3 sm:flex sm:items-center">
            <Button size="lg" className="min-w-0 sm:min-w-[10rem]" asChild>
              <Link href="/download">Download Desktop</Link>
            </Button>
            <Button
              size="lg"
              variant="outline"
              className="min-w-0 border-border bg-transparent sm:min-w-[10rem]"
              asChild
            >
              <Link href="/docs/getting-started/first-mission">Run a mission</Link>
            </Button>
          </div>
          <InstallSnippet className="mt-1 w-full max-w-md">{NPM_INSTALL}</InstallSnippet>
        </div>

        <div
          className="relative overflow-hidden rounded-[var(--radius-lg)] border border-border bg-card/30 shadow-[0_34px_110px_-54px_rgba(45,212,191,0.5)]"
          data-testid="hero-product"
        >
          <Image
            src="/product/run-review-1600x1000.png"
            alt="Pytxo Desktop Run Review showing the base revision, package digest, ownership plan, enforcement receipt, and exact prepared changes"
            width={1600}
            height={1000}
            className="h-auto w-full"
            sizes="(max-width: 1023px) 100vw, 58vw"
            priority
          />
        </div>
      </div>
    </section>
  );
}
