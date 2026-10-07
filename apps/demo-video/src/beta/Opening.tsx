import {Img, staticFile, useCurrentFrame} from "remotion";
import {AgentLogo, ink, line, muted, Page, progress, Reveal, type FilmProps} from "./shared";

export const Opening = (props: FilmProps) => {
  const frame = useCurrentFrame();
  const workers = props.workers.filter((worker, index, all) => all.findIndex(item => item.cli === worker.cli) === index);
  return <Page chapter="One job. Your agents.">
    <Reveal at={0} style={{position: "absolute", left: 88, top: 196, display: "flex", alignItems: "center", gap: 44}}>
      <Img src={staticFile("logo-mark.png")} style={{width: 134, height: 134, objectFit: "contain"}} />
      <div style={{fontSize: 176, lineHeight: 1.05, fontWeight: 650}}>Pytxo.</div>
    </Reveal>
    <Reveal at={28} style={{position: "absolute", top: 428, left: 92, fontSize: 64, lineHeight: 1.2}}>Let your agents work together.</Reveal>
    <div style={{position: "absolute", left: 92, top: 618, height: 1, width: 1736 * progress(frame, 36, 94), background: line}} />
    <div style={{position: "absolute", top: 680, left: 92, right: 92, display: "flex", justifyContent: "space-between"}}>
      {workers.map((worker, index) => <Reveal key={worker.cli} at={54 + index * 9} style={{display: "flex", alignItems: "center", gap: 18}}>
        <AgentLogo cli={worker.cli} size={42} /><span style={{fontSize: 31, fontWeight: 550, color: ink}}>{worker.vendor === "OpenAI Codex" ? "Codex" : worker.vendor}</span>
      </Reveal>)}
    </div>
    <Reveal at={104} style={{position: "absolute", left: 92, top: 832, fontSize: 34, color: muted}}>One request. Separate workspaces. A result you review.</Reveal>
  </Page>;
};

export const Request = () => {
  const frame = useCurrentFrame();
  const tasks = ["Search & filtering", "A dark theme", "Spanish translation", "Filter controls", "Input validation", "Documentation"];
  return <Page chapter="01 / The request">
    <Reveal style={{position: "absolute", left: 88, top: 180, fontSize: 94, fontWeight: 600, lineHeight: 1.1}}>One task board.<br />Six pieces of work.</Reveal>
    <div style={{position: "absolute", left: 96, right: 96, top: 475, display: "grid", gridTemplateColumns: "1fr 1fr", gap: "0 96px"}}>
      {tasks.map((task, index) => <Reveal at={24 + index * 12} key={task} style={{display: "flex", alignItems: "center", gap: 26, padding: "30px 0", borderBottom: `1px solid ${line}`, fontSize: 42}}>
        <span style={{color: muted, fontSize: 30, minWidth: 44}}>{String(index + 1).padStart(2, "0")}</span>{task}
      </Reveal>)}
    </div>
    <div style={{position: "absolute", left: 96, top: 913, height: 3, width: 1728 * progress(frame, 60, 250), background: ink}} />
  </Page>;
};
