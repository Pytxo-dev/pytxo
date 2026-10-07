import {useCurrentFrame, interpolate} from 'remotion';
import {clamp, easeInOut, ink} from '../style';

/** Editorial identity motif only. This is never rendered as a product screenshot. */
export const Optics = ({size = 1000, open = 1, phase = 0, subtle = false}: {size?: number; open?: number; phase?: number; subtle?: boolean}) => {
  const frame = useCurrentFrame();
  const sweep = interpolate(frame, [0, 300], [-28, 12], {...clamp, easing: easeInOut}) + phase;
  const aperture = 58 + 116 * open;
  return (
    <svg width={size} height={size} viewBox="0 0 1000 1000" fill="none" style={{overflow: 'visible'}}>
      <defs>
        <radialGradient id={`lens-${phase}`}>
          <stop offset="0" stopColor={ink.mint} stopOpacity="0.09" />
          <stop offset="0.54" stopColor={ink.cyan} stopOpacity="0.025" />
          <stop offset="1" stopColor={ink.bg} stopOpacity="0" />
        </radialGradient>
        <linearGradient id={`edge-${phase}`} x1="180" y1="200" x2="830" y2="760" gradientUnits="userSpaceOnUse">
          <stop stopColor={ink.cyan} /><stop offset="0.45" stopColor={ink.mint} /><stop offset="1" stopColor={ink.rose} />
        </linearGradient>
      </defs>
      <circle cx="500" cy="500" r="500" fill={`url(#lens-${phase})`} />
      <g opacity={subtle ? 0.25 : 1}>
        {[316, 325, 383, 416].map((radius, i) => <circle key={radius} cx="500" cy="500" r={radius} stroke={i === 0 ? `url(#edge-${phase})` : ink.line} strokeWidth={i === 0 ? 1.5 : 0.8} opacity={i === 0 ? 0.9 : 0.7} />)}
        <g transform={`rotate(${sweep} 500 500)`}>
          {Array.from({length: 6}, (_, i) => (
            <g key={i} transform={`rotate(${i * 60} 500 500)`}>
              <path d={`M ${500 + aperture} 500 L 658 226 L 818 408 L 659 683 Z`} fill="#10171b" fillOpacity="0.44" stroke={`url(#edge-${phase})`} strokeOpacity="0.47" strokeWidth="1.15" />
              <path d={`M ${500 + aperture + 4} 501 L 662 229`} stroke={i % 2 ? ink.rose : ink.cyan} strokeWidth="1" opacity="0.6" />
              <path d="M 828 500 H 869" stroke={ink.secondary} strokeWidth="1.2" />
              <path d="M 898 500 H 916" stroke={ink.mint} strokeWidth="2" />
            </g>
          ))}
        </g>
        {Array.from({length: 48}, (_, i) => (
          <line key={i} x1="500" y1={i % 4 === 0 ? 65 : 74} x2="500" y2="83" transform={`rotate(${i * 7.5} 500 500)`} stroke={i % 4 === 0 ? ink.secondary : ink.line} strokeWidth="1" />
        ))}
        <path d="M 480 500 H 520 M 500 480 V 520" stroke={ink.mint} strokeWidth="1" opacity="0.45" />
      </g>
    </svg>
  );
};
