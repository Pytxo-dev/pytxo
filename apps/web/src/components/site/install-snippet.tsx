"use client";

import { useState } from "react";
import { cn } from "@/lib/utils";

type InstallSnippetProps = {
  children: string;
  className?: string;
  copyLabel: string;
};

/** Copy the exact command bytes while allowing long URLs to wrap on narrow screens. */
export function InstallSnippet({ children, className, copyLabel }: InstallSnippetProps) {
  const [copyState, setCopyState] = useState<"idle" | "copied" | "failed">("idle");

  async function copyCommand() {
    try {
      await navigator.clipboard.writeText(children);
      setCopyState("copied");
    } catch {
      setCopyState("failed");
    }
  }

  return (
    <div className={cn("overflow-hidden rounded-[var(--radius-md)] border border-border bg-background/80", className)}>
      <div className="flex items-center justify-between border-b border-border px-4 py-1.5">
        <span className="font-mono text-xs text-muted-foreground">Command</span>
        <button
          type="button"
          aria-label={copyLabel}
          onClick={copyCommand}
          className="min-h-10 rounded-[var(--radius-md)] px-3 text-xs font-medium text-foreground transition-colors hover:bg-muted focus-visible:outline-2 focus-visible:outline-offset-2 focus-visible:outline-ring"
        >
          {copyState === "copied" ? "Copied" : "Copy"}
        </button>
      </div>
      <pre className="max-w-full overflow-x-auto px-4 py-3 font-mono text-sm leading-relaxed text-foreground/90 whitespace-pre-wrap [overflow-wrap:anywhere] sm:whitespace-pre sm:break-normal">
        {children}
      </pre>
      <span role="status" className={copyState === "failed" ? "block px-4 pb-3 text-xs text-destructive" : "sr-only"}>
        {copyState === "copied" ? "Command copied to clipboard." : copyState === "failed" ? "Copy unavailable. Select the command to copy it." : ""}
      </span>
    </div>
  );
}
