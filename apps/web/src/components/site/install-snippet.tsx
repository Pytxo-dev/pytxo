"use client";

import { useState, type ReactNode } from "react";

import s from "@/components/site/site.module.css";
import { cn } from "@/lib/utils";

type InstallSnippetProps = {
  children: string;
  /** What the command is for, shown beside the copy button. */
  label: ReactNode;
  className?: string;
  copyLabel: string;
};

/** Copy the exact command bytes while allowing long URLs to wrap on narrow screens. */
export function InstallSnippet({ children, label, className, copyLabel }: InstallSnippetProps) {
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
    <div className={cn(s.snip, className)}>
      <div className={s.snipHead}>
        <span>{label}</span>
        <button type="button" aria-label={copyLabel} onClick={copyCommand} className={cn(s.snipCopy, "touch-target relative")}>
          {copyState === "copied" ? "Copied" : "Copy"}
        </button>
      </div>
      <pre>{children}</pre>
      <span role="status" className={copyState === "failed" ? s.snipFail : "sr-only"}>
        {copyState === "copied" ? "Command copied to clipboard." : copyState === "failed" ? "Copy unavailable. Select the command to copy it." : ""}
      </span>
    </div>
  );
}
