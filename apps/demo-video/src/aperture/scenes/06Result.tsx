import type {ApertureFilmProps} from '../config';
import {Frame, Eyebrow, Reveal} from '../components/Frame';
import {NativeFootage} from '../components/NativeFootage';
import {ink} from '../style';

export const Result = ({evidence, clips, privacyEdits}: ApertureFilmProps) => (
  <Frame chapter="06 / 07" label="Observed post-state" evidence={evidence} footer="One recorded mission · source-bound evidence; no OS-wide sandbox or comparative speed claim">
    <div style={{position: 'absolute', left: 88, top: 185}}>
      <Eyebrow>Measured after Apply</Eyebrow>
      <Reveal delay={12} style={{fontSize: 87, lineHeight: 1.02, letterSpacing: '-0.055em', fontWeight: 550, marginTop: 33}}>
        The bytes<br /><span style={{color: ink.mint}}>match.</span>
      </Reveal>
      <Reveal delay={28} style={{marginTop: 34, fontSize: 27, lineHeight: 1.5, color: ink.secondary, maxWidth: 515}}>
        {evidence.changedFilesMatchingPackage} changed file hashes matched<br />the reviewed package.
      </Reveal>
      <Reveal delay={55} style={{marginTop: 47, paddingTop: 27, borderTop: `1px solid ${ink.line}`, maxWidth: 505}}>
        <div style={{fontSize: 96, lineHeight: 1, letterSpacing: '-0.065em', fontWeight: 520}}>{evidence.independentChecksPassed}<span style={{fontSize: 27, letterSpacing: '-0.015em', marginLeft: 21, color: ink.secondary}}>independent checks</span></div>
        <div style={{fontSize: 24, color: ink.secondary, marginTop: 19}}>{evidence.repositoryTestsPassed} repository tests also passed.</div>
      </Reveal>
      {evidence.receiptSurvivedRestart ? <Reveal delay={110} style={{fontSize: 25, color: ink.mint, marginTop: 40}}>Receipt verified after restart.</Reveal> : null}
    </div>
    <div style={{position: 'absolute', left: 683, top: 161, color: ink.secondary, fontSize: 21}}>Native result · {clips.result.playbackRate}× playback{privacyEdits.result.regions.length ? ' · host path redacted' : ''}</div>
    <NativeFootage clip={clips.result} privacy={privacyEdits.result} seconds={8} viewport={{x: 676, y: 213, width: 1156, height: 713}} />
    <div style={{position: 'absolute', left: 688, top: 946, fontSize: 20, letterSpacing: '0.01em', color: ink.secondary}}>Package {evidence.applyPackageId.slice(0, 16)} · {evidence.combinedChecksPassed} combined-candidate checks passed</div>
  </Frame>
);
