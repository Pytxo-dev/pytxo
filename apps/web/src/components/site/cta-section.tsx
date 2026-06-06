import Link from "next/link";

import { Button } from "@/components/ui/button";
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from "@/components/ui/card";

export function CtaSection() {
  return (
    <section className="overflow-hidden px-4 py-20 sm:px-6 sm:py-24">
      <div className="mx-auto max-w-3xl">
        <Card className="chroma-border chroma-glow overflow-hidden border-0 bg-card/40 text-center shadow-none">
          <CardHeader className="gap-4 px-6 pt-10 sm:px-14 sm:pt-14">
            <CardTitle className="text-2xl font-semibold sm:text-3xl">
              Ready to <span className="chroma-text">orchestrate</span>?
            </CardTitle>
            <CardDescription className="mx-auto max-w-lg text-base text-muted-foreground">
              Install the Rust CLI, wire MCP from Cursor or your IDE, and run your first
              three-agent wave in minutes.
            </CardDescription>
          </CardHeader>
          <CardContent className="flex flex-col items-center gap-3 px-6 pb-10 sm:flex-row sm:justify-center sm:px-14 sm:pb-14">
            <Button size="lg" className="chroma-glow w-full sm:w-auto" asChild>
              <Link href="/download">Download & install</Link>
            </Button>
            <Button
              size="lg"
              variant="outline"
              className="w-full border-white/15 bg-card/30 sm:w-auto"
              asChild
            >
              <Link href="/docs/getting-started/first-three-agent-run">
                First three-agent run
              </Link>
            </Button>
          </CardContent>
        </Card>
      </div>
    </section>
  );
}
