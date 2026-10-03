import {Img, staticFile, useCurrentFrame} from "remotion";
import {C, Label, MaskText, move, Native, Stage, type FilmProps} from "./design";

export const Opening = (props: FilmProps) => {
  const f = useCurrentFrame();
  const exit = move(f, 200, 239);
  return <Stage>
    <div style={{position: "absolute", left: 88, top: 68, display: "flex", alignItems: "center", gap: 30, transform: `translateY(${-exit * 370}px)`}}>
      <Img src={staticFile("logo-mark.png")} style={{width: 110, height: 110}} />
      <MaskText size={172}>Pytxo.</MaskText>
    </div>
    <div style={{position: "absolute", left: 1080, top: 94, transform: `translateY(${-exit * 370}px)`}}><MaskText at={20} size={66}>Your agents.</MaskText><MaskText at={36} size={66}>One workspace.</MaskText></div>
    <div style={{position: "absolute", left: 88, top: 336 - exit * 85, width: 1744, height: 588, overflow: "hidden", borderRadius: 6, transform: `translateY(${(1 - move(f, 0, 56)) * 60}px)`, opacity: 1 - exit}}>
      <Native props={props} assetKey="board-4" x={270} y={348} cropWidth={778} width={1744} height={588} />
    </div>
    <Label style={{position: "absolute", top: 274, left: 92, opacity: 1 - exit}}>One place to follow the work.</Label>
    <div style={{position: "absolute", left: 88, right: 88, bottom: 111, height: 3, background: C.ink, transformOrigin: "left", transform: `scaleX(${move(f, 24, 108) * (1 - exit)})`}} />
  </Stage>;
};
