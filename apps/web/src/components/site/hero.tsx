import Image from "next/image";
import Link from "next/link";

import { Button } from "@/components/ui/button";
import { InstallSnippet } from "@/components/site/install-snippet";
import { NPM_INSTALL } from "@/lib/site";

export function Hero() {
  return (
    <section className="relative overflow-hidden section-reveal">
      <div className="relative mx-auto flex max-w-6xl flex-col gap-10 px-4 pt-14 pb-16 sm:px-6 sm:pt-16 sm:pb-20 lg:flex-row lg:items-center lg:gap-14">
        <div className="flex flex-1 flex-col gap-5 text-left">
          <p className="chroma-text chroma-shift text-sm font-medium">Local agent hypervisor</p>
          <h1 className="text-4xl leading-[1.08] sm:text-5xl lg:text-[3.25rem]">
            Run coding agents in parallel.{" "}
            <span className="chroma-text chroma-shift">Stay in control of every change.</span>
          </h1>
          <p className="max-w-[36rem] text-base leading-relaxed text-muted-foreground sm:text-lg">
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

        <div className="hero-logo-panel flex w-full shrink-0 items-center justify-center lg:max-w-[48%] lg:min-h-[22rem]">
          <div className="hero-logo-mark flex flex-col items-center gap-4 px-8 py-12">
            <Image
              src="/logo.png"
              alt="Pytxo"
              width={200}
              height={200}
              className="size-36 sm:size-44 lg:size-52"
              priority
            />
            <p className="text-center text-sm font-medium tracking-tight text-foreground/90">
              Pytxo
            </p>
            <p className="max-w-[14rem] text-center text-xs text-muted-foreground">
              Local agent hypervisor
            </p>
          </div>
        </div>
      </div>
    </section>
  );
}
