import {readFile, mkdir, writeFile} from "node:fs/promises";
import {spawnSync} from "node:child_process";
import {fileURLToPath} from "node:url";

const root = fileURLToPath(new URL("../", import.meta.url));
const config = JSON.parse(await readFile(new URL("../motion-audio.json", import.meta.url), "utf8"));
await mkdir(new URL("../out/motion-review/", import.meta.url), {recursive: true});
const run = args => {
  const result = spawnSync("ffmpeg", ["-hide_banner", "-y", ...args], {cwd: root, encoding: "utf8", maxBuffer: 16 * 1024 * 1024});
  if (result.error) throw result.error;
  if (result.status !== 0) throw new Error(result.stderr);
  return result.stderr;
};
const input = file => `public/audio/cc0/${file}`;
const args = ["-i", input(config.music.file)];
config.cues.forEach(cue => args.push("-i", input(cue.file)));
// A short, eased dip makes the stale boundary audible without a theatrical alarm.
const dip = "if(lt(t,27.95),1,if(lt(t,28.13),1-(t-27.95)/0.18*0.88,if(lt(t,29.05),0.12,if(lt(t,29.65),0.12+(t-29.05)/0.6*0.88,1))))";
const filters = [`[0:a]atrim=start=${config.music.trimStartSeconds}:duration=50,asetpts=PTS-STARTPTS,aresample=48000,volume='${config.music.gain}*${dip}':eval=frame,afade=t=in:d=0.25,afade=t=out:st=48.5:d=1.5[music]`];
config.cues.forEach((cue, i) => filters.push(`[${i + 1}:a]aresample=48000,volume=${cue.gain},adelay=${Math.round(cue.seconds * 1000)}:all=1[s${i}]`));
filters.push(`[music]${config.cues.map((_, i) => `[s${i}]`).join("")}amix=inputs=${config.cues.length + 1}:normalize=0:duration=first,atrim=duration=50[mix]`);
run([...args, "-filter_complex", filters.join(";"), "-map", "[mix]", "-ar", "48000", "-ac", "2", "-c:a", "pcm_s24le", "out/motion-review/mix-raw.wav"]);
const analysis = run(["-i", "out/motion-review/mix-raw.wav", "-af", "loudnorm=I=-16:TP=-2:LRA=11:print_format=json", "-f", "null", "-"]);
const match = [...analysis.matchAll(/\{\s*"input_i"[\s\S]*?\}/g)].at(-1);
if (!match) throw new Error("Missing loudness measurement");
const measured = JSON.parse(match[0]);
if (![measured.input_i, measured.input_tp, measured.input_lra, measured.input_thresh, measured.target_offset].every(value => Number.isFinite(Number(value)))) throw new Error("Invalid loudness measurement");
const normalize = `loudnorm=I=-16:TP=-2:LRA=11:measured_I=${measured.input_i}:measured_TP=${measured.input_tp}:measured_LRA=${measured.input_lra}:measured_thresh=${measured.input_thresh}:offset=${measured.target_offset}:linear=true:print_format=json`;
run(["-i", "out/motion-review/mix-raw.wav", "-af", normalize, "-ar", "48000", "-ac", "2", "-c:a", "pcm_s24le", "public/audio/cc0/motion-mix.wav"]);
await writeFile(new URL("../out/motion-review/audio-mix.json", import.meta.url), JSON.stringify({targetLufs: -16, targetTruePeak: -2, sourceMeasurement: measured, cues: config.cues}, null, 2));
console.log("50-second stereo score mixed; final encoded audio still requires QA.");
