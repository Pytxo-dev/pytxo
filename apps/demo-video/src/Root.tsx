import "@fontsource-variable/geist";
import {Composition} from "remotion";
import {
  PytxoLaunchDemo,
  TRANSITION_FRAMES,
  type PytxoLaunchDemoProps,
} from "./PytxoLaunchDemo";
import voiceover from "./voiceover.json";

export const FPS = 30;
export const TOTAL_FRAMES = voiceover.reduce(
  (sum, scene) => sum + scene.durationInFrames,
  0,
) - (voiceover.length - 1) * TRANSITION_FRAMES;

export const RemotionRoot = () => {
  return (
    <Composition
      id="PytxoLaunchDemo"
      component={PytxoLaunchDemo}
      durationInFrames={TOTAL_FRAMES}
      fps={FPS}
      width={1920}
      height={1080}
      defaultProps={
        {
          includeVoiceover: false,
        } satisfies PytxoLaunchDemoProps
      }
    />
  );
};
