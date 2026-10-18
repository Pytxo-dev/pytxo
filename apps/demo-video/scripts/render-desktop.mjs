// Renders PytxoDesktop in 600-frame chunks. Remotion's compositor now and then
// fails one OffthreadVideo frame ("No frame found at position N") at a random
// spot, so a failed chunk is rendered again instead of the whole film. The
// audio is rendered once and muxed onto the joined chunks without re-encoding.
//   node scripts/render-desktop.mjs [out/pytxo-desktop-1.3.2.mp4]
import {execFileSync} from "node:child_process";
import {mkdirSync, rmSync, writeFileSync} from "node:fs";
import path from "node:path";

const out = path.resolve(process.argv[2] ?? "out/pytxo-desktop-1.3.2.mp4");
const work = path.resolve("out/desktop-chunks");
rmSync(work, {recursive: true, force: true});
mkdirSync(work, {recursive: true});
const remotion = (...args) => execFileSync("npx", ["remotion", ...args], {stdio: ["ignore", "ignore", "pipe"], shell: true});
const common = ["src/index.ts", "PytxoDesktop", "--concurrency=1", "--offthreadvideo-cache-size-in-bytes=400000000", "--offthreadvideo-threads=1", "--log=error"];

const total = Number(execFileSync("npx", ["remotion", "compositions", "src/index.ts"], {shell: true, stdio: ["ignore", "pipe", "ignore"]}).toString()
  .split("\n").find((line) => line.startsWith("PytxoDesktop")).trim().split(/\s+/)[3]);
const parts = [];
for (let from = 0; from < total; from += 600) {
  const file = path.join(work, `part${parts.length}.mp4`);
  for (let attempt = 1; ; attempt++) {
    try {
      remotion("render", ...common, file, `--frames=${from}-${Math.min(total, from + 600) - 1}`, "--codec=h264", "--crf=17", "--pixel-format=yuv420p", "--color-space=bt709", "--muted");
      break;
    } catch (error) {
      if (attempt === 3) throw error;
      console.log(`frames ${from}+ failed (attempt ${attempt}); rendering them again`);
    }
  }
  parts.push(file);
  console.log(`frames ${from}-${Math.min(total, from + 600) - 1} done`);
}
const audio = path.join(work, "audio.wav");
const props = path.join(work, "audio-props.json");
writeFileSync(props, JSON.stringify({audioOnly: true}));
remotion("render", ...common, audio, "--codec=wav", `--props=${props}`);
const list = path.join(work, "parts.txt");
writeFileSync(list, parts.map((file) => `file '${path.basename(file)}'`).join("\n") + "\n");
execFileSync("ffmpeg", ["-v", "error", "-y", "-f", "concat", "-safe", "0", "-i", list, "-i", audio, "-map", "0:v", "-map", "1:a",
  "-c:v", "copy", "-c:a", "aac", "-b:a", "320k", "-shortest", "-movflags", "+faststart", out]);
console.log(out);
