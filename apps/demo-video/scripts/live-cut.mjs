// Turns a native journey take (screen recording + receipt telemetry) into the
// PytxoLive shot list. Every shot is a range of the real recording; speed only
// shortens waits and is disclosed on screen as "sped up".
//   node scripts/live-cut.mjs <journey-dir>
// Copies screen.mp4 and result.mp4 into public/live and writes live-cut.json.
import {execFileSync} from "node:child_process";
import {existsSync, mkdirSync, readFileSync, writeFileSync} from "node:fs";
import path from "node:path";

const dir = process.argv[2];
if (!dir) throw new Error("usage: live-cut.mjs <journey-dir>");
const receipt = JSON.parse(readFileSync(path.join(dir, "receipt.json"), "utf8"));
const screen = JSON.parse(readFileSync(path.join(dir, "screen.json"), "utf8"));
const out = path.resolve("public/live");
mkdirSync(out, {recursive: true});
if (receipt.result !== "passed") throw new Error("Only a passed journey take can be cut.");

const sec = (epoch) => (epoch - screen.first) / 1000;
const m = Object.fromEntries(Object.entries(receipt.marks).map(([k, v]) => [k, sec(v)]));
const clicks = receipt.pointer.filter((p) => p.surface === "desktop").map((p) => ({...p, t: sec(p.t)}));
const hoverOf = (label, after = 0) => clicks.find((p) => p.kind === "hover" && p.label === label && p.t >= after);

const probe = execFileSync("ffprobe", ["-v", "error", "-select_streams", "v:0", "-show_entries", "stream=width,height", "-of", "csv=p=0", path.join(dir, "screen.mkv")]).toString().trim().split(",").map(Number);
// The Windows taskbar (bottom 60 px at 125%) stays out of frame.
const [W, FULL_H] = probe;
const H = FULL_H - 60;
// Pointer telemetry and the capture share physical screen pixels when both are DPI aware.
const focus = (p, w = 760, h = 430) => p ? {x: Math.max(0, Math.min(W - w, p.x - w / 2)), y: Math.max(0, Math.min(H - h, p.y - h / 2)), w, h} : null;

const newWork = hoverOf("New work");
const request = hoverOf("Request");
const firstAgent = clicks.find((p) => p.kind === "hover" && ["Claude Code", "Cursor Agent", "OpenCode", "Antigravity"].includes(p.label));
const build = hoverOf("Build plan");
const run = hoverOf("Run");
const review = hoverOf("Review changes");
const apply1 = hoverOf("Apply reviewed changes", m.read ?? m.review);
const refresh = hoverOf("Refresh review");

const shots = [
  {start: newWork.t - 0.6, end: m.typed + 0.4, speed: Math.max(1, (m.typed - m.typing) / 4), kicker: "01 · Describe", title: "Say what you want, once.", focus: focus(request, 1100, 620)},
  {start: firstAgent.t - 0.4, end: build.t + 0.8, speed: 1.6, kicker: "02 · Choose agents", title: "Put every agent you use on the job.", focus: focus(firstAgent, 900, 506)},
  {start: m.plan - 0.2, end: run.t + 1.0, speed: 1.4, kicker: "03 · Plan", title: "See the plan before anything runs.", focus: null},
  ...(m.dragged ? [{start: m.dragged - 2.6, end: m.dragged + 1.2, speed: 1, kicker: "04 · Fleet", title: "Every worker, on one canvas you can move.", focus: null}] : []),
  {start: (m.dragged ?? m.fleet ?? m.run) + 3, end: m.settled + 2, speed: Math.max(1, (m.settled - (m.dragged ?? m.run) - 3) / 6), kicker: "04 · Fleet", title: "They work side by side, in isolated copies.", focus: null, sped: true},
  {start: review.t - 0.4, end: (m.read ?? m.review + 3) + 0.8, speed: 1.2, kicker: "05 · Review", title: "Read every change and its checks.", focus: null},
  {start: apply1.t - 0.4, end: m.stale + 2.6, speed: 1, kicker: "06 · Stale", title: "Something changed? Apply refuses.", focus: null},
  {start: refresh.t - 0.4, end: m.applied + 2.2, speed: Math.max(1, (m.applied - refresh.t) / 7), kicker: "07 · Apply", title: "Refresh, then Apply the exact reviewed bytes.", focus: null, sped: m.applied - refresh.t > 9},
].map((s) => ({...s, start: Math.max(0, s.start)}));
for (const s of shots) if (!(s.end > s.start)) throw new Error(`Empty shot: ${s.title}`);

execFileSync("ffmpeg", ["-v", "error", "-y", "-i", path.join(dir, "screen.mkv"), "-c:v", "libx264", "-preset", "slow", "-crf", "14", "-pix_fmt", "yuv420p", "-movflags", "+faststart", "-g", "30", path.join(out, "screen.mp4")]);
let result = null;
if (existsSync(path.join(dir, "result-frames.txt"))) {
  execFileSync("ffmpeg", ["-v", "error", "-y", "-f", "concat", "-safe", "0", "-i", "result-frames.txt", "-vf", "fps=60,scale=trunc(iw/2)*2:trunc(ih/2)*2,format=yuv420p", "-c:v", "libx264", "-crf", "14", "-movflags", "+faststart", path.join(out, "result.mp4")], {cwd: dir});
  result = "live/result.mp4";
}

const pointer = clicks.filter((p) => p.x !== undefined || p.kind === "click").map(({t, kind, x, y, label}) => ({t, kind, x, y, label}));
writeFileSync("live-cut.json", JSON.stringify({width: W, height: H, video: "live/screen.mp4", result, shots, pointer, marks: m}, null, 2) + "\n");
console.log(`${shots.length} shots from a ${W}x${H} take; ${pointer.length} pointer events`);
