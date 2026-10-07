// PytxoFilm: a calm, Notion-style walkthrough of the recorded 2 October run.
// The interface is recreated in motion graphics from the run ledger
// (fleet-props.json, film-diff.json); nothing on screen is invented.
import {Audio} from "@remotion/media";
import {AbsoluteFill, Img, interpolate, Sequence, staticFile, useCurrentFrame} from "remotion";
import {Boundary, BOUNDARY_FRAMES} from "./Boundary";
import {Logo, Rise, T, fade, ledger, mono, ramp, sans, useFilmFonts, usePop, useSettle, vendorOf} from "./kit";
import {Plan, PLAN_FRAMES} from "./Plan";

const OPENING = 240, CLOSING = 300, OVERLAP = 24;
const PLAN_AT = OPENING - OVERLAP;
const BOUNDARY_AT = PLAN_AT + PLAN_FRAMES - OVERLAP;
const CLOSING_AT = BOUNDARY_AT + BOUNDARY_FRAMES - OVERLAP;
export const FILM_FRAMES = CLOSING_AT + CLOSING;
const AGENTS = ["codex", "claude", "cursor", "opencode", "agy"];

const Opening = () => {
  const frame = useCurrentFrame();
  return <AbsoluteFill style={{background: T.paper, alignItems: "center", justifyContent: "center"}}>
    <div style={{display: "flex", flexDirection: "column", alignItems: "center", gap: 46, transform: `translateY(${-ramp(frame, 180, 240) * 30}px)`}}>
      <Rise text="Put your coding agents" at={10} size={104} style={{justifyContent: "center"}} />
      <Rise text="on one job." at={26} size={104} color={T.muted} style={{justifyContent: "center", marginTop: -30}} />
      <div style={{display: "flex", gap: 22, marginTop: 10}}>
        {AGENTS.map((cli, index) => {
          const pop = usePop(70 + index * 7);
          return <div key={cli} style={{display: "flex", flexDirection: "column", alignItems: "center", gap: 12, transform: `translateY(${(1 - pop) * 24}px) scale(${0.9 + pop * 0.1})`, opacity: Math.min(1, pop)}}>
            <Logo cli={cli} size={76} />
            <span style={{fontFamily: sans, fontSize: 17, color: T.muted}}>{vendorOf(cli)}</span>
          </div>;
        })}
      </div>
    </div>
  </AbsoluteFill>;
};

const Closing = () => {
  const frame = useCurrentFrame();
  const mark = useSettle(8, 50);
  return <AbsoluteFill style={{background: T.paper, alignItems: "center", justifyContent: "center"}}>
    <div style={{display: "flex", flexDirection: "column", alignItems: "center", gap: 30}}>
      <Img src={staticFile("logo-mark.png")} style={{width: 112, height: 112, opacity: mark, transform: `scale(${0.85 + mark * 0.15})`}} />
      <Rise text="Put your coding agents on one job." at={24} size={76} style={{justifyContent: "center"}} />
      <div style={{fontFamily: sans, fontSize: 26, color: T.muted, opacity: fade(frame, 60, CLOSING + 40)}}>Pytxo Desktop · Windows beta · pytxo.com</div>
    </div>
    <div style={{position: "absolute", bottom: 54, fontFamily: mono, fontSize: 16, color: T.faint, opacity: fade(frame, 80, CLOSING + 40), textAlign: "center", lineHeight: 1.6}}>
      Recorded run, 2 October 2026: {ledger.workers.length} tasks, {ledger.vendors.length} agent CLIs, {ledger.checks.passed}/{ledger.checks.recorded} checks, {ledger.files.length} files applied.<br />
      Interface recreated from the run ledger · time compressed · agent output and files as recorded.
    </div>
  </AbsoluteFill>;
};

/** Crossfades each scene in over the previous one. */
const Scene = ({from, duration, children, name}: {from: number; duration: number; children: React.ReactNode; name: string}) => <Sequence name={name} from={from} durationInFrames={duration}>
  <FadeIn>{children}</FadeIn>
</Sequence>;
const FadeIn = ({children}: {children: React.ReactNode}) => {
  const frame = useCurrentFrame();
  return <AbsoluteFill style={{opacity: interpolate(frame, [0, OVERLAP], [0, 1], {extrapolateRight: "clamp"})}}>{children}</AbsoluteFill>;
};

const CUES: [number, string, number][] = [
  [80, "drop_002.ogg", 0.12],
  [PLAN_AT + 62, "click_003.ogg", 0.14],
  [PLAN_AT + 352, "click_003.ogg", 0.14],
  [PLAN_AT + 380, "switch_002.ogg", 0.1],
  [PLAN_AT + 1200, "confirmation_001.ogg", 0.08],
  [BOUNDARY_AT + 40, "drop_002.ogg", 0.12],
  [BOUNDARY_AT + 330, "click_003.ogg", 0.12],
  [BOUNDARY_AT + 530, "click_003.ogg", 0.14],
  [BOUNDARY_AT + 548, "drop_002.ogg", 0.2],
  [BOUNDARY_AT + 650, "click_003.ogg", 0.14],
  [BOUNDARY_AT + 800, "click_003.ogg", 0.14],
  [BOUNDARY_AT + 830, "confirmation_001.ogg", 0.12],
  [CLOSING_AT + 10, "drop_002.ogg", 0.12],
];

export const PytxoFilm = () => {
  useFilmFonts();
  return <AbsoluteFill style={{background: T.paper}}>
    <Scene name="Opening" from={0} duration={OPENING}><Opening /></Scene>
    <Scene name="Plan and run" from={PLAN_AT} duration={PLAN_FRAMES}><Plan /></Scene>
    <Scene name="Review, refuse, Apply" from={BOUNDARY_AT} duration={BOUNDARY_FRAMES}><Boundary /></Scene>
    <Scene name="Closing" from={CLOSING_AT} duration={CLOSING}><Closing /></Scene>
    <Audio src={staticFile("audio/cc0/holizna-breath.mp3")} trimBefore={8 * 60} volume={(f) => 0.5 * Math.min(1, f / 45, (FILM_FRAMES - f) / 120)} />
    {CUES.map(([at, file, gain], index) => <Sequence key={index} from={at} durationInFrames={90}><Audio src={staticFile(`audio/cc0/${file}`)} volume={gain} /></Sequence>)}
  </AbsoluteFill>;
};
