// Turns one cloud acceptance journey (tooling/acceptance) into film-cut.json for
// PytxoFilm: footage files, film marks and pointer telemetry in video seconds,
// and the facts the captions state, all from that run's own evidence.
//
//   node scripts/film-cut.mjs --acceptance <acceptance artifact dir> --footage <footage artifact dir> [--run <id>]
//
// The acceptance dir holds acceptance.json, fixture-applied.diff and journey/
// (receipt.json, screen.json, result.json); the footage dir holds screen.mp4
// (the native screen, real pointer included) and result-frames.mp4.
import { copyFileSync, mkdirSync, readFileSync, writeFileSync } from "node:fs";
import path from "node:path";

const args = Object.fromEntries(process.argv.slice(2).reduce((pairs, value, index, all) => (value.startsWith("--") ? [...pairs, [value.slice(2), all[index + 1]]] : pairs), []));
if (!args.acceptance || !args.footage) throw new Error("usage: --acceptance <dir> --footage <dir> [--run <id>]");
const read = (...parts) => JSON.parse(readFileSync(path.join(...parts), "utf8").replace(/^﻿/, ""));
const acceptance = read(args.acceptance, "acceptance.json");
const receipt = read(args.acceptance, "journey", "receipt.json");
const screen = read(args.acceptance, "journey", "screen.json");
const result = read(args.acceptance, "journey", "result.json");
if (receipt.result !== "passed") throw new Error(`Journey ${receipt.result}; film needs a passed run`);

const media = path.join(import.meta.dirname, "..", "public", "acceptance");
mkdirSync(media, { recursive: true });
for (const take of ["screen", "result-frames"]) copyFileSync(path.join(args.footage, `${take}.mp4`), path.join(media, `${take}.mp4`));

const seconds = (epoch, first) => Math.round((epoch - first) / 10) / 100;
const marks = {
  ...Object.fromEntries(Object.entries(receipt.marks).map(([name, t]) => [name, seconds(t, screen.first)])),
  ...Object.fromEntries(Object.entries(result.marks).map(([name, t]) => [name, seconds(t, result.first)])),
};
const pointer = [
  ...receipt.pointer.map(({ t, ...rest }) => ({ s: seconds(t, screen.first), ...rest })),
  ...result.pointer.map(({ t, ...rest }) => ({ s: seconds(t, result.first), ...rest })),
];

const diff = readFileSync(path.join(args.acceptance, "fixture-applied.diff"), "utf8").split(/\r?\n/);
const { checks } = receipt;
const cut = {
  source: { run: args.run ?? null, msiSha256: acceptance.msi_sha256, exeSha256: acceptance.install.exe_sha256, version: acceptance.install.display_version },
  desktop: { src: "acceptance/screen.mp4", duration: seconds(receipt.finished, screen.first) },
  result: { src: "acceptance/result-frames.mp4", duration: seconds(result.pointer.at(-1)?.t ?? result.first, result.first) + 3 },
  marks,
  pointer,
  facts: {
    workers: checks.fleet.workers,
    completed: checks.fleet.completed,
    files: checks.review.files.length,
    added: diff.filter((line) => line.startsWith("+") && !line.startsWith("+++")).length,
    removed: diff.filter((line) => line.startsWith("-") && !line.startsWith("---")).length,
    digest: checks.review.digest,
    refreshedDigest: checks.refreshed.digest,
    applied: checks.apply.status,
    testsAfterApply: acceptance.journey.tests_after_apply,
    replayed: acceptance.journey.replayed_files,
    resultLanguage: result.lang,
  },
};
const out = path.join(import.meta.dirname, "..", "film-cut.json");
writeFileSync(out, `${JSON.stringify(cut, null, 2)}\n`);
console.log(`wrote ${out}: ${Object.keys(marks).length} marks, ${pointer.length} pointer events, ${cut.facts.files} files +${cut.facts.added} -${cut.facts.removed}, replayed ${cut.facts.replayed}`);
