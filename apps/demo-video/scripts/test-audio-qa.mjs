import assert from "node:assert/strict";
import {
  evaluateAudioQa,
  parseLoudnorm,
  parseSilenceDetect,
} from "./audio-qa.mjs";

const loudness = parseLoudnorm(`
[Parsed_loudnorm_0] {
  "input_i" : "-16.20",
  "input_tp" : "-1.30",
  "input_lra" : "3.10",
  "input_thresh" : "-26.40"
}`);
assert.deepEqual(loudness, {
  integratedLufs: -16.2,
  truePeakDbtp: -1.3,
  thresholdDb: -26.4,
});
assert.deepEqual(
  evaluateAudioQa({loudness, silenceSegments: [], durationSeconds: 52}),
  [],
);

for (const [label, override, expected] of [
  ["too quiet", {integratedLufs: -19}, "audio loudness"],
  ["too loud", {integratedLufs: -13}, "audio loudness"],
  ["peak violation", {truePeakDbtp: -0.5}, "audio true peak"],
]) {
  const errors = evaluateAudioQa({
    loudness: {...loudness, ...override},
    silenceSegments: [],
    durationSeconds: 52,
  });
  assert.ok(errors.some((error) => error.includes(expected)), label);
}

const silences = parseSilenceDetect(
  `
[silencedetect] silence_start: 0
[silencedetect] silence_end: 1.25 | silence_duration: 1.25
[silencedetect] silence_start: 10
[silencedetect] silence_end: 12.5 | silence_duration: 2.5
[silencedetect] silence_start: 50.5
`,
  52,
);
assert.deepEqual(silences, [
  {start: 0, end: 1.25, duration: 1.25},
  {start: 10, end: 12.5, duration: 2.5},
  {start: 50.5, end: 52, duration: 1.5},
]);
const silenceErrors = evaluateAudioQa({
  loudness,
  silenceSegments: silences,
  durationSeconds: 52,
});
assert.ok(silenceErrors.some((error) => error.includes("leading silence")));
assert.ok(silenceErrors.some((error) => error.includes("internal silence")));
assert.ok(silenceErrors.some((error) => error.includes("trailing silence")));

const fullProgramErrors = evaluateAudioQa({
  loudness: {...loudness, integratedLufs: null},
  silenceSegments: [{start: 0, end: 52, duration: 52}],
  durationSeconds: 52,
});
assert.ok(fullProgramErrors.some((error) => error.includes("not finite")));
assert.ok(fullProgramErrors.some((error) => error.includes("full-program")));

process.stdout.write("Audio QA parser and thresholds passed deterministic tests.\n");
