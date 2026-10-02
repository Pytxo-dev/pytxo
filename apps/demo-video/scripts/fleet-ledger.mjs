// Builds fleet-props.json for PytxoFleetFilm from one recorded native run.
// Every number and worker line in the film comes from this ledger export.
//
//   node scripts/fleet-ledger.mjs --evidence D:/pytxo-native-acceptance/<root> \
//     --repo D:/.../taskboard [--stale-digest <first digest refused as stale>] //     [--stills <dir of native PNGs named by film key>]
import { DatabaseSync } from "node:sqlite";
import { copyFileSync, mkdirSync, readFileSync, writeFileSync } from "node:fs";
import { createHash } from "node:crypto";
import path from "node:path";
import { eventLines } from "../../desktop/src/lib/terminal-text.ts";

const args = Object.fromEntries(
  process.argv.slice(2).reduce((pairs, value, index, all) => (value.startsWith("--") ? [...pairs, [value.slice(2), all[index + 1]]] : pairs), []),
);
if (!args.evidence || !args.repo) throw new Error("usage: --evidence <acceptance root> --repo <fixture repo> [--stale-digest <hex>] [--out file]");

const CLIS = {
  codex: "OpenAI Codex",
  claude: "Claude Code",
  cursor: "Cursor Agent",
  opencode: "OpenCode",
  agy: "Antigravity",
};
// Workers read the operator's own agent configuration (skills, memories, MCP
// servers), so film text is allow-listed: commands, check results, added code
// and lines about the worker's owned files. Host paths and secrets never pass.
const PRIVATE = /mcp|[A-Za-z]:\\|\\Users\\|\/Users\/|B:\/~BUN|at <anonymous>|chunk-|api[_-]?key|token|session id|skill|memor|agents\.md/i;
// Tool plumbing that is accurate but unreadable on screen.
const NOISE = /matt|workflow|guidance|Get-Content|LiteralPath|Select-Object|\brg -|diff --git|^---|^\+\+\+|"source"|"scaffolded"|projfs|\.pytxo|PYTXO_|-a----|'"'|^-|-g '|upper|agent-\d/i;
const filmLine = (paths) => (line) =>
  !PRIVATE.test(line)
  && !NOISE.test(line)
  && (/^(\$ |✓|✗)/.test(line) || /^\+[^+]/.test(line) || paths.some((owned) => line.includes(owned)));

const launch = JSON.parse(readFileSync(path.join(args.evidence, "launch.json"), "utf8").replace(/^\uFEFF/, ""));
const store = new DatabaseSync(path.join(args.repo, ".pytxo", "data", "pytxo.db"), { readOnly: true });
const catalog = new DatabaseSync(path.join(args.evidence, "home", ".pytxo", "hypervisor.db"), { readOnly: true });

const run = store.prepare("select id, started_at, finished_at, status from runs order by started_at desc limit 1").get();
const contract = store.prepare("select prepared_manifest_json, prepared_digest, apply_status, applied_at from run_contracts where run_id = ?").get(run.id);
const draft = catalog.prepare("select title, mission_text, plan_json from flow_drafts where dispatched_run_id = ?").get(run.id);
const plan = JSON.parse(draft.plan_json);
const manifest = JSON.parse(contract.prepared_manifest_json);
const checks = manifest.candidate_verification?.checks ?? [];

const agents = store.prepare("select id, task_id, wave, status, exit_code from agents where run_id = ? order by task_id").all(run.id);
const events = (agentId) => store.prepare("select id, agent_id, ts, kind, payload from events where agent_id = ? order by id").all(agentId);

const workers = plan.tasks.map((task) => {
  const agent = agents.find((candidate) => candidate.task_id === task.id);
  const all = agent ? events(agent.id) : [];
  // Builds before 2026-10-02 appended each worker's whole output again after the
  // plan finished; nothing after the worker's last check result is the worker.
  const settled = all.findLastIndex((event) => ["agent-exit", "verify-ok", "verify-failed"].includes(event.kind));
  const recorded = settled >= 0 ? all.slice(0, settled + 1) : all;
  const cli = task.ade_id ?? plan.ade.requested;
  const output = eventLines(recorded)
    .filter(filmLine(task.paths))
    .map((line) => (line.length > 88 ? `${line.slice(0, 87)}…` : line))
    .slice(0, 48);
  const start = recorded.find((event) => event.kind === "agent-start")?.ts ?? null;
  const end = recorded.findLast((event) => ["agent-exit", "verify-ok", "verify-failed"].includes(event.kind))?.ts ?? null;
  return {
    taskId: task.id,
    wave: plan.waves.findIndex((wave) => wave.includes(task.id)),
    request: task.prompt,
    cli,
    vendor: CLIS[cli] ?? cli,
    paths: task.paths,
    dependsOn: task.dependencies,
    status: agent?.status ?? "not started",
    exitCode: agent?.exit_code ?? null,
    startedAt: start,
    endedAt: end,
    filesPrepared: manifest.files.filter((file) => file.task_id === task.id).map((file) => file.path),
    output,
  };
});

// Native captures, named by the beat that shows them, are copied next to the
// film and hash-locked so a render always shows the recorded run it claims.
const STILLS = ["board-1", "board-2", "board-3", "board-4", "review", "stale", "applied", "result"];
const assets = args.stills
  ? STILLS.map((key) => {
      const bytes = readFileSync(path.join(args.stills, `${key}.png`));
      const target = path.join(import.meta.dirname, "..", "public", "fleet", `${key}.png`);
      mkdirSync(path.dirname(target), { recursive: true });
      copyFileSync(path.join(args.stills, `${key}.png`), target);
      return { key, path: `fleet/${key}.png`, sha256: createHash("sha256").update(bytes).digest("hex"), width: bytes.readUInt32BE(16), height: bytes.readUInt32BE(20) };
    })
  : [];

const seconds = (from, to) => Math.round((Date.parse(to) - Date.parse(from)) / 1000);
const props = {
  schemaVersion: 1,
  kind: "fleet-film",
  candidate: { version: "1.2.2", msiSha256: launch.msi_sha256, executableSha256: launch.exe_sha256 },
  run: {
    id: run.id,
    title: draft.title,
    status: run.status,
    durationSeconds: seconds(run.started_at, run.finished_at),
    packageDigest: contract.prepared_digest,
    staleDigest: args["stale-digest"] ?? null,
    applyStatus: contract.apply_status,
    appliedAt: contract.applied_at,
  },
  waves: plan.waves.map((wave) => wave.length),
  vendors: [...new Set(workers.map((worker) => worker.vendor))],
  workers,
  files: manifest.files.map((file) => ({ path: file.path, kind: file.kind, taskId: file.task_id, vendor: workers.find((worker) => worker.taskId === file.task_id)?.vendor })),
  checks: { recorded: checks.length, passed: checks.filter((check) => check.passed).length },
  evidenceBoundary: {
    continuousFootage: false,
    pointerMotion: false,
    label: "Recorded native run · time compressed · native stills, continuous capture pending",
  },
  assets,
};

const out = args.out ?? path.join(import.meta.dirname, "..", "fleet-props.json");
writeFileSync(out, `${JSON.stringify(props, null, 2)}\n`);
console.log(`wrote ${out}: ${workers.length} workers, ${props.vendors.length} vendors, ${props.files.length} files, digest ${props.run.packageDigest.slice(0, 12)}…`);
