"use client";

import { useEffect, useRef, type RefObject } from "react";

/*
  Character-grid art for the marketing site. Both pieces draw only characters
  from the Pytxo aperture ramp onto a canvas, pause when off screen or hidden,
  and render one still frame when the visitor prefers reduced motion.
*/

const GLYPH = ["#7ee1ed", "#b4b9ff", "#bd9eef", "#e9bb91"];
const RAMP = " .:-=+*x#%@";
const GLITCH = "01<>/\\|+*x#%@";

type Draw = (context: CanvasRenderingContext2D, frame: { width: number; height: number; time: number; pointer: { x: number; y: number } | null }) => void;

function useCharCanvas(ref: RefObject<HTMLCanvasElement | null>, draw: Draw, fps = 30) {
  useEffect(() => {
    const canvas = ref.current;
    const context = canvas?.getContext("2d");
    if (!canvas || !context) return;
    const still = matchMedia("(prefers-reduced-motion: reduce)").matches;
    let width = 0, height = 0, frame = 0, previous = 0, visible = true, start = performance.now();
    let pointer: { x: number; y: number } | null = null;
    const resize = () => {
      const box = canvas.getBoundingClientRect();
      const ratio = Math.min(window.devicePixelRatio || 1, 2);
      width = box.width; height = box.height;
      canvas.width = Math.round(width * ratio); canvas.height = Math.round(height * ratio);
      context.setTransform(ratio, 0, 0, ratio, 0, 0);
      if (still) render(start);
    };
    const render = (time: number) => {
      context.clearRect(0, 0, width, height);
      draw(context, { width, height, time: (time - start) / 1000, pointer });
    };
    const loop = (time: number) => {
      frame = requestAnimationFrame(loop);
      if (!visible || document.hidden || time - previous < 1000 / fps) return;
      previous = time;
      render(time);
    };
    const move = (event: PointerEvent) => {
      const box = canvas.getBoundingClientRect();
      pointer = { x: event.clientX - box.left, y: event.clientY - box.top };
    };
    const leave = () => { pointer = null; };
    const resizer = new ResizeObserver(resize);
    resizer.observe(canvas);
    const observer = new IntersectionObserver(([entry]) => { visible = entry.isIntersecting; });
    observer.observe(canvas);
    resize();
    if (!still) {
      window.addEventListener("pointermove", move, { passive: true });
      document.addEventListener("pointerleave", leave);
      frame = requestAnimationFrame(loop);
    }
    void document.fonts?.ready.then(() => { start = performance.now() - 1; if (still) render(start); });
    return () => {
      cancelAnimationFrame(frame);
      resizer.disconnect();
      observer.disconnect();
      window.removeEventListener("pointermove", move);
      document.removeEventListener("pointerleave", leave);
    };
  }, [ref, draw, fps]);
}

/** A cell near the pointer brightens and flickers through glitch characters. */
function near(pointer: { x: number; y: number } | null, x: number, y: number, radius: number) {
  if (!pointer) return 0;
  const distance = Math.hypot(pointer.x - x, (pointer.y - y) * 1.3);
  return distance < radius ? 1 - distance / radius : 0;
}

const AGENTS = ["codex", "claude", "cursor", "opencode", "antigravity"];

/**
 * The aperture mark as a shaded ASCII sphere, with the five agent CLIs on
 * tilted orbits around it: many agents, one change.
 */
const drawAperture: Draw = (context, { width, height, time, pointer }) => {
  const cw = width < 640 ? 7 : 9, ch = width < 640 ? 12 : 15;
  context.font = `500 ${width < 640 ? 10 : 12}px "IBM Plex Mono", ui-monospace, monospace`;
  context.textAlign = "center";
  context.textBaseline = "middle";
  const narrow = width < 640;
  const cx = width / 2, cy = height * (narrow ? .3 : .47);
  const radius = Math.min(width * (narrow ? .44 : .3), height * .42, 330);
  const spin = time * .32;
  const light = { x: -.45, y: -.55, z: .7 };
  const cols = Math.ceil(width / cw), rows = Math.ceil(height / ch);
  const c0 = Math.max(0, Math.floor((cx - radius) / cw) - 1), c1 = Math.min(cols, Math.ceil((cx + radius) / cw) + 1);
  const r0 = Math.max(0, Math.floor((cy - radius) / ch) - 1), r1 = Math.min(rows, Math.ceil((cy + radius) / ch) + 1);
  for (let row = r0; row < r1; row += 1) {
    for (let col = c0; col < c1; col += 1) {
      const px = col * cw + cw / 2, py = row * ch + ch / 2;
      const x = (px - cx) / radius, y = (py - cy) / radius;
      const rr = x * x + y * y;
      if (rr > 1) continue;
      // The aperture's diagonal cut, fixed in screen space like the logo.
      if (Math.abs(y - .46 * x) < .085) continue;
      const z = Math.sqrt(1 - rr);
      const lon = Math.atan2(x, z) + spin, lat = Math.asin(y);
      const shade = Math.max(0, x * light.x + y * light.y + z * light.z);
      const bands = .55 + .45 * Math.sin(lon * 9) * Math.cos(lat * 7);
      const hot = near(pointer, px, py, 120);
      // Mostly light characters: the sphere sits behind the headline, not over it.
      const level = Math.min(RAMP.length - 1, Math.floor((.05 + shade * .42 + bands * .2 + hot * .55) * (RAMP.length - 1)));
      if (level < 1) continue;
      const tone = y < -.35 ? 0 : y < .25 ? 1 : x > .45 ? 3 : 2;
      context.globalAlpha = .16 + .42 * shade + hot * .5;
      context.fillStyle = GLYPH[tone];
      context.fillText(hot > .35 ? GLITCH[(col * 7 + row * 13 + Math.floor(time * 18)) % GLITCH.length] : RAMP[level], px, py);
    }
  }
  // Orbits: one tilted ellipse, five agents spaced around it, each with a fading trail.
  const rx = Math.min(width * .46, radius * 1.75), ry = radius * .42;
  const tilt = -.16;
  const at = (angle: number) => {
    const ox = Math.cos(angle) * rx, oy = Math.sin(angle) * ry;
    return [cx + ox * Math.cos(tilt) - oy * Math.sin(tilt), cy + ox * Math.sin(tilt) + oy * Math.cos(tilt)] as const;
  };
  context.globalAlpha = .22;
  context.fillStyle = "#7d7d87";
  for (let a = 0; a < Math.PI * 2; a += (cw * 1.6) / rx) {
    const [x, y] = at(a);
    const behind = Math.sin(a) < 0 && Math.hypot(x - cx, (y - cy)) < radius;
    if (!behind) context.fillText("·", x, y);
  }
  AGENTS.forEach((name, index) => {
    const angle = time * .22 + (index / AGENTS.length) * Math.PI * 2;
    const front = Math.sin(angle) > 0;
    for (let k = 7; k >= 0; k -= 1) {
      const [x, y] = at(angle - k * .035);
      if (!front && Math.hypot(x - cx, y - cy) < radius * .98) continue;
      context.globalAlpha = k === 0 ? .95 : .5 - k * .055;
      context.fillStyle = k === 0 ? "#f5f5f7" : "#45dccb";
      context.fillText(k === 0 ? "◆" : k < 3 ? "•" : "·", x, y);
    }
    const [x, y] = at(angle);
    if (!front && Math.hypot(x - cx, y - cy) < radius * .98) return;
    // Names fade while they cross the headline column.
    context.globalAlpha = .2 + .55 * Math.min(1, Math.abs(x - cx) / (rx * .75));
    context.fillStyle = "#c7c7ce";
    context.textAlign = "left";
    context.fillText(name, x + cw * 1.4, y);
    context.textAlign = "center";
  });
  context.globalAlpha = 1;
};

export function AsciiAperture({ className }: { className?: string }) {
  const ref = useRef<HTMLCanvasElement>(null);
  useCharCanvas(ref, drawAperture, 30);
  return <canvas ref={ref} aria-hidden="true" className={className} />;
}

/** The wordmark rasterised into the aperture ramp, with a slow shimmer that runs across it. */
function makeWordmark(text: string) {
  let cache: { width: number; mask: Float32Array; cols: number; rows: number } | null = null;
  const draw: Draw = (context, { width, height, time, pointer }) => {
    const cw = width < 640 ? 6 : 9, ch = width < 640 ? 10 : 14;
    const cols = Math.floor(width / cw), rows = Math.floor(height / ch);
    if (!cache || cache.width !== width) {
      const sample = document.createElement("canvas");
      sample.width = cols; sample.height = rows;
      const ink = sample.getContext("2d")!;
      ink.fillStyle = "#fff";
      ink.textBaseline = "middle";
      ink.textAlign = "center";
      // One sample pixel per cell. Cells are taller than wide, so the word is
      // stretched sideways by the cell ratio to come out at its true shape.
      const stretch = ch / cw;
      let size = rows * 1.25;
      ink.font = `700 ${size}px Sora, sans-serif`;
      const measured = ink.measureText(text).width * stretch;
      if (measured > cols * .96) size *= (cols * .96) / measured;
      ink.font = `700 ${size}px Sora, sans-serif`;
      ink.setTransform(stretch, 0, 0, 1, cols / 2, rows / 2 + 1);
      ink.fillText(text, 0, 0);
      const pixels = ink.getImageData(0, 0, cols, rows).data;
      const mask = new Float32Array(cols * rows);
      for (let index = 0; index < mask.length; index += 1) mask[index] = pixels[index * 4 + 3] / 255;
      cache = { width, mask, cols, rows };
    }
    context.font = `500 ${width < 640 ? 9 : 12}px "IBM Plex Mono", ui-monospace, monospace`;
    context.textAlign = "center";
    context.textBaseline = "middle";
    const spectrum = ["#f04da3", "#ff6a4a", "#f3c64e", "#79e07b", "#45dccb"];
    for (let row = 0; row < cache.rows; row += 1) {
      for (let col = 0; col < cache.cols; col += 1) {
        const ink = cache.mask[row * cache.cols + col];
        const px = col * cw + cw / 2, py = row * ch + ch / 2;
        const hot = near(pointer, px, py, 110);
        const wave = .5 + .5 * Math.sin(col * .18 - row * .35 - time * 2.2);
        if (ink < .08) {
          if (hot > .2) { context.globalAlpha = hot * .35; context.fillStyle = "#45dccb"; context.fillText("·", px, py); }
          continue;
        }
        const level = Math.min(RAMP.length - 1, Math.max(1, Math.floor((ink * .82 + wave * .18 + hot * .4) * (RAMP.length - 1))));
        const hue = (col / cache.cols) * (spectrum.length - 1);
        context.globalAlpha = .35 + ink * .35 + wave * .15 + hot * .3;
        context.fillStyle = spectrum[Math.round(hue)];
        context.fillText(hot > .4 ? GLITCH[(col * 5 + row * 11 + Math.floor(time * 16)) % GLITCH.length] : RAMP[level], px, py);
      }
    }
    context.globalAlpha = 1;
  };
  return draw;
}

const drawWordmark = makeWordmark("PYTXO");

export function AsciiWordmark({ className }: { className?: string }) {
  const ref = useRef<HTMLCanvasElement>(null);
  useCharCanvas(ref, drawWordmark, 24);
  return <canvas ref={ref} aria-hidden="true" className={className} />;
}
