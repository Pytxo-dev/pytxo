// Shared language for PytxoFilm: a quiet paper canvas, Sora and IBM Plex Mono
// (Pytxo Desktop's own faces), soft-shadowed windows, critically damped motion
// and one black cursor. Every number on screen comes from the recorded ledger.
import {useEffect, useState, type CSSProperties, type ReactNode} from "react";
import {continueRender, delayRender, Easing, Img, interpolate, spring, staticFile, useCurrentFrame, useVideoConfig} from "remotion";
import fleet from "../../fleet-props.json";
import diff from "../../film-diff.json";

export const FPS = 60;
export const T = {
  paper: "#F7F6F3",
  card: "#FFFFFF",
  ink: "#191918",
  soft: "#37352F",
  muted: "#787774",
  faint: "#AEACA6",
  rule: "#E6E4DF",
  hush: "#F1F0EC",
  green: "#0F7B6C",
  greenWash: "#E5F2EF",
  coral: "#C9472B",
  coralWash: "#FBECE7",
  amber: "#B7791F",
  code: "#FBFBFA",
};
export const sans = "'Sora', system-ui, sans-serif";
export const mono = "'IBM Plex Mono', ui-monospace, monospace";
export const shadow = "0 1px 2px rgba(25,25,24,.05), 0 10px 30px rgba(25,25,24,.07), 0 34px 70px rgba(25,25,24,.06)";

export const ledger = fleet;
export const changes = diff;

const FACES: [string, string, number][] = [
  ["Sora", "fonts/sora-latin-400-normal.woff2", 400],
  ["Sora", "fonts/sora-latin-500-normal.woff2", 500],
  ["Sora", "fonts/sora-latin-600-normal.woff2", 600],
  ["IBM Plex Mono", "fonts/ibm-plex-mono-latin-400-normal.woff2", 400],
  ["IBM Plex Mono", "fonts/ibm-plex-mono-latin-500-normal.woff2", 500],
];
/** Holds the render until both product faces are loaded. */
export const useFilmFonts = () => {
  const [handle] = useState(() => delayRender("Loading Sora and IBM Plex Mono"));
  useEffect(() => {
    Promise.all(FACES.map(([family, file, weight]) => new FontFace(family, `url(${staticFile(file)}) format("woff2")`, {weight: String(weight)}).load().then((face) => (document.fonts as unknown as Set<FontFace>).add(face))))
      .then(() => continueRender(handle))
      .catch((error) => { throw error; });
  }, [handle]);
};

/** Critically damped settle (no overshoot), 0 → 1, starting at frame `at`. */
export const useSettle = (at: number, duration = 42) => {
  const frame = useCurrentFrame();
  const {fps} = useVideoConfig();
  return spring({frame: frame - at, fps, config: {damping: 200, mass: 0.8}, durationInFrames: duration});
};
/** A small, friendly overshoot for things that appear. */
export const usePop = (at: number) => {
  const frame = useCurrentFrame();
  const {fps} = useVideoConfig();
  return spring({frame: frame - at, fps, config: {damping: 14, stiffness: 170, mass: 0.7}});
};
const glide = Easing.bezier(0.65, 0, 0.35, 1);
export const ramp = (frame: number, from: number, to: number) => interpolate(frame, [from, to], [0, 1], {extrapolateLeft: "clamp", extrapolateRight: "clamp", easing: glide});
export const mix = (a: number, b: number, t: number) => a + (b - a) * t;
export const fade = (frame: number, start: number, end: number, length = 18) => Math.min(ramp(frame, start, start + length), 1 - ramp(frame, end - length, end));

/** Headline words rise out of a mask one after another. */
export const Rise = ({text, at, size, weight = 600, color = T.ink, stagger = 4, style}: {text: string; at: number; size: number; weight?: number; color?: string; stagger?: number; style?: CSSProperties}) => {
  const frame = useCurrentFrame();
  const {fps} = useVideoConfig();
  return <div style={{fontFamily: sans, fontSize: size, fontWeight: weight, letterSpacing: "-0.035em", lineHeight: 1.08, color, display: "flex", flexWrap: "wrap", columnGap: size * 0.26, ...style}}>
    {text.split(" ").map((word, index) => {
      const t = spring({frame: frame - at - index * stagger, fps, config: {damping: 200, mass: 0.7}});
      return <span key={index} style={{display: "inline-block", overflow: "hidden", paddingBottom: size * 0.12, marginBottom: -size * 0.12}}>
        <span style={{display: "inline-block", transform: `translateY(${(1 - t) * 105}%)`, opacity: Math.min(1, t * 1.6)}}>{word}</span>
      </span>;
    })}
  </div>;
};

/** Short captions in the corner of a scene: a muted step label and one line. */
export const Caption = ({step, text, at, out}: {step: string; text: string; at: number; out: number}) => {
  const frame = useCurrentFrame();
  const visible = fade(frame, at, out, 16);
  return <div style={{position: "absolute", left: 120, top: 92, opacity: visible}}>
    <div style={{fontFamily: mono, fontSize: 20, color: T.muted, letterSpacing: "0.02em", marginBottom: 14}}>{step}</div>
    <Rise text={text} at={at} size={50} stagger={3} />
  </div>;
};

const LOGOS: Record<string, {file: string; tile: string; pad: number}> = {
  codex: {file: "fleet/logos/openai-on-dark.svg", tile: "#111111", pad: 0.22},
  claude: {file: "fleet/logos/anthropic.svg", tile: "#F0EEE6", pad: 0.26},
  cursor: {file: "fleet/logos/cursor-on-dark.svg", tile: "#111111", pad: 0.24},
  opencode: {file: "fleet/logos/opencode.svg", tile: "#211E1E", pad: 0.16},
  agy: {file: "fleet/logos/antigravity.png", tile: "#FFFFFF", pad: 0.12},
};
export const vendorOf = (cli: string) => ({codex: "OpenAI Codex", claude: "Claude Code", cursor: "Cursor Agent", opencode: "OpenCode", agy: "Antigravity"} as Record<string, string>)[cli] ?? cli;
export const Logo = ({cli, size, style}: {cli: string; size: number; style?: CSSProperties}) => {
  const logo = LOGOS[cli];
  return <div style={{width: size, height: size, borderRadius: size * 0.26, background: logo.tile, boxShadow: `inset 0 0 0 1px rgba(25,25,24,.08)`, display: "flex", alignItems: "center", justifyContent: "center", flex: "none", ...style}}>
    <Img src={staticFile(logo.file)} style={{width: size * (1 - logo.pad * 2), height: size * (1 - logo.pad * 2), objectFit: "contain"}} />
  </div>;
};

export const Window = ({children, style, title}: {children: ReactNode; style?: CSSProperties; title?: string}) => <div style={{position: "absolute", background: T.card, borderRadius: 18, boxShadow: shadow, border: `1px solid ${T.rule}`, overflow: "hidden", ...style}}>
  {title !== undefined && <div style={{height: 48, display: "flex", alignItems: "center", gap: 9, padding: "0 20px", borderBottom: `1px solid ${T.rule}`, fontFamily: sans, fontSize: 17, color: T.muted}}>
    {[0, 1, 2].map((dot) => <span key={dot} style={{width: 11, height: 11, borderRadius: 6, background: T.rule}} />)}
    <span style={{marginLeft: 10}}>{title}</span>
  </div>}
  {children}
</div>;

/** One black pointer moving between keyframes; a click dips it and leaves a ring. */
export type CursorKey = {at: number; x: number; y: number; click?: boolean};
export const Cursor = ({keys, visibleFrom = 0, visibleTo = Infinity}: {keys: CursorKey[]; visibleFrom?: number; visibleTo?: number}) => {
  const frame = useCurrentFrame();
  if (frame < visibleFrom || frame > visibleTo) return null;
  let x = keys[0].x, y = keys[0].y;
  for (let index = 1; index < keys.length; index++) {
    const from = keys[index - 1], to = keys[index];
    const travel = Math.min(36, Math.max(18, (to.at - from.at) * 0.7));
    if (frame >= to.at - travel) {
      const t = ramp(frame, to.at - travel, to.at);
      x = mix(from.x, to.x, t);
      y = mix(from.y, to.y, t);
    }
  }
  const click = keys.find((key) => key.click && frame >= key.at && frame < key.at + 24);
  const press = click ? 1 - Math.sin(Math.min(1, (frame - click.at) / 10) * Math.PI) * 0.12 : 1;
  const ring = click ? (frame - click.at) / 24 : 0;
  const opacity = Math.min(ramp(frame, visibleFrom, visibleFrom + 12), 1 - ramp(frame, visibleTo - 12, visibleTo));
  return <div style={{position: "absolute", left: 0, top: 0, transform: `translate(${x}px, ${y}px)`, opacity, zIndex: 50, pointerEvents: "none"}}>
    {click && <div style={{position: "absolute", left: -22, top: -22, width: 44, height: 44, borderRadius: 22, border: `2px solid ${T.ink}`, opacity: 0.35 * (1 - ring), transform: `scale(${0.5 + ring * 0.9})`}} />}
    <svg width="34" height="40" viewBox="0 0 17 20" style={{transform: `scale(${press})`, transformOrigin: "2px 2px", filter: "drop-shadow(0 2px 3px rgba(0,0,0,.22))"}}>
      <path d="M1.5 1.2 L1.5 16.2 L5.3 12.6 L7.9 18.6 L10.6 17.4 L8 11.6 L13.3 11.6 Z" fill={T.ink} stroke="#fff" strokeWidth="1.2" strokeLinejoin="round" />
    </svg>
  </div>;
};

export const Chip = ({children, tone = "plain", style}: {children: ReactNode; tone?: "plain" | "green" | "coral" | "amber" | "muted"; style?: CSSProperties}) => {
  const tones = {plain: [T.hush, T.soft], green: [T.greenWash, T.green], coral: [T.coralWash, T.coral], amber: ["#FBF3E4", T.amber], muted: [T.hush, T.muted]} as const;
  const [background, color] = tones[tone];
  return <span style={{display: "inline-flex", alignItems: "center", gap: 7, padding: "5px 11px", borderRadius: 8, background, color, fontFamily: sans, fontSize: 17, fontWeight: 500, whiteSpace: "nowrap", ...style}}>{children}</span>;
};

export const Check = ({size = 18, color = T.green}: {size?: number; color?: string}) => <svg width={size} height={size} viewBox="0 0 16 16"><path d="M3 8.5 L6.5 12 L13 4.5" fill="none" stroke={color} strokeWidth="2" strokeLinecap="round" strokeLinejoin="round" /></svg>;

export const short = (digest: string) => `${digest.slice(0, 8)}…${digest.slice(-4)}`;
