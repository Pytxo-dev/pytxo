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
import { GITHUB_URL } from "@/lib/site";

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
          Install the Pytxo v0.1.0 CLI — npm, install script, or build from source.
          Running <code className="text-foreground/90">pytxo</code> opens the terminal
          dashboard.
        </p>
      </div>

      <div className="mt-12 flex flex-col gap-5">
        <Card className="glass-panel chroma-edge-top border-white/8">
          <CardHeader>
            <div className="flex items-center gap-2">
              <CardTitle>npm (recommended)</CardTitle>
              <Badge className="bg-primary/20 text-primary">v0.1.0</Badge>
            </div>
            <CardDescription>Node 18+ — downloads the matching release binary</CardDescription>
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
            <CardDescription>macOS, Linux, or Windows PowerShell</CardDescription>
          </CardHeader>
          <CardContent className="flex flex-col gap-4">
            <pre className="overflow-x-auto rounded-lg border border-white/10 bg-black/40 p-4 font-mono text-sm text-foreground/90">
              {`# macOS / Linux\ncurl -fsSL https://raw.githubusercontent.com/Pytxo-dev/pytxo/main/tooling/scripts/install.sh | bash\n\n# Windows PowerShell\nirm https://raw.githubusercontent.com/Pytxo-dev/pytxo/main/tooling/scripts/install.ps1 | iex`}
            </pre>
          </CardContent>
        </Card>

        <Card className="glass-panel border-white/8">
          <CardHeader>
            <div className="flex items-center gap-2">
              <CardTitle>From source</CardTitle>
              <Badge variant="secondary" className="bg-white/5">
                Rust 1.85+
              </Badge>
            </div>
            <CardDescription>For contributors and bleeding-edge builds</CardDescription>
          </CardHeader>
          <CardContent className="flex flex-col gap-4">
            <pre className="overflow-x-auto rounded-lg border border-white/10 bg-black/40 p-4 font-mono text-sm text-foreground/90">
              git clone https://github.com/Pytxo-dev/pytxo.git{"\n"}
              cd pytxo{"\n"}
              cargo install --path crates/pytxo-cli
            </pre>
            <Button variant="outline" className="border-white/15 bg-card/30" asChild>
              <a href={GITHUB_URL} target="_blank" rel="noopener noreferrer">
                Open repository
              </a>
            </Button>
          </CardContent>
        </Card>

        <Card className="glass-panel border-white/8">
          <CardHeader>
            <CardTitle>Reality Deck (optional)</CardTitle>
            <CardDescription>Svelte 5 + Tauri v2 telemetry dashboard</CardDescription>
          </CardHeader>
          <CardContent className="flex flex-col gap-4">
            <pre className="overflow-x-auto rounded-lg border border-white/10 bg-black/40 p-4 font-mono text-sm text-foreground/90">
              cd apps/desktop{"\n"}
              npm install{"\n"}
              npm run tauri dev
            </pre>
            <p className="text-sm text-muted-foreground">
              See{" "}
              <Link href="/docs/developers/repo-layout" className="text-primary hover:underline">
                repository layout
              </Link>{" "}
              for monorepo structure.
            </p>
          </CardContent>
        </Card>

        <Card className="glass-panel border-white/8">
          <CardHeader>
            <CardTitle>Next steps</CardTitle>
          </CardHeader>
          <CardContent className="flex flex-col gap-3">
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
