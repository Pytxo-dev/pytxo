import {AbsoluteFill, Sequence} from "remotion";
import {Closing, Decision, Result} from "./Decision";
import {Fleet, Package} from "./Fleet";
import {Opening, Request} from "./Opening";
import {paper, type FilmProps} from "./shared";

export const PytxoBetaFilm = (props: FilmProps) => (
  <AbsoluteFill style={{background: paper}}>
    <Sequence name="Pytxo" from={0} durationInFrames={300}><Opening {...props} /></Sequence>
    <Sequence name="One request" from={300} durationInFrames={360}><Request /></Sequence>
    <Sequence name="The recorded fleet" from={660} durationInFrames={540}><Fleet {...props} /></Sequence>
    <Sequence name="One checked package" from={1200} durationInFrames={480}><Package {...props} /></Sequence>
    <Sequence name="Stale refusal" from={1680} durationInFrames={420}><Decision {...props} applied={false} /></Sequence>
    <Sequence name="Reviewed Apply" from={2100} durationInFrames={420}><Decision {...props} applied /></Sequence>
    <Sequence name="The result" from={2520} durationInFrames={420}><Result {...props} /></Sequence>
    <Sequence name="Keep the final say" from={2940} durationInFrames={420}><Closing /></Sequence>
  </AbsoluteFill>
);
