import {Audio} from "@remotion/media";
import {AbsoluteFill, Sequence, staticFile} from "remotion";
import {Apply} from "./Apply";
import {Boundary} from "./Boundary";
import {Closing} from "./Closing";
import {Dispatch} from "./Dispatch";
import {C, EvidenceNote, type FilmProps} from "./design";
import {Merge} from "./Merge";
import {Opening} from "./Opening";

export const PytxoMotionFilm = (props: FilmProps) => <AbsoluteFill style={{background: C.paper, fontFamily: "'Geist Variable', sans-serif", letterSpacing: 0}}>
  <Sequence name="Your agents, one workspace" from={0} durationInFrames={240}><Opening {...props} /></Sequence>
  <Sequence name="Split, run, hand off" from={240} durationInFrames={900}><Dispatch {...props} /></Sequence>
  <Sequence name="Converge into review" from={1140} durationInFrames={420}><Merge {...props} /></Sequence>
  <Sequence name="The boundary holds" from={1560} durationInFrames={480}><Boundary {...props} /></Sequence>
  <Sequence name="Recheck, review, Apply" from={2040} durationInFrames={720}><Apply {...props} /></Sequence>
  <Sequence name="Keep the final say" from={2760} durationInFrames={240}><Closing /></Sequence>
  <EvidenceNote />
  <Audio src={staticFile("audio/cc0/motion-mix.wav")} />
</AbsoluteFill>;
