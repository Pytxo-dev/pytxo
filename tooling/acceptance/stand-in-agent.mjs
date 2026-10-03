// Deterministic stand-in for an agent CLI, used only by cloud acceptance runs.
// It answers Desktop's readiness probes and, when run as a worker, appends one
// marked line to each file the task owns. It proves Pytxo's install, UI,
// scheduling, isolation, checks, Review and Apply on the real artifact; it says
// nothing about any vendor agent's quality.
//
//   node stand-in-agent.mjs <cli> [...vendor arguments]
import { existsSync, mkdirSync, readFileSync, writeFileSync } from "node:fs";
import path from "node:path";
import { setTimeout as delay } from "node:timers/promises";

const [cli, ...args] = process.argv.slice(2);
const said = (line) => process.stdout.write(`${line}\n`);
const joined = args.join(" ");

// Readiness probes (see apps/desktop/src-tauri/src/ipc_meta.rs).
if (args[0] === "--version") { said(`${cli} 0.0.0-pytxo-stand-in`); process.exit(0); }
if (cli === "codex" && joined === "login status") { said("Logged in using an API key (stand-in)"); process.exit(0); }
if (cli === "claude" && joined === "auth status --json") { said(JSON.stringify({ loggedIn: true })); process.exit(0); }
if (cli === "cursor-agent" && joined === "status") { said("Logged in (stand-in)"); process.exit(0); }
if (cli === "opencode" && joined === "auth list") { said("1 credentials (stand-in)"); process.exit(0); }

// Worker run: the runner always sets PYTXO_TASK_PROMPT (prompt plus handoff). Codex
// also receives it on stdin (`exec ... -`), so drain that pipe.
if (args.at(-1) === "-") readFileSync(0);
const prompt = process.env.PYTXO_TASK_PROMPT ?? "";
const owned = /Owned paths(?: \(JSON\))?: (\[[^\]]*\])/.exec(prompt)?.[1] ?? "[]";
const decode = (value) => value.replace(/\\u([0-9A-Fa-f]{4})/g, (_, hex) => String.fromCharCode(parseInt(hex, 16)));
const paths = owned.startsWith('["')
  ? JSON.parse(owned)
  : owned.slice(1, -1).split(";").map((entry) => decode(entry.trim())).filter(Boolean);
const task = /Task ID(?: \(JSON\))?: \[?"?([^\]"\s]+)/.exec(prompt)?.[1] ?? "task";
const note = `pytxo stand-in ${cli} ${task}`;

said(`Reading the request for ${task}`);
for (const file of paths) {
  const target = path.resolve(file);
  if (existsSync(target) && !path.extname(target)) continue; // An owned directory.
  await delay(1200);
  said(`Editing ${file}`);
  mkdirSync(path.dirname(target), { recursive: true });
  const before = existsSync(target) ? readFileSync(target, "utf8") : "";
  const ext = path.extname(target).toLowerCase();
  let after;
  if (ext === ".json") {
    const data = before.trim() ? JSON.parse(before) : {};
    data["pytxo.standIn"] = note;
    after = `${JSON.stringify(data, null, 2)}\n`;
  } else {
    const line = ext === ".css" ? `/* ${note} */`
      : ext === ".html" || ext === ".md" ? `<!-- ${note} -->`
      : `// ${note}`;
    after = `${before}${before && !before.endsWith("\n") ? "\n" : ""}${line}\n`;
  }
  writeFileSync(target, after);
  said(`+ ${path.basename(file)}: one marked line`);
}
await delay(800);
said(`Done: ${paths.length} owned path(s)`);
