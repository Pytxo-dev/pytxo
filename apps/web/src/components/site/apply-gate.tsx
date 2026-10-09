"use client";

import { useEffect, useRef, useState } from "react";

type Line = { at: number; cells: Array<[string, Tone?]> };
type Tone = "ok" | "fail" | "dim" | "cmd";

// Two illustrative runs of one request. In the first, every agent passes its own
// checks and the combined change still fails, so nothing is written; the second
// passes on the combined change and Apply opens.
const RUNS: Line[][] = [
  [
    { at: 0, cells: [["$ ", "dim"], ["pytxo run ", "cmd"], ['"add search, filters and dark mode"']] },
    { at: 900, cells: [["  codex   ", "dim"], ["search     "], ["✓ ", "ok"], ["own checks passed", "dim"]] },
    { at: 1500, cells: [["  claude  ", "dim"], ["filters    "], ["✓ ", "ok"], ["own checks passed", "dim"]] },
    { at: 2100, cells: [["  cursor  ", "dim"], ["dark-mode  "], ["✓ ", "ok"], ["own checks passed", "dim"]] },
    { at: 3000, cells: [["  gate    ", "dim"], ["combined   "], ["npm test", "cmd"]] },
    { at: 4100, cells: [["                     "], ["✗ 2 failed ", "fail"], ["search resets on theme toggle", "dim"]] },
    { at: 5000, cells: [["  apply   ", "dim"], ["blocked", "fail"], [" · nothing written to your project", "dim"]] },
  ],
  [
    { at: 0, cells: [["$ ", "dim"], ["pytxo run ", "cmd"], ['"keep the search term across themes"']] },
    { at: 900, cells: [["  claude  ", "dim"], ["filters    "], ["✓ ", "ok"], ["own checks passed", "dim"]] },
    { at: 1800, cells: [["  gate    ", "dim"], ["combined   "], ["npm test", "cmd"]] },
    { at: 2900, cells: [["                     "], ["✓ 48 passed", "ok"]] },
    { at: 3700, cells: [["  apply   ", "dim"], ["ready", "ok"], [" · 7 reviewed files → todo-app", "dim"]] },
  ],
];
const HOLD = 3200;
const TONE: Record<Tone, string> = {
  ok: "text-[var(--state-verified)]",
  fail: "text-[var(--state-refuted)]",
  dim: "text-[#7c7c86]",
  cmd: "text-[#f5f5f7]",
};

/** A terminal-style illustration of the Apply gate; the final frame of the first run without motion. */
export function ApplyGate({ className }: { className?: string }) {
  const box = useRef<HTMLDivElement>(null);
  const [run, setRun] = useState(0);
  const [shown, setShown] = useState(RUNS[0].length);

  useEffect(() => {
    const element = box.current;
    if (!element || matchMedia("(prefers-reduced-motion: reduce)").matches) return;
    let timers: number[] = [];
    let index = 0;
    const play = () => {
      const lines = RUNS[index];
      setRun(index);
      setShown(0);
      timers = lines.map((line, n) => window.setTimeout(() => setShown(n + 1), line.at + 250));
      timers.push(window.setTimeout(() => { index = (index + 1) % RUNS.length; play(); }, lines[lines.length - 1].at + HOLD));
    };
    const stop = () => { timers.forEach(clearTimeout); timers = []; };
    const observer = new IntersectionObserver(([entry]) => {
      if (entry.isIntersecting && !timers.length) { index = 0; play(); }
      else if (!entry.isIntersecting) { stop(); setRun(0); setShown(RUNS[0].length); }
    }, { threshold: .4 });
    observer.observe(element);
    return () => { observer.disconnect(); stop(); };
  }, []);

  const lines = RUNS[run];
  const settled = shown >= lines.length;
  return <figure className={className}>
    <div ref={box} className="relative rounded-md border border-white/15 bg-[#09090b] px-4 pb-4 pt-6 font-mono text-[10.5px] leading-[1.75] sm:px-5 sm:text-[13px]" role="img"
      aria-label="Illustration: three agents each pass their own checks, the checks on their combined change fail, and Apply stays blocked. In a second run the combined checks pass and Apply is ready.">
      <span aria-hidden className="absolute -top-[9px] left-4 bg-[#0b0b0d] px-2 text-[11px] uppercase tracking-[0.14em] text-[#c4c4cc]">apply gate <span className="text-[#7c7c86]">· run {run + 1}/2</span></span>
      <div aria-hidden className="min-h-[13.2em] overflow-x-auto whitespace-pre">
        {lines.slice(0, shown).map((line, n) => <div key={`${run}-${n}`} className="animate-[gate-line_.28s_steps(6)_both]">
          {line.cells.map(([text, tone], c) => <span key={c} className={tone ? TONE[tone] : "text-[#c4c4cc]"}>{text}</span>)}
          {n === shown - 1 && !settled ? <span className="ml-0.5 inline-block h-[1.05em] w-[0.6em] translate-y-[0.18em] bg-[#45dccb] animate-[gate-caret_1s_steps(1)_infinite]" /> : null}
        </div>)}
      </div>
    </div>
    <figcaption className="mt-3 text-xs text-[#7c7c86]">Illustration of Pytxo&apos;s Apply gate, not a recorded run.</figcaption>
  </figure>;
}
