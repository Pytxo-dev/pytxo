import type {ApertureFilmProps} from '../config';
import {Frame, SceneTitle} from '../components/Frame';
import {CaptureLabel, NativeFootage} from '../components/NativeFootage';
import {ink} from '../style';

export const Apply = ({evidence, clips, privacyEdits}: ApertureFilmProps) => (
  <Frame chapter="05 / 07" label="The consequential step" evidence={evidence} footer={clips.apply.cuts?.some((cut) => cut.atSeconds >= clips.apply.trimStartSeconds && cut.atSeconds < clips.apply.trimStartSeconds + 11 * clips.apply.playbackRate) ? 'Explicit Apply · source time cuts labeled at the edit points' : 'Explicit Apply · one continuous source interval; elapsed waits omitted between scenes'}>
    <SceneTitle>Your review. <span style={{color: ink.mint}}>Your Apply.</span></SceneTitle>
    <CaptureLabel clip={clips.apply} detail={privacyEdits.apply.regions.length ? 'host path redacted' : undefined} />
    <NativeFootage clip={clips.apply} privacy={privacyEdits.apply} seconds={11} />
  </Frame>
);
