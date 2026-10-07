import '@fontsource-variable/geist';
import {AbsoluteFill, Sequence} from 'remotion';
import {assertApertureEvidence, type ApertureFilmProps} from './config';
import {Opening} from './scenes/01Opening';
import {Mission} from './scenes/02Mission';
import {Structure} from './scenes/03Structure';
import {Review} from './scenes/04Review';
import {Apply} from './scenes/05Apply';
import {Result} from './scenes/06Result';
import {End} from './scenes/07End';
import {ink} from './style';

export {FILM_DURATION, FILM_FPS, parseApertureFilmProps, type ApertureFilmProps} from './config';

/** An authored 58-second film; its native intervals must be backed by the capture manifest. */
export const PytxoApertureFilm = (props: ApertureFilmProps) => {
  assertApertureEvidence(props);
  return (
    <AbsoluteFill style={{background: ink.bg}}>
      <Sequence from={0} durationInFrames={300} name="00–05 · Aperture / agent hypervisor"><Opening evidence={props.evidence}/></Sequence>
      <Sequence from={300} durationInFrames={420} name="05–12 · Credentials review rule"><Mission evidence={props.evidence}/></Sequence>
      <Sequence from={720} durationInFrames={540} name="12–21 · Run structure / diagram"><Structure evidence={props.evidence}/></Sequence>
      <Sequence from={1260} durationInFrames={780} name="21–34 · Native exact-diff review"><Review {...props}/></Sequence>
      <Sequence from={2040} durationInFrames={660} name="34–45 · Native explicit Apply"><Apply {...props}/></Sequence>
      <Sequence from={2700} durationInFrames={480} name="45–53 · Observed result"><Result {...props}/></Sequence>
      <Sequence from={3180} durationInFrames={300} name="53–58 · Identity / evidence"><End evidence={props.evidence}/></Sequence>
    </AbsoluteFill>
  );
};
