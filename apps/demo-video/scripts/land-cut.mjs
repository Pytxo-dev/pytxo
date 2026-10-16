// Turns a terminal take (scripts/land-take) into the PytxoLand shot list.
// Every shot is a range of the one recording; speed only shortens typing and
// waits (disclosed on screen as "sped up" from 4x) or slows a still frame so it
// can be read. Nothing on screen is re-created.
//   node scripts/land-cut.mjs <take-dir>
// Writes public/land/take.mp4 (seekable proxy of capture.mkv) and land-cut.json.
import {execFileSync, spawnSync} from "node:child_process";
import {mkdirSync, readFileSync, writeFileSync} from "node:fs";
import path from "node:path";

const dir = process.argv[2];
if (!dir) throw new Error("usage: land-cut.mjs <take-dir>");
const lines = readFileSync(path.join(dir, "timeline.jsonl"), "utf8").trim().split("\n").map((l) => JSON.parse(l));
const cmds = lines.filter((l) => l.label);
const done = lines.find((l) => l.note === "done");
if (!done) throw new Error("The take did not finish.");
const by = Object.fromEntries(cmds.map((c) => [c.label, c]));
const expect = {"land-blocked": 2, "land-ready": 0, "land-apply": 0};
for (const [label, code] of Object.entries(expect)) {
  if (by[label]?.exit !== code) throw new Error(`${label} exited ${by[label]?.exit}, expected ${code}`);
}

const capture = path.join(dir, "capture.mkv");
const [width, height] = execFileSync("ffprobe", ["-v", "error", "-select_streams", "v:0", "-show_entries", "stream=width,height", "-of", "csv=p=0", capture]).toString().trim().split(",").map(Number);

// Clock offset: the BLOCKED report is printed in one go when the first land run
// ends, so the biggest brightness jump near that moment marks its video time.
const take = JSON.parse(readFileSync(path.join(dir, "take.json"), "utf8"));
const mark = by["land-blocked"].end;
const est = mark - take.ffmpeg_started;
const stats = spawnSync("ffmpeg", ["-hide_banner", "-copyts", "-ss", String(Math.max(0, est - 6)), "-t", "14", "-i", capture, "-vf", "signalstats,metadata=print:key=lavfi.signalstats.YAVG", "-f", "null", "-"], {encoding: "utf8"}).stderr;
const samples = [...stats.matchAll(/pts_time:([\d.]+)[\s\S]*?YAVG=([\d.]+)/g)].map((m) => [Number(m[1]), Number(m[2])]);
let jump = null, best = 0;
for (let i = 1; i < samples.length; i++) {
  const d = samples[i][1] - samples[i - 1][1];
  if (d > best) { best = d; jump = samples[i][0]; }
}
if (jump === null || best < 0.5) throw new Error("Could not find the BLOCKED report in the capture.");
const offset = mark - jump;
const v = (epoch) => +(epoch - offset).toFixed(3);

// Rows (and columns) of the 122x32 grid (13x30 px cells) worth reading in each
// output: land-focus.json maps a command to [[fromRow, toRow, cols], ...]; the
// hold is split evenly between them.
const rows = ([from, to, cols = 122]) => ({x: 0, y: from * 30, w: cols * 13, h: (to - from) * 30});

const BEAT = {install: "install", version: "install", branches: "branches", "land-blocked": "blocked", "fix-rebase": "fix", "fix-edit": "fix", "fix-commit": "fix", "land-ready": "ready", "land-apply": "landed", graph: "landed", receipts: "receipts"};
const HOLD = {version: 1.6, branches: 3.2, "land-blocked": 7, "fix-edit": 3.5, "fix-commit": 1.2, "land-ready": 4.5, "land-apply": 3, graph: 3.5, receipts: 3.5};
const focusFor = JSON.parse(readFileSync(path.resolve("land-focus.json"), "utf8"));

const shots = [];
cmds.forEach((c, i) => {
  const next = cmds[i + 1];
  const beat = BEAT[c.label];
  // The prompt is on screen from "typed"; typing starts 0.8 s later.
  shots.push({label: c.label, beat, part: "type", start: v(c.typed) + 0.5, end: v(c.enter) + 0.15, speed: 1.6, focus: null});
  const wait = c.end - c.enter;
  if (wait > 4) shots.push({label: c.label, beat, part: "run", start: v(c.enter) + 0.15, end: v(c.end) - 0.1, speed: +((wait - 0.25) / 2.2).toFixed(2), focus: null});
  else shots.push({label: c.label, beat, part: "run", start: v(c.enter) + 0.15, end: v(c.end) - 0.1, speed: 1, focus: null});
  // A still frame of the finished output, slowed so it can be read.
  const stillEnd = Math.min(v(c.end) + 2.3, next ? v(next.typed) - 0.2 : v(done.t));
  const hold = HOLD[c.label] ?? 0.8;
  const focus = focusFor[c.label] ?? [null];
  const span = (stillEnd - v(c.end) - 0.05) / focus.length;
  focus.forEach((f, k) => {
    const start = v(c.end) + 0.05 + k * span;
    shots.push({label: c.label, beat, part: "hold", start: +start.toFixed(3), end: +(start + span).toFixed(3), speed: +((span * focus.length) / hold).toFixed(3), focus: f ? rows(f) : null});
  });
});
for (const s of shots) if (!(s.end > s.start)) throw new Error(`empty shot ${s.label}/${s.part}`);

mkdirSync("public/land", {recursive: true});
execFileSync("ffmpeg", ["-hide_banner", "-loglevel", "error", "-y", "-i", capture, "-c:v", "libx264", "-preset", "slow", "-crf", "12", "-pix_fmt", "yuv420p", "-g", "15", "-movflags", "+faststart", "public/land/take.mp4"]);
const landed = execFileSync("git", ["-C", path.join(dir, "mri"), "rev-parse", "--short", "master"]).toString().trim();
writeFileSync("land-cut.json", JSON.stringify({width, height, fps: 30, offset: +offset.toFixed(3), landed, commands: cmds.map((c) => ({label: c.label, exit: c.exit, seconds: +(c.end - c.enter).toFixed(1)})), shots}, null, 2) + "\n");
console.log(`land-cut.json: ${shots.length} shots, offset ${offset.toFixed(3)}, ${width}x${height}`);
