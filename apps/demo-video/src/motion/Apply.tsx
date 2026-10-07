import {useCurrentFrame} from "remotion";
import {C, Label, MaskText, mix, move, Native, Package, Stage, type FilmProps} from "./design";

export const Apply = (props: FilmProps) => {
  const f = useCurrentFrame();
  const crossing = move(f, 150, 235);
  const receipt = move(f, 257, 301);
  const result = move(f, 485, 546);
  const title = f < 80 ? "Rechecked." : f < 164 ? "Reviewed." : "Applied.";
  return <Stage>
    <div style={{position: "absolute", top: 100, left: 88, opacity: 1 - result}} key={title}><MaskText at={f < 80 ? 0 : f < 164 ? 80 : 164} size={132} style={{color: f >= 164 ? C.green : C.ink}}>{title}</MaskText></div>
    <Label style={{position: "absolute", left: 96, top: 294, opacity: move(f, 198, 235) * (1 - result)}}>Exactly the seven reviewed files.</Label>
    <div style={{opacity: 1 - receipt}}>
      <Package refreshed style={{left: mix(655, 1450, crossing), top: 510, opacity: 1 - move(f, 213, 250)}} />
      <div style={{position: "absolute", left: 1360, top: 368, width: 4, height: mix(480, 72, move(f, 112, 162)), background: C.green}} />
      <div style={{position: "absolute", left: 1360, bottom: 232, width: 4, height: mix(0, 72, move(f, 112, 162)), background: C.green}} />
      <Label style={{position: "absolute", left: 1430, top: 743}}>Your project</Label>
    </div>
    <div style={{position: "absolute", top: 438 - result * 110, left: 96, opacity: receipt * (1 - result), clipPath: `inset(0 ${(1 - receipt) * 100}% 0 0)`}}>
      <Label style={{marginBottom: 22}}>Recorded Desktop / Apply receipt</Label>
      <Native props={props} assetKey="applied" x={272} y={821} cropWidth={990} width={1728} height={326} style={{borderRadius: 5}} />
    </div>
    <div style={{position: "absolute", inset: "0 0 94px", background: "#1e1e1e", clipPath: `inset(${(1 - result) * 100}% 0 0 0)`}}>
      <div style={{position: "absolute", left: 92, top: 87, color: "#fafafa", fontSize: 80, fontWeight: 550, lineHeight: 1.15}}>Now part of<br />your project.</div>
      <Label style={{position: "absolute", left: 96, top: 373, width: 480, color: "#c3c8c5"}}>Your project, with<br />the reviewed changes.</Label>
      <Native props={props} assetKey="result" x={525} y={76} cropWidth={885} width={1190} height={713} style={{position: "absolute", left: 650, top: 106 + (1 - result) * 90}} />
      <div style={{position: "absolute", left: 96, right: 96, bottom: 90, height: 2, background: "#737976", transform: `scaleX(${move(f, 548, 658)})`, transformOrigin: "left"}} />
    </div>
  </Stage>;
};
