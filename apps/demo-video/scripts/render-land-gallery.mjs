// Product Hunt gallery: six 1270x760 stills, each one frame of the land take
// (public/land/take.mp4) cropped to the rows that matter. Times are video seconds.
//   node scripts/render-land-gallery.mjs <out-dir>
import {execFileSync} from "node:child_process";
import {mkdirSync, writeFileSync} from "node:fs";
import {tmpdir} from "node:os";
import path from "node:path";

const out = process.argv[2];
if (!out) throw new Error("usage: render-land-gallery.mjs <out-dir>");
mkdirSync(out, {recursive: true});
// rows: [first, end) terminal rows; cols: crop width from the left. Crops end on whole lines.
const stills = [
  ["01-blocked", {at: 135, rows: [12, 19], cols: 78, title: "Each branch passes alone. Together, they break."}],
  ["02-command", {at: 135, rows: [0, 12], cols: 114, title: "One command. Branches from any agent."}],
  ["03-next", {at: 135, rows: [22, 28], cols: 104, title: "BLOCKED, with the reason and the next step."}],
  ["04-fix", {at: 149.8, rows: [0, 16], cols: 80, title: "Fix it on one branch."}],
  ["05-ready", {at: 223, rows: [12, 25], cols: 122, title: "Same command: READY."}],
  ["06-landed", {at: 300.5, rows: [10, 31], cols: 104, title: "--apply lands it. Every decision leaves a receipt."}],
];
for (const [name, props] of stills) {
  const file = path.join(tmpdir(), `pytxo-land-${name}.props.json`);
  writeFileSync(file, JSON.stringify(props));
  execFileSync("npx", ["remotion", "still", "src/index.ts", "PytxoLandStill", path.join(out, `${name}.png`), `--props=${file}`, "--overwrite"], {stdio: "inherit", shell: true});
}
