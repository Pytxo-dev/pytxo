// Tempo, beat phase and loudness sections of a music file, for cutting to it.
//   node scripts/beats.mjs <audio>
// Prints BPM, the first beat (s) and a 1-second loudness strip with the
// biggest rises marked, so drops and breaks can be placed by hand.
import {execFileSync} from "node:child_process";

const file = process.argv[2];
if (!file) throw new Error("usage: beats.mjs <audio>");
const RATE = 11025;
const raw = execFileSync("ffmpeg", ["-v", "error", "-i", file, "-ac", "1", "-ar", String(RATE), "-f", "f32le", "-"], {maxBuffer: 1 << 30});
const pcm = new Float32Array(raw.buffer, raw.byteOffset, raw.byteLength / 4);

// Onset strength: positive change of log energy in 512-sample hops (~46 ms).
const HOP = 512;
const energy = [];
for (let i = 0; i + HOP <= pcm.length; i += HOP) {
  let sum = 0;
  for (let j = i; j < i + HOP; j++) sum += pcm[j] * pcm[j];
  energy.push(Math.log(1e-9 + sum / HOP));
}
const onset = energy.map((e, i) => Math.max(0, e - (energy[i - 1] ?? e)));
const hopSec = HOP / RATE;

// Tempo: the autocorrelation peak between 70 and 180 BPM.
let best = {bpm: 0, score: -1};
for (let bpm = 70; bpm <= 180; bpm += 0.25) {
  const lag = 60 / bpm / hopSec;
  let score = 0;
  for (let i = 0; i + lag * 4 < onset.length; i++) {
    const at = (k) => onset[Math.round(i + lag * k)];
    score += onset[i] * (at(1) + at(2) * .5 + at(4) * .25);
  }
  if (score > best.score) best = {bpm, score};
}
const period = 60 / best.bpm;
// Phase: the offset whose beat grid collects the most onset strength.
let phase = {t: 0, score: -1};
for (let t = 0; t < period; t += hopSec / 2) {
  let score = 0;
  for (let b = t; b < energy.length * hopSec; b += period) score += onset[Math.round(b / hopSec)] ?? 0;
  if (score > phase.score) phase = {t, score};
}

// Loudness per second (dB RMS), with the largest 2-second rises flagged.
const perSec = [];
for (let s = 0; s * RATE < pcm.length; s++) {
  let sum = 0, n = 0;
  for (let j = s * RATE; j < Math.min(pcm.length, (s + 1) * RATE); j++) { sum += pcm[j] * pcm[j]; n++; }
  perSec.push(10 * Math.log10(1e-12 + sum / n));
}
const rises = perSec.map((db, s) => ({s, rise: db - (perSec[s - 2] ?? db)})).sort((a, b) => b.rise - a.rise).slice(0, 6).map((r) => r.s);
console.log(`duration ${(pcm.length / RATE).toFixed(2)} s · ${best.bpm} BPM · beat ${period.toFixed(4)} s · first beat ${phase.t.toFixed(3)} s`);
console.log("biggest rises at (s):", rises.sort((a, b) => a - b).join(", "));
const lo = Math.min(...perSec), hi = Math.max(...perSec);
perSec.forEach((db, s) => console.log(`${String(s).padStart(4)}s ${db.toFixed(1).padStart(6)} dB ${"#".repeat(Math.round(((db - lo) / (hi - lo || 1)) * 40))}${rises.includes(s) ? "  <- rise" : ""}`));
