// PytxoLive: the real Pytxo Desktop, recorded natively during one journey take,
// shown full-bleed with one quiet caption per beat, an ElevenLabs voiceover and
// sound effects, and the music's drop landing as the agents start work. The
// pointer is redrawn from the take's own telemetry; the app is never re-created.
import {AbsoluteFill, Audio, Easing, interpolate, OffthreadVideo, Sequence, spring, staticFile, useCurrentFrame, useVideoConfig} from "remotion";
import {useFilmFonts} from "../film/kit";
import cut from "../../live-cut.json";

export const LIVE_FPS = 60;
// The intro and outro each hold their whole line plus a breath.
const INTRO = 300;
const OUTRO = 300;
const RESULT = 240;
const W = 1920, H = 1080;

type Rect = {x: number; y: number; w: number; h: number};
type Shot = {start: number; end: number; speed: number; kicker: string; title: string; focus: Rect | null; clip: string};

// Voiceover lines (seconds in vo.mp3) and the caption shown with each.
const VO = {
  intro: [0, 2.85],
  describe: [3.72, 5.24],
  agents: [6.13, 8.16],
  plan: [8.83, 10.7],
  fleet: [11.44, 14.92],
  review: [15.45, 17.62],
  stale: [18.38, 21.6],
  apply: [22.38, 25.3],
  outro: [25.77, 28.42],
} as const;
type Beat = keyof typeof VO;
// A shot's beat follows its kicker ("02 · Split"); Split shares Describe's line.
const BEAT_OF: Record<string, Beat> = {Describe: "describe", Split: "describe", "Choose agents": "agents", Plan: "plan", Fleet: "fleet", Review: "review", Stale: "stale", Apply: "apply"};
const hasSplit = (cut.shots as Shot[]).some((s) => s.kicker.endsWith("Split"));
const CAPTION: Partial<Record<Beat, string>> = {
  describe: hasSplit ? "Describe it once. Codex splits it." : "Describe it once.",
  agents: "Pick every agent you use.",
  plan: "See the plan first.",
  fleet: "They work side by side, isolated.",
  review: "Read every change and check.",
  stale: "Something changed? Apply refuses.",
  apply: "Apply exactly what you reviewed.",
};

// Each shot lasts at least as long as its line plus a breath.
const raw = cut.shots as Shot[];
const BEATS: Beat[] = raw.map((s) => BEAT_OF[s.kicker.split(" · ")[1]]);
const shots = raw.map((s, i) => {
  const beat = BEATS[i];
  const firstOfBeat = BEATS.indexOf(beat) === i;
  const need = firstOfBeat ? VO[beat][1] - VO[beat][0] + 0.5 : 0;
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

/** Deterministic pseudo-random for frame-stable ASCII. */
const hash = (a: number, b: number) => { const x = Math.sin(a * 127.1 + b * 311.7) * 43758.5453; return x - Math.floor(x); };
const CHARS = ".:+*=x";

/** A full-frame field of mono glyphs that breathes, then clears around the centre. */
const AsciiField = ({frame, fade}: {frame: number; fade: number}) => {
  const cols = 64, rows = 30;
  const cells = [];
  for (let r = 0; r < rows; r++) for (let c = 0; c < cols; c++) {
    const dx = (c - cols / 2) / (cols / 2), dy = (r - rows / 2) / (rows / 2);
    const dist = Math.sqrt(dx * dx + dy * dy);
    const wave = Math.sin(dist * 9 - frame / 9 + hash(c, r) * 6.28);
    const o = Math.max(0, wave) * .22 * fade * Math.min(1, dist * 1.4);
    if (o < .02) continue;
    cells.push(<text key={r * cols + c} x={c * 30 + 15} y={r * 36 + 20} opacity={o} fill={dist < .55 ? "#45dccb" : "#8b93ff"}>{CHARS[Math.floor(hash(c + Math.floor(frame / 6), r) * 6)]}</text>);
  }
  return <svg width={W} height={H} style={{position: "absolute", inset: 0, font: `500 18px ${mono}`}}>{cells}</svg>;
};

/** A short ASCII scan that sweeps across a cut. */
const AsciiWipe = ({at}: {at: number}) => {
  const frame = useCurrentFrame();
  const t = (frame - at + 10) / 22;
  if (t <= 0 || t >= 1) return null;
  const x = -200 + t * (W + 400);
  const cols = [];
  for (let i = 0; i < 9; i++) for (let r = 0; r < 30; r++) {
    const o = (1 - Math.abs(i - 4) / 5) * (.35 + .65 * hash(i + frame, r));
    cols.push(<text key={i * 30 + r} x={x + i * 22} y={r * 36 + 22} opacity={o} fill={i % 3 ? "#45dccb" : "#b98cff"}>{CHARS[Math.floor(hash(i * 7 + frame, r) * 6)]}</text>);
  }
  return (
    <svg width={W} height={H} style={{position: "absolute", inset: 0, font: `600 22px ${mono}`, pointerEvents: "none"}}>
      <rect x={x - 60} y={0} width={260} height={H} fill="rgba(9,9,11,.55)" />
      {cols}
    </svg>
  );
};

/** Types text out one glyph at a time, with a block cursor while it types. */
const Typed = ({text, at, perChar = 2.2, style}: {text: string; at: number; perChar?: number; style: React.CSSProperties}) => {
  const frame = useCurrentFrame();
  const n = Math.max(0, Math.min(text.length, Math.floor((frame - at) / perChar)));
  const typing = n < text.length && frame >= at;
  // The cursor hangs outside the text box so the words stay truly centred.
  return <div style={style}><span style={{position: "relative"}}>{text.slice(0, n)}<span style={{position: "absolute", left: "100%", opacity: typing || Math.floor(frame / 20) % 2 ? 1 : 0, color: "#45dccb"}}>▍</span></span></div>;
};

/** One quiet caption: a small dark pill near the bottom. */
const Caption = ({text, at, out}: {text: string; at: number; out: number}) => {
  const frame = useCurrentFrame();
  const p = interpolate(frame, [at, at + 18, out - 14, out], [0, 1, 1, 0], {extrapolateLeft: "clamp", extrapolateRight: "clamp", easing: Easing.bezier(.2, .8, .2, 1)});
  if (p <= 0) return null;
  return (
    <div style={{position: "absolute", left: 0, right: 0, bottom: 72, display: "flex", justifyContent: "center", opacity: p, transform: `translateY(${(1 - p) * 8}px)`}}>
      <div style={{padding: "18px 34px", borderRadius: 999, background: "rgba(9,9,11,.86)", border: "1px solid rgba(255,255,255,.12)", boxShadow: "0 24px 60px rgba(0,0,0,.5)", font: `600 46px ${sans}`, letterSpacing: "-.015em", color: ink}}>{text}</div>
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
  at: beat === "intro" ? 70 : beat === "outro" ? LIVE_FRAMES - OUTRO + 50 : starts[BEATS.indexOf(beat)] + 12,
  len: Math.round((VO[beat][1] - VO[beat][0]) * LIVE_FPS),
}));
// 0..1: how far the music is ducked, ramping over 12 frames around each line.
const duck = (f: number) => Math.max(0, ...voCues.map((v) => Math.min(1, (f - v.at + 14) / 12, (v.at + v.len + 14 - f) / 12)));

const Sound = () => (
  <>
    <Audio src={staticFile("live/audio/music.mp3")} trimBefore={MUSIC_FROM} volume={(f) => {
      const fade = Math.max(0, Math.min(1, f / 45, (LIVE_FRAMES - f) / 120));
      // Music (-7 LUFS master) sits well under the -15 LUFS voice: about -25 LUFS alone, -34 under speech.
      return fade * (0.12 - 0.085 * duck(f));
    }} />
    {voCues.map((v) => (
      <Sequence key={v.beat} from={v.at} durationInFrames={v.len + 6}>
        <Audio src={staticFile("live/audio/vo.wav")} trimBefore={Math.round(VO[v.beat][0] * LIVE_FPS)} />
      </Sequence>
    ))}
    {clickFrames.map((f, i) => <Sequence key={`c${i}`} from={f} durationInFrames={20}><Audio src={staticFile("live/audio/click.wav")} volume={0.45} /></Sequence>)}
    {starts.slice(1).map((f, i) => <Sequence key={`w${i}`} from={f - 8} durationInFrames={70}><Audio src={staticFile("live/audio/whoosh.wav")} volume={0.3} /></Sequence>)}
    {staleFrame !== null && <Sequence from={staleFrame} durationInFrames={60}><Audio src={staticFile("live/audio/thud.wav")} volume={0.9} /></Sequence>}
    {appliedFrame !== null && <Sequence from={appliedFrame} durationInFrames={120}><Audio src={staticFile("live/audio/chime.wav")} volume={0.7} /></Sequence>}
  </>
);

export const PytxoLive = () => {
  useFilmFonts();
  const frame = useCurrentFrame();
  return (
    <AbsoluteFill style={{background: bg, overflow: "hidden"}}>
      <Sequence durationInFrames={INTRO}>
        <AbsoluteFill style={{opacity: interpolate(frame, [INTRO - 18, INTRO], [1, 0], {extrapolateLeft: "clamp"})}}>
          <AsciiField frame={frame} fade={interpolate(frame, [0, 40, INTRO - 60, INTRO], [0, 1, 1, .3], {extrapolateRight: "clamp"})} />
          <AbsoluteFill style={{display: "grid", placeContent: "center", justifyItems: "center", gap: 40}}>
            <div style={{transform: `scale(${spring({frame: frame - 6, fps: 60, config: {damping: 18, mass: .9}})})`}}><Glyph size={320} phase={frame / 60 * .7} /></div>
            <Typed text="Many agents. One reviewed change." at={62} perChar={2.4} style={{font: `600 76px ${sans}`, letterSpacing: "-.03em", color: ink}} />
          </AbsoluteFill>
        </AbsoluteFill>
      </Sequence>
      {shots.map((_shot, i) => (
        <Sequence key={i} from={starts[i]} durationInFrames={frames(shots[i])}>
          <ShotView index={i} />
        </Sequence>
      ))}
      {starts.slice(1).map((at) => <AsciiWipe key={`wipe${at}`} at={at} />)}
      {cut.result && <AsciiWipe at={SHOTS_END} />}
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
        <AbsoluteFill style={{background: bg, opacity: interpolate(frame - (LIVE_FRAMES - OUTRO), [0, 18], [0, 1], {extrapolateRight: "clamp"})}}>
          <AsciiField frame={frame} fade={interpolate(frame - (LIVE_FRAMES - OUTRO), [0, 40], [0, .8], {extrapolateRight: "clamp"})} />
        </AbsoluteFill>
        <AbsoluteFill style={{display: "grid", placeContent: "center", justifyItems: "center", gap: 28, opacity: interpolate(frame - (LIVE_FRAMES - OUTRO), [0, 18], [0, 1], {extrapolateRight: "clamp"})}}>
          <Glyph size={240} phase={frame / 60 * .7} />
          <Typed text="pytxo" at={24} perChar={5} style={{font: `600 96px ${sans}`, letterSpacing: "-.035em", color: ink}} />
          <div style={{font: `400 32px ${sans}`, color: muted, opacity: interpolate(frame - (LIVE_FRAMES - OUTRO), [70, 95], [0, 1], {extrapolateLeft: "clamp", extrapolateRight: "clamp"})}}>Free Windows beta · pytxo.com</div>
          <div style={{marginTop: 30, font: `500 17px ${mono}`, color: "#6b6b74", letterSpacing: ".08em"}}>RECORDED IN PYTXO DESKTOP · STAND-IN AGENTS REPLAY A REAL RUN · WAITS SHORTENED</div>
        </AbsoluteFill>
      </Sequence>
      <Sound />
    </AbsoluteFill>
  );
};
