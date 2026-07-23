import Image from "next/image";
import Link from "next/link";

import { Button } from "@/components/ui/button";
import { InstallSnippet } from "@/components/site/install-snippet";
import { NPM_INSTALL } from "@/lib/site";

export function Hero() {
  return (
    <section className="relative overflow-hidden">
      <div className="relative mx-auto grid max-w-6xl gap-10 px-4 pt-14 pb-12 sm:px-6 sm:pt-16 lg:grid-cols-[minmax(0,1.05fr)_minmax(0,0.95fr)] lg:items-center lg:gap-12 lg:pb-16">
        <div className="flex flex-col gap-5 text-left">
          <div className="flex items-center gap-3">
            <Image
              src="/logo-mark.png"
              alt=""
              width={40}
              height={40}
              className="size-10"
              priority
            />
            <div className="flex flex-col gap-0.5">
              <span className="text-lg font-semibold tracking-tight text-foreground">Pytxo</span>
              <span className="text-xs font-medium text-muted-foreground">
                Local agent hypervisor
              </span>
            </div>
          </div>
          <h1 className="text-4xl leading-[1.1] tracking-tight sm:text-5xl lg:text-[3.1rem]">
            Run coding agents in parallel.{" "}
            <span style={{ color: "var(--brand-teal)" }}>Stay in control of every change.</span>
          </h1>
          <p className="max-w-[34rem] text-base leading-relaxed text-muted-foreground sm:text-lg">
            Coordinate Claude Code, Codex, and other terminal agents on your machine while you keep
            your IDE.
          </p>
          <InstallSnippet className="max-w-md">{NPM_INSTALL}</InstallSnippet>
          <div className="flex flex-col gap-3 sm:flex-row sm:items-center">
            <Button size="lg" className="min-w-[10rem]" asChild>
              <Link href="/download">Download</Link>
            </Button>
            <Button
              size="lg"
              variant="outline"
              className="min-w-[10rem] border-border bg-transparent"
              asChild
            >
              <Link href="/docs">Docs</Link>
            </Button>
          </div>
        </div>

        <figure className="w-full">
          <div className="hero-product-frame w-full">
            <Image
              src="/desktop-topology-hero.png"
              alt="Pytxo Desktop supervising parallel agent runs with structural topology and approvals"
              width={1536}
              height={1024}
              className="h-auto w-full"
              priority
              sizes="(max-width: 1024px) 100vw, 560px"
            />
          </div>
          <figcaption className="mt-3 text-center text-xs text-muted-foreground lg:text-left">
            Pytxo Desktop - structural topology, not a wall of terminals.
          </figcaption>
        </figure>
      </div>
    </section>
  );
}
