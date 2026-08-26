import Image from "next/image";
import Link from "next/link";

import { Button } from "@/components/ui/button";
import { InstallSnippet } from "@/components/site/install-snippet";
import { NPM_INSTALL } from "@/lib/site";

export function Hero() {
  return (
    <section className="relative border-b border-border" data-testid="marketing-hero">
      <div className="relative mx-auto grid max-w-7xl items-center gap-10 px-4 pb-20 pt-14 sm:px-6 sm:pb-24 sm:pt-16 lg:grid-cols-[minmax(0,1fr)_minmax(0,1.15fr)] lg:gap-14 lg:pb-28 lg:pt-20">
        <div className="flex max-w-2xl flex-col gap-5 text-left">
          <h1 className="max-w-2xl text-pretty text-[2.15rem] leading-[1.05] tracking-[-0.04em] sm:text-[2.85rem] lg:text-[2.55rem] 2xl:text-[2.9rem]">
            Coordinate coding agents. Review one{" "}
            <span className="chroma-text">result</span>.
          </h1>
          <p className="max-w-[38rem] text-[0.95rem] leading-relaxed text-muted-foreground sm:text-lg">
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

        <div className="bezel-frame bezel-frame--ribbon">
          <div className="bezel-frame__bar">
            <span className="status-lamp" data-state="live" aria-hidden />
            <span>Ops</span>
          </div>
          <div data-testid="hero-product" className="aspect-[16/10] bg-card">
            <Image
              src="/product/operations-1600x1000.png"
              alt="Pytxo Desktop Ops showing Needs you approvals and Running mission rows"
              width={1600}
              height={1000}
              className="h-full w-full object-cover object-top"
              sizes="(max-width: 1023px) 100vw, 58vw"
              priority
            />
          </div>
        </div>
      </div>
    </section>
  );
}
