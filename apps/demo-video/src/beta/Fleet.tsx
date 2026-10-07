import {useCurrentFrame} from "remotion";
import {AgentLogo, amber, green, ink, line, muted, Page, progress, Reveal, type FilmProps} from "./shared";

const positions = [{x: 96, y: 365}, {x: 510, y: 365}, {x: 924, y: 365}, {x: 1338, y: 365}, {x: 510, y: 740}, {x: 1170, y: 740}];
const labels = ["Search & filtering", "Dark theme", "Spanish", "Filter controls", "Validation", "Documentation"];

export const Fleet = (props: FilmProps) => {
  const frame = useCurrentFrame();
  const settled = frame >= 345;
  return <Page chapter="02 / The work">
    <Reveal style={{position: "absolute", left: 88, top: 163, fontSize: 88, fontWeight: 600, lineHeight: 1.1}}>Parallel where it can.<br /><span style={{color: muted}}>Ordered where it must.</span></Reveal>
    <svg width={1920} height={1080} style={{position: "absolute", inset: 0}} aria-hidden>
      {[0, 1, 2, 3].map(index => {
        const x = positions[index].x + 180;
        return <path key={index} d={`M ${x} 570 V 631 Q ${x} 651 ${x + (x > 690 ? -20 : 20)} 651 H 670 Q 690 651 690 671 V 735`} fill="none" stroke={line} strokeWidth={3} pathLength={1} strokeDasharray={1} strokeDashoffset={1 - progress(frame, 116 + index * 12, 200 + index * 12)} />;
      })}
      <path d="M 877 795 H 1164" fill="none" stroke={ink} strokeWidth={3} pathLength={1} strokeDasharray={1} strokeDashoffset={1 - progress(frame, 226, 292)} />
    </svg>
    {props.workers.map((worker, index) => {
      const location = positions[index];
      if (!location) return null;
      const produces = worker.filesPrepared.length > 0;
      return <Reveal key={worker.taskId} at={index < 4 ? 32 + index * 10 : 180 + (index - 4) * 62} style={{position: "absolute", ...{left: location.x, top: location.y}, width: 366}}>
        <div style={{display: "flex", alignItems: "center", gap: 18}}>
          <AgentLogo cli={worker.cli} size={40} />
          <div style={{fontSize: 30, fontWeight: 600}}>{worker.vendor === "OpenAI Codex" ? "Codex" : worker.vendor}</div>
        </div>
        <div style={{marginTop: 20, fontSize: 32, color: muted}}>{labels[index]}</div>
        <div style={{marginTop: 14, fontSize: 28, color: produces ? green : amber, opacity: settled ? progress(frame, 345, 372) : 0}}>{produces ? `${worker.filesPrepared.length} ${worker.filesPrepared.length === 1 ? "file" : "files"} prepared` : "No changes prepared"}</div>
      </Reveal>;
    })}
    <Reveal at={48} style={{position: "absolute", top: 702, left: 96, fontSize: 28, color: muted}}>{props.waves[0]} workers in parallel</Reveal>
  </Page>;
};

export const Package = (props: FilmProps) => {
  const owners = new Set(props.files.map(file => file.taskId)).size;
  return <Page chapter="03 / The result">
    <Reveal style={{position: "absolute", left: 88, top: 180, width: 720, fontSize: 80, fontWeight: 600, lineHeight: 1.12}}>One result.<br /><span style={{color: green}}>Checked together.</span></Reveal>
    <Reveal at={28} style={{position: "absolute", left: 96, top: 505, fontSize: 38, color: muted, lineHeight: 1.65}}>
      <div>{props.files.length} prepared files</div>
      <div>{props.checks.passed} / {props.checks.recorded} combined checks passed</div>
      <div>{owners} / {props.workers.length} workers prepared changes</div>
    </Reveal>
    <div style={{position: "absolute", left: 895, top: 183, width: 936, borderTop: `2px solid ${ink}`}}>
      {props.files.map((file, index) => <Reveal key={file.path} at={32 + index * 14} style={{padding: "20px 0", borderBottom: `1px solid ${line}`, fontSize: 30, display: "flex", alignItems: "center", gap: 16}}>
        <span style={{color: muted, fontSize: 26, width: 30}}>{file.kind === "add" ? "+" : file.kind === "delete" ? "-" : "M"}</span><span style={{flex: 1}}>{file.path}</span>
      </Reveal>)}
      <Reveal at={145} style={{marginTop: 28, fontSize: 28, color: muted}}>Package {props.run.packageDigest.slice(0, 16)}</Reveal>
    </div>
    <Reveal at={192} style={{position: "absolute", left: 96, top: 846, fontSize: 47, fontWeight: 550}}>You decide what enters your project.</Reveal>
  </Page>;
};
