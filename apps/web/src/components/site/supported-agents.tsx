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
    role: "Headless PTY child",
    config: 'cli_adapter = "agy"',
    mode: "Primary ADE",
  },
  {
    name: "Claude Code",
    role: "Headless PTY child",
    config: 'cli_adapter = "claude_code"',
    mode: "BYOK",
  },
  {
    name: "OpenAI Codex CLI",
    role: 'pytxo run --cmd "codex …"',
    config: "generic adapter",
    mode: "BYOK",
  },
  {
    name: "Aider / custom scripts",
    role: "Any shell command",
    config: "generic adapter",
    mode: "BYOK",
  },
  {
    name: "Cursor",
    role: "MCP (pytxo-mcp)",
    config: "IDE drives Pytxo",
    mode: "Local MCP",
  },
] as const;

export function SupportedAgents() {
  return (
    <section className="mx-auto max-w-5xl px-4 py-16 sm:px-6 sm:py-20">
      <div className="mb-8 flex flex-col gap-3 text-center">
        <h2 className="text-2xl font-semibold sm:text-3xl">
          Works with <span className="chroma-text">your CLI</span>
        </h2>
        <p className="mx-auto max-w-2xl text-muted-foreground">
          Pytxo is the hypervisor — you bring the terminal agent. Spawn agy, Claude Code,
          Codex, or any command via{" "}
          <code className="text-foreground/90">pytxo run --cmd</code>.
        </p>
      </div>
      <div className="glass-panel overflow-hidden rounded-xl border border-white/8">
        <Table>
          <TableHeader>
            <TableRow className="border-white/8 hover:bg-transparent">
              <TableHead>Agent</TableHead>
              <TableHead>Role in Pytxo</TableHead>
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
      <p className="mt-6 text-center text-sm text-muted-foreground">
        <Link
          href="/docs/getting-started/testing-ade-clis"
          className="font-medium text-primary hover:underline"
        >
          Testing ADE CLIs with Pytxo →
        </Link>
      </p>
    </section>
  );
}
