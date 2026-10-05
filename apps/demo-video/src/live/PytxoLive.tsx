// PytxoLive: the real Pytxo Desktop, recorded natively during one journey take,
// cut down and framed in a quiet dark stage. The pointer is redrawn from the
// take's own telemetry; nothing on the app window is re-created or animated.
import {AbsoluteFill, Easing, interpolate, OffthreadVideo, Sequence, spring, staticFile, useCurrentFrame, useVideoConfig} from "remotion";
import {useFilmFonts} from "../film/kit";
import cut from "../../live-cut.json";

export const LIVE_FPS = 60;
const INTRO = 150;
const OUTRO = 210;
const RESULT = 300;

type Rect = {x: number; y: number; w: number; h: number};
type Shot = {start: number; end: number; speed: number; kicker: string; title: string; focus: Rect | null; sped?: boolean};
const shots = cut.shots as Shot[];
const frames = (s: Shot) => Math.round(((s.end - s.start) / s.speed) * LIVE_FPS);
const starts = shots.reduce<number[]>((list, _s, i) => [...list, i ? list[i - 1] + frames(shots[i - 1]) : INTRO], []);
const SHOTS_END = starts.at(-1)! + frames(shots.at(-1)!);
export const LIVE_FRAMES = SHOTS_END + (cut.result ? RESULT : 0) + OUTRO;

const ink = "#F4F4F5";
const muted = "#8B8B94";
const bg = "#09090B";
const sans = "'Sora', system-ui, sans-serif";
const mono = "'IBM Plex Mono', ui-monospace, monospace";

// The stage: a 1600-wide window under a headline band.
const STAGE = {x: 160, y: 168, w: 1600};
const scale0 = STAGE.w / cut.width;
const stageH = cut.height * scale0;

const Dots = () => (
  <AbsoluteFill style={{backgroundImage: "radial-gradient(circle, rgba(255,255,255,.07) 1.2px, transparent 1.6px)", backgroundSize: "28px 28px", maskImage: "radial-gradient(ellipse 75% 70% at 50% 45%, #000 35%, transparent 85%)"}} />
);

const Glyph = ({size, phase}: {size: number; phase: number}) => {
  const shell = Array.from({length: 340}, (_, n) => {
    const y = 1 - 2 * (n + .5) / 340, r = Math.sqrt(1 - y * y), a = n * Math.PI * (3 - Math.sqrt(5));
    return {x: Math.cos(a) * r, y, z: Math.sin(a) * r, c: ".:+*=x"[n % 6]};
  });
  const colors = ["#45dccb", "#8b93ff", "#b98cff", "#f0a35e"];
  return (
    <svg width={size} height={size} viewBox="0 0 192 192">
      {shell.map((p, i) => {
        const x = p.x * Math.cos(phase) + p.z * Math.sin(phase), z = p.z * Math.cos(phase) - p.x * Math.sin(phase);
        if (z < -.15 || Math.abs(p.y - .46 * x) < .18) return null;
        return <text key={i} x={96 + x * 80} y={96 + p.y * 80} fill={colors[p.y < -.35 ? 0 : p.y < .25 ? 1 : x > .45 ? 3 : 2]} opacity={.3 + .7 * Math.max(0, z)} style={{font: `500 14px ${mono}`}} textAnchor="middle" dominantBaseline="central">{p.c}</text>;
      })}
    </svg>
  );
};

const Title = ({kicker, title, at, out}: {kicker: string; title: string; at: number; out?: number}) => {
  const frame = useCurrentFrame();
  const inP = interpolate(frame, [at, at + 24], [0, 1], {extrapolateLeft: "clamp", extrapolateRight: "clamp", easing: Easing.bezier(.2, .8, .2, 1)});
  const outP = out === undefined ? 1 : interpolate(frame, [out - 14, out], [1, 0], {extrapolateLeft: "clamp", extrapolateRight: "clamp"});
  return (
    <div style={{position: "absolute", left: 0, right: 0, top: 46, textAlign: "center", opacity: inP * outP, transform: `translateY(${(1 - inP) * 10}px)`}}>
      <div style={{font: `500 17px ${mono}`, letterSpacing: ".14em", textTransform: "uppercase", color: muted}}>{kicker}</div>
      <div style={{marginTop: 12, font: `600 46px ${sans}`, letterSpacing: "-.025em", color: ink}}>{title}</div>
    </div>
  );
};

/** Camera for a shot: identity, or a push-in that centres the focus rect in the stage. */
const cameraFor = (s: Shot) => {
  if (!s.focus) return {s: 1, x: 0, y: 0};
  const zoom = Math.min(1.7, Math.max(1.15, Math.min(cut.width / s.focus.w, cut.height / s.focus.h) * .8));
  const cx = (s.focus.x + s.focus.w / 2) * scale0, cy = (s.focus.y + s.focus.h / 2) * scale0;
  const x = Math.min(0, Math.max(STAGE.w - STAGE.w * zoom, STAGE.w / 2 - cx * zoom));
  const y = Math.min(0, Math.max(stageH - stageH * zoom, stageH / 2 - cy * zoom));
  return {s: zoom, x, y};
};

/** Video-time pointer position, gliding from the previous hover the way the driver moved it. */
const pointerAt = (t: number) => {
  const moves = cut.pointer.filter((p) => p.x !== undefined && p.x !== null) as {t: number; x: number; y: number; kind: string}[];
  let prev = moves[0], next = moves[0];
  for (const p of moves) { if (p.t <= t) { prev = next; next = p; } else break; }
  if (!next) return null;
  const glideSec = next.kind === "drag" ? 0.6 : 0.5;
  const k = Math.min(1, Math.max(0, (t - next.t) / glideSec));
  const e = k * k * (3 - 2 * k);
  return {x: prev.x + (next.x - prev.x) * e, y: prev.y + (next.y - prev.y) * e};
};

const Cursor = ({t}: {t: number}) => {
  const p = pointerAt(t);
  if (!p) return null;
  const click = cut.pointer.filter((q) => q.kind === "click" && q.t <= t && t - q.t < .45).at(-1);
  const ring = click ? (t - click.t) / .45 : 0;
  return (
    <div style={{position: "absolute", left: p.x * scale0, top: p.y * scale0, pointerEvents: "none"}}>
      {click && <div style={{position: "absolute", left: -22 - ring * 10, top: -22 - ring * 10, width: 44 + ring * 20, height: 44 + ring * 20, borderRadius: "50%", border: "2px solid rgba(255,255,255,.7)", opacity: 1 - ring}} />}
      <svg width="26" height="30" viewBox="0 0 26 30" style={{position: "absolute", left: -3, top: -2, filter: "drop-shadow(0 3px 6px rgba(0,0,0,.5))", transform: `scale(${click ? 1 - .12 * Math.sin(ring * Math.PI) : 1})`, transformOrigin: "3px 2px"}}>
        <path d="M3 2 L3 24 L9 18.5 L13 27 L17 25.2 L13 16.8 L21 16.8 Z" fill="#fff" stroke="#111" strokeWidth="1.6" strokeLinejoin="round" />
      </svg>
    </div>
  );
};

const Window = ({children, cam}: {children: React.ReactNode; cam: {s: number; x: number; y: number}}) => (
  <div style={{position: "absolute", left: STAGE.x, top: STAGE.y, width: STAGE.w, height: stageH, borderRadius: 14, overflow: "hidden", border: "1px solid rgba(255,255,255,.09)", boxShadow: "0 40px 120px rgba(0,0,0,.55), 0 0 0 1px rgba(0,0,0,.6)", background: "#000"}}>
    <div style={{position: "absolute", inset: 0, transformOrigin: "0 0", transform: `translate(${cam.x}px, ${cam.y}px) scale(${cam.s})`}}>{children}</div>
  </div>
);

const ShotView = ({shot, index}: {shot: Shot; index: number}) => {
  const frame = useCurrentFrame();
  const {fps} = useVideoConfig();
  const t = shot.start + (frame / fps) * shot.speed;
  const previous = index ? cameraFor(shots[index - 1]) : {s: 1, x: 0, y: 0};
  const target = cameraFor(shot);
  const k = spring({frame, fps, config: {damping: 200, mass: 1.1}, durationInFrames: 50});
  const settle = interpolate(frame, [frames(shot) - 40, frames(shot)], [0, 1], {extrapolateLeft: "clamp", extrapolateRight: "clamp", easing: Easing.inOut(Easing.cubic)});
  const nextCam = index < shots.length - 1 && !shots[index + 1].focus ? {s: 1, x: 0, y: 0} : target;
  const lerp = (a: number, b: number, p: number) => a + (b - a) * p;
  const mid = {s: lerp(previous.s, target.s, k), x: lerp(previous.x, target.x, k), y: lerp(previous.y, target.y, k)};
  const cam = {s: lerp(mid.s, nextCam.s, settle), x: lerp(mid.x, nextCam.x, settle), y: lerp(mid.y, nextCam.y, settle)};
  return (
    <>
      <Window cam={cam}>
        <OffthreadVideo src={staticFile(cut.video)} trimBefore={Math.round(shot.start * fps)} playbackRate={shot.speed} muted style={{width: STAGE.w, display: "block"}} />
        <Cursor t={t} />
      </Window>
      {shot.speed > 1.05 && (
        <div style={{position: "absolute", right: STAGE.x + 18, top: STAGE.y + stageH + 18, font: `500 15px ${mono}`, color: muted, letterSpacing: ".08em"}}>SPED UP {shot.speed.toFixed(1)}×</div>
      )}
    </>
  );
};

export const PytxoLive = () => {
  useFilmFonts();
  const frame = useCurrentFrame();
  const titleKey = (i: number) => shots[i].title;
  return (
    <AbsoluteFill style={{background: `radial-gradient(1400px 700px at 50% -10%, #17171c, ${bg} 70%)`, overflow: "hidden"}}>
      <Dots />
      {/* Intro: the aperture mark and the promise. */}
      <Sequence durationInFrames={INTRO}>
        <AbsoluteFill style={{display: "grid", placeContent: "center", justifyItems: "center", gap: 34, opacity: interpolate(frame, [INTRO - 16, INTRO], [1, 0], {extrapolateLeft: "clamp"})}}>
          <div style={{opacity: interpolate(frame, [0, 20], [0, 1], {extrapolateRight: "clamp"})}}><Glyph size={300} phase={frame / 60 * .5} /></div>
          <div style={{font: `600 72px ${sans}`, letterSpacing: "-.03em", color: ink, opacity: interpolate(frame, [18, 42], [0, 1], {extrapolateLeft: "clamp", extrapolateRight: "clamp"})}}>Many agents. One reviewed change.</div>
        </AbsoluteFill>
      </Sequence>
      {shots.map((shot, i) => (
        <Sequence key={i} from={starts[i]} durationInFrames={frames(shot)}>
          <ShotView shot={shot} index={i} />
        </Sequence>
      ))}
      {shots.map((shot, i) => (i && titleKey(i) === titleKey(i - 1) ? null : (
        <Title key={`t${i}`} kicker={shot.kicker} title={shot.title} at={starts[i]} out={(() => { let j = i; while (j + 1 < shots.length && titleKey(j + 1) === titleKey(i)) j++; return starts[j] + frames(shots[j]); })()} />
      )))}
      {cut.result && (
        <Sequence from={SHOTS_END} durationInFrames={RESULT}>
          <Title kicker="08 · Result" title="The applied app, running." at={0} out={RESULT} />
          <div style={{position: "absolute", left: STAGE.x + 100, top: STAGE.y, width: STAGE.w - 200, borderRadius: 14, overflow: "hidden", border: "1px solid rgba(255,255,255,.09)", boxShadow: "0 40px 120px rgba(0,0,0,.55)"}}>
            <OffthreadVideo src={staticFile(cut.result)} muted style={{width: "100%", display: "block"}} />
          </div>
        </Sequence>
      )}
      <Sequence from={LIVE_FRAMES - OUTRO}>
        <AbsoluteFill style={{display: "grid", placeContent: "center", justifyItems: "center", gap: 26, background: bg, opacity: interpolate(frame - (LIVE_FRAMES - OUTRO), [0, 18], [0, 1], {extrapolateRight: "clamp"})}}>
          <Glyph size={200} phase={frame / 60 * .5} />
          <div style={{font: `600 64px ${sans}`, letterSpacing: "-.03em", color: ink}}>pytxo</div>
          <div style={{font: `400 26px ${sans}`, color: muted}}>Free Windows beta · pytxo.com</div>
          <div style={{marginTop: 30, font: `500 14px ${mono}`, color: "#5b5b63", letterSpacing: ".08em"}}>RECORDED IN PYTXO DESKTOP · STAND-IN AGENTS REPLAY A REAL RUN · WAITS SHORTENED</div>
        </AbsoluteFill>
      </Sequence>
    </AbsoluteFill>
  );
};
