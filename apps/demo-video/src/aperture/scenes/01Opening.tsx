import {interpolate, useCurrentFrame} from 'remotion';
import type {ApertureEvidence} from '../config';
import {Frame, Reveal, Eyebrow} from '../components/Frame';
import {Optics} from '../components/Optics';
import {clamp, ease, ink} from '../style';

export const Opening = ({evidence}: {evidence: ApertureEvidence}) => {
  const frame = useCurrentFrame();
  return (
    <Frame chapter="01 / 07" label="Agent hypervisor" evidence={evidence}>
      <div style={{position: 'absolute', left: 985, top: -30, opacity: interpolate(frame, [0, 45], [0, 1], clamp), scale: interpolate(frame, [0, 180], [1.08, 1], {...clamp, easing: ease})}}>
        <Optics size={1100} open={interpolate(frame, [12, 132], [0, 1], {...clamp, easing: ease})} />
      </div>
      <div style={{position: 'absolute', left: 88, top: 261}}>
        <Eyebrow delay={15}>One harness. More agency.</Eyebrow>
        <Reveal delay={25} style={{marginTop: 44, fontSize: 132, fontWeight: 560, lineHeight: 0.99, letterSpacing: '-0.065em'}}>
          Keep your agent.<br /><span style={{color: ink.mint}}>Take control.</span>
        </Reveal>
        <Reveal delay={65} style={{marginTop: 48, fontSize: 33, color: ink.secondary, letterSpacing: '-0.02em'}}>Prepare. Run. Inspect. Apply.</Reveal>
      </div>
      <div style={{position: 'absolute', left: 88, top: 838, width: interpolate(frame, [80, 180], [0, 930], {...clamp, easing: ease}), height: 1, background: `linear-gradient(90deg, ${ink.mint}, ${ink.line})`}} />
    </Frame>
  );
};
