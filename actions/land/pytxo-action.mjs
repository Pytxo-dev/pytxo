// The Pytxo GitHub Action's two jobs around the check itself:
//   discover  find the open agent pull requests into this one's base branch and
//             fetch them as local branches (no tests run here)
//   comment   put the verdict in the job summary and in one pull-request
//             comment, updated in place
// The check step between them runs the tests without the token in its env.

import { spawnSync } from "node:child_process";
import crypto from "node:crypto";
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
/** The line every comment ends with, as the CLI's markdown does. */
export const INSTALL_HINT =
  "<sub>Add this check to any repo in one line: `uses: Pytxo-dev/pytxo/actions/land@v2` · see it work: `npx pytxo demo`</sub>";

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

/**
 * Updates Pytxo's comment on the PR, or creates it. Only a bot's comment
 * counts (anyone can paste the marker), and every page is searched.
 * "forbidden" when the token cannot write.
 */
export async function upsertComment(fetchImpl, { apiUrl, repo, number, token, body, create = true }) {
  let existing;
  for (let page = 1; !existing; page++) {
    const url = `${apiUrl}/repos/${repo}/issues/${number}/comments?per_page=100&page=${page}`;
    const list = await api(fetchImpl, url, token);
    if (!list.ok) return list.status === 403 ? "forbidden" : `HTTP ${list.status}`;
    const batch = await list.json();
    existing = batch.find((c) => c.user?.type === "Bot" && c.body?.includes(MARKER));
    if (batch.length < 100) break;
  }
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

/**
 * The comment: which PRs were checked, the CLI's verdict (or why there is
 * none), the notes, then the install line. The CLI's own closing lines (its
 * install line and verdict line) make way for the notes.
 */
export function commentBody({ prs, markdown, stderr, code, reason, notes = [] }) {
  const lines = [MARKER];
  if (reason) {
    lines.push(`### ⏸️ Nothing to check`, "", reason);
  } else {
    lines.push(`Checked together: ${prs.map((p) => `#${p.number} \`${p.branch}\``).join(" · ")}`, "");
    const verdict = markdown.trim().split(/\r?\n/);
    while (verdict.length && /^(\s*|<sub>.*<\/sub>|<!-- pytxo-verdict .*-->)$/.test(verdict.at(-1))) verdict.pop();
    if (verdict.length) {
      lines.push(verdict.join("\n"));
    } else {
      const tail = stderr.trim().split(/\r?\n/).slice(-20).join("\n");
      lines.push(`### ⚠️ Pytxo could not run the check (exit ${code})`, "", "```", tail || "no output", "```");
    }
  }
  if (notes.length) lines.push("", ...notes.map((n) => `> ${n}`));
  lines.push("", INSTALL_HINT);
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

// Pytxo Pro for private repositories in CI. Built and switched off: nothing
// is enforced, and no key is checked, until LICENSING is true and
// LICENSE_PUBLIC_KEY holds the public half of the signing key.
export const LICENSING = false;
export const TRIAL_DAYS = 14;
export const LICENSE_PUBLIC_KEY = null;
const BUY = "https://pytxo.com/plans";

/**
 * A key is `pytxo1.<payload>.<signature>`, both base64url; the payload is
 * JSON `{ "repo": "owner/name" or "owner/*", "exp": "2027-10-01" or null }`
 * signed with Ed25519. Verified offline: no account, no network.
 */
export function verifyLicense(key, repo, now, publicKey = LICENSE_PUBLIC_KEY) {
  const [version, payload, signature] = (key || "").trim().split(".");
  if (version !== "pytxo1" || !payload || !signature || !publicKey) return { valid: false, reason: "not a Pytxo license key" };
  const signed = crypto.verify(null, Buffer.from(payload), publicKey, Buffer.from(signature, "base64url"));
  if (!signed) return { valid: false, reason: "the license key's signature does not match" };
  const claims = JSON.parse(Buffer.from(payload, "base64url").toString("utf8"));
  const [owner] = repo.split("/");
  if (claims.repo !== repo && claims.repo !== `${owner}/*`) return { valid: false, reason: `the license key is for ${claims.repo}` };
  if (claims.exp && new Date(claims.exp) < now) return { valid: false, reason: `the license key expired on ${claims.exp}` };
  return { valid: true };
}

/** Public repositories are free. Private ones get TRIAL_DAYS, then need a key. */
export function licenseGate({ isPrivate, repo, key, trialStart, now, publicKey = LICENSE_PUBLIC_KEY }) {
  if (!isPrivate) return { allowed: true };
  const license = verifyLicense(key, repo, now, publicKey);
  if (license.valid) return { allowed: true };
  const left = Math.ceil(TRIAL_DAYS - (now - trialStart) / 86_400_000);
  if (left > 0) {
    return { allowed: true, notice: `Pytxo Pro trial for private repositories: ${left} day${left === 1 ? "" : "s"} left. ${BUY}` };
  }
  const why = key ? ` (${license.reason})` : "";
  return {
    allowed: false,
    notice: `The ${TRIAL_DAYS}-day trial for private repositories is over${why}. Add a Pytxo Pro license key as the PYTXO_LICENSE secret and pass it as \`license\`: ${BUY}`,
  };
}

/** When the workflow file was first committed: the trial's start, with no API call. */
// ponytail: renaming the workflow file restarts the trial; a server-side record if that matters.
function workflowAddedAt() {
  const file = (process.env.GITHUB_WORKFLOW_REF || "").split("@")[0].split("/").slice(2).join("/");
  const dates = file ? spawnSync("git", ["log", "--diff-filter=A", "--format=%cI", "--", file], { encoding: "utf8" }).stdout : "";
  const first = dates.trim().split(/\r?\n/).filter(Boolean).at(-1);
  return first ? new Date(first) : new Date();
}

async function discover() {
  const event = process.env.GITHUB_EVENT_NAME;
  if (event === "pull_request_target") {
    throw new Error(
      "refusing to run on pull_request_target: the check runs pull-request code, and pull_request_target gives it your secrets. Use `on: pull_request`.",
    );
  }
  // A push or a merge queue passes: failing it would hold up the queue.
  if (event !== "pull_request") {
    console.log(`::notice::Pytxo checks pull requests; nothing to do on ${event}.`);
    output("skip", "true");
    return;
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
  if (LICENSING && payload.repository?.private) {
    const gate = licenseGate({
      isPrivate: true,
      repo,
      key: process.env.PYTXO_LICENSE,
      trialStart: workflowAddedAt(),
      now: new Date(),
    });
    if (!gate.allowed) throw new Error(gate.notice);
    if (gate.notice) console.log(`::notice::${gate.notice}`);
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
    notes: state.notes,
  });
  fs.appendFileSync(process.env.GITHUB_STEP_SUMMARY, body);
  if (process.env.PYTXO_COMMENT === "false") return;
  const result = await upsertComment(fetch, {
    apiUrl: process.env.GITHUB_API_URL || "https://api.github.com",
    repo: process.env.GITHUB_REPOSITORY,
    number: state.number,
    token: process.env.PYTXO_TOKEN,
    body,
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
