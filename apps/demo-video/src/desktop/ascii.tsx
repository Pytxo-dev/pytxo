// The film's ASCII system, ported from the site's components/site/ascii-art.tsx:
// the aperture ramp, glitch set, Chroma glyph colours and wordmark spectrum,
// drawn on a canvas per frame. Decorative only; it never reports product state.
import {useLayoutEffect, useRef, useState} from "react";
import {continueRender, delayRender, staticFile, useCurrentFrame} from "remotion";
import {FACES} from "../film/kit";

export const RAMP = " .:-=+*x#%@";
export const GLITCH = "01<>/\\|+*x#%@";
export const GLYPH = ["#7ee1ed", "#b4b9ff", "#bd9eef", "#e9bb91"];
export const SPECTRUM = ["#f04da3", "#ff6a4a", "#f3c64e", "#79e07b", "#45dccb"];
export const INK = "#f4f4f5";
export const MONO = '"IBM Plex Mono", ui-monospace, monospace';
export const SANS = "Sora, system-ui, sans-serif";
const AGENTS = ["codex", "claude", "cursor", "opencode", "antigravity"];

/** Deterministic noise so every frame renders the same way twice. */
export const hash = (a: number, b: number) => { const x = Math.sin(a * 127.1 + b * 311.7) * 43758.5453; return x - Math.floor(x); };

let fonts: Promise<unknown> | null = null;
const loadFonts = () => fonts ??= Promise.all(FACES.map(([family, file, weight]) =>
  new FontFace(family, `url(${staticFile(file)}) format("woff2")`, {weight: String(weight)}).load().then((face) => (document.fonts as unknown as Set<FontFace>).add(face))));

type Draw = (context: CanvasRenderingContext2D, frame: number) => void;

/** A full-frame canvas redrawn every frame, held until the fonts it draws with are loaded. */
export const AsciiCanvas = ({draw, width = 1920, height = 1080}: {draw: Draw; width?: number; height?: number}) => {
  const ref = useRef<HTMLCanvasElement>(null);
  const frame = useCurrentFrame();
  const [ready, setReady] = useState(false);
  useLayoutEffect(() => {
    if (ready) return;
    const handle = delayRender("ASCII fonts");
    loadFonts().then(() => { setReady(true); continueRender(handle); });
  }, [ready]);
  useLayoutEffect(() => {
    const context = ref.current?.getContext("2d");
    if (!ready || !context) return;
    context.clearRect(0, 0, width, height);
    draw(context, frame);
  }, [ready, frame, draw, width, height]);
  return <canvas ref={ref} width={width} height={height} style={{position: "absolute", inset: 0, width, height, pointerEvents: "none"}} />;
};

/**
 * The aperture mark as a shaded ASCII sphere, the agent CLIs on one tilted
 * orbit around it (the site's hero art). `sweep` runs a glitch band across it.
 */
export const drawAperture = (context: CanvasRenderingContext2D, o: {time: number; cx: number; cy: number; radius: number; alpha?: number; orbits?: boolean; sweep?: number}) => {
  const {time, cx, cy, radius} = o;
  const alpha = o.alpha ?? 1;
  const cw = 9, ch = 15;
  context.font = `500 12px ${MONO}`;
  context.textAlign = "center";
  context.textBaseline = "middle";
  const spin = time * .32;
  const light = {x: -.45, y: -.55, z: .7};
  for (let row = Math.floor((cy - radius) / ch) - 1; row < Math.ceil((cy + radius) / ch) + 1; row += 1) {
    for (let col = Math.floor((cx - radius) / cw) - 1; col < Math.ceil((cx + radius) / cw) + 1; col += 1) {
      const px = col * cw + cw / 2, py = row * ch + ch / 2;
      const x = (px - cx) / radius, y = (py - cy) / radius;
      const rr = x * x + y * y;
      if (rr > 1) continue;
      if (Math.abs(y - .46 * x) < .085) continue;
      const z = Math.sqrt(1 - rr);
      const lon = Math.atan2(x, z) + spin, lat = Math.asin(y);
      const shade = Math.max(0, x * light.x + y * light.y + z * light.z);
      const bands = .55 + .45 * Math.sin(lon * 9) * Math.cos(lat * 7);
      const hot = o.sweep === undefined ? 0 : Math.max(0, 1 - Math.abs(px - o.sweep) / 90);
      const level = Math.min(RAMP.length - 1, Math.floor((.05 + shade * .42 + bands * .2 + hot * .55) * (RAMP.length - 1)));
      if (level < 1) continue;
      const tone = y < -.35 ? 0 : y < .25 ? 1 : x > .45 ? 3 : 2;
      context.globalAlpha = Math.min(1, (.16 + .42 * shade + hot * .5) * alpha);
      context.fillStyle = GLYPH[tone];
      context.fillText(hot > .35 ? GLITCH[(col * 7 + row * 13 + Math.floor(time * 18)) % GLITCH.length] : RAMP[level], px, py);
    }
  }
  if (o.orbits !== false) {
    const rx = radius * 1.75, ry = radius * .42, tilt = -.16;
    const at = (angle: number) => {
      const ox = Math.cos(angle) * rx, oy = Math.sin(angle) * ry;
      return [cx + ox * Math.cos(tilt) - oy * Math.sin(tilt), cy + ox * Math.sin(tilt) + oy * Math.cos(tilt)] as const;
    };
    context.globalAlpha = .22 * alpha;
    context.fillStyle = "#7d7d87";
    for (let a = 0; a < Math.PI * 2; a += (cw * 1.6) / rx) {
      const [x, y] = at(a);
      if (!(Math.sin(a) < 0 && Math.hypot(x - cx, y - cy) < radius)) context.fillText("·", x, y);
    }
    AGENTS.forEach((name, index) => {
      const angle = time * .22 + (index / AGENTS.length) * Math.PI * 2;
      const front = Math.sin(angle) > 0;
      for (let k = 7; k >= 0; k -= 1) {
        const [x, y] = at(angle - k * .035);
        if (!front && Math.hypot(x - cx, y - cy) < radius * .98) continue;
        context.globalAlpha = (k === 0 ? .95 : .5 - k * .055) * alpha;
        context.fillStyle = k === 0 ? INK : "#45dccb";
        context.fillText(k === 0 ? "◆" : k < 3 ? "•" : "·", x, y);
      }
      const [x, y] = at(angle);
      if (!front && Math.hypot(x - cx, y - cy) < radius * .98) return;
      context.globalAlpha = .75 * alpha;
      context.fillStyle = "#c7c7ce";
      context.textAlign = "left";
      context.fillText(name, x + cw * 1.4, y);
      context.textAlign = "center";
    });
  }
  context.globalAlpha = 1;
};

type Mask = {cols: number; rows: number; cw: number; ch: number; ink: Float32Array; left: number; top: number};
const masks = new Map<string, Mask>();

/** Coverage of `text` per character cell, measured once per text and size. */
const maskOf = (text: string, size: number, weight: number, cw: number, ch: number, cx: number, cy: number): Mask => {
  const key = `${text}|${size}|${weight}|${cw}|${ch}|${cx}|${cy}`;
  const cached = masks.get(key);
  if (cached) return cached;
  const probe = document.createElement("canvas").getContext("2d")!;
  probe.font = `${weight} ${size}px ${SANS}`;
  const width = probe.measureText(text).width;
  const cols = Math.ceil(width / cw) + 4, rows = Math.ceil(size * 1.35 / ch) + 2;
  const sample = document.createElement("canvas");
  sample.width = cols * cw; sample.height = rows * ch;
  const ink = sample.getContext("2d")!;
  ink.font = `${weight} ${size}px ${SANS}`;
  ink.fillStyle = "#fff";
  ink.textAlign = "center";
  ink.textBaseline = "middle";
  ink.fillText(text, sample.width / 2, sample.height / 2 + size * .04);
  const pixels = ink.getImageData(0, 0, sample.width, sample.height).data;
  const cover = new Float32Array(cols * rows);
  for (let r = 0; r < rows; r += 1) for (let c = 0; c < cols; c += 1) {
    let sum = 0;
    for (let y = 0; y < ch; y += 2) for (let x = 0; x < cw; x += 2) sum += pixels[((r * ch + y) * sample.width + c * cw + x) * 4 + 3];
    cover[r * cols + c] = sum / (255 * Math.ceil(ch / 2) * Math.ceil(cw / 2));
  }
  const mask = {cols, rows, cw, ch, ink: cover, left: cx - sample.width / 2, top: cy - sample.height / 2};
  masks.set(key, mask);
  return mask;
};

export type TitleSpec = {text: string; at: number; out: number; cx: number; cy: number; size: number; weight?: number; cell?: [number, number]; spectrum?: boolean; ascii?: boolean};

const SPREAD = 16;
/** Opacity of the crisp type that a decoded title resolves into (0 for ASCII-only titles). */
export const titleCrisp = (frame: number, t: TitleSpec) => t.ascii ? 0 : Math.max(0, Math.min(1, (frame - (t.at + SPREAD + 6)) / 12, (t.out - 12 - frame) / 8));

/**
 * A line that decodes out of glitch glyphs into the ramp, column by column,
 * then resolves into crisp Sora (drawn by the caller with titleCrisp); on exit
 * it breaks back into glyphs. ASCII-only titles keep the ramp and shimmer.
 */
export const drawTitle = (context: CanvasRenderingContext2D, frame: number, t: TitleSpec) => {
  const [cw, ch] = t.cell ?? [8, 13];
  const m = maskOf(t.text, t.size, t.weight ?? 600, cw, ch, t.cx, t.cy);
  const spread = SPREAD, decode = 9;
  context.font = `500 ${Math.round(ch * .82)}px ${MONO}`;
  context.textAlign = "center";
  context.textBaseline = "middle";
  for (let r = 0; r < m.rows; r += 1) for (let c = 0; c < m.cols; c += 1) {
    const ink = m.ink[r * m.cols + c];
    if (ink < .06) continue;
    const jitter = hash(c, r) * 6;
    const enter = frame - (t.at + (c / m.cols) * spread + jitter);
    const leave = frame - (t.out - 14 + (1 - c / m.cols) * 10 + jitter * .5);
    if (enter < -6 || leave > decode) continue;
    // Settled cells fade as the crisp type arrives; leaving cells flare back up.
    const settled = t.ascii ? 0 : Math.max(0, Math.min(1, (frame - (t.at + spread + 8)) / 14));
    const wave = t.ascii ? .5 + .5 * Math.sin(c * .18 - r * .35 - frame / 60 * 2.2) : 0;
    const exiting = leave > -decode;
    let alpha: number, glyph: string;
    if (enter < 0) { alpha = .25 * (1 + enter / 6); glyph = GLITCH[(c * 5 + r * 11 + frame) % GLITCH.length]; }
    else if (enter < decode || exiting) { alpha = .5 + .4 * ink; glyph = GLITCH[(c * 7 + r * 3 + frame * 2) % GLITCH.length]; }
    else { alpha = (.45 + .45 * ink + wave * .15) * (1 - settled * .92); glyph = RAMP[Math.min(RAMP.length - 1, Math.max(1, Math.floor((ink * .82 + wave * .18) * (RAMP.length - 1))))]; }
    if (exiting && leave > 0) alpha *= 1 - leave / decode;
    if (alpha <= .01) continue;
    context.globalAlpha = alpha;
    context.fillStyle = t.spectrum ? SPECTRUM[Math.round((c / m.cols) * (SPECTRUM.length - 1))] : GLYPH[(c + r) % 3 === 0 ? 0 : 1];
    context.fillText(glyph, m.left + c * cw + cw / 2, m.top + r * ch + ch / 2);
  }
  context.globalAlpha = 1;
};

/** A vertical band of ramp glyphs at screen x, the leading edge of a wipe. */
export const drawWipe = (context: CanvasRenderingContext2D, frame: number, x: number, height = 1080) => {
  const cw = 12, ch = 20, band = 22;
  context.font = `600 16px ${MONO}`;
  context.textAlign = "center";
  context.textBaseline = "middle";
  context.fillStyle = "rgba(6,6,9,.9)";
  context.fillRect(x - band * cw * .35, 0, band * cw * .7, height);
  for (let i = 0; i < band; i += 1) for (let r = 0; r < Math.ceil(height / ch); r += 1) {
    const d = Math.abs(i - band / 2) / (band / 2);
    const n = hash(i * 13 + frame, r);
    if (n < d * .9) continue;
    const level = Math.min(RAMP.length - 1, Math.floor((1 - d) * (RAMP.length - 1) * (.5 + n * .6)));
    if (level < 1) continue;
    context.globalAlpha = (1 - d) * (.4 + .6 * n);
    context.fillStyle = i % 4 === 0 ? GLYPH[2] : i % 3 === 0 ? GLYPH[3] : GLYPH[0];
    context.fillText(n > .92 ? GLITCH[(i + r + frame) % GLITCH.length] : RAMP[level], x + (i - band / 2) * cw, r * ch + ch / 2);
  }
  context.globalAlpha = 1;
};
