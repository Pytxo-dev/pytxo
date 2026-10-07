import {useCurrentFrame} from "remotion";
import {C, Label, MaskText, mix, move, Native, Package, Stage, type FilmProps} from "./design";

export const Boundary = (props: FilmProps) => {
  const f = useCurrentFrame();
  const evidence = move(f, 260, 320);
  return <Stage>
    <div style={{position: "absolute", top: 106, left: 88}}><MaskText size={96}>The project changed.</MaskText><MaskText at={130} size={96} style={{color: C.coral}}>Apply stopped.</MaskText></div>
    <div style={{opacity: 1 - evidence}}>
      <div style={{position: "absolute", left: 1360, top: 368, width: 4, height: 480 * move(f, 0, 64), background: C.ink}} />
      <Label style={{position: "absolute", left: 1430, top: 743}}>Your project</Label>
      <div style={{position: "absolute", left: 1418, top: 403 + (1 - move(f, 90, 135)) * -100, width: 352, padding: "26px 24px", boxSizing: "border-box", border: `2px solid ${C.coral}`, color: C.coral, fontSize: 36, opacity: move(f, 90, 122), transform: "rotate(3deg)"}}>A new file</div>
      <Package style={{left: mix(96, 655, move(f, 24, 128)), top: 510}} />
      <div style={{position: "absolute", left: 1356, top: 476, width: 12, height: 278, background: C.coral, transform: `scaleY(${move(f, 128, 151)})`}} />
      <Label style={{position: "absolute", top: 789, left: 96, opacity: move(f, 164, 202)}}>Review no longer matched the project.</Label>
    </div>
    <div style={{position: "absolute", left: 96, top: 476, transform: `translateY(${(1 - evidence) * 220}px)`, opacity: evidence}}>
      <Label style={{marginBottom: 22}}>Recorded Desktop / stale-package refusal</Label>
      <Native props={props} assetKey="stale" x={272} y={821} cropWidth={990} width={1728} height={326} style={{borderRadius: 5}} />
    </div>
    <div style={{position: "absolute", left: 96, top: 916, width: 1728 * move(f, 340, 420), height: 3, background: C.coral}} />
  </Stage>;
};
