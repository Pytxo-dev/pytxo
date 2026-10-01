import "@fontsource-variable/geist";
import {Composition} from "remotion";
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

const apertureProps = parseApertureFilmProps(importedApertureJson);
const r3StoryboardProps = parseR3StoryboardProps(importedR3StoryboardJson);
const r6CockpitProps = parseR6CockpitProofProps(importedR6CockpitJson);

export const RemotionRoot = () => {
  return (
    <>
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
    </>
  );
};
