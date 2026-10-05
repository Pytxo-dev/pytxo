// PytxoLive: the real Pytxo Desktop, recorded natively during one journey take,
// shown full-bleed with one quiet caption per beat, an ElevenLabs voiceover and
// sound effects, and the music's drop landing as the agents start work. The
// pointer is redrawn from the take's own telemetry; the app is never re-created.
import {AbsoluteFill, Audio, Easing, interpolate, OffthreadVideo, Sequence, spring, staticFile, useCurrentFrame, useVideoConfig} from "remotion";
import {useFilmFonts} from "../film/kit";
import cut from "../../live-cut.json";

export const LIVE_FPS = 60;
const INTRO = 180;
const OUTRO = 240;
const RESULT = 270;
const W = 1920, H = 1080;

type Rect = {x: number; y: number; w: number; h: number};
type Shot = {start: number; end: number; speed: number; kicker: string; title: string; focus: Rect | null; clip: string};

// Voiceover lines (seconds in vo.mp3) and the caption shown with each.
const VO = {
  intro: [0, 2.98],
  describe: [3.87, 5.4],
  agents: [6.18, 8.1],
  plan: [8.98, 10.88],
  fleet: [11.56, 15.02],
  review: [15.96, 18.2],
  stale: [18.97, 22.47],
  apply: [22.85, 25.7],
  outro: [26.14, 28.79],
} as const;
type Beat = keyof typeof VO;
const BEATS: Beat[] = ["describe", "agents", "plan", "fleet", "fleet", "review", "stale", "apply"];
const CAPTION: Partial<Record<Beat, string>> = {
  describe: "Describe it once.",
  agents: "Pick every agent you use.",
  plan: "See the plan first.",
  fleet: "They work side by side, isolated.",
  review: "Read every change and check.",
  stale: "Something changed? Apply refuses.",
  apply: "Apply exactly what you reviewed.",
};

// Each shot lasts at least as long as its line plus a breath.
const raw = cut.shots as Shot[];
const shots = raw.map((s, i) => {
  const beat = BEATS[i];
  const firstOfBeat = BEATS.indexOf(beat) === i;
  const need = firstOfBeat ? VO[beat][1] - VO[beat][0] + 0.9 : 0;
  const speed = need ? Math.max(1, Math.min(s.speed, (s.end - s.start) / need)) : s.speed;
  return {...s, speed, cutSpeed: s.speed, beat, firstOfBeat};
});
const frames = (s: {start: number; end: number; speed: number}) => Math.round(((s.end - s.start) / s.speed) * LIVE_FPS);
const starts = shots.reduce<number[]>((list, _s, i) => [...list, i ? list[i - 1] + frames(shots[i - 1]) : INTRO], []);
const SHOTS_END = starts.at(-1)! + frames(shots.at(-1)!);
export const LIVE_FRAMES = SHOTS_END + (cut.result ? RESULT : 0) + OUTRO;

const ink = "#F4F4F5";
const muted = "#8B8B94";
const bg = "#09090B";
const sans = "'Sora', system-ui, sans-serif";
const mono = "'IBM Plex Mono', ui-monospace, monospace";

// The take (1920 wide, taskbar cropped) is scaled to cover the frame.
const cover = Math.max(W / cut.width, H / cut.height);
// Anchored left: the sidebar stays whole; only window chrome on the right trims.
const offX = 0;

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

/** One quiet caption: a small dark pill near the bottom. */
const Caption = ({text, at, out}: {text: string; at: number; out: number}) => {
  const frame = useCurrentFrame();
  const p = interpolate(frame, [at, at + 18, out - 14, out], [0, 1, 1, 0], {extrapolateLeft: "clamp", extrapolateRight: "clamp", easing: Easing.bezier(.2, .8, .2, 1)});
  if (p <= 0) return null;
  return (
    <div style={{position: "absolute", left: 0, right: 0, bottom: 64, display: "flex", justifyContent: "center", opacity: p, transform: `translateY(${(1 - p) * 8}px)`}}>
      <div style={{padding: "14px 26px", borderRadius: 999, background: "rgba(9,9,11,.8)", border: "1px solid rgba(255,255,255,.1)", boxShadow: "0 18px 50px rgba(0,0,0,.45)", font: `500 34px ${sans}`, letterSpacing: "-.015em", color: ink}}>{text}</div>
    </div>
  );
};

const cameraFor = (s: Shot) => {
  if (!s.focus) return {s: 1, x: 0, y: 0};
  const zoom = Math.min(1.6, Math.max(1.15, Math.min(cut.width / s.focus.w, cut.height / s.focus.h) * .8));
  const cx = offX + (s.focus.x + s.focus.w / 2) * cover, cy = (s.focus.y + s.focus.h / 2) * cover;
  const x = Math.min(0, Math.max(W - W * zoom, W / 2 - cx * zoom));
  const y = Math.min(0, Math.max(H - H * zoom, H / 2 - cy * zoom));
  return {s: zoom, x, y};
};

type Move = {t: number; x: number; y: number; kind: string};
const moves = cut.pointer.filter((p) => p.x !== undefined && p.x !== null) as Move[];
const pointerAt = (t: number) => {
  let prev = moves[0], next = moves[0];
  for (const p of moves) { if (p.t <= t) { prev = next; next = p; } else break; }
  if (!next) return null;
  const k = Math.min(1, Math.max(0, (t - next.t) / (next.kind === "drag" ? 1.2 : 0.5)));
  const e = k * k * (3 - 2 * k);
  return {x: prev.x + (next.x - prev.x) * e, y: prev.y + (next.y - prev.y) * e};
};

const Cursor = ({t}: {t: number}) => {
  const p = pointerAt(t);
  if (!p) return null;
  const click = cut.pointer.filter((q) => q.kind === "click" && q.t <= t && t - q.t < .45).at(-1);
  const ring = click ? (t - click.t) / .45 : 0;
  return (
    <div style={{position: "absolute", left: offX + p.x * cover, top: p.y * cover}}>
      {click && <div style={{position: "absolute", left: -22 - ring * 10, top: -22 - ring * 10, width: 44 + ring * 20, height: 44 + ring * 20, borderRadius: "50%", border: "2px solid rgba(255,255,255,.7)", opacity: 1 - ring}} />}
      <svg width="30" height="34" viewBox="0 0 26 30" style={{position: "absolute", left: -3, top: -2, filter: "drop-shadow(0 3px 6px rgba(0,0,0,.5))", transform: `scale(${click ? 1 - .12 * Math.sin(ring * Math.PI) : 1})`, transformOrigin: "3px 2px"}}>
        <path d="M3 2 L3 24 L9 18.5 L13 27 L17 25.2 L13 16.8 L21 16.8 Z" fill="#fff" stroke="#111" strokeWidth="1.6" strokeLinejoin="round" />
      </svg>
    </div>
  );
};

const ShotView = ({index}: {index: number}) => {
  const frame = useCurrentFrame();
  const {fps} = useVideoConfig();
  const shot = shots[index];
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
    <AbsoluteFill style={{overflow: "hidden", background: "#000", opacity: interpolate(frame, [0, 10], [0, 1], {extrapolateRight: "clamp"})}}>
      <div style={{position: "absolute", inset: 0, transformOrigin: "0 0", transform: `translate(${cam.x}px, ${cam.y}px) scale(${cam.s})`}}>
        <OffthreadVideo src={staticFile(shot.clip)} playbackRate={shot.speed / shot.cutSpeed} muted style={{position: "absolute", left: offX, top: 0, width: cut.width * cover}} />
        <Cursor t={t} />
      </div>
    </AbsoluteFill>
  );
};

/** Video-time → film frame for a moment inside a shot (or null when cut away). */
const filmFrame = (t: number) => {
  for (let i = 0; i < shots.length; i++) {
    const s = shots[i];
    if (t >= s.start && t < s.end) return starts[i] + Math.round(((t - s.start) / s.speed) * LIVE_FPS);
  }
  return null;
};
const marks = cut.marks as Record<string, number>;
const clickFrames = cut.pointer.filter((p) => p.kind === "click").map((p) => filmFrame(p.t)).filter((f): f is number => f !== null);
const staleFrame = filmFrame(marks.stale);
const appliedFrame = filmFrame(marks.applied);

// The song's second drop (74 s) lands when the fleet starts working.
const FLEET_START = starts[BEATS.indexOf("fleet")];
const MUSIC_FROM = Math.max(0, 74 * LIVE_FPS - FLEET_START);
const voCues = (Object.keys(VO) as Beat[]).map((beat) => ({
  beat,
  at: beat === "intro" ? 24 : beat === "outro" ? LIVE_FRAMES - OUTRO + 30 : starts[BEATS.indexOf(beat)] + 12,
  len: Math.round((VO[beat][1] - VO[beat][0]) * LIVE_FPS),
}));
const speaking = (f: number) => voCues.some((v) => f >= v.at - 10 && f <= v.at + v.len + 10);

const Sound = () => (
  <>
    <Audio src={staticFile("live/audio/music.mp3")} trimBefore={MUSIC_FROM} volume={(f) => {
      const fade = Math.max(0, Math.min(1, f / 45, (LIVE_FRAMES - f) / 120));
      return fade * (speaking(f) ? 0.22 : 0.5);
    }} />
    {voCues.map((v) => (
      <Sequence key={v.beat} from={v.at} durationInFrames={v.len + 6}>
        <Audio src={staticFile("live/audio/vo.mp3")} trimBefore={Math.round(VO[v.beat][0] * LIVE_FPS)} />
      </Sequence>
    ))}
    {clickFrames.map((f, i) => <Sequence key={`c${i}`} from={f} durationInFrames={20}><Audio src={staticFile("live/audio/click.wav")} volume={0.55} /></Sequence>)}
    {starts.slice(1).map((f, i) => <Sequence key={`w${i}`} from={f - 8} durationInFrames={70}><Audio src={staticFile("live/audio/whoosh.mp3")} volume={0.25} /></Sequence>)}
    {staleFrame !== null && <Sequence from={staleFrame} durationInFrames={60}><Audio src={staticFile("live/audio/thud.mp3")} volume={0.7} /></Sequence>}
    {appliedFrame !== null && <Sequence from={appliedFrame} durationInFrames={120}><Audio src={staticFile("live/audio/chime.mp3")} volume={0.6} /></Sequence>}
  </>
);

export const PytxoLive = () => {
  useFilmFonts();
  const frame = useCurrentFrame();
  return (
    <AbsoluteFill style={{background: bg, overflow: "hidden"}}>
      <Sequence durationInFrames={INTRO}>
        <AbsoluteFill style={{display: "grid", placeContent: "center", justifyItems: "center", gap: 34, opacity: interpolate(frame, [INTRO - 16, INTRO], [1, 0], {extrapolateLeft: "clamp"})}}>
          <div style={{opacity: interpolate(frame, [0, 20], [0, 1], {extrapolateRight: "clamp"})}}><Glyph size={300} phase={frame / 60 * .5} /></div>
          <div style={{font: `600 72px ${sans}`, letterSpacing: "-.03em", color: ink, opacity: interpolate(frame, [18, 42], [0, 1], {extrapolateLeft: "clamp", extrapolateRight: "clamp"})}}>Many agents. One reviewed change.</div>
        </AbsoluteFill>
      </Sequence>
      {shots.map((_shot, i) => (
        <Sequence key={i} from={starts[i]} durationInFrames={frames(shots[i])}>
          <ShotView index={i} />
        </Sequence>
      ))}
      {shots.map((shot, i) => {
        if (!shot.firstOfBeat) return null;
        let j = i;
        while (j + 1 < shots.length && shots[j + 1].beat === shot.beat) j++;
        return <Caption key={`c${i}`} text={CAPTION[shot.beat]!} at={starts[i] + 6} out={starts[j] + frames(shots[j])} />;
      })}
      {cut.result && (
        <Sequence from={SHOTS_END} durationInFrames={RESULT}>
          <AbsoluteFill style={{background: "#000"}}>
            <OffthreadVideo src={staticFile(cut.result)} muted style={{width: W, height: H, objectFit: "cover"}} />
          </AbsoluteFill>
          <Caption text="The result, running." at={6} out={RESULT} />
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
      <Sound />
    </AbsoluteFill>
  );
};
