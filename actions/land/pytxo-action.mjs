// The Pytxo GitHub Action's two jobs around the check itself:
//   discover  find the open agent pull requests into this one's base branch and
//             fetch them as local branches (no tests run here)
//   comment   put the verdict in the job summary and in one pull-request
//             comment, updated in place
// The check step between them runs the tests without the token in its env.

import { spawnSync } from "node:child_process";
import fs from "node:fs";
import path from "node:path";
import { pathToFileURL } from "node:url";

export const DEFAULT_PATTERNS = ["claude/*", "codex/*", "copilot/*", "cursor/*", "devin/*", "jules/*", "agent/*"];

// Pull requests these accounts open are agent work whatever the branch is called.
export const AGENT_BOTS = new Set([
  "Copilot",
  "copilot-swe-agent[bot]",
  "claude[bot]",
  "cursor[bot]",
  "devin-ai-integration[bot]",
  "google-labs-jules[bot]",
]);

export const MARKER = "<!-- pytxo-check -->";
export const MAX_PRS = 10;

/** `*` matches any run of characters, slashes included; `?` matches one. */
export function glob(pattern, text) {
  const re = pattern.replace(/[.+^${}()|[\]\\]/g, "\\$&").replace(/\*/g, ".*").replace(/\?/g, ".");
  return new RegExp(`^${re}$`).test(text);
}

/** "claude/ codex/*" → ["claude/*", "codex/*"]; empty → the defaults. */
export function parsePatterns(text) {
  const patterns = (text || "")
    .split(/[\s,]+/)
    .filter(Boolean)
    .map((p) => (p.endsWith("/") ? `${p}*` : p));
  return patterns.length ? patterns : DEFAULT_PATTERNS;
}

export function isAgentPr(pr, patterns = DEFAULT_PATTERNS) {
  return AGENT_BOTS.has(pr.user?.login) || patterns.some((p) => glob(p, pr.head.ref));
}

const sameRepo = (pr) => pr.head.repo?.full_name === pr.base.repo.full_name;

/**
 * The pull requests to check together, in merge order: the other agent PRs
 * into the same base (oldest first), then this one, so a conflict names the
 * newest change. Fork PRs never join another PR's run: their code would run
 * with that run's token.
 */
export function selectPrs(current, open, patterns = DEFAULT_PATTERNS, max = MAX_PRS) {
  if (!isAgentPr(current, patterns)) {
    return { prs: [], reason: `#${current.number} is not an agent pull request (${patterns.join(", ")} or an agent account).` };
  }
  const others = open
    .filter((pr) => pr.number !== current.number && pr.base.ref === current.base.ref && isAgentPr(pr, patterns))
    .sort((a, b) => a.number - b.number);
  const forks = others.filter((pr) => !sameRepo(pr)).length;
  const eligible = others.filter(sameRepo);
  const left = eligible.slice(Math.max(0, eligible.length - (max - 1)));
  const prs = [...left, current];
  const notes = [];
  if (forks) notes.push(`${forks} agent pull request${forks === 1 ? "" : "s"} from forks left out.`);
  if (left.length < eligible.length) notes.push(`${eligible.length - left.length} older agent pull requests left out (at most ${max} at once).`);
  if (prs.length < 2) {
    return { prs, notes, reason: `#${current.number} is the only open agent pull request into ${current.base.ref}; there is nothing to combine.` };
  }
  return { prs, notes };
}

/** A safe local branch name for a PR head: its own name when that is usable. */
export function localName(pr, taken, base) {
  const name = pr.head.ref;
  const usable =
    /^[A-Za-z0-9][A-Za-z0-9._/-]*$/.test(name) &&
    !name.includes("..") &&
    !name.endsWith(".lock") &&
    !name.endsWith("/") &&
    name !== base &&
    !taken.has(name);
  const chosen = usable ? name : `pr-${pr.number}`;
  taken.add(chosen);
  return chosen;
}

async function api(fetchImpl, url, token, init = {}) {
  const response = await fetchImpl(url, {
    ...init,
    headers: {
      accept: "application/vnd.github+json",
      authorization: `Bearer ${token}`,
      "x-github-api-version": "2022-11-28",
      "user-agent": "pytxo-action",
      ...(init.body ? { "content-type": "application/json" } : {}),
    },
  });
  return response;
}

export async function listOpenPrs(fetchImpl, { apiUrl, repo, base, token }) {
  const prs = [];
  for (let page = 1; page <= 3; page++) {
    const url = `${apiUrl}/repos/${repo}/pulls?state=open&base=${encodeURIComponent(base)}&per_page=100&page=${page}`;
    const response = await api(fetchImpl, url, token);
    if (!response.ok) throw new Error(`listing pull requests: HTTP ${response.status}`);
    const batch = await response.json();
    prs.push(...batch);
    if (batch.length < 100) break;
  }
  return prs;
}

/** Updates Pytxo's comment on the PR, or creates it. "forbidden" when the token cannot write. */
export async function upsertComment(fetchImpl, { apiUrl, repo, number, token, body, create = true }) {
  const list = await api(fetchImpl, `${apiUrl}/repos/${repo}/issues/${number}/comments?per_page=100`, token);
  if (!list.ok) return list.status === 403 ? "forbidden" : `HTTP ${list.status}`;
  const existing = (await list.json()).find((c) => c.body?.includes(MARKER));
  if (!existing && !create) return "none";
  const response = existing
    ? await api(fetchImpl, `${apiUrl}/repos/${repo}/issues/comments/${existing.id}`, token, {
        method: "PATCH",
        body: JSON.stringify({ body }),
      })
    : await api(fetchImpl, `${apiUrl}/repos/${repo}/issues/${number}/comments`, token, {
        method: "POST",
        body: JSON.stringify({ body }),
      });
  if (response.ok) return existing ? "updated" : "created";
  return response.status === 403 ? "forbidden" : `HTTP ${response.status}`;
}

/** The comment: which PRs were checked, then the CLI's markdown verdict. */
export function commentBody({ prs, markdown, stderr, code, reason }) {
  const lines = [MARKER];
  if (reason) {
    lines.push(`### ⏸️ Nothing to check`, "", reason);
    return `${lines.join("\n")}\n`;
  }
  lines.push(`Checked together: ${prs.map((p) => `#${p.number} \`${p.branch}\``).join(" · ")}`, "");
  if (markdown.trim()) {
    lines.push(markdown.trim());
  } else {
    const tail = stderr.trim().split(/\r?\n/).slice(-20).join("\n");
    lines.push(`### ⚠️ Pytxo could not run the check (exit ${code})`, "", "```", tail || "no output", "```");
  }
  return `${lines.join("\n")}\n`;
}

function git(args, { auth } = {}) {
  const extra = auth ? ["-c", `http.${auth.server}/.extraheader=AUTHORIZATION: basic ${auth.basic}`] : [];
  const result = spawnSync("git", [...extra, ...args], { encoding: "utf8" });
  if (result.status !== 0) {
    throw new Error(`git ${args.join(" ")} failed: ${(result.stderr || "").trim()}`);
  }
  return result.stdout.trim();
}

function output(name, value) {
  fs.appendFileSync(process.env.GITHUB_OUTPUT, `${name}=${value}\n`);
}

const statePath = () => path.join(process.env.RUNNER_TEMP || ".", "pytxo-action.json");

async function discover() {
  const event = process.env.GITHUB_EVENT_NAME;
  if (event === "pull_request_target") {
    throw new Error(
      "refusing to run on pull_request_target: the check runs pull-request code, and pull_request_target gives it your secrets. Use `on: pull_request`.",
    );
  }
  if (event !== "pull_request") {
    throw new Error(`Pytxo checks pull requests; this run is a ${event} event. Use \`on: pull_request\`.`);
  }
  const payload = JSON.parse(fs.readFileSync(process.env.GITHUB_EVENT_PATH, "utf8"));
  const current = payload.pull_request;
  const token = process.env.PYTXO_TOKEN;
  const apiUrl = process.env.GITHUB_API_URL || "https://api.github.com";
  const repo = process.env.GITHUB_REPOSITORY;
  const patterns = parsePatterns(process.env.PYTXO_BRANCHES);
  const base = current.base.ref;

  const open = isAgentPr(current, patterns) ? await listOpenPrs(fetch, { apiUrl, repo, base, token }) : [];
  const selection = selectPrs(current, open, patterns);
  const state = { base, prs: [], notes: selection.notes || [], reason: selection.reason, number: current.number };
  if (selection.reason) {
    fs.writeFileSync(statePath(), JSON.stringify(state));
    console.log(selection.reason);
    output("skip", "true");
    return;
  }

  // Credentials only for these fetches, unless the checkout already has some.
  const server = process.env.GITHUB_SERVER_URL || "https://github.com";
  const configured = spawnSync("git", ["config", "--get", `http.${server}/.extraheader`], { encoding: "utf8" });
  const auth =
    token && !configured.stdout.trim() ? { server, basic: Buffer.from(`x-access-token:${token}`).toString("base64") } : undefined;
  if (git(["rev-parse", "--is-shallow-repository"]) === "true") {
    git(["fetch", "--quiet", "--unshallow", "--no-tags", "origin"], { auth });
  }
  const taken = new Set([base]);
  const prs = selection.prs.map((pr) => ({ number: pr.number, sha: pr.head.sha, branch: localName(pr, taken, base) }));
  git(
    [
      "fetch",
      "--quiet",
      "--no-tags",
      "--update-head-ok",
      "origin",
      `+refs/heads/${base}:refs/heads/${base}`,
      ...prs.map((p) => `+refs/pull/${p.number}/head:refs/heads/${p.branch}`),
    ],
    { auth },
  );
  state.prs = prs;
  fs.writeFileSync(statePath(), JSON.stringify(state));
  fs.writeFileSync(path.join(process.env.RUNNER_TEMP || ".", "pytxo-branches.txt"), `${prs.map((p) => p.branch).join("\n")}\n`);
  console.log(`Checking ${prs.map((p) => `#${p.number} ${p.branch}`).join(", ")} against ${base}`);
  output("skip", "false");
  output("base", base);
}

async function comment() {
  const temp = process.env.RUNNER_TEMP || ".";
  const read = (name) => (fs.existsSync(path.join(temp, name)) ? fs.readFileSync(path.join(temp, name), "utf8") : "");
  const state = JSON.parse(read("pytxo-action.json") || "{}");
  if (!state.number) return;
  const body = commentBody({
    prs: state.prs,
    markdown: read("pytxo-check.md"),
    stderr: read("pytxo-check.err"),
    code: process.env.PYTXO_CODE || "?",
    reason: state.reason,
  });
  const notes = state.notes?.length ? `\n${state.notes.map((n) => `> ${n}`).join("\n")}\n` : "";
  fs.appendFileSync(process.env.GITHUB_STEP_SUMMARY, `${body}${notes}`);
  if (process.env.PYTXO_COMMENT === "false") return;
  const result = await upsertComment(fetch, {
    apiUrl: process.env.GITHUB_API_URL || "https://api.github.com",
    repo: process.env.GITHUB_REPOSITORY,
    number: state.number,
    token: process.env.PYTXO_TOKEN,
    body: `${body}${notes}`,
    // "Nothing to combine" only updates a comment that is already there.
    create: !state.reason,
  });
  if (result === "forbidden") {
    console.log("::notice::This token cannot comment (a pull request from a fork gets a read-only one). The verdict is in the job summary.");
  } else if (!["created", "updated", "none"].includes(result)) {
    console.log(`::warning::Could not comment on #${state.number} (${result}). The verdict is in the job summary.`);
  }
}

const invoked = process.argv[1] && import.meta.url === pathToFileURL(path.resolve(process.argv[1])).href;
if (invoked) {
  const run = { discover, comment }[process.argv[2]];
  if (!run) {
    console.error("usage: pytxo-action.mjs discover|comment");
    process.exit(1);
  }
  run().catch((error) => {
    console.log(`::error::${error.message}`);
    process.exit(1);
  });
}
