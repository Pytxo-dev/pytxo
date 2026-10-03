import {Img, staticFile, useCurrentFrame} from "remotion";
import {C, Label, MaskText, move, Stage} from "./design";

export const Closing = () => {
  const f = useCurrentFrame();
  return <Stage>
    <div style={{position: "absolute", left: 94, top: 87, display: "flex", alignItems: "center", gap: 24}}><Img src={staticFile("logo-mark.png")} style={{width: 72, height: 72}} /><div style={{fontSize: 76, fontWeight: 620}}>Pytxo.</div></div>
    <div style={{position: "absolute", top: 335, left: 88}}><MaskText size={120}>Give agents the work.</MaskText><MaskText at={23} size={120}>Keep the final say.</MaskText></div>
    <div style={{position: "absolute", left: 98, top: 638, width: 1115, height: 8, background: C.green, transform: `scaleX(${move(f, 53, 106)})`, transformOrigin: "left"}} />
    <div style={{position: "absolute", left: 96, top: 808, fontSize: 46, fontWeight: 500}}>pytxo.com</div>
    <Label style={{position: "absolute", right: 96, top: 822}}>Windows / Beta candidate</Label>
  </Stage>;
};
