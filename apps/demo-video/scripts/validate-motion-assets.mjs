import assert from "node:assert/strict";
import {createHash} from "node:crypto";
import {readFile} from "node:fs/promises";

const root = new URL("../", import.meta.url);
const audio = JSON.parse(await readFile(new URL("motion-audio.json", root), "utf8"));
const props = JSON.parse(await readFile(new URL("fleet-props.json", root), "utf8"));
assert.equal(audio.durationSeconds, 50);
assert.equal(audio.music.license, "CC0-1.0");
for (const item of [audio.music, ...audio.sfx]) {
  assert.match(item.file, /^[a-z0-9_-]+\.(mp3|ogg)$/);
  const bytes = await readFile(new URL(`public/audio/cc0/${item.file}`, root));
  assert.equal(createHash("sha256").update(bytes).digest("hex"), item.sha256, `Audio source changed: ${item.file}`);
}
for (const cue of audio.cues) {
  assert.ok(audio.sfx.some(item => item.file === cue.file));
  assert.ok(cue.seconds >= 0 && cue.seconds < 49);
  assert.ok(cue.gain > 0 && cue.gain <= 0.3);
}
for (const key of ["board-4", "review", "stale", "applied", "result"]) {
  const asset = props.assets.find(item => item.key === key);
  assert.ok(asset && /^fleet\/[a-z0-9-]+\.png$/.test(asset.path));
  const bytes = await readFile(new URL(`public/${asset.path}`, root));
  assert.equal(createHash("sha256").update(bytes).digest("hex"), asset.sha256, `Native source changed: ${key}`);
}
console.log("Motion film: all five native captures and five CC0 audio sources match the manifest.");
