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

export const R6_COCKPIT_FPS = 30;
export const R6_COCKPIT_FRAMES = 12 * R6_COCKPIT_FPS;

type CockpitAsset = {
  key: "first-use" | "dense-canvas";
  path: string;
  sha256: string;
  width: number;
  height: number;
};

export type PytxoR6CockpitProofProps = {
  schemaVersion: 1;
  kind: "r6-native-still-cockpit-proof";
  candidate: {
    version: string;
    msiSha256: string;
    executableSha256: string;
    candidateReceiptSha256: string;
    stateReceiptSha256: string;
  };
  evidenceBoundary: {
    continuousFootage: false;
    pointerMotion: false;
    native4kSource: false;
    windowsScalePercent: number;
    windowDpi: number;
    label: string;
  };
  observed: {
    workers: number;
    waves: number;
    fitPercent: number;
    minimapVisible: boolean;
    redundantCommitRailAbsent: boolean;
  };
  assets: CockpitAsset[];
};

export const parseR6CockpitProofProps = (input: unknown): PytxoR6CockpitProofProps => {
  if (!input || typeof input !== "object") throw new Error("R6 cockpit proof props are missing");
  const candidate = input as Partial<PytxoR6CockpitProofProps>;
  if (
    candidate.schemaVersion !== 1
    || candidate.kind !== "r6-native-still-cockpit-proof"
    || !candidate.candidate
    || !candidate.evidenceBoundary
    || !candidate.observed
    || !Array.isArray(candidate.assets)
  ) {
    throw new Error("R6 cockpit proof props do not match schema version 1");
  }
  return candidate as PytxoR6CockpitProofProps;
};

const color = {
  canvas: "#050608",
  panel: "#0b0d11",
  line: "#252932",
  text: "#f7f7f8",
  muted: "#9ca4b2",
  cyan: "#44d7d0",
  green: "#62d89b",
  violet: "#9b73ff",
  amber: "#e6b066",
  magenta: "#ed5eb7",
};
const clamp = {extrapolateLeft: "clamp" as const, extrapolateRight: "clamp" as const};

const Brand = () => (
  <div style={{display: "flex", alignItems: "center", gap: 12, fontSize: 23, fontWeight: 620}}>
    <Img src={staticFile("logo-mark.png")} style={{width: 27, height: 27}} />
    Pytxo
  </div>
);

const Shell = ({children, props}: {children: ReactNode; props: PytxoR6CockpitProofProps}) => (
  <AbsoluteFill
    style={{
      background: `radial-gradient(circle at 82% 18%, rgba(68,215,208,0.07), transparent 28%), ${color.canvas}`,
      color: color.text,
      fontFamily: "Geist Variable, Geist, sans-serif",
      overflow: "hidden",
    }}
  >
    <div
      style={{
        position: "absolute",
        inset: 0,
        backgroundImage:
          "linear-gradient(rgba(255,255,255,0.018) 1px, transparent 1px), linear-gradient(90deg, rgba(255,255,255,0.018) 1px, transparent 1px)",
        backgroundSize: "48px 48px",
      }}
    />
    <div style={{position: "absolute", top: 0, left: 0, right: 0, height: 2, background: "linear-gradient(90deg,#f153a9,#e5b25d,#54d59a,#31d4d2,#9b73ff)"}} />
    <div style={{position: "absolute", left: 68, top: 38}}><Brand /></div>
    <div
      style={{
        position: "absolute",
        right: 68,
        top: 44,
        color: color.muted,
        fontFamily: "ui-monospace, SFMono-Regular, Consolas, monospace",
        fontSize: 13,
        letterSpacing: "0.045em",
      }}
    >
      {props.evidenceBoundary.label}
    </div>
    {children}
  </AbsoluteFill>
);

const Fade = ({children, duration}: {children: ReactNode; duration: number}) => {
  const frame = useCurrentFrame();
  const opacity = interpolate(frame, [0, 12, duration - 12, duration], [0, 1, 1, 0], clamp);
  return <AbsoluteFill style={{opacity}}>{children}</AbsoluteFill>;
};

const AsciiAperture = () => {
  const frame = useCurrentFrame();
  const {fps} = useVideoConfig();
  const tick = Math.floor((frame * 12) / fps);
  const chars = [".", ":", "+", "*", "=", "x"];
  const colors = [color.cyan, color.green, color.amber, color.magenta, color.violet];
  return (
    <div style={{position: "relative", width: 310, height: 310}}>
      {Array.from({length: 60}, (_, index) => {
        const ring = index % 3;
        const angle = (index / 60) * Math.PI * 2 + tick * 0.009;
        const radius = 76 + ring * 32 + Math.sin(index * 1.4 + tick * 0.12) * 7;
        return (
          <span
            key={index}
            style={{
              position: "absolute",
              left: 155 + Math.cos(angle) * radius,
              top: 155 + Math.sin(angle) * radius * 0.84,
              color: colors[(index + tick) % colors.length],
              fontFamily: "ui-monospace, SFMono-Regular, Consolas, monospace",
              fontSize: 18,
              opacity: 0.3 + ((index + tick) % 8) / 12,
              transform: "translate(-50%, -50%)",
            }}
          >
            {chars[(index + tick) % chars.length]}
          </span>
        );
      })}
      <Img
        src={staticFile("logo-mark.png")}
        style={{position: "absolute", left: 132, top: 132, width: 46, height: 46}}
      />
    </div>
  );
};

const NativeStill = ({
  asset,
  width,
  height,
  frame,
}: {
  asset: CockpitAsset;
  width: number;
  height: number;
  frame: number;
}) => {
  const zoom = interpolate(frame, [0, 140], [1, 1.008], clamp);
  return (
    <div
      style={{
        width,
        height,
        position: "relative",
        overflow: "hidden",
        border: `1px solid ${color.line}`,
        borderRadius: 11,
        background: color.panel,
        boxShadow: "0 30px 90px rgba(0,0,0,0.5)",
      }}
    >
      <Img
        src={staticFile(asset.path)}
        style={{
          width: "100%",
          height: "100%",
          objectFit: "contain",
          transform: `scale(${zoom})`,
          transformOrigin: "50% 50%",
        }}
      />
    </div>
  );
};

const FirstUseScene = ({props}: {props: PytxoR6CockpitProofProps}) => {
  const frame = useCurrentFrame();
  const {fps} = useVideoConfig();
  const asset = props.assets.find((item) => item.key === "first-use");
  if (!asset) throw new Error("Missing R6 first-use asset");
  const entrance = spring({frame, fps, config: {damping: 19, stiffness: 105, mass: 0.78}});
  return (
    <Fade duration={112}>
      <Shell props={props}>
        <div style={{position: "absolute", left: 78, top: 160, width: 500, opacity: entrance, transform: `translateY(${(1 - entrance) * 22}px)`}}>
          <div style={{color: color.cyan, fontSize: 17, letterSpacing: "0.12em", textTransform: "uppercase"}}>First use</div>
          <h1 style={{fontSize: 72, lineHeight: 1.02, letterSpacing: "-0.05em", fontWeight: 640, margin: "24px 0 0"}}>
            Your agents.<br />One clear place<br /><span style={{color: color.cyan}}>to work.</span>
          </h1>
          <div style={{marginTop: 30, color: color.muted, fontSize: 21, lineHeight: 1.45}}>
            Connect an agent. Choose a project.<br />Review every change.
          </div>
          <div style={{position: "absolute", left: 40, top: 520, opacity: 0.72}}><AsciiAperture /></div>
        </div>
        <div style={{position: "absolute", right: 70, top: 160}}>
          <NativeStill asset={asset} width={1130} height={707} frame={frame} />
        </div>
        <div style={{position: "absolute", right: 70, bottom: 52, color: color.muted, fontFamily: "ui-monospace, SFMono-Regular, Consolas, monospace", fontSize: 13}}>
          Native 125% scale · SHA {asset.sha256.slice(0, 12)}
        </div>
      </Shell>
    </Fade>
  );
};

const CanvasScene = ({props}: {props: PytxoR6CockpitProofProps}) => {
  const frame = useCurrentFrame();
  const asset = props.assets.find((item) => item.key === "dense-canvas");
  if (!asset) throw new Error("Missing R6 dense-canvas asset");
  return (
    <Fade duration={198}>
      <Shell props={props}>
        <div style={{position: "absolute", left: 68, top: 190, width: 310}}>
          <div style={{color: color.cyan, fontSize: 17, letterSpacing: "0.12em", textTransform: "uppercase"}}>Recorded execution</div>
          <h2 style={{fontSize: 52, lineHeight: 1.02, letterSpacing: "-0.045em", fontWeight: 620, margin: "19px 0 0"}}>
            The whole run. One focused canvas.
          </h2>
          <div style={{display: "grid", gridTemplateColumns: "1fr 1fr", gap: 9, marginTop: 34}}>
            {[`${props.observed.workers} workers`, `${props.observed.waves} waves`, `${props.observed.fitPercent}% Fit`, "minimap"].map((item) => (
              <div key={item} style={{border: `1px solid ${color.line}`, borderRadius: 6, background: "rgba(11,13,17,0.86)", padding: "10px 11px", color: color.muted, fontSize: 15}}>{item}</div>
            ))}
          </div>
          <div style={{marginTop: 34, color: color.muted, fontSize: 17, lineHeight: 1.5}}>
            Read-only topology.<br />Dependency edges only.<br />Review stays explicit.
          </div>
        </div>
        <div style={{position: "absolute", left: 420, top: 140}}>
          <NativeStill asset={asset} width={1430} height={895} frame={frame} />
        </div>
        <div
          style={{
            position: "absolute",
            left: 420,
            bottom: 30,
            padding: "13px 17px",
            border: `1px solid ${color.line}`,
            borderRadius: 7,
            background: "rgba(5,6,8,0.9)",
            color: color.muted,
            fontSize: 16,
          }}
        >
          Native 125% scale · SHA {asset.sha256.slice(0, 12)}
        </div>
      </Shell>
    </Fade>
  );
};

const CloseScene = ({props}: {props: PytxoR6CockpitProofProps}) => {
  const frame = useCurrentFrame();
  const {fps} = useVideoConfig();
  const entrance = spring({frame, fps, config: {damping: 20, stiffness: 105, mass: 0.82}});
  return (
    <Fade duration={78}>
      <Shell props={props}>
        <div style={{position: "absolute", left: 102, top: 242, opacity: entrance, transform: `translateY(${(1 - entrance) * 20}px)`}}>
          <div style={{color: color.cyan, fontSize: 18, letterSpacing: "0.12em", textTransform: "uppercase"}}>Pytxo Desktop</div>
          <div style={{fontSize: 88, lineHeight: 1.03, letterSpacing: "-0.055em", fontWeight: 640, marginTop: 27}}>
            See the work.<br />Review exact changes.<br /><span style={{color: color.cyan}}>Then decide what to save.</span>
          </div>
          <div style={{display: "flex", gap: 18, marginTop: 50, color: color.muted, fontSize: 20}}>
            <span>Recorded workers</span><span style={{color: color.line}}>•</span><span>Explicit review</span><span style={{color: color.line}}>•</span><span>Repository boundary</span>
          </div>
        </div>
        <div style={{position: "absolute", right: 120, top: 360, opacity: 0.82}}><AsciiAperture /></div>
        <div style={{position: "absolute", left: 102, bottom: 58, color: color.muted, fontFamily: "ui-monospace, SFMono-Regular, Consolas, monospace", fontSize: 13}}>
          R6 candidate {props.candidate.version} · MSI {props.candidate.msiSha256.slice(0, 12)} · no continuous-footage claim
        </div>
      </Shell>
    </Fade>
  );
};

export const PytxoR6CockpitProof = (props: PytxoR6CockpitProofProps) => (
  <AbsoluteFill style={{backgroundColor: color.canvas}}>
    <Sequence from={0} durationInFrames={112} premountFor={30}>
      <FirstUseScene props={props} />
    </Sequence>
    <Sequence from={96} durationInFrames={198} premountFor={30}>
      <CanvasScene props={props} />
    </Sequence>
    <Sequence from={282} durationInFrames={78} premountFor={30}>
      <CloseScene props={props} />
    </Sequence>
  </AbsoluteFill>
);
