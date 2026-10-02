import type {CSSProperties, ReactNode} from "react";
import {AbsoluteFill, Img, Sequence, interpolate, spring, staticFile, useCurrentFrame, useVideoConfig} from "remotion";

// Every count, name, path, digest and worker line comes from fleet-props.json,
// exported from one recorded native run by scripts/fleet-ledger.mjs.
export const FLEET_FPS = 30;
const SCENES = {
  wall: [0, 240],
  constellation: [240, 420],
  board: [420, 690],
  boundary: [690, 900],
  stale: [900, 1050],
  apply: [1050, 1200],
  result: [1200, 1350],
  end: [1350, 1560],
} as const;
export const FLEET_FRAMES = SCENES.end[1];

type Worker = {
  taskId: string;
  wave: number;
  request: string;
  cli: string;
  vendor: string;
  paths: string[];
  dependsOn: string[];
  status: string;
  exitCode: number | null;
  filesPrepared: string[];
  output: string[];
};
type Asset = {key: string; path: string; sha256: string; width: number; height: number};
export type PytxoFleetFilmProps = {
  schemaVersion: 1;
  kind: "fleet-film";
  run: {title: string; durationSeconds: number; packageDigest: string; staleDigest: string | null; applyStatus: string};
  waves: number[];
  vendors: string[];
  workers: Worker[];
  files: {path: string; kind: string; taskId: string; vendor: string}[];
  checks: {recorded: number; passed: number};
  evidenceBoundary: {label: string};
  assets: Asset[];
};

export const parseFleetFilmProps = (input: unknown): PytxoFleetFilmProps => {
  const props = input as Partial<PytxoFleetFilmProps> | null;
  if (!props || props.schemaVersion !== 1 || props.kind !== "fleet-film" || !props.run || !Array.isArray(props.workers) || !Array.isArray(props.assets)) {
    throw new Error("fleet-props.json does not match the fleet film schema; regenerate it with scripts/fleet-ledger.mjs");
  }
  return props as PytxoFleetFilmProps;
};

const color = {
  canvas: "#050608",
  panel: "#0b0d11",
  raised: "#12151b",
  line: "#252932",
  text: "#f7f7f8",
  muted: "#9ca4b2",
  green: "#62d89b",
  amber: "#f3c64e",
};
const SPECTRUM = "linear-gradient(90deg, #f04da3, #ff6a4a, #f3c64e, #79e07b, #45dccb)";
const ACCENT: Record<string, string> = {codex: "#45dccb", claude: "#ff8a5c", cursor: "#b4b9ff", opencode: "#f3c64e", agy: "#79e07b"};
const MONO = "'Cascadia Mono', Consolas, 'IBM Plex Mono', monospace";
const clamp = {extrapolateLeft: "clamp" as const, extrapolateRight: "clamp" as const};
const fade = (frame: number, from: number, to: number) => interpolate(frame, [from, to], [0, 1], clamp);

const Logo = ({cli, size}: {cli: string; size: number}) => {
  const box: CSSProperties = {width: size, height: size, flex: "none"};
  if (cli === "claude") return <div style={{...box, background: color.text, mask: `url(${staticFile("fleet/logos/anthropic.svg")}) center / contain no-repeat`}} />;
  const file = {codex: "openai-on-dark.svg", cursor: "cursor-on-dark.svg", opencode: "opencode.svg"}[cli];
  // Antigravity's mark is not cleared for public use; it is named, not drawn.
  if (!file) return <div style={{...box, borderRadius: size, background: ACCENT[cli] ?? color.muted}} />;
  return <Img src={staticFile(`fleet/logos/${file}`)} style={box} />;
};

const Disclosure = ({label}: {label: string}) => (
  <div style={{position: "absolute", right: 48, bottom: 30, color: color.muted, fontSize: 20, letterSpacing: 0.2}}>{label}</div>
);

const Caption = ({children, at, style}: {children: ReactNode; at: number; style?: CSSProperties}) => {
  const frame = useCurrentFrame();
  const {fps} = useVideoConfig();
  const enter = spring({frame: frame - at, fps, config: {damping: 200}});
  return (
    <div style={{position: "absolute", left: 96, bottom: 96, color: color.text, fontSize: 56, fontWeight: 650, letterSpacing: -1.2, opacity: enter, transform: `translateY(${(1 - enter) * 24}px)`, ...style}}>
      {children}
    </div>
  );
};

/** Six recorded workers stream their real output side by side; later waves wait for their inputs. */
const TerminalWall = ({props}: {props: PytxoFleetFilmProps}) => {
  const frame = useCurrentFrame();
  const waveStart = [12, 120, 168];
  const end = SCENES.wall[1] - 18;
  return (
    <AbsoluteFill style={{padding: "150px 60px 70px", display: "grid", gridTemplateColumns: "repeat(3, 1fr)", gridTemplateRows: "repeat(2, 1fr)", gap: 20}}>
      <div style={{position: "absolute", left: 60, top: 46, display: "flex", alignItems: "baseline", gap: 28}}>
        <span style={{fontSize: 64, fontWeight: 700, letterSpacing: -1.6, color: color.text, opacity: fade(frame, 0, 14)}}>One request.</span>
        <span style={{fontSize: 40, color: color.muted, opacity: fade(frame, 40, 60)}}>
          {props.workers.length} agents · {props.vendors.length} vendors · 1 repository
        </span>
      </div>
      {props.workers.map((worker, index) => {
        const start = waveStart[Math.min(worker.wave, 2)];
        const lines = [`› ${worker.request}`, ...worker.output];
        const progress = interpolate(frame, [start, end], [0, lines.length], clamp);
        const shown = lines.slice(Math.max(0, Math.floor(progress) - 10), Math.ceil(progress));
        const typing = progress % 1;
        const live = frame >= start && progress < lines.length;
        const done = progress >= lines.length;
        const unchanged = done && worker.filesPrepared.length === 0;
        const accent = ACCENT[worker.cli] ?? color.muted;
        const enter = fade(frame, index * 3, index * 3 + 12);
        return (
          <div key={worker.taskId} style={{display: "flex", flexDirection: "column", minHeight: 0, borderRadius: 14, border: `1.5px solid ${live ? accent : color.line}`, background: color.panel, overflow: "hidden", opacity: enter * (frame < start ? 0.45 : 1), boxShadow: live ? `0 0 0 1px ${accent}33, 0 18px 50px -24px ${accent}88` : "none"}}>
            <div style={{display: "flex", alignItems: "center", gap: 14, padding: "14px 18px", borderBottom: `1px solid ${color.line}`, background: color.raised}}>
              <Logo cli={worker.cli} size={34} />
              <span style={{fontSize: 30, fontWeight: 650, color: color.text}}>{worker.vendor}</span>
              <span style={{marginLeft: "auto", fontSize: 22, fontWeight: 600, color: frame < start ? color.muted : unchanged ? color.muted : done ? color.green : accent}}>
                {frame < start ? "waits for inputs" : unchanged ? "no changes" : done ? "✓ checks passed" : "● working"}
              </span>
            </div>
            <div style={{flex: 1, display: "flex", flexDirection: "column", justifyContent: "flex-end", padding: "12px 18px", fontFamily: MONO, fontSize: 20, lineHeight: 1.45, color: color.text}}>
              {shown.map((line, lineIndex) => {
                const last = lineIndex === shown.length - 1 && live;
                const text = last ? line.slice(0, Math.max(1, Math.round(line.length * typing))) : line;
                const tone = line.startsWith("✓") ? color.green : line.startsWith("$ ") || line.startsWith("›") ? accent : line.startsWith("+") ? "#b9f0c8" : color.text;
                return <div key={lineIndex} style={{whiteSpace: "nowrap", overflow: "hidden", textOverflow: "ellipsis", color: tone}}>{text}</div>;
              })}
            </div>
          </div>
        );
      })}
    </AbsoluteFill>
  );
};

/** The plan as a graph: waves left to right, a shared file forcing order. */
const Constellation = ({props}: {props: PytxoFleetFilmProps}) => {
  const frame = useCurrentFrame();
  const {fps} = useVideoConfig();
  const columns = [360, 960, 1560];
  const byWave = props.workers.reduce<Record<number, Worker[]>>((acc, worker) => ({...acc, [worker.wave]: [...(acc[worker.wave] ?? []), worker]}), {});
  const position = (worker: Worker) => {
    const peers = byWave[worker.wave];
    const index = peers.indexOf(worker);
    return {x: columns[Math.min(worker.wave, 2)], y: 280 + ((index + 0.5) * 560) / peers.length};
  };
  const shared = props.workers.flatMap((later) =>
    props.workers
      .filter((earlier) => earlier.wave < later.wave)
      .flatMap((earlier) => later.paths.filter((path) => earlier.paths.includes(path)).map((path) => ({from: earlier, to: later, path}))),
  );
  const edges = props.workers.flatMap((later) =>
    later.dependsOn
      .map((id) => props.workers.find((worker) => worker.taskId === id))
      .filter((from): from is Worker => !!from && !shared.some((edge) => edge.from === from && edge.to === later))
      .map((from) => ({from, to: later})),
  );
  const draw = interpolate(frame, [20, 80], [0, 1], clamp);
  const curve = (a: Worker, b: Worker) => {
    const p = position(a);
    const q = position(b);
    return `M ${p.x + 36} ${p.y} C ${(p.x + q.x) / 2} ${p.y}, ${(p.x + q.x) / 2} ${q.y}, ${q.x - 36} ${q.y}`;
  };
  return (
    <AbsoluteFill>
      {props.waves.map((count, wave) => (
        <div key={wave} style={{position: "absolute", top: 140, left: columns[wave] - 220, width: 440, textAlign: "center", color: color.muted, fontSize: 30, letterSpacing: 1.5, textTransform: "uppercase", opacity: fade(frame, wave * 8, wave * 8 + 16)}}>
          Wave {wave + 1}{count > 1 ? ` · ${count} at once` : ""}
        </div>
      ))}
      <svg width={1920} height={1080} style={{position: "absolute", inset: 0}}>
        {edges.map(({from, to}) => (
          <path key={`${from.taskId}-${to.taskId}`} d={curve(from, to)} fill="none" stroke={color.line} strokeWidth={3} pathLength={1} strokeDasharray={1} strokeDashoffset={1 - draw} />
        ))}
        {shared.map(({from, to}) => (
          <path key={`shared-${from.taskId}-${to.taskId}`} d={curve(from, to)} fill="none" stroke={color.amber} strokeWidth={5} pathLength={1} strokeDasharray={1} strokeDashoffset={1 - interpolate(frame, [70, 110], [0, 1], clamp)} />
        ))}
      </svg>
      {props.workers.map((worker, index) => {
        const {x, y} = position(worker);
        const pop = spring({frame: frame - index * 4, fps, config: {damping: 14, stiffness: 140}});
        return (
          <div key={worker.taskId} style={{position: "absolute", left: x - 150, top: y - 34, width: 300, display: "flex", flexDirection: "column", alignItems: "center", gap: 10, transform: `scale(${pop})`}}>
            <div style={{display: "grid", placeItems: "center", width: 68, height: 68, borderRadius: 68, background: color.raised, border: `2px solid ${ACCENT[worker.cli] ?? color.line}`}}>
              <Logo cli={worker.cli} size={34} />
            </div>
            <span style={{fontSize: 30, fontWeight: 650, color: color.text}}>{worker.vendor}</span>
          </div>
        );
      })}
      {shared.slice(0, 1).map(({path}) => (
        <div key={path} style={{position: "absolute", left: 0, right: 0, top: 880, textAlign: "center", fontFamily: MONO, fontSize: 30, color: color.amber, opacity: fade(frame, 100, 120)}}>
          shares {path} → runs after it
        </div>
      ))}
      <Caption at={120} style={{bottom: 60, left: 0, right: 0, textAlign: "center", fontSize: 48}}>Shared files wait their turn. Nothing overwrites anything.</Caption>
    </AbsoluteFill>
  );
};

const Still = ({asset, from, to, focus = {x: 50, y: 50, scale: 1}}: {asset: Asset; from: number; to: number; focus?: {x: number; y: number; scale: number}}) => {
  const frame = useCurrentFrame();
  const zoom = interpolate(frame, [from, to], [1, focus.scale], clamp);
  const opacity = Math.min(fade(frame, from, from + 12), 1 - fade(frame, to - 12, to));
  return (
    <AbsoluteFill style={{opacity, alignItems: "center", paddingTop: 104}}>
      <div style={{width: 1440, aspectRatio: `${asset.width} / ${asset.height}`, maxHeight: 800, borderRadius: 16, overflow: "hidden", border: `1px solid ${color.line}`, boxShadow: "0 40px 120px -40px rgba(69,220,203,0.35)"}}>
        <Img src={staticFile(asset.path)} style={{width: "100%", height: "100%", objectFit: "cover", transform: `scale(${zoom})`, transformOrigin: `${focus.x}% ${focus.y}%`}} />
      </div>
    </AbsoluteFill>
  );
};

const Chip = ({children}: {children: ReactNode}) => (
  <div style={{position: "absolute", left: 240, top: 30, padding: "10px 20px", borderRadius: 999, background: "rgba(5,6,8,0.86)", border: `1px solid ${color.line}`, color: color.text, fontSize: 26, fontWeight: 600}}>{children}</div>
);

const NativeBoard = ({props}: {props: PytxoFleetFilmProps}) => {
  const boards = ["board-1", "board-2", "board-3", "board-4"].map((key) => props.assets.find((asset) => asset.key === key)).filter((asset): asset is Asset => !!asset);
  const span = (SCENES.board[1] - SCENES.board[0]) / Math.max(1, boards.length);
  const minutes = Math.floor(props.run.durationSeconds / 60);
  return (
    <AbsoluteFill>
      {boards.map((asset, index) => (
        <Still key={asset.key} asset={asset} from={index * span} to={(index + 1) * span + 12} focus={{x: 35, y: 55, scale: 1.08}} />
      ))}
      <Chip>● Recorded in Pytxo Desktop · {minutes} m {props.run.durationSeconds % 60} s of work, shown in {Math.round((SCENES.board[1] - SCENES.board[0]) / FLEET_FPS)} s</Chip>
    </AbsoluteFill>
  );
};

const short = (digest: string) => `${digest.slice(0, 12)}…${digest.slice(-4)}`;

/** Every worker's files converge into one exact, checked package. */
const Boundary = ({props}: {props: PytxoFleetFilmProps}) => {
  const frame = useCurrentFrame();
  const {fps} = useVideoConfig();
  const card = spring({frame: frame - 40, fps, config: {damping: 200}});
  return (
    <AbsoluteFill>
      <svg width={1920} height={1080} style={{position: "absolute", inset: 0}}>
        {props.workers.map((worker, index) => {
          const y = 230 + index * 120;
          const draw = interpolate(frame, [index * 5, index * 5 + 40], [0, 1], clamp);
          return <path key={worker.taskId} d={`M 470 ${y} C 640 ${y}, 640 540, 800 540`} fill="none" stroke={ACCENT[worker.cli] ?? color.line} strokeWidth={3} opacity={0.8} pathLength={1} strokeDasharray={1} strokeDashoffset={1 - draw} />;
        })}
      </svg>
      {props.workers.map((worker, index) => (
        <div key={worker.taskId} style={{position: "absolute", left: 90, top: 230 + index * 120 - 26, width: 370, display: "flex", alignItems: "center", gap: 14, opacity: fade(frame, index * 5, index * 5 + 12)}}>
          <Logo cli={worker.cli} size={34} />
          <span style={{fontSize: 30, fontWeight: 650, color: color.text}}>{worker.vendor}</span>
          <span style={{marginLeft: "auto", fontSize: 24, color: worker.filesPrepared.length ? color.muted : color.muted}}>{worker.filesPrepared.length ? `${worker.filesPrepared.length} file${worker.filesPrepared.length === 1 ? "" : "s"}` : "no changes"}</span>
        </div>
      ))}
      <div style={{position: "absolute", left: 800, top: 150, width: 1030, padding: "34px 40px", borderRadius: 18, background: color.panel, border: "1.5px solid transparent", backgroundImage: `linear-gradient(${color.panel}, ${color.panel}), ${SPECTRUM}`, backgroundOrigin: "border-box", backgroundClip: "padding-box, border-box", opacity: card, transform: `translateX(${(1 - card) * 40}px)`}}>
        <div style={{fontSize: 48, fontWeight: 700, letterSpacing: -1, color: color.text}}>One exact package</div>
        <div style={{marginTop: 22, display: "grid", gap: 8}}>
          {props.files.map((file, index) => (
            <div key={file.path} style={{display: "flex", fontFamily: MONO, fontSize: 28, color: color.text, opacity: fade(frame, 60 + index * 6, 72 + index * 6)}}>
              <span style={{color: color.amber, width: 34}}>{file.kind === "add" ? "A" : file.kind === "delete" ? "D" : "M"}</span>
              <span>{file.path}</span>
              <span style={{marginLeft: "auto", fontFamily: "inherit", fontSize: 24, color: color.muted}}>{file.vendor}</span>
            </div>
          ))}
        </div>
        <div style={{marginTop: 26, paddingTop: 22, borderTop: `1px solid ${color.line}`, display: "flex", justifyContent: "space-between", alignItems: "baseline", opacity: fade(frame, 120, 140)}}>
          <span style={{fontSize: 32, fontWeight: 650, color: color.green}}>✓ {props.checks.passed} of {props.checks.recorded} checks passed</span>
          <span style={{fontFamily: MONO, fontSize: 30, color: color.text}}>{short(props.run.packageDigest)}</span>
        </div>
      </div>
      <Caption at={150} style={{left: 800, bottom: 70, fontSize: 46}}>Nothing lands until you approve these bytes.</Caption>
    </AbsoluteFill>
  );
};

const asset = (props: PytxoFleetFilmProps, key: string) => props.assets.find((candidate) => candidate.key === key);

export const PytxoFleetFilm = (props: PytxoFleetFilmProps) => {
  const stale = asset(props, "stale");
  const applied = asset(props, "applied");
  const result = asset(props, "result");
  const scene = (name: keyof typeof SCENES) => ({from: SCENES[name][0], durationInFrames: SCENES[name][1] - SCENES[name][0]});
  return (
    <AbsoluteFill style={{background: color.canvas, fontFamily: "'Geist Variable', Geist, system-ui, sans-serif"}}>
      <Sequence {...scene("wall")}><TerminalWall props={props} /></Sequence>
      <Sequence {...scene("constellation")}><Constellation props={props} /></Sequence>
      <Sequence {...scene("board")}><NativeBoard props={props} /></Sequence>
      <Sequence {...scene("boundary")}><Boundary props={props} /></Sequence>
      {stale && (
        <Sequence {...scene("stale")}>
          <Still asset={stale} from={0} to={150} focus={{x: 30, y: 92, scale: 1.5}} />
          <Caption at={30} style={{left: 240, bottom: 60}}>A file changed after review. Apply refused.</Caption>
        </Sequence>
      )}
      {applied && (
        <Sequence {...scene("apply")}>
          <Still asset={applied} from={0} to={150} focus={{x: 30, y: 92, scale: 1.5}} />
          <Caption at={30} style={{left: 240, bottom: 60}}>Rechecked. Applied: exactly {props.files.length} files.</Caption>
        </Sequence>
      )}
      {result && (
        <Sequence {...scene("result")}>
          <Still asset={result} from={0} to={150} focus={{x: 50, y: 25, scale: 1.12}} />
          <Caption at={20} style={{left: 240, bottom: 60}}>The task board, after Apply.</Caption>
        </Sequence>
      )}
      <Sequence {...scene("end")}><EndCard /></Sequence>
      <Disclosure label={props.evidenceBoundary.label} />
    </AbsoluteFill>
  );
};

const EndCard = () => {
  const frame = useCurrentFrame();
  const {fps} = useVideoConfig();
  const rise = spring({frame, fps, config: {damping: 200}});
  return (
    <AbsoluteFill style={{display: "grid", placeItems: "center", textAlign: "center"}}>
      <div style={{opacity: rise, transform: `translateY(${(1 - rise) * 30}px)`}}>
        <Img src={staticFile("logo-mark.png")} style={{width: 84, height: 84}} />
        <div style={{marginTop: 30, fontSize: 108, fontWeight: 700, letterSpacing: -4, lineHeight: 1.02, color: color.text}}>Every coding agent you have.</div>
        <div style={{display: "inline-block", fontSize: 108, fontWeight: 700, letterSpacing: -4, lineHeight: 1.1, backgroundImage: SPECTRUM, WebkitBackgroundClip: "text", color: "transparent", opacity: fade(frame, 20, 40)}}>At once.</div>
        <div style={{marginTop: 36, fontSize: 36, color: color.muted, opacity: fade(frame, 50, 70)}}>pytxo.com</div>
      </div>
    </AbsoluteFill>
  );
};
