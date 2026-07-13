import type { Metadata } from "next";
import Link from "next/link";

import { Badge } from "@/components/ui/badge";
import { Button } from "@/components/ui/button";
import { DownloadDesktop } from "@/components/site/download-desktop";
import { InstallSnippet } from "@/components/site/install-snippet";
import {
  GITHUB_URL,
  INSTALL_PS1_CMD,
  INSTALL_SH_CMD,
  NPM_INSTALL,
  NPM_URL,
  PYTXO_VERSION,
  RELEASES_URL,
} from "@/lib/site";

export const metadata: Metadata = {
  title: "Download",
  description: "Install Pytxo Desktop and the CLI for local multi-agent orchestration.",
};

export default function DownloadPage() {
  return (
    <div className="mx-auto max-w-4xl px-4 py-16 sm:px-6 sm:py-24">
      <div className="flex flex-col gap-3">
        <div className="flex flex-wrap items-center gap-3">
          <h1 className="text-4xl font-semibold tracking-tight sm:text-5xl">Download</h1>
          <Badge
            variant="outline"
            className="border-primary/30 bg-primary/10 font-mono text-xs text-primary"
          >
            v{PYTXO_VERSION}
          </Badge>
        </div>
        <p className="max-w-2xl text-lg text-muted-foreground">
          Start with Pytxo Desktop for topology, approvals, and diffs. The CLI powers
          orchestration underneath—install it once, or let Desktop guide you.
        </p>
      </div>

      <div className="mt-12 flex flex-col gap-10">
        <DownloadDesktop />

        <section className="border-t border-border pt-10">
          <div className="flex flex-wrap items-baseline gap-2">
            <h2 className="text-xl font-semibold tracking-tight sm:text-2xl">CLI</h2>
            <span className="text-sm text-muted-foreground">Required for runs</span>
          </div>
          <p className="mt-1 max-w-xl text-sm text-muted-foreground">
            Entry point for multi-agent jobs, doctor checks, and MCP. Desktop downloads the
            matching CLI at first run if needed.
          </p>

          <div className="mt-6 flex flex-col gap-6">
            <div>
              <div className="flex flex-wrap items-center justify-between gap-2">
                <h3 className="text-sm font-medium">
                  <a
                    href={NPM_URL}
                    target="_blank"
                    rel="noopener noreferrer"
                    className="hover:text-primary"
                  >
                    npm
                  </a>
                </h3>
                <span className="text-xs text-muted-foreground">Node 18+</span>
              </div>
              <InstallSnippet className="mt-3">{`${NPM_INSTALL}\npytxo doctor`}</InstallSnippet>
            </div>

            <div>
              <h3 className="text-sm font-medium">Install script</h3>
              <p className="mt-1 text-sm text-muted-foreground">
                macOS, Linux, or Windows PowerShell. No Node required.
              </p>
              <InstallSnippet className="mt-3">{`# macOS / Linux\n${INSTALL_SH_CMD}\n\n# Windows PowerShell\n${INSTALL_PS1_CMD}`}</InstallSnippet>
              <p className="mt-3 text-sm text-muted-foreground">
                Scripts live in the public{" "}
                <a
                  href={GITHUB_URL}
                  target="_blank"
                  rel="noopener noreferrer"
                  className="text-primary hover:underline"
                >
                  pytxo-releases
                </a>{" "}
                repository.
              </p>
            </div>

            <div className="flex flex-col gap-3 border-t border-border pt-6 sm:flex-row sm:items-center sm:justify-between">
              <div>
                <h3 className="text-sm font-medium">Prebuilt binaries</h3>
                <p className="mt-1 text-sm text-muted-foreground">
                  Manual download per platform from GitHub Releases.
                </p>
              </div>
              <Button variant="outline" className="border-border" asChild>
                <a href={RELEASES_URL} target="_blank" rel="noopener noreferrer">
                  Browse releases
                </a>
              </Button>
            </div>
          </div>
        </section>

        <section className="border-t border-border pt-10">
          <h2 className="text-lg font-semibold tracking-tight">Next steps</h2>
          <div className="mt-4 grid gap-1 sm:grid-cols-2">
            {[
              { href: "/docs/getting-started/first-three-agent-run", label: "First three-agent run" },
              { href: "/docs/getting-started/mcp-from-cursor", label: "Wire Cursor MCP" },
              { href: "/docs/getting-started/folder-trust", label: "Folder trust" },
              { href: "/docs/concepts/desktop", label: "Pytxo Desktop overview" },
              { href: "/docs/concepts/modular-projects", label: "Workspaces" },
              { href: "/docs/reference/pytxo-toml", label: "pytxo.toml reference" },
            ].map((item) => (
              <Link
                key={item.href}
                href={item.href}
                className="px-1 py-2 text-sm font-medium text-primary transition-colors hover:text-foreground"
              >
                {item.label}
              </Link>
            ))}
          </div>
        </section>
      </div>
    </div>
  );
}
