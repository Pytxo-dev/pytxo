import Image from "next/image";
import Link from "next/link";

import { Button } from "@/components/ui/button";
import { InstallSnippet } from "@/components/site/install-snippet";
import { DESKTOP_PRODUCT_NAME, NPM_INSTALL } from "@/lib/site";

export function Hero() {
  return (
    <section className="relative overflow-hidden section-reveal">
      <div className="relative mx-auto flex max-w-6xl flex-col gap-10 px-4 pt-14 pb-16 sm:px-6 sm:pt-16 sm:pb-20 lg:flex-row lg:items-center lg:gap-14">
        <div className="flex flex-1 flex-col gap-5 text-left">
          <p className="text-sm font-medium text-primary">Local agent hypervisor</p>
          <h1 className="text-4xl leading-[1.08] sm:text-5xl lg:text-[3.25rem]">
            Run coding agents in parallel.{" "}
            <span className="text-primary">Stay in control of every change.</span>
          </h1>
          <p className="max-w-[36rem] text-base leading-relaxed text-muted-foreground sm:text-lg">
            Coordinate Claude Code, Codex, and other terminal agents on your machine while you keep
            your IDE.
          </p>
          <InstallSnippet className="max-w-md">{NPM_INSTALL}</InstallSnippet>
          <div className="flex flex-col gap-3 sm:flex-row sm:items-center">
            <Button size="lg" className="chroma-glow min-w-[10rem]" asChild>
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

        <div className="relative w-full shrink-0 overflow-hidden rounded-[var(--radius-lg)] border border-border bg-card/40 lg:max-w-[52%]">
          <Image
            src="/desktop-topology-hero.png"
            alt={`${DESKTOP_PRODUCT_NAME} showing structural topology and run inspector`}
            width={1280}
            height={720}
            className="h-auto w-full object-cover"
            priority
            sizes="(max-width: 1024px) 100vw, 52vw"
          />
          <p className="border-t border-border px-4 py-2.5 text-xs text-muted-foreground">
            {DESKTOP_PRODUCT_NAME} - structural topology, not a wall of terminals
          </p>
        </div>
      </div>
    </section>
  );
}
