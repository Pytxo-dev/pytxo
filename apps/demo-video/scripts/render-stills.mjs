import {mkdir} from "node:fs/promises";
import path from "node:path";
import process from "node:process";
import {spawnSync} from "node:child_process";
import {fileURLToPath} from "node:url";

const here = path.dirname(fileURLToPath(import.meta.url));
const appRoot = path.resolve(here, "..");
const stillDir = path.join(appRoot, "out", "stills");
await mkdir(stillDir, {recursive: true});

const stills = [
  ["01-mission.png", 60],
  ["02-work-ownership.png", 300],
  ["03-review.png", 600],
  ["04-apply-click.png", 914],
  ["05-applied.png", 1040],
  ["06-outcome.png", 1300],
  ["07-end.png", 1500],
];

const run = (args) => {
  const [, ...remotionArgs] = args;
  const result = spawnSync(
    process.execPath,
    [path.join(appRoot, "node_modules", "@remotion", "cli", "remotion-cli.js"), ...remotionArgs],
    {
    cwd: appRoot,
    stdio: "inherit",
    },
  );
  if (result.error) throw result.error;
  if (result.status !== 0) process.exit(result.status ?? 1);
};

const runSystemFfmpeg = (args) => {
  const result = spawnSync("ffmpeg", args, {
    cwd: appRoot,
    stdio: "inherit",
  });
  if (result.error) throw result.error;
  if (result.status !== 0) process.exit(result.status ?? 1);
};

if (!process.argv.includes("--contact-only")) {
  for (const [filename, frame] of stills) {
    run([
      "remotion",
      "still",
      "src/index.ts",
      "PytxoLaunchDemo",
      path.join("out", "stills", filename),
      `--frame=${frame}`,
      "--overwrite",
    ]);
  }
}

const inputs = stills.flatMap(([filename]) => [
  "-i",
  path.join("out", "stills", filename),
]);
const scaled = stills
  .map((_, index) => `[${index}:v]scale=960:540[s${index}]`)
  .join(";");
const layout = stills
  .map((_, index) => `${(index % 4) * 960}_${Math.floor(index / 4) * 540}`)
  .join("|");
const stacks = stills.map((_, index) => `[s${index}]`).join("");

runSystemFfmpeg([
  ...inputs,
  "-filter_complex",
  `${scaled};${stacks}xstack=inputs=${stills.length}:layout=${layout}:fill=black[out]`,
  "-map",
  "[out]",
  "-frames:v",
  "1",
  "-update",
  "1",
  path.join("out", "pytxo-demo-contact-sheet.png"),
  "-y",
]);
