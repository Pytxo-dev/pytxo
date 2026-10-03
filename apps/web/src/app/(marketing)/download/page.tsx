import Link from "next/link";
import { pageMetadata } from "@/lib/page-metadata";

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
  CANDIDATE_PUBLISHED,
  CANDIDATE_VERSION,
  PUBLISHED_VERSION,
  RELEASES_URL,
} from "@/lib/site";

export const metadata = pageMetadata(
  "/download",
  "Download · Pytxo",
  "Windows: Pytxo Desktop plus an agent CLI you already use. macOS and Linux: the Pytxo CLI.",
);

export default function DownloadPage() {
  return (
    <div className="mx-auto max-w-[76rem] px-4 py-14 sm:px-6 sm:py-20 lg:px-10">
      <div className="flex flex-col gap-3">
        <div className="flex flex-wrap items-center gap-3">
          <h1 className="text-4xl font-semibold tracking-tight sm:text-5xl">Download</h1>
          <Badge
            variant="outline"
            className="border-primary/30 bg-primary/10 font-mono text-xs text-primary"
          >
            v{PUBLISHED_VERSION}
          </Badge>
        </div>
        <p className="max-w-2xl text-lg text-muted-foreground">
          On Windows, install Pytxo Desktop and at least one agent CLI you already use: Codex,
          Claude Code, Cursor Agent, OpenCode or Antigravity. On macOS and Linux, start with the
          Pytxo CLI; Desktop for them is not built yet.
        </p>
        {!CANDIDATE_PUBLISHED && <p className="max-w-2xl text-sm text-muted-foreground">
          The homepage shows the upcoming v{CANDIDATE_VERSION}, which adds mixed-agent runs. The
          download below installs the current public v{PUBLISHED_VERSION}, which runs Codex.
        </p>}
      </div>

      <div className="mt-10 flex flex-col gap-10">
        <DownloadDesktop />

        <section className="border-t border-border pt-10">
          <div className="flex flex-wrap items-baseline gap-2">
            <h2 className="text-xl font-semibold tracking-tight sm:text-2xl">CLI</h2>
            <span className="text-sm text-muted-foreground">
              Required on macOS and Linux; optional on Windows with Desktop
            </span>
          </div>
          <p className="mt-1 max-w-xl text-sm text-muted-foreground">
            Terminal missions, doctor checks, and MCP. Desktop on Windows already includes the
            local core. The CLI does not Apply repository changes; Apply stays in Desktop Review.
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
              <InstallSnippet className="mt-3" copyLabel="Copy npm setup commands">{`${NPM_INSTALL}\npytxo doctor`}</InstallSnippet>
            </div>

            <div>
              <h3 className="text-sm font-medium">Install script</h3>
              <p className="mt-1 text-sm text-muted-foreground">
                Windows PowerShell is first-class. macOS and Linux scripts are also available. No Node required.
              </p>
              <div className="mt-4 grid gap-4">
                <div>
                  <h4 className="text-sm font-medium">Windows PowerShell</h4>
                  <InstallSnippet className="mt-2" copyLabel="Copy Windows PowerShell install command">{INSTALL_PS1_CMD}</InstallSnippet>
                </div>
                <div>
                  <h4 className="text-sm font-medium">macOS / Linux</h4>
                  <InstallSnippet className="mt-2" copyLabel="Copy macOS and Linux install command">{INSTALL_SH_CMD}</InstallSnippet>
                </div>
              </div>
              <p className="mt-3 text-sm text-muted-foreground">
                Binaries and scripts live in the public{" "}
                <a
                  href={GITHUB_URL}
                  target="_blank"
                  rel="noopener noreferrer"
                  className="text-primary hover:underline"
                >
                  pytxo-releases
                </a>{" "}
                repository. The product source repository is currently private.
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
              { href: "/docs/getting-started/install", label: "Install by platform" },
              { href: "/docs/getting-started/first-mission", label: "First mission" },
              { href: "/docs/getting-started/review-and-apply", label: "Review and Apply" },
              { href: "/docs/getting-started/first-three-agent-run", label: "Collision demo" },
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
