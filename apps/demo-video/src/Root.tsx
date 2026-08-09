import "@fontsource-variable/geist";
import {Composition} from "remotion";
import {
  FPS,
  PytxoLaunchDemo,
  TOTAL_FRAMES,
  type PytxoLaunchDemoProps,
} from "./PytxoLaunchDemo";

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
          includeAudio: false,
        } satisfies PytxoLaunchDemoProps
      }
    />
  );
};
