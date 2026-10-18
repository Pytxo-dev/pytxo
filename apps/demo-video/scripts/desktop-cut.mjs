// Turns a native Desktop journey take (screen.mkv + receipt telemetry) into the
// PytxoDesktop shot list. Every shot is a range of the real recording; speed
// only shortens waits, and a shot faster than 1.5x says so on screen.
//   node scripts/desktop-cut.mjs <journey-dir> "<agents that worked, as credited>"
// Writes public/desktop/shotN.mp4 (taskbar cropped) and desktop-cut.json.
import {execFileSync} from "node:child_process";
import {mkdirSync, readFileSync, writeFileSync} from "node:fs";
import path from "node:path";

const dir = process.argv[2];
if (!dir) throw new Error("usage: desktop-cut.mjs <journey-dir>");
const receipt = JSON.parse(readFileSync(path.join(dir, "receipt.json"), "utf8"));
const screen = JSON.parse(readFileSync(path.join(dir, "screen.json"), "utf8"));
if (receipt.result !== "passed") throw new Error("Only a passed journey take can be cut.");
const out = path.resolve("public/desktop");
mkdirSync(out, {recursive: true});

const sec = (epoch) => (epoch - screen.first) / 1000;
const m = Object.fromEntries(Object.entries(receipt.marks).map(([k, v]) => [k, sec(v)]));
const pointer = receipt.pointer.filter((p) => p.surface === "desktop").map((p) => ({...p, t: sec(p.t)}));
const hover = (label, after = 0) => pointer.find((p) => p.kind === "hover" && p.label === label && p.t >= after);
const [W, FULL_H] = execFileSync("ffprobe", ["-v", "error", "-select_streams", "v:0", "-show_entries", "stream=width,height", "-of", "csv=p=0", path.join(dir, "screen.mkv")]).toString().trim().split(",").map(Number);
// The Windows taskbar (60 px at 125 %) is cropped away.
const H = FULL_H - 60;

const firstAgent = pointer.find((p) => p.kind === "hover" && ["OpenAI Codex", "Claude Code", "Cursor Agent", "OpenCode", "Antigravity"].includes(p.label));
const apply1 = hover("Apply reviewed changes", m.read ?? m.review);
const refresh = hover("Refresh review");
// Shots last whole bars of the track (desktop-music.json), so every cut lands
// on a downbeat. An anchor puts one recorded moment on a chosen bar line of
// its shot. cap: the fastest a shot may run; slower shots keep their end.
const track = JSON.parse(readFileSync("desktop-music.json", "utf8"));
const BAR = 4 * 60 / track.bpm;
const plan = [
  {id: "describe", start: m.typing - 1.2, end: m.typed + 0.4, bars: 3, cap: 6},
  ...(m.split ? [{id: "split", start: hover("Split").t - 0.4, end: m.split + 2.6, bars: 3, cap: 80, anchor: ["split", 2]}] : []),
  {id: "agents", start: firstAgent.t - 0.4, end: hover("Build plan").t + 0.6, bars: 2, cap: 3},
  {id: "plan", start: m.plan - 0.2, end: hover("Run").t + 1.0, bars: 3, cap: 3},
  {id: "fleet", start: (m.dragged ?? m.fleet ?? m.run) + 3, end: m.settled + 2.4, bars: 6, cap: 300, anchor: ["settled", 5]},
  {id: "review", start: hover("Review changes").t - 0.4, end: (m.read ?? m.review + 3) - 1.3, bars: 4, cap: 2.5},
  {id: "stale", start: apply1.t - 0.4, end: m.stale + 3.0, bars: 4, cap: 4, anchor: ["stale", 2.5]},
  {id: "apply", start: refresh.t - 0.4, end: m.applied + 2.6, bars: 4, cap: 10, anchor: ["applied", 2]},
  {id: "history", start: m.history - 0.3, end: m.history + 5.6, bars: 3, cap: 1.4},
];
const shots = plan.map(({bars, cap, anchor, ...s}) => {
  const length = bars * BAR;
  if (!anchor) {
    const speed = Math.min(cap, Math.max(1, (s.end - s.start) / length));
    const start = Math.max(0, s.end - speed * length);
    return {id: s.id, start, end: start + speed * length, speed, bars};
  }
  // Anchored: a speed ramp. The wait before the moment plays fast and the moment lands on
  // its bar line; from there the shot runs in real time, so it never runs into the next scene.
  const at = m[anchor[0]], ramp = anchor[1] * BAR;
  const speed = Math.min(cap, Math.max(1, (at - s.start) / ramp));
  const start = Math.max(0, at - speed * ramp);
  return {id: s.id, start, end: at + (length - ramp), speed, bars, ramp, at};
});
for (const s of shots) if (!(s.end > s.start)) throw new Error(`Empty shot: ${s.id}`);

// One short clip per shot at its cut speed (every frame a keyframe: exact compositor seeks), so renders never seek the long take.
shots.forEach((s, i) => {
  const crop = `crop=${W}:${H}:0:0`, hold = "fps=60,tpad=stop_mode=clone:stop_duration=1.5";
  const sped = `setpts=(PTS-STARTPTS)/${s.speed.toFixed(4)}`;
  const filter = s.ramp === undefined ? ["-vf", `${crop},${sped},${hold}`]
    : ["-filter_complex", `[0:v]${crop},split[a][b];[a]trim=end=${(s.at - s.start).toFixed(3)},${sped}[x];[b]trim=start=${(s.at - s.start).toFixed(3)},setpts=PTS-STARTPTS[y];[x][y]concat=n=2:v=1:a=0,${hold}`];
  execFileSync("ffmpeg", ["-v", "error", "-y", "-ss", s.start.toFixed(3), "-to", s.end.toFixed(3), "-i", path.join(dir, "screen.mkv"), "-an",
    ...filter,
    "-c:v", "libx264", "-preset", "fast", "-crf", "12", "-g", "1", "-pix_fmt", "yuv420p", "-movflags", "+faststart", path.join(out, `shot${i}.mp4`)]);
  s.clip = `desktop/shot${i}.mp4`;
});

const moves = pointer.filter((p) => p.x !== undefined || p.kind === "click").map(({t, kind, x, y, label}) => ({t, kind, x, y, label}));
writeFileSync("desktop-cut.json", JSON.stringify({width: W, height: H, lead: receipt.lead ?? null, team: process.argv[3] ?? receipt.lead, fleet: receipt.checks.fleet, review: receipt.checks.review, shots, pointer: moves, marks: m}, null, 2) + "\n");
console.log(shots.map((s) => `${s.id} ${(s.end - s.start).toFixed(1)}s @${s.speed.toFixed(2)}x`).join("\n"));
