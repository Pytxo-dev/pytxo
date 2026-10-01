import path from "node:path";
import process from "node:process";
import {mkdir} from "node:fs/promises";
import {spawnSync} from "node:child_process";
import {fileURLToPath} from "node:url";

const here = path.dirname(fileURLToPath(import.meta.url));
const appRoot = path.resolve(here, "..");
const startFrame = Number(process.argv[2]);
const endFrame = Number(process.argv[3]);
const outputName = process.argv[4] ?? "pytxo-demo-historical-pipeline-proof-4k.mp4";
const compositionFrames = 52 * 30;
const fps = 30;

if (
  !Number.isInteger(startFrame)
  || !Number.isInteger(endFrame)
  || startFrame < 0
  || endFrame < startFrame
  || endFrame >= compositionFrames
  || path.basename(outputName) !== outputName
  || !outputName.endsWith(".mp4")
  || !outputName.toLowerCase().includes("historical")
) {
  throw new Error(
    "Usage: node scripts/render-4k-clip.mjs <start-frame> <end-frame> [historical-output-name.mp4]; frames must be within 0-1559 and the name must preserve the historical provenance label",
  );
}

const outputRelative = path.join("out", outputName);
const renderRelative = path.join("out", outputName.replace(/\.mp4$/, ".render.mp4"));
const outputAbsolute = path.join(appRoot, outputRelative);
const nullOutput = process.platform === "win32" ? "NUL" : "/dev/null";

await mkdir(path.dirname(outputAbsolute), {recursive: true});

const run = (command, args, options = {}) => {
  const result = spawnSync(command, args, {
    cwd: appRoot,
    encoding: options.capture ? "utf8" : undefined,
    stdio: options.capture ? "pipe" : "inherit",
    maxBuffer: 32 * 1024 * 1024,
  });
  if (result.error) throw result.error;
  if (result.status !== 0) {
    throw new Error(
      `${command} failed with exit code ${result.status ?? "unknown"}`
      + (options.capture ? `:\n${result.stderr || result.stdout}` : ""),
    );
  }
  return result;
};

run(process.execPath, [path.join("scripts", "validate-assets.mjs"), "silent"]);
run(process.execPath, [
  path.join("node_modules", "@remotion", "cli", "remotion-cli.js"),
  "render",
  "src/index.ts",
  "PytxoLaunchDemo",
  renderRelative,
  `--frames=${startFrame}-${endFrame}`,
  "--scale=2",
  "--codec=h264",
  "--crf=16",
  "--pixel-format=yuv420p",
  "--color-space=bt709",
  "--muted",
  "--concurrency=1",
  "--overwrite",
]);
run(process.execPath, [
  path.join("scripts", "finalize-master.mjs"),
  renderRelative,
  outputRelative,
]);

const probe = run("ffprobe", [
  "-v",
  "error",
  "-show_entries",
  "format=duration:stream=codec_type,codec_name,width,height,r_frame_rate,pix_fmt,color_space,color_transfer,color_primaries",
  "-of",
  "json",
  outputRelative,
], {capture: true});
const metadata = JSON.parse(probe.stdout);
const video = metadata.streams.find((stream) => stream.codec_type === "video");
const audio = metadata.streams.find((stream) => stream.codec_type === "audio");
const duration = Number(metadata.format?.duration);
const expectedDuration = (endFrame - startFrame + 1) / fps;
const errors = [];

const expectEqual = (label, actual, expected) => {
  if (actual !== expected) errors.push(`${label}: expected ${expected}, received ${actual}`);
};

if (!video) {
  errors.push("video stream: missing");
} else {
  expectEqual("codec", video.codec_name, "h264");
  expectEqual("width", video.width, 3840);
  expectEqual("height", video.height, 2160);
  expectEqual("fps", video.r_frame_rate, "30/1");
  expectEqual("pixel format", video.pix_fmt, "yuv420p");
  expectEqual("color space", video.color_space, "bt709");
  expectEqual("color transfer", video.color_transfer, "bt709");
  expectEqual("color primaries", video.color_primaries, "bt709");
}
if (audio) errors.push("audio stream: proof clip must be silent");
if (!Number.isFinite(duration) || Math.abs(duration - expectedDuration) > 0.02) {
  errors.push(`duration: expected ${expectedDuration.toFixed(3)}, received ${duration}`);
}
if (errors.length > 0) {
  throw new Error(`4K clip validation failed:\n${errors.map((item) => `- ${item}`).join("\n")}`);
}

run("ffmpeg", [
  "-v",
  "error",
  "-i",
  outputRelative,
  "-f",
  "null",
  nullOutput,
]);

process.stdout.write(
  `Historical 4K pipeline proof passed: frames ${startFrame}-${endFrame}, ${duration.toFixed(3)}s, 3840x2160 at 30 fps, full decode complete.\n`,
);
