// Exports the real Apply result of the recorded October 2 fleet run for the film:
// each applied file's added/removed lines, with the task that prepared it.
//
//   node scripts/film-diff.mjs --repo <applied fixture repo> [--out film-diff.json]
//
// Reads only the fixture repository (agent-written demo code). Lines are checked
// against the same privacy pattern as the worker-output export.
import { execFileSync } from "node:child_process";
import { readFileSync, writeFileSync } from "node:fs";
import path from "node:path";

const args = Object.fromEntries(process.argv.slice(2).reduce((pairs, value, index, all) => (value.startsWith("--") ? [...pairs, [value.slice(2), all[index + 1]]] : pairs), []));
if (!args.repo) throw new Error("usage: --repo <applied fixture repo>");
const here = path.dirname(new URL(import.meta.url).pathname.replace(/^\/([A-Za-z]:)/, "$1"));
const props = JSON.parse(readFileSync(path.join(here, "..", "fleet-props.json"), "utf8"));
// File contents are demo fixture code, so only host paths and credentials are refused here.
const PRIVATE = /[A-Za-z]:\\|\\Users\\|\/Users\/|api[_-]?key|secret|password|session id/i;

const owner = (file) => props.workers.find((worker) => worker.filesPrepared?.includes(file));
const files = execFileSync("git", ["-C", args.repo, "diff", "--name-only", "HEAD"], { encoding: "utf8" }).split(/\r?\n/).filter(Boolean).sort();
const out = files.map((file) => {
  const diff = execFileSync("git", ["-C", args.repo, "-c", "core.autocrlf=false", "diff", "--no-color", "-U2", "HEAD", "--", file], { encoding: "utf8" });
  const lines = [];
  let before = 0, after = 0;
  for (const raw of diff.replace(/\r/g, "").split("\n")) {
    const hunk = /^@@ -(\d+)(?:,\d+)? \+(\d+)/.exec(raw);
    if (hunk) { before = +hunk[1]; after = +hunk[2]; lines.push({ kind: "hunk" }); continue; }
    if (/^(diff|index|---|\+\+\+)/.test(raw)) continue;
    const kind = raw.startsWith("+") ? "add" : raw.startsWith("-") ? "del" : raw.startsWith(" ") ? "same" : null;
    if (!kind) continue;
    const text = raw.slice(1);
    if (PRIVATE.test(text)) throw new Error(`Private-looking line in ${file}: ${text}`);
    lines.push({ kind, text, before: kind === "add" ? null : before, after: kind === "del" ? null : after });
    if (kind !== "add") before++;
    if (kind !== "del") after++;
  }
  const worker = owner(file);
  return { path: file, task: worker?.taskId ?? null, cli: worker?.cli ?? null, added: lines.filter((l) => l.kind === "add").length, removed: lines.filter((l) => l.kind === "del").length, lines };
});
const target = path.resolve(args.out ?? path.join(here, "..", "film-diff.json"));
writeFileSync(target, `${JSON.stringify({ source: "Recorded native fleet run, 2 October 2026 (run 348fcee9); applied fixture files", packageDigest: props.run.packageDigest, files: out }, null, 2)}\n`);
console.log(`${out.length} files, ${out.reduce((n, f) => n + f.added, 0)} added / ${out.reduce((n, f) => n + f.removed, 0)} removed lines -> ${target}`);
