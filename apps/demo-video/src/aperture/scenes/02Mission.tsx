import {interpolate, useCurrentFrame} from 'remotion';
import type {ApertureEvidence} from '../config';
import {Frame, Eyebrow, Reveal} from '../components/Frame';
import {clamp, ease, ink} from '../style';

export const Mission = ({evidence}: {evidence: ApertureEvidence}) => {
  const frame = useCurrentFrame();
  const progress = interpolate(frame, [85, 210], [0, 1], {...clamp, easing: ease});
  return (
    <Frame chapter="02 / 07" label="The recorded task" evidence={evidence}>
      <div style={{position: 'absolute', left: 88, top: 208}}>
        <Eyebrow>One change, across the project</Eyebrow>
        <Reveal delay={8} style={{fontSize: 112, fontWeight: 550, letterSpacing: '-0.06em', lineHeight: 1.04, marginTop: 36}}>
          Credentials paths.<br /><span style={{color: ink.mint}}>Require review.</span>
        </Reveal>
        <Reveal delay={36} style={{fontSize: 31, color: ink.secondary, lineHeight: 1.42, marginTop: 36}}>Change the rule. Explain it.<br />Then test the combined behavior.</Reveal>
      </div>
      <svg width="760" height="720" viewBox="0 0 760 720" style={{position: 'absolute', right: 48, top: 208}}>
        <defs><linearGradient id="mission-edge"><stop stopColor={ink.mint}/><stop offset="1" stopColor={ink.cyan}/></linearGradient></defs>
        <path d="M74 80 H602 L692 170 V598 H74 Z" fill="#0d1318" stroke={ink.line}/>
        <path d="M602 80 V170 H692" fill="none" stroke={ink.line}/>
        <path d="M74 80 H602 L692 170 V598 H74 Z" fill="none" stroke="url(#mission-edge)" pathLength="1" strokeDasharray="1" strokeDashoffset={1 - progress} strokeWidth="2" />
        <text x="112" y="140" fill={ink.secondary} fontSize="19" letterSpacing="2">TASK BRIEF / DIAGRAM</text>
        <text x="112" y="268" fill={ink.paper} fontSize="46" fontWeight="500" letterSpacing="-1.6">credentials/</text>
        <path d="M114 305 H626" stroke={ink.line}/>
        <g opacity={interpolate(frame, [125, 160], [0, 1], clamp)}>
          <text x="112" y="364" fill={ink.secondary} fontSize="23">REQUESTED REVIEW RULE</text>
          <text x="112" y="425" fill={ink.mint} fontSize="47" letterSpacing="-1.5">Required</text>
          <text x="112" y="533" fill={ink.secondary} fontSize="24">Code + documentation + tests</text>
        </g>
      </svg>
      <Reveal delay={90} style={{position: 'absolute', left: 88, bottom: 166, fontSize: 27, color: ink.secondary}}>
        Recorded with <span style={{color: ink.paper}}>{evidence.harness} {evidence.harnessVersion}</span>
      </Reveal>
    </Frame>
  );
};
