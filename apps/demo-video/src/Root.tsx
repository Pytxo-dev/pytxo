import "@fontsource-variable/geist";
import {Composition, Folder} from "remotion";
import {PytxoMotionFilm} from "./motion/PytxoMotionFilm";
import {Opening as MotionOpening} from "./motion/Opening";
import {Dispatch as MotionDispatch} from "./motion/Dispatch";
import {Merge as MotionMerge} from "./motion/Merge";
import {Boundary as MotionBoundary} from "./motion/Boundary";
import {Apply as MotionApply} from "./motion/Apply";
import {Closing as MotionClosing} from "./motion/Closing";
import {
  FPS,
  PytxoLaunchDemo,
  TOTAL_FRAMES,
  type PytxoLaunchDemoProps,
} from "./PytxoLaunchDemo";
import {PytxoApertureFilm, FILM_DURATION, FILM_FPS, parseApertureFilmProps} from "./aperture/PytxoApertureFilm";
import importedApertureJson from "../aperture-props.json";
import {
  PytxoR3Storyboard,
  R3_STORYBOARD_FPS,
  R3_STORYBOARD_FRAMES,
  parseR3StoryboardProps,
} from "./r3/PytxoR3Storyboard";
import importedR3StoryboardJson from "../r3-storyboard-props.json";
import {
  PytxoR6CockpitProof,
  R6_COCKPIT_FPS,
  R6_COCKPIT_FRAMES,
  parseR6CockpitProofProps,
} from "./r6/PytxoR6CockpitProof";
import importedR6CockpitJson from "../r6-cockpit-props.json";
import {FLEET_FPS, FLEET_FRAMES, PytxoFleetFilm, parseFleetFilmProps} from "./fleet/PytxoFleetFilm";
import importedFleetJson from "../fleet-props.json";
import {PytxoBetaFilm} from "./beta/PytxoBetaFilm";
import {FPS as BETA_FPS, FRAMES as BETA_FRAMES} from "./beta/shared";

const apertureProps = parseApertureFilmProps(importedApertureJson);
const r3StoryboardProps = parseR3StoryboardProps(importedR3StoryboardJson);
const r6CockpitProps = parseR6CockpitProofProps(importedR6CockpitJson);
const fleetProps = parseFleetFilmProps(importedFleetJson);

export const RemotionRoot = () => {
  return (
    <>
    <Composition id="PytxoMotionFilm" component={PytxoMotionFilm} durationInFrames={3000} fps={60} width={1920} height={1080} defaultProps={fleetProps} />
    <Folder name="Motion-Scenes">
      <Composition id="Motion-Opening" component={MotionOpening} durationInFrames={240} fps={60} width={1920} height={1080} defaultProps={fleetProps} />
      <Composition id="Motion-Dispatch" component={MotionDispatch} durationInFrames={900} fps={60} width={1920} height={1080} defaultProps={fleetProps} />
      <Composition id="Motion-Merge" component={MotionMerge} durationInFrames={420} fps={60} width={1920} height={1080} defaultProps={fleetProps} />
      <Composition id="Motion-Boundary" component={MotionBoundary} durationInFrames={480} fps={60} width={1920} height={1080} defaultProps={fleetProps} />
      <Composition id="Motion-Apply" component={MotionApply} durationInFrames={720} fps={60} width={1920} height={1080} defaultProps={fleetProps} />
      <Composition id="Motion-Closing" component={MotionClosing} durationInFrames={240} fps={60} width={1920} height={1080} />
    </Folder>
    <Composition
      id="PytxoBetaFilm"
      component={PytxoBetaFilm}
      durationInFrames={BETA_FRAMES}
      fps={BETA_FPS}
      width={1920}
      height={1080}
      defaultProps={fleetProps}
    />
    <Composition
      id="PytxoLaunchDemo"
      component={PytxoLaunchDemo}
      durationInFrames={TOTAL_FRAMES}
      fps={FPS}
      width={1920}
      height={1080}
      defaultProps={
        {
          includeAudio: false,
        } satisfies PytxoLaunchDemoProps
      }
    />
    <Composition
      id="PytxoApertureFilm"
      component={PytxoApertureFilm}
      durationInFrames={FILM_DURATION}
      fps={FILM_FPS}
      width={1920}
      height={1080}
      defaultProps={apertureProps}
    />
    <Composition
      id="PytxoR3Storyboard"
      component={PytxoR3Storyboard}
      durationInFrames={R3_STORYBOARD_FRAMES}
      fps={R3_STORYBOARD_FPS}
      width={1920}
      height={1080}
      defaultProps={r3StoryboardProps}
    />
    <Composition
      id="PytxoR6CockpitProof"
      component={PytxoR6CockpitProof}
      durationInFrames={R6_COCKPIT_FRAMES}
      fps={R6_COCKPIT_FPS}
      width={1920}
      height={1080}
      defaultProps={r6CockpitProps}
    />
    <Composition
      id="PytxoFleetFilm"
      component={PytxoFleetFilm}
      durationInFrames={FLEET_FRAMES}
      fps={FLEET_FPS}
      width={1920}
      height={1080}
      defaultProps={fleetProps}
    />
    </>
  );
};
