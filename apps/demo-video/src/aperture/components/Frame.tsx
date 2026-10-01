import type {ReactNode} from 'react';
import {AbsoluteFill, Img, interpolate, staticFile, useCurrentFrame} from 'remotion';
import type {ApertureEvidence} from '../config';
import {clamp, ease, font, ink} from '../style';

export const Wordmark = ({small = false}: {small?: boolean}) => (
  <div style={{display: 'flex', alignItems: 'center', gap: small ? 12 : 18, color: ink.paper}}>
    <Img src={staticFile('logo-mark.png')} style={{width: small ? 29 : 40, height: small ? 29 : 40, objectFit: 'contain'}} />
    <span style={{fontSize: small ? 26 : 36, fontWeight: 600, letterSpacing: '-0.055em'}}>Pytxo</span>
  </div>
);

export const Frame = ({children, chapter, label, evidence, footer = 'Native footage + run-structure diagrams · edited for time'}: {children: ReactNode; chapter: string; label: string; evidence: ApertureEvidence; footer?: string}) => (
  <AbsoluteFill style={{background: ink.bg, color: ink.paper, fontFamily: font, overflow: 'hidden'}}>
    {children}
    <div style={{position: 'absolute', left: 88, top: 55}}><Wordmark small /></div>
    <div style={{position: 'absolute', right: 88, top: 61, display: 'flex', alignItems: 'center', gap: 22, fontSize: 20, fontWeight: 450, letterSpacing: '0.035em', color: ink.secondary}}>
      <span style={{color: ink.mint, fontVariantNumeric: 'tabular-nums'}}>{chapter}</span><span>{label}</span>
    </div>
    <div style={{position: 'absolute', left: 88, right: 88, bottom: 39, display: 'flex', justifyContent: 'space-between', alignItems: 'center', gap: 40, color: ink.secondary, fontSize: 18, lineHeight: 1.3}}>
      <span>{footer}</span>
      <span style={{whiteSpace: 'nowrap'}}>Run {evidence.runId.slice(0, 8)} · build {evidence.executableSha256.slice(0, 10)}</span>
    </div>
  </AbsoluteFill>
);

export const Eyebrow = ({children, delay = 0}: {children: ReactNode; delay?: number}) => {
  const frame = useCurrentFrame();
  return <div style={{fontSize: 22, letterSpacing: '0.075em', textTransform: 'uppercase', color: ink.mint, opacity: interpolate(frame, [delay, delay + 24], [0, 1], clamp)}}>{children}</div>;
};

export const Reveal = ({children, delay = 0, style}: {children: ReactNode; delay?: number; style?: React.CSSProperties}) => {
  const frame = useCurrentFrame();
  return <div style={{...style, opacity: interpolate(frame, [delay, delay + 26], [0, 1], clamp), translate: `0 ${interpolate(frame, [delay, delay + 42], [28, 0], {...clamp, easing: ease})}px`}}>{children}</div>;
};

export const SceneTitle = ({children, top = 133}: {children: ReactNode; top?: number}) => (
  <div style={{position: 'absolute', left: 88, top, fontSize: 54, fontWeight: 560, letterSpacing: '-0.045em', lineHeight: 1.08}}>{children}</div>
);
