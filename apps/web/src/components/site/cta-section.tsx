import Link from "next/link";

import { Button } from "@/components/ui/button";
import { InstallSnippet } from "@/components/site/install-snippet";
import { NPM_INSTALL } from "@/lib/site";

export function CtaSection() {
  return (
    <section className="section-pad">
      <div className="mx-auto max-w-3xl border-t border-border pt-12 text-center">
        <h2 className="text-2xl sm:text-3xl">Install once. Run agents in parallel.</h2>
        <p className="mx-auto mt-3 max-w-lg text-muted-foreground">
          CLI for orchestration. Optional Desktop for topology, approvals, and diffs.
        </p>
        <InstallSnippet className="mx-auto mt-6 max-w-sm text-left">{NPM_INSTALL}</InstallSnippet>
        <div className="mt-6 flex flex-col items-center gap-4 sm:flex-row sm:justify-center">
          <Button size="lg" className="w-full min-w-[10rem] sm:w-auto" asChild>
            <Link href="/download">Download</Link>
          </Button>
          <Link
            href="/docs/getting-started/first-three-agent-run"
            className="text-sm font-medium text-primary underline-offset-4 hover:underline"
          >
            First three-agent run guide
          </Link>
        </div>
      </div>
    </section>
  );
}
