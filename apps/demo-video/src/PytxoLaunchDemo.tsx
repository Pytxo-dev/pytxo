import type {ReactNode} from "react";
import {Audio} from "@remotion/media";
import {AbsoluteFill, Img, Sequence, interpolate, staticFile, useCurrentFrame} from "remotion";
import evidence from "../../../tooling/benchmarks/results/astra-ci-native-2026-09-07.json";

export type PytxoLaunchDemoProps = {includeAudio: boolean};
export const FPS = 30;
export const TOTAL_FRAMES = 52 * FPS;
const colors = {canvas: "#080b0e", text: "#f5f7fa", muted: "#a5afb9", mint: "#38d6bd"};
const clamp = {extrapolateLeft: "clamp" as const, extrapolateRight: "clamp" as const};
const Logo = () => (
  <div style={{display: "flex", alignItems: "center", gap: 16, fontSize: 34, fontWeight: 650}}>
    <Img src={staticFile("logo-mark.png")} style={{width: 40, height: 40}} />Pytxo
  </div>
);
const Frame = ({children, label}: {children: ReactNode; label: string}) => {
  const frame = useCurrentFrame();
  return (
    <AbsoluteFill style={{backgroundColor: colors.canvas, color: colors.text, fontFamily: "Geist Variable, Geist, sans-serif"}}>
      <div style={{position: "absolute", left: 90, top: 65}}><Logo /></div>
      <div style={{position: "absolute", right: 90, top: 78, fontSize: 22, color: colors.muted}}>Edited native captures · 7 September 2026</div>
      <div style={{position: "absolute", left: 90, top: 165, fontSize: 22, color: colors.mint, letterSpacing: "0.08em", textTransform: "uppercase"}}>{label}</div>
      <AbsoluteFill style={{opacity: interpolate(frame, [0, 12], [0, 1], clamp), translate: `0 ${interpolate(frame, [0, 16], [12, 0], clamp)}px`}}>{children}</AbsoluteFill>
      <div style={{position: "absolute", left: 90, bottom: 35, color: colors.muted, fontSize: 19}}>Run {evidence.run_id.slice(0, 8)} · MSI {evidence.msi_sha256.slice(0, 12)} · Codex 0.153.4</div>
    </AbsoluteFill>
  );
};
const Copy = ({title, children}: {title: ReactNode; children?: ReactNode}) => (
  <div style={{position: "absolute", left: 90, top: 265, width: 930}}>
    <h1 style={{fontSize: 83, lineHeight: 1.04, letterSpacing: "-0.045em", fontWeight: 640, margin: 0}}>{title}</h1>
    {children ? <div style={{marginTop: 38, fontSize: 31, lineHeight: 1.45, color: colors.muted}}>{children}</div> : null}
  </div>
);
// Unmodified element captures from the native rehearsal, not UI mockups.
const Receipt = () => (
  <div style={{position: "absolute", right: 100, top: 185, width: 600, display: "grid", gap: 28}}>
    <Img src={staticFile("product/astra-ci-native-checks.png")} style={{width: "100%"}} />
    <Img src={staticFile("product/astra-ci-native-boundaries.png")} style={{width: "100%"}} />
  </div>
);
const Intro = () => (
  <Frame label="One observed mission">
    <div style={{position: "absolute", left: 90, top: 290, fontSize: 108, lineHeight: 1.08, fontWeight: 650, letterSpacing: "-0.05em"}}>
      One harness.<br />Three scoped tasks.<br /><span style={{color: colors.mint}}>One reviewed result.</span>
    </div>
  </Frame>
);
const Mission = () => (
  <Frame label="The recorded task">
    <Copy title={<>Code. Tests. Docs.<br />One plan.</>}>
      Require review for credentials paths.<br />Two tasks first. The dependent test task follows.
      <div style={{marginTop: 50, paddingTop: 30, borderTop: "1px solid #293139", color: colors.text}}>{evidence.planned_tasks} tasks · {evidence.waves} waves · up to {evidence.max_concurrent_workers} workers</div>
      <div style={{fontSize: 25, marginTop: 15}}>The primary checkout stayed unchanged before Apply.</div>
    </Copy>
    <Img src={staticFile("product/astra-ci-native-plan.png")} style={{position: "absolute", right: 80, top: 345, width: 750, height: "auto"}} />
  </Frame>
);
const Review = () => (
  <Frame label="Combined candidate verification">
    <Copy title={<>Check the exact<br />combined result.</>}>
      Pytxo reran each task’s <span style={{color: colors.text}}>npm test</span><br />on the frozen combined candidate.
      <div style={{marginTop: 35}}>The receipt shows both enforced<br />and advisory boundaries.</div>
      <div style={{marginTop: 55, fontSize: 25}}>Included source is bound to this package.<br />Excluded dependencies and toolchains are not attested.</div>
    </Copy>
    <Receipt />
  </Frame>
);
const Apply = () => {
  const frame = useCurrentFrame();
  const applied = frame >= 6 * FPS;
  return (
    <Frame label={applied ? "Recorded result" : "Explicit confirmation"}>
      <Copy title={applied ? <>The reviewed<br />package applied.</> : <>Review the changes.<br />Confirm Apply.</>}>
        {applied ? "The receipt now records the package as applied." : "All three exact diffs were inspected in the native UI rehearsal."}
        <div style={{marginTop: 40, fontSize: 25}}>The native UI was operated by test automation.<br />The edit does not reproduce its elapsed time.</div>
      </Copy>
      {applied ? <Img src={staticFile("product/astra-ci-native-applied.png")} style={{position: "absolute", right: 80, top: 380, width: 670, height: "auto"}} /> : <Img src={staticFile("product/astra-ci-native-apply-confirm.png")}
        style={{position: "absolute", right: 80, top: 365, width: 670, height: "auto"}} />}
    </Frame>
  );
};
const Outcome = () => (
  <Frame label="Observed post-state">
    <div style={{position: "absolute", left: 90, top: 245}}>
      <h1 style={{fontSize: 87, fontWeight: 640, letterSpacing: "-0.045em", margin: 0}}>Three matching file hashes.<br /><span style={{color: colors.mint}}>{evidence.native_apply.independent_acceptance.post_apply_passed} independent checks.</span></h1>
      <p style={{fontSize: 29, lineHeight: 1.45, color: colors.muted, marginTop: 30}}>{evidence.native_apply.post_apply_tests_passed} repository tests also passed. Applied bytes matched the package; the receipt survived restart.</p>
    </div>
    <Img src={staticFile("product/astra-ci-native-journal.png")} style={{position: "absolute", left: 90, top: 610, width: 1500, height: "auto"}} />
    <div style={{position: "absolute", left: 90, bottom: 105, fontSize: 25, color: colors.muted}}>One run. No comparative speed claim. No OS-wide sandbox guarantee.</div>
  </Frame>
);
const End = () => (
  <Frame label="Pytxo Beta source candidate">
    <div style={{position: "absolute", left: 90, top: 330, fontSize: 106, fontWeight: 640, lineHeight: 1.1, letterSpacing: "-0.05em"}}>
      Keep your agent.<br /><span style={{color: colors.mint}}>Add Pytxo.</span>
      <div style={{marginTop: 48, fontSize: 35, color: colors.muted, letterSpacing: "-0.01em"}}>pytxo.com · Evidence and limitations in DEMO.md</div>
    </div>
  </Frame>
);
const GlobalAudio = () => (
  <>
    <Audio src={staticFile("audio/narration/pytxo-demo-narration.mp3")} volume={1} />
    <Audio src={staticFile("audio/music/modern-chillout-future-calm.mp3")} loop loopVolumeCurveBehavior="extend"
      volume={(frame) => interpolate(frame, [0, FPS, TOTAL_FRAMES - FPS, TOTAL_FRAMES], [0, 0.1, 0.1, 0], clamp)} />
  </>
);
export const PytxoLaunchDemo = ({includeAudio}: PytxoLaunchDemoProps) => (
  <AbsoluteFill style={{backgroundColor: colors.canvas}}>
    <Sequence from={0} durationInFrames={4 * FPS} name="00–04 · One harness"><Intro /></Sequence>
    <Sequence from={4 * FPS} durationInFrames={9 * FPS} name="04–13 · Recorded mission"><Mission /></Sequence>
    <Sequence from={13 * FPS} durationInFrames={14 * FPS} name="13–27 · Candidate receipt"><Review /></Sequence>
    <Sequence from={27 * FPS} durationInFrames={12 * FPS} name="27–39 · Confirmation and Apply"><Apply /></Sequence>
    <Sequence from={39 * FPS} durationInFrames={8 * FPS} name="39–47 · Post-state"><Outcome /></Sequence>
    <Sequence from={47 * FPS} durationInFrames={5 * FPS} name="47–52 · End"><End /></Sequence>
    {includeAudio ? <GlobalAudio /> : null}
  </AbsoluteFill>
);
