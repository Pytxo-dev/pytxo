"use client";

import { Fragment, useEffect, useRef, useState, useSyncExternalStore } from "react";

import s from "@/components/site/site.module.css";
import { cn } from "@/lib/utils";
import capture from "../../../public/evidence/demo-landed-2.0.2-2026-11-12.json";

const RECORD_URL = "/evidence/demo-landed-2.0.2-2026-11-12.json";
const LINES = capture.output.trimEnd().split("\n");
const BRANCH = new RegExp(`(${Object.keys(capture.repo.branches).join("|")})`);
const LABEL = /^(LANDED|left out|landed|next)( +)(.*)$/;
const TOTAL = capture.duration_ms[0];
const SECONDS = (TOTAL / 1000).toFixed(1);
// When each row shows, in ms from the start: the command at once, then each line when the record says it arrived.
const AT = [0, ...capture.line_ms];
// The fix prompt's body folds under its own first line and ends at the blank line after it.
const FIX = LINES.findIndex((text) => text.startsWith("Fix prompt for "));
const FIX_END = LINES.indexOf("", FIX);

function withBranches(text: string) {
  return text.split(BRANCH).map((part, i) => (i % 2 ? <span key={i} className={s.br}>{part}</span> : part));
}

const REDUCE = "(prefers-reduced-motion: reduce)";
const onMotionChange = (change: () => void) => {
  const query = matchMedia(REDUCE);
  query.addEventListener("change", change);
  return () => query.removeEventListener("change", change);
};

/**
 * The recorded output of `npx pytxo demo`. Every line ships in the HTML, so it reads with JavaScript off; the fix
 * prompt's body sits in a disclosure under its own first line. In view, the rows replay at the recorded times while
 * the clock runs to the recorded total.
 */
export function RecordedDemo() {
  const term = useRef<HTMLDivElement>(null);
  const replay = useRef<() => void>(() => {});
  // The server renders the whole record; in a browser that allows motion it waits at 0 for the replay.
  const motion = useSyncExternalStore(onMotionChange, () => !matchMedia(REDUCE).matches, () => false);
  const [ms, setMs] = useState(0);

  useEffect(() => {
    if (!motion) return;
    let raf = 0, started = false;
    const play = () => {
      started = true;
      cancelAnimationFrame(raf);
      const start = performance.now();
      // A step every 50 ms is enough for the rows and the clock, and saves a render per frame.
      const tick = (now: number) => {
        const elapsed = now - start;
        setMs(elapsed >= TOTAL ? TOTAL : Math.floor(elapsed / 50) * 50);
        if (elapsed < TOTAL) raf = requestAnimationFrame(tick);
      };
      raf = requestAnimationFrame(tick);
    };
    replay.current = play;
    // Any part of the output above the bottom third of the screen starts it. A share of its area would never be reached
    // where the output is several screens tall, as at 400% zoom. A slow main thread can batch entries; the last is current.
    const io = new IntersectionObserver((entries) => {
      if (entries[entries.length - 1].isIntersecting && !started) { io.disconnect(); play(); }
    }, { rootMargin: "0px 0px -35% 0px" });
    io.observe(term.current!);
    return () => { io.disconnect(); cancelAnimationFrame(raf); replay.current = () => {}; };
  }, [motion]);

  const at = motion ? ms : TOTAL, done = at >= TOTAL;
  // Row 0 is the command; line i of the record is row i + 1.
  const on = (row: number) => at >= AT[row] && s.on;
  const line = (text: string, i: number) => {
    const label = LABEL.exec(text);
    return (
      <div key={i} className={cn(s.ln, label && s.lab, !text && s.gap, on(i + 1))}>
        {label ? (
          <Fragment>
            <span className={label[1] === "LANDED" ? s.hit : s.k}>{label[1]}</span>
            {label[2]}
            {withBranches(label[3])}
          </Fragment>
        ) : withBranches(text)}
      </div>
    );
  };
  return (
    <section className={cn(s.sec, "tabular-nums")} id="recorded" aria-labelledby="recorded-title" data-testid="recorded-demo">
      <div className={s.wrap}>
        <div className={s.recHead}>
          <h2 id="recorded-title">The demo, as it ran.</h2>
          {/* While it runs the clock counts whole tenths; finished, it shows the total as the footer rounds it. */}
          <p className={cn(s.clock, done && s.done)} aria-hidden="true">{done ? SECONDS : (Math.floor(at / 100) / 10).toFixed(1)}&nbsp;s</p>
        </div>
        <div ref={term} className={cn(s.term, motion && s.armed)} id="term" data-testid="recorded-output">
          <div className={cn(s.ln, s.run, on(0))}><span className={s.pr}>$</span> {capture.command}</div>
          {LINES.slice(0, FIX).map(line)}
          <details>
            <summary className={cn(s.ln, on(FIX + 1))}>{withBranches(LINES[FIX])}</summary>
            {LINES.slice(FIX + 1, FIX_END).map((text, i) => line(text, FIX + 1 + i))}
          </details>
          {LINES.slice(FIX_END).map((text, i) => line(text, FIX_END + i))}
        </div>
        <div className={s.recFoot}>
          <a href={RECORD_URL}>
            Recorded output · pytxo&nbsp;{capture.version} · {SECONDS}&nbsp;s · {capture.host.os}
          </a>
          <button type="button" className={s.replay} aria-controls="term" onClick={() => replay.current()}>Replay</button>
        </div>
      </div>
    </section>
  );
}
