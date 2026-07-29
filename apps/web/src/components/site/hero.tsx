import Image from "next/image";
import Link from "next/link";

import { Button } from "@/components/ui/button";
import { InstallSnippet } from "@/components/site/install-snippet";
import { NPM_INSTALL } from "@/lib/site";

export function Hero() {
  return (
    <>
      <section className="relative overflow-hidden" data-testid="marketing-hero">
        <div className="relative mx-auto grid max-w-7xl gap-8 px-4 pb-10 pt-8 sm:px-6 sm:pb-14 sm:pt-12 lg:grid-cols-[minmax(0,1.1fr)_minmax(0,0.9fr)] lg:items-center lg:gap-12 lg:pb-16">
          <div className="flex max-w-3xl flex-col gap-5 text-left">
            <p className="text-sm font-medium tracking-wide text-muted-foreground">
              Local agent hypervisor
            </p>
            <h1 className="max-w-3xl text-[2.25rem] leading-[1.03] tracking-tight sm:text-5xl lg:text-[2.85rem]">
              One mission. Parallel work.{" "}
              <span className="chroma-text">One verified result.</span>
            </h1>
            <p className="max-w-[36rem] text-base leading-relaxed text-muted-foreground sm:text-lg">
              Plan safely, run agents in isolation, and approve one reviewable result on your
              machine.
            </p>
            <div className="grid grid-cols-2 gap-3 sm:flex sm:items-center">
              <Button size="lg" className="min-w-0 sm:min-w-[10rem]" asChild>
                <Link href="/download">Download</Link>
              </Button>
              <Button
                size="lg"
                variant="outline"
                className="min-w-0 border-border bg-transparent sm:min-w-[10rem]"
                asChild
              >
                <Link href="/docs">Docs</Link>
              </Button>
            </div>
          </div>

          <div
            className="relative overflow-hidden rounded-[var(--radius-lg)] border border-border bg-card/30 shadow-[0_28px_90px_-48px_rgba(45,212,191,0.42)]"
            data-testid="hero-product"
          >
            <Image
              src="/product/operations-1600x1000.png"
              alt="Pytxo Desktop Operations showing active runs, approvals, isolated work, and estimated cost"
              width={1600}
              height={1000}
              className="h-auto w-full"
              sizes="(max-width: 1023px) 100vw, 52vw"
              priority
            />
          </div>
        </div>
      </section>

      <section className="border-y border-border bg-card/10" aria-label="Install Pytxo CLI">
        <div className="mx-auto flex max-w-7xl flex-col gap-3 px-4 py-5 sm:flex-row sm:items-center sm:justify-between sm:px-6">
          <p className="text-sm text-muted-foreground">
            Prefer the CLI? Start locally with one command.
          </p>
          <InstallSnippet className="w-full max-w-md sm:w-auto sm:min-w-[25rem]">
            {NPM_INSTALL}
          </InstallSnippet>
        </div>
      </section>
    </>
  );
}
