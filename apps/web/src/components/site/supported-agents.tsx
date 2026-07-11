import Link from "next/link";

import { Badge } from "@/components/ui/badge";
import {
  Table,
  TableBody,
  TableCell,
  TableHead,
  TableHeader,
  TableRow,
} from "@/components/ui/table";

const AGENTS = [
  {
    name: "Antigravity CLI (agy)",
    role: "Background agent process",
    config: 'cli_adapter = "agy"',
    mode: "Terminal agent",
  },
  {
    name: "Claude Code",
    role: "Background agent process",
    config: 'cli_adapter = "claude_code"',
    mode: "Your API keys",
  },
  {
    name: "OpenAI Codex CLI",
    role: 'pytxo run --cmd "codex …"',
    config: "generic adapter",
    mode: "Your API keys",
  },
  {
    name: "Aider / custom scripts",
    role: "Any shell command",
    config: "generic adapter",
    mode: "Your API keys",
  },
  {
    name: "Cursor",
    role: "Editor integration (MCP)",
    config: "IDE drives Pytxo",
    mode: "Local tools",
  },
] as const;

export function SupportedAgents() {
  return (
    <section className="mx-auto max-w-5xl px-4 py-16 sm:px-6 sm:py-20">
      <div className="mb-8 flex max-w-2xl flex-col gap-3">
        <h2 className="text-2xl font-semibold sm:text-3xl">Works with your CLI</h2>
        <p className="text-muted-foreground">
          Pytxo coordinates the agent. You choose the CLI. Run Antigravity (agy),
          Claude Code, Codex, or any shell command via{" "}
          <code className="text-foreground/90">pytxo run --cmd</code>.
        </p>
      </div>
      <div className="overflow-hidden rounded-xl border border-white/10 bg-card/30">
        <Table>
          <TableHeader>
            <TableRow className="border-white/8 hover:bg-transparent">
              <TableHead>Agent</TableHead>
              <TableHead>How it runs</TableHead>
              <TableHead className="hidden sm:table-cell">Config hint</TableHead>
              <TableHead>Mode</TableHead>
            </TableRow>
          </TableHeader>
          <TableBody>
            {AGENTS.map((agent) => (
              <TableRow key={agent.name} className="border-white/8">
                <TableCell className="font-medium">{agent.name}</TableCell>
                <TableCell className="text-muted-foreground">{agent.role}</TableCell>
                <TableCell className="hidden font-mono text-xs text-muted-foreground sm:table-cell">
                  {agent.config}
                </TableCell>
                <TableCell>
                  <Badge variant="outline" className="border-white/10 bg-card/30">
                    {agent.mode}
                  </Badge>
                </TableCell>
              </TableRow>
            ))}
          </TableBody>
        </Table>
      </div>
      <p className="mt-6 text-sm text-muted-foreground">
        <Link
          href="/docs/getting-started/testing-ade-clis"
          className="font-medium text-primary hover:underline"
        >
          Testing terminal agent CLIs with Pytxo
        </Link>
      </p>
    </section>
  );
}
