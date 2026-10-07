import type {ApertureFilmProps} from '../config';
import {Frame, SceneTitle} from '../components/Frame';
import {CaptureLabel, NativeFootage} from '../components/NativeFootage';
import {ink} from '../style';

export const Review = ({evidence, clips, privacyEdits}: ApertureFilmProps) => (
  <Frame chapter="04 / 07" label="Inspect the candidate" evidence={evidence} footer={`Native ${evidence.buildLabel} · ${evidence.recordingMethod}${clips.review.cuts?.length ? ' · time cut labeled' : ' · editorial crop only'}`}>
    <SceneTitle>Read the <span style={{color: ink.mint}}>exact changes.</span></SceneTitle>
    <CaptureLabel clip={clips.review} detail={privacyEdits.review.regions.length ? 'host path redacted' : undefined} />
    <NativeFootage clip={clips.review} privacy={privacyEdits.review} seconds={13} allowCameraMove />
  </Frame>
);
