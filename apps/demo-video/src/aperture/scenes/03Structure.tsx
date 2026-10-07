import {interpolate, useCurrentFrame} from 'remotion';
import type {ApertureEvidence} from '../config';
import {Frame, Eyebrow, Reveal} from '../components/Frame';
import {clamp, ease, ink} from '../style';

const Trace = ({d, start, duration, color = ink.mint}: {d: string; start: number; duration: number; color?: string}) => {
  const frame = useCurrentFrame();
  return <>
    <path d={d} fill="none" stroke={ink.line} strokeWidth="1.4" />
    <path d={d} fill="none" stroke={color} strokeWidth="2.8" pathLength="1" strokeDasharray="1" strokeDashoffset={interpolate(frame, [start, start + duration], [1, 0], {...clamp, easing: ease})} />
  </>;
};

const Node = ({x, y, n, label, scope, delay, color = ink.mint}: {x: number; y: number; n: string; label: string; scope: string; delay: number; color?: string}) => {
  const frame = useCurrentFrame();
  const reveal = interpolate(frame, [delay, delay + 30], [0, 1], clamp);
  return <g opacity={reveal} transform={`translate(${x},${y + (1 - reveal) * 14})`}>
    <circle r="26" fill={ink.bg} stroke={color} strokeWidth="1.5" />
    <text textAnchor="middle" dominantBaseline="central" fill={color} fontSize="19">{n}</text>
    <text x="-2" y="81" textAnchor="middle" fill={ink.paper} fontSize="37" fontWeight="500" letterSpacing="-1">{label}</text>
    <text x="-2" y="120" textAnchor="middle" fill={ink.secondary} fontSize="21">{scope}</text>
  </g>;
};

export const Structure = ({evidence}: {evidence: ApertureEvidence}) => {
  const frame = useCurrentFrame();
  return (
    <Frame chapter="03 / 07" label="Run structure / diagram" evidence={evidence} footer={`Plan ${evidence.dependencyEvidence.sha256.slice(0, 10)} · diagram with compressed timing; no speed comparison`}>
      <div style={{position: 'absolute', left: 88, top: 173}}>
        <Eyebrow>One {evidence.harness} harness</Eyebrow>
        <Reveal delay={9} style={{marginTop: 27, fontSize: 80, fontWeight: 540, letterSpacing: '-0.055em'}}>Independent work. In order.</Reveal>
      </div>
      <svg width="1744" height="590" viewBox="0 0 1744 590" style={{position: 'absolute', left: 88, top: 351, overflow: 'visible'}}>
        <path d="M362 23 H763 M832 23 H1192 M1258 23 H1672" stroke={ink.line}/>
        <text x="362" y="-1" fontSize="19" fill={ink.secondary} letterSpacing="1.3">WAVE 01</text>
        <text x="832" y="-1" fontSize="19" fill={ink.secondary} letterSpacing="1.3">WAVE 02</text>
        <text x="1258" y="-1" fontSize="19" fill={ink.secondary} letterSpacing="1.3">COMBINED CANDIDATE</text>
        <Trace d="M115 266 H244 C284 266 277 133 323 133 H496" start={25} duration={55} />
        <Trace d="M115 266 H244 C284 266 277 374 323 374 H496" start={25} duration={55} color={ink.cyan} />
        <Trace d="M548 133 H678 C746 133 714 266 804 266 H958" start={150} duration={70} />
        <Trace d="M548 374 H704 C758 374 750 430 810 430 H1168 C1230 430 1210 374 1277 374" start={235} duration={115} color={ink.cyan} />
        <Trace d="M1010 266 H1277" start={270} duration={80} />
        <text x="790" y="220" fill={ink.secondary} fontSize="17" letterSpacing="0.8">DEPENDS ON CODE</text>
        <text x="887" y="467" fill={ink.secondary} fontSize="17" letterSpacing="0.8">DOCS → PACKAGE INPUT</text>
        <g opacity={interpolate(frame, [0, 24], [0, 1], clamp)}>
          <circle cx="89" cy="266" r="26" fill={ink.bg} stroke={ink.secondary} strokeWidth="1.5"/>
          <path d="M82 254 97 266 82 278" stroke={ink.paper} fill="none" strokeWidth="2"/>
          <text x="86" y="347" textAnchor="middle" fontSize="31" fill={ink.paper}>{evidence.harness}</text>
          <text x="86" y="385" textAnchor="middle" fontSize="21" fill={ink.secondary}>one harness</text>
        </g>
        <Node x={522} y={133} n="01" label="Code" scope={evidence.scopePaths.code} delay={65}/>
        <Node x={522} y={374} n="02" label="Documentation" scope={evidence.scopePaths.docs} delay={65} color={ink.cyan}/>
        <Node x={984} y={266} n="03" label="Tests" scope={evidence.scopePaths.tests} delay={210}/>
        <g opacity={interpolate(frame, [330, 360], [0, 1], clamp)}>
          <path d="M1277 156 1555 156 1634 235 1634 390 1277 390Z" fill="#101a1c" stroke={ink.mint} strokeWidth="1.5"/>
          <path d="M1555 156 V235 H1634" stroke={ink.mint} fill="none" opacity="0.45"/>
          <text x="1310" y="211" fill={ink.mint} fontSize="20" letterSpacing="1">PREPARED</text>
          <text x="1310" y="286" fill={ink.paper} fontSize="44" fontWeight="500" letterSpacing="-1.5">One package.</text>
          <text x="1310" y="339" fill={ink.secondary} fontSize="24">Ready for exact-diff review.</text>
        </g>
      </svg>
      <Reveal delay={380} style={{position: 'absolute', left: 88, bottom: 101, fontSize: 26, color: ink.secondary}}>
        <span style={{color: ink.paper}}>3 scoped tasks · 2 waves · up to 2 workers.</span> Primary checkout unchanged before Apply.
      </Reveal>
    </Frame>
  );
};
