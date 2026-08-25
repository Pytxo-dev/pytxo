import type {ReactNode} from "react";
import {Audio} from "@remotion/media";
import {
  AbsoluteFill,
  Easing,
  Img,
  Sequence,
  interpolate,
  staticFile,
  useCurrentFrame,
} from "remotion";

export type PytxoLaunchDemoProps = {
  includeAudio: boolean;
};

export const FPS = 30;
export const TOTAL_FRAMES = 52 * FPS;

const FLOW_START = 4 * FPS;
const REVIEW_START = 13 * FPS;
const APPLY_START = 27 * FPS;
const OPERATIONS_START = 39 * FPS;
const END_START = 47 * FPS;
const CONTEXT_DISSOLVE_FRAMES = 8;

const colors = {
  canvas: "#050608",
  text: "#f5f7fa",
  muted: "#a5afb9",
  mint: "#38d6bd",
};

const clamp = {
  extrapolateLeft: "clamp" as const,
  extrapolateRight: "clamp" as const,
};
const ease = Easing.bezier(0.16, 1, 0.3, 1);

const ProductShot = ({
  src,
  cropAt,
  cropTo = 1,
  origin = "50% 50%",
}: {
  src: string;
  cropAt?: number;
  cropTo?: number;
  origin?: string;
}) => {
  const frame = useCurrentFrame();
  return (
    <AbsoluteFill style={{backgroundColor: colors.canvas, overflow: "hidden"}}>
      <Img
        src={staticFile(src)}
        style={{
          width: "100%",
          height: "100%",
          objectFit: "cover",
          transformOrigin: origin,
          scale:
            cropAt === undefined
              ? cropTo
              : interpolate(frame, [cropAt, cropAt + 20], [1, cropTo], {
                  ...clamp,
                  easing: ease,
                  output: "perceptual-scale",
                }),
        }}
      />
    </AbsoluteFill>
  );
};

const Logo = () => (
  <div
    style={{
      display: "flex",
      alignItems: "center",
      gap: 18,
      color: colors.text,
      fontSize: 36,
      fontWeight: 650,
      letterSpacing: "-0.035em",
    }}
  >
    <Img
      src={staticFile("logo-mark.png")}
      style={{width: 42, height: 42, objectFit: "contain"}}
    />
    Pytxo
  </div>
);

const SceneCopy = ({
  children,
  detail,
  position = "left",
}: {
  children: ReactNode;
  detail?: ReactNode;
  position?: "left" | "right";
}) => {
  const frame = useCurrentFrame();
  return (
    <div
      style={{
        position: "absolute",
        zIndex: 4,
        left: position === "left" ? 304 : undefined,
        right: position === "right" ? 82 : undefined,
        bottom: 62,
        maxWidth: 1050,
        color: colors.text,
        opacity: interpolate(frame, [8, 20], [0, 1], {
          ...clamp,
          easing: ease,
        }),
        translate: `0 ${interpolate(frame, [8, 20], [18, 0], {
          ...clamp,
          easing: ease,
        })}px`,
        textAlign: position,
        textShadow: "0 3px 18px rgba(0,0,0,.98), 0 1px 2px #000",
      }}
    >
      <div
        style={{
          display: "inline-flex",
          alignItems: "center",
          gap: 14,
          fontSize: 47,
          lineHeight: 1.08,
          fontWeight: 660,
          letterSpacing: "-0.038em",
        }}
      >
        <span
          style={{
            display: "inline-block",
            width: 32,
            height: 3,
            backgroundColor: colors.mint,
          }}
        />
        {children}
      </div>
      {detail ? (
        <div
          style={{
            marginTop: 13,
            color: colors.muted,
            fontSize: 25,
            lineHeight: 1.3,
            fontWeight: 540,
          }}
        >
          {detail}
        </div>
      ) : null}
    </div>
  );
};

const Intro = () => {
  const frame = useCurrentFrame();
  return (
    <AbsoluteFill
      style={{
        backgroundColor: colors.canvas,
        color: colors.text,
        fontFamily: "Geist Variable, Geist, sans-serif",
        opacity: interpolate(
          frame,
          [FLOW_START - CONTEXT_DISSOLVE_FRAMES, FLOW_START],
          [1, 0],
          clamp,
        ),
      }}
    >
      <ProductShot src="product/flow-plan-1920x1080.png" cropTo={1.04} />
      <AbsoluteFill style={{backgroundColor: "rgba(2,4,6,.82)"}} />
      <div
        style={{
          position: "absolute",
          left: 144,
          top: 122,
          opacity: interpolate(frame, [0, 14], [0, 1], {
            ...clamp,
            easing: ease,
          }),
        }}
      >
        <Logo />
      </div>
      <div
        style={{
          position: "absolute",
          left: 144,
          right: 144,
          top: 322,
          color: colors.text,
          fontSize: 100,
          lineHeight: 0.98,
          fontWeight: 680,
          letterSpacing: "-0.062em",
          opacity: interpolate(frame, [6, 24], [0, 1], {
            ...clamp,
            easing: ease,
          }),
          translate: `0 ${interpolate(frame, [6, 24], [24, 0], {
            ...clamp,
            easing: ease,
          })}px`,
          textShadow: "0 4px 30px rgba(0,0,0,.9)",
        }}
      >
        One mission.
        <br />
        Multiple agents.
        <br />
        <span style={{color: colors.mint}}>One reviewed result.</span>
      </div>
    </AbsoluteFill>
  );
};

const Flow = () => {
  const frame = useCurrentFrame();
  return (
    <AbsoluteFill
      style={{
        fontFamily: "Geist Variable, Geist, sans-serif",
        opacity: interpolate(frame, [0, CONTEXT_DISSOLVE_FRAMES - 1], [0, 1], clamp),
      }}
    >
      <ProductShot
        src="product/flow-plan-1920x1080.png"
        cropAt={3 * FPS}
        cropTo={1.14}
        origin="74% 50%"
      />
      <SceneCopy>Plan ownership before agents run.</SceneCopy>
    </AbsoluteFill>
  );
};

const Review = () => (
  <AbsoluteFill style={{fontFamily: "Geist Variable, Geist, sans-serif"}}>
    <ProductShot
      src="product/run-review-ready-1920x1080.png"
      cropAt={3 * FPS}
      cropTo={1.12}
      origin="70% 40%"
    />
    <SceneCopy>Review what Pytxo will enforce.</SceneCopy>
  </AbsoluteFill>
);

const Cursor = () => {
  const frame = useCurrentFrame();
  const clickProgress = interpolate(frame, [98, 104, 110], [1, 0.84, 1], clamp);
  return (
    <>
      <div
        style={{
          position: "absolute",
          zIndex: 7,
          left: interpolate(frame, [48, 96], [1150, 1532], {
            ...clamp,
            easing: ease,
          }),
          top: interpolate(frame, [48, 96], [300, 91], {
            ...clamp,
            easing: ease,
          }),
          opacity: interpolate(frame, [42, 52, 112, 118], [0, 1, 1, 0], clamp),
          scale: clickProgress,
          filter: "drop-shadow(0 3px 5px rgba(0,0,0,.75))",
        }}
      >
        <svg width="42" height="52" viewBox="0 0 42 52" aria-hidden="true">
          <path
            d="M4 3L36 29H22L30 47L21 51L13 32L4 41V3Z"
            fill="#f8fafc"
            stroke="#060708"
            strokeWidth="3"
            strokeLinejoin="round"
          />
        </svg>
      </div>
      <div
        style={{
          position: "absolute",
          zIndex: 6,
          left: 1494,
          top: 65,
          width: 86,
          height: 86,
          border: `3px solid ${colors.mint}`,
          borderRadius: "50%",
          opacity: interpolate(frame, [100, 104, 114], [0, 0.9, 0], clamp),
          scale: interpolate(frame, [100, 114], [0.55, 1.25], {
            ...clamp,
            easing: ease,
            output: "perceptual-scale",
          }),
        }}
      />
    </>
  );
};

const Apply = () => {
  const frame = useCurrentFrame();
  const applied = frame >= 112;
  return (
    <AbsoluteFill style={{fontFamily: "Geist Variable, Geist, sans-serif"}}>
      <ProductShot
        src={
          applied
            ? "product/run-review-applied-1920x1080.png"
            : "product/run-review-ready-1920x1080.png"
        }
        cropTo={1.12}
        origin="70% 40%"
      />
      <SceneCopy>Apply only the reviewed change set.</SceneCopy>
      {!applied ? <Cursor /> : null}
    </AbsoluteFill>
  );
};

const Operations = () => {
  return (
    <AbsoluteFill style={{fontFamily: "Geist Variable, Geist, sans-serif"}}>
      <ProductShot
        src="product/operations-1920x1080.png"
        cropAt={2 * FPS}
        cropTo={1.12}
        origin="75% 54%"
      />
      <SceneCopy detail="Signal Core · structure first">See decisions, not terminal noise.</SceneCopy>
    </AbsoluteFill>
  );
};

const End = () => {
  const frame = useCurrentFrame();
  return (
    <AbsoluteFill
      style={{
        backgroundColor: colors.canvas,
        color: colors.text,
        fontFamily: "Geist Variable, Geist, sans-serif",
        opacity: interpolate(frame, [0, CONTEXT_DISSOLVE_FRAMES - 1], [0, 1], clamp),
      }}
    >
      <ProductShot src="product/operations-1920x1080.png" cropTo={1.04} />
      <AbsoluteFill style={{backgroundColor: "rgba(2,4,6,.84)"}} />
      <div style={{position: "absolute", left: 144, top: 122}}>
        <Logo />
      </div>
      <div
        style={{
          position: "absolute",
          left: 144,
          right: 144,
          top: 358,
          textAlign: "center",
          color: colors.text,
          fontSize: 101,
          lineHeight: 1.02,
          fontWeight: 680,
          letterSpacing: "-0.058em",
          opacity: interpolate(frame, [8, 24], [0, 1], {
            ...clamp,
            easing: ease,
          }),
          translate: `0 ${interpolate(frame, [8, 24], [22, 0], {
            ...clamp,
            easing: ease,
          })}px`,
          textShadow: "0 4px 30px rgba(0,0,0,.9)",
        }}
      >
        Keep your agents. <span style={{color: colors.mint}}>Add Pytxo.</span>
      </div>
      <div
        style={{
          position: "absolute",
          left: 144,
          right: 144,
          top: 576,
          color: colors.muted,
          fontSize: 43,
          fontWeight: 560,
          letterSpacing: "-0.02em",
          textAlign: "center",
          opacity: interpolate(frame, [22, 38], [0, 1], {
            ...clamp,
            easing: ease,
          }),
        }}
      >
        pytxo.com
      </div>
    </AbsoluteFill>
  );
};

const GlobalAudio = () => (
  <>
    <Audio src={staticFile("audio/narration/pytxo-demo-narration.mp3")} volume={1} />
    <Audio
      src={staticFile("audio/music/modern-chillout-future-calm.mp3")}
      loop
      loopVolumeCurveBehavior="extend"
      volume={(frame) =>
        interpolate(
          frame,
          [0, FPS, TOTAL_FRAMES - FPS, TOTAL_FRAMES],
          [0, 0.1, 0.1, 0],
          clamp,
        )
      }
    />
    <Sequence from={11 * FPS} layout="none">
      <Audio src={staticFile("audio/sfx/plan-ready.wav")} volume={0.2} />
    </Sequence>
    <Sequence from={APPLY_START + 104} layout="none">
      <Audio src={staticFile("audio/sfx/apply-click.wav")} volume={0.22} />
    </Sequence>
    <Sequence from={APPLY_START + 112} layout="none">
      <Audio src={staticFile("audio/sfx/applied-confirmation.wav")} volume={0.18} />
    </Sequence>
  </>
);

export const PytxoLaunchDemo = ({includeAudio}: PytxoLaunchDemoProps) => (
  <AbsoluteFill style={{backgroundColor: colors.canvas}}>
    <Sequence from={0} durationInFrames={FLOW_START} name="00–04 · Mission">
      <Intro />
    </Sequence>
    <Sequence
      from={FLOW_START - CONTEXT_DISSOLVE_FRAMES}
      durationInFrames={REVIEW_START - FLOW_START + CONTEXT_DISSOLVE_FRAMES}
      name="04–13 · Flow plan"
    >
      <Flow />
    </Sequence>
    <Sequence
      from={REVIEW_START}
      durationInFrames={APPLY_START - REVIEW_START}
      name="13–27 · Run Review"
    >
      <Review />
    </Sequence>
    <Sequence
      from={APPLY_START}
      durationInFrames={OPERATIONS_START - APPLY_START}
      name="27–39 · Apply"
    >
      <Apply />
    </Sequence>
    <Sequence
      from={OPERATIONS_START}
      durationInFrames={END_START - OPERATIONS_START}
      name="39–47 · Operations"
    >
      <Operations />
    </Sequence>
    <Sequence
      from={END_START - CONTEXT_DISSOLVE_FRAMES}
      durationInFrames={TOTAL_FRAMES - END_START + CONTEXT_DISSOLVE_FRAMES}
      name="47–52 · End"
    >
      <End />
    </Sequence>
    {includeAudio ? <GlobalAudio /> : null}
  </AbsoluteFill>
);
