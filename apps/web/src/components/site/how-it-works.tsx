import Link from "next/link";

import { Button } from "@/components/ui/button";
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from "@/components/ui/card";
import { Separator } from "@/components/ui/separator";

const STEPS = [
  {
    step: "01",
    title: "Install",
    description: "Build the Rust CLI, run pytxo init and pytxo doctor in your git repo.",
    href: "/docs/getting-started/install",
  },
  {
    step: "02",
    title: "Wire MCP",
    description: "Point Cursor or your IDE at the local pytxo-mcp server for orchestration tools.",
    href: "/docs/getting-started/mcp-from-cursor",
  },
  {
    step: "03",
    title: "Run a wave",
    description: "Schedule agents with pytxo.toml — dry-run first, then execute in isolated worktrees.",
    href: "/docs/getting-started/first-three-agent-run",
  },
] as const;

export function HowItWorks() {
  return (
    <section className="border-y border-white/5 bg-card/15">
      <div className="mx-auto max-w-6xl px-4 py-20 sm:px-6 sm:py-24">
        <div className="text-center">
          <h2 className="text-3xl font-semibold tracking-tight sm:text-4xl">
            Up and running in <span className="chroma-text">three steps</span>
          </h2>
          <p className="mx-auto mt-4 max-w-xl text-muted-foreground">
            No cloud signup. No terminal grid. Just a hypervisor on your machine.
          </p>
        </div>

        <div className="mt-12 grid gap-6 md:grid-cols-3">
          {STEPS.map((item, index) => (
            <div key={item.step} className="flex flex-col gap-6">
              <Card className="glass-panel h-full border-white/8">
                <CardHeader>
                  <p className="text-xs font-medium uppercase tracking-widest text-primary">
                    {item.step}
                  </p>
                  <CardTitle>{item.title}</CardTitle>
                  <CardDescription className="leading-relaxed">
                    {item.description}
                  </CardDescription>
                </CardHeader>
                <CardContent>
                  <Button variant="link" className="h-auto p-0 text-primary" asChild>
                    <Link href={item.href}>Read guide →</Link>
                  </Button>
                </CardContent>
              </Card>
              {index < STEPS.length - 1 ? (
                <Separator className="hidden bg-white/10 md:hidden" />
              ) : null}
            </div>
          ))}
        </div>
      </div>
    </section>
  );
}
