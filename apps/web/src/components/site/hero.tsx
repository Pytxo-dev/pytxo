import Image from "next/image";
import Link from "next/link";

import { Button } from "@/components/ui/button";

export function Hero() {
  return (
    <section className="aperture-grid relative overflow-hidden border-b border-white/[0.06]" data-testid="marketing-hero">
      <div className="pointer-events-none absolute inset-x-[7%] top-[-30%] h-[70%] bg-[radial-gradient(ellipse_at_center,rgba(240,77,163,0.09),rgba(69,220,203,0.04)_35%,transparent_70%)]" />
      <div className="relative mx-auto grid min-h-[calc(100dvh-4rem)] max-w-[92rem] items-center gap-14 px-4 py-20 sm:px-6 lg:grid-cols-[minmax(0,1fr)_minmax(36rem,1.04fr)] lg:px-10 lg:py-24">
        <div className="flex w-full min-w-0 flex-col items-start text-left">
          {/*
            The headline is sized to fit inside its own grid column. It must never
            be allowed to size to its own content, or it overflows the column and
            renders underneath the product frame in the next column.
          */}
          <h1 className="max-w-full text-pretty text-[clamp(2.5rem,3.6vw,3.4rem)] font-medium leading-[1.02] tracking-[-0.05em]">
            <span className="block">Coordinate coding agents.</span>
            <span className="block">Review one result.</span>
          </h1>
          <p className="mt-7 max-w-[38rem] text-base leading-relaxed text-[#a9a9b2] sm:text-lg">
            Pytxo runs the agent CLIs you already have in isolated workspaces, keeps
            them off each other&apos;s files, and prepares one reviewable package. Nothing
            reaches your repository until you approve the exact bytes.
          </p>
          <div className="mt-9 flex w-full flex-col items-start gap-4 sm:w-auto sm:flex-row sm:items-center sm:gap-6">
            <Button size="lg" className="h-11 w-full rounded-[4px] bg-white px-6 text-black hover:bg-white/85 sm:w-auto sm:min-w-[11rem]" asChild>
              <Link href="/download">Download Pytxo</Link>
            </Button>
            <Link href="/docs" className="aperture-link text-sm text-[#a9a9b2] transition-colors hover:text-white">
              Read the docs
            </Link>
          </div>
          <p className="mt-8 font-mono text-[12px] leading-relaxed text-[#6f6f79]">
            Local-first. Your agent accounts and keys. Desktop ships for Windows today.
          </p>
        </div>

        <div className="group relative lg:translate-x-8">
          <div className="pointer-events-none absolute -inset-16 bg-[radial-gradient(circle_at_55%_45%,rgba(255,106,74,0.085),rgba(69,220,203,0.045)_35%,transparent_68%)] opacity-80" />
          {/*
            A deliberate crop at 1:1, not a shrunken window. Fitting the whole
            1600x1000 capture into this column renders its 11px operator text
            around 4px, which is a picture of an interface rather than an
            interface. The pixel offsets place the commit boundary panel at the
            frame's top-left, so what the hero shows is actually readable; the
            product section below carries the entire screen.
          */}
          <div className="relative overflow-hidden rounded-[6px] border border-white/15 bg-[#09090b] shadow-[0_32px_100px_rgba(0,0,0,0.48)]">
            <div className="flex h-10 items-center justify-between border-b border-white/[0.08] px-4 font-mono text-[11px] text-[#7d7d87]">
              <span>PYTXO DESKTOP / WORK &mdash; COMMIT BOUNDARY</span>
              <span className="flex items-center gap-2">
                <i className="size-1.5 rounded-full bg-[var(--state-verified)]" />LOCAL
              </span>
            </div>
            <div data-testid="hero-product" className="relative aspect-[16/10] overflow-hidden bg-card">
              <Image
                src="/product/work-1600x1000.png"
                alt="The commit boundary panel in Pytxo Desktop: the candidate outcome for run-8f2c and an enforcement receipt reporting workspace isolation enforced, host filesystem and network advisory only, and apply boundary enforced"
                width={1600}
                height={1000}
                className="absolute left-[-940px] top-[-196px] w-[1600px] max-w-none"
                sizes="1600px"
                priority
              />
            </div>
            {/*
              The only spectrum on the site. It is the brand signature, not a
              state indicator, so it is static and appears exactly once.
            */}
            <div className="execution-trace absolute inset-x-0 bottom-0" aria-hidden />
          </div>
        </div>
      </div>
    </section>
  );
}
