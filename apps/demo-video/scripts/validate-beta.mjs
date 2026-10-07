import assert from "node:assert/strict";
import {createHash} from "node:crypto";
import {readFile} from "node:fs/promises";
import {fileURLToPath} from "node:url";

const root = new URL("../", import.meta.url);
const props = JSON.parse(await readFile(new URL("fleet-props.json", root), "utf8"));

// The edit depicts this recorded run, not an arbitrary successful fleet.
assert.equal(props.run.id, "348fcee9-e6d8-4888-b7e8-14c215b59ce9");
assert.equal(props.run.applyStatus, "applied");
assert.deepEqual(props.waves, [4, 1, 1]);
assert.deepEqual(props.workers.map(worker => worker.cli), ["codex", "claude", "cursor", "opencode", "agy", "codex"]);
assert.deepEqual(props.workers.map(worker => worker.filesPrepared.length), [2, 2, 2, 0, 0, 1]);
assert.equal(props.files.length, 7);
assert.equal(new Set(props.files.map(file => file.path)).size, 7);
assert.deepEqual(props.checks, {recorded: 6, passed: 6});
assert.equal(props.evidenceBoundary.continuousFootage, false);
assert.equal(props.evidenceBoundary.pointerMotion, false);
for (const worker of props.workers) {
  assert.deepEqual(props.files.filter(file => file.taskId === worker.taskId).map(file => file.path).sort(), [...worker.filesPrepared].sort());
}
for (const key of ["stale", "applied", "result"]) {
  const asset = props.assets.find(item => item.key === key);
  assert.ok(asset && /^fleet\/[a-z0-9-]+\.png$/.test(asset.path), `Invalid native asset: ${key}`);
  const bytes = await readFile(new URL(`public/${asset.path}`, root));
  assert.equal(createHash("sha256").update(bytes).digest("hex"), asset.sha256, `${key}: native bytes changed`);
  assert.equal(bytes.subarray(0, 8).toString("hex"), "89504e470d0a1a0a");
  assert.equal(bytes.readUInt32BE(16), asset.width, `${key}: width`);
  assert.equal(bytes.readUInt32BE(20), asset.height, `${key}: height`);
}
for (const filename of ["logo-mark.png", "fleet/logos/openai-on-dark.svg", "fleet/logos/anthropic.svg", "fleet/logos/cursor-on-dark.svg", "fleet/logos/opencode.svg", "fleet/logos/antigravity.png"]) {
  assert.ok((await readFile(new URL(`public/${filename}`, root))).length > 0, `Missing ${filename}`);
}
console.log(`Beta film: recorded facts and native asset hashes verified (${fileURLToPath(root)}).`);
console.log("Silent edited sequence. Native stills from the Oct 2 candidate; not final-build or continuous-run evidence.");
