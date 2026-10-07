import {useCurrentFrame} from "remotion";
import {C, Document, Label, MaskText, mix, move, Native, Package, Stage, type FilmProps} from "./design";

export const Merge = (props: FilmProps) => {
  const f = useCurrentFrame();
  const converge = move(f, 132, 223);
  const reveal = move(f, 245, 300);
  return <Stage>
    <div style={{position: "absolute", top: 102, left: 88}}><MaskText size={100}>One package.</MaskText><MaskText at={28} size={100} style={{color: C.green}}>Yours to review.</MaskText></div>
    {props.files.map((file, i) => <Document key={file.path} number={i + 1} label={file.path.split("/").at(-1)!} contentOpacity={1 - move(f, 151, 177)} style={{left: mix(96 + (i < 4 ? i : i - 4 + 0.5) * 430, 620 + i * 9, converge), top: mix(i < 4 ? 385 : 640, 455 - i * 8, converge) + (1 - move(f, i * 8, i * 8 + 40)) * 100, width: 380, height: 190, transform: `rotate(${mix(i % 2 ? 1.4 : -1.4, (i - 3) * 1.5, converge)}deg)`, opacity: move(f, i * 8, i * 8 + 36) * (1 - move(f, 207, 234))}} />)}
    <Package style={{left: 620, top: 445, opacity: move(f, 214, 245) * (1 - reveal), transform: `scale(${mix(0.92, 1, move(f, 210, 248))})`}} />
    <div style={{position: "absolute", top: 438, left: 96, clipPath: `inset(0 ${(1 - reveal) * 100}% 0 0)`}}>
      <Native props={props} assetKey="review" x={744} y={294} cropWidth={760} width={1728} height={252} style={{borderRadius: 5}} />
    </div>
    <Label style={{position: "absolute", left: 96, top: 798, fontSize: 38, opacity: move(f, 300, 336)}}>7 prepared files. 6 recorded check commands passed.</Label>
    <div style={{position: "absolute", left: 96, top: 942, width: 1728, height: 3, background: C.ink, transformOrigin: "left", transform: `scaleX(${move(f, 256, 384)})`}} />
  </Stage>;
};
