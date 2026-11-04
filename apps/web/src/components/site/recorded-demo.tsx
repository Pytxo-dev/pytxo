"use client";

import { Fragment, useEffect, useRef, useState, useSyncExternalStore } from "react";

import s from "@/components/site/site.module.css";
import { cn } from "@/lib/utils";
import capture from "../../../public/evidence/demo-blocked-2.0.0-2026-10-30.json";

const RECORD_URL = "/evidence/demo-blocked-2.0.0-2026-10-30.json";
const LINES = capture.output.trimEnd().split("\n");
const BRANCH = new RegExp(`(${Object.keys(capture.repo.branches).join("|")})`);
const LABEL = /^(BLOCKED|why|next)( +)(.*)$/;
const TOTAL = capture.duration_ms[0];
// When each row shows, in ms from the start: the command, the setup lines, the verdict's three lines, then the rest.
// The record times the whole run, not each line, so the rows are paced to land by the recorded total.
const AT = [0, 240, 900, 1150, 1700, TOTAL - 400, TOTAL - 320, TOTAL - 240, TOTAL, TOTAL];
const ROWS = LINES.length + 1;
const count = (ms: number) => Array.from({ length: ROWS }, (_, i) => AT[i] ?? TOTAL).filter((at) => ms >= at).length;

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
 * The recorded output of `npx pytxo demo`. Every line ships in the HTML, so it reads with JavaScript off and screen
 * readers get all of it; in view, the rows replay at the recorded pace while the clock runs to the recorded time.
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

  const at = motion ? ms : TOTAL, shown = count(at), tenths = Math.floor(at / 100), done = at >= TOTAL;
  return (
    <section className={cn(s.sec, "tabular-nums")} id="recorded" aria-labelledby="recorded-title" data-testid="recorded-demo">
      <div className={s.wrap}>
        <div className={s.recHead}>
          <h2 id="recorded-title">The demo, as it ran.</h2>
          <p className={cn(s.clock, done && s.done)} aria-hidden="true">{(tenths / 10).toFixed(1)}&nbsp;s</p>
        </div>
        <div ref={term} className={cn(s.term, motion && s.armed)} id="term" data-testid="recorded-output">
          <div className={cn(s.ln, s.run, shown > 0 && s.on)}><span className={s.pr}>$</span> {capture.command}</div>
          {LINES.map((text, i) => {
            const label = LABEL.exec(text);
            return (
              <div key={i} className={cn(s.ln, label && s.lab, !text && s.gap, shown > i + 1 && s.on)}>
                {label ? (
                  <Fragment>
                    <span className={label[1] === "BLOCKED" ? s.hit : s.k}>{label[1]}</span>
                    {label[2]}
                    {withBranches(label[3])}
                  </Fragment>
                ) : withBranches(text)}
              </div>
            );
          })}
        </div>
        <div className={s.recFoot}>
          <a href={RECORD_URL}>
            Recorded output · pytxo&nbsp;{capture.version} · {(TOTAL / 1000).toFixed(1)}&nbsp;s · {capture.host.os}
          </a>
          <button type="button" className={s.replay} aria-controls="term" onClick={() => replay.current()}>Replay</button>
        </div>
      </div>
    </section>
  );
}
