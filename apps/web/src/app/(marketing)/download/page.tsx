import type { Metadata } from "next";
import Link from "next/link";

import { Badge } from "@/components/ui/badge";
import { Button } from "@/components/ui/button";
import {
  Card,
  CardContent,
  CardDescription,
  CardHeader,
  CardTitle,
} from "@/components/ui/card";
import {
  DESKTOP_DOWNLOADS,
  GITHUB_URL,
  INSTALL_PS1_CMD,
  INSTALL_SH_CMD,
  NPM_URL,
  PYTXO_VERSION,
  RELEASES_URL,
} from "@/lib/site";

export const metadata: Metadata = {
  title: "Download",
  description: "Install the Pytxo CLI and optional Reality Deck desktop app.",
};

export default function DownloadPage() {
  return (
    <div className="mx-auto max-w-3xl px-4 py-20 sm:px-6 sm:py-28">
      <div className="flex flex-col gap-4">
        <h1 className="text-4xl font-semibold tracking-tight sm:text-5xl">
          <span className="chroma-text">Download</span>
        </h1>
        <p className="text-lg text-muted-foreground">
          Install Pytxo v{PYTXO_VERSION} via npm or the public install script. Running{" "}
          <code className="text-foreground/90">pytxo</code> opens the Hypervisor Shell.
        </p>
      </div>

      <div className="mt-12 flex flex-col gap-5">
        <Card className="glass-panel chroma-edge-top border-white/8">
          <CardHeader>
            <div className="flex items-center gap-2">
              <CardTitle>
                <a
                  href={NPM_URL}
                  target="_blank"
                  rel="noopener noreferrer"
                  className="hover:text-primary"
                >
                  npm (recommended)
                </a>
              </CardTitle>
              <Badge className="bg-primary/20 text-primary">v{PYTXO_VERSION}</Badge>
            </div>
            <CardDescription>
              Node 18+ — downloads the matching binary from public GitHub Releases
            </CardDescription>
          </CardHeader>
          <CardContent className="flex flex-col gap-4">
            <pre className="overflow-x-auto rounded-lg border border-white/10 bg-black/40 p-4 font-mono text-sm text-foreground/90">
              npm i -g pytxo{"\n"}
              pytxo doctor
            </pre>
          </CardContent>
        </Card>

        <Card className="glass-panel border-white/8">
          <CardHeader>
            <CardTitle>Install script</CardTitle>
            <CardDescription>
              macOS, Linux, or Windows PowerShell — no Node required
            </CardDescription>
          </CardHeader>
          <CardContent className="flex flex-col gap-4">
            <pre className="overflow-x-auto rounded-lg border border-white/10 bg-black/40 p-4 font-mono text-sm text-foreground/90">
              {`# macOS / Linux\n${INSTALL_SH_CMD}\n\n# Windows PowerShell\n${INSTALL_PS1_CMD}`}
            </pre>
            <p className="text-sm text-muted-foreground">
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
          </CardContent>
        </Card>

        <Card className="glass-panel border-white/8">
          <CardHeader>
            <CardTitle>Prebuilt binaries</CardTitle>
            <CardDescription>Manual download per platform</CardDescription>
          </CardHeader>
          <CardContent className="flex flex-col gap-4">
            <p className="text-sm text-muted-foreground">
              Grab <code className="text-foreground/90">pytxo-linux-x64</code>,{" "}
              <code className="text-foreground/90">pytxo-darwin-arm64</code>,{" "}
              <code className="text-foreground/90">pytxo-windows-x64.exe</code>, and checksums
              from GitHub Releases.
            </p>
            <Button variant="outline" className="border-white/15 bg-card/30" asChild>
              <a href={RELEASES_URL} target="_blank" rel="noopener noreferrer">
                Browse releases
              </a>
            </Button>
          </CardContent>
        </Card>

        <Card className="glass-panel chroma-edge-top border-white/8">
          <CardHeader>
            <CardTitle>
              <span className="chroma-text">Reality Deck</span>
            </CardTitle>
            <CardDescription>
              Svelte 5 + Tauri v2 desktop telemetry — Chroma-themed space console
            </CardDescription>
          </CardHeader>
          <CardContent className="flex flex-col gap-4">
            <p className="text-sm text-muted-foreground">
              Passive telemetry for runs, waves, logs, diffs, and AST blast-radius topology.
              Auto-updates via the built-in Tauri updater when new releases ship.
            </p>
            <div className="grid gap-2 sm:grid-cols-2">
              <Button variant="outline" className="border-white/15 bg-card/30" asChild>
                <a href={DESKTOP_DOWNLOADS.windows}>Windows (.msi)</a>
              </Button>
              <Button variant="outline" className="border-white/15 bg-card/30" asChild>
                <a href={DESKTOP_DOWNLOADS.macosArm}>macOS Apple Silicon (.dmg)</a>
              </Button>
              <Button variant="outline" className="border-white/15 bg-card/30" asChild>
                <a href={DESKTOP_DOWNLOADS.macosX64}>macOS Intel (.dmg)</a>
              </Button>
              <Button variant="outline" className="border-white/15 bg-card/30" asChild>
                <a href={DESKTOP_DOWNLOADS.linux}>Linux (.AppImage)</a>
              </Button>
            </div>
            <p className="text-xs text-muted-foreground">
              Requires the Pytxo CLI for orchestration, or use the bundled sidecar when available.
              Installers are published alongside CLI binaries on{" "}
              <a
                href={RELEASES_URL}
                target="_blank"
                rel="noopener noreferrer"
                className="text-primary hover:underline"
              >
                GitHub Releases
              </a>
              .
            </p>
          </CardContent>
        </Card>

        <Card className="glass-panel border-white/8">
          <CardHeader>
            <CardTitle>Next steps</CardTitle>
          </CardHeader>
          <CardContent className="flex flex-col gap-3">
            <Link
              href="/docs/getting-started/folder-trust"
              className="text-sm font-medium text-primary hover:underline"
            >
              Folder trust & permission tiers →
            </Link>
            <Link
              href="/docs/getting-started/testing-ade-clis"
              className="text-sm font-medium text-primary hover:underline"
            >
              Testing ADE CLIs (agy, Claude, Codex) →
            </Link>
            <Link
              href="/docs/getting-started/first-three-agent-run"
              className="text-sm font-medium text-primary hover:underline"
            >
              Tutorial: first three-agent run →
            </Link>
            <Link
              href="/docs/getting-started/mcp-from-cursor"
              className="text-sm font-medium text-primary hover:underline"
            >
              Wire Cursor MCP →
            </Link>
            <Link
              href="/docs/reference/pytxo-toml"
              className="text-sm font-medium text-primary hover:underline"
            >
              pytxo.toml reference →
            </Link>
          </CardContent>
        </Card>
      </div>
    </div>
  );
}
