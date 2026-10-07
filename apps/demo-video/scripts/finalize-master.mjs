import {unlink} from "node:fs/promises";
import path from "node:path";
import process from "node:process";
import {spawnSync} from "node:child_process";
import {fileURLToPath} from "node:url";

const here = path.dirname(fileURLToPath(import.meta.url));
const appRoot = path.resolve(here, "..");
const input = process.argv[2];
const output = process.argv[3];

if (!input || !output || input === output) {
  throw new Error(
    "Usage: node scripts/finalize-master.mjs <render-input.mp4> <master-output.mp4>",
  );
}

const result = spawnSync(
  "ffmpeg",
  [
    "-hide_banner",
    "-i",
    input,
    "-map",
    "0",
    "-c",
    "copy",
    // Set H.264 VUI as well as container tags so decoders agree on BT.709.
    "-bsf:v",
    "h264_metadata=colour_primaries=1:transfer_characteristics=1:matrix_coefficients=1",
    "-color_primaries",
    "bt709",
    "-color_trc",
    "bt709",
    "-colorspace",
    "bt709",
    "-movflags",
    "+faststart",
    "-y",
    output,
  ],
  {
    cwd: appRoot,
    stdio: "inherit",
  },
);

if (result.error) throw result.error;
if (result.status !== 0) process.exit(result.status ?? 1);

await unlink(path.join(appRoot, input));
process.stdout.write(`finalized BT.709 master ${output}\n`);
