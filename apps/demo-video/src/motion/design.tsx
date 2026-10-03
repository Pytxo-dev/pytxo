import type {CSSProperties, ReactNode} from "react";
import {AbsoluteFill, Easing, Img, interpolate, staticFile, useCurrentFrame} from "remotion";
import type {PytxoFleetFilmProps} from "../fleet/PytxoFleetFilm";

export type FilmProps = PytxoFleetFilmProps;
export const FPS = 60;
export const FRAMES = 3000;
export const C = {paper: "#fafafa", ink: "#202222", muted: "#646969", rule: "#d5d9d7", green: "#186b53", wash: "#e7f1ec", coral: "#b44934"};
const curve = Easing.bezier(0.76, 0, 0.24, 1);
export const move = (f: number, a: number, b: number) => interpolate(f, [a, b], [0, 1], {extrapolateLeft: "clamp", extrapolateRight: "clamp", easing: curve});
export const mix = (a: number, b: number, t: number) => a + (b - a) * t;

export const Stage = ({children, dark = false}: {children: ReactNode; dark?: boolean}) => <AbsoluteFill style={{background: dark ? C.ink : C.paper, color: dark ? C.paper : C.ink, fontFamily: "'Geist Variable', sans-serif", letterSpacing: 0, overflow: "hidden"}}>{children}</AbsoluteFill>;

export const Label = ({children, style}: {children: ReactNode; style?: CSSProperties}) => <div style={{fontSize: 32, lineHeight: 1.3, color: C.muted, ...style}}>{children}</div>;

export const MaskText = ({children, at = 0, size = 100, style}: {children: ReactNode; at?: number; size?: number; style?: CSSProperties}) => {
  const f = useCurrentFrame();
  return <div style={{overflow: "hidden", fontSize: size, fontWeight: 580, lineHeight: 1.12, ...style}}><div style={{transform: `translateY(${(1 - move(f, at, at + 42)) * 112}%)`}}>{children}</div></div>;
};

export const Native = ({props, assetKey, x, y, cropWidth, width, height, style}: {props: FilmProps; assetKey: string; x: number; y: number; cropWidth: number; width: number; height: number; style?: CSSProperties}) => {
  const asset = props.assets.find(item => item.key === assetKey);
  if (!asset) throw new Error(`Missing native evidence ${assetKey}`);
  const scale = width / cropWidth;
  return <div style={{position: "relative", width, height, background: "#08090a", overflow: "hidden", ...style}}><Img src={staticFile(asset.path)} style={{position: "absolute", left: -x * scale, top: -y * scale, width: asset.width * scale, height: asset.height * scale, maxWidth: "none"}} /></div>;
};

export const Document = ({label, style, number}: {label: string; style?: CSSProperties; number?: number}) => <div style={{position: "absolute", width: 250, height: 190, background: "white", border: `2px solid ${C.ink}`, borderRadius: 4, padding: 24, boxSizing: "border-box", ...style}}>
  <div style={{display: "flex", justifyContent: "space-between", alignItems: "center", marginBottom: 24}}><div style={{width: 42, height: 3, background: C.ink}} /><span style={{fontSize: 28, color: C.muted}}>{number ? String(number).padStart(2, "0") : "M"}</span></div>
  <div style={{fontSize: 32, lineHeight: 1.25, overflowWrap: "anywhere", fontWeight: 500}}>{label}</div>
</div>;

export const Package = ({style, refreshed = false}: {style?: CSSProperties; refreshed?: boolean}) => <div style={{position: "absolute", width: 680, height: 216, background: C.paper, border: `3px solid ${refreshed ? C.green : C.ink}`, borderRadius: 4, padding: "30px 36px", boxSizing: "border-box", ...style}}>
  <Label style={{color: refreshed ? C.green : C.muted}}>{refreshed ? "Rechecked package" : "Review package"}</Label>
  <div style={{fontSize: 76, fontWeight: 550, marginTop: 4}}>7 exact files<span style={{fontSize: 50, float: "right", paddingTop: 14}}>{refreshed ? "+" : ""}</span></div>
</div>;

export const EvidenceNote = () => <div style={{position: "absolute", bottom: 0, left: 0, right: 0, padding: "24px 72px 28px", background: C.paper, display: "flex", justifyContent: "space-between", fontSize: 30, lineHeight: 1.3, color: C.muted}}><span>Recorded 2 Oct 2026 / Edited native stills</span><span>Pytxo Desktop</span></div>;
