import type {ReactNode} from "react";
import {
  AbsoluteFill,
  Img,
  Sequence,
  interpolate,
  spring,
  staticFile,
  useCurrentFrame,
  useVideoConfig,
} from "remotion";

export const R3_STORYBOARD_FPS = 30;
export const R3_STORYBOARD_FRAMES = 24 * R3_STORYBOARD_FPS;

type PrivacyMask = {
  x: number;
  y: number;
  width: number;
  height: number;
  label: string;
};

type StoryboardAsset = {
  key: "running" | "completed" | "review" | "applied";
  path: string;
  sha256: string;
  width: number;
  height: number;
  privacyMasks?: PrivacyMask[];
};

export type PytxoR3StoryboardProps = {
  schemaVersion: 1;
  kind: "r3-native-still-storyboard";
  candidate: {
    version: string;
    msiSha256: string;
    executableSha256: string;
    runId: string;
    packageDigest: string;
  };
  evidenceBoundary: {
    continuousFootage: false;
    pointerMotion: false;
    native4kSource: false;
    label: string;
  };
  assets: StoryboardAsset[];
};

export const parseR3StoryboardProps = (input: unknown): PytxoR3StoryboardProps => {
  if (!input || typeof input !== "object") throw new Error("R3 storyboard props are missing");
  const candidate = input as Partial<PytxoR3StoryboardProps>;
  if (
    candidate.schemaVersion !== 1
    || candidate.kind !== "r3-native-still-storyboard"
    || !candidate.candidate
    || !candidate.evidenceBoundary
    || !Array.isArray(candidate.assets)
  ) {
    throw new Error("R3 storyboard props do not match schema version 1");
  }
  return candidate as PytxoR3StoryboardProps;
};

const palette = {
  canvas: "#05070a",
  panel: "#0a0d11",
  line: "#20262e",
  text: "#f7f8fa",
  muted: "#9aa6b5",
  cyan: "#21d8d2",
  violet: "#8e5cff",
  amber: "#ffb45f",
  green: "#51d89b",
};
const clamp = {extrapolateLeft: "clamp" as const, extrapolateRight: "clamp" as const};

const Fade = ({children, duration}: {children: ReactNode; duration: number}) => {
  const frame = useCurrentFrame();
  const opacity = interpolate(frame, [0, 10, duration - 10, duration], [0, 1, 1, 0], clamp);
  return <AbsoluteFill style={{opacity}}>{children}</AbsoluteFill>;
};

const Brand = () => (
  <div style={{display: "flex", alignItems: "center", gap: 13, fontSize: 25, fontWeight: 650}}>
    <Img src={staticFile("logo-mark.png")} style={{width: 28, height: 28}} />
    Pytxo
  </div>
);

const StoryboardShell = ({children, props}: {children: ReactNode; props: PytxoR3StoryboardProps}) => (
  <AbsoluteFill
    style={{
      background: `radial-gradient(circle at 78% 18%, rgba(33,216,210,0.07), transparent 27%), ${palette.canvas}`,
      color: palette.text,
      fontFamily: "Geist Variable, Geist, sans-serif",
      overflow: "hidden",
    }}
  >
    <div
      style={{
        position: "absolute",
        inset: 0,
        opacity: 0.18,
        backgroundImage: "linear-gradient(rgba(255,255,255,0.025) 1px, transparent 1px), linear-gradient(90deg, rgba(255,255,255,0.025) 1px, transparent 1px)",
        backgroundSize: "48px 48px",
      }}
    />
    <div style={{position: "absolute", left: 70, top: 42}}><Brand /></div>
    <div style={{position: "absolute", left: 0, right: 0, top: 0, height: 2, background: "linear-gradient(90deg,#ff3da6,#ffc857,#46dc91,#18cce5,#8a52ff)"}} />
    <div style={{position: "absolute", right: 70, top: 50, color: palette.muted, fontSize: 16, letterSpacing: "0.08em", textTransform: "uppercase"}}>
      {props.evidenceBoundary.label}
    </div>
    {children}
  </AbsoluteFill>
);

const AsciiAperture = () => {
  const frame = useCurrentFrame();
  const {fps} = useVideoConfig();
  const tick = Math.floor((frame * 12) / fps);
  const chars = [".", ":", "+", "*", "=", "x"];
  const colors = [palette.cyan, "#55e1a8", palette.amber, "#f45ebc", palette.violet];
  return (
    <div style={{position: "relative", width: 330, height: 330}}>
      {Array.from({length: 72}, (_, index) => {
        const theta = (index / 72) * Math.PI * 2 + tick * 0.011;
        const ring = index % 3;
        const radius = 92 + ring * 35 + Math.sin(index * 1.7 + tick * 0.16) * 8;
        return (
          <span
            key={index}
            style={{
              position: "absolute",
              left: 165 + Math.cos(theta) * radius,
              top: 165 + Math.sin(theta) * radius * 0.82,
              color: colors[(index + tick) % colors.length],
              fontFamily: "ui-monospace, SFMono-Regular, Consolas, monospace",
              fontSize: 19,
              opacity: 0.38 + ((index + tick) % 7) / 12,
              transform: "translate(-50%, -50%)",
            }}
          >
            {chars[(index + tick) % chars.length]}
          </span>
        );
      })}
      <Img src={staticFile("logo-mark.png")} style={{position: "absolute", left: 139, top: 139, width: 52, height: 52}} />
    </div>
  );
};

const Intro = ({props}: {props: PytxoR3StoryboardProps}) => {
  const frame = useCurrentFrame();
  const {fps} = useVideoConfig();
  const entrance = spring({frame, fps, config: {damping: 18, stiffness: 110, mass: 0.8}});
  return (
    <Fade duration={3 * R3_STORYBOARD_FPS}>
      <StoryboardShell props={props}>
        <div style={{position: "absolute", left: 100, top: 250, transform: `translateY(${(1 - entrance) * 24}px)`, opacity: entrance}}>
          <div style={{fontSize: 24, color: palette.cyan, letterSpacing: "0.11em", textTransform: "uppercase"}}>One observed R3 mission</div>
          <div style={{fontSize: 91, lineHeight: 1.02, letterSpacing: "-0.055em", fontWeight: 640, marginTop: 25}}>
            Agents do the work.<br />You decide what<br /><span style={{color: palette.cyan}}>becomes real.</span>
          </div>
        </div>
        <div style={{position: "absolute", right: 150, top: 315}}><AsciiAperture /></div>
        <div style={{position: "absolute", left: 100, bottom: 66, color: palette.muted, fontSize: 20}}>
          Exact native stills · no continuous-motion claim
        </div>
      </StoryboardShell>
    </Fade>
  );
};

const Chips = ({items}: {items: string[]}) => (
  <div style={{display: "flex", flexWrap: "wrap", gap: 9, marginTop: 36}}>
    {items.map((item) => (
      <div key={item} style={{padding: "8px 12px", border: `1px solid ${palette.line}`, borderRadius: 6, color: palette.muted, fontSize: 17, background: "rgba(10,13,17,0.82)"}}>
        {item}
      </div>
    ))}
  </div>
);

const PrivacyMask = ({mask, asset}: {mask: PrivacyMask; asset: StoryboardAsset}) => (
  <div
    style={{
      position: "absolute",
      left: `${(mask.x / asset.width) * 100}%`,
      top: `${(mask.y / asset.height) * 100}%`,
      width: `${(mask.width / asset.width) * 100}%`,
      height: `${(mask.height / asset.height) * 100}%`,
      background: "linear-gradient(90deg, rgba(5,7,10,0.98), rgba(12,16,21,0.98))",
      border: `1px solid ${palette.line}`,
      color: palette.muted,
      display: "flex",
      alignItems: "center",
      paddingLeft: 10,
      fontFamily: "ui-monospace, SFMono-Regular, Consolas, monospace",
      fontSize: 11,
      letterSpacing: "0.035em",
      overflow: "hidden",
      whiteSpace: "nowrap",
    }}
  >
    {mask.label}
  </div>
);

const ProgressRail = ({active}: {active: number}) => {
  const labels = ["Execute", "Candidate", "Review", "Apply"];
  return (
    <div style={{position: "absolute", left: 70, right: 70, bottom: 35, display: "grid", gridTemplateColumns: "repeat(4,1fr)", gap: 8}}>
      {labels.map((label, index) => (
        <div key={label} style={{borderTop: `2px solid ${index <= active ? palette.cyan : palette.line}`, paddingTop: 9, color: index === active ? palette.text : palette.muted, fontSize: 14, letterSpacing: "0.08em", textTransform: "uppercase"}}>
          0{index + 1} · {label}
        </div>
      ))}
    </div>
  );
};

type NativeSceneProps = {
  props: PytxoR3StoryboardProps;
  assetKey: StoryboardAsset["key"];
  eyebrow: string;
  title: ReactNode;
  body: ReactNode;
  chips: string[];
  active: number;
  duration: number;
};

const NativeScene = ({props, assetKey, eyebrow, title, body, chips, active, duration}: NativeSceneProps) => {
  const frame = useCurrentFrame();
  const asset = props.assets.find((candidate) => candidate.key === assetKey);
  if (!asset) throw new Error(`Missing R3 storyboard asset: ${assetKey}`);
  const zoom = interpolate(frame, [0, duration], [1, 1.012], clamp);
  const imageOpacity = interpolate(frame, [0, 12], [0.6, 1], clamp);
  return (
    <Fade duration={duration}>
      <StoryboardShell props={props}>
        <div style={{position: "absolute", left: 72, top: 210, width: 420}}>
          <div style={{color: palette.cyan, fontSize: 18, letterSpacing: "0.11em", textTransform: "uppercase"}}>{eyebrow}</div>
          <h1 style={{fontSize: 57, lineHeight: 1.04, letterSpacing: "-0.045em", margin: "23px 0 0", fontWeight: 620}}>{title}</h1>
          <div style={{fontSize: 23, lineHeight: 1.45, color: palette.muted, marginTop: 27}}>{body}</div>
          <Chips items={chips} />
          <div style={{marginTop: 45, color: palette.muted, fontSize: 15, lineHeight: 1.45}}>
            Native still · {asset.sha256.slice(0, 12)}<br />No elapsed-time or pointer-motion claim
          </div>
        </div>
        <div
          style={{
            position: "absolute",
            left: 558,
            top: 139,
            width: 1282,
            height: 802,
            border: `1px solid ${palette.line}`,
            borderRadius: 9,
            overflow: "hidden",
            background: palette.panel,
            boxShadow: "0 34px 90px rgba(0,0,0,0.48)",
            opacity: imageOpacity,
          }}
        >
          <div
            style={{
              position: "absolute",
              inset: 0,
              transform: `scale(${zoom})`,
              transformOrigin: "50% 50%",
            }}
          >
            <Img
              src={staticFile(asset.path)}
              style={{width: "100%", height: "100%", objectFit: "contain"}}
            />
            {(asset.privacyMasks ?? []).map((mask, index) => <PrivacyMask key={`${asset.key}-${index}`} mask={mask} asset={asset} />)}
          </div>
        </div>
        <ProgressRail active={active} />
      </StoryboardShell>
    </Fade>
  );
};

const End = ({props}: {props: PytxoR3StoryboardProps}) => (
  <Fade duration={3 * R3_STORYBOARD_FPS}>
    <StoryboardShell props={props}>
      <div style={{position: "absolute", left: 100, top: 285}}>
        <div style={{fontSize: 92, lineHeight: 1.04, letterSpacing: "-0.055em", fontWeight: 640}}>
          Keep your agent.<br /><span style={{color: palette.cyan}}>Add a commit boundary.</span>
        </div>
        <div style={{fontSize: 27, color: palette.muted, marginTop: 36}}>One real Codex run · three reviewed files · seven independent checks</div>
        <div style={{fontFamily: "ui-monospace, SFMono-Regular, Consolas, monospace", fontSize: 17, color: palette.muted, marginTop: 50}}>
          R3 {props.candidate.executableSha256.slice(0, 12)} · run {props.candidate.runId.slice(0, 8)} · package {props.candidate.packageDigest.slice(0, 12)}
        </div>
      </div>
      <div style={{position: "absolute", right: 150, top: 355}}><AsciiAperture /></div>
    </StoryboardShell>
  </Fade>
);

export const PytxoR3Storyboard = (props: PytxoR3StoryboardProps) => (
  <AbsoluteFill style={{backgroundColor: palette.canvas}}>
    <Sequence from={0} durationInFrames={3 * R3_STORYBOARD_FPS} name="00–03 · Thesis"><Intro props={props} /></Sequence>
    <Sequence from={78} durationInFrames={6 * R3_STORYBOARD_FPS} name="02.6–08.6 · Execute">
      <NativeScene props={props} assetKey="running" eyebrow="01 / Execute" title={<>One real worker.<br />One isolated run.</>} body={<>Codex works in a private overlay. The canonical repository stays unchanged while the worker runs.</>} chips={["Orbit", "PTY", "1 worker"]} active={0} duration={6 * R3_STORYBOARD_FPS} />
    </Sequence>
    <Sequence from={246} durationInFrames={4 * R3_STORYBOARD_FPS} name="08.2–12.2 · Candidate">
      <NativeScene props={props} assetKey="completed" eyebrow="02 / Candidate" title={<>The work settles.<br />The checks do not.</>} body={<>Pytxo runs its own bounded verifier before Review becomes available.</>} chips={["Exit 0", "7 / 7 checks", "Candidate v3"]} active={1} duration={4 * R3_STORYBOARD_FPS} />
    </Sequence>
    <Sequence from={354} durationInFrames={6 * R3_STORYBOARD_FPS} name="11.8–17.8 · Review">
      <NativeScene props={props} assetKey="review" eyebrow="03 / Review" title={<>Inspect the<br />exact bytes.</>} body={<>Three prepared files. Combined checks passed. The canonical destination remains read-only.</>} chips={["3 files", "Digest bound", "Human decision"]} active={2} duration={6 * R3_STORYBOARD_FPS} />
    </Sequence>
    <Sequence from={522} durationInFrames={4 * R3_STORYBOARD_FPS} name="17.4–21.4 · Apply">
      <NativeScene props={props} assetKey="applied" eyebrow="04 / Apply" title={<>One explicit<br />decision.</>} body={<>The reviewed package commits atomically and the durable receipt records the result.</>} chips={["Committed", "Apply recorded", "7 / 7 post-Apply"]} active={3} duration={4 * R3_STORYBOARD_FPS} />
    </Sequence>
    <Sequence from={630} durationInFrames={3 * R3_STORYBOARD_FPS} name="21–24 · End"><End props={props} /></Sequence>
  </AbsoluteFill>
);
