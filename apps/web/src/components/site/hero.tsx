import Image from "next/image";
import Link from "next/link";

import { Button } from "@/components/ui/button";
import { InstallSnippet } from "@/components/site/install-snippet";
import { NPM_INSTALL } from "@/lib/site";

export function Hero() {
  return (
    <section className="relative overflow-hidden">
      <div className="relative mx-auto grid max-w-6xl gap-8 px-4 pt-10 pb-12 sm:px-6 sm:pt-12 lg:grid-cols-[minmax(0,1.1fr)_minmax(0,0.9fr)] lg:items-center lg:gap-14 lg:pb-16">
        <div className="flex flex-col gap-5 text-left">
          <p className="text-sm font-medium tracking-wide text-muted-foreground">
            Local agent hypervisor
          </p>
          <h1 className="text-4xl leading-[1.08] tracking-tight sm:text-5xl lg:text-[3.15rem]">
            Run coding agents in parallel.{" "}
            <span className="chroma-text">Stay in control of every change.</span>
          </h1>
          <p className="max-w-[34rem] text-base leading-relaxed text-muted-foreground sm:text-lg">
            Coordinate Claude Code, Codex, and other terminal agents on your machine while you keep
            your IDE.
          </p>
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
          <InstallSnippet className="max-w-md">{NPM_INSTALL}</InstallSnippet>
        </div>

        <div className="hero-chroma-mark flex w-full items-center justify-center py-10 sm:py-14">
          <Image
            src="/logo-mark.png"
            alt="Pytxo"
            width={320}
            height={320}
            className="hero-logo-mark size-44 sm:size-56 lg:size-64"
            priority
          />
        </div>
      </div>
    </section>
  );
}
