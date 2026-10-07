import assert from 'node:assert/strict';
import fs from 'node:fs/promises';
import ts from 'typescript';

const source = await fs.readFile(new URL('../src/aperture/config.ts', import.meta.url), 'utf8');
const js = ts.transpileModule(source, {compilerOptions: {target: ts.ScriptTarget.ES2022, module: ts.ModuleKind.ES2022}}).outputText;
const {parseApertureFilmProps} = await import(`data:text/javascript;base64,${Buffer.from(js).toString('base64')}`);
const base = JSON.parse(await fs.readFile(new URL('../aperture-props.json', import.meta.url), 'utf8'));
parseApertureFilmProps(base);
let rejected = 0;
function rejects(change, message) {
  const input = structuredClone(base);
  change(input);
  assert.throws(() => parseApertureFilmProps(input), message);
  rejected++;
}
rejects(p => {delete p.privacyEdits;}, /privacyEdits must be an object/);
rejects(p => {p.privacyEdits.apply.sourceSha256 = '0'.repeat(64);}, /different footage/);
rejects(p => {p.privacyEdits.apply.regions = null;}, /explicit array/);
const valid = {fromFrame: 0, untilFrame: 1, x: 1, y: 1, width: 180, height: 30, label: 'Host path redacted'};
for (const change of [
  {fromFrame: -1}, {untilFrame: 0}, {untilFrame: 661}, {fromFrame: .5},
  {x: 1599}, {y: 999}, {width: 0}, {height: 0}, {width: 159}, {height: 15}, {label: 'Native path'},
]) {
  rejects(p => {p.privacyEdits.apply.regions = [{...valid, ...change}];}, /privacy/);
}
console.log(`Privacy manifest: current record accepted; ${rejected} missing, stale, out-of-bounds or unlabeled edits rejected. This checks the edit contract, not visual coverage.`);
