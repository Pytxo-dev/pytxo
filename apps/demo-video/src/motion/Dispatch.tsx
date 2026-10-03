import {useCurrentFrame} from "remotion";
import {C, Label, MaskText, mix, move, Stage, type FilmProps} from "./design";

const tasks = ["Search & filtering", "Dark theme", "Spanish translation", "Filter controls", "Input validation", "Documentation"];
const names = ["Codex", "Claude Code", "Cursor Agent", "OpenCode", "Antigravity", "Codex"];

export const Dispatch = (props: FilmProps) => {
  const f = useCurrentFrame();
  const split = move(f, 98, 192);
  const arrange = move(f, 264, 348);
  const depart = move(f, 820, 899);
  const heading = f < 264 ? "One request. Six tasks." : f < 568 ? "Four work in parallel." : "Then the next. And the next.";
  return <Stage>
    <div style={{position: "absolute", top: 108 - depart * 350, left: 88}} key={heading}><MaskText at={f < 264 ? 0 : f < 568 ? 264 : 568} size={88}>{heading}</MaskText></div>
    <Label style={{position: "absolute", left: 94, top: 226, opacity: 1 - depart}}>{f < 264 ? "Improve this task board." : "6 tasks / 5 vendors / recorded order, time compressed"}</Label>
    <div style={{position: "absolute", left: 96, top: 368, width: 1728, height: 140, border: `2px solid ${C.ink}`, borderRadius: 4, opacity: 1 - split, display: "flex", alignItems: "center", padding: "0 38px", boxSizing: "border-box", fontSize: 56}}>Improve this task board.<span style={{marginLeft: "auto", fontSize: 62}}>+</span></div>
    <svg width={1920} height={1080} style={{position: "absolute", inset: 0, opacity: arrange * (1 - depart)}} aria-hidden>
      {[0, 1, 2, 3].map(i => <path key={i} d={`M 654 ${384 + i * 128} H 704 Q 730 ${384 + i * 128} 730 ${410 + i * 128} V 570 Q 730 596 756 596 H 817`} fill="none" stroke={C.rule} strokeWidth={3} />)}
      <path d="M 1238 596 H 1408" stroke={C.rule} strokeWidth={3} />
      <path d="M 730 596 H 817" stroke={C.ink} strokeWidth={4} pathLength={1} strokeDasharray={1} strokeDashoffset={1 - move(f, 535, 574)} />
      <path d="M 1238 596 H 1408" stroke={C.ink} strokeWidth={4} pathLength={1} strokeDasharray={1} strokeDashoffset={1 - move(f, 678, 714)} />
    </svg>
    {props.workers.map((worker, i) => {
      const gridX = 96 + (i % 3) * 582;
      const gridY = 353 + Math.floor(i / 3) * 238;
      const laneX = i < 4 ? 96 : i === 4 ? 818 : 1410;
      const laneY = i < 4 ? 331 + i * 128 : 505;
      const activeAt = i < 4 ? 368 : i === 4 ? 574 : 714;
      const finishedAt = i < 4 ? 498 : i === 4 ? 655 : 802;
      const active = move(f, activeAt, activeAt + 22);
      const done = move(f, finishedAt, finishedAt + 24);
      const hasFiles = worker.filesPrepared.length > 0;
      return <div key={worker.taskId} style={{position: "absolute", left: mix(mix(96, gridX, split), laneX, arrange), top: mix(mix(368, gridY, split), laneY, arrange) - depart * (80 + i * 16), width: mix(546, i < 4 ? 560 : 416, arrange), height: mix(198, i < 4 ? 108 : 182, arrange), opacity: split * (1 - depart), border: `2px solid ${done ? hasFiles ? C.green : C.muted : active ? C.ink : C.rule}`, borderRadius: 4, boxSizing: "border-box", background: C.paper, overflow: "hidden", padding: "17px 22px"}}>
        <div style={{position: "absolute", inset: 0, background: done && hasFiles ? C.wash : "#f0f1f0", transformOrigin: "left", transform: `scaleX(${active * (1 - done) + done * (hasFiles ? 1 : 0)})`, opacity: 0.8}} />
        <div style={{position: "relative", display: "flex", alignItems: "baseline", justifyContent: "space-between"}}><span style={{fontSize: 34, fontWeight: 620}}>{names[i]}</span><span style={{fontSize: 27, color: C.muted}}>{String(i + 1).padStart(2, "0")}</span></div>
        <div style={{position: "relative", marginTop: 7, fontSize: 30, color: C.muted, whiteSpace: "nowrap"}}>{done > 0.9 ? hasFiles ? `${worker.filesPrepared.length} ${worker.filesPrepared.length === 1 ? "file" : "files"} prepared` : "No changes prepared" : tasks[i]}</div>
        <div style={{position: "absolute", bottom: 0, left: 0, height: 3, width: `${move(f, activeAt, finishedAt) * 100}%`, background: hasFiles ? C.green : C.ink, opacity: arrange}} />
      </div>;
    })}
    <Label style={{position: "absolute", top: 884, left: 96, opacity: move(f, 380, 420) * (1 - depart)}}>{f < 714 ? "Separate workspaces. Shared run history." : "4 workers prepared changes. 2 prepared none."}</Label>
    <div style={{position: "absolute", right: 100, top: 863, opacity: arrange * (1 - depart), fontSize: 58, fontWeight: 550}}>4<span style={{color: C.muted, margin: "0 20px"}}>/</span>1<span style={{color: C.muted, margin: "0 20px"}}>/</span>1</div>
  </Stage>;
};
