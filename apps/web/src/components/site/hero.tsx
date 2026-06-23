import Image from "next/image";
import Link from "next/link";

import { Badge } from "@/components/ui/badge";
import { Button } from "@/components/ui/button";
import { NPM_INSTALL } from "@/lib/site";

const STATS = [
  { label: "Local-first", detail: "PTY + worktrees on your machine" },
  { label: "Rust core", detail: "Low-overhead orchestration" },
  { label: "BYOK", detail: "Your keys, your providers" },
] as const;

export function Hero() {
  return (
    <section className="relative overflow-hidden">
      <div className="relative mx-auto flex max-w-6xl flex-col items-center gap-12 px-4 py-24 text-center sm:px-6 sm:py-32 lg:flex-row lg:items-center lg:gap-16 lg:text-left">
        <div className="flex flex-1 flex-col gap-7">
          <div className="flex flex-wrap items-center justify-center gap-2 lg:justify-start">
            <Badge variant="secondary" className="bg-white/5 text-primary">
              Mission control for agent fleets
            </Badge>
            {STATS.map((stat) => (
              <Badge
                key={stat.label}
                variant="outline"
                className="border-white/10 bg-card/30 text-muted-foreground"
                title={stat.detail}
              >
                {stat.label}
              </Badge>
            ))}
          </div>
          <h1 className="text-4xl font-semibold leading-[1.1] tracking-tight sm:text-5xl lg:text-6xl">
            Coordinate headless agents.{" "}
            <span className="chroma-text">See the blast radius.</span>
          </h1>
          <p className="max-w-xl text-lg leading-relaxed text-muted-foreground lg:max-w-none">
            Trust your folder, pick a permission tier, and dispatch BYOK agents across
            DeepSeek, OpenRouter, Groq, and more — in isolated copies with wave plans you
            preview first. Multi-folder projects, cross-repo fleet runs, Galaxy approvals,
            and optional Reality Deck topology.
          </p>
          <pre className="mx-auto max-w-md overflow-x-auto rounded-lg border border-white/10 bg-black/40 px-4 py-3 font-mono text-sm text-foreground/90 lg:mx-0">
            {NPM_INSTALL}
          </pre>
          <div className="flex flex-col items-center gap-3 sm:flex-row lg:justify-start">
            <Button size="lg" className="chroma-glow min-w-[10rem]" asChild>
              <Link href="/download">Get started</Link>
            </Button>
            <Button
              size="lg"
              variant="outline"
              className="min-w-[10rem] border-white/15 bg-card/30"
              asChild
            >
              <Link href="/docs">Read the docs</Link>
            </Button>
            <Button
              size="lg"
              variant="ghost"
              className="min-w-[10rem] text-muted-foreground"
              asChild
            >
              <Link href="/download">Download</Link>
            </Button>
          </div>
        </div>
        <div className="chroma-border relative shrink-0 overflow-hidden p-10 chroma-glow">
          <div
            className="pointer-events-none absolute inset-0 rounded-[inherit] opacity-50"
            aria-hidden
            style={{
              background:
                "radial-gradient(circle at 50% 40%, color-mix(in oklab, var(--brand-violet) 25%, transparent), transparent 65%)",
            }}
          />
          <Image
            src="/logo.png"
            alt="Pytxo logo"
            width={280}
            height={280}
            className="relative size-44 sm:size-52 lg:size-60"
            priority
          />
        </div>
      </div>
    </section>
  );
}
