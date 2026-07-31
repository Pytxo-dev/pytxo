import type {CSSProperties, ReactNode} from "react";
import {Audio} from "@remotion/media";
import {TransitionSeries, linearTiming} from "@remotion/transitions";
import {fade} from "@remotion/transitions/fade";
import {
  AbsoluteFill,
  Easing,
  Img,
  interpolate,
  staticFile,
  useCurrentFrame,
} from "remotion";
import voiceover from "./voiceover.json";

export type PytxoLaunchDemoProps = {
  includeVoiceover: boolean;
};

type SceneProps = {
  duration: number;
  index: number;
};

export const TRANSITION_FRAMES = 12;

const colors = {
  canvas: "#050608",
  panelLifted: "#101318",
  line: "#29313a",
  muted: "#9aa6b5",
  text: "#f4f7fb",
  mint: "#2dd4bf",
  cyan: "#38bdf8",
  amber: "#f5b942",
  magenta: "#f153d2",
};

const ease = Easing.bezier(0.16, 1, 0.3, 1);
const clamp = {
  extrapolateLeft: "clamp" as const,
  extrapolateRight: "clamp" as const,
};

const reveal = (frame: number, delay = 0) =>
  interpolate(frame, [delay, delay + 24], [0, 1], {
    ...clamp,
    easing: ease,
  });

const lift = (frame: number, delay = 0) =>
  interpolate(frame, [delay, delay + 30], [32, 0], {
    ...clamp,
    easing: ease,
  });

const Brand = () => (
  <div
    style={{
      display: "flex",
      alignItems: "center",
      gap: 14,
      color: colors.text,
      fontSize: 26,
      fontWeight: 650,
      letterSpacing: "-0.025em",
    }}
  >
    <Img
      src={staticFile("logo-mark.png")}
      style={{width: 32, height: 32, objectFit: "contain"}}
    />
    Pytxo
  </div>
);

const Backdrop = ({accent = colors.mint}: {accent?: string}) => {
  const frame = useCurrentFrame();

  return (
    <AbsoluteFill style={{backgroundColor: colors.canvas, overflow: "hidden"}}>
      <div
        style={{
          position: "absolute",
          inset: 0,
          backgroundImage:
            "radial-gradient(circle at center, rgba(255,255,255,0.045) 1px, transparent 1px)",
          backgroundSize: "38px 38px",
          opacity: 0.34,
        }}
      />
      <div
        style={{
          position: "absolute",
          width: 1050,
          height: 1050,
          right: -320,
          top: -420,
          borderRadius: "50%",
          background: `radial-gradient(circle, ${accent}28 0%, transparent 68%)`,
          translate: `${interpolate(frame, [0, 300], [0, -55], clamp)}px ${interpolate(
            frame,
            [0, 300],
            [0, 45],
            clamp,
          )}px`,
        }}
      />
      <div
        style={{
          position: "absolute",
          width: 760,
          height: 760,
          left: -370,
          bottom: -460,
          borderRadius: "50%",
          background: `radial-gradient(circle, ${colors.magenta}19 0%, transparent 70%)`,
        }}
      />
    </AbsoluteFill>
  );
};

const SceneChrome = ({
  index,
  label,
  accent = colors.mint,
}: {
  index: number;
  label: string;
  accent?: string;
}) => (
  <>
    <div style={{position: "absolute", top: 54, left: 80}}>
      <Brand />
    </div>
    <div
      style={{
        position: "absolute",
        top: 60,
        right: 80,
        display: "flex",
        alignItems: "center",
        gap: 16,
        color: colors.muted,
        fontSize: 19,
        fontWeight: 620,
        letterSpacing: "0.08em",
        textTransform: "uppercase",
      }}
    >
      <span style={{color: accent}}>
        {String(index + 1).padStart(2, "0")} / 09
      </span>
      <span>{label}</span>
    </div>
  </>
);

const Scene = ({
  children,
  index,
  label,
  accent,
}: {
  children: ReactNode;
  index: number;
  label: string;
  accent?: string;
}) => (
  <AbsoluteFill
    style={{
      backgroundColor: colors.canvas,
      color: colors.text,
      fontFamily: "Geist Variable, Geist, sans-serif",
      overflow: "hidden",
    }}
  >
    <Backdrop accent={accent} />
    <SceneChrome index={index} label={label} accent={accent} />
    {children}
  </AbsoluteFill>
);

const Eyebrow = ({
  children,
  color = colors.mint,
}: {
  children: ReactNode;
  color?: string;
}) => (
  <div
    style={{
      display: "flex",
      alignItems: "center",
      gap: 14,
      color,
      fontSize: 21,
      fontWeight: 680,
      letterSpacing: "0.1em",
      textTransform: "uppercase",
    }}
  >
    <span style={{width: 40, height: 2, backgroundColor: color}} />
    {children}
  </div>
);

const Headline = ({
  children,
  style,
}: {
  children: ReactNode;
  style?: CSSProperties;
}) => (
  <div
    style={{
      maxWidth: 1600,
      fontSize: 92,
      lineHeight: 0.99,
      fontWeight: 680,
      letterSpacing: "-0.055em",
      textWrap: "balance",
      ...style,
    }}
  >
    {children}
  </div>
);

const ProductFrame = ({
  src,
  frame,
  duration,
  delay = 8,
  panFrom = [0, 0],
  panTo = [0, -18],
  zoomFrom = 1,
  zoomTo = 1.035,
  focus,
}: {
  src: string;
  frame: number;
  duration: number;
  delay?: number;
  panFrom?: [number, number];
  panTo?: [number, number];
  zoomFrom?: number;
  zoomTo?: number;
  focus?: {left: number; top: number; width: number; height: number; color?: string};
}) => {
  const progressFrame = Math.max(0, frame - delay);
  const cameraEnd = Math.max(1, duration - delay);

  return (
    <div
      style={{
        position: "relative",
        width: 1540,
        height: 790,
        overflow: "hidden",
        borderRadius: 24,
        border: "1px solid rgba(122, 136, 153, 0.42)",
        backgroundColor: "#050608",
        boxShadow:
          "0 55px 130px rgba(0,0,0,0.58), 0 0 0 1px rgba(255,255,255,0.025)",
        opacity: reveal(frame, delay),
        translate: `0 ${lift(frame, delay)}px`,
      }}
    >
      <Img
        src={staticFile(src)}
        style={{
          position: "absolute",
          left: 0,
          top: 0,
          width: 1540,
          height: 962.5,
          objectFit: "cover",
          objectPosition: "top center",
          scale: interpolate(progressFrame, [0, cameraEnd], [zoomFrom, zoomTo], clamp),
          translate: `${interpolate(
            progressFrame,
            [0, cameraEnd],
            [panFrom[0], panTo[0]],
            clamp,
          )}px ${interpolate(
            progressFrame,
            [0, cameraEnd],
            [panFrom[1], panTo[1]],
            clamp,
          )}px`,
        }}
      />
      {focus ? (
        <div
          style={{
            position: "absolute",
            left: focus.left,
            top: focus.top,
            width: focus.width,
            height: focus.height,
            borderRadius: 15,
            border: `2px solid ${focus.color ?? colors.mint}`,
            boxShadow: `0 0 44px ${focus.color ?? colors.mint}40`,
            opacity: interpolate(
              frame,
              [delay + 34, delay + 52, duration - 36, duration - 18],
              [0, 1, 1, 0],
              {...clamp, easing: ease},
            ),
          }}
        />
      ) : null}
    </div>
  );
};

const ProductSceneLayout = ({
  children,
  frame,
  eyebrow,
  eyebrowColor,
  headline,
  detail,
}: {
  children: ReactNode;
  frame: number;
  eyebrow: string;
  eyebrowColor?: string;
  headline: ReactNode;
  detail?: string;
}) => (
  <div
    style={{
      position: "absolute",
      inset: "132px 80px 54px",
      display: "flex",
      flexDirection: "column",
      alignItems: "center",
      gap: 22,
    }}
  >
    <div
      style={{
        width: 1540,
        display: "flex",
        alignItems: "end",
        justifyContent: "space-between",
        gap: 48,
        opacity: reveal(frame, 2),
        translate: `0 ${lift(frame, 2)}px`,
      }}
    >
      <div style={{display: "flex", flexDirection: "column", gap: 12}}>
        <Eyebrow color={eyebrowColor}>{eyebrow}</Eyebrow>
        <div
          style={{
            fontSize: 58,
            lineHeight: 1.02,
            fontWeight: 670,
            letterSpacing: "-0.045em",
          }}
        >
          {headline}
        </div>
      </div>
      {detail ? (
        <div
          style={{
            maxWidth: 560,
            color: colors.muted,
            fontSize: 25,
            lineHeight: 1.36,
            textAlign: "right",
          }}
        >
          {detail}
        </div>
      ) : null}
    </div>
    {children}
  </div>
);

const HookScene = ({index}: SceneProps) => {
  const frame = useCurrentFrame();

  return (
    <Scene index={index} label="The mission" accent={colors.cyan}>
      <div
        style={{
          position: "absolute",
          inset: 0,
          overflow: "hidden",
          opacity: interpolate(frame, [0, 24], [0, 0.36], clamp),
        }}
      >
        <Img
          src={staticFile("product/flow-1600x1000.png")}
          style={{
            width: 1920,
            height: 1200,
            objectFit: "cover",
            filter: "blur(3px)",
            scale: interpolate(frame, [0, 210], [1.05, 1.12], clamp),
            translate: `${interpolate(frame, [0, 210], [0, -32], clamp)}px ${interpolate(
              frame,
              [0, 210],
              [-20, -55],
              clamp,
            )}px`,
          }}
        />
        <AbsoluteFill
          style={{
            background:
              "linear-gradient(90deg, rgba(5,6,8,0.98) 8%, rgba(5,6,8,0.88) 48%, rgba(5,6,8,0.42) 100%)",
          }}
        />
      </div>
      <div
        style={{
          position: "absolute",
          inset: "190px 110px 100px",
          display: "flex",
          flexDirection: "column",
          justifyContent: "center",
          gap: 34,
          opacity: reveal(frame, 5),
          translate: `0 ${lift(frame, 5)}px`,
        }}
      >
        <Eyebrow color={colors.cyan}>Local agent hypervisor</Eyebrow>
        <Headline>
          One mission.
          <br />
          Several agents.
          <br />
          <span style={{color: colors.mint}}>One controlled result.</span>
        </Headline>
        <div style={{color: colors.muted, fontSize: 31, lineHeight: 1.42}}>
          Plan · isolate · schedule · verify · approve
        </div>
      </div>
    </Scene>
  );
};

const IntegrationsScene = ({duration, index}: SceneProps) => {
  const frame = useCurrentFrame();

  return (
    <Scene index={index} label="Agent sessions">
      <ProductSceneLayout
        frame={frame}
        eyebrow="Keep the tools you trust"
        headline={
          <>
            Your agents. <span style={{color: colors.mint}}>Their credentials.</span>
          </>
        }
        detail="Official vendor sessions stay with the vendor CLI. Pytxo reads only redacted readiness."
      >
        <ProductFrame
          src="product/integrations-1600x1000.png"
          frame={frame}
          duration={duration}
          panTo={[0, -24]}
          zoomTo={1.045}
          focus={{left: 260, top: 285, width: 1228, height: 330}}
        />
      </ProductSceneLayout>
    </Scene>
  );
};

const MissionScene = ({duration, index}: SceneProps) => {
  const frame = useCurrentFrame();

  return (
    <Scene index={index} label="Flow" accent={colors.cyan}>
      <ProductSceneLayout
        frame={frame}
        eyebrow="Outcome before commands"
        eyebrowColor={colors.cyan}
        headline={
          <>
            Start with <span style={{color: colors.cyan}}>what should happen.</span>
          </>
        }
        detail="Open your project or create the guided Git example. No provider key is required to inspect the workflow."
      >
        <ProductFrame
          src="product/flow-1600x1000.png"
          frame={frame}
          duration={duration}
          panFrom={[36, -8]}
          panTo={[-20, -30]}
          zoomFrom={1.01}
          zoomTo={1.06}
          focus={{left: 238, top: 140, width: 520, height: 470, color: colors.cyan}}
        />
      </ProductSceneLayout>
    </Scene>
  );
};

const RaceScene = ({duration, index}: SceneProps) => {
  const frame = useCurrentFrame();

  return (
    <Scene index={index} label="Race Shield" accent={colors.amber}>
      <ProductSceneLayout
        frame={frame}
        eyebrow="Conflict-aware before dispatch"
        eyebrowColor={colors.amber}
        headline={
          <>
            Overlaps become <span style={{color: colors.amber}}>execution waves.</span>
          </>
        }
        detail="Paths and dependencies are visible before the first agent process starts."
      >
        <ProductFrame
          src="product/flow-1600x1000.png"
          frame={frame}
          duration={duration}
          panFrom={[-24, -8]}
          panTo={[-68, -42]}
          zoomFrom={1.02}
          zoomTo={1.08}
          focus={{left: 776, top: 140, width: 724, height: 472, color: colors.amber}}
        />
      </ProductSceneLayout>
    </Scene>
  );
};

const WorkspaceCard = ({
  label,
  command,
  color,
  frame,
  delay,
}: {
  label: string;
  command: string;
  color: string;
  frame: number;
  delay: number;
}) => (
  <div
    style={{
      minHeight: 150,
      padding: "28px 30px",
      borderRadius: 18,
      border: `1px solid ${color}55`,
      backgroundColor: `${color}0e`,
      opacity: reveal(frame, delay),
      translate: `0 ${lift(frame, delay)}px`,
    }}
  >
    <div
      style={{
        color,
        fontSize: 20,
        fontWeight: 720,
        letterSpacing: "0.08em",
        textTransform: "uppercase",
      }}
    >
      {label}
    </div>
    <div
      style={{
        marginTop: 18,
        color: colors.text,
        fontFamily: "ui-monospace, SFMono-Regular, Consolas, monospace",
        fontSize: 25,
      }}
    >
      {command}
    </div>
  </div>
);

const IsolationScene = ({index}: SceneProps) => {
  const frame = useCurrentFrame();

  return (
    <Scene index={index} label="Blast Shield" accent={colors.magenta}>
      <div
        style={{
          position: "absolute",
          inset: "155px 100px 80px",
          display: "grid",
          gridTemplateColumns: "0.72fr 1.28fr",
          alignItems: "center",
          gap: 90,
        }}
      >
        <div
          style={{
            display: "flex",
            flexDirection: "column",
            gap: 30,
            opacity: reveal(frame, 4),
            translate: `0 ${lift(frame, 4)}px`,
          }}
        >
          <Eyebrow color={colors.magenta}>Safe until approve</Eyebrow>
          <Headline style={{fontSize: 82}}>
            Work happens away from your checkout.
          </Headline>
          <div style={{color: colors.muted, fontSize: 30, lineHeight: 1.45}}>
            Each agent receives an isolated workspace. The primary repository remains unchanged
            until the review gate.
          </div>
          <div
            style={{
              width: "fit-content",
              padding: "14px 19px",
              border: `1px solid ${colors.mint}55`,
              borderRadius: 12,
              backgroundColor: `${colors.mint}0d`,
              color: colors.mint,
              fontSize: 23,
              fontWeight: 630,
            }}
          >
            Primary checkout · 0 changes
          </div>
        </div>

        <div
          style={{
            position: "relative",
            display: "grid",
            gridTemplateColumns: "260px 1fr",
            alignItems: "center",
            gap: 85,
            minHeight: 690,
          }}
        >
          <svg
            viewBox="0 0 960 690"
            style={{position: "absolute", inset: 0, width: "100%", height: "100%"}}
          >
            <path
              d="M210 345 C370 345 360 125 520 125 M210 345 C370 345 360 345 520 345 M210 345 C370 345 360 565 520 565"
              fill="none"
              stroke={colors.line}
              strokeWidth="4"
              strokeDasharray="10 12"
            />
          </svg>
          <div
            style={{
              position: "relative",
              zIndex: 1,
              padding: "30px 26px",
              borderRadius: 20,
              border: `1px solid ${colors.line}`,
              backgroundColor: colors.panelLifted,
              textAlign: "center",
              opacity: reveal(frame, 18),
            }}
          >
            <div style={{fontSize: 28, fontWeight: 660}}>Primary repo</div>
            <div style={{marginTop: 8, color: colors.muted, fontSize: 21}}>protected</div>
          </div>
          <div
            style={{
              position: "relative",
              zIndex: 1,
              display: "grid",
              gap: 54,
            }}
          >
            <WorkspaceCard
              label="Workspace 01"
              command="codex"
              color={colors.cyan}
              frame={frame}
              delay={30}
            />
            <WorkspaceCard
              label="Workspace 02"
              command="claude"
              color={colors.magenta}
              frame={frame}
              delay={44}
            />
            <WorkspaceCard
              label="Workspace 03"
              command="cursor-agent"
              color={colors.amber}
              frame={frame}
              delay={58}
            />
          </div>
        </div>
      </div>
    </Scene>
  );
};

const OperationsScene = ({duration, index}: SceneProps) => {
  const frame = useCurrentFrame();

  return (
    <Scene index={index} label="Operations">
      <ProductSceneLayout
        frame={frame}
        eyebrow="State, not terminal noise"
        headline={
          <>
            Know exactly <span style={{color: colors.mint}}>what needs you.</span>
          </>
        }
        detail="Running work, responsive agents, path locks, approvals, isolation, cost, and one exact-run stop."
      >
        <ProductFrame
          src="product/operations-1600x1000.png"
          frame={frame}
          duration={duration}
          panFrom={[12, -4]}
          panTo={[-28, -34]}
          zoomTo={1.055}
          focus={{left: 236, top: 112, width: 1260, height: 215}}
        />
      </ProductSceneLayout>
    </Scene>
  );
};

const ApprovalScene = ({duration, index}: SceneProps) => {
  const frame = useCurrentFrame();

  return (
    <Scene index={index} label="Human control" accent={colors.amber}>
      <ProductSceneLayout
        frame={frame}
        eyebrow="Evidence before consequence"
        eyebrowColor={colors.amber}
        headline={
          <>
            Approve the result. <span style={{color: colors.amber}}>Not the promise.</span>
          </>
        }
        detail="Requester, workspace, action, consequence, and run evidence stay together at the decision."
      >
        <ProductFrame
          src="product/approvals-1600x1000.png"
          frame={frame}
          duration={duration}
          panFrom={[-16, -4]}
          panTo={[-54, -38]}
          zoomFrom={1.01}
          zoomTo={1.07}
          focus={{left: 730, top: 120, width: 770, height: 494, color: colors.amber}}
        />
      </ProductSceneLayout>
    </Scene>
  );
};

const Metric = ({
  value,
  label,
  detail,
  color,
  frame,
  delay,
}: {
  value: string;
  label: string;
  detail: string;
  color: string;
  frame: number;
  delay: number;
}) => (
  <div
    style={{
      display: "flex",
      flexDirection: "column",
      minHeight: 300,
      padding: "34px 38px",
      borderRadius: 22,
      border: `1px solid ${color}4f`,
      backgroundColor: `${color}0c`,
      opacity: reveal(frame, delay),
      translate: `0 ${lift(frame, delay)}px`,
    }}
  >
    <div
      style={{
        color,
        fontSize: 76,
        lineHeight: 1,
        fontWeight: 700,
        letterSpacing: "-0.055em",
      }}
    >
      {value}
    </div>
    <div style={{marginTop: 22, fontSize: 27, fontWeight: 650}}>{label}</div>
    <div style={{marginTop: 12, color: colors.muted, fontSize: 22, lineHeight: 1.42}}>
      {detail}
    </div>
  </div>
);

const BenchmarkScene = ({index}: SceneProps) => {
  const frame = useCurrentFrame();

  return (
    <Scene index={index} label="Measured locally" accent={colors.cyan}>
      <div
        style={{
          position: "absolute",
          inset: "152px 100px 88px",
          display: "flex",
          flexDirection: "column",
          justifyContent: "center",
          gap: 46,
        }}
      >
        <div
          style={{
            display: "flex",
            alignItems: "end",
            justifyContent: "space-between",
            gap: 80,
            opacity: reveal(frame, 3),
            translate: `0 ${lift(frame, 3)}px`,
          }}
        >
          <div style={{display: "flex", flexDirection: "column", gap: 18}}>
            <Eyebrow color={colors.cyan}>Reproducible harness · 30 July 2026</Eyebrow>
            <Headline style={{fontSize: 76}}>Proof with a boundary.</Headline>
          </div>
          <div
            style={{
              maxWidth: 610,
              color: colors.muted,
              fontSize: 25,
              lineHeight: 1.42,
              textAlign: "right",
            }}
          >
            Structural-byte and local control-plane measurements. Not model quality, token
            billing, or competitor outcome claims.
          </div>
        </div>
        <div style={{display: "grid", gridTemplateColumns: "1fr 1fr 1fr", gap: 22}}>
          <Metric
            value="82.9%"
            label="weighted byte reduction"
            detail="Signal scaffold output across 185 tracked Rust, TypeScript, and JavaScript files."
            color={colors.cyan}
            frame={frame}
            delay={22}
          />
          <Metric
            value="5 / 5"
            label="isolated agents exited 0"
            detail="Five local echo agents, two execution waves, and zero same-wave path collisions."
            color={colors.mint}
            frame={frame}
            delay={36}
          />
          <Metric
            value="0"
            label="primary checkout changes"
            detail="The local isolation run completed without modifying the primary checkout."
            color={colors.amber}
            frame={frame}
            delay={50}
          />
        </div>
      </div>
    </Scene>
  );
};

const ResultScene = ({index}: SceneProps) => {
  const frame = useCurrentFrame();
  const tiles = [
    {src: "product/integrations-1600x1000.png", left: -300, top: 80, rotate: -5},
    {src: "product/flow-1600x1000.png", left: 1280, top: 90, rotate: 5},
    {src: "product/operations-1600x1000.png", left: -280, top: 710, rotate: 4},
    {src: "product/approvals-1600x1000.png", left: 1270, top: 700, rotate: -4},
  ];

  return (
    <Scene index={index} label="Pytxo" accent={colors.mint}>
      {tiles.map((tile, tileIndex) => (
        <div
          key={tile.src}
          style={{
            position: "absolute",
            left: tile.left,
            top: tile.top,
            width: 720,
            height: 450,
            overflow: "hidden",
            borderRadius: 22,
            border: `1px solid ${colors.line}`,
            boxShadow: "0 38px 90px rgba(0,0,0,0.55)",
            opacity: interpolate(frame, [10 + tileIndex * 7, 40 + tileIndex * 7], [0, 0.24], {
              ...clamp,
              easing: ease,
            }),
            rotate: `${tile.rotate}deg`,
            scale: interpolate(frame, [0, 225], [1, 1.05], clamp),
          }}
        >
          <Img
            src={staticFile(tile.src)}
            style={{width: "100%", height: "100%", objectFit: "cover", objectPosition: "top"}}
          />
        </div>
      ))}
      <div
        style={{
          position: "absolute",
          inset: "170px 120px 90px",
          display: "flex",
          flexDirection: "column",
          alignItems: "center",
          justifyContent: "center",
          textAlign: "center",
          gap: 34,
          opacity: reveal(frame, 4),
          translate: `0 ${lift(frame, 4)}px`,
        }}
      >
        <Eyebrow>Keep your tools. Keep control.</Eyebrow>
        <Headline>
          One engineering mission.
          <br />
          <span style={{color: colors.mint}}>One safe, reviewable result.</span>
        </Headline>
        <div
          style={{
            marginTop: 16,
            display: "flex",
            alignItems: "center",
            gap: 22,
            padding: "20px 28px",
            borderRadius: 16,
            border: `1px solid ${colors.mint}66`,
            backgroundColor: `${colors.mint}0d`,
            color: colors.text,
            fontSize: 29,
            fontWeight: 650,
            opacity: reveal(frame, 44),
            scale: interpolate(frame, [44, 70], [0.95, 1], {...clamp, easing: ease}),
          }}
        >
          Run the guided example
          <span style={{color: colors.mint}}>pytxo.com →</span>
        </div>
      </div>
    </Scene>
  );
};

const sceneComponents = [
  HookScene,
  IntegrationsScene,
  MissionScene,
  RaceScene,
  IsolationScene,
  OperationsScene,
  ApprovalScene,
  BenchmarkScene,
  ResultScene,
];

export const PytxoLaunchDemo = ({includeVoiceover}: PytxoLaunchDemoProps) => (
  <AbsoluteFill style={{backgroundColor: colors.canvas}}>
    <TransitionSeries>
      {voiceover.map((scene, index) => {
        const SceneComponent = sceneComponents[index];
        if (!SceneComponent) return null;

        return (
          <>
            <TransitionSeries.Sequence
              key={scene.id}
              durationInFrames={scene.durationInFrames}
              name={`${index + 1}. ${scene.id}`}
            >
              <SceneComponent duration={scene.durationInFrames} index={index} />
              {includeVoiceover ? (
                <Audio src={staticFile(`voiceover/${scene.id}.mp3`)} />
              ) : null}
            </TransitionSeries.Sequence>
            {index < voiceover.length - 1 ? (
              <TransitionSeries.Transition
                key={`${scene.id}-transition`}
                presentation={fade()}
                timing={linearTiming({durationInFrames: TRANSITION_FRAMES})}
              />
            ) : null}
          </>
        );
      })}
    </TransitionSeries>
  </AbsoluteFill>
);
