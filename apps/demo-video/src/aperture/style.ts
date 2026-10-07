import {Easing} from 'remotion';

export const ink = {
  bg: '#090c10',
  paper: '#f3f0e8',
  secondary: '#adb7bd',
  line: '#30393f',
  mint: '#81eed2',
  cyan: '#6ab6ec',
  rose: '#dc9fad',
  muted: '#65727a',
};
export const font = 'Geist Variable, Geist, sans-serif';
export const clamp = {extrapolateLeft: 'clamp', extrapolateRight: 'clamp'} as const;
export const ease = Easing.bezier(0.16, 1, 0.3, 1);
export const easeInOut = Easing.bezier(0.76, 0, 0.24, 1);
export const bounds = {left: 88, right: 1832, top: 66, bottom: 1014};
