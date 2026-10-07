import path from "node:path";
import process from "node:process";
import {spawnSync} from "node:child_process";
import {fileURLToPath} from "node:url";
import {
  evaluateAudioQa,
  parseLoudnorm,
  parseSilenceDetect,
} from "./audio-qa.mjs";

const here = path.dirname(fileURLToPath(import.meta.url));
const appRoot = path.resolve(here, "..");
const master = process.argv[2];
const mode = process.argv[3] ?? "silent";
const profile = process.argv[4] ?? "launch";
const resolution = process.argv[5] ?? "1080p";
const expected = profile === "beta"
  ? {seconds: 56, fps: 60}
  : profile === "aperture"
  ? {seconds: 58, fps: 60}
  : profile === "r3-storyboard"
    ? {seconds: 24, fps: 30}
    : profile === "r6-cockpit"
      ? {seconds: 12, fps: 30}
    : {seconds: 52, fps: 30};
const frameSize = resolution === "2160p"
  ? {width: 3840, height: 2160}
  : {width: 1920, height: 1080};

if (
  !master
  || !["silent", "narrated"].includes(mode)
  || !["launch", "aperture", "r3-storyboard", "r6-cockpit", "beta"].includes(profile)
  || !["1080p", "2160p"].includes(resolution)
) {
  throw new Error(
    "Usage: node scripts/validate-master.mjs <master.mp4> [silent|narrated] [launch|aperture|r3-storyboard|r6-cockpit|beta] [1080p|2160p]",
  );
}

const runSystemFfmpeg = (args) => {
  const result = spawnSync("ffmpeg", args, {
    cwd: appRoot,
    encoding: "utf8",
    maxBuffer: 32 * 1024 * 1024,
  });
  if (result.error) throw result.error;
  if (result.status !== 0) {
    throw new Error(`ffmpeg failed:\n${result.stderr || result.stdout}`);
  }
  return {stdout: result.stdout, stderr: result.stderr};
};

const runSystemFfprobe = (args) => {
  const result = spawnSync("ffprobe", args, {
    cwd: appRoot,
    encoding: "utf8",
    maxBuffer: 32 * 1024 * 1024,
  });
  if (result.error) throw result.error;
  if (result.status !== 0) {
    throw new Error(`ffprobe failed:\n${result.stderr || result.stdout}`);
  }
  return {stdout: result.stdout, stderr: result.stderr};
};

const nullOutput = process.platform === "win32" ? "NUL" : "/dev/null";

const probe = runSystemFfprobe([
  "-v",
  "error",
  "-show_entries",
  "format=duration:stream=index,codec_type,codec_name,width,height,r_frame_rate,pix_fmt,color_space,color_transfer,color_primaries,sample_rate,bit_rate",
  "-of",
  "json",
  master,
]);
const metadata = JSON.parse(probe.stdout);
const video = metadata.streams.find((stream) => stream.codec_type === "video");
const audio = metadata.streams.find((stream) => stream.codec_type === "audio");
const duration = Number(metadata.format?.duration);
const errors = [];

const expectEqual = (label, actual, expected) => {
  if (actual !== expected) errors.push(`${label}: expected ${expected}, received ${actual}`);
};

if (!video) {
  errors.push("video stream: missing");
} else {
  expectEqual("codec", video.codec_name, "h264");
  expectEqual("width", video.width, frameSize.width);
  expectEqual("height", video.height, frameSize.height);
  expectEqual("fps", video.r_frame_rate, `${expected.fps}/1`);
  expectEqual("pixel format", video.pix_fmt, "yuv420p");
  expectEqual("color space", video.color_space, "bt709");
  expectEqual("color transfer", video.color_transfer, "bt709");
  expectEqual("color primaries", video.color_primaries, "bt709");
}
if (!Number.isFinite(duration) || Math.abs(duration - expected.seconds) > 0.02) {
  errors.push(`duration: expected ${expected.seconds}.000 seconds, received ${duration}`);
}

if (mode === "silent") {
  if (audio) errors.push("audio stream: silent master must not contain one");
} else if (!audio) {
  errors.push("audio stream: narrated master is missing AAC output");
} else {
  expectEqual("audio codec", audio.codec_name, "aac");
  expectEqual("audio sample rate", audio.sample_rate, "48000");
  const bitrate = Number(audio.bit_rate);
  if (!Number.isFinite(bitrate) || Math.abs(bitrate - 256000) > 16000) {
    errors.push(`audio bitrate: expected approximately 256000, received ${audio.bit_rate}`);
  }
}

const black = runSystemFfmpeg([
  "-hide_banner",
  "-i",
  master,
  "-vf",
  "blackdetect=d=0.3:pix_th=0.005:pic_th=0.995",
  "-an",
  "-f",
  "null",
  nullOutput,
]);
const blackDurations = [...black.stderr.matchAll(/black_duration:([0-9.]+)/g)].map(
  (match) => Number(match[1]),
);
if (blackDurations.some((value) => value >= 0.3)) {
  errors.push(`black frames: detected ${blackDurations.join(", ")} second segment(s)`);
}

const freeze = runSystemFfmpeg([
  "-hide_banner",
  "-i",
  master,
  "-vf",
  "freezedetect=n=-60dB:d=2",
  "-an",
  "-f",
  "null",
  nullOutput,
]);
const freezeStarts = [...freeze.stderr.matchAll(/freeze_start: ([0-9.]+)/g)].map(
  (match) => Number(match[1]),
);
process.stdout.write(
  `Freeze scan: ${freezeStarts.length} deterministic hold(s) over 2 seconds; inspect against the approved hold map.\n`,
);

if (mode === "narrated") {
  const loudness = runSystemFfmpeg([
    "-hide_banner",
    "-i",
    master,
    "-map",
    "0:a",
    "-af",
    "loudnorm=I=-16:TP=-1:LRA=11:print_format=json",
    "-f",
    "null",
    nullOutput,
  ]);
  const loudnessMetrics = parseLoudnorm(loudness.stderr);
  const threshold = loudnessMetrics?.thresholdDb;
  let silenceSegments = [];
  if (Number.isFinite(threshold)) {
    const silence = runSystemFfmpeg([
      "-hide_banner",
      "-i",
      master,
      "-map",
      "0:a",
      "-af",
      `silencedetect=noise=${threshold}dB:d=0.5`,
      "-f",
      "null",
      nullOutput,
    ]);
    silenceSegments = parseSilenceDetect(silence.stderr, duration);
  }
  errors.push(
    ...evaluateAudioQa({
      loudness: loudnessMetrics,
      silenceSegments,
      durationSeconds: duration,
    }),
  );
  process.stdout.write(
    `Narrated audio QA: ${loudnessMetrics?.integratedLufs ?? "unreadable"} LUFS, ` +
      `${loudnessMetrics?.truePeakDbtp ?? "unreadable"} dBTP, ` +
      `${silenceSegments.length} silence segment(s).\n`,
  );
}

if (errors.length > 0) {
  process.stderr.write(`Master validation failed:\n${errors.map((item) => `- ${item}`).join("\n")}\n`);
  process.exit(1);
}

process.stdout.write(
  `Master validation passed: ${profile}, ${mode}, ${duration.toFixed(3)}s, ${frameSize.width}x${frameSize.height} at ${expected.fps} fps.\n`,
);
