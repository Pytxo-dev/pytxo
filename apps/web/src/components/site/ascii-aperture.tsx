"use client";

import { useEffect, useRef, useState } from "react";
import { PauseIcon, PlayIcon } from "lucide-react";
import { APERTURE_COLORS, apertureFrame } from "@/lib/ascii-aperture";

const STILL = apertureFrame(0);

export function AsciiAperture() {
  const canvas = useRef<HTMLCanvasElement>(null);
  const [paused, setPaused] = useState(false);
  const [reduced, setReduced] = useState(false);
  const [moving, setMoving] = useState(false);

  useEffect(() => {
    const surface = canvas.current;
    const context = surface?.getContext("2d");
    if (!surface || !context) return;
    context.setTransform(2, 0, 0, 2, 0, 0);
    const media = matchMedia("(prefers-reduced-motion: reduce)");
    let visible = false, ready = false, alive = true;
    let timer: ReturnType<typeof setTimeout> | undefined;
    let phase = 0;
    const draw = () => {
      context.clearRect(0, 0, 320, 320);
      context.font = '500 12px "IBM Plex Mono", monospace';
      context.textAlign = "center";
      context.textBaseline = "middle";
      for (const point of apertureFrame(phase)) {
        context.fillStyle = APERTURE_COLORS[point.color];
        context.globalAlpha = 0.3 + 0.7 * Math.max(0, point.z);
        context.fillText(point.char, 160 + point.x * 138, 160 + point.y * 138);
      }
      phase += 0.014;
      timer = setTimeout(draw, 1000 / 12);
    };
    const sync = () => {
      clearTimeout(timer);
      setReduced(media.matches);
      const run = ready && visible && !document.hidden && !media.matches && !paused;
      setMoving(run);
      if (run) draw();
    };
    const observer = new IntersectionObserver(([entry]) => { visible = entry.isIntersecting; sync(); });
    observer.observe(surface);
    media.addEventListener("change", sync);
    document.addEventListener("visibilitychange", sync);
    document.fonts.ready.then(() => { if (alive) { ready = true; sync(); } });
    return () => {
      alive = false;
      clearTimeout(timer);
      observer.disconnect();
      media.removeEventListener("change", sync);
      document.removeEventListener("visibilitychange", sync);
    };
  }, [paused]);

  return <div className="ascii-aperture" data-testid="ascii-aperture" data-animated={moving}>
    <div className="ascii-aperture-art" aria-hidden="true">
      <svg viewBox="0 0 320 320" style={{ visibility: moving ? "hidden" : "visible" }}>
        {STILL.map((p, i) => <text key={i} x={(160 + p.x * 138).toFixed(3)} y={(160 + p.y * 138).toFixed(3)} fill={APERTURE_COLORS[p.color]} opacity={(0.3 + 0.7 * Math.max(0, p.z)).toFixed(3)}>{p.char}</text>)}
      </svg>
      <canvas ref={canvas} width={640} height={640} style={{ visibility: moving ? "visible" : "hidden" }} />
    </div>
    <button type="button" onClick={() => setPaused(value => !value)} disabled={reduced} aria-pressed={paused || reduced} className="ascii-motion-control">
      {paused || reduced ? <PlayIcon size={12} aria-hidden /> : <PauseIcon size={12} aria-hidden />}
      <span>{reduced ? "Motion reduced" : paused ? "Play animation" : "Pause animation"}</span>
    </button>
  </div>;
}
