import {interpolate, useCurrentFrame} from 'remotion';
import type {ApertureEvidence} from '../config';
import {Frame, Eyebrow, Reveal} from '../components/Frame';
import {Optics} from '../components/Optics';
import {clamp, ease, ink} from '../style';

export const End = ({evidence}: {evidence: ApertureEvidence}) => {
  const frame = useCurrentFrame();
  return (
    <Frame chapter="07 / 07" label="Pytxo · agent hypervisor" evidence={evidence} footer={`${evidence.buildLabel} · recorded ${evidence.capturedAt} · 60 fps film`}>
      <div style={{position: 'absolute', left: 995, top: -45, opacity: 0.7}}><Optics size={1080} phase={60} open={interpolate(frame, [0, 155], [1, 0.55], {...clamp, easing: ease})}/></div>
      <div style={{position: 'absolute', left: 88, top: 249}}>
        <Eyebrow>Your agents. Your authority.</Eyebrow>
        <Reveal delay={10} style={{fontSize: 133, fontWeight: 560, lineHeight: 1.02, letterSpacing: '-0.065em', marginTop: 37}}>
          Keep your agent.<br /><span style={{color: ink.mint}}>Add Pytxo.</span>
        </Reveal>
        <Reveal delay={36} style={{marginTop: 46, fontSize: 36, letterSpacing: '-0.025em', color: ink.secondary}}>pytxo.com</Reveal>
        <Reveal delay={55} style={{marginTop: 54, fontSize: 24, color: ink.secondary, maxWidth: 1230, lineHeight: 1.5}}>
          <span style={{color: ink.paper}}>Evidence + limitations</span><br />{evidence.evidenceLocationLabel}
        </Reveal>
      </div>
    </Frame>
  );
};
