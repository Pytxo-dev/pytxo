import {Video} from '@remotion/media';
import {interpolate, staticFile, useCurrentFrame} from 'remotion';
import type {NativeClip, NativePrivacyEdit} from '../config';
import {FILM_FPS} from '../config';
import {clamp, easeInOut, ink} from '../style';

export type FootageViewport = {x: number; y: number; width: number; height: number};

/** Source stays unchanged. Labeled privacy masks follow its source-frame coordinates. */
export const NativeFootage = ({clip, privacy, seconds, viewport = {x: 88, y: 220, width: 1744, height: 774}, allowCameraMove = false}: {clip: NativeClip; privacy: NativePrivacyEdit; seconds: number; viewport?: FootageViewport; allowCameraMove?: boolean}) => {
  const frame = useCurrentFrame();
  const camera = allowCameraMove ? interpolate(frame, [2.5 * FILM_FPS, 4 * FILM_FPS], [0, 1], {...clamp, easing: easeInOut}) : 1;
  const zoom = 1 + ((clip.focus?.zoom ?? 1) - 1) * camera;
  const fit = Math.min(viewport.width / clip.width, viewport.height / clip.height);
  const renderedWidth = clip.width * fit * zoom;
  const renderedHeight = clip.height * fit * zoom;
  const centerX = 0.5 + ((clip.focus?.x ?? 0.5) - 0.5) * camera;
  const centerY = 0.5 + ((clip.focus?.y ?? 0.5) - 0.5) * camera;
  const clampPosition = (desired: number, rendered: number, available: number) => rendered <= available ? (available - rendered) / 2 : Math.max(available - rendered, Math.min(0, desired));
  const x = clampPosition(viewport.width / 2 - centerX * renderedWidth, renderedWidth, viewport.width);
  const y = clampPosition(viewport.height / 2 - centerY * renderedHeight, renderedHeight, viewport.height);
  const sourceSecond = clip.trimStartSeconds + (frame / FILM_FPS) * clip.playbackRate;
  const sourceFrame = Math.floor(sourceSecond * clip.sourceFps + 0.00001);
  const regions = privacy.regions.filter(region => region.fromFrame <= sourceFrame && sourceFrame < region.untilFrame);
  const scale = fit * zoom;
  const recentCut = clip.cuts?.find((cut) => cut.atSeconds >= clip.trimStartSeconds && cut.atSeconds <= sourceSecond && sourceSecond < cut.atSeconds + 1.3 * clip.playbackRate);
  return (
    <>
    <div style={{position: 'absolute', left: viewport.x, top: viewport.y, width: viewport.width, height: viewport.height, borderRadius: 12, overflow: 'hidden', background: '#06080a', border: `1px solid ${ink.line}`, boxShadow: '0 28px 70px #0007'}}>
      <Video
        src={staticFile(clip.src)}
        trimBefore={Math.round(clip.trimStartSeconds * FILM_FPS)}
        trimAfter={Math.round((clip.trimStartSeconds + seconds * clip.playbackRate) * FILM_FPS)}
        playbackRate={clip.playbackRate}
        muted
        objectFit="contain"
        style={{position: 'absolute', left: x, top: y, width: renderedWidth, height: renderedHeight}}
        onError={(error) => {throw new Error(`Native capture ${clip.src} failed: ${error.message}`);}}
      />
      {regions.map((region, index) => <div key={index} style={{position: 'absolute', left: x + region.x * scale, top: y + region.y * scale, width: region.width * scale, height: region.height * scale, display: 'flex', alignItems: 'center', paddingInline: 6 * scale, boxSizing: 'border-box', overflow: 'hidden', background: '#101317', border: `${scale}px dashed #707780`, color: '#d3d8dd', fontFamily: 'Geist Variable, sans-serif', fontSize: 12 * scale, lineHeight: 1, whiteSpace: 'nowrap'}}>{region.label}</div>)}
    </div>
    {recentCut ? <div style={{position: 'absolute', right: 1920 - viewport.x - viewport.width, top: viewport.y - 26, fontSize: 19, lineHeight: 1, color: ink.mint}}>Time cut{recentCut.omittedSeconds === undefined ? '' : ` · ${Number(recentCut.omittedSeconds.toFixed(1))}s omitted`}</div> : null}
    </>
  );
};

export const CaptureLabel = ({clip, detail}: {clip: NativeClip; detail?: string}) => (
  <div style={{position: 'absolute', right: 88, top: 162, display: 'flex', alignItems: 'center', gap: 14, color: ink.secondary, fontSize: 21}}>
    <span style={{width: 6, height: 6, borderRadius: '50%', background: ink.mint}} />
    <span>Native recording · {clip.playbackRate === 1 ? '1×' : `${clip.playbackRate}×`} playback{detail ? ` · ${detail}` : ''}</span>
  </div>
);
