// PytxoDesktop: one native take of the released Pytxo Desktop 1.3.2, framed in
// the site's ASCII system and cut to the beat of the track in
// desktop-music.json. Every product shot is the real recording; the pointer is
// redrawn from the take's telemetry, shortened waits say so, and the ASCII art
// never reports product state.
import {AbsoluteFill, Audio, Easing, Freeze, interpolate, OffthreadVideo, Sequence, staticFile, useCurrentFrame} from "remotion";
import {useFilmFonts} from "../film/kit";
import {AsciiCanvas, drawAperture, drawTitle, drawWipe, GLITCH, hash, INK, MONO, SANS, titleCrisp, type TitleSpec} from "./ascii";
import cut from "../../desktop-cut.json";
import focus from "../../desktop-focus.json";
import track from "../../desktop-music.json";

export const DESKTOP_FPS = 60;
const FPS = DESKTOP_FPS;
const W = 1920, H = 1080;
const BG = "#060609";
const MUTED = "#8b8b94";
const ease = Easing.bezier(.16, 1, .3, 1);
const clamp = {extrapolateLeft: "clamp", extrapolateRight: "clamp"} as const;

// The beat grid. Film time equals track time until the outro.
const BEAT = 60 / track.bpm;
const BAR = 4 * BEAT;
const beatAt = (n: number) => track.firstBeat + n * BEAT;
const DROP = track.drop;
const f = (seconds: number) => Math.round(seconds * FPS);
const WIPE = f(BEAT);

type Rect = [number, number, number, number];
// free: the window may float above the frame's bottom edge, so a bar at the app's foot sits mid-frame, clear of the caption.
type Key = {at?: number; mark?: string; offset?: number; rect: Rect | null; free?: boolean};
// ramp: seconds into the shot where it drops to real time, at source time `at`.
type Shot = {id: string; start: number; end: number; speed: number; bars: number; clip: string; ramp?: number; at?: number};
const sourceAt = (s: Shot, sec: number) => s.ramp !== undefined && sec >= s.ramp ? s.at! + sec - s.ramp : s.start + sec * s.speed;
const screenAt = (s: Shot, t: number) => s.ramp !== undefined && t >= s.at! ? s.ramp + t - s.at! : (t - s.start) / s.speed;

const CAPTION: Record<string, [string, string]> = {
  describe: ["Describe", "One request, in plain words."],
  split: ["Split", `${cut.lead === "OpenAI Codex" ? "Codex" : cut.lead ?? "The lead agent"} splits it into owned tasks.`],
  agents: ["Team", "Put the agents you already use on it."],
  plan: ["Plan", "Every file has one owner."],
  fleet: ["Run", "Isolated copies. Your repo is untouched."],
  review: ["Review", "Checked as one combined change."],
  stale: ["Guard", "The repo moved after review. Apply refuses."],
  apply: ["Apply", "It writes exactly what you reviewed."],
  history: ["Record", "Every run and Apply, on the record."],
};

// Shots start on bar lines: the cold open holds three bars past the drop.
const shots = cut.shots as Shot[];
const OPEN_END = DROP + 3 * BAR;
const cuts = shots.reduce<number[]>((list, s, i) => [...list, (i ? list[i - 1] : OPEN_END) + s.bars * BAR], []);
const cutAt = (i: number) => f(i ? cuts[i - 1] : OPEN_END);
const OUT_AT = f(cuts.at(-1)!);
const OUTRO = f(7);
export const DESKTOP_FRAMES = OUT_AT + OUTRO;
const lengthOf = (i: number) => f(shots[i].bars * BAR);
const markFrame = (i: number, mark: string) => cutAt(i) + f(screenAt(shots[i], (cut.marks as Record<string, number>)[mark]));

// Base framing: the app at 84 % inside the ASCII field; captions live in the margin.
const BASE = .84;
const baseView = {s: BASE, x: (W - cut.width * BASE) / 2, y: (H - cut.height * BASE) / 2 - 6};
const viewOf = (rect: Rect | null, free = false) => {
  if (!rect) return baseView;
  const [fx, fy, fw, fh] = rect;
  const s = Math.max(BASE, Math.min(1.75, Math.min(W / fw, H / fh)));
  let x = W / 2 - (fx + fw / 2) * s, y = (free ? H * .5 : H / 2) - (fy + fh / 2) * s;
  if (s >= 1) { x = Math.min(0, Math.max(W - cut.width * s, x)); if (!free) y = Math.min(0, Math.max(H - cut.height * s, y)); }
  return {s, x, y};
};

/** Camera at a shot's local frame (0 = its downbeat): moves start on beats, with a punch on each downbeat of the run. */
const cameraAt = (i: number, local: number) => {
  const shot = shots[i];
  const length = lengthOf(i);
  const beat = BEAT * FPS;
  const keys = ((focus as unknown as Record<string, Key[]>)[shot.id] ?? [{at: 0, rect: null}]).map((k) => {
    const raw = k.mark ? markFrame(i, k.mark) - cutAt(i) + f(k.offset ?? 0) : f(k.at! < 0 ? shot.bars * BAR + k.at! : k.at!);
    return {f: Math.round(raw / beat) * beat, v: viewOf(k.rect, k.free)};
  });
  let v = keys[0].v;
  for (let k = 1; k < keys.length; k++) {
    const p = interpolate(local, [keys[k].f, keys[k].f + 2 * beat], [0, 1], {...clamp, easing: ease});
    v = {s: v.s + (keys[k].v.s - v.s) * p, x: v.x + (keys[k].v.x - v.x) * p, y: v.y + (keys[k].v.y - v.y) * p};
  }
  const sinceBar = (local % (4 * beat)) / FPS;
  const punch = shot.id === "fleet" && local >= 0 ? .014 * Math.exp(-sinceBar * 9) : 0;
  const drift = 1 + .02 * Math.max(0, local) / length + punch;
  return {s: v.s * drift, x: W / 2 - (W / 2 - v.x) * drift, y: H / 2 - (H / 2 - v.y) * drift};
};

type Move = {t: number; kind: string; x?: number; y?: number};
const moves = (cut.pointer as Move[]).filter((p) => typeof p.x === "number") as Required<Move>[];
const pointerAt = (t: number) => {
  let i = moves.findIndex((p) => p.t > t);
  if (i === -1) i = moves.length;
  const next = moves[i - 1];
  if (!next) return null;
  const prev = moves[i - 2] ?? next;
  const k = Math.min(1, Math.max(0, (t - next.t) / (next.kind === "drag" ? 1.2 : 0.5)));
  const e = k * k * (3 - 2 * k);
  return {x: prev.x + (next.x - prev.x) * e, y: prev.y + (next.y - prev.y) * e};
};

const Cursor = ({t, scale}: {t: number; scale: number}) => {
  const p = pointerAt(t);
  if (!p) return null;
  const click = cut.pointer.filter((q) => q.kind === "click" && q.t <= t && t - q.t < .45).at(-1);
  const ring = click ? (t - click.t) / .45 : 0;
  return (
    <div style={{position: "absolute", left: p.x, top: p.y, scale: String(1 / scale)}}>
      {click && <div style={{position: "absolute", left: -24 - ring * 12, top: -24 - ring * 12, width: 48 + ring * 24, height: 48 + ring * 24, borderRadius: "50%", border: "2px solid rgba(126,225,237,.85)", opacity: 1 - ring}} />}
      <svg width="30" height="34" viewBox="0 0 26 30" style={{position: "absolute", left: -3, top: -2, filter: "drop-shadow(0 3px 6px rgba(0,0,0,.55))", scale: String(click ? 1 - .12 * Math.sin(ring * Math.PI) : 1), transformOrigin: "3px 2px"}}>
        <path d="M3 2 L3 24 L9 18.5 L13 27 L17 25.2 L13 16.8 L21 16.8 Z" fill="#fff" stroke="#111" strokeWidth="1.6" strokeLinejoin="round" />
      </svg>
    </div>
  );
};

/** The take's own event, named: the journey writes a file into the repo after review. */
const Note = ({i, local}: {i: number; local: number}) => {
  const at = markFrame(i, "note") - cutAt(i), stale = markFrame(i, "stale") - cutAt(i);
  const o = Math.min(interpolate(local, [at, at + 14], [0, 1], clamp), interpolate(local, [stale - 14, stale - 2], [1, 0], clamp));
  if (o <= 0) return null;
  return <div style={{position: "absolute", top: 34, left: baseView.x, opacity: o, translate: `0 ${(1 - o) * 10}px`, font: `500 19px ${MONO}`, color: "#e9bb91", padding: "7px 14px", borderRadius: 4, background: "rgba(6,6,9,.86)", border: "1px solid #4a3a2a"}}>+ operator-note.txt written to the repo after review</div>;
};

const backdrop = (context: CanvasRenderingContext2D, frame: number) => {
  // The field breathes on every beat.
  const since = ((frame / FPS - track.firstBeat) % BEAT + BEAT) % BEAT;
  drawAperture(context, {time: frame / FPS, cx: W / 2, cy: H / 2, radius: 470, alpha: .3 + .22 * Math.exp(-since * 10), orbits: false});
};

/** One shot, revealed by the wipe in the beat before its downbeat, held under the next wipe. */
const ShotView = ({i}: {i: number}) => {
  const frame = useCurrentFrame();
  const local = frame - WIPE;
  const shot = shots[i];
  const cam = cameraAt(i, local);
  const t = sourceAt(shot, Math.max(0, local) / FPS);
  const sped = shot.speed >= 1.5 ? interpolate(local, [f(shot.ramp ?? 1e4) - 8, f(shot.ramp ?? 1e4)], [1, 0], clamp) : 0;
  const edge = interpolate(frame, [0, WIPE], [-120, W + 120], {...clamp, easing: Easing.inOut(Easing.cubic)});
  const video = <OffthreadVideo src={staticFile(shot.clip)} muted style={{position: "absolute", inset: 0, width: cut.width, height: cut.height}} />;
  return (
    <AbsoluteFill style={{clipPath: `inset(0 ${Math.max(0, W - edge)}px 0 0)`}}>
      <AbsoluteFill style={{background: BG}}><AsciiCanvas draw={backdrop} /></AbsoluteFill>
      <div style={{position: "absolute", left: 0, top: 0, width: cut.width, height: cut.height, transformOrigin: "0 0", translate: `${cam.x}px ${cam.y}px`, scale: String(cam.s), borderRadius: 14, overflow: "hidden", boxShadow: "0 0 0 1px rgba(255,255,255,.09), 0 40px 120px rgba(0,0,0,.65)"}}>
        {local < 0 ? <Freeze frame={0}>{video}</Freeze> : <Sequence from={WIPE} layout="none">{video}</Sequence>}
        <Cursor t={t} scale={cam.s} />
      </div>
      {/* Zoomed past the margin, captions get a dark floor under them. */}
      <AbsoluteFill style={{background: "linear-gradient(180deg, transparent 72%, rgba(4,4,6,.97) 90%)", opacity: interpolate(cam.s, [BASE, 1.05], [0, 1], clamp)}} />
      {shot.id === "stale" && <Note i={i} local={local} />}
      {sped > 0 && <div style={{position: "absolute", bottom: 30, opacity: sped, right: baseView.x, font: `500 17px ${MONO}`, letterSpacing: ".08em", color: "#c7c7ce", padding: "6px 12px", borderRadius: 4, background: "rgba(6,6,9,.82)", border: "1px solid #2a2a30"}}>{`SPED UP ${shot.speed < 3 ? shot.speed.toFixed(1) : Math.round(shot.speed)}×`}</div>}
    </AbsoluteFill>
  );
};

const Wipes = () => (
  <AsciiCanvas draw={(context, frame) => {
    shots.forEach((_s, i) => {
      const local = frame - (cutAt(i) - WIPE);
      if (local < 0 || local > WIPE) return;
      drawWipe(context, frame, interpolate(local, [0, WIPE], [-120, W + 120], {...clamp, easing: Easing.inOut(Easing.cubic)}));
    });
  }} />
);

/** Kicker and line in the margin under the frame, revealed from behind a mask on the downbeat. */
const Caption = ({number, kicker, line, at, out}: {number: number; kicker: string; line: string; at: number; out: number}) => {
  const frame = useCurrentFrame();
  if (frame < at || frame > out) return null;
  const enter = interpolate(frame, [at, at + 16], [100, 0], {...clamp, easing: ease});
  const leave = interpolate(frame, [out - 12, out], [0, -100], {...clamp, easing: Easing.in(Easing.cubic)});
  return (
    <div style={{position: "absolute", left: baseView.x, bottom: 26, display: "flex", alignItems: "baseline", gap: 22}}>
      <div style={{overflow: "hidden"}}><div style={{translate: `0 ${enter + leave}%`, font: `500 17px ${MONO}`, letterSpacing: ".16em", color: "#7ee1ed"}}>{`${String(number).padStart(2, "0")} ${kicker.toUpperCase()}`}</div></div>
      <div style={{overflow: "hidden", paddingBottom: 4}}><div style={{translate: `0 ${enter + leave}%`, font: `600 36px ${SANS}`, letterSpacing: "-.02em", color: INK}}>{line}</div></div>
    </div>
  );
};

// Cold open: one line per phrase of the intro, the break line on the drop.
const T = (spec: Omit<TitleSpec, "cx"> & {cx?: number}): TitleSpec => ({cx: W / 2, ...spec});
const OPEN_TITLES: TitleSpec[] = [
  T({text: "Your agents ship in parallel.", at: f(beatAt(0)), out: f(beatAt(7)), cy: 470, size: 92}),
  T({text: "Each branch passes.", at: f(beatAt(8)), out: f(OPEN_END) - 2, cy: 400, size: 92}),
  T({text: "Together, they break.", at: f(DROP) - 2, out: f(OPEN_END) - 2, cy: 520, size: 92, spectrum: true}),
];
const OUT_TITLES: TitleSpec[] = [
  T({text: "PYTXO", at: OUT_AT, out: DESKTOP_FRAMES + 60, cy: 390, size: 290, cell: [10, 15], spectrum: true, ascii: true}),
  T({text: "The landing layer for coding agents.", at: OUT_AT + f(2 * BEAT), out: DESKTOP_FRAMES + 60, cy: 610, size: 58}),
];

const Titles = ({titles}: {titles: TitleSpec[]}) => {
  const frame = useCurrentFrame();
  return (
    <>
      <AsciiCanvas draw={(context, at) => titles.forEach((t) => at >= t.at - 8 && at <= t.out + 2 && drawTitle(context, at, t))} />
      {titles.map((t) => {
        const o = titleCrisp(frame, t);
        return o > 0 && <div key={t.text} style={{position: "absolute", left: 0, right: 0, top: t.cy - t.size * .68, textAlign: "center", font: `${t.weight ?? 600} ${t.size}px ${SANS}`, letterSpacing: "-.03em", color: t.spectrum ? "#f3c64e" : INK, opacity: o, whiteSpace: "nowrap"}}>{t.text}</div>;
      })}
    </>
  );
};

/** A burst of glitch rows across the frame, decaying over a few frames from `at`. */
const drawBurst = (context: CanvasRenderingContext2D, frame: number, at: number) => {
  const k = (frame - at) / 10;
  if (k < 0 || k > 1) return;
  context.font = `600 18px ${MONO}`;
  context.textBaseline = "middle";
  for (let r = 0; r < 54; r++) {
    if (hash(r, at) > .45) continue;
    const y = r * 20 + 10, x0 = hash(r + 3, frame) * W * .6, len = 20 + hash(r, frame) * 60;
    context.globalAlpha = (1 - k) * .7;
    context.fillStyle = r % 3 ? "#7ee1ed" : "#f04da3";
    for (let c = 0; c < len; c++) context.fillText(GLITCH[(r + c + frame) % GLITCH.length], x0 + c * 11, y);
  }
  context.globalAlpha = 1;
};

const Open = () => {
  const frame = useCurrentFrame();
  if (frame > f(OPEN_END) + WIPE) return null;
  return (
    <AbsoluteFill style={{background: BG}}>
      <AsciiCanvas draw={(context, at) => {
        drawAperture(context, {time: at / FPS + 3, cx: W / 2, cy: H / 2, radius: 360, alpha: interpolate(at, [0, 30, f(DROP) - 20, f(DROP)], [0, .9, .9, .3], clamp), sweep: interpolate(at, [f(DROP), f(DROP) + 24], [W / 2 - 520, W / 2 + 520], clamp)});
        drawBurst(context, at, f(DROP));
      }} />
      <Titles titles={OPEN_TITLES} />
      {/* The real case behind the line, from the M2 first-use record. */}
      <div style={{position: "absolute", left: 0, right: 0, top: 640, textAlign: "center", font: `500 25px ${MONO}`, letterSpacing: ".01em", color: "#a9a9b2", opacity: interpolate(frame, [f(beatAt(18)), f(beatAt(18)) + 12, f(OPEN_END) - 14, f(OPEN_END)], [0, 1, 1, 0], clamp)}}>
        lukeed/mri · Claude Code renamed <span style={{color: INK}}>toVal</span> · Codex called it · combined: <span style={{color: "#ff6a4a"}}>ReferenceError: toVal is not defined</span>
      </div>
    </AbsoluteFill>
  );
};

const TEAM = (cut as {team?: string}).team ?? cut.lead ?? "agents";
const Outro = () => {
  const frame = useCurrentFrame();
  if (frame < OUT_AT - WIPE) return null;
  return (
    <AbsoluteFill style={{background: BG, opacity: interpolate(frame, [OUT_AT - WIPE, OUT_AT], [0, 1], clamp)}}>
      <AsciiCanvas draw={(context, at) => { drawAperture(context, {time: at / FPS, cx: W / 2, cy: H / 2, radius: 520, alpha: .18, orbits: true}); drawBurst(context, at, OUT_AT); }} />
      <Titles titles={OUT_TITLES} />
      <div style={{position: "absolute", left: 0, right: 0, top: 700, textAlign: "center", font: `500 30px ${SANS}`, color: MUTED, opacity: interpolate(frame, [OUT_AT + f(4 * BEAT), OUT_AT + f(4 * BEAT) + 14], [0, 1], clamp)}}>Download for Windows · pytxo.com</div>
      <div style={{position: "absolute", left: 0, right: 0, bottom: 54, textAlign: "center", font: `500 16px ${MONO}`, letterSpacing: ".1em", color: "#5d5d66", opacity: interpolate(frame, [OUT_AT + f(8 * BEAT), OUT_AT + f(8 * BEAT) + 14], [0, 1], clamp)}}>
        {`RECORDED IN PYTXO DESKTOP 1.3.2 · LIVE ${TEAM.toUpperCase()} · WAITS SHORTENED AND MARKED · MUSIC: ${track.credit.toUpperCase()}`}
      </div>
    </AbsoluteFill>
  );
};

const staleIndex = shots.findIndex((s) => s.id === "stale");
const STALE = staleIndex >= 0 ? markFrame(staleIndex, "stale") : -1;
const BACK = staleIndex >= 0 ? cutAt(staleIndex + 1) : -1;

const Music = () => (
  <>
    <Sequence durationInFrames={OUT_AT}>
      <Audio src={staticFile(track.src)} volume={(at) => {
        // The refusal is the one moment the music stops; it slams back on the Apply downbeat.
        const gap = STALE >= 0 && at >= STALE && at < BACK ? 0 : 1;
        return track.volume * gap * Math.min(1, (OUT_AT - at) / 2);
      }} />
    </Sequence>
    {/* The track's own closing stabs carry the wordmark. */}
    {track.ending.map((from, k) => (
      <Sequence key={from} from={OUT_AT + f(from - track.ending[0])} durationInFrames={k + 1 < track.ending.length ? f(track.ending[k + 1] - from) : OUTRO - f(from - track.ending[0])}>
        <Audio src={staticFile(track.src)} trimBefore={f(from)} volume={(at) => track.volume * Math.min(1, (OUTRO - f(from - track.ending[0]) - at) / 30)} />
      </Sequence>
    ))}
  </>
);

// audioOnly renders just the music, so an audio pass never asks the compositor for video frames.
export const PytxoDesktop = ({audioOnly = false}: {audioOnly?: boolean}) => {
  useFilmFonts();
  if (audioOnly) return <Music />;
  return (
    <AbsoluteFill style={{background: BG, overflow: "hidden"}}>
      <Open />
      {shots.map((shot, i) => (
        <Sequence key={shot.id} from={cutAt(i) - WIPE} durationInFrames={lengthOf(i) + 2 * WIPE}>
          <ShotView i={i} />
        </Sequence>
      ))}
      <Wipes />
      {shots.map((shot, i) => (
        <Caption key={`c${shot.id}`} number={i + 1} kicker={CAPTION[shot.id][0]} line={CAPTION[shot.id][1]} at={cutAt(i)} out={cutAt(i) + lengthOf(i) - WIPE} />
      ))}
      <Outro />
      <Music />
    </AbsoluteFill>
  );
};
