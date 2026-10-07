import type {CSSProperties, ReactNode} from "react";
import {AbsoluteFill, Easing, Img, interpolate, staticFile, useCurrentFrame} from "remotion";
import type {PytxoFleetFilmProps} from "../fleet/PytxoFleetFilm";

export type FilmProps = PytxoFleetFilmProps;
export const FPS = 60;
export const FRAMES = 3360;
export const ink = "#191c1e";
export const paper = "#f7f8f8";
export const muted = "#60676b";
export const line = "#d9dede";
export const green = "#15735e";
export const amber = "#a04d21";
export const ease = Easing.bezier(0.22, 1, 0.36, 1);
export const clamp = {extrapolateLeft: "clamp" as const, extrapolateRight: "clamp" as const};
export const progress = (frame: number, start: number, end: number) => interpolate(frame, [start, end], [0, 1], {...clamp, easing: ease});

export const Reveal = ({children, at = 0, style}: {children: ReactNode; at?: number; style?: CSSProperties}) => {
  const frame = useCurrentFrame();
  const p = progress(frame, at, at + 42);
  return <div style={{opacity: p, transform: `translateY(${(1 - p) * 24}px)`, ...style}}>{children}</div>;
};

export const Page = ({children, chapter, dark = false}: {children: ReactNode; chapter: string; dark?: boolean}) => (
  <AbsoluteFill style={{background: dark ? ink : paper, color: dark ? paper : ink, fontFamily: "'Geist Variable', sans-serif", letterSpacing: 0}}>
    <div style={{position: "absolute", inset: "56px 88px auto", display: "flex", justifyContent: "space-between", alignItems: "center", fontSize: 28}}>
      <span style={{fontWeight: 650}}>pytxo</span><span style={{color: dark ? "#b7c1c4" : muted}}>{chapter}</span>
    </div>
    {children}
    <div style={{position: "absolute", left: 88, right: 88, bottom: 42, display: "flex", justifyContent: "space-between", alignItems: "center", color: dark ? "#b7c1c4" : muted, fontSize: 26}}>
      <span>Recorded run / 2 Oct 2026</span><span>Edited sequence / Native stills</span>
    </div>
  </AbsoluteFill>
);

export const AgentLogo = ({cli, size = 56}: {cli: string; size?: number}) => {
  const src = {codex: "openai-on-dark.svg", claude: "anthropic.svg", cursor: "cursor-on-dark.svg", opencode: "opencode.svg", agy: "antigravity.png"}[cli];
  return <div style={{width: size + 28, height: size + 28, display: "grid", placeItems: "center", background: "#17191c", borderRadius: 8, flex: "none"}}>
    {src && <Img src={staticFile(`fleet/logos/${src}`)} style={{width: size, height: size, objectFit: "contain", filter: cli === "claude" ? "brightness(0) invert(1)" : undefined}} />}
  </div>;
};

export const NativeCrop = ({props, assetKey, crop, width, height, style}: {
  props: FilmProps; assetKey: string; crop: {x: number; y: number; width: number};
  width: number; height: number; style?: CSSProperties;
}) => {
  const asset = props.assets.find(item => item.key === assetKey);
  if (!asset) throw new Error(`Missing recorded native asset: ${assetKey}`);
  const scale = width / crop.width;
  return <div style={{position: "relative", width, height, overflow: "hidden", background: "#08090a", borderRadius: 8, ...style}}>
    <Img src={staticFile(asset.path)} style={{position: "absolute", width: asset.width * scale, maxWidth: "none", height: asset.height * scale, left: -crop.x * scale, top: -crop.y * scale}} />
  </div>;
};
