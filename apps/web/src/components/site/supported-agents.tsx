"use client";

import { useState } from "react";
import Link from "next/link";

import { cn } from "@/lib/utils";

const AGENTS = [
  {
    id: "claude",
    name: "Claude Code",
    role: "Background agent process",
    config: 'cli_adapter = "claude_code"',
    mode: "Claude Code-owned session",
  },
  {
    id: "codex",
    name: "OpenAI Codex CLI",
    role: 'pytxo run --cmd "codex ..."',
    config: "generic adapter",
    mode: "Codex-owned ChatGPT or API session",
  },
  {
    id: "cursor",
    name: "Cursor Agent",
    role: "Headless CLI or editor integration through MCP",
    config: "cursor-agent",
    mode: "Cursor-owned session",
  },
  {
    id: "gemini",
    name: "Gemini CLI",
    role: "Background agent process",
    config: "gemini",
    mode: "Gemini CLI-owned session",
  },
  {
    id: "opencode",
    name: "OpenCode",
    role: "Provider-flexible background agent",
    config: "opencode",
    mode: "OpenCode-owned provider session",
  },
  {
    id: "custom",
    name: "Aider, Antigravity, or scripts",
    role: "Any shell command",
    config: "generic adapter",
    mode: "Explicit selected credential only",
  },
] as const;

export function SupportedAgents() {
  const [active, setActive] = useState<(typeof AGENTS)[number]["id"]>("claude");

  return (
    <section className="mx-auto max-w-6xl px-4 py-16 sm:px-6 sm:py-20">
      <div className="mb-9 max-w-2xl">
        <h2 className="text-2xl font-semibold sm:text-3xl">Keep the agent CLI you trust</h2>
        <p className="mt-3 text-muted-foreground">
          Pytxo coordinates the process, paths, and approvals. Your chosen coding agent still does
          the work.
        </p>
      </div>

      <div className="hidden h-[22rem] gap-2 lg:flex" aria-label="Supported agent CLIs">
        {AGENTS.map((agent) => {
          const expanded = active === agent.id;
          return (
            <article
              key={agent.id}
              className={cn(
                "min-w-0 overflow-hidden rounded-[var(--radius-lg)] border bg-card/20 transition-[flex-grow,border-color,background-color] duration-500 ease-out",
                expanded ? "border-primary/35 bg-primary/[0.04]" : "border-border",
              )}
              style={{ flexBasis: 0, flexGrow: expanded ? 2.4 : 1 }}
              onMouseEnter={() => setActive(agent.id)}
            >
              <button
                type="button"
                aria-expanded={expanded}
                className="flex h-full w-full flex-col p-5 text-left outline-none transition-colors focus-visible:ring-2 focus-visible:ring-ring focus-visible:ring-inset active:translate-y-px"
                onClick={() => setActive(agent.id)}
                onFocus={() => setActive(agent.id)}
              >
                <span className="text-base font-semibold tracking-tight">{agent.name}</span>
                <span
                  className={cn(
                    "mt-auto grid gap-5 transition-opacity duration-300",
                    expanded ? "opacity-100" : "pointer-events-none opacity-0",
                  )}
                  aria-hidden={!expanded}
                >
                  <span className="text-sm leading-relaxed text-muted-foreground">
                    {agent.role}
                  </span>
                  <span className="border-t border-border pt-4">
                    <span className="block text-xs text-muted-foreground">Configuration</span>
                    <code className="mt-1 block font-mono text-sm text-foreground">
                      {agent.config}
                    </code>
                  </span>
                  <span className="text-sm font-medium text-primary">{agent.mode}</span>
                </span>
              </button>
            </article>
          );
        })}
      </div>

      <div className="grid gap-3 lg:hidden">
        {AGENTS.map((agent) => (
          <article
            key={agent.id}
            className="rounded-[var(--radius-lg)] border border-border bg-card/20 p-5"
          >
            <h3 className="font-semibold tracking-tight">{agent.name}</h3>
            <p className="mt-2 text-sm text-muted-foreground">{agent.role}</p>
            <div className="mt-4 grid gap-1 border-t border-border pt-4 sm:grid-cols-2">
              <code className="font-mono text-xs text-foreground">{agent.config}</code>
              <span className="text-xs text-primary sm:text-right">{agent.mode}</span>
            </div>
          </article>
        ))}
      </div>

      <p className="mt-7 text-sm text-muted-foreground">
        <Link
          href="/docs/developers/testing-agent-clis"
          className="font-medium text-primary underline-offset-4 hover:underline"
        >
          Test a terminal agent CLI with Pytxo
        </Link>
      </p>
    </section>
  );
}
