"use client";

import { useRef, useState } from "react";

import s from "@/components/site/site.module.css";
import { DEMO_COMMAND } from "@/lib/site";

type CopyCommandProps = {
  /** The paper block, or the header's quieter text button. */
  variant?: "block" | "nav";
  /** Passed to the hero's mark, which dims its glyphs under this button. */
  keep?: string;
};

/** The demo command as one button: a click copies it, and the label says whether that worked. */
export function CopyCommand({ variant = "block", keep }: CopyCommandProps) {
  const [state, setState] = useState<"idle" | "copied" | "failed">("idle");
  const timer = useRef<number | undefined>(undefined);

  async function copy() {
    try {
      await navigator.clipboard.writeText(DEMO_COMMAND);
      setState("copied");
    } catch {
      setState("failed");
    }
    window.clearTimeout(timer.current);
    timer.current = window.setTimeout(() => setState("idle"), 1600);
  }

  return (
    <>
      <button
        type="button"
        className={variant === "nav" ? s.navCopy : s.cmd}
        aria-label={`Copy ${DEMO_COMMAND}`}
        data-keep={keep}
        onClick={copy}
      >
        <code>{DEMO_COMMAND}</code>
        <span className={s.label}>{state === "copied" ? "Copied" : state === "failed" ? "Copy failed" : "Copy"}</span>
      </button>
      <span role="status" className="sr-only">
        {state === "copied" ? `Copied ${DEMO_COMMAND}` : state === "failed" ? "Copy failed" : ""}
      </span>
    </>
  );
}
