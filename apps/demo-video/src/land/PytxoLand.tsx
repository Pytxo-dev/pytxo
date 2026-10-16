// PytxoLand: one terminal take of the released `pytxo land` on lukeed/mri, using
// the branches Claude Code and Codex wrote in the M2 first-use run. Every frame of
// the terminal is the recording (scripts/land-take); speed only shortens typing
// and waits, and from 4x the factor is on screen. Captions and camera are the edit.
import {AbsoluteFill, Audio, interpolate, OffthreadVideo, Sequence, spring, staticFile, useCurrentFrame} from "remotion";
import {useFilmFonts} from "../film/kit";
import {AsciiField, Caption, Glyph, Typed} from "../live/PytxoLive";
import cut from "../../land-cut.json";

export const LAND_FPS = 30;
const W = 1920, H = 1080;
const INTRO = 120;
const OUTRO = 180;
const ink = "#F4F4F5";
const muted = "#8B8B94";
const bg = "#09090B";
const sans = "'Sora', system-ui, sans-serif";
const mono = "'IBM Plex Mono', ui-monospace, monospace";

type Rect = {x: number; y: number; w: number; h: number};
type Shot = {label: string; beat: string; part: string; start: number; end: number; speed: number; focus: Rect | null};
const shots = cut.shots as Shot[];
const len = (s: Shot) => Math.max(1, Math.round(((s.end - s.start) / s.speed) * LAND_FPS));
const starts = shots.reduce<number[]>((list, _s, i) => [...list, i ? list[i - 1] + len(shots[i - 1]) : INTRO], []);
const SHOTS_END = starts.at(-1)! + len(shots.at(-1)!);
export const LAND_FRAMES = SHOTS_END + OUTRO;

// The take sits 1:1 in the frame, like a window on a desk, with the caption band
// below it (the last terminal rows are often the ones that matter).
const X0 = (W - cut.width) / 2, Y0 = 40;
const CAPTION_TOP = Y0 + cut.height; // captions sit under this line
const CAPTION_SIZE = 34;

const CAPTION: Record<string, string> = {
  install: "Install the CLI from npm.",
  branches: "Claude Code and Codex each left a branch.",
  blocked: "Each passes alone. Together, they fail.",
  fix: "Fix it on one branch.",
  ready: "Same command: ready.",
  landed: "--apply lands it. One merge per branch.",
  receipts: "Every decision leaves a receipt.",
};

const clamp = (v: number, lo: number, hi: number) => Math.min(hi, Math.max(lo, v));
// Zoom keeps the take's left edge in view (output is left-aligned) and centres the
// focus rows above the caption band.
const cameraFor = (f: Rect | null) => {
  if (!f) return {s: 1, x: 0, y: 0};
  const s = clamp(Math.min((W - 96) / f.w, (CAPTION_TOP - 120) / f.h), 1, 1.4);
  const cy = Y0 + f.y + f.h / 2;
  return {s, x: clamp(48 - (X0 + f.x) * s, W - W * s, 0), y: clamp(CAPTION_TOP / 2 - cy * s, H - H * s, 0)};
};
const cams = shots.map((s) => cameraFor(s.focus));
const lerp = (a: number, b: number, p: number) => a + (b - a) * p;
const shotAt = (frame: number) => {
  let i = 0;
  while (i + 1 < shots.length && starts[i + 1] <= frame) i++;
  return i;
};
const cameraAt = (frame: number) => {
  const i = shotAt(frame);
  const from = i ? cams[i - 1] : cams[0];
  const k = spring({frame: frame - starts[i], fps: LAND_FPS, config: {damping: 200, mass: 1.1}, durationInFrames: 24});
  return {s: lerp(from.s, cams[i].s, k), x: lerp(from.x, cams[i].x, k), y: lerp(from.y, cams[i].y, k)};
};

const Take = ({index}: {index: number}) => {
  const shot = shots[index];
  return <OffthreadVideo src={staticFile("land/take.mp4")} trimBefore={Math.round(shot.start * LAND_FPS)} playbackRate={shot.speed} muted style={{position: "absolute", inset: 0, width: cut.width, height: cut.height}} />;
};

const firstOf = (part: string, label: string) => starts[shots.findIndex((s) => s.label === label && s.part === part)];
const enters = shots.map((s, i) => (s.part === "run" ? starts[i] : -1)).filter((f) => f >= 0);

const Sound = () => (
  <>
    <Audio src={staticFile("audio/cc0/holizna-breath.mp3")} volume={(f) => 0.16 * clamp(Math.min(f / 30, (LAND_FRAMES - f) / 75), 0, 1)} />
    {enters.map((f) => <Sequence key={`k${f}`} from={f} durationInFrames={15}><Audio src={staticFile("audio/cc0/click_003.ogg")} volume={0.3} /></Sequence>)}
    <Sequence from={firstOf("hold", "land-blocked")} durationInFrames={45}><Audio src={staticFile("audio/cc0/drop_002.ogg")} volume={0.75} /></Sequence>
    <Sequence from={firstOf("hold", "land-ready")} durationInFrames={60}><Audio src={staticFile("audio/cc0/confirmation_001.ogg")} volume={0.55} /></Sequence>
    <Sequence from={firstOf("hold", "land-apply")} durationInFrames={60}><Audio src={staticFile("audio/cc0/confirmation_001.ogg")} volume={0.7} /></Sequence>
  </>
);

export const PytxoLand = () => {
  useFilmFonts();
  const frame = useCurrentFrame();
  const cam = cameraAt(frame);
  const speed = frame >= INTRO ? shots[shotAt(frame)].speed : 1;
  const beats = [...new Set(shots.map((s) => s.beat))];
  const outroAt = LAND_FRAMES - OUTRO;
  return (
    <AbsoluteFill style={{background: bg, overflow: "hidden"}}>
      <Sequence from={INTRO - 12} durationInFrames={SHOTS_END - INTRO + 24}>
        <AbsoluteFill style={{transformOrigin: "0 0", transform: `translate(${cam.x}px, ${cam.y}px) scale(${cam.s})`, opacity: interpolate(frame, [INTRO - 12, INTRO + 6, SHOTS_END, SHOTS_END + 12], [0, 1, 1, 0], {extrapolateLeft: "clamp", extrapolateRight: "clamp"})}}>
          <div style={{position: "absolute", left: X0, right: X0, top: 12, display: "flex", justifyContent: "space-between", font: `500 15px ${mono}`, color: "#6b6b74", letterSpacing: ".06em"}}>
            <span>PYTXO 1.3.1 · LUKEED/MRI · REAL TERMINAL RECORDING</span>
            {speed >= 4 && <span style={{color: "#c7c7ce"}}>{`SPED UP ${Math.round(speed)}×`}</span>}
          </div>
          <div style={{position: "absolute", left: X0, top: Y0, width: cut.width, height: cut.height, borderRadius: 14, overflow: "hidden", border: "1px solid rgba(255,255,255,.10)", boxShadow: "0 40px 120px rgba(0,0,0,.65)", background: "#0d1117"}}>
            {shots.map((_s, i) => (
              <Sequence key={i} from={starts[i] - (INTRO - 12)} durationInFrames={len(shots[i])} layout="none">
                <Take index={i} />
              </Sequence>
            ))}
          </div>
        </AbsoluteFill>
      </Sequence>
      {/* Zoomed, the take runs past the caption band; fade it out there. */}
      <AbsoluteFill style={{background: `linear-gradient(transparent ${CAPTION_TOP - 90}px, ${bg} ${CAPTION_TOP + 10}px)`, opacity: clamp((cam.s - 1) / .2, 0, 1)}} />
      {beats.map((beat) => {
        const first = shots.findIndex((s) => s.beat === beat);
        const last = shots.map((s) => s.beat).lastIndexOf(beat);
        const from = beat === "blocked" || beat === "ready" ? firstOf("hold", shots[first].label) : starts[first];
        return <Caption key={beat} text={CAPTION[beat]} at={from + 4} out={starts[last] + len(shots[last])} bottom={10} size={CAPTION_SIZE} />;
      })}
      <Sequence durationInFrames={INTRO}>
        <AbsoluteFill style={{opacity: interpolate(frame, [INTRO - 14, INTRO], [1, 0], {extrapolateLeft: "clamp"})}}>
          <AsciiField frame={frame * 2} fade={interpolate(frame, [0, 20, INTRO - 30, INTRO], [0, 1, 1, .3], {extrapolateRight: "clamp"})} />
          <AbsoluteFill style={{display: "grid", placeContent: "center", justifyItems: "center", gap: 36}}>
            <div style={{transform: `scale(${spring({frame: frame - 3, fps: LAND_FPS, config: {damping: 18, mass: .9}})})`}}><Glyph size={280} phase={frame / 30 * .7} /></div>
            <Typed text="Both branches passed. Together, they broke." at={22} perChar={1.1} style={{font: `600 70px ${sans}`, letterSpacing: "-.03em", color: ink}} />
          </AbsoluteFill>
        </AbsoluteFill>
      </Sequence>
      <Sequence from={outroAt}>
        <AbsoluteFill style={{background: bg, opacity: interpolate(frame - outroAt, [0, 12], [0, 1], {extrapolateRight: "clamp"})}}>
          <AsciiField frame={frame * 2} fade={interpolate(frame - outroAt, [0, 20], [0, .8], {extrapolateRight: "clamp"})} />
        </AbsoluteFill>
        <AbsoluteFill style={{display: "grid", placeContent: "center", justifyItems: "center", gap: 24, opacity: interpolate(frame - outroAt, [0, 12], [0, 1], {extrapolateRight: "clamp"})}}>
          <Glyph size={220} phase={frame / 30 * .7} />
          <Typed text="pytxo" at={12} perChar={2.5} style={{font: `600 96px ${sans}`, letterSpacing: "-.035em", color: ink}} />
          <div style={{font: `500 34px ${mono}`, color: ink, opacity: interpolate(frame - outroAt, [36, 50], [0, 1], {extrapolateLeft: "clamp", extrapolateRight: "clamp"})}}>npm i -g pytxo</div>
          <div style={{font: `400 30px ${sans}`, color: muted, opacity: interpolate(frame - outroAt, [48, 62], [0, 1], {extrapolateLeft: "clamp", extrapolateRight: "clamp"})}}>Free and open source · pytxo.com</div>
          <div style={{marginTop: 26, font: `500 16px ${mono}`, color: "#6b6b74", letterSpacing: ".08em", textAlign: "center", lineHeight: 1.7}}>
            RECORDED WITH PYTXO 1.3.1 FROM NPM ON LUKEED/MRI · BRANCHES WRITTEN BY CLAUDE CODE AND CODEX
            <br />COMMANDS TYPED BY A SCRIPT · TYPING AND WAITS SHOWN FASTER · OUTPUT UNEDITED
          </div>
        </AbsoluteFill>
      </Sequence>
      <Sound />
    </AbsoluteFill>
  );
};

/** Product Hunt gallery still (1270x760): one frame of the take, cropped to the rows that matter. */
export type LandStillProps = {at: number; rows: [number, number]; cols: number; title: string};
export const PytxoLandStill = ({at, rows, cols, title}: LandStillProps) => {
  useFilmFonts();
  const cropH = (rows[1] - rows[0]) * 30;
  const cropW = cols * 13;
  const scale = Math.min(1190 / cropW, 560 / cropH, 1.4);
  return (
    <AbsoluteFill style={{background: bg, padding: "44px 40px 50px", display: "flex", flexDirection: "column", alignItems: "center", gap: 24}}>
      <div style={{font: `600 46px ${sans}`, letterSpacing: "-.02em", color: ink, textAlign: "center"}}>{title}</div>
      <div style={{flex: 1, display: "grid", placeItems: "center"}}>
        <div style={{width: cropW * scale + 32, height: cropH * scale + 32, borderRadius: 12, overflow: "hidden", border: "1px solid rgba(255,255,255,.10)", position: "relative", background: "#0d1117"}}>
          <OffthreadVideo src={staticFile("land/take.mp4")} trimBefore={Math.round(at * LAND_FPS)} muted style={{position: "absolute", left: 16, top: 16 - rows[0] * 30 * scale, width: cut.width * scale, height: cut.height * scale, clipPath: `inset(${rows[0] * 30 * scale}px ${(cut.width - cropW) * scale}px ${(cut.height - rows[1] * 30) * scale}px 0)`}} />
        </div>
      </div>
      <div style={{position: "absolute", bottom: 16, right: 22, font: `500 13px ${mono}`, color: "#6b6b74", letterSpacing: ".06em"}}>PYTXO 1.3.1 · REAL TERMINAL RECORDING · LUKEED/MRI</div>
    </AbsoluteFill>
  );
};
