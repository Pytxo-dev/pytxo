import {Img, staticFile, useCurrentFrame} from "remotion";
import {amber, green, ink, muted, NativeCrop, Page, progress, Reveal, type FilmProps} from "./shared";

export const Decision = (props: FilmProps & {applied: boolean}) => {
  const frame = useCurrentFrame();
  return <Page chapter={props.applied ? "05 / Your decision" : "04 / The boundary"}>
    <Reveal style={{position: "absolute", left: 88, top: 174, fontSize: 94, fontWeight: 600, lineHeight: 1.12}}>
      {props.applied ? "Rechecked. Reviewed." : "The project changed."}<br />
      <span style={{color: props.applied ? green : amber}}>{props.applied ? "Applied." : "Apply stopped."}</span>
    </Reveal>
    <Reveal at={36} style={{position: "absolute", left: 96, top: 448, fontSize: 40, color: muted, maxWidth: 1630, lineHeight: 1.45}}>
      {props.applied ? `Exactly ${props.files.length} reviewed files entered the task board.` : "A file added after review made the package stale."}
    </Reveal>
    <Reveal at={58} style={{position: "absolute", left: 88, top: 550}}>
      <div style={{fontSize: 28, color: muted, marginBottom: 24}}>Recorded Desktop / {props.applied ? "Apply receipt" : "stale-package refusal"}</div>
      <NativeCrop props={props} assetKey={props.applied ? "applied" : "stale"} crop={{x: 260, y: 817, width: 970}} width={1664} height={324} style={{transform: `scale(${1 + progress(frame, 90, 280) * 0.008})`, transformOrigin: "left center"}} />
    </Reveal>
  </Page>;
};

export const Result = (props: FilmProps) => <Page chapter="06 / After Apply" dark>
  <Reveal style={{position: "absolute", left: 88, top: 176, fontSize: 86, fontWeight: 600}}>Work becomes part of your project.</Reveal>
  <Reveal at={24} style={{position: "absolute", left: 296, top: 345}}>
    <NativeCrop props={props} assetKey="result" crop={{x: 510, y: 65, width: 910}} width={1328} height={555} />
  </Reveal>
</Page>;

export const Closing = () => <Page chapter="Your agents. Your project.">
  <Reveal style={{position: "absolute", left: 88, top: 200, display: "flex", alignItems: "center", gap: 40}}>
    <Img src={staticFile("logo-mark.png")} style={{width: 126, height: 126}} /><div style={{fontSize: 160, fontWeight: 650}}>Pytxo.</div>
  </Reveal>
  <Reveal at={32} style={{position: "absolute", left: 96, top: 472, fontSize: 78, lineHeight: 1.2, fontWeight: 500}}>Give agents the work.<br /><span style={{color: green}}>Keep the final say.</span></Reveal>
  <Reveal at={78} style={{position: "absolute", left: 96, top: 810, fontSize: 42, color: ink}}>pytxo.com</Reveal>
  <Reveal at={96} style={{position: "absolute", right: 96, top: 822, fontSize: 28, color: muted}}>Windows Desktop / Local beta candidate</Reveal>
</Page>;
