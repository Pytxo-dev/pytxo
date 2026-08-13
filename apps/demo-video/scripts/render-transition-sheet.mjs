import path from "node:path";
import process from "node:process";
import {spawnSync} from "node:child_process";
import {fileURLToPath} from "node:url";

const here = path.dirname(fileURLToPath(import.meta.url));
const appRoot = path.resolve(here, "..");
const frames = [112, 119, 120, 389, 390, 809, 810, 914, 922, 1169, 1170, 1402, 1409];
const selection = frames.map((frame) => `eq(n\\,${frame})`).join("+");

const result = spawnSync(
  "ffmpeg",
  [
    "-hide_banner",
    "-i",
    "out/pytxo-demo-silent.mp4",
    "-vf",
    `select='${selection}',scale=640:360,tile=4x4:padding=4:margin=4:color=black`,
    "-frames:v",
    "1",
    "-update",
    "1",
    "-fps_mode",
    "vfr",
    "-y",
    "out/pytxo-demo-transition-sheet.png",
  ],
  {
    cwd: appRoot,
    stdio: "inherit",
  },
);

if (result.error) throw result.error;
if (result.status !== 0) process.exit(result.status ?? 1);
